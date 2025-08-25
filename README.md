# ProvChainOrg

A distributed blockchain system implementing the GraphChain concept for supply chain traceability using RDF graphs and SPARQL queries. This project demonstrates semantic blockchain technology with provenance tracking capabilities and ontology integration.

## Overview

ProvChainOrg is a production-ready implementation of the GraphChain concept from the research paper "GraphChain – A Distributed Database with Explicit Semantics and Chained RDF Graphs" by Sopek et al. (2018). It combines blockchain security with semantic web technologies to create a distributed ledger for supply chain traceability with formal ontology support.

## Key Features

- **RDF-Native Blockchain**: Blocks reference RDF graphs directly with cryptographic integrity
- **Semantic Data Access**: Full SPARQL query support across all blockchain data
- **Ontology Integration**: Automatic loading and validation using traceability ontology
- **Flexible Ontology Management**: Runtime ontology configuration and switching
- **RDF Canonicalization**: Advanced canonicalization algorithm for semantic equivalence
- **Distributed P2P Network**: WebSocket-based peer communication and discovery (foundation)
- **Supply Chain Traceability**: Track products from origin to consumer with environmental monitoring
- **Modern Rust Implementation**: High performance with memory safety and comprehensive testing

## Architecture

### Core Components
- **Blockchain Engine**: Hash-linked blocks with RDF graph references
- **RDF Store**: Oxigraph triplestore with named graphs and SPARQL endpoint
- **Ontology System**: Flexible loading and validation of traceability ontology
- **Ontology Manager**: Runtime configuration and management of ontologies
- **Canonicalization**: Deterministic RDF graph hashing with blank node handling
- **Network Layer**: P2P messaging protocol and peer discovery (foundation)
- **Configuration**: Comprehensive node configuration management

### Data Model
```rust
pub struct Block {
    pub index: u64,           // Sequential block number
    pub timestamp: String,    // ISO 8601 timestamp
    pub data: String,         // RDF data in Turtle format
    pub previous_hash: String, // Link to previous block
    pub hash: String,         // This block's cryptographic hash
}
```

### RDF Graph Organization
- **Block Graphs**: `http://provchain.org/block/{index}` - Contains block's RDF data
- **Ontology Graph**: `http://provchain.org/ontology` - Traceability ontology
- **Metadata**: Block metadata stored as RDF triples

## Quick Start

### Prerequisites
- Rust 1.70+ 
- Cargo

### Single Node Demo
```bash
# Clone the repository
git clone https://github.com/anusornc/provchain-org.git
cd provchain-org

# Run the ontology-integrated demo
cargo run demo

# Run comprehensive tests
cargo test
```

### Custom Ontology Configuration
```bash
# Create custom ontology configuration
cp config/ontology.toml.example config/ontology.toml

# Edit the configuration to use different ontologies
nano config/ontology.toml

# Run with custom configuration
cargo run -- --config config/ontology.toml demo
```

### CLI Usage
```bash
# Add RDF file as new block
cargo run -- add-file test_data/simple_supply_chain_test.ttl

# Run SPARQL query
cargo run -- query queries/trace_by_batch_ontology.sparql

# Validate blockchain integrity
cargo run -- validate

# Dump blockchain as JSON
cargo run -- dump

# Run with custom ontology configuration
cargo run -- --ontology-config config/my_ontology.toml demo

# List loaded ontologies
cargo run -- list-ontologies
```

### Distributed Network (Foundation)
```bash
# Node 1 (Authority)
PROVCHAIN_PORT=8080 PROVCHAIN_AUTHORITY=true cargo run

# Node 2 (Regular)
PROVCHAIN_PORT=8081 PROVCHAIN_PEERS="127.0.0.1:8080" cargo run

# Node 3 (Regular)  
PROVCHAIN_PORT=8082 PROVCHAIN_PEERS="127.0.0.1:8080" cargo run
```

## Configuration

Create a `config.toml` file:

```toml
[network]
network_id = "provchain-org-default"
listen_port = 8080
known_peers = ["127.0.0.1:8081"]

[consensus]
is_authority = false

[storage]
data_dir = "./data"
persistent = true
store_type = "oxigraph"

[ontology]
path = "ontologies/generic_core.owl"
graph_name = "http://provchain.org/ontology"
auto_load = true
validate_data = false
```

## Ontology Management System

### Flexible Ontology Loading
The system now supports flexible ontology loading through a configuration-based approach:

```toml
# config/ontology.toml
[main_ontology]
path = "ontologies/generic_core.owl"
graph_iri = "http://provchain.org/ontology/core"
auto_load = true
validate_data = false

[resolution]
strategy = "FileSystem"

[namespaces]
core = "http://provchain.org/core#"
prov = "http://www.w3.org/ns/prov#"
xsd = "http://www.w3.org/2001/XMLSchema#"
rdfs = "http://www.w3.org/2000/01/rdf-schema#"
owl = "http://www.w3.org/2002/07/owl#"

[domain_ontologies.supply_chain]
path = "ontologies/supply-chain.owl"
graph_iri = "http://provchain.org/ontology/supply-chain"
enabled = true
priority = 100
```

### Runtime Ontology Switching
- Change ontologies without recompilation
- Support for domain-specific ontologies
- Multiple loading strategies (File System, HTTP, Embedded, Auto-detection)
- Dynamic namespace management

### Configuration Methods
1. **TOML Configuration Files** - `config/ontology.toml`
2. **Environment Variables** - `ONTOLGY_MAIN_PATH`, `ONTOLGY_AUTO_LOAD`
3. **Programmatic Configuration** - API-based setup
4. **Command-Line Arguments** - Future enhancement

### Domain-Specific Ontologies
- Supply Chain (`ontologies/supply-chain.owl`)
- Healthcare (`ontologies/healthcare.owl`)
- Pharmaceutical (`ontologies/pharmaceutical.owl`)
- Automotive (`ontologies/automotive.owl`)
- Digital Assets (`ontologies/digital_assets.owl`)

## Use Case: Supply Chain Traceability

### Complete Traceability Chain
1. **Farm Origin**: Raw milk batch with farmer attribution and batch ID
2. **Processing**: UHT processing activity with ingredient traceability
3. **Transport**: Cold chain logistics with environmental monitoring
4. **Retail**: Final destination with complete provenance chain

### Example RDF Data (Ontology-Based)
```turtle
@prefix core: <http://provchain.org/core#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:milkBatch1 a core:Batch ;
    core:hasIdentifier "MB001" ;
    core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
    prov:wasAttributedTo ex:FarmerJohn .

ex:FarmerJohn a core:Supplier ;
    rdfs:label "John's Dairy Farm" .

ex:transport1 a core:TransportProcess ;
    core:recordedAt "2025-08-08T14:00:00Z"^^xsd:dateTime ;
    prov:used ex:milkBatch1 ;
    core:hasCondition ex:condition1 .

ex:condition1 a core:EnvironmentalCondition ;
    core:hasTemperature "4.2"^^xsd:decimal ;
    core:hasHumidity "65.0"^^xsd:decimal .
```

### SPARQL Queries
```sparql
# Ontology-aware batch tracing
SELECT ?batch ?activity ?agent ?timestamp WHERE {
    ?batch a core:Batch ;
           core:hasIdentifier "MB001" .
    ?activity prov:used ?batch ;
              prov:wasAssociatedWith ?agent ;
              core:recordedAt ?timestamp .
} ORDER BY ?timestamp

# Environmental conditions monitoring
SELECT ?batch ?temp ?humidity WHERE {
    ?activity prov:used ?batch ;
              core:hasCondition ?condition .
    ?condition core:hasTemperature ?temp ;
               core:hasHumidity ?humidity .
    FILTER(?temp > 5.0)
}
```

## Detailed Codebase Analysis

### 1. Core Architecture

The project follows a modular architecture with several key components:

#### Core Module (`src/core/`)
- `blockchain.rs`: Implements the fundamental blockchain structure with blocks that store RDF data
- `entity.rs`: Defines a generic `TraceableEntity` model that can represent various types of entities across different domains
- `atomic_operations.rs`: Handles atomic operations for consistency

#### Storage Module (`src/storage/`)
- `rdf_store.rs`: The central component that wraps Oxigraph for RDF storage, SPARQL querying, and canonicalization

#### Transaction Module (`src/transaction/`)
- `transaction.rs`: Implements a full transaction system with inputs/outputs, signing, and validation
- `blockchain.rs`: Extends the core blockchain with transaction processing capabilities

#### Web Module (`src/web/`)
- Provides a REST API using Axum with endpoints for blockchain interaction, SPARQL queries, and traceability

#### Semantic Module (`src/semantic/`)
- Contains OWL reasoners and SHACL validators for semantic processing
- Integrates with the `owl2_rs` library for advanced OWL2 features

### 2. Key Features and Capabilities

#### Blockchain with RDF Integration
- Each block contains RDF data stored in named graphs
- Uses RDF canonicalization (RDFC-1.0) for consistent hashing
- Implements a transaction system with digital signatures (Ed25519)

#### Advanced Semantic Features
- Integration with a custom OWL2 library (`owl2_rs`) for ontology processing
- Support for `owl:hasKey` axioms for entity uniqueness constraints
- Property chain inference for transitive relationship discovery
- SHACL validation for data conformance

#### Multi-Domain Support
- Generic entity model that can be adapted to different domains
- Domain-specific adapters for supply chain, healthcare, and pharmaceutical industries
- Plugin interface for extending functionality

#### Trace Optimization
- Implements frontier reduction and pivot selection concepts from SSSP algorithms
- Optimized traceability queries for supply chain data

#### Wallet and Identity Management
- Multi-participant wallet system with role-based permissions
- Secure key storage and management
- Certificate management for participants

### 3. Technical Implementation Details

#### RDF Handling
- Uses Oxigraph as the RDF store with support for named graphs
- Implements both in-memory and persistent storage
- Supports RDF canonicalization for deterministic hashing
- Provides SPARQL query capabilities

#### Consensus and Validation
- Transaction validation with business logic checks
- Multi-signature support for critical operations
- Signature verification using Ed25519

#### Web API
- REST API built with Axum
- JWT-based authentication
- Endpoints for blockchain status, SPARQL queries, and traceability
- Input validation and sanitization

#### Performance Optimizations
- Frontier reduction in traceability queries
- Memory caching for RDF data
- Adaptive canonicalization algorithm selection

### 4. Domain-Specific Functionality

#### Supply Chain
- Models for products, batches, processes, and transport activities
- Environmental monitoring data integration
- Quality control and compliance tracking

#### Healthcare/Pharmaceutical
- Domain adapters for healthcare-specific entities and relationships
- Regulatory compliance features

### 5. Advanced Features

#### OWL2 Reasoning
- Enhanced reasoner with support for `owl:hasKey`, property chains, and qualified cardinality restrictions
- Integration with the custom `owl2_rs` library

#### Knowledge Graph
- Modules for knowledge graph construction and entity linking
- Analytics capabilities for supply chain insights

#### Production Features
- Monitoring and metrics collection
- Compliance checking
- Containerization support

## Project Structure

```
src/
├── blockchain.rs      # Core blockchain with ontology integration
├── rdf_store.rs      # RDF store with canonicalization & validation
├── demo.rs           # Ontology-integrated demo application
├── config.rs         # Comprehensive configuration management
├── network/          # Distributed networking foundation
│   ├── mod.rs        # Network manager
│   ├── messages.rs   # P2P message protocol
│   ├── peer.rs       # Peer connection management
│   └── discovery.rs  # Peer discovery protocol
├── ontology/         # Flexible ontology management system
│   ├── mod.rs        # Ontology module exports
│   ├── config.rs     # Ontology configuration management
│   ├── manager.rs    # Ontology loading and management
│   ├── loader.rs     # Configuration file loading
│   └── processor.rs  # Ontology processing and validation
└── lib.rs           # Library exports

ontologies/          # Flexible ontology management system
├── generic_core.owl      # Generic core ontology (primary)
├── supply-chain.owl      # Supply chain domain ontology
├── healthcare.owl        # Healthcare domain ontology
├── pharmaceutical.owl    # Pharmaceutical domain ontology
├── automotive.owl        # Automotive domain ontology
├── digital_assets.owl    # Digital assets domain ontology
├── advanced_owl2_reasoning.owl # OWL2 advanced features ontology
└── test-owl2.owl        # Test OWL2 features ontology

config/              # Configuration files
├── config.toml           # Main configuration
├── ontology.toml         # Ontology configuration
└── network.toml          # Network configuration

test_data/           # Sample RDF data files
├── minimal_test_data.ttl
├── simple_supply_chain_test.ttl
└── complete_supply_chain_test.ttl

queries/             # SPARQL query examples
├── trace_by_batch_ontology.sparql  # Ontology-aware queries
├── trace_by_batch.sparql
├── trace_origin.sparql
├── env_conditions_for_batch.sparql
└── blockchain_metadata.sparql

tests/               # Comprehensive test suite
├── blockchain_tests.rs
├── rdf_tests.rs
├── canonicalization_tests.rs
├── ontology_integration_tests.rs
├── blockchain_with_test_data.rs
├── test_data_validation.rs
├── demo_tests.rs
├── ontology_management_tests.rs
├── enhanced_performance_benchmarks.rs
└── domain_tests.rs
```

## Technology Stack

- **Rust**: Systems programming language for performance and safety
- **Oxigraph**: RDF triplestore for semantic data storage
- **Tokio**: Async runtime for networking (foundation)
- **WebSockets**: P2P communication protocol (foundation)
- **Serde**: Serialization for messages and configuration
- **SHA-256**: Cryptographic hashing with RDF canonicalization
- **Ed25519**: Digital signatures (prepared for consensus)
- **PROV-O**: W3C provenance ontology foundation
- **Turtle**: RDF serialization format

## Testing

### Comprehensive Test Suite
```bash
# Run all tests (27 tests across 8 suites)
cargo test

# Specific test categories
cargo test --test blockchain_tests           # Core blockchain (4 tests)
cargo test --test rdf_tests                 # RDF operations (2 tests)
cargo test --test canonicalization_tests    # RDF canonicalization (3 tests)
cargo test --test ontology_integration_tests # Ontology features (5 tests)
cargo test --test blockchain_with_test_data # Integration tests (3 tests)
cargo test --test test_data_validation      # Data validation (3 tests)

# Unit tests (12 tests)
cargo test --lib
```

### Test Coverage
- **Core Blockchain**: Block creation, validation, tampering detection
- **RDF Operations**: Graph storage, SPARQL queries, metadata
- **Canonicalization**: Blank node handling, semantic equivalence
- **Ontology Integration**: Loading, validation, environmental conditions
- **Ontology Management**: Flexible loading, configuration, domain ontologies
- **Network Foundation**: P2P messages, peer discovery, configuration
- **Data Validation**: Test data integrity, supply chain scenarios
- **Performance**: Enhanced traceability, OWL2 reasoning, property chains

## Key Innovations

### 1. RDF Canonicalization Algorithm
- Deterministic hashing of RDF graphs
- Blank node canonicalization with Magic_S/Magic_O placeholders
- Semantic equivalence detection
- Blockchain integrity with varying RDF representations

### 2. Ontology-Driven Validation
- Automatic ontology loading on blockchain initialization
- Flexible ontology configuration and loading
- Runtime ontology switching without recompilation
- Class-based validation for supply chain entities
- Required property enforcement
- Environmental condition integration
- Domain-specific ontology support

### 3. Semantic Blockchain Architecture
- Named graphs for block organization
- SPARQL queries across entire blockchain
- Metadata storage as RDF triples
- Provenance tracking with PROV-O compliance

## Documentation

- [Implementation Summary](IMPLEMENTATION_SUMMARY.md) - Complete technical overview
- [Testing Summary](TESTING_SUMMARY.md) - Comprehensive testing analysis
- [Ontology Integration](ONTOLOGY_INTEGRATION_COMPLETE.md) - Ontology features
- [GraphChain Comparison](GRAPHCHAIN_COMPARISON.md) - Research paper comparison
- [Run Instructions](Run.md) - Detailed usage instructions

## Research Background

This implementation is based on the GraphChain concept from:

> Sopek, M., Grądzki, P., Kosowski, W., Kuziński, D., Trójczak, R., & Trypuz, R. (2018). GraphChain – A Distributed Database with Explicit Semantics and Chained RDF Graphs. In Proceedings of The 2018 Web Conference Companion (WWW'18 Companion).

## Current Status

### ✅ Implemented Features
- Core blockchain with RDF graphs
- RDF canonicalization algorithm
- Ontology integration and validation
- Flexible ontology management system
- Comprehensive test suite (27 tests passing)
- SPARQL query capabilities
- Supply chain traceability demo
- Configuration management
- P2P networking foundation

### 🚧 In Development
- Full P2P network implementation
- Consensus mechanism (Proof-of-Authority)
- Advanced ontology reasoning
- Cross-node SPARQL queries

### 📋 Future Enhancements
- Multiple ontology support
- Geographic origin tracking
- Performance optimization
- Production deployment tools

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Contact

For questions or collaboration opportunities, please open an issue on GitHub.