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

# Run portable benchmark toolkit (Docker-based)
cd benchmark-toolkit && ./run.sh
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

### Benchmarking & Monitoring Infrastructure
- `benchmark-toolkit/` - Portable Docker-based benchmark suite for performance testing
  - Auto-detects hardware capabilities (4GB-32GB+ RAM)
  - Hardware profiles: low, medium, high, ultra
  - Compares ProvChain vs Neo4j performance
  - Includes Prometheus metrics and Grafana dashboards
  - Packageable for distribution (`package.sh`)
- `monitoring/` - Production monitoring stack
  - Prometheus metrics scraping configuration
  - Grafana dashboards for benchmark comparison
  - Jaeger distributed tracing integration

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
- Criterion-based micro-benchmarks in `benches/`
- **Portable Benchmark Toolkit** (`benchmark-toolkit/`) - Docker-based performance testing
  - One-command execution: `cd benchmark-toolkit && ./run.sh`
  - Auto-detects hardware and selects optimal configuration
  - Real-time Grafana dashboards at http://localhost:3000
  - CSV/JSON results export for analysis
  - Documentation: `BENCHMARKING.md` (central entry point)
- **Research Benchmarks** (`docs/benchmarking/`) - Academic publication support
  - Query Performance: SPARQL latency vs traditional systems
  - Write Throughput: Transactions per second comparison
  - Permission Overhead: Access control performance impact
  - Cross-Chain Sync: Inter-chain data interchange speed
  - Scalability: Performance vs dataset size analysis
- Measures Goodput (successful TPS) and Latency
- Note: iai-callgrind benchmark removed due to unmaintained bincode v1.3.3 dependency (RUSTSEC-2025-0141)

## Deployment

### Docker Deployment
- **Quick Start** (10 minutes): `deploy/README_QUICKSTART.md` - Single-node setup
- **Multi-node**: `deploy/docker-compose.3node.yml` - 3-node cluster deployment
- **Production**: `deploy/docker-compose.production.yml` - Full stack with monitoring
- **Benchmark Comparison**: `deploy/docker-compose.benchmark-comparison.yml` - Performance testing
- **Prebuilt Images**: `deploy/QUICKSTART_PREBUILT.md` - Deploy without building
- **Build Script**: `deploy/build-docker-image.sh` - Custom image builds
- **Hands-On Guide**: `deploy/HANDS_ON_DEPLOYMENT_GUIDE.md` (1431 lines) - Comprehensive deployment
- **Architecture**: `deploy/DOCKER_DEPLOYMENT_ARCHITECTURE.md` (1090 lines) - System design

### Environment Variables
- `JWT_SECRET` - Required for API authentication (32+ chars)
- Node-specific settings in `config.toml`

## Documentation

### Central Documentation
- **BENCHMARKING.md** - Central entry point for all performance testing resources
  - Portable Benchmark Toolkit (recommended for most users)
  - Documentation benchmarks (research-focused)
  - Developer benchmarks (component-level)
- **README.md** - Project overview with research objectives, quick start, and benchmark toolkit
- **CONTRIBUTING.md** - Comprehensive contributor guide with development setup, coding standards, and PR process
- **docs/README.md** - Main documentation index

### User Guides
- `docs/USER_MANUAL.md` - User manual hub
- `docs/user-manual/` - Comprehensive user manual (255 lines)
  - `00-quick-start/` - 10-minute setup, first transaction, overview
  - `03-querying-data/query-library.md` - 30+ SPARQL query examples
  - `05-configuration/network-setup.md` - Network configuration guide
  - `08-troubleshooting/troubleshooting.md` - Common issues and solutions (913 lines)

### Architecture & Project Health
- `docs/architecture/COMPONENT_OWNERSHIP.md` - Component ownership matrix and knowledge distribution
  - Bus factor analysis (currently 1 - critical risk)
  - Knowledge transfer priorities for consensus, OWL2 reasoning, and semantic layer
  - Documentation gaps and action items
- `docs/project-health/` - Project health analysis and improvement tracking
  - `clippy_warnings_deep_dive.md` - Detailed analysis of 254 clippy warnings with categorization
  - `dependency_analysis_deep_dive.md` - Comprehensive dependency health assessment (640 total, 67 direct)
  - `linear_tasks_export.md` - Action items for Linear with priorities and estimates

### Deployment Guides
- `docs/deployment/HANDS_ON_DEPLOYMENT_GUIDE.md` - Step-by-step deployment (1431 lines)
- `docs/deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md` - Architecture overview (1090 lines)
- `docs/deployment/SETUP_INSTALLATION_GUIDE.md` - Installation instructions
- `docs/deployment/COMPREHENSIVE_ANALYSIS_REPORT.md` - Deployment analysis

### Benchmarking Documentation
- `docs/benchmarking/README.md` - Research-focused benchmarking guide (309 lines)
- `benchmark-toolkit/README.md` - Toolkit documentation (408 lines)
- `benchmark-toolkit/QUICKSTART.md` - Quick reference card
- `benchmark-toolkit/DEPLOYMENT_GUIDE.md` - Usage guide
- `benchmark-toolkit/PORTABILITY.md` - Distribution guide

## Development Notes

- This is a research project for thesis: "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"
- Focus on semantic web standards: RDF, SPARQL, SHACL, OWL2
- Implements scientific benchmarking for semantic overhead evaluation
- Cross-chain bridge uses lock-and-mint pattern with SHACL validation

### Recent Enhancements

**Source Code Enhancements** (Commit 65ac20e):
- **Transaction Handlers** (`src/web/handlers/transaction.rs`)
  - Enhanced triple validation with URI and literal validation
  - Privacy key encryption support via `privacy_key_id` parameter
  - ChaCha20-Poly1305 encrypted block submission workflow
  - Secure block signing and hash recalculation for encrypted data
  - Improved error handling with specific error codes
- **Blockchain Validator** (`src/integrity/blockchain_validator.rs`)
  - Enhanced SPARQL-based chain reconstruction validation
  - Graph naming consistency checks (expected vs actual URIs)
  - RDF round-trip validation (extracted vs parsed triples)
  - Comprehensive metadata validation (timestamp, hash, previous_hash)

**Project Health & Contributor Documentation** (Commit 65ac20e):
- **CONTRIBUTING.md** - Comprehensive contributor guide (200+ lines)
  - Development setup with quick start instructions
  - Project structure and key components overview
  - Coding standards (Rust style, clippy linting)
  - Testing guidelines and commit standards
  - Pull request process and good first issues
- **Component Ownership Matrix** - Architecture documentation for bus factor mitigation
  - Critical: Current bus factor is 1 (single contributor risk)
  - Component ownership breakdown: blockchain core, consensus, semantic layer, security, integrity, web layer
  - Knowledge distribution assessment and transfer priorities
  - Documentation gaps identified for consensus algorithms and OWL2 reasoning
- **Project Health Analysis** - Deep dive into code quality and dependencies
  - Clippy warnings: 254 total (55% auto-fixable, 29% high priority manual fixes)
    - 70x "borrowed expression implements traits" (auto-fixable)
    - 29x "unnecessary if let" in semantic reasoners (manual fix needed)
    - 29x "field assignment outside initializer" (auto-fixable)
    - 13x "unused result" (high priority - error suppression risk)
  - Dependency health: 58/100 score, 640 transitive dependencies
    - 1 CRITICAL (owning_ref v0.4.1 - LOW risk for this project)
    - 1 UNSOUND (lru v0.12.5 - safe with current usage)
    - 3 UNMAINTAINED (accepted risk, documented in Cargo.toml)
  - Linear action items export with priorities and estimates
    - Critical: Address bus factor, fix 77 remaining clippy warnings
    - High: Security documentation, automated dependency updates
    - Medium: lru replacement evaluation, test coverage measurement

**Documentation & Benchmarking Infrastructure** (Commit 1bff4b0):
- **Portable Benchmark Toolkit** - Complete Docker-based testing infrastructure
  - Auto-detecting hardware profiles (low/medium/high/ultra)
  - One-command execution with automatic optimization
  - Real-time Grafana dashboards and Prometheus metrics
  - Neo4j comparison for performance validation
  - Packageable for distribution (`./package.sh`)
- **Comprehensive User Manual** - Role-based documentation structure
  - Quick start guides (10-minute setup, first transaction)
  - SPARQL query library with 30+ ready-to-use examples
  - Network setup and configuration guides
  - Troubleshooting guide (913 lines)
- **Deployment Documentation** - Multi-format deployment guides
  - Quick start (10-minute single-node setup)
  - Prebuilt image deployment guide
  - Hands-on deployment guide (1431 lines)
  - Docker deployment architecture (1090 lines)
  - Multi-node cluster deployment
- **Monitoring Stack** - Production observability
  - Prometheus metrics scraping configuration
  - Grafana dashboards for benchmark comparison
  - Jaeger distributed tracing integration
- **Test File Improvements** - Formatting and organization improvements
  - `tests/wallet_encryption_tests.rs` - Enhanced readability
  - `tests/key_rotation_tests.rs` - Minor formatting updates
  - `tests/load_tests.rs` - Minor formatting updates

**Transaction Handler Enhancements** (`src/web/handlers/transaction.rs`):
- Enhanced triple validation with URI and literal validation via `validate_uri()` and `validate_literal()`
- Privacy key encryption support via `privacy_key_id` parameter
- ChaCha20-Poly1305 encrypted block submission workflow
- Secure block signing and hash recalculation for encrypted data
- Improved error handling with specific error codes and messages

**Integrity Validation System** (`src/integrity/`):
- **Transaction Counter** - Parses RDF content to count actual triples/transactions
  - Supports multiple RDF formats: Turtle, N-Triples, RDF/XML
  - Validates count consistency between reported vs actual triples
  - Detects systematic counting errors and discrepancies
  - Fallback to line counting for unparseable content
- **Blockchain Validator** - Comprehensive blockchain integrity checking
  - Chain reconstruction validation from persistent RDF store
  - SPARQL-based block metadata extraction and validation
  - Block hash integrity verification with RDF canonicalization
  - Missing block detection and corrupted block identification
  - Metadata consistency checks (timestamp, hash, previous_hash, data_graph_iri)
  - Graph naming consistency validation (expected vs actual URIs)
  - Round-trip validation for RDF data extraction

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
