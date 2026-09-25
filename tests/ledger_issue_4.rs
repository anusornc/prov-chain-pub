//! Acceptance tests for manifest-authorized reference-node membership (Issue #4).

use ed25519_dalek::SigningKey;
use provchain_org::ledger::LedgerProfile;
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipError, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest, SignerAuthorizationError,
};
use provchain_org::network::peer_session::{
    AuthenticatedPeerSession, PeerMessageClass, PeerSessionError, PeerTransport,
    PeerTransportState, MAX_PENDING_PEER_HANDSHAKES, PEER_HANDSHAKE_TIMEOUT_MILLIS,
};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::network::reference::{
    ReferenceLedgerActivation, ReferenceNode, ReferenceNodeError,
};
use tempfile::tempdir;
use uuid::Uuid;

mod support;
use support::semantic::{bind_profile, semantic_package};

const NETWORK_ID: &str = "provchain.issue4";
const PROFILE_ID: &str = "issue4.reference";

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
        manifest_id: "issue4.membership".to_string(),
        version: 7,
        network_id: NETWORK_ID.to_string(),
        network_profile_id: PROFILE_ID.to_string(),
        members,
    }
    .sign(governance_key)
    .expect("sign membership manifest")
}

fn network_profile(
    manifest: &SignedMembershipManifest,
    validator_keys: Vec<String>,
) -> NetworkProfile {
    let package = semantic_package();
    NetworkProfile {
        profile_id: PROFILE_ID.to_string(),
        network_id: NETWORK_ID.to_string(),
        consensus: ConsensusProfile {
            consensus_type: "poa".to_string(),
            authority_keys: validator_keys,
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

#[test]
fn valid_governance_manifest_activates_reference_node_before_ledger_admission() {
    let data_dir = tempdir().expect("temporary node directory");
    let governance_key = signing_key(1);
    let identity_key = signing_key(2);
    let validator_key = signing_key(3);
    let node_id = Uuid::from_u128(1);
    let manifest = signed_manifest(
        &governance_key,
        vec![member(node_id, &identity_key, &validator_key)],
    );
    let profile = network_profile(
        &manifest,
        vec![hex::encode(validator_key.verifying_key().to_bytes())],
    );

    let node = ReferenceNode::start_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
    )
    .expect("valid membership contract should activate the node");

    assert_eq!(node.local_node_id(), node_id);
    assert_eq!(node.local_role(), MemberRole::Validator);
    assert_eq!(node.membership_manifest_digest(), manifest.digest());
    assert_eq!(node.journal_frame_count().expect("count frames"), 0);
}

#[derive(Debug, Clone, Copy)]
enum InvalidStartupCase {
    WrongGovernanceRoot,
    WrongManifestVersion,
    WrongNetwork,
    WrongNetworkProfile,
    WrongManifestDigest,
    WrongIdentityKey,
    GovernanceKeyReuse,
    RemovedIdentity,
    InactiveRole,
}

fn assert_startup_error(case: InvalidStartupCase, error: &ReferenceNodeError) {
    match case {
        InvalidStartupCase::WrongGovernanceRoot => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::InvalidGovernanceSignature)
        )),
        InvalidStartupCase::WrongManifestVersion => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::WrongManifestVersion)
        )),
        InvalidStartupCase::WrongNetwork => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::WrongNetwork)
        )),
        InvalidStartupCase::WrongNetworkProfile => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::WrongNetworkProfile)
        )),
        InvalidStartupCase::WrongManifestDigest => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::WrongManifestDigest)
        )),
        InvalidStartupCase::WrongIdentityKey => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::LocalIdentityKeyMismatch(_))
        )),
        InvalidStartupCase::GovernanceKeyReuse => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::GovernanceKeyReused(_))
        )),
        InvalidStartupCase::RemovedIdentity => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::RemovedLocalIdentity(_))
        )),
        InvalidStartupCase::InactiveRole => assert!(matches!(
            error,
            ReferenceNodeError::Membership(MembershipError::InactiveLocalRole { .. })
        )),
    }
}

#[test]
fn invalid_startup_contracts_fail_before_the_ledger_or_network_can_activate() {
    const CASES: [InvalidStartupCase; 9] = [
        InvalidStartupCase::WrongGovernanceRoot,
        InvalidStartupCase::WrongManifestVersion,
        InvalidStartupCase::WrongNetwork,
        InvalidStartupCase::WrongNetworkProfile,
        InvalidStartupCase::WrongManifestDigest,
        InvalidStartupCase::WrongIdentityKey,
        InvalidStartupCase::GovernanceKeyReuse,
        InvalidStartupCase::RemovedIdentity,
        InvalidStartupCase::InactiveRole,
    ];

    for case in CASES {
        let data_dir = tempdir().expect("temporary node directory");
        let governance_key = signing_key(10);
        let identity_key = signing_key(11);
        let validator_key = signing_key(12);
        let node_id = Uuid::from_u128(10);
        let mut manifest = signed_manifest(
            &governance_key,
            vec![member(node_id, &identity_key, &validator_key)],
        );
        let mut profile = network_profile(
            &manifest,
            vec![hex::encode(validator_key.verifying_key().to_bytes())],
        );
        let mut governance_root = governance_key.verifying_key();
        let mut local_identity_key = identity_key.clone();

        match case {
            InvalidStartupCase::WrongGovernanceRoot => {
                governance_root = signing_key(99).verifying_key();
            }
            InvalidStartupCase::WrongManifestVersion => {
                profile
                    .membership
                    .as_mut()
                    .expect("membership binding")
                    .manifest_version += 1;
            }
            InvalidStartupCase::WrongNetwork => {
                let mut wrong_network = manifest.manifest.clone();
                wrong_network.network_id = "another-network".to_string();
                manifest = wrong_network
                    .sign(&governance_key)
                    .expect("sign wrong-network manifest");
                profile.membership = Some(manifest.binding());
            }
            InvalidStartupCase::WrongNetworkProfile => {
                let mut wrong_profile = manifest.manifest.clone();
                wrong_profile.network_profile_id = "another-profile".to_string();
                manifest = wrong_profile
                    .sign(&governance_key)
                    .expect("sign wrong-profile manifest");
                profile.membership = Some(manifest.binding());
            }
            InvalidStartupCase::WrongManifestDigest => {
                profile
                    .membership
                    .as_mut()
                    .expect("membership binding")
                    .manifest_digest = "f".repeat(64);
            }
            InvalidStartupCase::WrongIdentityKey => {
                local_identity_key = signing_key(98);
            }
            InvalidStartupCase::GovernanceKeyReuse => {
                let mut reused_key = manifest.manifest.clone();
                reused_key.members[0].identity_public_key =
                    governance_key.verifying_key().to_bytes();
                manifest = reused_key
                    .sign(&governance_key)
                    .expect("sign governance-key-reuse manifest");
                profile.membership = Some(manifest.binding());
                local_identity_key = governance_key.clone();
            }
            InvalidStartupCase::RemovedIdentity => {
                let mut removed = manifest.manifest.clone();
                removed.members[0].status = MemberStatus::Removed;
                manifest = removed
                    .sign(&governance_key)
                    .expect("sign removed-member manifest");
                profile.membership = Some(manifest.binding());
            }
            InvalidStartupCase::InactiveRole => {
                let mut peer_only = manifest.manifest.clone();
                peer_only.members[0].roles = vec![MemberRole::Peer];
                peer_only.members[0].validator_public_key = None;
                manifest = peer_only
                    .sign(&governance_key)
                    .expect("sign peer-only manifest");
                profile.membership = Some(manifest.binding());
                profile.consensus.authority_keys.clear();
            }
        }

        let error = ReferenceNode::start_in_dir(
            data_dir.path(),
            ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
            profile,
            manifest,
            governance_root,
            NodeIdentity::new(node_id, MemberRole::Validator, local_identity_key),
        )
        .err()
        .expect("invalid startup must fail closed");

        assert_startup_error(case, &error);
        assert!(
            !data_dir.path().join("ledger.journal").exists(),
            "membership failure opened the ledger for {case:?}"
        );
    }
}

#[test]
fn two_authorized_nodes_complete_fresh_mutual_identity_proof() {
    let node_a_dir = tempdir().expect("temporary node A directory");
    let node_b_dir = tempdir().expect("temporary node B directory");
    let governance_key = signing_key(20);
    let node_a_identity = signing_key(21);
    let node_a_validator = signing_key(22);
    let node_b_identity = signing_key(23);
    let node_b_validator = signing_key(24);
    let node_a_id = Uuid::from_u128(20);
    let node_b_id = Uuid::from_u128(21);
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        node_a_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");
    let mut node_b = ReferenceNode::start_in_dir(
        node_b_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_b_id, MemberRole::Validator, node_b_identity),
    )
    .expect("start node B");

    let mut node_a_transport = node_a.quarantine_peer_transport();
    let mut node_b_transport = node_b.quarantine_peer_transport();
    let (node_a_session, node_b_session) = complete_handshake(
        &mut node_a,
        &node_a_transport,
        &mut node_b,
        &node_b_transport,
        node_b_id,
        1_000_000,
    );

    assert_eq!(node_a_session.local_node_id(), node_a_id);
    assert_eq!(node_a_session.authenticated_peer_id(), node_b_id);
    assert_eq!(node_b_session.local_node_id(), node_b_id);
    assert_eq!(node_b_session.authenticated_peer_id(), node_a_id);
    assert_eq!(
        node_a_session.transcript_digest(),
        node_b_session.transcript_digest()
    );
    assert_eq!(
        node_a_session.manifest_digest(),
        node_b_session.manifest_digest()
    );
    node_a_transport
        .bind_authenticated_session(node_a_session)
        .expect("bind node A's single-use session to its handshake transport");
    node_b_transport
        .bind_authenticated_session(node_b_session)
        .expect("bind node B's single-use session to its handshake transport");
    assert_eq!(node_a_transport.authenticated_peer_id(), Some(node_b_id));
    assert_eq!(node_b_transport.authenticated_peer_id(), Some(node_a_id));
}

#[test]
fn handshake_rejects_replay_stale_modified_identity_and_manifest_attempts() {
    let node_a_dir = tempdir().expect("temporary node A directory");
    let node_b_dir = tempdir().expect("temporary node B directory");
    let governance_key = signing_key(30);
    let node_a_identity = signing_key(31);
    let node_a_validator = signing_key(32);
    let node_b_identity = signing_key(33);
    let node_b_validator = signing_key(34);
    let removed_identity = signing_key(35);
    let node_a_id = Uuid::from_u128(30);
    let node_b_id = Uuid::from_u128(31);
    let removed_id = Uuid::from_u128(32);
    let removed_member = NetworkMember {
        node_id: removed_id,
        identity_public_key: removed_identity.verifying_key().to_bytes(),
        roles: vec![MemberRole::Peer],
        status: MemberStatus::Removed,
        validator_public_key: None,
    };
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
            removed_member,
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        node_a_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");
    let mut node_b = ReferenceNode::start_in_dir(
        node_b_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_b_id, MemberRole::Validator, node_b_identity),
    )
    .expect("start node B");

    let replayed_a_transport = node_a.quarantine_peer_transport();
    let replayed_b_transport = node_b.quarantine_peer_transport();
    let replayed_hello = node_a
        .begin_peer_handshake_at(&replayed_a_transport, node_b_id, 2_000_000)
        .expect("begin replay test");
    let replay_challenge = node_b
        .accept_peer_handshake_at(&replayed_b_transport, replayed_hello.clone(), 2_000_001)
        .expect("accept first hello");
    assert_eq!(
        node_b
            .accept_peer_handshake_at(&replayed_b_transport, replayed_hello, 2_000_002)
            .expect_err("replayed nonce must fail"),
        PeerSessionError::ReplayedNonce
    );
    let (replay_response, _) = node_a
        .answer_peer_challenge_at(&replayed_a_transport, replay_challenge, 2_000_003)
        .expect("answer first challenge");
    node_b
        .finish_peer_handshake_at(&replayed_b_transport, replay_response.clone(), 2_000_004)
        .expect("finish first proof");
    assert_eq!(
        node_b
            .finish_peer_handshake_at(&replayed_b_transport, replay_response, 2_000_005)
            .expect_err("replayed proof must fail"),
        PeerSessionError::ReplayedProof
    );

    let stale_a_transport = node_a.quarantine_peer_transport();
    let stale_b_transport = node_b.quarantine_peer_transport();
    let stale_hello = node_a
        .begin_peer_handshake_at(&stale_a_transport, node_b_id, 3_000_000)
        .expect("begin stale test");
    assert_eq!(
        node_b
            .accept_peer_handshake_at(
                &stale_b_transport,
                stale_hello,
                3_000_000 + PEER_HANDSHAKE_TIMEOUT_MILLIS + 1,
            )
            .expect_err("stale nonce must fail"),
        PeerSessionError::HandshakeExpired
    );

    let transcript_a_transport = node_a.quarantine_peer_transport();
    let transcript_b_transport = node_b.quarantine_peer_transport();
    let wrong_transcript_hello = node_a
        .begin_peer_handshake_at(&transcript_a_transport, node_b_id, 4_000_000)
        .expect("begin transcript test");
    let mut wrong_transcript = node_b
        .accept_peer_handshake_at(&transcript_b_transport, wrong_transcript_hello, 4_000_001)
        .expect("create transcript challenge");
    wrong_transcript.hello.context.manifest_digest[0] ^= 1;
    assert_eq!(
        node_a
            .answer_peer_challenge_at(&transcript_a_transport, wrong_transcript, 4_000_002)
            .expect_err("modified transcript must fail"),
        PeerSessionError::WrongTranscript
    );

    let identity_a_transport = node_a.quarantine_peer_transport();
    let identity_b_transport = node_b.quarantine_peer_transport();
    let wrong_identity_hello = node_a
        .begin_peer_handshake_at(&identity_a_transport, node_b_id, 5_000_000)
        .expect("begin identity-proof test");
    let mut wrong_identity = node_b
        .accept_peer_handshake_at(&identity_b_transport, wrong_identity_hello, 5_000_001)
        .expect("create identity challenge");
    wrong_identity.responder_proof[0] ^= 1;
    assert!(matches!(
        node_a
            .answer_peer_challenge_at(&identity_a_transport, wrong_identity, 5_000_002)
            .expect_err("wrong identity proof must fail"),
        PeerSessionError::InvalidIdentityProof(id) if id == node_b_id
    ));

    let binding_a_transport = node_a.quarantine_peer_transport();
    let binding_b_transport = node_b.quarantine_peer_transport();
    let wrong_binding_hello = node_a
        .begin_peer_handshake_at(&binding_a_transport, node_b_id, 5_500_000)
        .expect("begin transport-binding test");
    let mut wrong_transport_binding = node_b
        .accept_peer_handshake_at(&binding_b_transport, wrong_binding_hello, 5_500_001)
        .expect("create transport-bound challenge");
    let unrelated_transport = node_b.quarantine_peer_transport();
    wrong_transport_binding.responder_transport_id = unrelated_transport.transport_id();
    assert!(matches!(
        node_a
            .answer_peer_challenge_at(
                &binding_a_transport,
                wrong_transport_binding,
                5_500_002,
            )
            .expect_err("modified transport binding must invalidate the proof"),
        PeerSessionError::InvalidIdentityProof(id) if id == node_b_id
    ));

    let manifest_a_transport = node_a.quarantine_peer_transport();
    let manifest_b_transport = node_b.quarantine_peer_transport();
    let mut wrong_manifest = node_a
        .begin_peer_handshake_at(&manifest_a_transport, node_b_id, 6_000_000)
        .expect("begin manifest test");
    wrong_manifest.context.manifest_version += 1;
    assert_eq!(
        node_b
            .accept_peer_handshake_at(&manifest_b_transport, wrong_manifest, 6_000_001)
            .expect_err("wrong manifest membership must fail"),
        PeerSessionError::WrongSessionContext
    );

    let unknown_a_transport = node_a.quarantine_peer_transport();
    let unknown_b_transport = node_b.quarantine_peer_transport();
    let mut unknown_member = node_a
        .begin_peer_handshake_at(&unknown_a_transport, node_b_id, 7_000_000)
        .expect("begin unknown-member test");
    let unknown_id = Uuid::from_u128(999);
    unknown_member.initiator_node_id = unknown_id;
    assert_eq!(
        node_b
            .accept_peer_handshake_at(&unknown_b_transport, unknown_member, 7_000_001)
            .expect_err("unknown member must fail"),
        PeerSessionError::UnknownPeer(unknown_id)
    );

    let removed_a_transport = node_a.quarantine_peer_transport();
    let removed_b_transport = node_b.quarantine_peer_transport();
    let mut removed_member = node_a
        .begin_peer_handshake_at(&removed_a_transport, node_b_id, 8_000_000)
        .expect("begin removed-member test");
    removed_member.initiator_node_id = removed_id;
    assert_eq!(
        node_b
            .accept_peer_handshake_at(&removed_b_transport, removed_member, 8_000_001)
            .expect_err("removed member must fail"),
        PeerSessionError::RemovedPeer(removed_id)
    );
}

#[test]
fn unauthenticated_transports_are_quarantined_from_ordinary_peer_messages() {
    let node_a_dir = tempdir().expect("temporary node A directory");
    let node_b_dir = tempdir().expect("temporary node B directory");
    let governance_key = signing_key(40);
    let node_a_identity = signing_key(41);
    let node_a_validator = signing_key(42);
    let node_b_identity = signing_key(43);
    let node_b_validator = signing_key(44);
    let node_a_id = Uuid::from_u128(40);
    let node_b_id = Uuid::from_u128(41);
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        node_a_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");
    let mut node_b = ReferenceNode::start_in_dir(
        node_b_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_b_id, MemberRole::Validator, node_b_identity),
    )
    .expect("start node B");

    for ordinary_message in [
        PeerMessageClass::Consensus,
        PeerMessageClass::Admission,
        PeerMessageClass::Synchronization,
    ] {
        let mut transport = node_a.quarantine_peer_transport();
        transport
            .authorize_message(PeerMessageClass::Handshake)
            .expect("handshake frames are allowed while quarantined");
        transport
            .authorize_message(PeerMessageClass::Error)
            .expect("error frames are allowed while quarantined");
        assert_eq!(
            transport
                .authorize_message(ordinary_message)
                .expect_err("ordinary pre-auth frame must fail closed"),
            PeerSessionError::PreAuthenticationMessage(ordinary_message)
        );
        assert_eq!(transport.state(), PeerTransportState::Closed);
    }
    let mut closing_transport = node_a.quarantine_peer_transport();
    closing_transport
        .authorize_message(PeerMessageClass::Close)
        .expect("close frames are allowed while quarantined");
    assert_eq!(closing_transport.state(), PeerTransportState::Closed);
    assert_eq!(
        node_a.journal_frame_count().expect("count frames"),
        0,
        "rejected pre-auth traffic must not reach admission"
    );

    let mut authenticated_transport = node_a.quarantine_peer_transport();
    let responder_transport = node_b.quarantine_peer_transport();
    let (session, _) = complete_handshake(
        &mut node_a,
        &authenticated_transport,
        &mut node_b,
        &responder_transport,
        node_b_id,
        9_000_000,
    );
    authenticated_transport
        .bind_authenticated_session(session)
        .expect("bind verified session");
    for ordinary_message in [
        PeerMessageClass::Consensus,
        PeerMessageClass::Admission,
        PeerMessageClass::Synchronization,
    ] {
        authenticated_transport
            .authorize_message(ordinary_message)
            .expect("ordinary frame is eligible only after authentication");
    }
    assert_eq!(
        authenticated_transport.state(),
        PeerTransportState::Authenticated
    );
}

#[test]
fn session_capabilities_are_transport_bound_single_use_and_identity_unique() {
    let node_a_dir = tempdir().expect("temporary node A directory");
    let node_b_dir = tempdir().expect("temporary node B directory");
    let governance_key = signing_key(60);
    let node_a_identity = signing_key(61);
    let node_a_validator = signing_key(62);
    let node_b_identity = signing_key(63);
    let node_b_validator = signing_key(64);
    let node_a_id = Uuid::from_u128(60);
    let node_b_id = Uuid::from_u128(61);
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        node_a_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");
    let mut node_b = ReferenceNode::start_in_dir(
        node_b_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_b_id, MemberRole::Validator, node_b_identity),
    )
    .expect("start node B");

    let handshake_transport = node_a.quarantine_peer_transport();
    let responder_transport = node_b.quarantine_peer_transport();
    let (session, _) = complete_handshake(
        &mut node_a,
        &handshake_transport,
        &mut node_b,
        &responder_transport,
        node_b_id,
        12_000_000,
    );
    let mut unrelated_transport = node_a.quarantine_peer_transport();
    assert_eq!(
        unrelated_transport
            .bind_authenticated_session(session)
            .expect_err("a session cannot unlock another transport"),
        PeerSessionError::SessionTransportMismatch
    );
    assert_eq!(unrelated_transport.state(), PeerTransportState::Closed);

    let mut first_transport = node_a.quarantine_peer_transport();
    let first_responder_transport = node_b.quarantine_peer_transport();
    let (first_session, _) = complete_handshake(
        &mut node_a,
        &first_transport,
        &mut node_b,
        &first_responder_transport,
        node_b_id,
        13_000_000,
    );
    first_transport
        .bind_authenticated_session(first_session)
        .expect("first authenticated identity binding should succeed");

    let mut duplicate_transport = node_a.quarantine_peer_transport();
    let duplicate_responder_transport = node_b.quarantine_peer_transport();
    let (duplicate_session, _) = complete_handshake(
        &mut node_a,
        &duplicate_transport,
        &mut node_b,
        &duplicate_responder_transport,
        node_b_id,
        14_000_000,
    );
    assert_eq!(
        duplicate_transport
            .bind_authenticated_session(duplicate_session)
            .expect_err("one peer identity cannot bind two active transports"),
        PeerSessionError::DuplicatePeerIdentity(node_b_id)
    );
    assert_eq!(duplicate_transport.state(), PeerTransportState::Closed);

    first_transport
        .authorize_message(PeerMessageClass::Close)
        .expect("closing the first binding should release the peer identity");
    let mut replacement_transport = node_a.quarantine_peer_transport();
    let replacement_responder_transport = node_b.quarantine_peer_transport();
    let (replacement_session, _) = complete_handshake(
        &mut node_a,
        &replacement_transport,
        &mut node_b,
        &replacement_responder_transport,
        node_b_id,
        15_000_000,
    );
    replacement_transport
        .bind_authenticated_session(replacement_session)
        .expect("a closed binding should permit a fresh replacement transport");
}

#[test]
fn abandoned_handshakes_are_ttl_pruned_and_globally_bounded() {
    let data_dir = tempdir().expect("temporary node directory");
    let governance_key = signing_key(70);
    let node_a_identity = signing_key(71);
    let node_a_validator = signing_key(72);
    let node_b_identity = signing_key(73);
    let node_b_validator = signing_key(74);
    let node_a_id = Uuid::from_u128(70);
    let node_b_id = Uuid::from_u128(71);
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");

    let started_at = 16_000_000;
    let mut pending_transports = Vec::with_capacity(MAX_PENDING_PEER_HANDSHAKES);
    for offset in 0..MAX_PENDING_PEER_HANDSHAKES {
        let transport = node_a.quarantine_peer_transport();
        node_a
            .begin_peer_handshake_at(&transport, node_b_id, started_at + offset as u64)
            .expect("pending handshake below the global cap");
        pending_transports.push(transport);
    }
    let saturated_transport = node_a.quarantine_peer_transport();
    assert_eq!(
        node_a
            .begin_peer_handshake_at(
                &saturated_transport,
                node_b_id,
                started_at + MAX_PENDING_PEER_HANDSHAKES as u64,
            )
            .expect_err("pending handshakes must have a global cap"),
        PeerSessionError::HandshakeCapacityReached
    );

    let replacement_transport = node_a.quarantine_peer_transport();
    node_a
        .begin_peer_handshake_at(
            &replacement_transport,
            node_b_id,
            started_at + PEER_HANDSHAKE_TIMEOUT_MILLIS + 1,
        )
        .expect("expired pending handshakes must be pruned before capacity checks");
}

#[test]
fn peer_authentication_does_not_grant_validator_signer_authorization() {
    let node_a_dir = tempdir().expect("temporary node A directory");
    let node_b_dir = tempdir().expect("temporary node B directory");
    let node_c_dir = tempdir().expect("temporary node C directory");
    let governance_key = signing_key(50);
    let node_a_identity = signing_key(51);
    let node_a_validator = signing_key(52);
    let node_b_identity = signing_key(53);
    let node_b_validator = signing_key(54);
    let node_c_identity = signing_key(55);
    let node_a_id = Uuid::from_u128(50);
    let node_b_id = Uuid::from_u128(51);
    let node_c_id = Uuid::from_u128(52);
    let peer_only_member = NetworkMember {
        node_id: node_c_id,
        identity_public_key: node_c_identity.verifying_key().to_bytes(),
        roles: vec![MemberRole::Peer],
        status: MemberStatus::Active,
        validator_public_key: None,
    };
    let manifest = signed_manifest(
        &governance_key,
        vec![
            member(node_a_id, &node_a_identity, &node_a_validator),
            member(node_b_id, &node_b_identity, &node_b_validator),
            peer_only_member,
        ],
    );
    let profile = network_profile(
        &manifest,
        vec![
            hex::encode(node_a_validator.verifying_key().to_bytes()),
            hex::encode(node_b_validator.verifying_key().to_bytes()),
        ],
    );
    let mut node_a = ReferenceNode::start_in_dir(
        node_a_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_a_id, MemberRole::Validator, node_a_identity),
    )
    .expect("start node A");
    let mut node_b = ReferenceNode::start_in_dir(
        node_b_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile.clone(),
        manifest.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_b_id, MemberRole::Validator, node_b_identity.clone()),
    )
    .expect("start node B");
    let mut node_c = ReferenceNode::start_in_dir(
        node_c_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(), semantic_package()),
        profile,
        manifest,
        governance_key.verifying_key(),
        NodeIdentity::new(node_c_id, MemberRole::Peer, node_c_identity),
    )
    .expect("start peer-only node C");

    let mut transport_a_b = node_a.quarantine_peer_transport();
    let mut transport_b_a = node_b.quarantine_peer_transport();
    let (session_a_b, session_b_a) = complete_handshake(
        &mut node_a,
        &transport_a_b,
        &mut node_b,
        &transport_b_a,
        node_b_id,
        10_000_000,
    );
    assert_eq!(
        node_a
            .authorize_peer_signer(&transport_a_b, node_b_validator.verifying_key().to_bytes())
            .expect_err("signer checks require an authenticated transport binding"),
        SignerAuthorizationError::UnauthenticatedTransport
    );
    transport_a_b
        .bind_authenticated_session(session_a_b)
        .expect("bind validator peer session at node A");
    transport_b_a
        .bind_authenticated_session(session_b_a)
        .expect("bind validator peer session at node B");

    assert_eq!(
        node_a
            .authorize_peer_signer(&transport_a_b, node_b_identity.verifying_key().to_bytes())
            .expect_err("Node Identity Key must not act as a PoA key"),
        SignerAuthorizationError::ValidatorKeyMismatch(node_b_id)
    );
    let authorization = node_a
        .authorize_peer_signer(&transport_a_b, node_b_validator.verifying_key().to_bytes())
        .expect("separate active validator key should authorize the signer role");
    assert_eq!(authorization.node_id(), node_b_id);
    assert_eq!(
        authorization.validator_public_key(),
        node_b_validator.verifying_key().to_bytes()
    );
    assert_eq!(
        authorization.manifest_digest(),
        node_a.membership_manifest_digest()
    );

    let mut transport_a_c = node_a.quarantine_peer_transport();
    let mut transport_c_a = node_c.quarantine_peer_transport();
    let (session_a_c, session_c_a) = complete_handshake(
        &mut node_a,
        &transport_a_c,
        &mut node_c,
        &transport_c_a,
        node_c_id,
        11_000_000,
    );
    transport_a_c
        .bind_authenticated_session(session_a_c)
        .expect("bind peer-only session at node A");
    transport_c_a
        .bind_authenticated_session(session_c_a)
        .expect("bind peer-only session at node C");
    assert_eq!(
        node_a
            .authorize_peer_signer(&transport_a_c, signing_key(99).verifying_key().to_bytes())
            .expect_err("authenticated peer-only member is not a validator"),
        SignerAuthorizationError::ValidatorRoleInactive(node_c_id)
    );
}
