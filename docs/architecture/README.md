# ProvChainOrg Architecture Documentation Index

**Version:** 1.0  
**Last Updated:** 2026-01-17  
**Thesis:** Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability

---

## Documentation Structure

This directory contains comprehensive architecture documentation for ProvChainOrg, organized using the C4 Model and Arc42 framework.

### Quick Navigation

| Document | Level | Description | Status |
|----------|-------|-------------|--------|
| [System Context](./SYSTEM_CONTEXT.md) | C4 Level 1 | High-level system context, stakeholders, external systems | ✅ Complete |
| [Container Architecture](./CONTAINER_ARCHITECTURE.md) | C4 Level 2 | Container/service architecture and deployment | ✅ Complete |
| [Component Architecture](./COMPONENT_ARCHITECTURE.md) | C4 Level 3 | Detailed component design and interactions | ⏳ Planned |
| [Data Flow Architecture](./DATA_FLOW_ARCHITECTURE.md) | Supporting | Transaction and query flows | ✅ Complete |
| [Security Architecture](./SECURITY_ARCHITECTURE.md) | Supporting | Security layers and threat model | ✅ Complete |
| [Integration Architecture](./INTEGRATION_ARCHITECTURE.md) | Supporting | External system integrations | ✅ Complete |
| [Shared Ontology Network Use Cases](./SHARED_ONTOLOGY_NETWORK_USE_CASES.md) | Reference Model | Canonical actors, lifecycle, and end-to-end flows for the shared-ontology network | ✅ New |
| [Shared Ontology Network Working Plan](./SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md) | Working Plan | Current architecture truth, guardrails, and phased plan | ✅ Active |
| [Shared Ontology Network Architecture Figures](./SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md) | Figure Package | Paper-ready PNG/SVG architecture figures for the shared-ontology network model | ✅ New |
| [ADR/](./ADR/) | Decision Records | Historical architectural decisions | ✅ Expanded |
| [ADR 0014](./ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md) | Decision Record | Shared ontology packages + SPACL production path | ✅ New |

---

## C4 Model Documentation

### Level 1: System Context

**File:** `SYSTEM_CONTEXT.md`

**Contents:**
- System purpose and scope
- Stakeholder analysis
- External system integrations
- Quality attributes (performance, reliability, security)
- Business domain model

### Level 2: Container Architecture

**File:** `CONTAINER_ARCHITECTURE.md`

**Contents:**
- Web API container (Axum + JWT)
- Blockchain Core container (PoA/PBFT)
- Semantic Layer container (shared ontology packages + SPACL)
- RDF Store container (Oxigraph)
- Monitoring stack (Prometheus + Grafana)

### Level 3: Component Architecture

**File:** `COMPONENT_ARCHITECTURE.md`

**Contents:**
- Blockchain Core components (State Manager, Consensus Engine, Block Creator)
- Semantic Layer components (ontology manager, SHACL validator, SPACL integration)
- Web API components (Auth, Transaction Handler, Query Handler)

---

## Architecture Decision Records (ADRs)

### ADR Index

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| **Core Technology** |||||
| [0001](./ADR/0001-use-rust-for-blockchain-core.md) | Use Rust for Blockchain Core | Accepted | 2024-01-15 | Language |
| [0002](./ADR/0002-use-oxigraph-rdf-store.md) | Use Oxigraph for RDF Storage | Accepted | 2024-01-15 | RDF store |
| [0003](./ADR/0003-embedded-rdf-blocks.md) | Embed RDF Graphs in Blockchain Blocks | Accepted | 2024-01-15 | Data structure |
| **Cryptography & Security** |||||
| [0004](./ADR/0004-use-ed25519-signatures.md) | Use Ed25519 for Digital Signatures | Accepted | 2026-01-28 | Signatures ✨ NEW |
| [0005](./ADR/0005-use-chacha20-encryption.md) | Use ChaCha20-Poly1305 for Data Encryption | Accepted | 2026-01-28 | Encryption ✨ NEW |
| **Consensus & Networking** |||||
| [0006](./ADR/0006-dual-consensus-protocol.md) | Implement Dual Consensus (PoA/PBFT) | Accepted | 2026-01-28 | Consensus ✨ NEW |
| [0007](./ADR/0007-websocket-p2p-protocol.md) | Use WebSocket for P2P Communication | Accepted | 2026-01-28 | P2P ✨ NEW |
| **Application & Integration** |||||
| [0009](./ADR/0009-jwt-authentication.md) | Use JWT for API Authentication | Accepted | 2026-01-28 | Authentication ✨ NEW |
| [0010](./ADR/0010-owner-controlled-visibility.md) | Implement Owner-Controlled Data Visibility | Proposed | 2026-01-28 | Privacy |
| [0011](./ADR/0011-use-axum-framework.md) | Use Axum Web Framework | Proposed | 2026-01-28 | Web Framework |
| [0012](./ADR/0012-shacl-validation.md) | Implement SHACL Validation | Proposed | 2026-01-28 | Validation |
| **Observability** |||||
| [0013](./ADR/0013-monitoring-stack.md) | Use Prometheus + Grafana for Monitoring | Accepted | 2026-01-28 | Monitoring ✨ NEW |
| **Semantic & Data** |||||
| [0008](./ADR/0008-rdf-canonicalization.md) | Implement RDF Canonicalization for Deterministic Hashing | Proposed | 2026-01-28 | Hashing |
| [0014](./ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md) | Use Shared Ontology Packages and SPACL as the Production Semantic Path | Accepted | 2026-03-09 | Semantic architecture ✨ NEW |

---

## Quick Reference

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Language | Rust 1.70+ | Implementation |
| Runtime | Tokio | Async runtime |
| Semantic | SPACL `owl2-reasoner` + Oxigraph | Shared-ontology reasoning, validation, query support |
| Crypto | Ed25519 | Signatures |
| Encryption | ChaCha20-Poly1305 | Private data |
| Web | Axum | HTTP framework |
| Auth | JWT | Authentication |
| P2P | WebSocket | Peer communication |

### Quality Attributes

| Attribute | Target | Actual | Notes |
|-----------|--------|--------|-------|
| **Performance** |
| Write Throughput | > 8,000 TPS | **19.58 TPS** ⚠️ | Dev environment (single node) |
| Read Latency (P95) | < 100ms | 0.04-18ms ✅ | SPARQL queries |
| Block Time | 1-5 seconds | 1 second (PoA) ✅ | |
| **Reliability** |
| Availability | 99.9% | 99.95% ✅ |
| Fault Tolerance | 1/3 (PBFT) | Met ✅ |
| Data Integrity | 100% | Met ✅ |
| **Security** |
| Authentication | JWT + Ed25519 | Met ✅ |
| Encryption | ChaCha20-Poly1305 | Met ✅ |
| Audit Trail | Immutable | Met ✅ |

---

## Related Documentation

### External
- [Main README](../../README.md)
- [Contributing Guide](../../CONTRIBUTING.md)
- [User Manual](../USER_MANUAL.md)
- [Benchmarking Guide](../../BENCHMARKING.md)

---

## Contact

**Author:** Anusorn Chaikaew (Student Code: 640551018)  
**Thesis Advisor:** Associate Professor Dr. Ekkarat Boonchieng  
**Department:** Computer Science, Faculty of Science, Chiang Mai University
