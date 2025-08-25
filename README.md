# Git-RS: Educational Git Implementation 🦀

A minimal Git implementation in Rust designed for learning Git internals and understanding how version control systems work under the hood.

## 🎯 Project Goals

This project implements core Git functionality from scratch to understand:

- How Git stores objects (blobs, trees, commits) using content-addressed storage
- How the staging area (index) works as a three-way merge preparation
- How references and branches are managed through the filesystem
- Git's object model and SHA-1 hash-based storage system
- How Git tracks file changes across working directory, staging area, and commits

## ⚠️ Important: Educational Repository Structure

**For learning purposes, this implementation uses `.git-rs/` instead of `.git/` to avoid conflicts with actual Git repositories.**

When you run `git-rs init`, it creates:

- `.git-rs/` directory (not `.git/`)
- `.git-rs/git-rs-index` file (not `.git/index`)

This allows you to:

- Run git-rs commands in existing Git repositories without conflicts
- Compare git-rs behavior with real Git side-by-side
- Learn safely without affecting your actual Git workflow

## 🏗️ Architecture (Domain-Driven Design)

This project follows DDD principles with clean separation of concerns:

```text
src/
├── main.rs              # CLI entry point with clap
├── lib.rs               # Library exports and error handling
├── domain/              # 🧠 Core business logic
│   ├── repository.rs    # Repository aggregate root
│   ├── objects.rs       # Git objects (Blob, Tree, Commit)
│   ├── references.rs    # HEAD, branches, tags
│   └── index.rs         # Staging area model
├── infrastructure/      # 💾 Persistence layer
│   ├── object_store.rs  # File-based object database
│   ├── ref_store.rs     # Reference file management
│   └── index_store.rs   # Index file serialization
├── application/         # 🎯 Use cases (commands)
│   ├── init.rs          # ✅ Repository initialization
│   ├── add.rs           # ✅ File staging
│   ├── status.rs        # ✅ Working tree status
│   ├── commit.rs        # 🚧 Commit creation
│   ├── diff.rs          # 🚧 Content comparison
│   └── clone.rs         # 🚧 Repository cloning
└── cli/                 # 🖥️ Command line interface
    └── commands.rs      # Command handlers and user interaction
```

**Layer Responsibilities:**

- **Domain**: Pure business logic, no I/O dependencies
- **Infrastructure**: File system operations, serialization
- **Application**: Orchestrates domain and infrastructure
- **CLI**: User interface and command parsing

## 📊 Git Internals: Visual Guide

### Repository Structure (.git-rs/)

```text
.git-rs/
├── objects/              # Content-addressed object database
│   ├── 5a/
│   │   └── 1b2c3d...    # Blob object (file content)
│   ├── ab/
│   │   └── cd1234...    # Tree object (directory listing)
│   └── ef/
│       └── 567890...    # Commit object (snapshot + metadata)
├── refs/                 # Reference storage
│   ├── heads/           # Branch references
│   │   ├── main         # Contains: "5abc123def..."
│   │   └── feature-x    # Contains: "7def456ghi..."
│   └── tags/            # Tag references
├── HEAD                  # Current branch pointer
├── git-rs-index         # Staging area (JSON format)
├── config               # Repository configuration
└── description          # Repository description
```

### Object Storage Model

```text
Working Directory  →  Staging Area  →  Repository
     (files)           (git-rs-index)    (objects/)
        │                    │              │
        │── git add ─────────▶              │
        │                    │── commit ───▶
        │◀────────── checkout ──────────────│
```

### Hash-Based Object System

```text
Object Content → SHA-1 Hash → Storage Path
"Hello World"  → a1b2c3...  → .git-rs/objects/a1/b2c3...

Object Format:
"<type> <size>\0<content>"
"blob 11\0Hello World"
```

## 🔧 Implemented Commands

### ✅ `git-rs init` - Repository Initialization

**What it does:**

- Creates `.git-rs/` directory structure
- Initializes object database with proper subdirectories
- Sets up reference system (HEAD pointing to refs/heads/main)
- Creates configuration files

**Educational Insights:**

- How Git creates a repository from scratch
- Directory structure and file organization
- Reference initialization and HEAD management

**Example:**

```bash
git-rs init
# Creates: .git-rs/{objects,refs/{heads,tags},HEAD,config,description}
```

### ✅ `git-rs add` - File Staging

**What it does:**

- Reads file content and calculates SHA-1 hash
- Creates blob objects in object database with zlib compression
- Updates staging area (git-rs-index) with file paths and hashes
- Handles multiple files and directory recursion

**Educational Insights:**

- Content-addressed storage: identical content = same hash = same object
- How Git tracks file changes through content hashing
- The role of the staging area in preparing commits
- Object creation and compression techniques

**Example:**

```bash
git-rs add README.md src/
# Creates blob objects and updates git-rs-index
```

**Internal Process:**

1. Read file content: `"Hello World"`
2. Create blob: `"blob 11\0Hello World"`
3. Calculate hash: `SHA-1("blob 11\0Hello World") = 5ab2c3d...`
4. Store compressed object: `.git-rs/objects/5a/b2c3d...`
5. Update index: `{"README.md": {"hash": "5ab2c3d...", ...}}`

### ✅ `git-rs status` - Working Tree Status

**What it does:**

- Compares working directory against staging area and last commit
- Categorizes file changes: staged, modified, deleted, untracked
- Shows current branch and commit information
- Respects `.gitignore` patterns

**Educational Insights:**

- Git's "three trees" concept: working directory, index, HEAD
- How Git determines file status through hash comparison
- The relationship between different file states
- Gitignore pattern matching

**Status Categories:**

```text
Changes to be committed:     # In index, different from HEAD
  new file:   README.md
  modified:   src/main.rs

Changes not staged:          # In working dir, different from index  
  modified:   README.md
  deleted:    old_file.txt

Untracked files:            # In working dir, not in index
  new_feature.rs
```

**Example:**

```bash
git-rs status
# Shows comprehensive file state analysis
```

## 🧮 Hash Calculation Deep Dive

Git uses SHA-1 content addressing for all objects:

```rust
// Object format: "<type> <size>\0<content>"
let blob_content = b"Hello World";
let object_content = format!("blob {}\0", blob_content.len());
let full_content = [object_content.as_bytes(), blob_content].concat();
let hash = sha1::digest(&full_content); // "5ab2c3d4e5f6..."
```

**Why this matters:**

- Identical content produces identical hashes
- Deduplication: same file content stored only once
- Integrity: any corruption changes the hash
- Distributed: objects can be safely shared between repositories

## 🚧 Commands in Development

### 🔄 `git-rs commit` - Commit Creation

- Tree object creation from staging area
- Commit object with metadata (author, timestamp, message)
- Reference updates (branch pointer advancement)

### 📊 `git-rs diff` - Content Comparison

- Unified diff format generation
- Working directory vs staging area comparison
- Staged vs committed comparison

### 📥 `git-rs clone` - Repository Cloning

- Remote repository communication
- Object transfer and verification
- Reference mapping and checkout

## 🧪 Testing Strategy

Our test suite covers:

- **Unit tests**: Individual component behavior
- **Integration tests**: Command workflows
- **Property tests**: Hash consistency, object integrity
- **Cross-platform tests**: Windows, macOS, Linux compatibility

```bash
cargo test                    # Run all tests
cargo test --test integration # Integration tests only
cargo test domain::          # Domain layer tests
```

## 🚀 Usage Examples

### Basic Workflow

```bash
# Initialize repository
git-rs init

# Add files to staging
git-rs add README.md src/

# Check status
git-rs status

# Create commit (when implemented)
git-rs commit -m "Initial implementation"

# View differences (when implemented)
git-rs diff
git-rs diff --staged
```

### Educational Exploration

```bash
# Examine object database
find .git-rs/objects -type f
file .git-rs/objects/5a/b2c3d4...

# View staging area
cat .git-rs/git-rs-index | jq .

# Check references
cat .git-rs/HEAD
cat .git-rs/refs/heads/main
```

## 🎓 Learning Outcomes

After exploring this implementation, you'll understand:

1. **Git's Object Model**: How blobs, trees, and commits form a directed acyclic graph
2. **Content Addressing**: Why identical content produces identical hashes
3. **Three Trees**: Working directory, index, and HEAD relationships
4. **Reference System**: How branches and tags are just pointers to commits
5. **Staging Process**: Why the index exists and how it enables powerful workflows
6. **File System Integration**: How Git maps its abstract model to disk storage

## 🔍 Debugging and Introspection

Use these commands to explore git-rs internals:

```bash
# Object inspection
hexdump -C .git-rs/objects/5a/b2c3d4...

# Decompression (requires zlib tools)
zpipe -d < .git-rs/objects/5a/b2c3d4...

# Index inspection  
jq . .git-rs/git-rs-index

# Reference tracking
find .git-rs/refs -type f -exec echo {} \; -exec cat {} \;
```

## 📚 Educational Resources

Each command implementation includes:

- **Comprehensive documentation**: What, why, and how
- **Visual diagrams**: ASCII art showing data flow
- **Code examples**: Real-world usage patterns
- **Test cases**: Behavior verification
- **Comparison with Git**: How our implementation differs/matches

## 🤝 Contributing

This is primarily an educational project, but contributions are welcome:

- Bug fixes and improvements
- Additional test cases
- Documentation enhancements
- Performance optimizations
- New command implementations

## 📖 References

- [Git Internals Book](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [Git Source Code](https://github.com/git/git)
- [Pro Git Book](https://git-scm.com/book)
- [Git Object Model](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)

---

**Remember**: This implementation uses `.git-rs/` directories to avoid conflicts with real Git repositories, making it safe to experiment with in existing projects!
