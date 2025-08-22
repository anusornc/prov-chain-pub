# Qwen Code Context

This document provides context for Qwen Code about the 'UHT Traceability Blockchain (PoC)' project.

## Project Summary

This is a comprehensive Rust implementation for a blockchain where each block stores RDF triples in a named graph using an in-memory Oxigraph store. The ontology is based on PROV-O, extended for traceability, with a focus on UHT manufacturing but designed to be applicable to other supply chains.

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

## Acknowledged Limitations

*   **Blank Node Complexity:** While RDF canonicalization is implemented, complex blank node structures can still impact performance.
*   **Network Consensus:** While networking is implemented, advanced consensus mechanisms beyond basic P2P are still being developed.
*   **Scalability Testing:** Large-scale performance testing is ongoing for production deployment scenarios.