# ProvChainOrg

## Project Overview

ProvChainOrg is a distributed blockchain system in Rust that enhances blockchain with embedded ontology and knowledge graph for data traceability. It extends the "GraphChain" concept with semantic technologies, providing high-speed traceability, configurable consensus (PoA/PBFT), and cross-chain interoperability.

## Technology Stack

- **Language**: Rust 1.70+ (edition 2021)
- **Runtime**: Tokio async runtime
- **Semantic**: Oxigraph RDF/SPARQL triplestore
- **Cryptography**: Ed25519 signatures, ChaCha20-Poly1305 encryption
- **Web**: Axum framework with JWT authentication
- **Networking**: WebSockets for P2P communication
- **Workspace**: Includes `owl2-reasoner` as workspace member

## Build & Run Commands

```bash
# Build the project
cargo build

# Run main application
cargo run

# Run supply chain traceability demo
cargo run demo

# Start web server (default port 8080)
cargo run -- web-server --port 8080

# Run tests
cargo test

# Run benchmarks
cargo bench

# Run specific test suites
cargo test --test load_tests --release
```

## Architecture Overview

### Core Modules
- `src/core/` - Blockchain state and block management with Ed25519 signing
- `src/network/` - P2P networking and consensus (PoA/PBFT)
- `src/semantic/` - OWL2 reasoning and SHACL validation
  - `owl2_enhanced_reasoner.rs` - Full OWL2 feature support (hasKey, property chains, qualified cardinality)
  - `owl_reasoner.rs` - Base OWL reasoning with validation
- `src/security/` - Encryption and wallet management
- `src/integrity/` - Blockchain integrity validation
  - `transaction_counter.rs` - RDF parsing-based accurate transaction counting
  - `blockchain_validator.rs` - Chain reconstruction, hash integrity, corruption detection
  - `sparql_validator.rs` - SPARQL query validation and graph consistency
- `src/interop/` - Cross-chain bridge implementation
- `src/web/` - REST API handlers with JWT auth
- `src/knowledge_graph/` - Graph algorithms for traceability
- `src/analytics/` - Performance monitoring and metrics

### Key Binaries
- `provchain-org` (src/main.rs) - Main CLI application
- `owl2-integration-test` (src/bin/owl2_integration_test.rs) - Ontology integration tests

### Configuration
- `config.toml` - Node configuration (consensus type, network settings, storage)
- Environment: `JWT_SECRET` required for API authentication

## Project Patterns

### Error Handling
- Custom error types in `src/error.rs`
- Uses `anyhow` for error propagation
- `thiserror` for structured error types

### Async Runtime
- Tokio-based async/await throughout
- `async-trait` for trait implementations

### Storage
- Hybrid storage: RDF triples (public) + encrypted triples (private)
- Persistent via `src/storage/` module
- Oxigraph as RDF store backend

### Consensus
- Trait-based consensus manager in `src/network/consensus.rs`
- Runtime protocol switching via configuration
- Authority-based block creation

### Security
- JWT-based API authentication (`jsonwebtoken` crate)
- Ed25519 digital signatures for consensus and block validation
  - Each blockchain instance has a unique signing key (`SigningKey`)
  - Public key stored as hex-encoded `validator_public_key` for verification
  - Blocks signed with `signing_key.sign(block.hash.as_bytes())`
  - Signature verification via `Verifier::verify()` before adding blocks
- ChaCha20-Poly1305 for private data encryption
- Argon2 for password-based key derivation in wallet encryption
- Key rotation tracking with 90-day recommended interval
  - `should_rotate_key()` - Check if key needs rotation
  - `days_since_key_rotation()` - Monitor key age
  - Placeholder for `rotate_signing_key()` (requires blockchain consensus)
- RDF canonicalization for tamper-evident block hashing
  - Hash calculation uses `calculate_hash_with_store()` for RDF data
  - Combines metadata with canonicalized RDF hash for integrity

## Testing

### Test Structure
- `tests/` - Integration tests
- Unit tests alongside source code
- Benchmark suites in `benches/`
- Load tests in `tests/load_tests.rs`

### Key Test Files
- `tests/project_requirements_test.rs` - Consensus and bridge validation
- `tests/privacy_test.rs` - Encryption and wallet tests
- `tests/wallet_encryption_tests.rs` - Comprehensive ChaCha20-Poly1305 AEAD encryption tests (nonce uniqueness, tamper detection, key requirements, performance)
- `tests/key_rotation_tests.rs` - Ed25519 signing key rotation and lifecycle management
- `tests/enhanced_traceability_demo.rs` - Traceability validation
- `src/integrity/` - SPARQL consistency validation and graph integrity checking
  - Transaction count validation with RDF parsing
  - Blockchain reconstruction validation from persistent storage
  - Hash integrity verification across the chain
  - Corrupted block detection and metadata consistency

### Benchmarking
- Criterion-based benchmarks in `benches/`
- Performance toolkit in `benchmark-toolkit/`
- Measures Goodput (successful TPS) and Latency
- Note: iai-callgrind benchmark removed due to unmaintained bincode v1.3.3 dependency (RUSTSEC-2025-0141)

## Deployment

### Docker
- Multi-node deployment via `deploy/docker-compose.3node.yml`
- Production deployment with `deploy/docker-compose.production.yml`
- Prebuilt images available (see `deploy/QUICKSTART_PREBUILT.md`)

### Environment Variables
- `JWT_SECRET` - Required for API authentication (32+ chars)
- Node-specific settings in `config.toml`

## Documentation

- Main docs in `docs/`
- Architecture: `docs/ARCHITECTURE.md`
- User manual: `docs/USER_MANUAL.md`
- Benchmarking: `docs/benchmarking/`
- Deployment: `docs/deployment/`

## Development Notes

- This is a research project for thesis: "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"
- Focus on semantic web standards: RDF, SPARQL, SHACL, OWL2
- Implements scientific benchmarking for semantic overhead evaluation
- Cross-chain bridge uses lock-and-mint pattern with SHACL validation

### Recent Enhancements

**Integrity Validation System** (`src/integrity/`):
- **Transaction Counter** - Parses RDF content to count actual triples/transactions
  - Supports multiple RDF formats: Turtle, N-Triples, RDF/XML
  - Validates count consistency between reported vs actual triples
  - Detects systematic counting errors and discrepancies
  - Fallback to line counting for unparseable content
- **Blockchain Validator** - Comprehensive blockchain integrity checking
  - Chain reconstruction validation from persistent RDF store
  - Block hash integrity verification with RDF canonicalization
  - Missing block detection and corrupted block identification
  - Metadata consistency checks (timestamp, hash, previous_hash)
  - SPARQL-based block data extraction and round-trip validation

**Enhanced OWL2 Reasoner** (`src/semantic/owl2_enhanced_reasoner.rs`):
- Full OWL2 feature support beyond basic reasoning:
  - **owl:hasKey** - Uniqueness constraints for entity identification
  - **owl:propertyChainAxiom** - Transitive relationship inference
  - **owl:qualifiedCardinality** - Complex cardinality restrictions
- Processes OWL2 axioms from loaded ontologies via SPARQL queries
- Generates inferred relationships through property chain application
- Validates entity uniqueness and qualified cardinality constraints

**Blockchain Signing Architecture** (`src/core/blockchain.rs`):
- Each blockchain instance generates unique Ed25519 signing key on creation
- Blocks signed during creation with `signing_key.sign(block.hash.as_bytes())`
- Signature verification required before adding blocks to chain
- Key rotation tracking with 90-day recommended interval (placeholder implementation)
- Public key stored as hex-encoded for validator authentication

## Security Notes

### Documented Vulnerabilities
All security vulnerabilities and unmaintained dependencies are documented in `Cargo.toml` with detailed risk assessments and mitigation strategies:

- **RUSTSEC-2022-0040** (owning_ref v0.4.1): CRITICAL memory corruption vulnerability in transitive dependency chain via json-ld crate. Risk assessed as LOW for this project.
- **RUSTSEC-2023-0089** (atomic-polyfill v1.0.3): LOW risk - used for atomic operations on platforms without native support.
- **RUSTSEC-2024-0436** (paste v1.0.15): LOW risk - stable proc macro for code generation.
- **RUSTSEC-2024-0370** (proc-macro-error v1.0.4): LOW risk - widely used error handling crate.
- **RUSTSEC-2025-0141** (bincode v1.3.3): Removed iai-callgrind dependency; v2.0.1 still used directly with LOW risk assessment.
- **RUSTSEC-2026-0002** (lru v0.12.5): LOW-MEDIUM risk - IterMut violates Stacked Borrows in specific cases.

### Dependency Management
- Upgraded `reqwest` from 0.11 to 0.12 (fixes rustls-pemfile warning)
- Removed `iai-callgrind` benchmark dependency to eliminate unmaintained bincode v1.3.3
- Replaced unmaintained `opentelemetry-jaeger` with `opentelemetry-otlp` for distributed tracing
- Upgraded `prometheus` to 0.14 for protobuf>=3.7.2 compatibility
- All dependency upgrades focused on security vulnerability remediation
- Comprehensive security documentation added to Cargo.toml with quarterly re-evaluation schedule
