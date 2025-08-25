use serde::{Deserialize, Serialize};
use crate::domain::objects::ObjectHash;

/// Reference types in Git
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RefType {
    /// Branch reference (refs/heads/*)
    Branch,
    /// Tag reference (refs/tags/*)
    Tag,
    /// Remote branch reference (refs/remotes/*)
    RemoteBranch,
}

/// A Git reference pointing to a commit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitRef {
    pub name: String,
    pub hash: ObjectHash,
    pub ref_type: RefType,
}

impl GitRef {
    pub fn new(name: String, hash: ObjectHash, ref_type: RefType) -> Self {
        Self { name, hash, ref_type }
    }
    
    /// Get the full reference path (e.g., "refs/heads/main")
    pub fn full_name(&self) -> String {
        match self.ref_type {
            RefType::Branch => format!("refs/heads/{}", self.name),
            RefType::Tag => format!("refs/tags/{}", self.name),
            RefType::RemoteBranch => format!("refs/remotes/{}", self.name),
        }
    }
    
    /// Create a branch reference
    pub fn branch(name: String, hash: ObjectHash) -> Self {
        Self::new(name, hash, RefType::Branch)
    }
    
    /// Create a tag reference
    pub fn tag(name: String, hash: ObjectHash) -> Self {
        Self::new(name, hash, RefType::Tag)
    }
}

/// HEAD reference - points to the current branch or commit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HeadRef {
    /// Symbolic reference to a branch (normal case)
    Symbolic(String), // e.g., "refs/heads/main"
    /// Direct reference to a commit (detached HEAD)
    Direct(ObjectHash),
}

impl HeadRef {
    /// Create a symbolic HEAD reference to a branch
    pub fn symbolic(branch_name: &str) -> Self {
        Self::Symbolic(format!("refs/heads/{}", branch_name))
    }
    
    /// Create a direct HEAD reference to a commit
    pub fn direct(hash: ObjectHash) -> Self {
        Self::Direct(hash)
    }
    
    /// Check if HEAD is detached (pointing directly to a commit)
    pub fn is_detached(&self) -> bool {
        matches!(self, HeadRef::Direct(_))
    }
    
    /// Get the branch name if HEAD is symbolic
    pub fn branch_name(&self) -> Option<&str> {
        match self {
            HeadRef::Symbolic(ref_name) => {
                ref_name.strip_prefix("refs/heads/")
            }
            HeadRef::Direct(_) => None,
        }
    }
    
    /// Get the commit hash this HEAD points to
    /// Note: For symbolic references, you need to resolve the branch first
    pub fn direct_hash(&self) -> Option<&ObjectHash> {
        match self {
            HeadRef::Direct(hash) => Some(hash),
            HeadRef::Symbolic(_) => None,
        }
    }
}

impl std::fmt::Display for HeadRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadRef::Symbolic(ref_name) => write!(f, "ref: {}", ref_name),
            HeadRef::Direct(hash) => write!(f, "{}", hash),
        }
    }
}

/// Reference manager for handling Git references
#[derive(Debug, Clone)]
pub struct ReferenceManager {
    pub refs: Vec<GitRef>,
    pub head: Option<HeadRef>,
}

impl ReferenceManager {
    pub fn new() -> Self {
        Self {
            refs: Vec::new(),
            head: None,
        }
    }
    
    /// Add a new reference
    pub fn add_ref(&mut self, git_ref: GitRef) {
        // Remove existing reference with the same full name
        self.refs.retain(|r| r.full_name() != git_ref.full_name());
        self.refs.push(git_ref);
    }
    
    /// Find a reference by name
    pub fn find_ref(&self, name: &str) -> Option<&GitRef> {
        self.refs.iter().find(|r| r.name == name || r.full_name() == name)
    }
    
    /// Get all branch references
    pub fn branches(&self) -> Vec<&GitRef> {
        self.refs.iter()
            .filter(|r| r.ref_type == RefType::Branch)
            .collect()
    }
    
    /// Get all tag references
    pub fn tags(&self) -> Vec<&GitRef> {
        self.refs.iter()
            .filter(|r| r.ref_type == RefType::Tag)
            .collect()
    }
    
    /// Set HEAD to point to a branch
    pub fn set_head_to_branch(&mut self, branch_name: &str) {
        self.head = Some(HeadRef::symbolic(branch_name));
    }
    
    /// Set HEAD to point directly to a commit (detached HEAD)
    pub fn set_head_to_commit(&mut self, hash: ObjectHash) {
        self.head = Some(HeadRef::direct(hash));
    }
    
    /// Get the current HEAD reference
    pub fn get_head(&self) -> Option<&HeadRef> {
        self.head.as_ref()
    }
    
    /// Resolve HEAD to get the actual commit hash
    pub fn resolve_head(&self) -> Option<ObjectHash> {
        match &self.head {
            Some(HeadRef::Direct(hash)) => Some(hash.clone()),
            Some(HeadRef::Symbolic(ref_name)) => {
                self.find_ref(ref_name)
                    .map(|r| r.hash.clone())
            }
            None => None,
        }
    }
}

impl Default for ReferenceManager {
    fn default() -> Self {
        Self::new()
    }
}
