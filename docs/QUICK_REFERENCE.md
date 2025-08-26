# Git-RS Quick Reference 🚀

Quick reference guide for all git-rs commands and features.

## 📋 Command Overview

| Command | Status | --git-compat Support | Description |
|---------|--------|----------------------|-------------|
| `init` | ✅ Complete | ✅ Full Support | Initialize repository |
| `add` | ✅ Complete | 🚧 Placeholder | Stage files for commit |
| `status` | ✅ Complete | 🚧 Placeholder | Show working tree status |
| `commit` | ✅ Complete | 🚧 Placeholder | Create new commit |  
| `diff` | ✅ Complete | 🚧 Placeholder | Show changes between states |
| `clone` | ✅ Complete | 🚧 Placeholder | Clone remote repository |
| `log` | ❌ Not Implemented | ❌ Not Available | Show commit history |

## 🌐 Git Compatibility Mode

### Usage

```bash
# Educational mode (default) 
git-rs <command>              # Uses .git-rs/ directory

# Git compatibility mode
git-rs --git-compat <command> # Uses .git/ directory
```

### Directory Structure Comparison

| Mode | Git Directory | Index File | Use Case |
|------|---------------|------------|----------|
| Educational (default) | `.git-rs/` | `git-rs-index` | Safe learning, no conflicts |
| Compatible (`--git-compat`) | `.git/` | `index` | Git tool interoperability |

### Examples

```bash
# Safe learning environment
mkdir learn-git && cd learn-git
git-rs init                    # Creates .git-rs/
git-rs add file.txt           # Safe, no Git conflicts

# Git compatibility testing  
mkdir test-with-git && cd test-with-git
git-rs --git-compat init      # Creates .git/
git-rs --git-compat add file.txt
git status                    # Use real Git to verify!
```

## 🎯 Educational Features

### Visual Learning Aids

- **Detailed Console Output**: See exactly what Git does internally
- **Object Hash Display**: Understanding content-addressed storage
- **Directory Structure**: Explore .git-rs/ to learn Git internals
- **JSON Index Format**: Readable staging area representation

### Safety Features  

- **Non-destructive**: Default mode won't interfere with real Git repos
- **Side-by-side Comparison**: Run git-rs and git commands together
- **Educational Messaging**: Clear explanations of each operation

## 🔧 Quick Command Examples

### Repository Setup

```bash
# Initialize new repository (educational mode)
git-rs init

# Initialize for Git compatibility testing
git-rs --git-compat init
```

### File Operations

```bash
# Stage files
git-rs add file1.txt file2.txt
git-rs add .                    # Stage all changes

# Check status
git-rs status                   # See what's staged/modified

# Create commit
git-rs commit -m "Your message"

# View differences
git-rs diff                     # Unstaged changes
git-rs diff --cached            # Staged changes
```

### Repository Cloning

```bash
# Clone from GitHub
git-rs clone https://github.com/user/repo.git

# Clone to specific directory
git-rs clone https://github.com/user/repo.git my-project
```

## 🚧 Development Status

### Fully Implemented ✅

- Repository initialization with dual-mode support
- File staging with blob object creation
- Status checking with three-way comparison
- Commit creation with tree/commit objects
- Diff generation with unified format
- HTTP-based repository cloning

### Partially Implemented 🚧

- Git compatibility mode (init only, other commands have placeholders)
- Error handling and recovery

### Not Implemented ❌

- Log command (commit history traversal)
- Branch operations (create, switch, merge)
- Tag operations
- Push/pull operations  
- Advanced Git features (rebase, etc.)

## 📚 Learning Path

### Beginner

1. `git-rs init` - Understand repository structure
2. `git-rs add` - Learn object creation and staging
3. `git-rs status` - See the three Git trees
4. `git-rs commit` - Create your first commit object

### Intermediate

5. `git-rs diff` - Understand change detection
6. `git-rs clone` - Learn remote operations
7. Explore `.git-rs/` directory structure
8. Compare with real Git using `--git-compat` mode

### Advanced (Future)

9. `git-rs log` - Commit graph traversal (when implemented)
10. Branch and merge operations (planned)
11. Understanding pack files and compression
12. Building your own Git features

## 🔍 Troubleshooting

### Common Issues

**"Not a git repository"**
- Make sure you ran `git-rs init` first
- Check you're in the right directory
- Use `git-rs --git-compat` if testing with real Git

**File not found errors**
- Ensure file paths are correct
- Check file permissions  
- Use relative paths from repository root

**Compatibility mode not working**
- Use `--git-compat` before the command name
- Currently only `init` command fully supports compatibility mode
- Other commands have placeholders (development in progress)

## 💡 Pro Tips

- Use educational mode for learning, compatibility mode for testing
- Explore `.git-rs/objects/` to see how Git stores data
- Compare git-rs output with real git commands side-by-side
- Read the console output - it explains what's happening internally
- Check out the documentation in `docs/` for deep dives

---

For complete documentation, see:
- [`docs/COMMANDS.md`](COMMANDS.md) - Detailed command reference
- [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) - Git internals guide  
- [`docs/GIT_INTERNALS.md`](GIT_INTERNALS.md) - Deep technical details
