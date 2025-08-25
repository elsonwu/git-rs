use std::path::PathBuf;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::objects::{ObjectHash, FileMode};

/// Index entry representing a staged file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Creation time
    pub ctime: DateTime<Utc>,
    /// Modification time
    pub mtime: DateTime<Utc>,
    /// Device ID
    pub dev: u32,
    /// Inode number
    pub ino: u32,
    /// File mode (permissions)
    pub mode: FileMode,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// File size
    pub size: u64,
    /// Object hash of the file content
    pub hash: ObjectHash,
    /// Stage number (0 = normal, 1-3 = conflict resolution)
    pub stage: u16,
    /// File path relative to repository root
    pub path: PathBuf,
}

impl IndexEntry {
    pub fn new(
        path: PathBuf,
        hash: ObjectHash,
        size: u64,
        mode: FileMode,
    ) -> Self {
        let now = Utc::now();
        Self {
            ctime: now,
            mtime: now,
            dev: 0,
            ino: 0,
            mode,
            uid: 0,
            gid: 0,
            size,
            hash,
            stage: 0,
            path,
        }
    }
    
    /// Create an index entry from file metadata
    pub fn from_file_metadata(
        path: PathBuf,
        hash: ObjectHash,
        metadata: &std::fs::Metadata,
    ) -> Self {
        use std::os::unix::fs::MetadataExt;
        
        let mode = if metadata.is_file() {
            if metadata.mode() & 0o111 != 0 {
                FileMode::Executable
            } else {
                FileMode::Regular
            }
        } else if metadata.is_dir() {
            FileMode::Directory
        } else {
            FileMode::Symlink
        };
        
        Self {
            ctime: metadata.created()
                .ok()
                .and_then(|t| DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0))
                .unwrap_or_else(Utc::now),
            mtime: metadata.modified()
                .ok()
                .and_then(|t| DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0))
                .unwrap_or_else(Utc::now),
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            mode,
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.len(),
            hash,
            stage: 0,
            path,
        }
    }
    
    /// Get the file name
    pub fn name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }
    
    /// Check if this entry represents a regular file
    pub fn is_file(&self) -> bool {
        matches!(self.mode, FileMode::Regular | FileMode::Executable)
    }
    
    /// Check if this entry represents a directory
    pub fn is_dir(&self) -> bool {
        matches!(self.mode, FileMode::Directory)
    }
}

/// Git Index (staging area) containing staged files
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitIndex {
    /// Version of the index format
    pub version: u32,
    /// Map of file paths to index entries
    pub entries: HashMap<PathBuf, IndexEntry>,
}

impl GitIndex {
    pub fn new() -> Self {
        Self {
            version: 2, // Git index format version 2
            entries: HashMap::new(),
        }
    }
    
    /// Add a file to the index
    pub fn add_entry(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }
    
    /// Remove a file from the index
    pub fn remove_entry(&mut self, path: &PathBuf) -> Option<IndexEntry> {
        self.entries.remove(path)
    }
    
    /// Get an entry by path
    pub fn get_entry(&self, path: &PathBuf) -> Option<&IndexEntry> {
        self.entries.get(path)
    }
    
    /// Get all entries sorted by path
    pub fn get_sorted_entries(&self) -> Vec<&IndexEntry> {
        let mut entries: Vec<&IndexEntry> = self.entries.values().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }
    
    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Check if a file is staged
    pub fn is_staged(&self, path: &PathBuf) -> bool {
        self.entries.contains_key(path)
    }
    
    /// Get all staged file paths
    pub fn staged_paths(&self) -> Vec<&PathBuf> {
        self.entries.keys().collect()
    }
    
    /// Update an existing entry or add a new one
    pub fn update_entry(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }
}

impl Default for GitIndex {
    fn default() -> Self {
        Self::new()
    }
}
