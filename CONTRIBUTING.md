# Contributing to ProvChainOrg

Thank you for your interest in contributing to ProvChainOrg! This document will help you get started.

---

## Table of Contents

1. [About ProvChainOrg](#about-provchainorg)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Contribution Guidelines](#contribution-guidelines)
5. [Coding Standards](#coding-standards)
6. [Testing](#testing)
7. [Commit Guidelines](#commit-guidelines)
8. [Pull Request Process](#pull-request-process)
9. [Good First Issues](#good-first-issues)
10. [Getting Help](#getting-help)

---

## About ProvChainOrg

ProvChainOrg is a distributed blockchain system in Rust that enhances blockchain with embedded ontology and knowledge graph for data traceability. It extends the "GraphChain" concept with semantic technologies.

**Key Technologies**:
- **Language**: Rust 1.70+
- **Runtime**: Tokio async runtime
- **Semantic**: Oxigraph RDF/SPARQL triplestore
- **Cryptography**: Ed25519 signatures, ChaCha20-Poly1305 encryption
- **Consensus**: Pluggable (PoA/PBFT)

**Research Context**: This is a research project for the thesis "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability".

---

## Development Setup

### Prerequisites

- **Rust**: 1.70 or later ([Install Rust](https://rustup.rs/))
- **Git**: For version control
- **Docker** (optional): For containerized deployment
- **make** (optional): For convenience scripts

### Quick Start

```bash
# Clone the repository
git clone https://github.com/your-org/provchain-org.git
cd provchain-org

# Install dependencies
cargo fetch

# Run tests to verify setup
cargo test --all

# Build the project
cargo build --release

# Run the application
cargo run -- --help
```

### Development Workflow

```bash
# Watch mode for development
cargo watch -x check -x test -x run

# Format code
cargo fmt

# Run linter
cargo clippy --all-targets

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

### IDE Setup

**Recommended IDEs**:
- **VS Code** with rust-analyzer extension
- **IntelliJ IDEA** with Rust plugin
- **Neovim** with rust-tools.nvim

**VS Code Extensions**:
```
rust-analyzer
CodeLLDB
Even Better TOML
Crates
```

---

## Project Structure

```
provchain-org/
├── src/                      # Main source code
│   ├── core/                 # Core blockchain logic
│   │   ├── blockchain.rs     # Blockchain state & blocks
│   │   └── transaction.rs    # Transaction handling
│   ├── network/              # P2P networking
│   │   └── consensus.rs      # Consensus algorithms
│   ├── semantic/             # OWL2 reasoning
│   │   ├── owl2_enhanced_reasoner.rs
│   │   └── owl_reasoner.rs
│   ├── security/             # Cryptography & encryption
│   ├── integrity/            # Blockchain validation
│   ├── web/                  # REST API & WebSocket
│   └── main.rs               # Application entry point
├── owl2-reasoner/            # Workspace member (OWL2 reasoning)
│   └── src/
├── tests/                    # Integration tests
├── benches/                  # Criterion benchmarks
├── docs/                     # Documentation
├── deploy/                   # Docker & deployment configs
└── CLAUDE.md                 # Project instructions
```

### Key Components

| Component | Description | Files |
|-----------|-------------|-------|
| **Blockchain Core** | Block management, chain state | `src/core/blockchain.rs` |
| **Consensus** | PoA, PBFT implementations | `src/network/consensus.rs` |
| **Semantic Layer** | OWL2 reasoning, SHACL validation | `src/semantic/` |
| **Security** | Encryption, signatures, wallets | `src/security/` |
| **Integrity** | Chain validation, corruption detection | `src/integrity/` |
| **Web Layer** | REST API, WebSocket handlers | `src/web/` |
| **OWL2 Reasoner** | Standalone reasoning crate | `owl2-reasoner/` |

---

## Contribution Guidelines

### Ways to Contribute

1. **Bug Reports**: Found a bug? [Report it](#bug-reports)
2. **Feature Requests**: Have an idea? [Suggest it](#feature-requests)
3. **Code Contributions**: Fix bugs or add features [PR process](#pull-request-process)
4. **Documentation**: Improve docs, add examples
5. **Tests**: Add test coverage
6. **Code Review**: Review existing PRs

### Before You Contribute

1. **Check existing issues**: Search for similar issues/PRs
2. **Discuss large changes**: Open an issue first for major features
3. **Follow coding standards**: See [Coding Standards](#coding-standards)
4. **Write tests**: All code must be tested
5. **Update docs**: Update relevant documentation

### Contributor License Agreement

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

## Coding Standards

### Rust Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

We use clippy for additional linting:

```bash
# Run clippy
cargo clippy --all-targets

# Auto-fix safe warnings
cargo clippy --fix --allow-dirty
```

**Current Clippy Status**: 254 warnings (work in progress to reduce)

### Code Organization

**Module Structure**:
```rust
//! Module documentation (required)

// External imports (grouped)
use std::collections::HashMap;
use tokio::sync::Mutex;

// Local imports (grouped)
use crate::core::blockchain::Blockchain;
use crate::security::crypto::sign;

// Constants
const MAX_BLOCK_SIZE: usize = 1024 * 1024;

// Types
pub struct MyStruct { ... }

// Traits
impl MyStruct { ... }

// Implementations
impl Default for MyStruct {
    fn default() -> Self { ... }
}
```

**Error Handling**:
```rust
// Use Result for fallible operations
pub fn do_something() -> Result<(), Error> {
    // ...
}

// Use thiserror for custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation failed: {0}")]
    Validation(String),
}
```

### Documentation

**Public items must be documented**:
```rust
/// Validates a blockchain block.
///
/// # Arguments
///
/// * `block` - The block to validate
/// * `chain` - The blockchain to validate against
///
/// # Returns
///
/// Returns `Ok(())` if the block is valid, `Err(Error)` otherwise.
///
/// # Errors
///
/// This function will return an error if:
/// - Block hash is invalid
/// - Signature verification fails
/// - Timestamp is too far in the future
///
/// # Examples
///
/// ```no_run
/// use provchain_org::validate_block;
///
/// let result = validate_block(&block, &blockchain);
/// assert!(result.is_ok());
/// ```
pub fn validate_block(block: &Block, chain: &Blockchain) -> Result<(), Error> {
    // ...
}
```

### Security Considerations

**When handling security-sensitive code**:

1. **Cryptography**: Use established crates (ed25519-dalek, ChaCha20-Poly1305)
2. **Input Validation**: Always validate external input
3. **Error Messages**: Don't expose sensitive information in errors
4. **Secrets**: Never log secrets or keys
5. **Dependencies**: Review security advisories regularly

---

## Testing

### Test Organization

```
tests/                       # Integration tests
├── load_tests.rs           # Load testing
├── wallet_encryption_tests.rs
└── key_rotation_tests.rs

src/**/                      # Unit tests (in same files)
└── mod.rs
    #[cfg(test)]
    mod tests { ... }
```

### Running Tests

```bash
# All tests
cargo test --all

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Release mode (faster)
cargo test --release
```

### Writing Tests

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(0, "prev_hash".to_string());
        assert_eq!(block.index, 0);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_blockchain_integration() {
    let blockchain = Blockchain::new();
    let block = create_test_block();

    let result = blockchain.add_block(block).await;
    assert!(result.is_ok());
}
```

**Property-Based Tests** (using proptest):
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip(val in any::<u64>) {
        let serialized = serialize(val);
        let deserialized = deserialize(&serialized);
        assert_eq!(val, deserialized);
    }
}
```

### Test Coverage

**Current Status**: Coverage measurement not yet configured
**Target**: 80% coverage for critical paths

**Priority Areas**:
- Blockchain core (block creation, validation)
- Security (signatures, encryption)
- Consensus (PoA, PBFT)
- Semantic layer (OWL2 reasoning)

---

## Commit Guidelines

### Commit Message Format

We follow conventional commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/config changes
- `perf`: Performance improvements
- `security`: Security fixes

**Examples**:
```
feat(consensus): add PBFT consensus implementation

Implement PBFT consensus algorithm with three-phase commit.
Add view change mechanism for leader failure.

Closes #123

Co-Authored-By: Contributor <email@example.com>
```

```
fix(security): prevent replay attacks in PBFT

Add sequence number validation to prevent message replay.
Also add test coverage for replay attack scenarios.

Fixes #456
```

```
docs: update deployment guide for Docker

Add instructions for multi-node deployment.
Include troubleshooting section.
```

### Commit Best Practices

1. **Atomic commits**: One logical change per commit
2. **Clear messages**: Describe what and why, not how
3. **Reference issues**: Link to related issues/PRs
4. **Sign commits**: Use `git commit -S` for security
5. **Fixup commits**: Use `git commit --fixup` for follow-ups

---

## Pull Request Process

### Workflow

1. **Fork** the repository
2. **Create a branch** for your work
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make changes** following [Coding Standards](#coding-standards)
4. **Write tests** for your changes
5. **Run tests** and ensure they pass
6. **Update documentation** if needed
7. **Commit** with clear messages
8. **Push** to your fork
9. **Create a Pull Request**

### PR Description Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass locally

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] No new warnings generated
- [ ] Added tests for changes
- [ ] All tests passing
```

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one approval required
3. **Changes Addressed**: Respond to all review comments
4. **Squash Commits**: Maintain clean history
5. **Merge**: Maintainer merges after approval

### Getting Your PR Merged

- **Be responsive**: Address review feedback promptly
- **Be patient**: Reviewers may be in different time zones
- **Be open**: Consider suggestions constructively
- **Follow up**: Ping after 1 week if no response

---

## Good First Issues

Look for issues labeled `good first issue` or `help wanted`:

### Example Good First Issues

**Difficulty: Easy**
- [ ] Add test coverage for X module
- [ ] Improve error messages in Y function
- [ ] Update documentation for Z feature
- [ ] Fix clippy warnings in ABC file

**Difficulty: Medium**
- [ ] Refactor large function into smaller ones
- [ ] Add benchmark for performance-critical path
- [ ] Implement missing configuration option
- [ ] Add integration test for feature

**Difficulty: Hard**
- [ ] Implement new consensus algorithm
- [ ] Optimize OWL2 reasoning performance
- [ ] Design and implement new API endpoint

### Finding Issues to Work On

```bash
# List good first issues
gh issue list --label "good first issue"

# List help wanted
gh issue list --label "help wanted"

# List by priority
gh issue list --label "priority:high"
```

---

## Getting Help

### Resources

**Documentation**:
- [CLAUDE.md](./CLAUDE.md) - Project instructions
- [USER_MANUAL.md](./docs/USER_MANUAL.md) - User guide
- [BENCHMARKING.md](./BENCHMARKING.md) - Performance testing

**Rust Resources**:
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

**Semantic Web**:
- [RDF Primer](https://www.w3.org/TR/rdf11-primer/)
- [SPARQL Query Language](https://www.w3.org/TR/sparql11-query/)
- [OWL2 Web Ontology Language](https://www.w3.org/TR/owl2-primer/)

### Communication Channels

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and ideas
- **Pull Requests**: For code review

### Asking Questions

When asking for help:

1. **Search first**: Check if your question was already answered
2. **Be specific**: Include code snippets, error messages
3. **Provide context**: What are you trying to do?
4. **Show effort**: What have you tried?

**Good Question Template**:
```
## What I'm trying to do
[Describe your goal]

## What I've tried
[Show code, commands, research]

## Expected behavior
[What should happen]

## Actual behavior
[What actually happens, error messages]

## Environment
- Rust version:
- OS:
- ProvChainOrg version:
```

---

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Credited in release notes
- Thanked in project communications

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and considerate
- Use inclusive language
- Focus on constructive feedback
- Assume good intent

---

Thank you for contributing to ProvChainOrg! 🚀

---

*Last updated: 2026-01-14*
