Developer Documentation
======================

Comprehensive guides, API references, and technical resources for building applications with ProvChainOrg.

.. note::
   **Documentation Status**: This section is under active development. Many of the detailed guides referenced below are still being written. For the most current information, please refer to the main project documentation.

   **Key Resources Available**:
   - **[Contributing Guide](../../CONTRIBUTING.md)** - Development setup and contribution guidelines
   - **[Architecture Documentation](../architecture/README.md)** - C4 model architecture and design decisions
   - **[Deployment Guide](../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md)** - Deployment instructions
   - **[Project CLAUDE.md](../../CLAUDE.md)** - Project patterns and coding standards

Getting Started
---------------

New to ProvChainOrg development? Start with these resources:

**Prerequisites**
Before you begin development with ProvChainOrg, ensure you have:

- **Rust 1.70+**: `rustc --version`
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

3. **[../../CLAUDE.md](../../CLAUDE.md)**: Project patterns
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

3. **OWL2 Reasoner** (`owl2-reasoner/`): Semantic reasoning engine
   - Tableaux algorithm implementation
   - OWL2 RL support
   - Query optimization

4. **Network Layer** (`src/network/`): Peer-to-peer communication
   - Consensus protocols (PoA/PBFT)
   - WebSocket communication
   - Peer discovery

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
│   ├── semantic/       # OWL2 reasoning and SHACL validation
│   ├── security/       # Encryption and wallet management
│   ├── integrity/      # Blockchain validation
│   ├── interop/        # Cross-chain bridge
│   ├── web/            # REST API and JWT auth
│   └── analytics/      # Performance monitoring
├── owl2-reasoner/      # OWL2 reasoning sub-project
└── tests/              # Integration tests
```

Testing Framework
-----------------

**Testing Tools and Frameworks**

1. **Unit Testing**: Rust's built-in testing framework
   - Inline tests in source files
   - Module-level test organization

2. **Integration Testing**: End-to-end system testing
   - `tests/` directory for integration tests
   - Full system validation

3. **Performance Testing**: Criterion.rs for benchmarking
   - Microbenchmarking in `benches/`
   - Statistical analysis (95% confidence intervals)

4. **Load Testing**: High-volume transaction testing
   - Concurrent user simulation
   - Throughput measurement

**Running Tests**

.. code-block:: bash
   # Run all tests
   cargo test --workspace

   # Run specific test file
   cargo test --test load_tests

   # Run benchmarks
   cargo bench

   # Run owl2-reasoner tests
   cargo test -p owl2-reasoner

**Test Coverage**

Key test files include:
- `tests/project_requirements_test.rs` - Consensus and bridge validation
- `tests/privacy_test.rs` - Encryption and wallet tests
- `tests/enhanced_traceability_demo.rs` - Traceability validation
- `tests/load_tests.rs` - Performance testing (200 users × 100 requests)
- `owl2-reasoner/tests/` - OWL2 reasoner test suite (12 test files)

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

Current performance measurements (development environment):

- **OWL2 Reasoning**: 0.015-0.17ms per axiom
- **SPARQL Queries**: 0.04-18ms (P95 < 100ms target ✅)
- **Memory Usage**: ~200MB baseline
- **Write Throughput**: 19.58 TPS (single-node dev environment)

See `[../benchmarking/EXPERIMENTAL_RESULTS.md](../benchmarking/EXPERIMENTAL_RESULTS.md)` for detailed experimental results.

Deployment Guides
-----------------

**Deployment Documentation**

- **[../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md](../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md)**: Comprehensive deployment guide
- **[../deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md](../deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md)**: Docker deployment architecture

**Deployment Scenarios**

1. **Single Node**: Development and testing environments
2. **Multi-Node Network**: Production deployments
3. **Docker Deployment**: Containerized setups
4. **Benchmark Comparison**: Performance testing with baseline systems

**Quick Docker Deployment**

.. code-block:: bash
   # Quick start single-node deployment
   cd deploy
   docker-compose -f docker-compose.quickstart.yml up -d

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
