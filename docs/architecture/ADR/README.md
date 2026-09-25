# Architecture Decision Records (ADRs)

**Version:** 1.0
**Last Updated:** 2026-08-31

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
| [0027](./0027-use-participant-principals-for-privacy-identity.md) | Use Participant Principals for Privacy Identity | Accepted | 2026-08-30 | Privacy identity and key binding |
| [0028](./0028-make-privacy-control-ledger-authoritative.md) | Make Privacy Control Ledger-Authoritative | Accepted | 2026-08-30 | Durable privacy control state |
| [0029](./0029-require-participant-authorization-after-governed-bootstrap.md) | Require Participant Authorization After Governed Bootstrap | Accepted | 2026-08-30 | Privacy transition authorization |
| [0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) | Bootstrap the Principal and Initial Authorization Key Atomically | Accepted | 2026-08-30 | Atomic participant bootstrap |
| [0031](./0031-use-one-active-participant-key-per-purpose.md) | Use At Most One Active Participant Key per Purpose | Accepted | 2026-08-30 | Participant key lifecycle and rotation |
| [0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) | Make Privacy Grant Revocation Terminal and Prospective | Accepted | 2026-08-30 | Privacy Grant lifecycle and live release |
| [0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) | Use One Immutable Ciphertext and Per-Object DEK Envelopes | Accepted | 2026-08-30 | Protected-data envelope topology |
| [0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) | Pin ProtectedDataSuiteV1 and Canonical Privacy Encoding | Accepted | 2026-08-31 | Mandatory protected-data suite and codec |
| [0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) | Keep Participant Private Keys in Durable Client-Only Custody | Accepted | 2026-08-31 | Participant private-key custody boundary |
| [0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) | Pin ParticipantKeystoreSuiteV1 and Whole-Snapshot Custody Commit | Accepted | 2026-08-31 | Canonical participant keystore and local durability |

### Consensus & Networking

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0006](./0006-dual-consensus-protocol.md) | Implement Dual Consensus (PoA/PBFT) | Accepted | 2026-01-28 | Consensus |
| [0007](./0007-websocket-p2p-protocol.md) | Use WebSocket for P2P Communication | Accepted | 2026-01-28 | Networking |
| [0018](./0018-separate-node-commitment-from-network-convergence.md) | Separate Node Commitment from Network Convergence | Accepted | 2026-08-29 | PoA convergence semantics |
| [0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) | Require the Scheduled Authority for PoA Acceptance | Accepted | 2026-08-29 | PoA proposal eligibility |
| [0020](./0020-stall-poa-on-a-missed-turn.md) | Stall PoA on a Missed Turn | Accepted | 2026-08-29 | PoA missed-turn safety |
| [0021](./0021-use-a-governance-signed-membership-manifest.md) | Use a Governance-Signed Membership Manifest | Accepted | 2026-08-29 | Authenticated membership source |
| [0022](./0022-authenticate-peer-sessions-with-mutual-ed25519-challenge-response.md) | Authenticate Peer Sessions with Mutual Ed25519 Challenge-Response | Accepted | 2026-08-30 | Peer proof of possession |
| [0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) | Replicate Exact Committed Envelopes Before Declaring Convergence | Accepted | 2026-08-30 | Three-node PoA replication |

### Application & Integration

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0009](./0009-jwt-authentication.md) | Use JWT for API Authentication | Accepted | 2026-01-28 | Authentication |
| [0011](./0011-use-axum-framework.md) | Use Axum Web Framework | Proposed | 2026-01-28 | Web Framework |
| [0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) | Bound the ProvChain Bridge to Converged Source Evidence and Target Final Admission | Accepted | 2026-08-31 | ProvChain-to-ProvChain bridge |
| [0038](./0038-bind-source-bridge-export-envelopes-to-the-parent-ledger-prefix.md) | Bind Source Bridge Export Envelopes to the Parent Ledger Prefix | Accepted | 2026-09-05 | Non-genesis bridge prefix verification |

### Observability

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0013](./0013-monitoring-stack.md) | Use Prometheus + Grafana for Monitoring | Accepted | 2026-01-28 | Monitoring |

### Semantic & Data

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) | Use Shared Ontology Packages and SPACL as the Production Semantic Path | Accepted | 2026-03-09 | Semantic architecture |
| [0024](./0024-commit-canonical-post-block-public-provenance-state.md) | Commit the Canonical Post-Block Public Provenance State | Accepted | 2026-08-30 | Deterministic RDF state commitment |
| [0025](./0025-enforce-the-package-semantic-profile-at-final-admission.md) | Enforce the Package Semantic Profile at Final Admission | Accepted | 2026-08-30 | Package-declared SHACL enforcement |
| [0026](./0026-validate-the-full-staged-union-and-require-candidate-focus.md) | Validate the Full Staged Union and Require Candidate Focus | Accepted | 2026-08-30 | Cross-block semantic conformance |

### Ledger Admission & Persistence

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) | Make the Ledger Journal the Sole Commit Authority | Accepted | 2026-08-29 | Durable block commitment |
| [0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) | Use One Final Admission Boundary for Every Ledger Write | Accepted | 2026-08-29 | Fail-closed ledger admission |

### Research & Delivery Scope

| ID | Title | Status | Date | Topic |
|----|-------|--------|------|-------|
| [0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) | Bound Remediation to a Thesis-Defensible End-to-End Reference System | Accepted | 2026-08-29 | Thesis remediation scope |

---

## How to Use This Template

1. **Copy the template:** `cp template.md 0038-new-decision.md`
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
