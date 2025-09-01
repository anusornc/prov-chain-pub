# Implementation Plan: CLI-Based Domain-Specific Ontology Traceability System

## Overview

Implement a CLI-based system where participants specify their domain ontology at startup, ensuring all participants in a traceability network use the same ontology with strict SHACL validation that blocks invalid transactions.

Multiple paragraphs outlining the scope, context, and high-level approach. This implementation will transform the current single-ontology system into a domain-flexible system where UHT manufacturing participants run `cargo run -- web-server --ontology uht_manufacturing.owl`, pharmaceutical participants run `cargo run -- web-server --ontology pharmaceutical.owl`, and all participants in the same domain MUST use the same ontology. Each ontology has its own SHACL validation rules extending core SHACL, and invalid transactions are completely rejected, not warned. The system maintains startup-only ontology selection with no runtime switching, ensuring network consistency and data integrity throughout the blockchain lifetime.

## Types

Domain ontology configuration and SHACL validation system integration.

Detailed type definitions, interfaces, enums, or data structures with complete specifications:

```rust
pub struct OntologyConfig {
    pub domain_ontology_path: String,
    pub core_ontology_path: String,
    pub domain_shacl_path: String,
    pub core_shacl_path: String,
    pub validation_mode: ValidationMode,
    pub ontology_hash: String,
}

pub enum ValidationMode {
    Strict,  // Block invalid transactions (default)
}

pub struct ShaclValidator {
    pub core_shapes: Vec<ShaclShape>,
    pub domain_shapes: Vec<ShaclShape>,
    pub ontology_hash: String,
    pub validation_enabled: bool,
}

pub struct ShaclShape {
    pub target_class: String,
    pub properties: Vec<ShaclProperty>,
    pub constraints: Vec<ShaclConstraint>,
}

pub struct ValidationError {
    pub message: String,
    pub shape_violations: Vec<ShapeViolation>,
    pub transaction_id: Option<String>,
}
```

Enhanced CLI Command Structure:
```rust
pub enum Commands {
    WebServer {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        #[arg(long)]
        ontology: Option<String>,  // NEW: Domain ontology specification
    },
    AddFile { 
        path: String,
        #[arg(long)]
        ontology: Option<String>,
    },
    Query { 
        path: String,
        #[arg(long)]
        ontology: Option<String>,
    },
    // All commands get --ontology parameter
}
```

## Files

Detailed breakdown of file modifications and creations.

New files to be created (with full paths and purpose):
- `src/ontology/mod.rs` - Main ontology management module with public exports
- `src/ontology/shacl_validator.rs` - SHACL validation implementation with Oxigraph integration
- `src/ontology/domain_manager.rs` - Domain-specific ontology loading and consistency checking
- `src/ontology/error.rs` - Custom error types for ontology validation failures
- `shapes/core.shacl.ttl` - Core SHACL shapes extracted from current traceability.shacl.ttl
- `shapes/uht_manufacturing.shacl.ttl` - UHT manufacturing-specific SHACL shapes
- `shapes/pharmaceutical.shacl.ttl` - Pharmaceutical supply chain-specific SHACL shapes
- `shapes/automotive.shacl.ttl` - Automotive parts traceability-specific SHACL shapes
- `tests/ontology_validation_tests.rs` - Comprehensive ontology validation test suite

Existing files to be modified (with specific changes):
- `src/main.rs` - Add --ontology CLI parameter to all relevant commands, integrate ontology loading
- `src/config/mod.rs` - Add OntologyConfig struct, CLI parameter override support
- `src/core/blockchain.rs` - Integrate SHACL validation into add_block() and transaction processing
- `src/web/handlers.rs` - Add SHACL validation to transaction submission endpoints
- `src/lib.rs` - Add ontology module exports and public API
- `Cargo.toml` - Add SHACL validation dependencies if required
- `config/ontology.toml` - Enhanced configuration with domain-specific settings

Configuration file updates:
- Update `config.toml` with ontology validation settings and default paths

## Functions

Detailed breakdown of function modifications and additions.

New functions (name, signature, file path, purpose):
- `load_domain_ontology(ontology_path: &str) -> Result<OntologyConfig, OntologyError>` in `src/ontology/domain_manager.rs` - Load and validate domain ontology configuration
- `validate_transaction_with_shacl(transaction: &Transaction, validator: &ShaclValidator) -> Result<(), ValidationError>` in `src/ontology/shacl_validator.rs` - Validate RDF transaction data against SHACL shapes
- `load_shacl_shapes(core_path: &str, domain_path: &str) -> Result<ShaclValidator, ValidationError>` in `src/ontology/shacl_validator.rs` - Load and combine core and domain SHACL shapes
- `check_ontology_consistency(local_hash: &str, network_hash: &str) -> Result<(), ConsistencyError>` in `src/ontology/domain_manager.rs` - Verify ontology consistency across network participants
- `extract_rdf_from_transaction(transaction: &Transaction) -> Result<String, ExtractionError>` in `src/ontology/shacl_validator.rs` - Extract RDF data from transaction for validation

Modified functions (exact name, current file path, required changes):
- `main()` in `src/main.rs` - Parse --ontology CLI parameter, pass to blockchain initialization with ontology configuration
- `Blockchain::new()` in `src/core/blockchain.rs` - Accept OntologyConfig parameter, initialize SHACL validator
- `add_block(rdf_data: String)` in `src/core/blockchain.rs` - Add SHACL validation step before block creation, reject invalid data
- `submit_transaction()` in `src/web/handlers.rs` - Integrate SHACL validation, return validation errors to client
- `Config::load_or_default(path: &str)` in `src/config/mod.rs` - Support CLI ontology parameter override of config file settings

## Classes

Detailed breakdown of class modifications and additions.

New classes (name, file path, key methods, inheritance):
- `OntologyManager` in `src/ontology/domain_manager.rs` - Methods: load_ontology(), validate_consistency(), get_hash()
- `ShaclValidator` in `src/ontology/shacl_validator.rs` - Methods: validate(), load_shapes(), check_constraints()
- `DomainConfig` in `src/ontology/mod.rs` - Configuration container for domain-specific settings
- `ValidationError` in `src/ontology/error.rs` - Custom error type with detailed violation information
- `OntologyError` in `src/ontology/error.rs` - Error type for ontology loading and management failures

Modified classes (exact name, file path, specific modifications):
- `Blockchain` in `src/core/blockchain.rs` - Add ontology_config field, shacl_validator field, validation methods
- `Config` in `src/config/mod.rs` - Add ontology_config: Option<OntologyConfig> field
- `Transaction` in existing transaction module - Add validation metadata fields if needed

## Dependencies

Dependency modifications and integration requirements.

New packages, version changes, and integration requirements:
- No new external dependencies required - use existing Oxigraph for SHACL validation
- Leverage existing clap framework for CLI argument parsing
- Integrate with existing TOML configuration system
- Use existing RDF/Turtle parsing capabilities in Oxigraph

Integration with existing systems:
- SHACL validation integrated with Oxigraph RDF store
- CLI parameter system using existing clap framework
- Configuration management with existing config.toml structure
- Error handling using existing error types and patterns

## Testing

Test file requirements, existing test modifications, and validation strategies.

New test files and coverage:
- `tests/ontology_validation_tests.rs` - Test ontology loading, SHACL validation, error handling
- `tests/cli_ontology_tests.rs` - Test CLI parameter parsing and configuration override
- `tests/domain_consistency_tests.rs` - Test network consistency checking and ontology hash validation

Test scenarios to cover:
- Valid ontology loading for each domain (UHT manufacturing, pharmaceutical, automotive)
- SHACL validation success scenarios with valid RDF data
- SHACL validation failure scenarios with constraint violations
- CLI parameter parsing and configuration file override
- Transaction rejection on SHACL validation failure
- Network consistency checking with matching and mismatched ontology hashes
- Error handling and proper error message generation

Existing test modifications:
- Update blockchain tests to include ontology configuration
- Modify transaction tests to include SHACL validation scenarios
- Update configuration tests for new ontology settings

## Implementation Order

Numbered steps showing the logical order of changes to minimize conflicts and ensure successful integration.

1. **Create ontology module structure** - Set up `src/ontology/mod.rs` with basic module organization
2. **Implement ontology configuration** - Add OntologyConfig struct and basic loading in `src/config/mod.rs`
3. **Add CLI parameter parsing** - Modify `src/main.rs` to accept --ontology parameter
4. **Create SHACL shape files** - Extract core shapes and create domain-specific SHACL files
5. **Implement SHACL validator** - Create `src/ontology/shacl_validator.rs` with validation logic
6. **Integrate ontology loading** - Implement `src/ontology/domain_manager.rs` with ontology management
7. **Modify blockchain initialization** - Update `Blockchain::new()` to accept ontology configuration
8. **Add transaction validation** - Integrate SHACL validation into `add_block()` method
9. **Update web handlers** - Add validation to transaction submission endpoints
10. **Implement error handling** - Create comprehensive error types and handling
11. **Create test suite** - Implement all test files with comprehensive coverage
12. **Test domain ontologies** - Validate functionality with UHT, pharmaceutical, and automotive ontologies
13. **Integration testing** - Test complete CLI-to-validation workflow
14. **Documentation updates** - Update README and documentation for new CLI usage
