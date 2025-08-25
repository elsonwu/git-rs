use crate::domain::{index::*, objects::*, references::*};
use std::path::{Path, PathBuf};

/// Repository configuration
#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    pub user_name: String,
    pub user_email: String,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            user_name: "Git User".to_string(),
            user_email: "user@example.com".to_string(),
        }
    }
}

/// Git Repository - The main aggregate root in our domain
///
/// This represents a Git repository and encapsulates all the core Git functionality.
/// Following DDD principles, this is the main entry point for all repository operations.
#[derive(Debug, Clone)]
pub struct GitRepository {
    /// Path to the repository root (contains .git directory)
    pub root_path: PathBuf,
    /// Path to the .git directory
    pub git_dir: PathBuf,
    /// Repository configuration
    pub config: RepositoryConfig,
    /// Reference manager
    pub refs: ReferenceManager,
    /// Current index (staging area)
    pub index: GitIndex,
}

impl GitRepository {
    /// Create a new repository instance
    ///
    /// # Arguments
    /// * `root_path` - Path to the repository root directory
    ///
    /// # Example
    /// ```
    /// use git_rs::domain::GitRepository;
    /// use std::path::PathBuf;
    ///
    /// let repo = GitRepository::new(PathBuf::from("."));
    /// ```
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        let git_dir = root_path.join(".git-rs");

        Self {
            root_path,
            git_dir,
            config: RepositoryConfig::default(),
            refs: ReferenceManager::new(),
            index: GitIndex::new(),
        }
    }

    /// Check if this directory contains a Git repository
    pub fn is_repository(&self) -> bool {
        self.git_dir.exists() && self.git_dir.is_dir()
    }

    /// Get the repository root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get the .git directory path
    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    /// Get the objects directory path
    pub fn objects_dir(&self) -> PathBuf {
        self.git_dir.join("objects")
    }

    /// Get the refs directory path
    pub fn refs_dir(&self) -> PathBuf {
        self.git_dir.join("refs")
    }

    /// Get the heads directory path (branches)
    pub fn heads_dir(&self) -> PathBuf {
        self.refs_dir().join("heads")
    }

    /// Get the tags directory path
    pub fn tags_dir(&self) -> PathBuf {
        self.refs_dir().join("tags")
    }

    /// Get the index file path (using git-rs-index to avoid conflicts with Git's index)
    pub fn index_path(&self) -> PathBuf {
        self.git_dir.join("git-rs-index")
    }

    /// Get the HEAD file path
    pub fn head_path(&self) -> PathBuf {
        self.git_dir.join("HEAD")
    }

    /// Get the config file path
    pub fn config_path(&self) -> PathBuf {
        self.git_dir.join("config")
    }

    /// Get path to an object file given its hash
    pub fn object_path(&self, hash: &ObjectHash) -> PathBuf {
        self.objects_dir()
            .join(hash.dir_name())
            .join(hash.file_name())
    }

    /// Get path to a reference file
    pub fn ref_path(&self, ref_name: &str) -> PathBuf {
        if ref_name.starts_with("refs/") {
            self.git_dir.join(ref_name)
        } else {
            self.git_dir.join("refs").join("heads").join(ref_name)
        }
    }

    /// Convert an absolute path to a path relative to the repository root
    pub fn to_relative_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, std::io::Error> {
        let path = path.as_ref();
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        absolute_path
            .strip_prefix(&self.root_path)
            .map(|p| p.to_path_buf())
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path is not within repository",
                )
            })
    }

    /// Convert a relative path to an absolute path within the repository
    pub fn to_absolute_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root_path.join(path)
        }
    }

    /// Check if a file should be ignored
    /// For now, this is a simple implementation that ignores .git directory,
    /// common temporary files, and patterns from .gitignore.
    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        // Convert to string for easier pattern matching
        let path_str = path.to_string_lossy();

        // Always ignore .git directory
        if path_str.contains(".git") {
            return true;
        }

        // Ignore common temporary files
        if path_str.ends_with('~') || path_str.ends_with(".tmp") || path_str.ends_with(".swp") {
            return true;
        }

        // Check .gitignore patterns
        if let Ok(gitignore_content) = std::fs::read_to_string(self.root_path.join(".gitignore")) {
            for line in gitignore_content.lines() {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Simple pattern matching
                if let Some(pattern) = line.strip_suffix('/') {
                    // Directory pattern
                    if path_str.starts_with(pattern) || path_str.contains(&format!("/{}/", pattern))
                    {
                        return true;
                    }
                } else {
                    // File or glob pattern
                    if path_str.contains(line) || path_str.ends_with(line) {
                        return true;
                    }

                    // Handle simple wildcards
                    if let Some(extension) = line.strip_prefix("*.") {
                        if path_str.ends_with(&format!(".{}", extension)) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Option<String> {
        self.refs
            .get_head()
            .and_then(|head| head.branch_name())
            .map(|name| name.to_string())
    }

    /// Get current commit hash
    pub fn current_commit(&self) -> Option<ObjectHash> {
        self.refs.resolve_head()
    }

    /// Update repository configuration
    pub fn set_config(&mut self, config: RepositoryConfig) {
        self.config = config;
    }

    /// Create a signature using repository configuration
    pub fn create_signature(&self) -> Signature {
        Signature::new(
            self.config.user_name.clone(),
            self.config.user_email.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_repository() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        let repo = GitRepository::new(repo_path);

        assert_eq!(repo.root_path(), repo_path);
        assert_eq!(repo.git_dir(), repo_path.join(".git-rs"));
        assert!(!repo.is_repository()); // No .git directory created yet
    }

    #[test]
    fn test_repository_paths() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepository::new(repo_path);

        assert_eq!(repo.objects_dir(), repo_path.join(".git/objects"));
        assert_eq!(repo.refs_dir(), repo_path.join(".git/refs"));
        assert_eq!(repo.heads_dir(), repo_path.join(".git/refs/heads"));
        assert_eq!(repo.index_path(), repo_path.join(".git/git-rs-index"));
        assert_eq!(repo.head_path(), repo_path.join(".git/HEAD"));

        let hash = ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string());
        let expected_path =
            repo_path.join(".git/objects/12/34567890abcdef1234567890abcdef12345678");
        assert_eq!(repo.object_path(&hash), expected_path);
    }

    #[test]
    fn test_path_conversion() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = GitRepository::new(repo_path);

        // Test relative to absolute
        let relative_path = PathBuf::from("src/main.rs");
        let absolute_path = repo.to_absolute_path(&relative_path);
        assert_eq!(absolute_path, repo_path.join("src/main.rs"));

        // Test absolute to relative (would need actual file system setup for full test)
        let test_path = repo_path.join("test.txt");
        if let Ok(relative) = repo.to_relative_path(&test_path) {
            assert_eq!(relative, PathBuf::from("test.txt"));
        }
    }

    #[test]
    fn test_ignored_files() {
        let temp_dir = tempdir().unwrap();
        let repo = GitRepository::new(temp_dir.path());

        assert!(repo.is_ignored(".git/objects"));
        assert!(repo.is_ignored("file.tmp"));
        assert!(repo.is_ignored("backup~"));
        assert!(repo.is_ignored(".file.swp"));
        assert!(!repo.is_ignored("src/main.rs"));
        assert!(!repo.is_ignored("README.md"));
    }
}
