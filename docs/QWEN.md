# Qwen Code Context

This document provides context for Qwen Code about the 'UHT Traceability Blockchain (PoC)' project.

## Project Summary

This is a comprehensive Rust implementation for a blockchain where each block stores RDF triples in a named graph using an in-memory Oxigraph store. The ontology is based on PROV-O, extended for traceability, with a focus on UHT manufacturing but designed to be applicable to other supply chains.

The system is designed as a **permissioned blockchain for traceability** that supports multiple domains through **domain-flexible ontology integration**. Users configure which ontology to use at deployment time, and the system operates as a dedicated traceability infrastructure for that domain. Unlike systems that require runtime ontology switching, ProvChainOrg uses deployment-time configuration to ensure data consistency and system reliability while providing flexibility across different traceability domains.

### Key Features

*   **RDF Storage:** Utilizes Oxigraph as an RDF dataset with support for both in-memory and persistent storage using RocksDB.
*   **Named Graphs:** Each blockchain block's data is stored in a separate named graph, identified by a URI like `http://provchain.org/block/{index}`.
*   **RDF Metadata:** Blockchain metadata itself is stored as RDF within a dedicated graph (`http://provchain.org/blockchain`).
*   **Advanced Hashing:** Employs SHA-256 for block hashing with RDF canonicalization (RDFC-1.0) for consistent hashing.
*   **Persistent Storage:** Supports persistent storage with RocksDB backend, including backup/restore functionality.
*   **Web API:** Includes a complete REST API web server built with Axum for blockchain interaction.
*   **Transaction System:** Implements a full transaction blockchain with signing capabilities.
*   **Governance System:** Includes governance mechanisms for blockchain management.
*   **Wallet System:** Provides wallet functionality for key management and signing.
*   **Analytics & Knowledge Graph:** Contains modules for analytics and knowledge graph processing.
*   **Network Layer:** Implements P2P networking for distributed operation.
*   **Performance Optimizations:** Includes caching, compression, and other performance enhancements.

## Technology Stack

*   **Language:** Rust
*   **RDF/SPARQL Engine:** Oxigraph
*   **Hashing:** sha2 + hex
*   **Timestamps:** chrono
*   **CLI Framework:** clap
*   **Web Framework:** axum
*   **Serialization:** serde
*   **Error Handling:** anyhow
*   **Networking:** tokio, tokio-tungstenite
*   **Cryptography:** ed25519-dalek, rand
*   **Storage:** RocksDB (via Oxigraph)
*   **Compression:** lz4
*   **Configuration:** config, toml

## Project Architecture

The project is organized into multiple modules:

*   **core**: Core blockchain implementation (Block, Blockchain)
*   **storage**: RDF storage layer with persistence support
*   **transaction**: Transaction blockchain implementation
*   **web**: REST API web server
*   **network**: P2P networking layer
*   **semantic**: Semantic web and ontology handling
*   **analytics**: Analytics and reporting
*   **knowledge_graph**: Knowledge graph processing
*   **governance**: Governance mechanisms
*   **wallet**: Wallet and key management
*   **demo**: Demonstration functionality
*   **performance**: Performance optimizations
*   **production**: Production deployment features
*   **universal_demo**: Universal demonstration capabilities

## Implementation Features

### Core Blockchain
1.  Block and Blockchain data structures with RDF integration
2.  RDF canonicalization (RDFC-1.0) for consistent hashing
3.  Named graph storage for each block
4.  Block metadata stored as RDF
5.  Atomic operations for consistency
6.  State root hashing for integrity verification
7.  Validation mechanisms

### Storage
1.  In-memory and persistent storage options
2.  RocksDB backend for persistent storage
3.  Backup and restore functionality
4.  Memory caching for performance
5.  Storage statistics and monitoring
6.  Database integrity checking
7.  Optimization capabilities

### CLI Interface
1.  Add RDF files as blocks
2.  Run SPARQL queries
3.  Validate blockchain integrity
4.  Dump blockchain data as JSON
5.  Run built-in demos
6.  Start web server

### Web API
1.  REST API for blockchain interaction
2.  Authentication and authorization
3.  Health check endpoints
4.  Blockchain status monitoring
5.  Transaction submission
6.  Query execution

### Transaction System
1.  Full transaction blockchain implementation
2.  Digital signatures with Ed25519
3.  Multiple demo scenarios
4.  Wallet integration

### Advanced Features
1.  Governance mechanisms
2.  Knowledge graph processing
3.  Analytics capabilities
4.  Performance optimizations
5.  Production deployment features
6.  Universal demonstration framework

## Codebase Analysis (as of August 24, 2025)

### High-Level Structure

The codebase is organized as a standard Rust library crate (`provchain-org`) with multiple binary targets. The main library (`src/lib.rs`) exports numerous modules covering different aspects of the system. Key directories under `src/` include:

*   **`core/`**: Fundamental blockchain structures (`Block`, `Blockchain`) and atomic operations.
*   **`storage/`**: Implementation of the `RDFStore` using Oxigraph, including persistence with RocksDB, RDF canonicalization logic, and SPARQL query execution.
*   **`transaction/`**: Extensions to the core blockchain to support transactions, transaction pools, UTXO sets, and wallet integration.
*   **`web/`**: REST API server built with Axum, including authentication, routing, and handlers for various blockchain operations.
*   **`network/`**: Foundational P2P networking components (messages, peers, discovery, consensus basics).
*   **`semantic/`**: Ontology integration, OWL2 support, SHACL validation, and reasoning components for **domain-flexible traceability**.
*   **`wallet/`**: Wallet management, key generation/signing (Ed25519), and participant definitions.
*   **`analytics/`**: Modules for supply chain, sustainability, and predictive analytics.
*   **`knowledge_graph/`**: Builders and entity linking for knowledge graph processing.
*   **`performance/`**: Optimizations including canonicalization caching, concurrent operations, database tuning, and metrics.
*   **`production/`**: Features for production deployment like monitoring, compliance, security, and containerization.
*   **`demo/`, `uht_demo/`, `universal_demo/`**: Example applications and demonstrations of the system's capabilities.
*   **`utils/`**: Configuration and other utility functions.
*   **`domain/`, `ontology/`**: Domain-specific adapters and **flexible ontology management** for deployment-time configuration.

### Core Components

1.  **Blockchain (`src/core/blockchain.rs`)**:
    *   Defines the `Block` struct with fields for index, timestamp, RDF data (as Turtle string), previous hash, and block hash.
    *   Implements block creation and hashing logic. The block hash is calculated by combining block metadata with a canonical hash of the RDF data stored in the associated named graph.
    *   Defines the `Blockchain` struct holding the chain vector and an `RDFStore` instance.
    *   Manages the genesis block and provides methods to add new blocks. It ensures the RDF data is stored in the `RDFStore` and metadata is recorded.

2.  **RDF Store (`src/storage/rdf_store.rs`)**:
    *   A central component wrapping Oxigraph's `Store`.
    *   Handles adding RDF data to named graphs (`http://provchain.org/block/{index}`).
    *   Implements SPARQL querying capabilities.
    *   Contains the core logic for RDF canonicalization (RDFC-1.0 compliant), essential for deterministic block hashing.
    *   Manages persistent storage using RocksDB, including configuration, backup/restore, and integrity checks.
    *   Includes performance optimizations like caching and metrics collection.

3.  **Transaction Blockchain (`src/transaction/blockchain.rs`)**:
    *   Extends the core `Blockchain` to support transaction-based operations.
    *   Includes a `TransactionPool` for managing unconfirmed transactions.
    *   Integrates with `WalletManager` for participant authentication and permission checking.
    *   Maintains a transaction index and UTXO set for efficient transaction processing.
    *   Provides methods to submit transactions, validate them, and create blocks from pending transactions.

4.  **Web Server (`src/web/server.rs`)**:
    *   Built using the Axum framework.
    *   Provides a REST API with endpoints for blockchain status, block retrieval, SPARQL queries, product tracing, transaction submission, and wallet management.
    *   Implements JWT-based authentication for protected routes.
    *   Configures CORS policies for security.

5.  **Demos (`src/demo.rs`, `src/uht_demo.rs`)**:
    *   Contain example code demonstrating the system.
    *   Typically create a blockchain instance, add blocks with sample RDF data (often using the **configured traceability ontology**), and run SPARQL queries.
    *   Showcase the core functionality, including **domain-flexible ontology usage** and traceability features.

### Build System (Cargo.toml)

*   Defines the main library and several binary targets (main CLI, demos, tests).
*   Utilizes a wide range of dependencies:
    *   Core Rust ecosystem crates (`serde`, `chrono`, `anyhow`, `clap`).
    *   Networking (`tokio`, `tokio-tungstenite`).
    *   Cryptography (`ed25519-dalek`, `rand`).
    *   Web framework (`axum`).
    *   RDF processing (`oxigraph`).
    *   Utilities (`config`, `tracing`, `lz4`).
    *   Advanced features (`petgraph`, `ndarray`, `geo`, `plotters` for analytics).
*   Features are used to enable/disable certain functionalities, particularly end-to-end tests.

## Acknowledged Limitations

*   **Network Consensus:** While networking is implemented, advanced consensus mechanisms beyond basic P2P are still being developed.
*   **Scalability Testing:** Large-scale performance testing is ongoing for production deployment scenarios.