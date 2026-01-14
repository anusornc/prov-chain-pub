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
cargo run demo
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

The REST API and load tests require a `JWT_SECRET` for secure token generation. In development, you can set it as follows:

```bash
export JWT_SECRET="dev-secret-32-chars-long-minimum-for-testing"
```

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
