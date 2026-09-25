# ADR 0017: Use One Final Admission Boundary for Every Ledger Write

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Fail-closed ledger admission across local, network, synchronization, and bridge paths

---

## Decision

ProvChain will have one **Final Admission** boundary and it is the only component authorized to append an **Admitted Block Envelope** to the **Ledger Journal**.

Every ingress path presents an **Admission Candidate** to that boundary. This includes local CLI and API writes, transaction batching, PoA execution, block synchronization, bridge imports, and any later PBFT execution path. Those callers are adapters; none may mutate authoritative ledger history directly.

Final Admission has exactly two authoritative outcomes:

- **rejected** — no journal append and no mutation of authoritative admission, replay, privacy, or bridge state; or
- **committed** — the complete envelope is appended and durably synchronized as required by ADR 0016.

There is no observable or recoverable state called “admitted but not committed.” Proposal construction, consensus voting, compatibility checks, and other preflight validation may establish eligibility, but they cannot commit a block.

Ledger projections are updated or recovered only from committed journal history. Projection recovery is not a second admission path.

## Rationale

The current code has several routes with materially different behavior: local and HTTP paths create blocks through `add_block`, PoA and PBFT call `submit_signed_block`, synchronization validates an incoming block but recreates a different local block from its RDF data, and bridge import maintains replay state outside durable commitment. A single authority prevents those adapters from bypassing semantic, integrity, authorization, privacy, or durability rules.

Separating a pre-commit “admitted” verdict from the journal commit would also reintroduce a crash window and ambiguous client semantics. Final Admission therefore includes durable commitment rather than ending at validation.

## Consequences

- existing public mutation methods must become adapters to, or private implementation details behind, Final Admission;
- received blocks must retain their complete signed envelope rather than being recreated from payload data;
- replay and journal-authoritative **Effective Bridge State** must become atomic with or derivable from committed history;
- recovery replays projections without rerunning consensus or creating new blocks; and
- all rejection tests must prove that authoritative history and admission-related state remain unchanged.

## Related Decisions

- [ADR 0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) fixes the production semantic path that Final Admission must enforce.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the durable record and commit point owned by Final Admission.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) makes a bridge import an ordinary adapter and derives terminal import/replay facts only from the target journal.

## Implementation Status

Accepted architecture; implementation is pending. Current ledger mutation paths have not yet been consolidated behind this boundary.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
