# ProvChainOrg

## Project Overview

ProvChainOrg is a distributed blockchain system in Rust that enhances blockchain with embedded ontology and knowledge graph for data traceability. It extends the "GraphChain" concept with semantic technologies, providing high-speed traceability, configurable consensus (PoA/PBFT), and cross-chain interoperability.

## What's New (January 2026)

- **Baseline Comparison Infrastructure**: Native ProvChain + Docker baselines (Neo4j, Jena, Ethereum) for journal publication
- **Zero Clippy Warnings**: All source code, benchmarks, and tests now pass clippy with default settings

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

# Run baseline comparison experiments (journal publication)
cd docs/publication
docker-compose -f docker-compose.baseline-comparison.yml up -d
docker-compose -f docker-compose.baseline-comparison.yml run --rm benchmark-runner
docker-compose -f docker-compose.baseline-comparison.yml down

# Run baseline systems only (Neo4j, Jena, Ethereum) - ProvChainOrg runs on host
cd docs/publication
sudo docker compose -f docker-compose.baselines-only.yml up -d
sudo docker compose -f docker-compose.baselines-only.yml down

# Service management for native ProvChain (baseline comparison)
./scripts/provchain-service.sh start    # Start native ProvChain service
./scripts/provchain-service.sh stop     # Stop native ProvChain service
./scripts/provchain-service.sh status   # Check service status
./scripts/provchain-service.sh health   # Run health checks
./scripts/provchain-service.sh logs     # View service logs

# Test native ProvChain instance
./scripts/test-provchain.sh             # Run integration tests (health, JWT, transactions, SPARQL)

# Complete baseline comparison workflow (native ProvChain + Docker baselines)
./scripts/run-benchmark-comparison.sh   # Orchestrates full baseline comparison experiment
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

### Production Infrastructure
- `src/production/` - Enterprise-grade production deployment features
  - `security.rs` - Security hardening with TLS, rate limiting, CORS, audit logging, JWT authentication, and configurable security policies (password, session, API access)
  - `compliance.rs` - GDPR compliance framework with data handling policies
  - `monitoring.rs` - Prometheus metrics and observability
  - `container.rs` - Docker/Kubernetes orchestration support
  - `deployment.rs` - Production deployment configuration

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
- `config.toml` - Node configuration (consensus type, network settings, storage, web server, CORS)
  - Development mode includes default JWT secret (for demo/testing only)
  - Production: Override with `JWT_SECRET` environment variable for secure authentication
  - Network configuration: peers, ports, timeouts, connection limits
  - Consensus settings: authority mode, block interval, size limits
  - Storage: persistent RDF store with configurable cache size
  - Web server: host, port, JWT authentication, CORS settings
- Environment: `JWT_SECRET` overrides config file value for production security

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

#### Production Security Features
- TLS/SSL configuration with certificate management
- Rate limiting and DoS protection (configurable per-minute limits)
- CORS policy enforcement with origin whitelisting
- Security headers (CSP, X-Frame-Options, HSTS)
- Audit logging for security events (authentication, authorization, data access)
- Configurable security policies:
  - Password policy (length, complexity requirements)
  - Session policy (duration, idle timeout, concurrent sessions)
  - API access policy (authentication/authorization requirements)
  - Data access policy (RBAC, data classification)
  - Network access policy (IP whitelisting, firewall rules)
- GDPR compliance framework with right-to-be-forgotten implementation

## Testing

### Test Structure
- `tests/` - Integration tests (main project)
- `owl2-reasoner/tests/` - owl2-reasoner sub-project tests
- Unit tests alongside source code
- Benchmark suites in `benches/` (main) and `owl2-reasoner/benches/` (sub-project)
- Load tests in `tests/load_tests.rs`
- `tests/production_security_tests.rs` - Production security test suite (JWT validation, rate limiting, GDPR compliance, security policies)
- `tests/websocket_integration_tests.rs` - WebSocket integration tests (connection management, event broadcasting, multi-client scenarios)

### owl2-reasoner Sub-Project
The project includes `owl2-reasoner` as a workspace member - a high-performance OWL2 reasoning engine.

**Test Commands for owl2-reasoner:**
```bash
# Test owl2-reasoner specifically
cargo test -p owl2-reasoner

# Run specific test file in owl2-reasoner
cargo test -p owl2-reasoner --test turtle_parser_tests

# Run owl2-reasoner benchmarks
cargo bench -p owl2-reasoner

# Verify owl2-reasoner benchmarks compile
cargo check --benches --package owl2-reasoner

# Run tests for entire workspace
cargo test --workspace
```

**owl2-reasoner Structure:**
- `src/ontology/` - Ontology management with indexed storage
- `src/reasoning/` - Tableaux algorithm and rule-based inference
  - `query/` - Query engine with caching and optimization (API updated in commit d5ca53a)
  - `tableaux/` - Tableaux reasoning implementation
- `src/parser/` - Multi-format RDF parsing (Turtle, RDF/XML, N-Triples)
- `tests/` - 12 test files covering parsing, reasoning, and performance
  - **Turtle parser tests: 12/12 passing** (fixed malformed syntax, missing subjects)
- `benches/` - 27 benchmark suites for performance validation
  - **All benchmarks now compile** after API migration (execute_query → execute)

### Key Test Files
- Integration tests in `tests/` directory
- owl2-reasoner tests in `owl2-reasoner/tests/`
- Run `cargo test` for all tests
- See individual test files for specific functionality coverage

### Benchmarking
- See `BENCHMARKING.md` for comprehensive benchmarking documentation
- Portable toolkit: `cd benchmark-toolkit && ./run.sh`
- Research benchmarks: `docs/benchmarking/`
- Cargo benchmarks: `cargo bench`
- Baseline comparisons: `docs/publication/` (Neo4j, Jena, Ethereum)

## Deployment

### Deployment
- Quick start: `deploy/README_QUICKSTART.md`
- Multi-node: `deploy/docker-compose.3node.yml`
- Production: `deploy/docker-compose.production.yml`
- See `deploy/` directory for all deployment options

### Environment Variables
- `JWT_SECRET` - Required for API authentication (32+ chars)
- Node-specific settings in `config.toml`

## Documentation

See `docs/INDEX.md` for complete documentation navigation.

**Key entry points:**
- `README.md` - Project overview and quick start
- `CONTRIBUTING.md` - Development setup and coding standards
- `docs/INDEX.md` - Complete documentation index with navigation
- `docs/architecture/README.md` - Architecture documentation (C4 model, ADRs)
- `docs/USER_MANUAL.md` - User guide
- `docs/publication/` - Journal publication package with baseline comparisons
- `BENCHMARKING.md` - Performance testing resources

**Documentation Organization:**
- `docs/reviews/` - Code review and analysis reports (CODE_REVIEW_PRODUCTION_FEATURES.md, PBFT_CONSENSUS_CODE_REVIEW.md)
- `docs/security/` - Security documentation and test coverage (SECURITY_SETUP.md, SECURITY_TEST_COVERAGE_REPORT.md)
- `docs/project-health/` - Test coverage and code quality reports (TEST_COVERAGE_REPORT.md, clippy_analysis_*.md)
- `docs/benchmarking/` - Research-focused benchmarking and experimental results
- `docs/deployment/` - Deployment guides and Docker configurations
- `docs/archive/` - Historical documentation and implementation records
- `FILE_ORGANIZATION.md` - File placement standards for AI agents (root directory reference)

## Development Notes


- This is a research project for thesis: "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"
- Focus on semantic web standards: RDF, SPARQL, SHACL, OWL2
- Implements scientific benchmarking for semantic overhead evaluation
- Cross-chain bridge uses lock-and-mint pattern with SHACL validation

## Security Notes

- All security vulnerabilities documented in `Cargo.toml` with risk assessments
- Key dependencies: Ed25519 signatures, ChaCha20-Poly1305 encryption, JWT authentication
- See `Cargo.toml` for complete security advisory details
