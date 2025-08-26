use std::collections::HashMap;
use url::Url;

/// Represents a remote Git repository
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRepository {
    /// The URL of the remote repository
    pub url: Url,
    /// The name of the remote (typically "origin")
    pub name: String,
    /// Available references from the remote
    pub refs: HashMap<String, String>,
}

/// A reference from a remote repository
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRef {
    /// Reference name (e.g., "refs/heads/main")
    pub name: String,
    /// SHA-1 hash the reference points to
    pub hash: String,
    /// Whether this is a symbolic reference
    pub is_symbolic: bool,
}

/// Represents a Git pack file
#[derive(Debug)]
pub struct PackFile {
    /// Pack file header
    pub header: PackHeader,
    /// Objects in the pack file
    pub objects: Vec<PackObject>,
}

/// Pack file header information
#[derive(Debug, Clone)]
pub struct PackHeader {
    /// Pack file version (should be 2)
    pub version: u32,
    /// Number of objects in the pack
    pub object_count: u32,
}

/// A single object in a pack file
#[derive(Debug, Clone)]
pub struct PackObject {
    /// Object type (commit, tree, blob, tag)
    pub object_type: PackObjectType,
    /// Uncompressed size of the object
    pub size: u64,
    /// Object data (compressed)
    pub data: Vec<u8>,
    /// SHA-1 hash of the object
    pub hash: Option<String>,
}

/// Types of objects in a pack file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackObjectType {
    /// A commit object
    Commit = 1,
    /// A tree object  
    Tree = 2,
    /// A blob object
    Blob = 3,
    /// A tag object
    Tag = 4,
    /// Delta against another object in the pack
    OfsDelta = 6,
    /// Delta against an object referenced by SHA-1
    RefDelta = 7,
}

impl RemoteRepository {
    /// Create a new remote repository
    pub fn new(url: Url, name: String) -> Self {
        Self {
            url,
            name,
            refs: HashMap::new(),
        }
    }

    /// Add a reference to the remote
    pub fn add_ref(&mut self, name: String, hash: String) {
        self.refs.insert(name, hash);
    }

    /// Get the HEAD reference hash
    pub fn head_hash(&self) -> Option<&String> {
        // Try different common HEAD reference names
        self.refs.get("HEAD")
            .or_else(|| self.refs.get("refs/heads/main"))
            .or_else(|| self.refs.get("refs/heads/master"))
    }

    /// Get default branch name (main or master)
    pub fn default_branch(&self) -> Option<String> {
        if self.refs.contains_key("refs/heads/main") {
            Some("main".to_string())
        } else if self.refs.contains_key("refs/heads/master") {
            Some("master".to_string())
        } else {
            // Find first branch
            self.refs.keys()
                .find(|name| name.starts_with("refs/heads/"))
                .map(|name| name.strip_prefix("refs/heads/").unwrap().to_string())
        }
    }
}

impl From<u8> for PackObjectType {
    fn from(value: u8) -> Self {
        match value {
            1 => PackObjectType::Commit,
            2 => PackObjectType::Tree,
            3 => PackObjectType::Blob,
            4 => PackObjectType::Tag,
            6 => PackObjectType::OfsDelta,
            7 => PackObjectType::RefDelta,
            _ => PackObjectType::Blob, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_repository_creation() {
        let url = Url::parse("https://github.com/user/repo.git").unwrap();
        let remote = RemoteRepository::new(url.clone(), "origin".to_string());
        
        assert_eq!(remote.url, url);
        assert_eq!(remote.name, "origin");
        assert!(remote.refs.is_empty());
    }

    #[test]
    fn test_remote_ref_management() {
        let url = Url::parse("https://github.com/user/repo.git").unwrap();
        let mut remote = RemoteRepository::new(url, "origin".to_string());
        
        remote.add_ref("refs/heads/main".to_string(), "abc123".to_string());
        remote.add_ref("refs/heads/dev".to_string(), "def456".to_string());
        
        assert_eq!(remote.refs.len(), 2);
        assert_eq!(remote.refs.get("refs/heads/main"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_default_branch_detection() {
        let url = Url::parse("https://github.com/user/repo.git").unwrap();
        let mut remote = RemoteRepository::new(url, "origin".to_string());
        
        // Test main branch
        remote.add_ref("refs/heads/main".to_string(), "abc123".to_string());
        assert_eq!(remote.default_branch(), Some("main".to_string()));
        
        // Test fallback to master
        remote.refs.clear();
        remote.add_ref("refs/heads/master".to_string(), "def456".to_string());
        assert_eq!(remote.default_branch(), Some("master".to_string()));
        
        // Test first branch fallback
        remote.refs.clear();
        remote.add_ref("refs/heads/develop".to_string(), "ghi789".to_string());
        assert_eq!(remote.default_branch(), Some("develop".to_string()));
    }

    #[test]
    fn test_pack_object_type_conversion() {
        assert_eq!(PackObjectType::from(1), PackObjectType::Commit);
        assert_eq!(PackObjectType::from(2), PackObjectType::Tree);
        assert_eq!(PackObjectType::from(3), PackObjectType::Blob);
        assert_eq!(PackObjectType::from(4), PackObjectType::Tag);
        assert_eq!(PackObjectType::from(6), PackObjectType::OfsDelta);
        assert_eq!(PackObjectType::from(7), PackObjectType::RefDelta);
        assert_eq!(PackObjectType::from(99), PackObjectType::Blob); // Unknown defaults to blob
    }
}
