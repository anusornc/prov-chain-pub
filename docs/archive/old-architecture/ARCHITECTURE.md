# ProvChainOrg System Architecture

## Overview

ProvChainOrg is a comprehensive blockchain implementation for supply chain traceability using RDF semantics. The system combines blockchain security with semantic web technologies to create a distributed ledger for supply chain traceability with formal ontology support.

## Core Components

### 1. Blockchain Core (`src/core/`)

#### Block Structure
- Index: Sequential block number
- Timestamp: ISO 8601 timestamp
- Data: RDF data in Turtle format
- Previous Hash: Link to previous block
- Hash: This block's cryptographic hash
- State Root: State root hash for atomic consistency

#### Key Features
- **Atomic Operations**: Ensures consistency between blockchain state and RDF store through `AtomicOperationContext`
- **Canonicalization**: Uses RDF canonicalization for consistent hashing of graph data
- **Persistence**: Supports both in-memory and persistent storage with RocksDB backend

### 2. RDF Storage (`src/storage/`)

#### RDF Store
- Built on Oxigraph with support for named graphs
- Each blockchain block's data is stored in a separate named graph
- Blockchain metadata itself is stored as RDF in a dedicated graph

#### Key Features
- **Persistence**: File-based storage with N-Quads format
- **Backup/Restore**: Automated backup system with retention policies
- **Canonicalization**: Implements both custom and W3C RDFC-1.0 algorithms
- **Caching**: Memory cache with LRU eviction for performance
- **Integrity Checks**: Database validation and orphaned blank node detection

### 3. Transaction System (`src/transaction/`)

#### Transaction Types
- Production: Raw material production (farmer to batch)
- Processing: Manufacturing processes (UHT processing)
- Transport: Logistics and transport activities
- Quality: Quality control and certification
- Transfer: Ownership transfers between participants
- Environmental: Environmental monitoring data
- Compliance: Regulatory compliance events
- Governance: Governance transactions

#### Key Features
- **Digital Signatures**: Ed25519-based signing with multi-signature support
- **Validation**: Business logic validation for different transaction types
- **Transaction Pool**: Priority-based pending transaction management
- **RDF Integration**: Transactions include RDF data with semantic representation

### 4. Wallet System (`src/wallet.rs`)

#### Participant Types
- Producer: Raw material producers (farmers, suppliers)
- Manufacturer: Manufacturing facilities (UHT processors, packagers)
- LogisticsProvider: Logistics and transport providers
- QualityLab: Quality control laboratories
- Auditor: Regulatory authorities and auditors
- Retailer: Retail and distribution
- Administrator: System administrators

#### Key Features
- **Participant Management**: Role-based access control with different participant types
- **Permissions**: Granular permissions per participant type
- **Certificates**: Certificate management with expiration and status tracking
- **Key Management**: Secure storage of cryptographic keys
- **Reputation System**: Basic reputation scoring for participants

### 5. Knowledge Graph (`src/knowledge_graph/`)

#### Components
- **Entity Extraction**: Automated extraction of supply chain entities from RDF data
- **Relationship Discovery**: Identification of semantic relationships between entities
- **Graph Builder**: Pipeline for constructing knowledge graphs from blockchain data
- **Temporal Evolution**: Tracking of knowledge graph changes over time

### 6. Analytics (`src/analytics/`)

#### Modules
- **Supply Chain Analytics**: Risk assessment, supplier performance, quality metrics
- **Sustainability Tracking**: Carbon footprint calculation, ESG scoring
- **Predictive Analytics**: Demand forecasting, quality prediction, risk prediction
- **Comprehensive Reporting**: Executive summaries and detailed analytics reports

### 7. Trace Optimization (`src/trace_optimization.rs`)

#### Techniques
- **Frontier Reduction**: Techniques inspired by single-source shortest path algorithms
- **Pivot Selection**: Identification of key traceability entities for optimization
- **Performance Improvements**: Enhanced traceability query performance

### 8. Web Interface (`src/web/`)

#### Components
- **REST API**: Complete API for blockchain interaction
- **Authentication**: JWT-based authentication system
- **SPARQL Endpoint**: Query interface for RDF data
- **Static Serving**: Built-in static file server for web UI

## Key Technical Features

### 1. Semantic Blockchain
- Each block's data is stored as RDF in named graphs
- Blockchain metadata itself is stored as RDF triples
- Full SPARQL query support across all blockchain data
- Ontology integration with automatic loading and validation

### 2. Advanced Canonicalization
- Hybrid approach using both custom and W3C RDFC-1.0 algorithms
- Adaptive selection based on graph complexity analysis
- Performance benchmarks comparing both approaches
- Consistent hashing for semantic equivalence

### 3. Persistence and Reliability
- RocksDB backend for persistent storage
- Automated backup and restore functionality
- Data integrity verification and monitoring
- Memory caching for performance optimization

### 4. Supply Chain Specific Features
- Multi-participant wallet system with role-based permissions
- Comprehensive transaction types for supply chain operations
- Certificate management for compliance tracking
- Environmental monitoring and sustainability metrics

## System Integration Points

1. **Blockchain ↔ RDF Store**: Tight integration with atomic operations ensuring consistency
2. **Transactions ↔ Wallets**: Digital signatures and permission checking
3. **Knowledge Graph ↔ Analytics**: Analytics engine built on knowledge graph data
4. **Web API ↔ Core Components**: REST interface exposing all core functionality
5. **Trace Optimization ↔ Knowledge Graph**: Enhanced traceability using graph algorithms

## Data Flow

1. **Transaction Creation**: Participants create transactions with RDF data
2. **Signing**: Transactions are signed with participant wallets
3. **Validation**: Business logic and signature validation
4. **Blockchain Storage**: Valid transactions are added to blockchain blocks
5. **RDF Storage**: Block data is stored in named graphs in the RDF store
6. **Knowledge Graph Construction**: Entities and relationships are extracted from RDF data
7. **Analytics Processing**: Analytics engine processes knowledge graph data
8. **Web Interface**: Users interact with the system through REST API and web UI

## Security Features

- **Digital Signatures**: Ed25519 cryptography for transaction signing
- **Role-Based Access**: Granular permissions for different participant types
- **Certificate Management**: Validation of participant credentials
- **Data Integrity**: RDF canonicalization for consistent hashing
- **Authentication**: JWT-based authentication for web API

This architecture demonstrates a sophisticated integration of blockchain technology with semantic web standards, specifically tailored for supply chain traceability applications. The modular architecture allows for extensibility while maintaining clear separation of concerns between different components.