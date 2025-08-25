use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};

use crate::domain::objects::*;

/// Git Object Storage Implementation
/// 
/// This handles the physical storage and retrieval of Git objects in the filesystem.
/// Git stores objects in .git/objects/ directory using the following format:
/// - Objects are compressed using zlib
/// - Object hash determines the storage path: first 2 chars = directory, rest = filename
/// - Object content format: "{type} {size}\0{content}"
pub struct ObjectStore {
    objects_dir: PathBuf,
}

impl ObjectStore {
    /// Create a new object store
    pub fn new(objects_dir: PathBuf) -> Self {
        Self { objects_dir }
    }
    
    /// Initialize the objects directory structure
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.objects_dir)?;
        fs::create_dir_all(self.objects_dir.join("info"))?;
        fs::create_dir_all(self.objects_dir.join("pack"))?;
        Ok(())
    }
    
    /// Store a Git object and return its hash
    pub fn store_object(&self, object: &GitObject) -> crate::Result<ObjectHash> {
        let serialized = self.serialize_object(object)?;
        let hash = self.calculate_hash(&serialized);
        let object_path = self.get_object_path(&hash);
        
        // Create directory if it doesn't exist
        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Compress and write the object
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&serialized)?;
        let compressed = encoder.finish()?;
        
        fs::write(&object_path, compressed)?;
        
        Ok(hash)
    }
    
    /// Retrieve a Git object by its hash
    pub fn load_object(&self, hash: &ObjectHash) -> crate::Result<GitObject> {
        let object_path = self.get_object_path(hash);
        
        if !object_path.exists() {
            return Err(format!("Object {} not found", hash).into());
        }
        
        // Read and decompress the object
        let compressed = fs::read(&object_path)?;
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        self.deserialize_object(&decompressed)
    }
    
    /// Check if an object exists
    pub fn object_exists(&self, hash: &ObjectHash) -> bool {
        self.get_object_path(hash).exists()
    }
    
    /// List all object hashes
    pub fn list_objects(&self) -> crate::Result<Vec<ObjectHash>> {
        let mut objects = Vec::new();
        
        if !self.objects_dir.exists() {
            return Ok(objects);
        }
        
        for entry in fs::read_dir(&self.objects_dir)? {
            let entry = entry?;
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();
            
            // Skip info and pack directories
            if dir_name_str == "info" || dir_name_str == "pack" {
                continue;
            }
            
            // Must be exactly 2 characters for a valid object directory
            if dir_name_str.len() != 2 {
                continue;
            }
            
            let dir_path = entry.path();
            if dir_path.is_dir() {
                for file_entry in fs::read_dir(&dir_path)? {
                    let file_entry = file_entry?;
                    let file_name = file_entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    
                    // Combine directory and file name to get full hash
                    let full_hash = format!("{}{}", dir_name_str, file_name_str);
                    objects.push(ObjectHash::new(full_hash));
                }
            }
        }
        
        Ok(objects)
    }
    
    /// Get the file system path for an object
    fn get_object_path(&self, hash: &ObjectHash) -> PathBuf {
        self.objects_dir
            .join(hash.dir_name())
            .join(hash.file_name())
    }
    
    /// Calculate SHA-1 hash of object content
    fn calculate_hash(&self, content: &[u8]) -> ObjectHash {
        let mut hasher = Sha1::new();
        hasher.update(content);
        let result = hasher.finalize();
        ObjectHash::new(hex::encode(result))
    }
    
    /// Serialize a Git object to bytes
    fn serialize_object(&self, object: &GitObject) -> crate::Result<Vec<u8>> {
        let (object_type, content) = match object {
            GitObject::Blob(blob) => ("blob", blob.content.clone()),
            GitObject::Tree(tree) => ("tree", self.serialize_tree(tree)?),
            GitObject::Commit(commit) => ("commit", self.serialize_commit(commit)?),
        };
        
        let header = format!("{} {}\0", object_type, content.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(&content);
        
        Ok(result)
    }
    
    /// Deserialize bytes to a Git object
    fn deserialize_object(&self, data: &[u8]) -> crate::Result<GitObject> {
        // Find the null terminator that separates header from content
        let null_pos = data.iter().position(|&b| b == 0)
            .ok_or("Invalid object format: no null terminator")?;
        
        let header = String::from_utf8(data[0..null_pos].to_vec())?;
        let content = &data[null_pos + 1..];
        
        // Parse header: "type size"
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid object header format".into());
        }
        
        let object_type = parts[0];
        let size: usize = parts[1].parse()?;
        
        if content.len() != size {
            return Err("Object size mismatch".into());
        }
        
        match object_type {
            "blob" => Ok(GitObject::Blob(BlobObject::new(content.to_vec()))),
            "tree" => Ok(GitObject::Tree(self.deserialize_tree(content)?)),
            "commit" => Ok(GitObject::Commit(self.deserialize_commit(content)?)),
            _ => Err(format!("Unknown object type: {}", object_type).into()),
        }
    }
    
    /// Serialize a tree object
    fn serialize_tree(&self, tree: &TreeObject) -> crate::Result<Vec<u8>> {
        let mut result = Vec::new();
        
        for entry in &tree.entries {
            // Format: "{mode} {name}\0{20-byte-hash}"
            let mode_str = format!("{:o}", entry.mode.as_u32());
            result.extend_from_slice(mode_str.as_bytes());
            result.push(b' ');
            result.extend_from_slice(entry.name.as_bytes());
            result.push(0); // null terminator
            
            // Convert hex hash to binary
            let hash_bytes = hex::decode(&entry.hash.0)?;
            result.extend_from_slice(&hash_bytes);
        }
        
        Ok(result)
    }
    
    /// Deserialize a tree object
    fn deserialize_tree(&self, data: &[u8]) -> crate::Result<TreeObject> {
        let mut tree = TreeObject::new();
        let mut pos = 0;
        
        while pos < data.len() {
            // Find space after mode
            let space_pos = data[pos..].iter().position(|&b| b == b' ')
                .ok_or("Invalid tree format: no space after mode")?;
            
            let mode_str = String::from_utf8(data[pos..pos + space_pos].to_vec())?;
            let mode_num = u32::from_str_radix(&mode_str, 8)?;
            let mode = FileMode::from_u32(mode_num)
                .ok_or(format!("Invalid file mode: {}", mode_num))?;
            
            pos += space_pos + 1; // Skip past space
            
            // Find null terminator after name
            let null_pos = data[pos..].iter().position(|&b| b == 0)
                .ok_or("Invalid tree format: no null after name")?;
            
            let name = String::from_utf8(data[pos..pos + null_pos].to_vec())?;
            pos += null_pos + 1; // Skip past null
            
            // Read 20-byte hash
            if pos + 20 > data.len() {
                return Err("Invalid tree format: truncated hash".into());
            }
            
            let hash_bytes = &data[pos..pos + 20];
            let hash = ObjectHash::new(hex::encode(hash_bytes));
            pos += 20;
            
            tree.add_entry(TreeEntry::new(mode, name, hash));
        }
        
        Ok(tree)
    }
    
    /// Serialize a commit object
    fn serialize_commit(&self, commit: &CommitObject) -> crate::Result<Vec<u8>> {
        let mut result = String::new();
        
        result.push_str(&format!("tree {}\n", commit.tree));
        
        for parent in &commit.parents {
            result.push_str(&format!("parent {}\n", parent));
        }
        
        result.push_str(&format!("author {}\n", commit.author));
        result.push_str(&format!("committer {}\n", commit.committer));
        result.push('\n');
        result.push_str(&commit.message);
        
        Ok(result.into_bytes())
    }
    
    /// Deserialize a commit object
    fn deserialize_commit(&self, data: &[u8]) -> crate::Result<CommitObject> {
        let content = String::from_utf8(data.to_vec())?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut tree: Option<ObjectHash> = None;
        let mut parents = Vec::new();
        let mut author: Option<Signature> = None;
        let mut committer: Option<Signature> = None;
        let mut message_start = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                message_start = i + 1;
                break;
            }
            
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }
            
            match parts[0] {
                "tree" => tree = Some(ObjectHash::new(parts[1].to_string())),
                "parent" => parents.push(ObjectHash::new(parts[1].to_string())),
                "author" => author = Some(self.parse_signature(parts[1])?),
                "committer" => committer = Some(self.parse_signature(parts[1])?),
                _ => {} // Ignore unknown fields
            }
        }
        
        let tree = tree.ok_or("Missing tree in commit")?;
        let author = author.ok_or("Missing author in commit")?;
        let committer = committer.ok_or("Missing committer in commit")?;
        
        let message = if message_start < lines.len() {
            lines[message_start..].join("\n")
        } else {
            String::new()
        };
        
        Ok(CommitObject {
            tree,
            parents,
            author,
            committer,
            message,
        })
    }
    
    /// Parse a signature from "name <email> timestamp timezone" format
    fn parse_signature(&self, sig_str: &str) -> crate::Result<Signature> {
        // Simple parsing - in real implementation, this would be more robust
        let parts: Vec<&str> = sig_str.rsplitn(2, ' ').collect();
        if parts.len() != 2 {
            return Err("Invalid signature format".into());
        }
        
        let timestamp_str = parts[1];
        let name_email = parts[0];
        
        // Parse timestamp
        let timestamp: i64 = timestamp_str.parse()?;
        let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or("Invalid timestamp")?;
        
        // Parse name and email from "Name <email>" format
        if let Some(email_start) = name_email.rfind(" <") {
            let name = name_email[..email_start].to_string();
            let email_part = &name_email[email_start + 2..];
            if let Some(email_end) = email_part.find('>') {
                let email = email_part[..email_end].to_string();
                return Ok(Signature {
                    name,
                    email,
                    timestamp: datetime,
                });
            }
        }
        
        Err("Invalid name/email format".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_store_and_load_blob() {
        let temp_dir = tempdir().unwrap();
        let store = ObjectStore::new(temp_dir.path().join("objects"));
        store.init().unwrap();
        
        let blob = GitObject::Blob(BlobObject::from_string("Hello, World!".to_string()));
        let hash = store.store_object(&blob).unwrap();
        let loaded = store.load_object(&hash).unwrap();
        
        assert_eq!(blob, loaded);
        assert!(store.object_exists(&hash));
    }
    
    #[test]
    fn test_store_and_load_tree() {
        let temp_dir = tempdir().unwrap();
        let store = ObjectStore::new(temp_dir.path().join("objects"));
        store.init().unwrap();
        
        let mut tree = TreeObject::new();
        tree.add_entry(TreeEntry::new(
            FileMode::Regular,
            "file.txt".to_string(),
            ObjectHash::new("1234567890abcdef1234567890abcdef12345678".to_string()),
        ));
        
        let tree_object = GitObject::Tree(tree);
        let hash = store.store_object(&tree_object).unwrap();
        let loaded = store.load_object(&hash).unwrap();
        
        assert_eq!(tree_object, loaded);
    }
    
    #[test]
    fn test_store_and_load_commit() {
        let temp_dir = tempdir().unwrap();
        let store = ObjectStore::new(temp_dir.path().join("objects"));
        store.init().unwrap();
        
        let commit = CommitObject::new(
            ObjectHash::new("abcdef1234567890abcdef1234567890abcdef12".to_string()),
            vec![],
            Signature::new("Test User".to_string(), "test@example.com".to_string()),
            "Initial commit".to_string(),
        );
        
        let commit_object = GitObject::Commit(commit);
        let hash = store.store_object(&commit_object).unwrap();
        let loaded = store.load_object(&hash).unwrap();
        
        assert_eq!(commit_object, loaded);
    }
}
