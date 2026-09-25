# ADR 0005: Use ChaCha20-Poly1305 for Data Encryption

**Status:** Accepted
**Date:** 2026-01-28
**Context:** Private data encryption and owner-controlled visibility
**Clarified by:** [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md), [ADR 0028](./0028-make-privacy-control-ledger-authoritative.md), [ADR 0031](./0031-use-one-active-participant-key-per-purpose.md), [ADR 0032](./0032-make-privacy-grant-revocation-terminal-and-prospective.md), and [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md) — ChaCha20-Poly1305 is an encryption primitive, not an authorization model; the accepted structure uses one per-object DEK, one immutable payload ciphertext, and separate owner/grant envelopes. The Ed25519-derived owner-key and ACL passages below are historical proposals.
**Suite and codec fixed by:** [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md) — the mandatory `ProtectedDataSuiteV1` and canonical privacy encoding are fixed; implementation, Network Profile activation, and conformance evidence remain pending.

---

## Context

ProvChainOrg requires symmetric encryption for:

1. **Private Triples:** Encrypt sensitive RDF triples at rest
2. **Owner-Controlled Visibility:** Data owners control who can decrypt their data
3. **Performance:** Encryption/decryption must be fast (< 100µs for typical operations)
4. **Authenticated Encryption:** Must provide both confidentiality and integrity
5. **Compliance:** GDPR requirements for data protection

### Requirements

| Requirement | Priority | Target |
|-------------|----------|--------|
| **Performance** | High | Encryption/decryption < 100µs |
| **Security Level** | High | 256-bit key (128-bit security level) |
| **Authenticated** | High | AEAD (Authenticated Encryption with Associated Data) |
| **Ciphertext Integrity** | Medium | Detect tampering with encrypted data when key and nonce rules hold |
| **Simplicity** | Medium | Make key and nonce rules explicit and testable |

---

## Decision

**Use the ChaCha20-Poly1305 family as the accepted symmetric authenticated-encryption design family in ProvChainOrg.**

This ADR selects a primitive only. It does not authorize access or permit reuse of signing material
as an encryption key. ADR 0033 fixes the Protected Object and DEK-envelope topology, and ADR 0034
fixes the profile-declared cryptographic suite and canonical privacy encoding. Those decisions do
not by themselves demonstrate implementation, Network Profile activation, or conformance evidence.

### Historical Intended Scope

**The original decision intended ChaCha20-Poly1305 for:**
- Private RDF triple encryption (at rest)
- Encrypted SPARQL query results (in transit)
- Payload protection used only after the independent privacy-authorization path permits access
- Sensitive configuration values (e.g., API keys in production)

**Original selected parameters (not a description of current code):**
- Crate: `chacha20poly1305` (from RustCrypto project)
- Key size: 256 bits (32 bytes)
- Nonce size: 96 bits (12 bytes) - never reused with same key
- Tag size: 128 bits (16 bytes) - authentication tag

---

## Historical Rationale (Not Current Validation Evidence)

The comparisons and qualitative benefits below record the rationale available when this ADR was
accepted. They are not reproducible benchmark, side-channel, compliance, or suite-conformance
evidence for the thesis reference system. ADRs 0033 and 0034 control current protected-data claims,
and their implementation and conformance evidence remain pending.

### Alternatives Considered

| Algorithm | Pros | Cons | Decision |
|-----------|------|------|----------|
| **ChaCha20-Poly1305** | • Strong software performance rationale<br/>• AEAD built in<br/>• Well-specified nonce contract | • This ADR supplies no FIPS-validation evidence | ✅ **Chosen family** |
| **AES-GCM** | • Broad deployment<br/>• Hardware acceleration may be available | • Performance and side-channel properties depend on implementation and hardware<br/>• Strict nonce contract | ❌ Rejected here |
| **AES-256-CBC + HMAC** | • Widely trusted<br/>• FIPS-approved | • No authentication built-in<br/>• Two passes (encrypt + MAC)<br/>• Vulnerable to padding oracle | ❌ Rejected |
| **XSalsa20-Poly1305** | • Extended nonce (192 bits) | • Less common<br/>• Overkill for our use case | ❌ Rejected |

### Key Benefits

1. **Performance rationale:** Designed for strong software performance without requiring AES-NI;
   actual ProvChain performance must be established by reproducible project benchmarks

2. **Security rationale:** Provides an AEAD construction; resistance to side channels is an
   implementation and platform property, not established by this ADR

3. **Simplicity:** One integrated confidentiality-and-integrity primitive
   - Nonce uniqueness under a key remains mandatory and must be designed and tested
   - Random nonce generation alone is not evidence that reuse cannot occur
   - Authenticated encryption built-in (AEAD)

4. **Cross-platform rationale:** Does not require a specific hardware acceleration instruction;
   measured behavior remains environment-specific

### Historical Performance Comparison

The original ADR recorded point estimates for several algorithms but did not identify a command,
source benchmark, artifact, environment, or result manifest. Those figures are intentionally not
repeated as project evidence. Any later comparison must be regenerated by the benchmark/evidence
workflow and linked to immutable artifacts.

---

## Consequences

### Positive Impacts

1. **Performance potential:** Strong portable-software rationale, subject to measured validation
2. **Security:** Authenticated encryption when the selected implementation, key separation, nonce,
   associated-data, and error-handling rules are correct
3. **Confidentiality Support:** Provides an authenticated-encryption building block; legal and
   operational compliance requires separate controls and evidence
4. **Focused primitive:** One AEAD interface, while nonce uniqueness and key lifecycle remain
   explicit engineering obligations

### Negative Impacts

1. **Compliance:** This ADR supplies no FIPS-module or regulated-deployment validation evidence
   - **Mitigation:** Treat any regulated cryptographic profile and validation as a separate decision
   - **Impact:** No compliance claim follows from selecting an algorithm family

2. **Key Management:** Must manage a DEK per Protected Object and purpose-separated recipient keys
   - **Mitigation:** Use a profile-declared, purpose-separated key and wrapping contract
   - **Impact:** Operational complexity

---

## Historical Implementation Sketch (Non-Normative)

The code and snippets in this section are retained as decision history, not as a statement of
current implementation. The named `ChaCha20Encryption` type does not exist in the current tree.
`src/security/encryption.rs` instead exposes `PrivacyManager`, which currently uses
`XChaCha20Poly1305`, a 24-byte random nonce, a caller-supplied raw 32-byte key, and empty associated
data. ADR 0033 rejects treating that low-level primitive as a protected-data envelope system. The
mandatory `ProtectedDataSuiteV1` in ADR 0034 now fixes the payload AEAD, nonce construction, and
canonical encoding; this historical sketch is not normative for that suite.

### Code Locations

**Original sketch:**
- File: [`src/security/encryption.rs`](../../../src/security/encryption.rs)
- Proposed struct: `ChaCha20Encryption { key: Key, nonce: [u8; 12] }` (not present)

**Key Types:**
```rust
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

pub struct EncryptionKey {
    pub key: Key,        // 32 bytes (256 bits)
    pub owner_id: String, // Data owner identifier
}
```

### Encryption Flow

```rust
// Encrypt private RDF triples
use chacha20poly1305::{AeadInPlace, Key, Nonce, ChaCha20Poly1305};

let key = Key::from_slice(&encryption_key[..32]); // 256-bit key
let cipher = ChaCha20Poly1305::new(&key);
let nonce = Nonce::from_slice(&nonce_bytes[..12]); // 96-bit nonce

let mut buffer = plaintext_data.clone();
cipher.encrypt_in_place(nonce, &[], &mut buffer)?;
```

### Decryption Flow

```rust
let mut buffer = encrypted_data.clone();
let plaintext = cipher.decrypt_in_place(nonce, &[], &mut buffer)?;
```

### Historical Key-Management Proposal (non-normative)

The derivation sketch below is retained only as decision history. It must not be implemented: the
accepted architecture separates Participant authorization keys from recipient wrapping keys and
uses the per-object envelope topology in ADR 0033. ADR 0034 fixes the concrete suite and codec, but
their implementation, Network Profile activation, and conformance evidence remain pending.

**Per-Owner Keys:**
- Each data owner has unique encryption key
- Key derived from owner's Ed25519 signing key (HKDF)
- Keys rotated periodically (90-day default)

**Key Derivation:**
```rust
use hkdf::Hkdf;
use sha2::Sha256;

let master_key = owner_signing_key.to_bytes();
let hkdf = Hkdf::<Sha256>::new(None, &master_key);
let (enc_key, _nonce) = hkdf.expand(&[b"encryption"], 32)?;
```

### Historical Intended Testing (Not Current Coverage)

Current unit tests in [`src/security/encryption.rs`](../../../src/security/encryption.rs) cover a
raw-key round trip and wrong-key rejection for the legacy XChaCha helper. No current test proves the
historical HKDF sketch, nonce-reuse detection, envelope/context behavior, or the performance targets
below.

**Still-unmet validation criteria:**
- [ ] Encryption < 100µs for typical RDF data (1KB)
- [ ] Decryption < 100µs for typical RDF data (1KB)
- [ ] Authentication rejects tampered data
- [ ] Nonce reuse detected and prevented

---

## Performance Validation Status

No reproducible artifact in the current repository supports an actual/pass result for this ADR.
The earlier approximate encryption, decryption, key-generation, and throughput figures are
downgraded to unverified historical notes and must not be used in a paper table, thesis claim, or
acceptance gate. Validation remains pending a corrected evidence campaign with command, environment,
raw output, manifest hashes, and claim-to-artifact mapping.

---

## Related Decisions

- [ADR 0004](./0004-use-ed25519-signatures.md): Use Ed25519 for Signatures (ChaCha20 same designer: DJB)
- [ADR 0027](./0027-use-participant-principals-for-privacy-identity.md): Use Participant Principals for Privacy Identity
- [ADR 0033](./0033-use-one-immutable-ciphertext-and-per-object-dek-envelopes.md): Use one immutable payload and per-object DEK envelopes
- [ADR 0034](./0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md): Pin the mandatory protected-data suite and canonical privacy encoding

---

## References

- **RFC 8439:** ChaCha20 and Poly1305 for IETF Protocols
- **RFC 7539:** ChaCha20-Poly1305 for AEAD
- **RustCrypto Project:** https://github.com/RustCrypto/AEADs
- **WireGuard Protocol:** Uses ChaCha20-Poly1305 (real-world deployment)

---

**Authors:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Reviewers:** Associate Professor Dr. Ekkarat Boonchieng
**Approval Date:** 2026-01-28
