use std::path::Path;

use crate::domain::{objects::*, references::*};
use crate::infrastructure::{object_store::ObjectStore, ref_store::RefStore};

/// Git Log Use Case
///
/// This implements the `git log` command functionality.
///
/// ## What `git log` does:
/// 1. Resolves HEAD to get the current commit
/// 2. Walks the commit history following parent links
/// 3. Displays commit information in reverse chronological order
/// 4. Supports limiting the number of commits shown
///
/// ## Educational Insights:
/// - **Commit Graph Traversal**: How Git walks the commit history
/// - **Parent Relationships**: Understanding commit parent-child links
/// - **Object Resolution**: How references resolve to actual commit objects
/// - **History Representation**: Converting the commit DAG to linear history
///
/// ## Visual Guide - Commit History Walking:
/// ```text
/// HEAD -> main -> commit_c (latest)
///                     ↓ parent
///                 commit_b
///                     ↓ parent  
///                 commit_a (initial)
///                     ↓ parent
///                   (none)
/// ```
pub struct LogCommand;

/// Options for the log command
#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    /// Maximum number of commits to show (None = all)
    pub max_count: Option<usize>,
}

/// Result of log command containing commit information
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub hash: ObjectHash,
    pub commit: CommitObject,
}

/// Complete log result
#[derive(Debug, Clone)]
pub struct LogResult {
    pub entries: Vec<LogEntry>,
    pub total_commits: usize,
    pub has_more: bool,
}

impl LogResult {
    pub fn new(entries: Vec<LogEntry>, total_commits: usize, has_more: bool) -> Self {
        Self {
            entries,
            total_commits,
            has_more,
        }
    }
}

impl LogCommand {
    /// Show commit history starting from HEAD
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository root
    /// * `options` - Log command options
    ///
    /// # Returns
    /// * `Ok(LogResult)` - The commit history
    /// * `Err(...)` - If log failed
    pub fn log<P: AsRef<Path>>(repo_path: P, options: LogOptions) -> crate::Result<LogResult> {
        let repo_path = repo_path.as_ref();
        let git_dir = repo_path.join(".git-rs");

        if !git_dir.exists() {
            return Err("Not a git repository (or any of the parent directories): .git-rs".into());
        }

        println!("🔍 Loading commit history...");

        let ref_store = RefStore::new(git_dir.clone());
        let object_store = ObjectStore::new(git_dir.join("objects"));

        // Get HEAD reference
        let head = ref_store.load_head()?;
        let head = match head {
            Some(h) => h,
            None => {
                println!("📭 No commits found (empty repository)");
                return Ok(LogResult::new(vec![], 0, false));
            }
        };

        // Resolve HEAD to get starting commit hash
        let starting_commit_hash = Self::resolve_head_to_commit(&head, &ref_store)?;

        // Walk the commit history
        let mut entries = Vec::new();
        let mut current_hash = Some(starting_commit_hash);
        let mut count = 0;

        while let Some(hash) = current_hash.as_ref() {
            // Check if we've hit our limit
            if let Some(max) = options.max_count {
                if count >= max {
                    break;
                }
            }

            // Load the commit object
            let object = object_store.load_object(hash)?;
            let commit = match object.as_commit() {
                Some(c) => c.clone(),
                None => {
                    return Err(format!("Object {} is not a commit", hash).into());
                }
            };

            // Add to results
            entries.push(LogEntry {
                hash: hash.clone(),
                commit: commit.clone(),
            });

            // Move to parent commit
            current_hash = commit.parents.first().cloned();
            count += 1;
        }

        let has_more = current_hash.is_some();
        let total_entries = entries.len();

        println!("📊 Found {} commit(s)", total_entries);

        Ok(LogResult::new(entries, total_entries, has_more))
    }

    /// Show commit history with git compatibility mode
    ///
    /// # Arguments  
    /// * `repo_path` - Path to the repository root
    /// * `options` - Log command options
    /// * `git_compat` - Git compatibility mode
    ///
    /// # Returns
    /// * `Ok(LogResult)` - The commit history
    /// * `Err(...)` - If log failed
    pub fn log_with_compat<P: AsRef<Path>>(
        repo_path: P,
        options: LogOptions,
        git_compat: crate::domain::repository::GitCompatMode,
    ) -> crate::Result<LogResult> {
        let repo_path = repo_path.as_ref();
        let git_dir = match git_compat {
            crate::domain::repository::GitCompatMode::Educational => repo_path.join(".git-rs"),
            crate::domain::repository::GitCompatMode::Compatible => repo_path.join(".git"),
        };

        if !git_dir.exists() {
            let dir_name = match git_compat {
                crate::domain::repository::GitCompatMode::Educational => ".git-rs",
                crate::domain::repository::GitCompatMode::Compatible => ".git",
            };
            return Err(format!(
                "Not a git repository (or any of the parent directories): {}",
                dir_name
            )
            .into());
        }

        println!("🔍 Loading commit history...");

        let ref_store = RefStore::new(git_dir.clone());
        let object_store = ObjectStore::new(git_dir.join("objects"));

        // Get HEAD reference
        let head = ref_store.load_head()?;
        let head = match head {
            Some(h) => h,
            None => {
                println!("📭 No commits found (empty repository)");
                return Ok(LogResult::new(vec![], 0, false));
            }
        };

        // Resolve HEAD to get starting commit hash
        let starting_commit_hash = Self::resolve_head_to_commit(&head, &ref_store)?;

        // Walk the commit history
        let mut entries = Vec::new();
        let mut current_hash = Some(starting_commit_hash);
        let mut count = 0;

        while let Some(hash) = current_hash.as_ref() {
            // Check if we've hit our limit
            if let Some(max) = options.max_count {
                if count >= max {
                    break;
                }
            }

            // Load the commit object
            let object = object_store.load_object(hash)?;
            let commit = match object.as_commit() {
                Some(c) => c.clone(),
                None => {
                    return Err(format!("Object {} is not a commit", hash).into());
                }
            };

            // Add to results
            entries.push(LogEntry {
                hash: hash.clone(),
                commit: commit.clone(),
            });

            // Move to parent commit
            current_hash = commit.parents.first().cloned();
            count += 1;
        }

        let has_more = current_hash.is_some();
        let total_entries = entries.len();

        println!("📊 Found {} commit(s)", total_entries);

        Ok(LogResult::new(entries, total_entries, has_more))
    }

    /// Resolve HEAD reference to actual commit hash
    fn resolve_head_to_commit(head: &HeadRef, ref_store: &RefStore) -> crate::Result<ObjectHash> {
        match head {
            HeadRef::Direct(hash) => Ok(hash.clone()),
            HeadRef::Symbolic(ref_name) => {
                // Load all refs to find the branch
                let refs = ref_store.load_refs()?;

                // Extract branch name from refs/heads/branch_name
                // Handle cases where ref_name might be malformed like refs/heads/refs/heads/main
                let branch_name = if ref_name.starts_with("refs/heads/refs/heads/") {
                    ref_name.strip_prefix("refs/heads/refs/heads/").unwrap()
                } else if ref_name.starts_with("refs/heads/") {
                    ref_name.strip_prefix("refs/heads/").unwrap()
                } else {
                    return Err(format!("Invalid symbolic ref: {}", ref_name).into());
                };

                // Find the branch reference
                let branches = refs.branches();
                let branch_ref = branches
                    .iter()
                    .find(|r| r.name == branch_name)
                    .ok_or_else(|| format!("Branch '{}' not found", branch_name))?;

                Ok(branch_ref.hash.clone())
            }
        }
    }

    /// Format a log entry for display
    pub fn format_log_entry(entry: &LogEntry) -> String {
        let short_hash = &entry.hash.as_str()[..7];
        let message_lines: Vec<&str> = entry.commit.message.lines().collect();
        let first_line = message_lines.first().unwrap_or(&"");

        format!(
            "commit {} ({})\nAuthor: {} <{}>\nDate: {}\n\n    {}\n",
            entry.hash,
            short_hash,
            entry.commit.author.name,
            entry.commit.author.email,
            entry
                .commit
                .author
                .timestamp
                .format("%a %b %d %H:%M:%S %Y %z"),
            first_line
        )
    }

    /// Format the complete log result for display
    pub fn format_log_result(result: &LogResult) -> String {
        let mut output = String::new();

        for entry in &result.entries {
            output.push_str(&Self::format_log_entry(entry));
            output.push('\n');
        }

        if result.has_more {
            output.push_str("... (more commits available)\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_options_default() {
        let options = LogOptions::default();
        assert_eq!(options.max_count, None);
    }

    #[test]
    fn test_log_entry_formatting() {
        let hash = ObjectHash::new("a1b2c3d4e5f6789012345678901234567890abcd".to_string());
        let author = Signature::new("Test Author".to_string(), "test@example.com".to_string());
        let commit = CommitObject::new(
            hash.clone(),
            vec![],
            author,
            "Initial commit\n\nThis is the first commit in the repository.".to_string(),
        );

        let entry = LogEntry {
            hash: hash.clone(),
            commit,
        };

        let formatted = LogCommand::format_log_entry(&entry);
        assert!(formatted.contains("commit a1b2c3d4e5f6789012345678901234567890abcd"));
        assert!(formatted.contains("(a1b2c3d)"));
        assert!(formatted.contains("Author: Test Author <test@example.com>"));
        assert!(formatted.contains("Initial commit"));
    }

    #[test]
    fn test_log_empty_repository() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Try to log in a directory that doesn't have .git-rs
        let result = LogCommand::log(repo_path, LogOptions::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a git repository"));
    }

    #[test]
    fn test_log_result_creation() {
        let entries = vec![];
        let result = LogResult::new(entries, 0, false);

        assert_eq!(result.entries.len(), 0);
        assert_eq!(result.total_commits, 0);
        assert!(!result.has_more);
    }
}
