# ADR 0029: Require Participant Authorization After Governed Bootstrap

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Network-verifiable authority for every `PrivacyControlV1` transition
**Clarified by:** [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) — bootstrap is one atomic registration with a profile-bound governance signature and non-authorizing initial-key possession proof.
**Lifecycle clarified by:** [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) — later key binding adds non-authorizing purpose-specific possession evidence, while emergency key revocation is only `Active -> Revoked`.
**Grant authorization clarified by:** [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) — the parent-derived owner authorizes immutable grants and terminal prospective revocation.
**Envelope authorization clarified by:** [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — owner signatures bind exact object and DEK-envelope bytes but do not prove fresh-DEK or decryptability claims.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the exact protected-data algorithms, canonical encodings, and public wrapping-key possession verifier are fixed; implementation, Network Profile activation, and conformance evidence remain pending.
**Custody boundary fixed by:** [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) — participant proofs are produced only inside durable client custody; nodes, services, and the journal never possess participant private keys, and local custody never substitutes for parent-state authorization.
**Keystore suite fixed by:** [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) — exact prepared governance-request and node-submission bytes become durable before disclosure under a fixed whole-snapshot format; implementation, activation, and conformance evidence remain pending.

---

## Decision

Every `PrivacyControlV1` transition in the thesis reference system requires one closed, domain-separated **Transition Authorization Proof** in addition to the Scheduled Authority's separate PoA proposal signature. The authorization proof is a discriminated union: either a composite **Participant Bootstrap Authorization** for the bounded initial participant bootstrap or a **Participant Authorization Signature** for every participant-controlled transition. The composite bootstrap variant contains its governance authorization signature and the introduced key's separate non-authorizing Participant Key Possession Proof. A later `BindParticipantKey` retains exactly one Participant Authorization Signature as its sole authorization and carries exactly one separately tagged possession proof as variant-specific operation evidence, not as another authorizer. Unknown, missing, duplicate, additional, or wrong-kind authorization or required evidence fields reject.

The accepted authorization matrix is:

| Privacy-control operation | Required authorizer |
|---|---|
| `RegisterPrincipal`, which atomically establishes that principal and its active initial `privacy-authorization` Participant Key Version 1 | The Network-Profile-bound privacy-bootstrap governance authorizer, plus non-authorizing possession proof by the introduced key |
| Later `BindParticipantKey` for a schema-known, profile-enabled key purpose | The affected principal, using an active parent-state `privacy-authorization` Participant Key Version, plus non-authorizing purpose-specific possession proof by the introduced key |
| Emergency `RevokeParticipantKey`, which moves only `Active` to terminal `Revoked` | The affected principal, using an active parent-state `privacy-authorization` Participant Key Version |
| `CreateProtectedObject` | The declared owner, using that principal's active parent-state `privacy-authorization` Participant Key Version |
| `GrantAccess` | The protected-object owner derived from parent Effective Privacy State, using that owner's active `privacy-authorization` Participant Key Version |
| `RevokeGrant` | The protected-object owner derived from parent Effective Privacy State, using that owner's active `privacy-authorization` Participant Key Version |

The privacy-bootstrap governance authorizer is a purpose-separated governance role bound by the active Network Profile. Its authority is limited to registering a previously unknown Participant Principal and establishing that principal's first `privacy-authorization` key. It cannot create a protected object, grant or revoke access, rotate or change the status of a later participant key, recover a participant after loss of its sole authorization key, transfer ownership, or act as a general privacy administrator. A Membership Manifest role, API `Admin` or `Auditor` role, validator status, or possession of a governance key outside the profile's privacy-bootstrap role grants no bootstrap authority.

ADR 0030 fixes bootstrap as one atomic `RegisterPrincipal` transition. The active Network Profile supplies the dedicated privacy-bootstrap governance key binding, and the composite proof also requires the introduced key's separately domain-separated, non-authorizing possession signature. `BindParticipantKey` is never an alternative initial-bootstrap path.

A Participant Authorization Signature is made with the exact key reference `(Participant Principal UUID, purpose = privacy-authorization, version)` resolved as active in Effective Privacy State at the candidate's verified parent. A public key carried in the candidate, a key introduced by that transition, a client custody record, cached key possession, successful store unlock, or current account state cannot authorize the transition. An introduced key may supply only the separately tagged possession proof required by its binding schema; for later binding that proof is operation evidence outside the one Transition Authorization Proof and is verified under the parent profile's purpose-specific scheme. For a key-lifecycle operation the signer principal must equal the affected principal. For protected-object creation the signer must equal the declared owner. For grant and revocation, Final Admission derives the owner from parent state and does not trust a request-supplied owner field.

The authorization signature is domain-separated from governance bootstrap, key-possession, PoA proposal, Node Identity, Commit Receipt, JWT, and ordinary transaction signatures. Its canonical signed material binds the `PrivacyControlV1` and schema versions, network and exact Network Profile identities, transition identifier and variant, every canonical unsigned operation-core field and opaque operation byte string, expected ledger position and privacy revision, parent Ledger Prefix Hash, authorizer kind, signer principal when applicable, and signer key purpose, version, and fingerprint. For `BindParticipantKey`, ADR 0031's authorization and possession proofs bind the same unsigned core and transition identifier; the possession-proof bytes are then added as separate operation evidence and covered by the Proposal Digest rather than being recursively included in either proof. Governance proof uses its own discriminated authorizer reference rather than pretending that a governance key is a Participant Principal or Participant Key Version.

Final Admission verifies the privacy authorization proof and the Scheduled Authority's PoA evidence as independently mandatory gates. Participant or governance authorization proves permission for the privacy transition; the PoA signature proves only Consensus Acceptance for the next ledger turn. The scheduled validator may relay an exact participant-authorized transition but cannot change its signed bytes or substitute its validator, Node Identity, Membership Manifest, JWT, wallet, or administrator evidence. Conversely, a participant or bootstrap authorizer need not be a Network Member or validator.

Authorization is evaluated against the verified parent. If an active key signs a transition that changes its own later status, its eligibility is determined before that transition takes effect. Later retirement or revocation cannot retroactively invalidate a signature that was valid at its historical parent. Startup Verified Journal Replay checks each recorded participant proof against the key and owner state at that recorded parent and each bootstrap proof against the exact Network Profile and privacy-bootstrap governance binding recorded for that parent. It never re-evaluates historical intent against the current tip.

Delegation, administrator override, recipient co-signature, grantee-initiated revocation, ownership transfer, and post-loss governance recovery are unsupported in `PrivacyControlV1`. A recipient need not co-sign a grant, but possession of recipient key material cannot create or revoke one. Adding any substitute authorizer requires a later explicit Admission Kind or versioned authorization-matrix decision.

Against a healthy verified parent, a malformed authorizer reference, wrong proof kind or signature domain, unknown or inactive signer, wrong purpose/version/fingerprint, signer-principal mismatch, governance proof on a participant-only action, participant proof on a governance-only bootstrap, introduced-key self-authorization disguised as possession proof, owner mismatch, changed parent/profile/network/transition bytes, or attempted role/key substitution is a deterministic rejection. It appends nothing and mutates no privacy projection. A node-local inability to verify is node incapacity; unavailable participant custody blocks only client-side construction and signing and never walletless follower validation.

## Rationale

The current system conflates several narrower trust statements: JWT subject equality, an embedded transaction public key, a local wallet, an API role, a validator signature, and a network identity. None proves that the ledger-recorded owner authorized a privacy transition at a particular parent state.

Separating participant intent from PoA eligibility prevents a validator or administrator from becoming a de facto data owner. Resolving the signing key and protected-object owner from the parent ledger state also makes every healthy node reach the same decision and preserves historical validity across later key changes.

Restricting governance to bootstrap closes the initial no-key cycle without creating an ongoing superuser. The exact atomic bootstrap representation remains explicit rather than being inferred from an implementation shortcut.

## Consequences

- the canonical transition schema needs a closed participant-versus-bootstrap authorizer union and distinct signature domains;
- every participant-controlled transition must name a parent-state `privacy-authorization` key reference, never carry a self-authorizing verification key;
- `GrantAccess` and `RevokeGrant` authorization uses the parent-state object owner, not an owner asserted by the caller;
- a valid privacy authorization with invalid or out-of-turn PoA evidence still rejects, as does valid PoA evidence with missing or invalid privacy authorization;
- loss of the only usable participant authorization private key prevents new owner actions because v1 has no administrator, governance-recovery, or delegation path;
- ADR 0033 fixes the protected-data and DEK-envelope topology, and ADR 0034 fixes the exact mandatory cryptographic suite, canonical encodings, and wrapping-key possession verifier; implementation, Network Profile activation, and conformance evidence remain required before `PrivacyControlV1` activation;
- tests must cover every allowed matrix row plus governance overreach, participant self-bootstrap, introduced-key self-authorization, wrong-purpose and inactive keys, cross-principal signing, caller-owner mismatch, admin/JWT/node/PoA substitution, changed-parent and cross-profile replay, rejected-transition non-mutation, and historical replay after signer retirement; and
- three-node evidence must show walletless followers accepting the same exact valid proof and rejecting the same exact invalid proof from identical parent prefixes.

## Related Decisions

- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes both privacy authorization and PoA evidence mandatory gates before the sole commit point.
- [ADR 0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) defines the separate Scheduled Authority rule.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) defines network membership and governance trust, neither of which silently grants participant authority.
- [ADR 0022](./0022-authenticate-peer-sessions-with-mutual-ed25519-challenge-response.md) assigns a separate key and claim to peer identity.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) defines stable principals and purpose-bound key versions.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) defines the closed transition journal and parent-derived Effective Privacy State consumed by this matrix.
- [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) fixes the composite proof, exact bootstrap trust binding, and atomic principal/key creation rule.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) fixes later bind possession evidence, atomic rotation, terminal emergency revocation, and parent-state historical verification.
- [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md) fixes immutable grant creation, terminal revocation, and owner-proof replay semantics.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) fixes the exact structural object and grant bytes covered by owner authorization.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the exact protected-data suite, canonical privacy encoding, and public wrapping-key possession verifier.
- [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) confines participant proof production to non-authoritative client custody and excludes server-side signer or recovery substitutes.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) fixes durable prepared stages and exact-byte retry without making local custody an authorizer.

## Implementation Status

Accepted architecture; ADR 0034 fixes the protected-data suite and codec, ADR 0035 fixes client-only custody, and ADR 0036 fixes the exact at-rest keystore and prepared-stage design, but implementation, Network Profile activation, and conformance evidence remain pending. The current transaction signature embeds an arbitrary public key beside a claimed participant UUID and verifies only that embedded key. JWT middleware and server-side wallet lookup remain API/runtime checks rather than Final Admission evidence and are not conforming client custody. No `PrivacyControlV1` type, participant-key registry, profile-bound privacy-bootstrap authorizer, parent-state signer resolution, or signer-matrix tests exist. Current network configuration exposes PoA authority keys but no usable privacy-bootstrap binding, and the existing governance module is validator voting rather than this bootstrap authority. No implementation code changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
