use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;

use crate::domain::*;
use crate::infrastructure::*;

/// Git Status Use Case
/// 
/// This implements the `git status` command functionality.
/// 
/// ## What `git status` does:
/// 1. Compares working directory with staging area (index)
/// 2. Compares staging area with last commit
/// 3. Shows modified, added, deleted, and untracked files
/// 4. Displays current branch and commit information
/// 
/// ## Visual Guide - Git Status Areas:
/// ```text
/// Working Directory    Index (Staging)      Repository
/// ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
/// │ file1.txt       │  │ file1.txt       │  │ Last Commit     │
/// │ (modified)      │  │ (staged)        │  │ - file1.txt     │
/// │                 │  │                 │  │ - file2.txt     │
/// │ file2.txt       │  │ file2.txt       │  │                 │
/// │ (unmodified)    │  │ (staged)        │  │                 │
/// │                 │  │                 │  │                 │
/// │ file3.txt       │  │                 │  │                 │
/// │ (untracked)     │  │                 │  │                 │
/// └─────────────────┘  └─────────────────┘  └─────────────────┘
/// 
/// Status Output:
/// - Changes to be committed: file2.txt (new file)
/// - Changes not staged: file1.txt (modified)  
/// - Untracked files: file3.txt
/// ```
/// 
/// ## File States:
/// - **Untracked**: File exists in working directory but not in index or last commit
/// - **Added**: File is in index but not in last commit (new file)
/// - **Modified**: File content differs between working directory and index
/// - **Deleted**: File is in index but not in working directory
/// - **Staged**: File has changes ready to be committed
pub struct StatusCommand;

impl StatusCommand {
    /// Show the working tree status
    /// 
    /// # Arguments
    /// * `repo_path` - Path to the repository root
    /// * `options` - Status command options
    /// 
    /// # Returns
    /// * `Ok(StatusResult)` - Status information
    /// * `Err(...)` - If status check failed
    pub fn status<P: AsRef<Path>>(
        repo_path: P,
        options: StatusOptions,
    ) -> crate::Result<StatusResult> {
        let repo_path = repo_path.as_ref();
        let mut repo = GitRepository::new(repo_path);
        
        // Verify this is a Git repository
        if !repo.is_repository() {
            return Err("Not a git repository (or any of the parent directories): .git".into());
        }
        
        println!("📊 Checking repository status...");
        
        // Load repository state
        Self::load_repository_state(&mut repo)?;
        
        // Get current branch info
        let branch_info = Self::get_branch_info(&repo)?;
        
        // Analyze file changes
        let file_changes = Self::analyze_file_changes(&repo)?;
        
        // Create status result
        let mut result = StatusResult::new(branch_info);
        result.file_changes = file_changes;
        
        // Display status
        Self::display_status(&result, &options);
        
        Ok(result)
    }
    
    /// Load existing repository state
    fn load_repository_state(repo: &mut GitRepository) -> crate::Result<()> {
        // Load index
        let index_store = IndexStore::new(repo.index_path());
        repo.index = index_store.load_index()?;
        
        // Load references
        let ref_store = RefStore::new(repo.git_dir().to_path_buf());
        repo.refs = ref_store.load_refs()?;
        
        Ok(())
    }
    
    /// Get current branch information
    fn get_branch_info(repo: &GitRepository) -> crate::Result<BranchInfo> {
        let current_branch = repo.current_branch();
        let current_commit = repo.current_commit();
        
        let info = BranchInfo {
            current_branch,
            current_commit,
            ahead_behind: None, // TODO: Implement when we have remotes
        };
        
        Ok(info)
    }
    
    /// Analyze file changes across working directory, index, and last commit
    fn analyze_file_changes(repo: &GitRepository) -> crate::Result<FileChanges> {
        let mut changes = FileChanges::new();
        
        // Get all files from different areas
        let working_files = Self::get_working_directory_files(repo)?;
        let staged_files = Self::get_staged_files(repo);
        let committed_files = Self::get_committed_files(repo)?; // Will be empty until we have commits
        
        // Build sets for comparison
        let working_set: HashSet<PathBuf> = working_files.keys().cloned().collect();
        let staged_set: HashSet<PathBuf> = staged_files.keys().cloned().collect();
        let committed_set: HashSet<PathBuf> = committed_files.keys().cloned().collect();
        
        // Find changes to be committed (staged vs last commit)
        for path in &staged_set {
            if !committed_set.contains(path) {
                // New file
                changes.staged_new.push(path.clone());
            } else if staged_files.get(path) != committed_files.get(path) {
                // Modified file
                changes.staged_modified.push(path.clone());
            }
        }
        
        // Find deleted files (in last commit but not staged)
        for path in &committed_set {
            if !staged_set.contains(path) {
                changes.staged_deleted.push(path.clone());
            }
        }
        
        // Find changes not staged for commit (working vs staged)
        for path in &working_set {
            if staged_set.contains(path) {
                // File is tracked - check if modified
                if working_files.get(path) != staged_files.get(path) {
                    changes.modified.push(path.clone());
                }
            } else if committed_set.contains(path) {
                // File was in last commit but not staged
                changes.modified.push(path.clone());
            }
        }
        
        // Find deleted files (in staged/committed but not in working directory)
        for path in staged_set.union(&committed_set) {
            if !working_set.contains(path) {
                changes.deleted.push(path.clone());
            }
        }
        
        // Find untracked files
        for path in &working_set {
            if !staged_set.contains(path) && !committed_set.contains(path) {
                changes.untracked.push(path.clone());
            }
        }
        
        Ok(changes)
    }
    
    /// Get all files in working directory with their content hashes
    fn get_working_directory_files(repo: &GitRepository) -> crate::Result<std::collections::HashMap<PathBuf, ObjectHash>> {
        let mut files = std::collections::HashMap::new();
        Self::scan_directory_recursive(repo, repo.root_path(), &mut files)?;
        Ok(files)
    }
    
    /// Recursively scan directory for files
    fn scan_directory_recursive(
        repo: &GitRepository,
        dir_path: &Path,
        files: &mut std::collections::HashMap<PathBuf, ObjectHash>,
    ) -> crate::Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip .git directory and ignored files
            if repo.is_ignored(&path) {
                continue;
            }
            
            if path.is_file() {
                // Calculate hash for file content
                match fs::read(&path) {
                    Ok(content) => {
                        let blob = BlobObject::new(content);
                        let object_content = format!("blob {}\0", blob.content.len());
                        let mut full_content = object_content.into_bytes();
                        full_content.extend_from_slice(&blob.content);
                        
                        let hash = Self::calculate_hash(&full_content);
                        let rel_path = repo.to_relative_path(&path)?;
                        files.insert(rel_path, hash);
                    }
                    Err(e) => {
                        // Skip files we can't read (permissions, etc.)
                        eprintln!("⚠️  Skipping file {}: {}", path.display(), e);
                        continue;
                    }
                }
            } else if path.is_dir() {
                Self::scan_directory_recursive(repo, &path, files)?;
            }
        }
        
        Ok(())
    }
    
    /// Get files from staging area with their hashes
    fn get_staged_files(repo: &GitRepository) -> std::collections::HashMap<PathBuf, ObjectHash> {
        repo.index.entries.iter()
            .map(|(path, entry)| (path.clone(), entry.hash.clone()))
            .collect()
    }
    
    /// Get files from last commit with their hashes
    fn get_committed_files(_repo: &GitRepository) -> crate::Result<std::collections::HashMap<PathBuf, ObjectHash>> {
        // TODO: Implement when we have commits
        // For now, return empty since no commits exist yet
        Ok(std::collections::HashMap::new())
    }
    
    /// Calculate SHA-1 hash of content
    fn calculate_hash(content: &[u8]) -> ObjectHash {
        use sha1::{Sha1, Digest};
        let mut hasher = Sha1::new();
        hasher.update(content);
        let result = hasher.finalize();
        ObjectHash::new(hex::encode(result))
    }
    
    /// Display status in human-readable format
    fn display_status(result: &StatusResult, _options: &StatusOptions) {
        println!();
        
        // Branch information
        match &result.branch_info.current_branch {
            Some(branch) => println!("On branch {}", branch),
            None => println!("HEAD detached"),
        }
        
        // Commit information
        match &result.branch_info.current_commit {
            Some(commit) => println!("Latest commit: {}", &commit.as_str()[..8]),
            None => println!("No commits yet"),
        }
        
        println!();
        
        // Changes to be committed
        if !result.file_changes.staged_new.is_empty() 
            || !result.file_changes.staged_modified.is_empty() 
            || !result.file_changes.staged_deleted.is_empty() {
            println!("Changes to be committed:");
            println!("  (use \"git-rs commit\" to commit)");
            println!();
            
            for file in &result.file_changes.staged_new {
                println!("	new file:   {}", file.display());
            }
            for file in &result.file_changes.staged_modified {
                println!("	modified:   {}", file.display());
            }
            for file in &result.file_changes.staged_deleted {
                println!("	deleted:    {}", file.display());
            }
            println!();
        }
        
        // Changes not staged for commit
        if !result.file_changes.modified.is_empty() || !result.file_changes.deleted.is_empty() {
            println!("Changes not staged for commit:");
            println!("  (use \"git-rs add <file>...\" to update what will be committed)");
            println!("  (use \"git-rs checkout -- <file>...\" to discard changes)");
            println!();
            
            for file in &result.file_changes.modified {
                println!("	modified:   {}", file.display());
            }
            for file in &result.file_changes.deleted {
                println!("	deleted:    {}", file.display());
            }
            println!();
        }
        
        // Untracked files
        if !result.file_changes.untracked.is_empty() {
            println!("Untracked files:");
            println!("  (use \"git-rs add <file>...\" to include in what will be committed)");
            println!();
            
            for file in &result.file_changes.untracked {
                println!("	{}", file.display());
            }
            println!();
        }
        
        // Summary message
        if result.is_clean() {
            println!("nothing to commit, working tree clean");
        } else if result.has_staged_changes() {
            println!("Changes ready to be committed!");
        }
    }
}

/// Branch information
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub current_branch: Option<String>,
    pub current_commit: Option<ObjectHash>,
    pub ahead_behind: Option<(usize, usize)>, // (ahead, behind) remote
}

/// File changes across different areas
#[derive(Debug, Clone)]
pub struct FileChanges {
    // Changes to be committed (staged vs last commit)
    pub staged_new: Vec<PathBuf>,
    pub staged_modified: Vec<PathBuf>,
    pub staged_deleted: Vec<PathBuf>,
    
    // Changes not staged for commit (working vs staged)
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    
    // Untracked files
    pub untracked: Vec<PathBuf>,
}

impl FileChanges {
    pub fn new() -> Self {
        Self {
            staged_new: Vec::new(),
            staged_modified: Vec::new(),
            staged_deleted: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            untracked: Vec::new(),
        }
    }
}

/// Status command options
#[derive(Debug, Clone)]
pub struct StatusOptions {
    pub short_format: bool,
    pub porcelain: bool,
    pub show_ignored: bool,
}

impl Default for StatusOptions {
    fn default() -> Self {
        Self {
            short_format: false,
            porcelain: false,
            show_ignored: false,
        }
    }
}

/// Result of the status operation
#[derive(Debug, Clone)]
pub struct StatusResult {
    pub branch_info: BranchInfo,
    pub file_changes: FileChanges,
}

impl StatusResult {
    pub fn new(branch_info: BranchInfo) -> Self {
        Self {
            branch_info,
            file_changes: FileChanges::new(),
        }
    }
    
    /// Check if working tree is clean
    pub fn is_clean(&self) -> bool {
        self.file_changes.staged_new.is_empty()
            && self.file_changes.staged_modified.is_empty()
            && self.file_changes.staged_deleted.is_empty()
            && self.file_changes.modified.is_empty()
            && self.file_changes.deleted.is_empty()
            && self.file_changes.untracked.is_empty()
    }
    
    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> bool {
        !self.file_changes.staged_new.is_empty()
            || !self.file_changes.staged_modified.is_empty()
            || !self.file_changes.staged_deleted.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    
    fn create_test_repo_with_files() -> crate::Result<(tempfile::TempDir, GitRepository)> {
        let temp_dir = tempdir()?;
        let repo = crate::application::InitCommand::init(Some(temp_dir.path()))?;
        
        // Create test files
        let mut file1 = File::create(temp_dir.path().join("file1.txt"))?;
        file1.write_all(b"Hello, World!")?;
        
        let mut file2 = File::create(temp_dir.path().join("file2.txt"))?;
        file2.write_all(b"Test content")?;
        
        Ok((temp_dir, repo))
    }
    
    #[test]
    fn test_status_clean_repository() {
        let (temp_dir, _repo) = create_test_repo_with_files().unwrap();
        let repo_path = temp_dir.path();
        
        // Remove test files to have a clean repo
        std::fs::remove_file(repo_path.join("file1.txt")).unwrap();
        std::fs::remove_file(repo_path.join("file2.txt")).unwrap();
        
        let result = StatusCommand::status(repo_path, StatusOptions::default()).unwrap();
        
        assert!(result.is_clean());
        assert!(!result.has_staged_changes());
    }
    
    #[test]
    fn test_status_with_untracked_files() {
        let (temp_dir, _repo) = create_test_repo_with_files().unwrap();
        let repo_path = temp_dir.path();
        
        let result = StatusCommand::status(repo_path, StatusOptions::default()).unwrap();
        
        assert!(!result.is_clean());
        assert_eq!(result.file_changes.untracked.len(), 2);
        assert!(result.file_changes.untracked.contains(&PathBuf::from("file1.txt")));
        assert!(result.file_changes.untracked.contains(&PathBuf::from("file2.txt")));
    }
    
    #[test]
    fn test_status_with_staged_files() {
        let (temp_dir, _repo) = create_test_repo_with_files().unwrap();
        let repo_path = temp_dir.path();
        
        // Stage files
        crate::application::AddCommand::add(
            repo_path,
            &["file1.txt".to_string()],
            crate::application::AddOptions::default(),
        ).unwrap();
        
        let result = StatusCommand::status(repo_path, StatusOptions::default()).unwrap();
        
        assert!(!result.is_clean());
        assert!(result.has_staged_changes());
        assert_eq!(result.file_changes.staged_new.len(), 1);
        assert_eq!(result.file_changes.untracked.len(), 1);
        assert!(result.file_changes.staged_new.contains(&PathBuf::from("file1.txt")));
        assert!(result.file_changes.untracked.contains(&PathBuf::from("file2.txt")));
    }
}
