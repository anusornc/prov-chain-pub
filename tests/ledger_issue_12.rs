//! Acceptance tests for target admission of bounded bridge proofs (Issue #12).

use std::path::Path;

use ed25519_dalek::SigningKey;
use provchain_org::bridge::{
    BridgeImportOutcomeV1, BridgeOriginEvidenceV1, BridgeSourceTrustProfileV1,
    BridgeTargetProfileV1, MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
};
#[cfg(feature = "bridge-conformance")]
use provchain_org::ledger::LedgerCommitFailpoint;
use provchain_org::ledger::{AdmittedBlockEnvelope, LedgerProfile};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::peer_session::{AuthenticatedPeerSession, PeerTransport};
use provchain_org::network::profile::{
    ConsensusProfile, NetworkProfile, SemanticProfile, SourceBridgeProfileV1,
};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use serde_json::Value;
use tempfile::tempdir;
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const SOURCE_NETWORK_ID: &str = "provchain.issue11.source";
const SOURCE_PROFILE_ID: &str = "issue11.source.reference";
const TARGET_NETWORK_ID: &str = "provchain.issue11.target";
const TARGET_PROFILE_ID: &str = "issue12.target.reference";
const PUBLIC_PAYLOAD: &[u8] = b"<urn:issue11:batch:2> <urn:issue11:status> \"exact-public-copy\" .";

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

fn manifest(
    governance_key: &SigningKey,
    contract: (&str, u64, &str, &str),
    node_ids: &[Uuid; 3],
    identity_keys: &[SigningKey; 3],
    validator_keys: &[SigningKey; 3],
) -> SignedMembershipManifest {
    let (manifest_id, version, network_id, profile_id) = contract;
    MembershipManifest {
        manifest_id: manifest_id.to_string(),
        version,
        network_id: network_id.to_string(),
        network_profile_id: profile_id.to_string(),
        members: node_ids
            .iter()
            .zip(identity_keys)
            .zip(validator_keys)
            .map(|((&node_id, identity_key), validator_key)| {
                member(node_id, identity_key, validator_key)
            })
            .collect(),
    }
    .sign(governance_key)
    .expect("sign membership manifest")
}

fn base_profile(
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    validator_keys: &[SigningKey; 3],
) -> NetworkProfile {
    let package = semantic_package();
    NetworkProfile {
        profile_id: profile_id.to_string(),
        network_id: network_id.to_string(),
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
        privacy: None,
        bridge: None,
        bridge_source_trust: None,
    }
}

struct SourceContract {
    governance_key: SigningKey,
    manifest: SignedMembershipManifest,
    profile: NetworkProfile,
}

fn source_contract() -> SourceContract {
    let governance_key = signing_key(11);
    let identity_keys = [signing_key(12), signing_key(13), signing_key(14)];
    let validator_keys = [signing_key(15), signing_key(16), signing_key(17)];
    let node_ids = [
        Uuid::from_u128(110),
        Uuid::from_u128(120),
        Uuid::from_u128(130),
    ];
    let manifest = manifest(
        &governance_key,
        (
            "issue11.source.membership",
            11,
            SOURCE_NETWORK_ID,
            SOURCE_PROFILE_ID,
        ),
        &node_ids,
        &identity_keys,
        &validator_keys,
    );
    let mut profile = base_profile(
        SOURCE_NETWORK_ID,
        SOURCE_PROFILE_ID,
        &manifest,
        &validator_keys,
    );
    profile.bridge = Some(
        SourceBridgeProfileV1::new([0x31; 32], TARGET_NETWORK_ID, [0x42; 32])
            .expect("source outbound bridge rule"),
    );
    SourceContract {
        governance_key,
        manifest,
        profile,
    }
}

struct TargetContract {
    governance_key: SigningKey,
    manifest: SignedMembershipManifest,
    profile: NetworkProfile,
    identity_keys: [SigningKey; 3],
    validator_keys: [SigningKey; 3],
    node_ids: [Uuid; 3],
    bridge: BridgeTargetProfileV1,
}

fn target_contract(source: &SourceContract) -> TargetContract {
    let governance_key = signing_key(21);
    let identity_keys = [signing_key(22), signing_key(23), signing_key(24)];
    let validator_keys = [signing_key(25), signing_key(26), signing_key(27)];
    let node_ids = [
        Uuid::from_u128(210),
        Uuid::from_u128(220),
        Uuid::from_u128(230),
    ];
    let manifest = manifest(
        &governance_key,
        (
            "issue12.target.membership",
            12,
            TARGET_NETWORK_ID,
            TARGET_PROFILE_ID,
        ),
        &node_ids,
        &identity_keys,
        &validator_keys,
    );
    let mut profile = base_profile(
        TARGET_NETWORK_ID,
        TARGET_PROFILE_ID,
        &manifest,
        &validator_keys,
    );
    profile.bridge_source_trust = Some(
        BridgeSourceTrustProfileV1::new(
            TARGET_NETWORK_ID,
            [0x42; 32],
            &source.profile,
            &source.manifest,
            &source.governance_key.verifying_key(),
        )
        .expect("target-pinned source trust"),
    );
    let bridge = BridgeTargetProfileV1::activate(
        &profile,
        &source.profile,
        &source.manifest,
        source.governance_key.verifying_key(),
    )
    .expect("activate target bridge verifier");
    TargetContract {
        governance_key,
        manifest,
        profile,
        identity_keys,
        validator_keys,
        node_ids,
        bridge,
    }
}

fn start_target_node(data_dir: &Path, target: &TargetContract) -> ReferenceNode {
    start_target_node_for_member(data_dir, target, 0)
}

fn start_target_node_for_member(
    data_dir: &Path,
    target: &TargetContract,
    member_index: usize,
) -> ReferenceNode {
    let ledger_profile = bind_profile(LedgerProfile::new(TARGET_NETWORK_ID, TARGET_PROFILE_ID))
        .with_bridge_target_profile(target.bridge.clone());
    ReferenceNode::start_validator_in_dir(
        data_dir,
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        target.profile.clone(),
        target.manifest.clone(),
        target.governance_key.verifying_key(),
        NodeIdentity::new(
            target.node_ids[member_index],
            MemberRole::Validator,
            target.identity_keys[member_index].clone(),
        ),
        target.validator_keys[member_index].clone(),
    )
    .expect("start target bridge node")
}

fn start_target_observer(data_dir: &Path, target: &TargetContract) -> ReferenceNode {
    let ledger_profile = bind_profile(LedgerProfile::new(TARGET_NETWORK_ID, TARGET_PROFILE_ID))
        .with_bridge_target_profile(target.bridge.clone());
    ReferenceNode::start_in_dir(
        data_dir,
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        target.profile.clone(),
        target.manifest.clone(),
        target.governance_key.verifying_key(),
        NodeIdentity::new(
            target.node_ids[0],
            MemberRole::Validator,
            target.identity_keys[0].clone(),
        ),
    )
    .expect("start target observer without proposal key")
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
        .unwrap();
    let challenge = responder
        .accept_peer_handshake_at(responder_transport, hello, started_at_millis + 1)
        .unwrap();
    let (response, initiator_session) = initiator
        .answer_peer_challenge_at(initiator_transport, challenge, started_at_millis + 2)
        .unwrap();
    let responder_session = responder
        .finish_peer_handshake_at(responder_transport, response, started_at_millis + 3)
        .unwrap();
    (initiator_session, responder_session)
}

fn vector_proof() -> Vec<u8> {
    let vector: Value = serde_json::from_str(include_str!("vectors/bridge_export_v1.json"))
        .expect("parse source proof vector");
    hex::decode(
        vector["expected"]["proof_bundle"]
            .as_str()
            .expect("proof_bundle hex"),
    )
    .expect("decode proof bundle")
}

#[test]
fn valid_vector_import_commits_once_and_replay_returns_already_imported() {
    let source = source_contract();
    let target = target_contract(&source);
    let data_dir = tempdir().expect("temporary target ledger");
    let proof = vector_proof();

    let node = start_target_node(data_dir.path(), &target);
    let imported = node
        .import_bridge(&proof, PUBLIC_PAYLOAD, 120_000)
        .expect("valid source proof enters target PoA and Final Admission");
    let BridgeImportOutcomeV1::Imported { reference } = imported else {
        panic!("first valid import did not commit");
    };
    assert_eq!(reference.source_network_id(), SOURCE_NETWORK_ID);
    assert_eq!(reference.source_ledger_instance_id(), [0x31; 32]);
    assert_eq!(reference.source_position(), 0);
    assert_ne!(reference.source_envelope_hash(), [0; 32]);
    assert_ne!(reference.source_ledger_prefix_hash(), [0; 32]);
    assert_eq!(reference.target_network_id(), TARGET_NETWORK_ID);
    assert_eq!(reference.target_ledger_instance_id(), [0x42; 32]);
    assert_eq!(reference.target_position(), 0);
    assert_ne!(reference.target_envelope_hash(), [0; 32]);
    assert_ne!(reference.target_ledger_prefix_hash(), [0; 32]);
    assert_ne!(reference.proof_hash(), [0; 32]);
    assert_ne!(reference.payload_hash(), [0; 32]);
    assert_eq!(node.journal_frame_count().unwrap(), 1);
    let record = node.committed_envelope_record(0).unwrap();
    let envelope = AdmittedBlockEnvelope::decode(record.envelope_bytes()).unwrap();
    assert_eq!(envelope.public_provenance, PUBLIC_PAYLOAD);
    assert!(envelope.bridge_origin_evidence.is_some());
    drop(node);

    let restarted = start_target_observer(data_dir.path(), &target);
    let duplicate = restarted
        .import_bridge(&proof, PUBLIC_PAYLOAD, 130_000)
        .expect("exact retry is derived from target journal replay");
    let BridgeImportOutcomeV1::AlreadyImported {
        reference: duplicate_reference,
    } = duplicate
    else {
        panic!("exact retry was not classified AlreadyImported");
    };
    assert_eq!(duplicate_reference, reference);
    assert_eq!(restarted.journal_frame_count().unwrap(), 1);
    assert_eq!(restarted.effective_bridge_state().unwrap().len(), 1);
}

#[test]
fn invalid_evidence_consumes_no_id_and_changed_same_id_is_replay_conflict() {
    let source = source_contract();
    let target = target_contract(&source);
    let data_dir = tempdir().unwrap();
    let proof = vector_proof();
    let mut invalid = proof.clone();
    *invalid.last_mut().unwrap() ^= 1;
    let node = start_target_node(data_dir.path(), &target);

    assert!(node
        .import_bridge(&invalid, PUBLIC_PAYLOAD, 120_000)
        .is_err());
    assert_eq!(node.journal_frame_count().unwrap(), 0);
    assert!(node.effective_bridge_state().unwrap().is_empty());

    assert!(node
        .import_bridge(&proof, b"<urn:wrong> <urn:value> \"changed\" .", 120_000)
        .is_err());
    assert_eq!(node.journal_frame_count().unwrap(), 0);

    assert!(matches!(
        node.import_bridge(&proof, PUBLIC_PAYLOAD, 120_000).unwrap(),
        BridgeImportOutcomeV1::Imported { .. }
    ));
    assert!(matches!(
        node.import_bridge(&invalid, PUBLIC_PAYLOAD, 130_000)
            .unwrap(),
        BridgeImportOutcomeV1::ReplayConflict
    ));
    assert!(matches!(
        node.import_bridge(
            &proof,
            b"<urn:issue11:batch:2> <urn:issue11:status> \"changed-copy\" .",
            130_000,
        )
        .unwrap(),
        BridgeImportOutcomeV1::ReplayConflict
    ));
    assert_eq!(node.journal_frame_count().unwrap(), 1);
    assert_eq!(node.effective_bridge_state().unwrap().len(), 1);
}

#[test]
fn target_activation_rejects_source_or_semantic_trust_substitution() {
    let source = source_contract();
    let target = target_contract(&source);

    let mut substituted_source = source.profile.clone();
    substituted_source.consensus.authority_keys.swap(0, 1);
    assert!(BridgeTargetProfileV1::activate(
        &target.profile,
        &substituted_source,
        &source.manifest,
        source.governance_key.verifying_key(),
    )
    .is_err());

    let mut substituted_target = target.profile.clone();
    substituted_target.semantic.ontology_package_hash = "ab".repeat(32);
    assert!(BridgeTargetProfileV1::activate(
        &substituted_target,
        &source.profile,
        &source.manifest,
        source.governance_key.verifying_key(),
    )
    .is_err());

    assert!(BridgeTargetProfileV1::activate(
        &target.profile,
        &source.profile,
        &source.manifest,
        signing_key(99).verifying_key(),
    )
    .is_err());
}

#[test]
fn active_target_trust_substitution_and_out_of_turn_signer_fail_closed() {
    let source = source_contract();
    let target = target_contract(&source);

    let mut substituted_profile = target.profile.clone();
    let mut trust = serde_json::to_value(
        substituted_profile
            .bridge_source_trust
            .as_ref()
            .expect("target trust"),
    )
    .unwrap();
    trust["source_profile_id"] = Value::String("substituted.source.profile".to_string());
    substituted_profile.bridge_source_trust = Some(serde_json::from_value(trust).unwrap());
    substituted_profile
        .validate()
        .expect("substituted trust remains structurally valid");
    let data_dir = tempdir().unwrap();
    let ledger_profile = bind_profile(LedgerProfile::new(TARGET_NETWORK_ID, TARGET_PROFILE_ID))
        .with_bridge_target_profile(target.bridge.clone());
    assert!(ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        substituted_profile,
        target.manifest.clone(),
        target.governance_key.verifying_key(),
        NodeIdentity::new(
            target.node_ids[0],
            MemberRole::Validator,
            target.identity_keys[0].clone(),
        ),
        target.validator_keys[0].clone(),
    )
    .is_err());

    let out_of_turn_dir = tempdir().unwrap();
    let out_of_turn = start_target_node_for_member(out_of_turn_dir.path(), &target, 1);
    assert!(out_of_turn
        .import_bridge(&vector_proof(), PUBLIC_PAYLOAD, 120_000)
        .is_err());
    assert_eq!(out_of_turn.journal_frame_count().unwrap(), 0);
    assert!(out_of_turn.effective_bridge_state().unwrap().is_empty());
}

#[test]
fn target_origin_codec_rejects_noncanonical_and_oversized_evidence() {
    let source = source_contract();
    let target = target_contract(&source);
    let data_dir = tempdir().unwrap();
    let node = start_target_node(data_dir.path(), &target);
    node.import_bridge(&vector_proof(), PUBLIC_PAYLOAD, 120_000)
        .unwrap();
    let envelope =
        AdmittedBlockEnvelope::decode(node.committed_envelope_record(0).unwrap().envelope_bytes())
            .unwrap();
    let origin = envelope.bridge_origin_evidence.unwrap();
    assert!(BridgeOriginEvidenceV1::decode(&origin).is_ok());

    let mut mutations = Vec::new();
    let mut wrong_magic = origin.clone();
    wrong_magic[0] ^= 1;
    mutations.push(wrong_magic);
    let mut wrong_tag = origin.clone();
    wrong_tag[4] ^= 1;
    mutations.push(wrong_tag);
    let mut wrong_count = origin.clone();
    wrong_count[5] = 2;
    mutations.push(wrong_count);
    let mut zero_transfer = origin.clone();
    zero_transfer[11..43].fill(0);
    mutations.push(zero_transfer);
    let mut wrong_hash = origin.clone();
    wrong_hash[48] ^= 1;
    mutations.push(wrong_hash);
    let mut trailing = origin;
    trailing.push(0);
    mutations.push(trailing);
    mutations.push(vec![0; MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES + 1]);

    for mutation in mutations {
        assert!(BridgeOriginEvidenceV1::decode(&mutation).is_err());
    }
}

#[test]
fn exact_target_envelope_replication_converges_bridge_state_after_partition() {
    let source = source_contract();
    let target = target_contract(&source);
    let directories = [tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap()];
    let mut nodes = directories
        .iter()
        .enumerate()
        .map(|(index, directory)| start_target_node_for_member(directory.path(), &target, index))
        .collect::<Vec<_>>();
    let proof = vector_proof();

    assert!(matches!(
        nodes[0]
            .import_bridge(&proof, PUBLIC_PAYLOAD, 120_000)
            .unwrap(),
        BridgeImportOutcomeV1::Imported { .. }
    ));
    let record = nodes[0].committed_envelope_record(0).unwrap();

    for follower_index in 1..3 {
        let (producer_slice, follower_slice) = nodes.split_at_mut(follower_index);
        let producer = &mut producer_slice[0];
        let follower = &mut follower_slice[0];
        let mut producer_transport = producer.quarantine_peer_transport();
        let mut follower_transport = follower.quarantine_peer_transport();
        let (producer_session, follower_session) = complete_handshake(
            producer,
            &producer_transport,
            follower,
            &follower_transport,
            target.node_ids[follower_index],
            200_000 + follower_index as u64 * 100,
        );
        producer_transport
            .bind_authenticated_session(producer_session)
            .unwrap();
        follower_transport
            .bind_authenticated_session(follower_session)
            .unwrap();
        follower
            .receive_committed_envelope(&mut follower_transport, record.clone())
            .unwrap();
    }

    let states = nodes
        .iter()
        .map(|node| node.effective_bridge_state().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states[0], states[1]);
    assert_eq!(states[1], states[2]);
    for node in &nodes {
        assert_eq!(node.committed_envelope_record(0).unwrap(), record);
        assert_eq!(node.journal_frame_count().unwrap(), 1);
    }
}

#[cfg(feature = "bridge-conformance")]
#[test]
fn target_import_obeys_commit_projection_and_response_failure_boundaries() {
    for failpoint in [
        LedgerCommitFailpoint::BeforeJournalWrite,
        LedgerCommitFailpoint::BeforeJournalFsync,
    ] {
        let source = source_contract();
        let target = target_contract(&source);
        let data_dir = tempdir().unwrap();
        let proof = vector_proof();
        let node = start_target_node(data_dir.path(), &target);
        node.inject_next_bridge_commit_failpoint(failpoint).unwrap();
        assert!(node.import_bridge(&proof, PUBLIC_PAYLOAD, 120_000).is_err());
        assert_eq!(node.journal_frame_count().unwrap(), 0);
        assert!(matches!(
            node.import_bridge(&proof, PUBLIC_PAYLOAD, 120_000).unwrap(),
            BridgeImportOutcomeV1::Imported { .. }
        ));
        assert_eq!(node.journal_frame_count().unwrap(), 1);
    }

    let source = source_contract();
    let target = target_contract(&source);
    let data_dir = tempdir().unwrap();
    let proof = vector_proof();
    let node = start_target_node(data_dir.path(), &target);
    node.inject_next_bridge_commit_failpoint(
        LedgerCommitFailpoint::AfterJournalCommitBeforeProjection,
    )
    .unwrap();
    assert!(node.import_bridge(&proof, PUBLIC_PAYLOAD, 120_000).is_err());
    assert_eq!(node.journal_frame_count().unwrap(), 1);
    drop(node);
    let recovered = start_target_node(data_dir.path(), &target);
    assert!(matches!(
        recovered
            .import_bridge(&proof, PUBLIC_PAYLOAD, 130_000)
            .unwrap(),
        BridgeImportOutcomeV1::AlreadyImported { .. }
    ));

    let source = source_contract();
    let target = target_contract(&source);
    let data_dir = tempdir().unwrap();
    let proof = vector_proof();
    let node = start_target_node(data_dir.path(), &target);
    node.inject_next_bridge_commit_failpoint(LedgerCommitFailpoint::AfterProjectionBeforeResponse)
        .unwrap();
    assert!(node.import_bridge(&proof, PUBLIC_PAYLOAD, 120_000).is_err());
    assert!(matches!(
        node.import_bridge(&proof, PUBLIC_PAYLOAD, 130_000).unwrap(),
        BridgeImportOutcomeV1::AlreadyImported { .. }
    ));
    assert_eq!(node.journal_frame_count().unwrap(), 1);
}
