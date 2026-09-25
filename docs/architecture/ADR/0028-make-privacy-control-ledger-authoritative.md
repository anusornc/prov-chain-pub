# ADR 0028: Make Privacy Control Ledger-Authoritative

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Durable, replayable principal, key, ownership, grant, and revocation state
**Clarified by:** [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) — bootstrap governance and parent-state participant authorization are separate from PoA, node, JWT, and administrator evidence.
**Further clarified by:** [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) — `RegisterPrincipal` atomically creates the principal and active initial authorization key under one composite proof.
**Lifecycle clarified by:** [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) — key binding, rotation, and emergency revocation use a total at-most-one-active reducer with terminal statuses.
**Grant lifecycle clarified by:** [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) — grants are immutable, revocation is terminal and prospective, and historical state is never live release authority.
**Envelope structure clarified by:** [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — each object has one Protected Payload Ciphertext and per-object DEK, with one Owner DEK Envelope and one Grant DEK Envelope per grant.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the mandatory `ProtectedDataSuiteV1` and closed canonical `PrivacyControlV1` encoding are fixed; implementation, Network Profile activation, and conformance evidence remain pending.

---

## Decision

The Ledger Journal is the sole authority for effective privacy-control state. Participant Principal and Participant Key Version bindings, protected-object ownership, grants, and revocations change only through a committed **Privacy Control Transition**. A user database, wallet ACL, shared-secret map, RDF store, index, cache, or API request is never an independent authorization source.

Privacy control uses the dedicated closed Admission Kind `PrivacyControlV1`. The kind, canonical transition bytes, one discriminated Transition Authorization Proof, all variant-specific introduced-key references and non-authorizing possession evidence, ledger anchors, Network Profile identity, and all other admission-relevant references are covered by the Proposal Digest and complete Admitted Block Envelope. A caller cannot relabel an ordinary payload or unknown operation as privacy control.

`PrivacyControlV1` is a control-only canonical discriminated union with one transition per envelope and its own fail-closed Final Admission gate. Its initial closed variants are `RegisterPrincipal`, `BindParticipantKey`, `RevokeParticipantKey`, `CreateProtectedObject`, `GrantAccess`, and `RevokeGrant`. `RegisterPrincipal` atomically creates the previously unknown principal and its active initial `privacy-authorization` key version 1; `BindParticipantKey` applies only after that bootstrap. Unknown versions, variants, fields, or reference forms reject; this version has no implicit ignored-extension namespace. Principal deletion, ownership transfer, delegation, administrator override, and batch transitions are unsupported unless a later decision adds an explicit variant and reducer rule.

Every variant must have a complete canonical field schema, signer rule, and total deterministic reducer before the Network Profile may activate `PrivacyControlV1`. ADRs 0029–0034 fix the transition authorization matrix, atomic bootstrap, participant-key lifecycle, prospective grant lifecycle, protected-data envelope topology, mandatory suite, and canonical encoding. Acceptance closes the specification gap only: until the complete implementation and required conformance evidence exist and an exact Network Profile activates them, a partial dispatcher must reject the kind rather than infer authority from local roles or implement only some variants.

This dedicated gate is not a bypass around shared admission invariants. Final Admission still verifies Admission Kind, canonical encoding, network/profile and ledger anchors, consensus eligibility, signer proof under the applicable privacy-control rule, deterministic bounds, envelope construction, and durable journal commitment. `PrivacyControlV1` does not use ADR 0026's ordinary-candidate focus-node rule because its canonical control payload is not an ordinary public-provenance assertion set. Public RDF is forbidden in this kind; package-validated public provenance remains `OrdinaryProvenanceV1`. A future atomic mixed public/control operation requires an explicit combined kind and validation order and cannot be smuggled through either existing kind.

Privacy-control metadata is consensus-visible to the permissioned reference nodes. Network-visible evidence includes canonical principal, object, grant, and transition identifiers; public-key bytes and fingerprint, purpose, version, status, and effective ledger position; owner and grantee principal identifiers; algorithm or suite identifier; exact Protected Payload Ciphertext and commitments; exact Owner and Grant DEK Envelope bytes; and transition proof. Protected plaintext, private keys, wallet master keys, passwords, JWTs, account aliases, recovery secrets, and unwrapped content-encryption keys are forbidden from the transition and envelope. Validators perform ADR 0033 Public Envelope Validation without decrypting.

`CreateProtectedObject` atomically binds a preselected unique Protected Object Identifier, owner principal, one exact durable Protected Payload Ciphertext, its commitments, and exactly one Owner DEK Envelope in one transition. `GrantAccess` adds exactly one Grant DEK Envelope without re-encrypting the payload. Neither may use a wallet or external-store locator or persist only a boolean or digest. ADR 0033 fixes this self-contained topology, and ADR 0034 fixes the concrete `ProtectedDataSuiteV1` and canonical encoding; neither decision by itself activates the profile.

Each transition has a unique, domain-separated transition identifier and a deterministic expected privacy revision tied to the verified parent Ledger Prefix Hash. Canonical UUID and identifier bytes, network/profile identity, transition variant and every operation field, the complete discriminated authorizer and key references, expected revision, and parent ledger anchor enter the applicable domain-separated signature input or inputs. A participant-controlled transition has its required Participant Authorization Signature; `BindParticipantKey` also carries the introduced key's separately tagged, non-authorizing purpose-specific possession evidence defined by ADR 0031. The composite bootstrap proof has both the governance authorization signature and introduced-key possession signature over the shared unsigned core defined by ADR 0030. A governance authorizer is not encoded as a Participant Principal or Participant Key Version. Final Admission evaluates the candidate against the Effective Privacy State at its verified parent, rejects duplicate or already-consumed transition identifiers and failed preconditions, and stages exactly one next privacy state without authoritative mutation.

The transition signer and authorization proof resolve against the committed parent state; except for the separately governed principal/key bootstrap rule, a key introduced by a transition cannot authorize that same transition. The introduced bootstrap key supplies the non-authorizing possession proof defined by ADR 0030, not a Participant Authorization Signature. A committed change becomes effective at its ledger position for following candidates. Exact redelivery of an already committed envelope is idempotent synchronization handling under ADR 0023, while reusing the same logical transition identifier in a different envelope rejects.

The **Effective Privacy State** at ledger position `h` is the deterministic fold of all committed `PrivacyControlV1` transitions from the profile-defined genesis privacy state through `h`, in exact Ledger Journal order. It contains only the effective principal/key bindings, key lifecycle status, protected-object owner, and grant/revocation state needed for authorization. Ledger position, not an untrusted request timestamp or local wall clock, orders state transitions.

The versioned reducer is total: every `(transition variant, current state)` pair has either one deterministic next state or one deterministic rejection. Duplicate principal, key, object, or grant creation; unknown references; non-positive, duplicate, or skipped key versions; re-grant of an active grant; double revocation; an invalid key-status transition; owner self-grant; and any unsupported ownership transfer reject. Grants and account or key lifecycle changes never change the recorded object owner.

Final Admission has the same two outcomes defined by ADR 0017. Against a verified healthy parent, a deterministic schema, reference, signature, signer-purpose/status, authority, uniqueness, revision, bound, or state-transition failure rejects with no journal append and no mutation of privacy state or its projections. On success, the complete envelope append plus `fsync` is the only privacy-control commit point. In-memory state and indexes update only after that commit or during recovery.

A node-local journal, privacy projection, verifier, reducer, durable-storage, timeout, or resource failure during admission is not a conflicting authorization verdict. The node fails closed and cannot append, issue a Commit Receipt, or serve an authoritative privacy read until its journal and privacy projection are current and verified. A participant wallet, vault, private key, or unwrapped-key failure is outside follower validation: it blocks only local proposal creation, signing, or decryption that requires the unavailable secret and must not block an otherwise healthy node from validating, appending, or receipting an exact envelope. Deterministic Network Profile limits on transition, ciphertext, wrapped-key, or state size reject uniformly; local resource ceilings yield incapacity and no candidate verdict. If the committed parent privacy history itself cannot be reconstructed or violates its recorded schema/reducer rules, the node enters degraded, corrupt, or unsupported-ledger state rather than blaming the next candidate.

Verified Journal Replay reconstructs Effective Privacy State from the profile-defined genesis state and exact committed envelopes. Startup replay verifies journal framing, hashes, prefix continuity, recorded schema/reducer version, and transition integrity, then deterministically applies already committed transitions. It does not rerun consensus, consult current account or key status, use wall clock, readmit, reappend, re-sign, reorder, skip, or synthesize transitions. A follower receiving an envelope not yet in its local journal still presents that exact envelope to Final Admission; that catch-up path is not startup replay. Failure stops the privacy projection at the last verified ledger position and exposes degraded state.

Every current authorization read is evaluated against an Effective Privacy State projection whose Ledger Prefix Hash and position match the required verified release prefix. A missing, stale, partially replayed, locally edited, unsynchronized, or divergent projection fails closed. Under ADR 0032, an explicitly requested historical position is audit-only: it may expose truthful historical grant facts but can never authorize plaintext, wrapped-key, DEK, decryption-oracle, or capability-token release. Read and decrypt APIs bind the evaluated prefix and cannot silently serve an older projection. Local roles, `view_all`, request-supplied owner/grantee strings, or cached key possession cannot override the journal-derived result.

Only privacy-authorization views—such as the public participant-key registry, wallet ACL view, grant index, and protected-object owner index—are journal projections. Password verifiers, account sessions, username-to-principal login mappings, private participant keys, wallet master keys, unwrapped data keys, and recovery secrets are durable local credential or custody state: they are intentionally not journal-rebuildable and Final Admission never reads them. Wallet loss therefore affects capability, not the recorded owner or grant state.

The exact ordered transition history is already committed by the Envelope Hash and Ledger Prefix Hash. The Network Profile also fixes a canonical Effective Privacy State serialization and digest for projection validation and reproducible equality checks; this derived digest is separate from ADR 0024's Public Provenance Post-State Commitment and is not a new signed consensus commitment under this decision. A snapshot is only a discardable accelerator and must bind ledger position, Ledger Prefix Hash, schema/reducer version, and this digest.

All healthy nodes at the same Ledger Prefix Hash must derive byte-identical Effective Privacy State and digest without depending on aliases, wallets, secrets, decryption success, insertion order, clocks, or cache layout. A behind node is merely behind. A same-prefix/different-digest node is degraded or divergent and cannot issue a Commit Receipt or serve authoritative privacy reads.

## Rationale

The current system splits privacy behavior across JWT claims, request fields, local wallet shared secrets, benchmark policy rules, and block metadata. None provides a durable, three-node-replayable answer to who owns a protected object or who is currently granted access.

Putting control transitions through Final Admission makes privacy changes atomic with the same durable history used for blocks. It prevents a crash or rejected candidate from leaving a grant in one local map but absent from the journal, and it gives restart and catch-up one exact history from which to derive the same authorization state.

Keeping secret custody outside the journal preserves the boundary from ADR 0027: the network agrees on identities and authorization state without publishing the secrets that exercise decryption capability.

## Consequences

- `PrivacyControlV1` needs a canonical schema, domain separator, closed transition tag set, deterministic identifiers and preconditions, and type-specific Final Admission dispatcher;
- the reference format permits exactly one control transition per envelope and forbids public RDF in the control kind;
- participant authorization must resolve a purpose-bound key against verified parent Effective Privacy State, while bootstrap governance must resolve through its active Network Profile binding; neither accepts a self-contained authorizing public key;
- protected-object creation must atomically preserve the exact Protected Payload Ciphertext, owner binding, both commitments, and Owner DEK Envelope in the complete Admitted Block Envelope;
- privacy-control envelopes must preserve exact non-secret transition and opaque encrypted bytes across replication, restart, and catch-up;
- the effective privacy projection must be position-addressable, deterministic, idempotent, and rebuilt only from verified journal history;
- stale or unavailable privacy projections deny access or admission instead of falling back to JWT role, wallet contents, or request fields;
- private wallet custody and backup remain durable local responsibilities but cannot alter owner or grant truth;
- control metadata is visible to permissioned nodes, so hiding access relationships is not a thesis-reference claim;
- tests must cover valid and invalid transition schemas, unknown operations and fields, wrong kind, duplicate identifiers, failed prior-state preconditions, unknown principals/keys/objects, wrong-purpose keys, unauthorized signatures, rejected-transition non-mutation, projection staleness, and walletless follower validation;
- crash tests must cover failure immediately before and after the sole journal commit point and prove identical restart state;
- three-node tests must show identical Effective Privacy State for an identical Ledger Prefix Hash, node-local denial after a committed Privacy Grant revocation, historical-query non-bypass, and network-wide denial only after the revocation becomes Network-Converged;
- projection-loss and snapshot tests must compare canonical privacy-state digests, while same-prefix/different-digest nodes fail closed; and
- corruption tests must prove replay stops at the last verified position without silently dropping or repairing a committed privacy transition.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the complete durable envelope and single append-plus-`fsync` commit point.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) defines the atomic reject-or-commit transition used here.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) requires exact-envelope replication and matching ledger prefixes across the three reference nodes.
- [ADR 0026](./0026-validate-the-full-staged-union-and-require-candidate-focus.md) requires every non-ordinary Admission Kind to have an explicit fail-closed schema rather than a zero-focus bypass.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) defines the stable principals and purpose-bound key versions referenced by privacy control.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) fixes the closed governance-bootstrap and participant-authorization matrix.
- [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) fixes atomic principal/key creation and its composite bootstrap proof.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) fixes the total participant-key lifecycle reducer and variant-specific bind possession evidence.
- [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) fixes the grant reducer, prospective effect, historical non-bypass, and converged live-release boundary.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) fixes the single-ciphertext, per-object-DEK, owner-envelope, and grant-envelope topology.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the mandatory protected-data suite and closed canonical privacy encoding.

## Implementation Status

Accepted architecture; ADR 0034 fixes the exact suite and codec, but implementation, Network Profile activation, and conformance evidence remain pending. Current privacy lookup uses a node-local unversioned `key_id -> shared secret` map. The benchmark policy endpoint trusts request-supplied `actor_org` and `owner_org` strings plus a magic auditor name, and no product path records owner, grantee, grant, revoke, key status, or policy position in the ledger. Persistent block metadata records only whether encrypted data existed and reconstructs an empty encrypted payload after restart. `tests/privacy_test.rs` proves only in-memory correct-key and wrong-key behavior; other wallet and handler tests do not exercise ledger-authoritative privacy control. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
