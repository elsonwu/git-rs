# Git Internals: Educational Guide

## Overview

This document provides a comprehensive guide to Git internals, explaining how Git stores and manages data under the hood. This educational implementation focuses on clarity and understanding rather than performance.

## Table of Contents

1. [Git's Core Concepts](#gits-core-concepts)
2. [The .git Directory](#the-git-directory)
3. [Git Objects](#git-objects)
4. [References](#references)
5. [The Index (Staging Area)](#the-index-staging-area)
6. [How Git Commands Work](#how-git-commands-work)

## Git's Core Concepts

Git is fundamentally a **content-addressable filesystem** with a VCS interface. At its core:

- **Everything is content-addressed**: Objects are identified by SHA-1 hashes of their content
- **Immutability**: Once created, objects never change
- **Three main object types**: Blobs (files), Trees (directories), Commits (snapshots)
- **References**: Human-readable names pointing to commit hashes
- **Index**: Staging area between working directory and repository

## The .git Directory

When you run `git init`, Git creates a `.git` directory with this structure:

```
.git/
|-- objects/          (Object database)
|   |-- info/         (Metadata about objects)
|   |-- pack/         (Packed objects for efficiency)
|   `-- XX/           (Directories named by first 2 chars of hash)
|       `-- YYYYYY... (Object files named by remaining 38 chars)
|-- refs/             (References - human readable names)
|   |-- heads/        (Local branches)
|   |   `-- main      (Branch pointing to commit hash)
|   |-- tags/         (Tags)
|   `-- remotes/      (Remote branches)
|-- HEAD              (Current branch or commit)
|-- index             (Staging area)
|-- config            (Repository configuration)
`-- description       (Repository description)
```

### Key Files and Directories

- **objects/**: The heart of Git - stores all content as objects
- **refs/heads/**: Branch references (each file contains a commit hash)
- **HEAD**: Points to current branch (`ref: refs/heads/main`) or commit (detached HEAD)
- **index**: Binary file containing staged changes
- **config**: Repository-specific configuration

## Git Objects

Git stores everything as objects in `.git/objects/`. Each object has:
- **Type**: blob, tree, or commit
- **Size**: Content size in bytes  
- **Content**: The actual data
- **Hash**: SHA-1 hash of "type size\0content"

### Object Storage Format

Objects are stored compressed with zlib in files named by their hash:
- Directory: First 2 characters of hash (e.g., `ab/`)
- Filename: Remaining 38 characters (e.g., `cdef123...`)
- Content: `zlib_compress("type size\0content")`

### Blob Objects

Blobs store file content:

```
blob <size>\0<file content>
```

Example:
```
blob 13\0Hello, World!
```

**Visual representation:**
```
Working Directory    Object Database
┌─────────────────┐  ┌──────────────────────────┐
│ README.md       │  │ objects/ab/cdef123...    │
│ "Hello, World!" │──▶│ blob 13\0Hello, World!   │
└─────────────────┘  └──────────────────────────┘
```

### Tree Objects

Trees store directory information:

```
tree <size>\0<mode> <filename>\0<20-byte hash><mode> <filename>\0<20-byte hash>...
```

Example:
```
tree 68\0100644 README.md\0<20-byte-hash>40000 src\0<20-byte-hash>
```

**Visual representation:**
```
Directory Structure    Tree Object
┌─────────────────┐   ┌────────────────────────────┐
│ project/        │   │ tree 68\0                  │
│ ├── README.md   │──▶│ 100644 README.md\0<hash>  │
│ └── src/        │   │ 40000 src\0<hash>         │
└─────────────────┘   └────────────────────────────┘
```

### Commit Objects

Commits store snapshots and metadata:

```
commit <size>\0tree <tree-hash>
parent <parent-hash>
author <name> <email> <timestamp>
committer <name> <email> <timestamp>

<commit message>
```

**Visual representation:**
```
Commit Chain
┌─────────────────────────────────┐
│ commit abc123...                │
│ tree def456...                  │
│ parent 789abc...                │
│ author John <john@example.com>  │
│ committer John <john@example.com>│
│                                 │
│ Initial commit                  │
└─────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│ commit 789abc...                │
│ tree fed654...                  │
│ (no parent - root commit)       │
│ author John <john@example.com>  │
│ committer John <john@example.com>│
│                                 │
│ Add initial files               │
└─────────────────────────────────┘
```

## References

References map human-readable names to object hashes.

### Branch References (`refs/heads/`)

Each branch is a file containing a commit hash:

```bash
$ cat .git/refs/heads/main
abc123def456...
```

### HEAD Reference

HEAD points to the current branch or commit:

```bash
# Symbolic reference (normal)
$ cat .git/HEAD  
ref: refs/heads/main

# Direct reference (detached HEAD)
$ cat .git/HEAD
abc123def456...
```

**Visual representation:**
```
References           Object Database
┌─────────────────┐  ┌─────────────────────┐
│ HEAD            │  │                     │
│ ↓               │  │                     │
│ refs/heads/main │──▶│ commit abc123...    │
│ abc123def456... │  │ tree def456...      │
└─────────────────┘  │ parent 789abc...    │
                     │ ...                 │
                     └─────────────────────┘
```

## The Index (Staging Area)

The index is a binary file (`.git/index`) that tracks:
- Files to be included in the next commit
- File metadata (timestamps, permissions, sizes)
- Object hashes for staged content

**Visual representation:**
```
Working Directory    Index (Staging)      Repository
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ file1.txt       │  │ file1.txt       │  │ commit abc123   │
│ file2.txt       │──▶│ (staged)        │──▶│ tree def456     │
│ file3.txt       │  │ file3.txt       │  │ parent 789abc   │
│ (modified)      │  │ (staged)        │  └─────────────────┘
└─────────────────┘  └─────────────────┘
```

## How Git Commands Work

### `git init`

1. Creates `.git` directory structure
2. Initializes empty object database (`objects/`, `objects/info/`, `objects/pack/`)
3. Creates reference directories (`refs/heads/`, `refs/tags/`)
4. Creates HEAD pointing to `refs/heads/main` (even though main doesn't exist yet)
5. Creates initial config and description files

**After `git init`:**
```
.git/
|-- objects/          (empty)
|-- refs/heads/       (empty)
|-- refs/tags/        (empty)  
|-- HEAD              ("ref: refs/heads/main")
|-- config            (initial settings)
`-- description       (default description)
```

### `git add <file>` (Not yet implemented)

1. Calculate SHA-1 hash of file content
2. Create blob object in object database
3. Update index with file metadata and blob hash
4. File is now "staged" for next commit

### `git commit -m "message"` (Not yet implemented)

1. Create tree object from current index
2. Create commit object pointing to tree
3. Update current branch reference to new commit hash
4. Clear staging area (optionally)

### `git diff` (Not yet implemented)

Compares content between:
- Working directory vs index (`git diff`)
- Index vs last commit (`git diff --cached`)
- Two commits (`git diff commit1 commit2`)

## Implementation Status

### ✅ Completed Features

- **Domain Models**: All core Git concepts modeled
- **Object Storage**: Complete blob, tree, and commit storage/retrieval
- **Reference Management**: HEAD and branch reference handling
- **Index Management**: Staging area implementation
- **Repository Initialization**: Full `git init` functionality

### 🚧 Next Steps

- **File Staging**: `git add` command implementation
- **Commit Creation**: `git commit` command implementation  
- **Content Comparison**: `git diff` implementation
- **Repository Cloning**: `git clone` implementation
- **Status Reporting**: `git status` implementation
- **Commit History**: `git log` implementation

## Educational Notes

### Why This Architecture?

This implementation uses **Domain Driven Design (DDD)** to clearly separate concerns:

- **Domain Layer**: Pure business logic, no dependencies
- **Infrastructure Layer**: File system operations, serialization
- **Application Layer**: Use cases, orchestrating domain and infrastructure
- **CLI Layer**: User interface, command parsing

This makes the code easy to understand, test, and extend.

### Simplifications for Learning

1. **JSON Index**: We use JSON instead of Git's binary index format
2. **No Compression**: Objects are stored as-is for clarity
3. **No Packed Objects**: Each object is a separate file
4. **No Network**: Clone only works with local directories
5. **Limited .gitignore**: Simple pattern matching only

### Key Learning Points

1. **Content-Addressable Storage**: Everything identified by hash of content
2. **Immutable Objects**: Objects never change, only new ones are created
3. **Directed Acyclic Graph**: Commits form a DAG structure
4. **Three Areas**: Working directory, index, and repository
5. **Snapshot Model**: Git stores snapshots, not diffs

## Further Reading

- [Git Internals - Git SCM](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Git from the Bottom Up](https://jwiegley.github.io/git-from-the-bottom-up/)
- [Pro Git Book](https://git-scm.com/book)

---

*This guide accompanies the educational Git implementation in Rust. Each concept is implemented with extensive documentation and tests for learning purposes.*
