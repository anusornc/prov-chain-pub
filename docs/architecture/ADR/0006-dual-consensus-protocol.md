# ADR 0006: Implement Dual Consensus (PoA/PBFT)

**Status:** Accepted
**Date:** 2026-01-28
**Context:** Configurable consensus protocol for different deployment scenarios

---

## Context

ProvChainOrg operates in diverse environments requiring different consensus approaches:

1. **Private/Permissioned Networks:** Single authority trusted, fast consensus needed
2. **Public Consortium:** Multiple authorities, Byzantine fault tolerance required
3. **Development/Testing:** Fast block times, no distributed consensus needed
4. **Production:** Fault tolerance required, but performance critical

### Requirements

| Scenario | Need | Protocol Requirement |
|----------|------|----------------------|
| **Private Network** | Speed | Single authority, fast block time |
| **Consortium** | Fault Tolerance | Byzantine fault tolerance (BFT) |
| **Development** | Simplicity | No consensus needed (single node) |
| **Production** | Reliability | Configurable protocol |

---

## Decision

**Implement runtime-switchable dual consensus: Proof of Authority (PoA) and PBFT (Practical Byzantine Fault Tolerance).**

### Scope

**PoA (Proof of Authority):**
- Single designated authority signs blocks
- No voting required
- Fastest block time (~1 second)
- Use case: Private networks, development, single-node deployments

**PBFT (Practical Byzantine Fault Tolerance):**
- Multiple validators (3f+1 nodes for f faulty nodes)
- Three-phase consensus (pre-prepare, prepare, commit)
- Slower block time (~3 seconds)
- Use case: Public consortium, fault-tolerant deployments

**Implementation:**
- Trait-based consensus: `trait ConsensusProtocol`
- Runtime switching via configuration
- Both protocols share common blockchain state

---

## Rationale

### Alternatives Considered

| Protocol | Pros | Cons | Decision |
|----------|------|------|----------|
| **PoA / PBFT** | • Flexible for different scenarios<br/>• Runtime switchable | • Two implementations to maintain | ✅ **Chosen** |
| **PoA Only** | • Simple, fast | • No fault tolerance<br/>• Single point of failure | ❌ Insufficient |
| **PBFT Only** | • Fault tolerant | • Slower, complex<br/>• Overkill for private networks | ❌ Inefficient |
| **RAFT** | • Leader election built-in | • Not BFT (crash fault only)<br/>• More complex state machine | ❌ Rejected |
| **Tendermint** | • BFT, proven | • Separate consensus engine<br/>• More complex integration | ❌ Overkill |

### Key Benefits

1. **Flexibility:** Right protocol for each deployment scenario
2. **Performance:** PoA for fast private networks, PBFT for public
3. **Simplicity:** Common trait abstraction, easy to switch
4. **Fault Tolerance:** PBFT provides BFT when needed (3f+1 nodes)

---

## Implementation

### Code Locations

**Primary Implementation:**
- File: [`src/network/consensus.rs`](../../src/network/consensus.rs)
- Trait: `trait ConsensusProtocol`
- Implementations: `PoAConsensus`, `PBFTConsensus`

### Configuration

```toml
[consensus]
# "poa" or "pbft"
type = "poa"

# Authority public keys (PoA)
authority_keys = ["0xabc...", "0xdef..."]

# PBFT Validators
validators = ["0xabc...", "0xdef...", "0x123..."]

# Block interval (seconds)
block_interval = 1
```

### PoA Implementation

```rust
pub struct PoAConsensus {
    authority_key: SigningKey,
    block_interval: Duration,
}

impl ConsensusProtocol for PoAConsensus {
    fn propose_block(&self, transactions: Vec<Transaction>) -> Block {
        // Single authority creates block directly
        Block::new(transactions, &self.authority_key)
    }

    fn validate_block(&self, block: &Block) -> bool {
        // Verify authority signature
        block.verify(&authority_public_key)
    }
}
```

### PBFT Implementation

```rust
pub struct PBFTConsensus {
    validators: Vec<VerifyingKey>,
    my_index: usize,
    block_interval: Duration,
}

impl ConsensusProtocol for PBFTConsensus {
    fn propose_block(&self, transactions: Vec<Transaction>) -> Block {
        // Phase 1: Pre-prepare
        let proposed = Block::new(transactions, &self.signing_key);
        broadcast(PrePrepareMessage(proposed));
        proposed
    }

    fn validate_block(&self, block: &Block) -> bool {
        // Phase 2 & 3: Prepare and Commit
        collect_votes(PrepareMessage(block.hash));
        collect_votes(CommitMessage(block.hash));
        votes.count(true) > (2 * validators.len() / 3)
    }
}
```

---

## Performance Comparison

| Metric | PoA | PBFT | Notes |
|--------|-----|------|-------|
| **Block Time** | 1 second | 3 seconds | PBFT requires 3 network round trips |
| **Throughput** | Highest | Medium | PoA limited by single authority |
| **Fault Tolerance** | 0 (no BFT) | f Byzantine nodes | PBFT: 3f+1 nodes for f faults |
| **Network Messages** | 1 (block) | 3n² (n² per phase) | PBFT: O(n²) voting overhead |

---

## Related Decisions

- [ADR 0004](./0004-use-ed25519-signatures.md): Use Ed25519 for Signatures (used in both protocols)
- [ADR 0007](./0007-websocket-p2p-protocol.md): WebSocket for P2P (carries consensus messages)

---

**Authors:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Reviewers:** Associate Professor Dr. Ekkarat Boonchieng
**Approval Date:** 2026-01-28
