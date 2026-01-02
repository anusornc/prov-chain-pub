# ProvChain-Org FAQ (Frequently Asked Questions)

**Version:** 1.0 (Corrected for Accuracy)
**Last Updated:** January 2, 2026
**Audience:** End Users, Developers, System Administrators
**Note:** This FAQ contains ONLY currently-implemented features. Speculative or planned features are clearly marked.

---

## Table of Contents

1. [General Questions](#general-questions)
2. [Getting Started](#getting-started)
3. [Installation & Setup](#installation--setup)
4. [Usage & Features](#usage--features)
5. [Blockchain & Consensus](#blockchain--consensus)
6. [Semantic Web Features](#semantic-web-features)
7. [API & Integration](#api--integration)
8. [Troubleshooting](#troubleshooting)
9. [Performance & Scaling](#performance--scaling)
10. [Development & Contributing](#development--contributing)

---

## General Questions

### What is ProvChain-Org?

ProvChain-Org is a **research blockchain platform** combining blockchain technology with semantic web capabilities (OWL2/RDF). It's designed for supply chain traceability and provenance tracking.

**Current Capabilities:**
- Blockchain with cryptographic hashing (SHA-256)
- RDF triple store (Oxigraph) integration
- OWL2 ontology support
- SPARQL query interface
- RESTful API with JWT authentication
- WebSocket support

**Use Cases:**
- Research and experimentation with semantic blockchain
- Supply chain tracking demonstrations
- Educational purposes

---

### Who is ProvChain-Org for?

**Target Users:**
- **Researchers**: Experiment with semantic blockchain technology
- **Developers**: Learn blockchain and semantic web integration
- **Students**: Educational platform for distributed systems

**Note:** This is a research/educational project, not production-ready enterprise software.

---

### What makes ProvChain-Org different?

**Key Features:**

1. **Semantic Integration**: Combines blockchain with OWL2 ontology reasoning
2. **Rust-Based**: Memory-safe, performant implementation
3. **Educational**: Well-documented codebase for learning

**What it is NOT:**
- Not a cryptocurrency or financial blockchain
- Not production-ready for enterprise deployment
- Not

 a complete consensus implementation (limited PoA/PBFT)

---

### Is ProvChain-Org open source?

Yes, this is an open source research project.

---

### What are the system requirements?

**Minimum Requirements (Development/Testing):**
- **CPU**: 2 cores
- **RAM**: 4 GB
- **Storage**: 20 GB SSD
- **OS**: Linux (Ubuntu 20.04+), macOS
- **Dependencies**: Rust 1.70+, Node.js 18+ (optional frontend)

**For 3-Node Cluster Testing:**
- **CPU**: 4 cores per node
- **RAM**: 8 GB per node
- **Storage**: 50 GB SSD per node

---

## Getting Started

### How do I quickly test ProvChain-Org?

**Quick Start (Docker Compose - 3 nodes):**

```bash
# 1. Clone the repository
git clone https://github.com/your-org/provchain-org.git
cd provchain-org

# 2. Start 3-node cluster
cd deploy
docker-compose -f docker-compose.3node.yml up -d

# 3. Check logs
docker logs -f provchain-node1
```

**See Also:** `docs/deployment/SETUP_INSTALLATION_GUIDE.md`

---

### What are the main components?

**Core Components:**

1. **Blockchain Core** (`src/core/blockchain.rs`): Immutable ledger with SHA-256
2. **Network Layer** (`src/network/`): Peer-to-peer communication (partially implemented)
3. **Consensus** (`src/network/consensus.rs`): PoA and PBFT (basic implementation)
4. **Semantic Layer** (`src/semantic/`): OWL2 reasoner + RDF store (Oxigraph)
5. **Web API** (`src/web/`): Axum-based HTTP server with authentication

**Note:** Network consensus is partially implemented - suitable for testing, not production.

---

### Can I run a single node for development?

Yes! Single-node mode works well for development:

```bash
# Build and run
cargo build --release
./target/release/provchain-org --help

# See available commands:
# - demo: Run built-in demo
# - transaction-demo: Transaction demos
# - web-server: Start API server
# - owl2-demo: Semantic features demo
# - validate: Check blockchain integrity
```

---

## Installation & Setup

### How do I install from source?

**Prerequisites:**
- Rust 1.70+ (install via `rustup`)
- Git

**Build Steps:**

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone and build
git clone https://github.com/your-org/provchain-org.git
cd provchain-org
cargo build --release

# 3. Run
./target/release/provchain-org --help
```

**Build time**: ~10-15 minutes

---

### How do I configure a node?

Edit `config.toml` (example provided in repository):

```toml
[network]
network_id = "provchain-org-default"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = []
max_peers = 50

[consensus]
is_authority = false
authority_keys = []
block_interval = 10
max_block_size = 1048576

[storage]
data_dir = "./data"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

[logging]
level = "info"
format = "pretty"

[web]
host = "0.0.0.0"
port = 8080
```

**Required:** Set `JWT_SECRET` environment variable for authentication.

---

### How do I set up a 3-node cluster?

**Docker Compose Method (Recommended for Testing):**

```bash
cd deploy
docker-compose -f docker-compose.3node.yml up -d
```

This starts 3 nodes with automatic networking configuration.

**Manual Setup:** See `docs/deployment/SETUP_INSTALLATION_GUIDE.md`

---

### What ports need to be open?

**Currently Used Ports:**

| Port | Purpose |
|------|---------|
| 8080 | P2P network (partially implemented) |
| 8080 | Web API (configurable via config.toml) |

**Note:** Monitoring stack (Prometheus/Grafana/Jaeger) has configuration files but may require additional setup.

---

## Usage & Features

### What commands are available?

**CLI Commands** (run `./provchain-org --help`):

```bash
# Data management
add-file <path>         # Add Turtle RDF file as new block
query <path>            # Run SPARQL query file
validate                # Validate blockchain integrity
dump                    # Export blockchain as JSON

# Demos
demo                    # UHT manufacturing demo
transaction-demo        # Transaction blockchain demos
owl2-demo              # OWL2 semantic features demo

# Server
web-server             # Start REST API server (port 8080 default)
start-node             # Start full node with networking (experimental)

# Utilities
generate-key -o <file> # Generate Ed25519 keypair
enhanced-trace <batch> # Trace batch through supply chain
advanced-owl2          # Advanced OWL2 reasoning
trace-path --from <uri> --to <uri>  # Find path in knowledge graph
```

---

### What API endpoints are available?

**Implemented Endpoints** (requires JWT token for protected routes):

**Public Endpoints:**
```bash
GET  /health                    # Health check
POST /auth/login                # Get JWT token
GET  /api/trace                 # Trace path in knowledge graph
GET  /api/knowledge-graph       # Get knowledge graph data
```

**Protected Endpoints** (require `Authorization: Bearer <token>`):
```bash
# Blockchain
GET  /api/blockchain/status
GET  /api/blockchain/blocks
GET  /api/blockchain/blocks/:index
GET  /api/blockchain/blocks/:index/rdf-summary
GET  /api/blockchain/validate

# Transactions
GET  /api/transactions/recent
POST /api/transactions/create
POST /api/transactions/sign
POST /api/transactions/submit

# SPARQL
POST /api/sparql/query
GET  /api/sparql/config
POST /api/sparql/validate
GET  /api/sparql/queries         # Saved queries
POST /api/sparql/queries         # Save new query
DELETE /api/sparql/queries/:id

# Products/Traceability
GET  /api/products
GET  /api/products/:id
GET  /api/products/:id/trace
GET  /api/products/:id/provenance
GET  /api/products/:id/analytics
GET  /api/products/by-type/:type
GET  /api/products/by-participant/:participantId
GET  /api/products/:id/related
GET  /api/products/:id/validate
POST /api/participants

# Analytics
GET  /api/analytics

# RDF Data
POST /api/blockchain/add-triple

# Wallet
POST /api/wallet/register

# WebSocket
GET  /ws                        # WebSocket connection
```

**NOT Implemented** (do not exist):
- `/api/export` - use `dump` command instead
- `/api/import` - use `add-file` command instead
- `/api/authorities` - authority management is static
- `/api/ontology` - use command-line tools
- `/api/sync-status` - not implemented

---

### How do I authenticate with the API?

**1. Set JWT Secret** (required):
```bash
export JWT_SECRET="your-secure-random-secret-key-here"
```

**2. Start server:**
```bash
./provchain-org web-server --port 8080
```

**3. Login** (authentication is basic - suitable for development only):
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "changeme"}'
```

**4. Use token:**
```bash
curl http://localhost:8080/api/blockchain/status \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

**Security Warning:** Current authentication is basic and suitable only for development/testing, not production use.

---

### How do I query blockchain data?

**Via API:**
```bash
# Get blockchain status
curl http://localhost:8080/api/blockchain/status \
  -H "Authorization: Bearer $TOKEN"

# Get all blocks
curl http://localhost:8080/api/blockchain/blocks \
  -H "Authorization: Bearer $TOKEN"

# Get specific block
curl http://localhost:8080/api/blockchain/blocks/0 \
  -H "Authorization: Bearer $TOKEN"
```

**Via CLI:**
```bash
# Dump entire chain
./provchain-org dump

# Validate integrity
./provchain-org validate
```

---

### What semantic features are available?

**Implemented OWL2 Features:**

1. **RDF Triple Store**: Oxigraph-based storage
2. **SPARQL Queries**: Full SPARQL 1.1 support via Oxigraph
3. **Ontology Loading**: Load OWL2 ontologies from files
4. **Basic Reasoning**: Using owl2-reasoner library (limited capabilities)
5. **SHACL Validation**: Data shape validation (partially implemented)

**Example SPARQL Query:**
```bash
# Via API
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/sparql-query" \
  -d 'PREFIX : <http://provchain.org/ontology#>
      SELECT ?product ?location WHERE {
        ?product :locatedIn ?location .
      } LIMIT 10'
```

**Via CLI:**
```bash
# Save query to file.sparql, then:
./provchain-org query file.sparql
```

---

### Can I trace supply chain data?

**Yes! Several tracing commands:**

```bash
# Enhanced trace (uses OWL2 reasoning)
./provchain-org enhanced-trace "BATCH-001"

# Trace path between entities
./provchain-org trace-path \
  --from "http://provchain.org/data#Product123" \
  --to "http://provchain.org/data#Warehouse-A"

# Via API
curl "http://localhost:8080/api/trace?from=Product123&to=Warehouse-A"
```

---

## Blockchain & Consensus

### What consensus mechanism is used?

**Current Implementation:**

1. **Proof-of-Authority (PoA)**: Basic implementation
   - Authority nodes can propose blocks
   - Ed25519 signature validation
   - Round-robin rotation (partially implemented)

2. **PBFT (Practical Byzantine Fault Tolerance)**: Basic implementation
   - Code exists in `src/network/consensus.rs`
   - **Status:** Experimental, not fully tested

**Limitations:**
- Network layer is partially implemented
- Consensus is suitable for testing, not production
- Byzantine fault tolerance not fully validated
- Limited to small test clusters (3-5 nodes)

---

### How are blocks created?

**Block Structure:**
```rust
pub struct Block {
    index: u64,
    timestamp: i64,
    data: String,              // RDF turtle format
    encrypted_data: Option<Vec<u8>>,
    previous_hash: String,
    hash: String,
    state_root: Option<String>,
    validator: Option<String>,
    signature: Option<Vec<u8>>,
}
```

**Creation Process:**
1. Collect RDF triple data
2. Calculate hash (SHA-256)
3. Sign with authority key (Ed25519)
4. Add to chain
5. Persist to storage

**Via CLI:**
```bash
# Add data as new block
./provchain-org add-file data.ttl
```

---

### What happens if a node fails?

**Current Behavior:**

- **Single Node**: Data persists in `./data` directory, can restart
- **Multi-Node** (experimental): Network recovery not fully implemented

**Recommendation:** Use single-node for development, 3-node Docker Compose for testing only.

---

### Is the blockchain immutable?

**Yes** - blocks use cryptographic hash chains:
- Each block contains previous block's hash
- SHA-256 hashing
- Tampering detected by `validate` command

**Validation:**
```bash
./provchain-org validate
```

---

## API & Integration

### Is there an SDK or client library?

**No** - Currently only REST API available.

**Integration Methods:**
1. **REST API**: Use any HTTP client
2. **WebSocket**: Available at `/ws` endpoint
3. **CLI**: Command-line interface for scripting

**Example (Python):**
```python
import requests

# Authenticate
response = requests.post('http://localhost:8080/auth/login',
    json={'username': 'admin', 'password': 'changeme'})
token = response.json()['token']

# Query blockchain
headers = {'Authorization': f'Bearer {token}'}
blocks = requests.get('http://localhost:8080/api/blockchain/blocks',
    headers=headers).json()

print(f"Blockchain has {len(blocks)} blocks")
```

---

### Can I integrate with external systems?

**Yes**, via REST API. Integration options:

1. **REST API**: Standard HTTP/JSON
2. **RDF Export**: Use `dump` command to export as RDF
3. **SPARQL Queries**: Query semantic data programmatically
4. **WebSocket**: Real-time connection at `/ws`

**Limitations:**
- No message queue integration
- No webhooks
- No official SDKs

---

## Troubleshooting

### Node won't start - what should I check?

**Common Issues:**

1. **Port Already in Use:**
   ```bash
   # Check if port 8080 is used
   lsof -i :8080
   # Or use different port
   ./provchain-org web-server --port 9000
   ```

2. **JWT_SECRET Not Set:**
   ```bash
   export JWT_SECRET="your-secret-key"
   ```

3. **Data Directory Permissions:**
   ```bash
   chmod -R 755 ./data
   ```

4. **Check Logs:**
   ```bash
   # Set debug logging
   RUST_LOG=debug ./provchain-org web-server
   ```

---

### How do I view logs?

**Logging Configuration** (in `config.toml`):
```toml
[logging]
level = "info"  # Options: error, warn, info, debug, trace
format = "pretty"  # or "json"
```

**Runtime Logging:**
```bash
# Set log level via environment
RUST_LOG=debug ./provchain-org web-server

# Docker logs
docker logs -f provchain-node1
```

---

### How do I reset/clear the blockchain?

```bash
# Stop any running nodes
# Then delete data directory
rm -rf ./data

# Restart creates fresh blockchain
./provchain-org web-server
```

---

## Performance & Scaling

### How many transactions per second can it handle?

**Performance is NOT benchmarked for production use.**

**Ballpark estimates** (single node, no network latency):
- Simple transactions: ~50-100/sec
- With SPARQL queries: ~10-50/sec
- With OWL2 reasoning: ~1-10/sec

**Note:** These are rough estimates, not verified benchmarks. Performance varies greatly based on:
- Data complexity
- Query complexity
- Hardware specifications
- Reasoning requirements

**Benchmarks exist** in `benches/` directory but measure specific operations, not end-to-end TPS.

---

### How much storage is required?

**Storage Growth** (approximate):

- **Empty Installation**: ~200 MB (compiled binary + dependencies)
- **Per Block**: Varies by data size (typically 1-100 KB)
- **RDF Store**: Grows with triple count
- **Indexes**: Oxigraph creates indexes (~2-3x data size)

**Example:**
- 1,000 blocks with small data: ~10-50 MB
- 10,000 blocks: ~100-500 MB
- Heavy semantic data: Can grow significantly

**Check Storage:**
```bash
du -sh ./data
```

---

### Can I run multiple nodes on one machine?

**Yes, for testing:**

```bash
# Use different ports and data directories
./provchain-org web-server --port 8080 &  # Node 1
./provchain-org web-server --port 8081 &  # Node 2
./provchain-org web-server --port 8082 &  # Node 3
```

**Or use Docker Compose:**
```bash
cd deploy
docker-compose -f docker-compose.3node.yml up
```

---

## Development & Contributing

### How do I contribute?

**Contribution Process:**

1. Fork repository on GitHub
2. Create feature branch
3. Make changes following Rust conventions
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit pull request

---

### What are the coding standards?

**Rust Standards:**

1. **Formatting**: `cargo fmt` (rustfmt)
2. **Linting**: `cargo clippy` - must pass with no warnings
3. **Testing**: Add tests for new features
4. **Documentation**: Add `///` doc comments for public APIs
5. **Error Handling**: Use `Result<T, E>`, avoid `.unwrap()` in production code

**Example:**
```rust
/// Validates a block before adding to chain.
///
/// # Arguments
/// * `block` - The block to validate
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(String)` if invalid with error message
pub fn validate_block(block: &Block) -> Result<(), String> {
    // Implementation
}
```

---

### How do I run tests?

```bash
# All tests
cargo test

# Specific test
cargo test test_blockchain_validation

# With output
cargo test -- --nocapture

# Benchmarks
cargo bench
```

---

### Where is the documentation?

**Current Documentation:**

- **README.md**: Project overview
- **docs/deployment/SETUP_INSTALLATION_GUIDE.md**: Setup guide
- **docs/deployment/COMPREHENSIVE_ANALYSIS_REPORT.md**: Architecture analysis
- **docs/FAQ.md**: This file
- **Code Comments**: Inline documentation in source files

**Generate Rust Docs:**
```bash
cargo doc --open
```

---

## Additional Resources

### Related Projects

- **Hyperledger Fabric**: Enterprise blockchain framework
- **Apache Jena**: Semantic web framework (RDF/SPARQL)
- **Oxigraph**: RDF triple store (used by this project)
- **Protégé**: Ontology editor for OWL2

---

## Glossary

**Blockchain Terms:**
- **Block**: Batch of data with cryptographic hash
- **Hash**: SHA-256 cryptographic fingerprint
- **Consensus**: Agreement mechanism (PoA/PBFT in this project)
- **Immutability**: Cannot modify past blocks

**Semantic Web Terms:**
- **RDF (Resource Description Framework)**: Triple-based data model (Subject-Predicate-Object)
- **SPARQL**: Query language for RDF data
- **OWL2 (Web Ontology Language)**: Ontology definition language
- **Triple**: Subject-Predicate-Object data unit
- **Ontology**: Formal knowledge representation

**Consensus Terms:**
- **PoA (Proof-of-Authority)**: Consensus with pre-defined validators
- **PBFT (Practical Byzantine Fault Tolerance)**: Byzantine consensus algorithm
- **Authority**: Node authorized to create blocks
- **Ed25519**: Digital signature algorithm used

---

## Important Disclaimers

1. **Research Project**: This is a research/educational blockchain, not production software
2. **Limited Testing**: Consensus and networking are partially implemented and minimally tested
3. **No Security Audit**: Has not undergone professional security audit
4. **No Warranty**: Use at your own risk
5. **Active Development**: APIs and features may change

---

## FAQ Version History

- **v1.0** (2026-01-02): Initial corrected release with only factual capabilities

---

**Last Updated**: January 2, 2026
**Note**: This FAQ has been carefully reviewed to include only currently-implemented features. Speculative or planned features have been removed or clearly marked.

For questions, open an issue on GitHub.
