# ADR 0035: Keep Participant Private Keys in Durable Client-Only Custody

**Status:** Accepted
**Date:** 2026-08-31
**Context:** Durable participant capability without node, server, or ledger custody of private material
**Keystore suite fixed by:** [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) — one exact encrypted whole-snapshot format, Linux/ext4 single-writer commit, prepared-state retry, backup, and quarantined restore are fixed; implementation, activation, and conformance evidence remain pending.

---

## Decision

The bounded thesis-reference system uses one **Participant Key Custody** boundary named
`ParticipantKeyCustodyV1`. Every private counterpart of a `PrivacyControlV1` Participant Key
Version remains in a participant-controlled client component. A validator node, web service,
Scheduled Authority, Ledger Journal, Admitted Block Envelope, replica, Oxigraph store, index,
account database, JWT service, or server backup must never receive or retain a participant
passphrase, participant private key, unwrapped Protected-Object DEK, or protected plaintext.

This is a logical trust and process boundary, not a claim about physical location. A reference
client may run on the same host as a node for a controlled experiment, but it must use separate
storage, APIs, memory ownership, and privileges. Co-location does not permit the node or web
service to open the custody store, request a general signature or HPKE decapsulation, log a
secret, or include private material in node backup and recovery.

Participant Key Custody is deliberately **non-authoritative**. The client stores the exact private
capability and public identity needed to locate it, but it never decides the network lifecycle of
a Participant Key Version. `Active`, `Retired`, and `Revoked`, key-purpose eligibility, ownership,
and Privacy Grant status always come from Effective Privacy State at the exact verified ledger
prefix required by the operation. A local timestamp, label, cache, account role, file presence,
backup generation, or successful unlock cannot activate, retire, revoke, restore, or select a
ledger key.

Each bound custody record carries the following authenticated verification metadata:

```text
(network identity,
 Participant Principal UUID,
 Participant Key Purpose,
 version,
 algorithm,
 canonical public-key encoding,
 public-key bytes,
 fingerprint)
```

The protocol identity of a bound Participant Key Version remains exactly `(Participant Principal
UUID, Participant Key Purpose, version)` under ADR 0027; the expanded metadata above does not create
a second identity. Before ledger binding, the same fields describe only a local **Participant Key
Candidate**, not a Participant Key Version. The client derives the public key and ADR 0034
fingerprint again from the private material when it loads, verifies, backs up, or restores a record.
A mismatch, duplicate, prohibited cross-purpose reuse, unknown purpose or algorithm, malformed
private value, or principal/network mismatch fails closed. A free-form `key_id`, username, wallet
path, JWT subject, newest-local-key heuristic, or caller-supplied lifecycle status cannot substitute
for the protocol identity and its verification metadata.

The two current purposes expose different private operations:

- an unbound `privacy-authorization` Ed25519 Participant Key Candidate may produce only the exact
  introduced-key possession proof required by its one durable `RegisterPrincipal` or
  `BindParticipantKey` intent;
- an `Active` `privacy-authorization` Ed25519 key may produce only the exact participant-authorization
  proof allowed by ADRs 0029-0034;
- an `Active` `privacy-key-wrapping` P-256 key may decapsulate only for ADR 0034's typed
  construction-time Owner DEK Envelope self-check/recovery at an exact verified parent or for a
  typed current Live Privacy Release response with exact verified Network-Converged evidence;
- a `Retired` wrapping key may decapsulate only for that exact Live Privacy Release path, while an
  unbound wrapping-key candidate exposes only ADR 0034's narrow possession proof for its one
  durable pre-binding intent;
- every proof operation accepts a typed canonical operation intent and verified ledger context,
  validates principal, purpose, version, lifecycle, and parent eligibility, and derives its
  domain-separated digest internally; no caller-facing arbitrary-digest or generic-signature API
  exists;
- after binding, restoring or unlocking a wrapping key must not re-enable a generic ECDSA signing
  interface; and
- no caller-facing raw HPKE-decapsulation interface exists, and a `Retired` authorization key never
  signs.

The store retains all locally generated `Active` and `Retired` Participant Key Version records in
bounded v1. Retaining a Retired wrapping private key is mandatory because immutable Owner and Grant
DEK Envelopes continue to address that exact version and Live Privacy Release permits an exact
`Active` or `Retired` recipient key. Retired authorization private keys are also retained under the
confirmed bounded policy but are quarantined from every signing path and provide no current ledger
authority. A later minimization policy may remove them only through a new explicit custody-format
decision and evidence; it may not change historical ledger verification.

A conforming custody component never uses a `Revoked` key for signing, wrapping-key possession
proof, HPKE decapsulation, or current Live Privacy Release. Final Admission and the
Network-Converged managed-release boundary reject its prospective use even if a local record or
backup still contains private bytes. The live store retains an authenticated public tombstone
sufficient to explain the local record and rejects restore-based resurrection. This is an
enforcement boundary, not a cryptographic impossibility: an old nonconforming client or attacker
with copied private bytes and a previously obtained envelope may still perform an offline signature
or decapsulation. Removing revoked secret bytes from a later encrypted snapshot is desirable
hygiene, but this decision makes no secure-erasure claim: earlier backups, copied files, swap, crash
dumps, filesystem journals, SSD remapping, or an attacker's copy may retain them.

### Durable-before-disclosure ordering

Generating a participant key and binding its public key are not one cross-system transaction.
`ParticipantKeyCustodyV1` therefore fixes this client-side order:

1. generate the purpose-specific private key using the fallible OS-CSPRNG path required by ADR 0034;
2. derive and verify its canonical public binding and exact proof intent locally;
3. before the first disclosure, atomically and durably commit the encrypted Participant Key
   Candidate plus the exact canonical core, introduced-key possession proof, and exact bytes that
   may be sent to the required authorizer;
4. for ordinary participant-authorized binding, derive the active-key authorization internally and
   durably commit the exact complete transition before emitting it to a node;
5. for governance-authorized `RegisterPrincipal`, emit only the already-durable exact governance
   signing request, validate the returned governance signature against that request, and durably
   commit the exact complete governance-signed transition before emitting it to a node; and
6. after commitment or rejection is known, reconcile the local record against a verified current
   Effective Privacy State rather than assigning ledger status locally.

If the custody commit fails or its durability is indeterminate, the client emits no key-binding
proof, authorization request, or transition. Durable preparation is conservative: once a proof
intent is committed as disclosable, recovery treats it as potentially exposed even if a crash
occurred before the first send. A crash after the first local commit may therefore leave a prepared,
unbound Participant Key Candidate containing either an exact authorizer request or an exact complete
transition. It is quarantined and is not treated as Active; it permits only byte-identical retry of
the stored stage or later explicitly safe cleanup after reconciliation. In particular, bootstrap
recovery never constructs a complete submission until the matching governance signature has been
validated and durably joined to the exact stored request, and no transition reaches a node before
that complete submission is durable. Recovery must never regenerate a proof for a different parent,
profile, network, principal, purpose, or version. A crash after the ledger commits but before the
client records that observation is repaired by matching the exact public binding and fingerprint
from the ledger. ADR 0036 preserves every prepared/disclosable stage in a complete encrypted
snapshot and backup and makes ambiguous recovery quarantine rather than reset that state. Public
admission cannot prove that an arbitrary external participant persisted its private key;
durable-before-disclosure is a conformance claim about the reference client and its evidence, not a
new wallet-dependent consensus gate.

ADR 0036 selects the exact encrypted file format, passphrase byte rules, KDF, AEAD, fresh
salt/snapshot construction, bounds, atomic replacement system-call sequence, prepared-record
encoding, local error contract, backup, and restore as one closed `ParticipantKeystoreSuiteV1`.
That accepted design still does not activate `ParticipantKeyCustodyV1` or `PrivacyControlV1`:
implementation and the complete conformance package remain mandatory. A direct overwrite,
plaintext master-key file, plaintext JSON export, best-effort flush, implementation-default KDF,
per-device parameter negotiation, or platform-undefined rename is not conformance evidence.

### Backup, restore, and reconciliation

Every custody backup is made and opened only by the participant client. It is a complete,
authenticated, encrypted snapshot of the custody records and prepared/disclosable state required by
the bounded retention and durable-before-disclosure rules; it is never a plaintext export and no
ProvChain node or web-service backup endpoint may escrow its passphrase or plaintext. External
storage may hold only the opaque encrypted backup bytes.

A backup is considered verified only after an isolated client can authenticate and decode it,
rederive every public key and fingerprint, reject duplicates and malformed records, and compare its
key set with Effective Privacy State at an explicitly verified Network-Converged prefix. Merely
copying a file, hashing ciphertext, or recording a successful write does not prove recoverability.

Restore never changes ledger history or lifecycle state and never silently merges by key label. It
first opens into quarantine, validates the complete snapshot, then reconciles every exact tuple
against the selected ledger prefix. It reports missing current or historically required private
capabilities, extra unbound prepared records, and locally retained records whose ledger state is
now Revoked. It may install a usable store only through ADR 0036's whole-snapshot Custody Store
Commit. A stale but authentic backup can therefore create a **Custody Gap**; it can never roll back
the ledger, reactivate a key, substitute a newer key for an old envelope, or make a historical
grant authorize current release.

Bounded v1 generates private keys inside the custody boundary, accepts no raw-private-key import,
and exposes no raw-private-key export. Purpose-specific proof and decapsulation APIs never reveal
the private bytes. Opening an authenticated encrypted Custody Backup into quarantine is the only
key-ingress path after local generation. The store has exactly one exclusive writer; a second
writer, concurrent restore, or concurrent mutation fails closed. ADR 0036 pins the exact stable
locking and whole-snapshot atomic-replacement protocol; conformance remains unimplemented.

Ledger reconciliation detects a backup that omits a ledger-recorded key version. It cannot by
itself detect every replay of older local-only metadata, such as an earlier passphrase or storage
hardening state, when the key set is unchanged. Strong rollback detection for such state requires
an external monotonic witness and remains outside bounded v1.

### Loss and compromise boundaries

A **Custody Gap** is local capability loss, not a ledger lifecycle transition:

- loss of the sole usable `Active` authorization private key prevents new participant-controlled
  transitions and may make the existing Participant-Control Freeze effectively unavoidable;
- loss of an `Active` or `Retired` wrapping private key makes envelopes addressed to that exact
  version locally unreadable; rotation or a newer key cannot substitute for it;
- a successful managed release may still return ciphertext and an envelope that the client can no
  longer open; and
- loss of both the usable custody store and every valid backup, or loss of their passphrase, is
  irrecoverable in v1.

There is no server escrow, governance recovery, administrator reset, validator override, recovery
seed, social recovery, key reconstruction, or hidden node copy. Bootstrap governance cannot repair
a participant after loss and a backup cannot bypass ledger authorization.

Compromise is also prospective and cannot be undone by rewriting local storage. An attacker with
an `Active` authorization private key may create valid-looking participant proofs until a competing
ledger lifecycle transition commits; v1 cannot cryptographically distinguish that attacker from
the participant. Revoking an Active wrapping key can stop later managed release after the required
prefix, but cannot recall an already copied private key, DEK Envelope, DEK, or plaintext. A
compromised `Retired` wrapping key is worse: ADR 0031 makes Retired terminal and ADR 0034 keeps it
release-eligible, so v1 has no later revocation operation that can shut off its managed-release
eligibility. Re-encrypting a store or changing a passphrase does not neutralize a copied secret.

Zeroization, redacted errors, and avoidance of secret logging are mandatory hygiene but not proof
of secure deletion, constant-time behavior, memory locking, resistance to a compromised client,
or absence of copies in registers, allocators, swap, suspend images, or hardware. Those stronger
claims need separate implementation and operational evidence.

## Rationale

The ledger must remain self-contained and walletless for Final Admission, replay, follower
validation, and three-node convergence, while participant authorization and client-side DEK
recovery require secrets that validators must not possess. A durable client-only boundary preserves
both properties and prevents a convenient server wallet from silently becoming a decryption or
authorization escrow.

Retaining exact versions follows from immutable envelopes: rotating a wrapping key changes future
recipient selection but cannot rewrite an object or grant that names the old version. This trades a
larger retained-secret exposure surface for deterministic recovery of previously created protected
objects. The explicit compromise boundary prevents that availability choice from being presented
as revocable or erasable confidentiality.

Persisting a generated key before publishing its binding avoids the reference client committing a
public key whose private counterpart existed only in volatile memory. Keeping that ordering outside
consensus also preserves the public, deterministic admission boundary and accurately limits what
nodes can prove.

## Consequences

- The reference client needs a purpose/version-aware encrypted custody store separate from
  `src/wallet.rs`, node state, auth-user storage, and ledger projections.
- Object/grant construction, participant authorization, Live Privacy Release opening, rotation,
  backup, and restore are client operations; the service handles only public proofs, committed
  ciphertext/envelopes, and release evidence.
- Store presence and unlock success never confer authority; every operation resolves the exact
  ledger reference and status at the required prefix.
- Active and Retired private versions remain in the bounded store; the exposure cost and the
  especially severe Retired-wrapping-key compromise are explicit.
- ADR 0036 fixes the exact keystore cryptography, canonical bytes, bounded Linux/ext4 durability
  scope, rollback limitation, and limits; implementation and evidence remain the activation gate.
- Multi-device synchronization, multi-writer operation, OS keychains, HSM/KMS integration, enterprise
  escrow, governance or social recovery, remote wipe, external monotonic witnesses, secure deletion,
  rewrap, automatic re-encryption, production backup operations, and usability studies remain
  future work or later milestones.
- Evidence must cover secret-free node/server/journal state; durable-before-disclosure failpoints;
  prepared, potentially exposed, and committed crash recovery; byte-identical retry and
  different-parent proof rejection; exact Active/Retired selection; conforming Revoked non-use; stale,
  corrupt, wrong-network, and incomplete backup restore; public-key/fingerprint rederivation;
  Custody Gap reporting; restart; no secret logging; and walletless three-node convergence.

## Related Decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) keeps the public ledger
  commit boundary distinct from the client custody commit.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) separates stable principals,
  public key bindings, accounts, and wallets.
- [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md) makes ledger-derived privacy state
  authoritative while custody remains local and non-authoritative.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) defines the
  participant proofs produced with authorization private keys and excludes server substitutes.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) fixes immutable versions,
  Active/Retired/Revoked lifecycle, rotation, and Participant-Control Freeze.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) requires exact old
  wrapping-key versions to open immutable Owner and Grant DEK Envelopes.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes key schemes,
  the narrow wrapping-key PoP, release eligibility, and client-side HPKE/payload operations.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) fixes the exact
  encrypted snapshot, prepared-stage, commit/recovery, backup, restore, and Revoked-rewrite contract.

## Implementation Status

Accepted custody boundary and accepted ADR 0036 keystore design only. No conforming
`ParticipantKeyCustodyV1` or `ParticipantKeystoreSuiteV1` implementation exists. Current
`src/wallet.rs` is server-side legacy scaffolding: it stores one unversioned Ed25519 key and
free-form shared secrets, keeps a raw file-level master key, directly overwrites wallet files, and
can serialize raw private bytes and shared secrets into plaintext JSON backups. The query handler
can consult that server wallet and return decrypted plaintext. None of those paths is the accepted
client-only boundary, and they must be disabled or absent before privacy profile activation.

No implementation code, dependency, lockfile, toolchain declaration, benchmark, or conformance
evidence changed with this decision.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-31
