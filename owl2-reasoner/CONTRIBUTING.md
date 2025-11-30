# 🤝 Contributing to OWL2 Reasoner

*Welcome! We're excited you want to contribute to our high-performance OWL2 DL reasoning engine.*

## 🎯 Our Mission

The OWL2 Reasoner project aims to provide:
- **Complete OWL2 DL Support**: Full SROIQ(D) description logic implementation
- **Enterprise Performance**: Sub-millisecond reasoning with memory safety
- **Developer Experience**: Idiomatic Rust API with comprehensive documentation
- **Community Building**: Inclusive environment for semantic web development

## 🚀 Getting Started

### Prerequisites

1. **Rust 1.70+**: Latest stable Rust toolchain
2. **Git**: For version control
3. **Basic OWL2 Knowledge**: Understanding of semantic web concepts
4. **Rust Knowledge**: Comfort with ownership, lifetimes, and memory safety

### Development Setup

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/owl2-reasoner.git
cd owl2-reasoner

# 3. Add upstream remote
git remote add upstream https://github.com/original-org/owl2-reasoner.git

# 4. Install development dependencies
cargo install cargo-watch cargo-edit cargo-audit

# 5. Run full test suite to ensure everything works
cargo test --all-features
cargo bench
cargo fmt --check
cargo clippy -- -D warnings
```

### Development Workflow

```bash
# 1. Create a new branch for your feature
git checkout -b feature/your-feature-name

# 2. Make your changes with live testing
cargo watch -x test

# 3. Run comprehensive checks before committing
cargo fmt
cargo clippy -- -D warnings
cargo test --all-features
cargo bench

# 4. Commit with conventional commit messages
git commit -m "feat: add support for custom data property ranges"

# 5. Push to your fork
git push origin feature/your-feature-name

# 6. Create a Pull Request
```

## 📝 Contribution Guidelines

### Code Standards

#### ✅ What We Value

1. **Performance First**: Every contribution should maintain or improve performance
2. **Memory Safety**: Zero unsafe code without justification and comprehensive testing
3. **Documentation**: Public APIs must have rustdoc examples
4. **Testing**: All code must be thoroughly tested
5. **Semantic Web Compliance**: OWL2 DL and W3C standards adherence

#### 🎨 Code Style

We use standard Rust formatting with additional requirements:

```rust
// ✅ Good: Clear naming with comprehensive documentation
/// Performs consistency checking using optimized tableaux algorithm
///
/// # Arguments
/// * `ontology` - The ontology to check for consistency
///
/// # Returns
/// * `Ok(true)` if the ontology is consistent
/// * `Ok(false)` if the ontology has contradictions
/// * `Err(OwlError)` if reasoning fails
///
/// # Performance
/// This operation runs in ~80ns for typical ontologies and uses
/// O(n) memory where n is the number of axioms.
///
/// # Examples
/// ```rust
/// let reasoner = SimpleReasoner::new(ontology);
/// assert!(reasoner.is_consistent()?);
/// ```
pub fn is_consistent(&self) -> OwlResult<bool> {
    // Implementation...
}
```

```rust
// ❌ Bad: Unclear naming, no documentation, no examples
pub fn check(&self) -> bool {
    // Implementation...
}
```

### Testing Requirements

#### 🧪 Test Coverage

1. **Unit Tests**: Every public function must have tests
2. **Integration Tests**: Complex workflows need integration tests
3. **Performance Tests**: Critical paths need benchmarks
4. **Property-Based Tests**: Complex logic needs property-based testing

#### 📊 Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let ontology = create_test_ontology();
        let reasoner = SimpleReasoner::new(ontology);

        // Act
        let result = reasoner.is_consistent().unwrap();

        // Assert
        assert!(result);
    }

    #[test]
    fn test_error_conditions() {
        // Test error handling and edge cases
    }

    #[test]
    fn test_performance_requirements() {
        let start = Instant::now();
        // ... operation ...
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100));
    }

    proptest! {
        #[test]
        fn test_property_based_behavior(
            // Generate random test data
            class_count in 1..1000usize,
            axiom_count in 1..5000usize
        ) {
            // Test with random inputs
        }
    }
}
```

### Documentation Requirements

#### 📚 Public API Documentation

Every public item must have:
1. **Purpose**: What it does and why
2. **Parameters**: All inputs with types and constraints
3. **Returns**: What it returns and when it errors
4. **Performance**: Expected performance characteristics
5. **Examples**: Working code examples
6. **Panics**: When it might panic (prefer Result over panic)

#### 📖 Examples and Tutorials

- Add runnable examples for major features
- Update tutorial documentation for new capabilities
- Include performance benchmarking examples

## 🏗️ Architecture Guidelines

### Core Principles

1. **Memory Safety**: Use arena allocation, avoid clones when possible
2. **Performance**: Profile critical paths, optimize for sub-millisecond operations
3. **Modularity**: Clean separation between parsing, reasoning, and storage
4. **Extensibility**: Plugin architecture for different OWL2 profiles

### Project Structure

```
src/
├── lib.rs              # Public API and re-exports
├── iri.rs              # IRI management and caching
├── entities.rs         # OWL2 entities (classes, properties, individuals)
├── axioms/             # OWL2 axioms and logical statements
├── ontology.rs         # Ontology structure and management
├── storage/            # Storage backends and indexing
├── parser/             # Multi-format OWL2 parsers
├── reasoning/          # Reasoning engine and inference
├── profiles/           # OWL2 profile optimizations (EL, QL, RL)
└── validation/         # Ontology validation frameworks
```

### Adding New Features

1. **Design Phase**: Create issue with detailed design proposal
2. **Implementation**: Follow existing patterns and architecture
3. **Testing**: Comprehensive tests including performance benchmarks
4. **Documentation**: Full API documentation and examples
5. **Review**: Submit PR for community review

## 🐛 Bug Reports

### Bug Report Template

```markdown
## Bug Description
Brief description of the issue

## Steps to Reproduce
1. Create ontology with...
2. Call function...
3. Observe error...

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., macOS 14.0]
- Rust version: [e.g., 1.75.0]
- OWL2 Reasoner version: [e.g., 0.2.0]

## Additional Context
Ontology file, stack traces, performance measurements
```

## 🚀 Feature Requests

### Feature Request Template

```markdown
## Feature Description
Clear description of the feature

## Problem Statement
What problem does this solve?

## Proposed Solution
How you envision the feature working

## Alternatives Considered
Other approaches you've thought about

## Additional Context
Performance requirements, OWL2 compliance notes, etc.
```

## 📋 Pull Request Process

### Before Submitting

1. **Test Thoroughly**: `cargo test --all-features`
2. **Format Code**: `cargo fmt`
3. **Lint**: `cargo clippy -- -D warnings`
4. **Benchmarks**: `cargo bench` for performance-sensitive changes
5. **Documentation**: Update rustdoc and tutorial documentation

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Performance benchmarks run
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
```

### Review Process

1. **Automated Checks**: CI runs tests, benchmarks, and linting
2. **Peer Review**: At least one maintainer review required
3. **Performance Review**: Performance-sensitive changes need benchmarking
4. **Semantic Web Review**: OWL2 compliance verification

## 🏆 Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- **Be Respectful**: Welcome newcomers and experienced contributors alike
- **Be Constructive**: Focus on what is best for the community
- **Be Inclusive**: Respect different perspectives and experiences
- **Be Collaborative**: Work together to solve problems

### Getting Help

1. **Documentation**: Check `cargo doc --open` and tutorial files
2. **Issues**: Search existing issues before creating new ones
3. **Discussions**: Use GitHub Discussions for questions
4. **Community**: Join our community channels (links in README)

### Recognition

Contributors are recognized in:
- `AUTHORS.md` file for code contributors
- Release notes for significant contributions
- Community spotlights in announcements

## 🚀 Areas for Contribution

### High Priority

1. **Performance Optimization**: Critical path profiling and optimization
2. **Additional Parsers**: Support for more RDF formats
3. **SPARQL Integration**: Query engine for ontology data
4. **Visualization Tools**: Graph-based ontology visualization
5. **WebAssembly**: WASM bindings for browser usage

### Medium Priority

1. **SWRL Rules**: Semantic Web Rule Language support
2. **Import/Export**: Additional ontology format support
3. **CLI Tools**: Command-line interface for common operations
4. **IDE Integration**: Language server protocol support
5. **Benchmark Suite**: Comprehensive performance testing

### Community Contributions

1. **Examples**: Real-world usage examples
2. **Tutorials**: Domain-specific tutorials (biomedical, supply chain, etc.)
3. **Translations**: Documentation translation
4. **Community Support**: Helping others in discussions and issues

## 📞 Contact Information

- **Project Maintainers**: [Maintainer GitHub handles]
- **Community Discussions**: [Link to GitHub Discussions]
- **Bug Reports**: [Link to Issues]
- **Security Issues**: [Security contact information]

## 🎉 Thank You!

Your contributions help make semantic web technology more accessible and performant. Every contribution, whether documentation, code, or community support, is valuable and appreciated.

**Happy Contributing!** 🦉✨