#!/bin/bash

# Git-RS Formatting Script
# This script runs all formatting tools to ensure code quality

set -e

echo "🦀 Running Rust formatting..."
cargo fmt

echo "📝 Running markdown formatting..."
if command -v markdownlint-cli2 >/dev/null 2>&1; then
    markdownlint-cli2 --fix "**/*.md" "!target/**" "!node_modules/**"
    echo "✅ Markdown formatting complete"
else
    echo "⚠️  markdownlint-cli2 not installed. Run: npm install -g markdownlint-cli2"
fi

echo "🧹 Running Rust linting..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Running tests..."
cargo test

echo "🎉 All formatting and checks complete!"
