use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::*;
use crate::infrastructure::*;

/// Git Diff Use Case
///
/// This implements the `git diff` command functionality.
///
/// ## What `git diff` does:
/// 1. Compares different states of files (working vs staged, staged vs committed)
/// 2. Shows line-by-line differences in unified diff format
/// 3. Identifies added, modified, and deleted files
/// 4. Handles binary files appropriately
///
/// ## Visual Guide - Git Diff Modes:
/// ```text
/// Working Directory    Index (Staging)      Repository (HEAD)
/// ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
/// │ file1.txt       │  │ file1.txt       │  │ Last Commit     │
/// │ "Hello World"   │  │ "Hello Rust"    │  │ file1.txt       │
/// │ (modified)      │  │ (staged)        │  │ "Hello Git"     │
/// │                 │  │                 │  │                 │
/// │ file2.txt       │  │ file2.txt       │  │                 │
/// │ "New content"   │  │ (not staged)    │  │                 │
/// │ (untracked)     │  │                 │  │                 │
/// └─────────────────┘  └─────────────────┘  └─────────────────┘
///
/// git diff (working vs staged):
/// Shows changes in working directory that have not been staged
///
/// git diff --cached (staged vs committed):
/// Shows changes in staging area that will be committed
/// ```
///
/// ## Diff Format:
/// ```diff
/// diff --git a/file1.txt b/file1.txt
/// index abc123..def456 100644
/// --- a/file1.txt
/// +++ b/file1.txt
/// @@ -1,2 +1,2 @@
///  Hello
/// -Git
/// +Rust
/// ```
pub struct DiffCommand;

/// Options for diff command
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// Show staged changes instead of working directory changes
    pub cached: bool,
    /// Context lines around changes
    pub context_lines: usize,
    /// Show binary files as binary
    pub show_binary: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            cached: false,
            context_lines: 3,
            show_binary: false,
        }
    }
}

/// Result of a diff operation
#[derive(Debug)]
pub struct DiffResult {
    /// List of file diffs
    pub file_diffs: Vec<FileDiff>,
    /// Total number of files changed
    pub files_changed: usize,
    /// Total lines added
    pub lines_added: usize,
    /// Total lines removed
    pub lines_removed: usize,
}

/// Diff for a single file
#[derive(Debug)]
pub struct FileDiff {
    /// Path of the file
    pub path: PathBuf,
    /// Type of change
    pub change_type: FileChangeType,
    /// Hash of old version (if exists)
    pub old_hash: Option<String>,
    /// Hash of new version (if exists)
    pub new_hash: Option<String>,
    /// File mode
    pub mode: String,
    /// Diff chunks
    pub chunks: Vec<DiffChunk>,
    /// Whether this is a binary file
    pub is_binary: bool,
}

/// Type of file change
#[derive(Debug, PartialEq)]
pub enum FileChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
}

/// A chunk of diff showing changes
#[derive(Debug)]
pub struct DiffChunk {
    /// Starting line in old file
    pub old_start: usize,
    /// Number of lines in old file
    pub old_count: usize,
    /// Starting line in new file
    pub new_start: usize,
    /// Number of lines in new file
    pub new_count: usize,
    /// The actual diff lines
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug)]
pub struct DiffLine {
    /// Type of line change
    pub line_type: DiffLineType,
    /// The content of the line
    pub content: String,
}

/// Type of diff line
#[derive(Debug, PartialEq)]
pub enum DiffLineType {
    /// Context line (unchanged)
    Context,
    /// Added line
    Added,
    /// Removed line
    Removed,
}

impl DiffCommand {
    /// Show differences between different states
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository root
    /// * `options` - Diff command options
    ///
    /// # Returns
    /// * `Ok(DiffResult)` - The diff information
    /// * `Err(...)` - If diff failed
    pub fn diff<P: AsRef<Path>>(repo_path: P, options: DiffOptions) -> crate::Result<DiffResult> {
        let repo_path = repo_path.as_ref();
        let repo = GitRepository::new(repo_path);

        // Verify this is a Git repository
        if !repo.is_repository() {
            return Err(format!(
                "Not a git repository (or any of the parent directories): {}",
                repo_path.display()
            )
            .into());
        }

        let mut file_diffs = Vec::new();
        let mut lines_added = 0;
        let mut lines_removed = 0;

        let git_dir = repo_path.join(".git-rs");

        if options.cached {
            // Compare staged vs committed (git diff --cached)
            let file_diffs_result = Self::diff_staged_vs_committed(&git_dir)?;
            for diff in file_diffs_result {
                lines_added += diff
                    .chunks
                    .iter()
                    .map(|c| {
                        c.lines
                            .iter()
                            .filter(|l| l.line_type == DiffLineType::Added)
                            .count()
                    })
                    .sum::<usize>();
                lines_removed += diff
                    .chunks
                    .iter()
                    .map(|c| {
                        c.lines
                            .iter()
                            .filter(|l| l.line_type == DiffLineType::Removed)
                            .count()
                    })
                    .sum::<usize>();
                file_diffs.push(diff);
            }
        } else {
            // Compare working vs staged (git diff)
            let file_diffs_result = Self::diff_working_vs_staged(repo_path, &git_dir)?;
            for diff in file_diffs_result {
                lines_added += diff
                    .chunks
                    .iter()
                    .map(|c| {
                        c.lines
                            .iter()
                            .filter(|l| l.line_type == DiffLineType::Added)
                            .count()
                    })
                    .sum::<usize>();
                lines_removed += diff
                    .chunks
                    .iter()
                    .map(|c| {
                        c.lines
                            .iter()
                            .filter(|l| l.line_type == DiffLineType::Removed)
                            .count()
                    })
                    .sum::<usize>();
                file_diffs.push(diff);
            }
        }

        let files_changed = file_diffs.len();

        Ok(DiffResult {
            file_diffs,
            files_changed,
            lines_added,
            lines_removed,
        })
    }

    /// Compare working directory vs staged files
    fn diff_working_vs_staged(repo_path: &Path, git_dir: &Path) -> crate::Result<Vec<FileDiff>> {
        let mut diffs = Vec::new();

        // Load index
        let index_store = IndexStore::new(git_dir.join("git-rs-index"));
        let index = index_store.load_index()?;

        // Get working directory files
        let working_files = Self::get_working_directory_files(repo_path)?;

        // Compare each staged file with working directory version
        for (path, entry) in &index.entries {
            let working_path = repo_path.join(path);

            if working_path.exists() {
                // File exists in both working directory and index
                let working_content = fs::read(&working_path)?;
                let working_hash = Self::calculate_content_hash(&working_content);

                if working_hash != entry.hash.to_string() {
                    // File is modified
                    let staged_content =
                        Self::get_object_content(git_dir, &entry.hash.to_string())?;
                    let diff = Self::create_file_diff(
                        path.clone(),
                        Some(staged_content),
                        Some(working_content),
                        Some(entry.hash.to_string()),
                        Some(working_hash),
                        FileChangeType::Modified,
                    )?;
                    diffs.push(diff);
                }
            } else {
                // File was deleted from working directory
                let staged_content = Self::get_object_content(git_dir, &entry.hash.to_string())?;
                let diff = Self::create_file_diff(
                    path.clone(),
                    Some(staged_content),
                    None,
                    Some(entry.hash.to_string()),
                    None,
                    FileChangeType::Deleted,
                )?;
                diffs.push(diff);
            }
        }

        // Check for untracked files (exist in working directory but not in index)
        for (path, _) in working_files {
            if !index.entries.contains_key(&path) {
                let working_path = repo_path.join(&path);
                let working_content = fs::read(&working_path)?;
                let working_hash = Self::calculate_content_hash(&working_content);

                let diff = Self::create_file_diff(
                    path,
                    None,
                    Some(working_content),
                    None,
                    Some(working_hash),
                    FileChangeType::Added,
                )?;
                diffs.push(diff);
            }
        }

        Ok(diffs)
    }

    /// Compare staged files vs committed files
    fn diff_staged_vs_committed(git_dir: &Path) -> crate::Result<Vec<FileDiff>> {
        let mut diffs = Vec::new();

        // Load index
        let index_store = IndexStore::new(git_dir.join("git-rs-index"));
        let index = index_store.load_index()?;

        // Get HEAD commit files
        let head_files = Self::get_head_commit_files(git_dir)?;

        // Compare each staged file with committed version
        for (path, entry) in &index.entries {
            if let Some(head_hash) = head_files.get(path) {
                if entry.hash.to_string() != *head_hash {
                    // File is modified in staging
                    let staged_content =
                        Self::get_object_content(git_dir, &entry.hash.to_string())?;
                    let committed_content = Self::get_object_content(git_dir, head_hash)?;
                    let diff = Self::create_file_diff(
                        path.clone(),
                        Some(committed_content),
                        Some(staged_content),
                        Some(head_hash.clone()),
                        Some(entry.hash.to_string()),
                        FileChangeType::Modified,
                    )?;
                    diffs.push(diff);
                }
            } else {
                // File is new in staging (not in HEAD)
                let staged_content = Self::get_object_content(git_dir, &entry.hash.to_string())?;
                let diff = Self::create_file_diff(
                    path.clone(),
                    None,
                    Some(staged_content),
                    None,
                    Some(entry.hash.to_string()),
                    FileChangeType::Added,
                )?;
                diffs.push(diff);
            }
        }

        // Check for files deleted from staging (in HEAD but not in index)
        for (path, head_hash) in head_files {
            if !index.entries.contains_key(&path) {
                let committed_content = Self::get_object_content(git_dir, &head_hash)?;
                let diff = Self::create_file_diff(
                    path,
                    Some(committed_content),
                    None,
                    Some(head_hash),
                    None,
                    FileChangeType::Deleted,
                )?;
                diffs.push(diff);
            }
        }

        Ok(diffs)
    }

    /// Calculate hash for content (simple SHA-1 of blob format)
    fn calculate_content_hash(content: &[u8]) -> String {
        use sha1::{Digest, Sha1};

        let header = format!("blob {}{}", content.len(), '\0');
        let mut hasher = Sha1::new();
        hasher.update(header.as_bytes());
        hasher.update(content);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Get all files in the working directory
    fn get_working_directory_files(work_dir: &Path) -> crate::Result<HashMap<PathBuf, ()>> {
        let mut files = HashMap::new();
        Self::scan_directory_recursive(work_dir, work_dir, &mut files)?;
        Ok(files)
    }

    /// Recursively scan directory for files
    fn scan_directory_recursive(
        dir: &Path,
        repo_root: &Path,
        files: &mut HashMap<PathBuf, ()>,
    ) -> crate::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip git-rs directory
            if path.file_name().is_some_and(|name| name == ".git-rs") {
                continue;
            }

            if path.is_file() {
                // Convert to relative path from repo root
                let relative_path = path
                    .strip_prefix(repo_root)
                    .map_err(|_| format!("Path not within repository: {}", path.display()))?;
                files.insert(relative_path.to_path_buf(), ());
            } else if path.is_dir() {
                Self::scan_directory_recursive(&path, repo_root, files)?;
            }
        }
        Ok(())
    }

    /// Get all files from HEAD commit
    fn get_head_commit_files(git_dir: &Path) -> crate::Result<HashMap<PathBuf, String>> {
        let mut files = HashMap::new();

        // Read HEAD reference
        let ref_store = RefStore::new(git_dir.to_path_buf());
        if let Some(head_commit_hash) = ref_store.get_head()? {
            // Get commit object
            let object_store = ObjectStore::new(git_dir.join("objects"));
            let commit_obj = object_store.load_object(&head_commit_hash)?;

            if let GitObject::Commit(commit) = commit_obj {
                // Get tree object and extract files
                Self::extract_tree_files(
                    &object_store,
                    &commit.tree.to_string(),
                    &mut files,
                    &PathBuf::new(),
                )?;
            }
        }

        Ok(files)
    }

    /// Extract all files from a tree object recursively
    fn extract_tree_files(
        object_store: &ObjectStore,
        tree_hash: &str,
        files: &mut HashMap<PathBuf, String>,
        current_path: &Path,
    ) -> crate::Result<()> {
        let tree_hash_obj = ObjectHash::new(tree_hash.to_string());
        let tree_obj = object_store.load_object(&tree_hash_obj)?;

        if let GitObject::Tree(tree) = tree_obj {
            for entry in &tree.entries {
                let entry_path = current_path.join(&entry.name);

                match entry.mode {
                    FileMode::Regular | FileMode::Executable => {
                        // It's a file
                        files.insert(entry_path, entry.hash.to_string());
                    }
                    FileMode::Directory => {
                        // It's a directory (tree)
                        Self::extract_tree_files(
                            object_store,
                            &entry.hash.to_string(),
                            files,
                            &entry_path,
                        )?;
                    }
                    _ => {
                        // Handle other types if needed (symlinks, etc.)
                    }
                }
            }
        }

        Ok(())
    }

    /// Get content of an object by hash
    fn get_object_content(git_dir: &Path, hash: &str) -> crate::Result<Vec<u8>> {
        let object_store = ObjectStore::new(git_dir.join("objects"));
        let hash_obj = ObjectHash::new(hash.to_string());
        let obj = object_store.load_object(&hash_obj)?;

        match obj {
            GitObject::Blob(blob) => Ok(blob.content),
            _ => Err(format!("Object {} is not a blob", hash).into()),
        }
    }

    /// Create a file diff from old and new content
    fn create_file_diff(
        path: PathBuf,
        old_content: Option<Vec<u8>>,
        new_content: Option<Vec<u8>>,
        old_hash: Option<String>,
        new_hash: Option<String>,
        change_type: FileChangeType,
    ) -> crate::Result<FileDiff> {
        // Check if files are binary
        let is_binary =
            Self::is_binary_content(&old_content) || Self::is_binary_content(&new_content);

        let chunks = if is_binary {
            // For binary files, don't show line-by-line diff
            vec![]
        } else {
            // Convert to text and create diff
            let old_text = old_content
                .map(|c| String::from_utf8_lossy(&c).to_string())
                .unwrap_or_default();
            let new_text = new_content
                .map(|c| String::from_utf8_lossy(&c).to_string())
                .unwrap_or_default();

            Self::create_unified_diff(&old_text, &new_text)?
        };

        Ok(FileDiff {
            path,
            change_type,
            old_hash,
            new_hash,
            mode: "100644".to_string(), // TODO: Get actual file mode
            chunks,
            is_binary,
        })
    }

    /// Check if content is binary
    fn is_binary_content(content: &Option<Vec<u8>>) -> bool {
        if let Some(bytes) = content {
            // Simple heuristic: if we find null bytes in first 8KB, consider it binary
            let check_size = std::cmp::min(bytes.len(), 8192);
            bytes[..check_size].contains(&0)
        } else {
            false
        }
    }

    /// Create unified diff chunks from two text strings
    fn create_unified_diff(old_text: &str, new_text: &str) -> crate::Result<Vec<DiffChunk>> {
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();

        // Simple diff algorithm - for educational purposes
        // In a real implementation, you'd use Myers' algorithm or similar
        let mut chunks = Vec::new();

        let mut old_idx = 0;
        let mut new_idx = 0;

        while old_idx < old_lines.len() || new_idx < new_lines.len() {
            let mut chunk_lines = Vec::new();
            let chunk_old_start = old_idx + 1; // Line numbers are 1-based
            let chunk_new_start = new_idx + 1;
            let mut chunk_old_count = 0;
            let mut chunk_new_count = 0;

            // Find a block of differences
            while old_idx < old_lines.len() || new_idx < new_lines.len() {
                if old_idx >= old_lines.len() {
                    // Only new lines remain
                    chunk_lines.push(DiffLine {
                        line_type: DiffLineType::Added,
                        content: new_lines[new_idx].to_string(),
                    });
                    new_idx += 1;
                    chunk_new_count += 1;
                } else if new_idx >= new_lines.len() {
                    // Only old lines remain
                    chunk_lines.push(DiffLine {
                        line_type: DiffLineType::Removed,
                        content: old_lines[old_idx].to_string(),
                    });
                    old_idx += 1;
                    chunk_old_count += 1;
                } else if old_lines[old_idx] == new_lines[new_idx] {
                    // Lines are the same
                    chunk_lines.push(DiffLine {
                        line_type: DiffLineType::Context,
                        content: old_lines[old_idx].to_string(),
                    });
                    old_idx += 1;
                    new_idx += 1;
                    chunk_old_count += 1;
                    chunk_new_count += 1;

                    // If we've collected enough context, end this chunk
                    if chunk_lines.len() >= 10 {
                        // Simple chunk size limit
                        break;
                    }
                } else {
                    // Lines are different - simple approach: one removed, one added
                    chunk_lines.push(DiffLine {
                        line_type: DiffLineType::Removed,
                        content: old_lines[old_idx].to_string(),
                    });
                    chunk_lines.push(DiffLine {
                        line_type: DiffLineType::Added,
                        content: new_lines[new_idx].to_string(),
                    });
                    old_idx += 1;
                    new_idx += 1;
                    chunk_old_count += 1;
                    chunk_new_count += 1;
                }

                // Break if chunk gets too large
                if chunk_lines.len() >= 20 {
                    break;
                }
            }

            if !chunk_lines.is_empty() {
                chunks.push(DiffChunk {
                    old_start: chunk_old_start,
                    old_count: chunk_old_count,
                    new_start: chunk_new_start,
                    new_count: chunk_new_count,
                    lines: chunk_lines,
                });
            } else {
                break;
            }
        }

        Ok(chunks)
    }
}

impl DiffResult {
    /// Generate a summary of the diff results
    pub fn summary(&self) -> String {
        if self.files_changed == 0 {
            return String::from("No changes");
        }

        let mut parts = Vec::new();
        parts.push(format!(
            "{} file{} changed",
            self.files_changed,
            if self.files_changed == 1 { "" } else { "s" }
        ));

        if self.lines_added > 0 {
            parts.push(format!(
                "{} insertion{}",
                self.lines_added,
                if self.lines_added == 1 { "" } else { "s" }
            ));
        }

        if self.lines_removed > 0 {
            parts.push(format!(
                "{} deletion{}",
                self.lines_removed,
                if self.lines_removed == 1 { "" } else { "s" }
            ));
        }

        parts.join(", ")
    }

    /// Print the diff in unified format
    pub fn print_unified(&self) {
        for file_diff in &self.file_diffs {
            self.print_file_diff(file_diff);
        }

        if !self.file_diffs.is_empty() {
            println!("\n📊 {}", self.summary());
        }
    }

    fn print_file_diff(&self, file_diff: &FileDiff) {
        // Print file header
        match file_diff.change_type {
            FileChangeType::Added => {
                println!(
                    "diff --git a/{} b/{}",
                    file_diff.path.display(),
                    file_diff.path.display()
                );
                println!("new file mode {}", file_diff.mode);
                if let Some(hash) = &file_diff.new_hash {
                    println!("index 0000000..{} {}", &hash[..7], file_diff.mode);
                }
                println!("--- /dev/null");
                println!("+++ b/{}", file_diff.path.display());
            }
            FileChangeType::Deleted => {
                println!(
                    "diff --git a/{} b/{}",
                    file_diff.path.display(),
                    file_diff.path.display()
                );
                println!("deleted file mode {}", file_diff.mode);
                if let Some(hash) = &file_diff.old_hash {
                    println!("index {}..0000000 {}", &hash[..7], file_diff.mode);
                }
                println!("--- a/{}", file_diff.path.display());
                println!("+++ /dev/null");
            }
            FileChangeType::Modified => {
                println!(
                    "diff --git a/{} b/{}",
                    file_diff.path.display(),
                    file_diff.path.display()
                );
                if let (Some(old_hash), Some(new_hash)) = (&file_diff.old_hash, &file_diff.new_hash)
                {
                    println!(
                        "index {}..{} {}",
                        &old_hash[..7],
                        &new_hash[..7],
                        file_diff.mode
                    );
                }
                println!("--- a/{}", file_diff.path.display());
                println!("+++ b/{}", file_diff.path.display());
            }
        }

        if file_diff.is_binary {
            println!("Binary files differ");
        } else {
            // Print chunks
            for chunk in &file_diff.chunks {
                println!(
                    "@@ -{},{} +{},{} @@",
                    chunk.old_start, chunk.old_count, chunk.new_start, chunk.new_count
                );

                for line in &chunk.lines {
                    let prefix = match line.line_type {
                        DiffLineType::Context => " ",
                        DiffLineType::Added => "+",
                        DiffLineType::Removed => "-",
                    };
                    println!("{}{}", prefix, line.content);
                }
            }
        }

        println!(); // Empty line between files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize repository
        let _repo = crate::application::init::InitCommand::init(Some(&repo_path)).unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_diff_empty_repository() {
        let (_temp_dir, _repo_path) = setup_test_repo();

        let result = DiffCommand::diff(&_repo_path, DiffOptions::default()).unwrap();

        assert_eq!(result.files_changed, 0);
        assert_eq!(result.lines_added, 0);
        assert_eq!(result.lines_removed, 0);
        assert_eq!(result.summary(), "No changes");
    }

    #[test]
    fn test_diff_untracked_file() {
        let (_temp_dir, repo_path) = setup_test_repo();

        // Create an untracked file
        fs::write(repo_path.join("test.txt"), "Hello World\n").unwrap();

        let result = DiffCommand::diff(&repo_path, DiffOptions::default()).unwrap();

        assert_eq!(result.files_changed, 1);
        assert_eq!(result.lines_added, 1);
        assert_eq!(result.lines_removed, 0);
        assert!(result.summary().contains("1 file changed"));
        assert!(result.summary().contains("1 insertion"));

        let file_diff = &result.file_diffs[0];
        assert_eq!(file_diff.change_type, FileChangeType::Added);
        assert_eq!(file_diff.path, PathBuf::from("test.txt"));
        assert!(file_diff.old_hash.is_none());
        assert!(file_diff.new_hash.is_some());
    }

    #[test]
    fn test_diff_binary_file() {
        let (_temp_dir, repo_path) = setup_test_repo();

        // Create a binary file (contains null bytes)
        fs::write(repo_path.join("binary.dat"), [0, 1, 2, 3, 0, 5]).unwrap();

        let result = DiffCommand::diff(&repo_path, DiffOptions::default()).unwrap();

        assert_eq!(result.files_changed, 1);
        let file_diff = &result.file_diffs[0];
        assert!(file_diff.is_binary);
        assert!(file_diff.chunks.is_empty()); // Binary files don't have line chunks
    }

    #[test]
    fn test_diff_modified_lines() {
        let (_temp_dir, _repo_path) = setup_test_repo();

        // Test the unified diff algorithm
        let old_text = "line 1\nline 2\nline 3\n";
        let new_text = "line 1\nmodified line 2\nline 3\n";

        let chunks = DiffCommand::create_unified_diff(old_text, new_text).unwrap();

        assert_eq!(chunks.len(), 1);
        let chunk = &chunks[0];
        assert_eq!(chunk.old_start, 1);
        assert_eq!(chunk.new_start, 1);

        // Should have context lines + removed + added
        let removed_lines: Vec<_> = chunk
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .collect();
        let added_lines: Vec<_> = chunk
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Added)
            .collect();

        assert_eq!(removed_lines.len(), 1);
        assert_eq!(added_lines.len(), 1);
        assert!(removed_lines[0].content.contains("line 2"));
        assert!(added_lines[0].content.contains("modified line 2"));
    }

    #[test]
    fn test_hash_calculation() {
        let content = b"Hello World\n";
        let hash = DiffCommand::calculate_content_hash(content);

        // This should match git's blob hash calculation
        // echo -n "Hello World" | git hash-object --stdin
        // But we need to account for the blob header format: "blob <size>\0<content>"
        assert_eq!(hash.len(), 40); // SHA-1 is 40 hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_directory_scanning() {
        let (_temp_dir, repo_path) = setup_test_repo();

        // Create nested directory structure
        fs::create_dir_all(repo_path.join("src/lib")).unwrap();
        fs::write(repo_path.join("README.md"), "# Test\n").unwrap();
        fs::write(repo_path.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(repo_path.join("src/lib/mod.rs"), "// module\n").unwrap();

        let files = DiffCommand::get_working_directory_files(&repo_path).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.contains_key(&PathBuf::from("README.md")));
        assert!(files.contains_key(&PathBuf::from("src/main.rs")));
        assert!(files.contains_key(&PathBuf::from("src/lib/mod.rs")));
    }
}
