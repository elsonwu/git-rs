# Git-RS: Educational Git Implementation 🦀

A minimal Git implementation in Rust designed for learning Git internals and Rust programming.

## 🎯 Project Goals

This project implements core Git functionality to understand:
- How Git stores objects (blobs, trees, commits)
- How the staging area (index) works
- How references and branches are managed
- Git's object model and hash-based storage

## 🏗️ Architecture

Following Domain Driven Design (DDD) principles:

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── domain/              # Core domain models
│   ├── mod.rs
│   ├── repository.rs    # Repository aggregate
│   ├── objects.rs       # Git objects (blob, tree, commit)
│   ├── references.rs    # Refs and HEAD
│   └── index.rs         # Staging area
├── infrastructure/      # File system operations
│   ├── mod.rs
│   ├── object_store.rs  # Object database
│   ├── ref_store.rs     # Reference storage
│   └── index_store.rs   # Index file handling
├── application/         # Use cases
│   ├── mod.rs
│   ├── init.rs          # git init
│   ├── add.rs           # git add
│   ├── commit.rs        # git commit
│   ├── diff.rs          # git diff
│   └── clone.rs         # git clone
└── cli/                 # Command line interface
    ├── mod.rs
    └── commands.rs
```

## 🔍 Git Internals Visual Guide

### .git Directory Structure
```
.git/
├── objects/          # Object database
│   ├── 12/
│   │   └── 34abcd... # Blob/tree/commit objects
│   ├── info/
│   └── pack/
├── refs/             # References
│   ├── heads/        # Branch references
│   │   └── main      # Points to commit hash
│   └── tags/         # Tag references
├── HEAD              # Current branch reference
├── index             # Staging area
├── config            # Repository configuration
└── description       # Repository description
```

## 📋 Implementation Steps

- [ ] **Step 1**: Domain modeling and project setup
- [ ] **Step 2**: `git init` - Repository initialization
- [ ] **Step 3**: Object storage system (blobs, trees, commits)
- [ ] **Step 4**: `git add` - Staging files
- [ ] **Step 5**: `git commit` - Creating commits
- [ ] **Step 6**: `git diff` - Comparing content
- [ ] **Step 7**: `git clone` - Repository cloning
- [ ] **Step 8**: Additional utilities and improvements

## 🚀 Usage

```bash
# Initialize a new repository
git-rs init

# Add files to staging area
git-rs add <file>

# Create a commit
git-rs commit -m "Initial commit"

# Show differences
git-rs diff

# Clone a repository
git-rs clone <url>
```

## 🧪 Testing

```bash
cargo test
```

## 📚 Learning Resources

Each implementation step includes detailed documentation explaining:
- What Git does internally
- How the data structures work
- Visual representations of the process
- Code examples and tests

## 🤝 Contributing

This is an educational project. Feel free to explore, experiment, and learn!
