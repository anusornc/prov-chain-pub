//! Acceptance tests for universal ordinary-write Final Admission (Issue #3).

use ed25519_dalek::SigningKey;
use provchain_org::ledger::{
    AdmissionCandidate, AdmissionOutcome, AdmittedBlockEnvelope, Ledger, LedgerHash, LedgerProfile,
    MAX_PAYLOAD_BYTES,
};
use std::collections::BTreeMap;
use tempfile::tempdir;

mod support;
use support::semantic::{bind_profile, semantic_package};

fn profile() -> LedgerProfile {
    bind_profile(LedgerProfile::new("provchain.issue3", "issue3.reference"))
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[11u8; 32])
}

fn payload(label: &str) -> Vec<u8> {
    format!("@prefix ex: <http://example.org/> .\nex:batch ex:identifier \"{label}\" .")
        .into_bytes()
}

fn candidate(ledger: &Ledger, key: &SigningKey, label: &str, timestamp: u64) -> AdmissionCandidate {
    ledger
        .create_ordinary_candidate(payload(label), timestamp, key)
        .expect("create ordinary candidate")
}

fn committed_envelope(outcome: AdmissionOutcome) -> AdmittedBlockEnvelope {
    match outcome {
        AdmissionOutcome::Committed { envelope } => *envelope,
        AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LedgerSnapshot {
    journal_bytes: Vec<u8>,
    committed_prefix: Vec<AdmittedBlockEnvelope>,
    projected_rdf_len: usize,
    envelope_index: BTreeMap<LedgerHash, u64>,
}

fn snapshot(ledger: &Ledger) -> LedgerSnapshot {
    LedgerSnapshot {
        journal_bytes: ledger.journal_bytes().expect("read journal bytes"),
        committed_prefix: ledger.committed_envelopes().to_vec(),
        projected_rdf_len: ledger.rdf_store().len().expect("count projected RDF"),
        envelope_index: ledger.envelope_index().clone(),
    }
}

#[derive(Clone, Copy, Debug)]
enum IngressSurface {
    Local,
    Api,
    Batch,
    Synchronization,
    Replication,
    Proposal,
}

impl IngressSurface {
    const ALL: [Self; 6] = [
        Self::Local,
        Self::Api,
        Self::Batch,
        Self::Synchronization,
        Self::Replication,
        Self::Proposal,
    ];

    fn submit(
        self,
        ledger: &mut Ledger,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, provchain_org::ledger::LedgerError> {
        match self {
            Self::Local => ledger.local_adapter().submit(candidate),
            Self::Api => ledger.api_adapter().submit(candidate),
            Self::Batch => ledger.batch_adapter().submit(candidate),
            Self::Synchronization => ledger.synchronization_adapter().submit(candidate),
            Self::Replication => ledger.replication_adapter().submit(candidate),
            Self::Proposal => ledger.proposal_adapter().submit(candidate),
        }
    }
}

#[test]
fn every_supported_adapter_commits_the_same_candidate_as_identical_envelope_bytes() {
    let key = signing_key();
    let mut canonical_results = Vec::new();

    for adapter in IngressSurface::ALL {
        let data_dir = tempdir().expect("temporary ledger directory");
        let mut ledger = Ledger::open_in_dir(data_dir.path(), profile(), semantic_package())
            .expect("open ledger");
        let request = candidate(&ledger, &key, "B-3", 1_700_000_000_003);

        let envelope = committed_envelope(
            adapter
                .submit(&mut ledger, request)
                .expect("adapter submission should commit"),
        );

        canonical_results.push(
            envelope
                .canonical_bytes()
                .expect("encode committed envelope"),
        );
        assert_eq!(ledger.journal_frame_count().expect("count frames"), 1);
        assert_eq!(ledger.committed_envelopes(), &[envelope]);
        assert_eq!(ledger.rdf_store().len().expect("count RDF"), 1);
        assert_eq!(ledger.envelope_index().len(), 1);
    }

    assert!(canonical_results.windows(2).all(|pair| pair[0] == pair[1]));
}

#[derive(Clone, Copy, Debug)]
enum InvalidCase {
    MalformedOrdinaryPayload,
    OversizedPayload,
    TamperedDigest,
    WrongNetwork,
    WrongProfile,
    WrongParent,
    WrongIndex,
    MissingSignerContext,
    WrongStagedState,
    UnsupportedEncryptedPayload,
}

impl InvalidCase {
    const ALL: [Self; 10] = [
        Self::MalformedOrdinaryPayload,
        Self::OversizedPayload,
        Self::TamperedDigest,
        Self::WrongNetwork,
        Self::WrongProfile,
        Self::WrongParent,
        Self::WrongIndex,
        Self::MissingSignerContext,
        Self::WrongStagedState,
        Self::UnsupportedEncryptedPayload,
    ];
}

fn invalidate(
    mut candidate: AdmissionCandidate,
    case: InvalidCase,
    key: &SigningKey,
) -> AdmissionCandidate {
    match case {
        InvalidCase::MalformedOrdinaryPayload => {
            candidate.public_provenance = b"not valid Turtle".to_vec();
        }
        InvalidCase::OversizedPayload => {
            candidate.public_provenance = vec![b'x'; MAX_PAYLOAD_BYTES + 1];
        }
        InvalidCase::TamperedDigest => {
            candidate.proposal_digest[0] ^= 1;
        }
        InvalidCase::WrongNetwork => {
            candidate.network_id = "other-network".to_string();
            candidate = candidate.sign(key).expect("sign wrong-network candidate");
        }
        InvalidCase::WrongProfile => {
            candidate.profile_id = "other-profile".to_string();
            candidate = candidate.sign(key).expect("sign wrong-profile candidate");
        }
        InvalidCase::WrongParent => {
            candidate.previous_envelope_hash = [3; 32];
            candidate = candidate.sign(key).expect("sign wrong-parent candidate");
        }
        InvalidCase::WrongIndex => {
            candidate.index += 1;
            candidate = candidate.sign(key).expect("sign wrong-index candidate");
        }
        InvalidCase::MissingSignerContext => {
            candidate.proposer_public_key = [0; 32];
        }
        InvalidCase::WrongStagedState => {
            candidate.post_state_commitment = [9; 32];
            candidate = candidate.sign(key).expect("sign wrong-state candidate");
        }
        InvalidCase::UnsupportedEncryptedPayload => {
            candidate.encrypted_payload = Some(vec![7; 32]);
        }
    }
    candidate
}

#[test]
fn every_precommit_rejection_is_identical_and_preserves_all_authoritative_and_projected_state() {
    let key = signing_key();

    for case in InvalidCase::ALL {
        let mut expected_rejection = None;

        for adapter in IngressSurface::ALL {
            let data_dir = tempdir().expect("temporary ledger directory");
            let mut ledger = Ledger::open_in_dir(data_dir.path(), profile(), semantic_package())
                .expect("open ledger");
            let seed = candidate(&ledger, &key, "seed", 1_700_000_000_001);
            committed_envelope(ledger.local_adapter().submit(seed).expect("seed commit"));

            let invalid = invalidate(
                candidate(&ledger, &key, "invalid", 1_700_000_000_002),
                case,
                &key,
            );
            let before = snapshot(&ledger);
            let outcome = adapter
                .submit(&mut ledger, invalid)
                .expect("deterministic invalidity must produce a rejection outcome");

            assert!(matches!(outcome, AdmissionOutcome::Rejected { .. }));
            assert_eq!(snapshot(&ledger), before, "state changed for {case:?}");
            if let Some(expected) = &expected_rejection {
                assert_eq!(&outcome, expected, "adapter diverged for {case:?}");
            } else {
                expected_rejection = Some(outcome);
            }
        }
    }
}

#[test]
fn exact_candidate_retry_across_all_adapters_has_one_commit_and_one_response_identity() {
    let data_dir = tempdir().expect("temporary ledger directory");
    let key = signing_key();
    let mut ledger =
        Ledger::open_in_dir(data_dir.path(), profile(), semantic_package()).expect("open ledger");
    let request = candidate(&ledger, &key, "retry", 1_700_000_000_004);
    let mut committed_bytes = Vec::new();

    for adapter in IngressSurface::ALL {
        let envelope = committed_envelope(
            adapter
                .submit(&mut ledger, request.clone())
                .expect("exact retry should resolve as committed"),
        );
        committed_bytes.push(envelope.canonical_bytes().expect("encode envelope"));
    }

    assert!(committed_bytes.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(ledger.journal_frame_count().expect("count frames"), 1);
    assert_eq!(ledger.committed_envelopes().len(), 1);
    assert_eq!(ledger.envelope_index().len(), 1);
    assert_eq!(ledger.rdf_store().len().expect("count RDF"), 1);
}
