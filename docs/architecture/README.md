# ProvChainOrg Architecture Documentation Index

**Version:** 1.0  
**Last Updated:** 2026-01-17  
**Thesis:** Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability

---

## Documentation Structure

This directory contains comprehensive architecture documentation for ProvChainOrg, organized using the C4 Model and Arc42 framework.

### Quick Navigation

| Document | Level | Description |
|----------|-------|-------------|
| [System Context](./SYSTEM_CONTEXT.md) | C4 Level 1 | High-level system context, stakeholders, external systems |
| [Container Architecture](./CONTAINER_ARCHITECTURE.md) | C4 Level 2 | Container/service architecture and deployment |
| [Component Architecture](./COMPONENT_ARCHITECTURE.md) | C4 Level 3 | Detailed component design and interactions |
| [ADR/](./ADR/) | Decision Records | Historical architectural decisions |

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
- Semantic Layer container (OWL2 Reasoner)
- RDF Store container (Oxigraph)
- Monitoring stack (Prometheus + Grafana)

### Level 3: Component Architecture

**File:** `COMPONENT_ARCHITECTURE.md`

**Contents:**
- Blockchain Core components (State Manager, Consensus Engine, Block Creator)
- Semantic Layer components (OWL2 Reasoner, SHACL Validator, Query Optimizer)
- Web API components (Auth, Transaction Handler, Query Handler)

---

## Architecture Decision Records (ADRs)

### ADR Index

| ID | Title | Status | Date |
|----|-------|--------|------|
| 0001 | Use Rust for Blockchain Core | Accepted | 2024-01-15 |
| 0002 | Use Oxigraph for RDF Storage | Accepted | 2024-01-15 |
| 0003 | Embed RDF Graphs in Blockchain Blocks | Accepted | 2024-01-15 |

---

## Quick Reference

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Language | Rust 1.70+ | Implementation |
| Runtime | Tokio | Async runtime |
| Semantic | Oxigraph | RDF/SPARQL store |
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
