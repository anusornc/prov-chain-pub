//! Acceptance tests for participant registration and privacy-key lifecycle (Issue #8).

use std::collections::BTreeSet;

use ed25519_dalek::{Signer, SigningKey};
use p256::ecdsa::{
    signature::Signer as _, Signature as P256Signature, SigningKey as P256SigningKey,
};
use provchain_org::ledger::{
    AdmissionCandidate, AdmissionKind, AdmissionOutcome, Ledger, LedgerProfile,
};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use provchain_org::privacy::{
    EffectivePrivacyState, KeyLifecycleStatus, ParticipantKeyPurpose, PrivacyAdmissionAnchor,
    PrivacyControlTransition, PrivacyError, PrivacyKeyUse, PrivacyLifecycleConformanceProfile,
    PrivacyLifecycleProfile, UnsignedPrivacyTransition, CANONICAL_PRIVACY_ENCODING_V1,
    PRIVACY_CONTROL_V1,
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue8";
const PROFILE_ID: &str = "issue8.reference";

#[derive(Debug, PartialEq, Eq)]
struct ParsedRecord {
    tag: u8,
    fields: Vec<Vec<u8>>,
}

fn parse_record(bytes: &[u8]) -> ParsedRecord {
    assert!(bytes.starts_with(b"PCV1"), "record magic");
    let tag = bytes[4];
    let field_count = bytes[5] as usize;
    let mut cursor = 6;
    let mut fields = Vec::with_capacity(field_count);
    for expected_tag in 1..=field_count {
        assert_eq!(bytes[cursor], expected_tag as u8, "field order");
        cursor += 1;
        let length = u32::from_be_bytes(
            bytes[cursor..cursor + 4]
                .try_into()
                .expect("four-byte field length"),
        ) as usize;
        cursor += 4;
        fields.push(bytes[cursor..cursor + length].to_vec());
        cursor += length;
    }
    assert_eq!(cursor, bytes.len(), "no trailing bytes");
    ParsedRecord { tag, fields }
}

fn seeded_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seeded_p256_key(seed: u8) -> P256SigningKey {
    P256SigningKey::from_slice(&[seed; 32]).expect("valid deterministic P-256 fixture scalar")
}

fn p256_public_key(key: &P256SigningKey) -> [u8; 65] {
    key.verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed SEC1 P-256 point")
}

fn p256_possession_proof(key: &P256SigningKey, digest: &[u8; 32]) -> [u8; 64] {
    let signature: P256Signature = key.sign(digest);
    let signature = signature.normalize_s();
    signature.to_bytes().into()
}

fn high_s_p256_possession_proof(key: &P256SigningKey, digest: &[u8; 32]) -> [u8; 64] {
    let signature: P256Signature = key.sign(digest);
    let low_signature = signature.normalize_s();
    let (r, low_s) = low_signature.split_bytes();
    let curve_order =
        hex::decode("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551")
            .expect("P-256 group order fixture");
    let mut high_s = [0_u8; 32];
    let mut borrow = 0_i16;
    for index in (0..32).rev() {
        let difference = curve_order[index] as i16 - low_s[index] as i16 - borrow;
        if difference < 0 {
            high_s[index] = (difference + 256) as u8;
            borrow = 1;
        } else {
            high_s[index] = difference as u8;
            borrow = 0;
        }
    }
    assert_eq!(borrow, 0, "fixture subtraction cannot underflow");
    let high_signature =
        P256Signature::from_scalars(r, high_s).expect("valid high-S P-256 signature twin");
    assert_ne!(high_signature.normalize_s(), high_signature);
    high_signature.to_bytes().into()
}

fn privacy_vector_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("vectors/privacy_lifecycle_v1.json"))
        .expect("checked-in privacy lifecycle vector JSON")
}

fn vector_string<'a>(fixture: &'a serde_json::Value, pointer: &str) -> &'a str {
    fixture
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("missing vector string at {pointer}"))
}

fn vector_hex(fixture: &serde_json::Value, pointer: &str) -> Vec<u8> {
    hex::decode(vector_string(fixture, pointer))
        .unwrap_or_else(|error| panic!("invalid vector hex at {pointer}: {error}"))
}

fn vector_fixed<const N: usize>(fixture: &serde_json::Value, pointer: &str) -> [u8; N] {
    vector_hex(fixture, pointer)
        .try_into()
        .unwrap_or_else(|value: Vec<u8>| {
            panic!(
                "vector at {pointer} has length {}, expected {N}",
                value.len()
            )
        })
}

fn vector_u64(fixture: &serde_json::Value, pointer: &str) -> u64 {
    fixture
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("missing vector integer at {pointer}"))
}

fn vector_anchor(
    fixture: &serde_json::Value,
    name: &str,
    profile_hash: [u8; 32],
) -> PrivacyAdmissionAnchor {
    let base = format!("/inputs/anchors/{name}");
    let parent_envelope_pointer = format!("{base}/parent_envelope_hash_hex");
    let parent_envelope = fixture
        .pointer(&parent_envelope_pointer)
        .and_then(serde_json::Value::as_str)
        .map(|_| vector_fixed::<32>(fixture, &parent_envelope_pointer));
    PrivacyAdmissionAnchor::new(
        vector_string(fixture, "/inputs/network_id"),
        vector_string(fixture, "/inputs/profile_id"),
        profile_hash,
        vector_u64(fixture, &format!("{base}/position")),
        vector_u64(fixture, &format!("{base}/privacy_revision")),
        vector_fixed(fixture, &format!("{base}/parent_ledger_prefix_hash_hex")),
        parent_envelope,
    )
    .unwrap_or_else(|error| panic!("valid {name} vector anchor: {error}"))
}

fn validator_member(
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> NetworkMember {
    NetworkMember {
        node_id,
        identity_public_key: identity_key.verifying_key().to_bytes(),
        roles: vec![MemberRole::Peer, MemberRole::Validator],
        status: MemberStatus::Active,
        validator_public_key: Some(validator_key.verifying_key().to_bytes()),
    }
}

fn signed_manifest(
    governance_key: &SigningKey,
    members: Vec<NetworkMember>,
) -> SignedMembershipManifest {
    MembershipManifest {
        manifest_id: "issue8.membership".to_string(),
        version: 1,
        network_id: NETWORK_ID.to_string(),
        network_profile_id: PROFILE_ID.to_string(),
        members,
    }
    .sign(governance_key)
    .expect("sign Issue #8 membership manifest")
}

fn raw_privacy_network_profile(
    manifest: &SignedMembershipManifest,
    authority_keys: &[SigningKey],
    lifecycle_profile: PrivacyLifecycleProfile,
) -> anyhow::Result<NetworkProfile> {
    let package = semantic_package();
    NetworkProfile {
        profile_id: PROFILE_ID.to_string(),
        network_id: NETWORK_ID.to_string(),
        consensus: ConsensusProfile {
            consensus_type: "poa".to_string(),
            authority_keys: authority_keys
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
        privacy: Some(lifecycle_profile),
        bridge: None,
        bridge_source_trust: None,
    }
    .with_derived_privacy_content_hash()
}

fn privacy_network_profile(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    authority_keys: &[SigningKey],
    lifecycle_profile: PrivacyLifecycleProfile,
) -> NetworkProfile {
    let mut network_role_public_keys = BTreeSet::from([governance_key.verifying_key().to_bytes()]);
    for member in &manifest.manifest.members {
        network_role_public_keys.insert(member.identity_public_key);
        if let Some(validator_public_key) = member.validator_public_key {
            network_role_public_keys.insert(validator_public_key);
        }
    }
    for authority_key in authority_keys {
        network_role_public_keys.insert(authority_key.verifying_key().to_bytes());
    }
    let lifecycle_profile = lifecycle_profile
        .with_network_role_public_keys(network_role_public_keys)
        .expect("bind verified Network Profile role keys");
    raw_privacy_network_profile(manifest, authority_keys, lifecycle_profile)
        .expect("derive canonical Network Profile content hash")
}

fn conformance_lifecycle_for_ledger(
    bootstrap_key: &SigningKey,
) -> (PrivacyLifecycleProfile, PrivacyLifecycleConformanceProfile) {
    let governance_key = seeded_key(240);
    let identity_key = seeded_key(241);
    let validator_key = seeded_key(242);
    let manifest = signed_manifest(
        &governance_key,
        vec![validator_member(
            Uuid::from_u128(8_240),
            &identity_key,
            &validator_key,
        )],
    );
    let lifecycle_profile =
        PrivacyLifecycleProfile::new([1; 32], bootstrap_key.verifying_key().to_bytes())
            .expect("construct lifecycle declaration");
    let network_profile = privacy_network_profile(
        &governance_key,
        &manifest,
        std::slice::from_ref(&validator_key),
        lifecycle_profile,
    );
    let conformance = network_profile
        .privacy_lifecycle_conformance_slice(&manifest, &governance_key.verifying_key())
        .expect("verify lifecycle conformance context")
        .expect("lifecycle is declared");
    (
        network_profile
            .privacy
            .clone()
            .expect("bound lifecycle declaration"),
        conformance,
    )
}

#[test]
fn register_principal_has_exact_closed_canonical_bytes_and_independent_proofs() {
    let bootstrap_key = seeded_key(41);
    let participant_key = seeded_key(42);
    let profile_hash = [0xA5; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let principal = Uuid::from_bytes([0x11; 16]);
    let anchor =
        PrivacyAdmissionAnchor::genesis("provchain.issue8", "issue8.reference", profile_hash, 0, 1)
            .expect("canonical genesis anchor");

    let unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        anchor,
        principal,
        participant_key.verifying_key().to_bytes(),
    )
    .expect("canonical unsigned registration");
    let transition_id = unsigned.transition_id();
    let governance_signature = bootstrap_key
        .sign(&unsigned.authorization_digest())
        .to_bytes();
    let possession_signature = participant_key
        .sign(
            &unsigned
                .possession_digest()
                .expect("registration possession digest"),
        )
        .to_bytes();
    let transition = unsigned
        .complete_registration(governance_signature, possession_signature)
        .expect("complete registration");

    let complete = parse_record(&transition.canonical_bytes());
    assert_eq!(complete.tag, 0x40);
    assert_eq!(complete.fields.len(), 4);
    assert_eq!(complete.fields[1], transition_id);
    assert_eq!(complete.fields[2], governance_signature);
    assert_eq!(complete.fields[3], possession_signature);

    let core = parse_record(&complete.fields[0]);
    assert_eq!(core.tag, 0x30);
    assert_eq!(core.fields.len(), 14);
    assert_eq!(core.fields[0], PRIVACY_CONTROL_V1.as_bytes());
    assert_eq!(core.fields[1], CANONICAL_PRIVACY_ENCODING_V1.as_bytes());
    assert_eq!(core.fields[2], [0x01]);
    assert_eq!(core.fields[3], b"provchain.issue8");
    assert_eq!(core.fields[4], b"issue8.reference");
    assert_eq!(core.fields[5], profile_hash);
    assert_eq!(core.fields[6], 0_u64.to_be_bytes());
    assert_eq!(core.fields[7], 1_u64.to_be_bytes());
    assert_eq!(core.fields[8].len(), 32);
    assert_eq!(core.fields[9], [&[0_u8][..], &[0_u8; 32][..]].concat());
    assert_eq!(core.fields[10], principal.as_bytes());
    assert_eq!(core.fields[12], [0x01]);

    let initial_binding = parse_record(&core.fields[11]);
    assert_eq!(initial_binding.tag, 0x10);
    assert_eq!(initial_binding.fields.len(), 8);
    assert_eq!(initial_binding.fields[0], principal.as_bytes());
    assert_eq!(initial_binding.fields[1], [0x01]);
    assert_eq!(initial_binding.fields[2], 1_u32.to_be_bytes());
    assert_eq!(initial_binding.fields[3], [0x01]);
    assert_eq!(initial_binding.fields[4], [0x01]);
    assert_eq!(initial_binding.fields[6], [0x01]);
    assert_eq!(
        initial_binding.fields[7],
        participant_key.verifying_key().to_bytes()
    );

    let bootstrap_reference = parse_record(&core.fields[13]);
    assert_eq!(bootstrap_reference.tag, 0x12);
    assert_eq!(bootstrap_reference.fields.len(), 5);
    assert_eq!(bootstrap_reference.fields[0], [0x01]);
    assert_eq!(bootstrap_reference.fields[1], 1_u32.to_be_bytes());
    assert_eq!(bootstrap_reference.fields[2], [0x01]);
    assert_eq!(bootstrap_reference.fields[3], [0x01]);

    assert_eq!(
        PrivacyControlTransition::decode(&transition.canonical_bytes())
            .expect("strict canonical decode"),
        transition
    );
}

#[test]
fn checked_in_cross_implementation_vectors_match_active_lifecycle_surface() {
    let fixture = privacy_vector_fixture();
    assert_eq!(
        vector_string(&fixture, "/manifest/schema"),
        "ProvChainPrivacyLifecycleVectorsV1"
    );
    assert_eq!(
        vector_string(&fixture, "/manifest/rust_msrv"),
        env!("CARGO_PKG_RUST_VERSION")
    );
    assert_eq!(
        vector_string(&fixture, "/manifest/generator/source_sha256"),
        hex::encode(Sha256::digest(include_bytes!(
            "../scripts/generate_privacy_lifecycle_vectors.py"
        )))
    );
    assert_eq!(
        vector_string(&fixture, "/manifest/cargo_lock_sha256"),
        hex::encode(Sha256::digest(include_bytes!("../Cargo.lock")))
    );

    let bootstrap_seed = vector_fixed(&fixture, "/inputs/bootstrap_seed_hex");
    let authorization_seed = vector_fixed(&fixture, "/inputs/authorization_seed_hex");
    let wrapping_scalar = vector_fixed::<32>(&fixture, "/inputs/wrapping_private_scalar_hex");
    let bootstrap_key = SigningKey::from_bytes(&bootstrap_seed);
    let authorization_key = SigningKey::from_bytes(&authorization_seed);
    let wrapping_key =
        P256SigningKey::from_slice(&wrapping_scalar).expect("valid test-only P-256 vector scalar");
    assert_eq!(
        bootstrap_key.verifying_key().to_bytes(),
        vector_fixed::<32>(&fixture, "/inputs/bootstrap_public_key_hex")
    );
    assert_eq!(
        authorization_key.verifying_key().to_bytes(),
        vector_fixed::<32>(&fixture, "/inputs/authorization_public_key_hex")
    );
    assert_eq!(
        p256_public_key(&wrapping_key),
        vector_fixed::<65>(&fixture, "/inputs/wrapping_public_key_hex")
    );

    let profile_hash = vector_fixed(&fixture, "/inputs/profile_content_hash_hex");
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("construct vector lifecycle profile");
    assert_eq!(
        profile.bootstrap_fingerprint(),
        vector_fixed::<32>(&fixture, "/fingerprints/bootstrap_governance_hex")
    );
    let principal = Uuid::parse_str(vector_string(&fixture, "/inputs/principal_uuid"))
        .expect("vector principal UUID");

    let register_anchor = vector_anchor(&fixture, "register", profile_hash);
    let unsigned_register = UnsignedPrivacyTransition::register_principal(
        &profile,
        register_anchor.clone(),
        principal,
        authorization_key.verifying_key().to_bytes(),
    )
    .expect("construct vector registration");
    assert_eq!(
        unsigned_register.transition_id(),
        vector_fixed::<32>(&fixture, "/register/transition_id_hex")
    );
    assert_eq!(
        unsigned_register.authorization_digest(),
        vector_fixed::<32>(&fixture, "/register/authorization_digest_hex")
    );
    assert_eq!(
        unsigned_register.possession_digest(),
        Some(vector_fixed::<32>(
            &fixture,
            "/register/possession_digest_hex"
        ))
    );
    let governance_signature = bootstrap_key
        .sign(&unsigned_register.authorization_digest())
        .to_bytes();
    let register_possession_signature = authorization_key
        .sign(
            &unsigned_register
                .possession_digest()
                .expect("vector registration possession digest"),
        )
        .to_bytes();
    assert_eq!(
        governance_signature,
        vector_fixed::<64>(&fixture, "/register/governance_signature_hex")
    );
    assert_eq!(
        register_possession_signature,
        vector_fixed::<64>(&fixture, "/register/possession_signature_hex")
    );
    let registration = unsigned_register
        .complete_registration(governance_signature, register_possession_signature)
        .expect("complete vector registration");
    let register_complete = vector_hex(&fixture, "/register/complete_hex");
    assert_eq!(registration.canonical_bytes(), register_complete);
    assert_eq!(
        parse_record(&register_complete).fields[0],
        vector_hex(&fixture, "/register/core_hex")
    );
    assert_eq!(
        registration
            .introduced_key_reference()
            .expect("registration key reference")
            .fingerprint(),
        vector_fixed::<32>(&fixture, "/fingerprints/authorization_v1_hex")
    );
    assert_eq!(
        PrivacyControlTransition::decode(&register_complete).expect("decode vector registration"),
        registration
    );
    let state_v1 = EffectivePrivacyState::new()
        .stage_transition(&profile, &register_anchor, &registration)
        .expect("admit vector registration");

    let authorization_reference = registration
        .introduced_key_reference()
        .expect("active vector authorization reference");
    let bind_anchor = vector_anchor(&fixture, "bind", profile_hash);
    let unsigned_bind = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        bind_anchor.clone(),
        principal,
        1,
        p256_public_key(&wrapping_key),
        authorization_reference.clone(),
    )
    .expect("construct vector wrapping-key binding");
    assert_eq!(
        unsigned_bind.transition_id(),
        vector_fixed::<32>(&fixture, "/bind_wrapping/transition_id_hex")
    );
    assert_eq!(
        unsigned_bind.authorization_digest(),
        vector_fixed::<32>(&fixture, "/bind_wrapping/authorization_digest_hex")
    );
    let bind_possession_digest = unsigned_bind
        .possession_digest()
        .expect("vector binding possession digest");
    assert_eq!(
        bind_possession_digest,
        vector_fixed::<32>(&fixture, "/bind_wrapping/possession_digest_hex")
    );
    let bind_authorization_signature = authorization_key
        .sign(&unsigned_bind.authorization_digest())
        .to_bytes();
    let low_s_possession_proof = p256_possession_proof(&wrapping_key, &bind_possession_digest);
    assert_eq!(
        bind_authorization_signature,
        vector_fixed::<64>(&fixture, "/bind_wrapping/authorization_signature_hex")
    );
    assert_eq!(
        low_s_possession_proof,
        vector_fixed::<64>(&fixture, "/bind_wrapping/low_s_possession_proof_hex")
    );
    assert_eq!(
        high_s_p256_possession_proof(&wrapping_key, &bind_possession_digest),
        vector_fixed::<64>(&fixture, "/negative/p256_high_s_proof_hex")
    );
    let binding = unsigned_bind
        .complete_binding(bind_authorization_signature, low_s_possession_proof)
        .expect("complete vector wrapping-key binding");
    let bind_complete = vector_hex(&fixture, "/bind_wrapping/complete_hex");
    assert_eq!(binding.canonical_bytes(), bind_complete);
    assert_eq!(
        parse_record(&bind_complete).fields[0],
        vector_hex(&fixture, "/bind_wrapping/core_hex")
    );
    let wrapping_reference = binding
        .introduced_key_reference()
        .expect("vector wrapping-key reference");
    assert_eq!(
        wrapping_reference.fingerprint(),
        vector_fixed::<32>(&fixture, "/fingerprints/wrapping_v1_hex")
    );

    let high_s_binding = PrivacyControlTransition::decode(&vector_hex(
        &fixture,
        "/negative/p256_high_s_complete_hex",
    ))
    .expect("high-S vector is structurally canonical");
    assert!(
        state_v1
            .stage_transition(&profile, &bind_anchor, &high_s_binding)
            .is_err(),
        "high-S proof must fail closed"
    );
    let prehash_binding = PrivacyControlTransition::decode(&vector_hex(
        &fixture,
        "/negative/p256_prehash_complete_hex",
    ))
    .expect("prehash vector is structurally canonical");
    assert!(
        state_v1
            .stage_transition(&profile, &bind_anchor, &prehash_binding)
            .is_err(),
        "prehash proof must fail closed"
    );
    let zero_r_binding = PrivacyControlTransition::decode(&vector_hex(
        &fixture,
        "/negative/p256_zero_r_complete_hex",
    ))
    .expect("zero-r vector is structurally canonical");
    assert!(
        state_v1
            .stage_transition(&profile, &bind_anchor, &zero_r_binding)
            .is_err(),
        "invalid P-256 scalar must fail closed"
    );
    assert!(
        PrivacyControlTransition::decode(&vector_hex(&fixture, "/negative/p256_der_complete_hex"))
            .is_err(),
        "DER proof encoding must fail closed"
    );
    assert!(
        PrivacyControlTransition::decode(&vector_hex(
            &fixture,
            "/negative/invalid_p256_point_complete_hex"
        ))
        .is_err(),
        "off-curve P-256 point must fail closed"
    );
    assert_eq!(state_v1.revision(), 1, "negative vectors are non-mutating");

    assert_eq!(
        PrivacyControlTransition::decode(&bind_complete).expect("decode vector binding"),
        binding
    );
    let state_v2 = state_v1
        .stage_transition(&profile, &bind_anchor, &binding)
        .expect("admit vector wrapping-key binding");

    let revoke_anchor = vector_anchor(&fixture, "revoke", profile_hash);
    let unsigned_revoke = UnsignedPrivacyTransition::revoke_participant_key(
        &profile,
        revoke_anchor.clone(),
        principal,
        wrapping_reference.clone(),
        authorization_reference,
    )
    .expect("construct vector wrapping-key revocation");
    assert_eq!(
        unsigned_revoke.transition_id(),
        vector_fixed::<32>(&fixture, "/revoke_wrapping/transition_id_hex")
    );
    assert_eq!(
        unsigned_revoke.authorization_digest(),
        vector_fixed::<32>(&fixture, "/revoke_wrapping/authorization_digest_hex")
    );
    let revoke_signature = authorization_key
        .sign(&unsigned_revoke.authorization_digest())
        .to_bytes();
    assert_eq!(
        revoke_signature,
        vector_fixed::<64>(&fixture, "/revoke_wrapping/authorization_signature_hex")
    );
    let revocation = unsigned_revoke
        .complete_revocation(revoke_signature)
        .expect("complete vector wrapping-key revocation");
    let revoke_complete = vector_hex(&fixture, "/revoke_wrapping/complete_hex");
    assert_eq!(revocation.canonical_bytes(), revoke_complete);
    assert_eq!(
        parse_record(&revoke_complete).fields[0],
        vector_hex(&fixture, "/revoke_wrapping/core_hex")
    );
    assert_eq!(
        PrivacyControlTransition::decode(&revoke_complete).expect("decode vector revocation"),
        revocation
    );
    let state_v3 = state_v2
        .stage_transition(&profile, &revoke_anchor, &revocation)
        .expect("admit vector wrapping-key revocation");
    assert_eq!(state_v3.revision(), 3);
    assert_eq!(
        state_v3.key_status(&wrapping_reference),
        Some(KeyLifecycleStatus::Revoked)
    );
}

#[test]
fn canonical_decoder_rejects_ambiguous_unknown_malformed_and_unimplemented_forms() {
    let bootstrap_key = seeded_key(43);
    let participant_key = seeded_key(44);
    let profile_hash = [0xA6; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let anchor = PrivacyAdmissionAnchor::genesis(NETWORK_ID, PROFILE_ID, profile_hash, 0, 1)
        .expect("canonical genesis anchor");
    let unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        anchor,
        Uuid::from_u128(4_304),
        participant_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let transition = unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&unsigned.authorization_digest())
                .to_bytes(),
            participant_key
                .sign(&unsigned.possession_digest().expect("possession digest"))
                .to_bytes(),
        )
        .expect("complete registration");
    let canonical = transition.canonical_bytes();

    let mut wrong_magic = canonical.clone();
    wrong_magic[0] ^= 0x01;
    assert!(PrivacyControlTransition::decode(&wrong_magic).is_err());

    let mut missing_field = canonical.clone();
    missing_field[5] = 3;
    assert!(PrivacyControlTransition::decode(&missing_field).is_err());

    let core_length =
        u32::from_be_bytes(canonical[7..11].try_into().expect("complete core length")) as usize;
    let second_field_tag = 11 + core_length;
    let mut duplicate_field = canonical.clone();
    duplicate_field[second_field_tag] = 1;
    assert!(PrivacyControlTransition::decode(&duplicate_field).is_err());

    let mut trailing = canonical.clone();
    trailing.push(0);
    assert!(PrivacyControlTransition::decode(&trailing).is_err());

    let mut two_transitions = canonical.clone();
    two_transitions.extend_from_slice(&canonical);
    assert!(PrivacyControlTransition::decode(&two_transitions).is_err());

    let mut implemented_variant_with_wrong_shape = canonical.clone();
    implemented_variant_with_wrong_shape[4] = 0x43;
    assert!(matches!(
        PrivacyControlTransition::decode(&implemented_variant_with_wrong_shape),
        Err(PrivacyError::Malformed(_))
    ));

    let mut known_later_variant = canonical.clone();
    known_later_variant[4] = 0x44;
    assert!(matches!(
        PrivacyControlTransition::decode(&known_later_variant),
        Err(PrivacyError::Malformed(_))
    ));

    let mut unknown_variant = canonical;
    unknown_variant[4] = 0x7F;
    assert_eq!(
        PrivacyControlTransition::decode(&unknown_variant),
        Err(PrivacyError::UnknownTransitionRecord(0x7F))
    );
}

#[test]
fn registration_verifies_both_proofs_and_stages_one_atomic_effective_state_delta() {
    let bootstrap_key = seeded_key(51);
    let participant_key = seeded_key(52);
    let profile_hash = [0xB6; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let principal = Uuid::from_bytes([0x22; 16]);
    let anchor =
        PrivacyAdmissionAnchor::genesis("provchain.issue8", "issue8.reference", profile_hash, 0, 1)
            .expect("canonical genesis anchor");
    let unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        anchor.clone(),
        principal,
        participant_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let governance_signature = bootstrap_key
        .sign(&unsigned.authorization_digest())
        .to_bytes();
    let possession_signature = participant_key
        .sign(&unsigned.possession_digest().expect("possession digest"))
        .to_bytes();
    let transition = unsigned
        .complete_registration(governance_signature, possession_signature)
        .expect("complete registration");
    let introduced_key = transition
        .introduced_key_reference()
        .expect("registration introduces one key");

    let parent = EffectivePrivacyState::new();
    let committed = parent
        .stage_transition(&profile, &anchor, &transition)
        .expect("verify and stage registration");
    assert_eq!(parent.revision(), 0, "parent remains immutable");
    assert_eq!(committed.revision(), 1);
    assert!(committed.contains_principal(principal));
    assert_eq!(
        committed.key_status(&introduced_key),
        Some(KeyLifecycleStatus::Active)
    );
    assert_eq!(
        committed.active_key(principal, ParticipantKeyPurpose::PrivacyAuthorization),
        Some(&introduced_key)
    );
    assert_ne!(committed.digest(), parent.digest());

    let mut tampered_bytes = transition.canonical_bytes();
    *tampered_bytes.last_mut().expect("signature byte") ^= 0x01;
    let tampered = PrivacyControlTransition::decode(&tampered_bytes)
        .expect("tampered proof remains structurally canonical");
    assert!(parent
        .stage_transition(&profile, &anchor, &tampered)
        .expect_err("invalid possession proof must reject")
        .to_string()
        .contains("possession"));
    assert_eq!(parent.revision(), 0);
    assert!(!parent.contains_principal(principal));
}

#[test]
fn authorization_rotation_is_contiguous_and_self_revocation_freezes_participant_control() {
    let bootstrap_key = seeded_key(61);
    let authorization_v1 = seeded_key(62);
    let authorization_v2 = seeded_key(63);
    let authorization_v3 = seeded_key(64);
    let profile_hash = [0xC7; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let principal = Uuid::from_bytes([0x33; 16]);
    let register_anchor =
        PrivacyAdmissionAnchor::genesis("provchain.issue8", "issue8.reference", profile_hash, 0, 1)
            .expect("registration anchor");
    let register_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        register_anchor.clone(),
        principal,
        authorization_v1.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let register = register_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&register_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v1
                .sign(
                    &register_unsigned
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete registration");
    let state_v1 = EffectivePrivacyState::new()
        .stage_transition(&profile, &register_anchor, &register)
        .expect("stage registration");
    let key_v1 = register
        .introduced_key_reference()
        .expect("initial authorization key");

    let rotate_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        1,
        2,
        [0xD1; 32],
        Some([0xE1; 32]),
    )
    .expect("rotation anchor");
    let rotate_unsigned = UnsignedPrivacyTransition::bind_authorization_key(
        &profile,
        rotate_anchor.clone(),
        principal,
        2,
        authorization_v2.verifying_key().to_bytes(),
        key_v1.clone(),
    )
    .expect("rotation core");
    let rotate = rotate_unsigned
        .clone()
        .complete_binding(
            authorization_v1
                .sign(&rotate_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v2
                .sign(
                    &rotate_unsigned
                        .possession_digest()
                        .expect("rotation possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete rotation");
    let key_v2 = rotate
        .introduced_key_reference()
        .expect("replacement authorization key");
    let state_v2 = state_v1
        .stage_transition(&profile, &rotate_anchor, &rotate)
        .expect("stage atomic rotation");
    assert_eq!(
        state_v2.key_status(&key_v1),
        Some(KeyLifecycleStatus::Retired)
    );
    assert_eq!(
        state_v2.key_status(&key_v2),
        Some(KeyLifecycleStatus::Active)
    );
    assert_eq!(
        state_v2.active_key(principal, ParticipantKeyPurpose::PrivacyAuthorization),
        Some(&key_v2)
    );

    let revoke_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        2,
        3,
        [0xD2; 32],
        Some([0xE2; 32]),
    )
    .expect("revocation anchor");
    let revoke_unsigned = UnsignedPrivacyTransition::revoke_participant_key(
        &profile,
        revoke_anchor.clone(),
        principal,
        key_v2.clone(),
        key_v2.clone(),
    )
    .expect("self-revocation core");
    let revoke = revoke_unsigned
        .clone()
        .complete_revocation(
            authorization_v2
                .sign(&revoke_unsigned.authorization_digest())
                .to_bytes(),
        )
        .expect("complete self-revocation");
    let frozen = state_v2
        .stage_transition(&profile, &revoke_anchor, &revoke)
        .expect("stage self-revocation");
    assert_eq!(
        frozen.key_status(&key_v2),
        Some(KeyLifecycleStatus::Revoked)
    );
    assert_eq!(
        frozen.active_key(principal, ParticipantKeyPurpose::PrivacyAuthorization),
        None
    );

    let frozen_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        3,
        4,
        [0xD3; 32],
        Some([0xE3; 32]),
    )
    .expect("frozen parent anchor");
    let forbidden_unsigned = UnsignedPrivacyTransition::bind_authorization_key(
        &profile,
        frozen_anchor.clone(),
        principal,
        3,
        authorization_v3.verifying_key().to_bytes(),
        key_v2,
    )
    .expect("a client may encode a candidate against a stale/inactive reference");
    let forbidden = forbidden_unsigned
        .clone()
        .complete_binding(
            authorization_v2
                .sign(&forbidden_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v3
                .sign(
                    &forbidden_unsigned
                        .possession_digest()
                        .expect("replacement possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete frozen candidate");
    assert!(frozen
        .stage_transition(&profile, &frozen_anchor, &forbidden)
        .expect_err("participant-control freeze must be terminal in v1")
        .to_string()
        .contains("active privacy-authorization"));
    assert_eq!(frozen.revision(), 3);
}

#[test]
fn version_gaps_wrong_purpose_authorizers_and_global_key_reuse_are_rejected() {
    let bootstrap_key = seeded_key(65);
    let authorization_v1 = seeded_key(66);
    let authorization_v2 = seeded_key(67);
    let wrapping_v1 = seeded_p256_key(68);
    let profile_hash = [0xC8; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let principal = Uuid::from_u128(6_567);
    let register_anchor =
        PrivacyAdmissionAnchor::genesis(NETWORK_ID, PROFILE_ID, profile_hash, 0, 1)
            .expect("registration anchor");
    let register_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        register_anchor.clone(),
        principal,
        authorization_v1.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let register = register_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&register_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v1
                .sign(
                    &register_unsigned
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete registration");
    let authorization_reference = register
        .introduced_key_reference()
        .expect("initial authorization reference");
    let state_v1 = EffectivePrivacyState::new()
        .stage_transition(&profile, &register_anchor, &register)
        .expect("stage registration");

    let revision_two_anchor = PrivacyAdmissionAnchor::new(
        NETWORK_ID,
        PROFILE_ID,
        profile_hash,
        1,
        2,
        [0xC1; 32],
        Some([0xC2; 32]),
    )
    .expect("revision-two anchor");
    let gap_unsigned = UnsignedPrivacyTransition::bind_authorization_key(
        &profile,
        revision_two_anchor.clone(),
        principal,
        3,
        authorization_v2.verifying_key().to_bytes(),
        authorization_reference.clone(),
    )
    .expect("encode version-gap candidate");
    let gap = gap_unsigned
        .clone()
        .complete_binding(
            authorization_v1
                .sign(&gap_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v2
                .sign(&gap_unsigned.possession_digest().expect("possession digest"))
                .to_bytes(),
        )
        .expect("complete version-gap candidate");
    assert!(state_v1
        .stage_transition(&profile, &revision_two_anchor, &gap)
        .expect_err("key versions must be contiguous")
        .to_string()
        .contains("contiguous"));
    assert_eq!(state_v1.revision(), 1);

    let wrapping_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        revision_two_anchor.clone(),
        principal,
        1,
        p256_public_key(&wrapping_v1),
        authorization_reference.clone(),
    )
    .expect("wrapping binding core");
    let wrapping = wrapping_unsigned
        .clone()
        .complete_binding(
            authorization_v1
                .sign(&wrapping_unsigned.authorization_digest())
                .to_bytes(),
            p256_possession_proof(
                &wrapping_v1,
                &wrapping_unsigned
                    .possession_digest()
                    .expect("wrapping possession digest"),
            ),
        )
        .expect("complete wrapping binding");
    let wrapping_reference = wrapping
        .introduced_key_reference()
        .expect("wrapping key reference");
    let state_v2 = state_v1
        .stage_transition(&profile, &revision_two_anchor, &wrapping)
        .expect("stage wrapping binding");

    let revision_three_anchor = PrivacyAdmissionAnchor::new(
        NETWORK_ID,
        PROFILE_ID,
        profile_hash,
        2,
        3,
        [0xC3; 32],
        Some([0xC4; 32]),
    )
    .expect("revision-three anchor");
    let wrong_purpose_unsigned = UnsignedPrivacyTransition::bind_authorization_key(
        &profile,
        revision_three_anchor.clone(),
        principal,
        2,
        authorization_v2.verifying_key().to_bytes(),
        wrapping_reference,
    )
    .expect("encode wrong-purpose authorizer candidate");
    let wrong_purpose = wrong_purpose_unsigned
        .clone()
        .complete_binding(
            authorization_v1
                .sign(&wrong_purpose_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v2
                .sign(
                    &wrong_purpose_unsigned
                        .possession_digest()
                        .expect("authorization possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete wrong-purpose candidate");
    assert!(state_v2
        .stage_transition(&profile, &revision_three_anchor, &wrong_purpose)
        .expect_err("wrapping key cannot authorize a transition")
        .to_string()
        .contains("authorizer"));

    let duplicate_key_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        revision_three_anchor.clone(),
        Uuid::from_u128(6_568),
        authorization_v1.verifying_key().to_bytes(),
    )
    .expect("encode duplicate-key registration");
    let duplicate_key = duplicate_key_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&duplicate_key_unsigned.authorization_digest())
                .to_bytes(),
            authorization_v1
                .sign(
                    &duplicate_key_unsigned
                        .possession_digest()
                        .expect("duplicate-key possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete duplicate-key registration");
    assert!(state_v2
        .stage_transition(&profile, &revision_three_anchor, &duplicate_key)
        .expect_err("participant public keys cannot be globally reused")
        .to_string()
        .contains("already bound"));
    assert_eq!(state_v2.revision(), 2);
}

#[test]
fn wrapping_rotation_preserves_only_exact_historical_release_and_revocation_is_terminal() {
    let bootstrap_key = seeded_key(71);
    let authorization_key = seeded_key(72);
    let wrapping_v1 = seeded_p256_key(73);
    let wrapping_v2 = seeded_p256_key(74);
    let profile_hash = [0xD8; 32];
    let profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("activate lifecycle profile");
    let principal = Uuid::from_bytes([0x44; 16]);
    let register_anchor =
        PrivacyAdmissionAnchor::genesis("provchain.issue8", "issue8.reference", profile_hash, 0, 1)
            .expect("registration anchor");
    let register_unsigned = UnsignedPrivacyTransition::register_principal(
        &profile,
        register_anchor.clone(),
        principal,
        authorization_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let register = register_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&register_unsigned.authorization_digest())
                .to_bytes(),
            authorization_key
                .sign(
                    &register_unsigned
                        .possession_digest()
                        .expect("registration possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete registration");
    let authorization_reference = register
        .introduced_key_reference()
        .expect("authorization key reference");
    let state_v1 = EffectivePrivacyState::new()
        .stage_transition(&profile, &register_anchor, &register)
        .expect("stage registration");

    let bind_v1_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        1,
        2,
        [0xA1; 32],
        Some([0xB1; 32]),
    )
    .expect("first wrapping anchor");
    let bind_v1_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        bind_v1_anchor.clone(),
        principal,
        1,
        p256_public_key(&wrapping_v1),
        authorization_reference.clone(),
    )
    .expect("first wrapping binding core");
    let high_s_binding = bind_v1_unsigned
        .clone()
        .complete_binding(
            authorization_key
                .sign(&bind_v1_unsigned.authorization_digest())
                .to_bytes(),
            high_s_p256_possession_proof(
                &wrapping_v1,
                &bind_v1_unsigned
                    .possession_digest()
                    .expect("wrapping possession digest"),
            ),
        )
        .expect("complete high-S negative vector");
    assert!(state_v1
        .stage_transition(&profile, &bind_v1_anchor, &high_s_binding)
        .expect_err("strict P-256 verifier must reject high-S proof")
        .to_string()
        .contains("high-S"));
    assert_eq!(state_v1.revision(), 1, "negative vector is non-mutating");
    let bind_v1 = bind_v1_unsigned
        .clone()
        .complete_binding(
            authorization_key
                .sign(&bind_v1_unsigned.authorization_digest())
                .to_bytes(),
            p256_possession_proof(
                &wrapping_v1,
                &bind_v1_unsigned
                    .possession_digest()
                    .expect("wrapping possession digest"),
            ),
        )
        .expect("complete first wrapping binding");
    let key_v1 = bind_v1
        .introduced_key_reference()
        .expect("wrapping key v1 reference");
    let state_v2 = state_v1
        .stage_transition(&profile, &bind_v1_anchor, &bind_v1)
        .expect("stage first wrapping binding");
    assert!(state_v2.key_is_eligible(&key_v1, PrivacyKeyUse::CreateEnvelope));
    assert!(state_v2.key_is_eligible(&key_v1, PrivacyKeyUse::HistoricalRelease));
    assert!(!state_v2.key_is_eligible(&key_v1, PrivacyKeyUse::AuthorizeTransition));

    let rotate_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        2,
        3,
        [0xA2; 32],
        Some([0xB2; 32]),
    )
    .expect("wrapping rotation anchor");
    let rotate_unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
        &profile,
        rotate_anchor.clone(),
        principal,
        2,
        p256_public_key(&wrapping_v2),
        authorization_reference.clone(),
    )
    .expect("wrapping rotation core");
    let rotate = rotate_unsigned
        .clone()
        .complete_binding(
            authorization_key
                .sign(&rotate_unsigned.authorization_digest())
                .to_bytes(),
            p256_possession_proof(
                &wrapping_v2,
                &rotate_unsigned
                    .possession_digest()
                    .expect("rotation possession digest"),
            ),
        )
        .expect("complete wrapping rotation");
    let key_v2 = rotate
        .introduced_key_reference()
        .expect("wrapping key v2 reference");
    let state_v3 = state_v2
        .stage_transition(&profile, &rotate_anchor, &rotate)
        .expect("stage wrapping rotation");
    assert_eq!(
        state_v3.key_status(&key_v1),
        Some(KeyLifecycleStatus::Retired)
    );
    assert!(!state_v3.key_is_eligible(&key_v1, PrivacyKeyUse::CreateEnvelope));
    assert!(state_v3.key_is_eligible(&key_v1, PrivacyKeyUse::HistoricalRelease));
    assert!(state_v3.key_is_eligible(&key_v2, PrivacyKeyUse::CreateEnvelope));

    let revoke_anchor = PrivacyAdmissionAnchor::new(
        "provchain.issue8",
        "issue8.reference",
        profile_hash,
        3,
        4,
        [0xA3; 32],
        Some([0xB3; 32]),
    )
    .expect("wrapping revocation anchor");
    let revoke_unsigned = UnsignedPrivacyTransition::revoke_participant_key(
        &profile,
        revoke_anchor.clone(),
        principal,
        key_v2.clone(),
        authorization_reference,
    )
    .expect("wrapping revocation core");
    let revoke = revoke_unsigned
        .clone()
        .complete_revocation(
            authorization_key
                .sign(&revoke_unsigned.authorization_digest())
                .to_bytes(),
        )
        .expect("complete wrapping revocation");
    let state_v4 = state_v3
        .stage_transition(&profile, &revoke_anchor, &revoke)
        .expect("stage wrapping revocation");
    assert_eq!(
        state_v4.key_status(&key_v2),
        Some(KeyLifecycleStatus::Revoked)
    );
    assert!(!state_v4.key_is_eligible(&key_v2, PrivacyKeyUse::CreateEnvelope));
    assert!(!state_v4.key_is_eligible(&key_v2, PrivacyKeyUse::HistoricalRelease));
}

#[test]
fn final_admission_commits_control_only_bytes_and_replay_rebuilds_identical_privacy_state() {
    let bootstrap_key = seeded_key(81);
    let participant_key = seeded_key(82);
    let proposer_key = seeded_key(83);
    let (lifecycle_profile, conformance_lifecycle) =
        conformance_lifecycle_for_ledger(&bootstrap_key);
    let ledger_profile = bind_profile(
        LedgerProfile::new("provchain.issue8", "issue8.reference")
            .with_privacy_conformance_slice(conformance_lifecycle),
    );
    let data_dir = tempdir().expect("Issue #8 ledger directory");
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), ledger_profile.clone(), semantic_package())
            .expect("open privacy ledger");
    let principal = Uuid::from_bytes([0x55; 16]);
    let anchor = ledger
        .next_privacy_admission_anchor()
        .expect("derive journal-authoritative privacy parent");
    let unsigned = UnsignedPrivacyTransition::register_principal(
        &lifecycle_profile,
        anchor,
        principal,
        participant_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let registration = unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&unsigned.authorization_digest())
                .to_bytes(),
            participant_key
                .sign(&unsigned.possession_digest().expect("possession digest"))
                .to_bytes(),
        )
        .expect("complete registration");
    let introduced_key = registration
        .introduced_key_reference()
        .expect("initial key reference");
    let candidate = ledger
        .create_privacy_candidate(&registration, 1_700_000_008_001, &proposer_key)
        .expect("build outer PoA-authenticated candidate");

    let committed = match ledger
        .local_adapter()
        .submit(candidate)
        .expect("Final Admission result")
    {
        AdmissionOutcome::Committed { envelope } => *envelope,
        AdmissionOutcome::Rejected { reason } => panic!("registration rejected: {reason}"),
    };
    assert_eq!(committed.admission_kind, AdmissionKind::PrivacyControlV1);
    assert_eq!(
        committed.privacy_control.as_deref(),
        Some(registration.canonical_bytes().as_slice())
    );
    assert!(committed.public_provenance.is_empty());
    assert!(committed.encrypted_payload.is_none());
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 1);
    assert_eq!(ledger.rdf_store().len().expect("public RDF projection"), 0);
    assert_eq!(
        ledger
            .effective_privacy_state()
            .expect("verified privacy projection")
            .key_status(&introduced_key),
        Some(KeyLifecycleStatus::Active)
    );
    let committed_digest = ledger
        .effective_privacy_state()
        .expect("verified privacy projection")
        .digest();
    let committed_bytes = committed
        .canonical_bytes()
        .expect("canonical committed envelope bytes");
    drop(ledger);

    let replayed = Ledger::open_in_dir(data_dir.path(), ledger_profile, semantic_package())
        .expect("verified replay");
    assert_eq!(
        replayed
            .effective_privacy_state()
            .expect("replayed privacy projection")
            .digest(),
        committed_digest
    );
    assert_eq!(
        replayed.committed_envelopes()[0]
            .canonical_bytes()
            .expect("replayed canonical bytes"),
        committed_bytes
    );
    assert_eq!(replayed.rdf_store().len().expect("replayed RDF"), 0);
}

#[test]
fn invalid_privacy_proof_rejects_and_exact_retry_is_idempotent_without_mutation() {
    let bootstrap_key = seeded_key(84);
    let participant_key = seeded_key(85);
    let proposer_key = seeded_key(86);
    let (lifecycle_profile, conformance_lifecycle) =
        conformance_lifecycle_for_ledger(&bootstrap_key);
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID)
            .with_privacy_conformance_slice(conformance_lifecycle),
    );
    let data_dir = tempdir().expect("Issue #8 rejection ledger directory");
    let mut ledger = Ledger::open_in_dir(data_dir.path(), ledger_profile, semantic_package())
        .expect("open privacy ledger");
    let anchor = ledger
        .next_privacy_admission_anchor()
        .expect("derive registration anchor");
    let unsigned = UnsignedPrivacyTransition::register_principal(
        &lifecycle_profile,
        anchor,
        Uuid::from_u128(8_586),
        participant_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let registration = unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&unsigned.authorization_digest())
                .to_bytes(),
            participant_key
                .sign(&unsigned.possession_digest().expect("possession digest"))
                .to_bytes(),
        )
        .expect("complete registration");
    let valid_candidate = ledger
        .create_privacy_candidate(&registration, 1_700_000_008_002, &proposer_key)
        .expect("valid candidate");
    let before_tip = ledger.tip();
    let before_privacy_digest = ledger
        .effective_privacy_state()
        .expect("verified privacy projection")
        .digest();
    let before_public_quads = ledger.rdf_store().len().expect("public RDF projection");

    let mut invalid_candidate = valid_candidate.clone();
    *invalid_candidate
        .privacy_control
        .as_mut()
        .expect("privacy transition bytes")
        .last_mut()
        .expect("possession signature byte") ^= 0x01;
    let invalid_candidate = invalid_candidate
        .sign(&proposer_key)
        .expect("outer proposer can sign structurally canonical invalid proof bytes");
    match ledger
        .local_adapter()
        .submit(invalid_candidate)
        .expect("deterministic Final Admission outcome")
    {
        AdmissionOutcome::Rejected { reason } => {
            assert!(
                reason.contains("possession"),
                "unexpected rejection: {reason}"
            );
        }
        AdmissionOutcome::Committed { .. } => panic!("invalid possession proof committed"),
    }
    assert_eq!(ledger.tip(), before_tip);
    assert_eq!(
        ledger
            .effective_privacy_state()
            .expect("verified privacy projection")
            .digest(),
        before_privacy_digest
    );
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
    assert_eq!(
        ledger.rdf_store().len().expect("public RDF projection"),
        before_public_quads
    );

    match ledger
        .local_adapter()
        .submit(valid_candidate.clone())
        .expect("valid Final Admission outcome")
    {
        AdmissionOutcome::Committed { .. } => {}
        AdmissionOutcome::Rejected { reason } => panic!("valid registration rejected: {reason}"),
    }
    let committed_tip = ledger.tip();
    let committed_digest = ledger
        .effective_privacy_state()
        .expect("verified privacy projection")
        .digest();
    match ledger
        .local_adapter()
        .submit(valid_candidate)
        .expect("exact lost-response retry has a deterministic outcome")
    {
        AdmissionOutcome::Committed { envelope } => {
            assert_eq!(envelope.index, 0);
            assert_eq!(envelope.envelope_hash, committed_tip.envelope_hash);
        }
        AdmissionOutcome::Rejected { reason } => {
            panic!("exact committed-envelope retry was not idempotent: {reason}")
        }
    }
    assert_eq!(ledger.tip(), committed_tip);
    assert_eq!(
        ledger
            .effective_privacy_state()
            .expect("verified privacy projection")
            .digest(),
        committed_digest
    );
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 1);
}

#[test]
fn privacy_bootstrap_key_cannot_reuse_consensus_or_membership_trust_roles() {
    let governance_key = seeded_key(87);
    let identity_key = seeded_key(88);
    let validator_key = seeded_key(89);
    let node_id = Uuid::from_u128(8_789);
    let manifest = signed_manifest(
        &governance_key,
        vec![validator_member(node_id, &identity_key, &validator_key)],
    );

    let validator_reuse =
        PrivacyLifecycleProfile::new([0xEB; 32], validator_key.verifying_key().to_bytes())
            .expect("construct role-reuse fixture");
    let validator_reuse_error = raw_privacy_network_profile(
        &manifest,
        std::slice::from_ref(&validator_key),
        validator_reuse,
    )
    .expect_err("bootstrap key cannot reuse a PoA authority key");
    assert!(validator_reuse_error.to_string().contains("authority"));

    let governance_reuse =
        PrivacyLifecycleProfile::new([0xEC; 32], governance_key.verifying_key().to_bytes())
            .expect("construct governance-reuse fixture");
    let governance_reuse_profile = raw_privacy_network_profile(
        &manifest,
        std::slice::from_ref(&validator_key),
        governance_reuse,
    )
    .expect("derive governance-reuse profile hash");
    let data_dir = tempdir().expect("role-separation startup directory");
    let error = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(
            bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID)),
            semantic_package(),
        ),
        governance_reuse_profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
        validator_key,
    )
    .err()
    .expect("bootstrap key cannot reuse the membership governance root");
    assert!(error
        .to_string()
        .contains("reuses a verified Network Profile key role"));

    let governance_role_key = seeded_key(180);
    let identity_role_key = seeded_key(181);
    let validator_role_key = seeded_key(182);
    let bootstrap_key = seeded_key(183);
    let participant_key = seeded_key(184);
    let proposer_key = seeded_key(185);
    let manifest = signed_manifest(
        &governance_role_key,
        vec![validator_member(
            Uuid::from_u128(8_180),
            &identity_role_key,
            &validator_role_key,
        )],
    );
    let declared = PrivacyLifecycleProfile::new([1; 32], bootstrap_key.verifying_key().to_bytes())
        .expect("construct valid lifecycle declaration");
    let profile = privacy_network_profile(
        &governance_role_key,
        &manifest,
        std::slice::from_ref(&validator_role_key),
        declared,
    );
    let lifecycle_profile = profile
        .privacy
        .clone()
        .expect("canonical lifecycle declaration");
    assert!(profile
        .activate_privacy_lifecycle(&manifest, &governance_role_key.verifying_key())
        .expect_err("production activation remains behind the Issue #10 evidence gate")
        .to_string()
        .contains("complete Issue #10 evidence gate"));
    let conformance = profile
        .privacy_lifecycle_conformance_slice(&manifest, &governance_role_key.verifying_key())
        .expect("verify incomplete lifecycle conformance context")
        .expect("lifecycle is declared");
    let profile_hash = lifecycle_profile.network_profile_content_hash();

    let mut profile_tamper = profile.clone();
    profile_tamper.consensus.block_interval += 1;
    assert!(profile_tamper
        .validate()
        .expect_err("profile mutation must invalidate the embedded content hash")
        .to_string()
        .contains("content hash"));
    assert!(PrivacyLifecycleProfile::new([2; 32], [0; 32])
        .expect_err("weak bootstrap key must fail profile activation")
        .to_string()
        .contains("weak"));

    let registration_anchor =
        PrivacyAdmissionAnchor::genesis(NETWORK_ID, PROFILE_ID, profile_hash, 0, 1)
            .expect("role-reuse registration anchor");
    for (role, public_key) in [
        ("governance", governance_role_key.verifying_key().to_bytes()),
        ("identity", identity_role_key.verifying_key().to_bytes()),
        ("validator", validator_role_key.verifying_key().to_bytes()),
    ] {
        assert!(
            UnsignedPrivacyTransition::register_principal(
                &lifecycle_profile,
                registration_anchor.clone(),
                Uuid::new_v4(),
                public_key,
            )
            .expect_err("participant registration must reject every Network Profile key role")
            .to_string()
            .contains("Network Profile key role"),
            "missing RegisterPrincipal {role}-role rejection"
        );
    }
    assert!(UnsignedPrivacyTransition::register_principal(
        &lifecycle_profile,
        registration_anchor.clone(),
        Uuid::new_v4(),
        [0; 32],
    )
    .expect_err("weak participant authorization key must reject")
    .to_string()
    .contains("weak"));

    let permissive_profile =
        PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
            .expect("construct hostile incomplete role context");
    let reused_registration_unsigned = UnsignedPrivacyTransition::register_principal(
        &permissive_profile,
        registration_anchor.clone(),
        Uuid::from_u128(8_181),
        identity_role_key.verifying_key().to_bytes(),
    )
    .expect("hostile client can encode a role-reusing canonical transition");
    let reused_registration = reused_registration_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&reused_registration_unsigned.authorization_digest())
                .to_bytes(),
            identity_role_key
                .sign(
                    &reused_registration_unsigned
                        .possession_digest()
                        .expect("role-reuse possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete hostile role-reuse registration");
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID).with_privacy_conformance_slice(conformance),
    );
    let role_data_dir = tempdir().expect("role-reuse Final Admission ledger");
    let mut ledger = Ledger::open_in_dir(
        role_data_dir.path(),
        ledger_profile.clone(),
        semantic_package(),
    )
    .expect("open role-reuse ledger");
    let tip = ledger.tip();
    let candidate = AdmissionCandidate::new_privacy_control(
        &ledger_profile,
        0,
        tip.envelope_hash,
        tip.state_commitment,
        1_700_000_008_180,
        &reused_registration,
    )
    .expect("construct hostile outer candidate")
    .sign(&proposer_key)
    .expect("sign hostile outer candidate");
    assert!(matches!(
        ledger
            .local_adapter()
            .submit(candidate)
            .expect("deterministic role-reuse verdict"),
        AdmissionOutcome::Rejected { reason } if reason.contains("Network Profile key role")
    ));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);

    let legitimate_unsigned = UnsignedPrivacyTransition::register_principal(
        &lifecycle_profile,
        registration_anchor,
        Uuid::from_u128(8_184),
        participant_key.verifying_key().to_bytes(),
    )
    .expect("legitimate registration core");
    let legitimate_registration = legitimate_unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&legitimate_unsigned.authorization_digest())
                .to_bytes(),
            participant_key
                .sign(
                    &legitimate_unsigned
                        .possession_digest()
                        .expect("legitimate possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete legitimate registration");
    let authorizer = legitimate_registration
        .introduced_key_reference()
        .expect("legitimate authorization reference");
    let candidate = ledger
        .create_privacy_candidate(&legitimate_registration, 1_700_000_008_181, &proposer_key)
        .expect("construct legitimate registration candidate");
    assert!(matches!(
        ledger
            .local_adapter()
            .submit(candidate)
            .expect("commit legitimate registration"),
        AdmissionOutcome::Committed { .. }
    ));

    let bind_anchor = ledger
        .next_privacy_admission_anchor()
        .expect("role-reuse binding anchor");
    for (role, public_key) in [
        ("governance", governance_role_key.verifying_key().to_bytes()),
        ("identity", identity_role_key.verifying_key().to_bytes()),
        ("validator", validator_role_key.verifying_key().to_bytes()),
    ] {
        assert!(
            UnsignedPrivacyTransition::bind_authorization_key(
                &lifecycle_profile,
                bind_anchor.clone(),
                Uuid::from_u128(8_184),
                2,
                public_key,
                authorizer.clone(),
            )
            .expect_err("participant binding must reject every Network Profile key role")
            .to_string()
            .contains("Network Profile key role"),
            "missing BindParticipantKey {role}-role rejection"
        );
    }

    let reused_bind_unsigned = UnsignedPrivacyTransition::bind_authorization_key(
        &permissive_profile,
        bind_anchor,
        Uuid::from_u128(8_184),
        2,
        validator_role_key.verifying_key().to_bytes(),
        authorizer,
    )
    .expect("hostile client can encode a role-reusing bind");
    let reused_bind = reused_bind_unsigned
        .clone()
        .complete_binding(
            participant_key
                .sign(&reused_bind_unsigned.authorization_digest())
                .to_bytes(),
            validator_role_key
                .sign(
                    &reused_bind_unsigned
                        .possession_digest()
                        .expect("role-reuse bind possession digest"),
                )
                .to_bytes(),
        )
        .expect("complete hostile role-reuse bind");
    let tip = ledger.tip();
    let candidate = AdmissionCandidate::new_privacy_control(
        &ledger_profile,
        1,
        tip.envelope_hash,
        tip.state_commitment,
        1_700_000_008_182,
        &reused_bind,
    )
    .expect("construct hostile bind candidate")
    .sign(&proposer_key)
    .expect("sign hostile bind candidate");
    assert!(matches!(
        ledger
            .local_adapter()
            .submit(candidate)
            .expect("deterministic bind role-reuse verdict"),
        AdmissionOutcome::Rejected { reason } if reason.contains("Network Profile key role")
    ));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 1);
}

#[test]
fn poa_replicates_one_exact_privacy_envelope_and_three_nodes_rebuild_identical_state() {
    let governance_key = seeded_key(91);
    let bootstrap_key = seeded_key(92);
    let participant_key = seeded_key(93);
    let identity_keys = [seeded_key(94), seeded_key(95), seeded_key(96)];
    let validator_keys = [seeded_key(97), seeded_key(98), seeded_key(99)];
    let node_ids = [
        Uuid::from_u128(810),
        Uuid::from_u128(820),
        Uuid::from_u128(830),
    ];
    let lifecycle_profile =
        PrivacyLifecycleProfile::new([0xFA; 32], bootstrap_key.verifying_key().to_bytes())
            .expect("activate Issue #8 lifecycle profile");
    let manifest = signed_manifest(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                validator_member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let profile = privacy_network_profile(
        &governance_key,
        &manifest,
        &validator_keys,
        lifecycle_profile,
    );
    let lifecycle_profile = profile
        .privacy
        .clone()
        .expect("canonical lifecycle profile");
    assert!(profile
        .activate_privacy_lifecycle(&manifest, &governance_key.verifying_key())
        .expect_err("production activation remains behind the Issue #10 evidence gate")
        .to_string()
        .contains("complete Issue #10 evidence gate"));
    let conformance = profile
        .privacy_lifecycle_conformance_slice(&manifest, &governance_key.verifying_key())
        .expect("verify Issue #8 conformance context")
        .expect("privacy lifecycle declared");
    let ledger_profile = bind_profile(
        LedgerProfile::new(NETWORK_ID, PROFILE_ID).with_privacy_conformance_slice(conformance),
    );
    let data_dirs: Vec<_> = (0..3)
        .map(|_| tempdir().expect("temporary Issue #8 conformance ledger"))
        .collect();
    let mut ledgers: Vec<_> = data_dirs
        .iter()
        .map(|directory| {
            Ledger::open_in_dir(directory.path(), ledger_profile.clone(), semantic_package())
                .expect("open non-deployable privacy conformance ledger")
        })
        .collect();

    let anchor = ledgers[0]
        .next_privacy_admission_anchor()
        .expect("journal-derived registration anchor");
    let unsigned = UnsignedPrivacyTransition::register_principal(
        &lifecycle_profile,
        anchor,
        Uuid::from_u128(8_008),
        participant_key.verifying_key().to_bytes(),
    )
    .expect("registration core");
    let registration = unsigned
        .clone()
        .complete_registration(
            bootstrap_key
                .sign(&unsigned.authorization_digest())
                .to_bytes(),
            participant_key
                .sign(&unsigned.possession_digest().expect("possession digest"))
                .to_bytes(),
        )
        .expect("complete registration");
    let introduced_key = registration
        .introduced_key_reference()
        .expect("registered authorization key");
    let candidate = ledgers[0]
        .create_privacy_candidate(&registration, 50_000, &validator_keys[0])
        .expect("construct signed privacy candidate");
    match ledgers[0]
        .proposal_adapter()
        .submit(candidate)
        .expect("conformance producer commits privacy transition")
    {
        AdmissionOutcome::Committed { .. } => {}
        AdmissionOutcome::Rejected { reason } => panic!("privacy transition rejected: {reason}"),
    }
    let exact_envelope = ledgers[0].committed_envelopes()[0]
        .canonical_bytes()
        .expect("producer exact committed envelope");
    let producer_state = ledgers[0]
        .effective_privacy_state()
        .expect("producer privacy projection")
        .clone();

    for follower in ledgers.iter_mut().skip(1) {
        assert!(matches!(
            follower
                .conformance_admit_exact_envelope(&exact_envelope)
                .expect("follower Final-Admits exact producer privacy envelope"),
            AdmissionOutcome::Committed { .. }
        ));
    }

    for ledger in &ledgers {
        assert_eq!(
            ledger.committed_envelopes()[0]
                .canonical_bytes()
                .expect("node durable envelope"),
            exact_envelope
        );
        let state = ledger
            .effective_privacy_state()
            .expect("node privacy projection");
        assert_eq!(state.digest(), producer_state.digest());
        assert_eq!(
            state.key_status(&introduced_key),
            Some(KeyLifecycleStatus::Active)
        );
    }
}
