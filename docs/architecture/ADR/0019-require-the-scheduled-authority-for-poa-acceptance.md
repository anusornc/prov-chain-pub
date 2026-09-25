# ADR 0019: Require the Scheduled Authority for PoA Acceptance

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Deterministic PoA proposal eligibility and competing-block prevention

---

## Decision

A PoA Block Proposal receives **Consensus Acceptance** only when its authenticated signer is an active validator under the Membership Manifest referenced by the active Network Profile and is the **Scheduled Authority** for that proposal's ledger turn.

Membership in the authority set and a valid signature are necessary but not sufficient. A correctly signed proposal from another authorized validator must be rejected when that validator is not scheduled for the turn. Every receiving node independently recomputes the schedule and checks the same rule before presenting the proposal to Final Admission.

Follower acknowledgements or quorum are not part of PoA Consensus Acceptance. They remain part of the separate Network Convergence observation defined by ADR 0018.

The deterministic schedule and its missed-turn behavior must be identical on every node. ADR 0020 fixes that schedule as height-derived round-robin and requires the network to stall rather than permit timeout-based takeover when the Scheduled Authority does not propose.

## Rationale

Treating any authorized signer as eligible permits two consortium members to create competing validly signed proposals for the same height. A unique Scheduled Authority gives every node the same eligibility answer without importing PBFT quorum semantics into the PoA reference milestone.

## Consequences

- Signer Authorization and Consensus Acceptance remain distinct admission gates;
- proposals must carry enough ledger-turn context for every node to recompute proposer eligibility;
- authority ordering and scheduling inputs must come from shared network truth rather than node-local mutable state;
- proposal validation and Final Admission must both reject an out-of-turn signer; and
- tests must include a valid authority key producing an out-of-turn proposal and prove fail-closed rejection on every node.

## Related Decisions

- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes Consensus Acceptance eligibility evidence rather than a commit path.
- [ADR 0018](./0018-separate-node-commitment-from-network-convergence.md) separates PoA eligibility and local commitment from follower convergence.
- [ADR 0020](./0020-stall-poa-on-a-missed-turn.md) defines the deterministic schedule and safety-first missed-turn behavior.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) defines the authenticated validator membership source used by this decision.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires the target Scheduled Authority even when a complete source bridge proof is present.

## Implementation Status

Accepted architecture; implementation is pending. Current PoA proposal validation verifies membership, signature, structure, and timing but does not verify that the signer is the Scheduled Authority.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
