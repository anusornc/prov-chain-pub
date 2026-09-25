# ADR 0027: Use Participant Principals for Privacy Identity

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Stable, network-verifiable ownership and grantee identity across authentication and wallet-key changes
**Clarified by:** [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) — principal, key, ownership, grant, and revocation truth changes only through committed `PrivacyControlV1` transitions and journal-derived state.
**Further clarified by:** [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) — a principal and its initial authorization key are created together under governance authorization plus non-authorizing key-possession proof.
**Lifecycle clarified by:** [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) — versions are strictly contiguous per purpose, at most one is active, rotation retires atomically, and emergency revocation is terminal.
**Envelope role clarified by:** [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — `privacy-key-wrapping` is the dedicated recipient purpose for Owner and Grant DEK Envelopes and never substitutes for participant authorization.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the exact wrapping-key algorithm and encoding, public possession-proof verifier, mandatory `ProtectedDataSuiteV1`, and canonical privacy encoding are fixed; implementation, Network Profile activation, and conformance evidence remain pending.
**Custody boundary fixed by:** [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) — participant private keys remain in durable client-only, non-authoritative custody; nodes, services, and the journal never escrow them.
**Keystore suite fixed by:** [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) — the exact at-rest format, cryptographic construction, whole-snapshot commit, recovery, backup, and restore design are fixed; implementation, activation, and conformance evidence remain pending.

---

## Decision

The sole privacy owner and grantee identity in the thesis-reference system is the **Participant Principal**, identified by one stable UUID. Privacy ownership, authorization grants, revocations, audit records, and protected-resource metadata refer to that UUID. A username, display name, role string, wallet object, public key, or free-form `key_id` is never a privacy principal.

An API account is a credential-bearing login mechanism that acts for a Participant Principal. Every JWT used to request a participant privacy action must carry the canonical Participant Principal UUID in `sub`. A username may remain a login alias, but token issuance must resolve it to the bound principal first. Each token names exactly one principal; account-to-principal cardinality outside that token does not alter the privacy model.

JWT authentication is an API-boundary credential, not replicated ledger evidence. Final Admission must not trust a username, JWT, or caller-supplied participant identifier as proof of an owner action. Every participant-controlled admission-relevant privacy action must bind the Participant Principal, an authorized **Participant Key Version**, and a Participant Authorization Signature that every node can verify independently at the candidate ledger position. The sole no-parent-key exception is the profile-bound Participant Bootstrap Authorization defined by ADRs 0029 and 0030; it atomically establishes a new principal and initial authorization-key binding and requires the introduced key's separate non-authorizing possession proof, but cannot authorize an owner, grant, revoke, or later key-lifecycle action.

A Participant Key Version is an immutable public-key binding identified by the tuple `(Participant Principal UUID, key purpose, version)`. Its network-visible binding includes the canonical principal, a closed purpose, a strictly contiguous positive version within that purpose, the algorithm/profile identifier, the public key material and its canonical fingerprint or commitment, and its lifecycle status. ADR 0031 fixes the exact `Active`, `Retired`, and `Revoked` reducer; a new key version never overwrites the identity or historical meaning of an earlier version.

Key purposes are explicit and non-interchangeable. In particular, participant authorization signing, privacy key agreement or key wrapping, wallet-at-rest encryption, JWT service signing, node identity, PoA block signing, and governance signing occupy separate roles or namespaces. Possession of one role's key does not authorize another role, and non-participant service or node keys are not Participant Key Versions. ADR 0033 names the closed `privacy-key-wrapping` purpose and fixes its envelope topology; ADR 0034 fixes its concrete algorithm, encoding, and public possession-proof verifier without activating the profile.

Private keys, unwrapped content-encryption keys, recovery secrets, and protected plaintext never enter the Ledger Journal, Public Provenance State, JWT claims, network-visible principal registry, node or service custody, API responses, logs, diagnostics, or plaintext backups. Participant private keys remain exclusively in the participant-controlled client boundary fixed by ADR 0035 and may be persisted only under authenticated at-rest protection. Client-store presence, unlock success, or backup state is non-authoritative; the network-verifiable binding contains only the public evidence needed to authenticate a key version. ADR 0036 fixes the exact `ParticipantKeystoreSuiteV1` at-rest format, KDF, AEAD, and whole-snapshot replacement contract; its implementation, activation, and conformance evidence remain pending.

Any ciphertext key identifier is a cryptographic lookup reference, not an owner or grantee identity. A canonical privacy key reference must resolve to a Participant Key Version; merely supplying that reference, a principal UUID, or a public key grants no authority. An opaque content-key identifier may identify encrypted material but cannot grant access by itself. Free-form `key_id` strings cannot establish ownership, grant membership, signer authorization, or key lifecycle status.

Account rename, password reset, token refresh, role change, client-custody relocation, and participant-key rotation do not change the Participant Principal or rewrite prior ownership. Disabling an account may prevent new API actions but does not erase the principal or its ledger history. Loss or deletion of client-held private material affects the principal's ability to sign or decrypt; it does not transfer or delete ownership.

API roles may restrict which endpoints a principal can call, but `Admin`, `Auditor`, or a local `view_all` flag does not create privacy ownership or a decryption grant. Such an actor must be represented by its own Participant Principal and receive whatever explicit authorization the privacy lifecycle permits.

Participant Principals are distinct from Network Members and node identities. A permissioned node may submit or relay an action for a participant, but Membership Manifest presence and Node Identity Key possession do not make that node the data owner. Conversely, a Participant Principal need not be a consensus authority.

The participant and public key bindings used for authorization must be durable and independently reproducible by every reference node. This decision fixes their identity semantics; ADR 0028 fixes the ledger-authoritative transition representation, while ADR 0035 keeps the corresponding private capabilities in a separate client-only store that is never network authority. Until those models are implemented, the current username store and wallet files do not constitute the required network authority or conforming custody.

Malformed or non-canonical principal identifiers, an unknown principal, a token/principal mismatch, an unknown or wrong-purpose key version, a signature/key mismatch, or an unprovable binding fails closed before any privacy or ledger mutation. An unavailable client custody store is participant-side incapacity, not node incapacity, network evidence, or proof that another principal owns the data.

## Rationale

The current API authenticates a username but privacy and transaction handlers parse the JWT subject as a participant UUID. Wallet registration independently generates another UUID and currently returns a simulated key. Without one canonical subject, an authenticated account cannot prove that it owns the wallet or private data it attempts to manage.

Treating keys as identities also makes ordinary rotation destructive: changing a key would appear to change the owner, while a leaked or deleted key could make ownership ambiguous. A stable Participant Principal preserves domain identity while immutable, purpose-bound key versions provide replaceable cryptographic evidence.

Separating API credentials, participant keys, and node keys keeps each trust statement narrow. A JWT proves an API login, a participant signature proves an owner or grantee action, and a Node Identity Key proves permissioned-network membership; none silently substitutes for another.

## Consequences

- the authentication user registry needs an explicit Participant Principal UUID binding and token generation must place that UUID, not username, in participant JWT subjects;
- wallet registration must create or attach to an authorized Participant Principal rather than mint an unrelated simulated identity;
- participant key records need canonical purposes and immutable versions, with private material confined to durable client-only custody under ADR 0035;
- privacy schemas, grants, ciphertext metadata, audit output, and APIs must use Participant Principal UUIDs and canonical key-version references;
- Final Admission needs network-verifiable participant and key-binding evidence rather than trusting API middleware state;
- key rotation and emergency revocation require the closed ADR 0031 lifecycle rather than deletion or identity reassignment; recovery after revocation of the sole authorization key is outside v1 and remains future work;
- existing `can_view_all`, username, and caller-chosen `key_id` paths cannot be used as privacy authorization evidence;
- fixtures must cover login alias to principal resolution, canonical JWT subject, token/principal mismatch, account rename, unknown principal, wrong key purpose, unknown and historical key versions, key rotation without ownership change, participant-versus-node identity separation, and absence of private key material from ledger evidence; and
- reproducible three-node and restart evidence must show identical principal/key resolution once the durable lifecycle representation is defined.

## Related Decisions

- [ADR 0004](./0004-use-ed25519-signatures.md) selects Ed25519 for signatures but does not make a signing key the participant identity.
- [ADR 0005](./0005-use-chacha20-encryption.md) selects the payload AEAD family but does not define owner identity or key distribution.
- [ADR 0009](./0009-jwt-authentication.md) establishes API authentication; this decision narrows participant JWT subject semantics.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) requires participant-authorized privacy mutations to share the fail-closed durable admission boundary.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) governs network members, which remain distinct from Participant Principals.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) makes committed privacy transitions the authority for participant and key bindings.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) assigns bootstrap governance and parent-state participant signatures to their closed transition variants.
- [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) defines atomic registration and proof of possession for the initial authorization key.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) defines strict per-purpose versions, atomic rotation, terminal emergency revocation, and historical validity.
- [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) defines immutable owner-to-grantee authority and prospective terminal grant revocation.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) defines the dedicated wrapping purpose and separates per-object DEK delivery from authorization.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the mandatory protected-data suite, wrapping-key proof, and canonical privacy encoding.
- [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) keeps exact participant private-key versions in a durable client boundary without making local custody authoritative.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) fixes the exact non-authoritative participant-keystore format and local durability contract.

## Implementation Status

Accepted architecture; ADR 0034 fixes the protected-data suite and codec, ADR 0035 fixes the client-only custody boundary, and ADR 0036 fixes the exact at-rest keystore design, but implementation, Network Profile activation, and conformance evidence remain pending. The current user record contains username, password hash, and role but no Participant Principal binding; token generation writes username into `sub`; privacy handlers require that value to parse as a UUID. The wallet registration endpoint constructs an unrelated participant, does not call a conforming client custody component, and returns `SIMULATED_PUBLIC_KEY`. Wallet privacy state is an unversioned `key_id -> shared secret` map, and encrypted data carries a caller-selected `key_id`. Transaction signatures carry their own public key beside a claimed signer UUID instead of resolving a registered key version. The wallet backup path serializes private bytes and shared secrets to plaintext JSON. No current structure provides a durable cross-node principal/key registry or conforming client-only custody. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
