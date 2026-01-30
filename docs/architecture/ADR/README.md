# Architecture Decision Records (ADRs)

**Version:** 1.0
**Last Updated:** 2026-01-28

---

## What are ADRs?

Architecture Decision Records (ADRs) document significant architectural decisions in the ProvChainOrg project. Each ADR captures:

1. **Context:** The problem or opportunity
2. **Decision:** What was decided
3. **Rationale:** Why this decision was made
4. **Consequences:** Positive and negative impacts
5. **Related Decisions:** Links to related ADRs

ADRs provide traceability for architectural choices and help future maintainers understand the reasoning behind design decisions.

---

## ADR Index

### Core Technology Decisions

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0001](./0001-use-rust-for-blockchain-core.md) | Use Rust for Blockchain Core | Accepted | 2024-01-15 | Language selection |
| [0002](./0002-use-oxigraph-rdf-store.md) | Use Oxigraph for RDF Storage | Accepted | 2024-01-15 | RDF store |
| [0003](./0003-embedded-rdf-blocks.md) | Embed RDF Graphs in Blockchain Blocks | Accepted | 2024-01-15 | Data structure |

### Cryptography & Security

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0004](./0004-use-ed25519-signatures.md) | Use Ed25519 for Digital Signatures | Accepted | 2026-01-28 | Signatures |
| [0005](./0005-use-chacha20-encryption.md) | Use ChaCha20-Poly1305 for Data Encryption | Accepted | 2026-01-28 | Encryption |

### Consensus & Networking

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0006](./0006-dual-consensus-protocol.md) | Implement Dual Consensus (PoA/PBFT) | Accepted | 2026-01-28 | Consensus |
| [0007](./0007-websocket-p2p-protocol.md) | Use WebSocket for P2P Communication | Accepted | 2026-01-28 | Networking |

### Application & Integration

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0009](./0009-jwt-authentication.md) | Use JWT for API Authentication | Accepted | 2026-01-28 | Authentication |
| [0010](./0010-owner-controlled-visibility.md) | Implement Owner-Controlled Data Visibility | Proposed | 2026-01-28 | Privacy |
| [0011](./0011-use-axum-framework.md) | Use Axum Web Framework | Proposed | 2026-01-28 | Web Framework |
| [0012](./0012-shacl-validation.md) | Implement SHACL Validation | Proposed | 2026-01-28 | Validation |

### Observability

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0013](./0013-monitoring-stack.md) | Use Prometheus + Grafana for Monitoring | Accepted | 2026-01-28 | Monitoring |

### Semantic & Data

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0008](./0008-rdf-canonicalization.md) | Implement RDF Canonicalization for Deterministic Hashing | Proposed | 2026-01-28 | Hashing |

---

## How to Use This Template

1. **Copy the template:** `cp template.md 0014-new-decision.md`
2. **Fill in sections:** Follow the template structure
3. **Update index:** Add entry to this README
4. **Mark status:** Set to Proposed, Accepted, Deprecated, or Superseded
5. **Get review:** Share with team for feedback
6. **Update status:** Change to Accepted after approval

---

## Template

All ADRs should use the [template.md](./template.md) which includes:

- **Context:** Problem statement and constraints
- **Decision:** Clear statement of what was decided
- **Rationale:** Why this decision was made, with alternatives
- **Performance Validation:** Benchmarks and metrics
- **Consequences:** Positive and negative impacts
- **Related Decisions:** Links to related ADRs
- **Implementation:** Code locations and testing strategy
- **References:** External sources and standards

---

## ADR Lifecycle

```
┌─────────────┐
│  Proposed   │ ← Initial draft, under discussion
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Accepted   │ ← Approved, implemented
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Deprecated  │ ← Superseded by new decision
└─────────────┘
```

---

## Contact

**Maintainer:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Thesis Advisor:** Associate Professor Dr. Ekkarat Boonchieng
**Department:** Computer Science, Faculty of Science, Chiang Mai University
