# ADR 0018: Separate Node Commitment from Network Convergence

**Status:** Accepted
**Date:** 2026-08-29
**Context:** PoA acknowledgement, replication, recovery, and three-node thesis evidence

---

## Decision

**Committed Block** is a node-relative durable fact. A node considers a block committed after its own Final Admission has appended and synchronized the complete Admitted Block Envelope to that node's Ledger Journal.

**Network Convergence** is a separate distributed fact. For the thesis reference system, a block is a **Network-Converged Block** only when all three reference nodes have independently committed the identical envelope at the same ledger position and share the identical committed prefix through that position.

A local API may report `committed` after its node's durable commit. It must report or prove `converged` separately; local commitment must not be labelled network finality. Consensus acceptance makes a proposal eligible for each node's Final Admission but is not itself local commitment or network convergence.

The PoA producer does not wait for a follower quorum to define local commitment. Nodes that restart or rejoin reconcile and rebroadcast committed journal history until the required nodes converge.

## Rationale

Making the word “committed” require follower acknowledgements would add a new quorum/finality protocol to the scoped PoA milestone and blur the deliberate boundary with future PBFT work. Conversely, treating the producer's local journal as proof of three-node behavior would not demonstrate the thesis requirement. Separate terms make both claims testable without overstating either one.

## Consequences

- a locally committed block may remain temporarily unconverged during delay, partition, or node outage;
- client receipts and observability must distinguish local commitment from measured network convergence;
- follower nodes must Final-Admit the producer's identical signed envelope rather than recreate a new block from payload data;
- restart and catch-up tests must prove convergence from committed journal history; and
- thesis evidence must compare all three committed prefixes and envelope hashes, not only the producer's height or broadcast event.

## Related Decisions

- [ADR 0006](./0006-dual-consensus-protocol.md) retains PBFT as a later consensus direction rather than importing quorum finality into the current PoA milestone.
- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) requires three-node PoA convergence and places PBFT in future work.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines each node's durable commit point.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) requires each node to commit through the same Final Admission boundary.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) defines exact-envelope replication and signed evidence for this convergence condition.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) consumes all-three source receipts as bounded bridge evidence while preserving the distinction between target-local Imported and Bridge-Converged state.

## Implementation Status

Accepted architecture; implementation is pending. The current PoA announcement and synchronization paths do not yet commit the same complete envelope on all three nodes.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
