# ADR 0031: Use At Most One Active Participant Key per Purpose

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Deterministic participant-key rotation, emergency revocation, and historical verification
**Envelope integration clarified by:** [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — the schema-known `privacy-key-wrapping` purpose receives per-object DEKs and remains separate from participant authorization.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the wrapping-key algorithm and encoding, fingerprint, public possession-proof verifier, and canonical privacy encoding are fixed; implementation, Network Profile activation, and conformance evidence remain pending.
**Custody boundary fixed by:** [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) — exact Active and Retired private versions remain in durable client-only custody, local storage never determines lifecycle status, and Revoked material is never usable or restorable as authority.
**Keystore suite fixed by:** [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) — the exact retained-key records, Revoked tombstones, reconciliation, and whole-snapshot rewrite design are fixed; implementation, activation, and conformance evidence remain pending.

---

## Decision

`PrivacyControlV1` uses a closed, ledger-derived lifecycle for every Participant Key Version. The only lifecycle statuses are `Active`, `Retired`, and `Revoked`. Status and effective ledger position are reducer-derived state, not caller-selected metadata. `Pending`, `Suspended`, `Expired`, deleted, reactivated, and locally disabled states do not exist in this protocol version, and local wall-clock time never changes network-authoritative key eligibility.

For each `(Participant Principal UUID, key purpose)` pair recorded in Effective Privacy State, there is **at most one** `Active` key. Immediately after a successful bootstrap or binding for that purpose, exactly one version is `Active`. Once a purpose has been bound, zero active versions is permitted only after emergency revocation of that purpose's active version; a never-bound purpose has no recorded pair. `RegisterPrincipal` remains the only way to create `privacy-authorization` version 1 and creates it as `Active` under ADR 0030.

Participant-key versions are unsigned 32-bit integers in the range `1..=u32::MAX`, scoped independently to one principal and purpose. They are strictly contiguous and immutable. `BindParticipantKey` must request version 1 for a never-before-bound non-authorization purpose, or exactly `n + 1`, where `n` is the greatest version already recorded for that principal and purpose. Zero, a gap, reuse, rollback, duplicate version, and overflow reject. A key record is never overwritten or deleted.

The key-purpose namespace is closed by the `PrivacyControlV1` schema, and the exact parent Network Profile selects only schema-known purposes. Each enabled purpose must pin its allowed public-key algorithm and encoding, fingerprint derivation, proof-of-possession scheme and verifier, canonical proof encoding, and deterministic key/proof bounds. `privacy-authorization` is mandatory and uses the profile-pinned Ed25519 scheme. ADR 0033 adds the schema-known `privacy-key-wrapping` purpose, and ADR 0034 fixes its concrete wrapping scheme and public verifier. Neither decision activates it: an unknown purpose, locally invented purpose, unsupported algorithm, or purpose without the complete deterministic verifier still makes profile activation or candidate admission fail closed, and activation remains pending the conforming implementation and evidence required by ADR 0034.

`BindParticipantKey` is the sole rotation and later-purpose-binding operation. Its closed canonical core binds:

- the protocol, transition-schema, and `BindParticipantKey` variant identifiers;
- canonical network identity and exact parent Network Profile identity and content hash;
- expected ledger position and privacy revision, parent Ledger Prefix Hash, and parent envelope identity;
- the affected Participant Principal UUID;
- the schema-known, profile-enabled purpose and required contiguous version;
- exact algorithm and proof-scheme identifiers, canonical public-key bytes, and verifier-derived canonical fingerprint; and
- the exact parent-state active `privacy-authorization` authorizer key reference.

Caller-supplied current status, target status, effective position, predecessor status, timestamps, reasons, account fields, wallet locators, or arbitrary metadata are forbidden. The reducer derives whether the purpose is new, the greatest recorded version, and any current active predecessor from verified parent Effective Privacy State. Exact public-key bytes or a canonical fingerprint already bound to any participant principal, participant purpose, or Network Profile key role cannot be reused.

Every `BindParticipantKey` has exactly one Participant Authorization Signature made by the affected principal's active parent-state `privacy-authorization` key. That signature is the sole transition authorization. The introduced key additionally supplies exactly one separately tagged **Participant Key Possession Proof** under the target purpose's parent-profile scheme. This proof is required operation evidence, not a second Transition Authorization Proof and never authorizes its own binding. Missing, duplicate, additional, wrong-purpose, wrong-scheme, or invalid possession evidence rejects.

The two bind proofs use an acyclic, domain-separated construction. Let `C` be the profile-pinned canonical length-delimited bytes of the closed unsigned bind core, excluding the transition identifier, authorization signature, possession proof bytes, Proposal Digest, proposer signature, Envelope Hash, and Commit Receipts. Let:

```text
T = SHA-256("provchain/privacy-key-bind/transition-id/v1" || C)
A = SHA-256("provchain/privacy-key-bind/participant-authorization/v1" || T || C)
P = SHA-256("provchain/privacy-key-bind/key-possession/v1" || T || C)
```

The parent-state active `privacy-authorization` key signs the raw 32 bytes of `A`. The introduced key's profile-pinned proof scheme proves possession over the raw 32 bytes of `P`; for an Ed25519 signing key this is an Ed25519 signature, while a future non-signing purpose must define its own deterministic public verifier before activation. The complete canonical transition then adds `T`, the exact Participant Authorization Signature, and the exact possession-proof encoding. Only afterward are the PoA Proposal Digest and Envelope Hash constructed. Neither proof contains the other, so no signature cycle exists, while the shared `C` and `T` prevent cross-principal, cross-purpose, cross-profile, or cross-parent proof transplantation.

For a purpose with one parent-state `Active` key, successful `BindParticipantKey(n + 1)` stages one indivisible reducer delta: the predecessor becomes `Retired` and the new version becomes `Active`. `Retired` is created only by that atomic rotation and is terminal. It cannot be supplied directly, revoked later, reactivated, or deleted. A projection must never expose both the old and new version as active or expose a committed half-rotation.

For a non-authorization purpose whose greatest version is `Revoked` and which therefore has no active version, the affected principal's still-active `privacy-authorization` key may authorize `BindParticipantKey(n + 1)`. The revoked predecessor remains `Revoked`; no retirement is synthesized, and the new contiguous version becomes `Active`. This restores only that non-authorization purpose. It does not alter any historical signature, grant, Protected Payload Ciphertext, DEK Envelope, or object ownership.

`RevokeParticipantKey` is the sole emergency lifecycle operation; there is no generic caller-facing status editor. Its closed canonical core uses the same network, profile, parent, expected-position, expected-revision, affected-principal, and exact active authorizer anchors and names exactly one target `(principal, purpose, version)`. Target status `Revoked` is implied by the variant and is not a caller field. The target must be the affected principal's parent-state `Active` version for that purpose. The active parent-state `privacy-authorization` key supplies the sole Participant Authorization Signature. A `privacy-authorization` key may authorize its own revocation because eligibility is evaluated at the verified parent before the transition takes effect.

The revocation transition has its own acyclic canonical construction. Let `C_revoke` be the profile-pinned canonical length-delimited bytes containing the protocol, transition-schema, and `RevokeParticipantKey` variant identifiers; network identity and exact parent Network Profile identity and content hash; expected ledger position and privacy revision; parent Ledger Prefix Hash and parent envelope identity; affected principal; exact target principal, purpose, and version; authorizer kind exactly participant; and the exact parent-state active `privacy-authorization` authorizer key reference. The variant itself fixes the target state as `Revoked`. `C_revoke` excludes the transition identifier, authorization signature, Proposal Digest, proposer signature, Envelope Hash, and Commit Receipts. Let:

```text
T_revoke = SHA-256("provchain/privacy-key-revoke/transition-id/v1" || C_revoke)
A_revoke = SHA-256("provchain/privacy-key-revoke/participant-authorization/v1" || T_revoke || C_revoke)
```

The resolved parent-state authorization key signs the raw 32 bytes of `A_revoke`. The complete canonical transition then adds `T_revoke` and the exact Participant Authorization Signature before the Scheduled Authority constructs the Proposal Digest and, later, the Envelope Hash. The fixed domain strings and common length-delimited encoding separate this proof from bind, bootstrap, possession, and PoA proofs and prevent cross-operation, cross-key, cross-profile, or cross-parent transplantation without creating a signature cycle.

The only legal status transition is `Active -> Revoked`. `Revoked` is terminal. There is no direct transition to `Retired`, no `Retired -> Revoked`, no `Revoked -> Active`, no deletion, and no status-only replacement. Revoking the sole active `privacy-authorization` key leaves that purpose with zero active versions and permanently freezes every later participant-controlled transition for that principal in v1, including binding a replacement authorization key. Bootstrap governance, PoA validators, Network Members, node keys, JWT roles, API administrators, grantees, and the participant client custody store cannot recover or override the freeze. Recovery is future work requiring a new explicit protocol decision.

Revoking another purpose makes that key version permanently ineligible as network-authoritative evidence for prospective actions. The purpose has no eligible version until the still-authorized participant successfully binds its next contiguous version, and only that new version becomes `Active`; the revoked predecessor remains `Revoked`. This does not disable the principal's other purposes and does not itself revoke object ownership or Privacy Grants. ADR 0032 fixes the separate prospective grant effect and forbids historical key or grant state from authorizing a Live Privacy Release.

Retirement and revocation are nonretroactive. Final Admission and Verified Journal Replay evaluate every recorded signature and possession proof against the key status, purpose, algorithm, Network Profile, and parent state recorded for that transition. A proof valid at its historical parent remains historically valid after its signer is retired or revoked. The same old key is ineligible to authorize a new candidate after the lifecycle change commits. Historical Privacy Grant, Protected Payload Ciphertext, and DEK Envelope bytes remain immutable journal facts, and this protocol makes no claim that an already disclosed private key, Protected-Object DEK, DEK Envelope, or plaintext can be recalled.

Concurrent operations are serialized by the exact parent Ledger Prefix Hash, expected privacy revision, and transition identifier. If two binds, or a bind and revoke, are built from the same parent, at most the first successfully appended envelope can commit; every conflicting candidate then has a stale parent or revision and rejects without mutation. No wall-clock ordering or local last-write-wins rule is permitted.

Final Admission stages the complete lifecycle delta in memory only after all canonical-schema, parent/profile, authorizer, possession, uniqueness, version, status, bounds, and PoA gates pass. Failure before the Ledger Journal append-plus-`fsync` commit point changes no authoritative state or projection. Once `fsync` succeeds, the complete transition and its whole lifecycle delta are committed even if an in-memory registry, Oxigraph view, or index update fails; the node becomes degraded and reconstructs those projections through Verified Journal Replay.

Against a healthy verified parent, malformed or noncanonical fields, unknown purpose/status/proof kinds, version gap or overflow, duplicate or globally reused public material, wrong algorithm or fingerprint, inactive or wrong-purpose authorizer, introduced-key self-authorization, missing or invalid possession evidence, illegal status edge, stale parent/revision, or invalid PoA evidence is a deterministic rejection. Local verifier, storage, timeout, memory, or resource failure is node incapacity and yields no append, Commit Receipt, or global invalid-candidate verdict.

## Rationale

At-most-one active version gives every node one deterministic answer when it resolves a purpose at a ledger position. Atomic rotation prevents a crash-visible interval with two active keys or no replacement, while preserving every historical reference.

Separating emergency revocation from routine rotation keeps their meanings narrow: rotation supersedes a usable key, while revocation immediately disables a currently usable key and may intentionally leave zero active versions. Allowing a later bind only for a non-authorization purpose preserves recoverability where the participant still has its sole authorization capability without creating a governance or administrator back door after authorization-key loss.

Separating the introduced key's possession evidence from the parent's authorization signature prevents a new key from authorizing itself. Profile-pinned purpose-specific verification keeps later non-signing key types possible without pretending that every proof of possession is an Ed25519 signature.

## Consequences

- Effective Privacy State needs a total reducer and canonical digest representation for the three statuses, strict per-purpose versions, and at-most-one-active invariant;
- the profile and transition schemas need closed purpose, algorithm, proof-scheme, status, key-reference, and proof-evidence encodings with deterministic bounds;
- participant clients need durable purpose- and version-aware custody under ADR 0035, including exact Active and Retired private versions, but custody state remains non-authoritative and unavailable to follower validation;
- no account, client custody store, wall-clock monitor, global blockchain signer, API role, or node configuration may retire, revoke, reactivate, or select a Participant Key Version outside Final Admission;
- authorization-key self-revocation is deliberately irreversible in v1, so clients must enumerate any active grants and warn clearly that the owner may become unable to revoke them before constructing it, even though UI confirmation is not ledger evidence;
- fixtures must cover canonical bind and revoke golden vectors; cross-domain and proof-transplant rejection; new-purpose version 1; contiguous rotation; gaps, duplicates, and overflow; missing or invalid possession proof; wrong-purpose proof; one-active and zero-active states; atomic predecessor retirement; post-revocation non-authorization rebinding; authorization self-revocation freeze; every forbidden status edge; global key reuse; and rejected-candidate non-mutation;
- concurrency and crash fixtures must cover same-parent competing binds/revokes, failures immediately before and after journal `fsync`, partial projection failure, restart, and exact lifecycle reconstruction;
- historical fixtures must prove an old key valid at its recorded parent, ineligible for a new post-commit transition, and still verifiable after current-tip retirement or revocation; and
- three-node evidence must show walletless followers deriving identical key records, lifecycle statuses, active-key resolution, and Effective Privacy State digests from identical journal prefixes.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the append-plus-`fsync` commit point and rebuildable projections.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) requires the full lifecycle reducer delta to pass one fail-closed admission boundary.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) defines stable principals and immutable purpose-bound key versions.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) defines the journal-authoritative transition and Effective Privacy State model.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) makes the affected principal's active authorization key the sole later authorizer.
- [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) creates the initial active authorization key atomically with the principal.
- [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) keeps grant status orthogonal to key status and defines the consequence of Participant-Control Freeze for Active grants.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) makes Active wrapping-key references prerequisites for new object and grant envelopes.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the wrapping-key algorithm, public possession proof, and canonical privacy encoding used by this lifecycle.
- [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) fixes non-authoritative client retention and operation gating for Active, Retired, and Revoked private-key records.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) fixes exact retained-key records, public Revoked tombstones, and atomic custody rewrites while leaving lifecycle authority in the ledger.

## Implementation Status

Accepted architecture; ADR 0034 fixes the protected-data suite and codec, ADR 0035 fixes client-only custody semantics, and ADR 0036 fixes the exact at-rest keystore and rewrite design, but implementation, Network Profile activation, and conformance evidence remain pending. `src/wallet.rs` stores one generic unversioned Ed25519 signing key and a free-form `key_id -> secret` map in server-side legacy scaffolding rather than a conforming participant client. Transaction signatures in `src/transaction/transaction.rs` carry their own public key beside a claimed participant UUID instead of resolving a purpose, version, and parent-state lifecycle status, and `src/web/handlers/transaction.rs` signs through one server-side wallet key. The existing key-rotation monitor and tests concern a wall-clock policy for the global blockchain signer; the corresponding rotation path in `src/core/blockchain.rs` is unimplemented. No participant-key lifecycle, purpose-specific possession proof, atomic rotation, emergency revocation, client custody, replay, crash, or three-node convergence evidence exists. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
