# ADR 0007: Use WebSocket for P2P Communication

**Status:** Accepted
**Date:** 2026-01-28
**Context:** Peer-to-peer networking protocol

---

## Decision

**Use WebSocket for all peer-to-peer communication in ProvChainOrg.**

### Rationale

**Alternatives Considered:**
| Protocol | Pros | Cons | Decision |
|----------|------|------|----------|
| **WebSocket** | • Bidirectional<br/>• Low overhead<br/>• Browser-compatible | • Requires proxy support | ✅ **Chosen** |
| HTTP/2 | • Stream multiplexing | • Unidirectional (server→client) | ❌ Rejected |
| gRPC | • High performance | • Complex NAT traversal | ❌ Rejected |
| libp2p | • Full P2P stack | • Complexity overkill | ❌ Rejected |

### Benefits

1. **Bidirectional:** Real-time block propagation + voting
2. **Low Overhead:** Binary framing, efficient bandwidth
3. **Browser-Compatible:** Future web UI support
4. **Simple:** Easy to implement and debug

---

## Implementation

**Protocol:** tokio-tungstenite
**Port:** 8080 (same as Web API)
**Message Format:** JSON (MessagePack for v2)

---

**Authors:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Approval Date:** 2026-01-28
