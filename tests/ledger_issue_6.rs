//! Acceptance tests for exact-envelope replication and convergence evidence (Issue #6).

use ed25519_dalek::SigningKey;
use provchain_org::ledger::{AdmissionOutcome, LedgerProfile};
use provchain_org::network::convergence::{
    CommitReceipt, CommitStatus, CommittedEnvelopeRange, CommittedEnvelopeRecord,
    CommittedRangeRequest, ConvergenceError, LedgerPrefixCheckpoint, MAX_COMMITTED_RANGE_RECORDS,
};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::peer_session::{
    AuthenticatedPeerSession, PeerTransport, PeerTransportState,
};
use provchain_org::network::poa::{PoAError, PoAProposalRequest};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue6";
const PROFILE_ID: &str = "issue6.reference";

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
        manifest_id: "issue6.membership".to_string(),
        version: 9,
        network_id: NETWORK_ID.to_string(),
        network_profile_id: PROFILE_ID.to_string(),
        members,
    }
    .sign(governance_key)
    .expect("sign membership manifest")
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

fn ledger_profile() -> LedgerProfile {
    bind_profile(LedgerProfile::new(NETWORK_ID, PROFILE_ID))
}

struct TestNode {
    _data_dir: TempDir,
    node: ReferenceNode,
}

fn start_validator_node(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> TestNode {
    let data_dir = tempdir().expect("temporary validator directory");
    let node = start_validator_at(
        data_dir.path(),
        governance_key,
        manifest,
        profile,
        node_id,
        identity_key,
        validator_key,
    );
    TestNode {
        _data_dir: data_dir,
        node,
    }
}

fn start_validator_at(
    data_dir: &Path,
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> ReferenceNode {
    ReferenceNode::start_validator_in_dir(
        data_dir,
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
        validator_key.clone(),
    )
    .expect("activate reference validator")
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
        .expect("authenticate responder and answer challenge");
    let responder_session = responder
        .finish_peer_handshake_at(responder_transport, response, started_at_millis + 3)
        .expect("authenticate initiator response");
    (initiator_session, responder_session)
}

fn committed(outcome: AdmissionOutcome) {
    if let AdmissionOutcome::Rejected { reason } = outcome {
        panic!("proposal was rejected: {reason}");
    }
}

#[test]
fn follower_final_admits_the_exact_producer_envelope_and_receipts_follow_local_fsync() {
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
    let profile = network_profile(&manifest, &validator_keys);
    let mut producer = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    let mut follower = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_ids[1],
        &identity_keys[1],
        &validator_keys[1],
    );

    let turn = producer.node.pending_poa_turn().expect("producer turn");
    committed(
        producer
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn,
                10_000,
                b"<urn:issue6:block:0> <urn:issue6:status> \"committed\" .",
            ))
            .expect("scheduled producer commits"),
    );
    let producer_record = producer
        .node
        .committed_envelope_record(0)
        .expect("read exact committed producer record");
    let producer_receipt = producer
        .node
        .local_commit_receipt(0)
        .expect("receipt is available only for a durable local frame");
    assert_eq!(producer_receipt.node_id(), node_ids[0]);
    assert_eq!(
        producer_receipt.envelope_hash(),
        producer_record.envelope_hash()
    );
    assert_eq!(
        producer_receipt.ledger_prefix_hash(),
        producer_record.ledger_prefix_hash()
    );
    assert_eq!(
        CommitReceipt::decode(&producer_receipt.canonical_bytes())
            .expect("decode canonical receipt"),
        producer_receipt
    );
    assert!(matches!(
        producer
            .node
            .commit_status(0, std::slice::from_ref(&producer_receipt))
            .expect("local commit status"),
        CommitStatus::LocalCommitted {
            matching_receipts: 1,
            required_receipts: 3,
        }
    ));
    assert!(matches!(
        producer
            .node
            .commit_status(
                0,
                &[producer_receipt.clone(), producer_receipt.clone()]
            )
            .expect_err("duplicate node evidence cannot advance convergence"),
        ConvergenceError::DuplicateReceipt(node_id) if node_id == node_ids[0]
    ));

    let mut producer_transport = producer.node.quarantine_peer_transport();
    let mut follower_transport = follower.node.quarantine_peer_transport();
    let mut quarantined_transport = follower.node.quarantine_peer_transport();
    assert!(follower
        .node
        .receive_committed_envelope(&mut quarantined_transport, producer_record.clone())
        .is_err());
    assert_eq!(quarantined_transport.state(), PeerTransportState::Closed);
    assert_eq!(
        follower
            .node
            .journal_frame_count()
            .expect("follower frames"),
        0
    );
    let (producer_session, follower_session) = complete_handshake(
        &mut producer.node,
        &producer_transport,
        &mut follower.node,
        &follower_transport,
        node_ids[1],
        20_000,
    );
    producer_transport
        .bind_authenticated_session(producer_session)
        .expect("bind producer session");
    follower_transport
        .bind_authenticated_session(follower_session)
        .expect("bind follower session");

    let follower_receipt = follower
        .node
        .receive_committed_envelope(&mut follower_transport, producer_record.clone())
        .expect("follower verifies and final-admits exact producer envelope");
    let follower_record = follower
        .node
        .committed_envelope_record(0)
        .expect("read follower's durable record");

    assert_eq!(follower_record, producer_record);
    assert_eq!(
        follower
            .node
            .journal_frame_count()
            .expect("follower frames"),
        1
    );
    assert_eq!(follower_receipt.node_id(), node_ids[1]);
    assert_eq!(
        follower_receipt.envelope_hash(),
        producer_record.envelope_hash()
    );
    assert_eq!(
        follower_receipt.ledger_prefix_hash(),
        producer_record.ledger_prefix_hash()
    );
}

#[test]
fn three_identity_signed_matching_receipts_are_evidence_of_network_convergence() {
    let governance_key = signing_key(30);
    let identity_keys = [signing_key(31), signing_key(32), signing_key(33)];
    let validator_keys = [signing_key(34), signing_key(35), signing_key(36)];
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
    let profile = network_profile(&manifest, &validator_keys);
    let mut nodes: Vec<_> = (0..3)
        .map(|index| {
            start_validator_node(
                &governance_key,
                &manifest,
                &profile,
                node_ids[index],
                &identity_keys[index],
                &validator_keys[index],
            )
        })
        .collect();

    let turn = nodes[0].node.pending_poa_turn().expect("height-zero turn");
    committed(
        nodes[0]
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn,
                30_000,
                b"<urn:issue6:batch:7> <urn:issue6:state> \"released\" .",
            ))
            .expect("producer commit"),
    );
    let record = nodes[0]
        .node
        .committed_envelope_record(0)
        .expect("producer record");
    let mut receipts = vec![nodes[0]
        .node
        .local_commit_receipt(0)
        .expect("producer receipt")];

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
            40_000 + follower_index as u64 * 100,
        );
        producer_transport
            .bind_authenticated_session(producer_session)
            .expect("bind producer session");
        follower_transport
            .bind_authenticated_session(follower_session)
            .expect("bind follower session");
        receipts.push(
            follower
                .receive_committed_envelope(&mut follower_transport, record.clone())
                .expect("replicate exact committed record"),
        );
    }

    for node in &nodes {
        match node
            .node
            .commit_status(0, &receipts)
            .expect("evaluate convergence evidence")
        {
            CommitStatus::NetworkConverged { evidence } => {
                assert_eq!(evidence.ledger_position(), 0);
                assert_eq!(evidence.envelope_hash(), record.envelope_hash());
                assert_eq!(evidence.ledger_prefix_hash(), record.ledger_prefix_hash());
                assert_eq!(evidence.receipts().len(), 3);
            }
            status => panic!("expected Network-Converged, got {status:?}"),
        }
    }

    let mut tampered_bytes = receipts[2].canonical_bytes();
    let final_byte = tampered_bytes
        .last_mut()
        .expect("receipt contains a signature byte");
    *final_byte ^= 1;
    let tampered = CommitReceipt::decode(&tampered_bytes).expect("shape remains canonical");
    let mut tampered_receipts = receipts.clone();
    tampered_receipts[2] = tampered;
    assert!(nodes[0].node.commit_status(0, &tampered_receipts).is_err());
}

#[test]
fn partitioned_node_rejoins_with_bounded_exact_prefix_sync_and_retry_is_idempotent() {
    let governance_key = signing_key(60);
    let identity_keys = [signing_key(61), signing_key(62), signing_key(63)];
    let validator_keys = [signing_key(64), signing_key(65), signing_key(66)];
    let node_ids = [
        Uuid::from_u128(610),
        Uuid::from_u128(620),
        Uuid::from_u128(630),
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
    let profile = network_profile(&manifest, &validator_keys);
    let mut node_a = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    let mut node_b = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_ids[1],
        &identity_keys[1],
        &validator_keys[1],
    );
    let mut node_c = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_ids[2],
        &identity_keys[2],
        &validator_keys[2],
    );

    let mut a_to_b = node_a.node.quarantine_peer_transport();
    let mut b_to_a = node_b.node.quarantine_peer_transport();
    let (a_b_session, b_a_session) = complete_handshake(
        &mut node_a.node,
        &a_to_b,
        &mut node_b.node,
        &b_to_a,
        node_ids[1],
        70_000,
    );
    a_to_b
        .bind_authenticated_session(a_b_session)
        .expect("bind A-to-B session");
    b_to_a
        .bind_authenticated_session(b_a_session)
        .expect("bind B-to-A session");

    let mut a_to_c = node_a.node.quarantine_peer_transport();
    let mut c_to_a = node_c.node.quarantine_peer_transport();
    let (a_c_session, c_a_session) = complete_handshake(
        &mut node_a.node,
        &a_to_c,
        &mut node_c.node,
        &c_to_a,
        node_ids[2],
        71_000,
    );
    a_to_c
        .bind_authenticated_session(a_c_session)
        .expect("bind A-to-C session");
    c_to_a
        .bind_authenticated_session(c_a_session)
        .expect("bind C-to-A session");

    let turn_zero = node_a.node.pending_poa_turn().expect("height-zero turn");
    committed(
        node_a
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn_zero,
                100_000,
                b"<urn:issue6:block:0> <urn:issue6:value> \"zero\" .",
            ))
            .expect("A commits height zero"),
    );
    let record_zero = node_a
        .node
        .committed_envelope_record(0)
        .expect("height-zero record");
    node_b
        .node
        .receive_committed_envelope(&mut b_to_a, record_zero)
        .expect("B receives height zero while C is partitioned");

    let turn_one = node_b.node.pending_poa_turn().expect("height-one turn");
    committed(
        node_b
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn_one,
                110_000,
                b"<urn:issue6:block:1> <urn:issue6:value> \"one\" .",
            ))
            .expect("B commits height one"),
    );
    let record_one = node_b
        .node
        .committed_envelope_record(1)
        .expect("height-one record");
    node_a
        .node
        .receive_committed_envelope(&mut a_to_b, record_one.clone())
        .expect("A receives height one while C remains partitioned");
    assert_eq!(node_c.node.journal_frame_count().expect("C frames"), 0);
    let stale_local_turn = node_c
        .node
        .pending_poa_turn()
        .expect("partitioned C still sees height zero");
    assert!(matches!(
        node_c
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                stale_local_turn,
                115_000,
                b"<urn:issue6:missed> <urn:issue6:value> \"no-takeover\" .",
            ))
            .expect_err("partitioned C cannot take over a missed scheduled turn"),
        PoAError::OutOfTurn { height: 0, .. }
    ));
    assert!(matches!(
        node_c
            .node
            .receive_committed_envelope(&mut c_to_a, record_one)
            .expect_err("a gap cannot be filled by accepting a later record first"),
        ConvergenceError::Gap {
            expected_index: 0,
            received_index: 1,
        }
    ));
    assert_eq!(node_c.node.journal_frame_count().expect("C frames"), 0);

    let checkpoint = node_c
        .node
        .ledger_prefix_checkpoint()
        .expect("C empty checkpoint");
    assert_eq!(
        LedgerPrefixCheckpoint::decode(&checkpoint.canonical_bytes())
            .expect("checkpoint transport codec round trip"),
        checkpoint
    );
    assert!(matches!(
        CommittedRangeRequest::new(checkpoint, MAX_COMMITTED_RANGE_RECORDS + 1)
            .expect_err("range request must be bounded"),
        ConvergenceError::InvalidRangeLimit(_)
    ));
    let request = CommittedRangeRequest::new(checkpoint, 8).expect("bounded range request");
    assert_eq!(
        CommittedRangeRequest::decode(&request.canonical_bytes())
            .expect("range-request transport codec round trip"),
        request
    );
    let range = node_a
        .node
        .export_committed_range(&mut a_to_c, request)
        .expect("A serves the contiguous missing range");
    assert_eq!(range.records().len(), 2);
    assert_eq!(
        CommittedEnvelopeRange::decode(&range.canonical_bytes())
            .expect("range transport codec round trip"),
        range
    );
    let one_record_request =
        CommittedRangeRequest::new(checkpoint, 1).expect("single-record range request");
    assert!(matches!(
        node_c
            .node
            .import_committed_range(&mut c_to_a, one_record_request, range.clone())
            .expect_err("a response cannot exceed its originating request limit"),
        ConvergenceError::RangeExceedsRequestedLimit {
            requested: 1,
            received: 2,
        }
    ));
    assert_eq!(node_c.node.journal_frame_count().expect("C frames"), 0);
    let first_receipts = node_c
        .node
        .import_committed_range(&mut c_to_a, request, range.clone())
        .expect("C verifies and commits the exact contiguous range");
    assert_eq!(first_receipts.len(), 2);
    assert_eq!(node_c.node.journal_frame_count().expect("C frames"), 2);
    assert_eq!(
        node_c
            .node
            .committed_envelope_record(1)
            .expect("C height one"),
        node_a
            .node
            .committed_envelope_record(1)
            .expect("A height one")
    );

    let retry_receipts = node_c
        .node
        .import_committed_range(&mut c_to_a, request, range)
        .expect("lost range response may be retried exactly");
    assert_eq!(retry_receipts, first_receipts);
    assert_eq!(node_c.node.journal_frame_count().expect("C frames"), 2);

    let turn_two = node_c.node.pending_poa_turn().expect("height-two turn");
    assert_eq!(
        node_c
            .node
            .scheduled_authority(turn_two.height())
            .expect("height-two authority")
            .node_id(),
        node_ids[2]
    );
    committed(
        node_c
            .node
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn_two,
                120_000,
                b"<urn:issue6:block:2> <urn:issue6:value> \"two\" .",
            ))
            .expect("rejoined C takes its deterministic turn"),
    );
}

#[test]
fn producer_and_follower_restart_recover_exact_records_and_lost_receipts() {
    let governance_key = signing_key(90);
    let identity_keys = [signing_key(91), signing_key(92), signing_key(93)];
    let validator_keys = [signing_key(94), signing_key(95), signing_key(96)];
    let node_ids = [
        Uuid::from_u128(910),
        Uuid::from_u128(920),
        Uuid::from_u128(930),
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
    let profile = network_profile(&manifest, &validator_keys);
    let producer_dir = tempdir().expect("producer directory");
    let follower_dir = tempdir().expect("follower directory");

    let producer = start_validator_at(
        producer_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    let turn = producer.pending_poa_turn().expect("producer turn");
    committed(
        producer
            .submit_poa_request(PoAProposalRequest::ordinary(
                turn,
                200_000,
                b"<urn:issue6:restart> <urn:issue6:value> \"durable\" .",
            ))
            .expect("producer durable commit"),
    );
    let record_before_failure = producer
        .committed_envelope_record(0)
        .expect("record before producer failure");
    let producer_receipt_before_failure = producer
        .local_commit_receipt(0)
        .expect("receipt before producer failure");
    drop(producer);

    let mut producer = start_validator_at(
        producer_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    assert_eq!(
        producer
            .committed_envelope_record(0)
            .expect("record after producer restart"),
        record_before_failure
    );
    assert_eq!(
        producer
            .local_commit_receipt(0)
            .expect("receipt after producer restart"),
        producer_receipt_before_failure
    );

    let mut follower = start_validator_at(
        follower_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[1],
        &identity_keys[1],
        &validator_keys[1],
    );
    let follower_receipt_before_lost_response = {
        let mut producer_transport = producer.quarantine_peer_transport();
        let mut follower_transport = follower.quarantine_peer_transport();
        let (producer_session, follower_session) = complete_handshake(
            &mut producer,
            &producer_transport,
            &mut follower,
            &follower_transport,
            node_ids[1],
            210_000,
        );
        producer_transport
            .bind_authenticated_session(producer_session)
            .expect("bind producer session");
        follower_transport
            .bind_authenticated_session(follower_session)
            .expect("bind follower session");
        follower
            .receive_committed_envelope(&mut follower_transport, record_before_failure.clone())
            .expect("follower commits before its response is lost")
    };
    drop(follower);

    let mut follower = start_validator_at(
        follower_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[1],
        &identity_keys[1],
        &validator_keys[1],
    );
    let mut producer_transport = producer.quarantine_peer_transport();
    let mut follower_transport = follower.quarantine_peer_transport();
    let (producer_session, follower_session) = complete_handshake(
        &mut producer,
        &producer_transport,
        &mut follower,
        &follower_transport,
        node_ids[1],
        220_000,
    );
    producer_transport
        .bind_authenticated_session(producer_session)
        .expect("bind restarted producer session");
    follower_transport
        .bind_authenticated_session(follower_session)
        .expect("bind restarted follower session");
    let retry_receipt = follower
        .receive_committed_envelope(&mut follower_transport, record_before_failure)
        .expect("exact retry after restart returns reproducible receipt");
    assert_eq!(retry_receipt, follower_receipt_before_lost_response);
    assert_eq!(follower.journal_frame_count().expect("follower frames"), 1);
}

#[test]
fn divergent_prefix_fails_closed_without_first_wins_timestamp_or_projection_history() {
    let governance_key = signing_key(120);
    let identity_keys = [signing_key(121), signing_key(122), signing_key(123)];
    let validator_keys = [signing_key(124), signing_key(125), signing_key(126)];
    let node_ids = [
        Uuid::from_u128(1_210),
        Uuid::from_u128(1_220),
        Uuid::from_u128(1_230),
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
    let profile = network_profile(&manifest, &validator_keys);
    let primary_dir = tempdir().expect("primary producer directory");
    let fork_dir = tempdir().expect("fork producer directory");
    let follower_dir = tempdir().expect("follower directory");
    let mut primary = start_validator_at(
        primary_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    let fork = start_validator_at(
        fork_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[0],
        &identity_keys[0],
        &validator_keys[0],
    );
    let mut follower = start_validator_at(
        follower_dir.path(),
        &governance_key,
        &manifest,
        &profile,
        node_ids[1],
        &identity_keys[1],
        &validator_keys[1],
    );

    let primary_turn = primary.pending_poa_turn().expect("primary turn");
    committed(
        primary
            .submit_poa_request(PoAProposalRequest::ordinary(
                primary_turn,
                300_000,
                b"<urn:issue6:fork> <urn:issue6:value> \"primary\" .",
            ))
            .expect("primary commit"),
    );
    let fork_turn = fork.pending_poa_turn().expect("fork turn");
    committed(
        fork.submit_poa_request(PoAProposalRequest::ordinary(
            fork_turn,
            999_999,
            b"<urn:issue6:fork> <urn:issue6:value> \"later-but-conflicting\" .",
        ))
        .expect("isolated fork commit"),
    );
    let primary_record = primary
        .committed_envelope_record(0)
        .expect("primary record");
    let fork_record = fork.committed_envelope_record(0).expect("fork record");
    assert_ne!(primary_record, fork_record);
    let fork_receipt = fork
        .local_commit_receipt(0)
        .expect("fork receipt is valid only for the fork prefix");
    assert!(matches!(
        primary
            .commit_status(0, &[fork_receipt])
            .expect_err("a signed receipt for another exact prefix cannot match"),
        ConvergenceError::ReceiptMismatch { .. }
    ));
    let mut modified_envelope_bytes = primary_record.canonical_bytes();
    let payload_offset = modified_envelope_bytes
        .windows(b"primary".len())
        .position(|window| window == b"primary")
        .expect("record contains the primary RDF literal");
    modified_envelope_bytes[payload_offset] ^= 1;
    assert!(CommittedEnvelopeRecord::decode(&modified_envelope_bytes).is_err());

    let mut primary_transport = primary.quarantine_peer_transport();
    let mut follower_transport = follower.quarantine_peer_transport();
    let (primary_session, follower_session) = complete_handshake(
        &mut primary,
        &primary_transport,
        &mut follower,
        &follower_transport,
        node_ids[1],
        310_000,
    );
    primary_transport
        .bind_authenticated_session(primary_session)
        .expect("bind primary session");
    follower_transport
        .bind_authenticated_session(follower_session)
        .expect("bind follower session");
    let first_receipt = follower
        .receive_committed_envelope(&mut follower_transport, primary_record.clone())
        .expect("commit primary record");

    assert!(matches!(
        follower
            .receive_committed_envelope(&mut follower_transport, fork_record)
            .expect_err("a later timestamp cannot replace a committed prefix"),
        ConvergenceError::DivergentPrefix { index: 0, .. }
    ));

    let mut wrong_prefix_bytes = primary_record.canonical_bytes();
    *wrong_prefix_bytes
        .last_mut()
        .expect("record includes prefix commitment") ^= 1;
    let wrong_prefix = CommittedEnvelopeRecord::decode(&wrong_prefix_bytes)
        .expect("record codec alone cannot know the receiver's prefix");
    assert!(matches!(
        follower
            .receive_committed_envelope(&mut follower_transport, wrong_prefix)
            .expect_err("a conflicting prefix commitment fails closed"),
        ConvergenceError::DivergentPrefix { index: 0, .. }
    ));

    let retry_receipt = follower
        .receive_committed_envelope(&mut follower_transport, primary_record.clone())
        .expect("exact duplicate remains idempotent");
    assert_eq!(retry_receipt, first_receipt);
    assert_eq!(follower.journal_frame_count().expect("follower frames"), 1);
    assert_eq!(
        follower
            .committed_envelope_record(0)
            .expect("unchanged follower prefix"),
        primary_record
    );
}

struct ProcessContract {
    governance_key: SigningKey,
    identity_keys: [SigningKey; 3],
    validator_keys: [SigningKey; 3],
    node_ids: [Uuid; 3],
    manifest: SignedMembershipManifest,
    profile: NetworkProfile,
}

fn process_contract() -> ProcessContract {
    let governance_key = signing_key(150);
    let identity_keys = [signing_key(151), signing_key(152), signing_key(153)];
    let validator_keys = [signing_key(154), signing_key(155), signing_key(156)];
    let node_ids = [
        Uuid::from_u128(1_510),
        Uuid::from_u128(1_520),
        Uuid::from_u128(1_530),
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
    let profile = network_profile(&manifest, &validator_keys);
    ProcessContract {
        governance_key,
        identity_keys,
        validator_keys,
        node_ids,
        manifest,
        profile,
    }
}

fn process_node(contract: &ProcessContract, root: &Path, index: usize) -> ReferenceNode {
    start_validator_at(
        &root.join(format!("node-{index}")),
        &contract.governance_key,
        &contract.manifest,
        &contract.profile,
        contract.node_ids[index],
        &contract.identity_keys[index],
        &contract.validator_keys[index],
    )
}

fn worker_path(name: &str) -> std::path::PathBuf {
    std::env::var_os(name)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| panic!("missing worker path {name}"))
}

#[test]
#[ignore = "subprocess entrypoint used only by the Issue #6 process campaign"]
fn issue6_reference_node_process_worker() {
    let Some(command) = std::env::var_os("PROVCHAIN_ISSUE6_WORKER_COMMAND") else {
        return;
    };
    let command = command.to_string_lossy();
    let root = worker_path("PROVCHAIN_ISSUE6_WORKER_ROOT");
    let record_path = worker_path("PROVCHAIN_ISSUE6_WORKER_RECORD");
    let receipt_path = worker_path("PROVCHAIN_ISSUE6_WORKER_RECEIPT");
    let contract = process_contract();

    match command.as_ref() {
        "produce" => {
            let producer = process_node(&contract, &root, 0);
            let turn = producer.pending_poa_turn().expect("process producer turn");
            committed(
                producer
                    .submit_poa_request(PoAProposalRequest::ordinary(
                        turn,
                        400_000,
                        b"<urn:issue6:process> <urn:issue6:value> \"exact\" .",
                    ))
                    .expect("process producer commit"),
            );
            let record = producer
                .committed_envelope_record(0)
                .expect("process producer record");
            let receipt = producer
                .local_commit_receipt(0)
                .expect("process producer receipt");
            fs::write(record_path, record.canonical_bytes()).expect("write process record");
            fs::write(receipt_path, receipt.canonical_bytes()).expect("write producer receipt");
        }
        "follow" => {
            let follower_index: usize = std::env::var("PROVCHAIN_ISSUE6_WORKER_NODE_INDEX")
                .expect("follower index")
                .parse()
                .expect("numeric follower index");
            assert!(matches!(follower_index, 1 | 2));
            let record = CommittedEnvelopeRecord::decode(
                &fs::read(record_path).expect("read process record"),
            )
            .expect("decode process record");
            let mut producer = process_node(&contract, &root, 0);
            let mut follower = process_node(&contract, &root, follower_index);
            let producer_transport = producer.quarantine_peer_transport();
            let mut follower_transport = follower.quarantine_peer_transport();
            let (_producer_session, follower_session) = complete_handshake(
                &mut producer,
                &producer_transport,
                &mut follower,
                &follower_transport,
                contract.node_ids[follower_index],
                410_000 + follower_index as u64 * 100,
            );
            follower_transport
                .bind_authenticated_session(follower_session)
                .expect("bind process follower session");
            drop(producer);
            let receipt = follower
                .receive_committed_envelope(&mut follower_transport, record)
                .expect("process follower exact commit");
            fs::write(receipt_path, receipt.canonical_bytes()).expect("write follower receipt");
        }
        "verify" => {
            let node = process_node(&contract, &root, 0);
            let receipt_paths = [
                worker_path("PROVCHAIN_ISSUE6_WORKER_RECEIPT_A"),
                worker_path("PROVCHAIN_ISSUE6_WORKER_RECEIPT_B"),
                worker_path("PROVCHAIN_ISSUE6_WORKER_RECEIPT_C"),
            ];
            let receipts: Vec<_> = receipt_paths
                .iter()
                .map(|path| {
                    CommitReceipt::decode(&fs::read(path).expect("read process receipt"))
                        .expect("decode process receipt")
                })
                .collect();
            assert!(matches!(
                node.commit_status(0, &receipts)
                    .expect("verify process convergence"),
                CommitStatus::NetworkConverged { .. }
            ));
        }
        other => panic!("unknown Issue #6 process worker command {other}"),
    }
    println!("issue6-worker-pid={}", std::process::id());
}

fn run_process_worker(
    command: &str,
    root: &Path,
    record_path: &Path,
    receipt_path: &Path,
    follower_index: Option<usize>,
    all_receipts: Option<[&Path; 3]>,
) -> u32 {
    let executable = std::env::current_exe().expect("current Issue #6 test executable");
    let mut worker = Command::new(executable);
    worker
        .arg("--ignored")
        .arg("--exact")
        .arg("issue6_reference_node_process_worker")
        .arg("--nocapture")
        .env("PROVCHAIN_ISSUE6_WORKER_COMMAND", command)
        .env("PROVCHAIN_ISSUE6_WORKER_ROOT", root)
        .env("PROVCHAIN_ISSUE6_WORKER_RECORD", record_path)
        .env("PROVCHAIN_ISSUE6_WORKER_RECEIPT", receipt_path);
    if let Some(index) = follower_index {
        worker.env("PROVCHAIN_ISSUE6_WORKER_NODE_INDEX", index.to_string());
    }
    if let Some([receipt_a, receipt_b, receipt_c]) = all_receipts {
        worker
            .env("PROVCHAIN_ISSUE6_WORKER_RECEIPT_A", receipt_a)
            .env("PROVCHAIN_ISSUE6_WORKER_RECEIPT_B", receipt_b)
            .env("PROVCHAIN_ISSUE6_WORKER_RECEIPT_C", receipt_c);
    }
    let output = worker.output().expect("run Issue #6 process worker");
    assert!(
        output.status.success(),
        "process worker {command} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("worker stdout is UTF-8");
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("issue6-worker-pid="))
        .expect("worker reports its process id")
        .parse()
        .expect("worker process id is numeric")
}

#[test]
fn three_independent_reference_node_processes_commit_byte_identical_journals() {
    let root = tempdir().expect("process campaign root");
    let record_path = root.path().join("committed-record.bin");
    let receipt_paths = [
        root.path().join("receipt-a.bin"),
        root.path().join("receipt-b.bin"),
        root.path().join("receipt-c.bin"),
    ];

    let producer_pid = run_process_worker(
        "produce",
        root.path(),
        &record_path,
        &receipt_paths[0],
        None,
        None,
    );
    let follower_b_pid = run_process_worker(
        "follow",
        root.path(),
        &record_path,
        &receipt_paths[1],
        Some(1),
        None,
    );
    let follower_c_pid = run_process_worker(
        "follow",
        root.path(),
        &record_path,
        &receipt_paths[2],
        Some(2),
        None,
    );
    let process_ids: BTreeSet<_> = [producer_pid, follower_b_pid, follower_c_pid]
        .into_iter()
        .collect();
    assert_eq!(
        process_ids.len(),
        3,
        "each reference node ran in its own OS process"
    );

    let journal_a =
        fs::read(root.path().join("node-0/ledger.journal")).expect("read process A journal");
    let journal_b =
        fs::read(root.path().join("node-1/ledger.journal")).expect("read process B journal");
    let journal_c =
        fs::read(root.path().join("node-2/ledger.journal")).expect("read process C journal");
    assert_eq!(journal_a, journal_b);
    assert_eq!(journal_a, journal_c);

    let receipt_refs = [
        receipt_paths[0].as_path(),
        receipt_paths[1].as_path(),
        receipt_paths[2].as_path(),
    ];
    run_process_worker(
        "verify",
        root.path(),
        &record_path,
        &receipt_paths[0],
        None,
        Some(receipt_refs),
    );
}
