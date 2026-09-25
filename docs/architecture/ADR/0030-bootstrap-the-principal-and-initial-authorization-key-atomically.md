# ADR 0030: Bootstrap the Principal and Initial Authorization Key Atomically

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Complete, non-self-authorizing creation of a Participant Principal
**Lifecycle clarified by:** [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) — the bootstrap key is the first active lifecycle version and later versions use atomic rotation or terminal emergency revocation.
**Canonical encoding fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the closed canonical `PrivacyControlV1` encoding used by `RegisterPrincipal` is fixed without changing this ADR's Ed25519 bootstrap-proof semantics; implementation, Network Profile activation, and conformance evidence remain pending.

---

## Decision

`RegisterPrincipal` is the sole participant-bootstrap transition in `PrivacyControlV1`. It atomically creates one previously unknown Participant Principal and binds and activates that principal's initial `privacy-authorization` Participant Key Version 1. The principal and key become effective together only when the complete Admitted Block Envelope reaches the Ledger Journal append-plus-`fsync` commit point. No committed or replayed intermediate state may contain one without the other.

The active thesis-reference Network Profile contains exactly one versioned **Privacy Bootstrap Governance Key** binding. The binding declares the `privacy-bootstrap` purpose, Ed25519 algorithm and proof-scheme version, exact 32-byte public key, scheme-derived SHA-256 fingerprint, and active status. These fields are covered by the exact Network Profile identity and content hash used at the candidate's verified parent.

This key is a dedicated trust role and key binding. It is distinct from and cannot reuse public-key bytes assigned to the Governance Trust Root or Membership Manifest signer, a Node Identity Key, PoA validator key, Participant Key Version, JWT service key, or other profile key role. Final Admission resolves it only from the verified parent Network Profile. A candidate-supplied key, local node configuration, Membership Manifest role, current governance module, wallet, JWT, API role, or validator status cannot replace it. A missing, duplicate, malformed, inactive, wrong-algorithm, fingerprint-mismatched, or prohibited key-reuse binding makes Network Profile activation fail closed. In-place bootstrap-key rotation or recovery is outside the bounded reference system and requires a later governed Network Profile version.

The closed canonical `RegisterPrincipal` core contains only:

- the `PrivacyControlV1`, transition-schema, and `RegisterPrincipal` variant identifiers;
- canonical network identity and exact Network Profile identity and content hash;
- expected ledger position and privacy revision, parent Ledger Prefix Hash, and parent envelope identity or genesis marker;
- one canonical, non-nil new Participant Principal UUID;
- the initial key tuple: purpose exactly `privacy-authorization`, version exactly `1`, algorithm exactly Ed25519 under the profile scheme, exact 32-byte public key, and verifier-derived canonical fingerprint;
- authorizer kind exactly `ParticipantBootstrapAuthorizationV1`; and
- the exact profile-bound Privacy Bootstrap Governance Key reference.

Active status and effective ledger position are reducer-derived results, not caller fields. Username, display name, account or organization role, wallet or key locator, request timestamp, object, grant, ciphertext, plaintext, arbitrary metadata, and extension fields are forbidden. The initial public key and fingerprint must not already be bound to another Participant Principal or profile key role.

`ParticipantBootstrapAuthorizationV1` is one composite Transition Authorization Proof containing exactly two cryptographic proofs:

1. the profile-bound Privacy Bootstrap Governance Key signature, which is the sole authorization for registration; and
2. a **Participant Key Possession Proof** made by the introduced key, which proves control of the private counterpart but grants no authority and is not a Participant Authorization Signature.

The introduced key therefore never authorizes its own registration. Missing, duplicate, additional, swapped, or wrong-kind key references or signatures reject the complete transition. The bootstrap-governance signer and introduced key must be distinct.

The two signatures use an acyclic construction over the same unsigned registration core. Let `C` be the profile-pinned canonical length-delimited bytes of the closed `RegisterPrincipal` core, excluding the transition identifier, both signatures, Proposal Digest, proposer signature, Envelope Hash, and Commit Receipts. Let:

```text
T = SHA-256("provchain/privacy-bootstrap/transition-id/v1" || C)
G = SHA-256("provchain/privacy-bootstrap/governance-authorization/v1" || T || C)
P = SHA-256("provchain/privacy-bootstrap/key-possession/v1" || T || C)
```

The governance key signs the raw 32 bytes of `G`; the introduced key signs the raw 32 bytes of `P`. Fixed domain strings and every concatenated field use the profile-pinned length-delimited encoding, so no variable-length ambiguity is permitted. The complete canonical transition then adds `T`, the exact governance signature, and the exact possession signature. Only afterward does the Scheduled Authority construct and sign the Proposal Digest over those complete transition bytes; the Envelope Hash is computed only after proposer evidence exists. Neither bootstrap signature covers or contains the other signature, Proposal Digest, proposer signature, Envelope Hash, or receipt, so the dependency graph has no signature cycle.

Final Admission recomputes the key fingerprints, `C`, `T`, `G`, and `P`; verifies both Ed25519 signatures strictly; verifies every profile, parent, position, revision, uniqueness, schema, and protocol-bound precondition; and then stages one indivisible Effective Privacy State delta containing the principal and active key version 1. It performs no authoritative mutation before all gates succeed. `BindParticipantKey` cannot perform initial bootstrap: for the `privacy-authorization` purpose it applies only to later contiguous versions, and for a new non-authorization purpose it applies only after the principal already exists with its active authorization key.

Because `C` binds the network, exact profile and governance-key reference, parent anchors, expected position and revision, principal, and introduced key, the composite proof is valid only for that one bootstrap candidate at that parent. An exact uncommitted retry may be evaluated again only while the parent is unchanged. Exact redelivery of an already committed envelope is idempotent synchronization handling only under ADR 0023. A different parent, network, profile, governance binding, principal, or key invalidates the signatures. Reusing consumed `T` in another envelope rejects; recomputing a transition for an already registered principal or bound key rejects the reducer precondition.

Verified Journal Replay verifies the historical governance proof against the exact bootstrap-key binding in the Network Profile recorded for that parent and verifies the possession proof against the initial public key in the canonical core. It atomically reconstructs the same principal/key pair without readmission, current-profile substitution, wallet access, or private key material. Healthy nodes at the same Ledger Prefix Hash must derive the same pair and Effective Privacy State digest.

Against a verified healthy parent, malformed or noncanonical bytes, an unknown or extra field, nil or duplicate principal, duplicate key or fingerprint, wrong purpose/version/status/algorithm/key length, a supplied fingerprint mismatch, wrong proof kind/domain/count/reference, either missing or invalid signature, mismatched derived identifier, parent/profile/network mismatch, proof transplantation, prohibited key reuse, or invalid/out-of-turn PoA evidence is a deterministic candidate rejection. Rejection appends nothing and changes neither Effective Privacy State nor any projection.

Failure before journal `fsync` commits neither principal nor key. Once `fsync` succeeds, both are committed even if a projection update fails; the node then becomes degraded and rebuilds both from Verified Journal Replay. A local verifier, journal, storage, timeout, or resource failure yields node incapacity and no global verdict, append, or receipt. A corrupt or unreconstructable historical parent/profile binding is degraded, corrupt, or unsupported-ledger state rather than a verdict on the next candidate. Missing bootstrap-governance or participant private custody blocks local proof creation only and never blocks a walletless follower from validating the exact public proofs.

## Rationale

A two-transition bootstrap would durably expose a registered-but-unkeyed principal, require special rules for whether it may own or receive data, and risk stranding it if the second request never commits. It provides no benefit to the bounded reference system.

The governance signature resolves the initial no-parent-key authorization cycle, while the introduced-key possession proof prevents governance or a transcription error from binding a key whose private counterpart is not controlled. Treating them as one composite authorization-proof variant preserves the closed one-proof rule without mistaking possession for participant authority.

Binding a distinct governance key through the Network Profile makes the bootstrap trust source reproducible across the three nodes and prevents existing membership, validator, administrator, or prototype-governance mechanisms from becoming an accidental enrollment superuser.

## Consequences

- every committed Participant Principal has an active version-1 `privacy-authorization` key from its first visible ledger state;
- no API account, wallet record, standalone `BindParticipantKey`, or local database insert can create a privacy principal;
- profile activation and bootstrap admission need exact key-role separation, canonical fingerprinting, and strict Ed25519 verification;
- the composite proof and signature dependency order become part of the versioned protocol and reproducible evidence;
- bootstrap-key replacement, governance or administrator recovery after loss or revocation of the sole `privacy-authorization` capability, self-registration, batch registration, and participant metadata enrollment remain outside v1;
- fixtures must cover both-proof success; either-proof absence or corruption; proof mix-and-match; changed principal, key, parent, network, or profile; wrong governance/manifest/PoA/node/admin key; wrong purpose/version/status; duplicate principal/key/fingerprint/transition; later-bind substitution; and rejected-transition non-mutation;
- crash fixtures must cover failure immediately before and after journal `fsync`, post-commit projection failure, restart replay, and exact principal/key reconstruction; and
- three-node evidence must show walletless followers deriving an identical active principal/key binding and Effective Privacy State digest from the same committed envelope.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the single append-plus-`fsync` commit point used for the atomic pair.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) requires all bootstrap gates to complete before authoritative mutation.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) defines a distinct membership trust boundary that cannot substitute for this bootstrap key.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) defines the stable principal and versioned participant key.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) defines `RegisterPrincipal` as a ledger-authoritative privacy transition.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) limits governance authority to bootstrap and requires participant authorization afterward.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) defines the lifecycle followed by the initial active key and every later participant-key version.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the shared canonical privacy encoding while preserving the distinct Ed25519 bootstrap proof.

## Implementation Status

Accepted architecture; ADR 0034 fixes the shared canonical privacy encoding, but implementation, Network Profile activation, and conformance evidence remain pending. `NetworkProfile` currently contains only consensus and semantic sections and has no privacy-bootstrap governance binding. `TransactionPayload` has no `PrivacyControlV1` or `RegisterPrincipal` variant, generic transaction signatures carry their own public key and claimed UUID, and the existing governance module is validator voting rather than this bootstrap signer. Existing privacy tests exercise only encryption/decryption and do not prove composite authorization, atomic state, crash recovery, or three-node replay. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
