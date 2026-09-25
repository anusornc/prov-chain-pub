//! Acceptance tests for package-selected semantic Final Admission (Issue #7).

use ed25519_dalek::SigningKey;
use oxigraph::model::{GraphName, NamedNode, Quad};
use provchain_org::ledger::{
    AdmissionCandidate, AdmissionOutcome, AdmittedBlockEnvelope, Ledger, LedgerError, LedgerProfile,
};
use provchain_org::network::convergence::CommittedRangeRequest;
use provchain_org::network::membership::{
    MemberRole, MemberStatus, MembershipManifest, NetworkMember, NodeIdentity,
    SignedMembershipManifest,
};
use provchain_org::network::peer_session::{AuthenticatedPeerSession, PeerTransport};
use provchain_org::network::poa::{PoAError, PoAProposalRequest};
use provchain_org::network::profile::{ConsensusProfile, NetworkProfile, SemanticProfile};
use provchain_org::network::reference::{ReferenceLedgerActivation, ReferenceNode};
use provchain_org::ontology::{
    ActivatedSemanticPackage, OntologyPackageManifest, SemanticAdmissionError,
    SEMANTIC_EXECUTION_PROFILE_V1,
};
use std::fs;
use std::path::Path;
use tempfile::{tempdir, TempDir};
use uuid::Uuid;

const ONTOLOGY: &str = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
"#;

const SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
    sh:targetClass ex:Record ;
    sh:property [ sh:path ex:identifier ; sh:minCount 1 ; sh:maxCount 1 ] .
"#;

fn write(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write semantic fixture");
}

fn manifest_in(directory: &Path, shapes: &str) -> OntologyPackageManifest {
    manifest_with_ontology(directory, ONTOLOGY, shapes)
}

fn manifest_with_ontology(
    directory: &Path,
    ontology: &str,
    shapes: &str,
) -> OntologyPackageManifest {
    let core_ontology = directory.join("core.ttl");
    let domain_ontology = directory.join("domain.ttl");
    let core_shapes = directory.join("core-shapes.ttl");
    let domain_shapes = directory.join("domain-shapes.ttl");
    write(&core_ontology, ontology);
    write(&domain_ontology, ontology);
    write(&core_shapes, shapes);
    write(&domain_shapes, shapes);

    OntologyPackageManifest {
        package_id: "provchain.issue7.fixture".to_string(),
        package_version: "1.0.0".to_string(),
        core_ontology_path: core_ontology.to_string_lossy().into_owned(),
        domain_ontology_path: domain_ontology.to_string_lossy().into_owned(),
        core_shacl_path: core_shapes.to_string_lossy().into_owned(),
        domain_shacl_path: domain_shapes.to_string_lossy().into_owned(),
        semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
        mappings: vec![],
        validation_mode: "strict".to_string(),
        package_hash: None,
    }
}

fn ledger_profile(package: &ActivatedSemanticPackage) -> LedgerProfile {
    LedgerProfile::new("provchain.issue7", "issue7.reference").with_semantic_package(
        package.package_id(),
        package.package_version(),
        package.package_hash(),
    )
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[17; 32])
}

fn seeded_signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn candidate(ledger: &Ledger, payload: impl AsRef<[u8]>, timestamp: u64) -> AdmissionCandidate {
    ledger
        .create_ordinary_candidate(payload, timestamp, &signing_key())
        .expect("create semantic admission candidate")
}

fn committed(outcome: AdmissionOutcome) -> AdmittedBlockEnvelope {
    match outcome {
        AdmissionOutcome::Committed { envelope } => *envelope,
        AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
    }
}

fn rejection(outcome: AdmissionOutcome) -> String {
    match outcome {
        AdmissionOutcome::Rejected { reason } => reason,
        AdmissionOutcome::Committed { .. } => panic!("candidate unexpectedly committed"),
    }
}

fn semantic_member(
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

fn semantic_membership(
    governance_key: &SigningKey,
    members: Vec<NetworkMember>,
) -> SignedMembershipManifest {
    MembershipManifest {
        manifest_id: "issue7.three-node.membership".to_string(),
        version: 1,
        network_id: "provchain.issue7".to_string(),
        network_profile_id: "issue7.reference".to_string(),
        members,
    }
    .sign(governance_key)
    .expect("sign three-node membership")
}

fn semantic_network_profile(
    package: &ActivatedSemanticPackage,
    membership: &SignedMembershipManifest,
    authority_keys: &[SigningKey],
) -> NetworkProfile {
    NetworkProfile {
        profile_id: "issue7.reference".to_string(),
        network_id: "provchain.issue7".to_string(),
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
        membership: Some(membership.binding()),
        privacy: None,
        bridge: None,
        bridge_source_trust: None,
    }
}

struct SemanticTestNode {
    _data_dir: TempDir,
    node: ReferenceNode,
}

fn start_semantic_validator(
    package: &ActivatedSemanticPackage,
    governance_key: &SigningKey,
    membership: &SignedMembershipManifest,
    profile: &NetworkProfile,
    node_id: Uuid,
    identity_key: &SigningKey,
    validator_key: &SigningKey,
) -> SemanticTestNode {
    let data_dir = tempdir().expect("semantic validator directory");
    let node = ReferenceNode::start_validator_in_dir(
        data_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(package), package.clone()),
        profile.clone(),
        membership.clone(),
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key.clone()),
        validator_key.clone(),
    )
    .expect("start semantic reference validator");
    SemanticTestNode {
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
        .expect("answer peer challenge");
    let responder_session = responder
        .finish_peer_handshake_at(responder_transport, response, started_at_millis + 3)
        .expect("finish peer handshake");
    (initiator_session, responder_session)
}

#[test]
fn package_identity_is_path_independent_and_activation_binds_executable_assets() {
    let first_dir = tempdir().expect("first fixture directory");
    let second_dir = tempdir().expect("second fixture directory");
    let first = manifest_in(first_dir.path(), SHAPES);
    let second = manifest_in(second_dir.path(), SHAPES);

    assert_eq!(
        first.compute_package_hash().expect("first package digest"),
        second
            .compute_package_hash()
            .expect("second package digest"),
        "local resolution paths must not enter package identity"
    );

    let activated = ActivatedSemanticPackage::activate(&first).expect("activate package");
    assert_eq!(activated.package_id(), first.package_id);
    assert_eq!(
        activated.semantic_execution_profile_id(),
        SEMANTIC_EXECUTION_PROFILE_V1
    );
    assert_eq!(
        activated.package_hash(),
        first.compute_package_hash().expect("bound package digest")
    );
}

#[test]
fn blank_node_labels_are_document_local_across_shape_artifacts() {
    let fixture_dir = tempdir().expect("fixture directory");
    let core_shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ; sh:targetClass ex:Record ; sh:property _:same .
_:same sh:path ex:identifier ; sh:minCount 1 .
"#;
    let domain_shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:OtherShape a sh:NodeShape ; sh:targetClass ex:Other ; sh:property _:same .
_:same sh:path ex:name ; sh:minCount 1 .
"#;
    let manifest = manifest_in(fixture_dir.path(), core_shapes);
    write(Path::new(&manifest.domain_shacl_path), domain_shapes);
    ActivatedSemanticPackage::activate(&manifest)
        .expect("identical source labels in separate shape documents must not merge");
}

#[test]
fn checked_in_default_package_activates_under_its_declared_profile() {
    let manifest = OntologyPackageManifest::load_from_file("config/ontology_package.toml")
        .expect("load checked-in package manifest");
    let package = ActivatedSemanticPackage::activate(&manifest)
        .expect("activate checked-in semantic package");
    assert_eq!(
        package.package_hash(),
        "70e4fdd190887944531a9c984020af6fc0afeb7aff3a6a26972dbfd6d2ae9a9f"
    );
}

#[test]
fn semantic_execution_profile_selection_must_be_explicit_in_both_manifests() {
    let fixture_dir = tempdir().expect("fixture directory");
    let package_manifest = manifest_in(fixture_dir.path(), SHAPES);
    let package_toml = toml::to_string(&package_manifest).expect("serialize package manifest");
    let package_without_execution_profile = package_toml
        .lines()
        .filter(|line| !line.starts_with("semantic_execution_profile_id"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(toml::from_str::<OntologyPackageManifest>(&package_without_execution_profile).is_err());

    let package_hash = package_manifest
        .compute_package_hash()
        .expect("package digest");
    let network_profile = NetworkProfile {
        profile_id: "issue7.reference".to_string(),
        network_id: "provchain.issue7".to_string(),
        consensus: ConsensusProfile::default(),
        semantic: SemanticProfile {
            ontology_package_id: package_manifest.package_id.clone(),
            ontology_package_version: package_manifest.package_version.clone(),
            ontology_package_hash: package_hash,
            semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
            validation_mode: "strict".to_string(),
        },
        membership: None,
        privacy: None,
        bridge: None,
        bridge_source_trust: None,
    };
    let network_toml = toml::to_string(&network_profile).expect("serialize network profile");
    let network_without_execution_profile = network_toml
        .lines()
        .filter(|line| !line.starts_with("semantic_execution_profile_id"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(toml::from_str::<NetworkProfile>(&network_without_execution_profile).is_err());
}

#[test]
fn unsupported_validating_construct_fails_package_activation() {
    let fixture_dir = tempdir().expect("fixture directory");
    let unsupported = SHAPES.replace(
        "sh:targetClass ex:Record ;",
        "sh:targetClass ex:Record ; sh:closed true ;",
    );
    let manifest = manifest_in(fixture_dir.path(), &unsupported);

    let error = ActivatedSemanticPackage::activate(&manifest)
        .expect_err("unsupported sh:closed must fail closed");
    assert!(error.to_string().contains("unsupported SHACL predicate"));
}

#[test]
fn declared_constraints_cannot_be_silently_dropped_from_the_execution_plan() {
    let fixture_dir = tempdir().expect("fixture directory");
    let unattached = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ; sh:targetClass ex:Record .
ex:Orphan a sh:PropertyShape ; sh:path ex:identifier ; sh:minCount 1 .
"#;
    let manifest = manifest_in(fixture_dir.path(), unattached);
    let error = ActivatedSemanticPackage::activate(&manifest)
        .expect_err("unattached constraints must not be ignored");
    assert!(error.to_string().contains("property shape"));
}

#[test]
fn custom_constraints_and_ill_typed_profile_parameters_fail_activation() {
    let custom_dir = tempdir().expect("custom constraint directory");
    let custom = SHAPES.replace(
        "sh:minCount 1 ;",
        "sh:minCount 1 ; ex:customConstraint true ;",
    );
    ActivatedSemanticPackage::activate(&manifest_in(custom_dir.path(), &custom))
        .expect_err("a custom property-shape constraint must not be ignored");

    let count_dir = tempdir().expect("ill-typed count directory");
    let ill_typed = SHAPES.replace("sh:minCount 1 ;", "sh:minCount \"1\" ;");
    ActivatedSemanticPackage::activate(&manifest_in(count_dir.path(), &ill_typed))
        .expect_err("sh:minCount must use an xsd:integer parameter");

    let pattern_dir = tempdir().expect("ill-typed pattern directory");
    let ill_typed = SHAPES.replace(
        "sh:minCount 1 ; sh:maxCount 1",
        "sh:minCount 1 ; sh:maxCount 1 ; sh:pattern \"identifier\"@en",
    );
    ActivatedSemanticPackage::activate(&manifest_in(pattern_dir.path(), &ill_typed))
        .expect_err("sh:pattern must use an xsd:string parameter");
}

#[test]
fn missing_empty_malformed_or_wrong_digest_assets_fail_activation() {
    let empty_dir = tempdir().expect("empty shapes directory");
    let empty = manifest_in(empty_dir.path(), SHAPES);
    write(Path::new(&empty.core_shacl_path), "");
    ActivatedSemanticPackage::activate(&empty).expect_err("empty shapes must fail activation");

    let malformed_dir = tempdir().expect("malformed shapes directory");
    let malformed = manifest_in(malformed_dir.path(), SHAPES);
    write(
        Path::new(&malformed.domain_shacl_path),
        "not valid Turtle {",
    );
    ActivatedSemanticPackage::activate(&malformed)
        .expect_err("malformed shapes must fail activation");

    let missing_dir = tempdir().expect("missing ontology directory");
    let mut missing = manifest_in(missing_dir.path(), SHAPES);
    missing.domain_ontology_path = missing_dir
        .path()
        .join("absent.ttl")
        .to_string_lossy()
        .into_owned();
    ActivatedSemanticPackage::activate(&missing)
        .expect_err("missing ontology must fail activation");

    let empty_ontology_dir = tempdir().expect("empty ontology directory");
    let prefix_only = manifest_with_ontology(
        empty_ontology_dir.path(),
        "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n",
        SHAPES,
    );
    ActivatedSemanticPackage::activate(&prefix_only)
        .expect_err("an ontology artifact without executable statements must fail activation");

    let digest_dir = tempdir().expect("wrong digest directory");
    let mut wrong_digest = manifest_in(digest_dir.path(), SHAPES);
    wrong_digest.package_hash = Some("00".repeat(32));
    ActivatedSemanticPackage::activate(&wrong_digest)
        .expect_err("wrong package digest must fail activation");
}

#[test]
fn pattern_uses_xpath_regex_and_the_sparql_string_form_of_iris() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property [
    sh:path ex:identifier ;
    sh:pattern "^http://example.org/\\i\\c*$"
  ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), shapes))
        .expect("activate XPath-regex package");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open XPath-regex ledger");

    let valid_iri = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier ex:Alpha_1 ."#,
        900,
    );
    committed(
        ledger
            .local_adapter()
            .submit(valid_iri)
            .expect("IRI values use their SPARQL STR form"),
    );

    let invalid_iri = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:identifier <http://example.org/1invalid> ."#,
        901,
    );
    let reason = rejection(
        ledger
            .local_adapter()
            .submit(invalid_iri)
            .expect("nonmatching IRI is a deterministic verdict"),
    );
    assert!(reason.contains("pattern"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 1);
}

#[test]
fn final_admission_validates_full_staged_union_and_requires_candidate_focus() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let mut ledger =
        Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package.clone())
            .expect("open semantic ledger");

    let first = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier "A-1" ."#,
        1,
    );
    committed(ledger.local_adapter().submit(first).expect("admit first"));

    let extension = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:note "extension without repeated type" ."#,
        2,
    );
    committed(
        ledger
            .api_adapter()
            .submit(extension)
            .expect("admit cross-block extension"),
    );

    let duplicate = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:identifier "A-1" ."#,
        3,
    );
    committed(
        ledger
            .batch_adapter()
            .submit(duplicate)
            .expect("collapse duplicate in validation union"),
    );

    let later_violation = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:identifier "A-2" ."#,
        4,
    );
    let reason = rejection(
        ledger
            .proposal_adapter()
            .submit(later_violation)
            .expect("deterministic rejection"),
    );
    assert!(reason.contains("maxCount"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 3);

    let object_only = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:unrelated ex:references ex:record ."#,
        5,
    );
    let reason = rejection(
        ledger
            .bridge_origin_adapter()
            .submit(object_only)
            .expect("object-only rejection"),
    );
    assert!(reason.contains("no package-selected focus-node subject"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 3);
}

#[test]
fn restart_reconstructs_semantic_parent_state_from_verified_journal_history() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let profile = ledger_profile(&package);

    {
        let mut ledger = Ledger::open_in_dir(ledger_dir.path(), profile.clone(), package.clone())
            .expect("open initial ledger");
        let first = candidate(
            &ledger,
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier "A-1" ."#,
            6,
        );
        committed(ledger.local_adapter().submit(first).expect("commit parent"));
    }

    let mut restarted = Ledger::open_in_dir(ledger_dir.path(), profile, package)
        .expect("reopen from verified journal");
    let extension = candidate(
        &restarted,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:note "recognized after restart" ."#,
        7,
    );
    committed(
        restarted
            .synchronization_adapter()
            .submit(extension)
            .expect("admit extension using replayed parent"),
    );
    let violation = candidate(
        &restarted,
        br#"@prefix ex: <http://example.org/> .
ex:record ex:identifier "A-2" ."#,
        8,
    );
    let reason = rejection(
        restarted
            .replication_adapter()
            .submit(violation)
            .expect("reject against replayed full state"),
    );
    assert!(reason.contains("maxCount"));
    assert_eq!(restarted.journal_frame_count().expect("journal frames"), 2);
}

#[test]
fn three_reference_nodes_agree_after_independent_admission_and_exact_catch_up() {
    let package_dir = tempdir().expect("package directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let governance_key = seeded_signing_key(50);
    let identity_keys = [
        seeded_signing_key(51),
        seeded_signing_key(52),
        seeded_signing_key(53),
    ];
    let validator_keys = [
        seeded_signing_key(54),
        seeded_signing_key(55),
        seeded_signing_key(56),
    ];
    let node_ids = [
        Uuid::from_u128(7_100),
        Uuid::from_u128(7_200),
        Uuid::from_u128(7_300),
    ];
    let membership = semantic_membership(
        &governance_key,
        node_ids
            .iter()
            .zip(identity_keys.iter())
            .zip(validator_keys.iter())
            .map(|((&node_id, identity_key), validator_key)| {
                semantic_member(node_id, identity_key, validator_key)
            })
            .collect(),
    );
    let profile = semantic_network_profile(&package, &membership, &validator_keys);
    let mut nodes: Vec<_> = (0..3)
        .map(|index| {
            start_semantic_validator(
                &package,
                &governance_key,
                &membership,
                &profile,
                node_ids[index],
                &identity_keys[index],
                &validator_keys[index],
            )
        })
        .collect();

    let builder_dir = tempdir().expect("candidate builder directory");
    let mut builder = Ledger::open_in_dir(
        builder_dir.path(),
        ledger_profile(&package),
        package.clone(),
    )
    .expect("open candidate builder");
    let valid = builder
        .create_ordinary_candidate(
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier "A-1" ."#,
            1_000,
            &validator_keys[0],
        )
        .expect("build height-zero candidate");
    committed(
        builder
            .proposal_adapter()
            .submit(valid.clone())
            .expect("advance candidate builder"),
    );

    for node in nodes.iter().take(2) {
        assert!(node
            .node
            .accept_signed_poa_proposal(valid.clone())
            .expect("independent semantic acceptance")
            .is_committed());
    }
    let canonical_record = nodes[0]
        .node
        .committed_envelope_record(0)
        .expect("first node record");
    assert_eq!(
        nodes[1]
            .node
            .committed_envelope_record(0)
            .expect("second node record"),
        canonical_record
    );

    {
        let (first_two, third) = nodes.split_at_mut(2);
        let producer = &mut first_two[0].node;
        let follower = &mut third[0].node;
        let mut producer_transport = producer.quarantine_peer_transport();
        let mut follower_transport = follower.quarantine_peer_transport();
        let (producer_session, follower_session) = complete_handshake(
            producer,
            &producer_transport,
            follower,
            &follower_transport,
            node_ids[2],
            10_000,
        );
        producer_transport
            .bind_authenticated_session(producer_session)
            .expect("bind producer session");
        follower_transport
            .bind_authenticated_session(follower_session)
            .expect("bind follower session");
        let checkpoint = follower
            .ledger_prefix_checkpoint()
            .expect("empty follower checkpoint");
        let request = CommittedRangeRequest::new(checkpoint, 1).expect("bounded catch-up request");
        let range = producer
            .export_committed_range(&mut producer_transport, request)
            .expect("export exact semantic range");
        follower
            .import_committed_range(&mut follower_transport, request, range)
            .expect("catch up through semantic Final Admission");
    }
    assert_eq!(
        nodes[2]
            .node
            .committed_envelope_record(0)
            .expect("caught-up node record"),
        canonical_record
    );

    let invalid = builder
        .create_ordinary_candidate(
            br#"@prefix ex: <http://example.org/> .
ex:record ex:identifier "A-2" ."#,
            11_000,
            &validator_keys[1],
        )
        .expect("build height-one violation");
    let mut reasons = Vec::new();
    for node in &nodes {
        let error = node
            .node
            .accept_signed_poa_proposal(invalid.clone())
            .expect_err("all nodes reject the same semantic transition");
        match error {
            PoAError::FinalAdmissionRejected { reason, .. } => reasons.push(reason),
            other => panic!("expected semantic Final Admission rejection, got {other:?}"),
        }
        assert_eq!(node.node.journal_frame_count().expect("journal frames"), 1);
    }
    assert!(reasons.iter().all(|reason| reason.contains("maxCount")));
    assert!(reasons.windows(2).all(|pair| pair[0] == pair[1]));
}

#[derive(Clone, Copy)]
enum Ingress {
    Local,
    Api,
    Batch,
    Synchronization,
    Replication,
    Proposal,
    BridgeOrigin,
}

impl Ingress {
    const ALL: [Self; 7] = [
        Self::Local,
        Self::Api,
        Self::Batch,
        Self::Synchronization,
        Self::Replication,
        Self::Proposal,
        Self::BridgeOrigin,
    ];

    fn submit(
        self,
        ledger: &mut Ledger,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, LedgerError> {
        match self {
            Self::Local => ledger.local_adapter().submit(candidate),
            Self::Api => ledger.api_adapter().submit(candidate),
            Self::Batch => ledger.batch_adapter().submit(candidate),
            Self::Synchronization => ledger.synchronization_adapter().submit(candidate),
            Self::Replication => ledger.replication_adapter().submit(candidate),
            Self::Proposal => ledger.proposal_adapter().submit(candidate),
            Self::BridgeOrigin => ledger.bridge_origin_adapter().submit(candidate),
        }
    }
}

#[test]
fn every_ordinary_ingress_observes_identical_semantic_verdict_and_envelope() {
    let package_dir = tempdir().expect("package directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let mut canonical = Vec::new();

    for ingress in Ingress::ALL {
        let ledger_dir = tempdir().expect("ledger directory");
        let mut ledger =
            Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package.clone())
                .expect("open ledger");
        let valid = candidate(
            &ledger,
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:identifier "A-1" ."#,
            10,
        );
        let envelope = committed(ingress.submit(&mut ledger, valid).expect("valid verdict"));
        canonical.push(envelope.canonical_bytes().expect("canonical envelope"));
    }
    assert!(canonical.windows(2).all(|pair| pair[0] == pair[1]));

    let mut expected_reason = None;
    for ingress in Ingress::ALL {
        let ledger_dir = tempdir().expect("ledger directory");
        let mut ledger =
            Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package.clone())
                .expect("open ledger");
        let invalid = candidate(
            &ledger,
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ."#,
            11,
        );
        let reason = rejection(
            ingress
                .submit(&mut ledger, invalid)
                .expect("deterministic semantic verdict"),
        );
        assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
        if let Some(expected) = &expected_reason {
            assert_eq!(&reason, expected);
        } else {
            expected_reason = Some(reason);
        }
    }
}

#[test]
fn semantic_rejection_issues_neither_journal_frame_nor_commit_receipt() {
    let package_dir = tempdir().expect("package directory");
    let node_dir = tempdir().expect("node directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let governance_key = seeded_signing_key(40);
    let identity_key = seeded_signing_key(41);
    let validator_key = seeded_signing_key(42);
    let node_id = Uuid::from_u128(700);
    let membership = MembershipManifest {
        manifest_id: "issue7.membership".to_string(),
        version: 1,
        network_id: "provchain.issue7".to_string(),
        network_profile_id: "issue7.reference".to_string(),
        members: vec![NetworkMember {
            node_id,
            identity_public_key: identity_key.verifying_key().to_bytes(),
            roles: vec![MemberRole::Peer, MemberRole::Validator],
            status: MemberStatus::Active,
            validator_public_key: Some(validator_key.verifying_key().to_bytes()),
        }],
    }
    .sign(&governance_key)
    .expect("sign membership");
    let network_profile = NetworkProfile {
        profile_id: "issue7.reference".to_string(),
        network_id: "provchain.issue7".to_string(),
        consensus: ConsensusProfile {
            consensus_type: "poa".to_string(),
            authority_keys: vec![hex::encode(validator_key.verifying_key().to_bytes())],
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
        membership: Some(membership.binding()),
        privacy: None,
        bridge: None,
        bridge_source_trust: None,
    };
    let node = ReferenceNode::start_validator_in_dir(
        node_dir.path(),
        ReferenceLedgerActivation::new(ledger_profile(&package), package),
        network_profile,
        membership,
        governance_key.verifying_key(),
        NodeIdentity::new(node_id, MemberRole::Validator, identity_key),
        validator_key,
    )
    .expect("start semantic validator");
    let turn = node.pending_poa_turn().expect("pending PoA turn");
    let error = node
        .submit_poa_request(PoAProposalRequest::ordinary(
            turn,
            12,
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ."#,
        ))
        .expect_err("semantic violation must reject the signed proposal");
    assert!(matches!(
        error,
        PoAError::FinalAdmissionRejected { reason, .. } if reason.contains("minCount")
    ));
    assert_eq!(node.journal_frame_count().expect("journal frames"), 0);
    assert!(node.local_commit_receipt(0).is_err());
}

#[test]
fn subclass_entailment_selects_focus_and_evaluates_class_constraints() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
ex:SpecialRecord a owl:Class ; rdfs:subClassOf ex:Record .
ex:Agent a owl:Class .
ex:SpecialAgent a owl:Class ; rdfs:subClassOf ex:Agent .
"#;
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
    sh:targetClass ex:Record ;
    sh:property [ sh:path ex:related ; sh:class ex:Agent ; sh:minCount 1 ] .
"#;
    let manifest = manifest_with_ontology(package_dir.path(), ontology, shapes);
    let package = ActivatedSemanticPackage::activate(&manifest).expect("activate package");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open ledger");
    let valid = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:SpecialRecord ; ex:related ex:agent .
ex:agent a ex:SpecialAgent ."#,
        20,
    );
    committed(ledger.local_adapter().submit(valid).expect("SPACL verdict"));
}

#[test]
fn unknown_asserted_classes_are_deterministic_nonconformance_not_engine_failure() {
    let package_dir = tempdir().expect("package directory");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
ex:Agent a owl:Class .
"#;
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property [ sh:path ex:related ; sh:class ex:Agent ; sh:minCount 1 ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_with_ontology(
        package_dir.path(),
        ontology,
        shapes,
    ))
    .expect("activate package");

    let unrelated_dir = tempdir().expect("unrelated ledger directory");
    let mut unrelated = Ledger::open_in_dir(
        unrelated_dir.path(),
        ledger_profile(&package),
        package.clone(),
    )
    .expect("open unrelated ledger");
    let unknown_focus = candidate(
        &unrelated,
        br#"@prefix ex: <http://example.org/> .
ex:unknown a ex:Unknown ."#,
        23,
    );
    let reason = rejection(
        unrelated
            .local_adapter()
            .submit(unknown_focus)
            .expect("unknown target class yields a deterministic verdict"),
    );
    assert!(reason.contains("no package-selected focus-node subject"));

    let class_dir = tempdir().expect("class ledger directory");
    let mut class_ledger = Ledger::open_in_dir(class_dir.path(), ledger_profile(&package), package)
        .expect("open class ledger");
    let unknown_value_class = candidate(
        &class_ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:related ex:unknown .
ex:unknown a ex:Unknown ."#,
        24,
    );
    let reason = rejection(
        class_ledger
            .local_adapter()
            .submit(unknown_value_class)
            .expect("unknown value class yields a deterministic verdict"),
    );
    assert!(reason.contains("class"));
}

#[test]
fn semantic_data_graph_excludes_package_artifacts_and_inferred_triples() {
    let exclusion_package_dir = tempdir().expect("exclusion package directory");
    let exclusion_ledger_dir = tempdir().expect("exclusion ledger directory");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
sh:NodeShape a owl:Class .
"#;
    let shapes = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:DeclaredClassShape a sh:NodeShape ;
  sh:targetClass owl:Class ;
  sh:property [ sh:path ex:identifier ; sh:minCount 1 ] .
ex:ShapeArtifactShape a sh:NodeShape ;
  sh:targetClass sh:NodeShape ;
  sh:property [ sh:path ex:marker ; sh:minCount 1 ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_with_ontology(
        exclusion_package_dir.path(),
        ontology,
        shapes,
    ))
    .expect("activate exclusion package");
    let mut ledger = Ledger::open_in_dir(
        exclusion_ledger_dir.path(),
        ledger_profile(&package),
        package,
    )
    .expect("open exclusion ledger");
    let candidate_class = candidate(
        &ledger,
        br#"@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .
ex:CandidateClass a owl:Class ; ex:identifier "declared" ."#,
        21,
    );
    committed(
        ledger
            .local_adapter()
            .submit(candidate_class)
            .expect("package artifacts stay outside asserted state"),
    );

    let inference_package_dir = tempdir().expect("inference package directory");
    let inference_ledger_dir = tempdir().expect("inference ledger directory");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
ex:SpecialRecord a owl:Class ; rdfs:subClassOf ex:Record .
"#;
    let shapes = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property [ sh:path rdf:type ; sh:hasValue ex:Record ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_with_ontology(
        inference_package_dir.path(),
        ontology,
        shapes,
    ))
    .expect("activate inference package");
    let mut ledger = Ledger::open_in_dir(
        inference_ledger_dir.path(),
        ledger_profile(&package),
        package,
    )
    .expect("open inference ledger");
    let subclass_only = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:SpecialRecord ."#,
        22,
    );
    let reason = rejection(
        ledger
            .local_adapter()
            .submit(subclass_only)
            .expect("inference remains a validation view"),
    );
    assert!(reason.contains("hasValue"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
}

#[test]
fn document_local_blank_nodes_do_not_merge_across_staged_graphs() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open ledger");

    for (timestamp, identifier) in [(30, "A-1"), (31, "A-2")] {
        let payload = format!(
            "@prefix ex: <http://example.org/> .\n_:same a ex:Record ; ex:identifier \"{identifier}\" ."
        );
        let next = candidate(&ledger, payload, timestamp);
        committed(
            ledger
                .local_adapter()
                .submit(next)
                .expect("blank-node scoped admission"),
        );
    }
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 2);
}

#[test]
fn datatype_rejects_ill_typed_sparql_supported_literals() {
    let cases = [
        ("integer", "not-an-integer"),
        ("unsignedByte", "256"),
        ("double", "not-a-double"),
        ("boolean", "maybe"),
        ("dateTime", "2026-99-99T25:61:61Z"),
        ("string", "\\u0000"),
    ];

    for (index, (datatype, lexical)) in cases.into_iter().enumerate() {
        let package_dir = tempdir().expect("package directory");
        let shapes = format!(
            r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
    sh:targetClass ex:Record ;
    sh:property [ sh:path ex:value ; sh:datatype xsd:{datatype} ] .
"#
        );
        let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), &shapes))
            .expect("activate datatype package");
        let ledger_dir = tempdir().expect("ledger directory");
        let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
            .expect("open datatype ledger");
        let invalid = candidate(
            &ledger,
            format!(
                r#"@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:value "{lexical}"^^xsd:{datatype} ."#
            ),
            200 + index as u64,
        );

        let reason = rejection(
            ledger
                .local_adapter()
                .submit(invalid)
                .expect("ill-typed datatype rejection"),
        );
        assert!(
            reason.contains("datatype"),
            "expected datatype violation for xsd:{datatype}, got {reason}"
        );
        assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
    }
}

#[test]
fn supported_constraint_matrix_uses_rdf_terms_and_pinned_comparisons() {
    let package_dir = tempdir().expect("package directory");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .
ex:Record a owl:Class .
ex:Agent a owl:Class .
"#;
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:name "bounded constraint fixture" ;
  sh:message "diagnostic only" ;
  sh:description "covers every Issue 7 constraint" ;
  sh:property
    [ sh:path ex:identifier ; sh:minCount 1 ; sh:maxCount 1 ;
      sh:datatype xsd:string ; sh:pattern "^[A-Z]-[0-9]+$" ],
    [ sh:path ex:related ; sh:minCount 1 ; sh:class ex:Agent ],
    [ sh:path ex:status ; sh:in ("open" "closed") ; sh:hasValue "open" ],
    [ sh:path ex:amount ; sh:minInclusive 0 ; sh:maxInclusive 100 ] .
"#;
    let manifest = manifest_with_ontology(package_dir.path(), ontology, shapes);
    let package = ActivatedSemanticPackage::activate(&manifest).expect("activate package");
    let valid_tail = r#"
ex:record ex:related ex:agent ; ex:status "open" ; ex:amount 42 .
ex:agent a ex:Agent ."#;
    let cases = [
        ("minCount", "ex:record a ex:Record ."),
        (
            "maxCount",
            "ex:record a ex:Record ; ex:identifier \"A-1\", \"A-2\" .",
        ),
        ("datatype", "ex:record a ex:Record ; ex:identifier 1 ."),
        ("pattern", "ex:record a ex:Record ; ex:identifier \"bad\" ."),
        (
            "class",
            "ex:record a ex:Record ; ex:identifier \"A-1\" ; ex:related \"literal\" .",
        ),
        (
            "in",
            "ex:record a ex:Record ; ex:identifier \"A-1\" ; ex:status \"other\" .",
        ),
        (
            "hasValue",
            "ex:record a ex:Record ; ex:identifier \"A-1\" ; ex:status \"closed\" .",
        ),
        (
            "minInclusive",
            "ex:record a ex:Record ; ex:identifier \"A-1\" ; ex:amount -1 .",
        ),
        (
            "maxInclusive",
            "ex:record a ex:Record ; ex:identifier \"A-1\" ; ex:amount 101 .",
        ),
    ];

    for (index, (component, fragment)) in cases.into_iter().enumerate() {
        let ledger_dir = tempdir().expect("ledger directory");
        let mut ledger =
            Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package.clone())
                .expect("open ledger");
        let payload = match component {
            "minCount" => format!(
                "@prefix ex: <http://example.org/> .\n{fragment}\n{valid_tail}"
            ),
            "maxCount" | "datatype" | "pattern" => format!(
                "@prefix ex: <http://example.org/> .\n{fragment}\n{valid_tail}"
            ),
            "class" => format!(
                "@prefix ex: <http://example.org/> .\n{fragment}\nex:record ex:status \"open\" ; ex:amount 42 .\nex:agent a ex:Agent ."
            ),
            "in" | "hasValue" => format!(
                "@prefix ex: <http://example.org/> .\n{fragment}\nex:record ex:related ex:agent ; ex:amount 42 .\nex:agent a ex:Agent ."
            ),
            "minInclusive" | "maxInclusive" => format!(
                "@prefix ex: <http://example.org/> .\n{fragment}\nex:record ex:related ex:agent ; ex:status \"open\" .\nex:agent a ex:Agent ."
            ),
            _ => unreachable!(),
        };
        let invalid = candidate(&ledger, payload, 100 + index as u64);
        let reason = rejection(
            ledger
                .local_adapter()
                .submit(invalid)
                .expect("constraint rejection"),
        );
        assert!(
            reason.contains(component),
            "expected {component} violation, got {reason}"
        );
        assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
    }

    let ledger_dir = tempdir().expect("valid ledger directory");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open valid ledger");
    let valid = candidate(
        &ledger,
        format!(
            "@prefix ex: <http://example.org/> .\nex:record a ex:Record ; ex:identifier \"A-1\" .\n{valid_tail}"
        ),
        200,
    );
    committed(ledger.local_adapter().submit(valid).expect("valid matrix"));
}

#[test]
fn integer_range_comparison_does_not_lose_precision() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property [ sh:path ex:amount ; sh:minInclusive 9007199254740993 ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), shapes))
        .expect("activate exact-numeric package");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open ledger");
    let below_bound = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:amount 9007199254740992 ."#,
        300,
    );
    let reason = rejection(
        ledger
            .local_adapter()
            .submit(below_bound)
            .expect("deterministic numeric verdict"),
    );
    assert!(reason.contains("minInclusive"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);

    let comparison_error = candidate(
        &ledger,
        br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:amount "not numeric" ."#,
        301,
    );
    let reason = rejection(
        ledger
            .local_adapter()
            .submit(comparison_error)
            .expect("comparison errors are deterministic violations"),
    );
    assert!(reason.contains("minInclusive"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
}

#[test]
fn in_and_has_value_compare_complete_rdf_terms() {
    for (index, (component, constraint)) in [
        ("in", "sh:in (\"same\"@en)"),
        ("hasValue", "sh:hasValue \"same\"@en"),
    ]
    .into_iter()
    .enumerate()
    {
        let package_dir = tempdir().expect("package directory");
        let ledger_dir = tempdir().expect("ledger directory");
        let shapes = format!(
            r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property [ sh:path ex:value ; {constraint} ] .
"#
        );
        let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), &shapes))
            .expect("activate RDF-term package");
        let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
            .expect("open ledger");
        let same_lexical_different_term = candidate(
            &ledger,
            br#"@prefix ex: <http://example.org/> .
ex:record a ex:Record ; ex:value "same" ."#,
            310 + index as u64,
        );
        let reason = rejection(
            ledger
                .local_adapter()
                .submit(same_lexical_different_term)
                .expect("RDF-term verdict"),
        );
        assert!(reason.contains(component));
    }
}

#[test]
fn temporal_range_comparison_is_pinned_and_complete() {
    let package_dir = tempdir().expect("package directory");
    let shapes = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:RecordShape a sh:NodeShape ;
  sh:targetClass ex:Record ;
  sh:property
    [ sh:path ex:day ; sh:minInclusive "2026-01-01"^^xsd:date ],
    [ sh:path ex:instant ; sh:maxInclusive "2026-01-01T00:00:00Z"^^xsd:dateTime ] .
"#;
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), shapes))
        .expect("activate temporal package");
    for (index, (component, day, instant)) in [
        ("minInclusive", "2025-12-31", "2025-12-31T23:59:59Z"),
        ("maxInclusive", "2026-01-01", "2026-01-01T00:00:01Z"),
    ]
    .into_iter()
    .enumerate()
    {
        let ledger_dir = tempdir().expect("ledger directory");
        let mut ledger =
            Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package.clone())
                .expect("open ledger");
        let payload = format!(
            r#"@prefix ex: <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
ex:record a ex:Record ; ex:day "{day}"^^xsd:date ; ex:instant "{instant}"^^xsd:dateTime ."#
        );
        let invalid = candidate(&ledger, payload, 320 + index as u64);
        let reason = rejection(
            ledger
                .local_adapter()
                .submit(invalid)
                .expect("temporal verdict"),
        );
        assert!(reason.contains(component));
    }
}

#[test]
fn deterministic_focus_work_bound_rejects_without_node_incapacity() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let mut shapes = String::from(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n",
    );
    for index in 0..1_001 {
        shapes.push_str(&format!(
            "ex:Shape{index} a sh:NodeShape ; sh:targetClass ex:Record .\n"
        ));
    }
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), &shapes))
        .expect("activate bounded package");
    let mut ledger = Ledger::open_in_dir(ledger_dir.path(), ledger_profile(&package), package)
        .expect("open ledger");
    let mut payload = String::from("@prefix ex: <http://example.org/> .\n");
    for index in 0..1_000 {
        payload.push_str(&format!("ex:record{index} a ex:Record .\n"));
    }
    let oversized_work = candidate(&ledger, payload, 330);
    let outcome = ledger.local_adapter().submit(oversized_work);
    let reason = rejection(outcome.expect("bound exceedance is a deterministic verdict"));
    assert!(reason.contains("focus selection exceeds profile work bound"));
    assert_eq!(ledger.journal_frame_count().expect("journal frames"), 0);
}

#[test]
fn a_nonconforming_committed_parent_is_node_incapacity_not_candidate_rejection() {
    let package_dir = tempdir().expect("package directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let parent = vec![Quad::new(
        NamedNode::new("http://example.org/record").expect("subject IRI"),
        NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").expect("predicate IRI"),
        NamedNode::new("http://example.org/Record").expect("object IRI"),
        GraphName::DefaultGraph,
    )];
    assert!(matches!(
        package.validate_committed_state(&parent),
        Err(SemanticAdmissionError::NodeIncapacity(_))
    ));
}

#[test]
fn ledger_open_rejects_a_package_or_execution_profile_mismatch() {
    let package_dir = tempdir().expect("package directory");
    let ledger_dir = tempdir().expect("ledger directory");
    let package = ActivatedSemanticPackage::activate(&manifest_in(package_dir.path(), SHAPES))
        .expect("activate package");
    let wrong_hash = LedgerProfile::new("provchain.issue7", "issue7.reference")
        .with_semantic_package(package.package_id(), package.package_version(), "00");
    assert!(matches!(
        Ledger::open_in_dir(ledger_dir.path(), wrong_hash, package.clone()),
        Err(LedgerError::InvalidProfile(_))
    ));

    let wrong_execution = ledger_profile(&package)
        .with_semantic_execution_profile("provchain.semantic-execution.unsupported");
    assert!(matches!(
        Ledger::open_in_dir(ledger_dir.path(), wrong_execution, package),
        Err(LedgerError::InvalidProfile(_))
    ));
}
