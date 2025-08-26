use crate::domain::index::GitIndex;
use crate::domain::objects::{
    CommitObject, GitObject, ObjectHash, Signature, TreeEntry, TreeObject,
};
use crate::domain::references::GitRef;
use crate::infrastructure::index_store::IndexStore;
use crate::infrastructure::object_store::ObjectStore;
use crate::infrastructure::ref_store::RefStore;
use std::path::Path;

/// Options for commit command
#[derive(Debug, Default)]
pub struct CommitOptions {
    /// Allow empty commits
    pub allow_empty: bool,
    /// Author name (if different from committer)
    pub author_name: Option<String>,
    /// Author email (if different from committer)  
    pub author_email: Option<String>,
}

/// Result of a commit operation
#[derive(Debug)]
pub struct CommitResult {
    pub commit_hash: ObjectHash,
    pub tree_hash: ObjectHash,
    pub message: String,
    pub files_committed: usize,
    pub is_root_commit: bool,
}

impl CommitResult {
    pub fn summary(&self) -> String {
        if self.is_root_commit {
            format!(
                "🎉 Root commit created: {} ({} files)",
                &self.commit_hash.as_str()[..8],
                self.files_committed
            )
        } else {
            format!(
                "✅ Commit created: {} ({} files)",
                &self.commit_hash.as_str()[..8],
                self.files_committed
            )
        }
    }
}

/// Commit command implementation
pub struct CommitCommand;

impl CommitCommand {
    /// Create a new commit from the staged changes
    pub fn commit(
        repo_path: &Path,
        message: &str,
        options: CommitOptions,
    ) -> crate::Result<CommitResult> {
        let git_dir = repo_path.join(".git-rs");

        // Initialize stores
        let object_store = ObjectStore::new(git_dir.clone());
        let index_store = IndexStore::new(git_dir.join("git-rs-index"));
        let ref_store = RefStore::new(git_dir.clone());

        // Load the current index
        let index = index_store.load_index()?;

        // Check if there are staged changes
        if index.entries.is_empty() && !options.allow_empty {
            return Err(
                "No changes added to commit. Use 'git-rs add' to stage files first.".into(),
            );
        }

        // Create tree object from index
        let tree_hash = Self::create_tree_from_index(&object_store, &index)?;

        // Get current HEAD to determine parent
        let current_head = ref_store.get_head()?;
        let parent_commit = match current_head {
            Some(head) => {
                // Check if tree has actually changed
                if let Ok(parent_commit_obj) = object_store.load_object(&head) {
                    if let GitObject::Commit(parent_commit) = parent_commit_obj {
                        if parent_commit.tree == tree_hash && !options.allow_empty {
                            return Err("No changes to commit. Working tree is clean.".into());
                        }
                        Some(head)
                    } else {
                        return Err("HEAD does not point to a valid commit".into());
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        // Create signature for author and committer
        let (author, _committer) = Self::create_signatures(&options)?;

        // Create commit object
        let parents = if let Some(parent) = parent_commit {
            vec![parent]
        } else {
            vec![]
        };

        let commit_obj = CommitObject::new(tree_hash.clone(), parents, author, message.to_string());

        let is_root_commit = commit_obj.is_root_commit();

        // Store commit object
        let commit_hash = object_store.store_object(&GitObject::Commit(commit_obj))?;

        // Update HEAD reference
        let current_branch = ref_store.get_current_branch()?;
        let branch_name = current_branch.unwrap_or_else(|| "main".to_string());

        let branch_ref = GitRef::branch(branch_name, commit_hash.clone());
        ref_store.store_ref(&branch_ref)?;

        Ok(CommitResult {
            commit_hash,
            tree_hash,
            message: message.to_string(),
            files_committed: index.entries.len(),
            is_root_commit,
        })
    }

    /// Create tree object from index entries
    fn create_tree_from_index(
        object_store: &ObjectStore,
        index: &GitIndex,
    ) -> crate::Result<ObjectHash> {
        let mut entries = Vec::new();

        for (path, entry) in &index.entries {
            let tree_entry = TreeEntry {
                mode: entry.mode,
                name: path.to_string_lossy().to_string(),
                hash: entry.hash.clone(),
            };
            entries.push(tree_entry);
        }

        // Sort entries by name (Git requirement)
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        let tree_obj = TreeObject { entries };
        object_store.store_object(&GitObject::Tree(tree_obj))
    }

    /// Create author and committer signatures
    fn create_signatures(options: &CommitOptions) -> crate::Result<(Signature, Signature)> {
        // Try to get from git config first, then fall back to defaults
        let (default_name, default_email) = Self::get_git_config()?;

        let author_name = options.author_name.as_deref().unwrap_or(&default_name);
        let author_email = options.author_email.as_deref().unwrap_or(&default_email);

        let author = Signature::new(author_name.to_string(), author_email.to_string());
        let committer = Signature::new(default_name, default_email);

        Ok((author, committer))
    }

    /// Get git configuration for user name and email
    fn get_git_config() -> crate::Result<(String, String)> {
        // For now, use environment variables or defaults
        // In a full implementation, this would read from .git/config
        let name = std::env::var("GIT_AUTHOR_NAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "Git User".to_string());

        let email = std::env::var("GIT_AUTHOR_EMAIL")
            .unwrap_or_else(|_| format!("{}@example.com", name.to_lowercase().replace(' ', ".")));

        Ok((name, email))
    }

    /// Validate commit message
    pub fn validate_message(message: &str) -> crate::Result<()> {
        if message.trim().is_empty() {
            return Err("Commit message cannot be empty".into());
        }

        if message.len() > 72 {
            println!("⚠️  Warning: Commit message is longer than 72 characters");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::add::{AddCommand, AddOptions};
    use crate::application::init::InitCommand;
    use tempfile::TempDir;

    #[test]
    fn test_commit_validation() {
        assert!(CommitCommand::validate_message("Valid message").is_ok());
        assert!(CommitCommand::validate_message("").is_err());
        assert!(CommitCommand::validate_message("   \n  ").is_err());
    }

    #[test]
    fn test_commit_flow() -> crate::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository
        InitCommand::init(Some(repo_path))?;

        // Create a test file
        let test_file = repo_path.join("test.txt");
        std::fs::write(&test_file, "Hello, World!")?;

        // Add the file
        let add_options = AddOptions::default();
        AddCommand::add(repo_path, &["test.txt".to_string()], add_options)?;

        // Commit the file
        let commit_options = CommitOptions::default();
        let result = CommitCommand::commit(repo_path, "Initial commit", commit_options)?;

        assert!(result.is_root_commit);
        assert_eq!(result.files_committed, 1);
        assert_eq!(result.message, "Initial commit");

        Ok(())
    }
}
