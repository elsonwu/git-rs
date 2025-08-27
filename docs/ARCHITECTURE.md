# Git-RS Internals Documentation 🧠

This document provides a deep dive into Git's internal mechanisms as implemented in git-rs.

## 📂 Repository Structure

Git-rs supports two directory structure modes for different use cases:

### Educational Mode (Default): `.git-rs/`

Safe for learning - uses `.git-rs/` to avoid conflicts with real Git repositories:

```text
.git-rs/
├── objects/              # Object database (content-addressed storage)
│   ├── 5a/
│   │   └── 1b2c3d4e...  # Blob object (file content)
│   ├── ab/
│   │   └── cd1234ef...  # Tree object (directory listing)
│   ├── fe/
│   │   └── dcba9876...  # Commit object (snapshot + metadata)
│   ├── info/            # Object database metadata
│   └── pack/            # Packed objects (future feature)
├── refs/                # Reference storage
│   ├── heads/          # Branch references
│   │   ├── main        # Contains: commit hash
│   │   └── feature-x   # Contains: commit hash  
│   └── tags/           # Tag references
│       └── v1.0        # Contains: commit hash
├── HEAD                 # Current branch pointer
├── git-rs-index        # Staging area (JSON format)
├── config              # Repository configuration
└── description         # Repository description
```

### Git Compatibility Mode: `.git/`

Activated with `--git-compat` flag - uses standard Git structure for interoperability:

```text
.git/
├── objects/              # Same object database structure
│   ├── 5a/
│   │   └── 1b2c3d4e...  # Identical object format
│   └── ...              # Same as educational mode
├── refs/                # Same reference structure
│   ├── heads/          
│   └── tags/           
├── HEAD                 # Same HEAD format
├── index               # Standard Git index name
├── config              # Same configuration format
└── description         # Same description format
```

### Mode Selection

| Command | Directory Created | Index File | Use Case |
|---------|-------------------|------------|----------|
| `git-rs init` | `.git-rs/` | `git-rs-index` | Safe learning |
| `git-rs --git-compat init` | `.git/` | `index` | Git compatibility testing |

## 🎯 Object Model

Git stores everything as objects in a content-addressed database:

### Blob Objects (File Content)

```text
Format: "blob <size>\0<content>"
Example: "blob 11\0Hello World"
SHA-1: 5d41402abc4b2a76b9719d911017c592
Storage: .git-rs/objects/5d/41402abc4b2a76b9719d911017c592
```

### Tree Objects (Directory Listings)

```text
Format: "tree <size>\0<entries>"
Entry: "<mode> <name>\0<20-byte-sha>"
Example: "tree 37\0100644 hello.txt\0[20-byte-hash]"
```

### Commit Objects (Snapshots)

```text
Format: "commit <size>\0<content>"
Content:
tree <tree-hash>
parent <parent-hash>  # (optional, for non-initial commits)
author <name> <email> <timestamp> <timezone>
committer <name> <email> <timestamp> <timezone>

<commit message>
```

## 🔄 Three Trees Concept

Git manages content through three main areas:

### 1. Working Directory

- Your actual files on disk
- What you see and edit
- Can contain untracked files

### 2. Staging Area (Index)

- Snapshot of what will go into the next commit
- Stored in `.git-rs/git-rs-index` (JSON format in our implementation)
- Acts as a buffer between working directory and repository

### 3. Repository (HEAD)

- The last committed snapshot
- Stored as commit objects in `.git-rs/objects/`
- Referenced by branch pointers in `.git-rs/refs/heads/`

### State Transitions

```text
Working Directory ──add──▶ Staging Area ──commit──▶ Repository
      ▲                                                 │
      └─────────────── checkout ─────────────────────┘
```

## 📋 Index Format

Our implementation uses JSON for educational clarity:

```json
{
  "entries": {
    "README.md": {
      "hash": "5d41402abc4b2a76b9719d911017c592",
      "mode": "100644",
      "size": 11,
      "ctime": 1692000000,
      "mtime": 1692000000
    },
    "src/main.rs": {
      "hash": "a1b2c3d4e5f6789012345678901234567890abcd",
      "mode": "100644", 
      "size": 245,
      "ctime": 1692000100,
      "mtime": 1692000100
    }
  },
  "version": 1
}
```

**File Modes:**

- `100644`: Regular file
- `100755`: Executable file
- `120000`: Symbolic link
- `040000`: Directory (tree)

## 🔗 Reference System

References are human-readable names pointing to objects:

### HEAD

- Points to current branch
- Content: `ref: refs/heads/main`
- Special case: detached HEAD contains commit hash directly

### Branches

- Stored in `.git-rs/refs/heads/`
- Each file contains a commit hash
- Example: `.git-rs/refs/heads/main` → `a1b2c3d4...`

### Tags

- Stored in `.git-rs/refs/tags/`
- Point to commit objects (or tag objects for annotated tags)
- Example: `.git-rs/refs/tags/v1.0` → `e5f6g7h8...`

## 🧮 Hash Calculation

Git uses SHA-1 for content addressing:

### Blob Hash Calculation

```rust
fn calculate_blob_hash(content: &[u8]) -> String {
    let header = format!("blob {}\0", content.len());
    let full_content = [header.as_bytes(), content].concat();
    sha1::digest(&full_content)
}
```

### Tree Hash Calculation

```rust
fn calculate_tree_hash(entries: &[(String, String, String)]) -> String {
    let mut content = Vec::new();
    for (mode, name, hash) in entries {
        content.extend_from_slice(mode.as_bytes());
        content.push(b' ');
        content.extend_from_slice(name.as_bytes());
        content.push(b'\0');
        content.extend_from_slice(&hex::decode(hash).unwrap());
    }
    let header = format!("tree {}\0", content.len());
    let full_content = [header.as_bytes(), &content].concat();
    sha1::digest(&full_content)
}
```

## 📊 Status Determination Algorithm

How git-rs determines file status:

```text
1. Scan working directory → get current file hashes
2. Load staging area → get staged file hashes  
3. Load HEAD commit → get committed file hashes
4. Compare:
   - staged_hash != committed_hash → "Changes to be committed"
   - working_hash != staged_hash → "Changes not staged for commit"  
   - working_exists && !staged_exists → "Untracked files"
   - !working_exists && staged_exists → "deleted"
```

### Status Matrix

| Working | Staged | HEAD | Status |
|---------|--------|------|---------|
| A       | A      | A    | Clean |
| A       | A      | -    | New file (staged) |
| A       | A      | B    | Modified (staged) |
| A       | B      | B    | Modified (unstaged) |
| A       | -      | -    | Untracked |
| A       | -      | B    | Deleted (staged) |
| -       | A      | A    | Deleted (unstaged) |

## 🗜️ Object Storage Details

### Compression

Objects are compressed using zlib deflate:

```rust
use flate2::{Compress, Compression};

fn compress_object(content: &[u8]) -> Result<Vec<u8>> {
    let mut compressor = Compress::new(Compression::default(), false);
    let mut output = Vec::new();
    compressor.compress_vec(content, &mut output, flate2::FlushCompress::Finish)?;
    Ok(output)
}
```

### Directory Structure

Objects are stored with first 2 hex digits as directory name:

- Hash: `a1b2c3d4e5f6...`
- Path: `.git-rs/objects/a1/b2c3d4e5f6...`

This prevents having too many files in one directory.

## 🔍 Educational Insights

### Why Content Addressing?

1. **Deduplication**: Identical content stored only once
2. **Integrity**: Corruption changes hash, detectable
3. **Distributed**: Objects transferable between repositories
4. **Immutability**: Objects never change, only referenced

### Why Three Trees?

1. **Flexibility**: Stage partial changes
2. **Safety**: Review before committing
3. **Efficiency**: Only stage what changed
4. **Workflows**: Support complex merge scenarios

### Why SHA-1 (historically)?

1. **Collision resistance**: Extremely unlikely for different content
2. **Performance**: Fast to calculate
3. **Fixed size**: Always 40 hex characters
4. **Distributed**: Works across different systems

## 🚀 Implementation Benefits

Our educational implementation:

- **Uses JSON** for index (readable vs binary)
- **Comprehensive logging** shows internal operations
- **Separate directory** (`.git-rs/`) avoids conflicts
- **Domain-driven design** separates concerns clearly
- **Extensive testing** verifies behavior

## 🔬 Debugging Git Internals

### Inspect Objects

```bash
# Find all objects
find .git-rs/objects -type f

# Examine object (compressed)
hexdump -C .git-rs/objects/5d/41402abc4b2a76b9719d911017c592

# Decompress object (requires zpipe or similar)
zpipe -d < .git-rs/objects/5d/41402abc... | hexdump -C
```

### Inspect Index

```bash
# View staging area
cat .git-rs/git-rs-index | jq .

# Pretty print
jq '.entries | keys[]' .git-rs/git-rs-index
```

### Inspect References

```bash
# Current branch
cat .git-rs/HEAD

# All branches
find .git-rs/refs/heads -type f -exec echo {} \; -exec cat {} \;

# Branch content
cat .git-rs/refs/heads/main
```

## 🎯 Next Steps for Learning

1. **Implement log command**: Display commit history and graph traversal
2. **Branch operations**: Create, switch, merge branches  
3. **Enhanced remote operations**: Push, fetch, pull
4. **Advanced features**: Rebasing, cherry-picking, submodules
5. **Performance optimization**: Pack files, delta compression
