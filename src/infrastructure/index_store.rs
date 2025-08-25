use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::domain::index::*;

/// Index Storage Implementation
///
/// This handles the physical storage and retrieval of the Git index (staging area).
/// The index is stored in .git/index as a binary file containing staged file information.
///
/// For simplicity, we'll use JSON format instead of Git's complex binary format.
/// In a real implementation, you'd want to use Git's actual index format.
pub struct IndexStore {
    index_path: PathBuf,
}

impl IndexStore {
    /// Create a new index store
    pub fn new(index_path: PathBuf) -> Self {
        Self { index_path }
    }

    /// Load the index from disk
    pub fn load_index(&self) -> crate::Result<GitIndex> {
        if !self.index_path.exists() {
            return Ok(GitIndex::new());
        }

        let content = fs::read_to_string(&self.index_path)?;
        if content.trim().is_empty() {
            return Ok(GitIndex::new());
        }

        let index: GitIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// Save the index to disk
    pub fn save_index(&self, index: &GitIndex) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(index)?;
        fs::write(&self.index_path, content)?;
        Ok(())
    }

    /// Clear the index (remove all staged files)
    pub fn clear_index(&self) -> crate::Result<()> {
        let empty_index = GitIndex::new();
        self.save_index(&empty_index)
    }

    /// Check if the index file exists
    pub fn index_exists(&self) -> bool {
        self.index_path.exists()
    }

    /// Remove the index file
    pub fn remove_index(&self) -> crate::Result<()> {
        if self.index_path.exists() {
            fs::remove_file(&self.index_path)?;
        }
        Ok(())
    }

    /// Get the index file path
    pub fn index_path(&self) -> &PathBuf {
        &self.index_path
    }
}

/// Binary Index Store Implementation
///
/// This is a more advanced implementation that uses a binary format
/// similar to Git's actual index format, but simplified for educational purposes.
pub struct BinaryIndexStore {
    index_path: PathBuf,
}

impl BinaryIndexStore {
    /// Create a new binary index store
    pub fn new(index_path: PathBuf) -> Self {
        Self { index_path }
    }

    /// Load the index from binary format
    pub fn load_index(&self) -> crate::Result<GitIndex> {
        if !self.index_path.exists() {
            return Ok(GitIndex::new());
        }

        let mut file = fs::File::open(&self.index_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        self.deserialize_index(&buffer)
    }

    /// Save the index in binary format
    pub fn save_index(&self, index: &GitIndex) -> crate::Result<()> {
        let serialized = self.serialize_index(index)?;
        let mut file = fs::File::create(&self.index_path)?;
        file.write_all(&serialized)?;
        Ok(())
    }

    /// Serialize index to binary format
    ///
    /// Simplified format:
    /// - 4 bytes: signature "DIRC" (DIRectory Cache)
    /// - 4 bytes: version number
    /// - 4 bytes: number of entries
    /// - For each entry:
    ///   - Entry data (simplified)
    fn serialize_index(&self, index: &GitIndex) -> crate::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write signature
        buffer.extend_from_slice(b"DIRC");

        // Write version (big-endian)
        buffer.extend_from_slice(&index.version.to_be_bytes());

        // Write number of entries
        let entry_count = index.entries.len() as u32;
        buffer.extend_from_slice(&entry_count.to_be_bytes());

        // Write entries (simplified - just use JSON for each entry)
        for entry in index.get_sorted_entries() {
            let entry_json = serde_json::to_string(entry)?;
            let entry_bytes = entry_json.as_bytes();

            // Write entry length
            let entry_len = entry_bytes.len() as u32;
            buffer.extend_from_slice(&entry_len.to_be_bytes());

            // Write entry data
            buffer.extend_from_slice(entry_bytes);
        }

        Ok(buffer)
    }

    /// Deserialize index from binary format
    fn deserialize_index(&self, data: &[u8]) -> crate::Result<GitIndex> {
        let mut pos = 0;

        // Check signature
        if data.len() < 4 || &data[0..4] != b"DIRC" {
            return Err("Invalid index file signature".into());
        }
        pos += 4;

        // Read version
        if data.len() < pos + 4 {
            return Err("Invalid index file: missing version".into());
        }
        let version = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;

        // Read entry count
        if data.len() < pos + 4 {
            return Err("Invalid index file: missing entry count".into());
        }
        let entry_count =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;

        let mut index = GitIndex::new();
        index.version = version;

        // Read entries
        for _ in 0..entry_count {
            // Read entry length
            if data.len() < pos + 4 {
                return Err("Invalid index file: missing entry length".into());
            }
            let entry_len =
                u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            pos += 4;

            // Read entry data
            if data.len() < pos + entry_len {
                return Err("Invalid index file: truncated entry data".into());
            }
            let entry_bytes = &data[pos..pos + entry_len];
            let entry_json = String::from_utf8(entry_bytes.to_vec())?;
            let entry: IndexEntry = serde_json::from_str(&entry_json)?;

            index.add_entry(entry);
            pos += entry_len;
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::objects::{FileMode, ObjectHash};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_json_index_store() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("index");
        let store = IndexStore::new(index_path);

        // Create test index
        let mut index = GitIndex::new();
        let entry = IndexEntry::new(
            PathBuf::from("test.txt"),
            ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string()),
            13,
            FileMode::Regular,
        );
        index.add_entry(entry);

        // Save and load
        store.save_index(&index).unwrap();
        let loaded_index = store.load_index().unwrap();

        assert_eq!(index.len(), loaded_index.len());
        assert_eq!(index.version, loaded_index.version);
    }

    #[test]
    fn test_binary_index_store() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("index.bin");
        let store = BinaryIndexStore::new(index_path);

        // Create test index
        let mut index = GitIndex::new();
        let entry = IndexEntry::new(
            PathBuf::from("test.txt"),
            ObjectHash::new("abcdef1234567890abcdef1234567890abcdef12".to_string()),
            13,
            FileMode::Regular,
        );
        index.add_entry(entry);

        // Save and load
        store.save_index(&index).unwrap();
        let loaded_index = store.load_index().unwrap();

        assert_eq!(index.len(), loaded_index.len());
        assert_eq!(index.version, loaded_index.version);
    }

    #[test]
    fn test_empty_index() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("empty_index");
        let store = IndexStore::new(index_path);

        // Load non-existent index should return empty index
        let index = store.load_index().unwrap();
        assert!(index.is_empty());
        assert_eq!(index.version, 2);
    }

    #[test]
    fn test_clear_index() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("clear_index");
        let store = IndexStore::new(index_path);

        // Create and save a non-empty index
        let mut index = GitIndex::new();
        let entry = IndexEntry::new(
            PathBuf::from("test.txt"),
            ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string()),
            13,
            FileMode::Regular,
        );
        index.add_entry(entry);
        store.save_index(&index).unwrap();

        // Clear the index
        store.clear_index().unwrap();
        let cleared_index = store.load_index().unwrap();
        assert!(cleared_index.is_empty());
    }
}
