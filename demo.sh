#!/usr/bin/env bash

# Git-RS Demo Script
# This script demonstrates the git-rs implementation

set -e

echo "🦀 Git-RS Educational Implementation Demo"
echo "========================================"
echo ""

# Clean up any previous demo
rm -rf /tmp/git-rs-demo
mkdir -p /tmp/git-rs-demo
cd /tmp/git-rs-demo

echo "📁 Created demo directory: $(pwd)"
echo ""

echo "🚀 Step 1: Initialize a new repository"
echo "Command: git-rs init"
echo ""
cargo run --manifest-path /Users/elsonwu/www/git-rs/Cargo.toml -- init
echo ""

echo "📊 Step 2: Examine the created .git structure"
echo "Directory structure:"
find .git -type f -o -type d | sort | sed 's/^/  /'
echo ""

echo "📄 Step 3: Examine key files"
echo ""
echo ".git/HEAD contents:"
cat .git/HEAD | sed 's/^/  /'
echo ""

echo ".git/config contents:"
cat .git/config | sed 's/^/  /'
echo ""

echo "🎯 Step 4: Test error handling"
echo "Trying to initialize again (should fail):"
echo ""
cargo run --manifest-path /Users/elsonwu/www/git-rs/Cargo.toml -- init || echo "✅ Correctly failed as expected"
echo ""

echo "✨ Demo completed!"
echo ""
echo "📚 What we've implemented so far:"
echo "  ✅ Complete .git directory structure creation"
echo "  ✅ Object database initialization"  
echo "  ✅ Reference system (HEAD, branches)"
echo "  ✅ Repository configuration"
echo "  ✅ Error handling for existing repositories"
echo ""
echo "🚧 Coming next:"
echo "  - git add (staging files)"
echo "  - git commit (creating commits)"
echo "  - git diff (comparing changes)"
echo "  - git status (repository status)"
echo ""
echo "🔗 For more details, see:"
echo "  - README.md: Project overview and usage"
echo "  - GIT_INTERNALS.md: Detailed explanation of Git internals"
echo "  - Source code: Extensively documented with examples"
