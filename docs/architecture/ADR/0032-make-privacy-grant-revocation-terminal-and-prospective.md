# ADR 0032: Make Privacy Grant Revocation Terminal and Prospective

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Durable owner-to-grantee authority that ends predictably without claiming cryptographic recall
**Envelope structure clarified by:** [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — Grant Delivery Material contains exactly one grant-specific DEK envelope, built without a Grant Identifier cycle.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the concrete wrapping algorithm, canonical encoding, recipient-key eligibility, deterministic bounds, and Public Envelope Validation rules are fixed; implementation, Network Profile activation, and conformance evidence remain pending.

---

## Decision

A **Privacy Grant** is an immutable, uniquely identified, ledger-authoritative authorization from the owner of one Protected Object to one grantee Participant Principal. An Active grant is a necessary grant-authority condition for current network-mediated access under the protocol; it is not independently sufficient for release or decryption. A Grant DEK Envelope combined with its matching private key may instead supply the cryptographic ability to recover the object's DEK. Neither substitutes for the other.

`PrivacyControlV1` uses exactly two grant lifecycle statuses: `Active` and terminal `Revoked`. Status and effective ledger position are reducer-derived and never caller fields. There is no pending, suspended, expired, deleted, reactivated, partially revoked, or wall-clock-driven grant state. Grant identity, object, owner, grantee, permission, creation position, and any bound Grant Delivery Material are immutable after creation.

For each `(Protected Object identifier, grantee Participant Principal UUID)` pair recorded in Effective Privacy State, there is at most one `Active` Privacy Grant. `GrantAccess` creates exactly one Active grant. A never-granted pair has no record; after revocation the pair may have zero Active grants while retaining one or more terminal historical records. A later grant to the same pair is permitted only through a fresh `GrantAccess` with a fresh Grant Identifier. It never reactivates, overwrites, deletes, or reuses an earlier grant.

The only v1 grant permission is the closed full-object permission `ReadProtectedObjectV1`. It does not authorize object mutation, ownership transfer, onward delegation, grant creation, grant revocation, partial fields, derived scopes, or time-bounded access. Any later permission or scope requires a versioned schema decision rather than a free-form string.

`GrantAccess` requires an existing Protected Object and an existing, distinct grantee Participant Principal. Final Admission derives the immutable object owner from parent Effective Privacy State; the transition cannot carry a caller-selected owner. The owner must authorize the transition using the exact Active parent-state `privacy-authorization` Participant Key Version required by ADR 0029. Owner self-grant, grantee signature substitution, administrator or validator proof, recipient proof of possession, unknown object or grantee, an already-Active object/grantee pair, or a reused Grant Identifier rejects.

The closed canonical `GrantAccess` core binds:

- the `PrivacyControlV1`, transition-schema, and `GrantAccess` variant identifiers;
- canonical network identity and exact parent Network Profile identity and content hash;
- expected ledger position and privacy revision, parent Ledger Prefix Hash, and parent envelope identity;
- the exact Protected Object identifier;
- the exact parent-derived owner Participant Principal UUID;
- the grantee Participant Principal UUID;
- permission exactly `ReadProtectedObjectV1`;
- one versioned, bounded, canonical Grant Delivery Material field `D` whose entire value is exactly one ADR 0033 Grant DEK Envelope, with every required public recipient-key reference inside that envelope's canonical header; and
- authorizer kind exactly participant plus the exact parent-state owner `privacy-authorization` key reference.

Grant Delivery Material is cryptographic material, not grant authority or proof of successful delivery or decryptability. ADR 0033 fixes its topology, its acyclic pre-`D` Delivery Context Digest `Q`, and its one exact Grant DEK Envelope. ADR 0034 fixes the mandatory `ProtectedDataSuiteV1`, including the concrete wrapping algorithm, canonical encoding, recipient-key eligibility, deterministic bounds, and Public Envelope Validation rules. `GrantAccess` and `PrivacyControlV1` still fail closed until the conforming implementation and evidence exist and an exact Network Profile activates them.

Let `C_grant` be the profile-pinned canonical length-delimited bytes of that closed unsigned core, excluding the Grant Identifier, Participant Authorization Signature, Proposal Digest, proposer signature, Envelope Hash, and Commit Receipts. Let:

```text
G = SHA-256("provchain/privacy-grant/grant-id/v1" || C_grant)
A_grant = SHA-256("provchain/privacy-grant/owner-authorization/v1" || G || C_grant)
```

The raw 32 bytes of `G` are both the Grant Identifier and the unique `GrantAccess` transition identifier. The resolved owner authorization key signs the raw 32 bytes of `A_grant`. The complete transition adds `G` and the exact Participant Authorization Signature before the Scheduled Authority constructs the Proposal Digest and, later, the Envelope Hash. ADR 0033 constructs the Grant DEK Envelope against pre-`D` digest `Q` before this formula derives `G`, avoiding a cycle. Because parent anchors and all grant content enter `C_grant`, a later regrant necessarily derives a new identifier, and proof transplantation across an object, grantee, delivery field, parent, profile, or network fails.

`RevokeGrant` is the sole grant-lifecycle operation after creation. It names exactly one Grant Identifier whose parent-state status is `Active`; pair-only targeting is forbidden. Final Admission resolves the grant, object, grantee, immutable owner, and owner authorization key from the verified parent. The owner alone supplies the Participant Authorization Signature. Target status `Revoked` is implied by the variant and is not a caller field.

Let `C_grant_revoke` be the profile-pinned canonical length-delimited bytes containing the protocol, transition-schema, and `RevokeGrant` variant identifiers; network identity and exact parent Network Profile identity and content hash; expected ledger position and privacy revision; parent Ledger Prefix Hash and parent envelope identity; exact Grant Identifier; the parent-derived object, grantee, owner, permission, and creation position; authorizer kind exactly participant; and the exact Active parent-state owner authorization-key reference. It excludes the revocation transition identifier, Participant Authorization Signature, Proposal Digest, proposer signature, Envelope Hash, and Commit Receipts. Let:

```text
T_grant_revoke = SHA-256("provchain/privacy-grant/revoke-transition-id/v1" || C_grant_revoke)
A_grant_revoke = SHA-256("provchain/privacy-grant/revoke-owner-authorization/v1" || T_grant_revoke || C_grant_revoke)
```

The owner key signs the raw 32 bytes of `A_grant_revoke`. The complete transition adds `T_grant_revoke` and the exact signature before PoA Proposal Digest and Envelope Hash construction. A caller-supplied owner, grantee, status, reason, timestamp, expiry, replacement grant, arbitrary scope, metadata, extra proof, or delivery-byte mutation rejects.

The only legal grant status edge is `Active -> Revoked`. `Revoked` is terminal. Double revocation, revocation of an unknown or non-Active grant, reactivation, deletion, in-place replacement, or reuse of the old Grant Identifier rejects with no mutation. A successful revocation changes only that grant's prospective authority. It never changes object ownership, participant-key state, the Protected Payload Ciphertext, its commitments, Owner DEK Envelope, Grant DEK Envelope, or any earlier journal bytes.

Participant-key lifecycle and grant lifecycle are orthogonal. Rotating or revoking an owner or grantee key never silently activates, retires, or revokes a Privacy Grant. If wrapping-key material changes, the existing grant and its bound Grant DEK Envelope remain immutable; ADR 0034 fixes the separate conjunctive `ProtectedDataSuiteV1` rule for whether a Live Privacy Release accepts the recorded key's current lifecycle status. Producing an envelope for another key requires an explicit owner-authorized operation rather than projection mutation. If an owner enters Participant-Control Freeze while grants remain Active, those grants become stranded and unrevocable in v1 because no governance, administrator, validator, node, or grantee override exists.

Grant revocation is **prospective authorization**, not cryptographic erasure. A conforming ProvChain service must not make a new **Live Privacy Release** for a grantee unless the exact grant is Active at the release decision's verified current prefix and every `ProtectedDataSuiteV1` condition also passes. ADR 0033 narrows the bounded-v1 output to the exact Protected Payload Ciphertext plus that grant's exact Grant DEK Envelope for client-side decryption; plaintext, an unwrapped DEK, a private key, server-side decryption, and decryption-oracle output remain forbidden.

An explicitly historical Effective Privacy State is audit-only. It may truthfully report that a grant was Active before revocation, but it can never authorize a Live Privacy Release after the current state has revoked that grant. A historical-position parameter, old snapshot, cached ACL, retained wallet secret, API role, `view_all`, caller-supplied owner/grantee value, or possession of old public evidence cannot make the managed release boundary evaluate an earlier grant state.

For the bounded three-node reference system, every Live Privacy Release decision is linearized at one exact **Network-Converged** ledger prefix. The release boundary must establish a three-node read barrier with matching ledger position, Ledger Prefix Hash, canonical Effective Privacy State digest, and required Commit Receipts, and must hold that grant-state prefix stable relative to Final Admission until the allow-or-deny decision and exact response material are fixed. An equivalent mechanism is acceptable only if reproducible concurrency evidence proves the same no-check-then-release-race property. The release result binds the evaluated prefix and Grant Identifier without writing secrets to logs or audit metadata.

A stale, behind, partially replayed, degraded, divergent, unsynchronized, or partitioned release service cannot establish that barrier and fails closed without releasing protected material. Node-local denial begins when that node commits `RevokeGrant` at journal append plus `fsync`; network-wide denial is claimed only after the identical revocation envelope becomes Network-Converged under ADR 0023. A release decision linearized before the revocation may complete afterward and cannot be recalled; a decision linearized after the revocation commit or during an unresolved convergence window must deny.

The Ledger Journal remains immutable. Exact Protected Payload Ciphertext and DEK Envelope bytes already committed remain available for Verified Journal Replay and permissioned-node integrity verification. This decision cannot recall plaintext, a Protected-Object DEK, a private key, a DEK Envelope, or other access-enabling material already obtained before revocation, and it cannot prevent offline cryptographic use by a party that already possesses sufficient material. If a grantee also operates a permissioned node and holds the matching private key, replication itself may already have delivered the opaque Grant DEK Envelope. The thesis claim is therefore durable, prospective protocol authorization and managed-release denial, never retroactive confidentiality or deletion.

Stronger cryptographic cut-off for later content requires a new Protected Object or version with fresh plaintext encryption, a fresh Protected-Object DEK, a fresh Owner DEK Envelope, and fresh Grant DEK Envelopes for every remaining grantee. `RevokeGrant` does not perform that work implicitly, and the bounded v1 protocol has no automatic re-encryption, envelope replacement, expiry, delegation, grantee-initiated revocation, administrator override, or cryptographic-recall path.

Concurrent transitions serialize through the exact parent Ledger Prefix Hash and expected privacy revision. Two grants for the same pair, grant versus revoke, two revocations, grant re-creation, owner-key self-revocation versus grant revocation, and reused-identifier candidates built from one parent cannot both commit. After the first append, every conflicting candidate has a stale parent or failed parent-state precondition and rejects without mutation.

Final Admission stages the complete grant or revocation delta only after all schema, canonicalization, profile, parent, authorizer, object, principal, uniqueness, lifecycle, Grant Delivery Material, bound, and PoA checks pass. Failure before journal `fsync` commits no grant change and releases no protected material. Once `fsync` succeeds, the whole delta is committed even if the Active-pair index, Effective Privacy State projection, Oxigraph view, or another cache fails; the node becomes degraded, serves no authoritative privacy release, and rebuilds from Verified Journal Replay.

Replay verifies each historical owner signature, object and grantee reference, Grant Identifier, Grant Delivery Material, and reducer precondition against that transition's recorded parent and profile. It preserves historically Active views for audit, reconstructs the same current Active-pair index and privacy-state digest, and never releases or decrypts protected material. Later revocation does not invalidate the historical `GrantAccess` signature; it terminates only current and later managed authority.

## Rationale

An immutable grant record plus terminal revocation gives every node a deterministic answer at each ledger position and preserves the evidence needed to explain who authorized access and when it ended. A fresh identifier for regranting avoids rewriting or ambiguously reactivating history.

Separating grant authority from cryptographic ability prevents two opposite errors: possession of wrapped material does not create permission, and an Active grant does not prove that the grantee possesses usable private material. Keeping key and grant lifecycles orthogonal also prevents routine key rotation from silently changing owner intent.

The converged release barrier trades availability and latency for a defensible revocation boundary. It makes stale-node and check-then-release races observable and fail closed in the three-node reference campaign without claiming that an asynchronous operational deployment already has production-grade global revocation.

## Consequences

- Effective Privacy State needs immutable grant records, a closed `Active`/`Revoked` status, a globally unique Grant Identifier index, and an at-most-one-Active `(object, grantee)` index;
- participant-facing historical queries and integrity replay must be separated from the sole Live Privacy Release boundary;
- the three-node reference service needs a reproducible converged read barrier or equivalent linearizable mechanism and must prefer denial to stale or partitioned release;
- ADR 0034 instantiates ADR 0033's fixed `D` topology with concrete algorithms, canonical encodings, recipient-key eligibility, public validation, and deterministic bounds, while implementation, Network Profile activation, and conformance evidence remain required before `GrantAccess` or `PrivacyControlV1` can activate;
- clients must enumerate and warn about Active grants before owner authorization-key self-revocation, because Participant-Control Freeze can strand those grants permanently in v1;
- fixtures must cover canonical grant/revoke golden vectors, cross-domain and proof transplantation, wrong owner or grantee, admin/node/PoA substitution, unknown object/principal, owner self-grant, duplicate Active pair, global identifier reuse, fresh-ID regrant, terminal double revocation, and rejected-transition non-mutation;
- live-release fixtures must cover historical-audit visibility without protected-material release, cached/historical-state bypass attempts, stale/degraded/partitioned nodes, grant-check/revoke races on both sides of the linearization point, pre-revocation no-recall, and absence of secrets from release evidence;
- crash fixtures must cover failures before and after journal `fsync`, partial grant-index/projection failure, restart, and exact reconstruction without release; and
- three-node evidence must show identical grant records, statuses, Active-pair indexes, Effective Privacy State digests, node-local denial after commit, and network-wide denial only after convergence.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) preserves exact privacy envelopes and fixes append plus `fsync` as the sole commit point.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes grant and revocation transitions reject-or-commit atomically.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) defines exact-envelope replication, Commit Receipts, and the network-wide convergence threshold.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) defines the owner and grantee identities used by grants.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) defines the journal-authoritative privacy transition and projection model.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) requires the parent-state object owner to authorize both grant and revocation.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) defines the independent key lifecycle and Participant-Control Freeze.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) fixes one Grant DEK Envelope per grant and its acyclic delivery context.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the concrete suite, canonical codec, and recipient-key eligibility used by grant delivery and live release.

## Implementation Status

Accepted architecture; ADR 0034 fixes the exact suite and codec, but implementation, Network Profile activation, and conformance evidence remain pending. No `PrivacyControlV1`, Protected Object, Privacy Grant, `GrantAccess`, `RevokeGrant`, Effective Privacy State, converged release barrier, or grant-aware read path exists. `src/security/encryption.rs` stores only ciphertext, nonce, and a caller-selected `key_id`; `src/wallet.rs` authorizes possession through an unversioned local `key_id -> secret` map. The encrypted query path in `src/web/handlers/query.rs` decrypts whenever a JWT subject parses as a wallet UUID and that wallet contains the named secret, without consulting a grant, revocation, current privacy prefix, or three-node convergence. The benchmark policy path trusts caller-supplied owner/actor strings and a magic auditor name. Persistent block metadata records only an encrypted-data boolean and reconstructs empty encrypted payload bytes after restart. `tests/privacy_test.rs` proves only correct-key success and wrong-key failure; other wallet and handler tests do not exercise grants or revocation. No current code or test is evidence for this decision, and no implementation code changed with it.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
