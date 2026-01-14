# Component Ownership & Knowledge Matrix

**Purpose**: Document component ownership to reduce bus factor risk
**Last Updated**: 2026-01-14
**Status**: ⚠️ **CRITICAL** - Single owner for all components

---

## Bus Factor Alert

**Current Bus Factor**: 1 🔴
**Target Bus Factor**: 3+
**Risk**: Critical - Project continuity at risk

---

## Core Components

### 1. Blockchain Core

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🟡 Low

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| Block Management | `src/core/blockchain.rs` | High | Documented |
| Transaction Handling | `src/core/transaction.rs` | Medium | Partially documented |
| State Management | `src/core/state.rs` | High | Needs documentation |
| Block Creation | `src/core/blockchain.rs:300-500` | High | Documented |
| Hash Integrity | `src/core/blockchain.rs:600-700` | High | Documented |

**Key Knowledge**:
- Ed25519 block signing mechanism
- RDF canonicalization for block hashing
- Chain state transitions
- Block validation logic

**Documentation**:
- [CLAUDE.md - Blockchain Signing Architecture](../CLAUDE.md#blockchain-signing-architecture)
- [Architectural Overview](./ARCHITECTURE.md) (needed)

---

### 2. Consensus System

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🔴 Critical

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| Consensus Manager | `src/network/consensus.rs` | Very High | Needs documentation |
| PoA Implementation | `src/network/consensus/poa.rs` | High | Partially documented |
| PBFT Implementation | `src/network/consensus/pbft.rs` | Very High | Needs documentation |
| Message Signing | `src/network/consensus/signing.rs` | High | Documented |

**Key Knowledge**:
- Pluggable consensus architecture
- View change mechanism (PBFT)
- Message authentication
- Leader election

**Documentation Gaps**:
- ❌ Consensus algorithm flow diagrams
- ❌ State machine documentation
- ❌ Message format specifications

---

### 3. Semantic Layer (OWL2)

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🔴 Critical

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| OWL2 Enhanced Reasoner | `src/semantic/owl2_enhanced_reasoner.rs` | Very High | Partially documented |
| Base OWL Reasoner | `src/semantic/owl_reasoner.rs` | High | Partially documented |
| SHACL Validation | `src/semantic/shacl_validator.rs` | Medium | Needs documentation |
| SPARQL Integration | `src/semantic/sparql_query.rs` | High | Partially documented |

**Key Knowledge**:
- hasKey constraint validation
- Property chain inference
- Qualified cardinality
- SPARQL query patterns

**Documentation**:
- [CLAUDE.md - OWL2 Reasoner](../CLAUDE.md#enhanced-owl2-reasoner)
- [User Manual - Query Library](../user-manual/03-querying-data/query-library.md)

---

### 4. Security Module

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🟡 Medium

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| Wallet Encryption | `src/security/wallet.rs` | High | Well documented |
| Key Rotation | `src/security/key_rotation.rs` | Medium | Well documented |
| Digital Signatures | `src/security/signature.rs` | High | Partially documented |
| ChaCha20 Encryption | `src/security/encryption.rs` | Medium | Documented |

**Key Knowledge**:
- Ed25519 signing key management
- ChaCha20-Poly1305 AEAD encryption
- Argon2 password derivation
- Key rotation lifecycle

**Documentation**:
- [CLAUDE.md - Security](../CLAUDE.md#security)
- [Wallet Encryption Tests](../../tests/wallet_encryption_tests.rs)

---

### 5. Integrity Validation

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🟢 Good

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| Transaction Counter | `src/integrity/transaction_counter.rs` | Medium | Well documented |
| Blockchain Validator | `src/integrity/blockchain_validator.rs` | High | Well documented |
| SPARQL Validator | `src/integrity/sparql_validator.rs` | High | Well documented |
| Corruption Detection | `src/integrity/blockchain_validator.rs:400-600` | High | Documented |

**Documentation**:
- [CLAUDE.md - Integrity Validation System](../CLAUDE.md#integrity-validation-system)

---

### 6. Web Layer

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🟡 Medium

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| REST API Handlers | `src/web/handlers/*.rs` | Medium | Partially documented |
| WebSocket Handler | `src/web/websocket.rs` | High | Needs documentation |
| JWT Authentication | `src/web/auth.rs` | Medium | Documented |
| API Models | `src/web/models.rs` | Low | Documented |

**Key Knowledge**:
- Axum framework integration
- JWT token handling
- WebSocket connection management
- CORS configuration

---

### 7. OWL2 Reasoner (Crate)

**Owner**: @anusorn (primary)
**Backup**: ⚠️ None
**Knowledge Distribution**: 🔴 Critical

| Component | File | Complexity | Status |
|-----------|------|------------|--------|
| Tableaux Algorithm | `owl2-reasoner/src/reasoning/tableaux.rs` | Very High | Needs documentation |
| Query Engine | `owl2-reasoner/src/reasoning/query/engine.rs` | Very High | Partially documented |
| Cache System | `owl2-reasoner/src/reasoning/query/cache.rs` | High | Needs documentation |
| Profile System | `owl2-reasoner/src/profiles/*.rs` | High | Needs documentation |

**Documentation**:
- [owl2-reasoner/CLAUDE.md](../../owl2-reasoner/CLAUDE.md)

---

## Knowledge Transfer Priority

### 🔴 Critical - Immediate Action Needed

1. **Consensus Algorithms** (PBFT, PoA)
   - Document state machines
   - Create sequence diagrams
   - Add inline code comments
   - Create walkthrough examples

2. **OWL2 Reasoning**
   - Document tableaux algorithm
   - Explain property chain inference
   - Create query engine architecture doc
   - Add reasoning examples

3. **Semantic Layer Integration**
   - Document OWL2 → Blockchain integration
   - Explain SPARQL patterns
   - Create ontology loading guide

### 🟡 High Priority

4. **Web Layer**
   - Document WebSocket flow
   - API endpoint documentation
   - Authentication flow diagrams

5. **Performance Optimizations**
   - Document caching strategies
   - Query optimization patterns
   - Memory management

### 🟢 Medium Priority

6. **Testing Infrastructure**
   - Test patterns documentation
   - Integration test guides
   - Benchmark documentation

---

## Onboarding Checklist for New Maintainers

### Phase 1: Environment Setup (Day 1)

- [ ] Development environment configured
- [ ] Project builds successfully
- [ ] Tests pass locally
- [ ] Can run the application
- [ ] Can run benchmarks

### Phase 2: Core Understanding (Week 1)

- [ ] Read CLAUDE.md thoroughly
- [ ] Read all documentation in docs/
- [ ] Understand blockchain core architecture
- [ ] Understand consensus mechanism
- [ ] Understand semantic layer

### Phase 3: Component Specialization (Month 1)

Choose 2-3 components to specialize in:

**Blockchain Core Track**:
- [ ] Block creation flow
- [ ] Block validation
- [ ] Chain state management
- [ ] Hash integrity verification

**Consensus Track**:
- [ ] PoA implementation
- [ ] PBFT implementation
- [ ] Message signing
- [ ] View changes

**Semantic Layer Track**:
- [ ] OWL2 reasoning basics
- [ ] SHACL validation
- [ ] SPARQL queries
- [ ] Ontology loading

### Phase 4: Active Contribution (Ongoing)

- [ ] Fix 3-5 issues
- [ ] Add test coverage
- [ ] Review 1-2 PRs
- [ ] Document one component
- [ ] Present knowledge to team

---

## Succession Planning

### Current State

| Role | Current Holder | Backup | Readiness |
|------|----------------|--------|-----------|
| **Lead Maintainer** | @anusorn | None | 🔴 |
| **Security Lead** | @anusorn | None | 🔴 |
| **Semantic Tech Lead** | @anusorn | None | 🔴 |
| **Infrastructure Lead** | @anusorn | None | 🔴 |

### Target State (6-12 months)

| Role | Primary | Backup | Readiness |
|------|---------|--------|-----------|
| **Lead Maintainer** | @anusorn | @contributor1 | 🟢 |
| **Security Lead** | @anusorn | @contributor2 | 🟡 |
| **Semantic Tech Lead** | @contributor3 | @anusorn | 🟡 |
| **Infrastructure Lead** | @contributor4 | @contributor1 | 🟢 |

---

## Documentation Roadmap

### Q1 2026: Critical Documentation

1. **Architecture Overview**
   - [ ] System architecture diagram
   - [ ] Component interaction diagrams
   - [ ] Data flow diagrams
   - [ ] Deployment architecture

2. **Consensus Documentation**
   - [ ] PoA algorithm walkthrough
   - [ ] PBFT algorithm walkthrough
   - [ ] State machine documentation
   - [ ] Message format specifications

3. **OWL2 Reasoner Documentation**
   - [ ] Tableaux algorithm explanation
   - [ ] Query engine architecture
   - [ ] Cache system design
   - [ ] Performance characteristics

### Q2 2026: Operational Documentation

4. **Runbooks**
   - [ ] Deployment runbook
   - [ ] Troubleshooting guide
   - [ ] Performance tuning
   - [ ] Incident response

5. **Developer Guides**
   - [ ] Deep dive for each component
   - [ ] Performance optimization guide
   - [ ] Security audit checklist
   - [ ] Code review guidelines

---

## Risk Mitigation

### Immediate Actions (This Week)

1. **Document Critical Paths**
   - Block creation and validation
   - Consensus message flow
   - Key operations

2. **Create Architecture Diagrams**
   - High-level system architecture
   - Component interactions
   - Data flows

3. **Setup Knowledge Sharing**
   - Weekly knowledge transfer sessions
   - Code walkthrough videos
   - Written decision records

### Short-term (This Month)

4. **Expand Contributor Base**
   - Identify potential contributors
   - Create onboarding program
   - Mentor first contributors

5. **Improve Documentation**
   - Add inline code comments
   - Create component guides
   - Document design decisions

### Long-term (This Quarter)

6. **Build Community**
   - Target 2-3 active contributors
   - Establish review process
   - Create contribution incentives

---

## Measuring Progress

### Bus Factor Improvement Metrics

| Metric | Current | Month 1 | Month 3 | Month 6 |
|--------|---------|---------|---------|---------|
| Active Contributors | 1 | 2 | 3 | 4 |
| Components with Backup | 0/7 | 2/7 | 4/7 | 6/7 |
| Documentation Coverage | 30% | 50% | 70% | 90% |
| PR Reviewers | 1 | 2 | 3 | 3 |

### Knowledge Distribution Score

**Current**: 15/100 🔴
**Month 1 Target**: 40/100 🟡
**Month 3 Target**: 60/100 🟢
**Month 6 Target**: 80/100 🟢

---

## Contact

**Primary Maintainer**: @anusorn
**Backup**: ⚠️ None (urgent: identify backups)

**For questions about components**:
- GitHub Issues: For specific questions
- GitHub Discussions: For general discussions
- Email: (when established)

---

*This document should be updated monthly as new contributors join and knowledge is transferred.*
*Last updated: 2026-01-14*
*Next review: 2026-02-14*
