# Git-RS: Educational Git Implementation 🦀

[![CI](https://github.com/elsonwu/git-rs/workflows/CI/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/ci.yml)
[![Documentation](https://github.com/elsonwu/git-rs/workflows/Documentation/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/docs.yml)
[![Quality](https://github.com/elsonwu/git-rs/workflows/Quality/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/quality.yml)
[![Maintenance](https://github.com/elsonwu/git-rs/workflows/Maintenance/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/maintenance.yml)

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
│   ├── commit.rs        # ✅ Commit creation
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

### ✅ `git-rs commit` - Commit Creation

**What it does:**

- Creates tree objects from staged files in index
- Generates commit objects with metadata (author, timestamp, message)
- Updates branch references to point to new commit
- Handles both root commits and commits with parents
- Validates commit messages and detects empty commits

**Educational Insights:**

- How Git creates immutable snapshots from staged changes
- Tree object construction and hierarchical file organization
- Commit object format with parent relationships
- Reference management and branch pointer updates
- The difference between root commits and regular commits

**Example:**

```bash
git-rs commit -m "Initial implementation"
# Creates tree object, commit object, and updates branch ref
```

**Internal Process:**

1. Load staged files from index: `git-rs-index`
2. Create tree entries: `{name: "README.md", mode: 100644, hash: "5ab2c3d..."}`
3. Store tree object: `tree 42\0<tree-content>` → `7def456ghi...`
4. Create commit object with:
   - Tree hash: `7def456ghi...`
   - Parent commits (if any)
   - Author/committer signatures
   - Commit message
5. Store commit object: `commit 156\0<commit-content>` → `9abc123def...`
6. Update branch reference: `.git-rs/refs/heads/main` → `9abc123def...`

**Commit Object Format:**

```text
tree 7def456ghi789...
parent 1abc234def567... (if not root commit)
author John Doe <john@example.com> 1692000000 +0000
committer John Doe <john@example.com> 1692000000 +0000

Initial implementation
```

### ✅ `git-rs diff` - Content Comparison

**What it does:**

- Generates unified diff format output showing line-by-line changes
- Compares working directory vs staging area (default)
- Compares staging area vs last commit (with `--cached`)
- Detects binary files and handles them appropriately
- Shows file headers and line context for all changes

**Educational Insights:**

- How diff algorithms work to compare file contents
- Understanding unified diff format used by patch tools
- The difference between staged and unstaged changes
- Binary file detection and handling strategies

**Example:**

```bash
# Show unstaged changes
git-rs diff

# Show staged changes  
git-rs diff --cached
```

**Output Format:**

```diff
diff --git a/README.md b/README.md
index 1234567..abcdefg 100644
--- a/README.md
+++ b/README.md
@@ -1,3 +1,4 @@
 # Git-RS
 
-Old line
+New line
+Added line
```

## 🚧 Commands in Development

### 🔄 `git-rs clone` - Repository Cloning

- Remote repository communication
- Object transfer and verification
- Reference mapping and checkout

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

# Create commit
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

## 📖 Documentation

### 📚 Core Documentation

- **[🏗️ Architecture Guide](docs/ARCHITECTURE.md)** - System design and Git internals deep dive
- **[⚡ Command Reference](docs/COMMANDS.md)** - Complete reference for all implemented commands
- **[🔍 Git Internals Explained](docs/GIT_INTERNALS.md)** - Educational exploration of Git concepts
- **[📊 Project Status](docs/STATUS.md)** - Development roadmap and contribution guidelines
- **[🦀 API Documentation](https://elsonwu.github.io/git-rs/)** - Generated Rust API docs

### 🎯 Learning Paths

**For Git Beginners:**

1. Start with this README for project overview
2. Read [Git Internals Explained](docs/GIT_INTERNALS.md) to understand core concepts
3. Try hands-on examples in [Command Reference](docs/COMMANDS.md)
4. Explore [Architecture Guide](docs/ARCHITECTURE.md) for implementation details

**For Developers:**

1. Review [Project Status](docs/STATUS.md) for current state and roadmap
2. Study [Architecture Guide](docs/ARCHITECTURE.md) for system design
3. Check [API Documentation](https://elsonwu.github.io/git-rs/) for code reference
4. Follow contribution guidelines below

**For Rust Learners:**

1. Examine [Architecture Guide](docs/ARCHITECTURE.md) for Domain-Driven Design patterns
2. Browse [API Documentation](https://elsonwu.github.io/git-rs/) for Rust idioms
3. Look at GitHub Actions workflows in `.github/workflows/` for CI/CD examples

### 🔍 Key Concepts You'll Learn

- **Git Object Model**: How blobs, trees, and commits form a directed acyclic graph
- **Content Addressing**: SHA-1 hashing and object identification system
- **Three Trees**: Working directory, index, and HEAD relationships
- **Reference System**: How branches and tags are pointers to commits
- **Domain-Driven Design**: Clean architecture with separated concerns in Rust
- **Testing Strategies**: Unit tests, integration tests, and property-based testing

### 🆘 Getting Help

- **Git concepts questions**: Check [Git Internals Explained](docs/GIT_INTERNALS.md)
- **Usage questions**: See [Command Reference](docs/COMMANDS.md)
- **Implementation questions**: Review [Architecture Guide](docs/ARCHITECTURE.md)
- **Bug reports**: Open an issue on [GitHub](https://github.com/elsonwu/git-rs/issues)
- **Feature requests**: Check [Project Status](docs/STATUS.md) first, then open an issue

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
