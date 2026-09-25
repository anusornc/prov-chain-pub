# ADR 0020: Stall PoA on a Missed Turn

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Safety and liveness when a Scheduled Authority is unavailable without a quorum or view-change protocol

---

## Decision

The thesis reference system assigns each next ledger position to exactly one **Scheduled Authority** by a deterministic, height-derived round-robin over the authority order declared by the active Network Profile. Every node must derive the same **PoA Turn** from the same committed prefix and profile.

The configured `block_interval` is the minimum permitted spacing between eligible proposals. It is not a takeover timeout and does not authorize another validator to propose for the same ledger position.

If the Scheduled Authority does not produce the next proposal, the network enters a **Stalled Network** condition. No other authority may take over that turn. When the authority returns, it may produce or rebroadcast the proposal for that still-pending position, subject to all normal admission checks.

Distinct signed Block Proposals by the Scheduled Authority for the same PoA Turn constitute **Equivocation**. Nodes must fail closed; they must not select a winner by timestamp, arrival order, or in-place chain replacement.

Automatic failover, timeout-based leader replacement, quorum certificates, and view change are outside the current PoA reference-system scope. They are future consensus work.

## Rationale

Allowing timeout-based takeover without quorum evidence can cause different partitions to accept different authorities for the same ledger position. Stalling sacrifices availability when the scheduled authority is offline, but preserves a deterministic safety rule that every reference node can verify independently. This trade-off matches the bounded thesis goal while leaving operational high availability to a later protocol milestone.

## Consequences

- PoA liveness depends on the Scheduled Authority for the pending turn;
- authority ordering and the schedule inputs must be identical, versioned network truth;
- node health and evidence tooling must distinguish a stalled network from a converged or failed ledger;
- a restarted authority must resume or rebroadcast work for the still-pending ledger position rather than advance a node-local round;
- tests must prove that an unavailable scheduled authority causes no height advancement, an out-of-turn authority is rejected, recovery resumes the pending turn, and equivocation fails closed; and
- current conflict handling that prefers an earlier timestamp or replaces an in-memory block must be removed from the conforming thesis-reference PoA path.

## Related Decisions

- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) places automatic failover and operational deployment outside the current milestone.
- [ADR 0018](./0018-separate-node-commitment-from-network-convergence.md) separates node-local commitment from three-node convergence.
- [ADR 0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) requires the uniquely scheduled signer for PoA Consensus Acceptance.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) applies this rule to all ordinary, privacy-control, and bridge requests through one pre-sign coordinator and a crash-safe exact-proposal Signing Fence.

## Implementation Status

Accepted architecture; implementation is pending. Current authority rotation advances from node-local mutable state after local block production, while synchronization conflict handling can prefer an earlier timestamp and replace an in-memory block. There is no unified request coordinator or crash-safe exact-proposal Signing Fence. These behaviors do not establish the deterministic, fail-closed rule in this ADR.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
