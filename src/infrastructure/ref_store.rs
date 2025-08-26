use std::fs;
use std::path::PathBuf;

use crate::domain::objects::ObjectHash;
use crate::domain::references::*;

/// Reference Storage Implementation
///
/// This handles the physical storage and retrieval of Git references.
/// References are stored as files in the .git/refs/ directory:
/// - .git/refs/heads/* for branches
/// - .git/refs/tags/* for tags  
/// - .git/HEAD for the current branch/commit
pub struct RefStore {
    refs_dir: PathBuf,
    git_dir: PathBuf,
}

impl RefStore {
    /// Create a new reference store
    pub fn new(git_dir: PathBuf) -> Self {
        let refs_dir = git_dir.join("refs");
        Self { refs_dir, git_dir }
    }

    /// Initialize the refs directory structure
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.refs_dir)?;
        fs::create_dir_all(self.refs_dir.join("heads"))?;
        fs::create_dir_all(self.refs_dir.join("tags"))?;
        Ok(())
    }

    /// Load all references from the file system
    pub fn load_refs(&self) -> crate::Result<ReferenceManager> {
        let mut ref_manager = ReferenceManager::new();

        // Load branch references
        let heads_dir = self.refs_dir.join("heads");
        if heads_dir.exists() {
            self.load_refs_from_dir(&heads_dir, &mut ref_manager, RefType::Branch)?;
        }

        // Load tag references
        let tags_dir = self.refs_dir.join("tags");
        if tags_dir.exists() {
            self.load_refs_from_dir(&tags_dir, &mut ref_manager, RefType::Tag)?;
        }

        // Load HEAD reference
        ref_manager.head = self.load_head()?;

        Ok(ref_manager)
    }

    /// Save all references to the file system
    pub fn save_refs(&self, ref_manager: &ReferenceManager) -> crate::Result<()> {
        // Save all refs
        for git_ref in &ref_manager.refs {
            self.save_ref(git_ref)?;
        }

        // Save HEAD
        if let Some(head) = &ref_manager.head {
            self.save_head(head)?;
        }

        Ok(())
    }

    /// Save a single reference
    pub fn save_ref(&self, git_ref: &GitRef) -> crate::Result<()> {
        let ref_path = self.get_ref_path(git_ref);

        // Create parent directory if it doesn't exist
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write the hash to the reference file
        fs::write(&ref_path, format!("{}\n", git_ref.hash))?;

        Ok(())
    }

    /// Load a single reference
    pub fn load_ref(&self, ref_name: &str, ref_type: RefType) -> crate::Result<Option<GitRef>> {
        let ref_path = match ref_type {
            RefType::Branch => self.refs_dir.join("heads").join(ref_name),
            RefType::Tag => self.refs_dir.join("tags").join(ref_name),
            RefType::RemoteBranch => self.refs_dir.join("remotes").join(ref_name),
        };

        if !ref_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&ref_path)?;
        let hash = ObjectHash::new(content.trim().to_string());

        Ok(Some(GitRef::new(ref_name.to_string(), hash, ref_type)))
    }

    /// Delete a reference
    pub fn delete_ref(&self, git_ref: &GitRef) -> crate::Result<()> {
        let ref_path = self.get_ref_path(git_ref);

        if ref_path.exists() {
            fs::remove_file(&ref_path)?;
        }

        // Try to remove empty parent directories
        if let Some(parent) = ref_path.parent() {
            let _ = fs::remove_dir(parent); // Ignore errors - directory might not be empty
        }

        Ok(())
    }

    /// Save HEAD reference
    pub fn save_head(&self, head: &HeadRef) -> crate::Result<()> {
        let head_path = self.git_dir.join("HEAD");
        let content = format!("{}\n", head);
        fs::write(&head_path, content)?;
        Ok(())
    }

    /// Load HEAD reference
    pub fn load_head(&self) -> crate::Result<Option<HeadRef>> {
        let head_path = self.git_dir.join("HEAD");

        if !head_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&head_path)?;
        let content = content.trim();

        if content.starts_with("ref: ") {
            // Symbolic reference
            let ref_name = content.strip_prefix("ref: ").unwrap();
            Ok(Some(HeadRef::Symbolic(ref_name.to_string())))
        } else {
            // Direct reference to commit hash
            Ok(Some(HeadRef::Direct(ObjectHash::new(content.to_string()))))
        }
    }

    /// Update HEAD to point to a branch
    pub fn set_head_to_branch(&self, branch_name: &str) -> crate::Result<()> {
        let head = HeadRef::symbolic(branch_name);
        self.save_head(&head)
    }

    /// Update HEAD to point directly to a commit (detached HEAD)
    pub fn set_head_to_commit(&self, hash: ObjectHash) -> crate::Result<()> {
        let head = HeadRef::direct(hash);
        self.save_head(&head)
    }

    /// Create or update a branch reference
    pub fn create_branch(&self, name: &str, hash: ObjectHash) -> crate::Result<GitRef> {
        let git_ref = GitRef::branch(name.to_string(), hash);
        self.save_ref(&git_ref)?;
        Ok(git_ref)
    }

    /// Create or update a tag reference
    pub fn create_tag(&self, name: &str, hash: ObjectHash) -> crate::Result<GitRef> {
        let git_ref = GitRef::tag(name.to_string(), hash);
        self.save_ref(&git_ref)?;
        Ok(git_ref)
    }

    /// List all branch names
    pub fn list_branches(&self) -> crate::Result<Vec<String>> {
        let heads_dir = self.refs_dir.join("heads");
        if !heads_dir.exists() {
            return Ok(Vec::new());
        }

        let mut branches = Vec::new();
        self.collect_ref_names(&heads_dir, &mut branches, String::new())?;
        Ok(branches)
    }

    /// List all tag names
    pub fn list_tags(&self) -> crate::Result<Vec<String>> {
        let tags_dir = self.refs_dir.join("tags");
        if !tags_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tags = Vec::new();
        self.collect_ref_names(&tags_dir, &mut tags, String::new())?;
        Ok(tags)
    }

    /// Get the file system path for a reference
    fn get_ref_path(&self, git_ref: &GitRef) -> PathBuf {
        match git_ref.ref_type {
            RefType::Branch => self.refs_dir.join("heads").join(&git_ref.name),
            RefType::Tag => self.refs_dir.join("tags").join(&git_ref.name),
            RefType::RemoteBranch => self.refs_dir.join("remotes").join(&git_ref.name),
        }
    }

    /// Load references from a directory recursively
    fn load_refs_from_dir(
        &self,
        dir: &PathBuf,
        ref_manager: &mut ReferenceManager,
        ref_type: RefType,
    ) -> crate::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // Read the hash from the file
                let content = fs::read_to_string(&path)?;
                let hash = ObjectHash::new(content.trim().to_string());

                // Get the reference name (relative to refs/heads or refs/tags)
                let ref_name = path
                    .strip_prefix(&self.refs_dir)
                    .map_err(|_| "Invalid ref path")?
                    .strip_prefix(match ref_type {
                        RefType::Branch => "heads",
                        RefType::Tag => "tags",
                        RefType::RemoteBranch => "remotes",
                    })
                    .map_err(|_| "Invalid ref path")?
                    .to_string_lossy()
                    .to_string();

                let git_ref = GitRef::new(ref_name, hash, ref_type);
                ref_manager.add_ref(git_ref);
            } else if path.is_dir() {
                // Recursively load subdirectories
                self.load_refs_from_dir(&path, ref_manager, ref_type)?;
            }
        }

        Ok(())
    }

    /// Recursively collect reference names from a directory
    #[allow(clippy::only_used_in_recursion)]
    fn collect_ref_names(
        &self,
        dir: &PathBuf,
        names: &mut Vec<String>,
        prefix: String,
    ) -> crate::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            let full_name = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_file() {
                names.push(full_name);
            } else if path.is_dir() {
                self.collect_ref_names(&path, names, full_name)?;
            }
        }

        Ok(())
    }

    /// Get the current HEAD commit hash (resolving symbolic references)
    pub fn get_head(&self) -> crate::Result<Option<ObjectHash>> {
        let head = self.load_head()?;
        match head {
            Some(HeadRef::Direct(hash)) => Ok(Some(hash)),
            Some(HeadRef::Symbolic(ref_name)) => {
                // Extract branch name and load its hash
                if let Some(branch_name) = ref_name.strip_prefix("refs/heads/") {
                    if let Some(branch_ref) = self.load_ref(branch_name, RefType::Branch)? {
                        Ok(Some(branch_ref.hash))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get the current branch name (if HEAD is symbolic)
    pub fn get_current_branch(&self) -> crate::Result<Option<String>> {
        let head = self.load_head()?;
        match head {
            Some(HeadRef::Symbolic(ref_name)) => {
                Ok(ref_name.strip_prefix("refs/heads/").map(|s| s.to_string()))
            }
            _ => Ok(None),
        }
    }

    /// Store a single reference (convenience method)
    pub fn store_ref(&self, git_ref: &GitRef) -> crate::Result<()> {
        self.save_ref(git_ref)?;
        
        // If this is the current branch, update HEAD to point to it
        if let Some(current_branch) = self.get_current_branch()? {
            if current_branch == git_ref.name && git_ref.ref_type == RefType::Branch {
                let head = HeadRef::symbolic(&format!("refs/heads/{}", git_ref.name));
                self.save_head(&head)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_branch() {
        let temp_dir = tempdir().unwrap();
        let store = RefStore::new(temp_dir.path().to_path_buf());
        store.init().unwrap();

        let hash = ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string());
        let branch = store.create_branch("main", hash.clone()).unwrap();

        let loaded = store.load_ref("main", RefType::Branch).unwrap().unwrap();
        assert_eq!(branch, loaded);
        assert_eq!(loaded.hash, hash);
    }

    #[test]
    fn test_save_and_load_head() {
        let temp_dir = tempdir().unwrap();
        let store = RefStore::new(temp_dir.path().to_path_buf());
        store.init().unwrap();

        // Test symbolic HEAD
        let head = HeadRef::symbolic("main");
        store.save_head(&head).unwrap();
        let loaded_head = store.load_head().unwrap().unwrap();
        assert_eq!(head, loaded_head);

        // Test direct HEAD
        let hash = ObjectHash::new("abcdef1234567890abcdef1234567890abcdef12".to_string());
        let direct_head = HeadRef::direct(hash.clone());
        store.save_head(&direct_head).unwrap();
        let loaded_direct = store.load_head().unwrap().unwrap();
        assert_eq!(direct_head, loaded_direct);
    }

    #[test]
    fn test_list_branches() {
        let temp_dir = tempdir().unwrap();
        let store = RefStore::new(temp_dir.path().to_path_buf());
        store.init().unwrap();

        let hash = ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string());
        store.create_branch("main", hash.clone()).unwrap();
        store.create_branch("develop", hash).unwrap();

        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 2);
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"develop".to_string()));
    }
}
