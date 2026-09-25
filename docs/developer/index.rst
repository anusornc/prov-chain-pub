Developer Documentation
======================

Comprehensive guides, API references, and technical resources for building applications with ProvChainOrg.

.. note::
   **Documentation Status**: This section is under active development. Many of the detailed guides referenced below are still being written. For the most current information, please refer to the main project documentation.

   **Current Capability Boundary**: The repository contains public-RDF/Oxigraph foundations,
   selected ontology checks, local PoA and networking scaffolding, and focused benchmark
   artifacts. It does not yet provide the complete-envelope journal/replay contract, universal
   Final Admission, authenticated membership, reproducible three-node PoA convergence, the
   durable privacy lifecycle, or the bounded ProvChain-to-ProvChain bridge. PBFT, operational
   deployment, and production-pilot controls are future work. Test-file names do not establish
   those pending end-to-end properties.

   **Key Resources Available**:
   - **[Contributing Guide](../../CONTRIBUTING.md)** - Development setup and contribution guidelines
   - **[Architecture Documentation](../architecture/README.md)** - C4 model architecture and design decisions
   - **[Local Execution Guide](../Run.md)** - Development/reference execution; operational deployment remains future work
   - **[Project AGENTS.md](../../AGENTS.md)** - Project patterns, source-of-truth routing, and coding standards
   - **[Thesis-Code Alignment Review](../reviews/THESIS_CODE_ALIGNMENT_REVIEW_2026-07-10.md)** - Current implementation and evidence gaps

Getting Started
---------------

New to ProvChainOrg development? Start with these resources:

**Prerequisites**
Before you begin development with ProvChainOrg, ensure you have:

- **Rust 1.87+**: `rustc --version`
- **Git**: For version control
- **Docker**: For containerized deployment (optional)
- **Python 3.7+**: For client library development (optional)

**Quick Start**

.. code-block:: bash
   # Install Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install development tools
   rustup component add clippy rustfmt
   cargo install cargo-watch cargo-audit

   # Clone and build ProvChainOrg
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org
   cargo build

**Key Developer Resources**

1. **[../../CONTRIBUTING.md](../../CONTRIBUTING.md)**: Complete contributor guide
   - Development setup instructions
   - Project structure overview
   - Coding standards and conventions
   - Pull request process

2. **[../architecture/README.md](../architecture/README.md)**: Architecture documentation
   - C4 Model diagrams
   - Architectural Decision Records (ADRs)
   - Technology stack details

3. **[../../AGENTS.md](../../AGENTS.md)**: Project patterns and current source-of-truth routing
   - Error handling patterns
   - Async runtime usage
   - Security best practices

Development Environment
-----------------------

Setting up your development environment for maximum productivity:

**Core Development Tools**
1. **Rust Toolchain**: Primary development language
2. **Cargo**: Package manager and build tool
3. **Clippy**: Linting and code quality
4. **Rustfmt**: Code formatting
5. **Criterion**: Performance benchmarking

**IDE and Editor Support**
- **Visual Studio Code**: With rust-analyzer extension
- **IntelliJ IDEA**: With Rust plugin
- **Vim/Neovim**: With rust.vim plugin
- **Emacs**: With rust-mode

**Build Commands**

.. code-block:: bash
   # Build the project
   cargo build

   # Run tests
   cargo test

   # Run with logging
   RUST_LOG=debug cargo run

   # Run benchmarks
   cargo bench

API Documentation
-----------------

**Core APIs**

ProvChainOrg provides several interfaces for application development:

1. **REST API**: HTTP-based interface for standard operations
   - JWT authentication required
   - Endpoints for blockchain operations
   - SPARQL query interface

2. **SPARQL API**: Semantic query interface for complex data analysis
   - Standard SPARQL 1.1 protocol
   - Ontology-aware querying
   - Custom reasoning support

3. **WebSocket API**: Real-time communication for event-driven applications
   - Block subscription
   - Real-time updates

**Quick API Example**

.. code-block:: bash
   # Get JWT token
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"demo","password":"demo"}'

   # Submit RDF data
   curl -X POST http://localhost:8080/api/transactions \
     -H "Authorization: Bearer YOUR_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"triples": "@prefix : <#> . :s :p :o ."}'

Architecture Overview
--------------------

**Key Architectural Components**

1. **Blockchain Engine** (`src/core/`): Core consensus and block management
   - Block structure and validation
   - Ed25519 digital signatures
   - Chain state management

2. **RDF Store** (`src/storage/`): Semantic data storage and querying
   - Oxigraph triplestore backend
   - Persistent RDF storage
   - SPARQL query processing

3. **Ontology Package Integration** (`src/ontology/` + `Cargo.toml`): production orchestration path
   - Package/profile loading and selected SHACL constraint support
   - SPACL-backed subclass-aware checks on focused paths
   - Complete package-declared staged-union enforcement remains pending

4. **Network Layer** (`src/network/`): Peer-to-peer communication
   - Local PoA reference candidate; exact three-node convergence/recovery evidence is pending
   - Experimental PBFT code is not a current supported consensus profile
   - WebSocket and peer-discovery scaffolding; authenticated membership and exact-envelope
     replication are pending

5. **API Layer** (`src/web/`): External interface management
   - REST API handlers
   - JWT authentication
   - SPARQL endpoint

**Module Structure**

```
provchain-org/
├── src/
│   ├── core/           # Blockchain core (block, state, signatures)
│   ├── storage/        # RDF storage and persistence
│   ├── network/        # P2P networking and consensus
│   ├── semantic/       # Legacy/demo OWL modules plus ontology/shape assets
│   ├── security/       # Encryption and wallet management
│   ├── integrity/      # Blockchain validation
│   ├── interop/        # Legacy in-process bridge prototype; ADR 0037 target pending
│   ├── web/            # REST API and JWT auth
│   └── analytics/      # Performance monitoring
└── tests/              # Integration tests
```

Testing Framework
-----------------

**Testing Tools and Frameworks**

1. **Unit Testing**: Rust's built-in testing framework
   - Inline tests in source files
   - Module-level test organization

2. **Integration Testing**: Component and integration-path checks
   - `tests/` directory for focused integration fixtures
   - Not proof of the pending thesis-reference end-to-end contract

3. **Performance Testing**: Criterion.rs for benchmarking
   - Microbenchmarking in `benches/`
   - Statistical analysis (95% confidence intervals)

4. **Load Testing**: Profile-gated custom test scaffolding
   - Useful for harness development and diagnostics
   - No ledger-throughput value from the legacy custom harness is admitted as current evidence

**Running Tests**

.. code-block:: bash
   # Run all tests
   cargo test --workspace

   # Run specific test file
   cargo test --test load_tests -- --ignored

   # Run benchmarks
   cargo bench

   # Run OWL2 integration tests
   cargo test --test owl2_feature_tests

**Test Coverage**

Key test files include:
- `tests/project_requirements_test.rs` - Legacy component/requirements checks; not three-node
  convergence or bounded-bridge evidence
- `tests/privacy_test.rs` - Cryptographic and wallet component checks; not the durable privacy lifecycle
- `tests/enhanced_traceability_demo.rs` - Traceability validation
- `tests/load_tests.rs` - Profile-gated custom load-test scaffolding; its former summary
  statistics are withdrawn from the publication evidence path
- `tests/owl2_*` and `tests/enhanced_owl2_*` - OWL2 integration test suites

Security Guidelines
-------------------

**Security Best Practices**

1. **Input Validation**: Sanitizing all external data
2. **Authentication**: JWT-based API authentication
3. **Authorization**: Role-based access control
4. **Data Encryption**: ChaCha20-Poly1305 for private data
5. **Digital Signatures**: Ed25519 for block signing
6. **Audit Logging**: Comprehensive security logging

**Key Security Features**

- **JWT Authentication**: Secure API access with configurable secrets
- **Ed25519 Signatures**: Each blockchain instance has unique signing key
- **Privacy Encryption**: Optional triple-level encryption with ChaCha20-Poly1305
- **Wallet Management**: Argon2-based key derivation for wallet encryption
- **Key Rotation**: 90-day recommended signing key rotation interval

**Development Workflow**

.. code-block:: bash
   # Fork and clone the repository
   git clone https://github.com/your-username/provchain-org.git
   cd provchain-org

   # Create feature branch
   git checkout -b feature/new-feature

   # Make changes and test
   cargo test
   cargo clippy
   cargo fmt

   # Commit and push
   git commit -am "Add new feature"
   git push origin feature/new-feature

   # Create pull request

Performance Optimization
------------------------

**Key Optimization Areas**

1. **Query Optimization**: Efficient SPARQL query patterns
2. **Caching**: Memory and disk caching strategies
3. **Parallel Processing**: Concurrent operation handling
4. **Resource Management**: CPU and memory optimization
5. **Network Efficiency**: Bandwidth and latency optimization

**Performance Benchmarks**

Use the `benchmark evidence boundary
<../benchmarking/BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md>`_ and `paper evidence index
<../paper_submission/PAPER_EVIDENCE_INDEX.md>`_ before quoting a measurement. Current admitted
quantitative evidence is family-scoped: focused ontology-admission Criterion artifacts and curated
trace-query campaigns. No corrected ledger-throughput value is currently admitted.
`EXPERIMENTAL_RESULTS.md <../benchmarking/EXPERIMENTAL_RESULTS.md>`_ is a dated mixed historical
record whose custom load-test section has been withdrawn.

Deployment Guides
-----------------

**Deployment References**

- `Run.md <../Run.md>`_: local development/reference execution
- `Container Architecture <../architecture/CONTAINER_ARCHITECTURE.md>`_: target/reference topology,
  not operational deployment evidence

Operational deployment and production-pilot controls remain future work.

**Deployment Scenarios**

1. **Single Node**: Development and testing environments
2. **Multi-Node Network**: Future controlled three-node PoA convergence/recovery evidence
3. **Docker Deployment**: Containerized setups
4. **Benchmark Comparison**: Performance testing with baseline systems

**Container Scaffolding**

`deploy/docker-compose.node.yml` and `deploy/docker-compose.3node.yml` are development/reference
configuration, not operational deployment or convergence evidence. Verify them against the current
working plan before use; there is no `deploy/docker-compose.quickstart.yml` in this revision.

**Configuration Management**

Configuration is managed via `config.toml` at the project root:

.. code-block:: toml
   [network]
   listen_port = 8080
   known_peers = ["192.168.1.100:8080"]

   [storage]
   data_dir = "./data"
   persistent = true

   [consensus]
   is_authority = false

   [web]
   host = "127.0.0.1"
   port = 8080
   jwt_secret = "your-secret-key-here"

Troubleshooting
---------------

**Common Issues**

1. **Build Issues**: Dependency conflicts and compilation errors
2. **Runtime Errors**: Configuration problems and data issues
3. **Performance Problems**: Slow queries and high resource usage
4. **Network Issues**: Connectivity problems and synchronization failures
5. **Security Issues**: Authentication failures and access problems

**Debugging Tools**

.. code-block:: bash
   # Enable debug logging
   export RUST_LOG=debug
   cargo run

   # Run with specific log level
   export RUST_LOG=provchain=trace
   cargo run

   # Profile performance
   cargo bench

   # Check for security vulnerabilities
   cargo audit

   # Run health checks
   curl http://localhost:8080/health

Community and Support
---------------------

**Support Channels**

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Technical discussions and Q&A
- **Contributing Guide**: See `[../../CONTRIBUTING.md](../../CONTRIBUTING.md)`

**Documentation Resources**

- **[README.md](../../README.md)**: Project overview and quick start
- **[Plan.md](../Plan.md)**: Detailed project roadmap
- **[Run.md](../Run.md)**: Execution instructions
- **[USER_MANUAL.md](../USER_MANUAL.md)**: End-user documentation

**Further Reading**

- **[../architecture/ADR/](../architecture/ADR/)**: Architectural Decision Records
- **[../project-health/](../project-health/)**: Project health analysis
- **[../benchmarking/](../benchmarking/)**: Performance benchmarking guides

.. note::
   The ProvChainOrg developer documentation is continuously evolving. For the most current information, refer to the main project README and the contributing guide. If you have suggestions for additional documentation, please contribute through our GitHub repository.
