# ADR 0024: Commit the Canonical Post-Block Public Provenance State

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Deterministic RDF state evidence across independent nodes, restart, and journal replay

---

## Decision

The reference ledger fixes one **State Commitment Scheme** in its genesis-bound Network Profile. The bounded milestone does not permit an in-place scheme change: changing the scheme requires a new governed ledger or network epoch. The scheme fixes the RDF input syntax and version, base-IRI or relative-IRI policy, document-local blank-node scope, reserved public-graph namespace and graph-name derivation, graph-role inclusion rules, domain separator, RDFC and hash versions, byte encoding, and deterministic protocol bounds.

Every Block Proposal and resulting Admitted Block Envelope must bind the scheme identifier, its parent's **Post-State Commitment**, and the Post-State Commitment produced after applying the candidate's public RDF assertions. The genesis parent value is the scheme-defined commitment of an empty Public Provenance State. This state transition supplements rather than replaces the exact ledger-chain anchors: the proposal must also bind the prior envelope identity and prior Ledger Prefix Hash, and Final Admission verifies all of them against its current committed tip.

The Scheduled Authority first constructs a non-authoritative state preview from its verified committed parent and candidate, calculates the proposed post-state value, and then signs the proposal. The preview grants no admission or commitment status. Final Admission independently reconstructs the staged state transition against its own current parent and verifies the proposed value.

The proposer signs a domain-separated **Proposal Digest** over every admission-relevant field except the signature itself. Those fields include the exact public payload or its digest, encrypted content or its digest when present, network/profile/manifest/package identities and hashes, ledger-chain anchors, deterministic graph identity, parent and post-state commitments, and applicable **Bridge Origin Evidence**. The **Envelope Hash** is calculated from the complete canonical envelope after the proposer signature is available; the Ledger Prefix Hash then cumulatively commits that exact envelope. These distinct layers prevent a signature-from-envelope-hash cycle.

The **Public Provenance State** is the consensus-visible RDF dataset formed only from asserted public provenance content admitted from genesis through the stated ledger position. Each envelope's public payload is parsed under its scheme-defined syntax and base policy, its blank nodes receive a document-local scope before dataset union, and its assertions are placed in a reserved named graph derived only from already-known network and ledger identifiers. The graph rule must not depend on a block or envelope hash that itself covers the state commitment.

The Post-State Commitment is SHA-256 over the scheme's domain-separated byte sequence containing the canonical N-Quads output of a conforming W3C RDFC-1.0 canonicalization of that post-block dataset. An implementation must not silently substitute a different or partial parser, dataset-construction rule, canonicalizer, or digest algorithm.

The committed public dataset excludes ontology and SHACL package files, package signatures, encrypted payload bytes, confidential plaintext, ledger bookkeeping metadata, block and peer signatures, Commit Receipts, inferred triples, query indexes, caches, and every other Ledger Projection. Consensus-visible public assertions remain included even when they describe privacy or semantic-validation events. Excluded admission-relevant artifacts remain covered by the Proposal Digest and are also bound by their exact bytes, identities, versions, or content hashes in the Admitted Block Envelope and Ledger Prefix Hash.

Final Admission constructs the candidate post-state in non-authoritative staging, verifies both ledger-chain anchors, independently canonicalizes the accumulated public dataset, and compares the computed parent and post-state commitments with the signed proposal values before any Ledger Journal append. A mismatch, malformed dataset, unsupported scheme, incomplete canonicalization, or violation of a deterministic Network Profile protocol bound rejects the Admission Candidate with no authoritative mutation.

The active Network Profile fixes deterministic input and work bounds for both the candidate and the complete accumulated post-state. At minimum these cover encoded RDF size, quad count, blank-node count, and a bounded structural-work policy. Exceeding a protocol bound is an invalid-candidate verdict shared by conforming nodes.

A node-local timeout, memory guard, I/O failure, or other resource exhaustion is different: it is node incapacity and a degraded/recovery condition, not evidence that a protocol-conforming candidate is globally invalid. The node must not commit, project, or issue a Commit Receipt until it can complete verification, but it also must not emit a conflicting validity verdict. No resource path may accept a digest produced by truncated canonicalization.

**Verified Journal Replay** starts from the scheme-defined empty dataset, checks each already committed envelope's ledger anchors and parent commitment, applies its public assertions under the original scheme, and recomputes its Post-State Commitment. Replay verifies journal integrity and rebuilds projections; it does not rerun consensus, readmit or reappend records, truncate history, or silently uncommit an envelope. A mismatch is ledger corruption or an unsupported-ledger condition: replay stops projection at the last verified position and reports degraded recovery state. An optimized or incremental implementation is permitted only when it is demonstrably byte-equivalent to full conforming canonicalization.

The existing `Block.state_root` calculation is not this commitment and cannot support the thesis claim. It snapshots the store before the candidate insertion, hashes raw store quads including projection and ontology content, does not canonicalize blank nodes conformingly, and is not covered by the current block hash/signature. It must be replaced or explicitly versioned away from the production admission path; it must not be relabelled as post-state evidence.

## Rationale

Three-node convergence requires more than identical local store implementations. Independent nodes and restart replay must derive the same digest from RDF datasets that are isomorphic modulo blank-node relabelling and quad order. RDFC-1.0 supplies a standard canonical N-Quads representation for that purpose; it does not normalize entailment, datatype lexical equivalence, or ontology semantics. The domain-separated SHA-256 digest gives the envelope a compact, versioned state commitment.

Restricting the committed dataset to asserted public provenance avoids circular hashing through block metadata and avoids treating locally derived inference, decryption, or index state as consensus truth. The Ledger Prefix Hash still commits to the complete ordered envelopes, including separately bound ciphertext and semantic-contract evidence; the Post-State Commitment instead answers the narrower question, "What public RDF provenance state results after this block?"

Canonicalizing the full accumulated dataset is costlier than hashing the current raw store or only the new payload. The reference system accepts that cost within profile-declared bounds because reproducibility and cross-node proof are in thesis scope. Performance optimizations may follow only after equivalence is demonstrated.

## Consequences

- the canonical envelope needs versioned parent and post-state commitments whose values are covered by both proposer evidence and the envelope hash;
- Proposal Digest, Envelope Hash, Ledger Prefix Hash, and Post-State Commitment are four distinct values with distinct proof meanings and must never be substituted for one another;
- Final Admission needs an isolated candidate-state view and cannot mutate the authoritative Oxigraph projection before the journal commit point;
- ontology packages, SHACL shapes, ciphertext, and other excluded artifacts need their own envelope-bound identity or content-digest fields rather than accidental inclusion in an RDF-store root;
- replay and integrity validation must verify the commitment transition at every ledger position, not only compare a cached root at the tip;
- conformance evidence must cover input syntax and IRI policy, document-scoped blank nodes, quad-order independence, independently allocated blank-node labels, reserved named graphs, restart and replay, inclusion and exclusion boundaries, mutated payloads, wrong ledger and state parents, unsupported scheme versions, protocol-bound rejection, and node-local resource incapacity;
- all three reference nodes must activate identical canonicalization identifiers and Network Profile bounds before participating; and
- benchmark evidence must report the cost of full post-state canonicalization and may not present the current pre-insertion `state_root` benchmark as evidence for this decision.

Duplicate asserted quads and a block with no public assertions may legitimately leave the Post-State Commitment unchanged. The Proposal Digest, Envelope Hash, and Ledger Prefix Hash still change because they commit the distinct block and complete envelope history.

## Related Decisions

- [ADR 0003](./0003-embedded-rdf-blocks.md) establishes RDF payloads but does not evidence a conforming production canonicalizer.
- [ADR 0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) keeps shared ontology packages and SPACL as semantic contracts rather than ledger-state projections.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) makes the complete envelope the durable source from which state is replayed.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) owns staged commitment verification before the sole journal append.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) distinguishes this public RDF state digest from the Ledger Prefix Hash over exact envelope history.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) keeps exact Bridge Origin Evidence outside Public Provenance State while requiring the unchanged imported RDF to enter target state and receive a new target Post-State Commitment.

## Implementation Status

Accepted architecture; implementation is pending. Current proposal creation calculates `state_root` before inserting the candidate, the calculation hashes the raw contents of the shared Oxigraph store, and block hash/signature validation does not bind or recheck that value. No implementation code changed with this decision.

## Reference

- [W3C RDF Dataset Canonicalization 1.0](https://www.w3.org/TR/rdf-canon/)

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
