use ed25519_dalek::SigningKey;
use provchain_org::ledger::{
    AdmissionCandidate, AdmissionOutcome, AdmittedBlockEnvelope, Ledger, LedgerError,
    LedgerProfile, MAX_PAYLOAD_BYTES,
};
use tempfile::tempdir;

mod support;
use support::semantic::{bind_profile, semantic_package};

fn profile() -> LedgerProfile {
    bind_profile(LedgerProfile::new("provchain.issue2", "issue2.reference"))
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn candidate(ledger: &Ledger, key: &SigningKey) -> provchain_org::ledger::AdmissionCandidate {
    ledger
        .create_ordinary_candidate(
            br#"@prefix ex: <http://example.org/> .
ex:batch ex:identifier "B-1" ."#,
            1_700_000_000_000,
            key,
        )
        .expect("create ordinary candidate")
}

fn submit_local(
    ledger: &mut Ledger,
    candidate: AdmissionCandidate,
) -> Result<AdmissionOutcome, LedgerError> {
    ledger.local_adapter().submit(candidate)
}

#[test]
fn ordinary_candidate_commits_one_canonical_envelope_and_replays_projections() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");

    let candidate = candidate(&ledger, &key);

    let envelope = match submit_local(&mut ledger, candidate)
        .expect("ordinary candidate should be committed")
    {
        AdmissionOutcome::Committed { envelope } => *envelope,
        AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
    };

    assert_eq!(ledger.committed_envelopes().len(), 1);
    assert_eq!(ledger.journal_frame_count().expect("read journal"), 1);

    let canonical_bytes = envelope.canonical_bytes().expect("encode envelope");
    assert_eq!(
        AdmittedBlockEnvelope::decode(&canonical_bytes).expect("decode canonical envelope"),
        envelope
    );

    assert_eq!(ledger.rdf_store().len().expect("count projected RDF"), 1);
    assert_eq!(ledger.envelope_index().len(), 1);

    drop(ledger);
    let reopened =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("reopen ledger");

    assert_eq!(reopened.committed_envelopes(), &[envelope]);
    assert_eq!(reopened.rdf_store().len().expect("count replayed RDF"), 1);
    assert_eq!(reopened.envelope_index().len(), 1);
}

#[test]
fn deterministic_rejections_leave_the_authoritative_journal_unchanged() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");

    let mut cases = Vec::new();
    let mut malformed = candidate(&ledger, &key);
    malformed.public_provenance = b"not valid turtle".to_vec();
    cases.push(malformed);

    let mut wrong_parent = candidate(&ledger, &key);
    wrong_parent.previous_envelope_hash = [1; 32];
    wrong_parent = wrong_parent.sign(&key).expect("resign wrong parent");
    cases.push(wrong_parent);

    let mut wrong_network = candidate(&ledger, &key);
    wrong_network.network_id = "other-network".to_string();
    wrong_network = wrong_network.sign(&key).expect("resign wrong network");
    cases.push(wrong_network);

    let mut wrong_profile = candidate(&ledger, &key);
    wrong_profile.profile_id = "other-profile".to_string();
    wrong_profile = wrong_profile.sign(&key).expect("resign wrong profile");
    cases.push(wrong_profile);

    let mut oversized = candidate(&ledger, &key);
    oversized.public_provenance = vec![b'x'; MAX_PAYLOAD_BYTES + 1];
    cases.push(oversized);

    for invalid in cases {
        assert!(matches!(
            submit_local(&mut ledger, invalid),
            Ok(AdmissionOutcome::Rejected { .. })
        ));
        assert_eq!(ledger.journal_frame_count().expect("read journal"), 0);
        assert!(ledger.committed_envelopes().is_empty());
    }
}

#[test]
fn noncanonical_and_tampered_envelopes_are_rejected() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");
    let request = candidate(&ledger, &key);
    let committed = match submit_local(&mut ledger, request).unwrap() {
        AdmissionOutcome::Committed { envelope } => *envelope,
        AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
    };

    let canonical = committed.canonical_bytes().expect("encode envelope");
    let mut trailing = canonical.clone();
    trailing.push(0);
    assert!(matches!(
        AdmittedBlockEnvelope::decode(&trailing),
        Err(LedgerError::NonCanonicalEnvelope)
    ));

    let mut tampered = canonical;
    let payload_marker = tampered
        .windows(3)
        .position(|window| window == b"B-1")
        .expect("payload marker")
        + 2;
    tampered[payload_marker] = b'2';
    assert!(matches!(
        AdmittedBlockEnvelope::decode(&tampered),
        Err(LedgerError::TamperedEnvelope(_))
    ));
    assert_eq!(ledger.journal_frame_count().unwrap(), 1);
}

#[test]
fn journal_corruption_is_detected_without_rebuilding_from_unverified_bytes() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");
    let request = candidate(&ledger, &key);
    submit_local(&mut ledger, request).expect("commit candidate");
    let journal_path = ledger.journal_path().to_path_buf();
    drop(ledger);

    let mut bytes = std::fs::read(&journal_path).expect("read journal");
    let last = bytes.last_mut().expect("journal frame bytes");
    *last ^= 1;
    std::fs::write(&journal_path, &bytes).expect("tamper journal");

    assert!(matches!(
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()),
        Err(LedgerError::CorruptJournal(_))
    ));
    assert_eq!(std::fs::read(&journal_path).unwrap(), bytes);
}

#[test]
fn torn_tail_requires_explicit_recovery_without_duplicate_commit() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");
    let first_candidate = candidate(&ledger, &key);
    submit_local(&mut ledger, first_candidate).expect("commit candidate");
    let journal_path = ledger.journal_path().to_path_buf();
    let verified_length = ledger.journal_bytes().expect("read journal").len();
    drop(ledger);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&journal_path)
        .expect("open journal for injected torn tail");
    std::io::Write::write_all(&mut file, &[0, 0, 0, 100, 1, 2, 3]).expect("append torn tail");
    drop(file);
    let torn_length = std::fs::metadata(&journal_path)
        .expect("stat torn journal")
        .len();

    let mut reopened = Ledger::open_in_dir(data_dir.path(), profile(), semantic_package())
        .expect("open torn journal");
    assert!(reopened.is_degraded());
    assert_eq!(reopened.journal_frame_count().unwrap(), 1);
    assert_eq!(
        reopened.journal_bytes().unwrap().len(),
        torn_length as usize
    );
    assert!(matches!(
        reopened.replay(),
        Err(LedgerError::CorruptJournal(_))
    ));

    reopened.repair_torn_tail().expect("repair torn tail");
    assert!(!reopened.is_degraded());
    assert_eq!(reopened.journal_bytes().unwrap().len(), verified_length);
    assert_eq!(reopened.committed_envelopes().len(), 1);

    let next_candidate = candidate(&reopened, &key);
    submit_local(&mut reopened, next_candidate).expect("continue after explicit recovery");
    assert_eq!(reopened.journal_frame_count().unwrap(), 2);
    assert_eq!(reopened.committed_envelopes().len(), 2);
}

#[cfg(unix)]
#[test]
fn journal_has_one_process_owner_at_a_time() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");

    assert!(matches!(
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()),
        Err(LedgerError::Io(_))
    ));

    drop(ledger);
    Ledger::open_in_dir(data_dir.path(), profile(), semantic_package())
        .expect("reopen after owner closes");
}

#[test]
fn post_state_commitment_is_for_the_full_public_dataset_not_turtle_order() {
    let first_data_dir = tempdir().expect("first temporary ledger directory");
    let second_data_dir = tempdir().expect("second temporary ledger directory");
    let key = signing_key();
    let first_payload = br#"@prefix ex: <http://example.org/> .
ex:batch ex:identifier "B-1" ; ex:source "sensor-a" ."#;
    let first_payload_reordered = br#"@prefix ex: <http://example.org/> .
ex:batch ex:source "sensor-a" ; ex:identifier "B-1" ."#;
    let second_payload = br#"@prefix ex: <http://example.org/> .
ex:batch ex:status "released" ; ex:identifier "B-1" ."#;
    let second_payload_reordered = br#"@prefix ex: <http://example.org/> .
ex:batch ex:identifier "B-1" ; ex:status "released" ."#;

    let mut first = Ledger::open_in_dir(first_data_dir.path(), profile(), semantic_package())
        .expect("open first");
    let first_candidate = first
        .create_ordinary_candidate(first_payload, 1_700_000_000_000, &key)
        .expect("create first candidate");
    submit_local(&mut first, first_candidate).expect("commit first candidate");
    let second_candidate = first
        .create_ordinary_candidate(second_payload, 1_700_000_000_001, &key)
        .expect("create second candidate");
    submit_local(&mut first, second_candidate).expect("commit second candidate");

    let mut second = Ledger::open_in_dir(second_data_dir.path(), profile(), semantic_package())
        .expect("open second");
    let reordered_first_candidate = second
        .create_ordinary_candidate(first_payload_reordered, 1_700_000_000_000, &key)
        .expect("create reordered first candidate");
    submit_local(&mut second, reordered_first_candidate).expect("commit reordered first candidate");
    let reordered_second_candidate = second
        .create_ordinary_candidate(second_payload_reordered, 1_700_000_000_001, &key)
        .expect("create reordered second candidate");
    submit_local(&mut second, reordered_second_candidate)
        .expect("commit reordered second candidate");

    assert_eq!(
        first.tip().state_commitment,
        second.tip().state_commitment,
        "RDF statement order must not change the public post-state commitment"
    );
}

#[test]
fn post_state_commitment_is_invariant_to_blank_node_labels() {
    let first_data_dir = tempdir().expect("first temporary ledger directory");
    let second_data_dir = tempdir().expect("second temporary ledger directory");
    let key = signing_key();
    let first_payload = br#"@prefix ex: <http://example.org/> .
_:first ex:part _:second .
_:second ex:value "one" ."#;
    let second_payload = br#"@prefix ex: <http://example.org/> .
_:alpha ex:part _:beta .
_:beta ex:value "one" ."#;

    let mut first = Ledger::open_in_dir(first_data_dir.path(), profile(), semantic_package())
        .expect("open first");
    let first_candidate = first
        .create_ordinary_candidate(first_payload, 1_700_000_000_000, &key)
        .expect("create first candidate");
    submit_local(&mut first, first_candidate).expect("commit first candidate");

    let mut second = Ledger::open_in_dir(second_data_dir.path(), profile(), semantic_package())
        .expect("open second");
    let second_candidate = second
        .create_ordinary_candidate(second_payload, 1_700_000_000_000, &key)
        .expect("create second candidate");
    submit_local(&mut second, second_candidate).expect("commit second candidate");

    assert_eq!(
        first.tip().state_commitment,
        second.tip().state_commitment,
        "blank-node labels are not part of the public state identity"
    );
}

#[test]
fn blank_nodes_are_scoped_per_payload_before_state_union() {
    let first_data_dir = tempdir().expect("first temporary ledger directory");
    let second_data_dir = tempdir().expect("second temporary ledger directory");
    let key = signing_key();
    let first_block = br#"@prefix ex: <http://example.org/> .
_:same ex:value "one" ."#;
    let first_block_relabelled = br#"@prefix ex: <http://example.org/> .
_:first ex:value "one" ."#;
    let second_block = br#"@prefix ex: <http://example.org/> .
_:same ex:value "two" ."#;
    let second_block_relabelled = br#"@prefix ex: <http://example.org/> .
_:second ex:value "two" ."#;

    let mut first = Ledger::open_in_dir(first_data_dir.path(), profile(), semantic_package())
        .expect("open first");
    let first_candidate = first
        .create_ordinary_candidate(first_block, 1_700_000_000_000, &key)
        .expect("create first candidate");
    submit_local(&mut first, first_candidate).expect("commit first candidate");
    let second_candidate = first
        .create_ordinary_candidate(second_block, 1_700_000_000_001, &key)
        .expect("create second candidate");
    submit_local(&mut first, second_candidate).expect("commit second candidate");

    let mut second = Ledger::open_in_dir(second_data_dir.path(), profile(), semantic_package())
        .expect("open second");
    let relabelled_first_candidate = second
        .create_ordinary_candidate(first_block_relabelled, 1_700_000_000_000, &key)
        .expect("create relabelled first candidate");
    submit_local(&mut second, relabelled_first_candidate)
        .expect("commit relabelled first candidate");
    let relabelled_second_candidate = second
        .create_ordinary_candidate(second_block_relabelled, 1_700_000_000_001, &key)
        .expect("create relabelled second candidate");
    submit_local(&mut second, relabelled_second_candidate)
        .expect("commit relabelled second candidate");

    assert_eq!(
        first.tip().state_commitment,
        second.tip().state_commitment,
        "same source label in separate payloads must not alias across blocks"
    );
}
