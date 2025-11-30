# OWL2 Reasoner - AI Agent Guide

## Project Overview

**OWL2 Reasoner** is a high-performance, feature-complete OWL2 DL (Description Logic) reasoning engine implemented in Rust. It provides complete SROIQ(D) description logic support with enterprise-grade performance, memory safety, and comprehensive multi-format parsing capabilities.

- **Version**: 0.2.0
- **Language**: Rust (Edition 2021, requires 1.70+)
- **Lines of Code**: ~51,000 across 215+ files
- **License**: MIT/Apache 2.0
- **Repository**: https://github.com/anusornc/owl2-reasoner

## Technology Stack

### Core Dependencies
- **RDF/OWL Parsing**: `rio_api`, `rio_turtle`, `rio_xml`, `oxiri`, `json-ld`
- **Data Structures**: `petgraph`, `indexmap`, `hashbrown`, `bit-set`
- **Serialization**: `serde`, `serde_json`, `serde-xml-rs`, `xmltree`
- **Error Handling**: `thiserror`, `anyhow`
- **Logging**: `log`, `env_logger`
- **Performance**: `rayon`, `crossbeam`, `dashmap`, `bumpalo`, `smallvec`, `parking_lot`, `lru`
- **Testing**: `proptest`, `criterion`, `tempfile`, `clap`

### Optional Features
- **HTTP Support**: `reqwest` (for import resolution)
- **Web Service**: `warp`, `tokio`, `uuid`, `async-trait` (REST API interface)

## Architecture

### Module Organization

```
src/
├── lib.rs                      # Main library interface (159 lines)
├── reasoning/                  # Reasoning algorithms (16 files)
│   ├── tableaux/              # Tableaux reasoning engine
│   │   ├── core.rs            # Core tableaux implementation
│   │   ├── expansion.rs       # Rule expansion strategies
│   │   ├── blocking.rs        # Blocking strategies
│   │   ├── dependency.rs      # Dependency-directed backtracking
│   │   ├── graph.rs           # Assertion graph management
│   │   ├── memory.rs          # Memory-efficient storage
│   │   └── parallel.rs        # Parallel reasoning
│   ├── classification.rs      # Ontology classification
│   ├── consistency.rs         # Consistency checking
│   ├── query.rs               # SPARQL-like query engine
│   ├── rules.rs               # Rule-based reasoning
│   └── batch_operations.rs    # Batch processing
├── parser/                     # Multi-format parsers (31 files)
│   ├── turtle.rs              # RDF/Turtle parser
│   ├── rdf_xml.rs             # RDF/XML streaming parser
│   ├── rdf_xml_legacy.rs      # Legacy RDF/XML parser
│   ├── owl_xml.rs             # OWL/XML parser
│   ├── manchester/            # Manchester syntax parser
│   ├── json_ld/               # JSON-LD 1.1 support
│   └── import_resolver.rs     # Ontology import resolution
├── validation/                 # Validation framework (16 files)
│   ├── academic_validation.rs # Research-grade validation
│   ├── enterprise_validation.rs # Production compliance
│   ├── compliance_reporter.rs # Detailed reporting
│   └── benchmark_suite.rs     # Validation benchmarking
├── profiles/                   # OWL2 profile optimizations
│   ├── el/                    # EL profile (polynomial complexity)
│   ├── ql/                    # QL profile (query answering)
│   └── rl/                    # RL profile (rule-based)
├── axioms/                     # OWL2 axiom implementations
│   ├── core.rs                # Core axiom structures
│   ├── class_axioms.rs        # Class-related axioms
│   ├── class_expressions.rs   # Complex class expressions
│   └── property_expressions.rs # Property expressions
├── memory_*.rs                 # Memory management modules
├── cache*.rs                   # Caching systems
├── entities.rs                 # OWL2 entities
├── ontology.rs                 # Ontology management
├── error.rs                    # Error handling
└── config.rs                   # Configuration management
```

### Key Components

1. **Reasoning Engine**: Tableaux-based algorithm with optimizations for EL, QL, RL profiles
2. **Parser Suite**: Multi-format support (Turtle, RDF/XML, OWL/XML, JSON-LD, Manchester)
3. **Memory Management**: Arena allocation, leak detection, graceful degradation
4. **Caching System**: Multi-layered caching with LRU eviction and compression
5. **Validation Framework**: Academic and enterprise validation with compliance reporting
6. **Query Engine**: SPARQL-like pattern matching with optimization

## Build and Test Commands

### Building

```bash
# Standard build
cargo build

# Release build with optimizations
cargo build --release

# Build with all features
cargo build --all-features

# Check without building
cargo check
```

### Testing

```bash
# Run core library tests (37 tests)
cargo test --lib

# Run all tests including integration tests
cargo test

# Run specific test suite
cargo test --test integration_tests
cargo test --test equality_reasoning_tests
cargo test --test performance_tests
cargo test --test memory_safety_tests

# Run with output capture disabled
cargo test -- --nocapture

# Run with release optimizations
cargo test --release
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench tableaux_benchmarks
cargo bench --bench memory_safety_benchmarks
cargo bench --bench query_optimization_bench
cargo bench --bench cache_performance

# Run algorithmic complexity analysis
cargo bench --bench algorithmic_complexity

# Run memory tracking benchmarks
cargo bench --bench memory_tracking_bench
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Generate documentation without dependencies
cargo doc --no-deps --open
```

### Coverage Analysis

```bash
# Run coverage script (requires cargo-tarpaulin)
./scripts/coverage.sh

# Install cargo-tarpaulin if needed
cargo install cargo-tarpaulin
```

### Validation Scripts

```bash
# Run comprehensive validation
./scripts/run_validation.sh

# Run green group tests
./scripts/run_green_group_tests.sh

# Run benchmarks
./scripts/run_benchmarks.sh

# Validate system
./scripts/validate_system.sh
```

## Code Style Guidelines

### Rust Style

- **Edition**: 2021
- **Formatting**: Standard `rustfmt` configuration
- **Linting**: Clippy with strict warnings (`-D warnings`)
- **Documentation**: All public APIs must have comprehensive documentation
- **Error Handling**: No `unwrap()` in production code, use proper error types
- **Memory Safety**: All memory operations must be validated
- **Thread Safety**: Concurrent operations must be properly synchronized

### Naming Conventions

- **Modules**: `snake_case`
- **Structs**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Traits**: `PascalCase`
- **Enums**: `PascalCase` for types, `CamelCase` for variants

### Code Quality Standards

1. **Zero Compilation Warnings**: All code must compile without warnings
2. **Comprehensive Error Handling**: Use `Result<T, E>` and custom error types
3. **Memory Safety**: Leverage Rust's ownership system, use arena allocation
4. **Documentation**: All public items must have rustdoc comments with examples
5. **Tests**: New features must include unit tests and integration tests
6. **Benchmarks**: Performance-critical code must have benchmarks

## Testing Strategy

### Test Organization

- **Unit Tests**: Embedded in source files (`#[cfg(test)]` modules)
- **Integration Tests**: `tests/` directory (12 comprehensive test files)
- **Benchmarks**: `benches/` directory (25 benchmark files)
- **Examples**: `examples/` directory (30+ example files)

### Test Categories

1. **Core Library Tests**: 37 unit tests covering core functionality
2. **Integration Tests**: End-to-end validation of complete workflows
   - `integration_tests.rs` - Main integration suite
   - `equality_reasoning_tests.rs` - Equality reasoning validation
   - `performance_tests.rs` - Performance regression tests
   - `memory_safety_tests.rs` - Memory safety validation
   - `parser_tests.rs` - Parser validation
   - `profile_optimization_tests.rs` - Profile-specific tests
   - `turtle_parser_tests.rs` - Turtle parser validation

3. **Memory Safety Tests**: Real-time memory monitoring and validation
4. **Performance Tests**: Continuous performance validation with benchmarks
5. **Concurrency Tests**: Thread safety validation for parallel operations

### Test Data

- **Test Ontologies**: Located in `examples/test_ontologies/`
- **Generated Data**: `src/test_data_generator.rs` for synthetic ontologies
- **EPCIS Test Data**: GS1 EPCIS 2.0 standard test cases

## Security Considerations

### Memory Safety

- **Arena Allocation**: Bump allocator for efficient memory management
- **Leak Detection**: Automatic memory leak detection and reporting
- **Memory Limits**: Configurable memory constraints with graceful degradation
- **Memory Monitoring**: Real-time memory usage tracking
- **Emergency Protection**: Emergency system protection mechanisms

### Input Validation

- **Parser Validation**: Comprehensive syntax validation for all formats
- **IRI Validation**: Proper IRI syntax and scheme validation
- **Ontology Validation**: Academic and enterprise validation frameworks
- **Compliance Checking**: OAEI (Ontology Alignment Evaluation Initiative) integration

### Concurrency Safety

- **Thread-Safe Design**: Full concurrent reasoning capabilities
- **Synchronization**: Proper use of `Arc`, `Mutex`, `RwLock`, `DashMap`
- **Race Condition Prevention**: Careful design of shared state access
- **Deadlock Prevention**: Structured locking protocols

## Development Workflow

### Branch Strategy

- **Main Branch**: Production-ready code, all tests must pass
- **Feature Branches**: `feature/description` for new features
- **Bugfix Branches**: `bugfix/description` for bug fixes
- **Release Branches**: `release/vX.Y.Z` for release preparation

### Pull Request Process

1. **Fork Repository**: Create a fork of the main repository
2. **Create Branch**: Create a feature branch from main
3. **Implement Feature**: Follow code style guidelines
4. **Add Tests**: Include comprehensive tests for new functionality
5. **Run Validation**: Execute full test suite and benchmarks
6. **Update Documentation**: Update relevant documentation
7. **Submit PR**: Create pull request with detailed description

### Pre-Commit Checklist

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] Clippy passes with no warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated
- [ ] New tests added for new features
- [ ] Benchmarks added for performance-critical code

## Performance Characteristics

### Benchmark Results

Based on comprehensive benchmarking:

- **Consistency Checking**: ~80ns for typical ontologies (constant time)
- **IRI Creation**: 525μs for 500 IRIs, scaling linearly
- **Ontology Operations**: 3ms for 500 axioms, 154ms for 5,000 axioms
- **Memory Usage**: 1.2MB for 1,000 entities with arena allocation
- **Scalability**: Linear scaling for most operations
- **Profile Optimizations**: EL, QL, RL profiles provide significant speedups

### Optimization Strategies

1. **Algorithmic**: Tableaux algorithm with dependency-directed backtracking
2. **Data Structures**: Optimized graph structures, indexed storage
3. **Caching**: Multi-layer caching with compression and LRU eviction
4. **Parallelism**: Rayon-based parallel processing for independent operations
5. **Memory**: Arena allocation, small vector optimization, string interning

## Deployment

### Binary Targets

- **Library**: `src/lib.rs` - Main library interface
- **Validation Tool**: `src/bin/owl2_validation.rs` - CLI validation tool
- **Performance Analyzer**: `scripts/analyze_tableaux_performance.rs` - Performance analysis

### Features

- **Default**: `rio-xml`, `http`, `tokio`, `uuid`, `async-trait`
- **Web Service**: `warp`, `uuid`, `tokio`, `async-trait` (optional)
- **HTTP Support**: `reqwest` (optional)

### Production Deployment

The system is production-ready with:
- Zero compilation warnings
- Comprehensive error handling
- Memory safety guarantees
- Thread-safe operations
- Performance optimizations
- Graceful degradation under stress

## Documentation

### Available Documentation

- **Quick Start**: `docs/QUICK_START.md` - 5-minute getting started
- **Interactive Tutorial**: `docs/INTERACTIVE_TUTORIAL.md` - 6 hands-on lessons
- **Examples Showcase**: `docs/EXAMPLES_SHOWCASE.md` - Real-world demonstrations
- **API Reference**: `docs/API_REFERENCE.md` - Complete API documentation
- **Contributing Guide**: `CONTRIBUTING.md` - Development workflow
- **Memory Safe Testing**: `docs/MEMORY_SAFE_TESTING.md` - Testing guidelines

### Generating Documentation

```bash
# Generate and open API documentation
cargo doc --open

# Generate without dependencies
cargo doc --no-deps --open

# Build technical documentation
./scripts/build-technical-docs.sh
```

## Verification Rules

**STRICT VERIFICATION PROTOCOL - NO EXCEPTIONS**

### Rule #1: Evidence-Based Claims Only
- Never claim "tests pass" without running `cargo test`
- Never claim "production ready" without comprehensive testing
- Never claim "performance optimized" without benchmarks
- Always provide actual command output as evidence

### Rule #2: Honest Status Reporting
Use accurate status format:
```
## ACTUAL STATUS (VERIFIED)
- Core Library Tests: 37/37 pass (VERIFIED with `cargo test --lib`)
- Integration Tests: X/Y pass (VERIFIED)
- Performance: Average Xs execution (VERIFIED with `cargo bench`)
- Compilation: X warnings, Y errors (VERIFIED with `cargo build`)
```

### Rule #3: Run-to-Verify Protocol
Before claiming success, you must:
1. Run the actual command
2. Show the real output
3. Count actual results
4. Verify compilation
5. Test performance

### Rule #4: Compilation Error Honesty
If compilation fails:
- Report exact error messages
- Count all compilation errors
- Provide specific error details
- Do not claim "minor issues"

## Byterover MCP Integration

You are given two tools from Byterover MCP server:

### 1. `byterover-store-knowledge`
**MUST** always use this tool when:
- Learning new patterns, APIs, or architectural decisions
- Encountering error solutions or debugging techniques
- Finding reusable code patterns or utility functions
- Completing significant task or plan implementation

### 2. `byterover-retrieve-knowledge`
**MUST** always use this tool when:
- Starting new task or implementation to gather context
- Before making architectural decisions
- When debugging issues to check for previous solutions
- Working with unfamiliar parts of the codebase

## Contact & Support

- **Project Lead**: Anusorn Chaikaew
- **Issues**: GitHub Issues
- **Source**: https://github.com/anusornc/owl2-reasoner
- **Documentation**: https://docs.rs/owl2-reasoner/

---

**Built with ❤️ in Rust for the Semantic Web**

*A complete OWL2 DL reasoning system with comprehensive documentation, multi-format parsing support, and performance optimizations for semantic web applications.*
