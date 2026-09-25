# ADR 0037: Bound the ProvChain Bridge to Converged Source Evidence and Target Final Admission

**Status:** Accepted
**Date:** 2026-08-31
**Context:** Thesis-bounded ProvChain-to-ProvChain interchange, source trust, replay, and crash recovery

---

## Decision

The thesis-reference bridge is one protocol named `ProvChainBridgeSuiteV1`. It copies one complete
public `OrdinaryProvenanceV1` payload, byte for byte, from one explicitly trusted ProvChain ledger
instance into one explicitly named ProvChain ledger instance. It is a one-hop provenance import,
not asset movement, lock-and-mint, ontology transformation, SPV, a heterogeneous-chain adapter, or
two-phase atomic commitment across ledgers.

A source network must first commit a target-bound **Bridge Export Declaration** in the same exact
source Admitted Block Envelope as the public payload. That block becomes exportable only after all
three pinned source nodes have committed the identical source envelope and prefix and issued their
matching Commit Receipts. A target accepts the resulting **Bridge Proof Bundle** only under a
target-profile-pinned **Bridge Source Trust Binding**.

The source proof never authorizes a target write. The target Scheduled Authority must include the
exact proof and copied payload in a normal target PoA Block Proposal. Target Final Admission then
independently applies every target integrity, consensus, state-commitment, package, full staged-union
SHACL, candidate-focus, bridge, and replay gate before the one target Ledger Journal append plus
`fsync`. The target Admitted Block Envelope commits the exact **Bridge Origin Evidence**. The target
journal is the only source of terminal imported/replay state.

This decision is an accepted architecture contract. It does not claim that the current bridge,
ledger journal, membership, PoA replication, or Final Admission code implements the contract.

[ADR 0038](./0038-bind-source-bridge-export-envelopes-to-the-parent-ledger-prefix.md) corrects
the missing authenticated parent-prefix input needed to realize this decision's non-genesis
one-step source Ledger Prefix verification while preserving the five-field proof bundle.

## Exact scope

`ProvChainBridgeSuiteV1` permits exactly:

- one source ProvChain ledger instance and one target ProvChain ledger instance;
- one native, non-bridged source envelope;
- `OrdinaryProvenanceV1` with a non-empty public RDF payload and no confidential or privacy-control
  content;
- byte-for-byte copying of the complete source public-payload field;
- the same State Commitment Scheme, Semantic Execution Profile, and Ontology Package digest on
  both networks; and
- one direct source-to-target hop under static, bilateral profile bindings.

The **Ledger Instance Identifier** (`LedgerInstanceId32`) is a public, nonzero 32-byte value drawn
once from a cryptographically secure random generator before the Network Profile and genesis
envelope are constructed. It is embedded verbatim in that ledger's genesis-bound Network Profile;
the profile's canonical content hash, genesis proposal, genesis envelope, and every later envelope
therefore bind it. The identifier is not derived from the profile, manifest, genesis Envelope Hash,
current tip, or another ledger. Every source and target reference binds both `network_id` and
`LedgerInstanceId32`.

The source and target identifiers are selected and exchanged before either bilateral profile is
constructed, so each profile can bind the peer identifier without a mutual profile/genesis-hash
cycle. Independent genesis construction or ledger reinitialization requires a fresh identifier;
replicas and recovery copies of the same exact journal retain it. A verifier rejects an all-zero,
absent, wrong-length, or target-binding-mismatched identifier. Reuse across independently
constructed histories is a profile-authoring safety violation; one isolated proof cannot establish
global non-reuse when all pinned profile and receipt authorities collude. A bare `network_id`,
profile name, height, latest tip, or informal “epoch” does not identify a ledger instance.

The source envelope must not already contain Bridge Origin Evidence. Subsets, selected triples,
reformatted RDF, changed blank-node labels, normalization by the relayer, inferred triples, and
mapped terms are not the declared payload. A correction or a different target requires a new
source block and therefore a new transfer identity.

## Static bilateral trust contract

The source Network Profile authorizes an outbound bridge rule that binds:

- the exact target `network_id` and target `LedgerInstanceId32`;
- `ProvChainBridgeSuiteV1` and `ExactPublicPayloadV1` copy mode;
- the common State Commitment Scheme, Semantic Execution Profile, and Ontology Package digest;
- `OrdinaryProvenanceV1` as the only permitted source Admission Kind; and
- the bridge-specific payload, proof, receipt, and target-envelope bounds in this decision.

The source rule does not pin the complete target profile hash. This avoids a mutual profile-hash
cycle. The target Network Profile instead contains a Bridge Source Trust Binding that pins:

- the exact source `network_id` and source `LedgerInstanceId32`;
- the exact source Network Profile identity and canonical content hash;
- the source Governance Trust Root raw Ed25519 public key;
- the signed source Membership Manifest identity, version, and canonical content hash;
- the exact three source convergence-node identities and their Node Identity keys as resolved from
  that manifest;
- the exact ordered source PoA authority set used to verify the source Scheduled Authority;
- the permitted source State Commitment Scheme, Semantic Execution Profile, Ontology Package
  identity and digest, and Admission Kind; and
- `AllThreeCommitReceiptsV1` as the only receipt policy.

Proof-supplied profiles, manifests, roots, keys, network names, or receipt signers never create
trust. They are evidence only when their exact values satisfy an already activated target Bridge
Source Trust Binding. The signed source manifest must be valid under its pinned Governance Trust
Root and be the exact manifest bound by the pinned source profile.

For bounded v1, the exact source Network Profile and signed Membership Manifest are genesis-bound
and fixed for that complete ledger instance; an in-place source profile or manifest transition is
unsupported. The target binding is the one active at the target proposal's parent. Profile,
manifest, package, trust-root, or bridge-policy rotation is not inferred from proof-carried data. A
changed source artifact requires a new source ledger instance, and a changed target trust policy
requires a new target ledger instance until a separate governed profile-transition decision
exists. Prior committed imports remain historical and are never rolled back.

## CanonicalBridgeEncodingV1

All bridge-specific records use one closed binary codec:

```text
record = ASCII "BCV1" || record_tag:u8 || field_count:u8 || fields
field  = field_tag:u8 || value_length:u32be || value[value_length]
```

Field tags are sequential from `0x01` in exactly the order declared for each record. Integers are
fixed-width big-endian. Digests, keys, signatures, ledger-instance identifiers, and transfer
identifiers are raw bytes. Network, Network Profile, Membership Manifest, and node identifiers are
`1..=128` ASCII bytes matching `[a-z0-9][a-z0-9._:-]*`. Every required identifier ordering is
unsigned lexicographic order over those exact ASCII bytes, with no locale, case folding, Unicode,
or natural-number interpretation. A decoder rejects wrong magic, record tag, field count, tag,
order, or length; unknown, duplicate, absent, noncanonical, or forbidden-empty fields; unchecked
arithmetic or limit overflow; nested trailing bytes; and outer trailing bytes.

The bridge never signs or hashes delimiter-concatenated text, display UUIDs, JSON, Serde output,
Rust layout, `bincode`, hex, base64, platform `usize`, filesystem paths, or transport framing.
Canonical source profile, signed manifest, and Admitted Block Envelope bytes retain their own
protocol encodings and are embedded as opaque exact byte fields. Bridge activation fails closed
until those encodings and their signature transcripts are fixed, implemented, and accepted; the
bridge codec does not make an ambiguous upstream artifact canonical by wrapping it. This decision
separately fixes the bridge-specific Commit Receipt codec and signature transcript below.

The bridge tag table is closed:

| Tag | Record |
|---:|---|
| `0x01` | `BridgeExportCoreV1` |
| `0x02` | `BridgeExportDeclarationV1` |
| `0x03` | `BridgeCommitReceiptCoreV1` |
| `0x04` | `BridgeCommitReceiptV1` |
| `0x10` | `BridgeProofBundleV1` |
| `0x11` | `BridgeOriginEvidenceV1` |
| `0x12` | `BridgeImportEvidenceV1` |

No unknown-field or extension rule exists in v1. A later bridge format requires a new suite/codec
decision and explicit historical decoding.

Bridge domain hashes use:

```text
DHash(label, record) = SHA-256(
    u16be(length(label)) || ASCII label ||
    u32be(length(record)) || exact record bytes
)
```

The label is part of the scheme definition and cannot be substituted or NUL-terminated.

### StrictEd25519V1 receipt verification profile

`StrictEd25519V1` is used only for the bridge-specific Commit Receipts in this decision. Its
normative acceptance predicate is the exact behavior of `ed25519-dalek` `2.2.0`:

1. accept exactly 32 raw public-key bytes `A32` as a compressed Edwards-Y encoding and parse them
   with `VerifyingKey::from_bytes` from that pinned version; its documented ZIP-215 point-validation
   behavior is normative for v1 rather than substituting RFC 8032/NIST point-decoding criteria;
2. accept exactly 64 raw signature bytes as `R32 || S32`, where `R32` is a compressed Edwards-Y
   encoding and `S32` is the RFC 8032 little-endian scalar encoding; the pinned parser must reject
   unless `S < L`;
3. pass the exact 32-byte `DHash` result specified for the receipt below as the message to
   `VerifyingKey::verify_strict`; this is PureEd25519 message verification, not Ed25519ph or
   Ed25519ctx; and
4. accept if and only if that exact pinned `verify_strict` call accepts.

Bridge activation pins the crate's resolved feature surface as well as its version. The conforming
direct declaration is exactly:

```toml
ed25519-dalek = { version = "=2.2.0", default-features = false, features = ["fast", "zeroize"] }
```

For every bridge-enabled build and conformance-test target, the complete Cargo feature graph for
`ed25519-dalek` must contain only `fast` and `zeroize`. In particular,
`legacy_compatibility` and `serde` must be absent; no workspace member, development dependency, or
transitive dependency may re-enable them through Cargo's additive feature unification. Protocol
records use the canonical raw fixed-size bytes above rather than serde-defined key or signature
encoding. A different version or resolved feature set requires a successor suite decision and full
acceptance-corpus revalidation; it is not an implementation-compatible substitution.

Receipt creation uses the corresponding manifest-resolved Node Identity signing key and the pinned
version's deterministic PureEd25519 `SigningKey::sign(M)` behavior over that same exact 32-byte
message `M`. Prehash, context, externally randomized nonce, or compatibility signing modes are
forbidden. Re-signing the same `M` with the same Node Identity key must reproduce the same raw
`R32 || S32` bytes and those bytes must satisfy the verifier above.

Consequently, v1 requires the pinned method's canonical-scalar check, point decoding, small-order
public-key and `R` rejection, and strict verification equation. Its exact point-encoding and
torsion behavior is protocol behavior, not an implementation preference. For an accepted parsed
public point `A`, canonical scalar `S`, base point `B`, exact encoded `R32`, exact encoded public key
`A32`, and message `M = DHash(...)`, the strict equation is:

```text
k = SHA-512(R32 || A32 || M) reduced modulo the Ed25519 group order L
R32 == compress([S]B - [k]A)
```

Both the public point and decompressed `R` must be non-small-order under the pinned method. Ordinary
`verify`, batch verification, prehashed/context modes, compatibility fallbacks, cofactored
alternatives, or a different library's notion of “strict” are not substitutes. The exact
dependency version, allowed feature graph, and lock must be preserved for bridge activation, and an
independent implementation must match the required acceptance corpus byte for byte. This pin does
not redefine the separate upstream Network Profile, Membership Manifest, or Block Proposal
signature contracts.

## Source Bridge Export Declaration

`BridgeExportCoreV1` (`0x01`) has exactly ten fields:

1. source `network_id`;
2. source `LedgerInstanceId32`;
3. source Network Profile identity;
4. source Network Profile canonical content hash32;
5. intended source ledger position `u64be`;
6. target `network_id`;
7. target `LedgerInstanceId32`;
8. source Admission Kind tag `OrdinaryProvenanceV1 = 0x01`;
9. copy-mode tag `ExactPublicPayloadV1 = 0x01`; and
10. SHA-256 digest32 of the exact complete source public-payload bytes.

The **Bridge Transfer ID** is not caller-selected and is not a UUID:

```text
T32 = DHash(
    "provchain/bridge/transfer-id/v1",
    exact BridgeExportCoreV1 bytes
)
```

`BridgeExportDeclarationV1` (`0x02`) has exactly two fields: the complete canonical
`BridgeExportCoreV1` and its derived `T32`. The source Proposal Digest and source Admitted Block
Envelope cover that exact declaration. Source Final Admission verifies the declaration, outbound
profile rule, position, target ledger instance, Admission Kind, absence of prior bridge origin,
exact public-payload digest, and recomputed `T32` before source journal commitment.

An export declaration created after source commitment, a relayer-chosen target, a changed transfer
identifier, or a signature over copied block fields is not a conforming export intent. The source
commit is a network-attested declaration to copy public provenance. It does not lock, burn, escrow,
or conserve an asset and it is not rolled back when the target is unavailable or rejects.

The declaration is envelope-bound admission evidence and is excluded from source Public
Provenance State and its SHACL Data Graph. The exact public payload remains the only source content
copied into target Public Provenance State.

## Canonical bridge Commit Receipt

For `ProvChainBridgeSuiteV1`, the canonical realization of ADR 0023's Commit Receipt is fixed rather
than interpreted from an open-ended acknowledgement. `BridgeCommitReceiptCoreV1` (`0x03`) has
exactly eleven fields:

1. `network_id`;
2. `LedgerInstanceId32`;
3. Network Profile identity;
4. Network Profile canonical content hash32;
5. Membership Manifest identity;
6. Membership Manifest version `u64be`;
7. Membership Manifest canonical content hash32;
8. receipt node identity;
9. ledger position `u64be`;
10. committed Envelope Hash32; and
11. resulting Ledger Prefix Hash32.

`BridgeCommitReceiptV1` (`0x04`) has exactly three fields: the complete canonical core, signature
scheme tag `StrictEd25519V1 = 0x01`, and raw signature64. The Node Identity Key resolved for the
receipt node signs exactly:

```text
DHash(
    "provchain/bridge/commit-receipt-signature/v1",
    exact BridgeCommitReceiptCoreV1 bytes
)
```

Verification uses the exact raw manifest-listed Ed25519 public key and rejects a malformed key or
signature encoding. The receipt has no timestamp, nonce, local path, retry counter, or transport
metadata. Reissuing it for the same node and exact committed facts therefore produces the same
signature and complete bytes. It remains convergence evidence only; its codec and signature do not
turn it into a consensus vote or commit authority.

## Source Convergence Evidence and Bridge Proof Bundle

`BridgeProofBundleV1` (`0x10`) contains exactly:

1. `T32`;
2. the exact canonical source Network Profile bytes;
3. the exact canonical signed source Membership Manifest bytes;
4. the exact source Admitted Block Envelope bytes containing the declaration and payload; and
5. one receipt-sequence field encoded as exactly three repetitions of
   `u32be(receipt_length) || exact BridgeCommitReceiptV1 bytes`, strictly sorted by source node
   identity.

No relayer signature is required for authority. Any transport may carry the exact bundle; target
trust comes from the pinned artifacts, source proposal evidence, and receipts rather than the
relayer or an Authenticated Peer Session.

The target recomputes and verifies, without trusting decoded claims:

1. every canonical structure and deterministic bound;
2. the source profile and manifest hashes against its Bridge Source Trust Binding;
3. the manifest governance signature and its binding to the source network/profile;
4. the source ledger-instance identifier, active manifest, package, semantic profile, and state
   scheme in the exact source envelope;
5. the source envelope's canonical form, Envelope Hash, declared ledger position, chain anchors,
   and one-step parent-prefix-to-prefix computation;
6. the source proposer signature, manifest authorization, and Scheduled Authority for that source
   PoA turn;
7. the exact Bridge Export Declaration, outbound rule, target ledger instance, source payload
   bytes/digest, copy mode, and recomputed `T32`;
8. exactly the three distinct expected source node identities, with no missing, duplicate, unknown,
   inactive, or extra signer;
9. each `BridgeCommitReceiptV1` strict Node Identity signature and exact source network,
   `LedgerInstanceId32`, profile, manifest, position, Envelope Hash, and Ledger Prefix Hash binding;
   and
10. equality of the envelope and prefix facts across all three receipts.

This is **Source Convergence Evidence** under the static six-node reference trust model. The target
can verify the source envelope and its one-step prefix transition, but it does not receive or replay
the complete source ledger history. Three matching receipts attest that the three pinned source
nodes recorded the same envelope and prefix; they do not constitute an SPV/Merkle proof, PBFT/BFT
finality, a generic threshold bridge, trustless finality, or cryptographic proof of physical disk
persistence. The false-convergence bound depends on at least one pinned source receipt signer
remaining honest and refusing to sign before its own conforming commit. One unavailable source
node prevents a v1 proof from becoming exportable.

## PoA proposal coordination and signing fence

Bridge activation requires both the source and target Scheduled Authority paths to use one
per-ledger **PoA Proposal Coordinator** for the pending turn. Every ordinary, privacy-control, and
bridge unsigned proposal request enters that same coordinator. For one verified parent and PoA
Turn, it serializes request selection, coalesces byte-identical requests, and completes every
deterministically evaluable, non-mutating preflight check before selecting one request. Preflight is
advisory only: it cannot append, reserve a transfer, mutate Effective Bridge State, or establish an
authoritative admission result.

The coordinator may refresh a parent and rerun complete preflight only while the request is
unselected, no fence persistence has begun, and the signing operation has not been invoked. Those
three conditions form the complete refresh boundary. Selection fixes the exact proposal body for
the turn and immediately begins the fence transition; once selection or a fence-write attempt has
occurred, parent advancement cannot authorize a different body even if no signature response is
known.

Before invoking the Scheduled Authority's signing operation, a crash-safe **PoA Signing Fence**
must be durably synchronized and bind that ledger instance, PoA Turn, active authority-key
reference, exact canonical unsigned proposal-body bytes, and Proposal Digest. Every process able
to invoke that authority key must share the fence's exclusive single-writer boundary; inability to
prove exclusive ownership stalls rather than signs. A fence write or `fsync` failure, ambiguous
write result, or missing, truncated, corrupt, or conflicting recovery state stalls the turn; it
does not restore the request to the refreshable state. Once the exact fence is verified, recovery
may invoke the signer only for that body, reproduce its deterministic signature and exact signed
proposal, or rebroadcast retained exact signed bytes. No other body is eligible.

The Scheduled Authority signs at most one distinct proposal for a PoA Turn. After the fence
boundary, only that exact fenced body and its deterministic signature may be recovered, presented,
or rebroadcast, including when it is unknown whether the signer returned. A different signed
proposal for the same turn is Equivocation even if it arrives first elsewhere, has a different
Admission Kind, or would pass Final Admission. Final Admission is never a first-wins concurrency or
fork-choice mechanism. A deterministic Final Admission rejection of the signed proposal is a
safety incident and leaves the network stalled; it does not authorize a replacement proposal for
that turn. A node-incapacity or unknown-commit result permits journal recovery followed only by
recovery or retry of the exact fenced proposal.

The fence for a turn may be retired or advanced only after Verified Journal Replay proves that the
exact fenced proposal produced the envelope committed at that turn. Until then, the exact fenced
body remains the only signable body, including across restart. A different committed envelope at
the turn is Equivocation or divergence and cannot retire the fence as success.

The Signing Fence is non-authoritative safety state. It cannot make a block committed, advance a
Ledger Prefix, consume `T32`, or update a ledger projection, and therefore does not replace the
Ledger Journal or alter its sole append-plus-`fsync` commit point. For an uncommitted turn it is
deliberately not a rebuildable Ledger Projection: loss of the exact fence state stalls instead of
reconstructing a different proposal from queued requests. Its exact crash-safe storage and
retirement realization requires a separate core PoA decision and conformance evidence; source or
target bridge activation fails closed until that realization exists.

## Target proposal and Final Admission

`BridgeOriginEvidenceV1` (`0x11`) has exactly three fields: `T32`, SHA-256 of the exact complete
Bridge Proof Bundle bytes, and the exact complete Bridge Proof Bundle bytes. It is admission
evidence excluded from Public Provenance State, while the copied RDF payload is asserted target
public provenance.

The target bridge adapter may receive, strictly decode, and relay an unsigned bridge request, but
it cannot construct or sign an Admission Candidate, append, mark a transfer accepted, insert RDF,
or choose a target signer. Before asking for a proposal, it bounds the request and consults verified
Effective Bridge State. An exact existing entry produces the derived `AlreadyImported` response
and returns its stored target reference; a same-`T32` conflict produces `ReplayConflict`. Neither
response enters Final Admission or mutates authoritative state. Only an absent `T32` becomes an
unsigned request to the PoA Proposal
Coordinator. If it remains valid, is selected for the current turn, and is fenced before signing,
the target Scheduled Authority creates a normal `OrdinaryProvenanceV1` Block Proposal whose
Proposal Digest binds:

- the target ledger parent, active target profile/manifest/package, and normal consensus fields;
- the exact copied public payload or its digest;
- `T32`;
- the exact Bridge Origin Evidence and proof-bundle digest; and
- the target parent and proposed Post-State Commitments.

Before any authoritative mutation, target Final Admission verifies at least:

1. the target expected parent, ledger instance, and active contracts;
2. target signer authorization, Scheduled Authority, proposal signature, and PoA Consensus
   Acceptance;
3. the bridge suite, closed codec, proof size, and target Bridge Source Trust Binding;
4. all Source Convergence Evidence checks above;
5. recomputed `T32`, absence of prior Bridge Origin Evidence at source, and byte-for-byte equality
   of the target public payload with the declared source payload;
6. absence of `T32` from the parent Effective Bridge State;
7. all ordinary integrity, RDF parsing, chain-anchor, Post-State Commitment, Ontology Package,
   complete staged-union SHACL, Candidate Focus Node, and deterministic-bound gates; and
8. the complete target Admitted Block Envelope before one Ledger Journal append plus `fsync`.

The ordering of pure checks may be optimized only when the observable verdict and no-mutation
guarantee are equivalent. Source semantic acceptance never substitutes for target validation. The
target reparses the unchanged payload under the common scheme, stages it against its own committed
public state, evaluates its own complete package-bound semantic contract, and computes its own
Post-State Commitment.

Only successful target journal append plus `fsync` creates an imported fact. The exact proof bytes
remain in the target envelope so Verified Journal Replay can reverify the committed origin and
reconstruct replay state without an external bridge database, source RPC, relayer cache, or
Oxigraph availability.

Final Admission retains ADR 0017's exactly two authoritative outcomes: committed or rejected. After
fence persistence begins, an absent signer or admission response, node-incapacity result, or unknown
commit outcome requires the adapter first to recover the target journal and verified Signing Fence
and refresh journal-derived state. An exact committed import then yields `AlreadyImported`; a
committed same-`T32` mismatch yields `ReplayConflict`. If the transfer and turn remain uncommitted,
only the exact fenced body may be signed or its deterministic exact signature recovered, presented,
or rebroadcast. If a distinct envelope advanced that same turn, the condition is Equivocation or
divergence, not a benign stale-parent race.

Only a request that is still unselected, has not begun fence persistence, and has never invoked the
signer may be refreshed after parent advancement and evaluated for a later target turn.
Canonical-proof, signature, trust-binding, identifier, and payload-binding failures remain failures
for those exact proof bytes. Full-union SHACL, candidate-focus, parent/post-state, and other
parent-dependent preflight checks may produce a different advisory verdict for such a refreshable
request at a later parent. After selection begins, a deterministic rejection or uncertain fence or
signer outcome stalls the turn and cannot be converted into a different proposal. No rejected
attempt consumes `T32`, and node incapacity is not a global rejection.

## Effective Bridge State and terminal outcomes

**Effective Bridge State** is the deterministic target-journal projection:

```text
T32 -> Imported {
    proof_bundle_hash32,
    source_network_id,
    source_ledger_instance_id32,
    source_position_u64,
    source_envelope_hash32,
    source_ledger_prefix_hash32,
    public_payload_hash32,
    target_network_id,
    target_ledger_instance_id32,
    target_position_u64,
    target_envelope_hash32
}
```

There is no authoritative `Pending`, `Validated`, `Accepted`, or `Rejected` bridge table and no
process-local accepted-ID set. The terminal target-ledger rules are:

- absent `T32` plus the one valid signed proposal for that PoA Turn may commit exactly one
  `Imported` entry;
- the same `T32`, proof-bundle hash, source reference, payload hash, and target ledger instance
  returns `AlreadyImported` with the stored target position and Envelope Hash and performs no
  append;
- the same `T32` with any differing proof, source reference, payload, or target ledger instance
  returns `ReplayConflict`, performs no append, and reports a safety incident;
- concurrent unsigned requests are serialized by the PoA Proposal Coordinator and byte-identical
  requests are coalesced before selection; and
- after selection and the fence transition begin, only the exact selected body and its
  deterministic signed proposal may be recovered or retried. A distinct signed proposal for the
  same turn is Equivocation, and the target expected-parent check or Final Admission must not choose
  a first winner.

An invalid or unavailable target does not consume `T32`; an attacker cannot poison the terminal
map by submitting malformed evidence first. Fixing a source declaration or source payload requires
a newly committed source export and a new `T32`; fixing incomplete or malformed transport evidence
may retain `T32` because the source declaration core is unchanged. A target-state-dependent verdict
may change after parent advancement only while the request remains unselected, fence persistence
has not begun, signing has not been invoked, and complete new-parent preflight is rerun. A
post-selection deterministic rejection or uncertain fence/signer outcome stalls its turn. Node
incapacity is not a global rejection.

`Imported` is node-relative, like Committed Block. A transfer is **Bridge-Converged** only after
all three target nodes commit the identical target envelope and prefix and issue their normal
matching target Commit Receipts. `BridgeImportEvidenceV1` (`0x12`) may report `T32`, the committed
target envelope/prefix reference, and the available target receipts only through the exact fields
below; it is derived evidence, not a second commit record or a source-ledger completion transition.
It is not self-authenticating: without the exact referenced committed target envelope/history and
the target profile/manifest trust context needed to verify them, the record is untrusted transport
metadata and proves no target commitment.

`BridgeImportEvidenceV1` has exactly seven fields: `T32`, target `network_id`, target
`LedgerInstanceId32`, target ledger position `u64be`, target Envelope Hash32, target Ledger Prefix
Hash32, and one receipt-sequence field. The sequence starts with `receipt_count:u8` in `0..=3`,
followed by exactly that many repetitions of
`u32be(receipt_length) || exact BridgeCommitReceiptV1 bytes`, sorted by target node identity. A
zero count has no following receipt bytes and reports only the imported target reference. For every
nonzero count, each receipt must be canonical, strictly signed by its manifest-resolved Node
Identity Key, and name a distinct member of the exact three active target receipt nodes bound by the
profile and manifest in the referenced target envelope. Every receipt must match the record's exact
target network, `LedgerInstanceId32`, position, Envelope Hash, and Ledger Prefix Hash, and must match
that envelope's exact Network Profile and Membership Manifest identities, versions, and hashes. An
unknown, inactive, duplicate, extra, wrongly ordered, malformed, or mismatched receipt invalidates
the complete import-evidence record. A count of three establishes Bridge-Converged only when the
valid receipts name all three exact target nodes. The complete record is at most `16_384` bytes.

The protocol supports retrying delivery at least once and enforces one target-ledger effect per
`T32`. It does not claim exactly-once transport, cross-ledger atomicity, or that an acknowledgement
reached the source.

## Failure and recovery contract

| Failure | Authoritative result | Required behavior |
|---|---|---|
| Source commit has fewer than three matching receipts | Not exportable | Reconstruct source envelope from the journal and recollect deterministic receipts |
| Source proof cache is lost | Source history unchanged | Rebuild the bundle from exact source journal/profile/manifest/receipt evidence |
| Bundle is malformed, noncanonical, or oversized | No target candidate/append | Reject before unbounded allocation or admission |
| Source trust/profile/manifest/receipt check fails in adapter/preflight | No authoritative ledger result; no selection, fence, signature, or mutation | Supply corrected transport evidence under the same immutable binding; changing accepted trust requires a successor target ledger or future transition ADR, not a mutable v1 profile |
| Source proof is valid but target semantics fail during refreshable preflight | No authoritative ledger result; no selection, fence, signature, or mutation | Never bypass target SHACL; reevaluate only after verified parent advancement while still unselected, unfenced, and signer-uninvoked; correcting source bytes requires a new export |
| Verified target parent advances before selection or any fence-write/signing attempt | Request remains unselected and unfenced | Refresh the parent, rerun every preflight gate, and reconsider it only through the coordinator for the new PoA Turn |
| Target Scheduled Authority is unavailable before selection or any fence-write/signing attempt | Off-ledger delivery remains pending | Retry the still-unselected request after normal target PoA resumes; no authority takeover |
| Fence write/`fsync` fails or has an ambiguous result | No authoritative ledger result; turn safety is unresolved | Do not refresh or select another body; recover and verify the exact fence or stall fail closed |
| Exact fence is durable but crash/unavailability occurs before signer invocation | Exact body and turn are locked; no authoritative ledger result | Recover the journal and exact fence, then invoke the signer only for that exact body |
| Signer invocation occurred but its response is absent or unknown | Exact body and turn are locked; no assumed ledger result | Recover the journal and fence; reproduce or recover only the deterministic exact signature and never construct another body |
| Signed target proposal fails a deterministic Final Admission gate | Rejected with no journal/bridge mutation; Stalled Network and safety incident | Never sign a replacement for that turn; diagnose the invariant failure and retain or recover the exact fenced proposal |
| Target timeout, memory, I/O, or local verifier failure after signer invocation | Node incapacity, no global verdict | Recover the journal and Signing Fence, then recover or retry only the exact fenced proposal if its turn remains uncommitted |
| Crash or error after signer invocation and before target commit outcome is known | No assumed result | Recover and verify the target journal and Signing Fence, query `T32`, and never construct an alternative proposal for that turn |
| Target `fsync` succeeds but response is lost | `Imported` | Exact retry returns `AlreadyImported` and the existing target reference |
| Projection update fails after commit | `Imported`, node degraded | Verified Journal Replay rebuilds Effective Bridge State and projections |
| Target commits locally but target receipts are incomplete | Imported, not Bridge-Converged | Retry exact target-envelope replication, not source import |
| Target restarts | Journal remains authority | Replay reconstructs identical Effective Bridge State |
| A successor target ledger instance omits the source trust binding | Prior ledger imports remain historical | Proofs into the successor reject; no rollback or rewrite |
| Distinct target proposals are signed for one PoA Turn | Equivocation and safety incident | Fail closed and stall; do not select by arrival, target parent, Final Admission, or bridge status |
| Distinct source-converged evidence conflicts at one source position | Safety incident | Reject, halt that path, and report; never overwrite or choose a fork |

The source-proof and target-semantic preflight rows describe only the refreshable pre-selection
path. If any equivalent deterministic failure is discovered after proposal selection and signing,
the signed Final Admission rejection row governs: reject without journal mutation, report a safety
incident, and stall rather than signing a replacement.

An append or `fsync` I/O error can leave the target commit outcome indeterminate until ADR 0016
journal recovery resolves the exact record. A separate bridge WAL, replay database, or outbox must
not become a competing commit authority. Off-ledger delivery queues may be rebuildable operational
helpers only.

## Deterministic bounds

The reference Network Profiles fix all normal envelope bounds and additionally require:

- exact copied public payload: `1..=262_144` bytes;
- canonical source Network Profile artifact: at most `32_768` bytes;
- canonical signed source Membership Manifest: at most `65_536` bytes;
- exact declaration-bearing source Admitted Block Envelope: at most `393_216` bytes;
- each canonical Commit Receipt: at most `4_096` bytes, exactly three;
- complete Bridge Proof Bundle: at most `503_883` bytes;
- complete Bridge Origin Evidence: at most `503_968` bytes; and
- complete target Admitted Block Envelope: at most the existing reference `1_048_576`-byte block
  limit, with all target-envelope bytes outside the copied payload and complete Bridge Origin
  Evidence together bounded to at most `262_187` bytes.

Nodes use checked arithmetic and preflight outer lengths before nested decoding or allocation. A
profile that cannot satisfy the complete target-envelope inequality cannot activate the bridge.
The proof maximum includes its record/field framing and follows from
`6 + 25 + 32 + 32_768 + 65_536 + 393_216 + 3 * (4 + 4_096) = 503_883`.
The origin maximum then includes its record/field framing:
`6 + 15 + 32 + 32 + 503_883 = 503_968`. The complete target envelope is therefore at most
`262_144 + 503_968 + 262_187 = 1_028_299` bytes, leaving `20_277` bytes of mandatory headroom below
`1_048_576`; all outer target-envelope framing is inside the final term.
These bounds deliberately pay for duplicated source-payload bytes inside the proof and target
payload. Removing that duplication requires a later canonical witness decision and equivalent
replay evidence.

## Required conformance evidence and activation gate

Bridge activation and thesis-result wording remain fail closed until reproducible evidence includes:

- golden byte and digest vectors for every bridge record and transfer-ID derivation, verified by an
  independent decoder;
- `StrictEd25519V1` vectors that combine RFC 8032 algorithm positives with receipt-specific exact
  32-byte messages, plus explicit expected verdicts for `S = L`, `S > L`, malformed and
  noncanonical public-key/`R` encodings, small-order public keys and `R`, torsion-sensitive cases,
  ZIP-215-versus-RFC point-decoding boundaries, wrong message/key/signature, and
  ordinary-versus-strict differential cases. The pinned verifier and an independent verifier must
  agree on the complete corpus; batch, prehash, and context modes must be rejected by the profile;
- archived `Cargo.toml`, `Cargo.lock`, `cargo metadata --locked --format-version 1`, and
  `cargo tree -e features -i ed25519-dalek@2.2.0 --locked` output for every bridge-enabled build and
  evidence target, with a CI assertion that the resolved crate version is exactly `2.2.0`, its
  enabled feature set is exactly `fast,zeroize`, and `legacy_compatibility` is absent. The `S = L`
  and `S > L` vectors must also execute against the shipped evidence binary, not only a helper;
- a strict negative corpus for every field/tag/order/count/length/trailing-byte mutation and every
  signed or hashed field;
- wrong source/target network or `LedgerInstanceId32`, all-zero/wrong-length identifiers,
  substituted root/profile/manifest/package, inactive or out-of-turn authority, proof-supplied key,
  and invalid governance/proposal/receipt signatures;
- profile-authoring fixtures that preserve the identifier for exact replicas/recovery but flag its
  reuse across two independently constructed histories as a safety violation;
- zero, one, two, duplicate, unknown, extra, wrong-envelope, wrong-position, and wrong-prefix
  receipt cases;
- exact payload success plus changed, reformatted, subset, mapped, private, and re-bridged payload
  rejection;
- a source-conformant payload that fails the target's full staged-union SHACL rule;
- direct-import and out-of-turn target-signer rejection;
- simultaneous ordinary, privacy-control, and bridge unsigned requests proving coordinator
  serialization, byte-identical coalescing, full preflight before selection, and at most one distinct
  signature for each PoA Turn;
- parent advancement before selection/fence persistence proving refresh occurs only while the
  request is unselected, unfenced, and signer-uninvoked, plus parent advancement after the boundary
  proving the exact selected body remains locked;
- fence write and `fsync` failure/ambiguous-result injection; missing, truncated, corrupt, and
  conflicting recovery state; crash after durable fence but before signer invocation; unknown
  signer response; and crash after signature production, all proving fail-closed exact-body
  recovery with no replacement;
- exact matching journal replay permitting fence retirement; a distinct same-turn journal envelope
  producing an Equivocation/divergence safety incident; and crash after journal `fsync` but before
  fence retirement proving recovery retires only the exact matching fence;
- deterministic signed rejection proving a stalled turn with no replacement, and injected distinct
  signed proposals proving Equivocation rather than first-wins admission;
- concurrent processes or signing services able to invoke one authority key proving exclusive
  single-writer fence ownership;
- duplicate, concurrent-delivery, conflicting, restart, lost-response, and post-commit
  projection-failure tests proving one target-journal effect;
- failpoints before, during, and after target append/`fsync`, followed by verified recovery;
- two independent three-process ProvChain networks demonstrating source 3/3 convergence, target
  admission, target 3/3 convergence, exact target envelope/prefix equality, restart, partition,
  dropped proof, and dropped receipt recovery; and
- an archived evidence bundle containing all six node configs, both profiles, signed manifests,
  packages and hashes, exact source/target envelopes, source and target receipts, proof bytes,
  `T32`, commands, raw logs, source revision, dependency lock, toolchain, environment, and claim
  mapping.

Current unit tests that push a fabricated in-memory block, register one arbitrary trusted key,
reject a UUID replay in one process, and assert destination chain length are legacy prototype tests.
They are not evidence for this decision.

## Rationale

The source export declaration prevents a relayer from assigning a target or transfer identity to an
already committed block after the fact. The profile-bound, preselected source
`LedgerInstanceId32` lets the target reject a different same-named ledger without creating a mutual
genesis-hash cycle. It is an opaque profile-bound identifier, not proof of global uniqueness or
genesis ancestry. An exact target trust binding prevents proof-carried keys from authorizing
themselves. All three receipts match the already accepted three-node Network Convergence condition
without importing PBFT quorum language into PoA.

Copying exact public payload bytes and requiring identical semantic scheme/package contracts makes
the bridge small enough to explain and test. Independent target Final Admission preserves the
target consortium's authority and catches constraints that depend on target history. Journal-derived
terminal state closes the current crash window between an in-memory replay marker and target block
creation. Serializing work before selection and durably fencing one exact selected proposal keeps
bridge retry behavior inside the PoA no-equivocation rule instead of misusing Final Admission as
fork choice.

The design intentionally accepts static bilateral trust, 3/3 availability, duplicated proof bytes,
and no source-side rollback. Those are visible reference-system trade-offs, not hidden production
claims.

## Alternatives considered

| Alternative | Decision |
|---|---|
| Caller UUID plus one configured authority signature | Rejected: identity, source commitment, membership, convergence, and replay durability remain unproved |
| Relayer-created proof for any earlier source block | Rejected: source never committed the target-bound export intent |
| Direct bridge append after source verification | Rejected: bypasses target Scheduled Authority and Final Admission |
| Let Final Admission choose among concurrently signed proposals | Rejected: distinct proposals for one PoA Turn are Equivocation, not a first-wins race |
| Separate durable replay database | Rejected: creates a second authority and atomicity problem beside the target journal |
| Majority or configurable receipt threshold | Rejected for v1: changes the accepted all-three reference convergence and invites BFT/finality overclaims |
| Automatic ontology mapping | Deferred: transformed semantics and mapper governance require a separate contract and evidence |
| Lock/mint, burn/release, or two-phase asset transfer | Deferred/out of scope: requires asset conservation and cross-ledger failure semantics not needed for provenance copy |
| Heterogeneous/SPV bridge | Future work under ADR 0015 |

## Consequences

- source Final Admission and canonical envelopes need a bridge-export declaration field and a
  profile-bound outbound rule;
- target profiles need explicit foreign-source trust bindings rather than mutable trusted-key
  registration;
- the source receipt, profile, manifest, and envelope canonical byte/signature contracts become
  activation prerequisites;
- the exact `ed25519-dalek` `2.2.0`, default-off `fast,zeroize` feature graph, strict
  receipt-verification behavior, shipped-binary scalar boundaries, and cross-verifier corpus become
  activation prerequisites; Cargo feature unification must not introduce `legacy_compatibility`;
- source and target PoA paths need one request coordinator and a crash-safe Signing Fence that
  prevents a second distinct signature for a turn without becoming a ledger commit authority;
- the target Scheduled Authority and ordinary semantic path cannot be bypassed by bridge code;
- bridge replay and origin projections must be rebuilt only from target journal history;
- one offline source node blocks bridge export and one unavailable target Scheduled Authority stalls
  target import;
- imported public RDF is duplicated across ledgers and proof/envelope storage;
- transport authentication can limit exposure but cannot replace end-to-end proof verification;
- metrics must distinguish Source Convergence, target-local Imported, and Bridge-Converged; and
- manuscript and thesis wording must keep this architecture separate from implementation and
  reproduced six-node evidence.

## Explicit non-goals

V1 does not provide asset ownership transfer, token mint/burn, escrow, cross-ledger atomicity,
exactly-once delivery, wall-clock freshness, source rollback detection, dynamic foreign-trust
rotation, generic threshold/BFT finality, header/Merkle/SPV verification, heterogeneous ledgers,
RDF or ontology mapping, private-data sharing, grant/key transfer, multihop relaying, production
operations, or bridge economics. Those remain future work or later milestones.

## Related Decisions

- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) keeps only a bounded
  ProvChain-to-ProvChain bridge in the thesis-reference milestone.
- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md) makes the exact target
  envelope and Bridge Origin Evidence durable at one commit point.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) makes the bridge an
  adapter and keeps replay state atomic with or derived from the journal.
- [ADR 0018](./0018-separate-node-commitment-from-network-convergence.md) distinguishes source/target
  local commitment from network convergence.
- [ADR 0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) prevents source evidence
  from bypassing the target PoA turn.
- [ADR 0020](./0020-stall-poa-on-a-missed-turn.md) requires one proposal per PoA Turn, exact retry,
  and fail-closed handling of Equivocation rather than first-wins selection.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) supplies authenticated source
  membership and governance-root binding.
- [ADR 0022](./0022-authenticate-peer-sessions-with-mutual-ed25519-challenge-response.md) keeps peer
  authentication distinct from bridge proof authority.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) defines exact
  envelope replication and Commit Receipts consumed as source and target convergence evidence.
- [ADR 0024](./0024-commit-canonical-post-block-public-provenance-state.md) binds bridge-origin
  evidence and requires independent target state commitment.
- [ADR 0025](./0025-enforce-the-package-semantic-profile-at-final-admission.md) and
  [ADR 0026](./0026-validate-the-full-staged-union-and-require-candidate-focus.md) require complete
  target semantic enforcement without a bridge exemption.

## References

- [RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA)](https://www.rfc-editor.org/rfc/rfc8032)
- [`ed25519-dalek` 2.2.0 `VerifyingKey::verify_strict`](https://docs.rs/ed25519-dalek/2.2.0/ed25519_dalek/struct.VerifyingKey.html#method.verify_strict)
- [`ed25519-dalek` 2.2.0 strict-verifier source](https://docs.rs/ed25519-dalek/2.2.0/src/ed25519_dalek/verifying.rs.html)
- [`ed25519-dalek` 2.2.0 scalar-parser feature boundary](https://docs.rs/ed25519-dalek/2.2.0/src/ed25519_dalek/signature.rs.html)

## Implementation Status

Accepted architecture; implementation is pending. Current `src/interop/bridge.rs` signs an
ambiguous delimiter-joined string over copied block fields with one caller-supplied key, accepts a
caller UUID, permits process-time bare-key trust, treats optional candidate-local SHACL as enough,
stores replay IDs only in a process-local `HashSet`, and calls `Blockchain::add_block` without
preserving source evidence. Its source block may be an unpersisted in-memory value, and its tests do
not use the Ledger Journal, authenticated membership, three Commit Receipts, source or target Final
Admission, a unified PoA Proposal Coordinator, a crash-safe Signing Fence, the strict receipt
verification profile, restart recovery, or two three-node networks. No Rust source, dependency,
lockfile, toolchain, runtime data, benchmark, or conformance artifact changed with this decision.
`Cargo.lock` currently resolves `ed25519-dalek` `2.2.0`, but `Cargo.toml` still declares the broader
`2.0` compatibility range with `serde` and default features. A lockfile does not record the resolved
feature graph, so the exact version/default-off/allowed-feature declaration and evidence that
`legacy_compatibility` is absent remain pending activation prerequisites.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-31
