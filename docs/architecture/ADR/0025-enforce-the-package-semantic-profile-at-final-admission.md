# ADR 0025: Enforce the Package Semantic Profile at Final Admission

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Package-declared semantic admission with identical verdicts across every ledger ingress path
**Clarified by:** [ADR 0026](./0026-validate-the-full-staged-union-and-require-candidate-focus.md) — authoritative conformance uses the complete staged union and an ordinary candidate must contribute a package-selected focus-node subject.

---

## Decision

Every ordinary provenance Admission Candidate must satisfy the active **Ontology Package** under one versioned, network-bound **Semantic Execution Profile** before Final Admission may append its Admitted Block Envelope to the Ledger Journal. Proposal-time checking is optional preflight only; Final Admission independently produces the authoritative verdict for local/API, PoA, synchronization, and bridge ingress.

The active Network Profile binds the exact Ontology Package identity, version, path-independent content digest, and Semantic Execution Profile identifier. The Proposal Digest and Admitted Block Envelope bind the same values. Local filesystem paths are resolution details and must not affect package identity; the package digest must derive from a canonical package manifest, logical asset roles or names, and the exact declared asset bytes.

The first thesis-reference Semantic Execution Profile is deliberately narrower than arbitrary SHACL. It accepts:

- node shapes selected by `sh:targetClass`;
- property shapes attached with `sh:property`, where `sh:path` is one predicate IRI rather than a complex property path;
- `sh:minCount`, `sh:maxCount`, `sh:datatype`, `sh:class`, `sh:in`, `sh:pattern` without flags, `sh:minInclusive`, `sh:maxInclusive`, and `sh:hasValue`; and
- non-validating `sh:message`, `sh:name`, and `sh:description` annotations.

The constraint components follow their RDF-term-correct W3C SHACL definitions. In particular, `sh:datatype`, `sh:in`, and `sh:hasValue` must compare RDF terms rather than display strings; numeric and temporal range comparisons must use the profile's pinned datatype/comparison rules; and a comparison error is a violation rather than an implicit pass.

The profile also fixes one SPACL-backed subclass-entailment rule for `sh:targetClass` focus selection and `sh:class` evaluation. The rule-set and ontology inputs are package-bound and versioned. Entailment is computed as a validation view only: it does not add inferred triples to Public Provenance State or its Post-State Commitment. A reasoner initialization or execution failure cannot silently downgrade the node to exact-type checking.

This bounded implementation must not claim W3C SHACL Core processor conformance, because that conformance class requires support for the full SHACL Core language. SHACL-SPARQL, custom constraint components, complex property paths, recursion, deactivation, severity policy, and every validating construct outside the enumerated profile are unsupported. Encountering one in a declared shapes graph is a package-activation error, never an instruction to ignore it.

Before a node activates the Network Profile, it must compile and inspect every core and domain shapes artifact in the bound package. Activation fails closed when an artifact is missing, empty, malformed, ill-formed for the profile, uses an unsupported construct, cannot be executed completely, has a wrong content digest, or cannot initialize the required validator and SPACL entailment rule. At least one executable active node shape must exist. Matching package metadata without executable shapes is not semantic compatibility.

Final Admission strictly parses the candidate under ADR 0024, stages the candidate into the post-block Public Provenance State without authoritative mutation, derives the profile-defined validation input, and evaluates every applicable constraint in all core and domain shapes. Semantic conformance requires a complete successful execution with no constraint violation. An envelope-bound package or profile mismatch, malformed candidate RDF, semantic non-conformance, deterministic RDF-term comparison violation, or violation of a deterministic protocol bound rejects the candidate before journal append.

A missing or unavailable local validator or reasoner, engine execution failure not deterministically attributable to the candidate, node-local timeout, memory, I/O, or resource failure follows ADR 0024: it is node incapacity and degraded state, not a globally invalid-candidate verdict. The node cannot commit or issue a Commit Receipt, but it must not fall back to partial, candidate-only, exact-class-only, or disabled validation. Missing, malformed, unsupported, or partially executable package assets remain package-activation failures; a node that cannot activate the bound package cannot participate in admission.

Validation reports, human-readable messages, execution timings, and local engine internals are diagnostic projections and do not enter consensus. Nodes recompute the verdict from the package digest, Semantic Execution Profile, staged public state, and proposal-bound inputs. The thesis claim is therefore package-declared enforcement for this enumerated profile, not support for arbitrary SHACL documents.

This decision fixes the authority, supported feature boundary, and fail-closed behavior. ADR 0026 fixes the exact dataset-to-SHACL-data-graph rule and the non-vacuous candidate focus-node rule.

## Rationale

The current package metadata and startup compatibility checks cannot prove semantic enforcement when the validator silently ignores constraints that the package declares. Identical package IDs are insufficient unless every node can execute every verdict-affecting construct under identical rules.

Putting the check in Final Admission prevents direct signed-block submission, PoA, synchronization, or bridge adapters from bypassing semantics. Rejecting a package at activation is safer and easier to evidence than discovering an unsupported shape only after a Scheduled Authority has locally committed a block.

A bounded execution profile matches the thesis packages while keeping the claim testable. Claiming general SHACL Core or SHACL-SPARQL support would require substantially more syntax, constraint components, recursion behavior, target mechanisms, and processor conformance evidence than this milestone needs.

## Consequences

- the Network Profile, package manifest, proposal, and envelope need a common Semantic Execution Profile identity and path-independent package digest;
- the production validator must compile declared shapes into an executable plan and prove that no verdict-affecting construct was dropped;
- the custom validator must implement the full enumerated constraint matrix with RDF-term-correct behavior and pinned SPACL subclass entailment;
- every Final Admission caller must receive the same semantic verdict from the same staged public state and package, while any earlier validation remains advisory;
- package activation and admission diagnostics must distinguish invalid package, invalid candidate, and node incapacity;
- positive and negative fixtures must cover every supported constraint component, unsupported constructs, malformed and empty shape graphs, reasoner failure, package mismatch, every ingress path, cross-node verdict equality, and restart/catch-up; and
- reproducible evidence must record the exact Network Profile, package digest, Semantic Execution Profile, SPACL/rule-set revision, validator build, fixtures, and test command.

## Related Decisions

- [ADR 0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) establishes the package and production semantic boundary refined here.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes this semantic verdict mandatory for every ledger-writing adapter.
- [ADR 0024](./0024-commit-canonical-post-block-public-provenance-state.md) defines the staged post-block public dataset from which semantic validation input is derived.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires an exact bridge-imported payload to pass the same target package and Final Admission verdict as every other ordinary provenance candidate.

## Implementation Status

Accepted architecture; implementation is pending. The current manifest hashes local path strings as well as contents and declares no execution-profile semantics. Semantic checking occurs during selected proposal construction, is skipped when no validator is configured, and is absent from `submit_signed_block`. PoA and other direct submission paths can therefore bypass it. The current validator extracts and meaningfully evaluates only part of the declared shapes, silently omits several checked-in constraint families, accepts empty/no-target validation vacuously, and silently drops to no reasoner when SPACL initialization fails. No implementation code changed with this decision.

## Reference

- [W3C Shapes Constraint Language (SHACL)](https://www.w3.org/TR/shacl/)

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
