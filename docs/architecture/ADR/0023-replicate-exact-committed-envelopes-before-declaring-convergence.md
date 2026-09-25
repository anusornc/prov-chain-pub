# ADR 0023: Replicate Exact Committed Envelopes Before Declaring Convergence

**Status:** Accepted
**Date:** 2026-08-30
**Context:** End-to-end three-node PoA convergence without adding a follower voting protocol

---

## Decision

The thesis reference system uses producer-first, post-commit replication of the exact canonical **Admitted Block Envelope**. This is a one-phase replication and evidence protocol, not a proposal-vote round.

The Scheduled Authority first presents its Admission Candidate to local Final Admission. Only after the exact envelope bytes have been appended to the Ledger Journal and `fsync` has succeeded may the node read or expose those committed bytes for replication. It broadcasts the unchanged canonical envelope together with its envelope hash and **Ledger Prefix Hash**. A metadata-only announcement cannot establish replication or convergence.

Each follower receives the envelope through an Authenticated Peer Session, independently rechecks the active membership and profile bindings, Scheduled Authority, PoA signature and turn, chain anchor, integrity fields, and every other Final Admission gate, and presents the unchanged canonical bytes to its own Final Admission. A follower must not reconstruct a new block from RDF content or replace any field supplied by the committed envelope.

After and only after its own journal append and `fsync` succeed, each node issues a signed **Commit Receipt**. The receipt is signed with that node's Node Identity Key and binds at least the node, network, Network Profile, Membership Manifest, ledger position, envelope hash, and Ledger Prefix Hash. The producer's local receipt plus matching receipts from both followers establishes a Network-Converged Block for the three-node reference topology.

Commit Receipts are convergence evidence only. They are not follower votes, do not grant Consensus Acceptance, do not gate or undo node-local commitment, and do not introduce quorum finality. Receipt collection must be durable or reproducibly recoverable, and an unacknowledged committed envelope remains a retry obligation across process restart.

On reconnect or rejoin, authenticated nodes compare ledger position and Ledger Prefix Hash. A behind node requests bounded, contiguous ranges of exact envelope bytes anchored to its last matching prefix and passes them sequentially through Final Admission. Any authenticated node with the matching committed prefix may serve the range; the original producer is not a permanent availability dependency.

Receiving an identical already committed envelope is idempotent and may produce the same receipt again. A gap triggers range synchronization before admission continues. A different envelope or prefix at an already committed position is divergence or equivocation evidence: the node must fail closed, halt that synchronization path, and report the conflict. It must never overwrite, rewind, recreate, select a longest chain, or choose a winner by timestamp or arrival order.

## Rationale

ADR 0018 intentionally makes commitment node-relative and keeps follower acknowledgements outside PoA consensus. Producer-first exact-envelope replication preserves that boundary while providing observable proof that all three reference nodes durably recorded identical bytes and history.

Using the Ledger Journal as the replication source closes the crash window between local commit and broadcast: after restart the node can rediscover what must be sent. Signed receipts make convergence evidence reproducible without pretending that the receipts caused commitment. Anchored range synchronization supplies the same validation path for normal propagation, packet loss, restart, and rejoin.

## Consequences

- the P2P protocol must carry complete canonical envelope bytes, range requests/responses, prefix checkpoints, and signed Commit Receipts;
- the local durable-commit result must expose the exact journal record and hashes that may be replicated;
- receipt and retry state must survive restart or be deterministically reconstructed from journal and peer checkpoints;
- every ingress path, including live propagation and catch-up, must use the same Final Admission implementation;
- an offline follower may catch up later, while the next Scheduled Authority can propose only from the committed prefix it actually holds;
- convergence APIs and evidence bundles must distinguish local commitment, pending replication, divergence, and matching three-node convergence;
- tests must cover commit-before-broadcast crash, dropped broadcast, duplicate delivery, dropped receipt, restart, rejoin, bounded range catch-up, future-envelope gap, modified envelope, wrong prefix, and conflicting committed position; and
- current metadata announcements, bare-block responses, payload-only reconstruction, timestamp conflict replacement, and non-retrying stale requests cannot remain on the conforming thesis-reference PoA path.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the source bytes and durable commit point used for replication.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) requires identical admission logic for producer, follower, and catch-up paths.
- [ADR 0018](./0018-separate-node-commitment-from-network-convergence.md) defines node-local commitment and three-node convergence as separate facts.
- [ADR 0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) defines the proposal eligibility evidence followers must recompute.
- [ADR 0020](./0020-stall-poa-on-a-missed-turn.md) forbids fork choice or takeover when progress is unavailable.
- [ADR 0022](./0022-authenticate-peer-sessions-with-mutual-ed25519-challenge-response.md) authenticates the peer session while leaving ledger evidence independently verifiable.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires exact matching receipts from all three pinned source nodes as bridge evidence without converting them into consensus votes, SPV, or generic finality.

## Implementation Status

Accepted architecture; implementation is pending. The current producer commits locally and broadcasts only block metadata; followers merely log the announcement; `start-node` does not register the synchronization manager; bare-block synchronization recreates a different block from RDF data; and conflicts may overwrite in-memory history according to timestamp.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
