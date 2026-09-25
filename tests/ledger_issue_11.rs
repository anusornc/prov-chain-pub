//! Acceptance tests for exact source bridge declarations and export proof (Issue #11).

use ed25519_dalek::SigningKey;
use provchain_org::bridge::{
    BridgeCommitReceiptV1, BridgeExportDeclarationV1, BridgeExportTargetV1, BridgeProofBundleV1,
    BridgeSourceProfileV1, LedgerInstanceId32, MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
    MAX_BRIDGE_PROOF_BUNDLE_BYTES,
};
use provchain_org::ledger::{
    AdmissionOutcome, AdmittedBlockEnvelope, LedgerProfile, MAX_PAYLOAD_BYTES,
};
#[cfg(feature = "bridge-conformance")]
use provchain_org::ledger::{Ledger, LedgerCommitFailpoint};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::peer_session::{AuthenticatedPeerSession, PeerTransport};
use provchain_org::network::poa::PoAProposalRequest;
use provchain_org::network::profile::{
    ConsensusProfile, NetworkProfile, SemanticProfile, SourceBridgeProfileV1,
};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use tempfile::{tempdir, TempDir};
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue11.source";
const PROFILE_ID: &str = "issue11.source.reference";

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn member(node_id: Uuid, identity_key: &SigningKey, validator_key: &SigningKey) -> NetworkMember {
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
        manifest_id: "issue11.source.membership".to_string(),
        version: 11,
        network_id: NETWORK_ID.to_string(),
        network_profile_id: PROFILE_ID.to_string(),
        members,
    }
    .sign(governance_key)
    .expect("sign source membership manifest")
}

fn network_profile(
    manifest: &SignedMembershipManifest,
    authority_keys: &[SigningKey],
) -> NetworkProfile {
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
        privacy: None,
        bridge: None,
        bridge_source_trust: None,
    }
}

struct TestNode {
    _data_dir: TempDir,
    node: ReferenceNode,
}

fn start_bridge_node(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    bridge_profile: &BridgeSourceProfileV1,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> TestNode {
    let data_dir = tempdir().expect("temporary source node directory");
    let ledger_profile = bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID))
        .with_bridge_source_profile(bridge_profile.clone());
    let node = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
        validator_key.clone(),
    )
    .expect("activate source bridge node");
    TestNode {
        _data_dir: data_dir,
        node,
    }
}

fn complete_handshake(
    initiator: &mut ReferenceNode,
    initiator_transport: &PeerTransport,
    responder: &mut ReferenceNode,
    responder_transport: &PeerTransport,
    responder_node_id: Uuid,
    started_at_millis: u64,
) -> (AuthenticatedPeerSession, AuthenticatedPeerSession) {
    let hello = initiator
        .begin_peer_handshake_at(initiator_transport, responder_node_id, started_at_millis)
        .expect("begin peer handshake");
    let challenge = responder
        .accept_peer_handshake_at(responder_transport, hello, started_at_millis + 1)
        .expect("accept peer hello");
    let (response, initiator_session) = initiator
        .answer_peer_challenge_at(initiator_transport, challenge, started_at_millis + 2)
        .expect("authenticate responder");
    let responder_session = responder
        .finish_peer_handshake_at(responder_transport, response, started_at_millis + 3)
        .expect("authenticate initiator");
    (initiator_session, responder_session)
}

#[test]
fn source_final_admission_commits_one_exact_target_bound_declaration() {
    let governance_key = signing_key(1);
    let identity_keys = [signing_key(2), signing_key(3), signing_key(4)];
    let validator_keys = [signing_key(5), signing_key(6), signing_key(7)];
    let node_ids = [
        Uuid::from_u128(10),
        Uuid::from_u128(20),
        Uuid::from_u128(30),
    ];
    let manifest = signed_manifest(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let mut network_profile = network_profile(&manifest, &validator_keys);
    let source_ledger = LedgerInstanceId32::new([0x11; 32]).expect("source ledger id");
    let target = BridgeExportTargetV1::new(
        "provchain.issue11.target",
        LedgerInstanceId32::new([0x22; 32]).expect("target ledger id"),
    )
    .expect("target binding");
    network_profile.bridge = Some(
        SourceBridgeProfileV1::new(
            source_ledger.bytes(),
            target.network_id(),
            target.ledger_instance_id().bytes(),
        )
        .expect("canonical Network Profile bridge rule"),
    );
    let bridge_profile = BridgeSourceProfileV1::from_network_profile(
        &network_profile,
        &manifest,
        source_ledger,
        target.clone(),
    )
    .expect("canonical source bridge profile");
    let ledger_profile = bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID))
        .with_bridge_source_profile(bridge_profile);
    let data_dir = tempdir().expect("temporary source node directory");
    let node = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        network_profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_ids[0], MemberRole::Validator, identity_keys[0].clone()),
        validator_keys[0].clone(),
    )
    .expect("activate source reference node");
    let source = TestNode {
        _data_dir: data_dir,
        node,
    };
    let payload = b"<urn:issue11:batch:1> <urn:issue11:status> \"exportable\" .";
    let turn = source.node.pending_poa_turn().expect("source turn");
    let wrong_target = BridgeExportTargetV1::new(
        "provchain.issue11.wrong-target",
        LedgerInstanceId32::new([0x23; 32]).unwrap(),
    )
    .unwrap();
    assert!(source
        .node
        .submit_poa_request(PoAProposalRequest::bridge_export(
            turn,
            11_000,
            payload,
            wrong_target,
        ))
        .is_err());
    assert_eq!(source.node.journal_frame_count().unwrap(), 0);
    assert!(source
        .node
        .submit_poa_request(PoAProposalRequest::bridge_export(
            turn,
            11_000,
            vec![b'x'; MAX_PAYLOAD_BYTES + 1],
            target.clone(),
        ))
        .is_err());
    assert_eq!(source.node.journal_frame_count().unwrap(), 0);
    let outcome = source
        .node
        .submit_poa_request(PoAProposalRequest::bridge_export(
            turn,
            11_000,
            payload,
            target.clone(),
        ))
        .expect("scheduled source authority commits declaration");
    let AdmissionOutcome::Committed { envelope } = outcome else {
        panic!("source bridge declaration was rejected");
    };

    assert_eq!(envelope.index, 0);
    assert_eq!(envelope.public_provenance, payload);
    let declaration_bytes = envelope
        .bridge_export_declaration
        .as_deref()
        .expect("declaration is committed in the same envelope");
    let declaration =
        BridgeExportDeclarationV1::decode(declaration_bytes).expect("canonical declaration");
    assert_eq!(declaration.core().source_position(), envelope.index);
    assert_eq!(declaration.core().target(), &target);
    assert_eq!(declaration.core().public_payload_sha256(), sha256(payload));
}

#[test]
fn export_requires_exactly_three_pinned_durable_receipts_and_reproduces_source_bytes() {
    let governance_key = signing_key(11);
    let identity_keys = [signing_key(12), signing_key(13), signing_key(14)];
    let validator_keys = [signing_key(15), signing_key(16), signing_key(17)];
    let node_ids = [
        Uuid::from_u128(110),
        Uuid::from_u128(120),
        Uuid::from_u128(130),
    ];
    let manifest = signed_manifest(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let mut profile = network_profile(&manifest, &validator_keys);
    let target = BridgeExportTargetV1::new(
        "provchain.issue11.target",
        LedgerInstanceId32::new([0x42; 32]).expect("target ledger id"),
    )
    .expect("target binding");
    let source_ledger = LedgerInstanceId32::new([0x31; 32]).expect("source ledger id");
    profile.bridge = Some(
        SourceBridgeProfileV1::new(
            source_ledger.bytes(),
            target.network_id(),
            target.ledger_instance_id().bytes(),
        )
        .expect("canonical Network Profile bridge rule"),
    );
    let bridge_profile = BridgeSourceProfileV1::from_network_profile(
        &profile,
        &manifest,
        source_ledger,
        target.clone(),
    )
    .expect("source bridge profile");
    let mut nodes: Vec<_> = (0..3)
        .map(|index| {
            start_bridge_node(
                &governance_key,
                &manifest,
                &profile,
                &bridge_profile,
                node_ids[index],
                &identity_keys[index],
                &validator_keys[index],
            )
        })
        .collect();
    let payload = b"<urn:issue11:batch:2> <urn:issue11:status> \"exact-public-copy\" .";
    let turn = nodes[0].node.pending_poa_turn().expect("source turn");
    let outcome = nodes[0]
        .node
        .submit_poa_request(PoAProposalRequest::bridge_export(
            turn, 21_000, payload, target,
        ))
        .expect("commit export declaration");
    assert!(matches!(outcome, AdmissionOutcome::Committed { .. }));
    let record = nodes[0]
        .node
        .committed_envelope_record(0)
        .expect("source committed record");
    let mut receipts = vec![nodes[0]
        .node
        .local_bridge_commit_receipt(0)
        .expect("producer bridge receipt")];
    assert!(nodes[0].node.export_bridge(0, &[]).is_err());
    assert!(nodes[0].node.export_bridge(0, &receipts).is_err());

    for (follower_index, &follower_id) in node_ids.iter().enumerate().skip(1) {
        let (producer_slice, follower_slice) = nodes.split_at_mut(follower_index);
        let producer = &mut producer_slice[0].node;
        let follower = &mut follower_slice[0].node;
        let mut producer_transport = producer.quarantine_peer_transport();
        let mut follower_transport = follower.quarantine_peer_transport();
        let (producer_session, follower_session) = complete_handshake(
            producer,
            &producer_transport,
            follower,
            &follower_transport,
            follower_id,
            30_000 + follower_index as u64 * 100,
        );
        producer_transport
            .bind_authenticated_session(producer_session)
            .expect("bind producer session");
        follower_transport
            .bind_authenticated_session(follower_session)
            .expect("bind follower session");
        follower
            .receive_committed_envelope(&mut follower_transport, record.clone())
            .expect("replicate exact declaration envelope");
        receipts.push(
            follower
                .local_bridge_commit_receipt(0)
                .expect("follower bridge receipt"),
        );
        if follower_index == 1 {
            assert!(nodes[0].node.export_bridge(0, &receipts).is_err());
        }
    }

    receipts.reverse();
    let artifact = nodes[0]
        .node
        .export_bridge(0, &receipts)
        .expect("three pinned receipts make exact source bytes exportable");
    assert_eq!(artifact.public_payload(), payload);
    assert_eq!(
        artifact.proof_bundle().source_envelope_bytes(),
        record.envelope_bytes()
    );
    assert_eq!(
        artifact.proof_bundle().source_profile_bytes(),
        bridge_profile.canonical_bytes().unwrap()
    );
    assert_eq!(
        artifact.proof_bundle().source_profile_bytes(),
        profile.canonical_bytes().unwrap(),
        "proof field 2 is the exact canonical source Network Profile"
    );
    assert_eq!(
        artifact.proof_bundle().signed_manifest_bytes(),
        manifest.canonical_bytes()
    );
    assert_eq!(artifact.proof_bundle().receipts().len(), 3);
    let envelope = AdmittedBlockEnvelope::decode(artifact.proof_bundle().source_envelope_bytes())
        .expect("proof carries exact source envelope");
    assert_eq!(envelope.public_provenance, payload);
    assert!(envelope.encrypted_payload.is_none());
    assert!(envelope.privacy_control.is_none());
    assert_eq!(
        artifact.declaration_bytes(),
        envelope.bridge_export_declaration.as_deref().unwrap()
    );
    let proof_bytes = artifact
        .proof_bundle()
        .canonical_bytes()
        .expect("canonical proof bytes");
    let decode_proof = |bytes: &[u8]| {
        BridgeProofBundleV1::decode_verified(
            bytes,
            &bridge_profile,
            &manifest,
            &governance_key.verifying_key(),
        )
    };
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("vectors/bridge_export_v1.json"))
            .expect("parse independent bridge vectors");
    let expected = &vectors["expected"];
    assert_eq!(
        expected["source_profile"],
        hex::encode(artifact.proof_bundle().source_profile_bytes())
    );
    assert_eq!(
        expected["signed_manifest"],
        hex::encode(artifact.proof_bundle().signed_manifest_bytes())
    );
    assert_eq!(
        expected["source_envelope"],
        hex::encode(artifact.proof_bundle().source_envelope_bytes())
    );
    assert_eq!(
        expected["declaration"],
        hex::encode(artifact.declaration_bytes())
    );
    assert_eq!(expected["proof_bundle"], hex::encode(&proof_bytes));
    for (index, receipt) in artifact.proof_bundle().receipts().iter().enumerate() {
        assert_eq!(
            expected["receipts"][index],
            hex::encode(receipt.canonical_bytes().unwrap())
        );
    }
    assert_eq!(
        decode_proof(&proof_bytes).expect("strict trusted proof round trip"),
        *artifact.proof_bundle()
    );
    let reproducibility = artifact.reproducibility();
    assert_eq!(
        reproducibility.transfer_id(),
        artifact.proof_bundle().transfer_id()
    );
    assert_eq!(reproducibility.source_position(), envelope.index);
    assert_eq!(
        reproducibility.declaration_sha256(),
        sha256(artifact.declaration_bytes())
    );
    assert_eq!(reproducibility.public_payload_sha256(), sha256(payload));
    assert_eq!(
        reproducibility.source_profile_sha256(),
        sha256(artifact.proof_bundle().source_profile_bytes())
    );
    assert_eq!(
        reproducibility.signed_manifest_sha256(),
        sha256(artifact.proof_bundle().signed_manifest_bytes())
    );
    assert_eq!(
        reproducibility.source_envelope_sha256(),
        sha256(artifact.proof_bundle().source_envelope_bytes())
    );
    assert_eq!(
        reproducibility.source_envelope_hash(),
        envelope.envelope_hash
    );
    assert_eq!(
        reproducibility.ledger_prefix_hash(),
        artifact.proof_bundle().receipts()[0]
            .core()
            .ledger_prefix_hash()
    );
    assert_eq!(reproducibility.manifest_digest(), manifest.digest());
    assert_eq!(reproducibility.proof_bundle_sha256(), sha256(&proof_bytes));
    let expected_metadata = &expected["reproducibility"];
    assert_eq!(
        expected_metadata["source_position"],
        reproducibility.source_position()
    );
    for (name, actual) in [
        ("declaration_sha256", reproducibility.declaration_sha256()),
        (
            "public_payload_sha256",
            reproducibility.public_payload_sha256(),
        ),
        (
            "source_profile_sha256",
            reproducibility.source_profile_sha256(),
        ),
        (
            "signed_manifest_sha256",
            reproducibility.signed_manifest_sha256(),
        ),
        (
            "source_envelope_sha256",
            reproducibility.source_envelope_sha256(),
        ),
        (
            "source_envelope_hash",
            reproducibility.source_envelope_hash(),
        ),
        ("ledger_prefix_hash", reproducibility.ledger_prefix_hash()),
        ("manifest_digest", reproducibility.manifest_digest()),
        ("proof_bundle_sha256", reproducibility.proof_bundle_sha256()),
    ] {
        assert_eq!(expected_metadata[name], hex::encode(actual), "{name}");
    }
    for private_seed in 11u8..=17 {
        assert!(!proof_bytes
            .windows(32)
            .any(|window| window == [private_seed; 32]));
    }
    for forbidden_text in [b"passphrase".as_slice(), b"private-key", b"dek"] {
        assert!(!proof_bytes
            .windows(forbidden_text.len())
            .any(|window| window == forbidden_text));
    }
    assert_eq!(
        nodes[0]
            .node
            .local_bridge_commit_receipt(0)
            .expect("deterministic receipt retry"),
        receipts[2]
    );
    artifact
        .require_exact_payload(payload)
        .expect("complete exact payload is permitted");
    for forbidden in [
        payload[..payload.len() - 1].to_vec(),
        b"<urn:issue11:batch:2> <urn:issue11:status> \"mapped\" .".to_vec(),
        b"<urn:issue11:batch:2>\n <urn:issue11:status> \"exact-public-copy\" .".to_vec(),
        Vec::new(),
    ] {
        assert!(artifact.require_exact_payload(&forbidden).is_err());
    }

    let canonical_receipts: Vec<_> = artifact.proof_bundle().receipts().to_vec();
    assert!(nodes[0]
        .node
        .export_bridge(0, &canonical_receipts[..2])
        .is_err());
    let mut duplicate = canonical_receipts.clone();
    duplicate[2] = duplicate[1].clone();
    assert!(nodes[0].node.export_bridge(0, &duplicate).is_err());
    let mut extra = canonical_receipts.clone();
    extra.push(canonical_receipts[2].clone());
    assert!(nodes[0].node.export_bridge(0, &extra).is_err());

    let mut bad_signature = canonical_receipts[0].canonical_bytes().unwrap();
    *bad_signature.last_mut().unwrap() ^= 1;
    let mut wrong_receipts = canonical_receipts.clone();
    wrong_receipts[0] = BridgeCommitReceiptV1::decode(&bad_signature)
        .expect("signature mutation preserves canonical receipt shape");
    assert!(nodes[0].node.export_bridge(0, &wrong_receipts).is_err());

    let mut noncanonical_scalar = canonical_receipts[0].signature();
    noncanonical_scalar[32..].copy_from_slice(&[
        0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
    ]);
    let mut small_order_r = canonical_receipts[0].signature();
    small_order_r[..32].fill(0);
    small_order_r[0] = 1;
    for invalid_signature in [noncanonical_scalar, small_order_r] {
        let mut wrong_receipts = canonical_receipts.clone();
        wrong_receipts[0] = receipt_with_signature(&canonical_receipts[0], invalid_signature);
        assert!(nodes[0].node.export_bridge(0, &wrong_receipts).is_err());
    }

    for core_field in [1usize, 2, 3, 5, 6, 7, 8, 9, 10, 11] {
        let mutated = mutate_receipt_core_field(&canonical_receipts[0], core_field);
        let mut wrong_receipts = canonical_receipts.clone();
        wrong_receipts[0] = mutated;
        assert!(
            nodes[0].node.export_bridge(0, &wrong_receipts).is_err(),
            "mutated receipt core field {core_field} must fail closed"
        );
    }

    let mut wrong_magic = proof_bytes.clone();
    wrong_magic[0] ^= 1;
    assert!(decode_proof(&wrong_magic).is_err());
    let mut wrong_count = proof_bytes.clone();
    wrong_count[5] = 4;
    assert!(decode_proof(&wrong_count).is_err());
    let mut duplicate_tag = proof_bytes.clone();
    let second_tag = bridge_field_range(&duplicate_tag, 2).0 - 5;
    duplicate_tag[second_tag] = 1;
    assert!(decode_proof(&duplicate_tag).is_err());
    let mut wrong_source_profile = proof_bytes.clone();
    let profile_range = bridge_field_range(&wrong_source_profile, 2);
    wrong_source_profile[profile_range.1 - 1] ^= 1;
    assert!(decode_proof(&wrong_source_profile).is_err());
    let mut wrong_signed_manifest = proof_bytes.clone();
    let signed_manifest_range = bridge_field_range(&wrong_signed_manifest, 3);
    wrong_signed_manifest[signed_manifest_range.1 - 1] ^= 1;
    assert!(decode_proof(&wrong_signed_manifest).is_err());
    assert!(BridgeProofBundleV1::decode_verified(
        &proof_bytes,
        &bridge_profile,
        &manifest,
        &signing_key(99).verifying_key(),
    )
    .is_err());
    let mut wrong_source_envelope = proof_bytes.clone();
    let envelope_range = bridge_field_range(&wrong_source_envelope, 4);
    wrong_source_envelope[envelope_range.1 - 1] ^= 1;
    assert!(decode_proof(&wrong_source_envelope).is_err());
    let mut trailing = proof_bytes.clone();
    trailing.push(0);
    assert!(decode_proof(&trailing).is_err());
    assert!(decode_proof(&vec![0; MAX_BRIDGE_PROOF_BUNDLE_BYTES + 1]).is_err());

    let unordered = reverse_receipt_sequence(&proof_bytes);
    assert!(decode_proof(&unordered).is_err());
}

#[test]
fn source_bridge_activation_rejects_a_different_live_profile() {
    let governance_key = signing_key(31);
    let identity_keys = [signing_key(32), signing_key(33), signing_key(34)];
    let validator_keys = [signing_key(35), signing_key(36), signing_key(37)];
    let node_ids = [
        Uuid::from_u128(310),
        Uuid::from_u128(320),
        Uuid::from_u128(330),
    ];
    let manifest = signed_manifest(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let mut profile = network_profile(&manifest, &validator_keys);
    let target = BridgeExportTargetV1::new(
        "provchain.issue11.target",
        LedgerInstanceId32::new([0x52; 32]).unwrap(),
    )
    .unwrap();
    let source_ledger = LedgerInstanceId32::new([0x51; 32]).unwrap();
    profile.bridge = Some(
        SourceBridgeProfileV1::new(
            source_ledger.bytes(),
            target.network_id(),
            target.ledger_instance_id().bytes(),
        )
        .unwrap(),
    );
    let bridge_profile =
        BridgeSourceProfileV1::from_network_profile(&profile, &manifest, source_ledger, target)
            .unwrap();
    let data_dir = tempdir().unwrap();
    let ledger_profile = bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID))
        .with_bridge_source_profile(bridge_profile.clone());
    assert!(ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile.clone(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        signing_key(99).verifying_key(),
        NodeIdentity::new(node_ids[0], MemberRole::Validator, identity_keys[0].clone(),),
        validator_keys[0].clone(),
    )
    .is_err());
    assert!(!data_dir.path().join("ledger.journal").exists());

    let mut changed_profile = profile;
    changed_profile.consensus.block_interval += 1;
    assert!(ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        changed_profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_ids[0], MemberRole::Validator, identity_keys[0].clone()),
        validator_keys[0].clone(),
    )
    .is_err());
    assert!(!data_dir.path().join("ledger.journal").exists());
}

#[test]
fn closed_bridge_codec_rejects_tag_count_order_transfer_scheme_and_bound_mutations() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("vectors/bridge_export_v1.json")).unwrap();
    let declaration = hex::decode(vectors["expected"]["declaration"].as_str().unwrap()).unwrap();
    let decoded = BridgeExportDeclarationV1::decode(&declaration).unwrap();
    assert_eq!(decoded.canonical_bytes().unwrap(), declaration);

    let mut mutations = Vec::new();
    let mut wrong_magic = declaration.clone();
    wrong_magic[0] ^= 1;
    mutations.push(wrong_magic);
    let mut wrong_tag = declaration.clone();
    wrong_tag[4] = 0x7f;
    mutations.push(wrong_tag);
    let mut wrong_count = declaration.clone();
    wrong_count[5] = 3;
    mutations.push(wrong_count);
    let mut duplicate_first_tag = declaration.clone();
    duplicate_first_tag[6] = 2;
    mutations.push(duplicate_first_tag);
    let mut wrong_transfer_id = declaration.clone();
    let transfer_range = bridge_field_range(&wrong_transfer_id, 2);
    wrong_transfer_id[transfer_range.1 - 1] ^= 1;
    mutations.push(wrong_transfer_id);
    let mut trailing = declaration.clone();
    trailing.push(0);
    mutations.push(trailing);
    mutations.push(vec![0; MAX_BRIDGE_EXPORT_DECLARATION_BYTES + 1]);
    for mutation in mutations {
        assert!(BridgeExportDeclarationV1::decode(&mutation).is_err());
    }

    let receipt = hex::decode(vectors["expected"]["receipts"][0].as_str().unwrap()).unwrap();
    let mut wrong_scheme = receipt.clone();
    let scheme = bridge_field_range(&wrong_scheme, 2);
    wrong_scheme[scheme.0] = 2;
    assert!(BridgeCommitReceiptV1::decode(&wrong_scheme).is_err());
    let mut trailing_receipt = receipt;
    trailing_receipt.push(0);
    assert!(BridgeCommitReceiptV1::decode(&trailing_receipt).is_err());
}

#[cfg(feature = "bridge-conformance")]
#[test]
fn source_declaration_obeys_journal_commit_crash_and_response_loss_boundaries() {
    for failpoint in [
        LedgerCommitFailpoint::BeforeJournalWrite,
        LedgerCommitFailpoint::BeforeJournalFsync,
    ] {
        let (_data_dir, mut ledger, _profile, target, validator_key) = bridge_conformance_ledger();
        let candidate = ledger
            .create_bridge_export_candidate(
                b"<urn:issue11:failure> <urn:issue11:phase> \"pre-commit\" .",
                target,
                41_000,
                &validator_key,
            )
            .unwrap();
        ledger.inject_next_commit_failpoint(failpoint);
        assert!(ledger
            .conformance_admit_bridge_candidate(candidate.clone())
            .is_err());
        assert_eq!(ledger.journal_frame_count().unwrap(), 0);
        assert!(!ledger.is_degraded());
        assert!(matches!(
            ledger
                .conformance_admit_bridge_candidate(candidate)
                .expect("exact retry can commit after known pre-commit failure"),
            AdmissionOutcome::Committed { .. }
        ));
        assert_eq!(ledger.journal_frame_count().unwrap(), 1);
    }

    let (data_dir, mut ledger, profile, target, validator_key) = bridge_conformance_ledger();
    let candidate = ledger
        .create_bridge_export_candidate(
            b"<urn:issue11:failure> <urn:issue11:phase> \"projection\" .",
            target,
            42_000,
            &validator_key,
        )
        .unwrap();
    ledger.inject_next_commit_failpoint(LedgerCommitFailpoint::AfterJournalCommitBeforeProjection);
    assert!(ledger
        .conformance_admit_bridge_candidate(candidate)
        .is_err());
    assert_eq!(ledger.journal_frame_count().unwrap(), 1);
    assert!(ledger.is_degraded());
    drop(ledger);
    let recovered = Ledger::open_in_dir(data_dir.path(), profile, semantic_package())
        .expect("verified replay rebuilds declaration projection after committed crash");
    assert_eq!(recovered.journal_frame_count().unwrap(), 1);
    assert!(recovered.committed_envelopes()[0]
        .bridge_export_declaration
        .is_some());

    let (_data_dir, mut ledger, _profile, target, validator_key) = bridge_conformance_ledger();
    let candidate = ledger
        .create_bridge_export_candidate(
            b"<urn:issue11:failure> <urn:issue11:phase> \"response-loss\" .",
            target,
            43_000,
            &validator_key,
        )
        .unwrap();
    ledger.inject_next_commit_failpoint(LedgerCommitFailpoint::AfterProjectionBeforeResponse);
    assert!(ledger
        .conformance_admit_bridge_candidate(candidate.clone())
        .is_err());
    assert_eq!(ledger.journal_frame_count().unwrap(), 1);
    assert!(!ledger.is_degraded());
    assert!(matches!(
        ledger
            .conformance_admit_bridge_candidate(candidate)
            .expect("exact response-loss retry is idempotent"),
        AdmissionOutcome::Committed { .. }
    ));
    assert_eq!(ledger.journal_frame_count().unwrap(), 1);
}

#[cfg(feature = "bridge-conformance")]
fn bridge_conformance_ledger() -> (
    TempDir,
    Ledger,
    LedgerProfile,
    BridgeExportTargetV1,
    SigningKey,
) {
    let governance_key = signing_key(41);
    let identity_keys = [signing_key(42), signing_key(43), signing_key(44)];
    let validator_keys = [signing_key(45), signing_key(46), signing_key(47)];
    let node_ids = [
        Uuid::from_u128(410),
        Uuid::from_u128(420),
        Uuid::from_u128(430),
    ];
    let manifest = signed_manifest(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let mut network = network_profile(&manifest, &validator_keys);
    let target = BridgeExportTargetV1::new(
        "provchain.issue11.target",
        LedgerInstanceId32::new([0x62; 32]).unwrap(),
    )
    .unwrap();
    let source_ledger = LedgerInstanceId32::new([0x61; 32]).unwrap();
    network.bridge = Some(
        SourceBridgeProfileV1::new(
            source_ledger.bytes(),
            target.network_id(),
            target.ledger_instance_id().bytes(),
        )
        .unwrap(),
    );
    let source = BridgeSourceProfileV1::from_network_profile(
        &network,
        &manifest,
        source_ledger,
        target.clone(),
    )
    .unwrap();
    let profile =
        bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID)).with_bridge_source_profile(source);
    let data_dir = tempdir().unwrap();
    let ledger = Ledger::open_in_dir(data_dir.path(), profile.clone(), semantic_package()).unwrap();
    (data_dir, ledger, profile, target, validator_keys[0].clone())
}

fn bridge_field_range(record: &[u8], field_number: usize) -> (usize, usize) {
    assert_eq!(&record[..4], b"BCV1");
    let mut offset = 6usize;
    for number in 1..=usize::from(record[5]) {
        assert_eq!(usize::from(record[offset]), number);
        let length =
            u32::from_be_bytes(record[offset + 1..offset + 5].try_into().unwrap()) as usize;
        let start = offset + 5;
        let end = start + length;
        if number == field_number {
            return (start, end);
        }
        offset = end;
    }
    panic!("missing bridge field {field_number}");
}

fn mutate_receipt_core_field(
    receipt: &BridgeCommitReceiptV1,
    field_number: usize,
) -> BridgeCommitReceiptV1 {
    let mut bytes = receipt.canonical_bytes().unwrap();
    let (core_start, core_end) = bridge_field_range(&bytes, 1);
    let (field_start, field_end) = bridge_field_range(&bytes[core_start..core_end], field_number);
    let absolute_last = core_start + field_end - 1;
    bytes[absolute_last] = match bytes[absolute_last] {
        b'a'..=b'y' => bytes[absolute_last] + 1,
        b'z' => b'y',
        b'0'..=b'8' => bytes[absolute_last] + 1,
        b'9' => b'8',
        value => value ^ 1,
    };
    assert!(field_start < field_end);
    BridgeCommitReceiptV1::decode(&bytes).expect("field mutation preserves canonical shape")
}

fn receipt_with_signature(
    receipt: &BridgeCommitReceiptV1,
    signature: [u8; 64],
) -> BridgeCommitReceiptV1 {
    let mut bytes = receipt.canonical_bytes().unwrap();
    let (start, end) = bridge_field_range(&bytes, 3);
    assert_eq!(end - start, 64);
    bytes[start..end].copy_from_slice(&signature);
    BridgeCommitReceiptV1::decode(&bytes).expect("raw invalid signature remains codec-canonical")
}

fn reverse_receipt_sequence(proof: &[u8]) -> Vec<u8> {
    let mut proof = proof.to_vec();
    let (start, end) = bridge_field_range(&proof, 5);
    let sequence = &proof[start..end];
    let mut offset = 0usize;
    let mut frames = Vec::new();
    while offset < sequence.len() {
        let length = u32::from_be_bytes(sequence[offset..offset + 4].try_into().unwrap()) as usize;
        let frame_end = offset + 4 + length;
        frames.push(sequence[offset..frame_end].to_vec());
        offset = frame_end;
    }
    frames.reverse();
    let reversed: Vec<u8> = frames.into_iter().flatten().collect();
    proof[start..end].copy_from_slice(&reversed);
    proof
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).into()
}
