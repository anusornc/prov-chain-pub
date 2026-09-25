//! Acceptance tests for protected-object creation and client-only custody (Issue #9).

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;

use chacha20poly1305_v11::aead::{Aead, KeyInit, Payload};
use chacha20poly1305_v11::ChaCha20Poly1305;
use ed25519_dalek::{Signer, SigningKey};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use hpke::aead::ChaCha20Poly1305 as HpkeChaCha20Poly1305;
use hpke::kdf::HkdfSha256;
use hpke::kem::DhP256HkdfSha256;
use hpke::{
    setup_receiver, setup_sender_with_rng, Deserializable, Kem as KemTrait, OpModeR, OpModeS,
    Serializable,
};
use p256::ecdsa::{
    signature::Signer as _, Signature as P256Signature, SigningKey as P256SigningKey,
};
use provchain_org::custody::{
    CustodyCommitFailpoint, CustodyError, CustodyPassphrase, CustodyRecovery,
    CustodySnapshotReference, ParticipantCustody,
};
use provchain_org::ledger::{
    AdmissionKind, AdmissionOutcome, Ledger, LedgerCommitFailpoint, LedgerProfile,
};
use provchain_org::network::convergence::{NetworkConvergenceEvidence, PrivacyStateReceipt};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember,
};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::privacy::{
    EffectivePrivacyState, GrantDekEnvelope, LivePrivacyReleaseRequest, OwnerDekEnvelope,
    ParticipantKeyReference, PrivacyAdmissionAnchor, PrivacyControlTransition,
    PrivacyLifecycleConformanceProfile, PrivacyLifecycleProfile, PrivacyPermission,
    PrivacyReleaseEnvelope, ProtectedPayloadCiphertext, UnsignedPrivacyTransition,
    MAX_PROTECTED_CONTENT_BYTES, PROTECTED_DATA_SUITE_V1,
};
use rand_core_v10::{Infallible, TryCryptoRng, TryRng};
use sha2::{Digest, Sha256};
use sha2_v11::{Digest as DigestV11, Sha256 as Sha256V11};
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue9";
const PROFILE_ID: &str = "issue9.reference";

fn seeded_ed25519(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seeded_p256(seed: u8) -> P256SigningKey {
    P256SigningKey::from_slice(&[seed; 32]).expect("valid deterministic P-256 fixture scalar")
}

fn p256_public(key: &P256SigningKey) -> [u8; 65] {
    key.verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 public key")
}

fn p256_proof(key: &P256SigningKey, digest: &[u8; 32]) -> [u8; 64] {
    let signature: P256Signature = key.sign(digest);
    signature.normalize_s().to_bytes().into()
}

fn encode_record(tag: u8, fields: &[&[u8]]) -> Vec<u8> {
    let mut bytes = b"PCV1".to_vec();
    bytes.push(tag);
    bytes.push(u8::try_from(fields.len()).expect("bounded fixture fields"));
    for (index, field) in fields.iter().enumerate() {
        bytes.push(u8::try_from(index + 1).expect("bounded fixture field tag"));
        bytes.extend_from_slice(
            &u32::try_from(field.len())
                .expect("bounded fixture field")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(field);
    }
    bytes
}

fn domain_hash(domain: &str, payload: &[u8]) -> [u8; 32] {
    Sha256::digest(encode_record(0x01, &[domain.as_bytes(), payload])).into()
}

struct FixedHpkeRng {
    entropy: [u8; 32],
    consumed: bool,
    invalid_use: bool,
}

impl FixedHpkeRng {
    fn new(entropy: [u8; 32]) -> Self {
        Self {
            entropy,
            consumed: false,
            invalid_use: false,
        }
    }

    fn exactly_consumed(&self) -> bool {
        self.consumed && !self.invalid_use
    }
}

impl TryRng for FixedHpkeRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        self.invalid_use = true;
        Ok(0)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.invalid_use = true;
        Ok(0)
    }

    fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), Self::Error> {
        if self.consumed || destination.len() != self.entropy.len() {
            self.invalid_use = true;
            destination.fill(0);
            return Ok(());
        }
        destination.copy_from_slice(&self.entropy);
        self.consumed = true;
        Ok(())
    }
}

impl TryCryptoRng for FixedHpkeRng {}

fn parent_reference(parent: [u8; 32]) -> [u8; 33] {
    let mut reference = [0_u8; 33];
    reference[0] = 1;
    reference[1..].copy_from_slice(&parent);
    reference
}

#[allow(clippy::too_many_arguments)]
fn deterministic_protected_object(
    profile: &PrivacyLifecycleProfile,
    anchor: PrivacyAdmissionAnchor,
    owner: Uuid,
    authorization_reference: ParticipantKeyReference,
    wrapping_reference: ParticipantKeyReference,
    authorization_key: &SigningKey,
    wrapping_key: &P256SigningKey,
    object_id: Uuid,
    dek: [u8; 32],
    hpke_entropy: [u8; 32],
    plaintext: &[u8],
) -> PrivacyControlTransition {
    let extract_salt: [u8; 32] =
        Sha256V11::digest(b"provchain/protected-data/dek-extract-salt/v1").into();
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(&extract_salt), &dek);
    let pcc_info = encode_record(
        0x02,
        &[
            b"provchain/protected-data/pcc-key/v1",
            PROTECTED_DATA_SUITE_V1.as_bytes(),
        ],
    );
    let mut pcc_key = [0_u8; 32];
    hkdf.expand(&pcc_info, &mut pcc_key)
        .expect("derive deterministic PCC key");
    let mut pcc_mac =
        <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(&pcc_key).expect("PCC HMAC key");
    pcc_mac.update(&encode_record(
        0x04,
        &[b"provchain/protected-data/pcc/v1", plaintext],
    ));
    let pcc: [u8; 32] = pcc_mac.finalize().into_bytes().into();

    let position = anchor.expected_ledger_position().to_be_bytes();
    let revision = anchor.expected_privacy_revision().to_be_bytes();
    let profile_hash = anchor.profile_content_hash();
    let parent_prefix = anchor.parent_ledger_prefix_hash();
    let parent = anchor
        .parent_envelope_hash()
        .map(parent_reference)
        .unwrap_or([0_u8; 33]);
    let payload_tag = [0x21];
    let envelope_tag = [0x23];
    let wrapping_reference_bytes = wrapping_reference.canonical_bytes();
    let context = encode_record(
        0x20,
        &[
            b"PrivacyControlV1",
            b"CanonicalPrivacyEncodingV1",
            &[0x04],
            anchor.network_id().as_bytes(),
            anchor.profile_id().as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &parent_prefix,
            &parent,
            object_id.as_bytes(),
            owner.as_bytes(),
            &wrapping_reference_bytes,
            &payload_tag,
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &envelope_tag,
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &pcc,
        ],
    );
    let object_context = domain_hash("provchain/protected-object/encryption-context/v1", &context);
    let payload_info = encode_record(
        0x03,
        &[
            b"provchain/protected-data/payload-key/v1",
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &object_context,
        ],
    );
    let mut payload_key = [0_u8; 32];
    hkdf.expand(&payload_info, &mut payload_key)
        .expect("derive deterministic payload key");
    let ciphertext_and_tag = ChaCha20Poly1305::new_from_slice(&payload_key)
        .expect("payload key")
        .encrypt(
            &[0_u8; 12].into(),
            Payload {
                msg: plaintext,
                aad: &object_context,
            },
        )
        .expect("seal deterministic payload");
    let payload_bytes = encode_record(
        0x21,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &[0_u8; 12],
            &pcc,
            &ciphertext_and_tag,
        ],
    );
    let encrypted_payload_commitment = domain_hash(
        "provchain/protected-data/encrypted-payload-commitment/v1",
        &payload_bytes,
    );
    let owner_header = encode_record(
        0x22,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            anchor.network_id().as_bytes(),
            anchor.profile_id().as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &parent_prefix,
            &parent,
            object_id.as_bytes(),
            owner.as_bytes(),
            &wrapping_reference_bytes,
            &pcc,
            &encrypted_payload_commitment,
        ],
    );
    let header_hash = domain_hash(
        "provchain/protected-data/hpke-owner-header/v1",
        &owner_header,
    );
    let mut owner_info = [0_u8; 64];
    owner_info[..32].copy_from_slice(&object_context);
    owner_info[32..].copy_from_slice(&header_hash);

    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_public = <Kem as KemTrait>::PublicKey::from_bytes(&p256_public(wrapping_key))
        .expect("deterministic HPKE recipient key");
    let mut rng = FixedHpkeRng::new(hpke_entropy);
    let (encapsulated, mut sender) = setup_sender_with_rng::<HpkeAead, HpkeKdf, Kem>(
        &OpModeS::Base,
        &recipient_public,
        &owner_info,
        &mut rng,
    )
    .expect("deterministic HPKE sender context");
    assert!(
        rng.exactly_consumed(),
        "HPKE consumes one exact 32-byte seed"
    );
    let wrapped_dek = sender.seal(&dek, b"").expect("wrap deterministic DEK");
    let encapsulated: [u8; 65] = encapsulated
        .to_bytes()
        .as_slice()
        .try_into()
        .expect("canonical HPKE encapsulated key");
    let wrapped_dek: [u8; 48] = wrapped_dek.try_into().expect("canonical wrapped DEK");
    let envelope_bytes = encode_record(
        0x23,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &object_context,
            &owner_header,
            &encapsulated,
            &wrapped_dek,
        ],
    );
    let unsigned = UnsignedPrivacyTransition::create_protected_object(
        profile,
        anchor,
        object_id,
        owner,
        authorization_reference,
        wrapping_reference,
        object_context,
        ProtectedPayloadCiphertext::decode(&payload_bytes).expect("canonical protected payload"),
        encrypted_payload_commitment,
        OwnerDekEnvelope::decode(&envelope_bytes).expect("canonical owner envelope"),
    )
    .expect("construct deterministic protected-object transition");
    let signature = authorization_key
        .sign(&unsigned.authorization_digest())
        .to_bytes();
    unsigned
        .complete_object_creation(signature)
        .expect("authorize deterministic protected-object transition")
}

fn custody_error<T>(result: Result<T, CustodyError>) -> CustodyError {
    match result {
        Ok(_) => panic!("expected participant-custody failure"),
        Err(error) => error,
    }
}

fn make_private_directory(path: &std::path::Path) {
    std::fs::create_dir(path).expect("create private fixture directory");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .expect("set private fixture directory mode");
}

fn make_private_file(path: &std::path::Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("write private fixture file");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .expect("set private fixture file mode");
}

fn assert_bytes_exclude(label: &str, bytes: &[u8], secrets: &[&[u8]]) {
    for secret in secrets {
        assert!(!secret.is_empty(), "secret fixture must be nonempty");
        assert!(
            !bytes.windows(secret.len()).any(|window| window == *secret),
            "client-only secret appeared in {label}"
        );
    }
}

fn assert_file_artifacts_exclude(root: &std::path::Path, secrets: &[&[u8]]) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).expect("enumerate controlled fixture artifacts") {
            let entry = entry.expect("read controlled fixture entry");
            let file_type = entry.file_type().expect("read fixture file type");
            if file_type.is_dir() {
                pending.push(entry.path());
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let bytes = std::fs::read(entry.path()).expect("read controlled fixture artifact");
            assert_bytes_exclude(
                &format!("artifact {}", entry.path().display()),
                &bytes,
                secrets,
            );
        }
    }
}

struct Issue9ConformanceContext {
    profile: PrivacyLifecycleProfile,
    capability: PrivacyLifecycleConformanceProfile,
    manifest: provchain_org::network::membership::SignedMembershipManifest,
    node_ids: [Uuid; 3],
    identity_keys: [SigningKey; 3],
}

fn conformance_lifecycle_for_ledger(bootstrap_key: &SigningKey) -> Issue9ConformanceContext {
    let governance_key = seeded_ed25519(111);
    let identity_keys = [
        seeded_ed25519(112),
        seeded_ed25519(114),
        seeded_ed25519(116),
    ];
    let validator_keys = [
        seeded_ed25519(113),
        seeded_ed25519(115),
        seeded_ed25519(117),
    ];
    let node_ids = [
        Uuid::from_u128(0x91_0111),
        Uuid::from_u128(0x91_0112),
        Uuid::from_u128(0x91_0113),
    ];
    let members = node_ids
        .iter()
        .zip(&identity_keys)
        .zip(&validator_keys)
        .map(|((&node_id, identity_key), validator_key)| NetworkMember {
            node_id,
            identity_public_key: identity_key.verifying_key().to_bytes(),
            roles: vec![MemberRole::Peer, MemberRole::Validator],
            status: MemberStatus::Active,
            validator_public_key: Some(validator_key.verifying_key().to_bytes()),
        })
        .collect();
    let manifest = MembershipManifest {
        manifest_id: "issue9.membership".to_string(),
        version: 1,
        network_id: NETWORK_ID.to_string(),
        network_profile_id: PROFILE_ID.to_string(),
        members,
    }
    .sign(&governance_key)
    .expect("sign Issue #9 membership manifest");
    let mut role_keys = BTreeSet::from([governance_key.verifying_key().to_bytes()]);
    for (identity_key, validator_key) in identity_keys.iter().zip(&validator_keys) {
        role_keys.insert(identity_key.verifying_key().to_bytes());
        role_keys.insert(validator_key.verifying_key().to_bytes());
    }
    let lifecycle = PrivacyLifecycleProfile::new([1; 32], bootstrap_key.verifying_key().to_bytes())
        .expect("construct lifecycle declaration")
        .with_network_role_public_keys(role_keys)
        .expect("bind all Network Profile roles");
    let package = semantic_package();
    let network_profile = NetworkProfile {
        profile_id: PROFILE_ID.to_string(),
        network_id: NETWORK_ID.to_string(),
        consensus: ConsensusProfile {
            consensus_type: "poa".to_string(),
            authority_keys: validator_keys
                .iter()
                .map(|key| hex::encode(key.verifying_key().to_bytes()))
                .collect(),
            block_interval: 10,
            max_block_size: 1_048_576,
        },
        semantic: SemanticProfile {
            ontology_package_id: package.package_id().to_string(),
            ontology_package_version: package.package_version().to_string(),
            ontology_package_hash: package.package_hash().to_string(),
            semantic_execution_profile_id: package.semantic_execution_profile_id().to_string(),
            validation_mode: "strict".to_string(),
        },
        membership: Some(manifest.binding()),
        privacy: Some(lifecycle),
        bridge: None,
        bridge_source_trust: None,
    }
    .with_derived_privacy_content_hash()
    .expect("derive canonical Network Profile hash");
    let conformance = network_profile
        .privacy_lifecycle_conformance_slice(&manifest, &governance_key.verifying_key())
        .expect("verify Issue #9 conformance context")
        .expect("privacy lifecycle declared");
    Issue9ConformanceContext {
        profile: network_profile
            .privacy
            .clone()
            .expect("bound lifecycle declaration"),
        capability: conformance,
        manifest,
        node_ids,
        identity_keys,
    }
}

fn submit_privacy(
    ledger: &mut Ledger,
    transition: &PrivacyControlTransition,
    timestamp: u64,
    proposer: &SigningKey,
) -> Vec<u8> {
    let candidate = ledger
        .create_privacy_candidate(transition, timestamp, proposer)
        .expect("create exact privacy candidate");
    match ledger
        .local_adapter()
        .submit(candidate)
        .expect("run Final Admission")
    {
        AdmissionOutcome::Committed { envelope } => {
            assert_eq!(envelope.admission_kind, AdmissionKind::PrivacyControlV1);
            envelope
                .canonical_bytes()
                .expect("canonical committed envelope")
        }
        AdmissionOutcome::Rejected { reason } => panic!("privacy candidate rejected: {reason}"),
    }
}

fn replicate_latest_privacy_envelope(producer: &Ledger, followers: &mut [Ledger]) -> Vec<u8> {
    let exact_envelope = producer
        .committed_envelopes()
        .last()
        .expect("producer committed envelope")
        .canonical_bytes()
        .expect("canonical producer envelope");
    for follower in followers {
        assert!(matches!(
            follower
                .conformance_admit_exact_envelope(&exact_envelope)
                .expect("follower Final-Admits exact producer bytes"),
            AdmissionOutcome::Committed { .. }
        ));
    }
    exact_envelope
}

fn convergence_evidence(
    producer: &Ledger,
    followers: &[Ledger],
    context: &Issue9ConformanceContext,
) -> NetworkConvergenceEvidence {
    assert_eq!(followers.len(), 2);
    let ledgers = [producer, &followers[0], &followers[1]];
    let receipts: Vec<_> = ledgers
        .iter()
        .enumerate()
        .map(|(index, ledger)| {
            ledger
                .conformance_commit_receipt(
                    context.node_ids[index],
                    &context.manifest,
                    &context.identity_keys[index],
                )
                .expect("issue exact durable conformance receipt")
        })
        .collect();
    let privacy_receipts: Vec<_> = ledgers
        .iter()
        .enumerate()
        .map(|(index, ledger)| {
            ledger
                .privacy_state_receipt(
                    context.node_ids[index],
                    &context.manifest,
                    &context.identity_keys[index],
                )
                .expect("issue exact privacy-state convergence receipt")
        })
        .collect();
    producer
        .privacy_network_convergence_evidence(
            context.node_ids[0],
            &context.manifest,
            &context.node_ids,
            &receipts,
            &privacy_receipts,
        )
        .expect("three matching receipts establish the current exact prefix")
}

#[allow(clippy::too_many_arguments)]
fn deterministic_grant_envelope(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    wrapping_reference: &ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
    wrapping_key: &P256SigningKey,
    dek: [u8; 32],
    hpke_entropy: [u8; 32],
) -> GrantDekEnvelope {
    let position = anchor.expected_ledger_position().to_be_bytes();
    let revision = anchor.expected_privacy_revision().to_be_bytes();
    let profile_hash = anchor.profile_content_hash();
    let parent_prefix = anchor.parent_ledger_prefix_hash();
    let parent = anchor
        .parent_envelope_hash()
        .map(parent_reference)
        .unwrap_or([0_u8; 33]);
    let wrapping_reference_bytes = wrapping_reference.canonical_bytes();
    let permission = [PrivacyPermission::ReadProtectedObjectV1 as u8];
    let envelope_tag = [0x26];
    let suite = PROTECTED_DATA_SUITE_V1.as_bytes();
    let header_bytes = encode_record(
        0x25,
        &[
            suite,
            anchor.network_id().as_bytes(),
            anchor.profile_id().as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &parent_prefix,
            &parent,
            object_id.as_bytes(),
            owner.as_bytes(),
            grantee.as_bytes(),
            &permission,
            &wrapping_reference_bytes,
            &protected_content_commitment,
            &encrypted_payload_commitment,
        ],
    );
    let delivery_context_bytes = encode_record(
        0x24,
        &[
            b"PrivacyControlV1",
            b"CanonicalPrivacyEncodingV1",
            &[0x05],
            anchor.network_id().as_bytes(),
            anchor.profile_id().as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &parent_prefix,
            &parent,
            object_id.as_bytes(),
            owner.as_bytes(),
            grantee.as_bytes(),
            &permission,
            suite,
            &protected_content_commitment,
            &encrypted_payload_commitment,
            &wrapping_reference_bytes,
            &envelope_tag,
            suite,
        ],
    );
    let grant_delivery_context = domain_hash(
        "provchain/privacy-grant/delivery-context/v1",
        &delivery_context_bytes,
    );
    let header_hash = domain_hash(
        "provchain/protected-data/hpke-grant-header/v1",
        &header_bytes,
    );
    let mut info = [0_u8; 64];
    info[..32].copy_from_slice(&grant_delivery_context);
    info[32..].copy_from_slice(&header_hash);

    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_public = <Kem as KemTrait>::PublicKey::from_bytes(&p256_public(wrapping_key))
        .expect("deterministic grant HPKE recipient key");
    let mut rng = FixedHpkeRng::new(hpke_entropy);
    let (encapsulated, mut sender) = setup_sender_with_rng::<HpkeAead, HpkeKdf, Kem>(
        &OpModeS::Base,
        &recipient_public,
        &info,
        &mut rng,
    )
    .expect("deterministic grant HPKE sender context");
    assert!(rng.exactly_consumed());
    let wrapped_dek = sender.seal(&dek, b"").expect("wrap grant DEK");
    let encapsulated: [u8; 65] = encapsulated
        .to_bytes()
        .as_slice()
        .try_into()
        .expect("canonical grant HPKE encapsulated key");
    let wrapped_dek: [u8; 48] = wrapped_dek.try_into().expect("canonical grant wrapped DEK");
    let envelope_bytes = encode_record(
        0x26,
        &[
            suite,
            &grant_delivery_context,
            &header_bytes,
            &encapsulated,
            &wrapped_dek,
        ],
    );
    GrantDekEnvelope::decode(&envelope_bytes).expect("canonical grant DEK envelope")
}

struct RegisteredFixture {
    profile: PrivacyLifecycleProfile,
    state: EffectivePrivacyState,
    principal: Uuid,
    authorization_key: SigningKey,
    authorization_reference: ParticipantKeyReference,
    wrapping_reference: ParticipantKeyReference,
}

fn registered_fixture() -> RegisteredFixture {
    let bootstrap_key = seeded_ed25519(81);
    let authorization_key = seeded_ed25519(82);
    let wrapping_key = seeded_p256(83);
    let profile_hash = [0x91; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("construct lifecycle profile");
    let principal = Uuid::from_u128(0x91_0001);

    let registration_anchor =
        PrivacyAdmissionAnchor::genesis(NETWORK_ID, PROFILE_ID, profile_hash, 0, 1)
            .expect("registration anchor");
    let unsigned_registration = UnsignedPrivacyTransition::register_principal(
        &profile,
        registration_anchor.clone(),
        principal,
        authorization_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let registration = unsigned_registration
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&unsigned_registration.authorization_digest())
                .to_bytes(),
            authorization_key
                .sign(
                    &unsigned_registration
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("registration transition");
    let state = EffectivePrivacyState::new()
        .stage_transition(&profile, &registration_anchor, &registration)
        .expect("register principal");
    let authorization_reference = registration
        .introduced_key_reference()
        .expect("authorization reference");

    let binding_anchor = PrivacyAdmissionAnchor::new(
        NETWORK_ID,
        PROFILE_ID,
        profile_hash,
        1,
        2,
        [0x11; 32],
        Some([0x12; 32]),
    )
    .expect("binding anchor");
    let unsigned_binding = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        binding_anchor.clone(),
        principal,
        1,
        p256_public(&wrapping_key),
        authorization_reference.clone(),
    )
    .expect("wrapping binding core");
    let possession_digest = unsigned_binding
        .possession_digest()
        .expect("wrapping possession digest");
    let binding = unsigned_binding
        .clone()
        .complete_binding(
            authorization_key
                .sign(&unsigned_binding.authorization_digest())
                .to_bytes(),
            p256_proof(&wrapping_key, &possession_digest),
        )
        .expect("wrapping binding transition");
    let wrapping_reference = binding
        .introduced_key_reference()
        .expect("wrapping reference");
    let state = state
        .stage_transition(&profile, &binding_anchor, &binding)
        .expect("bind wrapping key");

    RegisteredFixture {
        profile,
        state,
        principal,
        authorization_key,
        authorization_reference,
        wrapping_reference,
    }
}

#[test]
fn public_final_admission_accepts_exact_ciphertext_without_any_private_key() {
    let fixture = registered_fixture();
    let profile_hash = fixture.profile.network_profile_content_hash();
    let object_id = Uuid::from_bytes([0x91, 0, 0, 0, 0, 0, 0x40, 1, 0x80, 0, 0, 0, 0, 0, 0, 1]);
    let anchor = PrivacyAdmissionAnchor::new(
        NETWORK_ID,
        PROFILE_ID,
        profile_hash,
        2,
        3,
        [0x21; 32],
        Some([0x22; 32]),
    )
    .expect("object anchor");
    let parent = parent_reference([0x22; 32]);
    let position = 2_u64.to_be_bytes();
    let revision = 3_u64.to_be_bytes();
    let payload_tag = [0x21];
    let envelope_tag = [0x23];
    let pcc = [0x31; 32];
    let context = encode_record(
        0x20,
        &[
            b"PrivacyControlV1",
            b"CanonicalPrivacyEncodingV1",
            &[0x04],
            NETWORK_ID.as_bytes(),
            PROFILE_ID.as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &[0x21; 32],
            &parent,
            object_id.as_bytes(),
            fixture.principal.as_bytes(),
            &fixture.wrapping_reference.canonical_bytes(),
            &payload_tag,
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &envelope_tag,
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &pcc,
        ],
    );
    let object_context = domain_hash("provchain/protected-object/encryption-context/v1", &context);
    let ciphertext_and_tag = [0x41; 17];
    let payload_bytes = encode_record(
        0x21,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &[0; 12],
            &pcc,
            &ciphertext_and_tag,
        ],
    );
    let encrypted_payload_commitment = domain_hash(
        "provchain/protected-data/encrypted-payload-commitment/v1",
        &payload_bytes,
    );
    let owner_header = encode_record(
        0x22,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            NETWORK_ID.as_bytes(),
            PROFILE_ID.as_bytes(),
            &profile_hash,
            &position,
            &revision,
            &[0x21; 32],
            &parent,
            object_id.as_bytes(),
            fixture.principal.as_bytes(),
            &fixture.wrapping_reference.canonical_bytes(),
            &pcc,
            &encrypted_payload_commitment,
        ],
    );
    let hpke_enc = p256_public(&seeded_p256(84));
    let wrapped_dek = [0x51; 48];
    let envelope_bytes = encode_record(
        0x23,
        &[
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &object_context,
            &owner_header,
            &hpke_enc,
            &wrapped_dek,
        ],
    );

    let unsigned = UnsignedPrivacyTransition::create_protected_object(
        &fixture.profile,
        anchor.clone(),
        object_id,
        fixture.principal,
        fixture.authorization_reference,
        fixture.wrapping_reference,
        object_context,
        ProtectedPayloadCiphertext::decode(&payload_bytes).expect("canonical payload"),
        encrypted_payload_commitment,
        OwnerDekEnvelope::decode(&envelope_bytes).expect("canonical owner envelope"),
    )
    .expect("construct public object transition");
    let transition = unsigned
        .clone()
        .complete_object_creation(
            fixture
                .authorization_key
                .sign(&unsigned.authorization_digest())
                .to_bytes(),
        )
        .expect("complete owner-authorized object transition");
    let complete = transition.canonical_bytes();
    assert_eq!(complete[4], 0x43, "closed CreateProtectedObject tag");
    assert_eq!(
        PrivacyControlTransition::decode(&complete).expect("decode exact public transition"),
        transition
    );

    let child = fixture
        .state
        .stage_transition(&fixture.profile, &anchor, &transition)
        .expect("keyless public admission");
    let object = child.protected_object(object_id).expect("committed object");
    assert_eq!(object.payload().canonical_bytes(), payload_bytes);
    assert_eq!(object.owner_envelope().canonical_bytes(), envelope_bytes);
    assert_eq!(
        object.encrypted_payload_commitment(),
        encrypted_payload_commitment
    );
    assert_eq!(child.revision(), 3);
}

#[test]
fn independent_protected_object_vectors_self_open_and_reject_context_tampering() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("vectors/protected_object_v1.json"))
            .expect("checked-in protected-object vectors");
    let hex_field = |section: &str, field: &str| {
        hex::decode(
            vectors[section][field]
                .as_str()
                .unwrap_or_else(|| panic!("missing vector field {section}.{field}")),
        )
        .unwrap_or_else(|_| panic!("invalid vector hex {section}.{field}"))
    };
    let network_id = vectors["inputs"]["network_id"]
        .as_str()
        .expect("vector network id");
    let profile_id = vectors["inputs"]["profile_id"]
        .as_str()
        .expect("vector profile id");
    let profile_hash: [u8; 32] = hex_field("inputs", "profile_content_hash_hex")
        .try_into()
        .expect("profile hash");
    let bootstrap = SigningKey::from_bytes(
        &hex_field("inputs", "bootstrap_seed_hex")
            .try_into()
            .expect("bootstrap seed"),
    );
    let authorization = SigningKey::from_bytes(
        &hex_field("inputs", "authorization_seed_hex")
            .try_into()
            .expect("authorization seed"),
    );
    let wrapping = P256SigningKey::from_slice(&hex_field("inputs", "wrapping_private_scalar_hex"))
        .expect("wrapping scalar");
    let principal = Uuid::parse_str(
        vectors["inputs"]["principal_uuid"]
            .as_str()
            .expect("principal UUID"),
    )
    .expect("valid principal UUID");
    let profile = PrivacyLifecycleProfile::new(profile_hash, bootstrap.verifying_key().to_bytes())
        .expect("vector lifecycle profile");
    let registration_anchor =
        PrivacyAdmissionAnchor::genesis(network_id, profile_id, profile_hash, 0, 1)
            .expect("registration anchor");
    let register_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        registration_anchor.clone(),
        principal,
        authorization.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let registration = register_unsigned
        .clone()
        .complete_registration(
            bootstrap
                .sign(&register_unsigned.authorization_digest())
                .to_bytes(),
            authorization
                .sign(
                    &register_unsigned
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("registration transition");
    let state = EffectivePrivacyState::new()
        .stage_transition(&profile, &registration_anchor, &registration)
        .expect("registered vector principal");
    let authorization_reference = registration
        .introduced_key_reference()
        .expect("authorization reference");
    let binding_anchor = PrivacyAdmissionAnchor::new(
        network_id,
        profile_id,
        profile_hash,
        1,
        2,
        [0xB1; 32],
        Some([0xC1; 32]),
    )
    .expect("binding anchor");
    let bind_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        binding_anchor.clone(),
        principal,
        1,
        p256_public(&wrapping),
        authorization_reference,
    )
    .expect("wrapping binding core");
    let binding = bind_unsigned
        .clone()
        .complete_binding(
            authorization
                .sign(&bind_unsigned.authorization_digest())
                .to_bytes(),
            p256_proof(
                &wrapping,
                &bind_unsigned
                    .possession_digest()
                    .expect("wrapping possession digest"),
            ),
        )
        .expect("wrapping binding");
    let state = state
        .stage_transition(&profile, &binding_anchor, &binding)
        .expect("bound vector wrapping key");
    let object_anchor = PrivacyAdmissionAnchor::new(
        network_id,
        profile_id,
        profile_hash,
        2,
        3,
        [0xB2; 32],
        Some([0xC2; 32]),
    )
    .expect("object anchor");
    let complete = hex_field("create_protected_object", "complete_hex");
    for secret in [
        hex_field("inputs", "dek_fixture_hex"),
        hex_field("inputs", "authorization_seed_hex"),
        hex_field("inputs", "wrapping_private_scalar_hex"),
    ] {
        assert!(
            !complete
                .windows(secret.len())
                .any(|window| window == secret.as_slice()),
            "public transition must not carry a raw DEK or participant private key"
        );
    }
    let transition = PrivacyControlTransition::decode(&complete)
        .expect("independent complete CreateProtectedObject vector");
    assert_eq!(transition.canonical_bytes(), complete);
    let vector_transition_id: [u8; 32] = hex_field("create_protected_object", "transition_id_hex")
        .try_into()
        .expect("transition identifier");
    assert_eq!(transition.transition_id(), vector_transition_id);
    let child = state
        .stage_transition(&profile, &object_anchor, &transition)
        .expect("walletless admission of independent vector");
    let object_id = transition
        .protected_object_id()
        .expect("created vector object");
    let object = child
        .protected_object(object_id)
        .expect("vector object state");
    assert_eq!(
        object.payload().canonical_bytes(),
        hex_field("create_protected_object", "payload_hex")
    );

    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_private = <Kem as KemTrait>::PrivateKey::from_bytes(&hex_field(
        "inputs",
        "wrapping_private_scalar_hex",
    ))
    .expect("vector HPKE recipient private key");
    let encapsulated = <Kem as KemTrait>::EncappedKey::from_bytes(&hex_field(
        "create_protected_object",
        "encapsulated_key_hex",
    ))
    .expect("vector HPKE encapsulated point");
    let mut receiver = setup_receiver::<HpkeAead, HpkeKdf, Kem>(
        &OpModeR::Base,
        &recipient_private,
        &encapsulated,
        &hex_field("create_protected_object", "owner_info_hex"),
    )
    .expect("vector HPKE receiver context");
    let dek = receiver
        .open(
            &hex_field("create_protected_object", "wrapped_dek_hex"),
            b"",
        )
        .expect("open vector owner envelope");
    assert_eq!(dek, hex_field("inputs", "dek_fixture_hex"));
    let extract_salt: [u8; 32] =
        Sha256V11::digest(b"provchain/protected-data/dek-extract-salt/v1").into();
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(&extract_salt), &dek);
    let payload_info = encode_record(
        0x03,
        &[
            b"provchain/protected-data/payload-key/v1",
            PROTECTED_DATA_SUITE_V1.as_bytes(),
            &hex_field("create_protected_object", "object_context_hex"),
        ],
    );
    let mut payload_key = [0_u8; 32];
    hkdf.expand(&payload_info, &mut payload_key)
        .expect("derive vector payload key");
    let plaintext = ChaCha20Poly1305::new_from_slice(&payload_key)
        .expect("payload key")
        .decrypt(
            &[0_u8; 12].into(),
            Payload {
                msg: object.payload().ciphertext_and_tag(),
                aad: &hex_field("create_protected_object", "object_context_hex"),
            },
        )
        .expect("open vector payload");
    assert_eq!(plaintext, hex_field("inputs", "protected_content_hex"));
    let pcc_info = encode_record(
        0x02,
        &[
            b"provchain/protected-data/pcc-key/v1",
            PROTECTED_DATA_SUITE_V1.as_bytes(),
        ],
    );
    let mut pcc_key = [0_u8; 32];
    hkdf.expand(&pcc_info, &mut pcc_key)
        .expect("derive vector PCC key");
    let mut pcc_mac =
        <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(&pcc_key).expect("PCC HMAC key");
    pcc_mac.update(&encode_record(
        0x04,
        &[b"provchain/protected-data/pcc/v1", plaintext.as_slice()],
    ));
    assert_eq!(
        pcc_mac.finalize().into_bytes().as_slice(),
        hex_field("create_protected_object", "pcc_hex")
    );

    for field in [
        "tampered_context_complete_hex",
        "invalid_encapsulated_point_complete_hex",
        "truncated_complete_hex",
    ] {
        assert!(
            PrivacyControlTransition::decode(&hex_field("negative", field)).is_err(),
            "negative vector {field} must fail closed"
        );
    }
    let mut bad_signature = transition.canonical_bytes();
    *bad_signature.last_mut().expect("signature byte") ^= 1;
    let bad_signature = PrivacyControlTransition::decode(&bad_signature)
        .expect("signature corruption remains structurally canonical");
    assert!(state
        .stage_transition(&profile, &object_anchor, &bad_signature)
        .is_err());
    assert_eq!(
        state.revision(),
        2,
        "rejected candidate does not mutate parent"
    );
}

#[test]
fn exact_replication_artifacts_exclude_controlled_client_secrets() {
    let root = tempfile::tempdir().expect("temporary exact-replication root");
    let bootstrap_seed = [0xc1; 32];
    let authorization_seed = [0xc2; 32];
    let wrapping_scalar = [0xc3; 32];
    let dek = [0xc4; 32];
    let hpke_entropy = [0xc5; 32];
    let plaintext = b"issue9 deterministic exact-replication plaintext\0\xff";
    let passphrase_bytes = b"issue9-deterministic-artifact-passphrase";
    let bootstrap_key = SigningKey::from_bytes(&bootstrap_seed);
    let authorization_key = SigningKey::from_bytes(&authorization_seed);
    let wrapping_key =
        P256SigningKey::from_slice(&wrapping_scalar).expect("controlled wrapping scalar");
    let proposer = seeded_ed25519(0xc6);
    let context = conformance_lifecycle_for_ledger(&bootstrap_key);
    let profile = context.profile.clone();
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID)
            .with_privacy_conformance_slice(context.capability.clone()),
    );
    let mut producer = Ledger::open_in_dir(
        root.path().join("producer"),
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open controlled producer");
    let mut followers: Vec<_> = ["follower-1", "follower-2"]
        .iter()
        .map(|name| {
            Ledger::open_in_dir(
                root.path().join(name),
                ledger_profile.clone(),
                semantic_package(),
            )
            .expect("open controlled follower")
        })
        .collect();
    let principal = Uuid::from_u128(0x91_0000_0000_4001_8000_0000_0000_0091);

    let registration_anchor = producer
        .next_privacy_admission_anchor()
        .expect("journal-derived registration anchor");
    let registration_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        registration_anchor,
        principal,
        authorization_key.verifying_key().to_bytes(),
    )
    .expect("controlled registration core");
    let registration = registration_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&registration_unsigned.authorization_digest())
                .to_bytes(),
            authorization_key
                .sign(
                    &registration_unsigned
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("controlled registration");
    submit_privacy(&mut producer, &registration, 1_700_000_009_101, &proposer);
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let authorization_reference = registration
        .introduced_key_reference()
        .expect("controlled authorization reference");

    let binding_anchor = producer
        .next_privacy_admission_anchor()
        .expect("journal-derived binding anchor");
    let binding_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        binding_anchor,
        principal,
        1,
        p256_public(&wrapping_key),
        authorization_reference.clone(),
    )
    .expect("controlled wrapping-key binding core");
    let binding = binding_unsigned
        .clone()
        .complete_binding(
            authorization_key
                .sign(&binding_unsigned.authorization_digest())
                .to_bytes(),
            p256_proof(
                &wrapping_key,
                &binding_unsigned
                    .possession_digest()
                    .expect("wrapping-key possession digest"),
            ),
        )
        .expect("controlled wrapping-key binding");
    submit_privacy(&mut producer, &binding, 1_700_000_009_102, &proposer);
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let wrapping_reference = binding
        .introduced_key_reference()
        .expect("controlled wrapping reference");

    let object_anchor = producer
        .next_privacy_admission_anchor()
        .expect("journal-derived object anchor");
    let object_id = Uuid::from_u128(0x91_0000_0000_4001_8000_0000_0000_0092);
    let object_transition = deterministic_protected_object(
        &profile,
        object_anchor.clone(),
        principal,
        authorization_reference.clone(),
        wrapping_reference.clone(),
        &authorization_key,
        &wrapping_key,
        object_id,
        dek,
        hpke_entropy,
        plaintext,
    );
    assert_eq!(
        deterministic_protected_object(
            &profile,
            object_anchor,
            principal,
            authorization_reference,
            wrapping_reference,
            &authorization_key,
            &wrapping_key,
            object_id,
            dek,
            hpke_entropy,
            plaintext,
        )
        .canonical_bytes(),
        object_transition.canonical_bytes(),
        "controlled entropy produces one exact object transition"
    );

    let mut invalid_transition = object_transition.canonical_bytes();
    *invalid_transition.last_mut().expect("owner signature byte") ^= 1;
    let invalid_transition = PrivacyControlTransition::decode(&invalid_transition)
        .expect("signature corruption remains structurally canonical");
    let rejection_output = producer
        .create_privacy_candidate(&invalid_transition, 1_700_000_009_103, &proposer)
        .expect_err("invalid proof must fail before an outer candidate is emitted")
        .to_string()
        .into_bytes();
    assert_eq!(producer.journal_frame_count().unwrap(), 2);

    let committed_object = submit_privacy(
        &mut producer,
        &object_transition,
        1_700_000_009_104,
        &proposer,
    );
    assert_eq!(
        replicate_latest_privacy_envelope(&producer, &mut followers),
        committed_object,
        "followers receive the producer's exact committed object envelope"
    );
    let evidence = convergence_evidence(&producer, &followers, &context);
    producer
        .network_converged_privacy_prefix(&evidence)
        .expect("three exact durable receipts cover the object prefix");

    let passphrase = CustodyPassphrase::new(passphrase_bytes).expect("controlled passphrase");
    let custody_ledger = Ledger::open_in_dir(
        root.path().join("custody-prefix-ledger"),
        ledger_profile,
        semantic_package(),
    )
    .expect("open independent empty custody prefix");
    let mut custody = ParticipantCustody::new(root.path().join("controlled-custody"));
    let prepared_request = custody
        .initialize(
            &custody_ledger
                .verified_privacy_prefix()
                .expect("verified empty custody prefix"),
            &passphrase,
        )
        .expect("durably seal controlled passphrase custody");

    let secrets: [&[u8]; 7] = [
        &bootstrap_seed,
        &authorization_seed,
        &wrapping_scalar,
        &dek,
        &hpke_entropy,
        plaintext,
        passphrase_bytes,
    ];
    let ledgers = [&producer, &followers[0], &followers[1]];
    for (index, ledger) in ledgers.iter().enumerate() {
        assert_eq!(
            ledger.committed_envelopes()[2]
                .canonical_bytes()
                .expect("canonical replicated object envelope"),
            committed_object,
            "node {index} retains exact producer bytes"
        );
        assert_bytes_exclude(
            &format!("node {index} journal"),
            &ledger.journal_bytes().expect("read verified journal bytes"),
            &secrets,
        );
        let object = ledger
            .effective_privacy_state()
            .expect("public privacy projection")
            .protected_object(object_id)
            .expect("projected controlled object");
        assert_bytes_exclude(
            &format!("node {index} protected-payload projection"),
            &object.payload().canonical_bytes(),
            &secrets,
        );
        assert_bytes_exclude(
            &format!("node {index} owner-envelope projection"),
            &object.owner_envelope().canonical_bytes(),
            &secrets,
        );
    }
    assert_bytes_exclude("public rejection output", &rejection_output, &secrets);
    assert_bytes_exclude(
        "prepared public custody response",
        prepared_request.canonical_bytes(),
        &secrets,
    );
    assert_bytes_exclude(
        "network-convergence evidence output",
        format!("{evidence:?}").as_bytes(),
        &secrets,
    );
    assert_file_artifacts_exclude(root.path(), &secrets);

    for (name, source) in [
        ("ledger", include_str!("../src/ledger.rs")),
        (
            "reference node",
            include_str!("../src/network/reference.rs"),
        ),
    ] {
        for forbidden in [
            "pub fn decrypt_protected_object",
            "pub fn open_protected_payload",
            "pub fn unwrap_protected_dek",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must expose no node-side protected-data decryption API"
            );
        }
    }
}

#[test]
fn generated_custody_is_durable_before_disclosure_and_constructs_real_objects() {
    let root = tempfile::tempdir().expect("temporary custody parent");
    let custody_path = root.path().join("participant-custody");
    let passphrase =
        CustodyPassphrase::new(b"issue9-test-passphrase").expect("valid test passphrase");
    let bootstrap_key = seeded_ed25519(91);
    let proposer = seeded_ed25519(92);
    let conformance_context = conformance_lifecycle_for_ledger(&bootstrap_key);
    let profile = conformance_context.profile.clone();
    let ledger_directory = root.path().join("ledger");
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID)
            .with_privacy_conformance_slice(conformance_context.capability.clone()),
    );
    let mut ledger = Ledger::open_in_dir(
        &ledger_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open walletless Issue #9 ledger");
    let follower_directories = [
        root.path().join("ledger-follower-1"),
        root.path().join("ledger-follower-2"),
    ];
    let mut follower_ledgers: Vec<_> = follower_directories
        .iter()
        .map(|directory| {
            Ledger::open_in_dir(directory, ledger_profile.clone(), semantic_package())
                .expect("open exact-envelope conformance follower")
        })
        .collect();
    let registration_anchor = ledger
        .next_privacy_admission_anchor()
        .expect("journal-derived registration anchor");
    let mut custody = ParticipantCustody::new(&custody_path);

    let governance_request = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified empty ledger prefix");
        custody
            .initialize(&prefix, &passphrase)
            .expect("commit prepared governance request before returning it")
    };
    let first_snapshot = std::fs::read(custody.live_path()).expect("committed encrypted snapshot");
    assert!(first_snapshot.starts_with(b"PKS1"));
    assert!(
        !first_snapshot
            .windows(governance_request.authorization_public_key().len())
            .any(|window| window == governance_request.authorization_public_key()),
        "public/private key material is encrypted inside the custody snapshot"
    );
    let wrong_passphrase =
        CustodyPassphrase::new(b"wrong-issue9-passphrase").expect("valid alternate passphrase");
    assert_eq!(
        custody_error({
            let prefix = ledger
                .verified_privacy_prefix()
                .expect("verified empty ledger prefix");
            ParticipantCustody::new(&custody_path)
                .pending_governance_request(&prefix, &wrong_passphrase)
        }),
        CustodyError::KeystoreOpenFailed,
        "wrong passphrase and authenticated-open failures share one external result"
    );

    let governance_signature = bootstrap_key
        .sign(&governance_request.authorization_digest())
        .to_bytes();
    let registration_submission = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified empty ledger prefix");
        custody
            .accept_governance_signature(&prefix, &passphrase, governance_signature)
            .expect("commit prepared node submission before returning it")
    };
    let exact_retry = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified empty ledger prefix");
        ParticipantCustody::new(&custody_path)
            .pending_submission(&prefix, &passphrase)
            .expect("reopen committed custody")
            .expect("recover exact prepared registration")
    };
    assert_eq!(
        exact_retry.canonical_bytes(),
        registration_submission.canonical_bytes(),
        "restart returns exact stored submission bytes"
    );
    submit_privacy(
        &mut ledger,
        registration_submission.transition(),
        1_700_000_009_001,
        &proposer,
    );
    replicate_latest_privacy_envelope(&ledger, &mut follower_ledgers);
    let registration_evidence =
        convergence_evidence(&ledger, &follower_ledgers, &conformance_context);
    let registration_report = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified registration prefix");
        custody
            .reconcile(&prefix, &passphrase)
            .expect("commit registration as Bound custody")
    };
    assert!(registration_report.gaps().is_empty());
    let backup_directory = root.path().join("backups");
    let live_before_backup = std::fs::read(custody.live_path()).expect("live bytes before backup");
    let stale_backup = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified registration prefix");
        custody
            .create_backup(&backup_directory, &prefix, &passphrase)
            .expect("create exact encrypted snapshot backup")
    };
    assert_eq!(
        std::fs::read(custody.live_path()).expect("live bytes after backup"),
        live_before_backup,
        "backup creation is read-only with respect to live custody"
    );
    assert_eq!(
        std::fs::read(stale_backup.path()).expect("backup bytes"),
        std::fs::read(custody.live_path()).expect("live bytes"),
        "backup is byte-identical rather than newly sealed"
    );

    let principal = governance_request.principal();
    let wrapping_submission = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified wrapping-key prefix");
        custody
            .prepare_wrapping_key(&prefix, &passphrase)
            .expect("generate and durably prepare wrapping key")
    };
    let prepared_backup_directory = root.path().join("prepared-backups");
    let prepared_backup = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified pre-binding prefix");
        custody
            .create_backup(&prepared_backup_directory, &prefix, &passphrase)
            .expect("back up exact still-current prepared wrapping transition")
    };
    let prepared_restore_path = root.path().join("restored-prepared");
    let mut prepared_restore = ParticipantCustody::new(&prepared_restore_path);
    {
        let prefix = ledger
            .network_converged_privacy_prefix(&registration_evidence)
            .expect("Network-Converged pre-binding prefix");
        prepared_restore
            .restore_backup(
                prepared_backup.path(),
                prepared_backup.reference(),
                &prefix,
                &passphrase,
            )
            .expect("restore still-current prepared wrapping transition");
        let restored_submission = prepared_restore
            .pending_submission(&prefix, &passphrase)
            .expect("restored custody is immediately usable")
            .expect("prepared wrapping transition survives restore");
        assert_eq!(
            restored_submission.canonical_bytes(),
            wrapping_submission.canonical_bytes(),
            "restore retains exact non-registration PreparedNode state"
        );
    }
    let stale_parent_ledger_directory = root.path().join("stale-parent-ledger");
    let mut stale_parent_ledger = Ledger::open_in_dir(
        &stale_parent_ledger_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open independent exact-parent ledger");
    submit_privacy(
        &mut stale_parent_ledger,
        registration_submission.transition(),
        1_700_000_009_001,
        &proposer,
    );
    let ordinary = stale_parent_ledger
        .create_ordinary_candidate(
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier "issue9-stale-parent" ."#,
            1_700_000_009_001_500,
            &proposer,
        )
        .expect("construct intervening ordinary block");
    assert!(matches!(
        stale_parent_ledger
            .local_adapter()
            .submit(ordinary)
            .expect("commit intervening ordinary block"),
        AdmissionOutcome::Committed { .. }
    ));
    assert_eq!(
        custody_error({
            let prefix = stale_parent_ledger
                .verified_privacy_prefix()
                .expect("verified prefix after intervening ordinary block");
            prepared_restore.pending_submission(&prefix, &passphrase)
        }),
        CustodyError::LedgerConflict,
        "prepared bytes are unusable as soon as the exact ledger parent changes"
    );
    submit_privacy(
        &mut ledger,
        wrapping_submission.transition(),
        1_700_000_009_002,
        &proposer,
    );
    replicate_latest_privacy_envelope(&ledger, &mut follower_ledgers);
    let wrapping_evidence = convergence_evidence(&ledger, &follower_ledgers, &conformance_context);
    assert!(
        ledger
            .network_converged_privacy_prefix(&registration_evidence)
            .is_err(),
        "historical convergence evidence cannot authorize current restore"
    );
    let state = ledger
        .effective_privacy_state()
        .expect("journal-derived wrapping-key state")
        .clone();
    {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified wrapping-key state");
        custody
            .reconcile(&prefix, &passphrase)
            .expect("commit wrapping key as Bound custody");
    }

    let stale_report = {
        let prefix = ledger
            .network_converged_privacy_prefix(&wrapping_evidence)
            .expect("Network-Converged wrapping-key state");
        ParticipantCustody::verify_backup(
            stale_backup.path(),
            stale_backup.reference(),
            &prefix,
            &passphrase,
        )
        .expect("isolated verification authenticates and reconciles the old backup")
    };
    assert_eq!(
        stale_report.gaps(),
        std::slice::from_ref(
            state
                .active_key(
                    principal,
                    provchain_org::privacy::ParticipantKeyPurpose::PrivacyKeyWrapping,
                )
                .expect("current wrapping key")
        ),
        "a stale authentic backup reports the ledger-recorded missing key"
    );

    let restored_path = root.path().join("restored-stale");
    let mut restored = ParticipantCustody::new(&restored_path);
    let restore_report = {
        let prefix = ledger
            .network_converged_privacy_prefix(&wrapping_evidence)
            .expect("Network-Converged wrapping-key state");
        restored
            .restore_backup(
                stale_backup.path(),
                stale_backup.reference(),
                &prefix,
                &passphrase,
            )
            .expect("restore authentic backup through quarantine")
    };
    assert_eq!(restore_report.gaps(), stale_report.gaps());
    assert_ne!(
        std::fs::read(restored.live_path()).expect("restored successor"),
        std::fs::read(stale_backup.path()).expect("raw backup"),
        "restore installs a freshly sealed successor, never raw backup bytes"
    );
    let restored_backup_directory = root.path().join("restored-backups");
    {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified wrapping-key state");
        restored
            .create_backup(&restored_backup_directory, &prefix, &passphrase)
            .expect("successful restore leaves custody ready for normal operations");
    }
    assert_eq!(
        custody_error({
            let prefix = ledger
                .network_converged_privacy_prefix(&wrapping_evidence)
                .expect("Network-Converged wrapping-key state");
            restored.restore_backup(
                stale_backup.path(),
                stale_backup.reference(),
                &prefix,
                &passphrase,
            )
        }),
        CustodyError::RecoveryConflict,
        "restore never overwrites an existing live custody store"
    );

    let expected = CustodySnapshotReference::new(
        stale_backup.reference().file_id(),
        stale_backup.reference().generation(),
        stale_backup.reference().digest(),
        "wrong.network",
        stale_backup.reference().principal(),
    )
    .expect("syntactically valid wrong-network reference");
    assert_eq!(
        custody_error({
            let prefix = ledger
                .network_converged_privacy_prefix(&wrapping_evidence)
                .expect("Network-Converged wrapping-key state");
            ParticipantCustody::verify_backup(stale_backup.path(), &expected, &prefix, &passphrase)
        }),
        CustodyError::KeystoreOpenFailed,
        "expected identity mismatch has the same closed open failure"
    );

    let backup_bytes = std::fs::read(stale_backup.path()).expect("read controlled backup bytes");
    let temporary_restore_path = root.path().join("temporary-restore-only");
    make_private_directory(&temporary_restore_path);
    let temporary_quarantine = temporary_restore_path.join("restore-quarantine");
    make_private_directory(&temporary_quarantine);
    let temporary_source = temporary_quarantine
        .join(".participant-keystore-restore-source.tmp.11111111111111111111111111111111");
    make_private_file(&temporary_source, &backup_bytes);
    assert_eq!(
        {
            let prefix = ledger
                .verified_privacy_prefix()
                .expect("verified wrapping-key state");
            ParticipantCustody::new(&temporary_restore_path)
                .recover(&prefix, &passphrase)
                .expect("temporary quarantine copy is durably discarded")
        },
        CustodyRecovery::AbsentReady,
        "a temporary quarantine copy is not durable restore intent"
    );
    assert!(!temporary_source.exists());

    let ambiguous_restore_path = root.path().join("ambiguous-restore");
    make_private_directory(&ambiguous_restore_path);
    let ambiguous_quarantine = ambiguous_restore_path.join("restore-quarantine");
    make_private_directory(&ambiguous_quarantine);
    make_private_file(
        &ambiguous_quarantine.join("participant-keystore-restore-source.first.pks1"),
        &backup_bytes,
    );
    make_private_file(
        &ambiguous_quarantine.join("participant-keystore-restore-source.second.pks1"),
        &backup_bytes,
    );
    assert_eq!(
        custody_error({
            let prefix = ledger
                .network_converged_privacy_prefix(&wrapping_evidence)
                .expect("Network-Converged wrapping-key state");
            ParticipantCustody::new(&ambiguous_restore_path).recover_restore(&prefix, &passphrase)
        }),
        CustodyError::RecoveryConflict,
        "multiple canonical restore sources are never selected heuristically"
    );

    let corrupt_restore_path = root.path().join("corrupt-restore");
    make_private_directory(&corrupt_restore_path);
    let corrupt_quarantine = corrupt_restore_path.join("restore-quarantine");
    make_private_directory(&corrupt_quarantine);
    make_private_file(
        &corrupt_quarantine.join("participant-keystore-restore-source.corrupt.pks1"),
        b"not-a-participant-keystore",
    );
    assert_eq!(
        custody_error({
            let prefix = ledger
                .network_converged_privacy_prefix(&wrapping_evidence)
                .expect("Network-Converged wrapping-key state");
            ParticipantCustody::new(&corrupt_restore_path).recover_restore(&prefix, &passphrase)
        }),
        CustodyError::RecoveryConflict,
        "a corrupt durable restore source remains fail closed"
    );

    let restore_crash_path = root.path().join("restore-crash");
    let mut restore_crash = ParticipantCustody::new(&restore_crash_path);
    restore_crash.inject_next_commit_failpoint(CustodyCommitFailpoint::CrashAfterCandidateDurable);
    assert_eq!(
        custody_error({
            let prefix = ledger
                .network_converged_privacy_prefix(&wrapping_evidence)
                .expect("Network-Converged wrapping-key state");
            restore_crash.restore_backup(
                stale_backup.path(),
                stale_backup.reference(),
                &prefix,
                &passphrase,
            )
        }),
        CustodyError::CommitIndeterminate
    );
    let mut restore_restart = ParticipantCustody::new(&restore_crash_path);
    let CustodyRecovery::Ready(recovered) = ({
        let prefix = ledger
            .network_converged_privacy_prefix(&wrapping_evidence)
            .expect("Network-Converged wrapping-key state");
        restore_restart
            .recover_restore(&prefix, &passphrase)
            .expect("recovery authenticates the quarantine source and exact restore successor")
    }) else {
        panic!("restore successor must become ready")
    };
    assert_eq!(recovered.gaps(), stale_report.gaps());

    let alternate_authorization = seeded_ed25519(109);
    let alternate_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        registration_anchor.clone(),
        principal,
        alternate_authorization.verifying_key().to_bytes(),
    )
    .expect("conflicting fixture registration");
    let alternate_registration = alternate_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&alternate_unsigned.authorization_digest())
                .to_bytes(),
            alternate_authorization
                .sign(
                    &alternate_unsigned
                        .possession_digest()
                        .expect("possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete conflicting fixture registration");
    let conflicting_ledger_directory = root.path().join("conflicting-ledger");
    let mut conflicting_ledger = Ledger::open_in_dir(
        &conflicting_ledger_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open conflicting journal authority");
    submit_privacy(
        &mut conflicting_ledger,
        &alternate_registration,
        1_700_000_009_090,
        &proposer,
    );
    let mut conflicting_followers: Vec<_> = [
        root.path().join("conflicting-ledger-follower-1"),
        root.path().join("conflicting-ledger-follower-2"),
    ]
    .iter()
    .map(|directory| {
        Ledger::open_in_dir(directory, ledger_profile.clone(), semantic_package())
            .expect("open conflicting conformance follower")
    })
    .collect();
    replicate_latest_privacy_envelope(&conflicting_ledger, &mut conflicting_followers);
    let conflicting_evidence = convergence_evidence(
        &conflicting_ledger,
        &conflicting_followers,
        &conformance_context,
    );
    let empty_restore_path = root.path().join("restore-empty");
    let mut empty_restore = ParticipantCustody::new(&empty_restore_path);
    assert_eq!(
        custody_error({
            let prefix = conflicting_ledger
                .network_converged_privacy_prefix(&conflicting_evidence)
                .expect("Network-Converged conflicting ledger prefix");
            empty_restore.restore_backup(
                stale_backup.path(),
                stale_backup.reference(),
                &prefix,
                &passphrase,
            )
        }),
        CustodyError::RestoreNoInstallableState
    );
    assert!(!empty_restore.live_path().exists());
    assert_eq!(
        custody_error({
            let prefix = conflicting_ledger
                .verified_privacy_prefix()
                .expect("verified conflicting ledger prefix");
            empty_restore.initialize(&prefix, &passphrase)
        }),
        CustodyError::RecoveryConflict,
        "pending restore intent blocks unrelated generation-one creation"
    );
    assert_eq!(
        custody_error({
            let prefix = conflicting_ledger
                .network_converged_privacy_prefix(&conflicting_evidence)
                .expect("Network-Converged conflicting ledger prefix");
            ParticipantCustody::new(&empty_restore_path).recover(&prefix, &passphrase)
        }),
        CustodyError::NetworkConvergenceRequired,
        "ordinary recovery cannot install a durable restore source"
    );
    assert_eq!(
        custody_error({
            let prefix = conflicting_ledger
                .network_converged_privacy_prefix(&conflicting_evidence)
                .expect("Network-Converged conflicting ledger prefix");
            ParticipantCustody::new(&empty_restore_path).recover_restore(&prefix, &passphrase)
        }),
        CustodyError::RestoreNoInstallableState
    );

    assert_eq!(
        custody_error({
            let prefix = ledger
                .verified_privacy_prefix()
                .expect("verified object prefix");
            custody.create_protected_object(&prefix, &passphrase, b"")
        }),
        CustodyError::InvalidProtectedContent,
        "protected content must be nonempty"
    );
    assert_eq!(
        custody_error({
            let prefix = ledger
                .verified_privacy_prefix()
                .expect("verified object prefix");
            custody.create_protected_object(
                &prefix,
                &passphrase,
                &vec![0_u8; MAX_PROTECTED_CONTENT_BYTES + 1],
            )
        }),
        CustodyError::InvalidProtectedContent,
        "protected content must remain within the protocol bound"
    );
    let plaintext = b"participant-only protected content";
    let object_transition = {
        let prefix = ledger
            .verified_privacy_prefix()
            .expect("verified object prefix");
        custody
            .create_protected_object(&prefix, &passphrase, plaintext)
            .expect("construct, self-check, and authorize protected object")
    };
    assert!(
        !object_transition
            .canonical_bytes()
            .windows(plaintext.len())
            .any(|window| window == plaintext),
        "candidate carries no plaintext"
    );
    let object_id = object_transition
        .protected_object_id()
        .expect("created object identifier");
    let transition_bytes = object_transition.canonical_bytes();
    let valid_candidate = ledger
        .create_privacy_candidate(&object_transition, 1_700_000_009_003, &proposer)
        .expect("create object candidate");
    let before_rejection = ledger
        .effective_privacy_state()
        .expect("verified object parent")
        .digest();
    let mut invalid_candidate = valid_candidate.clone();
    *invalid_candidate
        .privacy_control
        .as_mut()
        .expect("privacy transition bytes")
        .last_mut()
        .expect("owner signature byte") ^= 1;
    let invalid_candidate = invalid_candidate
        .sign(&proposer)
        .expect("outer-sign the malformed public proof fixture");
    assert!(matches!(
        ledger
            .local_adapter()
            .submit(invalid_candidate)
            .expect("deterministic rejection outcome"),
        AdmissionOutcome::Rejected { .. }
    ));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 2);
    assert_eq!(
        ledger
            .effective_privacy_state()
            .expect("unchanged object parent")
            .digest(),
        before_rejection,
        "rejected candidate cannot mutate journal-authoritative state"
    );
    ledger.inject_next_commit_failpoint(LedgerCommitFailpoint::BeforeJournalWrite);
    assert!(
        ledger
            .local_adapter()
            .submit(valid_candidate.clone())
            .is_err(),
        "injected pre-commit journal failure is observable"
    );
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 2);
    assert_eq!(
        ledger
            .effective_privacy_state()
            .expect("unchanged pre-commit object parent")
            .digest(),
        before_rejection,
        "a pre-commit failure cannot publish the protected object"
    );

    let parent_envelopes: Vec<_> = ledger
        .committed_envelopes()
        .iter()
        .map(|envelope| {
            envelope
                .canonical_bytes()
                .expect("canonical parent envelope")
        })
        .collect();
    let fsync_failure_directory = root.path().join("object-before-fsync");
    let mut fsync_failure = Ledger::open_in_dir(
        &fsync_failure_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open object pre-fsync conformance ledger");
    for envelope in &parent_envelopes {
        fsync_failure
            .conformance_admit_exact_envelope(envelope)
            .expect("replicate object parent prefix");
    }
    fsync_failure.inject_next_commit_failpoint(LedgerCommitFailpoint::BeforeJournalFsync);
    assert!(fsync_failure
        .local_adapter()
        .submit(valid_candidate.clone())
        .is_err());
    drop(fsync_failure);
    let fsync_reopened = Ledger::open_in_dir(
        &fsync_failure_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("reopen rolled-back pre-fsync object journal");
    assert_eq!(fsync_reopened.journal_frame_count().unwrap(), 2);
    assert!(fsync_reopened
        .effective_privacy_state()
        .unwrap()
        .protected_object(object_id)
        .is_none());

    let projection_failure_directory = root.path().join("object-after-fsync-before-projection");
    let mut projection_failure = Ledger::open_in_dir(
        &projection_failure_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open object projection-failure conformance ledger");
    for envelope in &parent_envelopes {
        projection_failure
            .conformance_admit_exact_envelope(envelope)
            .expect("replicate object parent prefix");
    }
    projection_failure
        .inject_next_commit_failpoint(LedgerCommitFailpoint::AfterJournalCommitBeforeProjection);
    assert!(matches!(
        projection_failure
            .local_adapter()
            .submit(valid_candidate.clone()),
        Err(provchain_org::ledger::LedgerError::ProjectionFailed { .. })
    ));
    assert!(matches!(
        projection_failure.effective_privacy_state(),
        Err(provchain_org::ledger::LedgerError::Degraded)
    ));
    assert_eq!(projection_failure.journal_frame_count().unwrap(), 3);
    drop(projection_failure);
    let projection_replayed = Ledger::open_in_dir(
        &projection_failure_directory,
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("replay exact object after projection failure");
    assert_eq!(projection_replayed.journal_frame_count().unwrap(), 3);
    assert_eq!(
        projection_replayed.committed_envelopes()[2]
            .privacy_control
            .as_deref(),
        Some(transition_bytes.as_slice())
    );
    assert!(projection_replayed
        .effective_privacy_state()
        .unwrap()
        .protected_object(object_id)
        .is_some());

    ledger.inject_next_commit_failpoint(LedgerCommitFailpoint::AfterProjectionBeforeResponse);
    assert!(matches!(
        ledger.local_adapter().submit(valid_candidate.clone()),
        Err(provchain_org::ledger::LedgerError::ResponseLost { .. })
    ));
    assert_eq!(
        ledger.journal_frame_count().expect("journal frames"),
        3,
        "lost response occurs only after the object journal commit"
    );
    let committed_envelope = match ledger
        .synchronization_adapter()
        .submit(valid_candidate)
        .expect("exact committed-envelope retry after lost response")
    {
        AdmissionOutcome::Committed { envelope } => envelope
            .canonical_bytes()
            .expect("canonical committed object envelope"),
        AdmissionOutcome::Rejected { reason } => {
            panic!("committed object retry rejected: {reason}")
        }
    };
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 3);
    let child = ledger
        .effective_privacy_state()
        .expect("walletless public admission state");
    assert_eq!(
        child.protected_object(object_id).unwrap().owner(),
        principal
    );
    assert_eq!(
        ledger.committed_envelopes()[2].privacy_control.as_deref(),
        Some(transition_bytes.as_slice()),
        "journal authority retains exact ciphertext and envelope bytes"
    );
    let journal_bytes = std::fs::read(ledger.journal_path()).expect("read exact journal bytes");
    for secret in [plaintext.as_ref(), b"issue9-test-passphrase".as_ref()] {
        assert!(
            !journal_bytes
                .windows(secret.len())
                .any(|window| window == secret),
            "journal must not contain client-only secret bytes"
        );
    }
    assert_file_artifacts_exclude(
        root.path(),
        &[plaintext.as_ref(), b"issue9-test-passphrase".as_ref()],
    );
    let privacy_digest = child.digest();
    drop(ledger);

    let replayed = Ledger::open_in_dir(&ledger_directory, ledger_profile, semantic_package())
        .expect("verified journal replay without participant custody");
    assert_eq!(
        replayed
            .effective_privacy_state()
            .expect("replayed public privacy state")
            .digest(),
        privacy_digest
    );
    assert_eq!(
        replayed.committed_envelopes()[2]
            .canonical_bytes()
            .expect("replayed envelope bytes"),
        committed_envelope
    );
}

#[test]
fn custody_crash_boundaries_require_deterministic_recovery_before_disclosure() {
    let root = tempfile::tempdir().expect("temporary custody parent");
    let passphrase =
        CustodyPassphrase::new(b"issue9-crash-passphrase").expect("valid test passphrase");
    let bootstrap_key = seeded_ed25519(101);
    let conformance_context = conformance_lifecycle_for_ledger(&bootstrap_key);
    let ledger = Ledger::open_in_dir(
        root.path().join("ledger"),
        bind_profile(
            LedgerProfile::new(NETWORK_ID, PROFILE_ID)
                .with_privacy_conformance_slice(conformance_context.capability),
        ),
        semantic_package(),
    )
    .expect("open verified empty ledger");
    let prefix = ledger
        .verified_privacy_prefix()
        .expect("verified empty ledger prefix");

    let aborted_path = root.path().join("aborted");
    let mut aborted = ParticipantCustody::new(&aborted_path);
    aborted.inject_next_commit_failpoint(CustodyCommitFailpoint::BeforePublish);
    assert_eq!(
        custody_error(aborted.initialize(&prefix, &passphrase)),
        CustodyError::Aborted
    );
    assert!(!aborted.live_path().exists());
    assert_eq!(
        ParticipantCustody::new(&aborted_path)
            .recover(&prefix, &passphrase)
            .expect("clean abort recovers as absent"),
        CustodyRecovery::AbsentReady
    );

    for (name, failpoint, expected_error) in [
        (
            "candidate-durable",
            CustodyCommitFailpoint::CrashAfterCandidateDurable,
            CustodyError::CommitIndeterminate,
        ),
        (
            "published-indeterminate",
            CustodyCommitFailpoint::AfterPublishBeforeDirectorySync,
            CustodyError::CommitIndeterminate,
        ),
        (
            "committed-quarantined",
            CustodyCommitFailpoint::AfterCommitBeforeReopen,
            CustodyError::CommittedButQuarantined,
        ),
    ] {
        let custody_path = root.path().join(name);
        let mut custody = ParticipantCustody::new(&custody_path);
        custody.inject_next_commit_failpoint(failpoint);
        assert_eq!(
            custody_error(custody.initialize(&prefix, &passphrase)),
            expected_error
        );
        assert_eq!(
            custody_error(custody.pending_governance_request(&prefix, &passphrase)),
            CustodyError::CommitIndeterminate
        );

        let mut restarted = ParticipantCustody::new(&custody_path);
        assert!(matches!(
            restarted
                .recover(&prefix, &passphrase)
                .expect("startup authenticates and deterministically completes recovery"),
            CustodyRecovery::Ready(_)
        ));
        let first = restarted
            .pending_governance_request(&prefix, &passphrase)
            .expect("ready store")
            .expect("durable prepared governance request");
        let second = ParticipantCustody::new(&custody_path)
            .pending_governance_request(&prefix, &passphrase)
            .expect("restart opens exact committed snapshot")
            .expect("same prepared governance request");
        assert_eq!(first.canonical_bytes(), second.canonical_bytes());
        assert_eq!(first.principal(), second.principal());
    }
    assert_file_artifacts_exclude(root.path(), &[b"issue9-crash-passphrase"]);
}

#[test]
fn converged_grant_release_is_opaque_and_revocation_is_prefix_bound() {
    let root = tempfile::tempdir().expect("temporary Issue #10 convergence root");
    let bootstrap_key = seeded_ed25519(121);
    let context = conformance_lifecycle_for_ledger(&bootstrap_key);
    let profile = context.profile.clone();
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID)
            .with_privacy_conformance_slice(context.capability.clone()),
    );
    let proposer = seeded_ed25519(122);
    let mut producer = Ledger::open_in_dir(
        root.path().join("producer"),
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open Issue #10 producer");
    let mut followers: Vec<_> = ["follower-1", "follower-2"]
        .iter()
        .map(|name| {
            Ledger::open_in_dir(
                root.path().join(name),
                ledger_profile.clone(),
                semantic_package(),
            )
            .expect("open Issue #10 follower")
        })
        .collect();

    let owner = Uuid::from_u128(0x10_0001);
    let grantee = Uuid::from_u128(0x10_0002);
    let owner_authorization_seed = [0xd1; 32];
    let grantee_authorization_seed = [0xd2; 32];
    let owner_wrapping_scalar = [0xd3; 32];
    let grantee_wrapping_scalar = [0xd4; 32];
    let dek = [0xd5; 32];
    let object_hpke_entropy = [0xd6; 32];
    let grant_hpke_entropy = [0xd7; 32];
    let owner_authorization = SigningKey::from_bytes(&owner_authorization_seed);
    let grantee_authorization = SigningKey::from_bytes(&grantee_authorization_seed);
    let owner_wrapping = seeded_p256(owner_wrapping_scalar[0]);
    let grantee_wrapping = seeded_p256(grantee_wrapping_scalar[0]);

    let owner_registration_anchor = producer
        .next_privacy_admission_anchor()
        .expect("owner registration anchor");
    let owner_registration_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        owner_registration_anchor,
        owner,
        owner_authorization.verifying_key().to_bytes(),
    )
    .expect("owner registration core");
    let owner_registration = owner_registration_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&owner_registration_unsigned.authorization_digest())
                .to_bytes(),
            owner_authorization
                .sign(
                    &owner_registration_unsigned
                        .possession_digest()
                        .expect("owner registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("owner registration");
    submit_privacy(
        &mut producer,
        &owner_registration,
        1_700_000_010_001,
        &proposer,
    );
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let owner_authorization_reference = owner_registration
        .introduced_key_reference()
        .expect("owner authorization reference");

    let grantee_registration_anchor = producer
        .next_privacy_admission_anchor()
        .expect("grantee registration anchor");
    let grantee_registration_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        grantee_registration_anchor,
        grantee,
        grantee_authorization.verifying_key().to_bytes(),
    )
    .expect("grantee registration core");
    let grantee_registration = grantee_registration_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&grantee_registration_unsigned.authorization_digest())
                .to_bytes(),
            grantee_authorization
                .sign(
                    &grantee_registration_unsigned
                        .possession_digest()
                        .expect("grantee registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("grantee registration");
    submit_privacy(
        &mut producer,
        &grantee_registration,
        1_700_000_010_002,
        &proposer,
    );
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let grantee_authorization_reference = grantee_registration
        .introduced_key_reference()
        .expect("grantee authorization reference");

    let owner_binding_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        producer
            .next_privacy_admission_anchor()
            .expect("owner wrapping binding anchor"),
        owner,
        1,
        p256_public(&owner_wrapping),
        owner_authorization_reference.clone(),
    )
    .expect("owner wrapping binding core");
    let owner_binding = owner_binding_unsigned
        .clone()
        .complete_binding(
            owner_authorization
                .sign(&owner_binding_unsigned.authorization_digest())
                .to_bytes(),
            p256_proof(
                &owner_wrapping,
                &owner_binding_unsigned
                    .possession_digest()
                    .expect("owner wrapping possession digest"),
            ),
        )
        .expect("owner wrapping binding");
    submit_privacy(&mut producer, &owner_binding, 1_700_000_010_003, &proposer);
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let owner_wrapping_reference = owner_binding
        .introduced_key_reference()
        .expect("owner wrapping reference");

    let grantee_binding_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        producer
            .next_privacy_admission_anchor()
            .expect("grantee wrapping binding anchor"),
        grantee,
        1,
        p256_public(&grantee_wrapping),
        grantee_authorization_reference.clone(),
    )
    .expect("grantee wrapping binding core");
    let grantee_binding = grantee_binding_unsigned
        .clone()
        .complete_binding(
            grantee_authorization
                .sign(&grantee_binding_unsigned.authorization_digest())
                .to_bytes(),
            p256_proof(
                &grantee_wrapping,
                &grantee_binding_unsigned
                    .possession_digest()
                    .expect("grantee wrapping possession digest"),
            ),
        )
        .expect("grantee wrapping binding");
    submit_privacy(
        &mut producer,
        &grantee_binding,
        1_700_000_010_004,
        &proposer,
    );
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let grantee_wrapping_reference = grantee_binding
        .introduced_key_reference()
        .expect("grantee wrapping reference");

    let object_id = Uuid::from_bytes([
        0x10, 0, 0, 0, 0, 0, 0x40, 0x10, 0x80, 0, 0, 0, 0, 0, 0, 0x10,
    ]);
    let plaintext = b"issue10 converged grant plaintext";
    let object_transition = deterministic_protected_object(
        &profile,
        producer
            .next_privacy_admission_anchor()
            .expect("protected object anchor"),
        owner,
        owner_authorization_reference.clone(),
        owner_wrapping_reference.clone(),
        &owner_authorization,
        &owner_wrapping,
        object_id,
        dek,
        object_hpke_entropy,
        plaintext,
    );
    let committed_object = submit_privacy(
        &mut producer,
        &object_transition,
        1_700_000_010_005,
        &proposer,
    );
    assert_eq!(
        replicate_latest_privacy_envelope(&producer, &mut followers),
        committed_object
    );
    let (payload, object_commitment, owner_envelope, object_context) = {
        let object = producer
            .effective_privacy_state()
            .expect("object privacy state")
            .protected_object(object_id)
            .expect("committed protected object");
        (
            object.payload().clone(),
            object.encrypted_payload_commitment(),
            object.owner_envelope().clone(),
            object.object_context(),
        )
    };

    let grant_anchor = producer
        .next_privacy_admission_anchor()
        .expect("grant anchor");
    let grant_delivery = deterministic_grant_envelope(
        &grant_anchor,
        object_id,
        owner,
        grantee,
        &grantee_wrapping_reference,
        payload.protected_content_commitment(),
        object_commitment,
        &grantee_wrapping,
        dek,
        grant_hpke_entropy,
    );
    let grant_unsigned = UnsignedPrivacyTransition::grant_access(
        &profile,
        grant_anchor,
        object_id,
        owner,
        grantee,
        PrivacyPermission::ReadProtectedObjectV1,
        grant_delivery.clone(),
        owner_authorization_reference.clone(),
    )
    .expect("grant core");
    let grant = grant_unsigned
        .clone()
        .complete_grant_access(
            owner_authorization
                .sign(&grant_unsigned.authorization_digest())
                .to_bytes(),
        )
        .expect("grant transition");
    let committed_grant = submit_privacy(&mut producer, &grant, 1_700_000_010_006, &proposer);
    assert_eq!(
        replicate_latest_privacy_envelope(&producer, &mut followers),
        committed_grant
    );
    let grant_id = grant.privacy_grant_id().expect("grant identifier");
    let grant_evidence = convergence_evidence(&producer, &followers, &context);
    assert_eq!(
        grant_evidence.privacy_state_digest(),
        Some(
            producer
                .effective_privacy_state()
                .expect("grant privacy state")
                .digest()
        )
    );
    assert_eq!(grant_evidence.privacy_receipts().len(), 3);
    let incomplete_privacy_receipts = grant_evidence.privacy_receipts()[..2].to_vec();
    for failure_mode in ["degraded-node", "partition-missing-receipt"] {
        assert!(
            producer
                .privacy_network_convergence_evidence(
                    context.node_ids[0],
                    &context.manifest,
                    &context.node_ids,
                    grant_evidence.receipts(),
                    &incomplete_privacy_receipts,
                )
                .is_err(),
            "{failure_mode} cannot establish the privacy release barrier"
        );
    }
    let mut tampered_privacy_receipts = grant_evidence.privacy_receipts().to_vec();
    let mut tampered_privacy_receipt = tampered_privacy_receipts[0].canonical_bytes();
    *tampered_privacy_receipt
        .last_mut()
        .expect("privacy-state signature byte") ^= 1;
    tampered_privacy_receipts[0] = PrivacyStateReceipt::decode(&tampered_privacy_receipt)
        .expect("signature mutation remains structurally canonical");
    assert!(
        producer
            .privacy_network_convergence_evidence(
                context.node_ids[0],
                &context.manifest,
                &context.node_ids,
                grant_evidence.receipts(),
                &tampered_privacy_receipts,
            )
            .is_err(),
        "tampered privacy-state evidence cannot establish the release barrier"
    );
    let generic_evidence = producer
        .conformance_network_convergence_evidence(
            context.node_ids[0],
            &context.manifest,
            &context.node_ids,
            grant_evidence.receipts(),
        )
        .expect("generic Issue #6 evidence remains independently verifiable");
    assert!(
        producer
            .network_converged_privacy_prefix(&generic_evidence)
            .is_err(),
        "generic envelope convergence cannot authorize a privacy release"
    );
    let grant_request = LivePrivacyReleaseRequest::grantee(
        object_id,
        grantee,
        grantee_wrapping_reference.clone(),
        grant_id,
    )
    .expect("grantee release request");
    let response = producer
        .live_privacy_release(&grant_evidence, &grant_request)
        // Race side A: a release linearized before RevokeGrant is allowed.
        .expect("active grant release at exact convergence barrier");
    assert_eq!(
        response.payload().canonical_bytes(),
        payload.canonical_bytes()
    );
    assert_eq!(response.evidence().grant_id(), Some(grant_id));
    match response.envelope() {
        PrivacyReleaseEnvelope::Grant(envelope) => {
            assert_eq!(envelope.canonical_bytes(), grant_delivery.canonical_bytes());
        }
        PrivacyReleaseEnvelope::Owner(_) => panic!("grantee release returned owner envelope"),
    }
    let follower_response = followers[0]
        .live_privacy_release(&grant_evidence, &grant_request)
        .expect("follower serves the same converged grant response");
    assert_eq!(
        follower_response.canonical_bytes(),
        response.canonical_bytes()
    );

    let grant_creation_position = producer
        .effective_privacy_state()
        .expect("grant state")
        .privacy_grant(&grant_id)
        .expect("grant history")
        .creation_ledger_position();
    let revoke_unsigned = UnsignedPrivacyTransition::revoke_grant(
        &profile,
        producer
            .next_privacy_admission_anchor()
            .expect("grant revoke anchor"),
        grant_id,
        object_id,
        grantee,
        owner,
        PrivacyPermission::ReadProtectedObjectV1,
        grant_creation_position,
        owner_authorization_reference,
    )
    .expect("grant revoke core");
    let revoke = revoke_unsigned
        .clone()
        .complete_grant_revocation(
            owner_authorization
                .sign(&revoke_unsigned.authorization_digest())
                .to_bytes(),
        )
        .expect("grant revoke transition");
    submit_privacy(&mut producer, &revoke, 1_700_000_010_007, &proposer);
    assert!(
        producer
            .live_privacy_release(&grant_evidence, &grant_request)
            // Race side B: the same pre-revocation evidence is stale after the
            // local revoke journal fsync and must not bypass the new prefix.
            .is_err(),
        "local journal fsync makes the old grant evidence stale"
    );
    replicate_latest_privacy_envelope(&producer, &mut followers);
    let revoked_evidence = convergence_evidence(&producer, &followers, &context);
    let revoked_prefix = producer
        .network_converged_privacy_prefix(&revoked_evidence)
        .expect("revoked current prefix is converged");
    let release_custody = ParticipantCustody::new(root.path().join("release-custody"));
    let release_passphrase = CustodyPassphrase::new(b"issue10-release-passphrase")
        .expect("valid release custody passphrase");
    assert_eq!(
        release_custody
            .open_live_release(
                &revoked_prefix,
                &release_passphrase,
                &response,
                object_context,
            )
            .expect_err("pre-revocation response cannot open at a later prefix"),
        CustodyError::ProtectedDataOpenFailed
    );
    for ledger in [&producer, &followers[0], &followers[1]] {
        let state = ledger
            .effective_privacy_state()
            .expect("replayed converged grant state");
        assert_eq!(
            state.digest(),
            producer.effective_privacy_state().unwrap().digest()
        );
        assert_eq!(
            state
                .privacy_grant(&grant_id)
                .expect("revoked history")
                .status(),
            provchain_org::privacy::PrivacyGrantStatus::Revoked
        );
    }
    assert!(
        producer
            .live_privacy_release(&revoked_evidence, &grant_request)
            // After the revoke converges, the stable post-revocation barrier
            // denies the same grantee request on every node.
            .is_err(),
        "converged terminal revocation denies the grantee"
    );
    assert!(
        followers[0]
            .live_privacy_release(&revoked_evidence, &grant_request)
            .is_err(),
        "all converged followers deny the terminal grant"
    );

    let owner_request =
        LivePrivacyReleaseRequest::owner(object_id, owner, owner_wrapping_reference)
            .expect("owner release request");
    let owner_response = producer
        .live_privacy_release(&revoked_evidence, &owner_request)
        .expect("owner release remains independent of grantee revocation");
    match owner_response.envelope() {
        PrivacyReleaseEnvelope::Owner(envelope) => {
            assert_eq!(envelope.canonical_bytes(), owner_envelope.canonical_bytes());
        }
        PrivacyReleaseEnvelope::Grant(_) => panic!("owner release returned grant envelope"),
    }

    let secrets: [&[u8]; 7] = [
        &owner_authorization_seed,
        &grantee_authorization_seed,
        &owner_wrapping_scalar,
        &grantee_wrapping_scalar,
        &dek,
        &object_hpke_entropy,
        &grant_hpke_entropy,
    ];
    for (index, ledger) in [&producer, &followers[0], &followers[1]].iter().enumerate() {
        assert_bytes_exclude(
            &format!("Issue #10 node {index} journal"),
            &ledger.journal_bytes().expect("read exact journal"),
            &secrets,
        );
    }
    assert_bytes_exclude(
        "Issue #10 grant response",
        &response.canonical_bytes(),
        &secrets,
    );
    assert_bytes_exclude(
        "Issue #10 owner response",
        &owner_response.canonical_bytes(),
        &secrets,
    );
    assert!(
        !response
            .canonical_bytes()
            .windows(plaintext.len())
            .any(|window| window == plaintext),
        "live release response carries ciphertext rather than plaintext"
    );
}
