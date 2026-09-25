# ADR 0016: Make the Ledger Journal the Sole Commit Authority

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Durable block commitment, crash recovery, and projection consistency

---

## Decision

A block becomes a **Committed Block** only when its complete, versioned **Admitted Block Envelope** has been appended to the **Ledger Journal** and that append has been durably synchronized. The successful append plus `fsync` is the single commit point.

The envelope must be self-contained enough to verify and reconstruct the committed block. It therefore carries the public provenance payload, encrypted payload when present, semantic package and network-profile identity, chain fields and state commitment, validator identity and signature, and any applicable **Bridge Origin Evidence** or receipt evidence.

The in-memory chain, Oxigraph graphs, and indexes are **Ledger Projections**, not independent commit authorities. They must be derivable and idempotently rebuildable from the Ledger Journal. A projection failure after commit places the node in a degraded or recovery state; it does not silently discard or reverse the Committed Block.

No API or network path may acknowledge a block as committed or emit a committed-block notification before the journal append is durably synchronized. Consensus proposal messages remain distinct from committed-block notifications.

The participant-local **Custody Store Commit** fixed by ADR 0036 is a separate durability boundary. A directory-`fsync`ed custody snapshot can make exact private-client state durable, but it cannot commit a block, advance a Ledger Prefix, establish Network Convergence, or make a Participant Key Version network-authoritative. Conversely, a Ledger Journal commit does not prove that any participant retained the corresponding private key.

This decision defines durable commit truth. [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) assigns that commit point to the single Final Admission boundary; its exact ordered gates remain a separate decision.

## Rationale

ProvChain cannot honestly provide one atomic transaction across an append-only file, Oxigraph, in-memory state, and separate indexes. Choosing one complete durable record as the authority gives crash recovery a deterministic source of truth and prevents transient query or cache state from deciding ledger membership.

This also removes the current durability ambiguity in which persistence records only block metadata, RDF data lives in a separate file, encrypted payload content is not reconstructed, and WAL synchronization can be batched after the method described as the commit point.

## Consequences

- the durable record format requires explicit versioning and migration policy;
- each committed append incurs a durability barrier unless a later decision introduces semantics-preserving group commit;
- projection writers and recovery must be idempotent and expose degraded state rather than conceal divergence;
- complete encrypted and semantic identity data must survive restart; and
- tests must inject crashes before and after the journal commit point and prove deterministic recovery.

## Related Decisions

- [ADR 0002](./0002-use-oxigraph-rdf-store.md) still selects Oxigraph as the RDF query and projection backend; its ACID boundary does not make Oxigraph the ledger-wide commit authority.
- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) makes atomic fail-closed admission and durable privacy part of the current thesis-reference milestone.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes Final Admission the only owner of the journal commit transition.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) defines the distinct participant-local Custody Store Commit without creating a second ledger commit authority.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires exact Bridge Origin Evidence and terminal Imported state to commit in, or derive solely from, the target journal.

## Implementation Status

Accepted architecture; implementation is pending. The current persistence and block-admission paths do not yet satisfy this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
