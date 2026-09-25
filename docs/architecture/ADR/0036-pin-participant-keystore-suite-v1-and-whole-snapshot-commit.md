# ADR 0036: Pin ParticipantKeystoreSuiteV1 and Whole-Snapshot Custody Commit

**Status:** Accepted
**Date:** 2026-08-31
**Context:** Reproducible encrypted participant custody, crash-safe exact retry, backup, and restore

---

## Decision

The bounded thesis-reference system uses one indivisible at-rest and local-durability contract named
`ParticipantKeystoreSuiteV1`, paired with one closed byte codec named
`CanonicalParticipantKeystoreEncodingV1`. The suite is mandatory for the client-only
`ParticipantKeyCustodyV1` boundary fixed by ADR 0035. It is not a cipher menu, per-device policy,
Serde format, server wallet, or Network Profile negotiation point.

The v1 store has exactly one Participant Principal, one network identity, one live whole-snapshot
file, and one exclusive writer. Private keys are generated inside custody; raw-private-key import
and export do not exist. The only post-creation ingress is a complete encrypted Custody Backup
opened into Restore Quarantine and reconciled against one current verified Network-Converged ledger
prefix. Custody Snapshot generation, digest, and local commit state are never ledger authority.
Ledger Journal append plus `fsync` remains the sole block-commit authority under ADR 0016.

The complete suite comprises all of the following. Omitting or substituting any element is
nonconforming and keeps `ParticipantKeyCustodyV1` and `PrivacyControlV1` fail closed.

### CanonicalParticipantKeystoreEncodingV1

Keystore records occupy a namespace separate from ADR 0034's closed `PCV1` records. Every record is:

```text
record = ASCII "PKV1"                         # 4 bytes
      || record_tag:u8
      || field_count:u8
      || fields

field  = field_tag:u8
      || value_length:u32be
      || value[value_length]
```

Field tags are `0x01` upward in exactly the order listed for that record. A decoder rejects a wrong
magic, record tag, field count, tag, order, or length; a duplicate, unknown, absent, or forbidden
empty field; a nested record that does not consume its complete containing field; an unchecked
length or arithmetic overflow; and any trailing byte. Integers are fixed-width big-endian. UUIDs,
identifiers, digests, keys, signatures, ciphertext, and proofs are raw bytes. JSON, Serde, CBOR,
PHC strings, DER, PEM, base64, hex, platform `usize`, and Rust layout are not keystore encodings.

The tag table is closed:

| Tag | Record |
|---:|---|
| `0x01` | `KeystoreHeaderV1` |
| `0x02` | `KeystoreFileKeyInfoV1` |
| `0x10` | `ParticipantKeystoreSnapshotV1` |
| `0x11` | `ParticipantKeystoreEntryListV1` |
| `0x20` | `ParticipantPrivateKeyMaterialV1` |
| `0x21` | `BoundParticipantPrivateKeyV1` |
| `0x22` | `RevokedParticipantKeyTombstoneV1` |
| `0x23` | `ParticipantKeyBindingAuthorizationRequestV1` |
| `0x24` | `PreparedGovernanceRequestV1` |
| `0x25` | `PreparedNodeSubmissionV1` |

No extension or unknown-field rule exists in v1. A later format uses a new suite/codec decision and
an explicit historical decoder rather than interpreting an unused value.

### Exact file header and bounds

Each logical store draws a nonzero raw `file_id16` once from the fallible OS CSPRNG and retains it
across snapshots, byte-identical backups, and restored successors. Every newly sealed snapshot draws
an independently fresh nonzero `snapshot_id16` and raw `salt16`. Generation is `u64be`, begins at
one, increases by exactly one, and rejects overflow.

`KeystoreHeaderV1` (`0x01`) has exactly 17 fields and is exactly 269 bytes including `PKV1` framing:

| Field | Exact value |
|---:|---|
| 1 | suite ID: the 26 ASCII bytes `ParticipantKeystoreSuiteV1` |
| 2 | codec ID: the 38 ASCII bytes `CanonicalParticipantKeystoreEncodingV1` |
| 3 | `file_id16` |
| 4 | `snapshot_id16` |
| 5 | generation `u64be` |
| 6 | previous-snapshot reference33 |
| 7 | Argon2 algorithm tag `0x02` (`Argon2id`) |
| 8 | Argon2 version tag `0x13` |
| 9 | memory cost `u32be(65_536)` KiB |
| 10 | time cost `u32be(3)` |
| 11 | parallelism `u32be(4)` lanes |
| 12 | Argon output length `u32be(32)` |
| 13 | fresh `salt16` |
| 14 | HKDF-SHA256 tag `0x01` |
| 15 | IETF ChaCha20-Poly1305 tag `0x01` |
| 16 | fixed-zero12, one-use-key nonce-scheme tag `0x01` |
| 17 | plaintext length `u32be`, `1..=262_144` |

For generation one, the previous-snapshot reference is exactly `0x00 || zero32`. For every later
normal successor it is `0x01 || SHA-256(exact complete predecessor file bytes)`. A restored
successor similarly names the exact backup file as its lineage predecessor even though its live
installation target is absent. When a predecessor is available, the client verifies the same
`file_id`, exact generation increment, and exact complete-file digest before publication.

The complete file grammar is:

```text
AAD  = ASCII "PKS1" || u32be(269) || exact KeystoreHeaderV1
File = AAD            # exactly 277 bytes
    || ciphertext[header.plaintext_length]
    || full_tag16
```

There is no encoded ciphertext length or trailing data: it is exactly `plaintext_length + 16`.
The complete file is at most 262,437 bytes. Before passphrase derivation or attacker-sized
allocation, opening preflights the exact outer magic, header length, fixed record structure,
fixed algorithm parameters, plaintext range, checked total length, and end of file. Fixed fields
are conformance assertions, not algorithm negotiation.

The **Custody Snapshot Digest** is SHA-256 of these exact complete file bytes. Together,
`(file_id16, generation, Custody Snapshot Digest)` is the local Custody Snapshot Reference used for
compare-and-swap and recovery. It is not a Ledger Prefix Hash, Envelope Hash, Commit Receipt, proof
of freshness, or external rollback witness.

### Exact passphrase and Argon2id contract

A v1 passphrase is exactly `15..=128` bytes and every byte is printable ASCII `0x20..=0x7e`,
including space. An input record delimiter or terminal newline is framing and is excluded. Every
in-range leading or trailing space is data. No trimming, case folding, Unicode normalization,
implicit conversion, alternate spelling, NUL, truncation, or hidden terminator is allowed.
Passphrases are never accepted through a command-line argument, environment variable, config file,
log, diagnostic, or node/server API. The participant client obtains the bytes from a non-echoing
interactive input or a protected client-owned descriptor and gives those exact bytes to the KDF.

The KDF invocation is exactly:

```text
P = exact passphrase bytes
S = header.salt16
K = empty secret
X = empty associated data
algorithm = Argon2id (type y = 2)
version   = 0x13
m_cost    = 65_536 KiB
t_cost    = 3
lanes     = 4
T         = root32
```

The RustCrypto parameter construction is exactly:

```text
Params::new(65_536, 3, 4, Some(32))
Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
```

The implementation must use `hash_password_into_with_memory` with an explicitly supplied arena of
exactly 65,536 Argon2 Blocks, or 67,108,864 bytes. The arena is allocated fallibly before use and
owned from its first initialized block by an RAII guard whose destructor zeroizes every initialized
Block on ordinary return, error, or panic unwind. The convenience `hash_password_into` allocator is
not conforming evidence because the reviewed `argon2 0.5.3` implementation drops its allocated
arena without explicitly wiping the complete arena. The passphrase buffer, `root32`, derived-key
buffers, decrypted snapshot, and private-key working buffers also receive best-effort zeroization.

These fixed costs are for reproducible bounded evidence, not proof of passphrase strength,
resistance to every offline attack, automatic tuning for a device, memory locking, or secure
erasure. A lost passphrase remains irrecoverable in v1.

### HKDF-SHA256 file-key separation

The HKDF-Extract salt is the exact 51 ASCII bytes:

```text
provchain/participant-keystore/hkdf-extract-salt/v1
```

`KeystoreFileKeyInfoV1` (`0x02`) is exactly 196 bytes and has seven fields:

1. the 47 ASCII bytes `provchain/participant-keystore/file-aead-key/v1`;
2. the 26-byte suite ID `ParticipantKeystoreSuiteV1`;
3. the 38-byte codec ID `CanonicalParticipantKeystoreEncodingV1`;
4. `file_id16`;
5. `snapshot_id16`;
6. generation `u64be`; and
7. output length `u32be(32)`.

Derivation is exactly:

```text
PRK32  = HKDF-Extract-SHA256(fixed_salt51, root32)
K_file = HKDF-Expand-SHA256(PRK32, exact_info196, 32)
```

The info is independent of the input keying material. The Argon salt affects `root32`; the full
header, including that salt and every fixed parameter, is authenticated as AEAD associated data.
No root key, salt, prior digest, ciphertext digest, or implementation-specific path is appended to
the info.

### One-use AEAD and exact retries

Each new Custody Snapshot performs exactly one RFC 8439 IETF ChaCha20-Poly1305 Seal:

```text
(ciphertext, full_tag16) = Seal(
    K_file,
    nonce = 12 zero bytes,
    AAD = exact AAD277,
    plaintext = exact ParticipantKeystoreSnapshotV1 bytes)
```

There is no XChaCha variant, random or serialized nonce, chunking, second Seal, tag truncation,
detached unbound field, compression, or padding. `K_file` is structurally one-use because every
new seal uses a fresh salt and snapshot ID in a generation-bound derivation. If any intended
plaintext or header byte changes, or already-produced candidate file bytes are lost, that candidate
is abandoned and a new salt and snapshot ID are drawn. A durability or network retry never invokes
Seal: it writes or emits the already-produced complete bytes byte for byte.

No new mutation may begin while an earlier commit outcome is indeterminate or an authenticated
recovery candidate exists. This prevents two locally constructed children from treating the same
Custody Snapshot as their expected target.

### Snapshot entries and generate-only key material

The AEAD plaintext is exactly one `ParticipantKeystoreSnapshotV1` (`0x10`) record with three fields:

1. `network_id`, `1..=128` ASCII bytes matching ADR 0034's
   `[a-z0-9][a-z0-9._:-]*` rule;
2. one non-nil Participant Principal UUID16; and
3. one exact `ParticipantKeystoreEntryListV1` record.

`ParticipantKeystoreEntryListV1` (`0x11`) contains count `u32be` in `1..=1_024` and one sequence
field. The sequence is exactly `count` repetitions of
`u32be(entry_length) || entry_record`; each entry is `1..=20_480` bytes and the enclosing plaintext
remains at most 262,144 bytes. Entries are strictly sorted by `(purpose_tag, version_u32)` and that
tuple is unique across Bound, Prepared, and Tombstone records. An unknown entry kind, duplicate or
out-of-order tuple, duplicate canonical public key or fingerprint, prohibited cross-purpose reuse,
or inconsistent nested identity rejects.

`ParticipantPrivateKeyMaterialV1` (`0x20`, at most 256 bytes) has exactly eight fields: purpose tag,
version `u32be` in `1..=u32::MAX`, algorithm tag, public-key encoding tag, canonical public-key
bytes, ADR 0034 fingerprint32, private-key encoding tag, and private bytes32. Only these combinations
exist:

| Purpose | Algorithm/public form | Private form |
|---|---|---|
| `privacy-authorization=0x01` | `Ed25519=0x01`, `Ed25519Raw32=0x01`, 32-byte public key | `Ed25519Seed32=0x01`, 32-byte seed |
| `privacy-key-wrapping=0x02` | `P256=0x02`, `Sec1UncompressedP256=0x02`, 65-byte point | `P256Scalar32BE=0x02`, 32-byte scalar in `1..n-1` |

Every structural open, mutation, backup verification, and restore rederives the canonical public
key and ADR 0034 fingerprint from the secret and requires byte-exact equality. A mismatch, invalid
scalar/point, small-order or noncanonical Ed25519 public value under the strict ADR 0034 verifier,
snapshot-local duplicate, or prohibited snapshot-local cross-purpose reuse fails closed. An offline
open cannot establish network-global uniqueness. Internal rederivation and structural comparison
inside the custody boundary are verification, not a purpose-specific private operation. Before any
purpose-specific private operation (including authorization signing, possession proof, HPKE
decapsulation, or protected-data opening) or outbound bytes, before a backup becomes Verified, and
before restore installation, the client reconciles every public key and fingerprint against one
current verified Network-Converged prefix and rejects a conflicting ledger-global binding or
reuse. Secret-bearing types must not implement unrestricted `Clone`, `Debug`, `Display`, Serde
serialization, or raw byte export.

`BoundParticipantPrivateKeyV1` (`0x21`) contains exactly one
`ParticipantPrivateKeyMaterialV1`. It stores no local `Active`, `Retired`, or `Revoked` flag:
Effective Privacy State at the operation's exact verified prefix remains authoritative.

`RevokedParticipantKeyTombstoneV1` (`0x22`) has purpose, version, algorithm, public encoding,
canonical public bytes, and fingerprint. It contains no private bytes and does not independently
declare lifecycle authority; it must match ledger-proven terminal Revocation during reconciliation.

### Durable prepared stages

`ParticipantKeyBindingAuthorizationRequestV1` (`0x23`, at most 9,216 bytes) has exactly seven fields:

1. ADR 0034 authorizer tag: bootstrap governance `0x01` or participant `0x02`;
2. exact canonical required-authorizer reference, at most 512 bytes;
3. exact canonical `RegisterPrincipal` or `BindParticipantKey` unsigned core, at most 8,192 bytes;
4. recomputed `T_register32` or `T_bind32`;
5. introduced-key possession-proof scheme tag;
6. exact possession proof64; and
7. recomputed `G_register32` or `A_bind32`.

All nested network, profile, parent, principal, purpose, version, algorithm, encoding, public key,
fingerprint, core, proof, transition identifier, authorizer reference, and digest values must agree.
The request is the exact complete context that may be presented to the bootstrap governance signer;
that signer signs only its stored `G_register32`. A participant-authorized bind uses the same closed
request internally but does not disclose it to an external signer.

`PreparedGovernanceRequestV1` (`0x24`, at most 10,240 bytes) contains exactly one private-key
material record and the exact matching authorization-request record. It is legal only for
`RegisterPrincipal` under bootstrap governance. This is the first durable-before-disclosure state.
Only the already-stored request bytes may be emitted.

`PreparedNodeSubmissionV1` (`0x25`, at most 20,480 bytes) contains the same private-key material,
the same authorization request, and one exact matching ADR 0034 complete transition of at most
8,192 bytes. For bootstrap, the returned governance signature must first verify against the exact
stored request and `G_register32`; for participant binding, the current bound Active authorization
key signs the exact stored `A_bind32` inside custody. In either case, the complete transition must
be committed in this state before any byte is sent to a node.

There is no standalone durable unprepared-candidate record. Construction either reaches the
appropriate complete prepared state before disclosure or discards the undisclosed candidate.
Recovery and retry select an exact `(network_id, principal UUID, purpose, version, stage)` and can
return only the stored request or complete transition bytes after verifying their nested digests.
They never accept replacement bytes, re-sign, re-canonicalize, change parent/profile, or regenerate
a proof. A partial send, lost response, or duplicate response is resolved by exact retry and ledger
reconciliation, not by interpreting transport outcome as commitment.

Once a verified Network-Converged prefix contains the exact public binding and fingerprint, the
prepared entry becomes Bound through a new whole-snapshot commit. If its expected ledger parent is
stale or a conflicting binding committed, its secret and prepared bytes remain unusable in
quarantine; they are never adapted to a new parent. Cleanup is allowed only after reconciliation
proves that the exact potentially exposed proof cannot become current.

### Qualified Linux/ext4 single-writer profile

The v1 durability claim is deliberately narrow: Linux kernel 5.6 or newer, initially on a local
ext4 filesystem and exact mount/storage profile covered by the conformance crash campaign. The
reference target is `x86_64-unknown-linux-gnu`. The client rejects NFS, CIFS/SMB, FUSE, overlay,
network, userspace, cross-mount, or otherwise unqualified filesystems; unsupported directory
`fsync`; and a kernel without `openat2`. XFS and other local filesystems require the same evidence
campaign before support. Merely succeeding on a temporary filesystem is not qualification.

Startup qualification matches the pinned directory's device major/minor to exactly one entry in
the current mount namespace's `/proc/self/mountinfo`, requires its filesystem type to be exactly
`ext4`, and records the running kernel release, mount options, and storage profile identifier used
by the crash campaign. Missing, malformed, ambiguous, moved, or nonmatching mount information fails
closed. The client repeats this check before publication; a matching ext-family `fstatfs` magic
alone is insufficient because ext2, ext3, and ext4 share that magic value.

The custody directory is reached from a trusted directory descriptor using `openat2` with
`RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_XDEV`, then pinned and
revalidated by device/inode and mount identity. It must be a real directory owned by the effective
UID with mode exactly `0700`. The live name is exactly `participant-keystore.pks1`. Live, lock,
temporary, backup, and quarantine files must be regular files owned by the effective UID, on their
pinned mount, with link count one and mode exactly `0600` unless a later decision narrows it further.
All live-file opens are relative to the pinned directory and use `openat2` with
`O_RDONLY | O_CLOEXEC | O_NOFOLLOW` plus the same resolution constraints; they never create or
truncate the live pathname.

The stable lock is the separate permanent `.participant-keystore.lock` inode, never the replaceable
live snapshot. A first client creates it with
`O_CREAT | O_EXCL | O_RDWR | O_CLOEXEC | O_NOFOLLOW`, mode `0600`, then file- and directory-fsyncs
it; later clients open it without truncation under the same no-follow checks. The client takes a
nonblocking `flock(LOCK_EX | LOCK_NB)`, also holds a process-local mutex, and retains that same open
file description through recovery, mutation, backup selection, restore installation, and any
pre-send readiness gate. It never infers a stale lock from a PID and never deletes or replaces the
lock inode. `flock` is advisory; the owned `0700` directory is part of the boundary.

Temporary candidates use the exact prefix `.participant-keystore.tmp.` followed by 32 lowercase
hex digits encoding a fresh random16. They are named recovery artifacts, not `O_TMPFILE` objects.
Creation is relative to the pinned directory with
`O_CREAT | O_EXCL | O_RDWR | O_CLOEXEC | O_NOFOLLOW`, process `umask 077`, and mode `0600`.
The client applies `fchmod(0600)` and rechecks type, owner, mode, device/mount, inode, and link count.
It never opens, truncates, or reuses an existing candidate.

This boundary addresses conforming-client crashes, partial writes, path substitution by other UIDs,
and accidental concurrent clients. It does not protect against root, a hostile process with the
same UID, compromised client memory, storage firmware that lies about flush completion, or rollback
of the entire custody directory.

### Custody Store Commit and expected target

Every normal mutation carries the exact currently authenticated Custody Snapshot Reference as its
**Expected Custody Target**. Genesis and restore installation require the target to be `ABSENT`.
The expected custody target is a local compare-and-swap guard and is distinct from the Expected
Ledger Parent embedded in a prepared transition. Under the stable lock, the client reopens and
authenticates the live file and compares the complete expected tuple immediately before publication.

The state machine is:

```text
LockedReady(P)
  -> TempCreated(C)
  -> TempWrittenAndVerified(C)
  -> TempDurable(C)             # fsync(candidate file)
  -> TargetCASConfirmed(P)
  -> PublishedUnconfirmed(C)    # same-directory renameat2
  -> Committed(C)               # fsync(custody directory): sole custody commit point
  -> Ready(C)                   # reopen and authenticate
```

The exact operation is:

1. serialize and seal the complete successor once;
2. write all bytes with bounded positional-write loops, retrying only `EINTR` and rejecting zero
   progress, overflow, incomplete completion, `ENOSPC`, `EDQUOT`, `EIO`, and every other error;
3. reread the candidate, verify exact size and complete-file digest, and `fsync(candidate_fd)`;
4. revalidate the candidate inode, pinned directory and lock identities, and authenticated expected
   live target;
5. for genesis/restore, publish with same-directory `renameat2(RENAME_NOREPLACE)`; for a normal
   replacement, use same-directory `renameat2` with flags zero only after the target CAS check;
6. verify that the live pathname now names the candidate inode;
7. `fsync(custody_directory_fd)`; and
8. reopen, authenticate, rederive, and verify the live snapshot before permitting secret use or
   outbound request/transition bytes.

Successful directory `fsync` in step 7 is the sole **Custody Store Commit** point. File `fsync`
alone is not commitment because it does not make the directory entry durable. Custody Store Commit
does not commit a block, produce a Commit Receipt, or establish Network Convergence.

A definite pre-rename failure may return `Aborted` only after the exact candidate inode is unlinked,
the custody directory is successfully fsynced, and the live target is reverified as the expected
parent. Any uncertainty is `CommitIndeterminate` and blocks all secret use and outbound bytes until
recovery. In particular, an error or crash during or after rename but before successful directory
`fsync` is indeterminate; the client never reports the old or new snapshot as chosen by assumption.
A failure after successful directory `fsync` is `CommittedButQuarantined`, not aborted.

### Deterministic startup and indeterminate recovery

After passphrase entry, startup recovery runs under the stable lock before any other custody
operation. It authenticates rather than trusting filenames or modification times. The candidate
set is every directory entry with the exact temporary prefix, opened relative to the pinned
directory. A security-metadata violation is a conflict, not removable garbage.

With one authenticated live parent, a candidate is an **eligible child** only when it fully opens
under the same passphrase, has the same file ID, network and principal, names the exact live-file
digest, and increments generation exactly once. A candidate that is truncated, fails outer
structure, or fails authentication under the already-proved live-store passphrase is a
**removable artifact**. A fully authenticated candidate that is not the exact eligible child is a
**conflicting candidate**.

With an absent live target, a candidate is eligible only when it is the sole candidate and is
either generation-one with the exact zero predecessor and no pending restore source, or is an exact
Restore Successor backed by the one durable Restore Quarantine source defined below. A canonical
directory-fsynced Restore Quarantine source is **pending** whenever the live target is absent; a
temporary quarantine copy is not intent. At most one pending source is permitted. Without an
authenticated live file, recovery never deletes or chooses among candidate artifacts: a wrong
passphrase cannot be safely distinguished from an unrelated or damaged candidate.

The following priority-ordered cases are mutually exclusive:

| Priority | Complete observation | Required result |
|---:|---|---|
| 1 | Live pathname, candidate, or pending restore source violates owner, mode, type, link, path, mount, or pinned-inode rules; or a present live file cannot be fully authenticated | `RecoveryConflict`; retain artifacts and emit nothing |
| 2 | Authenticated live plus any conflicting candidate, or more than one eligible child | `RecoveryConflict`; retain artifacts and emit nothing |
| 3 | Authenticated live plus exactly one eligible child and zero conflicting candidates | remove every removable artifact by verified inode identity and directory-fsync; then file-fsync and publish that exact child through CAS, rename, and directory-fsync |
| 4 | Authenticated live plus no eligible or conflicting candidate | durably remove every removable artifact, file-fsync the live file, directory-fsync, reauthenticate and reconcile, then `Ready` |
| 5 | Absent live target, empty candidate set, and no pending restore source | directory-fsync and enter `AbsentReady`; only a new explicit generation-one operation may proceed |
| 6 | Absent live target, empty candidate set, and exactly one pending restore source | enter `RestorePending`; authenticate that exact source and re-run current-prefix reconciliation. Return `RestoreNoInstallableState` if empty, otherwise seal one fresh Restore Successor and run the normal commit protocol |
| 7 | Absent live target plus exactly one fully authenticated eligible generation-one or Restore Successor candidate | file-fsync and complete `RENAME_NOREPLACE` publication; a Restore Successor is re-reconciled before publication |
| 8 | Absent live target plus any other candidate/source combination, including multiple pending sources | `RecoveryConflict`; retain every artifact and emit nothing |

A surviving exact eligible successor is treated as potentially intended even if the previous
caller did not observe success; recovery completes those same candidate bytes rather than
reconstructing or reencrypting them. Multiple candidates are never resolved by newest generation,
timestamp, lexical name, or best effort. A previously renamed candidate is no longer a temporary
directory entry: the authenticated live-snapshot cases resolve it, and an exact prepared stage
inside that snapshot supplies retry identity. A new mutation cannot start until recovery produces
one authenticated `Ready` or `AbsentReady` state or a named fail-closed terminal result.

### Backup and independent verification

A Custody Backup is an immutable byte-for-byte copy of one committed Custody Snapshot. Creating a
backup does not seal again, draw randomness, change generation, or alter the live store.

Under the custody lock, the client authenticates the live snapshot and binds backup selection to
its exact Custody Snapshot Reference. It copies the exact bytes into a `0600` candidate named
`.participant-keystore-backup.tmp.<random16-hex32>` in a separately pinned, owned `0700`, qualified
local-ext4 backup directory; verifies size and digest;
file-fsyncs; publishes with `RENAME_NOREPLACE` to the exact canonical name
`participant-keystore-backup.<file_id-hex32>.g<generation-decimal>.<digest-hex64>.pks1`; and
directory-fsyncs. Hex is lowercase and fixed-width; generation is canonical unsigned decimal with
no leading zero. An existing final name is accepted only
when its exact complete bytes and digest match. The protocol never overwrites an existing backup.
Opaque transport copies outside that directory carry no reference durability claim until reopened
and verified.

Backup creation is not backup verification. A backup is **Verified** only when an isolated client
with no live-key lookup path:

1. checks its expected file ID, generation, digest, network, and principal;
2. applies all structural, KDF, AEAD, nested-record, public-key-rederivation, fingerprint, uniqueness,
   and prepared-stage checks; and
3. reconciles its complete key/prepared set against one explicitly verified current
   Network-Converged ledger prefix.

A digest-only copy check, successful write, successful AEAD open without semantic validation, or
comparison to a historical ledger prefix is not Verified. If the live store advances while the
isolated check runs, the named backup remains a verified snapshot of its generation but cannot be
reported as the latest generation.

### Restore Quarantine, reconciliation, and re-encryption

Restore never installs raw backup bytes and never merges or overwrites a live store. The caller must
supply the expected backup `(file_id, generation, digest, network, principal)`; a passphrase alone
does not select or authorize a store. Restore-source creation requires an absent live target and no
different pending canonical source; an existing source can resume only when its exact complete
bytes and expected tuple match. Within a separately pinned, qualified, owned `0700` directory
named `restore-quarantine`, the client first copies the opened backup descriptor to
`.participant-keystore-restore-source.tmp.<random16-hex32>`, verifies exact bytes and digest,
file-fsyncs it, and publishes with `RENAME_NOREPLACE` to
`participant-keystore-restore-source.<file_id-hex32>.g<generation-decimal>.<digest-hex64>.pks1`
before quarantine-directory `fsync`. The same lowercase/fixed-width hex and canonical-decimal
rules as backup names apply. An existing final source is accepted only when its exact complete
bytes match. This directory-fsynced immutable source, created by the explicit restore operation,
is the sole durable restore intent; no inferred flag or filename timestamp can replace it. Normal
key lookup never searches quarantine.

While still quarantined, restore authenticates the complete file, enforces every bound, rederives
all public identities and fingerprints, validates every prepared byte/digest relationship, and
requires one current verified Network-Converged ledger prefix. Without that prefix, the backup
remains quarantined and no live file is created.

Reconciliation classifies each exact tuple:

- ledger-matched `Active` or `Retired` private material is retained as a Bound entry, while the
  lifecycle label remains ledger-derived;
- a ledger-proven `Revoked` tuple loses its secret from the live successor and becomes the exact
  public Tombstone;
- a ledger-matching still-current prepared request/submission retains its exact stored bytes;
- a stale, conflicting, or orphaned prepared/bound secret remains only in the durable raw
  quarantine artifact and is never a live key source; and
- every ledger-recorded required key absent from the backup produces a reported Custody Gap and
  makes the affected operation fail closed.

Restore then seals a new successor using the same exact passphrase, same `file_id`,
`generation = backup.generation + 1`, previous reference equal to the exact backup digest, and a
fresh salt and snapshot ID. It installs only into an absent live target through the standard
`RENAME_NOREPLACE` and directory-fsync commit. A present live target rejects; v1 has no automatic
merge, choose-newest, destructive replacement, passphrase-change, or rewrap workflow. Thus a stale
but authentic backup may yield a Custody Gap or quarantined local-only state, but it cannot roll
back ledger lifecycle, reactivate a key, substitute a key version, or authorize historical release.

A Restore Successor is eligible across restart only when exactly one canonical quarantine source
fully authenticates under the same passphrase and matches the candidate's predecessor digest,
file ID, source generation, network, and principal; the candidate generation is exactly source
generation plus one; and re-running deterministic reconciliation against one current verified
Network-Converged prefix produces the candidate's exact plaintext entry list. A missing, duplicate,
ambiguous, mismatched, or no-longer-current source/candidate pair remains quarantined and returns
`RecoveryConflict`; it is never published by assumption.

If deterministic reconciliation would leave zero live entries, the client creates no Restore
Successor and leaves the target absent and the source durable in Restore Quarantine. It returns the
distinct fail-closed result `RestoreNoInstallableState`; it does not encode an empty entry list,
invent a tombstone, or treat an empty store as a successful restore. The pending source continues
to block generation-one creation and selection of a different restore source.

Generation and predecessor digest detect malformed local lineage and many accidental ordering
errors. They cannot detect replay of an older authentic directory or every old backup whose ledger
key set still matches. Cloned stores can create branches with the same `file_id`. External
monotonic rollback/fork detection remains future work.

### Durable Revoked lifecycle and tombstone rewrite

At the first verified Network-Converged prefix where a locally held key is `Revoked`, the client
immediately denies every use based on ledger state, before local rewrite. It then enters
`RevocationRewritePending` and commits a freshly sealed successor replacing the secret-bearing entry
with its public Tombstone. Until that commit or any indeterminate outcome is resolved, the custody
component emits no private operation or prepared bytes. Restore performs the same rewrite before
installation.

Removing secret bytes from the new plaintext snapshot is hygiene only. Old encrypted snapshots,
backups, quarantine artifacts, filesystem journals, copy-on-write history, swap, crash dumps, SSD
remapping, and attacker copies may remain. The suite makes no physical secure-erasure, remote-wipe,
or revocation-of-copied-secret claim.

### Errors, logging, and secret lifetime

Opening an existing file collapses a wrong passphrase, AEAD authentication failure, noncanonical
decrypted record, key/public mismatch, fingerprint mismatch, and expected-identity mismatch into
one external `KeystoreOpenFailed` result. It exposes no partial plaintext or secret-bearing
diagnostic. Invalid passphrase syntax can be rejected before opening because it describes caller
input, while allocation, lock, path, unsupported-filesystem, and I/O incapacity remain separate
redacted operational categories. `CommitIndeterminate`, `CommittedButQuarantined`,
`RecoveryConflict`, `RestoreNoInstallableState`, and Custody Gap remain distinct because they
mandate different fail-closed recovery actions. No timing-equivalence claim follows.

Passphrases, roots, file keys, private keys, DEKs, plaintext, and prepared signing material never
enter logs, tracing fields, panic messages, metrics labels, diagnostics, node/server storage,
Ledger Journal, Oxigraph, indexes, API responses, JWT/account state, core dumps claimed as safe, or
plaintext backups. Best-effort zeroization is required on returning and unwind paths, but cannot
cover `abort`, `SIGKILL`, registers, allocator copies, kernel buffers, swap, suspend images, or a
compromised process. Constant-time execution, locked memory, HSM protection, and secure deletion are
not v1 claims.

### Exact reference dependency and toolchain gate

The future conforming implementation pins exact versions, disables defaults, enables only reviewed
features, commits `Cargo.lock`, and tests with `--locked`. The keystore-specific additions are:

```toml
argon2 = { version = "=0.5.3", default-features = false, features = ["zeroize"] }
ed25519-dalek = { version = "=2.2.0", default-features = false, features = ["fast", "zeroize"] }
rustix = { version = "=1.1.3", default-features = false, features = ["std", "fs", "process"] }
```

The implementation reuses ADR 0034's exact/default-off
`p256=0.14.0`, `chacha20poly1305=0.11.0`, `hkdf=0.13.0`, `sha2=0.11.0`,
`getrandom=0.4.3`, `rand_core=0.10.1`, and `zeroize=1.8.2` selections rather than creating a second
crypto family. `getrandom::SysRng` is the fallible production entropy source; deterministic RNG is
permitted only in clearly labelled fixtures. The full selected stack requires a project MSRV of at
least Rust 1.85. Current broad dependencies, current Rust 1.70+ documentation, or a coincidental
lockfile resolution are not conformance.

Any dependency, feature, syscall binding, target, kernel, or filesystem change requires rerunning
the complete vector and crash campaign. A later dependency update may be recorded without changing
the wire format only after it reproduces every canonical vector and invariant; activation cannot
silently drift.

## Rationale

A whole encrypted snapshot gives the small bounded reference store one object to authenticate,
copy, compare, publish, and recover. Fresh per-snapshot derivation makes a fixed zero nonce safe by
construction while exact stored-file retry avoids resealing ambiguity. The explicit prepared states
make “durable before disclosure” observable across a crash without moving participant secrets into
the ledger or node.

File `fsync`, atomic rename, and directory `fsync` solve different parts of the local durability
problem. Treating the final directory `fsync` as the only local commit point makes pre-send gating
and recovery conservative and testable. A separate stable lock inode survives replacement of the
live snapshot, unlike a lock attached to the file being renamed.

Restoring through quarantine and a newly encrypted successor prevents raw stale backup bytes from
becoming live authority. Ledger reconciliation preserves the central invariant that local presence
never selects lifecycle state, while still reporting irrecoverable Custody Gaps honestly.

The narrow ASCII and ext4 choices trade portability and human usability for byte-level and
crash-evidence closure in the thesis reference system. Unicode passphrases, other filesystems, and
multi-device workflows can be designed later without pretending that unspecified normalization or
rename semantics are reproducible now.

## Consequences

- The accepted client store format, KDF, file key, AEAD, canonical records, limits, prepared stages,
  local compare-and-swap, commit point, recovery, backup, restore, and Revoked rewrite are now fixed.
- The store rewrites at most 262,437 bytes and performs a fixed 64 MiB Argon2id operation per open or
  new seal. Exact performance and UX cost must be measured; it is not inferred from parameters.
- The clear header exposes that a file is PKS1 plus its suite, generation, file/snapshot IDs, KDF
  parameters, salt, and plaintext length. Network, principal, records, and secrets remain encrypted.
- One principal per store and one exclusive writer eliminate merge semantics but rule out
  multi-principal bundles, active-active devices, and concurrent writers.
- A committed backup is a byte-identical historical snapshot; a restore is a new encrypted lineage
  successor. Neither changes ledger history or proves current usability without reconciliation.
- The exact format does not activate privacy. Implementation, locked dependencies/toolchain,
  historical decoder, startup self-test, crash evidence, and end-to-end evidence remain mandatory.
- Node Final Admission, Verified Journal Replay, and three-node PoA convergence remain walletless;
  custody success/failure is participant-side capability and never follower-validation input.
- PBFT, heterogeneous/SPV bridging, human usability study, operational deployment, production-pilot
  controls, multi-device sync, HSM/KMS/OS keychains, recovery escrow, governance/social recovery,
  external rollback witness, passphrase rewrap, live-store replacement, and secure erasure remain
  future work or later milestones.

## Alternatives Rejected

- **Serde/JSON/CBOR or an extensible field map:** not byte-canonical and permits parser/default drift.
- **Implementation-default or per-device KDF parameters:** obstructs reproducible vectors and creates
  unreviewed downgrade/negotiation states.
- **Random stored nonce with a reusable file key:** adds persistent uniqueness state without benefit;
  v1 instead derives a fresh one-use key and pins zero12.
- **In-place overwrite or file-only flush:** permits partial state or a nondurable directory entry.
- **Locking the live snapshot:** rename replaces its inode while old descriptors keep the old lock.
- **Choosing the newest recovery file:** timestamps/generation alone cannot resolve sibling or
  adversarial candidates safely.
- **Directly installing or merging a backup:** could resurrect stale prepared or Revoked material and
  lets local labels override ledger state.
- **Node/server escrow or recovery override:** violates the client-only trust boundary and walletless
  verification model.
- **Unicode “as typed” in v1:** lacks a pinned normalization form and Unicode-version contract.
- **Treating generation/digest as anti-rollback:** an attacker or operator can replay an authentic
  older directory without an external monotonic witness.

## Required Evidence Before Activation

The conformance package must be corrected, reproducible, machine-readable, and generated from a
clean locked build. It includes:

1. RFC 9106 Argon2id Section 5.3 KATs plus suite vectors for generation-one Prepared Governance,
   Prepared Node Submission, Bound Ed25519 and P-256 entries, retained Retired material, Revoked
   Tombstone, exact backup copy, quarantined restore, and restored re-encryption;
2. for every positive vector: labelled test-only passphrase ASCII/hex, IDs, generation/parent,
   salt, decoded header, exact header/AAD/info/plaintext/ciphertext/tag/file hex, `root32`, PRK,
   `K_file`, complete-file digest, rederived public/fingerprint values, and exact retry bytes;
3. a strict negative corpus for every bit-flipped/truncated/extra/misordered/duplicate/unknown or
   over-limit field; wrong passphrase/identity; nonexact KDF value; malformed private/public value;
   duplicate key; nested digest/proof/transition mismatch; stale parent; revoked-secret
   resurrection; and attempted reseal under one construction;
4. parser and state-machine property/fuzz tests proving bounded allocation, strict canonicality,
   unique ordering, no partial plaintext, and preflight rejection before Argon2 for oversized or
   structurally invalid files;
5. deterministic syscall failure and `SIGKILL` injection at lock, create, every partial-write
   offset, readback, file-fsync, target-CAS, rename, pathname verification, directory-fsync,
   reopen, backup, quarantine, restore, and Revoked-rewrite boundaries;
6. concurrent-process, permission, owner/mode, symlink, hardlink, mount-crossing, `ENOSPC`, `EDQUOT`,
   `EIO`, read-only, unsupported-filesystem, multiple-child, and out-of-band-target fixtures;
7. abrupt-power-loss and replay tests on the declared Linux/ext4 kernel, mount options, and storage
   stack, showing only the exact old or exact new authenticated snapshot and no output before a
   directory-fsynced prepared state;
8. backup/restore evidence proving no live merge/overwrite, isolated full verification, current
   Network-Converged reconciliation, Custody Gap reporting, stale-stage quarantine, fresh restore
   ciphertext, and no live Revoked secret;
9. secret-boundary tests proving no participant secret/plaintext in nodes, journals, replicas,
   Oxigraph, indexes, JWT/account state, APIs, logs, traces, diagnostics, panic text, or server
   backups, plus best-effort arena/buffer wipe instrumentation without upgrading it to erasure; and
10. an evidence manifest recording generator/verifier commit and exact commands, `rustc`, Cargo,
    target, kernel, filesystem/mount/storage details, `Cargo.lock` SHA-256, dependency versions,
    enabled features/checksums, fixture/result hashes, raw results, and an independent vector
    verifier outcome.

End-to-end evidence must show client generation, both prepared stages, exact retry, binding,
purpose-specific signing/opening, rotation retention, revocation rewrite, restart, backup, restore,
and Custody Gap behavior while the three validator nodes remain secret-free and converge over the
same exact Admitted Block Envelopes. ADR acceptance is not that evidence.

## Related Decisions

- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) limits the target to
  a thesis-defensible end-to-end reference system and sends production controls to later milestones.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) defines the distinct public
  ledger commit authority; this ADR defines only a participant-local Custody Store Commit.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) fixes the stable participant
  identity and purpose/version key binding stored here only as non-authoritative custody metadata.
- [ADR 0029](./0029-require-participant-authorization-after-governed-bootstrap.md) defines the exact
  participant and bootstrap authorization boundaries represented by prepared stages.
- [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md) fixes immutable key versions and
  ledger-derived Active, Retired, and Revoked behavior.
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) explains why exact old
  wrapping private-key versions remain necessary.
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) fixes the public
  privacy codec, key algorithms, fingerprints, proofs, HPKE, and release rules used by this store.
- [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) fixes the
  client-only, generate-only trust boundary, operation gating, retention, and loss/compromise model.

## Implementation Status

Accepted design only. No conforming `ParticipantKeystoreSuiteV1`,
`CanonicalParticipantKeystoreEncodingV1`, client-only custody process, exact codec, guarded Argon
arena, stable writer lock, snapshot commit/recovery engine, prepared-state retry, backup verifier,
restore quarantine, Revoked rewrite, startup gate, or evidence package exists.

Current `src/wallet.rs` remains nonconforming server-side scaffolding: raw secret-bearing Wallet
values can be cloned/serialized, a plaintext file-level master key exists, wallet files are directly
overwritten, plaintext JSON backup can contain private bytes/shared secrets, and the private at-rest
format is only nonce plus AEAD(JSON). Server handlers can own/sign/decrypt through this wallet.
Current dependencies are broad and older than the accepted stack, no project MSRV/toolchain is
declared, and no production stable-lock dependency is direct. Existing wallet/persistence/backup
tests do not prove this ADR.

No Rust source, `Cargo.toml`, `Cargo.lock`, toolchain declaration, runtime data, benchmark result, or
conformance claim changed with this decision. Privacy activation remains fail closed.

## References

- [RFC 9106: Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications](https://www.rfc-editor.org/rfc/rfc9106.html)
- [RFC 5869: HKDF](https://www.rfc-editor.org/rfc/rfc5869.html)
- [RFC 8439: ChaCha20 and Poly1305 for IETF Protocols](https://www.rfc-editor.org/rfc/rfc8439.html)
- [NIST SP 800-63B: Authentication and Authenticator Management](https://pages.nist.gov/800-63-4/sp800-63b/authenticators/)
- [Linux `openat2(2)`](https://man7.org/linux/man-pages/man2/openat2.2.html)
- [Linux `open(2)`](https://man7.org/linux/man-pages/man2/open.2.html)
- [Linux `flock(2)`](https://man7.org/linux/man-pages/man2/flock.2.html)
- [Linux `rename(2)`](https://man7.org/linux/man-pages/man2/rename.2.html)
- [Linux `fsync(2)`](https://man7.org/linux/man-pages/man2/fsync.2.html)
- [RustCrypto `argon2` 0.5.3 source](https://docs.rs/crate/argon2/0.5.3/source/src/lib.rs)

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-31
