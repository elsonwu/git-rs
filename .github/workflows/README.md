# GitHub Actions Workflows 🔄

This directory contains comprehensive CI/CD workflows for the git-rs project to ensure code quality, security, and educational value.

## 🧪 Workflows Overview

### [`ci.yml`](.github/workflows/ci.yml) - Continuous Integration

**Triggers**: Push to main/develop, Pull Requests
**Purpose**: Core quality checks for every commit

- **Format Check**: Ensures code follows Rust formatting standards (`cargo fmt`)
- **Clippy Lint**: Static analysis and best practices enforcement (`cargo clippy`)  
- **Cross-Platform Tests**: Tests on Ubuntu and macOS (Windows not supported)
- **MSRV Check**: Ensures compatibility with Minimum Supported Rust Version (1.70.0)
- **Documentation Build**: Verifies all documentation builds correctly
- **Security Audit**: Checks for known security vulnerabilities
- **Unused Dependencies**: Detects unnecessary dependencies
- **Integration Tests**: End-to-end testing of git-rs workflow
- **CI Success Gate**: All checks must pass for PR approval

### [`release.yml`](.github/workflows/release.yml) - Release Management

**Triggers**: Git tags (`v*`), Manual dispatch
**Purpose**: Automated release builds and deployment

- **Multi-Platform Builds**: Creates binaries for Linux and macOS (x64/ARM)
- **GitHub Releases**: Automatically creates releases with binaries
- **Cross-Compilation**: Ensures git-rs works on supported platforms
- **Artifact Upload**: Makes binaries available for download

### [`docs.yml`](.github/workflows/docs.yml) - Documentation Quality

**Triggers**: Changes to source code or markdown files
**Purpose**: Maintains high-quality educational documentation

- **API Documentation**: Builds and deploys Rust documentation
- **GitHub Pages**: Publishes docs to hosted site
- **Markdown Linting**: Ensures consistent markdown formatting
- **Link Validation**: Checks for broken links in documentation
- **Educational Content Verification**: Tests examples from README/docs
- **Documentation Completeness**: Ensures all features are documented

### [`quality.yml`](.github/workflows/quality.yml) - Code Quality Analysis

**Triggers**: Push to main/develop, Pull Requests  
**Purpose**: Deep code quality and performance analysis

- **Code Coverage**: Measures test coverage with detailed reports
- **Complexity Analysis**: Tracks lines of code, binary size, dependencies
- **Performance Benchmarks**: Measures hash calculation performance
- **Memory Analysis**: Checks for memory leaks using Valgrind
- **Static Analysis**: Security and dependency analysis

### [`maintenance.yml`](.github/workflows/maintenance.yml) - Automated Maintenance

**Triggers**: Weekly schedule, Manual dispatch
**Purpose**: Keeps dependencies and security up-to-date

- **Dependency Review**: Checks for outdated dependencies
- **Automated Updates**: Creates PRs for dependency updates
- **Security Advisories**: Regular security vulnerability scanning
- **Maintenance Reports**: Generates maintenance status reports

## 🛡️ Quality Gates

### For Pull Requests

All PRs must pass:

- ✅ Code formatting (`cargo fmt --check`)
- ✅ Linting (`cargo clippy`)
- ✅ All tests on multiple platforms
- ✅ Documentation builds
- ✅ Security audit passes
- ✅ No unused dependencies
- ✅ Integration tests pass

### For Releases

Releases require:

- ✅ All CI checks pass
- ✅ Git tag format: `v*` (e.g., `v0.1.0`)
- ✅ Cross-platform binary builds
- ✅ Automated release notes generation

## 🎓 Educational Benefits

These workflows serve as learning examples for:

### **Rust CI/CD Best Practices**

- Multi-platform testing strategies
- Cargo tool integration (fmt, clippy, audit)
- Cross-compilation setup
- Security-first approach

### **GitHub Actions Patterns**

- Matrix builds for multiple configurations
- Caching strategies for faster builds
- Artifact management and deployment
- Workflow dependencies and gates

### **Code Quality Standards**

- Automated formatting enforcement
- Static analysis integration
- Test coverage measurement
- Performance monitoring

### **Documentation as Code**

- Automated documentation deployment
- Link validation and content verification
- Educational example testing

## 🔧 Configuration Files

### [`link-check-config.json`](link-check-config.json)

Configuration for markdown link checking:

- Ignores localhost/development links
- Retry logic for flaky external links
- Timeout and fallback settings

### Caching Strategy

All workflows use intelligent caching:

- Cargo registry and dependencies
- Build artifacts
- Tool installations
- Platform-specific optimizations

## 🚀 Local Development

### Run Checks Locally

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Documentation
cargo doc --no-deps --document-private-items

# Security audit
cargo audit
```

### Simulate CI Environment

```bash
# Install tools used in CI
cargo install cargo-audit cargo-udeps cargo-llvm-cov

# Run comprehensive local check
./scripts/local-ci.sh  # (if created)
```

## 🎯 Workflow Benefits for Learning

1. **Practical DevOps**: See real-world CI/CD implementation
2. **Quality Culture**: Understand importance of automated quality checks
3. **Security Awareness**: Learn about dependency and vulnerability management
4. **Cross-Platform Development**: Experience multi-OS compatibility challenges
5. **Documentation Standards**: See how to maintain high-quality docs
6. **Performance Monitoring**: Learn to track performance over time

## 📊 Status Badges

Add to README.md for visibility:

```markdown
[![CI](https://github.com/elsonwu/git-rs/workflows/CI/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/ci.yml)
[![Docs](https://github.com/elsonwu/git-rs/workflows/Documentation/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/docs.yml)
[![Security](https://github.com/elsonwu/git-rs/workflows/Maintenance/badge.svg)](https://github.com/elsonwu/git-rs/actions/workflows/maintenance.yml)
```

## 🤝 Contributing to Workflows

### Adding New Checks

1. Consider educational value alongside practical benefit
2. Ensure cross-platform compatibility
3. Add appropriate caching for performance
4. Document the purpose and benefits
5. Test locally before committing

### Workflow Maintenance

- Keep GitHub Actions versions updated
- Monitor workflow performance and costs
- Review security best practices regularly
- Update Rust toolchain versions as needed

These workflows ensure git-rs maintains high quality while serving as an excellent example of modern Rust project automation and best practices!
