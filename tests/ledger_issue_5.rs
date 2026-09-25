//! Acceptance tests for scheduled PoA coordination and signing safety (Issue #5).

use ed25519_dalek::SigningKey;
use provchain_org::ledger::{
    AdmissionOutcome, AdmittedBlockEnvelope, Ledger, LedgerError, LedgerHash, LedgerProfile,
};
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::poa::{PoAError, PoAProposalRequest};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::{tempdir, TempDir};
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue5";
const PROFILE_ID: &str = "issue5.reference";

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
        manifest_id: "issue5.membership".to_string(),
        version: 8,
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

fn start_node(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
) -> TestNode {
    let data_dir = tempdir().expect("temporary node directory");
    let node = ReferenceNode::start_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
    )
    .expect("activate reference node");
    TestNode {
        _data_dir: data_dir,
        node,
    }
}

fn start_validator_node(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> TestNode {
    start_validator_node_with_ledger_profile(
        governance_key,
        manifest,
        profile,
        node_id,
        identity_key,
        validator_key,
        ledger_profile(),
    )
}

fn start_validator_node_with_ledger_profile(
    governance_key: &SigningKey,
    manifest: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
    ledger_profile: LedgerProfile,
) -> TestNode {
    let data_dir = tempdir().expect("temporary validator directory");
    let node = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile, semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
        validator_key.clone(),
    )
    .expect("activate reference validator");
    TestNode {
        _data_dir: data_dir,
        node,
    }
}

fn committed_envelope_hash(outcome: AdmissionOutcome) -> LedgerHash {
    match outcome {
        AdmissionOutcome::Committed { envelope } => envelope.envelope_hash,
        AdmissionOutcome::Rejected { reason } => panic!("proposal was rejected: {reason}"),
    }
}

fn committed_envelope(outcome: AdmissionOutcome) -> Box<AdmittedBlockEnvelope> {
    match outcome {
        AdmissionOutcome::Committed { envelope } => envelope,
        AdmissionOutcome::Rejected { reason } => panic!("proposal was rejected: {reason}"),
    }
}

#[test]
fn every_node_derives_the_same_scheduled_authority_from_height_and_manifest_bound_order() {
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
        vec![
            member(node_ids[2], &identity_keys[2], &validator_keys[2]),
            member(node_ids[0], &identity_keys[0], &validator_keys[0]),
            member(node_ids[1], &identity_keys[1], &validator_keys[1]),
        ],
    );
    let authority_order = [
        validator_keys[1].clone(),
        validator_keys[2].clone(),
        validator_keys[0].clone(),
    ];
    let expected_node_order = [node_ids[1], node_ids[2], node_ids[0]];
    let profile = network_profile(&manifest, &authority_order);
    let nodes = [
        start_node(
            &governance_key,
            &manifest,
            &profile,
            node_ids[0],
            &identity_keys[0],
        ),
        start_node(
            &governance_key,
            &manifest,
            &profile,
            node_ids[1],
            &identity_keys[1],
        ),
        start_node(
            &governance_key,
            &manifest,
            &profile,
            node_ids[2],
            &identity_keys[2],
        ),
    ];

    for height in 0..9 {
        let expected_node_id = expected_node_order[height as usize % expected_node_order.len()];
        let expected_key = authority_order[height as usize % authority_order.len()]
            .verifying_key()
            .to_bytes();
        for node in &nodes {
            let scheduled = node
                .node
                .scheduled_authority(height)
                .expect("derive scheduled authority");
            assert_eq!(scheduled.node_id(), expected_node_id);
            assert_eq!(scheduled.validator_public_key(), expected_key);
        }
    }
}

#[test]
fn out_of_turn_validator_is_rejected_and_a_missed_turn_does_not_take_over() {
    let governance_key = signing_key(20);
    let node_a_identity = signing_key(21);
    let node_a_validator = signing_key(22);
    let node_b_identity = signing_key(23);
    let node_b_validator = signing_key(24);
    let node_a_id = Uuid::from_u128(100);
    let node_b_id = Uuid::from_u128(200);
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
        ],
    );
    let profile = network_profile(
        &manifest,
        &[node_a_validator.clone(), node_b_validator.clone()],
    );
    let node_a = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_a_id,
        &node_a_identity,
        &node_a_validator,
    );
    let node_b = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_b_id,
        &node_b_identity,
        &node_b_validator,
    );

    let node_a_turn = node_a.node.pending_poa_turn().expect("node A turn");
    let node_b_turn = node_b.node.pending_poa_turn().expect("node B turn");
    assert_eq!(node_a_turn, node_b_turn);
    assert_eq!(node_a_turn.height(), 0);

    let request = PoAProposalRequest::ordinary(
        node_a_turn,
        9_999_999,
        b"<urn:issue5:event> <urn:issue5:status> \"created\" .",
    );
    let error = node_b
        .node
        .submit_poa_request(request.clone())
        .expect_err("an out-of-turn validator must not take over");
    assert!(matches!(
        error,
        PoAError::OutOfTurn {
            scheduled_node_id,
            attempted_node_id,
            height: 0,
        } if scheduled_node_id == node_a_id && attempted_node_id == node_b_id
    ));
    assert_eq!(node_b.node.journal_frame_count().expect("node B frames"), 0);
    assert_eq!(
        node_b.node.pending_poa_turn().expect("still pending"),
        node_b_turn,
        "elapsed request time must not advance or reassign the missed turn"
    );

    let invalid_preflight =
        PoAProposalRequest::ordinary(node_a_turn, 9_999_997, b"this is not valid RDF provenance");
    assert!(matches!(
        node_a
            .node
            .submit_poa_request(invalid_preflight)
            .expect_err("failed preflight must not select or fence the request"),
        PoAError::Ledger(_)
    ));
    assert_eq!(node_a.node.journal_frame_count().expect("node A frames"), 0);

    let builder_dir = tempdir().expect("temporary proposal builder directory");
    let builder = Ledger::open_in_dir(builder_dir.path(), ledger_profile(), semantic_package())
        .expect("open isolated proposal builder");
    let out_of_turn_signed = builder
        .create_ordinary_candidate(
            b"<urn:issue5:out-of-turn> <urn:issue5:status> \"signed\" .",
            9_999_998,
            &node_b_validator,
        )
        .expect("build out-of-turn signed proposal");
    assert!(matches!(
        node_a
            .node
            .accept_signed_poa_proposal(out_of_turn_signed)
            .expect_err("receivers must reject a valid signature from an unscheduled validator"),
        PoAError::UnscheduledSigner {
            height: 0,
            scheduled_validator_public_key,
            attempted_validator_public_key,
        } if scheduled_validator_public_key == node_a_validator.verifying_key().to_bytes()
            && attempted_validator_public_key == node_b_validator.verifying_key().to_bytes()
    ));
    assert_eq!(node_a.node.journal_frame_count().expect("node A frames"), 0);

    let outcome = node_a
        .node
        .submit_poa_request(request)
        .expect("the scheduled authority may propose");
    assert!(outcome.is_committed());
    assert_eq!(node_a.node.journal_frame_count().expect("node A frames"), 1);
}

#[test]
fn concurrent_identical_requests_coalesce_and_conflicts_cannot_replace_the_selected_body() {
    let governance_key = signing_key(40);
    let identity_key = signing_key(41);
    let validator_key = signing_key(42);
    let node_id = Uuid::from_u128(400);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let TestNode {
        _data_dir: data_dir,
        node,
    } = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_id,
        &identity_key,
        &validator_key,
    );
    let node = Arc::new(node);
    let turn = node.pending_poa_turn().expect("pending turn");
    let request = PoAProposalRequest::ordinary(
        turn,
        40_000,
        b"<urn:issue5:coalesce> <urn:issue5:status> \"selected\" .",
    );
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let node = Arc::clone(&node);
        let request = request.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            node.submit_poa_request(request)
        }));
    }
    barrier.wait();

    let first_hash = committed_envelope_hash(
        handles
            .remove(0)
            .join()
            .expect("first request thread")
            .expect("first request outcome"),
    );
    let second_hash = committed_envelope_hash(
        handles
            .remove(0)
            .join()
            .expect("second request thread")
            .expect("second request outcome"),
    );
    assert_eq!(first_hash, second_hash);
    assert_eq!(node.journal_frame_count().expect("journal frames"), 1);

    let conflict = PoAProposalRequest::ordinary(
        turn,
        40_001,
        b"<urn:issue5:conflict> <urn:issue5:status> \"replacement\" .",
    );
    let error = node
        .submit_poa_request(conflict)
        .expect_err("a conflicting request cannot replace the selected body");
    assert!(matches!(
        error,
        PoAError::ProposalConflict { height: 0, .. }
    ));
    assert_eq!(node.journal_frame_count().expect("journal frames"), 1);

    let next_turn = node.pending_poa_turn().expect("next pending turn");
    let conflicting_requests = [
        PoAProposalRequest::ordinary(
            next_turn,
            50_000,
            b"<urn:issue5:race-a> <urn:issue5:status> \"candidate\" .",
        ),
        PoAProposalRequest::ordinary(
            next_turn,
            50_001,
            b"<urn:issue5:race-b> <urn:issue5:status> \"candidate\" .",
        ),
    ];
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for request in conflicting_requests {
        let node = Arc::clone(&node);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            node.submit_poa_request(request)
        }));
    }
    barrier.wait();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("conflicting request thread"))
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Ok(outcome) if outcome.is_committed()))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(PoAError::ProposalConflict { height: 1, .. })))
            .count(),
        1
    );
    assert_eq!(node.journal_frame_count().expect("journal frames"), 2);

    drop(node);
    drop(data_dir);
}

#[test]
fn lost_response_restart_recovers_the_exact_fenced_proposal_without_a_second_commit() {
    let governance_key = signing_key(60);
    let identity_key = signing_key(61);
    let validator_key = signing_key(62);
    let node_id = Uuid::from_u128(600);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let data_dir = tempdir().expect("temporary validator directory");

    let node = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
        validator_key.clone(),
    )
    .expect("start validator before response loss");
    let turn = node.pending_poa_turn().expect("pending turn");
    let request = PoAProposalRequest::ordinary(
        turn,
        60_000,
        b"<urn:issue5:recovery> <urn:issue5:status> \"durable\" .",
    );
    let committed = committed_envelope(
        node.submit_poa_request(request.clone())
            .expect("commit before caller loses the response"),
    );
    drop(node); // The caller retained no usable response across this process boundary.

    let restarted = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
        validator_key,
    )
    .expect("restart validator and recover fence");
    let recovered = committed_envelope(
        restarted
            .submit_poa_request(request.clone())
            .expect("exact retry must recover the fenced proposal"),
    );
    assert_eq!(
        recovered, committed,
        "body and deterministic signature changed"
    );
    assert_eq!(
        restarted
            .journal_frame_count()
            .expect("journal frame count after recovery"),
        1
    );

    let conflict = PoAProposalRequest::ordinary(
        turn,
        60_001,
        b"<urn:issue5:recovery> <urn:issue5:status> \"replacement\" .",
    );
    assert!(matches!(
        restarted
            .submit_poa_request(conflict)
            .expect_err("restart cannot refresh a fenced body"),
        PoAError::ProposalConflict { height: 0, .. }
    ));
    assert_eq!(
        restarted
            .journal_frame_count()
            .expect("journal frame count after conflict"),
        1
    );
}

#[test]
fn distinct_signed_proposals_for_one_turn_are_equivocation_not_timestamp_fork_choice() {
    let governance_key = signing_key(70);
    let identity_key = signing_key(71);
    let validator_key = signing_key(72);
    let node_id = Uuid::from_u128(700);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let node = start_node(&governance_key, &manifest, &profile, node_id, &identity_key);
    let builder_dir = tempdir().expect("temporary proposal builder directory");
    let builder = Ledger::open_in_dir(builder_dir.path(), ledger_profile(), semantic_package())
        .expect("open isolated proposal builder");
    let later_timestamp = builder
        .create_ordinary_candidate(
            b"<urn:issue5:equivocation> <urn:issue5:value> \"first-arrival\" .",
            70_001,
            &validator_key,
        )
        .expect("build first signed proposal");
    let first_digest = later_timestamp.proposal_digest;
    let mut invalid_signature = later_timestamp.clone();
    invalid_signature.proposer_signature[0] ^= 0xff;
    assert!(matches!(
        node.node
            .accept_signed_poa_proposal(invalid_signature)
            .expect_err("invalid signer evidence must fail Consensus Acceptance"),
        PoAError::InvalidSignedProposal(_)
    ));
    assert_eq!(node.node.journal_frame_count().expect("journal frames"), 0);
    let earlier_timestamp = builder
        .create_ordinary_candidate(
            b"<urn:issue5:equivocation> <urn:issue5:value> \"replacement\" .",
            70_000,
            &validator_key,
        )
        .expect("build conflicting signed proposal");
    let conflicting_digest = earlier_timestamp.proposal_digest;

    assert!(node
        .node
        .accept_signed_poa_proposal(later_timestamp)
        .expect("scheduled signed proposal reaches Final Admission")
        .is_committed());
    let error = node
        .node
        .accept_signed_poa_proposal(earlier_timestamp)
        .expect_err("an earlier timestamp cannot replace a signed proposal");
    assert!(matches!(
        error,
        PoAError::Equivocation {
            height: 0,
            selected_proposal_digest,
            conflicting_proposal_digest,
        } if selected_proposal_digest == first_digest
            && conflicting_proposal_digest == conflicting_digest
    ));
    assert_eq!(node.node.journal_frame_count().expect("journal frames"), 1);
}

#[test]
fn deterministic_final_admission_rejection_stalls_without_replacement() {
    let governance_key = signing_key(90);
    let identity_key = signing_key(91);
    let validator_key = signing_key(92);
    let node_id = Uuid::from_u128(900);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let TestNode {
        _data_dir: data_dir,
        node,
    } = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_id,
        &identity_key,
        &validator_key,
    );

    let rejected_builder_dir = tempdir().expect("temporary rejected builder directory");
    let rejected_profile = bind_profile(LedgerProfile::new(NETWORK_ID, "issue5.wrong-profile"));
    let rejected_builder = Ledger::open_in_dir(
        rejected_builder_dir.path(),
        rejected_profile,
        semantic_package(),
    )
    .expect("open rejected proposal builder");
    let rejected = rejected_builder
        .create_ordinary_candidate(
            b"<urn:issue5:rejected> <urn:issue5:status> \"locked\" .",
            90_000,
            &validator_key,
        )
        .expect("build signed proposal that Final Admission rejects");
    let rejected_digest = rejected.proposal_digest;
    assert!(matches!(
        node.accept_signed_poa_proposal(rejected)
            .expect_err("deterministic rejection must stall the selected turn"),
        PoAError::FinalAdmissionRejected { height: 0, .. }
    ));
    assert_eq!(node.journal_frame_count().expect("journal frames"), 0);

    let local_replacement = PoAProposalRequest::ordinary(
        node.pending_poa_turn()
            .expect("rejected turn remains pending"),
        90_001,
        b"<urn:issue5:local-replacement> <urn:issue5:status> \"valid\" .",
    );
    assert!(matches!(
        node.submit_poa_request(local_replacement)
            .expect_err("local signing cannot replace an observed rejected proposal"),
        PoAError::Equivocation {
            height: 0,
            selected_proposal_digest,
            ..
        } if selected_proposal_digest == rejected_digest
    ));

    let replacement_builder_dir = tempdir().expect("temporary replacement builder directory");
    let replacement_builder = Ledger::open_in_dir(
        replacement_builder_dir.path(),
        ledger_profile(),
        semantic_package(),
    )
    .expect("open replacement proposal builder");
    let replacement = replacement_builder
        .create_ordinary_candidate(
            b"<urn:issue5:replacement> <urn:issue5:status> \"valid\" .",
            90_001,
            &validator_key,
        )
        .expect("build otherwise valid replacement");
    let replacement_digest = replacement.proposal_digest;
    assert!(matches!(
        node
            .accept_signed_poa_proposal(replacement)
            .expect_err("a rejected signed proposal cannot be replaced"),
        PoAError::Equivocation {
            height: 0,
            selected_proposal_digest,
            conflicting_proposal_digest,
        } if selected_proposal_digest == rejected_digest
            && conflicting_proposal_digest == replacement_digest
    ));
    assert_eq!(node.journal_frame_count().expect("journal frames"), 0);

    drop(node);
    let restarted = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
        validator_key,
    )
    .expect("restart validator with durable rejected-proposal evidence");
    let restart_replacement = PoAProposalRequest::ordinary(
        restarted
            .pending_poa_turn()
            .expect("rejected turn remains pending after restart"),
        90_002,
        b"<urn:issue5:restart-replacement> <urn:issue5:status> \"valid\" .",
    );
    assert!(matches!(
        restarted
            .submit_poa_request(restart_replacement)
            .expect_err("restart cannot forget the rejected signed proposal"),
        PoAError::Equivocation {
            height: 0,
            selected_proposal_digest,
            ..
        } if selected_proposal_digest == rejected_digest
    ));
    assert_eq!(
        restarted
            .journal_frame_count()
            .expect("journal frames after restart conflict"),
        0
    );
}

#[test]
fn a_non_signing_node_preserves_rejected_proposal_identity_across_restart() {
    let governance_key = signing_key(95);
    let identity_key = signing_key(96);
    let validator_key = signing_key(97);
    let node_id = Uuid::from_u128(950);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let TestNode {
        _data_dir: data_dir,
        node,
    } = start_node(&governance_key, &manifest, &profile, node_id, &identity_key);
    let builder_dir = tempdir().expect("temporary rejected-proposal builder");
    let wrong_profile = bind_profile(LedgerProfile::new(NETWORK_ID, "issue5.peer-wrong-profile"));
    let builder = Ledger::open_in_dir(builder_dir.path(), wrong_profile, semantic_package())
        .expect("open rejected-proposal builder");
    let rejected = builder
        .create_ordinary_candidate(
            b"<urn:issue5:peer-rejection> <urn:issue5:status> \"locked\" .",
            95_000,
            &validator_key,
        )
        .expect("build rejected signed proposal");
    let rejected_digest = rejected.proposal_digest;
    assert!(matches!(
        node.accept_signed_poa_proposal(rejected),
        Err(PoAError::FinalAdmissionRejected { height: 0, .. })
    ));
    drop(node);

    let restarted = ReferenceNode::start_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
    )
    .expect("restart non-signing node with coordinator journal");
    let replacement_dir = tempdir().expect("temporary replacement builder");
    let replacement_builder =
        Ledger::open_in_dir(replacement_dir.path(), ledger_profile(), semantic_package())
            .expect("open replacement builder");
    let replacement = replacement_builder
        .create_ordinary_candidate(
            b"<urn:issue5:peer-rejection> <urn:issue5:status> \"replacement\" .",
            95_001,
            &validator_key,
        )
        .expect("build conflicting signed proposal");
    let replacement_digest = replacement.proposal_digest;
    assert!(matches!(
        restarted
            .accept_signed_poa_proposal(replacement)
            .expect_err("restart cannot forget a rejected signed proposal"),
        PoAError::Equivocation {
            height: 0,
            selected_proposal_digest,
            conflicting_proposal_digest,
        } if selected_proposal_digest == rejected_digest
            && conflicting_proposal_digest == replacement_digest
    ));
}

#[test]
fn block_interval_is_preflighted_for_local_and_received_signed_proposals() {
    let governance_key = signing_key(100);
    let identity_key = signing_key(101);
    let validator_key = signing_key(102);
    let node_id = Uuid::from_u128(1_000);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let node = start_validator_node(
        &governance_key,
        &manifest,
        &profile,
        node_id,
        &identity_key,
        &validator_key,
    );

    let first_payload = b"<urn:issue5:spacing:first> <urn:issue5:status> \"committed\" .";
    let second_payload = b"<urn:issue5:spacing:second> <urn:issue5:status> \"committed\" .";
    let first_turn = node.node.pending_poa_turn().expect("first turn");
    assert!(node
        .node
        .submit_poa_request(PoAProposalRequest::ordinary(
            first_turn,
            100_000,
            first_payload,
        ))
        .expect("genesis proposal has no predecessor spacing")
        .is_committed());

    let second_turn = node.node.pending_poa_turn().expect("second turn");
    assert!(matches!(
        node.node
            .submit_poa_request(PoAProposalRequest::ordinary(
                second_turn,
                109_999,
                second_payload,
            ))
            .expect_err("proposal before the ten-second boundary must be rejected preflight"),
        PoAError::ProposalTooEarly {
            height: 1,
            timestamp_millis: 109_999,
            minimum_timestamp_millis: 110_000,
        }
    ));
    assert_eq!(
        node.node
            .journal_frame_count()
            .expect("one committed frame"),
        1
    );
    assert!(node
        .node
        .submit_poa_request(PoAProposalRequest::ordinary(
            second_turn,
            110_000,
            second_payload,
        ))
        .expect("proposal exactly at the spacing boundary")
        .is_committed());

    let builder_dir = tempdir().expect("temporary spacing builder");
    let mut builder = Ledger::open_in_dir(builder_dir.path(), ledger_profile(), semantic_package())
        .expect("open spacing proposal builder");
    for (timestamp, payload) in [
        (100_000, first_payload.as_slice()),
        (110_000, second_payload.as_slice()),
    ] {
        let candidate = builder
            .create_ordinary_candidate(payload, timestamp, &validator_key)
            .expect("rebuild committed prefix candidate");
        assert!(builder
            .proposal_adapter()
            .submit(candidate)
            .expect("rebuild committed prefix")
            .is_committed());
    }
    let received_too_early = builder
        .create_ordinary_candidate(
            b"<urn:issue5:spacing:third> <urn:issue5:status> \"too-early\" .",
            119_999,
            &validator_key,
        )
        .expect("build signed too-early proposal");
    assert!(matches!(
        node.node
            .accept_signed_poa_proposal(received_too_early)
            .expect_err("received signed proposal must obey the same interval"),
        PoAError::ProposalTooEarly {
            height: 2,
            timestamp_millis: 119_999,
            minimum_timestamp_millis: 120_000,
        }
    ));
    assert_eq!(
        node.node
            .journal_frame_count()
            .expect("two committed frames"),
        2
    );
}

#[test]
fn deterministic_envelope_size_failure_does_not_begin_selection_or_signing() {
    let governance_key = signing_key(110);
    let identity_key = signing_key(111);
    let validator_key = signing_key(112);
    let node_id = Uuid::from_u128(1_100);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(&manifest, std::slice::from_ref(&validator_key));
    let TestNode {
        _data_dir: data_dir,
        node,
    } = start_validator_node_with_ledger_profile(
        &governance_key,
        &manifest,
        &profile,
        node_id,
        &identity_key,
        &validator_key,
        ledger_profile().with_max_envelope_bytes(256),
    );
    let fence_path = data_dir.path().join("poa-signing-fence.journal");
    let fence_before = std::fs::read(&fence_path).expect("read initial signing fence");
    let turn = node.pending_poa_turn().expect("pending turn");

    assert!(matches!(
        node.submit_poa_request(PoAProposalRequest::ordinary(
            turn,
            110_000,
            b"<urn:issue5:size> <urn:issue5:status> \"too-large-envelope\" .",
        ))
        .expect_err("known envelope-size failure must precede fence persistence"),
        PoAError::Ledger(LedgerError::OversizedEnvelope(_))
    ));
    assert_eq!(node.journal_frame_count().expect("no ledger commit"), 0);
    assert_eq!(
        std::fs::read(&fence_path).expect("read unchanged signing fence"),
        fence_before,
        "preflight failure wrote signing-fence selection state"
    );

    assert!(matches!(
        node.submit_poa_request(PoAProposalRequest::ordinary(
            turn,
            110_001,
            b"<urn:issue5:size> <urn:issue5:status> \"still-unselected\" .",
        ))
        .expect_err("a different request must remain preflightable, not conflict-locked"),
        PoAError::Ledger(LedgerError::OversizedEnvelope(_))
    ));
}
