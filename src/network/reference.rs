//! Manifest-gated reference node attached to the universal durable ledger.
//!
//! Activation verifies membership before opening the ledger, so callers never
//! receive networking or admission capabilities from an invalid startup contract.

use std::path::Path;
use std::sync::Mutex;

use ed25519_dalek::{SigningKey, VerifyingKey};
use thiserror::Error;
use uuid::Uuid;

use crate::bridge::{
    BridgeCommitReceiptV1, BridgeError, BridgeExportArtifactV1, BridgeImportOutcomeV1,
    BridgeImportPreflightV1, BridgeProofBundleV1, BridgeSourceProfileV1, EffectiveBridgeStateV1,
};
use crate::ledger::{AdmissionCandidate, Ledger, LedgerError, LedgerProfile};
use crate::network::convergence::{
    committed_record_at, committed_records_at, evaluate_commit_status,
    evaluate_privacy_commit_status, export_committed_range, issue_local_receipt,
    ledger_prefix_checkpoint, validate_range_for_admission, validate_record_for_admission,
    CommitReceipt, CommitStatus, CommittedEnvelopeRange, CommittedEnvelopeRecord,
    CommittedRangeRequest, ConvergenceError, LedgerPrefixCheckpoint, PrivacyStateReceipt,
};
use crate::network::membership::{
    ActiveMembership, MemberRole, MembershipError, NodeIdentity, SignedMembershipManifest,
    SignerAuthorization, SignerAuthorizationError,
};
use crate::network::peer_session::{
    AuthenticatedPeerSession, PeerHandshakeChallenge, PeerHandshakeHello, PeerHandshakeResponse,
    PeerMessageClass, PeerSessionAuthenticator, PeerSessionError, PeerTransport,
    PeerTransportRegistry,
};
use crate::network::poa::{
    signing_fence_ledger_binding, AuthoritySchedule, ExactEnvelopeIngress, PoAError,
    PoAProposalOutcome, PoAProposalRequest, PoATurn, ProposalCoordinatorState, ScheduledAuthority,
    SIGNING_FENCE_FILENAME,
};
use crate::network::profile::NetworkProfile;
use crate::ontology::ActivatedSemanticPackage;
use crate::privacy::{
    EffectivePrivacyState, LivePrivacyReleaseRequest, LivePrivacyReleaseResponse,
    PrivacyAdmissionAnchor,
};

/// Exact ledger profile plus its already activated semantic runtime.
#[derive(Debug, Clone)]
pub struct ReferenceLedgerActivation {
    ledger_profile: LedgerProfile,
    semantic_package: ActivatedSemanticPackage,
}

impl ReferenceLedgerActivation {
    /// Bind the durable ledger configuration to its executable package.
    pub fn new(ledger_profile: LedgerProfile, semantic_package: ActivatedSemanticPackage) -> Self {
        Self {
            ledger_profile,
            semantic_package,
        }
    }
}

/// A bounded reference node whose admission authority remains [`Ledger`].
pub struct ReferenceNode {
    network_profile: NetworkProfile,
    membership: ActiveMembership,
    peer_authenticator: PeerSessionAuthenticator,
    peer_transport_registry: PeerTransportRegistry,
    authority_schedule: AuthoritySchedule,
    validator_signing_key: Option<SigningKey>,
    proposal_coordinator: Mutex<ProposalCoordinatorState>,
    ledger: Mutex<Ledger>,
}

impl ReferenceNode {
    /// Validate membership first, then open the sole durable ledger authority.
    pub fn start_in_dir<P: AsRef<Path>>(
        data_dir: P,
        ledger_activation: ReferenceLedgerActivation,
        network_profile: NetworkProfile,
        signed_manifest: SignedMembershipManifest,
        governance_root: VerifyingKey,
        identity: NodeIdentity,
    ) -> Result<Self, ReferenceNodeError> {
        Self::start_with_validator_in_dir(
            data_dir,
            ledger_activation,
            network_profile,
            signed_manifest,
            governance_root,
            identity,
            None,
        )
    }

    /// Activate a reference validator with its separate PoA proposal key.
    pub fn start_validator_in_dir<P: AsRef<Path>>(
        data_dir: P,
        ledger_activation: ReferenceLedgerActivation,
        network_profile: NetworkProfile,
        signed_manifest: SignedMembershipManifest,
        governance_root: VerifyingKey,
        identity: NodeIdentity,
        validator_signing_key: SigningKey,
    ) -> Result<Self, ReferenceNodeError> {
        Self::start_with_validator_in_dir(
            data_dir,
            ledger_activation,
            network_profile,
            signed_manifest,
            governance_root,
            identity,
            Some(validator_signing_key),
        )
    }

    fn start_with_validator_in_dir<P: AsRef<Path>>(
        data_dir: P,
        ledger_activation: ReferenceLedgerActivation,
        network_profile: NetworkProfile,
        signed_manifest: SignedMembershipManifest,
        governance_root: VerifyingKey,
        identity: NodeIdentity,
        validator_signing_key: Option<SigningKey>,
    ) -> Result<Self, ReferenceNodeError> {
        let data_dir = data_dir.as_ref();
        let ReferenceLedgerActivation {
            mut ledger_profile,
            semantic_package,
        } = ledger_activation;
        let activated_privacy = network_profile
            .activate_privacy_lifecycle(&signed_manifest, &governance_root)
            .map_err(|error| ReferenceNodeError::PrivacyProfileActivation(error.to_string()))?;
        #[cfg(feature = "privacy-conformance")]
        if ledger_profile.privacy_conformance_slice.is_some() {
            return Err(ReferenceNodeError::LedgerProfileMismatch(
                "privacy_conformance_slice",
            ));
        }
        if ledger_profile.privacy_lifecycle.is_some()
            && ledger_profile.privacy_lifecycle != activated_privacy
        {
            return Err(ReferenceNodeError::LedgerProfileMismatch(
                "privacy_lifecycle",
            ));
        }
        if let Some(source) = &ledger_profile.bridge_source_profile {
            source
                .matches_active_context(&network_profile, &signed_manifest)
                .map_err(|error| ReferenceNodeError::BridgeProfileActivation(error.to_string()))?;
        }
        ledger_profile.privacy_lifecycle = activated_privacy;
        let membership = ActiveMembership::activate(
            &network_profile,
            signed_manifest,
            governance_root,
            identity,
        )?;
        if let Some(signing_key) = &validator_signing_key {
            membership.authorize_validator_signer(
                membership.local_node_id(),
                signing_key.verifying_key().to_bytes(),
            )?;
        }
        validate_ledger_profile(&network_profile, &ledger_profile)?;
        let authority_schedule = AuthoritySchedule::from_contract(&network_profile, &membership)?;
        let peer_authenticator = PeerSessionAuthenticator::new(&network_profile, &membership);
        let peer_transport_registry = PeerTransportRegistry::default();
        let mut ledger = Ledger::open_in_dir(data_dir, ledger_profile, semantic_package)?;
        let ledger_binding = signing_fence_ledger_binding(
            &network_profile,
            membership.manifest_digest(),
            membership.local_node_id(),
            ledger.journal_path(),
        );
        let mut proposal_coordinator =
            ProposalCoordinatorState::open(data_dir.join(SIGNING_FENCE_FILENAME), ledger_binding)?;
        proposal_coordinator.validate_committed_locks(&ledger)?;
        if let Some(signing_key) = &validator_signing_key {
            proposal_coordinator.recover(
                &mut ledger,
                &authority_schedule,
                membership.local_node_id(),
                signing_key,
            )?;
        }
        // Recovery is complete before the node exposes networking or admission methods.
        Ok(Self {
            network_profile,
            membership,
            peer_authenticator,
            peer_transport_registry,
            authority_schedule,
            validator_signing_key,
            proposal_coordinator: Mutex::new(proposal_coordinator),
            ledger: Mutex::new(ledger),
        })
    }

    /// Verified local logical node identity.
    pub fn local_node_id(&self) -> Uuid {
        self.membership.local_node_id()
    }

    /// Verified local role selected at startup.
    pub fn local_role(&self) -> MemberRole {
        self.membership.local_role()
    }

    /// Canonical digest of the active profile-bound manifest.
    pub fn membership_manifest_digest(&self) -> [u8; 32] {
        self.membership.manifest_digest()
    }

    /// Count durable journal frames without exposing the mutable RDF store.
    pub fn journal_frame_count(&self) -> Result<usize, crate::ledger::LedgerError> {
        self.ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?
            .journal_frame_count()
    }

    /// Inject one target journal-boundary failure for bridge conformance tests.
    #[cfg(feature = "bridge-conformance")]
    #[doc(hidden)]
    pub fn inject_next_bridge_commit_failpoint(
        &self,
        failpoint: crate::ledger::LedgerCommitFailpoint,
    ) -> Result<(), LedgerError> {
        self.ledger
            .lock()
            .map_err(|_| LedgerError::Degraded)?
            .inject_next_commit_failpoint(failpoint);
        Ok(())
    }

    /// Snapshot the journal-derived participant/key lifecycle projection.
    pub fn effective_privacy_state(
        &self,
    ) -> Result<EffectivePrivacyState, crate::ledger::LedgerError> {
        self.ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?
            .effective_privacy_state()
            .cloned()
    }

    /// Snapshot Effective Bridge State derived only from verified target journal replay.
    pub fn effective_bridge_state(&self) -> Result<EffectiveBridgeStateV1, LedgerError> {
        self.ledger
            .lock()
            .map_err(|_| LedgerError::Degraded)?
            .effective_bridge_state()
            .cloned()
    }

    /// Derive the next privacy transition anchor from verified local journal history.
    pub fn next_privacy_admission_anchor(
        &self,
    ) -> Result<PrivacyAdmissionAnchor, crate::ledger::LedgerError> {
        self.ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?
            .next_privacy_admission_anchor()
    }

    /// Exact active network contract.
    pub fn network_profile(&self) -> &NetworkProfile {
        &self.network_profile
    }

    /// Serve one opaque protected-object release after the exact current
    /// Network-Converged barrier has been validated by the durable ledger.
    pub fn live_privacy_release(
        &self,
        evidence: &crate::network::convergence::NetworkConvergenceEvidence,
        request: &LivePrivacyReleaseRequest,
    ) -> Result<LivePrivacyReleaseResponse, LedgerError> {
        let ledger = self.ledger.lock().map_err(|_| LedgerError::Degraded)?;
        ledger.live_privacy_release(evidence, request)
    }

    /// Emit this node's signed privacy-state receipt for one current durable
    /// envelope. The state digest is derived from the journal projection, never
    /// supplied by the caller.
    pub fn local_privacy_state_receipt(
        &self,
        index: u64,
    ) -> Result<PrivacyStateReceipt, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        if ledger.tip().index != Some(index) {
            return Err(ConvergenceError::MissingCommittedEnvelope(index));
        }
        ledger
            .privacy_state_receipt(
                self.local_node_id(),
                self.membership.signed_manifest(),
                self.membership.identity_key(),
            )
            .map_err(ConvergenceError::from)
    }

    /// Derive the unique manifest-authorized validator for a ledger height.
    pub fn scheduled_authority(&self, height: u64) -> Result<ScheduledAuthority, PoAError> {
        Ok(self.authority_schedule.authority_at(height))
    }

    /// Return the pending turn derived only from verified local ledger history.
    pub fn pending_poa_turn(&self) -> Result<PoATurn, PoAError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        Ok(PoATurn::from_tip(ledger.tip()))
    }

    /// Process one unsigned ordinary request when this node is scheduled.
    pub fn submit_poa_request(
        &self,
        request: PoAProposalRequest,
    ) -> Result<PoAProposalOutcome, PoAError> {
        let signing_key = self
            .validator_signing_key
            .as_ref()
            .ok_or(PoAError::MissingLocalSigningKey)?;
        let mut coordinator = self
            .proposal_coordinator
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let mut ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        coordinator.submit(
            request,
            &mut ledger,
            &self.authority_schedule,
            self.local_node_id(),
            signing_key,
            self.network_profile.consensus.max_block_size,
        )
    }

    /// Verify, classify, and route one exact source proof through target PoA and Final Admission.
    ///
    /// # Errors
    ///
    /// Returns `BridgeImportError` when the proof fails closed verification,
    /// the local node holds no validator signing key, the PoA coordinator
    /// rejects the proposal, or Final Admission fails.
    pub fn import_bridge(
        &self,
        proof_bytes: &[u8],
        public_provenance: &[u8],
        timestamp_millis: u64,
    ) -> Result<BridgeImportOutcomeV1, BridgeImportError> {
        let preflight = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?
            .preflight_bridge_import(proof_bytes, public_provenance)?;
        match preflight {
            BridgeImportPreflightV1::AlreadyImported(reference) => {
                return Ok(BridgeImportOutcomeV1::AlreadyImported { reference });
            }
            BridgeImportPreflightV1::ReplayConflict => {
                return Ok(BridgeImportOutcomeV1::ReplayConflict);
            }
            BridgeImportPreflightV1::Ready(_) => {}
        }
        let signing_key = self
            .validator_signing_key
            .as_ref()
            .ok_or(PoAError::MissingLocalSigningKey)?;
        let mut coordinator = self
            .proposal_coordinator
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let mut ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let ready = match ledger.preflight_bridge_import(proof_bytes, public_provenance)? {
            BridgeImportPreflightV1::AlreadyImported(reference) => {
                return Ok(BridgeImportOutcomeV1::AlreadyImported { reference });
            }
            BridgeImportPreflightV1::ReplayConflict => {
                return Ok(BridgeImportOutcomeV1::ReplayConflict);
            }
            BridgeImportPreflightV1::Ready(verified) => verified,
        };
        let request = PoAProposalRequest::bridge_import(
            PoATurn::from_tip(ledger.tip()),
            timestamp_millis,
            public_provenance,
            &ready.origin_bytes,
        );
        match coordinator.submit(
            request,
            &mut ledger,
            &self.authority_schedule,
            self.local_node_id(),
            signing_key,
            self.network_profile.consensus.max_block_size,
        )? {
            crate::ledger::AdmissionOutcome::Committed { .. } => {
                let reference = ledger
                    .effective_bridge_state()?
                    .get(&ready.transfer_id)
                    .cloned()
                    .ok_or_else(|| {
                        BridgeImportError::Rejected(
                            "committed import is absent from Effective Bridge State".to_string(),
                        )
                    })?;
                Ok(BridgeImportOutcomeV1::Imported { reference })
            }
            crate::ledger::AdmissionOutcome::Rejected { reason } => {
                Err(BridgeImportError::Rejected(reason))
            }
        }
    }

    /// Apply PoA Consensus Acceptance before submitting a signed proposal to Final Admission.
    pub fn accept_signed_poa_proposal(
        &self,
        candidate: AdmissionCandidate,
    ) -> Result<PoAProposalOutcome, PoAError> {
        let mut coordinator = self
            .proposal_coordinator
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let mut ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        coordinator.accept_signed(
            candidate,
            &mut ledger,
            &self.authority_schedule,
            self.network_profile.consensus.max_block_size,
        )
    }

    /// Read one exact envelope and prefix commitment only after local durability.
    pub fn committed_envelope_record(
        &self,
        index: u64,
    ) -> Result<CommittedEnvelopeRecord, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        committed_record_at(&ledger, index)
    }

    /// Return the journal-derived anchor for the next bounded catch-up request.
    pub fn ledger_prefix_checkpoint(&self) -> Result<LedgerPrefixCheckpoint, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        ledger_prefix_checkpoint(&ledger)
    }

    /// Serve a bounded contiguous exact-envelope range to an authenticated peer.
    pub fn export_committed_range(
        &self,
        transport: &mut PeerTransport,
        request: CommittedRangeRequest,
    ) -> Result<CommittedEnvelopeRange, ConvergenceError> {
        self.authorize_peer_transport(transport, PeerMessageClass::Synchronization)?;
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        export_committed_range(&ledger, request)
    }

    /// Verify and Final-Admit a bounded exact range, emitting one local receipt
    /// per durable frame. Exact replay is idempotent for lost responses.
    pub fn import_committed_range(
        &self,
        transport: &mut PeerTransport,
        request: CommittedRangeRequest,
        range: CommittedEnvelopeRange,
    ) -> Result<Vec<CommitReceipt>, ConvergenceError> {
        self.authorize_peer_transport(transport, PeerMessageClass::Synchronization)?;
        let mut coordinator = self
            .proposal_coordinator
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let mut ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        validate_range_for_admission(&ledger, request, &range)?;
        for record in range.records() {
            coordinator.accept_exact_envelope(
                record.envelope_bytes(),
                ExactEnvelopeIngress::Synchronization,
                &mut ledger,
                &self.authority_schedule,
                self.network_profile.consensus.max_block_size,
            )?;
        }
        let durable_records =
            committed_records_at(&ledger, range.start_index(), range.records().len())?;
        if durable_records.as_slice() != range.records() {
            let mismatch_index = durable_records
                .iter()
                .zip(range.records())
                .find_map(|(durable, received)| (durable != received).then_some(received.index()))
                .unwrap_or(range.start_index());
            return Err(ConvergenceError::DivergentPrefix {
                index: mismatch_index,
                reason: "Final Admission did not retain the exact synchronized journal frames and prefixes"
                    .to_string(),
            });
        }
        durable_records
            .iter()
            .map(|durable_record| {
                issue_local_receipt(
                    self.local_node_id(),
                    &self.network_profile.network_id,
                    &self.network_profile.profile_id,
                    self.membership.signed_manifest(),
                    self.membership.identity_key(),
                    durable_record,
                )
            })
            .collect()
    }

    /// Reproducibly emit this node's identity-signed receipt for a durable frame.
    pub fn local_commit_receipt(&self, index: u64) -> Result<CommitReceipt, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        let record = committed_record_at(&ledger, index)?;
        issue_local_receipt(
            self.local_node_id(),
            &self.network_profile.network_id,
            &self.network_profile.profile_id,
            self.membership.signed_manifest(),
            self.membership.identity_key(),
            &record,
        )
    }

    /// Reproducibly sign the bridge-specific facts for one local durable export envelope.
    pub fn local_bridge_commit_receipt(
        &self,
        index: u64,
    ) -> Result<BridgeCommitReceiptV1, BridgeError> {
        let (source, record) = self.bridge_source_record(index)?;
        let envelope = crate::ledger::AdmittedBlockEnvelope::decode(record.envelope_bytes())
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        if envelope.bridge_export_declaration.is_none() {
            return Err(BridgeError::Rejected(
                "durable source envelope has no bridge export declaration".to_string(),
            ));
        }
        BridgeCommitReceiptV1::sign(
            &source,
            self.membership.signed_manifest(),
            self.local_node_id(),
            index,
            record.envelope_hash(),
            record.ledger_prefix_hash(),
            self.membership.identity_key(),
        )
    }

    /// Export exact committed source bytes only after all three pinned receipts verify.
    pub fn export_bridge(
        &self,
        index: u64,
        receipts: &[BridgeCommitReceiptV1],
    ) -> Result<BridgeExportArtifactV1, BridgeError> {
        let (source, record) = self.bridge_source_record(index)?;
        let mut receipts = receipts.to_vec();
        receipts.sort_by(|left, right| left.core().node_id().cmp(right.core().node_id()));
        let proof = BridgeProofBundleV1::new(
            &source,
            self.membership.signed_manifest(),
            record.envelope_bytes().to_vec(),
            record.ledger_prefix_hash(),
            receipts,
        )?;
        BridgeExportArtifactV1::new(proof)
    }

    fn bridge_source_record(
        &self,
        index: u64,
    ) -> Result<(BridgeSourceProfileV1, CommittedEnvelopeRecord), BridgeError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| BridgeError::Rejected("source ledger is degraded".to_string()))?;
        let source = ledger
            .profile()
            .bridge_source_profile
            .clone()
            .ok_or_else(|| {
                BridgeError::InvalidProfile(
                    "source bridge profile is not active on this ledger".to_string(),
                )
            })?;
        let record = committed_record_at(&ledger, index)
            .map_err(|error| BridgeError::Rejected(error.to_string()))?;
        Ok((source, record))
    }

    /// Verify and commit exact producer bytes received on an authenticated peer
    /// session, then emit a receipt for this node's own durable journal frame.
    pub fn receive_committed_envelope(
        &self,
        transport: &mut PeerTransport,
        record: CommittedEnvelopeRecord,
    ) -> Result<CommitReceipt, ConvergenceError> {
        self.authorize_peer_transport(transport, PeerMessageClass::Admission)?;
        let mut coordinator = self
            .proposal_coordinator
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        let mut ledger = self
            .ledger
            .lock()
            .map_err(|_| PoAError::CoordinatorUnavailable)?;
        validate_record_for_admission(&ledger, &record)?;
        coordinator.accept_exact_envelope(
            record.envelope_bytes(),
            ExactEnvelopeIngress::Replication,
            &mut ledger,
            &self.authority_schedule,
            self.network_profile.consensus.max_block_size,
        )?;
        let durable_record = committed_record_at(&ledger, record.index())?;
        if durable_record != record {
            return Err(ConvergenceError::DivergentPrefix {
                index: record.index(),
                reason: "Final Admission did not retain the exact received envelope and prefix"
                    .to_string(),
            });
        }
        issue_local_receipt(
            self.local_node_id(),
            &self.network_profile.network_id,
            &self.network_profile.profile_id,
            self.membership.signed_manifest(),
            self.membership.identity_key(),
            &durable_record,
        )
    }

    /// Evaluate local commit versus three-node convergence without treating
    /// receipts as votes or as a prerequisite for local commit.
    pub fn commit_status(
        &self,
        index: u64,
        receipts: &[CommitReceipt],
    ) -> Result<CommitStatus, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        let record = committed_record_at(&ledger, index)?;
        let expected_node_ids: Vec<_> = self
            .authority_schedule
            .ordered_authorities()
            .iter()
            .map(ScheduledAuthority::node_id)
            .collect();
        evaluate_commit_status(
            self.local_node_id(),
            &self.network_profile.network_id,
            &self.network_profile.profile_id,
            self.membership.signed_manifest(),
            &expected_node_ids,
            &record,
            receipts,
        )
    }

    /// Evaluate receipt-backed convergence including the canonical Effective
    /// Privacy State digest required by the Issue #10 release barrier.
    pub fn privacy_commit_status(
        &self,
        index: u64,
        receipts: &[CommitReceipt],
        privacy_receipts: &[PrivacyStateReceipt],
    ) -> Result<CommitStatus, ConvergenceError> {
        let ledger = self
            .ledger
            .lock()
            .map_err(|_| crate::ledger::LedgerError::Degraded)?;
        let record = committed_record_at(&ledger, index)?;
        let expected_node_ids: Vec<_> = self
            .authority_schedule
            .ordered_authorities()
            .iter()
            .map(ScheduledAuthority::node_id)
            .collect();
        let status = evaluate_privacy_commit_status(
            self.local_node_id(),
            &self.network_profile.network_id,
            &self.network_profile.profile_id,
            self.membership.signed_manifest(),
            &expected_node_ids,
            &record,
            receipts,
            privacy_receipts,
        )?;
        if let CommitStatus::NetworkConverged { ref evidence } = status {
            ledger.network_converged_privacy_prefix(evidence)?;
        }
        Ok(status)
    }

    /// Start a fresh outbound handshake at a trusted Unix-millisecond time.
    pub fn begin_peer_handshake_at(
        &mut self,
        transport: &PeerTransport,
        responder_node_id: Uuid,
        now_millis: u64,
    ) -> Result<PeerHandshakeHello, PeerSessionError> {
        let transport_id =
            transport.handshake_binding(self.local_node_id(), self.membership_manifest_digest())?;
        self.peer_authenticator
            .begin_at(transport_id, responder_node_id, now_millis)
    }

    /// Validate an inbound hello and return the responder's fresh proof.
    pub fn accept_peer_handshake_at(
        &mut self,
        transport: &PeerTransport,
        hello: PeerHandshakeHello,
        now_millis: u64,
    ) -> Result<PeerHandshakeChallenge, PeerSessionError> {
        let transport_id =
            transport.handshake_binding(self.local_node_id(), self.membership_manifest_digest())?;
        self.peer_authenticator
            .accept_at(transport_id, hello, now_millis)
    }

    /// Authenticate the responder and return the initiator proof and local session.
    pub fn answer_peer_challenge_at(
        &mut self,
        transport: &PeerTransport,
        challenge: PeerHandshakeChallenge,
        now_millis: u64,
    ) -> Result<(PeerHandshakeResponse, AuthenticatedPeerSession), PeerSessionError> {
        let transport_id =
            transport.handshake_binding(self.local_node_id(), self.membership_manifest_digest())?;
        self.peer_authenticator
            .answer_at(transport_id, challenge, now_millis)
    }

    /// Authenticate the initiator and finish the responder's local session.
    pub fn finish_peer_handshake_at(
        &mut self,
        transport: &PeerTransport,
        response: PeerHandshakeResponse,
        now_millis: u64,
    ) -> Result<AuthenticatedPeerSession, PeerSessionError> {
        let transport_id =
            transport.handshake_binding(self.local_node_id(), self.membership_manifest_digest())?;
        self.peer_authenticator
            .finish_at(transport_id, response, now_millis)
    }

    /// Create a new transport quarantined from all ordinary peer behavior.
    pub fn quarantine_peer_transport(&self) -> PeerTransport {
        PeerTransport::quarantined(
            self.local_node_id(),
            self.membership_manifest_digest(),
            self.peer_transport_registry.clone(),
        )
    }

    /// Check a peer's separate PoA key and validator role after peer authentication.
    pub fn authorize_peer_signer(
        &self,
        transport: &PeerTransport,
        presented_validator_key: [u8; 32],
    ) -> Result<SignerAuthorization, SignerAuthorizationError> {
        let Some((local_node_id, peer_id, manifest_digest)) = transport.authenticated_binding()
        else {
            return Err(SignerAuthorizationError::UnauthenticatedTransport);
        };
        if local_node_id != self.local_node_id() {
            return Err(SignerAuthorizationError::TransportLocalIdentityMismatch);
        }
        if manifest_digest != self.membership_manifest_digest() {
            return Err(SignerAuthorizationError::TransportManifestMismatch);
        }
        self.membership
            .authorize_validator_signer(peer_id, presented_validator_key)
    }

    fn authorize_peer_transport(
        &self,
        transport: &mut PeerTransport,
        message_class: PeerMessageClass,
    ) -> Result<Uuid, ConvergenceError> {
        transport.authorize_message(message_class)?;
        let Some((local_node_id, peer_id, manifest_digest)) = transport.authenticated_binding()
        else {
            return Err(ConvergenceError::TransportBindingMismatch);
        };
        if local_node_id != self.local_node_id()
            || manifest_digest != self.membership_manifest_digest()
        {
            return Err(ConvergenceError::TransportBindingMismatch);
        }
        Ok(peer_id)
    }
}

fn validate_ledger_profile(
    network: &NetworkProfile,
    ledger: &LedgerProfile,
) -> Result<(), ReferenceNodeError> {
    if ledger.network_id != network.network_id {
        return Err(ReferenceNodeError::LedgerProfileMismatch("network_id"));
    }
    if ledger.profile_id != network.profile_id {
        return Err(ReferenceNodeError::LedgerProfileMismatch("profile_id"));
    }
    if ledger.ontology_package_id != network.semantic.ontology_package_id {
        return Err(ReferenceNodeError::LedgerProfileMismatch(
            "ontology_package_id",
        ));
    }
    if ledger.ontology_package_version != network.semantic.ontology_package_version {
        return Err(ReferenceNodeError::LedgerProfileMismatch(
            "ontology_package_version",
        ));
    }
    if ledger.ontology_package_hash != network.semantic.ontology_package_hash {
        return Err(ReferenceNodeError::LedgerProfileMismatch(
            "ontology_package_hash",
        ));
    }
    if ledger.semantic_execution_profile_id != network.semantic.semantic_execution_profile_id {
        return Err(ReferenceNodeError::LedgerProfileMismatch(
            "semantic_execution_profile_id",
        ));
    }
    if ledger
        .privacy_lifecycle
        .as_ref()
        .map(|privacy| privacy.profile())
        != network.privacy.as_ref()
    {
        return Err(ReferenceNodeError::LedgerProfileMismatch(
            "privacy_lifecycle",
        ));
    }
    match (&ledger.bridge_target_profile, &network.bridge_source_trust) {
        (Some(target), Some(_)) => {
            if !target
                .matches_network_profile(network)
                .map_err(|error| ReferenceNodeError::BridgeProfileActivation(error.to_string()))?
            {
                return Err(ReferenceNodeError::LedgerProfileMismatch(
                    "bridge_source_trust",
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(ReferenceNodeError::LedgerProfileMismatch(
                "bridge_source_trust",
            ));
        }
    }
    Ok(())
}

/// Fail-closed bridge import errors before an observable import outcome exists.
#[derive(Debug, Error)]
pub enum BridgeImportError {
    /// Target ledger verification or projection failed.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Target PoA coordination or scheduled-authority admission failed.
    #[error(transparent)]
    PoA(#[from] PoAError),
    /// Final Admission rejected the target envelope before commit.
    #[error("bridge import rejected: {0}")]
    Rejected(String),
}

/// Fail-closed reference-node startup errors.
#[derive(Debug, Error)]
pub enum ReferenceNodeError {
    /// Governance or local membership activation failed.
    #[error(transparent)]
    Membership(#[from] MembershipError),
    /// The durable ledger could not be opened or verified.
    #[error(transparent)]
    Ledger(#[from] crate::ledger::LedgerError),
    /// The active membership cannot produce the profile's deterministic PoA schedule.
    #[error(transparent)]
    PoA(#[from] PoAError),
    /// The configured local proposal key is not authorized for this validator.
    #[error(transparent)]
    SignerAuthorization(#[from] SignerAuthorizationError),
    /// The reference ledger is not bound to the exact active network contract.
    #[error("ledger profile does not match the Network Profile field {0}")]
    LedgerProfileMismatch(&'static str),
    /// The privacy profile could not be authenticated against governance and membership.
    #[error("privacy lifecycle profile activation failed: {0}")]
    PrivacyProfileActivation(String),
    /// The configured bridge source facts differ from the independently active context.
    #[error("source bridge profile activation failed: {0}")]
    BridgeProfileActivation(String),
}
