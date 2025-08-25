use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Git Object Hash - A 40-character hexadecimal string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectHash(pub String);

impl ObjectHash {
    /// Create a new object hash from a string
    pub fn new(hash: String) -> Self {
        Self(hash)
    }
    
    /// Get the hash as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the first 2 characters for directory name
    pub fn dir_name(&self) -> &str {
        &self.0[0..2]
    }
    
    /// Get the remaining characters for file name
    pub fn file_name(&self) -> &str {
        &self.0[2..]
    }
}

impl std::fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Git Object Types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitObjectType {
    Blob,
    Tree,
    Commit,
}

impl std::fmt::Display for GitObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitObjectType::Blob => write!(f, "blob"),
            GitObjectType::Tree => write!(f, "tree"),
            GitObjectType::Commit => write!(f, "commit"),
        }
    }
}

/// A Git Blob object represents file content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlobObject {
    pub content: Vec<u8>,
}

impl BlobObject {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
    
    pub fn from_string(content: String) -> Self {
        Self {
            content: content.into_bytes(),
        }
    }
    
    pub fn content_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }
    
    pub fn size(&self) -> usize {
        self.content.len()
    }
}

/// File mode constants (similar to Unix file permissions)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FileMode {
    /// Regular file
    Regular = 0o100644,
    /// Executable file
    Executable = 0o100755,
    /// Symbolic link
    Symlink = 0o120000,
    /// Directory (tree)
    Directory = 0o040000,
}

impl FileMode {
    pub fn from_u32(mode: u32) -> Option<Self> {
        match mode {
            0o100644 => Some(FileMode::Regular),
            0o100755 => Some(FileMode::Executable),
            0o120000 => Some(FileMode::Symlink),
            0o040000 => Some(FileMode::Directory),
            _ => None,
        }
    }
    
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// A Tree Entry represents a file or subdirectory in a tree object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub mode: FileMode,
    pub name: String,
    pub hash: ObjectHash,
}

impl TreeEntry {
    pub fn new(mode: FileMode, name: String, hash: ObjectHash) -> Self {
        Self { mode, name, hash }
    }
}

/// A Git Tree object represents a directory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeObject {
    pub entries: Vec<TreeEntry>,
}

impl TreeObject {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    pub fn add_entry(&mut self, entry: TreeEntry) {
        self.entries.push(entry);
        // Keep entries sorted by name
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
    }
    
    pub fn find_entry(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }
}

impl Default for TreeObject {
    fn default() -> Self {
        Self::new()
    }
}

/// Signature for author/committer information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

impl Signature {
    pub fn new(name: String, email: String) -> Self {
        Self {
            name,
            email,
            timestamp: Utc::now(),
        }
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} <{}> {}",
            self.name,
            self.email,
            self.timestamp.timestamp()
        )
    }
}

/// A Git Commit object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitObject {
    pub tree: ObjectHash,
    pub parents: Vec<ObjectHash>,
    pub author: Signature,
    pub committer: Signature,
    pub message: String,
}

impl CommitObject {
    pub fn new(
        tree: ObjectHash,
        parents: Vec<ObjectHash>,
        author: Signature,
        message: String,
    ) -> Self {
        let committer = author.clone();
        Self {
            tree,
            parents,
            author,
            committer,
            message,
        }
    }
    
    pub fn is_root_commit(&self) -> bool {
        self.parents.is_empty()
    }
}

/// A Git Object that can be stored in the object database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitObject {
    Blob(BlobObject),
    Tree(TreeObject),
    Commit(CommitObject),
}

impl GitObject {
    pub fn object_type(&self) -> GitObjectType {
        match self {
            GitObject::Blob(_) => GitObjectType::Blob,
            GitObject::Tree(_) => GitObjectType::Tree,
            GitObject::Commit(_) => GitObjectType::Commit,
        }
    }
    
    pub fn as_blob(&self) -> Option<&BlobObject> {
        match self {
            GitObject::Blob(blob) => Some(blob),
            _ => None,
        }
    }
    
    pub fn as_tree(&self) -> Option<&TreeObject> {
        match self {
            GitObject::Tree(tree) => Some(tree),
            _ => None,
        }
    }
    
    pub fn as_commit(&self) -> Option<&CommitObject> {
        match self {
            GitObject::Commit(commit) => Some(commit),
            _ => None,
        }
    }
}
