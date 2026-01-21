# Gemini Summary: ProvChainOrg

This document provides a summary of the ProvChainOrg project based on the information found in `README.md`, `Cargo.toml`, and `docs/Plan.md`.

## Project Goal

The goal of this project is to implement a distributed blockchain system for supply chain traceability. It uses RDF (Resource Description Framework) to store data and provides SPARQL for querying. The project is a production-ready implementation of the "GraphChain" concept.

## Technology Stack

*   **Language:** Rust
*   **RDF Store:** Oxigraph
*   **Hashing:** sha2 + hex
*   **Timestamps:** chrono
*   **Networking:** tokio, tokio-tungstenite
*   **Cryptography:** ed25519-dalek
*   **Configuration:** config, toml
*   **Web Framework:** axum

## Building and Running

### Prerequisites
- Rust 1.70+
- Cargo

### Single Node Demo
```bash
# Run the ontology-integrated demo
cargo run demo
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
```

### Web Server
```bash
# Start web server
cargo run -- web-server --port 8080
```

### Testing
```bash
# Run all tests
cargo test
```

## Development Conventions

*   The project follows a phased development approach, with each phase focusing on a specific set of features.
*   The project has a comprehensive test suite, including unit, integration, and end-to-end tests.
*   The project uses a custom RDF canonicalization algorithm for consistent hashing.
*   The project has a well-defined project structure, with separate directories for source code, documentation, tests, and data.
