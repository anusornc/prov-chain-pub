[byterover-mcp]

You are given two tools from Byterover MCP server, including
## 1. `byterover-store-knowledge`
You `MUST` always use this tool when:

+ Learning new patterns, APIs, or architectural decisions from the codebase
+ Encountering error solutions or debugging techniques
+ Finding reusable code patterns or utility functions
+ Completing any significant task or plan implementation

## 2. `byterover-retrieve-knowledge`
You `MUST` always use this tool when:

+ Starting any new task or implementation to gather relevant context
+ Before making architectural decisions to understand existing patterns
+ When debugging issues to check for previous solutions
+ Working with unfamiliar parts of the codebase

---

# OWL2 Reasoner

## Module Overview

High-performance OWL2 reasoning engine implemented in Rust, providing complete OWL2 DL support with SROIQ(D) description logic. Serves as the semantic reasoning foundation for the ProvChainOrg blockchain system.

## Technology Stack

- **Language**: Rust 1.70+ (edition 2021)
- **RDF Processing**: rio_api, rio_turtle, rio_xml for multi-format parsing
- **Graph Algorithms**: petgraph for reasoning operations
- **JSON-LD**: Full 1.1 support via json-ld 0.21
- **Performance**: rayon for parallelization, dashmap for concurrent access
- **Serialization**: serde, serde_json, bincode 2.0.1

## Architecture

### Core Modules
- `ontology` - Ontology management with indexed storage
- `entities` - OWL2 entities (classes, properties, individuals)
- `axioms` - Logical statements and relationships
- `reasoning` - Tableaux algorithm and rule-based inference
- `parser` - Multi-format parsing (Turtle, RDF/XML, OWL/XML, N-Triples)
- `iri` - IRI management with caching
- `cache` - Configurable caching with eviction strategies
- `memory` - Memory leak prevention and monitoring
- `epcis` - GS1 EPCIS ontology for supply chain traceability
- `validation` - Academic validation and benchmarking

### Performance Features
- String interning and Arc-based sharing for memory efficiency
- Indexed axiom storage with O(1) access patterns
- Multi-layered caching with TTL expiration
- Hash join algorithms and query pattern optimization
- Parallel reasoning via rayon

## Build & Run

```bash
# Build the library
cargo build

# Run benchmarks (27 benchmark suites)
cargo bench

# Run examples
cargo run --example family_ontology
cargo run --example json_ld_example
cargo run --example epcis_validation_suite

# Run validation
cargo run --bin owl2_validation

# Run tests
cargo test
```

## Key Patterns

### Error Handling
- Custom error types in `error.rs` using thiserror
- OwlError and OwlResult for type-safe error propagation

### Memory Management
- String interning for IRIs to reduce allocations
- Arc-based sharing for axiom storage
- Memory leak prevention via monitoring system
- Configurable cache with LRU eviction

### Performance Optimization
- Parallel reasoning with rayon
- Concurrent data structures (dashmap, crossbeam)
- Query optimization with pattern reordering
- Memory profiling tools (memmap2, sysinfo)

## Security Considerations

### Dependency Vulnerabilities
See root `Cargo.toml` for comprehensive security documentation:
- **RUSTSEC-2022-0040** (owning_ref v0.4.1): Transitive dependency via json-ld crate. Risk assessed as LOW.
- **RUSTSEC-2026-0002** (lru v0.12.5): Direct dependency - IterMut violates Stacked Borrows in specific cases. LOW-MEDIUM risk.

### Best Practices
- No user-controlled input directly flows into owning_ref operations
- Memory-safe patterns enforced by Rust type system
- Regular dependency updates via cargo update
- iai-callgrind benchmark removed due to unmaintained bincode v1.3.3 dependency (RUSTSEC-2025-0141)

## Integration with ProvChainOrg

- Used for semantic validation of blockchain transactions
- EPCIS ontology support for supply chain traceability
- SHACL validation for data integrity
- SPARQL-like query engine for pattern matching

## Recent Improvements

**Code Quality & Testing** (January 2026):
- **Turtle Parser Test Fixes** (`tests/turtle_parser_tests.rs`)
  - Fixed 8 failing tests by correcting malformed Turtle syntax
  - Added missing subjects to property assertions (e.g., `:age "30"` → `:John :age "30"`)
  - Removed unsupported typed literals with language tags
  - Fixed OWL namespace typo (`w2.org` → `w3.org`)
  - Added proper class declarations for multi-prefix tests
  - Updated test assertions to match actual parser behavior
- **Clippy Auto-fixes Applied** (reasoning modules)
  - Changed `assert_eq!` with boolean expressions to `assert!` for clarity
  - Used `is_empty()` instead of `len() == 0` comparisons
  - Removed unnecessary `.clone()` calls
  - Fixed empty slice references (`&vec![]` → `&[]`)
  - Applied to: cache.rs, engine.rs, optimized_engine.rs, memory.rs
- **rustfmt Formatting** (Commit a6ba29c)
  - Applied standard Rust formatting across entire library (7 files)
  - All CI checks pass: cargo fmt, cargo check, cargo test
  - No functional changes, formatting only
