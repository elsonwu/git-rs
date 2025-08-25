use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::*;
use crate::infrastructure::*;

/// Git Add Use Case
///
/// This implements the `git add` command functionality.
///
/// ## What `git add` does:
/// 1. Reads file content from working directory
/// 2. Creates blob objects in the object database
/// 3. Updates the index (staging area) with file metadata
/// 4. Prepares files for the next commit
///
/// ## Visual Guide - Git Add Process:
/// ```text
/// Working Directory       Index (Staging)        Object Database
/// ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
/// │ file.txt        │    │                 │    │                 │
/// │ "Hello World"   │    │                 │    │                 │
/// │                 │    │                 │    │                 │
/// │ [git add file]  │───▶│ file.txt        │───▶│ objects/ab/     │
/// │                 │    │ - hash: abc123  │    │   cdef123...    │
/// │                 │    │ - size: 11      │    │ blob 11\0       │
/// │                 │    │ - mode: 100644  │    │ Hello World     │
/// │                 │    │ - staged ✓      │    │                 │
/// └─────────────────┘    └─────────────────┘    └─────────────────┘
/// ```
///
/// ## Index Entry Structure:
/// Each staged file creates an IndexEntry containing:
/// - File path relative to repository root
/// - Object hash (SHA-1 of content)  
/// - File size and permissions
/// - Timestamps and metadata
/// - Stage number (0 = normal, 1-3 = merge conflicts)
///
/// ## Staging Strategies:
/// - `git add file.txt` - Stage specific file
/// - `git add dir/` - Stage all files in directory
/// - `git add .` - Stage all files in current directory and subdirectories
/// - `git add --all` or `git add -A` - Stage all changes (including deletions)
pub struct AddCommand;

impl AddCommand {
    /// Add files to the staging area
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository root
    /// * `file_paths` - List of file paths to stage
    /// * `options` - Add command options
    ///
    /// # Returns
    /// * `Ok(AddResult)` - Information about staged files
    /// * `Err(...)` - If staging failed
    pub fn add<P: AsRef<Path>>(
        repo_path: P,
        file_paths: &[String],
        options: AddOptions,
    ) -> crate::Result<AddResult> {
        let repo_path = repo_path.as_ref();
        let mut repo = GitRepository::new(repo_path);

        // Verify this is a Git repository
        if !repo.is_repository() {
            return Err("Not a git repository (or any of the parent directories): .git".into());
        }

        println!("🔍 Adding files to staging area...");

        // Load existing repository state
        Self::load_repository_state(&mut repo)?;

        // Resolve file paths to actual files
        let resolved_files = Self::resolve_file_paths(&repo, file_paths, &options)?;

        if resolved_files.is_empty() {
            println!("⚠️  No files to add");
            return Ok(AddResult::new());
        }

        // Initialize stores
        let object_store = ObjectStore::new(repo.objects_dir());
        let index_store = IndexStore::new(repo.index_path());

        let mut result = AddResult::new();

        // Process each file
        for file_path in resolved_files {
            match Self::stage_file(&repo, &object_store, &file_path) {
                Ok(entry) => {
                    println!("   ✓ Staged: {}", file_path.display());
                    repo.index.add_entry(entry.clone());
                    result.staged_files.push(entry);
                }
                Err(e) => {
                    println!("   ✗ Failed to stage {}: {}", file_path.display(), e);
                    result.failed_files.push((file_path, e.to_string()));
                }
            }
        }

        // Save updated index
        index_store.save_index(&repo.index)?;

        println!("📊 Staging Summary:");
        println!("   Staged: {} files", result.staged_files.len());
        if !result.failed_files.is_empty() {
            println!("   Failed: {} files", result.failed_files.len());
        }

        Ok(result)
    }

    /// Load existing repository state (index, refs, etc.)
    fn load_repository_state(repo: &mut GitRepository) -> crate::Result<()> {
        // Load index
        let index_store = IndexStore::new(repo.index_path());
        repo.index = index_store.load_index()?;

        // Load references
        let ref_store = RefStore::new(repo.git_dir().to_path_buf());
        repo.refs = ref_store.load_refs()?;

        Ok(())
    }

    /// Resolve file paths based on add options
    fn resolve_file_paths(
        repo: &GitRepository,
        file_paths: &[String],
        options: &AddOptions,
    ) -> crate::Result<Vec<PathBuf>> {
        let mut resolved = Vec::new();

        for file_path in file_paths {
            let path = Path::new(file_path);

            // Convert to absolute path
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                repo.root_path.join(path)
            };

            // Check if path exists
            if !abs_path.exists() {
                if !options.ignore_missing {
                    return Err(format!("pathspec '{}' did not match any files", file_path).into());
                }
                continue;
            }

            if abs_path.is_file() {
                // Single file
                resolved.push(abs_path);
            } else if abs_path.is_dir() {
                // Directory - recursively add files
                Self::collect_files_from_directory(repo, &abs_path, &mut resolved, options)?;
            }
        }

        // Remove duplicates and sort
        resolved.sort();
        resolved.dedup();

        Ok(resolved)
    }

    /// Recursively collect files from a directory
    fn collect_files_from_directory(
        repo: &GitRepository,
        dir_path: &Path,
        files: &mut Vec<PathBuf>,
        options: &AddOptions,
    ) -> crate::Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip if ignored
            if repo.is_ignored(&path) {
                continue;
            }

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() && options.recursive {
                Self::collect_files_from_directory(repo, &path, files, options)?;
            }
        }

        Ok(())
    }

    /// Stage a single file
    fn stage_file(
        repo: &GitRepository,
        object_store: &ObjectStore,
        file_path: &Path,
    ) -> crate::Result<IndexEntry> {
        // Read file content
        let content = fs::read(file_path)?;
        let metadata = fs::metadata(file_path)?;

        // Create blob object
        let blob = BlobObject::new(content);
        let blob_object = GitObject::Blob(blob);
        let blob_hash = object_store.store_object(&blob_object)?;

        // Convert to relative path within repository
        let relative_path = repo.to_relative_path(file_path)?;

        // Create index entry
        let entry = IndexEntry::from_file_metadata(relative_path, blob_hash, &metadata);

        Ok(entry)
    }

    /// Show what would be added without actually adding
    pub fn dry_run<P: AsRef<Path>>(
        repo_path: P,
        file_paths: &[String],
        options: AddOptions,
    ) -> crate::Result<Vec<PathBuf>> {
        let repo = GitRepository::new(repo_path);

        if !repo.is_repository() {
            return Err("Not a git repository".into());
        }

        let resolved_files = Self::resolve_file_paths(&repo, file_paths, &options)?;

        println!("📋 Files that would be added:");
        for file in &resolved_files {
            if let Ok(rel_path) = repo.to_relative_path(file) {
                println!("   {}", rel_path.display());
            }
        }

        Ok(resolved_files)
    }
}

/// Options for the add command
#[derive(Debug, Clone)]
pub struct AddOptions {
    /// Add files recursively from directories
    pub recursive: bool,
    /// Include all files (including deleted ones)
    pub all: bool,
    /// Force add ignored files
    pub force: bool,
    /// Don't error on missing files
    pub ignore_missing: bool,
    /// Show what would be added (dry run)
    pub dry_run: bool,
}

impl Default for AddOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            all: false,
            force: false,
            ignore_missing: false,
            dry_run: false,
        }
    }
}

/// Result of the add operation
#[derive(Debug, Clone)]
pub struct AddResult {
    pub staged_files: Vec<IndexEntry>,
    pub failed_files: Vec<(PathBuf, String)>,
}

impl AddResult {
    pub fn new() -> Self {
        Self {
            staged_files: Vec::new(),
            failed_files: Vec::new(),
        }
    }

    pub fn total_staged(&self) -> usize {
        self.staged_files.len()
    }

    pub fn has_failures(&self) -> bool {
        !self.failed_files.is_empty()
    }
}

impl Default for AddResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_repo() -> crate::Result<(tempfile::TempDir, GitRepository)> {
        let temp_dir = tempdir()?;
        let repo = crate::application::InitCommand::init(Some(temp_dir.path()))?;
        Ok((temp_dir, repo))
    }

    fn create_test_file(dir: &Path, name: &str, content: &str) -> crate::Result<PathBuf> {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(file_path)
    }

    #[test]
    fn test_add_single_file() {
        let (temp_dir, _repo) = create_test_repo().unwrap();
        let repo_path = temp_dir.path();

        // Create test file
        create_test_file(repo_path, "test.txt", "Hello, World!").unwrap();

        // Add file
        let result =
            AddCommand::add(repo_path, &["test.txt".to_string()], AddOptions::default()).unwrap();

        assert_eq!(result.total_staged(), 1);
        assert!(!result.has_failures());

        // Verify file was staged
        let staged_file = &result.staged_files[0];
        assert_eq!(staged_file.path, PathBuf::from("test.txt"));
        assert_eq!(staged_file.size, 13); // "Hello, World!" length
    }

    #[test]
    fn test_add_multiple_files() {
        let (temp_dir, _repo) = create_test_repo().unwrap();
        let repo_path = temp_dir.path();

        // Create test files
        create_test_file(repo_path, "file1.txt", "Content 1").unwrap();
        create_test_file(repo_path, "file2.txt", "Content 2").unwrap();

        // Add files
        let result = AddCommand::add(
            repo_path,
            &["file1.txt".to_string(), "file2.txt".to_string()],
            AddOptions::default(),
        )
        .unwrap();

        assert_eq!(result.total_staged(), 2);
        assert!(!result.has_failures());
    }

    #[test]
    fn test_add_directory() {
        let (temp_dir, _repo) = create_test_repo().unwrap();
        let repo_path = temp_dir.path();

        // Create subdirectory with files
        let sub_dir = repo_path.join("src");
        fs::create_dir(&sub_dir).unwrap();
        create_test_file(&sub_dir, "main.rs", "fn main() {}").unwrap();
        create_test_file(&sub_dir, "lib.rs", "// Library").unwrap();

        // Add directory
        let result =
            AddCommand::add(repo_path, &["src".to_string()], AddOptions::default()).unwrap();

        assert_eq!(result.total_staged(), 2);

        // Check that files have correct relative paths
        let paths: Vec<_> = result
            .staged_files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"src/main.rs".to_string()));
        assert!(paths.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn test_add_nonexistent_file() {
        let (temp_dir, _repo) = create_test_repo().unwrap();
        let repo_path = temp_dir.path();

        // Try to add non-existent file
        let result = AddCommand::add(
            repo_path,
            &["nonexistent.txt".to_string()],
            AddOptions::default(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("did not match any files"));
    }

    #[test]
    fn test_add_not_git_repo() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Try to add in non-git directory
        let result = AddCommand::add(repo_path, &["test.txt".to_string()], AddOptions::default());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a git repository"));
    }

    #[test]
    fn test_dry_run() {
        let (temp_dir, _repo) = create_test_repo().unwrap();
        let repo_path = temp_dir.path();

        // Create test file
        create_test_file(repo_path, "test.txt", "Hello, World!").unwrap();

        // Dry run
        let files =
            AddCommand::dry_run(repo_path, &["test.txt".to_string()], AddOptions::default())
                .unwrap();

        assert_eq!(files.len(), 1);

        // Verify index is still empty
        let repo = GitRepository::new(repo_path);
        let index_store = IndexStore::new(repo.index_path());
        let index = index_store.load_index().unwrap();
        assert!(index.is_empty());
    }
}
