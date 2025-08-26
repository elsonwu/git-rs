# Git-RS Command Reference 📚

Complete reference for all git-rs commands with educational insights.

## 🚀 `git-rs init`

Initialize a new git-rs repository.

### Syntax

```bash
git-rs init [directory]
```

### What It Does

Creates a new git-rs repository in the current directory (or specified directory), setting up the complete `.git-rs/` directory structure needed for version control.

### Educational Insights

- **Repository Structure**: See exactly how Git organizes its internal data
- **Object Database**: Understanding how Git prepares to store content
- **Reference System**: How branches and HEAD are initialized
- **Configuration**: Default settings that Git repositories need

### Directory Structure Created

```text
.git-rs/
├── objects/          # Content-addressed object database
│   ├── info/        # Object database metadata
│   └── pack/        # Packed objects (future feature)
├── refs/            # Reference storage
│   ├── heads/       # Branch references
│   └── tags/        # Tag references
├── HEAD             # Current branch pointer
├── config           # Repository configuration
└── description      # Repository description
```

### Examples

```bash
# Initialize in current directory
git-rs init

# Initialize in specific directory (future feature)
git-rs init my-project
```

### Internal Process

1. **Validation**: Check if directory is already a repository
2. **Structure Creation**: Create `.git-rs/` and all subdirectories
3. **Object Database**: Initialize empty object store with proper permissions
4. **References**: Create HEAD pointing to `refs/heads/main`
5. **Configuration**: Write default config and description files

---

## 📥 `git-rs add`

Stage files for the next commit.

### Syntax

```bash
git-rs add <file>...
git-rs add <directory>...
git-rs add .
```

### What It Does

Reads file content, creates blob objects in the object database, and updates the staging area (index) to include these files in the next commit.

### Educational Insights

- **Content Addressing**: See how identical content gets the same hash
- **Blob Creation**: Understanding how Git stores file content
- **Index Management**: How the staging area prepares commits
- **Hash Calculation**: SHA-1 content addressing in action

### File Processing

```text
File Content → Blob Object → Object Database → Index Update
"Hello" → "blob 5\0Hello" → SHA-1 hash → .git-rs/objects/aa/bb...
```

### Examples

```bash
# Add single file
git-rs add README.md

# Add multiple files
git-rs add file1.txt file2.txt

# Add directory recursively
git-rs add src/

# Add all files in current directory
git-rs add .
```

### Internal Process

1. **File Resolution**: Expand paths and directories
2. **Content Reading**: Read file bytes from disk
3. **Hash Calculation**: Calculate SHA-1 of "blob <size>\0<content>"
4. **Object Creation**: Store compressed blob in `.git-rs/objects/`
5. **Index Update**: Record file path, hash, and metadata in staging area

### Object Storage Format

```rust
// Blob object format
let content = b"Hello World";
let header = format!("blob {}\0", content.len());
let object = [header.as_bytes(), content].concat();
let hash = sha1::digest(&object);  // 5d41402abc4b2a76b9719d911017c592

// Storage location
// .git-rs/objects/5d/41402abc4b2a76b9719d911017c592 (compressed with zlib)
```

---

## 📊 `git-rs status`

Show the working tree status.

### Syntax

```bash
git-rs status
```

### What It Does

Compares the working directory, staging area (index), and HEAD commit to show what files have been modified, staged, or are untracked.

### Educational Insights

- **Three Trees**: Understanding Git's core data model
- **Content Comparison**: How Git determines file changes
- **Hash-Based Tracking**: Why Git is so efficient at change detection
- **File States**: The complete lifecycle of files in Git

### Status Categories

#### Changes to be committed (Staged)

Files in the staging area that differ from the last commit:

```bash
Changes to be committed:
  new file:   README.md      # New file staged
  modified:   src/main.rs    # Modified file staged  
  deleted:    old_file.txt   # Deleted file staged
```

#### Changes not staged for commit (Modified)

Files in working directory that differ from staging area:

```bash
Changes not staged for commit:
  modified:   README.md      # File modified after staging
  deleted:    old_file.txt   # File deleted after staging
```

#### Untracked files

Files in working directory not in staging area:

```bash
Untracked files:
  new_feature.rs            # New file not yet added
  temp_file.txt            # File not tracked by Git
```

### Examples

```bash
# Show repository status
git-rs status

# Typical output interpretation:
# - Green (staged): Ready for commit
# - Red (modified): Changed since last add
# - Untracked: Not under version control
```

### Internal Algorithm

```text
1. Scan working directory → compute file hashes
2. Load staging area (.git-rs/git-rs-index) → get staged hashes
3. Load HEAD commit → get committed hashes (if any)
4. Compare hashes:
   - staged ≠ committed → "Changes to be committed"
   - working ≠ staged → "Changes not staged"
   - working exists, not in staged → "Untracked"
```

### Status Matrix

| Working | Staged | HEAD | Status |
|---------|--------|------|---------|
| exists  | same   | same | Clean (no output) |
| exists  | same   | none | New file (staged) |
| exists  | same   | diff | Modified (staged) |
| exists  | diff   | same | Modified (unstaged) |
| exists  | none   | none | Untracked |
| exists  | none   | same | Deleted (staged) |
| none    | same   | same | Deleted (unstaged) |

---

## � `git-rs commit`

Create a new commit from the staged changes.

### Syntax

```bash
git-rs commit -m "<message>"
git-rs commit --message "<message>"
```

### Parameters

- `-m, --message <MSG>` - Commit message (required)

### What It Does

Creates a permanent snapshot of all staged changes, building a complete directory tree and updating the current branch reference to point to the new commit.

### Educational Insights

- **Tree Object Creation**: How Git builds directory trees from the staging area
- **Commit Object Format**: Understanding Git's commit data structure
- **Reference Management**: How branch pointers move to track history
- **SHA-1 Chaining**: How commits form a linked list through parent references
- **Atomic Operations**: Why commits are all-or-nothing snapshots

### Internal Process

1. **Validation**: Ensure staging area has changes and message is provided
2. **Tree Building**: Create tree objects recursively from staged files
3. **Commit Creation**: Build commit object with metadata and tree hash
4. **Object Storage**: Store commit object with SHA-1 hash
5. **Reference Update**: Move current branch HEAD to new commit

### Examples

```bash
# Commit staged changes with a message
git-rs commit -m "Add user authentication system"

# After staging files:
git-rs add src/auth.rs
git-rs add tests/auth_tests.rs
git-rs commit -m "Implement secure password hashing"
```

### Commit Object Format

```rust
// Example commit object structure
let commit_content = format!(
    "tree {}\nparent {}\nauthor {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
    tree_hash,          // Root tree SHA-1
    parent_hash,        // Previous commit SHA-1
    author_name, author_email, timestamp,
    committer_name, committer_email, timestamp,
    commit_message
);

// Object format: "commit <size>\0<content>"
// Storage: .git-rs/objects/ab/cdef123... (compressed)
```

### Tree Structure

```text
# Example tree built from staging area:
100644 blob a1b2c3d4  README.md
040000 tree e5f6g7h8  src/
  100644 blob i9j0k1l2    main.rs
  100644 blob m3n4o5p6    lib.rs
040000 tree q7r8s9t0  tests/
  100644 blob u1v2w3x4    integration_tests.rs
```

### Reference Updates

```bash
# Before commit
cat .git-rs/HEAD           # ref: refs/heads/main
cat .git-rs/refs/heads/main  # 1234567890abcdef... (parent commit)

# After commit  
cat .git-rs/refs/heads/main  # abcdef1234567890... (new commit)
```

### Error Conditions

- **No staged changes**: "Nothing to commit, working tree clean"
- **Missing message**: "Please provide a commit message using -m"
- **Repository not initialized**: "Not a git repository"

---

## � `git-rs diff`

Show changes between different states in unified diff format.

### Syntax

```bash
git-rs diff [--cached]
```

### What It Does

Displays line-by-line differences between:

- **Default**: Working directory vs staging area
- **--cached**: Staging area vs last commit

### Educational Insights

- **Unified Diff Format**: Standard format used by patch tools
- **Content Comparison**: How Git compares file contents
- **Binary Detection**: Different handling for binary files
- **Change Visualization**: Understanding what lines changed

### Options

- `--cached`: Show differences between staging area and last commit
- Default (no flag): Show differences between working directory and staging area

### Output Format

```diff
diff --git a/file.txt b/file.txt
index 1234567..abcdefg 100644
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line 1
-old line 2
+new line 2
 line 3
```

### Examples

```bash
# Show unstaged changes
git-rs diff

# Show staged changes
git-rs diff --cached

# After modifying a file
echo "new content" >> file.txt
git-rs diff  # Shows the addition
```

### Educational Value

- **Understanding Diffs**: How version control systems represent changes
- **Patch Format**: Standard format for sharing changes
- **Change Detection**: Algorithms for comparing file content

---

## 🚧 Future Commands (In Development)

### `git-rs clone`

Copy a repository from remote location.

- **Object Transfer**: Download all objects from remote
- **Reference Mapping**: Set up local branches
- **Working Directory**: Populate files from HEAD commit

---

## 🎓 Educational Features

### Hash Exploration

```bash
# After adding a file, explore the object:
git-rs add hello.txt

# Find the object (hash will be displayed during add)
find .git-rs/objects -name "*" -type f

# Examine object structure (requires zlib tools)
zpipe -d < .git-rs/objects/5d/41402abc... | hexdump -C
```

### Index Inspection

```bash
# View staging area (our index is JSON for readability)
cat .git-rs/git-rs-index | jq .

# See what's staged
jq '.entries | keys[]' .git-rs/git-rs-index
```

### Reference Tracking

```bash
# See current branch
cat .git-rs/HEAD

# List all branches (when implemented)
find .git-rs/refs/heads -type f
```

---

## 🔍 Troubleshooting

### "Not a git repository"

```bash
# Solution: Initialize repository first
git-rs init
```

### "Path not within repository"

```bash
# Solution: Run commands from within repository root
cd /path/to/repository
git-rs status
```

### "Permission denied"

```bash
# Solution: Check file/directory permissions
ls -la .git-rs/
chmod -R u+w .git-rs/
```

---

## 💡 Tips for Learning

1. **Compare with Real Git**: Run both `git status` and `git-rs status` to see differences
2. **Explore Objects**: Use the hash debugging tips to examine object storage
3. **Watch Index Changes**: Monitor `.git-rs/git-rs-index` as you add files
4. **Understand Hashes**: Same content = same hash across all repositories
5. **Three Trees**: Always think Working → Index → HEAD when analyzing status

---

## 📖 Related Documentation

- [README.md](README.md) - Project overview and architecture
- [ARCHITECTURE.md](ARCHITECTURE.md) - Deep dive into Git internals
- [Git Internals Book](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects) - Official Git documentation

Each command in git-rs is designed to be educational first, functional second. The goal is to understand how Git works internally, not to replace Git for production use.
