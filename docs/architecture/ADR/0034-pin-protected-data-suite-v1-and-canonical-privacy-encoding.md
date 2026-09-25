# ADR 0034: Pin ProtectedDataSuiteV1 and Canonical Privacy Encoding

**Status:** Accepted
**Date:** 2026-08-31
**Context:** One interoperable, fail-closed byte and cryptographic contract for `PrivacyControlV1`
**Custody boundary fixed by:** [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) — participant private keys and protected-data opening remain in durable client-only custody; nodes, services, and the journal never possess them.
**Keystore suite fixed by:** [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) — the exact at-rest keystore format, KDF/AEAD construction, whole-snapshot commit, backup, and restore design are fixed; implementation, activation, and conformance evidence remain pending.

---

## Decision

The bounded thesis-reference Network Profile has exactly one protected-data suite named
`ProtectedDataSuiteV1` and exactly one privacy-control codec named
`CanonicalPrivacyEncodingV1`. They are one indivisible profile choice: a node either implements
and verifies the complete contract in this decision or treats `PrivacyControlV1` as unsupported.
An object, grant, wallet, API request, node configuration, or implementation cannot negotiate,
substitute, omit, or partially enable algorithms or encodings.

This decision completes the design contract required by ADRs 0028-0033. It does **not** activate
the profile and is not implementation evidence. Activation remains fail closed until all six
privacy transitions, the total reducer, the canonical decoder, public verifiers, participant-side
constructors, journal replay, Network-Converged release path, dependency lock, and required vectors
exist and pass the evidence gate below. A later cryptographic or encoding change requires a new
suite identifier and a governed new ledger epoch or Network Profile; committed v1 bytes are never
reinterpreted in place.

### CanonicalPrivacyEncodingV1

Every suite, context, proof, core, complete transition, envelope, and release-evidence record uses
the following binary encoding. JSON, TOML, CBOR, protobuf, Serde output, Rust struct layout, DER,
hex, base64, and platform-native integer or `usize` representations are never consensus codecs.

```text
record = 0x50 0x43 0x56 0x31       // ASCII "PCV1"
         || record_tag:u8
         || field_count:u8
         || field[0]
         || ...

field  = field_tag:u8
         || value_length:u32be
         || value[value_length]
```

Within every closed schema below, field tags are `0x01`, `0x02`, ... in the listed order and
`field_count` is exactly the number of listed fields. A decoder rejects a wrong magic, record tag,
field count, field order, field tag, fixed-field length, non-minimal or invalid value, unknown or
duplicate field, absent required field, forbidden empty value, nested noncanonical record,
over-limit allocation, or trailing byte. There are no optional fields or ignored extension fields.
The only zero-length value explicitly permitted is HPKE associated data, which is an algorithm
input and is not serialized as a record field.

Unsigned integers are fixed-width big-endian: key versions use `u32be`; ledger positions and
privacy revisions use `u64be`. A hash, digest, fingerprint, commitment, or derived identifier is
raw 32 bytes. A UUID is raw 16 bytes in RFC 9562 network byte order, never text. `network_id` and
`profile_id` are `1..=128` ASCII bytes matching `[a-z0-9][a-z0-9._:-]*`; their TOML spelling is not
hashed directly. The parent-envelope reference is exactly 33 bytes: `0x00 || zero32` for genesis
or `0x01 || envelope_hash32` otherwise. All other empty strings, nil UUIDs, alternate textual
spellings, and unknown enum values reject.

The outer Admission Kind identifier is exactly the ASCII bytes `PrivacyControlV1`. The transition
schema identifier is exactly `CanonicalPrivacyEncodingV1`, and the suite identifier is exactly the
20 ASCII bytes `ProtectedDataSuiteV1`. These identifiers appear only where the schema lists them;
implementations must not add redundant local version fields.

Scoped one-byte values are closed:

| Scope | Values |
|---|---|
| Transition variant | `RegisterPrincipal=0x01`, `BindParticipantKey=0x02`, `RevokeParticipantKey=0x03`, `CreateProtectedObject=0x04`, `GrantAccess=0x05`, `RevokeGrant=0x06` |
| Participant key purpose | `privacy-authorization=0x01`, `privacy-key-wrapping=0x02` |
| Key lifecycle | `Active=0x01`, `Retired=0x02`, `Revoked=0x03` |
| Authorizer | `ParticipantBootstrapAuthorizationV1=0x01`, `Participant=0x02` |
| Key algorithm | `Ed25519=0x01`, `P256=0x02` |
| Public-key encoding | `Ed25519Raw32=0x01`, `Sec1UncompressedP256=0x02` |
| Possession proof | `Ed25519Raw64=0x01`, `EcdsaP256Sha256RawLowS64=0x02` |
| Permission | `ReadProtectedObjectV1=0x01` |
| Envelope kind | `Owner=0x01`, `Grant=0x02` |
| Live release path | `Owner=0x01`, `Grantee=0x02` |
| Profile suite code | `ProtectedDataSuiteV1=0x01` |

Record tags are also closed:

| Tag | Record |
|---:|---|
| `0x01` | `DomainHashInputV1` |
| `0x02` | `KdfInfoV1` |
| `0x03` | `KdfContextInfoV1` |
| `0x04` | `ProtectedContentMacInputV1` |
| `0x05` | `KeyFingerprintInputV1` |
| `0x10` | `ParticipantKeyBindingV1` |
| `0x11` | `ParticipantKeyReferenceV1` |
| `0x12` | `BootstrapGovernanceKeyReferenceV1` |
| `0x20` | `ObjectEncryptionContextV1` |
| `0x21` | `ProtectedPayloadCiphertextV1` |
| `0x22` | `OwnerDekHeaderV1` |
| `0x23` | `OwnerDekEnvelopeV1` |
| `0x24` | `GrantDeliveryContextV1` |
| `0x25` | `GrantDekHeaderV1` |
| `0x26` | `GrantDekEnvelopeV1` |
| `0x30`-`0x35` | unsigned cores in transition-variant order |
| `0x40`-`0x45` | complete transitions in transition-variant order |
| `0x50` | `OwnerLivePrivacyReleaseEvidenceV1` |
| `0x51` | `GranteeLivePrivacyReleaseEvidenceV1` |
| `0x52` | `LivePrivacyReleaseResponseV1` |

`DomainHashInputV1` contains `(domain_ascii, payload_bytes)`. Define:

```text
DHash(domain, payload) = SHA-256(ENCODE(DomainHashInputV1,
                                       [domain_ascii, payload_bytes]))
```

Every ADR 0030-0033 expression previously written as `SHA-256("domain" || X ...)` is instantiated
by this `DHash` rule. In the formulas below, each `C_*` value is the exact complete encoded unsigned
core record for that variant, including its `PCV1` framing. `T32 || C_*`, `G32 || C_*`, and similar
payloads are exactly one raw fixed-width 32-byte digest followed immediately by that one
self-framing core record; they are not wrapped in an additional record before becoming the
`payload_bytes` field of `DomainHashInputV1`:

```text
T_register = DHash("provchain/privacy-bootstrap/transition-id/v1", C_register)
G_register = DHash("provchain/privacy-bootstrap/governance-authorization/v1",
                   T_register || C_register)
P_register = DHash("provchain/privacy-bootstrap/key-possession/v1",
                   T_register || C_register)

T_bind = DHash("provchain/privacy-key-bind/transition-id/v1", C_bind)
A_bind = DHash("provchain/privacy-key-bind/participant-authorization/v1",
               T_bind || C_bind)
P_bind = DHash("provchain/privacy-key-bind/key-possession/v1", T_bind || C_bind)

T_key_revoke = DHash("provchain/privacy-key-revoke/transition-id/v1", C_key_revoke)
A_key_revoke = DHash("provchain/privacy-key-revoke/participant-authorization/v1",
                     T_key_revoke || C_key_revoke)

T_object_create = DHash("provchain/protected-object/create-transition-id/v1", C_object_create)
A_object_create = DHash("provchain/protected-object/owner-authorization/v1",
                        T_object_create || C_object_create)

G_grant = DHash("provchain/privacy-grant/grant-id/v1", C_grant)
A_grant = DHash("provchain/privacy-grant/owner-authorization/v1", G_grant || C_grant)

T_grant_revoke = DHash("provchain/privacy-grant/revoke-transition-id/v1", C_grant_revoke)
A_grant_revoke = DHash("provchain/privacy-grant/revoke-owner-authorization/v1",
                       T_grant_revoke || C_grant_revoke)
```

`T_register`, `T_bind`, `T_key_revoke`, `T_object_create`, `G_grant`, and `T_grant_revoke` are the
six derived identifiers stored in complete transitions. The other values are the exact digests
signed or proved under ADRs 0030-0033. These formulas preserve their accepted dependency graphs
while removing both raw-domain-concatenation and nested-record ambiguity.

### Closed key and transition records

`ParticipantKeyReferenceV1` contains, in order:

1. principal UUID16;
2. purpose tag1;
3. version `u32be`;
4. algorithm tag1;
5. encoding tag1; and
6. fingerprint32.

`ParticipantKeyBindingV1` contains those six fields followed by possession-proof-scheme tag1 and
canonical public-key bytes. `BootstrapGovernanceKeyReferenceV1` contains profile role
`privacy-bootstrap=0x01`, version `u32be`, Ed25519 algorithm and raw-encoding tags, and
fingerprint32. The complete referenced profile binding supplies its public key; a transition may
not substitute it.

Every fingerprint carried by these key records is exactly:

```text
fingerprint = DHash(
    "provchain/privacy-key/fingerprint/v1",
    ENCODE(KeyFingerprintInputV1,
           [scheme_identifier, algorithm_tag, encoding_tag, public_key_bytes]))
```

The scheme identifier is exactly `PrivacyBootstrapGovernanceEd25519V1` for the Network-Profile-
bound privacy-bootstrap governance key, `PrivacyAuthorizationEd25519V1` for Ed25519 participant
authorization keys, and `ProtectedDataSuiteV1` for P-256 wrapping keys. The bootstrap formula uses
the exact public key from the complete referenced profile binding; a transition cannot supply
different input bytes. Ed25519 public keys are raw 32 bytes and their signatures and possession
proofs are raw 64 bytes under strict Ed25519 verification; DER or an alternate point/key encoding
rejects. Global key-reuse checks compare canonical public bytes as well as fingerprints, so a
different scheme domain cannot conceal reused key material.

Every unsigned transition core begins with this exact common field order:

1. protocol identifier `PrivacyControlV1`;
2. transition-schema identifier `CanonicalPrivacyEncodingV1`;
3. transition variant tag1;
4. `network_id`;
5. parent `profile_id`;
6. parent Network Profile content hash32;
7. expected ledger position `u64be`;
8. expected privacy revision `u64be`;
9. parent Ledger Prefix Hash32; and
10. parent-envelope reference33.

The remaining ordered fields and complete-record forms are:

| Variant / record tags | Unsigned-core suffix | Complete transition fields |
|---|---|---|
| `RegisterPrincipal` / `0x30`, `0x40` | new principal UUID16; initial authorization `ParticipantKeyBindingV1`; bootstrap authorizer tag; `BootstrapGovernanceKeyReferenceV1` | core bytes; `T32`; governance Ed25519 signature64; introduced-key Ed25519 possession signature64 |
| `BindParticipantKey` / `0x31`, `0x41` | affected principal UUID16; introduced `ParticipantKeyBindingV1`; participant authorizer tag; active parent authorization `ParticipantKeyReferenceV1` | core bytes; `T32`; participant Ed25519 authorization signature64; introduced-key possession proof64 |
| `RevokeParticipantKey` / `0x32`, `0x42` | affected principal UUID16; target principal UUID16; target purpose tag; target version `u32be`; participant authorizer tag; active parent authorization key reference | core bytes; `T_revoke32`; participant Ed25519 authorization signature64 |
| `CreateProtectedObject` / `0x33`, `0x43` | object UUID16; owner UUID16; active owner authorization key reference; active owner wrapping-key reference; `O32`; complete payload record; `EPC32`; complete owner envelope; participant authorizer tag | core bytes; `T_object_create32`; owner Ed25519 authorization signature64 |
| `GrantAccess` / `0x34`, `0x44` | object UUID16; parent-derived owner UUID16; grantee UUID16; permission tag; complete `GrantDekEnvelopeV1` as `D`; participant authorizer tag; active owner authorization key reference | core bytes; `G32`; owner Ed25519 authorization signature64 |
| `RevokeGrant` / `0x35`, `0x45` | grant identifier32; parent-derived object UUID16; grantee UUID16; owner UUID16; permission tag; grant creation position `u64be`; participant authorizer tag; active owner authorization key reference | core bytes; `T_grant_revoke32`; owner Ed25519 authorization signature64 |

The variant tag inside a core must match its record tag and complete-record tag. The derived
identifier and signature formulas, authorization matrix, parent-state equality rules, reducer
effects, and excluded outer PoA/envelope evidence remain exactly those in ADRs 0029-0033. Status,
effective position, timestamps, reasons, aliases, wallet locators, account fields, and arbitrary
metadata are never caller fields.

### Protected content, object identity, and one-use key schedule

Protected Content Bytes are an opaque byte string of `1..=262144` bytes. V1 performs no Unicode,
RDF, media-type, or line-ending normalization; no compression; and no padding. The exact plaintext
length is therefore visible from ciphertext length. Empty or oversized content rejects locally
before candidate construction.

The Object Identifier is an RFC 9562 UUIDv4 generated from an operating-system CSPRNG before
encryption and encoded as raw UUID16. It must carry version bits `0100`, variant bits `10`, be
non-nil, and be unique at the verified parent. Public validators can verify bits and ledger
uniqueness but cannot prove entropy quality.

For every newly constructed object, the participant generates a fresh 32-byte root DEK from a
fallible operating-system CSPRNG. The DEK is never reused for another object or regenerated retry.
Define:

```text
salt = SHA-256(ASCII("provchain/protected-data/dek-extract-salt/v1"))
PRK  = HKDF-Extract-SHA256(salt, DEK32)

K_pcc = HKDF-Expand-SHA256(
    PRK,
    ENCODE(KdfInfoV1,
           ["provchain/protected-data/pcc-key/v1", "ProtectedDataSuiteV1"]),
    32)

K_payload = HKDF-Expand-SHA256(
    PRK,
    ENCODE(KdfContextInfoV1,
           ["provchain/protected-data/payload-key/v1",
            "ProtectedDataSuiteV1", O32]),
    32)
```

The full 32-byte Protected Content Commitment is:

```text
PCC = HMAC-SHA256(
    K_pcc,
    ENCODE(ProtectedContentMacInputV1,
           ["provchain/protected-data/pcc/v1", ProtectedContentBytes]))
```

PCC is computed before `O` and is independent of `O`, the payload nonce, ciphertext/tag, EPC,
either DEK envelope, every transition identifier, and every signature. Its keyed construction
avoids publishing a bare dictionary-testable hash of low-entropy content. A keyless validator can
check only its 32-byte canonical form and signed placement, not its correctness.

`ObjectEncryptionContextV1` contains the common fields 1-10 above with variant
`CreateProtectedObject`, followed by object UUID16, owner UUID16, exact owner wrapping-key
reference, payload-record tag `0x21`, suite identifier, owner-envelope tag `0x23`, suite
identifier, and PCC32. Then:

```text
O = DHash("provchain/protected-object/encryption-context/v1",
          ObjectEncryptionContextV1_bytes)
```

Payload encryption is exactly one invocation of IETF ChaCha20-Poly1305 from RFC 8439 with
`K_payload`, nonce `zero12 = 0x00 * 12`, Protected Content Bytes as plaintext, raw `O32` as
associated data, and the full 16-byte tag appended to the ciphertext. The zero nonce is safe only
because this decision makes `K_payload` one-use. A conforming constructor must not invoke Seal a
second time under that key. An exact retry reuses the already constructed byte-identical payload;
a rebuilt or changed candidate discards the object identifier, DEK, PRK, derived keys, and prior
cryptographic output and starts with new randomness.

`ProtectedPayloadCiphertextV1` contains suite identifier, recorded `zero12`, PCC32, and exact
ciphertext-and-tag bytes in that order. Its public commitment is:

```text
EPC = DHash("provchain/protected-data/encrypted-payload-commitment/v1",
            ProtectedPayloadCiphertextV1_bytes)
```

EPC is stored outside the payload record and can never enter its own hash input. Implementations
zeroize the DEK, PRK, and derived keys as soon as their required local construction or recovery
work finishes; zeroization is an implementation obligation, not proof of erasure.

### P-256 wrapping-key binding and one-time possession proof

The `privacy-key-wrapping` purpose has exactly one public-key scheme:

- algorithm `P256=0x02`;
- encoding `Sec1UncompressedP256=0x02`, exactly `0x04 || X32 || Y32`, 65 bytes;
- possession proof `EcdsaP256Sha256RawLowS64=0x02`; and
- the full 32-byte common fingerprint formula above with scheme identifier
  `ProtectedDataSuiteV1` and canonical `pk65`.

A generated private scalar is selected by a fallible CSPRNG-backed P-256 key-generation rejection
routine in `1..n-1`; it is not arbitrary bytes reduced modulo `n`. Registration and admission
reject a compressed or hybrid point, infinity, coordinate outside the field, off-curve point,
wrong length or prefix, invalid DH result, duplicate public key, or duplicate fingerprint.

The Participant Key Possession Proof signs the raw ADR 0031 possession digest `P32` using ordinary
ECDSA P-256 with SHA-256, so the ECDSA message representative is `SHA-256(P32)`. Prehash APIs and
DER are forbidden. Signing is deterministic RFC 6979. The wire proof is exactly `r32 || s32`, each
unsigned fixed-width big-endian with leading zeros, and both scalars must be in `1..n-1`. The
signer replaces high `s` by `n-s`; the verifier rejects high-S before ordinary ECDSA verification
and must not normalize then accept. The constants are:

```text
n      = FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
n_half = 7FFFFFFF800000007FFFFFFFFFFFFFFFDE737D56D38BCF4279DCE5617E3192A8
```

The proof verifies under the exact same P-256 public point later used by DHKEM. Reusing that scalar
for one binding-time ECDSA proof is a narrow registration proof-of-possession exception, not a
general dual-use key. A conforming participant client exposes only an atomic generate-and-bind proof path,
accepts this key's signature only in the target-purpose possession-proof slot, and disables the
signing path after binding while retaining the scalar solely for HPKE decapsulation. The ledger
cannot prove that external software never signs. This decision makes no general NIST, HSM, FIPS,
or independent-audit claim, and HSMs that enforce single-purpose usage may reject the design.

### HPKE owner and grant envelopes

Both envelope kinds use RFC 9180 Base mode `0x00` with exactly:

- KEM `0x0010`: DHKEM(P-256, HKDF-SHA256);
- KDF `0x0001`: HKDF-SHA256; and
- AEAD `0x0003`: ChaCha20Poly1305.

The recipient public key and fresh per-envelope ephemeral key use the same validated uncompressed
SEC1 format. Each Owner or Grant envelope wraps exactly the raw root DEK32. There is no serialized
HPKE nonce: one-shot sequence number zero derives it internally. The wire output is exactly
`enc65` plus `wrapped_dek_ciphertext48`, including the full 16-byte tag. Base, PSK, Auth, and
AuthPSK modes, any other curve, KDF, AEAD, compression, multi-recipient envelope, or caller nonce
reject.

`OwnerDekHeaderV1` contains, in order: suite identifier; network ID; parent profile ID and content
hash32; expected ledger position and privacy revision; parent Ledger Prefix Hash32 and parent
envelope reference33; object UUID16; owner UUID16; exact owner wrapping-key reference; PCC32; and
EPC32. `OwnerDekEnvelopeV1` contains suite identifier, `O32`, complete owner header bytes, `enc65`,
and wrapped-DEK ciphertext48.

`GrantDeliveryContextV1` contains the common fields 1-10 with variant `GrantAccess`, followed by
object UUID16; parent-derived owner UUID16; grantee UUID16; permission tag; referenced payload
suite identifier; PCC32; EPC32; exact grantee wrapping-key reference; grant-envelope tag `0x26`;
and suite identifier. Then:

```text
Q = DHash("provchain/privacy-grant/delivery-context/v1",
          GrantDeliveryContextV1_bytes)
```

`GrantDekHeaderV1` contains, in order: suite identifier; network ID; parent profile ID and content
hash32; expected ledger position and privacy revision; parent Ledger Prefix Hash32 and parent
envelope reference33; object UUID16; parent-derived owner UUID16; grantee UUID16; permission tag;
exact grantee wrapping-key reference; PCC32; and EPC32. `GrantDekEnvelopeV1` contains suite
identifier, `Q32`, complete grant header bytes, `enc65`, and wrapped-DEK ciphertext48. That whole
record is Grant Delivery Material `D`; final `G` is derived afterward and never enters `Q`, the
header, HPKE info, or the envelope.

HPKE context bytes are exact:

```text
owner_header_hash = DHash(
    "provchain/protected-data/hpke-owner-header/v1", OwnerDekHeaderV1_bytes)
owner_info = O32 || owner_header_hash             // exactly 64 bytes

grant_header_hash = DHash(
    "provchain/protected-data/hpke-grant-header/v1", GrantDekHeaderV1_bytes)
grant_info = Q32 || grant_header_hash             // exactly 64 bytes

hpke_aad = empty byte string
```

An exact retry reuses the complete envelope; every new envelope construction uses a fresh HPKE
ephemeral scalar. The owner constructor must locally open its new owner envelope, open the payload,
and check PCC/EPC before submission. A grant constructor must recover the existing root DEK through
the exact owner envelope, open and verify the existing payload and commitments, and then wrap that
same root DEK to the grantee. These client-side checks provide functional assurance but are not
facts that walletless Final Admission can prove.

### Admission, replay, release, and failures

At object creation and grant creation, the exact recipient wrapping key must be `Active` at the
verified parent. Live Privacy Release resolves the exact key referenced by the committed Owner or
Grant envelope at one verified Network-Converged prefix. It permits that key only when its current
status is `Active` or `Retired`; `Revoked` denies. It never substitutes the newest key. This
deliberately preserves access to immutable old envelopes after routine rotation. Because ADR 0031
makes `Retired` terminal, a retired key cannot later be disabled by `RevokeParticipantKey`; a
later-discovered compromise cannot be shut off through v1 managed release. Rewrap, new-object
recovery, or a revised lifecycle is future work.

`OwnerLivePrivacyReleaseEvidenceV1` contains network ID, profile ID and content hash32, converged
ledger position `u64be`, converged Ledger Prefix Hash32, object UUID16, authenticated requester
UUID16, selected recipient key reference, and owner release-path tag.
`GranteeLivePrivacyReleaseEvidenceV1` contains those same first eight semantic values, the grantee
release-path tag, and Grant Identifier32. They have different record tags and fixed field counts;
there is no optional grant field.
`LivePrivacyReleaseResponseV1` contains evidence bytes, the exact committed payload record, exactly
one applicable owner or grant envelope, and EPC32. Requester equality, current ownership, Active
grant status for the grantee path, key status, and all release-path equalities are evaluated at the
same prefix. The server never returns or logs plaintext, an unwrapped DEK, a private key, a
historical-view capability, or a decryption-oracle result.

Before emitting or accepting a response, the release boundary applies these deterministic equality
rules. Owner evidence requires an Owner envelope; Grantee evidence requires a Grant envelope, and
the other pairing rejects. Evidence network, profile identity/content hash, converged position and
prefix, object, requester, path, and selected recipient-key reference must equal the corresponding
committed object or Active-grant state and envelope-header values at that one prefix. For the
Grantee path, its Grant Identifier must resolve to the exact Active `GrantAccess` record whose
committed Grant envelope bytes are returned. The selected key reference must be byte-identical to
the envelope header's recipient reference. The response payload must be the object's exact
committed payload record; its suite and PCC must equal the envelope header, and recomputing EPC from
that payload must equal both the response EPC and envelope-header EPC. Any path, tag, reference,
state, committed-byte, PCC, suite, or EPC mismatch returns only `ProtectedDataOpenFailed` and no
release record.

Final Admission remains public and walletless. It can verify closed schemas and lengths; identifiers
and enum values; UUID bits and parent uniqueness; P-256 point validity, fingerprint, key reference
and status; deterministic low-S possession proof; `O`, `Q`, EPC, transition identifiers,
participant signatures, PoA evidence, and outer commitments. It cannot prove DEK entropy,
freshness or nonreuse; fixed-nonce safety; PCC or payload-tag correctness; HPKE decryptability or
internal info use; same-DEK wrapping across opaque envelopes; current private-key custody; secure
erasure; retroactive revocation; HPKE sender authentication, forward secrecy after recipient-key
compromise, or length hiding.

Malformed public canonical bytes or a deterministic public proof, bound, reference, status,
parent, or reducer failure is an invalid candidate and appends nothing. Missing or unsupported
suite code, verifier, dependency, vector self-test, historical decoder, storage, memory, I/O, or
local resource is node incapacity or unsupported/degraded state; it yields no global invalid verdict,
append, or Commit Receipt. Any participant-side entropy, key recovery, HPKE Open, payload AEAD,
PCC, or local self-check failure occurs before submission or is exposed externally only as
`ProtectedDataOpenFailed`, with no partial plaintext, secret-dependent detail, ledger mutation, or
secret logging. Collapsing the error is not by itself a constant-time or oracle-resistance proof.

Journal append plus `fsync` remains the sole commit point. Followers replicate and validate exact
opaque bytes; projections never regenerate cryptographic material. Verified Journal Replay uses
the exact suite and codec recorded for the historical parent, not the current library default or
profile. An unknown committed historical suite stops replay in unsupported/degraded state rather
than reinterpretation. Historical audit never becomes Live Privacy Release authority.

### Numeric limits

All limits apply before allocation and are consensus-visible. A profile may impose a lower whole-
envelope limit only if it thereby keeps `PrivacyControlV1` inactive.

| Item | Exact size or accepted range |
|---|---:|
| Network/profile identifier | `1..=128` bytes |
| UUID | 16 bytes |
| Hash, digest, fingerprint, PCC, EPC, `O`, `Q`, `T`, or `G` | 32 bytes |
| Key version | 4 bytes, value `1..=u32::MAX` |
| Position or revision | 8 bytes |
| Parent envelope reference | 33 bytes |
| Ed25519 public key | 32 bytes |
| P-256 public key or HPKE `enc` | 65 bytes |
| Authorization or possession signature | 64 bytes |
| DEK, PRK, or derived symmetric key | 32 bytes, never serialized except wrapped DEK plaintext input |
| Protected Content Bytes | `1..=262144` bytes |
| Payload nonce | 12 zero bytes |
| Payload ciphertext and tag | `17..=262160` bytes |
| Canonical payload record | `107..=262250` bytes |
| HPKE wrapped DEK ciphertext | 48 bytes |
| Key binding or key-reference record | at most 512 bytes |
| Object/delivery context or public header | at most 2048 bytes |
| Owner or Grant DEK envelope | at most 2304 bytes |
| Any non-create complete privacy transition | at most 8192 bytes |
| Complete `CreateProtectedObject` transition | at most 270336 bytes |
| Live release evidence | at most 4096 bytes |
| Complete Live Privacy Release response | at most 270336 bytes |

The thesis reference profile retains a whole Admitted Block Envelope limit of 1,048,576 bytes. The
270,336-byte transition ceiling leaves deterministic space for outer proposal, PoA, profile,
journal-framing, and receipt evidence; the complete envelope must still satisfy the exact parent
profile limit. Decoders stream hashes and state serialization where possible and never allocate
from an unchecked `u32`; Rust `usize` never enters canonical bytes.

### Reference implementation and activation evidence

The selected Rust reference stack is exact, default-off, and lockfile-controlled:

```toml
hpke = { version = "=0.14.0", default-features = false, features = ["alloc", "chacha", "nistp"] }
p256 = { version = "=0.14.0", default-features = false, features = ["ecdsa"] }
chacha20poly1305 = { version = "=0.11.0", default-features = false, features = ["alloc", "zeroize"] }
hkdf = { version = "=0.13.0", default-features = false }
hmac = { version = "=0.13.0", default-features = false }
sha2 = { version = "=0.11.0", default-features = false }
getrandom = { version = "=0.4.3", default-features = false, features = ["sys_rng"] }
rand_core = { version = "=0.10.1", default-features = false }
zeroize = { version = "=1.8.2", default-features = false }
```

`hpke`'s `nistp` feature also compiles curves that this wire profile rejects. Production code uses
explicit `*_with_rng` APIs and a fallible OS-CSPRNG path; it does not enable convenience APIs that
can turn entropy failure into an implicit panic. Exact direct pins, reviewed transitive versions,
checksums, and features enter the committed `Cargo.lock`, and validation runs with `--locked`.
Crate selection is reproducibility evidence, not an audit, FIPS certification, constant-time proof,
or permission to accept alternate wire algorithms.

These versions require Rust 1.85 or newer and edition-2024 dependency support. Before suite
implementation can activate, the project must set and test a project MSRV of at least 1.85 and
update its current public `Rust 1.70+` guidance. The current host toolchain may be newer, but that is
not a portable build claim. No dependency or MSRV file is changed by accepting this ADR.

Profile activation requires all of the following reproducible evidence:

1. RFC 9180 Appendix A.5.1 Base-mode P-256/HKDF-SHA256/AES fixture coverage for the KEM/KDF path,
   plus a project ChaCha20Poly1305 HPKE vector using the fixed selected AEAD;
2. RFC 8439 AEAD, RFC 5869 HKDF, and RFC 2104 HMAC known-answer tests;
3. RFC 6979 P-256/SHA-256 deterministic signing plus a separate protocol low-S vector and strict
   high-S, DER, prehash, invalid-scalar, and invalid-point rejection;
4. machine-readable cross-implementation project vectors for canonical fields and complete
   Register, Bind, RevokeKey, Create, Grant, and RevokeGrant bytes; PCC, `O`, payload, EPC, owner
   envelope, `Q`, grant envelope, `G`, release response, and every context-tamper case;
5. controlled owner and grantee recovery of the same DEK and plaintext, exact-retry equality,
   new-candidate inequality, revoked denial, Active/Retired release, and generic private failures;
6. walletless Final Admission, rejected-candidate non-mutation, append/`fsync` crash boundaries,
   restart replay, projection rebuild, and three-node byte/prefix/privacy-state equality; and
7. a checked-in manifest recording generator commit, exact command, Rust and target versions,
   `Cargo.lock` hash, crate checksums/features, fixture hashes, and raw machine-readable results.

Deterministic private scalars, DEKs, UUIDs, and RNG streams exist only in labelled test fixtures.
Production constructors never accept fixture RNGs, and evidence output never contains production
private keys, DEKs, PRKs, derived keys, or plaintext.

## Rationale

One mandatory suite removes downgrade and negotiation branches from the bounded reference system
and gives the thesis one reproducible claim surface. The keyed PCC hides equality for low-entropy
content from keyless observers, while the public EPC, `O`, `Q`, outer signatures, and journal hashes
commit exact durable bytes without pretending validators can inspect authenticated plaintext.

Deriving a one-use payload key from a fresh per-object root DEK makes a fixed zero nonce satisfy the
RFC 8439 uniqueness requirement without a probabilistic collision argument. HPKE Base mode gives a
standard recipient-encryption construction with fixed P-256 encodings and vector support. The
separate public P-256 possession proof closes the wrapping-key registration gap, while its narrow
one-time signature boundary acknowledges rather than hides the cross-purpose risk.

The closed codec makes hashes, proofs, restart replay, and three-node equality independent of
Serde, architecture, insertion order, and implementation language. Explicit bounds keep parsing
and evidence feasible under the current one-mebibyte profile.

## Consequences

- `PrivacyControlV1` now has a complete design-level suite and byte contract, but remains inactive
  until implementation and evidence satisfy every gate in this ADR.
- Objects and grants disclose identifiers, principals, recipient-key references, suite, record
  sizes, and exact protected-content length; v1 does not claim traffic or metadata hiding.
- Owner and grantee participant clients must durably retain exact wrapping private-key versions needed by
  immutable envelopes. A Retired key remains managed-release eligible and cannot later be revoked
  under the accepted lifecycle.
- HPKE Base supplies no sender authentication by itself. Participant authorization and the complete
  Admitted Block Envelope authenticate the submitted opaque bytes, not their hidden correctness.
- Public admission claims stay deliberately weaker than controlled functional-vector claims.
- Classical security is approximately 128-bit and not post-quantum.
- Algorithm agility, in-place migration, rewrap, proxy re-encryption, automatic re-encryption,
  ownership transfer, object mutation, expiry, hidden recipients, padding, cryptographic recall,
  server plaintext, HSM/KMS integration, FIPS validation, independent audit, and post-quantum
  replacement remain future work.
- PBFT, a heterogeneous or SPV bridge, a human usability study, operational deployment, and
  production-pilot controls remain later milestones under ADR 0015 and are not implied by this
  suite decision.

## Related Decisions

- [ADR 0005](./0005-use-chacha20-encryption.md) is the historical algorithm-family decision that
  this ADR narrows to exact IETF ChaCha20-Poly1305 semantics.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) keeps exact suite bytes in
  the sole append-plus-`fsync` commit authority.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) supplies the
  reject-or-complete-commit boundary.
- [ADR 0018](./0018-separate-node-commitment-from-network-convergence.md) defines the stable release
  barrier used here.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) requires
  byte-identical follower replication.
- [ADR 0024](./0024-commit-canonical-post-block-public-provenance-state.md) excludes protected
  ciphertext and envelopes from Public Provenance State.
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md) through
  [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) define the identity,
  transition, authorization, lifecycle, grant, and envelope structures instantiated here.
- [ADR 0035](./0035-keep-participant-private-keys-in-durable-client-only-custody.md) confines the
  private operations and exact Active/Retired key versions required by this suite to non-authoritative
  participant client custody.
- [ADR 0036](./0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md) fixes the exact
  local at-rest suite and snapshot-durability contract for those private key versions without
  changing the public protected-data codec.

## Implementation Status

Accepted architecture only. No conforming codec, `PrivacyControlV1` transition, P-256 wrapping-key
registry, PoP verifier, HPKE envelope, keyed PCC, one-use payload construction, Active/Retired
release path, participant client custody, startup vector gate, or historical suite decoder exists.
ADR 0036 fixes the exact at-rest `ParticipantKeystoreSuiteV1` design, but its implementation,
activation gate, and conformance evidence remain pending. Current code instead uses
partial XChaCha20-Poly1305/raw-shared-secret scaffolding with empty AAD and free-form key IDs; it is
not this suite. `NetworkProfile` has no suite or canonical privacy section, the current dependency
graph and public MSRV are not the pinned reference stack, and no activation evidence exists. No
implementation code, dependency, lockfile, or toolchain declaration changed with this decision.

## References

- [RFC 8439: ChaCha20 and Poly1305 for IETF Protocols](https://www.rfc-editor.org/rfc/rfc8439.html)
- [RFC 9180: Hybrid Public Key Encryption](https://www.rfc-editor.org/rfc/rfc9180.html)
- [RFC 5869: HKDF](https://www.rfc-editor.org/rfc/rfc5869.html)
- [RFC 2104: HMAC](https://www.rfc-editor.org/rfc/rfc2104.html)
- [RFC 6979: Deterministic DSA and ECDSA](https://www.rfc-editor.org/rfc/rfc6979.html)
- [RFC 7518 Section 3.4: raw ECDSA signature encoding](https://www.rfc-editor.org/rfc/rfc7518.html#section-3.4)
- [RFC 9562: UUIDs](https://www.rfc-editor.org/rfc/rfc9562.html)
- [RFC 9528 Section 9.2: cryptographic key separation](https://www.rfc-editor.org/rfc/rfc9528.html#section-9.2)
- [NIST SP 800-56A Rev. 3](https://csrc.nist.gov/pubs/sp/800/56/a/r3/final)
- [`hpke` 0.14.0](https://docs.rs/crate/hpke/0.14.0)
- [`p256` 0.14.0](https://docs.rs/crate/p256/0.14.0)

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-31
