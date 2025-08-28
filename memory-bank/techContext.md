# Tech Context - Technologies and Development Setup

## Core Technology Stack

### Frontend Technology Stack

#### React Framework
**React (v18+) with TypeScript**
- **Component-Based Architecture**: Reusable, modular UI components
- **Type Safety**: TypeScript for compile-time error checking
- **State Management**: React Context API and Hooks
- **Performance**: Virtual DOM and efficient rendering
- **Developer Experience**: Hot reloading, debugging tools

#### Build Tools
**Vite (v7+)**
- **Fast Development Server**: Instant server start and hot module replacement
- **Optimized Production Build**: Tree-shaking and code splitting
- **TypeScript Support**: Built-in TypeScript compilation
- **Plugin Ecosystem**: Rich ecosystem of plugins and integrations

#### UI Components and Styling
**Custom Design System**
- **Component Library**: Button, Card, Badge, Alert, LoadingSpinner, Input, TextArea
- **Styling**: CSS Modules with utility-first approach
- **Responsive Design**: Mobile-first responsive components
- **Accessibility**: WCAG 2.1 AA compliant components
- **Theming**: Dark/light mode support with context provider

#### Data Visualization
**Cytoscape.js**
- **Graph Visualization**: Interactive network and relationship diagrams
- **Layout Algorithms**: Multiple layout options for different use cases
- **Styling**: Custom node and edge styling
- **Interaction**: Click, hover, and selection events

#### HTTP Client
**Axios**
- **Promise-Based**: Modern promise-based HTTP client
- **Interceptors**: Request/response interceptors for authentication
- **Error Handling**: Comprehensive error handling capabilities
- **TypeScript Support**: Type definitions for better developer experience

### Programming Language
**Rust (Edition 2021)**
- **Why Rust**: Memory safety, performance, concurrency, strong type system
- **Version**: 1.70+ required
- **Key Features Used**: Async/await, traits, generics, error handling with `Result<T, E>`
- **Memory Management**: Zero-cost abstractions, ownership system prevents data races
- **Performance**: Compiled to native code, minimal runtime overhead

### RDF and Semantic Web Technologies

#### Oxigraph (v0.4)
- **Role**: Core RDF triplestore and SPARQL engine
- **Features**: Named graphs, SPARQL 1.1, RDF 1.1 compliance
- **Storage**: In-memory and persistent (RocksDB backend)
- **Performance**: Optimized for read-heavy workloads
- **Integration**: Native Rust implementation, no FFI overhead

#### RDF Standards Compliance
- **RDF 1.1**: Full support for triples, quads, named graphs
- **SPARQL 1.1**: Query, update, federated query capabilities
- **Turtle**: Primary serialization format for human readability
- **N-Quads**: Backup and bulk data format
- **RDFC-1.0**: W3C RDF canonicalization for consistent hashing

#### Ontology Technologies
- **OWL 2**: Web Ontology Language for domain modeling
- **PROV-O**: W3C Provenance Ontology as foundation
- **SHACL**: Shapes Constraint Language for data validation
- **Custom OWL2 Library**: `owl2_rs` for advanced reasoning

### Cryptography and Security

#### Ed25519 Digital Signatures
- **Library**: `ed25519-dalek` v2.0
- **Features**: Fast signing/verification, small signatures (64 bytes)
- **Use Cases**: Transaction signing, participant authentication
- **Security**: Curve25519 elliptic curve, resistant to side-channel attacks

#### Hashing
- **SHA-256**: `sha2` crate for block hashing
- **Hex Encoding**: `hex` crate for hash representation
- **RDF Canonicalization**: Custom + W3C RDFC-1.0 algorithms
- **Performance**: Hardware acceleration when available

#### Random Number Generation
- **Library**: `rand` v0.8
- **CSPRNG**: Cryptographically secure random number generation
- **Use Cases**: Key generation, nonce creation, UUID generation

### Networking and Concurrency

#### Tokio Async Runtime
- **Version**: v1.0 with "full" features
- **Capabilities**: Async I/O, timers, filesystem, networking
- **Thread Pool**: Work-stealing scheduler for CPU-bound tasks
- **Integration**: Foundation for all async operations

#### WebSocket Communication
- **Library**: `tokio-tungstenite` v0.20
- **Protocol**: WebSocket for P2P communication
- **Features**: Binary/text frames, compression, ping/pong
- **Use Cases**: Peer discovery, block synchronization, consensus

#### UUID Generation
- **Library**: `uuid` v1.0 with v4 and serde features
- **Use Cases**: Participant IDs, transaction IDs, message correlation
- **Format**: RFC 4122 compliant UUIDs

### Web Framework and API

#### Axum Web Framework
- **Version**: v0.7
- **Architecture**: Modular, composable, type-safe routing
- **Features**: Middleware, extractors, response types
- **Performance**: Built on Hyper, minimal overhead
- **Integration**: Native Tokio integration

#### HTTP and Middleware
- **Hyper**: v1.0 HTTP implementation
- **Tower**: v0.4 middleware and service abstractions
- **Tower-HTTP**: v0.5 with CORS, static files, headers
- **MIME**: v0.3 for content type handling

#### Authentication
- **JWT**: `jsonwebtoken` v9.2 for stateless authentication
- **Bcrypt**: v0.15 for password hashing
- **Features**: Token validation, role-based access control

### Data Serialization and Configuration

#### Serde Ecosystem
- **Core**: `serde` v1.0 with derive features
- **JSON**: `serde_json` v1.0 for API responses
- **YAML**: `serde_yaml` v0.9 for configuration
- **Features**: Custom serializers, field renaming, validation

#### Configuration Management
- **Config**: v0.13 for layered configuration
- **TOML**: v0.8 for configuration file format
- **Environment**: Support for environment variable overrides
- **Validation**: Type-safe configuration with defaults

#### Time and Dates
- **Chrono**: v0.4 with serde features
- **Features**: ISO 8601 timestamps, timezone handling
- **Blockchain**: Consistent timestamp format across blocks
- **Serialization**: JSON-compatible date formats

### Storage and Persistence

#### RocksDB (via Oxigraph)
- **Role**: Persistent storage backend for RDF data
- **Features**: LSM-tree storage, compression, transactions
- **Performance**: Optimized for write-heavy workloads
- **Backup**: Point-in-time snapshots and restoration

#### Compression
- **LZ4**: v1.24 for fast compression/decompression
- **Use Cases**: Network message compression, storage optimization
- **Performance**: Prioritizes speed over compression ratio

#### Encoding
- **Base64**: v0.21 for binary data encoding
- **Use Cases**: Cryptographic keys, binary data in JSON
- **Standards**: RFC 4648 compliant encoding

### Development and Testing

#### CLI Framework
- **Clap**: v4.5 with derive features
- **Features**: Subcommands, argument validation, help generation
- **Integration**: Type-safe command-line interface

#### Error Handling
- **Anyhow**: v1.0 for application errors
- **ThisError**: v1.0 for library errors
- **Pattern**: `Result<T, E>` throughout codebase
- **Context**: Rich error context and chaining

#### Logging and Monitoring
- **Tracing**: v0.1 for structured logging
- **Tracing-Subscriber**: v0.3 for log formatting
- **Env-Logger**: v0.10 for environment-based configuration
- **Log**: v0.4 as logging facade

#### Testing Framework
- **Built-in**: Rust's built-in test framework
- **Tempfile**: v3.8 for temporary test files
- **Reqwest**: v0.11 for HTTP client testing
- **Coverage**: 27 tests across 8 test suites

### Performance and Analytics

#### Benchmarking
- **Criterion**: v0.5 with HTML reports
- **Features**: Statistical analysis, regression detection
- **Benchmarks**: Canonicalization, consensus, trace optimization

#### Numerical Computing
- **NDArray**: v0.15 for multi-dimensional arrays
- **Use Cases**: Analytics, machine learning, statistics
- **Performance**: BLAS integration for linear algebra

#### Graph Algorithms
- **Petgraph**: v0.6 for graph data structures
- **Use Cases**: Knowledge graph analysis, trace optimization
- **Algorithms**: Shortest path, centrality, clustering

#### Geographic Computing
- **Geo**: v0.26 for geographic computations
- **Use Cases**: Location-based traceability, supply chain mapping
- **Features**: Distance calculations, geometric operations

### Visualization and Reporting

#### Chart Generation
- **Plotters**: v0.3 for chart and graph generation
- **Output**: PNG, SVG, HTML canvas
- **Use Cases**: Analytics dashboards, performance reports

#### Template Engines
- **Handlebars**: v4.0 for report templates
- **Askama**: v0.12 with Axum integration for web templates
- **Use Cases**: HTML reports, email templates, documentation

### Production and Monitoring

#### Metrics Collection
- **Prometheus**: v0.13 for metrics collection
- **Metrics**: v0.22 as metrics facade
- **Metrics-Exporter-Prometheus**: v0.13 for Prometheus integration
- **Features**: Counters, gauges, histograms, labels

#### Distributed Tracing
- **OpenTelemetry**: v0.21 for distributed tracing
- **OpenTelemetry-Jaeger**: v0.20 for Jaeger backend
- **Features**: Span tracking, trace correlation, performance analysis

#### System Information
- **Sysinfo**: v0.30 for system metrics
- **Num-CPUs**: v1.0 for CPU core detection
- **Use Cases**: Resource monitoring, thread pool sizing

#### Utilities
- **Dirs**: v5.0 for standard directory locations
- **Lazy-Static**: v1.4 for static initialization
- **Regex**: v1.0 for pattern matching
- **Async-Trait**: v0.1 for async traits

## Development Environment Setup

### Prerequisites
```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Development tools
rustup component add clippy rustfmt
cargo install cargo-watch cargo-audit
```

### Project Structure
```
provchain-org/
├── Cargo.toml              # Project configuration and dependencies
├── src/                    # Source code
│   ├── lib.rs             # Library exports
│   ├── main.rs            # CLI application
│   ├── core/              # Core blockchain
│   ├── storage/           # RDF storage
│   ├── transaction/       # Transaction system
│   ├── network/           # P2P networking
│   ├── semantic/          # Ontology integration
│   ├── web/               # Web API
│   └── ...
├── tests/                 # Integration tests
├── benches/               # Performance benchmarks
├── ontologies/            # Domain ontologies
├── config/                # Configuration files
├── test_data/             # Sample RDF data
└── queries/               # SPARQL queries
```

### Build Configuration
```toml
[package]
name = "provchain-org"
version = "0.1.0"
edition = "2021"

[lib]
name = "provchain_org"
path = "src/lib.rs"

[[bin]]
name = "provchain-org"
path = "src/main.rs"

[features]
default = []
e2e = []  # End-to-end testing
```

### Development Commands
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Run with logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

### Configuration Files

#### Main Configuration (`config.toml`)
```toml
[network]
network_id = "provchain-org-default"
listen_port = 8080
known_peers = ["127.0.0.1:8081"]

[storage]
data_dir = "./data"
persistent = true
store_type = "oxigraph"

[ontology]
path = "ontologies/generic_core.owl"
auto_load = true
validate_data = false
```

#### Ontology Configuration (`config/ontology.toml`)
```toml
[main_ontology]
path = "ontologies/generic_core.owl"
graph_iri = "http://provchain.org/ontology/core"
auto_load = true

[domain_ontologies.supply_chain]
path = "ontologies/supply-chain.owl"
enabled = true
priority = 100
```

### Environment Variables
```bash
# Network configuration
PROVCHAIN_PORT=8080
PROVCHAIN_PEERS="127.0.0.1:8081,127.0.0.1:8082"
PROVCHAIN_AUTHORITY=true

# Storage configuration
PROVCHAIN_DATA_DIR="./data"
PROVCHAIN_PERSISTENT=true

# Ontology configuration
ONTOLOGY_MAIN_PATH="ontologies/generic_core.owl"
ONTOLOGY_AUTO_LOAD=true

# Logging configuration
RUST_LOG=info
RUST_BACKTRACE=1
```

## Performance Characteristics

### Benchmarking Results
- **RDF Canonicalization**: ~1ms for typical supply chain graphs
- **Block Creation**: ~5ms including RDF storage and hashing
- **SPARQL Queries**: ~10ms for complex traceability queries
- **Transaction Validation**: ~2ms including signature verification

### Memory Usage
- **Base Runtime**: ~50MB for core blockchain and RDF store
- **Per Block**: ~1KB metadata + variable RDF data size
- **Caching**: Configurable memory cache (default 100MB)
- **Peak Usage**: Scales with active dataset size

### Storage Requirements
- **RDF Data**: ~2-5x overhead compared to raw data
- **Blockchain Metadata**: ~1KB per block
- **Indexes**: ~20% of RDF data size
- **Compression**: LZ4 compression reduces storage by ~30%

## Integration Patterns

### REST API Integration
```rust
// Example API client usage
let client = reqwest::Client::new();
let response = client
    .post("http://localhost:8080/api/v1/query")
    .json(&sparql_query)
    .send()
    .await?;
```

### Configuration Integration
```rust
// Example configuration loading
let config = Config::builder()
    .add_source(config::File::with_name("config"))
    .add_source(config::Environment::with_prefix("PROVCHAIN"))
    .build()?;
```

### Async Integration
```rust
// Example async blockchain operation
#[tokio::main]
async fn main() -> Result<()> {
    let blockchain = Blockchain::new().await?;
    let block = blockchain.add_block(rdf_data).await?;
    Ok(())
}
```

This technical context provides the foundation for understanding and working with the ProvChainOrg codebase, covering all major technologies, dependencies, and development practices.
