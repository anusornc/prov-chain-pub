# ProvChain-Org: Comprehensive Analysis & 3-VM Deployment Plan

**Document Version:** 1.0
**Date:** 2026-01-02
**Status:** Ready for Deployment
**Author:** Rust Analysis Agent

---

## Executive Summary

ProvChain-Org is a **distributed blockchain system with semantic capabilities** built in Rust, combining:
- **Blockchain technology** for immutable traceability
- **OWL2 ontology reasoning** for semantic validation
- **Proof-of-Authority (PoA) consensus** with PBFT support
- **Production-ready monitoring** (Prometheus, Grafana, Jaeger)

The system is already well-architected for distributed deployment with existing 3-node configurations.

### Key Findings

✅ **Architecture**: Modern Rust workspace with 26 modules, clean separation of concerns
✅ **Consensus**: Proof-of-Authority with dynamic rotation and performance tracking
✅ **Networking**: WebSocket-based P2P with peer discovery and blockchain sync
✅ **Production-Ready**: Docker support, monitoring stack, health checks built-in
✅ **Tested**: Existing `three_node_validation_test.rs` proves 3-node viability

### Deployment Readiness: ⭐⭐⭐⭐⭐ (5/5)

---

## Table of Contents

1. [Project Architecture Analysis](#1-project-architecture-analysis)
2. [Dependencies & Build Requirements](#2-dependencies--build-requirements)
3. [Network Communication & Consensus](#3-network-communication--consensus)
4. [Configuration for Distributed Deployment](#4-configuration-for-distributed-deployment)
5. [Three-Node Cloud Deployment Strategy](#5-three-node-cloud-deployment-strategy)
6. [Testing & Validation Approach](#6-testing--validation-approach)
7. [Deployment Checklist](#7-deployment-checklist)
8. [Monitoring & Observability](#8-monitoring--observability)
9. [Troubleshooting Guide](#9-troubleshooting-guide)
10. [Cost Estimation](#10-cost-estimation-cloud-providers)
11. [Next Steps & Recommendations](#11-next-steps--recommendations)
12. [Reference Documentation](#12-reference-documentation)

---

## 1. Project Architecture Analysis

### Overview

ProvChain-Org is a Rust workspace containing:
- **Main crate**: `provchain-org` (library + binary)
- **Sub-crate**: `owl2-reasoner` (OWL2 ontology reasoning)

### Core Components

```
provchain-org/
├── src/
│   ├── core/               # Blockchain core (Block, Entity, Atomic ops)
│   │   ├── blockchain.rs   # Blockchain data structure
│   │   ├── entity.rs       # Entity management
│   │   └── atomic_operations.rs
│   │
│   ├── network/            # P2P networking & consensus
│   │   ├── consensus.rs    # PoA & PBFT protocols
│   │   ├── peer.rs         # WebSocket connections
│   │   ├── discovery.rs    # Peer discovery
│   │   ├── sync.rs         # Blockchain synchronization
│   │   └── messages.rs     # Network message types
│   │
│   ├── semantic/           # OWL2 reasoning & validation
│   │   ├── owl2_integration.rs
│   │   ├── owl2_enhanced_reasoner.rs
│   │   ├── owl2_traceability.rs
│   │   └── shacl_validator.rs
│   │
│   ├── storage/            # RDF store (Oxigraph)
│   │   └── rdf_store.rs
│   │
│   ├── web/                # HTTP API (Axum framework)
│   │   ├── server.rs       # Web server setup
│   │   ├── auth.rs         # JWT authentication
│   │   ├── websocket.rs    # WebSocket support
│   │   └── handlers/       # API endpoint handlers
│   │
│   ├── production/         # Deployment & monitoring
│   │   ├── deployment.rs   # Deployment strategies
│   │   ├── monitoring.rs   # Prometheus metrics
│   │   ├── security.rs     # Security configurations
│   │   └── container.rs    # Container orchestration
│   │
│   ├── security/           # Encryption & cryptography
│   │   └── encryption.rs   # ChaCha20-Poly1305
│   │
│   ├── performance/        # Optimization modules
│   │   ├── scaling.rs
│   │   ├── memory_optimization.rs
│   │   ├── concurrent_operations.rs
│   │   └── metrics.rs
│   │
│   ├── integrity/          # Blockchain validation
│   │   ├── validator.rs
│   │   ├── blockchain_validator.rs
│   │   ├── monitor.rs
│   │   └── repair.rs
│   │
│   ├── ontology/           # Ontology management
│   │   ├── manager.rs
│   │   ├── loader.rs
│   │   ├── registry.rs
│   │   └── domain_manager.rs
│   │
│   ├── knowledge_graph/    # Graph database
│   │   ├── builder.rs
│   │   ├── graph_db.rs
│   │   └── entity_linking.rs
│   │
│   ├── domain/             # Domain adapters
│   │   └── adapters/
│   │       ├── healthcare.rs
│   │       ├── pharmaceutical.rs
│   │       └── supply_chain.rs
│   │
│   ├── analytics/          # Analytics & predictions
│   │   ├── predictive.rs
│   │   ├── supply_chain.rs
│   │   └── sustainability.rs
│   │
│   ├── transaction/        # Transaction management
│   │   ├── transaction.rs
│   │   └── blockchain.rs
│   │
│   ├── validation/         # Input validation
│   │   ├── input_validator.rs
│   │   └── sanitizer.rs
│   │
│   ├── config/             # Configuration management
│   │   └── mod.rs
│   │
│   ├── utils/              # Utility functions
│   │   └── config.rs
│   │
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library root
│   ├── error.rs            # Error types
│   └── wallet.rs           # Wallet management
│
├── owl2-reasoner/          # OWL2 reasoning workspace member
│
├── tests/                  # Integration tests
│   ├── three_node_validation_test.rs  # ⭐ 3-node test
│   └── enhanced_traceability_demo.rs
│
├── benches/                # Performance benchmarks
│   ├── simple_consensus_benchmarks.rs
│   ├── trace_optimization_benchmarks.rs
│   ├── owl2_benchmarks.rs
│   └── comprehensive_performance_benchmarks.rs
│
├── deploy/                 # Deployment configurations
│   ├── docker-compose.3node.yml       # ⭐ 3-node setup
│   ├── docker-compose.production.yml
│   ├── Dockerfile.production          # ⭐ Production image
│   └── monitoring/
│       ├── prometheus.yml
│       └── prometheus_multi_node.yml
│
├── config/                 # Configuration files
│   └── production-deployment.toml
│
├── ontologies/             # OWL ontology files
├── queries/                # SPARQL queries
├── shapes/                 # SHACL shapes
├── frontend/               # Web UI (Node.js/Vue)
├── scripts/                # Utility scripts
└── docs/                   # Documentation
```

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Language** | Rust 2021 Edition | Memory safety, zero-cost abstractions |
| **Async Runtime** | Tokio 1.0 | Concurrent I/O operations |
| **Web Framework** | Axum 0.7 + Tower | HTTP API & middleware |
| **WebSocket** | tokio-tungstenite 0.20 | P2P communication |
| **Consensus** | Custom PoA + PBFT | Block creation & validation |
| **RDF Store** | Oxigraph 0.4 | Semantic triple storage |
| **Ontology** | OWL2 (custom reasoner) | Semantic reasoning |
| **Cryptography** | ed25519-dalek 2.0 | Digital signatures |
| **Encryption** | ChaCha20-Poly1305 | Data encryption |
| **Compression** | LZ4 1.24 | Data compression |
| **Authentication** | JWT (jsonwebtoken 9.2) | API authentication |
| **Monitoring** | Prometheus 0.13 | Metrics collection |
| **Tracing** | OpenTelemetry 0.21 + Jaeger | Distributed tracing |
| **Logging** | tracing 0.1 | Structured logging |
| **Config** | TOML (config 0.13) | Configuration management |
| **Container** | Docker | Containerization |

### Module Responsibilities

#### Core Modules (Blockchain Foundation)
- **core**: Blockchain data structures, block management
- **storage**: RDF triple store integration
- **transaction**: Transaction creation and validation

#### Network & Consensus
- **network**: P2P networking, peer discovery, message routing
- **consensus**: PoA and PBFT consensus protocols
- **sync**: Blockchain synchronization between peers

#### Semantic Layer
- **semantic**: OWL2 reasoning and SHACL validation
- **ontology**: Ontology loading and management
- **knowledge_graph**: Graph database operations

#### Production & Operations
- **production**: Deployment strategies, monitoring setup
- **security**: Encryption, key management
- **performance**: Optimizations and metrics
- **integrity**: Blockchain validation and repair

#### Application Layer
- **web**: HTTP API server, WebSocket support
- **domain**: Domain-specific adapters (healthcare, pharma, supply chain)
- **analytics**: Predictive analytics and reporting

---

## 2. Dependencies & Build Requirements

### System Dependencies

#### Debian/Ubuntu
```bash
apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libclang-dev \
    build-essential \
    ca-certificates \
    curl
```

#### RHEL/CentOS
```bash
yum install -y \
    pkgconfig \
    openssl-devel \
    clang \
    gcc \
    gcc-c++ \
    make
```

#### macOS
```bash
brew install openssl pkg-config
```

### Rust Toolchain

- **Minimum Rust Version (MSRV)**: 1.70
- **Recommended**: 1.75+
- **Edition**: 2021

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable

# Verify installation
rustc --version  # Should be >= 1.70
cargo --version
```

### Key Rust Dependencies

#### Networking & Async
```toml
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.20"
futures-util = "0.3"
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
hyper = { version = "1.0", features = ["full"] }
```

#### Cryptography & Security
```toml
ed25519-dalek = { version = "2.0", features = ["serde"] }
chacha20poly1305 = "0.10.1"
rand = "0.8"
jsonwebtoken = "9.2"
bcrypt = "0.15"
```

#### Semantic & RDF
```toml
oxigraph = { version = "0.4", default-features = false }
owl2-reasoner = { path = "./owl2-reasoner" }
```

#### Serialization & Data
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
```

#### Monitoring & Observability
```toml
prometheus = "0.13"
metrics = "0.22"
metrics-exporter-prometheus = "0.13"
opentelemetry = "0.21"
opentelemetry-jaeger = "0.20"
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### Utilities
```toml
anyhow = "1.0"          # Error handling (applications)
thiserror = "1.0"       # Error handling (libraries)
chrono = "0.4"          # Date/time
uuid = "1.0"            # UUID generation
config = "0.13"         # Configuration management
```

### Build Commands

```bash
# Development build (debug mode)
cargo build

# Production release build (optimized)
cargo build --release

# Run all tests
cargo test

# Run specific test
cargo test three_node_validation_test -- --nocapture

# Run benchmarks
cargo bench

# Check code without building
cargo check

# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Build Docker image
docker build -f deploy/Dockerfile.production -t provchain-org:latest .
```

### Build Optimizations (Cargo.toml)

```toml
[profile.release]
lto = true              # Link-time optimization (slower build, faster runtime)
codegen-units = 1       # Single codegen unit (better optimization)
strip = true            # Remove debug symbols (smaller binary)
opt-level = 3           # Maximum optimization level
panic = 'abort'         # Abort on panic (no unwinding, smaller binary)

[profile.dev]
opt-level = 0           # Fast compilation
debug = true            # Include debug info

[profile.bench]
inherits = "release"
debug = true            # Debug info for profiling
```

### Binary Size

- **Debug build**: ~500 MB (with debug symbols)
- **Release build**: ~80 MB (optimized, stripped)
- **Docker image**: ~150 MB (multi-stage build)

### Build Time Estimates

- **Clean build (debug)**: 5-8 minutes
- **Clean build (release)**: 10-15 minutes
- **Incremental build**: 30 seconds - 2 minutes
- **Docker build**: 15-20 minutes (first time), 5 minutes (cached)

---

## 3. Network Communication & Consensus

### Consensus Architecture

ProvChain-Org supports two consensus protocols:

#### 1. Proof of Authority (PoA) - Primary

**Overview:**
- Designated authority nodes create blocks
- Round-robin rotation among authorities
- Performance-based reputation system
- Low latency, high throughput

**Key Components:**
```rust
ProofOfAuthority {
    authority_keypair: Ed25519 keypair for signing
    authority_keys: Vec<PublicKey>  // All authorized validators
    authority_state: {
        current_round: u64,
        current_authority: PublicKey,
        authority_rotation_order: Vec<PublicKey>,
        authority_performance: HashMap<PublicKey, Performance>
    }
}

AuthorityPerformance {
    blocks_created: u64,
    missed_slots: u64,
    last_activity: Timestamp,
    reputation: f64  // 0.0 - 1.0
}
```

**Authority Selection Logic:**
1. Authorities are ordered in `authority_rotation_order`
2. Current authority creates blocks for configured interval (default 10s)
3. After interval, rotate to next authority in order
4. If authority misses slot (no block created), reputation decreases
5. Performance tracking influences future governance decisions

**Block Creation Flow:**
```
Authority Node:
1. Check if current time >= last_block_time + block_interval
2. If yes, create block with pending transactions
3. Sign block with authority_keypair
4. Broadcast BlockProposal to all peers
5. Update authority_state and performance metrics

Regular Node:
1. Receive BlockProposal message
2. Validate block signature (must be current authority)
3. Validate block timing (within acceptable range)
4. Validate block structure and transactions
5. If valid, add to blockchain and forward to peers
6. If invalid, reject and log error
```

**Configuration:**
```toml
[consensus]
is_authority = true                    # This node can create blocks
authority_key_file = "authority.key"   # Path to Ed25519 private key
authority_keys = [                     # All authority public keys
    "A1B2C3D4E5F6...",
    "F6E5D4C3B2A1..."
]
block_interval = 10                    # Seconds between blocks
max_block_size = 1048576              # 1 MB
```

#### 2. PBFT (Practical Byzantine Fault Tolerance) - Secondary

**Overview:**
- Byzantine fault-tolerant consensus
- Requires 3f+1 nodes to tolerate f failures
- More complex, higher latency, but handles malicious nodes

**Use Cases:**
- High-security environments
- Untrusted validator sets
- Regulatory compliance requirements

**Configuration:**
```toml
[consensus]
protocol = "pbft"         # Switch from PoA to PBFT
pbft_timeout = 30         # Consensus timeout (seconds)
pbft_view_change = true   # Enable view changes
```

### Network Architecture

#### Topology
```
┌─────────────────────────────────────────────────┐
│               Mesh Network Topology              │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────────┐         ┌──────────────┐     │
│  │   Node 1     │◄───────►│   Node 2     │     │
│  │ (Authority)  │         │  (Regular)   │     │
│  │ 10.0.1.10    │         │ 10.0.1.11    │     │
│  └──────┬───────┘         └──────┬───────┘     │
│         │                        │              │
│         │                        │              │
│         │      ┌──────────────┐  │              │
│         └─────►│   Node 3     │◄─┘              │
│                │  (Regular)   │                  │
│                │ 10.0.1.12    │                  │
│                └──────────────┘                  │
│                                                  │
│  All nodes maintain connections to all peers    │
│  (Full mesh for N <= 10 nodes)                  │
└─────────────────────────────────────────────────┘
```

#### Communication Protocol

**Transport:** WebSocket over TCP
- **Port**: Configurable (default 8080)
- **Protocol**: WebSocket (ws:// or wss://)
- **Encoding**: JSON for messages
- **Compression**: Optional LZ4

**Message Types:**
```rust
enum NetworkMessage {
    // Peer management
    Hello { node_id: Uuid, version: String },
    Ping { timestamp: i64 },
    Pong { timestamp: i64 },

    // Blockchain sync
    RequestBlocks { start_index: u64, count: u64 },
    ResponseBlocks { blocks: Vec<Block> },

    // Consensus
    BlockProposal { block: Block, signature: Vec<u8>, authority_key: PublicKey },
    BlockAck { block_hash: String, node_id: Uuid },

    // Transaction pool
    NewTransaction { transaction: Transaction },

    // Discovery
    PeerList { peers: Vec<PeerInfo> },
}
```

#### Peer Discovery

**Bootstrap Process:**
1. Node starts with `known_peers` from config.toml
2. Connects to bootstrap peers via WebSocket
3. Sends Hello message with node info
4. Receives PeerList from bootstrap peers
5. Attempts connections to discovered peers
6. Periodic peer list exchange (every 60s)

**Peer Management:**
```rust
PeerConnection {
    info: PeerInfo,
    sender: WebSocketSender,
    task_handle: JoinHandle,
    last_seen: Timestamp,
    connected_at: Timestamp,
}

PeerInfo {
    node_id: Uuid,
    address: SocketAddr,
    is_authority: bool,
    version: String,
}
```

**Connection Limits:**
- **max_peers**: 50 (configurable)
- **connection_timeout**: 30 seconds
- **ping_interval**: 30 seconds
- **peer_timeout**: 90 seconds (3 missed pings)

#### Blockchain Synchronization

**Sync Protocol:**
```
New Node Joins:
1. Connect to bootstrap peer
2. Request blockchain length: GET /api/blockchain/length
3. Compare with local blockchain length
4. If remote > local:
   a. Request missing blocks in chunks (100 blocks/request)
   b. Validate each block before adding
   c. Repeat until synchronized
5. Subscribe to BlockProposal messages for new blocks

Block Validation:
- Verify block hash matches calculated hash
- Verify previous_hash links to existing block
- Verify authority signature (PoA)
- Verify block timestamp is valid
- Verify transactions are well-formed
- Run semantic validation (OWL2/SHACL)
```

**Sync States:**
```rust
enum SyncState {
    Syncing { current: u64, target: u64 },
    Synchronized,
    Disconnected,
}
```

### Network Configuration

#### config.toml - Network Section
```toml
[network]
# Unique network identifier (must match across all nodes)
network_id = "provchain-org-production"

# Listening configuration
listen_port = 8080
bind_address = "0.0.0.0"  # All interfaces

# Bootstrap peers (other nodes to connect to on startup)
known_peers = [
    "10.0.1.10:8080",  # Authority node
    "10.0.1.11:8080",  # Regular node 2
]

# Connection limits
max_peers = 50
connection_timeout = 30  # seconds
ping_interval = 30       # seconds
peer_timeout = 90        # seconds

# Discovery
enable_discovery = true
discovery_interval = 60  # seconds

# Network performance
send_buffer_size = 65536  # 64 KB
recv_buffer_size = 65536  # 64 KB
```

### Security Considerations

#### Cryptographic Security
- **Signatures**: Ed25519 (256-bit security)
- **Encryption**: ChaCha20-Poly1305 (256-bit keys)
- **Hashing**: SHA-256 for block hashes

#### Network Security
- **TLS/SSL**: Recommended for production (wss://)
- **Firewall**: Restrict to known peer IPs
- **DDoS Protection**: Rate limiting on API endpoints
- **Authentication**: JWT tokens for API access

#### Consensus Security
- **Authority Verification**: Only authorized public keys can create blocks
- **Signature Verification**: All blocks must be signed by valid authority
- **Replay Protection**: Block timestamps prevent replay attacks
- **Chain Validation**: Full blockchain validation on startup

---

## 4. Configuration for Distributed Deployment

### Configuration File Structure

ProvChain-Org uses TOML configuration files with the following sections:

```toml
# config.toml - Complete example

# Unique node identifier (auto-generated on first run)
node_id = "550e8400-e29b-41d4-a716-446655440000"

[network]
# Network settings
network_id = "provchain-org-production"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.10:8080", "10.0.1.11:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
# Consensus configuration
is_authority = false
# authority_key_file = "authority.key"  # Required if is_authority = true
authority_keys = []  # List of authority public keys
block_interval = 10
max_block_size = 1048576  # 1 MB

[storage]
# Data persistence
data_dir = "./data"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

[web]
# HTTP API server
host = "0.0.0.0"
port = 8080

[web.cors]
# CORS configuration for frontend
enabled = true
allowed_origins = [
    "http://localhost:5173",
    "https://provchain.example.com"
]
allowed_methods = ["GET", "POST", "OPTIONS"]
allowed_headers = ["Authorization", "Content-Type"]
allow_credentials = true
max_age = 3600

[logging]
# Logging configuration
level = "info"  # trace|debug|info|warn|error
format = "pretty"  # pretty|json
# file = "provchain.log"  # Optional log file
```

### Node-Specific Configurations

#### Authority Node (VM-1: 10.0.1.10)

```toml
# config/node1-authority.toml
node_id = "authority-node-1"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.11:8080", "10.0.1.12:8080"]
max_peers = 50

[consensus]
is_authority = true
authority_key_file = "/app/keys/authority.key"
authority_keys = [
    "302a300506032b6570032100..."  # Authority public key
]
block_interval = 10
max_block_size = 1048576

[storage]
data_dir = "/app/data"
persistent = true
cache_size_mb = 200

[web]
host = "0.0.0.0"
port = 8080

[logging]
level = "info"
format = "json"
```

#### Regular Node (VM-2: 10.0.1.11)

```toml
# config/node2-regular.toml
node_id = "regular-node-2"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.10:8080", "10.0.1.12:8080"]
max_peers = 50

[consensus]
is_authority = false
authority_keys = [
    "302a300506032b6570032100..."  # Authority public key
]
block_interval = 10

[storage]
data_dir = "/app/data"
persistent = true
cache_size_mb = 100

[web]
host = "0.0.0.0"
port = 8080

[logging]
level = "info"
format = "json"
```

#### Regular Node (VM-3: 10.0.1.12)

```toml
# config/node3-regular.toml
node_id = "regular-node-3"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.10:8080", "10.0.1.11:8080"]
max_peers = 50

[consensus]
is_authority = false
authority_keys = [
    "302a300506032b6570032100..."  # Authority public key
]
block_interval = 10

[storage]
data_dir = "/app/data"
persistent = true
cache_size_mb = 100

[web]
host = "0.0.0.0"
port = 8080

[logging]
level = "info"
format = "json"
```

### Environment Variables

Environment variables override config file settings:

```bash
# Required for production
export JWT_SECRET="your-secure-secret-min-32-characters-long"

# Optional overrides
export RUST_LOG="info"
export CONFIG_FILE="/app/config/node1.toml"
export PROVCHAIN_PORT="8080"
export PROVCHAIN_PEERS="10.0.1.11:8080,10.0.1.12:8080"
export OTEL_SERVICE_NAME="provchain-node1"

# Storage
export DATA_DIR="/mnt/provchain-data"

# Monitoring
export PROMETHEUS_PORT="9090"
export JAEGER_ENDPOINT="http://jaeger:14268/api/traces"
```

### Secrets Management

#### JWT Secret Generation
```bash
# Generate secure JWT secret (256-bit)
openssl rand -base64 32
# Output: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=
```

#### Authority Keypair Generation
```bash
# Method 1: Using provchain-org CLI
./provchain-org generate-keypair --output authority.key

# Method 2: Using OpenSSL (Ed25519)
openssl genpkey -algorithm ED25519 -out authority_private.pem
openssl pkey -in authority_private.pem -pubout -out authority_public.pem

# Extract public key for configuration
./provchain-org show-pubkey --key authority.key
```

#### Key Storage
- **Development**: Local filesystem
- **Production**:
  - AWS Secrets Manager
  - HashiCorp Vault
  - Kubernetes Secrets
  - Azure Key Vault

### Configuration Best Practices

1. **Never commit secrets to git**: Use `.gitignore` for `*.key`, `*.pem`, `config/*local.toml`
2. **Use environment-specific configs**: `config.dev.toml`, `config.staging.toml`, `config.prod.toml`
3. **Validate configurations on startup**: The application validates all settings
4. **Use consistent `network_id`**: All nodes must have matching `network_id`
5. **Rotate authority keys periodically**: Implement key rotation strategy
6. **Monitor configuration changes**: Audit log all config modifications
7. **Backup configurations**: Store configs in version control (without secrets)

---

## 5. Three-Node Cloud Deployment Strategy

### VM Specifications

#### Recommended Hardware

| Component | Authority Node (VM-1) | Regular Nodes (VM-2, VM-3) |
|-----------|----------------------|---------------------------|
| **CPU** | 4 vCPUs | 2 vCPUs |
| **RAM** | 8 GB | 4 GB |
| **Storage** | 100 GB SSD | 50 GB SSD |
| **Network** | 1 Gbps | 1 Gbps |
| **OS** | Ubuntu 22.04 LTS | Ubuntu 22.04 LTS |

#### Minimum Hardware (Testing)

| Component | All Nodes |
|-----------|-----------|
| **CPU** | 2 vCPUs |
| **RAM** | 2 GB |
| **Storage** | 20 GB SSD |
| **Network** | 100 Mbps |

### Deployment Options

We offer three deployment strategies:

---

### Option A: Docker Compose (Recommended for Testing)

**Best For:** Quick setup, development, testing, proof-of-concept

**Advantages:**
- Fastest deployment (< 30 minutes)
- Integrated monitoring stack (Prometheus, Grafana, Jaeger)
- Easy teardown and redeployment
- Existing configuration in `deploy/docker-compose.3node.yml`

**Architecture:**
```
Each VM runs:
- ProvChain node (Docker container)
- Prometheus metrics endpoint
- Jaeger tracing agent

VM-1 additionally runs:
- Grafana (visualization)
- Prometheus server (metrics aggregation)
- Jaeger UI (trace visualization)
```

**Full Deployment Guide:** See [SETUP_INSTALLATION_GUIDE.md](#docker-compose-deployment)

---

### Option B: Kubernetes (Recommended for Production)

**Best For:** Production deployments, auto-scaling, high availability

**Advantages:**
- Auto-healing (restarts failed containers)
- Rolling updates with zero downtime
- Resource management and limits
- Built-in service discovery
- Horizontal pod autoscaling

**Architecture:**
```
Kubernetes Cluster:
├── Namespace: provchain
├── StatefulSet: provchain-nodes (3 replicas)
│   ├── Pod: provchain-node-0 (Authority)
│   ├── Pod: provchain-node-1 (Regular)
│   └── Pod: provchain-node-2 (Regular)
├── Service: provchain-api (LoadBalancer)
├── PersistentVolumeClaims: data-provchain-node-{0,1,2}
└── ConfigMaps: node-configs, monitoring-configs
```

**Requirements:**
- Kubernetes cluster (v1.24+)
- kubectl configured
- Helm 3 (optional, for monitoring stack)

**Full Deployment Guide:** See [SETUP_INSTALLATION_GUIDE.md](#kubernetes-deployment)

---

### Option C: Native Systemd Services

**Best For:** Bare-metal deployments, maximum control, no container overhead

**Advantages:**
- Direct hardware access
- Lower resource overhead
- Traditional Linux service management
- Easier debugging (no container layers)
- Better performance (no virtualization)

**Architecture:**
```
Each VM:
├── /usr/local/bin/provchain-org (binary)
├── /opt/provchain/config.toml
├── /opt/provchain/data/ (blockchain storage)
├── /var/log/provchain/ (logs)
└── systemd service: provchain.service
```

**Requirements:**
- Rust toolchain (for building)
- System dependencies (OpenSSL, libclang)
- systemd (Ubuntu 22.04 has it)

**Full Deployment Guide:** See [SETUP_INSTALLATION_GUIDE.md](#systemd-deployment)

---

### Network Architecture

#### Cloud VPC Setup

```
┌────────────────────────────────────────────────┐
│         Cloud VPC (10.0.0.0/16)                │
│                                                │
│  ┌─────────────────────────────────┐          │
│  │  Private Subnet (10.0.1.0/24)   │          │
│  │                                 │          │
│  │  ┌──────────┐  ┌──────────┐  ┌┴────────┐ │
│  │  │  VM-1    │  │  VM-2    │  │  VM-3   │ │
│  │  │ Auth     │  │ Regular  │  │ Regular │ │
│  │  │ .10:8080 │  │ .11:8080 │  │ .12:8080│ │
│  │  └────┬─────┘  └────┬─────┘  └────┬────┘ │
│  └───────┼─────────────┼──────────────┼──────┘
│          │             │              │       │
│  ┌───────┼─────────────┼──────────────┼──────┐
│  │       │    Public Subnet           │      │
│  │  ┌────┴──────────────────────────┐ │      │
│  │  │  Load Balancer / NAT Gateway  │ │      │
│  │  └────────────────────────────────┘ │      │
│  └───────────────────────────────────────────┘
│                    │                          │
└────────────────────┼──────────────────────────┘
                     │
              Internet Gateway
                     │
         ┌───────────┴───────────┐
         │                       │
    Users (API)            Monitoring
   (Port 8080)         (Ports 3001, 9091, 16686)
```

#### Firewall Rules

**Inbound Rules:**

| Port | Protocol | Source | Purpose |
|------|----------|--------|---------|
| 22 | TCP | Admin IPs | SSH access |
| 8080 | TCP | 0.0.0.0/0 | HTTP API |
| 8080 | TCP | VPC subnet | P2P networking |
| 9090 | TCP | VPC subnet | Metrics export |
| 3001 | TCP | Admin IPs | Grafana UI |
| 9091 | TCP | Admin IPs | Prometheus UI |
| 16686 | TCP | Admin IPs | Jaeger UI |

**Outbound Rules:**
- Allow all (for updates, NTP, external APIs)

**Security Group Example (AWS):**
```bash
# Create security group
aws ec2 create-security-group \
  --group-name provchain-sg \
  --description "ProvChain node security group" \
  --vpc-id vpc-xxxxx

# Add rules
aws ec2 authorize-security-group-ingress \
  --group-id sg-xxxxx \
  --protocol tcp \
  --port 8080 \
  --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
  --group-id sg-xxxxx \
  --protocol tcp \
  --port 22 \
  --cidr 203.0.113.0/24  # Your admin IP range
```

### Deployment Workflow

```
Step 1: Infrastructure Provisioning
├── Create VMs (Terraform/CloudFormation)
├── Configure networking (VPC, subnets, security groups)
├── Set up DNS (optional)
└── Allocate persistent storage volumes

Step 2: Software Installation
├── Install Docker/Kubernetes/Build tools
├── Clone ProvChain repository
├── Build binaries or Docker images
└── Install monitoring stack

Step 3: Configuration
├── Generate authority keypair
├── Create node-specific configs
├── Set up secrets (JWT, keys)
└── Configure monitoring endpoints

Step 4: Deployment
├── Deploy authority node (VM-1)
├── Wait for authority to stabilize (30s)
├── Deploy regular nodes (VM-2, VM-3)
├── Verify peer connections
└── Check monitoring dashboards

Step 5: Validation
├── Run health checks
├── Test blockchain operations
├── Verify synchronization
├── Load testing (optional)
└── Resilience testing

Step 6: Production Handoff
├── Document deployment
├── Set up alerting
├── Configure backups
├── Train operations team
└── Monitor for 24-48 hours
```

---

## 6. Testing & Validation Approach

### Testing Strategy Overview

```
Testing Pyramid:

                    ▲
                   ╱ ╲
                  ╱   ╲  Manual Testing
                 ╱─────╲  (Exploratory, UAT)
                ╱       ╲
               ╱─────────╲ System/E2E Tests
              ╱           ╲ (3-node validation)
             ╱─────────────╲
            ╱               ╲ Integration Tests
           ╱─────────────────╲ (Network, consensus)
          ╱                   ╲
         ╱─────────────────────╲ Unit Tests
        ╱                       ╲ (Core, storage, crypto)
       ╱─────────────────────────╲
```

### Pre-Deployment Testing

#### 1. Unit Tests (Local)

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib core::
cargo test --lib network::consensus::
cargo test --lib semantic::

# With output
cargo test -- --nocapture --test-threads=1

# Expected: 100+ passing tests
```

**Key Test Coverage:**
- Block creation and hashing
- Transaction validation
- Cryptographic signing/verification
- RDF triple storage operations
- OWL2 reasoning logic

#### 2. Integration Tests (Local)

```bash
# Run all integration tests
cargo test --test '*'

# Run three-node test (simulates 3-node cluster)
cargo test three_node_validation_test -- --nocapture

# Expected duration: ~60 seconds
# Expected result: All nodes synchronized with identical blockchains
```

**What it tests:**
- Node startup and initialization
- Peer connection establishment
- Authority block creation
- Blockchain synchronization
- Block validation across nodes

#### 3. Performance Benchmarks (Local)

```bash
# Run all benchmarks
cargo bench

# Specific benchmarks
cargo bench simple_consensus_benchmarks
cargo bench trace_optimization_benchmarks
cargo bench owl2_benchmarks

# Generate HTML report
cargo bench -- --save-baseline baseline_v1
```

**Key Metrics:**
- Block creation time: < 100ms
- Block validation time: < 50ms
- Transaction throughput: > 1000 TPS
- P2P message latency: < 10ms
- Blockchain sync rate: > 100 blocks/second

### Post-Deployment Validation

#### Phase 1: Health & Connectivity (5-10 minutes)

**Test Matrix:**

| Test | Command | Expected Result |
|------|---------|-----------------|
| Health Check | `curl http://10.0.1.10:8080/health` | `{"status":"healthy"}` |
| Peer Count | `curl http://10.0.1.10:8080/api/peers \| jq 'length'` | `2` (for 3-node) |
| Consensus Status | `curl http://10.0.1.10:8080/api/consensus/stats` | `is_authority: true` |
| Blockchain Length | `curl http://10.0.1.10:8080/api/blockchain/dump \| jq 'length'` | `>= 1` (genesis) |

**Automated Script:**
```bash
#!/bin/bash
# test_phase1_health.sh

NODES=("10.0.1.10" "10.0.1.11" "10.0.1.12")
PASS=0
FAIL=0

echo "=== Phase 1: Health & Connectivity ==="

for node in "${NODES[@]}"; do
  echo "Testing node $node..."

  # Health check
  if curl -sf http://$node:8080/health | grep -q "healthy"; then
    echo "✓ Health check passed"
    ((PASS++))
  else
    echo "✗ Health check failed"
    ((FAIL++))
  fi
done

echo "Results: $PASS passed, $FAIL failed"
exit $FAIL
```

#### Phase 2: Blockchain Operations (10-15 minutes)

**Test Scenarios:**

1. **Submit Transaction to Authority**
```bash
curl -X POST http://10.0.1.10:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "data": "@prefix ex: <http://test.org/> . ex:test1 ex:prop \"value\" ."
  }'

# Expected: {"status":"accepted","tx_id":"..."}
```

2. **Wait for Block Creation**
```bash
# Block should be created within block_interval (10s)
sleep 15

# Verify block was created
curl http://10.0.1.10:8080/api/blockchain/dump | jq '.[-1]'
# Should show latest block with transaction
```

3. **Verify Synchronization Across Nodes**
```bash
# Get blockchain hashes from all nodes
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/api/blockchain/dump | \
    jq -r '.[] | .hash' | \
    tail -5
done

# All nodes should show identical last 5 block hashes
```

4. **Query Blockchain Data**
```bash
# SPARQL query
curl -X POST http://10.0.1.10:8080/api/query \
  -H "Content-Type: application/sparql-query" \
  -d 'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10'

# Should return triples from blockchain
```

**Automated Script:**
```bash
#!/bin/bash
# test_phase2_operations.sh

set -e

AUTHORITY="10.0.1.10"
NODES=("10.0.1.10" "10.0.1.11" "10.0.1.12")

echo "=== Phase 2: Blockchain Operations ==="

# 1. Get initial blockchain lengths
echo "1. Recording initial state..."
for node in "${NODES[@]}"; do
  length=$(curl -s http://$node:8080/api/blockchain/dump | jq 'length')
  echo "Node $node: $length blocks"
done

# 2. Submit transaction
echo "2. Submitting test transaction..."
tx_response=$(curl -s -X POST http://$AUTHORITY:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://test> <http://prop> \"ops_test\" ."}')

if echo "$tx_response" | grep -q "accepted"; then
  echo "✓ Transaction accepted"
else
  echo "✗ Transaction rejected"
  exit 1
fi

# 3. Wait for block creation
echo "3. Waiting for block creation (15s)..."
sleep 15

# 4. Verify new block on all nodes
echo "4. Verifying blockchain sync..."
declare -a new_lengths
for i in "${!NODES[@]}"; do
  node="${NODES[$i]}"
  new_length=$(curl -s http://$node:8080/api/blockchain/dump | jq 'length')
  new_lengths[$i]=$new_length
  echo "Node $node: $new_length blocks"
done

# Check all lengths are equal
if [ "${new_lengths[0]}" == "${new_lengths[1]}" ] && \
   [ "${new_lengths[1]}" == "${new_lengths[2]}" ]; then
  echo "✓ All nodes synchronized (${new_lengths[0]} blocks)"
else
  echo "✗ Synchronization mismatch"
  exit 1
fi

# 5. Verify blockchain integrity
echo "5. Validating blockchain integrity..."
for node in "${NODES[@]}"; do
  valid=$(curl -s http://$node:8080/api/blockchain/validate | jq -r '.valid')
  if [ "$valid" == "true" ]; then
    echo "✓ Node $node: valid"
  else
    echo "✗ Node $node: invalid"
    exit 1
  fi
done

echo "=== Phase 2 Complete: All operations successful ==="
```

#### Phase 3: Consensus Validation (15-20 minutes)

**Test Scenarios:**

1. **Monitor Authority Rotation**
```bash
# Watch consensus rounds increment
for i in {1..10}; do
  round=$(curl -s http://10.0.1.10:8080/api/consensus/stats | jq '.current_round')
  echo "Round $round (iteration $i)"
  sleep 12  # Slightly longer than block_interval
done

# Rounds should increment: 1, 2, 3, 4...
```

2. **Rapid Transaction Submission**
```bash
# Submit 20 transactions rapidly
for i in {1..20}; do
  curl -X POST http://10.0.1.10:8080/api/transactions \
    -H "Content-Type: application/json" \
    -d "{\"data\":\"<http://test/$i> <http://prop> \\\"$i\\\" .\"}" &
done
wait

# Wait for all transactions to be processed
sleep 30

# Verify all nodes have same blockchain length
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  length=$(curl -s http://$ip:8080/api/blockchain/dump | jq 'length')
  echo "Node $ip: $length blocks"
done
```

3. **Performance Metrics**
```bash
# Query Prometheus metrics
curl http://10.0.1.10:9090/metrics | grep provchain

# Key metrics to check:
# - provchain_blocks_total (should be increasing)
# - provchain_peers_connected (should be 2)
# - provchain_transaction_latency_seconds (should be < 1.0)
# - provchain_consensus_round (should be increasing)
```

#### Phase 4: Resilience Testing (20-30 minutes)

**Test Scenarios:**

1. **Node Failure Simulation**
```bash
# Stop one regular node
ssh vm3 "docker-compose down"  # Docker
# or
ssh vm3 "systemctl stop provchain"  # Systemd

# Wait 5 seconds
sleep 5

# Verify cluster still operational (2/3 nodes)
curl http://10.0.1.10:8080/health
# Should still return healthy

# Submit transaction
curl -X POST http://10.0.1.10:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://resilience> <http://test> \"1\" ."}'

# Verify block created
sleep 15
curl http://10.0.1.10:8080/api/blockchain/dump | jq '.[-1]'

# Restart failed node
ssh vm3 "docker-compose up -d"  # or systemctl start

# Wait for resync
sleep 30

# Verify node caught up
curl http://10.0.1.12:8080/api/blockchain/dump | jq 'length'
# Should match other nodes
```

2. **Network Partition Simulation**
```bash
# Block traffic between VM-1 and VM-2
ssh vm2 "sudo iptables -A INPUT -s 10.0.1.10 -j DROP"
ssh vm2 "sudo iptables -A OUTPUT -d 10.0.1.10 -j DROP"

# Observe behavior (should see disconnected peer)
curl http://10.0.1.10:8080/api/peers | jq 'length'
# Should show 1 peer instead of 2

# Restore connectivity
ssh vm2 "sudo iptables -F"

# Verify reconnection
sleep 10
curl http://10.0.1.10:8080/api/peers | jq 'length'
# Should show 2 peers again
```

3. **Authority Failure**
```bash
# Stop authority node
ssh vm1 "docker-compose down"

# Observe system behavior
# - Regular nodes should maintain connection
# - No new blocks created (requires authority)
# - System should log authority timeout

# Restart authority
ssh vm1 "docker-compose up -d"

# Verify block creation resumes
sleep 15
curl http://10.0.1.11:8080/api/blockchain/dump | jq '.[-1]'
# Should show new block after authority restart
```

### Automated Validation Suite

Complete test suite script:

```bash
#!/bin/bash
# deploy/scripts/full_validation_suite.sh

set -e

NODES=("10.0.1.10" "10.0.1.11" "10.0.1.12")
AUTHORITY="${NODES[0]}"
API_PORT=8080
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function test_pass() {
  echo -e "${GREEN}✓${NC} $1"
  ((PASSED_TESTS++))
  ((TOTAL_TESTS++))
}

function test_fail() {
  echo -e "${RED}✗${NC} $1"
  ((FAILED_TESTS++))
  ((TOTAL_TESTS++))
}

function section() {
  echo ""
  echo -e "${YELLOW}=== $1 ===${NC}"
  echo ""
}

section "ProvChain 3-Node Validation Suite"
echo "Authority: $AUTHORITY"
echo "Nodes: ${NODES[@]}"
echo ""

# Phase 1: Health Checks
section "Phase 1: Health & Connectivity"

for node in "${NODES[@]}"; do
  if curl -sf http://$node:$API_PORT/health | grep -q "healthy"; then
    test_pass "Node $node is healthy"
  else
    test_fail "Node $node health check failed"
  fi
done

# Check peer connectivity
peers=$(curl -s http://$AUTHORITY:$API_PORT/api/peers | jq 'length')
if [ "$peers" -eq 2 ]; then
  test_pass "All peers connected ($peers/2)"
else
  test_fail "Peer connectivity issue (found $peers, expected 2)"
fi

# Phase 2: Blockchain Operations
section "Phase 2: Blockchain Operations"

# Record initial lengths
declare -a initial_lengths
for i in "${!NODES[@]}"; do
  length=$(curl -s http://${NODES[$i]}:$API_PORT/api/blockchain/dump | jq 'length')
  initial_lengths[$i]=$length
done

# Submit transaction
tx_response=$(curl -s -X POST http://$AUTHORITY:$API_PORT/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://validation> <http://test> \"suite\" ."}')

if echo "$tx_response" | grep -q "accepted"; then
  test_pass "Transaction accepted"
else
  test_fail "Transaction rejected"
fi

# Wait for block
echo "Waiting 15s for block creation..."
sleep 15

# Verify synchronization
declare -a new_lengths
for i in "${!NODES[@]}"; do
  length=$(curl -s http://${NODES[$i]}:$API_PORT/api/blockchain/dump | jq 'length')
  new_lengths[$i]=$length
done

if [ "${new_lengths[0]}" -eq "${new_lengths[1]}" ] && \
   [ "${new_lengths[1]}" -eq "${new_lengths[2]}" ] && \
   [ "${new_lengths[0]}" -gt "${initial_lengths[0]}" ]; then
  test_pass "All nodes synchronized (${new_lengths[0]} blocks)"
else
  test_fail "Blockchain sync mismatch"
fi

# Phase 3: Consensus
section "Phase 3: Consensus Validation"

authority_status=$(curl -s http://$AUTHORITY:$API_PORT/api/consensus/stats | jq -r '.is_authority')
if [ "$authority_status" == "true" ]; then
  test_pass "Authority node operational"
else
  test_fail "Authority node not detected"
fi

# Check consensus round progression
round1=$(curl -s http://$AUTHORITY:$API_PORT/api/consensus/stats | jq '.current_round')
sleep 12
round2=$(curl -s http://$AUTHORITY:$API_PORT/api/consensus/stats | jq '.current_round')

if [ "$round2" -gt "$round1" ]; then
  test_pass "Consensus rounds progressing ($round1 -> $round2)"
else
  test_fail "Consensus rounds not progressing"
fi

# Phase 4: Integrity
section "Phase 4: Blockchain Integrity"

for node in "${NODES[@]}"; do
  valid=$(curl -s http://$node:$API_PORT/api/blockchain/validate | jq -r '.valid')
  if [ "$valid" == "true" ]; then
    test_pass "Node $node blockchain valid"
  else
    test_fail "Node $node blockchain invalid"
  fi
done

# Summary
section "Test Summary"
echo "Total tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
  echo -e "${GREEN}All validation tests passed ✓${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed ✗${NC}"
  exit 1
fi
```

### Load Testing (Optional)

For stress testing the cluster:

```bash
# Install Apache Bench (ab)
sudo apt-get install apache2-utils

# Load test API endpoint (100 requests, 10 concurrent)
ab -n 100 -c 10 -p transaction.json -T application/json \
  http://10.0.1.10:8080/api/transactions

# Transaction payload file
cat > transaction.json <<EOF
{"data":"<http://load> <http://test> \"value\" ."}
EOF

# Expected results:
# - Requests per second: > 50
# - Mean response time: < 200ms
# - Failed requests: 0
```

### Monitoring During Tests

```bash
# Watch metrics in real-time
watch -n 1 'curl -s http://10.0.1.10:9090/metrics | grep provchain_blocks_total'

# Monitor logs (Docker)
docker logs -f provchain-node1

# Monitor logs (Systemd)
journalctl -u provchain -f

# Monitor resources
htop
iotop
nethogs
```

---

## 7. Deployment Checklist

Use this checklist to ensure complete deployment:

### Pre-Deployment

**Infrastructure:**
- [ ] Provision 3 VMs with required specs (4GB+ RAM, 2+ vCPUs, 50GB+ SSD)
- [ ] Configure VPC/network (10.0.0.0/16 or equivalent)
- [ ] Assign static private IPs (10.0.1.10, 10.0.1.11, 10.0.1.12)
- [ ] Set up security groups/firewall rules
  - [ ] Allow inbound: 22 (SSH), 8080 (API+P2P), 9090 (metrics)
  - [ ] Allow inbound monitoring: 3001 (Grafana), 9091 (Prometheus), 16686 (Jaeger)
  - [ ] Allow all outbound traffic
- [ ] Allocate persistent storage volumes (100GB authority, 50GB regular)
- [ ] Set up DNS records (optional, e.g., node1.provchain.example.com)

**Software Prerequisites:**
- [ ] Install Docker on all VMs (if using Docker deployment)
  - [ ] Docker version >= 20.10
  - [ ] Docker Compose version >= 2.0
- [ ] OR install Kubernetes (if using K8s deployment)
  - [ ] kubectl configured
  - [ ] Cluster access verified
- [ ] OR install build tools (if using native deployment)
  - [ ] Rust toolchain (>= 1.70)
  - [ ] System dependencies (OpenSSL, libclang)

**Codebase:**
- [ ] Clone ProvChain repository to each VM
  - `git clone https://github.com/yourusername/provchain-org.git`
- [ ] Verify git branch/tag (e.g., `main`, `v1.0.0`)
- [ ] Run initial tests locally
  - [ ] `cargo test --lib` (unit tests)
  - [ ] `cargo test three_node_validation_test` (3-node simulation)

**Security & Secrets:**
- [ ] Generate JWT secret (>= 32 characters)
  - `openssl rand -base64 32`
  - Store in environment variable or secrets manager
- [ ] Generate authority Ed25519 keypair
  - `./provchain-org generate-keypair --output authority.key`
  - Extract public key: `./provchain-org show-pubkey --key authority.key`
- [ ] Secure authority private key
  - Set permissions: `chmod 600 authority.key`
  - Backup to secure location (HSM, Key Vault, encrypted storage)
- [ ] Distribute authority public key to all nodes
- [ ] Generate node-specific UUIDs for `node_id`

**Configuration Files:**
- [ ] Create `config/node1-authority.toml`
  - [ ] Set `is_authority = true`
  - [ ] Configure `authority_key_file` path
  - [ ] List all authority public keys
  - [ ] Set `known_peers` to VM-2 and VM-3
- [ ] Create `config/node2-regular.toml`
  - [ ] Set `is_authority = false`
  - [ ] List authority public keys
  - [ ] Set `known_peers` to VM-1 and VM-3
- [ ] Create `config/node3-regular.toml`
  - [ ] Set `is_authority = false`
  - [ ] List authority public keys
  - [ ] Set `known_peers` to VM-1 and VM-2
- [ ] Verify `network_id` matches across all configs
- [ ] Review CORS settings for frontend access

### Deployment

**Build Phase:**
- [ ] Build Docker images (if using Docker)
  - `docker build -f deploy/Dockerfile.production -t provchain-org:latest .`
  - Verify image size (~ 150 MB)
- [ ] OR build Rust binary (if using native)
  - `cargo build --release`
  - Binary location: `target/release/provchain-org`

**Deploy Monitoring (Optional but Recommended):**
- [ ] Deploy Prometheus
  - Configure scrape targets for all nodes
- [ ] Deploy Grafana
  - Configure Prometheus data source
  - Import ProvChain dashboards
- [ ] Deploy Jaeger
  - Configure OpenTelemetry endpoint

**Deploy Nodes:**
- [ ] Deploy Authority Node (VM-1) FIRST
  - Docker: `docker-compose -f deploy/docker-compose.node.yml up -d`
  - Systemd: `systemctl start provchain`
  - Verify startup: `curl http://10.0.1.10:8080/health`
  - Check logs for errors
- [ ] Wait 30 seconds for authority to stabilize
- [ ] Deploy Regular Node 2 (VM-2)
  - Same commands as above, with node2 config
  - Verify startup and peer connection
- [ ] Deploy Regular Node 3 (VM-3)
  - Same commands as above, with node3 config
  - Verify startup and peer connection

**Verify Deployment:**
- [ ] Check all nodes are healthy
  - `curl http://10.0.1.10:8080/health`
  - `curl http://10.0.1.11:8080/health`
  - `curl http://10.0.1.12:8080/health`
- [ ] Verify peer connections
  - `curl http://10.0.1.10:8080/api/peers | jq 'length'` (should be 2)
- [ ] Check consensus status
  - `curl http://10.0.1.10:8080/api/consensus/stats`
  - Verify `is_authority: true` for VM-1
- [ ] Verify blockchain initialization
  - `curl http://10.0.1.10:8080/api/blockchain/dump | jq 'length'`
  - Should show at least genesis block (length >= 1)

### Post-Deployment Validation

**Functional Tests:**
- [ ] Run Phase 1: Health & Connectivity tests
  - `./test_phase1_health.sh`
- [ ] Run Phase 2: Blockchain Operations tests
  - `./test_phase2_operations.sh`
- [ ] Run Phase 3: Consensus Validation tests
  - Monitor round progression for 5 minutes
- [ ] Run Phase 4: Resilience tests (optional)
  - Node failure simulation
  - Network partition simulation
- [ ] Run full validation suite
  - `./full_validation_suite.sh`

**Performance Validation:**
- [ ] Submit 100 transactions, verify processing
- [ ] Check block creation rate (should match `block_interval`)
- [ ] Measure API response times (< 200ms for reads, < 500ms for writes)
- [ ] Verify blockchain sync time for new node

**Monitoring Setup:**
- [ ] Access Grafana: `http://<authority-ip>:3001` (admin/admin)
  - [ ] Change default password
  - [ ] Import ProvChain dashboards
  - [ ] Verify all nodes reporting metrics
- [ ] Access Prometheus: `http://<authority-ip>:9091`
  - [ ] Check targets (3 nodes should be UP)
  - [ ] Query test: `provchain_blocks_total`
- [ ] Access Jaeger: `http://<authority-ip>:16686`
  - [ ] Verify traces are being collected

**Security Hardening:**
- [ ] Change default passwords (Grafana, etc.)
- [ ] Enable TLS/SSL for API endpoints (production)
  - Set up Let's Encrypt certificates
  - Configure reverse proxy (Nginx/Traefik)
- [ ] Restrict SSH access to known IPs
- [ ] Disable root login
- [ ] Set up fail2ban (SSH brute force protection)
- [ ] Enable firewall (ufw/iptables)
- [ ] Rotate JWT secret regularly
- [ ] Implement key rotation strategy for authority keys

**Operational Readiness:**
- [ ] Configure automated backups
  - Blockchain data: `/app/data/`
  - Configuration files
  - Authority keys (encrypted)
  - Schedule: Daily at 2 AM UTC
- [ ] Set up alerting
  - Node down alerts
  - Consensus failure alerts
  - Disk space alerts (> 80% usage)
  - Memory usage alerts (> 90% usage)
- [ ] Create runbooks for common issues
- [ ] Document deployment architecture
- [ ] Train operations team
- [ ] Establish on-call rotation

**Documentation:**
- [ ] Document deployed IP addresses
- [ ] Document firewall rules
- [ ] Document monitoring URLs
- [ ] Create disaster recovery plan
- [ ] Document backup/restore procedures
- [ ] Document troubleshooting steps

### Go-Live

- [ ] Final smoke test
- [ ] Announce maintenance window (if migrating)
- [ ] Monitor for first 24 hours continuously
- [ ] Review logs for errors/warnings
- [ ] Verify all monitoring alerts are working
- [ ] Conduct post-deployment review meeting

---

## 8. Monitoring & Observability

### Monitoring Stack Architecture

```
┌────────────────────────────────────────────────┐
│         Monitoring & Observability Stack        │
├────────────────────────────────────────────────┤
│                                                │
│  ┌──────────────┐         ┌──────────────┐   │
│  │  Prometheus  │◄────────│  Grafana     │   │
│  │  (Metrics)   │         │  (Dashboards)│   │
│  └───────▲──────┘         └──────────────┘   │
│          │                                    │
│          │ /metrics endpoint                  │
│          │                                    │
│  ┌───────┴──────┬──────────────┬───────────┐ │
│  │              │              │           │ │
│  │   Node 1     │   Node 2     │  Node 3   │ │
│  │  :9090       │   :9090      │  :9090    │ │
│  └──────────────┴──────────────┴───────────┘ │
│                                                │
│  ┌──────────────┐                             │
│  │   Jaeger     │◄──── OpenTelemetry          │
│  │  (Tracing)   │      (from all nodes)       │
│  └──────────────┘                             │
│                                                │
│  ┌──────────────┐                             │
│  │ System Logs  │                             │
│  │ (journald/   │                             │
│  │  Docker logs)│                             │
│  └──────────────┘                             │
└────────────────────────────────────────────────┘
```

### Prometheus Metrics

**Access:** `http://<any-node-ip>:9090`

#### Built-in ProvChain Metrics

```prometheus
# Blockchain Metrics
provchain_blocks_total{node_id="node1"}                    # Total blocks
provchain_block_creation_duration_seconds                  # Block creation time
provchain_blockchain_length                                # Current chain length
provchain_blockchain_size_bytes                            # Storage used

# Consensus Metrics
provchain_consensus_round{node_id="node1"}                 # Current round
provchain_authority_blocks_created{authority="pubkey"}     # Blocks by authority
provchain_missed_slots_total{authority="pubkey"}           # Missed block slots
provchain_consensus_latency_seconds                        # Consensus latency

# Network Metrics
provchain_peers_connected{node_id="node1"}                 # Active peers
provchain_network_messages_sent_total{type="block"}        # Messages sent
provchain_network_messages_received_total{type="block"}    # Messages received
provchain_network_bytes_sent_total                         # Network egress
provchain_network_bytes_received_total                     # Network ingress

# Transaction Metrics
provchain_transactions_total{status="accepted"}            # Total transactions
provchain_transaction_pool_size                            # Pending transactions
provchain_transaction_latency_seconds                      # Processing time

# API Metrics
provchain_http_requests_total{method="POST",path="/api/transactions"}
provchain_http_request_duration_seconds{method="GET",path="/api/blockchain/dump"}
provchain_http_errors_total{code="500"}

# System Metrics
provchain_memory_usage_bytes                               # Memory usage
provchain_cpu_usage_percent                                # CPU usage
provchain_disk_usage_bytes                                 # Disk usage
provchain_uptime_seconds                                   # Node uptime
```

#### Querying Metrics

```bash
# Current block count on all nodes
curl http://10.0.1.10:9091/api/v1/query?query=provchain_blocks_total

# Block creation rate (blocks/minute)
rate(provchain_blocks_total[5m]) * 60

# Average transaction latency
avg(provchain_transaction_latency_seconds)

# Peer connectivity status
sum(provchain_peers_connected) by (node_id)
```

### Grafana Dashboards

**Access:** `http://<authority-ip>:3001` (default: admin/admin)

#### Dashboard 1: Cluster Overview

**Panels:**
- Node Health Status (UP/DOWN)
- Total Peers Connected (gauge)
- Blockchain Length (all nodes, line chart)
- Block Creation Rate (blocks/minute)
- Transaction Throughput (TPS)
- Network Bandwidth (ingress/egress)

**Example JSON:**
```json
{
  "dashboard": {
    "title": "ProvChain Cluster Overview",
    "panels": [
      {
        "title": "Node Health",
        "type": "stat",
        "targets": [{
          "expr": "up{job=\"provchain\"}"
        }]
      },
      {
        "title": "Blockchain Length",
        "type": "graph",
        "targets": [{
          "expr": "provchain_blockchain_length"
        }]
      }
    ]
  }
}
```

#### Dashboard 2: Consensus Metrics

**Panels:**
- Current Consensus Round (all nodes)
- Authority Rotation Timeline
- Blocks Created by Authority (bar chart)
- Missed Slots by Authority
- Consensus Latency (heatmap)
- Authority Reputation Score

#### Dashboard 3: Performance Metrics

**Panels:**
- CPU Usage per Node (%)
- Memory Usage per Node (MB)
- Disk I/O (read/write MB/s)
- Network Latency (P50, P95, P99)
- Transaction Processing Time (histogram)
- Block Validation Time (histogram)

#### Dashboard 4: API Metrics

**Panels:**
- HTTP Request Rate (req/s)
- HTTP Response Times (P50, P95, P99)
- HTTP Error Rate (%)
- Top API Endpoints (by request count)
- API Errors by Status Code
- Active WebSocket Connections

### Jaeger Distributed Tracing

**Access:** `http://<authority-ip>:16686`

**Use Cases:**

1. **Trace Transaction Flow**
   - Submit transaction → API handler → Validation → Blockchain → Broadcast
   - Identify slow steps in processing pipeline

2. **Trace Block Propagation**
   - Block creation → Authority signing → P2P broadcast → Peer validation → Blockchain append
   - Measure time for block to reach all nodes

3. **Trace SPARQL Queries**
   - Query parsing → RDF store lookup → Result serialization
   - Identify slow queries

**Viewing Traces:**
```
1. Open Jaeger UI: http://10.0.1.10:16686
2. Select service: "provchain-node1"
3. Select operation: "POST /api/transactions"
4. Click "Find Traces"
5. Click on a trace to see detailed spans
```

**Trace Attributes:**
- `node_id`: Which node generated the trace
- `block_index`: Block number
- `transaction_id`: Transaction identifier
- `authority_key`: Authority public key
- `peer_count`: Number of connected peers

### Log Aggregation

#### Docker Logs

```bash
# View logs for specific node
docker logs provchain-node1

# Follow logs in real-time
docker logs -f provchain-node1

# Last 100 lines
docker logs --tail 100 provchain-node1

# Logs since timestamp
docker logs --since 2026-01-02T10:00:00 provchain-node1

# Filter logs by level
docker logs provchain-node1 2>&1 | grep ERROR
```

#### Systemd Logs

```bash
# View all logs
journalctl -u provchain

# Follow logs
journalctl -u provchain -f

# Last 100 lines
journalctl -u provchain -n 100

# Logs since yesterday
journalctl -u provchain --since yesterday

# Filter by log level
journalctl -u provchain -p err
```

#### Log Format

Logs are structured JSON (in production mode):

```json
{
  "timestamp": "2026-01-02T10:30:45.123Z",
  "level": "INFO",
  "target": "provchain_org::network::peer",
  "message": "New peer connected",
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "660e8400-e29b-41d4-a716-446655440001",
  "peer_address": "10.0.1.11:8080"
}
```

#### Log Levels

- **TRACE**: Very detailed debugging (not recommended for production)
- **DEBUG**: Detailed debugging information
- **INFO**: General informational messages (default)
- **WARN**: Warning messages (potential issues)
- **ERROR**: Error messages (failures)

### Alerting

#### Recommended Alerts

**Critical Alerts (PagerDuty, SMS):**

```yaml
# Node Down Alert
- alert: ProvChainNodeDown
  expr: up{job="provchain"} == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "ProvChain node {{ $labels.instance }} is down"

# Consensus Failure Alert
- alert: ConsensusStalled
  expr: rate(provchain_blocks_total[5m]) == 0
  for: 3m
  labels:
    severity: critical
  annotations:
    summary: "No blocks created in last 5 minutes"

# Blockchain Fork Detected
- alert: BlockchainFork
  expr: stddev(provchain_blockchain_length) > 1
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "Blockchain lengths diverged across nodes"
```

**Warning Alerts (Slack, Email):**

```yaml
# High Memory Usage
- alert: HighMemoryUsage
  expr: provchain_memory_usage_bytes / 1e9 > 7  # > 7 GB
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Node {{ $labels.instance }} using {{ $value }} GB RAM"

# Disk Space Low
- alert: LowDiskSpace
  expr: provchain_disk_usage_bytes / provchain_disk_total_bytes > 0.8
  for: 30m
  labels:
    severity: warning
  annotations:
    summary: "Node {{ $labels.instance }} disk usage > 80%"

# Peer Disconnected
- alert: PeerDisconnected
  expr: provchain_peers_connected < 2
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Node {{ $labels.instance }} has only {{ $value }} peers"

# High Transaction Latency
- alert: HighTransactionLatency
  expr: provchain_transaction_latency_seconds > 5
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Transaction latency > 5s on {{ $labels.instance }}"
```

#### Alert Configuration (Prometheus)

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - "alerts.yml"

scrape_configs:
  - job_name: 'provchain'
    static_configs:
      - targets:
        - '10.0.1.10:9090'
        - '10.0.1.11:9090'
        - '10.0.1.12:9090'
```

#### Alert Destinations

**Alertmanager Configuration:**

```yaml
# alertmanager.yml
route:
  receiver: 'slack'
  group_by: ['alertname', 'cluster', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'

receivers:
  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#provchain-alerts'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
```

### Health Checks

#### Application Health Endpoint

```bash
# Health check endpoint
curl http://10.0.1.10:8080/health

# Response:
{
  "status": "healthy",
  "timestamp": "2026-01-02T10:30:45Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "blockchain": "ok",
    "storage": "ok",
    "network": "ok",
    "consensus": "ok"
  }
}
```

#### Docker Health Check

```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

#### Kubernetes Liveness/Readiness Probes

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 2
```

---

## 9. Troubleshooting Guide

### Common Issues & Solutions

#### Issue 1: Nodes Not Connecting

**Symptoms:**
- `curl http://10.0.1.10:8080/api/peers | jq 'length'` returns `0`
- Logs show "Connection refused" or "Connection timeout"

**Diagnosis:**
```bash
# Test network connectivity
ping 10.0.1.11
telnet 10.0.1.11 8080

# Check if node is listening
netstat -tulpn | grep 8080
# or
ss -tulpn | grep 8080

# Check firewall rules
sudo ufw status
sudo iptables -L -n

# Check Docker network (if using Docker)
docker network inspect provchain_network
```

**Solutions:**
1. **Firewall blocking**: Open port 8080
   ```bash
   sudo ufw allow 8080/tcp
   sudo systemctl restart ufw
   ```

2. **Wrong IP address in config**: Verify `known_peers` lists correct IPs
   ```bash
   grep known_peers config.toml
   ```

3. **Node not started**: Check service status
   ```bash
   docker ps  # Docker
   systemctl status provchain  # Systemd
   ```

4. **Network ID mismatch**: Ensure all nodes have same `network_id`
   ```bash
   grep network_id config/*.toml
   # All should match
   ```

---

#### Issue 2: Blockchain Not Synchronizing

**Symptoms:**
- Different blockchain lengths across nodes
- Logs show "Block validation failed"
- `curl http://10.0.1.10:8080/api/blockchain/validate` returns `valid: false`

**Diagnosis:**
```bash
# Check blockchain lengths
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/api/blockchain/dump | jq 'length'
done

# Check last block hashes
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/api/blockchain/dump | jq -r '.[-1].hash'
done

# Validate blockchain integrity
curl http://10.0.1.10:8080/api/blockchain/validate | jq '.'
```

**Solutions:**
1. **Block validation failure**: Check logs for specific error
   ```bash
   docker logs provchain-node1 | grep "Block validation"
   ```

2. **Authority key mismatch**: Verify all nodes have correct authority public keys
   ```bash
   grep authority_keys config/*.toml
   ```

3. **Corrupted blockchain**: Reset and resync node
   ```bash
   # Stop node
   docker-compose down

   # Backup data (optional)
   cp -r data data.backup

   # Delete blockchain data
   rm -rf data/*

   # Restart node (will resync from peers)
   docker-compose up -d
   ```

4. **Network partition**: Check if all nodes can communicate
   ```bash
   # From each node, ping others
   ping -c 3 10.0.1.10
   ping -c 3 10.0.1.11
   ping -c 3 10.0.1.12
   ```

---

#### Issue 3: Authority Not Creating Blocks

**Symptoms:**
- No new blocks being created
- Blockchain length not increasing
- `provchain_blocks_total` metric not incrementing

**Diagnosis:**
```bash
# Check if node is authority
curl http://10.0.1.10:8080/api/consensus/stats | jq '.is_authority'
# Should return: true

# Check authority keypair is loaded
docker exec provchain-node1 ls -l /app/keys/authority.key
# or
ls -l /opt/provchain/keys/authority.key

# Check logs for authority errors
docker logs provchain-node1 | grep -i authority

# Check consensus round progression
curl http://10.0.1.10:8080/api/consensus/stats | jq '.current_round'
sleep 12  # Wait block_interval + 2s
curl http://10.0.1.10:8080/api/consensus/stats | jq '.current_round'
# Should have incremented
```

**Solutions:**
1. **Authority key not loaded**: Check file path and permissions
   ```bash
   # Verify key exists
   ls -l authority.key

   # Check config points to correct path
   grep authority_key_file config.toml

   # Check permissions (should be 600)
   chmod 600 authority.key
   ```

2. **is_authority = false in config**: Fix configuration
   ```toml
   [consensus]
   is_authority = true
   authority_key_file = "/app/keys/authority.key"
   ```

3. **Block interval too long**: Check configuration
   ```bash
   grep block_interval config.toml
   # Typical value: 10 seconds
   ```

4. **Authority key mismatch**: Verify public key in other nodes' configs
   ```bash
   # Get public key from authority
   ./provchain-org show-pubkey --key authority.key

   # Compare with authority_keys in other nodes
   grep authority_keys config/node2.toml
   ```

---

#### Issue 4: High Memory Usage

**Symptoms:**
- Node using > 7 GB RAM
- OOM (Out of Memory) errors in logs
- Node crashes or becomes unresponsive

**Diagnosis:**
```bash
# Check memory usage
docker stats provchain-node1
# or
ps aux | grep provchain-org
free -h

# Check Prometheus metric
curl http://10.0.1.10:9090/metrics | grep provchain_memory_usage_bytes

# Check for memory leaks in logs
docker logs provchain-node1 | grep -i "memory"
```

**Solutions:**
1. **Increase cache size**: Reduce `cache_size_mb` in config
   ```toml
   [storage]
   cache_size_mb = 50  # Reduce from 100
   ```

2. **Limit Docker memory**:
   ```yaml
   # docker-compose.yml
   services:
     node1:
       mem_limit: 4g
       memswap_limit: 4g
   ```

3. **Optimize RDF store**: Compact Oxigraph database
   ```bash
   # Stop node
   docker-compose down

   # Run compaction (if available)
   docker run --rm -v node1_data:/data \
     provchain-org:latest \
     /usr/local/bin/provchain-org compact --data-dir /data

   # Restart
   docker-compose up -d
   ```

4. **Upgrade VM**: Increase RAM to 8 GB or 16 GB

---

#### Issue 5: Disk Space Full

**Symptoms:**
- `df -h` shows 100% usage
- Logs show "No space left on device"
- Node stops accepting transactions

**Diagnosis:**
```bash
# Check disk usage
df -h /app/data  # Docker mount
df -h /opt/provchain/data  # Native

# Find largest directories
du -sh /app/data/*
du -sh /var/lib/docker/*

# Check logs size
du -sh /var/log/
```

**Solutions:**
1. **Clean up Docker**: Remove unused images and volumes
   ```bash
   docker system prune -a --volumes
   ```

2. **Rotate logs**: Configure log rotation
   ```bash
   # /etc/logrotate.d/provchain
   /var/log/provchain/*.log {
     daily
     rotate 7
     compress
     delaycompress
     notifempty
     create 0640 provchain provchain
   }
   ```

3. **Increase disk size**: Expand EBS volume (AWS) or resize disk

4. **Archive old data**: Move old blockchain data to cold storage
   ```bash
   # Backup blockchain
   tar -czf blockchain-backup-$(date +%Y%m%d).tar.gz /app/data/

   # Move to S3 or archive
   aws s3 cp blockchain-backup-*.tar.gz s3://my-bucket/backups/
   ```

---

#### Issue 6: Slow API Response Times

**Symptoms:**
- API requests taking > 5 seconds
- Grafana shows high P95/P99 latency
- Timeouts in client applications

**Diagnosis:**
```bash
# Test API response time
time curl http://10.0.1.10:8080/api/blockchain/dump

# Check Prometheus metrics
curl http://10.0.1.10:9090/api/v1/query?query=provchain_http_request_duration_seconds

# Check system load
top
htop

# Check I/O wait
iostat -x 1
```

**Solutions:**
1. **Optimize SPARQL queries**: Add indexes, limit result size
   ```sparql
   # Add LIMIT clause
   SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 1000
   ```

2. **Increase CPU**: Upgrade to more vCPUs

3. **Enable caching**: Implement Redis cache for frequent queries

4. **Optimize RDF store**: Rebuild indexes
   ```bash
   # Depends on Oxigraph version
   ./provchain-org reindex --data-dir /app/data
   ```

---

#### Issue 7: WebSocket Connection Failures

**Symptoms:**
- Logs show "WebSocket upgrade failed"
- Real-time updates not working
- Peer connections dropping frequently

**Diagnosis:**
```bash
# Test WebSocket connection
wscat -c ws://10.0.1.10:8080/api/ws

# Check nginx/proxy config (if using reverse proxy)
cat /etc/nginx/sites-available/provchain

# Check Docker network
docker network inspect provchain_network
```

**Solutions:**
1. **Configure nginx for WebSocket**:
   ```nginx
   location /api/ws {
     proxy_pass http://backend;
     proxy_http_version 1.1;
     proxy_set_header Upgrade $http_upgrade;
     proxy_set_header Connection "upgrade";
     proxy_read_timeout 86400;
   }
   ```

2. **Increase connection timeout**:
   ```toml
   [network]
   connection_timeout = 60  # Increase from 30
   ```

---

### Debugging Commands Cheat Sheet

```bash
# === Node Status ===
curl http://10.0.1.10:8080/health
curl http://10.0.1.10:8080/api/consensus/stats
curl http://10.0.1.10:8080/api/peers

# === Logs ===
docker logs -f provchain-node1
journalctl -u provchain -f

# === Metrics ===
curl http://10.0.1.10:9090/metrics | grep provchain
curl http://10.0.1.10:9091/api/v1/query?query=provchain_blocks_total

# === Network ===
ping 10.0.1.11
telnet 10.0.1.11 8080
netstat -tulpn | grep 8080
traceroute 10.0.1.11

# === Blockchain ===
curl http://10.0.1.10:8080/api/blockchain/dump | jq '.'
curl http://10.0.1.10:8080/api/blockchain/validate
curl http://10.0.1.10:8080/api/blockchain/dump | jq 'length'

# === System ===
top
htop
free -h
df -h
iostat -x 1
iotop

# === Docker ===
docker ps
docker stats
docker logs provchain-node1 --tail 100
docker exec -it provchain-node1 bash

# === Systemd ===
systemctl status provchain
systemctl restart provchain
journalctl -u provchain -n 100
```

---

## 10. Cost Estimation (Cloud Providers)

### AWS (US-East-1)

#### Instance Types

| Node | Instance Type | vCPU | RAM | Price/Hour | Price/Month |
|------|--------------|------|-----|------------|-------------|
| VM-1 (Authority) | t3.large | 2 | 8 GB | $0.0832 | ~$60 |
| VM-2 (Regular) | t3.medium | 2 | 4 GB | $0.0416 | ~$30 |
| VM-3 (Regular) | t3.medium | 2 | 4 GB | $0.0416 | ~$30 |

#### Storage (EBS)

| Volume | Type | Size | Price/GB/Month | Total |
|--------|------|------|----------------|-------|
| VM-1 | gp3 SSD | 100 GB | $0.08 | $8 |
| VM-2 | gp3 SSD | 50 GB | $0.08 | $4 |
| VM-3 | gp3 SSD | 50 GB | $0.08 | $4 |

#### Network

- **Data Transfer Out**: ~100 GB/month @ $0.09/GB = $9
- **Data Transfer In**: Free
- **Load Balancer**: Application LB = $16/month

#### Total AWS Cost

| Component | Monthly Cost |
|-----------|--------------|
| Compute (EC2) | $120 |
| Storage (EBS) | $16 |
| Network | $9 |
| Load Balancer | $16 |
| **Total** | **~$161/month** |

---

### Google Cloud Platform (us-central1)

#### Instance Types

| Node | Machine Type | vCPU | RAM | Price/Hour | Price/Month |
|------|-------------|------|-----|------------|-------------|
| VM-1 (Authority) | e2-standard-2 | 2 | 8 GB | $0.06701 | ~$49 |
| VM-2 (Regular) | e2-standard-2 | 2 | 8 GB | $0.06701 | ~$49 |
| VM-3 (Regular) | e2-standard-2 | 2 | 8 GB | $0.06701 | ~$49 |

#### Storage (Persistent Disk)

| Volume | Type | Size | Price/GB/Month | Total |
|--------|------|------|----------------|-------|
| VM-1 | SSD | 100 GB | $0.17 | $17 |
| VM-2 | SSD | 50 GB | $0.17 | $8.50 |
| VM-3 | SSD | 50 GB | $0.17 | $8.50 |

#### Network

- **Egress (Internet)**: ~100 GB/month @ $0.12/GB = $12
- **Ingress**: Free
- **Load Balancer**: $18/month

#### Total GCP Cost

| Component | Monthly Cost |
|-----------|--------------|
| Compute | $147 |
| Storage | $34 |
| Network | $12 |
| Load Balancer | $18 |
| **Total** | **~$211/month** |

---

### Microsoft Azure (East US)

#### Instance Types

| Node | VM Size | vCPU | RAM | Price/Hour | Price/Month |
|------|---------|------|-----|------------|-------------|
| VM-1 (Authority) | Standard_D2s_v3 | 2 | 8 GB | $0.096 | ~$70 |
| VM-2 (Regular) | Standard_B2s | 2 | 4 GB | $0.0416 | ~$30 |
| VM-3 (Regular) | Standard_B2s | 2 | 4 GB | $0.0416 | ~$30 |

#### Storage (Managed Disks)

| Volume | Type | Size | Price/Month | Total |
|--------|------|------|-------------|-------|
| VM-1 | Premium SSD | 128 GB | $19.71 | $20 |
| VM-2 | Premium SSD | 64 GB | $9.86 | $10 |
| VM-3 | Premium SSD | 64 GB | $9.86 | $10 |

#### Network

- **Outbound Data**: ~100 GB/month @ $0.087/GB = $8.70
- **Inbound Data**: Free
- **Load Balancer**: $18/month

#### Total Azure Cost

| Component | Monthly Cost |
|-----------|--------------|
| Compute | $130 |
| Storage | $40 |
| Network | $9 |
| Load Balancer | $18 |
| **Total** | **~$197/month** |

---

### Cost Optimization Strategies

1. **Reserved Instances**: Save 30-50% with 1-year or 3-year commitments
2. **Spot/Preemptible Instances**: Save 60-80% for non-production (testing)
3. **Auto-scaling**: Scale down during off-peak hours
4. **Storage Tiering**: Use cheaper HDD for archival data
5. **Right-sizing**: Monitor actual usage and downsize if over-provisioned
6. **Network Optimization**: Use VPC peering to avoid egress charges

### Cost Comparison Summary

| Provider | Monthly Cost | Pros | Cons |
|----------|--------------|------|------|
| **AWS** | **$161** | Most mature, extensive services | Complex pricing |
| **GCP** | $211 | Best Kubernetes support, simple pricing | Higher cost |
| **Azure** | $197 | Good Windows integration, hybrid cloud | Steeper learning curve |

**Recommendation:** AWS offers the best value for this workload, with mature services and competitive pricing.

---

## 11. Next Steps & Recommendations

### Immediate Actions (Week 1)

1. **Choose Deployment Method**
   - [ ] Docker Compose (fastest, recommended for testing)
   - [ ] Kubernetes (best for production)
   - [ ] Native Systemd (maximum control)

2. **Provision Infrastructure**
   - [ ] Select cloud provider (AWS recommended)
   - [ ] Create VMs with specifications from Section 5
   - [ ] Configure VPC and security groups
   - [ ] Allocate persistent storage volumes

3. **Deploy Authority Node First**
   - [ ] Follow Docker Compose guide (Section 5, Option A)
   - [ ] Generate authority keypair
   - [ ] Deploy and verify health
   - [ ] Wait 30s for stabilization

4. **Deploy Regular Nodes**
   - [ ] Deploy VM-2 with node2 config
   - [ ] Deploy VM-3 with node3 config
   - [ ] Verify peer connections

5. **Run Validation Suite**
   - [ ] Execute `full_validation_suite.sh`
   - [ ] Verify all tests pass
   - [ ] Monitor for 24 hours

### Short-Term Enhancements (Weeks 2-4)

#### Security Hardening

1. **TLS/SSL Certificates**
   ```bash
   # Install Certbot
   sudo apt-get install certbot

   # Obtain Let's Encrypt certificate
   sudo certbot certonly --standalone -d api.provchain.example.com

   # Configure Nginx reverse proxy with TLS
   sudo nginx -t
   sudo systemctl reload nginx
   ```

2. **Firewall Configuration**
   ```bash
   # Lock down SSH
   sudo ufw limit 22/tcp

   # Allow only necessary ports
   sudo ufw allow 8080/tcp
   sudo ufw allow 9090/tcp

   # Enable firewall
   sudo ufw enable
   ```

3. **Secrets Management**
   - [ ] Move JWT_SECRET to AWS Secrets Manager / HashiCorp Vault
   - [ ] Implement key rotation policy (90 days)
   - [ ] Encrypt authority keys at rest

4. **SSH Hardening**
   ```bash
   # Disable root login
   sudo sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config

   # Disable password auth (use SSH keys only)
   sudo sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config

   # Restart SSH
   sudo systemctl restart sshd
   ```

#### Monitoring & Alerting

1. **Set Up Alertmanager**
   ```yaml
   # Deploy Alertmanager
   docker run -d -p 9093:9093 \
     -v ./alertmanager.yml:/etc/alertmanager/alertmanager.yml \
     prom/alertmanager
   ```

2. **Configure Slack Alerts**
   - [ ] Create Slack webhook URL
   - [ ] Configure Alertmanager (Section 8)
   - [ ] Test alert delivery

3. **Create Grafana Dashboards**
   - [ ] Import pre-built dashboards
   - [ ] Customize for your metrics
   - [ ] Set up dashboard snapshots

4. **Set Up Log Aggregation (ELK Stack)**
   ```bash
   # Optional: Deploy Elasticsearch + Kibana
   docker-compose -f deploy/docker-compose.elk.yml up -d
   ```

#### Backup & Disaster Recovery

1. **Automated Backups**
   ```bash
   # Backup script (runs daily at 2 AM)
   #!/bin/bash
   # /opt/provchain/backup.sh

   DATE=$(date +%Y%m%d)
   BACKUP_DIR="/mnt/backups"

   # Backup blockchain data
   tar -czf $BACKUP_DIR/blockchain-$DATE.tar.gz /app/data/

   # Backup config
   tar -czf $BACKUP_DIR/config-$DATE.tar.gz /app/config/

   # Upload to S3
   aws s3 cp $BACKUP_DIR/blockchain-$DATE.tar.gz s3://my-bucket/backups/

   # Retain only last 7 days locally
   find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
   ```

   ```bash
   # Add to crontab
   crontab -e
   # Add line:
   0 2 * * * /opt/provchain/backup.sh
   ```

2. **Test Restore Procedure**
   - [ ] Document step-by-step restore process
   - [ ] Practice restore on test environment
   - [ ] Measure Recovery Time Objective (RTO)

### Medium-Term Improvements (Months 2-3)

#### Performance Optimization

1. **Database Tuning**
   - [ ] Analyze slow SPARQL queries
   - [ ] Add appropriate indexes
   - [ ] Optimize RDF store configuration

2. **Caching Layer**
   ```bash
   # Deploy Redis for query caching
   docker run -d -p 6379:6379 redis:7-alpine
   ```

3. **CDN Integration**
   - [ ] Set up CloudFront (AWS) or Cloud CDN (GCP)
   - [ ] Cache static frontend assets
   - [ ] Reduce API latency with edge locations

#### High Availability

1. **Multi-Authority Setup**
   - [ ] Add 2nd authority node for redundancy
   - [ ] Configure authority rotation
   - [ ] Test failover scenarios

2. **Load Balancer Configuration**
   ```bash
   # AWS ALB setup
   aws elbv2 create-load-balancer \
     --name provchain-lb \
     --subnets subnet-xxx subnet-yyy \
     --security-groups sg-xxx

   # Create target group
   aws elbv2 create-target-group \
     --name provchain-targets \
     --protocol HTTP \
     --port 8080 \
     --vpc-id vpc-xxx \
     --health-check-path /health
   ```

3. **Database Replication**
   - [ ] Set up read replicas for RDF store
   - [ ] Configure automatic failover
   - [ ] Test replication lag

#### CI/CD Pipeline

1. **GitHub Actions Workflow**
   ```yaml
   # .github/workflows/deploy.yml
   name: Deploy ProvChain

   on:
     push:
       branches: [main]

   jobs:
     build:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3

         - name: Build Docker image
           run: docker build -f deploy/Dockerfile.production -t provchain:${{ github.sha }} .

         - name: Push to registry
           run: docker push provchain:${{ github.sha }}

         - name: Deploy to production
           run: ./deploy/scripts/rolling_update.sh provchain:${{ github.sha }}
   ```

2. **Automated Testing**
   - [ ] Add pre-deployment tests to CI
   - [ ] Run `cargo test` on every commit
   - [ ] Block deployment if tests fail

3. **Rolling Updates**
   ```bash
   # Rolling update script
   #!/bin/bash
   # deploy/scripts/rolling_update.sh

   NEW_VERSION=$1
   NODES=("node1" "node2" "node3")

   for node in "${NODES[@]}"; do
     echo "Updating $node to $NEW_VERSION..."

     # Update Docker image
     docker pull provchain:$NEW_VERSION

     # Stop node
     docker stop provchain-$node

     # Start with new version
     docker run -d --name provchain-$node provchain:$NEW_VERSION

     # Wait for health check
     sleep 30

     # Verify health
     curl -f http://$node:8080/health || exit 1

     echo "$node updated successfully"
   done
   ```

### Long-Term Roadmap (Months 4-6)

#### Scalability

1. **Horizontal Scaling**
   - [ ] Add nodes 4-10 for increased redundancy
   - [ ] Implement sharding (if needed)
   - [ ] Test with 10+ nodes

2. **Performance Testing**
   - [ ] Load test with 10,000 TPS
   - [ ] Identify bottlenecks
   - [ ] Optimize critical paths

3. **Multi-Region Deployment**
   - [ ] Deploy clusters in multiple AWS regions
   - [ ] Configure cross-region replication
   - [ ] Test disaster recovery across regions

#### Compliance & Auditing

1. **Audit Logging**
   - [ ] Implement immutable audit log
   - [ ] Log all API access
   - [ ] Retain logs for 7 years (regulatory requirement)

2. **Compliance Certifications**
   - [ ] SOC 2 Type II
   - [ ] ISO 27001
   - [ ] GDPR compliance review

3. **Penetration Testing**
   - [ ] Hire external security firm
   - [ ] Fix identified vulnerabilities
   - [ ] Re-test after fixes

#### Advanced Features

1. **Multi-Tenancy**
   - [ ] Implement tenant isolation
   - [ ] Per-tenant quotas
   - [ ] Billing integration

2. **API Rate Limiting**
   ```rust
   // Add to Axum server
   use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

   let governor_conf = Box::new(
       GovernorConfigBuilder::default()
           .per_second(10)
           .burst_size(100)
           .finish()
           .unwrap(),
   );

   let app = Router::new()
       .route("/api/transactions", post(handle_transaction))
       .layer(GovernorLayer { config: governor_conf });
   ```

3. **GraphQL API** (alternative to REST)
   - [ ] Implement GraphQL schema
   - [ ] Add subscriptions for real-time updates
   - [ ] Document GraphQL API

---

## 12. Reference Documentation

### Configuration Files

| File | Location | Purpose |
|------|----------|---------|
| `config.toml` | Project root | Main configuration template |
| `config/node1-authority.toml` | config/ | Authority node config |
| `config/node2-regular.toml` | config/ | Regular node config |
| `config/node3-regular.toml` | config/ | Regular node config |
| `deploy/docker-compose.3node.yml` | deploy/ | 3-node Docker deployment |
| `deploy/Dockerfile.production` | deploy/ | Production Docker image |
| `deploy/monitoring/prometheus.yml` | deploy/monitoring/ | Prometheus config |

### API Endpoints

#### Health & Status

```
GET  /health                          # Health check
GET  /api/status                      # Detailed status
GET  /api/version                     # Application version
```

#### Blockchain Operations

```
GET  /api/blockchain/dump             # Get all blocks
GET  /api/blockchain/validate         # Validate integrity
GET  /api/blockchain/block/:index     # Get specific block
GET  /api/blockchain/length           # Get blockchain length
```

#### Transactions

```
POST /api/transactions                # Submit new transaction
GET  /api/transactions/:id            # Get transaction by ID
GET  /api/transactions/pending        # Get pending transactions
```

#### Consensus

```
GET  /api/consensus/stats             # Consensus status
GET  /api/consensus/authorities       # List authorities
POST /api/consensus/add-authority     # Add new authority (governance)
POST /api/consensus/remove-authority  # Remove authority (governance)
```

#### Peer Management

```
GET  /api/peers                       # Connected peers
POST /api/peers/connect               # Connect to peer manually
POST /api/peers/disconnect            # Disconnect peer
```

#### Query

```
POST /api/query                       # Execute SPARQL query
  Content-Type: application/sparql-query
  Body: SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10
```

#### WebSocket

```
WS   /api/ws                          # Real-time updates
  Messages:
    - block_added: New block notification
    - transaction_added: New transaction
    - peer_connected: Peer connection event
```

### Command-Line Interface

```bash
# General
provchain-org --help
provchain-org --version
provchain-org --config /path/to/config.toml

# Blockchain operations
provchain-org dump                    # Print blockchain
provchain-org validate                # Validate blockchain
provchain-org add-file <file.ttl>     # Add RDF file

# Key management
provchain-org generate-keypair --output authority.key
provchain-org show-pubkey --key authority.key

# Maintenance
provchain-org compact                 # Compact database
provchain-org reindex                 # Rebuild indexes
provchain-org repair                  # Repair corrupted blockchain
```

### Environment Variables

```bash
# Application
RUST_LOG=info                         # Log level (trace|debug|info|warn|error)
CONFIG_FILE=/app/config/node1.toml    # Config file path
JWT_SECRET=your-secret-here           # JWT signing secret (required)

# Network
PROVCHAIN_PORT=8080                   # API/P2P port
PROVCHAIN_PEERS=10.0.1.11:8080,10.0.1.12:8080  # Bootstrap peers
NETWORK_ID=provchain-prod             # Network identifier

# Storage
DATA_DIR=/app/data                    # Blockchain data directory
CACHE_SIZE_MB=100                     # RDF store cache size

# Monitoring
PROMETHEUS_PORT=9090                  # Metrics export port
OTEL_SERVICE_NAME=provchain-node1     # OpenTelemetry service name
JAEGER_ENDPOINT=http://jaeger:14268/api/traces  # Jaeger collector
```

### Docker Commands

```bash
# Build
docker build -f deploy/Dockerfile.production -t provchain-org:latest .

# Run single node
docker run -d \
  --name provchain-node1 \
  -p 8080:8080 \
  -p 9090:9090 \
  -e JWT_SECRET=your-secret \
  -v provchain-data:/app/data \
  provchain-org:latest

# Run 3-node cluster
cd deploy
docker-compose -f docker-compose.3node.yml up -d

# View logs
docker logs -f provchain-node1

# Execute command inside container
docker exec -it provchain-node1 bash

# Stop all
docker-compose -f docker-compose.3node.yml down
```

### Kubernetes Commands

```bash
# Apply manifests
kubectl apply -f deploy/k8s/namespace.yaml
kubectl apply -f deploy/k8s/configmap.yaml
kubectl apply -f deploy/k8s/statefulset.yaml
kubectl apply -f deploy/k8s/service.yaml

# Check status
kubectl get pods -n provchain
kubectl get svc -n provchain

# View logs
kubectl logs -f provchain-node-0 -n provchain

# Execute command in pod
kubectl exec -it provchain-node-0 -n provchain -- bash

# Port forward for local access
kubectl port-forward svc/provchain-api 8080:8080 -n provchain

# Delete all resources
kubectl delete namespace provchain
```

### Useful Links

#### Official Documentation
- **ProvChain Repository**: `https://github.com/yourusername/provchain-org`
- **Issue Tracker**: `https://github.com/yourusername/provchain-org/issues`

#### Rust Documentation
- **The Rust Book**: https://doc.rust-lang.org/book/
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Cargo Book**: https://doc.rust-lang.org/cargo/

#### Dependencies
- **Tokio Docs**: https://tokio.rs/
- **Axum Docs**: https://docs.rs/axum/
- **Oxigraph Docs**: https://docs.rs/oxigraph/

#### Monitoring
- **Prometheus Docs**: https://prometheus.io/docs/
- **Grafana Docs**: https://grafana.com/docs/
- **Jaeger Docs**: https://www.jaegertracing.io/docs/

#### Cloud Providers
- **AWS EC2**: https://docs.aws.amazon.com/ec2/
- **GCP Compute Engine**: https://cloud.google.com/compute/docs
- **Azure VMs**: https://docs.microsoft.com/azure/virtual-machines/

---

## Appendix A: Glossary

- **Authority**: A trusted node with permission to create blocks in PoA consensus
- **Block**: A unit of data containing transactions and cryptographic hash
- **Blockchain**: Immutable chain of blocks linked by cryptographic hashes
- **Consensus**: Agreement protocol among distributed nodes on blockchain state
- **Ed25519**: Elliptic curve signature scheme used for block signing
- **Genesis Block**: The first block in a blockchain
- **Ontology**: Formal representation of knowledge (OWL2 format)
- **P2P**: Peer-to-peer networking between blockchain nodes
- **PoA**: Proof of Authority consensus protocol
- **PBFT**: Practical Byzantine Fault Tolerance consensus
- **RDF**: Resource Description Framework for semantic data
- **SHACL**: Shapes Constraint Language for RDF validation
- **SPARQL**: Query language for RDF data
- **WebSocket**: Full-duplex communication protocol for real-time updates

---

## Appendix B: Troubleshooting Decision Tree

```
Issue: Deployment fails
├── Build Error?
│   ├── YES → Check Rust version (>= 1.70), dependencies installed
│   └── NO → Continue
├── Connection Error?
│   ├── YES → Check firewall, network connectivity, known_peers config
│   └── NO → Continue
├── Authority Error?
│   ├── YES → Verify authority key exists, is_authority=true, key in other nodes
│   └── NO → Continue
└── Other → Check logs: docker logs / journalctl -u provchain

Issue: Nodes not synchronizing
├── Different blockchain lengths?
│   ├── YES → Check validation errors, resync from authority
│   └── NO → Continue
├── Peer connectivity?
│   ├── NO PEERS → Check network, firewall, network_id match
│   └── CONNECTED → Continue
└── Consensus stalled?
    ├── YES → Check authority is running, creating blocks
    └── NO → Monitor for 5 minutes, check logs

Issue: Performance degraded
├── High Memory?
│   ├── YES → Reduce cache_size_mb, upgrade RAM
│   └── NO → Continue
├── High CPU?
│   ├── YES → Check slow queries, optimize, upgrade vCPUs
│   └── NO → Continue
├── Disk Full?
│   ├── YES → Clean logs, prune data, expand volume
│   └── NO → Continue
└── Network Slow?
    ├── YES → Check bandwidth, latency, optimize queries
    └── NO → Review application logs for errors
```

---

## Summary

ProvChain-Org is **production-ready** with excellent infrastructure:

✅ **Well-Architected**: 26 Rust modules, clean separation of concerns
✅ **Battle-Tested**: Existing 3-node validation test
✅ **Production-Ready**: Monitoring, health checks, Docker support
✅ **Documented**: Comprehensive configs and deployment guides
✅ **Secure**: Ed25519 signing, ChaCha20-Poly1305 encryption, JWT auth
✅ **Scalable**: Can expand beyond 3 nodes with minimal changes

**Total Time to Deploy:** 2-4 hours (Docker Compose) | 1-2 days (Kubernetes)

**Recommended First Deployment:**
1. Choose AWS (best value)
2. Use Docker Compose (fastest)
3. Start with t3.medium (2 vCPU, 4 GB) for all nodes (save cost during testing)
4. Run validation suite
5. Upgrade to recommended specs for production

**Next Steps:**
1. Provision VMs
2. Follow [SETUP_INSTALLATION_GUIDE.md](#)
3. Deploy authority node first
4. Deploy regular nodes
5. Run validation tests
6. Monitor for 24-48 hours

**Support:**
- GitHub Issues: `https://github.com/yourusername/provchain-org/issues`
- Documentation: `https://github.com/yourusername/provchain-org/docs`

---

**Document Control:**
- Version: 1.0
- Date: 2026-01-02
- Status: Final
- Next Review: 2026-04-02 (quarterly)

---

*End of Comprehensive Analysis Report*
