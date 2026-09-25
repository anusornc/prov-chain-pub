# ADR 0026: Validate the Full Staged Union and Require Candidate Focus

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Deterministic cross-block semantic validation without zero-focus conformance

---

## Decision

For every ordinary provenance Admission Candidate, Final Admission derives one deterministic **SHACL Data Graph** from the complete staged post-block Public Provenance State and validates that graph before the Ledger Journal append. Candidate-local validation is preflight at most and cannot produce the authoritative admission verdict.

The SHACL Data Graph is the RDF set-union view of all asserted triples in every public-provenance named graph included by ADR 0024. Final Admission derives it from the exact staged dataset transition, never from Oxigraph or another Ledger Projection. Construction removes only the graph-name position, preserves each RDF term and the scheme-defined document-local blank-node identity, and collapses duplicate triples according to RDF graph set semantics. Identical source labels from different envelope-local blank-node scopes never merge. Ontology and shapes artifacts, ledger metadata, ciphertext and confidential plaintext, inferred triples, receipts, indexes, caches, and other projections remain excluded. The active Ontology Package and its SPACL entailment inputs are separate, envelope-bound validation inputs under ADR 0025.

This union is a deterministic validation view, not a replacement for Public Provenance State. The named-graph dataset remains the input to the Post-State Commitment. Consequently, identical assertions in different ledger graphs may collapse to one triple for SHACL evaluation while remaining separately attributable in the committed dataset and complete envelope history.

Let `F(S)` be every focus node selected by every active package node shape from staged SHACL Data Graph `S`, using the package-bound `sh:targetClass` and pinned SPACL subclass-entailment rule from ADR 0025. Final Admission evaluates every applicable declared constraint for every node in `F(S)`, including focus nodes whose facts were admitted in earlier blocks. It does not restrict the authoritative check to newly added triples or candidate-touched nodes.

Let `T(C)` be the RDF subjects of the candidate's asserted public triples after scheme-defined parsing, IRI resolution, and envelope-local blank-node scoping. The intersection compares those exact scoped RDF terms. A **Candidate Focus Node** is a node in `F(S) ∩ T(C)`. An ordinary provenance candidate is eligible only when this intersection is non-empty and the complete staged SHACL Data Graph conforms. An empty `F(S)`, empty `T(C)`, or empty intersection is a deterministic ordinary-candidate rejection, not conformance and not node incapacity.

This rule permits a candidate to extend an already recognized entity without restating its `rdf:type`, because focus selection uses the complete staged graph. Merely referring to a recognized entity as an object does not satisfy the candidate-focus requirement. Supporting data for an unrecognized subject cannot be admitted alone unless the package gives that subject a target or the data accompanies at least one candidate-touched package focus node.

The candidate-focus rule establishes non-vacuous package participation; it is not a closed-world vocabulary rule. Once a candidate has a Candidate Focus Node, unrelated public assertions remain governed only by the active package's declared constraints and other admission invariants. A duplicate-only assertion about a focus node also satisfies the intersection even when the union graph does not change; the distinct proposal, envelope, and ledger-prefix commitments still identify that admission.

An implementation may replace a full scan with incremental evaluation only if it is proven verdict-equivalent to constructing `S`, deriving the complete `F(S)`, and validating every applicable constraint. Candidate-local evaluation, cached focus sets without verified invalidation, or partial validation after a resource guard are not equivalent fallbacks.

The deterministic protocol bounds and node-incapacity distinction from ADRs 0024 and 0025 apply to union construction, focus selection, entailment, and constraint evaluation. Exceeding a network-bound deterministic limit rejects the candidate. If the verified parent conforms and applying the candidate makes the staged post-state nonconforming, Final Admission rejects that candidate. If reconstructing the committed parent instead reveals that it was already nonconforming, the node reports degraded, corrupt, or unsupported ledger state and produces no verdict about the new candidate. A local engine, timeout, memory, or I/O failure likewise yields no global candidate verdict; the degraded node cannot append or issue a Commit Receipt.

**Admission Kind** is not a caller-selected exemption. It is a closed protocol value covered by the Proposal Digest and Admitted Block Envelope and independently verified by Final Admission. This decision recognizes `OrdinaryProvenanceV1` for non-genesis candidates carrying public provenance assertions; an unknown, mismatched, or ungoverned kind rejects. Genesis and any future governance, package-transition, or private-only kind must have its own fail-closed schema and admission rule before it can enter the closed set. An exact bridge-imported public provenance payload uses `OrdinaryProvenanceV1` and receives no semantic bypass. Commit Receipts and membership artifacts remain admission evidence rather than public provenance unless a later decision explicitly changes that boundary.

Final Admission derives the applicable package and Semantic Execution Profile from the active Network Profile at the candidate ledger position; the candidate may prove the required identities but cannot choose its validator. The thesis-reference ledger starts from a state that conforms to this binding. This ADR does not authorize a package transition: absent a separately accepted transition rule, a package/profile change is unsupported and rejects. Any future transition that changes targets or constraints must prove that the current full SHACL Data Graph conforms before activation or use an explicitly governed migration or new epoch.

## Rationale

Candidate-local validation cannot evaluate constraints whose evidence spans blocks. A new Batch may refer to a Product typed in an earlier block, while a later statement about an existing Product may introduce a second value that violates `sh:maxCount`. Validating only the candidate can reject the former incorrectly and accept the latter incorrectly.

SHACL target selection can also return no focus nodes, causing a syntactically valid but semantically unrelated payload to appear conformant because no constraint ran. Requiring the candidate itself to contribute a subject selected as a focus node makes ordinary provenance admission non-vacuous without requiring that an existing entity repeat its type in every block.

Full-state evaluation is more expensive than touched-node evaluation. The reference system accepts that bounded cost because deterministic, cross-block conformance is part of the thesis claim. Equivalent incremental validation remains an optimization that must be evidenced, not a different semantic contract.

## Consequences

- Final Admission needs a deterministic dataset-to-union transformation distinct from the named-dataset Post-State Commitment;
- focus selection and subclass entailment must run against the complete staged union, while inferred triples remain outside asserted state;
- every existing focus node is revalidated after the candidate transition, so an already nonconforming imported or legacy state cannot be hidden by an unrelated candidate;
- bridge adapters cannot claim conformance from validating only imported payload triples;
- Admission Kind must be signed, envelope-bound, and checked against a closed Final Admission dispatch table rather than trusted from a caller;
- non-ordinary envelope kinds need explicit schemas and cannot reuse absence of focus nodes as an admission bypass;
- positive fixtures must cover a new focus node and extension of an existing focus node without repeated type;
- negative fixtures must cover zero focus, object-only reference to a focus node, cross-block `sh:class`, later `sh:maxCount` violation, unrelated subjects, unknown or mismatched Admission Kind, and an explicitly unsupported special envelope;
- reproducibility fixtures must cover named-graph ordering, duplicate triples across graphs, document-scoped blank nodes, subclass-selected focus nodes, three-node verdict equality, restart/catch-up, deterministic bounds, and node incapacity; and
- an incremental implementation needs differential evidence against the full-union reference algorithm over positive, negative, and randomized multi-block histories.

## Related Decisions

- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) owns the authoritative staged validation before journal append.
- [ADR 0024](./0024-commit-canonical-post-block-public-provenance-state.md) defines the asserted named-graph dataset from which the validation union is derived.
- [ADR 0025](./0025-enforce-the-package-semantic-profile-at-final-admission.md) defines the package, supported constraints, entailment rule, and fail-closed execution boundary applied here.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires exact public-payload copying and complete target staged-union validation without mapping or a bridge exemption.

## Implementation Status

Accepted architecture; implementation is pending. The current validator creates a fresh Oxigraph store from only the candidate Turtle, selects targets through exact `rdf:type`, and returns success when no target instance is found. Existing tests also permit empty shapes and do not evidence cross-block references, later changes to existing focus nodes, deterministic union construction, or non-vacuous focus. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
