# ProvChainOrg: Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability

ProvChainOrg is a research-driven distributed blockchain system implemented in Rust. It serves as the primary implementation for the project: **"Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"**. 

The project extends the "GraphChain" concept by integrating semantic technologies directly into the blockchain core, providing high-speed traceability, configurable consensus, cross-chain interoperability, and granular data privacy.

## 🎓 Project Objectives & Contributions

This project satisfies the following research objectives:
1. **RDF-Native Data Structure**: Redesigning the blockchain to store data as RDF triples, enabling machine-readable provenance.
2. **Multi-Consensus Architecture**: A configurable consensus layer supporting selectable protocols (PoA/PBFT).
3. **Data Owner Permission Control**: Granular visibility control using ChaCha20-Poly1305 encryption for private triples.
4. **Cross-Chain Data Interchange**: A secure bridge for transferring traceable assets between independent networks with SHACL validation.
5. **Knowledge Graph Traceability**: Using embedded ontologies and optimized graph algorithms (SSSP-inspired) for microsecond-latency product tracing.

## 🚀 Key Features

- **Embedded Ontology Engine**: Built-in **Oxigraph** triplestore with full **SPARQL** and **SHACL** validation support.
- **Selectable Consensus**: Runtime protocol switching between **Proof-of-Authority (PoA)** and **PBFT (Prototype)** via configuration.
- **Verified Block Signatures**: Consensus nodes sign `block.hash` with **Ed25519** and use validator public-key hex identities.
- **Granular Privacy**: Hybrid on-chain storage supporting both public triples and **ChaCha20-Poly1305 encrypted** private data.
- **Secure Cross-Chain Bridge**: Lock-and-Mint foundation using **Ed25519 digital signatures** and automated **SHACL compliance** checks for ingested data.
- **Optimized Traceability**: Implements **Frontier Reduction** and **Pivot Selection** for high-performance supply chain backtracking.
- **Scientific Benchmarking**: Evaluation suite measuring **Goodput** (Successful TPS) and **Latency**, specifically tuned for semantic overhead.

## 🛠️ Technology Stack

- **Language**: Rust (Memory safety, High concurrency)
- **Semantic Store**: Oxigraph (RDF/SPARQL)
- **Cryptography**: Ed25519 (Signatures), ChaCha20-Poly1305 (Encryption)
- **Web API**: Axum (RESTful Modular Handlers)
- **Networking**: Tokio / WebSockets (P2P Foundation)
- **Ontology**: OWL2 / PROV-O / SHACL

## 📦 Architecture

### Core Modules
- `src/core/`: Blockchain state and block management.
- `src/network/consensus.rs`: Trait-based multi-protocol consensus manager.
- `src/security/encryption.rs`: Privacy engine for data visibility control.
- `src/interop/bridge.rs`: Cross-chain data interchange logic.
- `src/web/handlers/`: Modular REST API handlers (Auth, Transaction, Query).
- `src/semantic/`: OWL2 reasoning and SHACL validation systems.

## 🚦 Quick Start

### Prerequisites
- Rust 1.70+
- Cargo

### Single Node Demo
```bash
# Clone the repository
git clone https://github.com/anusornc/provchain-org.git
cd provchain-org

# Run the supply chain traceability demo
cargo run -- examples basic-supply-chain

# Or try the full GS1 EPCIS UHT demo
cargo run --example gs1_epcis_uht_demo
```

### CLI Usage
The system provides a powerful CLI for interacting with the blockchain:

```bash
# Add RDF file as new block
cargo run -- add-file test_data/simple_supply_chain_test.ttl

# Run SPARQL query
cargo run -- query queries/trace_by_batch_ontology.sparql

# Validate blockchain integrity
cargo run -- validate

# Dump blockchain to stdout
cargo run -- dump

# Start web server
cargo run -- web-server --port 8080
```

## 🎮 Examples & Demos

ProvChain includes multiple examples demonstrating different features and complexity levels:

### Built-in CLI Examples

| Command | Description | Complexity |
|---------|-------------|------------|
| `cargo run -- examples list` | List all available examples | - |
| `cargo run -- examples basic-supply-chain` | Simple supply chain traceability | ⭐⭐ |
| `cargo run -- examples transaction-workflow` | Transaction signing & multi-party | ⭐⭐⭐ |
| `cargo run -- examples owl2-reasoning` | OWL2 features (hasKey, property chains) | ⭐⭐⭐ |
| `cargo run -- examples web-server` | Web UI with demo data | ⭐⭐ |
| `cargo run -- examples gs1-epcis-uht` | GS1 EPCIS reference demo | ⭐⭐⭐ |

### Standalone Examples

```bash
# Full GS1 EPCIS UHT Supply Chain (113 blocks, 2076 triples)
cargo run --example gs1_epcis_uht_demo

# Web UI starter with sample data
cargo run --example demo_ui

# WAL persistence demonstration
cargo run --example persistence_demo
```

### Demo Complexity Overview

```
Simple                              Complex
   │                                   │
   ▼                                   ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ persistence  │  │ basic-supply │  │ transaction  │  │ gs1_epcis    │
│ _demo        │  │ -chain       │  │ -workflow    │  │ _uht_demo    │
│              │  │              │  │              │  │              │
│ • WAL basics │  │ • 4 blocks   │  │ • Signing    │  │ • 113 blocks │
│ • 1 block    │  │ • Simple RDF │  │ • UTXO       │  │ • 2076 triples│
│ • ~2s        │  │ • ~2s        │  │ • Multi-party│  │ • 8 phases   │
│              │  │              │  │ • ~5s        │  │ • ~7s        │
└──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
     10 lines          100 lines         500 lines         776 lines
```

### GS1 EPCIS UHT Demo Phases

The `gs1_epcis_uht_demo` example demonstrates a complete UHT milk supply chain:

1. **Milk Collection** - Farm (Wisconsin Organic Dairy)
2. **Cold Chain Transport** - Farm to Processing Plant
3. **Quality Control** - Reception & Pre-Processing (4 tests)
4. **UHT Processing** - 137°C/4 seconds aseptic processing
5. **Aseptic Packaging** - Tetra Pak cartons
6. **Cold Storage** - 4°C hold
7. **Distribution** - Plant to Distribution Center
8. **Retail Delivery** - Stocking at Metro Supermarkets

Features demonstrated:
- ✅ GS1 EPCIS standard compliance
- ✅ OWL2 hasKey validation (batch uniqueness)
- ✅ Property chain inference
- ✅ Qualified cardinality (4 QC tests)
- ✅ Full SPARQL traceability queries
- ✅ 100-event load test (64ms/event)

### Running Project Benchmarks
To generate performance data for the project evaluation:
```bash
# Ensure JWT_SECRET is set for API tests
export JWT_SECRET=$(openssl rand -base64 32)
cargo test --test load_tests --release -- --ignored
```

## 📊 Performance Benchmarking

The project includes a **portable benchmark toolkit** for comprehensive performance evaluation against traditional systems (Neo4j, Ethereum, Hyperledger Fabric, etc.).

### Quick Start

```bash
cd benchmark-toolkit
./run.sh
```

The toolkit automatically:
- Detects your hardware capabilities
- Configures optimal settings
- Runs comprehensive benchmarks
- Generates comparison reports
- Displays real-time visualizations

### Key Features

- **Auto-detection**: Adapts to 4GB-32GB+ RAM machines
- **One-command execution**: No manual configuration needed
- **Portable**: Copy anywhere, runs on any machine with Docker
- **Comprehensive**: Query performance, write throughput, permission overhead

### Results

- **Grafana Dashboard**: http://localhost:3000 (real-time metrics)
- **Summary Report**: `benchmark-toolkit/results/summary.md`
- **Raw Data**: `benchmark-toolkit/results/benchmark_results.csv`

### Hardware Profiles

| Profile | RAM | Dataset | Time | Best For |
|---------|-----|---------|------|----------|
| **Low** | 4GB | 100 tx | ~5 min | Laptops |
| **Medium** | 8GB | 1,000 tx | ~15 min | Standard ✅ |
| **High** | 16GB | 5,000 tx | ~45 min | Workstations |
| **Ultra** | 32GB+ | 10,000 tx | ~2 hours | Servers |

### For Detailed Documentation

See the [Benchmark Toolkit Guide](docs/benchmarking/) or visit:
- 📘 [Full Documentation](benchmark-toolkit/README.md)
- 🚀 [Quick Reference](benchmark-toolkit/QUICKSTART.md)
- 📦 [Deployment Guide](benchmark-toolkit/DEPLOYMENT_GUIDE.md)
- 🔄 [Portability Guide](benchmark-toolkit/PORTABILITY.md)

## ⚙️ Configuration

The system is highly configurable via `config.toml`:

```toml
[consensus]
consensus_type = "poa" # Options: "poa", "pbft"
is_authority = true
block_interval = 5

[ontology]
path = "ontologies/generic_core.owl"
validate_data = true # Enables SHACL validation for every block
```

## 🔐 API Authentication

The REST API and load tests require a `JWT_SECRET` (environment variable, 32+ chars). In development, you can set it as follows:

```bash
export JWT_SECRET="dev-secret-32-chars-long-minimum-for-testing"
```

### First Admin Bootstrap (One-Time)

No default users are created. For initial setup, configure a bootstrap token and call `/auth/bootstrap` once:

```bash
export PROVCHAIN_BOOTSTRAP_TOKEN="$(openssl rand -base64 32)"

curl -X POST http://localhost:8080/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "AdminPassword123!",
    "bootstrap_token": "'"$PROVCHAIN_BOOTSTRAP_TOKEN"'"
  }'
```

After the first user exists, `/auth/bootstrap` is disabled and returns conflict.

### Admin User Management API

With an admin JWT token, manage users via:

```bash
# List users
curl -X GET http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer <admin-token>"

# List users with filters + pagination
curl -X GET "http://localhost:8080/api/admin/users?page=1&limit=20&role=processor&q=proc" \
  -H "Authorization: Bearer <admin-token>"

# Create user
curl -X POST http://localhost:8080/api/admin/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{
    "username":"processor1",
    "password":"ProcessorPass123!",
    "role":"processor"
  }'

# Delete user
curl -X DELETE http://localhost:8080/api/admin/users/processor1 \
  -H "Authorization: Bearer <admin-token>"

# Rotate user password
curl -X PUT http://localhost:8080/api/admin/users/processor1/password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{"new_password":"ProcessorNewPass456!"}'
```

Admin actions are audit-logged to `./data/admin_audit.log` by default.
Set `PROVCHAIN_AUDIT_LOG_PATH` to override the log file path.

## 🧪 Verification

The project includes a comprehensive verification suite:
- `tests/project_requirements_test.rs`: Validates Consensus and Bridge.
- `tests/privacy_test.rs`: Validates Encryption and Wallet key management.
- `tests/load_tests.rs`: Measures Goodput and Latency under stress.

## 📝 Documentation

- [Project Completion Report](docs/THESIS_COMPLETION_REPORT.md) - Summary of technical fulfillment.
- [Architecture Guide](docs/ARCHITECTURE.md) - Detailed design patterns.
- [User Manual](docs/USER_MANUAL.md) - End-user instructions.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup instructions
- Coding standards and guidelines
- Pull request process
- Good first issues

**⚠️ Urgent**: We're looking for contributors to help reduce bus factor risk. See [Component Ownership](docs/architecture/COMPONENT_OWNERSHIP.md) for details.

## License

This project is licensed under the MIT License.

## Contact

**Anusorn Chaikaew** - Student Code 640551018
*Chiang Mai University, Faculty of Science, Department of Computer Science*
