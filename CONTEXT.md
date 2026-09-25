# ProvChain Domain Language

ProvChain is a permissioned traceability network whose participating organizations share operational and semantic contracts for provenance records. This glossary fixes project-specific terms that otherwise blur research evidence, implementation state, and deployment claims.

## Delivery Scope

**Thesis-Defensible End-to-End Reference System**:
A bounded research system whose implemented end-to-end behavior and reproducible evidence support the thesis claims explicitly kept in scope. It does not imply operational production readiness.
_Avoid_: production-ready system, production pilot, complete platform

**Production Pilot**:
An externally operated deployment assessed for operational security, availability, recovery, upgrades, and organizational controls. It is a later milestone, not a synonym for the thesis reference system.
_Avoid_: reference system, thesis prototype

## Ledger Admission

**Block Proposal**:
A proposed block and its proposer attestation offered to the active consensus protocol. It expresses intent but does not establish Final Admission or commitment.
_Avoid_: admission candidate, committed block, transaction

**Proposal Digest**:
The cryptographic commitment to all admission-relevant proposal fields that the Scheduled Authority signs before Final Admission.
_Avoid_: envelope hash, ledger prefix hash, post-state commitment

**Admission Candidate**:
The complete but uncommitted block and supporting evidence presented to Final Admission for a yes-or-no decision. It remains untrusted until every admission gate succeeds.
_Avoid_: admitted block, block proposal, committed block

**Final Admission**:
The single fail-closed transition that either rejects an Admission Candidate without changing authoritative ledger history or durably records it as a Committed Block.
_Avoid_: proposal validation, RDF insertion, consensus vote

**Admitted Block Envelope**:
The canonical, self-contained record of a block that has passed admission, including the provenance content and evidence needed to verify its semantic contract, chain position, signer, privacy state, and any bridge origin.
_Avoid_: block metadata, RDF payload, block proposal

**Envelope Hash**:
The cryptographic identity of one complete canonical Admitted Block Envelope, including its proposer evidence.
_Avoid_: proposal digest, ledger prefix hash, post-state commitment

**Committed Block**:
A node-relative Admitted Block Envelope that belongs to that node's durable ledger history because Final Admission recorded it. Projection availability and other nodes' progress do not change its committed status.
_Avoid_: proposed block, cached block, stored RDF graph

**Ledger Journal**:
The authoritative append-only history of Committed Blocks and their order, from which the node's derived ledger views can be restored.
_Avoid_: RDF store, chain index, WAL

**Ledger Projection**:
A derived representation of committed history for runtime access or query. It may be rebuilt without changing which blocks are committed.
_Avoid_: source of truth, ledger journal

**Verified Journal Replay**:
The integrity-checked reading of already committed Ledger Journal history before rebuilding Ledger Projections. It verifies history but does not readmit blocks or change commitment.
_Avoid_: final admission, chain synchronization, journal repair

**Public Provenance State**:
The consensus-visible dataset of asserted public RDF provenance through one committed ledger position. It excludes semantic-contract artifacts, derived or inferred facts, confidential material, and projection bookkeeping.
_Avoid_: Oxigraph contents, inferred knowledge graph, private view

**Post-State Commitment**:
An envelope-bound cryptographic commitment to the Public Provenance State that results after applying that envelope's block.
_Avoid_: pre-block state root, whole-store root, ledger prefix hash

**State Commitment Scheme**:
The versioned network contract that fixes how Public Provenance State is constructed, canonicalized, bounded, and cryptographically committed.
_Avoid_: hash algorithm alone, local RDF-store configuration

## Semantic Contract and Admission

**Ontology Package**:
The content-addressed, versioned semantic contract shared by one permissioned network, comprising its provenance and domain ontologies, declared shapes, mappings, and conformance assets.
_Avoid_: local ontology directory, domain configuration, ontology file alone

**Semantic Execution Profile**:
The versioned network rule set that fixes the accepted semantic constructs and the deterministic validation and entailment behavior used for admission.
_Avoid_: full SHACL support, validator configuration, best-effort validation

**Semantic Conformance**:
The reproducible result that a staged post-block Public Provenance State satisfies every applicable constraint in the active Ontology Package under its Semantic Execution Profile.
_Avoid_: package compatibility, proposal preflight, validation without focus nodes

**SHACL Data Graph**:
The deterministic set-union graph derived from every asserted triple in staged post-block Public Provenance State for semantic validation. It is a validation view, not the committed named-graph dataset.
_Avoid_: candidate graph, Oxigraph store, Post-State Commitment input

**Candidate Focus Node**:
A package-selected focus node in the staged SHACL Data Graph that is also the subject of at least one public assertion in the Admission Candidate.
_Avoid_: any RDF subject, referenced object, repeated target-class declaration

**Admission Kind**:
The closed, signed envelope classification that selects a type-specific Final Admission rule. It is protocol evidence rather than a caller-declared bypass.
_Avoid_: request type, optional tag, semantic-validation exemption

## Network Consensus and Convergence

**Signer Authorization**:
The determination that an authenticated signer belongs to the active Membership Manifest's authorized signer set for the claimed role. It does not establish eligibility for a particular ledger turn.
_Avoid_: peer authentication, signature verification alone, consensus acceptance

**Scheduled Authority**:
The authorized PoA validator exclusively assigned to propose the next block for a particular ledger turn under the network's deterministic schedule.
_Avoid_: any authorized validator, connected peer, consensus quorum

**Consensus Acceptance**:
Independently verifiable protocol evidence that a Block Proposal satisfied the active consensus rule for a particular ledger position. It makes the proposal eligible for Final Admission but does not make it committed.
_Avoid_: signer authorization, final admission, durable commitment

**PoA Turn**:
The assignment of the next ledger position to one Scheduled Authority under the deterministic round-robin authority order declared by the active Network Profile.
_Avoid_: wall-clock timeout, follower quorum, PBFT view

**Stalled Network**:
A safety-preserving condition in which no new block is eligible because the Scheduled Authority has not produced the next proposal. Existing committed history remains unchanged.
_Avoid_: converged network, failed ledger, authority takeover

**Equivocation**:
The production of distinct signed Block Proposals for the same PoA Turn by the same Scheduled Authority. It is a consensus fault and must not be treated as a fork-choice input.
_Avoid_: retry, timestamp conflict, valid alternative block

**PoA Proposal Coordinator**:
The one per-ledger authority-side component that serializes every unsigned request for the pending PoA Turn, coalesces byte-identical requests, and completes read-only preflight before selecting at most one exact proposal for signing.
_Avoid_: Final Admission, request queue with independent signers, fork choice

**PoA Signing Fence**:
The crash-safe, single-writer, non-ledger-commit safety record that binds one ledger instance, PoA Turn, key reference, and exact canonical unsigned proposal body before the Scheduled Authority signs, so recovery can reproduce or rebroadcast only the resulting exact signed proposal.
_Avoid_: Committed Block, Ledger Journal commit, replay database, second proposal slot

Once proposal selection or fence persistence begins, the exact body is no longer refreshable even
if the signature response is unknown. Parent refresh is allowed only while the request is
unselected, fence persistence has not begun, and the signer has not been invoked.

**Network Convergence**:
The condition in which all required reference nodes have committed an identical ledger prefix through a target position.
_Avoid_: local commitment, availability, PBFT finality

**Network-Converged Block**:
A Committed Block whose identical Admitted Block Envelope and preceding ledger history have also been committed at the same position by all required reference nodes.
_Avoid_: locally committed block, quorum-certified block, projected block

**Commit Receipt**:
Node-signed evidence that a particular Admitted Block Envelope and its ledger prefix were durably committed at that node. It is convergence evidence, not a consensus vote or commit authority.
_Avoid_: acknowledgement, quorum vote, block signature

**Ledger Prefix Hash**:
A deterministic commitment to the exact ordered Admitted Block Envelopes from genesis through a stated ledger position.
_Avoid_: tip block hash, post-state commitment, chain length

## Network Membership

**Membership Manifest**:
The versioned, governance-authenticated declaration of the network identities and roles admitted under an active Network Profile. It is the sole membership source of truth for the permissioned network.
_Avoid_: peer list, bootstrap configuration, authority key list

**Network Member**:
An organization or node identity whose verification key and claimed role are active in the Membership Manifest.
_Avoid_: connected peer, semantically compatible node, discovered node

**Governance Trust Root**:
The pre-established authority identity that a node trusts to authenticate the Membership Manifest bound to its Network Profile.
_Avoid_: validator key, peer identity, TLS certificate alone

**Node Identity Key**:
The member key dedicated to proving a node's identity during peer authentication. It is distinct from governance, wallet, and PoA block-signing keys.
_Avoid_: authority key, validator key, governance key

**Authenticated Peer Session**:
A live peer association whose endpoints mutually proved possession of active Membership Manifest identity keys through a fresh, mutually bound exchange. It establishes membership at session setup but does not imply transport confidentiality or authentication of every later message.
_Avoid_: connected socket, semantically compatible peer, secure transport

## Bridge Interchange

**Ledger Instance Identifier**:
The public nonzero 32-byte CSPRNG value selected before genesis and embedded in one ledger's genesis-bound Network Profile, used with its network ID to distinguish that exact history without a mutual profile/genesis-hash cycle.
_Avoid_: network ID, profile ID, genesis Envelope Hash, current tip, informal epoch

**Bounded ProvChain-to-ProvChain Bridge**:
The one-hop import of one exact public `OrdinaryProvenanceV1` payload between two explicitly bound ProvChain ledger instances under `ProvChainBridgeSuiteV1`.
_Avoid_: asset bridge, lock-and-mint, heterogeneous bridge, SPV bridge, ontology mapping

**Bridge Source Trust Binding**:
The target Network Profile's pre-established binding to one exact source ledger, source profile and signed Membership Manifest, Governance Trust Root, three source receipt signers, and permitted bridge and semantic contracts.
_Avoid_: proof-supplied key, trusted-authority list, peer session, matching network name

**Bridge Export Declaration**:
The source-envelope-bound declaration that one exact public payload at one source position is intended for one exact target ledger under the bounded copy protocol.
_Avoid_: relayer request, post-commit export proof, asset lock, caller transfer ID

**Prefix-Bound Bridge Export Envelope**:
A source Admitted Block Envelope whose proposer evidence authenticates its parent Ledger Prefix, allowing a target to verify the exported envelope's one-step prefix transition.
_Avoid_: receipt parent hash, bridge proof prefix, retrofitted export evidence

**Bridge Transfer ID**:
The domain-separated deterministic digest of one canonical Bridge Export Declaration core, including both ledger instances, the source position, and exact public-payload digest.
_Avoid_: UUID, request ID, source envelope hash, transport correlation ID

**StrictEd25519V1**:
The bridge-receipt signature profile whose acceptance predicate is the exact `ed25519-dalek` 2.2.0 `VerifyingKey::verify_strict` result over the receipt's 32-byte domain hash under the default-off `fast,zeroize` feature graph, with `legacy_compatibility` forbidden and no ordinary, batch, prehash, context, or compatibility verifier substitute.
_Avoid_: any valid Ed25519 signature, implementation-selected strictness, proposal signature profile

**Source Convergence Evidence**:
The verifiable relation established by one exact declaration-bearing source envelope and matching Commit Receipts from all three pinned source nodes for the same ledger position, Envelope Hash, and Ledger Prefix Hash.
_Avoid_: SPV proof, PBFT finality, trustless finality, one authority signature

**Bridge Proof Bundle**:
The canonical five-field transfer artifact containing the Bridge Transfer ID, exact source profile, exact signed manifest, exact source envelope, and exactly three receipts that collectively establish Source Convergence Evidence.
_Avoid_: copied state root, relayer signature, block payload alone

**Bridge Origin Evidence**:
The exact verified Bridge Proof Bundle and its binding committed in the target Admitted Block Envelope.
_Avoid_: RDF provenance payload, bridge log, external receipt database

**Bridge Import**:
The adapter action that presents an unsigned exact source-declared public payload and Bridge Origin Evidence request to the shared target PoA Proposal Coordinator, after which only the selected Scheduled Authority proposal can enter ordinary Final Admission.
_Avoid_: direct append, mapped import, source-authorized target commit

**Effective Bridge State**:
The target-journal-derived map of Bridge Transfer IDs to terminal Imported source and target references. It contains no authoritative Pending, Validated, or Rejected state.
_Avoid_: in-memory replay set, bridge database, delivery queue

**AlreadyImported**:
The derived bridge-adapter response when an exact retry names the same committed Bridge Transfer ID, proof, payload, source reference, and target ledger instance. It returns the stored target reference, performs no append, and is not a Final Admission outcome.
_Avoid_: new commit, replay acceptance, delivery acknowledgement

**ReplayConflict**:
The derived fail-closed bridge-adapter response when an existing Bridge Transfer ID is reused with different proof, payload, source reference, or target ledger instance. It creates no candidate or append and is not a Final Admission outcome.
_Avoid_: duplicate retry, fork choice, overwrite

**Bridge-Converged**:
The condition in which all three target nodes have committed the identical bridge-origin target envelope and prefix. It is distinct from source convergence and target-local Imported state.
_Avoid_: transfer complete, local import, source finality

## Privacy Identity

**Participant Principal**:
The stable UUID-based subject that owns protected data and receives privacy grants. It persists across login-account, wallet-location, and key-version changes.
_Avoid_: username, wallet, key ID, Network Member

**Participant Key Purpose**:
A schema-known, Network-Profile-enabled role that separates how a Participant Key Version may be used and pins its allowed algorithm, encoding, fingerprint, and possession-proof verifier. A key enabled for one purpose cannot silently satisfy another.
_Avoid_: API role, free-form key label, key ID, Network Member role

**Privacy Key-Wrapping Purpose**:
The closed `privacy-key-wrapping` Participant Key Purpose used to receive a Protected-Object DEK inside an Owner or Grant DEK Envelope, with the sole non-decapsulation use being ADR 0034's narrow pre-binding public possession proof.
_Avoid_: privacy-authorization, wallet-at-rest key, PoA key, free-form encryption key

**ProtectedDataSuiteV1**:
The single mandatory Network-Profile-pinned protected-data contract for bounded v1: HKDF/HMAC-SHA256 subkeys and content commitment, one-use IETF ChaCha20-Poly1305 payload protection, P-256 wrapping keys with deterministic low-S ECDSA possession proof, and RFC 9180 Base-mode P-256/HKDF-SHA256/ChaCha20Poly1305 DEK envelopes. Objects and grants cannot negotiate alternatives. The design is accepted, but implementation and profile activation remain fail closed pending conformance evidence.
_Avoid_: algorithm preference, cipher menu, per-object suite, local wallet setting

**Canonical Privacy Encoding V1**:
The closed binary record codec paired with `ProtectedDataSuiteV1`. It fixes record and field tags, field order, u32 length framing, big-endian integers, raw UUID/hash/key/signature forms, bounds, and strict rejection of unknown, duplicate, missing, noncanonical, or trailing bytes for every `PrivacyControlV1` context, core, proof, transition, envelope, and release-evidence record.
_Avoid_: Serde format, JSON schema, Rust struct layout, implementation-defined encoding

**Participant Key Version**:
An immutable, purpose-bound public-key binding for one Participant Principal and version. Its private counterpart remains outside authoritative ledger state under Participant Key Custody.
_Avoid_: Participant Principal, shared secret, Node Identity Key

**Participant Key Candidate**:
An unbound client-local private key plus its authenticated prospective public binding and, when prepared for disclosure, one exact durable proof intent with either the exact authorizer request or the complete authorized submission for its current stage. It is quarantined and has no ledger lifecycle status until the matching binding commits.
_Avoid_: Participant Key Version, Active key, pending ledger key, retryable arbitrary signer

**Participant Key Custody**:
The participant-controlled, client-only, non-authoritative durable holding of private counterparts for exact Participant Key Versions and quarantined Participant Key Candidates. Node, server, ledger, projection, account, and JWT state never become custody merely because they know a public binding.
_Avoid_: server wallet, node-local key store, ledger key, account credential

**ParticipantKeystoreSuiteV1**:
The single mandatory bounded-v1 contract for encrypting, committing, recovering, backing up, and restoring one participant's client-only custody state. It is a local capability-preservation contract and never a source of ledger authority.
_Avoid_: wallet format, cipher preference, server keystore, Network Profile negotiation

**Custody Snapshot**:
One immutable encrypted whole-store generation containing the exact retained key records and prepared disclosure state for one network and Participant Principal.
_Avoid_: wallet row, mutable key file, ledger snapshot, projection

**Custody Snapshot Reference**:
The local tuple of file ID, generation, and digest of the exact complete Custody Snapshot bytes. It identifies an expected local lineage state but is not a freshness or anti-rollback witness.
_Avoid_: Ledger Prefix Hash, Envelope Hash, Commit Receipt, latest-file timestamp

**Expected Custody Target**:
The exact current Custody Snapshot Reference, or `ABSENT` for genesis or restore, that a local whole-snapshot publication must still match. It is independent of the Expected Ledger Parent carried by a privacy transition.
_Avoid_: Expected Ledger Parent, newest generation, current ledger tip

**Custody Store Commit**:
The successful participant-local publication of one complete Custody Snapshot under the accepted durability contract. It preserves a client capability but does not commit a block, authorize a transition, or establish Network Convergence.
_Avoid_: Ledger Journal Commit, Final Admission, Committed Block, Commit Receipt

**Prepared Governance Request**:
The durable client-local state containing a generated bootstrap candidate and the one exact governance request that may be disclosed byte for byte. It has no ledger lifecycle status or authority.
_Avoid_: pending ledger key, registration approval, retryable request template

**Prepared Node Submission**:
The durable client-local state containing a generated candidate and the one exact complete authorized transition that may be submitted or retried byte for byte. Node receipt is not evidence that it committed.
_Avoid_: committed transition, mutable transaction draft, regenerated retry

**Restore Quarantine**:
The client-local state in which an encrypted Custody Backup is authenticated, rederived, and reconciled without becoming a live key source. Installation requires a new Custody Snapshot and an absent live target.
_Avoid_: restored wallet, staging ledger, backup mount, live merge

**RestoreNoInstallableState**:
The fail-closed restore result when current ledger reconciliation would leave no encodable live custody entry. The source remains durable in Restore Quarantine and the live target remains absent; the client neither invents an entry nor reports restore success.
_Avoid_: empty wallet, successful restore, Custody Gap, deleted backup

**CommitIndeterminate**:
The custody condition in which local publication may have crossed rename but directory durability is not yet proved. All private operations and outbound prepared bytes remain blocked until exact recovery resolves it.
_Avoid_: aborted write, successful commit, retry from reconstructed state

**Revocation Rewrite Pending**:
The fail-closed custody condition after verified ledger Revocation and before a new live Custody Snapshot has replaced the affected secret with a public tombstone. Ledger-derived non-use is immediate even though byte removal is not secure erasure.
_Avoid_: key still active, secure deletion, local revocation authority

**Custody Backup**:
An immutable exact encrypted copy of one committed Custody Snapshot that can be restored only through quarantine, full verification, current ledger reconciliation, and a newly encrypted successor. It is neither server escrow nor authority to change a key's lifecycle.
_Avoid_: plaintext wallet export, node backup, recovery override, key registry

**Custody Gap**:
The local condition in which a ledger-recorded Participant Key Version lacks its usable private counterpart in Participant Key Custody. It blocks the affected local authorization or decryption capability without changing the key's ledger-derived status.
_Avoid_: Revoked key, Retired key, Participant-Control Freeze, ledger inconsistency

**Participant Key Lifecycle Status**:
The ledger-derived `Active`, `Retired`, or `Revoked` state of a Participant Key Version. Only the Active authorization key may authorize a new participant transition, and only the Active wrapping key may be selected for a new object or grant envelope. A Retired wrapping key remains eligible solely to open an already-current immutable envelope that names that exact version during Live Privacy Release; a Revoked key is never eligible. `Retired` is terminal supersession produced only by atomic rotation; `Revoked` is terminal emergency disablement; neither rewrites validity at an earlier recorded parent.
_Avoid_: account status, certificate status, grant status, wallet availability

**Participant Key Rotation**:
The atomic `BindParticipantKey` change that activates the next contiguous key version for one principal and purpose while retiring its prior Active version.
_Avoid_: key overwrite, status-only transition, bootstrap, reactivation

**Participant-Control Freeze**:
The irreversible v1 condition caused by revoking a principal's sole Active `privacy-authorization` key, preventing that principal from authorizing later participant-controlled transitions without deleting the principal, ownership, grants, or history.
_Avoid_: account disablement, object deletion, grant revocation, temporary suspension

**Transition Authorization Proof**:
The closed bootstrap-governance-or-participant proof attached to one Privacy Control Transition to establish who authorized it at the verified parent. It is independent of PoA Consensus Acceptance.
_Avoid_: PoA signature, Node Identity proof, JWT, wallet credential

**Participant Bootstrap**:
The single committed `RegisterPrincipal` transition that atomically creates a Participant Principal and binds and activates its initial version-1 `privacy-authorization` Participant Key Version.
_Avoid_: account signup, sequential registration, self-registration, key recovery

**Privacy Bootstrap Governance Key**:
The distinct, versioned Network Profile key binding authorized only to approve Participant Bootstrap.
_Avoid_: Governance Trust Root, Membership Manifest signer, PoA key, privacy administrator

**Participant Bootstrap Authorization**:
A proof signed by the dedicated privacy-bootstrap governance key bound by the active Network Profile, used only to authorize Participant Bootstrap when no parent-state participant key can yet do so.
_Avoid_: self-registration, administrator override, Participant Authorization Signature

**Participant Key Possession Proof**:
A non-authorizing, domain-separated proof generated with a newly introduced participant key under the purpose-specific scheme pinned by the Network Profile, proving control of the private counterpart without authorizing its own registration or binding. `privacy-authorization` uses strict raw Ed25519; `privacy-key-wrapping` uses deterministic SHA-256 ECDSA over the same P-256 point, encoded as canonical low-S raw `r32 || s32` solely for binding-time proof. Unsupported or incompletely specified schemes reject.
_Avoid_: Participant Authorization Signature, Participant Bootstrap Authorization, PoA signature, wallet existence

**Participant Authorization Signature**:
A domain-separated signature over one canonical Privacy Control Transition and its parent-ledger anchor, made with the required active Participant Key Version resolved from parent Effective Privacy State. It proves participant intent, not PoA acceptance or API login.
_Avoid_: JWT signature, wallet possession, transaction signature, Proposal Digest signature

**Privacy Control Transition**:
A signed, committed change to principal/key bindings, protected-object ownership, grants, or revocations under the closed privacy-control protocol.
_Avoid_: wallet update, ACL edit, API permission change

**Effective Privacy State**:
The authoritative owner, key, grant, and revocation state obtained by folding committed Privacy Control Transitions through one ledger position.
_Avoid_: wallet ACL, JWT role, current cache

## Protected Data and Grants

**Protected Content Bytes**:
The exact opaque non-empty byte string supplied to `ProtectedDataSuiteV1`, bounded to 262,144 bytes. V1 applies no text, RDF, media, compression, or padding transformation, so ciphertext length reveals its exact length.
_Avoid_: canonical RDF, normalized text, padded payload, external object

**Protected Object**:
An immutable, uniquely identified ledger resource that binds one owner Participant Principal to one exact Protected Payload Ciphertext and one Owner DEK Envelope. It contains no protected plaintext or unwrapped DEK.
_Avoid_: private block, external object locator, Privacy Grant, plaintext dataset

**Protected-Object Data-Encryption Key (DEK)**:
A fresh CSPRNG-generated 32-byte root key scoped to exactly one Protected Object. Suite-derived one-use subkeys protect its payload and keyed content commitment, while Owner and Grant DEK Envelopes wrap the same raw root DEK. Only encrypted envelopes of it enter the ledger.
_Avoid_: Participant Key Version, owner key, shared secret, key ID

**Protected Payload Ciphertext**:
The single immutable canonical `ProtectedDataSuiteV1` record committed by a Protected Object. It contains the suite identifier, fixed zero12 nonce, Protected Content Commitment, and exact IETF ChaCha20-Poly1305 ciphertext plus full tag produced once under a one-use derived payload key with raw `O` as AAD. Authenticity of its content is established only by a DEK holder, not by Public Envelope Validation. Grants add DEK Envelopes without creating another payload record.
_Avoid_: encrypted wallet, per-grantee ciphertext, plaintext, external payload locator

**Payload Ciphertext-and-Tag Bytes**:
The exact opaque AEAD ciphertext and authentication-tag byte field inside one Protected Payload Ciphertext record.
_Avoid_: Protected Payload Ciphertext record, plaintext, Encrypted Payload Commitment

**Protected Content Commitment**:
The full 32-byte HMAC-SHA256 value over exact Protected Content Bytes under a DEK-derived, domain-separated key, constructed before the Object Encryption Context Digest. It is independent of that digest, all ciphertext, every Owner or Grant DEK Envelope, and outer signatures. Public validators check only its canonical form and signed binding, not its keyed correctness.
_Avoid_: Encrypted Payload Commitment, public plaintext hash, payload authentication by validators

**Object Encryption Context Digest**:
The public, verifier-recomputable 32-byte digest `O` over one Protected Object's canonical pre-encryption context. It is bound into payload AEAD context and the Owner DEK Envelope by conforming constructors, but public recomputation alone does not prove that opaque cryptographic operations used it.
_Avoid_: Encrypted Payload Commitment, transition identifier, state root, decryption proof

**Encrypted Payload Commitment**:
The public, verifier-recomputable, domain-separated SHA-256 value over the exact Canonical Privacy Encoding V1 Protected Payload Ciphertext record. It binds durable bytes but does not prove plaintext authenticity or correct encryption.
_Avoid_: Protected Content Commitment, state root, plaintext hash, decryption proof

**DEK Envelope**:
The umbrella term for a versioned canonical record that addresses one Protected-Object DEK to one exact wrapping-key version. Its only bounded-v1 roles are Owner DEK Envelope and Grant DEK Envelope; there is no generic envelope transition or multi-recipient variant.
_Avoid_: raw DEK, shared secret, Privacy Grant, ciphertext

**Owner DEK Envelope**:
The immutable closed RFC 9180 Base-mode P-256 DEK Envelope committed with a Protected Object. It contains `O`, an exact public header, `enc65`, and the 48-byte encrypted root DEK addressed to the owner's exact wrapping-key version. It is neither ownership evidence nor public proof of decryptability.
_Avoid_: Privacy Grant, owner signature, raw DEK, wallet secret

**Privacy Grant**:
A uniquely identified, ledger-authoritative authorization relationship from one Protected Object owner to one grantee Participant Principal. While Active, it is a necessary grant-authority condition for Live Privacy Release; it does not itself convey a decryption key, prove decryptability, or suffice for release.
_Avoid_: ACL entry, API role, Recipient-Wrapped Key Material, key possession

**Prospective Grant Revocation**:
The terminal end of a Privacy Grant's authority. For Live Privacy Release it takes effect when the exact verified Network-Converged release prefix includes the revocation; it preserves prior history and cannot recall key material or plaintext already disclosed.
_Avoid_: erasure, retroactive invalidation, key deletion, automatic re-encryption

**Grant Delivery Material**:
The versioned canonical field `D` bound into one `GrantAccess` transition whose value is exactly one Grant DEK Envelope and has no sibling delivery fields. Every required public reference is inside that envelope's canonical header. It is neither a delivery receipt nor proof of decryptability.
_Avoid_: delivery receipt, decryptability proof, grant authority, raw DEK

**Grant DEK Envelope**:
The grant-specific closed RFC 9180 Base-mode P-256 DEK Envelope containing `Q`, an exact public header, `enc65`, and the 48-byte encryption of the existing object's same root DEK to the grantee's exact wrapping-key version. It may enable recovery with the matching private key but is not the Privacy Grant or grant authority.
_Avoid_: Recipient-Wrapped Key Material, Privacy Grant, permission, raw DEK, shared secret

**Delivery Context Digest**:
The public, verifier-recomputable 32-byte digest `Q` over one proposed grant's canonical pre-delivery context. It precedes Grant Identifier `G` to avoid a cycle and is bound into Grant DEK Envelope processing by conforming constructors; public recomputation alone does not prove internal KDF/AAD use.
_Avoid_: Grant Identifier, Grant Delivery Material, delivery receipt, decryption proof

**Public Envelope Validation**:
Final Admission's walletless verification of Canonical Privacy Encoding V1, the mandatory suite, P-256 points and low-S possession proof, public-key references and statuses, bounds, `O`, `Q`, EPC, signed bindings, and exact envelope bytes. It does not prove keyed PCC or payload-tag correctness, internal HPKE context use, DEK freshness, same-DEK construction, private-key custody, or decryptability.
_Avoid_: decryption test, recipient acknowledgement, key possession, plaintext validation

**Live Privacy Release**:
A current-state, network-mediated decision that returns the exact Protected Payload Ciphertext and exactly one applicable Owner or Grant DEK Envelope plus response evidence bound to one exact verified Network-Converged release prefix, object, authenticated requester, selected recipient key, and owner/grant path. The exact referenced key may be Active or Retired but never Revoked; it is never replaced by the newest key. The requester must equal the current owner on the owner path or both the Active grant's grantee and selected wrapping-key principal on the grantee path. It never returns plaintext or an unwrapped DEK and is never authorized by a historical audit view.
_Avoid_: historical query, offline decryption, grant-history report, integrity replay, decryption oracle
