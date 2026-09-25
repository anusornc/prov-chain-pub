//! Universal durable admission for ordinary-provenance writes.
//!
//! The module deliberately puts the deep interface behind seven sealed concrete
//! ingress adapters. Each adapter can only submit a complete
//! [`AdmissionCandidate`] to the same private Final Admission implementation.
//! This module owns canonical envelope construction, every deterministic
//! pre-commit gate, the append-plus-`fsync` commit point, and projection recovery
//! from verified journal history.
//!
//! The older `Blockchain`/WAL runtime remains compatibility scaffolding rather
//! than a conforming reference-ledger ingress. Later node-runtime tickets can
//! consume these adapters without inheriting a second append capability.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::model::{BlankNode, GraphName, NamedNode, Quad, Subject, Term};
use oxigraph::store::Store;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Cursor, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::bridge::{
    BridgeError, BridgeExportCoreV1, BridgeExportDeclarationV1, BridgeExportTargetV1,
    BridgeImportPreflightV1, BridgeOriginEvidenceV1, BridgeSourceProfileV1, BridgeTargetProfileV1,
    EffectiveBridgeStateV1, MAX_BRIDGE_EXPORT_DECLARATION_BYTES, MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
};
use crate::network::convergence::{
    CommitReceipt, CommitStatus, NetworkConvergenceEvidence, PrivacyStateReceipt,
    REQUIRED_REFERENCE_RECEIPTS,
};
use crate::network::membership::SignedMembershipManifest;
use crate::ontology::{
    ActivatedSemanticPackage, SemanticAdmissionVerdict, SEMANTIC_EXECUTION_PROFILE_V1,
};
#[cfg(feature = "privacy-conformance")]
use crate::privacy::PrivacyLifecycleConformanceProfile;
use crate::privacy::{
    ActivatedPrivacyLifecycleProfile, EffectivePrivacyState, LivePrivacyReleaseEvidence,
    LivePrivacyReleasePath, LivePrivacyReleaseRequest, LivePrivacyReleaseResponse,
    PrivacyAdmissionAnchor, PrivacyControlTransition, PrivacyError, PrivacyKeyUse,
    PrivacyLifecycleProfile, PrivacyPermission, PrivacyReleaseEnvelope,
};

/// A fixed-size digest used for proposal, envelope, and state identities.
pub type LedgerHash = [u8; 32];

/// The first ordinary write kind supported by this tracer bullet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AdmissionKind {
    /// A public RDF provenance assertion set.
    OrdinaryProvenanceV1 = 1,
    /// One closed CanonicalPrivacyEncodingV1 control transition.
    PrivacyControlV1 = 2,
}

impl AdmissionKind {
    fn from_byte(value: u8) -> Result<Self, LedgerError> {
        match value {
            1 => Ok(Self::OrdinaryProvenanceV1),
            2 => Ok(Self::PrivacyControlV1),
            _ => Err(LedgerError::MalformedEnvelope(format!(
                "unknown admission kind {value}"
            ))),
        }
    }
}

/// The versioned public-state commitment scheme supported by this ledger.
///
/// The variant fixes Turtle input with no base-IRI expansion, block-index
/// document-local blank-node scope, the reserved block graph namespace,
/// RDFC-1.0 canonical N-Quads, and SHA-256. A future scheme must use a new
/// variant so replay never silently applies changed canonicalization rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StateCommitmentScheme {
    /// Issue #2's bounded RDFC-1.0/SHA-256/N-Quads scheme.
    Rdfc10Sha256NQuadsV1 = 1,
}

impl StateCommitmentScheme {
    fn from_byte(value: u8) -> Result<Self, LedgerError> {
        match value {
            1 => Ok(Self::Rdfc10Sha256NQuadsV1),
            _ => Err(LedgerError::MalformedEnvelope(format!(
                "unknown state commitment scheme {value}"
            ))),
        }
    }
}

/// The network-wide identity that an Issue #2 ledger binds into every envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerProfile {
    /// Network identifier.
    pub network_id: String,
    /// Exact network-profile identifier.
    pub profile_id: String,
    /// Shared ontology-package identifier.
    pub ontology_package_id: String,
    /// Shared ontology-package version.
    pub ontology_package_version: String,
    /// Content identity of the shared ontology package.
    pub ontology_package_hash: String,
    /// Bounded semantic execution profile selected by the network.
    pub semantic_execution_profile_id: String,
    /// Versioned public-state commitment and canonicalization policy.
    pub state_commitment_scheme: StateCommitmentScheme,
    /// Maximum canonical envelope size accepted by this profile.
    pub max_envelope_bytes: usize,
    /// Maximum public RDF quads admitted into the accumulated state.
    pub max_public_state_quads: usize,
    /// Maximum blank nodes canonicalized in the accumulated state.
    pub max_public_state_blank_nodes: usize,
    /// Maximum bounded structural canonicalization work per state transition.
    pub max_canonicalization_work: usize,
    /// Optional exact source-profile and outbound rule for Issue #11 exports.
    pub(crate) bridge_source_profile: Option<BridgeSourceProfileV1>,
    /// Optional independently activated target proof verifier for Issue #12 imports.
    pub(crate) bridge_target_profile: Option<BridgeTargetProfileV1>,
    /// Optional profile-pinned Issue #8 participant/key lifecycle contract.
    pub(crate) privacy_lifecycle: Option<ActivatedPrivacyLifecycleProfile>,
    /// Governance-verified, non-deployable Issue #8/#9 conformance slice.
    #[cfg(feature = "privacy-conformance")]
    pub(crate) privacy_conformance_slice: Option<PrivacyLifecycleConformanceProfile>,
}

impl LedgerProfile {
    /// Create a profile with the repository's bounded default semantic package.
    pub fn new(network_id: impl Into<String>, profile_id: impl Into<String>) -> Self {
        Self {
            network_id: network_id.into(),
            profile_id: profile_id.into(),
            ontology_package_id: "provchain.shared-ontology.default".to_string(),
            ontology_package_version: "0.1.0".to_string(),
            ontology_package_hash: "0".repeat(64),
            semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
            state_commitment_scheme: StateCommitmentScheme::Rdfc10Sha256NQuadsV1,
            max_envelope_bytes: MAX_ENVELOPE_BYTES,
            max_public_state_quads: MAX_PUBLIC_STATE_QUADS,
            max_public_state_blank_nodes: MAX_CANONICALIZATION_BLANK_NODES,
            max_canonicalization_work: MAX_CANONICALIZATION_WORK,
            bridge_source_profile: None,
            bridge_target_profile: None,
            privacy_lifecycle: None,
            #[cfg(feature = "privacy-conformance")]
            privacy_conformance_slice: None,
        }
    }

    /// Bind the profile to one exact ontology package identity.
    pub fn with_semantic_package(
        mut self,
        package_id: impl Into<String>,
        package_version: impl Into<String>,
        package_hash: impl Into<String>,
    ) -> Self {
        self.ontology_package_id = package_id.into();
        self.ontology_package_version = package_version.into();
        self.ontology_package_hash = package_hash.into();
        self
    }

    /// Bind a non-default semantic execution profile identifier.
    pub fn with_semantic_execution_profile(
        mut self,
        semantic_execution_profile_id: impl Into<String>,
    ) -> Self {
        self.semantic_execution_profile_id = semantic_execution_profile_id.into();
        self
    }

    /// Set a lower profile-specific envelope limit for bounded tests/deployments.
    pub fn with_max_envelope_bytes(mut self, max_envelope_bytes: usize) -> Self {
        self.max_envelope_bytes = max_envelope_bytes;
        self
    }

    /// Set lower deterministic bounds for public-state canonicalization.
    pub fn with_state_commitment_bounds(
        mut self,
        max_public_state_quads: usize,
        max_public_state_blank_nodes: usize,
        max_canonicalization_work: usize,
    ) -> Self {
        self.max_public_state_quads = max_public_state_quads;
        self.max_public_state_blank_nodes = max_public_state_blank_nodes;
        self.max_canonicalization_work = max_canonicalization_work;
        self
    }

    /// Bind the non-target-writing source half of `ProvChainBridgeSuiteV1`.
    pub fn with_bridge_source_profile(
        mut self,
        bridge_source_profile: BridgeSourceProfileV1,
    ) -> Self {
        self.bridge_source_profile = Some(bridge_source_profile);
        self
    }

    /// Bind the target verifier used by journal-derived bridge import admission.
    pub fn with_bridge_target_profile(
        mut self,
        bridge_target_profile: BridgeTargetProfileV1,
    ) -> Self {
        self.bridge_target_profile = Some(bridge_target_profile);
        self
    }

    /// Activate the closed participant/key lifecycle slice for this exact profile.
    pub fn with_privacy_lifecycle(
        mut self,
        privacy_lifecycle: ActivatedPrivacyLifecycleProfile,
    ) -> Self {
        self.privacy_lifecycle = Some(privacy_lifecycle);
        self
    }

    /// Bind the non-deployable Issue #8/#9 privacy conformance slice.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn with_privacy_conformance_slice(
        mut self,
        privacy_conformance_slice: PrivacyLifecycleConformanceProfile,
    ) -> Self {
        self.privacy_conformance_slice = Some(privacy_conformance_slice);
        self
    }

    fn validate(&self) -> Result<(), LedgerError> {
        if self.state_commitment_scheme != StateCommitmentScheme::Rdfc10Sha256NQuadsV1 {
            return Err(LedgerError::InvalidProfile(
                "unsupported state commitment scheme".to_string(),
            ));
        }
        validate_text(&self.network_id, "network_id", MAX_TEXT_BYTES)?;
        validate_text(&self.profile_id, "profile_id", MAX_TEXT_BYTES)?;
        validate_text(
            &self.ontology_package_id,
            "ontology_package_id",
            MAX_TEXT_BYTES,
        )?;
        validate_text(
            &self.ontology_package_version,
            "ontology_package_version",
            MAX_TEXT_BYTES,
        )?;
        validate_text(
            &self.ontology_package_hash,
            "ontology_package_hash",
            MAX_TEXT_BYTES,
        )?;
        validate_text(
            &self.semantic_execution_profile_id,
            "semantic_execution_profile_id",
            MAX_TEXT_BYTES,
        )?;
        if self.semantic_execution_profile_id != SEMANTIC_EXECUTION_PROFILE_V1 {
            return Err(LedgerError::InvalidProfile(format!(
                "unsupported semantic execution profile {}",
                self.semantic_execution_profile_id
            )));
        }
        if self.max_envelope_bytes == 0 || self.max_envelope_bytes > MAX_ENVELOPE_BYTES {
            return Err(LedgerError::InvalidProfile(format!(
                "max_envelope_bytes must be between 1 and {MAX_ENVELOPE_BYTES}"
            )));
        }
        if self.max_public_state_quads == 0 || self.max_public_state_quads > MAX_PUBLIC_STATE_QUADS
        {
            return Err(LedgerError::InvalidProfile(format!(
                "max_public_state_quads must be between 1 and {MAX_PUBLIC_STATE_QUADS}"
            )));
        }
        if self.max_public_state_blank_nodes == 0
            || self.max_public_state_blank_nodes > MAX_CANONICALIZATION_BLANK_NODES
        {
            return Err(LedgerError::InvalidProfile(format!(
                "max_public_state_blank_nodes must be between 1 and {MAX_CANONICALIZATION_BLANK_NODES}"
            )));
        }
        if self.max_canonicalization_work == 0
            || self.max_canonicalization_work > MAX_CANONICALIZATION_WORK
        {
            return Err(LedgerError::InvalidProfile(format!(
                "max_canonicalization_work must be between 1 and {MAX_CANONICALIZATION_WORK}"
            )));
        }
        #[cfg(feature = "privacy-conformance")]
        if self.privacy_lifecycle.is_some() && self.privacy_conformance_slice.is_some() {
            return Err(LedgerError::InvalidProfile(
                "privacy activation and conformance slice are mutually exclusive".to_string(),
            ));
        }
        if let Some(privacy_lifecycle) = self.privacy_profile() {
            privacy_lifecycle
                .validate()
                .map_err(|error| LedgerError::InvalidProfile(error.to_string()))?;
        }
        if let Some(bridge_source_profile) = &self.bridge_source_profile {
            if bridge_source_profile.network_id() != self.network_id
                || bridge_source_profile.profile_id() != self.profile_id
                || bridge_source_profile
                    .canonical_bytes()
                    .map_err(|error| LedgerError::InvalidProfile(error.to_string()))?
                    .is_empty()
            {
                return Err(LedgerError::InvalidProfile(
                    "bridge source profile does not match the ledger profile".to_string(),
                ));
            }
        }
        if self.bridge_source_profile.is_some() && self.bridge_target_profile.is_some() {
            return Err(LedgerError::InvalidProfile(
                "source and target bridge roles are mutually exclusive".to_string(),
            ));
        }
        if let Some(target) = &self.bridge_target_profile {
            if !target.matches_target(
                &self.network_id,
                &self.profile_id,
                &self.ontology_package_id,
                &self.ontology_package_version,
                &self.ontology_package_hash,
                &self.semantic_execution_profile_id,
            ) {
                return Err(LedgerError::InvalidProfile(
                    "bridge target profile does not match the ledger semantic contract".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn privacy_profile(&self) -> Option<&PrivacyLifecycleProfile> {
        let activated = self
            .privacy_lifecycle
            .as_ref()
            .map(ActivatedPrivacyLifecycleProfile::profile);
        #[cfg(feature = "privacy-conformance")]
        {
            activated.or_else(|| {
                self.privacy_conformance_slice
                    .as_ref()
                    .map(PrivacyLifecycleConformanceProfile::profile)
            })
        }
        #[cfg(not(feature = "privacy-conformance"))]
        {
            activated
        }
    }

    fn state_commitment_bounds(&self) -> StateCommitmentBounds {
        StateCommitmentBounds {
            max_public_state_quads: self.max_public_state_quads,
            max_public_state_blank_nodes: self.max_public_state_blank_nodes,
            max_canonicalization_work: self.max_canonicalization_work,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StateCommitmentBounds {
    max_public_state_quads: usize,
    max_public_state_blank_nodes: usize,
    max_canonicalization_work: usize,
}

/// An uncommitted ordinary provenance request plus its proposer evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionCandidate {
    /// Closed admission classification.
    pub admission_kind: AdmissionKind,
    /// Candidate network identity.
    pub network_id: String,
    /// Candidate profile identity.
    pub profile_id: String,
    /// Candidate ontology package identity.
    pub ontology_package_id: String,
    /// Candidate ontology package version.
    pub ontology_package_version: String,
    /// Candidate ontology package content identity.
    pub ontology_package_hash: String,
    /// Candidate semantic execution profile identity.
    pub semantic_execution_profile_id: String,
    /// Versioned public-state commitment and canonicalization policy.
    pub state_commitment_scheme: StateCommitmentScheme,
    /// Expected next ledger index.
    pub index: u64,
    /// Expected parent envelope hash.
    pub previous_envelope_hash: LedgerHash,
    /// Expected parent post-state commitment.
    pub previous_state_commitment: LedgerHash,
    /// Caller-supplied deterministic timestamp in milliseconds since Unix epoch.
    pub timestamp_millis: u64,
    /// Exact public RDF provenance bytes.
    pub public_provenance: Vec<u8>,
    /// Optional opaque encrypted payload reserved for later write kinds.
    pub encrypted_payload: Option<Vec<u8>>,
    /// Exact canonical PrivacyControlV1 transition bytes for the privacy kind.
    pub privacy_control: Option<Vec<u8>>,
    /// Exact canonical BridgeExportDeclarationV1 bytes for a source export.
    pub bridge_export_declaration: Option<Vec<u8>>,
    /// Exact canonical BridgeOriginEvidenceV1 for a target import.
    pub(crate) bridge_origin_evidence: Option<Vec<u8>>,
    /// Candidate's deterministic post-state commitment.
    pub post_state_commitment: LedgerHash,
    /// Proposer's Ed25519 verification key.
    pub proposer_public_key: [u8; 32],
    /// Digest signed by the proposer.
    pub proposal_digest: LedgerHash,
    /// Proposer signature over `proposal_digest`.
    pub proposer_signature: [u8; 64],
}

impl AdmissionCandidate {
    /// Construct an unsigned ordinary candidate for an exact parent position.
    pub fn new_ordinary(
        profile: &LedgerProfile,
        index: u64,
        previous_envelope_hash: LedgerHash,
        previous_state_commitment: LedgerHash,
        timestamp_millis: u64,
        public_provenance: impl AsRef<[u8]>,
    ) -> Result<Self, LedgerError> {
        profile.validate()?;
        let public_provenance = public_provenance.as_ref().to_vec();
        validate_public_provenance(&public_provenance)?;

        Ok(Self {
            admission_kind: AdmissionKind::OrdinaryProvenanceV1,
            network_id: profile.network_id.clone(),
            profile_id: profile.profile_id.clone(),
            ontology_package_id: profile.ontology_package_id.clone(),
            ontology_package_version: profile.ontology_package_version.clone(),
            ontology_package_hash: profile.ontology_package_hash.clone(),
            semantic_execution_profile_id: profile.semantic_execution_profile_id.clone(),
            state_commitment_scheme: profile.state_commitment_scheme,
            index,
            previous_envelope_hash,
            previous_state_commitment,
            timestamp_millis,
            post_state_commitment: calculate_payload_state_commitment(
                index,
                &public_provenance,
                profile.state_commitment_bounds(),
            )?,
            public_provenance,
            encrypted_payload: None,
            privacy_control: None,
            bridge_export_declaration: None,
            bridge_origin_evidence: None,
            proposer_public_key: [0; 32],
            proposal_digest: [0; 32],
            proposer_signature: [0; 64],
        })
    }

    /// Construct an unsigned privacy-control candidate for an exact parent position.
    pub fn new_privacy_control(
        profile: &LedgerProfile,
        index: u64,
        previous_envelope_hash: LedgerHash,
        previous_state_commitment: LedgerHash,
        timestamp_millis: u64,
        transition: &PrivacyControlTransition,
    ) -> Result<Self, LedgerError> {
        profile.validate()?;
        if profile.privacy_profile().is_none() {
            return Err(LedgerError::InvalidProfile(
                "privacy lifecycle is not active for this ledger profile".to_string(),
            ));
        }
        let privacy_control = transition.canonical_bytes();
        PrivacyControlTransition::decode(&privacy_control)
            .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))?;
        Ok(Self {
            admission_kind: AdmissionKind::PrivacyControlV1,
            network_id: profile.network_id.clone(),
            profile_id: profile.profile_id.clone(),
            ontology_package_id: profile.ontology_package_id.clone(),
            ontology_package_version: profile.ontology_package_version.clone(),
            ontology_package_hash: profile.ontology_package_hash.clone(),
            semantic_execution_profile_id: profile.semantic_execution_profile_id.clone(),
            state_commitment_scheme: profile.state_commitment_scheme,
            index,
            previous_envelope_hash,
            previous_state_commitment,
            timestamp_millis,
            public_provenance: Vec::new(),
            encrypted_payload: None,
            privacy_control: Some(privacy_control),
            bridge_export_declaration: None,
            bridge_origin_evidence: None,
            post_state_commitment: previous_state_commitment,
            proposer_public_key: [0; 32],
            proposal_digest: [0; 32],
            proposer_signature: [0; 64],
        })
    }

    /// Sign the exact candidate body with the proposer key.
    pub fn sign(mut self, signing_key: &SigningKey) -> Result<Self, LedgerError> {
        self.proposer_public_key = signing_key.verifying_key().to_bytes();
        self.proposal_digest = self.compute_proposal_digest()?;
        self.proposer_signature = signing_key.sign(&self.proposal_digest).to_bytes();
        self.validate_shape()?;
        Ok(self)
    }

    /// Set the independently staged post-state commitment before signing.
    ///
    /// The low-level constructor can calculate the state of a single payload,
    /// which is sufficient at genesis.  A caller constructing a candidate for
    /// an existing ledger prefix must replace that value with the full staged
    /// dataset commitment, normally by using [`Ledger::create_ordinary_candidate`].
    pub fn with_post_state_commitment(mut self, post_state_commitment: LedgerHash) -> Self {
        self.post_state_commitment = post_state_commitment;
        self
    }

    /// Return the canonical unsigned proposal bytes used by the proposal digest.
    pub fn proposal_bytes(&self) -> Result<Vec<u8>, LedgerError> {
        self.encode_proposal_body()
    }

    /// Decode the exact canonical unsigned body retained by the PoA Signing Fence.
    pub(crate) fn from_proposal_bytes(bytes: &[u8]) -> Result<Self, LedgerError> {
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(LedgerError::OversizedEnvelope(bytes.len()));
        }

        let mut reader = CanonicalReader::new(bytes);
        reader.magic(PROPOSAL_MAGIC)?;
        let version = reader.u16()?;
        let admission_kind = AdmissionKind::from_byte(reader.u8()?)?;
        let version_has_bridge_export = version == BRIDGE_EXPORT_ENVELOPE_VERSION;
        let version_has_bridge_import = version == BRIDGE_IMPORT_ENVELOPE_VERSION;
        validate_codec_version(
            version,
            admission_kind,
            version_has_bridge_export,
            version_has_bridge_import,
        )
        .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))?;
        let network_id = reader.string(MAX_TEXT_BYTES, "network_id")?;
        let profile_id = reader.string(MAX_TEXT_BYTES, "profile_id")?;
        let ontology_package_id = reader.string(MAX_TEXT_BYTES, "ontology_package_id")?;
        let ontology_package_version = reader.string(MAX_TEXT_BYTES, "ontology_package_version")?;
        let ontology_package_hash = reader.string(MAX_TEXT_BYTES, "ontology_package_hash")?;
        let semantic_execution_profile_id =
            reader.string(MAX_TEXT_BYTES, "semantic_execution_profile_id")?;
        let state_commitment_scheme = StateCommitmentScheme::from_byte(reader.u8()?)?;
        let index = reader.u64()?;
        let previous_envelope_hash = reader.fixed::<32>()?;
        let previous_state_commitment = reader.fixed::<32>()?;
        let timestamp_millis = reader.u64()?;
        let public_provenance = reader.bytes(MAX_PAYLOAD_BYTES, "public_provenance")?;
        let encrypted_payload = reader.optional_bytes(MAX_PAYLOAD_BYTES, "encrypted_payload")?;
        let privacy_control = if version == PRIVACY_ENVELOPE_VERSION {
            reader.optional_bytes(MAX_PRIVACY_TRANSITION_BYTES, "privacy_control")?
        } else {
            None
        };
        let bridge_export_declaration = if version == BRIDGE_EXPORT_ENVELOPE_VERSION {
            Some(reader.bytes(
                MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
                "bridge_export_declaration",
            )?)
        } else {
            None
        };
        let bridge_origin_evidence = if version == BRIDGE_IMPORT_ENVELOPE_VERSION {
            Some(reader.bytes(MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES, "bridge_origin_evidence")?)
        } else {
            None
        };
        let post_state_commitment = reader.fixed::<32>()?;
        let proposer_public_key = reader.fixed::<32>()?;
        reader.finish()?;

        if proposer_public_key == [0; 32] {
            return Err(LedgerError::MalformedCandidate(
                "unsigned proposal has no proposer public key".to_string(),
            ));
        }
        VerifyingKey::from_bytes(&proposer_public_key).map_err(|error| {
            LedgerError::MalformedCandidate(format!("invalid proposer public key: {error}"))
        })?;
        let mut candidate = Self {
            admission_kind,
            network_id,
            profile_id,
            ontology_package_id,
            ontology_package_version,
            ontology_package_hash,
            semantic_execution_profile_id,
            state_commitment_scheme,
            index,
            previous_envelope_hash,
            previous_state_commitment,
            timestamp_millis,
            public_provenance,
            encrypted_payload,
            privacy_control,
            bridge_export_declaration,
            bridge_origin_evidence,
            post_state_commitment,
            proposer_public_key,
            proposal_digest: [0; 32],
            proposer_signature: [0; 64],
        };
        if candidate.encode_proposal_body()? != bytes {
            return Err(LedgerError::NonCanonicalEnvelope);
        }
        candidate.proposal_digest = candidate.compute_proposal_digest()?;
        Ok(candidate)
    }

    /// Verify the canonical digest and Ed25519 evidence on a received proposal.
    pub(crate) fn verify_signed_proposal(&self) -> Result<(), LedgerError> {
        self.validate_shape()?;
        if self.compute_proposal_digest()? != self.proposal_digest {
            return Err(LedgerError::MalformedCandidate(
                "proposal digest is not canonical for candidate fields".to_string(),
            ));
        }
        let verifying_key =
            VerifyingKey::from_bytes(&self.proposer_public_key).map_err(|error| {
                LedgerError::MalformedCandidate(format!("invalid proposer public key: {error}"))
            })?;
        let signature = Signature::from_bytes(&self.proposer_signature);
        verifying_key
            .verify(&self.proposal_digest, &signature)
            .map_err(|error| {
                LedgerError::MalformedCandidate(format!("invalid proposer signature: {error}"))
            })
    }

    /// Recover the signed proposal view of an authenticated committed envelope.
    ///
    /// Replication uses this only for PoA schedule and signing-fence checks. The
    /// exact received envelope bytes, rather than this view, remain the object
    /// submitted to Final Admission.
    pub(crate) fn from_admitted_envelope(envelope: &AdmittedBlockEnvelope) -> Self {
        Self {
            admission_kind: envelope.admission_kind,
            network_id: envelope.network_id.clone(),
            profile_id: envelope.profile_id.clone(),
            ontology_package_id: envelope.ontology_package_id.clone(),
            ontology_package_version: envelope.ontology_package_version.clone(),
            ontology_package_hash: envelope.ontology_package_hash.clone(),
            semantic_execution_profile_id: envelope.semantic_execution_profile_id.clone(),
            state_commitment_scheme: envelope.state_commitment_scheme,
            index: envelope.index,
            previous_envelope_hash: envelope.previous_envelope_hash,
            previous_state_commitment: envelope.previous_state_commitment,
            timestamp_millis: envelope.timestamp_millis,
            public_provenance: envelope.public_provenance.clone(),
            encrypted_payload: envelope.encrypted_payload.clone(),
            privacy_control: envelope.privacy_control.clone(),
            bridge_export_declaration: envelope.bridge_export_declaration.clone(),
            bridge_origin_evidence: envelope.bridge_origin_evidence.clone(),
            post_state_commitment: envelope.post_state_commitment,
            proposer_public_key: envelope.proposer_public_key,
            proposal_digest: envelope.proposal_digest,
            proposer_signature: envelope.proposer_signature,
        }
    }

    fn compute_proposal_digest(&self) -> Result<LedgerHash, LedgerError> {
        Ok(hash_parts(
            PROPOSAL_DIGEST_DOMAIN,
            &[&self.encode_proposal_body()?],
        ))
    }

    fn validate_shape(&self) -> Result<(), LedgerError> {
        validate_common_fields(&self.common_fields())?;
        if self.proposer_public_key == [0; 32]
            || self.proposal_digest == [0; 32]
            || self.proposer_signature == [0; 64]
        {
            return Err(LedgerError::MalformedCandidate(
                "proposer evidence is incomplete".to_string(),
            ));
        }
        Ok(())
    }

    fn encode_proposal_body(&self) -> Result<Vec<u8>, LedgerError> {
        validate_common_fields(&self.common_fields())?;

        let mut writer = CanonicalWriter::new(PROPOSAL_MAGIC);
        let version = codec_version(
            self.admission_kind,
            self.bridge_export_declaration.is_some(),
            self.bridge_origin_evidence.is_some(),
        );
        writer.u16(version);
        writer.u8(self.admission_kind as u8);
        writer.string(&self.network_id, MAX_TEXT_BYTES)?;
        writer.string(&self.profile_id, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_id, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_version, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_hash, MAX_TEXT_BYTES)?;
        writer.string(&self.semantic_execution_profile_id, MAX_TEXT_BYTES)?;
        writer.u8(self.state_commitment_scheme as u8);
        writer.u64(self.index);
        writer.fixed(&self.previous_envelope_hash);
        writer.fixed(&self.previous_state_commitment);
        writer.u64(self.timestamp_millis);
        writer.bytes(&self.public_provenance, MAX_PAYLOAD_BYTES)?;
        writer.optional_bytes(self.encrypted_payload.as_deref(), MAX_PAYLOAD_BYTES)?;
        if version == PRIVACY_ENVELOPE_VERSION {
            writer.optional_bytes(
                self.privacy_control.as_deref(),
                MAX_PRIVACY_TRANSITION_BYTES,
            )?;
        }
        if version == BRIDGE_EXPORT_ENVELOPE_VERSION {
            writer.bytes(
                self.bridge_export_declaration.as_deref().ok_or_else(|| {
                    LedgerError::MalformedCandidate(
                        "bridge export proposal is missing its declaration".to_string(),
                    )
                })?,
                MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
            )?;
        }
        if version == BRIDGE_IMPORT_ENVELOPE_VERSION {
            writer.bytes(
                self.bridge_origin_evidence.as_deref().ok_or_else(|| {
                    LedgerError::MalformedCandidate(
                        "bridge import proposal is missing origin evidence".to_string(),
                    )
                })?,
                MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
            )?;
        }
        writer.fixed(&self.post_state_commitment);
        writer.fixed(&self.proposer_public_key);
        Ok(writer.finish())
    }

    fn common_fields(&self) -> CommonAdmissionFields<'_> {
        CommonAdmissionFields {
            admission_kind: self.admission_kind,
            network_id: &self.network_id,
            profile_id: &self.profile_id,
            ontology_package_id: &self.ontology_package_id,
            ontology_package_version: &self.ontology_package_version,
            ontology_package_hash: &self.ontology_package_hash,
            semantic_execution_profile_id: &self.semantic_execution_profile_id,
            state_commitment_scheme: self.state_commitment_scheme,
            timestamp_millis: self.timestamp_millis,
            public_provenance: &self.public_provenance,
            encrypted_payload: self.encrypted_payload.as_deref(),
            privacy_control: self.privacy_control.as_deref(),
            bridge_export_declaration: self.bridge_export_declaration.as_deref(),
            bridge_origin_evidence: self.bridge_origin_evidence.as_deref(),
        }
    }
}

/// The canonical self-contained record that becomes a committed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBlockEnvelope {
    /// Envelope codec version.
    pub version: u16,
    /// Closed admission classification.
    pub admission_kind: AdmissionKind,
    /// Network identity.
    pub network_id: String,
    /// Profile identity.
    pub profile_id: String,
    /// Ontology package identity.
    pub ontology_package_id: String,
    /// Ontology package version.
    pub ontology_package_version: String,
    /// Ontology package content identity.
    pub ontology_package_hash: String,
    /// Bound semantic execution profile identity.
    pub semantic_execution_profile_id: String,
    /// Versioned public-state commitment and canonicalization policy.
    pub state_commitment_scheme: StateCommitmentScheme,
    /// Position in the journal-derived ledger prefix.
    pub index: u64,
    /// Previous envelope hash.
    pub previous_envelope_hash: LedgerHash,
    /// Previous post-state commitment.
    pub previous_state_commitment: LedgerHash,
    /// Deterministic timestamp in milliseconds since Unix epoch.
    pub timestamp_millis: u64,
    /// Exact public RDF provenance bytes.
    pub public_provenance: Vec<u8>,
    /// Optional opaque encrypted payload reserved for a future governed kind.
    pub encrypted_payload: Option<Vec<u8>>,
    /// Exact canonical PrivacyControlV1 transition bytes for a privacy envelope.
    pub privacy_control: Option<Vec<u8>>,
    /// Exact canonical source Bridge Export Declaration, excluded from public state.
    pub bridge_export_declaration: Option<Vec<u8>>,
    /// Exact canonical target BridgeOriginEvidenceV1, excluded from public state.
    pub bridge_origin_evidence: Option<Vec<u8>>,
    /// Post-state commitment.
    pub post_state_commitment: LedgerHash,
    /// Proposer verification key.
    pub proposer_public_key: [u8; 32],
    /// Signed proposal digest.
    pub proposal_digest: LedgerHash,
    /// Proposer signature.
    pub proposer_signature: [u8; 64],
    /// Hash of the complete envelope body, including proposer evidence.
    pub envelope_hash: LedgerHash,
}

impl AdmittedBlockEnvelope {
    /// Encode the envelope using the closed, deterministic binary codec.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, LedgerError> {
        self.validate_shape()?;
        self.validate_authentication()?;
        self.encode()
    }

    /// Decode and fully authenticate one canonical envelope.
    pub fn decode(bytes: &[u8]) -> Result<Self, LedgerError> {
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(LedgerError::OversizedEnvelope(bytes.len()));
        }

        let mut reader = CanonicalReader::new(bytes);
        reader.magic(ENVELOPE_MAGIC)?;
        let version = reader.u16()?;
        let admission_kind = AdmissionKind::from_byte(reader.u8()?)?;
        let version_has_bridge_export = version == BRIDGE_EXPORT_ENVELOPE_VERSION;
        let version_has_bridge_import = version == BRIDGE_IMPORT_ENVELOPE_VERSION;
        validate_codec_version(
            version,
            admission_kind,
            version_has_bridge_export,
            version_has_bridge_import,
        )?;
        let network_id = reader.string(MAX_TEXT_BYTES, "network_id")?;
        let profile_id = reader.string(MAX_TEXT_BYTES, "profile_id")?;
        let ontology_package_id = reader.string(MAX_TEXT_BYTES, "ontology_package_id")?;
        let ontology_package_version = reader.string(MAX_TEXT_BYTES, "ontology_package_version")?;
        let ontology_package_hash = reader.string(MAX_TEXT_BYTES, "ontology_package_hash")?;
        let semantic_execution_profile_id =
            reader.string(MAX_TEXT_BYTES, "semantic_execution_profile_id")?;
        let state_commitment_scheme = StateCommitmentScheme::from_byte(reader.u8()?)?;
        let index = reader.u64()?;
        let previous_envelope_hash = reader.fixed::<32>()?;
        let previous_state_commitment = reader.fixed::<32>()?;
        let timestamp_millis = reader.u64()?;
        let public_provenance = reader.bytes(MAX_PAYLOAD_BYTES, "public_provenance")?;
        let encrypted_payload = reader.optional_bytes(MAX_PAYLOAD_BYTES, "encrypted_payload")?;
        let privacy_control = if version == PRIVACY_ENVELOPE_VERSION {
            reader.optional_bytes(MAX_PRIVACY_TRANSITION_BYTES, "privacy_control")?
        } else {
            None
        };
        let bridge_export_declaration = if version == BRIDGE_EXPORT_ENVELOPE_VERSION {
            Some(reader.bytes(
                MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
                "bridge_export_declaration",
            )?)
        } else {
            None
        };
        let bridge_origin_evidence = if version == BRIDGE_IMPORT_ENVELOPE_VERSION {
            Some(reader.bytes(MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES, "bridge_origin_evidence")?)
        } else {
            None
        };
        let post_state_commitment = reader.fixed::<32>()?;
        let proposer_public_key = reader.fixed::<32>()?;
        let proposal_digest = reader.fixed::<32>()?;
        let proposer_signature = reader.fixed::<64>()?;
        let envelope_hash = reader.fixed::<32>()?;
        reader.finish()?;

        let envelope = Self {
            version,
            admission_kind,
            network_id,
            profile_id,
            ontology_package_id,
            ontology_package_version,
            ontology_package_hash,
            semantic_execution_profile_id,
            state_commitment_scheme,
            index,
            previous_envelope_hash,
            previous_state_commitment,
            timestamp_millis,
            public_provenance,
            encrypted_payload,
            privacy_control,
            bridge_export_declaration,
            bridge_origin_evidence,
            post_state_commitment,
            proposer_public_key,
            proposal_digest,
            proposer_signature,
            envelope_hash,
        };
        envelope.validate_shape()?;
        let canonical = envelope.encode()?;
        if canonical != bytes {
            return Err(LedgerError::NonCanonicalEnvelope);
        }
        envelope.validate_authentication()?;
        Ok(envelope)
    }

    /// Return the envelope's cryptographic identity.
    pub fn hash(&self) -> LedgerHash {
        self.envelope_hash
    }

    fn encode(&self) -> Result<Vec<u8>, LedgerError> {
        let mut writer = CanonicalWriter::new(ENVELOPE_MAGIC);
        self.encode_fields(&mut writer, true)?;
        let bytes = writer.finish();
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(LedgerError::OversizedEnvelope(bytes.len()));
        }
        Ok(bytes)
    }

    fn encode_without_hash(&self) -> Result<Vec<u8>, LedgerError> {
        let mut writer = CanonicalWriter::new(ENVELOPE_MAGIC);
        self.encode_fields(&mut writer, false)?;
        Ok(writer.finish())
    }

    fn encode_fields(
        &self,
        writer: &mut CanonicalWriter,
        include_hash: bool,
    ) -> Result<(), LedgerError> {
        writer.u16(self.version);
        writer.u8(self.admission_kind as u8);
        writer.string(&self.network_id, MAX_TEXT_BYTES)?;
        writer.string(&self.profile_id, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_id, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_version, MAX_TEXT_BYTES)?;
        writer.string(&self.ontology_package_hash, MAX_TEXT_BYTES)?;
        writer.string(&self.semantic_execution_profile_id, MAX_TEXT_BYTES)?;
        writer.u8(self.state_commitment_scheme as u8);
        writer.u64(self.index);
        writer.fixed(&self.previous_envelope_hash);
        writer.fixed(&self.previous_state_commitment);
        writer.u64(self.timestamp_millis);
        writer.bytes(&self.public_provenance, MAX_PAYLOAD_BYTES)?;
        writer.optional_bytes(self.encrypted_payload.as_deref(), MAX_PAYLOAD_BYTES)?;
        if self.version == PRIVACY_ENVELOPE_VERSION {
            writer.optional_bytes(
                self.privacy_control.as_deref(),
                MAX_PRIVACY_TRANSITION_BYTES,
            )?;
        }
        if self.version == BRIDGE_EXPORT_ENVELOPE_VERSION {
            writer.bytes(
                self.bridge_export_declaration.as_deref().ok_or_else(|| {
                    LedgerError::MalformedEnvelope(
                        "bridge export envelope is missing its declaration".to_string(),
                    )
                })?,
                MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
            )?;
        }
        if self.version == BRIDGE_IMPORT_ENVELOPE_VERSION {
            writer.bytes(
                self.bridge_origin_evidence.as_deref().ok_or_else(|| {
                    LedgerError::MalformedEnvelope(
                        "bridge import envelope is missing origin evidence".to_string(),
                    )
                })?,
                MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
            )?;
        }
        writer.fixed(&self.post_state_commitment);
        writer.fixed(&self.proposer_public_key);
        writer.fixed(&self.proposal_digest);
        writer.fixed(&self.proposer_signature);
        if include_hash {
            writer.fixed(&self.envelope_hash);
        }
        Ok(())
    }

    fn compute_envelope_hash(&self) -> Result<LedgerHash, LedgerError> {
        Ok(hash_parts(
            ENVELOPE_HASH_DOMAIN,
            &[&self.encode_without_hash()?],
        ))
    }

    fn verify_proposer_signature(&self) -> Result<(), LedgerError> {
        let verifying_key =
            VerifyingKey::from_bytes(&self.proposer_public_key).map_err(|error| {
                LedgerError::TamperedEnvelope(format!("invalid proposer public key: {error}"))
            })?;
        let signature = Signature::from_bytes(&self.proposer_signature);
        verifying_key
            .verify(&self.proposal_digest, &signature)
            .map_err(|error| {
                LedgerError::TamperedEnvelope(format!("invalid proposer signature: {error}"))
            })
    }

    fn validate_authentication(&self) -> Result<(), LedgerError> {
        let proposal_digest = hash_parts(
            PROPOSAL_DIGEST_DOMAIN,
            &[&candidate_body_from_envelope(self)?],
        );
        if proposal_digest != self.proposal_digest {
            return Err(LedgerError::TamperedEnvelope(
                "proposal digest does not match canonical envelope fields".to_string(),
            ));
        }
        if self.compute_envelope_hash()? != self.envelope_hash {
            return Err(LedgerError::TamperedEnvelope(
                "envelope hash does not match canonical body".to_string(),
            ));
        }
        self.verify_proposer_signature()
    }

    fn validate_shape(&self) -> Result<(), LedgerError> {
        validate_codec_version(
            self.version,
            self.admission_kind,
            self.bridge_export_declaration.is_some(),
            self.bridge_origin_evidence.is_some(),
        )?;
        validate_common_fields(&self.common_fields())?;
        if self.post_state_commitment == [0; 32]
            || self.proposer_public_key == [0; 32]
            || self.proposal_digest == [0; 32]
            || self.proposer_signature == [0; 64]
            || self.envelope_hash == [0; 32]
        {
            return Err(LedgerError::MalformedEnvelope(
                "required envelope identity/evidence is empty".to_string(),
            ));
        }
        Ok(())
    }

    fn common_fields(&self) -> CommonAdmissionFields<'_> {
        CommonAdmissionFields {
            admission_kind: self.admission_kind,
            network_id: &self.network_id,
            profile_id: &self.profile_id,
            ontology_package_id: &self.ontology_package_id,
            ontology_package_version: &self.ontology_package_version,
            ontology_package_hash: &self.ontology_package_hash,
            semantic_execution_profile_id: &self.semantic_execution_profile_id,
            state_commitment_scheme: self.state_commitment_scheme,
            timestamp_millis: self.timestamp_millis,
            public_provenance: &self.public_provenance,
            encrypted_payload: self.encrypted_payload.as_deref(),
            privacy_control: self.privacy_control.as_deref(),
            bridge_export_declaration: self.bridge_export_declaration.as_deref(),
            bridge_origin_evidence: self.bridge_origin_evidence.as_deref(),
        }
    }
}

/// The result of the deterministic Final Admission decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionOutcome {
    /// The exact envelope is now durably committed and projections are current.
    Committed {
        /// The canonical committed envelope.
        envelope: Box<AdmittedBlockEnvelope>,
    },
    /// The candidate was rejected before the journal commit point.
    Rejected {
        /// Stable human-readable reason for the deterministic rejection.
        reason: String,
    },
}

impl AdmissionOutcome {
    /// Return whether this outcome committed an envelope.
    pub fn is_committed(&self) -> bool {
        matches!(self, Self::Committed { .. })
    }
}

/// A journal replay result before projections are rebuilt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedJournalReplay {
    /// Envelopes in their exact committed order.
    pub envelopes: Vec<AdmittedBlockEnvelope>,
    /// Byte offset through the last complete verified frame.
    pub valid_bytes: u64,
    /// Whether an incomplete final frame was found.
    pub has_torn_tail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedJournalFrame {
    envelope_bytes: Vec<u8>,
    envelope: AdmittedBlockEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedJournalFrameReplay {
    frames: Vec<VerifiedJournalFrame>,
    valid_bytes: u64,
    has_torn_tail: bool,
}

/// A journal/replay operational failure.
#[derive(Debug, Error)]
pub enum LedgerError {
    /// Filesystem failure outside a deterministic candidate rejection.
    #[error("ledger I/O error: {0}")]
    Io(#[from] io::Error),
    /// Invalid profile configuration.
    #[error("invalid ledger profile: {0}")]
    InvalidProfile(String),
    /// Candidate shape failure.
    #[error("malformed admission candidate: {0}")]
    MalformedCandidate(String),
    /// Envelope shape or codec failure.
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(String),
    /// Envelope bytes use a different representation than the canonical codec.
    #[error("non-canonical envelope encoding")]
    NonCanonicalEnvelope,
    /// Envelope exceeds the fixed safety bound.
    #[error("envelope is oversized: {0} bytes")]
    OversizedEnvelope(usize),
    /// Envelope or signature tampering was detected.
    #[error("tampered envelope: {0}")]
    TamperedEnvelope(String),
    /// Journal bytes after a complete frame are unrecoverably corrupt.
    #[error("corrupt ledger journal: {0}")]
    CorruptJournal(String),
    /// A journal write or synchronization operation did not establish commit.
    #[error("journal commit failed: {0}")]
    JournalCommitFailed(String),
    /// A projection failed after the journal commit point.
    #[error("projection failed after journal commit for {envelope_hash}: {reason}")]
    ProjectionFailed {
        /// Committed envelope identity.
        envelope_hash: String,
        /// Projection failure detail.
        reason: String,
    },
    /// A deterministic protocol bound prevents unbounded canonicalization work.
    #[error("ledger protocol bound exceeded: {0}")]
    ProtocolBoundExceeded(String),
    /// The durable commit succeeded but the caller response was lost.
    #[error("commit response unavailable after journal commit for {envelope_hash}")]
    ResponseLost {
        /// Committed envelope identity to use for exact retry resolution.
        envelope_hash: String,
    },
    /// The node must replay before accepting another write.
    #[error("ledger projections are degraded; verified replay is required")]
    Degraded,
    /// Receipt-backed convergence evidence does not match the current ledger.
    #[error("invalid network-convergence evidence: {0}")]
    InvalidConvergenceEvidence(String),
    /// The node could not completely execute the active semantic contract.
    #[error("semantic admission incapacity: {0}")]
    SemanticIncapacity(String),
    /// The node could not reconstruct or execute its activated privacy lifecycle verifier.
    #[error("privacy lifecycle incapacity: {0}")]
    PrivacyIncapacity(String),
    /// A live protected-object release failed closed at the exact convergence barrier.
    #[error("live privacy release denied: {0}")]
    PrivacyReleaseDenied(String),
}

/// Append-only durable Ledger Journal. Its append method is private so
/// application callers can only commit through the sealed ingress adapters.
#[derive(Debug)]
pub struct LedgerJournal {
    path: PathBuf,
    file: File,
    fail_next_write: bool,
    fail_next_fsync: bool,
    requires_replay: bool,
}

impl LedgerJournal {
    /// Open a journal file, creating and synchronizing its fixed header.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        lock_journal(&file)?;

        if file.metadata()?.len() == 0 {
            file.write_all(JOURNAL_MAGIC)?;
            file.sync_all()?;
        }

        let mut journal = Self {
            path,
            file,
            fail_next_write: false,
            fail_next_fsync: false,
            requires_replay: false,
        };
        let replay = journal.verified_replay()?;
        journal.requires_replay = replay.has_torn_tail;
        Ok(journal)
    }

    /// Path of the authoritative journal file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read and verify the journal without rebuilding projections.
    pub fn verified_replay(&self) -> Result<VerifiedJournalReplay, LedgerError> {
        let replay = self.verified_frame_replay()?;
        Ok(VerifiedJournalReplay {
            envelopes: replay
                .frames
                .into_iter()
                .map(|frame| frame.envelope)
                .collect(),
            valid_bytes: replay.valid_bytes,
            has_torn_tail: replay.has_torn_tail,
        })
    }

    fn verified_frame_replay(&self) -> Result<VerifiedJournalFrameReplay, LedgerError> {
        let bytes = fs::read(&self.path)?;
        if bytes.len() < JOURNAL_MAGIC.len() || &bytes[..JOURNAL_MAGIC.len()] != JOURNAL_MAGIC {
            return Err(LedgerError::CorruptJournal(
                "invalid or missing journal header".to_string(),
            ));
        }

        let mut offset = JOURNAL_MAGIC.len();
        let mut frames = Vec::new();
        while offset < bytes.len() {
            let frame_start = offset;
            if bytes.len() - offset < FRAME_LENGTH_BYTES {
                return Ok(VerifiedJournalFrameReplay {
                    frames,
                    valid_bytes: frame_start as u64,
                    has_torn_tail: true,
                });
            }

            let frame_len = u32::from_be_bytes(
                bytes[offset..offset + FRAME_LENGTH_BYTES]
                    .try_into()
                    .map_err(|_| LedgerError::CorruptJournal("invalid frame length".to_string()))?,
            ) as usize;
            offset += FRAME_LENGTH_BYTES;
            if frame_len == 0 || frame_len > MAX_ENVELOPE_BYTES {
                return Err(LedgerError::CorruptJournal(format!(
                    "invalid frame length {frame_len} at byte {frame_start}"
                )));
            }

            let frame_total = frame_len
                .checked_add(FRAME_CHECKSUM_BYTES)
                .ok_or_else(|| LedgerError::CorruptJournal("frame length overflow".to_string()))?;
            if bytes.len() - offset < frame_total {
                return Ok(VerifiedJournalFrameReplay {
                    frames,
                    valid_bytes: frame_start as u64,
                    has_torn_tail: true,
                });
            }

            let envelope_bytes = &bytes[offset..offset + frame_len];
            offset += frame_len;
            let checksum = &bytes[offset..offset + FRAME_CHECKSUM_BYTES];
            offset += FRAME_CHECKSUM_BYTES;
            let expected_checksum = hash_parts(JOURNAL_FRAME_DOMAIN, &[envelope_bytes]);
            if checksum != expected_checksum {
                return Err(LedgerError::CorruptJournal(format!(
                    "frame checksum mismatch at byte {frame_start}"
                )));
            }
            frames.push(VerifiedJournalFrame {
                envelope_bytes: envelope_bytes.to_vec(),
                envelope: AdmittedBlockEnvelope::decode(envelope_bytes)?,
            });
        }

        Ok(VerifiedJournalFrameReplay {
            frames,
            valid_bytes: offset as u64,
            has_torn_tail: false,
        })
    }

    /// Return the raw journal bytes for diagnostics/evidence.
    pub fn bytes(&self) -> Result<Vec<u8>, LedgerError> {
        Ok(fs::read(&self.path)?)
    }

    #[cfg(test)]
    fn inject_next_write_failure(&mut self) {
        self.fail_next_write = true;
    }

    #[cfg(test)]
    fn inject_next_fsync_failure(&mut self) {
        self.fail_next_fsync = true;
    }

    fn requires_replay(&self) -> bool {
        self.requires_replay
    }

    fn clear_replay_requirement(&mut self) {
        self.requires_replay = false;
    }

    fn append(&mut self, envelope: &AdmittedBlockEnvelope) -> Result<(), LedgerError> {
        if self.requires_replay {
            return Err(LedgerError::JournalCommitFailed(
                "journal durability outcome is uncertain; verified replay is required".to_string(),
            ));
        }
        let envelope_bytes = envelope.canonical_bytes()?;
        if envelope_bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(LedgerError::OversizedEnvelope(envelope_bytes.len()));
        }

        let replay = self.verified_replay()?;
        if replay.has_torn_tail {
            self.requires_replay = true;
            return Err(LedgerError::JournalCommitFailed(
                "journal has an incomplete tail; explicit torn-tail recovery is required"
                    .to_string(),
            ));
        }

        let start = self.file.seek(SeekFrom::End(0))?;
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(LedgerError::JournalCommitFailed(
                "injected journal write failure".to_string(),
            ));
        }

        let frame_len = u32::try_from(envelope_bytes.len())
            .map_err(|_| LedgerError::OversizedEnvelope(envelope_bytes.len()))?;
        let checksum = hash_parts(JOURNAL_FRAME_DOMAIN, &[&envelope_bytes]);
        let result = (|| -> Result<(), io::Error> {
            self.file.write_all(&frame_len.to_be_bytes())?;
            self.file.write_all(&envelope_bytes)?;
            self.file.write_all(&checksum)?;
            Ok(())
        })();
        if let Err(error) = result {
            self.requires_replay = true;
            return Err(LedgerError::JournalCommitFailed(format!(
                "journal write outcome is uncertain: {error}; verified replay is required"
            )));
        }

        if self.fail_next_fsync {
            self.fail_next_fsync = false;
            if let Err(error) = self.rollback_append(start) {
                self.requires_replay = true;
                return Err(LedgerError::JournalCommitFailed(format!(
                    "injected journal fsync failure and rollback failed: {error}"
                )));
            }
            return Err(LedgerError::JournalCommitFailed(
                "injected journal fsync failure".to_string(),
            ));
        }

        if let Err(error) = self.file.sync_all() {
            self.requires_replay = true;
            return Err(LedgerError::JournalCommitFailed(format!(
                "journal fsync outcome is uncertain: {error}; verified replay is required"
            )));
        }
        Ok(())
    }

    fn rollback_append(&mut self, start: u64) -> Result<(), LedgerError> {
        self.file.set_len(start)?;
        self.file.seek(SeekFrom::End(0))?;
        self.file.sync_all()?;
        Ok(())
    }

    fn truncate_to(&self, valid_bytes: u64) -> Result<(), LedgerError> {
        let file = OpenOptions::new().write(true).open(&self.path)?;
        file.set_len(valid_bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

fn lock_journal(file: &File) -> io::Result<()> {
    #[cfg(unix)]
    {
        // SAFETY: `file` is an open descriptor owned by this journal, and the
        // call only applies an advisory lock to that descriptor.
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(unix))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "durable journal locking is unsupported on this platform",
        ))
    }
}

/// Journal-derived runtime views.  None of these fields is commit authority.
pub struct LedgerProjections {
    chain: Vec<AdmittedBlockEnvelope>,
    rdf_store: Store,
    envelope_index: BTreeMap<LedgerHash, u64>,
    privacy_state: EffectivePrivacyState,
    bridge_state: EffectiveBridgeStateV1,
}

impl LedgerProjections {
    fn new() -> Result<Self, LedgerError> {
        Ok(Self {
            chain: Vec::new(),
            rdf_store: Store::new()
                .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?,
            envelope_index: BTreeMap::new(),
            privacy_state: EffectivePrivacyState::new(),
            bridge_state: EffectiveBridgeStateV1::default(),
        })
    }

    fn rebuild(
        envelopes: &[AdmittedBlockEnvelope],
        profile: &LedgerProfile,
    ) -> Result<Self, LedgerError> {
        let mut projections = Self::new()?;
        let mut previous_hash = [0; 32];
        let mut previous_state = genesis_state_commitment();
        let mut ledger_prefix_hash = crate::network::convergence::genesis_ledger_prefix_hash();
        for (expected_index, envelope) in envelopes.iter().enumerate() {
            validate_envelope_against_profile(envelope, profile)?;
            let expected_index = expected_index as u64;
            if envelope.index != expected_index {
                return Err(LedgerError::CorruptJournal(format!(
                    "expected journal index {expected_index}, got {}",
                    envelope.index
                )));
            }
            if envelope.previous_envelope_hash != previous_hash {
                return Err(LedgerError::CorruptJournal(format!(
                    "wrong parent at index {}",
                    envelope.index
                )));
            }
            if envelope.previous_state_commitment != previous_state {
                return Err(LedgerError::CorruptJournal(format!(
                    "wrong parent state commitment at index {}",
                    envelope.index
                )));
            }
            let expected_post_state = match envelope.admission_kind {
                AdmissionKind::OrdinaryProvenanceV1 => projections
                    .staged_post_state_commitment(
                        envelope.index,
                        &envelope.public_provenance,
                        profile.state_commitment_bounds(),
                    )
                    .map_err(|error| LedgerError::CorruptJournal(error.to_string()))?,
                AdmissionKind::PrivacyControlV1 => previous_state,
            };
            if envelope.post_state_commitment != expected_post_state {
                return Err(LedgerError::CorruptJournal(format!(
                    "wrong post-state commitment at index {}",
                    envelope.index
                )));
            }
            let proposal_digest = hash_parts(
                PROPOSAL_DIGEST_DOMAIN,
                &[&candidate_body_from_envelope(envelope)?],
            );
            if proposal_digest != envelope.proposal_digest {
                return Err(LedgerError::CorruptJournal(format!(
                    "wrong proposal digest at index {}",
                    envelope.index
                )));
            }
            if envelope.compute_envelope_hash()? != envelope.envelope_hash {
                return Err(LedgerError::CorruptJournal(format!(
                    "wrong envelope hash at index {}",
                    envelope.index
                )));
            }
            envelope.verify_proposer_signature()?;
            let resulting_ledger_prefix_hash =
                crate::network::convergence::extend_ledger_prefix_hash(
                    ledger_prefix_hash,
                    envelope.index,
                    &envelope.canonical_bytes()?,
                );
            let staged_privacy = match envelope.admission_kind {
                AdmissionKind::OrdinaryProvenanceV1 => None,
                AdmissionKind::PrivacyControlV1 => Some(
                    stage_privacy_envelope(
                        profile,
                        &projections.privacy_state,
                        ledger_prefix_hash,
                        envelope,
                    )
                    .map_err(|error| LedgerError::CorruptJournal(error.to_string()))?,
                ),
            };
            let staged_bridge = if let Some(origin) = &envelope.bridge_origin_evidence {
                let target = profile.bridge_target_profile.as_ref().ok_or_else(|| {
                    LedgerError::CorruptJournal(
                        "bridge import exists without an active target profile".to_string(),
                    )
                })?;
                let verified = target
                    .verify_origin(origin, &envelope.public_provenance)
                    .map_err(|error| LedgerError::CorruptJournal(error.to_string()))?;
                let mut state = projections.bridge_state.clone();
                state
                    .insert_verified(
                        verified,
                        envelope.index,
                        envelope.envelope_hash,
                        resulting_ledger_prefix_hash,
                    )
                    .map_err(|error| LedgerError::CorruptJournal(error.to_string()))?;
                Some(state)
            } else {
                None
            };
            projections.apply(envelope, staged_privacy, staged_bridge)?;
            ledger_prefix_hash = resulting_ledger_prefix_hash;
            previous_hash = envelope.envelope_hash;
            previous_state = envelope.post_state_commitment;
        }
        Ok(projections)
    }

    fn staged_post_state_commitment(
        &self,
        index: u64,
        public_provenance: &[u8],
        bounds: StateCommitmentBounds,
    ) -> Result<LedgerHash, LedgerError> {
        let mut staged_quads = Vec::new();
        for result in self.rdf_store.iter() {
            let quad = result.map_err(|error| {
                LedgerError::MalformedEnvelope(format!(
                    "public state projection read failed: {error}"
                ))
            })?;
            staged_quads.push(quad);
        }
        staged_quads.extend(projected_public_quads(index, public_provenance)?);
        calculate_public_state_commitment(&staged_quads, bounds)
    }

    fn apply(
        &mut self,
        envelope: &AdmittedBlockEnvelope,
        staged_privacy_state: Option<EffectivePrivacyState>,
        staged_bridge_state: Option<EffectiveBridgeStateV1>,
    ) -> Result<(), LedgerError> {
        if self.envelope_index.contains_key(&envelope.envelope_hash) {
            return Ok(());
        }
        if let Some(state) = staged_bridge_state {
            self.bridge_state = state;
        }
        match envelope.admission_kind {
            AdmissionKind::OrdinaryProvenanceV1 => {
                for projected_quad in
                    projected_public_quads(envelope.index, &envelope.public_provenance)?
                {
                    self.rdf_store.insert(&projected_quad).map_err(|error| {
                        LedgerError::MalformedEnvelope(format!(
                            "RDF projection insert failed: {error}"
                        ))
                    })?;
                }
            }
            AdmissionKind::PrivacyControlV1 => {
                self.privacy_state = staged_privacy_state.ok_or_else(|| {
                    LedgerError::MalformedEnvelope(
                        "privacy projection was not staged before apply".to_string(),
                    )
                })?;
            }
        }
        self.envelope_index
            .insert(envelope.envelope_hash, envelope.index);
        self.chain.push(envelope.clone());
        Ok(())
    }
}

/// The durable ledger behind the single universal Final Admission seam.
///
/// Protected-data decryption is deliberately absent from this node-side API:
///
/// ```compile_fail
/// use provchain_org::ledger::Ledger;
/// use uuid::Uuid;
///
/// fn node_cannot_decrypt(ledger: &Ledger, object_id: Uuid) {
///     let _ = ledger.decrypt_protected_object(object_id);
/// }
/// ```
pub struct Ledger {
    profile: LedgerProfile,
    semantic_package: ActivatedSemanticPackage,
    journal: LedgerJournal,
    projections: LedgerProjections,
    fail_next_projection: bool,
    fail_next_response: bool,
    degraded: bool,
}

/// Deterministic journal-boundary failure used by conformance tests.
#[cfg(any(feature = "privacy-conformance", feature = "bridge-conformance"))]
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerCommitFailpoint {
    /// Fail before any journal frame can be committed.
    BeforeJournalWrite,
    /// Fail while synchronizing the newly written journal frame.
    BeforeJournalFsync,
    /// Fail projection application after the journal commit point.
    AfterJournalCommitBeforeProjection,
    /// Lose the response after journal commit and projection application.
    AfterProjectionBeforeResponse,
}

/// Opaque, borrow-scoped proof of one current journal-verified privacy prefix.
///
/// Only [`Ledger`] can construct this capability. Its borrow prevents the same
/// ledger from advancing while participant custody validates an operation, so
/// callers cannot combine an arbitrary privacy projection with an unrelated
/// admission anchor.
pub struct VerifiedPrivacyLedgerPrefix<'ledger> {
    profile: &'ledger PrivacyLifecycleProfile,
    state: &'ledger EffectivePrivacyState,
    next_anchor: PrivacyAdmissionAnchor,
}

impl VerifiedPrivacyLedgerPrefix<'_> {
    pub(crate) fn profile(&self) -> &crate::privacy::PrivacyLifecycleProfile {
        self.profile
    }

    pub(crate) fn state(&self) -> &EffectivePrivacyState {
        self.state
    }

    pub(crate) fn next_anchor(&self) -> &PrivacyAdmissionAnchor {
        &self.next_anchor
    }
}

/// Opaque proof that the current journal-verified privacy prefix is also backed
/// by the exact three-receipt Network-Converged evidence from Issue #6.
pub struct NetworkConvergedPrivacyPrefix<'ledger> {
    verified: VerifiedPrivacyLedgerPrefix<'ledger>,
}

impl NetworkConvergedPrivacyPrefix<'_> {
    pub(crate) fn verified(&self) -> &VerifiedPrivacyLedgerPrefix<'_> {
        &self.verified
    }

    pub(crate) fn current_ledger_position(&self) -> Option<u64> {
        self.verified
            .next_anchor
            .expected_ledger_position()
            .checked_sub(1)
    }

    pub(crate) fn current_ledger_prefix_hash(&self) -> LedgerHash {
        self.verified.next_anchor.parent_ledger_prefix_hash()
    }
}

impl<'ledger> std::ops::Deref for NetworkConvergedPrivacyPrefix<'ledger> {
    type Target = VerifiedPrivacyLedgerPrefix<'ledger>;

    fn deref(&self) -> &Self::Target {
        &self.verified
    }
}

/// The one implementation authorized to commit to a [`Ledger`].
///
/// It is private so an ingress cannot construct a replacement boundary or gain
/// access to the journal append capability.
struct FinalAdmission<'ledger> {
    ledger: &'ledger mut Ledger,
}

impl FinalAdmission<'_> {
    fn submit(&mut self, candidate: AdmissionCandidate) -> Result<AdmissionOutcome, LedgerError> {
        self.ledger.final_admit(candidate)
    }

    fn submit_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
    ) -> Result<AdmissionOutcome, LedgerError> {
        self.ledger.final_admit_exact_envelope(envelope_bytes)
    }
}

macro_rules! define_write_adapter {
    ($(#[$adapter_doc:meta])* $adapter:ident, $factory:ident, $factory_doc:literal) => {
        $(#[$adapter_doc])*
        pub struct $adapter<'ledger> {
            admission: FinalAdmission<'ledger>,
        }

        impl $adapter<'_> {
            /// Submit the complete candidate to universal Final Admission.
            pub fn submit(
                &mut self,
                candidate: AdmissionCandidate,
            ) -> Result<AdmissionOutcome, LedgerError> {
                self.admission.submit(candidate)
            }
        }

        impl Ledger {
            #[doc = $factory_doc]
            pub fn $factory(&mut self) -> $adapter<'_> {
                $adapter {
                    admission: self.final_admission(),
                }
            }
        }
    };
}

define_write_adapter!(
    /// Sealed adapter for a complete local library or CLI candidate.
    LocalWriteAdapter,
    local_adapter,
    "Borrow the sealed local-write adapter."
);
define_write_adapter!(
    /// Sealed adapter for a complete candidate decoded by the HTTP API.
    ApiWriteAdapter,
    api_adapter,
    "Borrow the sealed API-write adapter."
);
define_write_adapter!(
    /// Sealed adapter for one complete candidate assembled from an atomic batch.
    BatchWriteAdapter,
    batch_adapter,
    "Borrow the sealed batch-write adapter."
);
define_write_adapter!(
    /// Sealed adapter for a complete candidate received during prefix synchronization.
    SynchronizationWriteAdapter,
    synchronization_adapter,
    "Borrow the sealed synchronization-write adapter."
);
define_write_adapter!(
    /// Sealed adapter for a complete candidate received through replication.
    ReplicationWriteAdapter,
    replication_adapter,
    "Borrow the sealed replication-write adapter."
);
define_write_adapter!(
    /// Sealed adapter for a complete candidate selected by proposal coordination.
    ProposalWriteAdapter,
    proposal_adapter,
    "Borrow the sealed proposal-write adapter."
);
/// Legacy ordinary-write adapter that cannot carry target bridge evidence.
pub struct BridgeOriginWriteAdapter<'ledger> {
    admission: FinalAdmission<'ledger>,
}

impl BridgeOriginWriteAdapter<'_> {
    /// Submit only an ordinary candidate without BridgeOriginEvidenceV1.
    pub fn submit(
        &mut self,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, LedgerError> {
        if candidate.bridge_origin_evidence.is_some() {
            return Err(LedgerError::MalformedCandidate(
                "target bridge imports must enter through the PoA Proposal Coordinator".to_string(),
            ));
        }
        self.admission.submit(candidate)
    }
}

impl Ledger {
    /// Borrow the legacy bridge-origin adapter for non-import compatibility only.
    pub fn bridge_origin_adapter(&mut self) -> BridgeOriginWriteAdapter<'_> {
        BridgeOriginWriteAdapter {
            admission: self.final_admission(),
        }
    }
}

impl ReplicationWriteAdapter<'_> {
    /// Submit exact producer envelope bytes through universal Final Admission.
    pub(crate) fn submit_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
    ) -> Result<AdmissionOutcome, LedgerError> {
        self.admission.submit_exact_envelope(envelope_bytes)
    }
}

impl SynchronizationWriteAdapter<'_> {
    /// Submit exact catch-up envelope bytes through universal Final Admission.
    pub(crate) fn submit_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
    ) -> Result<AdmissionOutcome, LedgerError> {
        self.admission.submit_exact_envelope(envelope_bytes)
    }
}

impl Ledger {
    /// Open a journal file and rebuild all projections through verified replay.
    pub fn open<P: AsRef<Path>>(
        journal_path: P,
        profile: LedgerProfile,
        semantic_package: ActivatedSemanticPackage,
    ) -> Result<Self, LedgerError> {
        profile.validate()?;
        validate_activated_semantic_package(&profile, &semantic_package)?;
        let journal = LedgerJournal::open(journal_path)?;
        let replay = journal.verified_replay()?;
        let projections = LedgerProjections::rebuild(&replay.envelopes, &profile)?;
        let parent_quads = asserted_public_state_quads(&replay.envelopes)
            .map_err(|error| LedgerError::SemanticIncapacity(error.to_string()))?;
        semantic_package
            .validate_committed_state(&parent_quads)
            .map_err(|error| LedgerError::SemanticIncapacity(error.to_string()))?;
        Ok(Self {
            profile,
            semantic_package,
            journal,
            projections,
            fail_next_projection: false,
            fail_next_response: false,
            degraded: replay.has_torn_tail,
        })
    }

    /// Open `<data_dir>/ledger.journal`.
    pub fn open_in_dir<P: AsRef<Path>>(
        data_dir: P,
        profile: LedgerProfile,
        semantic_package: ActivatedSemanticPackage,
    ) -> Result<Self, LedgerError> {
        Self::open(
            data_dir.as_ref().join("ledger.journal"),
            profile,
            semantic_package,
        )
    }

    /// Return the profile bound to this ledger.
    pub fn profile(&self) -> &LedgerProfile {
        &self.profile
    }

    /// Return the authoritative journal path.
    pub fn journal_path(&self) -> &Path {
        self.journal.path()
    }

    /// Return the journal-derived committed envelope projection.
    pub fn committed_envelopes(&self) -> &[AdmittedBlockEnvelope] {
        &self.projections.chain
    }

    /// Return the Oxigraph projection rebuilt from committed envelopes.
    pub fn rdf_store(&self) -> &Store {
        &self.projections.rdf_store
    }

    /// Return the derived envelope-to-index map.
    pub fn envelope_index(&self) -> &BTreeMap<LedgerHash, u64> {
        &self.projections.envelope_index
    }

    /// Return the verified journal-derived participant/key lifecycle projection.
    ///
    /// Authoritative privacy reads fail closed while the ledger requires replay
    /// or any projection is known to be degraded.
    pub fn effective_privacy_state(&self) -> Result<&EffectivePrivacyState, LedgerError> {
        if self.degraded || self.journal.requires_replay() {
            return Err(LedgerError::Degraded);
        }
        Ok(&self.projections.privacy_state)
    }

    /// Return Effective Bridge State rebuilt exclusively from verified target journal history.
    pub fn effective_bridge_state(&self) -> Result<&EffectiveBridgeStateV1, LedgerError> {
        if self.degraded || self.journal.requires_replay() {
            return Err(LedgerError::Degraded);
        }
        Ok(&self.projections.bridge_state)
    }

    /// Bind the active profile, replay-derived state, and exact next anchor into
    /// one unforgeable capability for participant-side custody operations.
    pub fn verified_privacy_prefix(&self) -> Result<VerifiedPrivacyLedgerPrefix<'_>, LedgerError> {
        let state = self.effective_privacy_state()?;
        let profile = self.profile.privacy_profile().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "privacy lifecycle is not active for this ledger profile".to_string(),
            )
        })?;
        let next_anchor = self.next_privacy_admission_anchor()?;
        Ok(VerifiedPrivacyLedgerPrefix {
            profile,
            state,
            next_anchor,
        })
    }

    /// Strengthen the current local privacy prefix with exact Issue #6
    /// Network-Converged evidence for this network, profile, position, and
    /// ordered journal prefix.
    pub fn network_converged_privacy_prefix(
        &self,
        evidence: &NetworkConvergenceEvidence,
    ) -> Result<NetworkConvergedPrivacyPrefix<'_>, LedgerError> {
        let state = self.effective_privacy_state()?;
        let tip = self.tip();
        let position = tip.index.ok_or_else(|| {
            LedgerError::InvalidConvergenceEvidence(
                "genesis has no receipt-backed committed envelope".to_string(),
            )
        })?;
        let prefix_hash = self.current_ledger_prefix_hash()?;
        let state_digest = state.digest();
        let mut privacy_receipt_nodes = BTreeSet::new();
        let privacy_receipts_match = evidence.privacy_receipts().len()
            == REQUIRED_REFERENCE_RECEIPTS
            && evidence.privacy_receipts().iter().all(|receipt| {
                receipt.privacy_state_digest() == state_digest
                    && receipt.network_id() == self.profile.network_id
                    && receipt.profile_id() == self.profile.profile_id
                    && receipt.ledger_position() == position
                    && receipt.envelope_hash() == tip.envelope_hash
                    && receipt.ledger_prefix_hash() == prefix_hash
                    && privacy_receipt_nodes.insert(receipt.node_id())
                    && evidence
                        .receipts()
                        .iter()
                        .any(|commit_receipt| commit_receipt == receipt.commit_receipt())
            });
        if evidence.ledger_position() != position
            || evidence.envelope_hash() != tip.envelope_hash
            || evidence.ledger_prefix_hash() != prefix_hash
            || evidence.receipts().len() != REQUIRED_REFERENCE_RECEIPTS
            || evidence.privacy_state_digest() != Some(state_digest)
            || !privacy_receipts_match
            || evidence.receipts().iter().any(|receipt| {
                receipt.network_id() != self.profile.network_id
                    || receipt.profile_id() != self.profile.profile_id
                    || receipt.ledger_position() != position
                    || receipt.envelope_hash() != tip.envelope_hash
                    || receipt.ledger_prefix_hash() != prefix_hash
            })
        {
            return Err(LedgerError::InvalidConvergenceEvidence(
                "evidence does not match the current local network/profile prefix".to_string(),
            ));
        }
        Ok(NetworkConvergedPrivacyPrefix {
            verified: VerifiedPrivacyLedgerPrefix {
                profile: self.profile.privacy_profile().ok_or_else(|| {
                    LedgerError::InvalidProfile(
                        "privacy lifecycle is not active for this ledger profile".to_string(),
                    )
                })?,
                state,
                next_anchor: self.next_privacy_admission_anchor()?,
            },
        })
    }

    /// Return one opaque protected-object release only after exact three-node
    /// Network-Converged evidence has been checked against this current ledger.
    ///
    /// The node copies the immutable ciphertext and exactly one applicable
    /// envelope. It never opens, reconstructs, or returns plaintext or a DEK.
    pub fn live_privacy_release(
        &self,
        evidence: &NetworkConvergenceEvidence,
        request: &LivePrivacyReleaseRequest,
    ) -> Result<LivePrivacyReleaseResponse, LedgerError> {
        let converged = self.network_converged_privacy_prefix(evidence)?;
        let state = converged.state();
        let profile = converged.profile();
        let object = state.protected_object(request.object_id()).ok_or_else(|| {
            LedgerError::PrivacyReleaseDenied("protected object does not exist".to_string())
        })?;
        let recipient_key = request.recipient_key();
        if recipient_key.purpose() != crate::privacy::ParticipantKeyPurpose::PrivacyKeyWrapping {
            return Err(LedgerError::PrivacyReleaseDenied(
                "release recipient is not a wrapping key".to_string(),
            ));
        }

        let (release_evidence, envelope) = match request.path() {
            LivePrivacyReleasePath::Owner => {
                if request.grant_id().is_some()
                    || request.requester() != object.owner()
                    || recipient_key != object.wrapping_key()
                    || !state.key_is_eligible(recipient_key, PrivacyKeyUse::HistoricalRelease)
                {
                    return Err(LedgerError::PrivacyReleaseDenied(
                        "owner release is not authorized by the current state".to_string(),
                    ));
                }
                let release_evidence = LivePrivacyReleaseEvidence::owner(
                    self.profile.network_id.clone(),
                    self.profile.profile_id.clone(),
                    profile.network_profile_content_hash(),
                    evidence.ledger_position(),
                    evidence.ledger_prefix_hash(),
                    object.object_id(),
                    request.requester(),
                    recipient_key.clone(),
                )
                .map_err(|error| LedgerError::PrivacyReleaseDenied(error.to_string()))?;
                (
                    release_evidence,
                    PrivacyReleaseEnvelope::Owner(object.owner_envelope().clone()),
                )
            }
            LivePrivacyReleasePath::Grantee => {
                let grant_id = request.grant_id().ok_or_else(|| {
                    LedgerError::PrivacyReleaseDenied(
                        "grantee release is missing its grant identifier".to_string(),
                    )
                })?;
                let grant = state
                    .active_privacy_grant(object.object_id(), request.requester())
                    .filter(|grant| grant.grant_id() == grant_id)
                    .ok_or_else(|| {
                        LedgerError::PrivacyReleaseDenied(
                            "grant is not active for this object and requester".to_string(),
                        )
                    })?;
                if grant.permission() != PrivacyPermission::ReadProtectedObjectV1
                    || recipient_key != grant.delivery().wrapping_key()
                    || !state.key_is_eligible(recipient_key, PrivacyKeyUse::HistoricalRelease)
                {
                    return Err(LedgerError::PrivacyReleaseDenied(
                        "grantee release is not authorized by the current state".to_string(),
                    ));
                }
                let release_evidence = LivePrivacyReleaseEvidence::grantee(
                    self.profile.network_id.clone(),
                    self.profile.profile_id.clone(),
                    profile.network_profile_content_hash(),
                    evidence.ledger_position(),
                    evidence.ledger_prefix_hash(),
                    object.object_id(),
                    request.requester(),
                    recipient_key.clone(),
                    grant_id,
                )
                .map_err(|error| LedgerError::PrivacyReleaseDenied(error.to_string()))?;
                (
                    release_evidence,
                    PrivacyReleaseEnvelope::Grant(grant.delivery().clone()),
                )
            }
        };

        LivePrivacyReleaseResponse::new(
            release_evidence,
            object.payload().clone(),
            envelope,
            object.encrypted_payload_commitment(),
        )
        .map_err(|error| LedgerError::PrivacyReleaseDenied(error.to_string()))
    }

    /// Issue one identity-signed receipt for the current durable envelope and
    /// the current Effective Privacy State digest.
    pub fn privacy_state_receipt(
        &self,
        node_id: uuid::Uuid,
        manifest: &SignedMembershipManifest,
        identity_key: &SigningKey,
    ) -> Result<PrivacyStateReceipt, LedgerError> {
        self.profile.privacy_profile().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "privacy-state receipt requires an active privacy lifecycle".to_string(),
            )
        })?;
        let state_digest = self.effective_privacy_state()?.digest();
        let index = self.tip().index.ok_or_else(|| {
            LedgerError::InvalidConvergenceEvidence(
                "cannot issue a privacy-state receipt for virtual genesis".to_string(),
            )
        })?;
        let record = crate::network::convergence::committed_record_at(self, index)
            .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?;
        crate::network::convergence::issue_local_privacy_state_receipt(
            node_id,
            &self.profile.network_id,
            &self.profile.profile_id,
            manifest,
            identity_key,
            &record,
            state_digest,
        )
        .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))
    }

    /// Verify three envelope receipts and three matching signed privacy-state
    /// receipts for the current exact prefix.
    pub fn privacy_network_convergence_evidence(
        &self,
        local_node_id: uuid::Uuid,
        manifest: &SignedMembershipManifest,
        expected_node_ids: &[uuid::Uuid],
        receipts: &[CommitReceipt],
        privacy_receipts: &[PrivacyStateReceipt],
    ) -> Result<NetworkConvergenceEvidence, LedgerError> {
        self.profile.privacy_profile().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "privacy convergence evidence requires an active privacy lifecycle".to_string(),
            )
        })?;
        let index = self.tip().index.ok_or_else(|| {
            LedgerError::InvalidConvergenceEvidence("cannot converge virtual genesis".to_string())
        })?;
        let record = crate::network::convergence::committed_record_at(self, index)
            .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?;
        let status = crate::network::convergence::evaluate_privacy_commit_status(
            local_node_id,
            &self.profile.network_id,
            &self.profile.profile_id,
            manifest,
            expected_node_ids,
            &record,
            receipts,
            privacy_receipts,
        )
        .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?;
        match status {
            CommitStatus::NetworkConverged { evidence }
                if evidence.privacy_state_digest()
                    == Some(self.effective_privacy_state()?.digest()) =>
            {
                Ok(evidence)
            }
            CommitStatus::NetworkConverged { .. } => Err(LedgerError::InvalidConvergenceEvidence(
                "privacy-state receipts do not match the local effective privacy state".to_string(),
            )),
            CommitStatus::LocalCommitted {
                matching_receipts,
                required_receipts,
            } => Err(LedgerError::InvalidConvergenceEvidence(format!(
                "only {matching_receipts} of {required_receipts} receipts match"
            ))),
        }
    }

    /// Submit exact producer envelope bytes through the normal replication
    /// Final Admission path of a non-deployable conformance ledger.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn conformance_admit_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
    ) -> Result<AdmissionOutcome, LedgerError> {
        if self.profile.privacy_conformance_slice.is_none() {
            return Err(LedgerError::InvalidProfile(
                "exact conformance admission requires the non-deployable privacy slice".to_string(),
            ));
        }
        self.replication_adapter()
            .submit_exact_envelope(envelope_bytes)
    }

    /// Issue one identity-signed receipt for the current durable envelope of a
    /// non-deployable conformance ledger.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn conformance_commit_receipt(
        &self,
        node_id: uuid::Uuid,
        manifest: &SignedMembershipManifest,
        identity_key: &SigningKey,
    ) -> Result<CommitReceipt, LedgerError> {
        if self.profile.privacy_conformance_slice.is_none() {
            return Err(LedgerError::InvalidProfile(
                "conformance receipt requires the non-deployable privacy slice".to_string(),
            ));
        }
        let index = self.tip().index.ok_or_else(|| {
            LedgerError::InvalidConvergenceEvidence(
                "cannot issue a receipt for virtual genesis".to_string(),
            )
        })?;
        let record = crate::network::convergence::committed_record_at(self, index)
            .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?;
        crate::network::convergence::issue_local_receipt(
            node_id,
            &self.profile.network_id,
            &self.profile.profile_id,
            manifest,
            identity_key,
            &record,
        )
        .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))
    }

    /// Verify three current conformance-ledger receipts with the existing Issue
    /// #6 evaluator and return its opaque Network-Converged evidence.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn conformance_network_convergence_evidence(
        &self,
        local_node_id: uuid::Uuid,
        manifest: &SignedMembershipManifest,
        expected_node_ids: &[uuid::Uuid],
        receipts: &[CommitReceipt],
    ) -> Result<NetworkConvergenceEvidence, LedgerError> {
        if self.profile.privacy_conformance_slice.is_none() {
            return Err(LedgerError::InvalidProfile(
                "convergence evidence requires the non-deployable privacy slice".to_string(),
            ));
        }
        let index = self.tip().index.ok_or_else(|| {
            LedgerError::InvalidConvergenceEvidence("cannot converge virtual genesis".to_string())
        })?;
        let record = crate::network::convergence::committed_record_at(self, index)
            .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?;
        match crate::network::convergence::evaluate_commit_status(
            local_node_id,
            &self.profile.network_id,
            &self.profile.profile_id,
            manifest,
            expected_node_ids,
            &record,
            receipts,
        )
        .map_err(|error| LedgerError::InvalidConvergenceEvidence(error.to_string()))?
        {
            CommitStatus::NetworkConverged { evidence } => Ok(evidence),
            CommitStatus::LocalCommitted {
                matching_receipts,
                required_receipts,
            } => Err(LedgerError::InvalidConvergenceEvidence(format!(
                "only {matching_receipts} of {required_receipts} receipts match"
            ))),
        }
    }

    /// Return the number of complete verified journal frames.
    pub fn journal_frame_count(&self) -> Result<usize, LedgerError> {
        Ok(self.journal.verified_replay()?.envelopes.len())
    }

    /// Return raw journal bytes for conformance evidence.
    pub fn journal_bytes(&self) -> Result<Vec<u8>, LedgerError> {
        self.journal.bytes()
    }

    /// Submit one source declaration candidate through Final Admission while
    /// exercising deterministic commit-boundary failures in conformance tests.
    #[cfg(feature = "bridge-conformance")]
    #[doc(hidden)]
    pub fn conformance_admit_bridge_candidate(
        &mut self,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, LedgerError> {
        if self.profile.bridge_source_profile.is_none()
            || candidate.bridge_export_declaration.is_none()
        {
            return Err(LedgerError::InvalidProfile(
                "bridge conformance admission requires an active source declaration profile"
                    .to_string(),
            ));
        }
        self.proposal_adapter().submit(candidate)
    }

    /// Return the literal envelope payload of every complete verified journal
    /// frame. Replication derives wire records from this authority rather than
    /// reserializing the in-memory projection.
    pub(crate) fn verified_committed_envelope_bytes(&self) -> Result<Vec<Vec<u8>>, LedgerError> {
        if self.degraded || self.journal.requires_replay() {
            return Err(LedgerError::Degraded);
        }
        let replay = self.journal.verified_frame_replay()?;
        if replay.has_torn_tail || replay.frames.len() != self.projections.chain.len() {
            return Err(LedgerError::Degraded);
        }
        if replay
            .frames
            .iter()
            .zip(&self.projections.chain)
            .any(|(frame, projected)| &frame.envelope != projected)
        {
            return Err(LedgerError::Degraded);
        }
        Ok(replay
            .frames
            .into_iter()
            .map(|frame| frame.envelope_bytes)
            .collect())
    }

    fn current_ledger_prefix_hash(&self) -> Result<LedgerHash, LedgerError> {
        let mut prefix = crate::network::convergence::genesis_ledger_prefix_hash();
        for (index, envelope_bytes) in self
            .verified_committed_envelope_bytes()?
            .into_iter()
            .enumerate()
        {
            let index = u64::try_from(index).map_err(|_| {
                LedgerError::ProtocolBoundExceeded("ledger prefix length exceeds u64".to_string())
            })?;
            prefix = crate::network::convergence::extend_ledger_prefix_hash(
                prefix,
                index,
                &envelope_bytes,
            );
        }
        Ok(prefix)
    }

    /// Return the current journal-derived tip, using a virtual parent at genesis.
    pub fn tip(&self) -> LedgerTip {
        match self.projections.chain.last() {
            Some(envelope) => LedgerTip {
                index: Some(envelope.index),
                envelope_hash: envelope.envelope_hash,
                state_commitment: envelope.post_state_commitment,
            },
            None => LedgerTip {
                index: None,
                envelope_hash: [0; 32],
                state_commitment: genesis_state_commitment(),
            },
        }
    }

    /// Build and sign the next ordinary candidate without mutating the ledger.
    pub fn create_ordinary_candidate(
        &self,
        public_provenance: impl AsRef<[u8]>,
        timestamp_millis: u64,
        signing_key: &SigningKey,
    ) -> Result<AdmissionCandidate, LedgerError> {
        self.create_unsigned_ordinary_candidate(
            public_provenance,
            timestamp_millis,
            signing_key.verifying_key().to_bytes(),
        )?
        .sign(signing_key)
    }

    /// Build and sign the next source export declaration without mutating the ledger.
    pub fn create_bridge_export_candidate(
        &self,
        public_provenance: impl AsRef<[u8]>,
        target: BridgeExportTargetV1,
        timestamp_millis: u64,
        signing_key: &SigningKey,
    ) -> Result<AdmissionCandidate, LedgerError> {
        self.create_unsigned_bridge_export_candidate(
            public_provenance,
            target,
            timestamp_millis,
            signing_key.verifying_key().to_bytes(),
        )?
        .sign(signing_key)
    }

    /// Derive the exact journal-authoritative parent anchor for the next privacy transition.
    pub fn next_privacy_admission_anchor(&self) -> Result<PrivacyAdmissionAnchor, LedgerError> {
        let privacy_profile = self.profile.privacy_profile().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "privacy lifecycle is not active for this ledger profile".to_string(),
            )
        })?;
        let tip = self.tip();
        PrivacyAdmissionAnchor::new(
            self.profile.network_id.clone(),
            self.profile.profile_id.clone(),
            privacy_profile.network_profile_content_hash(),
            tip.index.map_or(0, |index| index + 1),
            self.projections
                .privacy_state
                .revision()
                .checked_add(1)
                .ok_or_else(|| {
                    LedgerError::ProtocolBoundExceeded("privacy revision overflow".to_string())
                })?,
            self.current_ledger_prefix_hash()?,
            tip.index.map(|_| tip.envelope_hash),
        )
        .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))
    }

    /// Build and sign the next canonical privacy-control candidate without mutating the ledger.
    pub fn create_privacy_candidate(
        &self,
        transition: &PrivacyControlTransition,
        timestamp_millis: u64,
        signing_key: &SigningKey,
    ) -> Result<AdmissionCandidate, LedgerError> {
        self.create_unsigned_privacy_candidate(
            transition,
            timestamp_millis,
            signing_key.verifying_key().to_bytes(),
        )?
        .sign(signing_key)
    }

    /// Deterministically preflight and construct an unsigned privacy candidate for PoA.
    pub(crate) fn create_unsigned_privacy_candidate(
        &self,
        transition: &PrivacyControlTransition,
        timestamp_millis: u64,
        proposer_public_key: [u8; 32],
    ) -> Result<AdmissionCandidate, LedgerError> {
        let privacy_profile = self.profile.privacy_profile().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "privacy lifecycle is not active for this ledger profile".to_string(),
            )
        })?;
        let anchor = self.next_privacy_admission_anchor()?;
        self.projections
            .privacy_state
            .stage_transition(privacy_profile, &anchor, transition)
            .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))?;
        let tip = self.tip();
        let mut candidate = AdmissionCandidate::new_privacy_control(
            &self.profile,
            tip.index.map_or(0, |index| index + 1),
            tip.envelope_hash,
            tip.state_commitment,
            timestamp_millis,
            transition,
        )?;
        candidate.proposer_public_key = proposer_public_key;
        candidate.proposal_digest = candidate.compute_proposal_digest()?;
        Ok(candidate)
    }

    /// Preflight and build the exact next unsigned body without mutating state.
    pub(crate) fn create_unsigned_ordinary_candidate(
        &self,
        public_provenance: impl AsRef<[u8]>,
        timestamp_millis: u64,
        proposer_public_key: [u8; 32],
    ) -> Result<AdmissionCandidate, LedgerError> {
        let tip = self.tip();
        let mut candidate = AdmissionCandidate::new_ordinary(
            &self.profile,
            tip.index.map_or(0, |index| index + 1),
            tip.envelope_hash,
            tip.state_commitment,
            timestamp_millis,
            public_provenance,
        )?;
        candidate.post_state_commitment = self.projections.staged_post_state_commitment(
            candidate.index,
            &candidate.public_provenance,
            self.profile.state_commitment_bounds(),
        )?;
        candidate.proposer_public_key = proposer_public_key;
        candidate.proposal_digest = candidate.compute_proposal_digest()?;
        Ok(candidate)
    }

    /// Preflight and build one target-bound source export declaration.
    pub(crate) fn create_unsigned_bridge_export_candidate(
        &self,
        public_provenance: impl AsRef<[u8]>,
        target: BridgeExportTargetV1,
        timestamp_millis: u64,
        proposer_public_key: [u8; 32],
    ) -> Result<AdmissionCandidate, LedgerError> {
        let public_provenance = public_provenance.as_ref();
        let source = self.profile.bridge_source_profile.as_ref().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "source bridge declaration is unavailable without an activated source profile"
                    .to_string(),
            )
        })?;
        let tip = self.tip();
        let index = tip.index.map_or(0, |index| index + 1);
        let core = BridgeExportCoreV1::new(source, index, target, public_provenance)
            .map_err(bridge_candidate_error)?;
        let declaration = BridgeExportDeclarationV1::new(core)
            .map_err(bridge_candidate_error)?
            .canonical_bytes()
            .map_err(bridge_candidate_error)?;
        let mut candidate = AdmissionCandidate::new_ordinary(
            &self.profile,
            index,
            tip.envelope_hash,
            tip.state_commitment,
            timestamp_millis,
            public_provenance,
        )?;
        candidate.bridge_export_declaration = Some(declaration);
        candidate.post_state_commitment = self.projections.staged_post_state_commitment(
            candidate.index,
            &candidate.public_provenance,
            self.profile.state_commitment_bounds(),
        )?;
        candidate.proposer_public_key = proposer_public_key;
        candidate.proposal_digest = candidate.compute_proposal_digest()?;
        Ok(candidate)
    }

    /// Rebuild authoritative bridge state and classify a raw import before consensus.
    pub(crate) fn preflight_bridge_import(
        &mut self,
        proof_bytes: &[u8],
        public_provenance: &[u8],
    ) -> Result<BridgeImportPreflightV1, LedgerError> {
        if self.degraded || self.journal.requires_replay() {
            return Err(LedgerError::Degraded);
        }
        let target = self.profile.bridge_target_profile.as_ref().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "target bridge import is unavailable without an activated verifier".to_string(),
            )
        })?;
        let replay = self.journal.verified_replay()?;
        let authoritative = LedgerProjections::rebuild(&replay.envelopes, &self.profile)?;
        if authoritative.bridge_state != self.projections.bridge_state {
            return Err(LedgerError::ProjectionFailed {
                envelope_hash: "bridge-state".to_string(),
                reason: "effective bridge projection diverges from verified journal replay"
                    .to_string(),
            });
        }
        let classification = authoritative
            .bridge_state
            .classify(target, proof_bytes, public_provenance)
            .map_err(bridge_candidate_error)?;
        if matches!(classification, BridgeImportPreflightV1::Ready(_)) {
            let parent_quads = asserted_public_state_quads(&replay.envelopes)
                .map_err(|error| LedgerError::SemanticIncapacity(error.to_string()))?;
            let candidate_quads =
                projected_public_quads(replay.envelopes.len() as u64, public_provenance)
                    .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))?;
            match self
                .semantic_package
                .validate_transition(&parent_quads, &candidate_quads)
            {
                Ok(SemanticAdmissionVerdict::Conforms) => {}
                Ok(SemanticAdmissionVerdict::Rejected { reason }) => {
                    return Err(LedgerError::MalformedCandidate(reason));
                }
                Err(error) => {
                    self.degraded = true;
                    return Err(LedgerError::SemanticIncapacity(error.to_string()));
                }
            }
        }
        Ok(classification)
    }

    /// Build the exact unsigned target import body; PoA still owns signing and submission.
    pub(crate) fn create_unsigned_bridge_import_candidate(
        &self,
        public_provenance: impl AsRef<[u8]>,
        bridge_origin_evidence: impl AsRef<[u8]>,
        timestamp_millis: u64,
        proposer_public_key: [u8; 32],
    ) -> Result<AdmissionCandidate, LedgerError> {
        let public_provenance = public_provenance.as_ref();
        let origin = bridge_origin_evidence.as_ref();
        let target = self.profile.bridge_target_profile.as_ref().ok_or_else(|| {
            LedgerError::InvalidProfile(
                "target bridge import is unavailable without an activated verifier".to_string(),
            )
        })?;
        let verified = target
            .verify_origin(origin, public_provenance)
            .map_err(bridge_candidate_error)?;
        if self
            .projections
            .bridge_state
            .get(&verified.transfer_id)
            .is_some()
        {
            return Err(LedgerError::MalformedCandidate(
                "bridge transfer ID is already present in Effective Bridge State".to_string(),
            ));
        }
        let tip = self.tip();
        let mut candidate = AdmissionCandidate::new_ordinary(
            &self.profile,
            tip.index.map_or(0, |index| index + 1),
            tip.envelope_hash,
            tip.state_commitment,
            timestamp_millis,
            public_provenance,
        )?;
        candidate.bridge_origin_evidence = Some(origin.to_vec());
        candidate.post_state_commitment = self.projections.staged_post_state_commitment(
            candidate.index,
            &candidate.public_provenance,
            self.profile.state_commitment_bounds(),
        )?;
        candidate.proposer_public_key = proposer_public_key;
        candidate.proposal_digest = candidate.compute_proposal_digest()?;
        Ok(candidate)
    }

    /// Check the exact encoded envelope size without requiring a pre-fence signature.
    pub(crate) fn preflight_candidate_envelope_size(
        &self,
        candidate: &AdmissionCandidate,
    ) -> Result<(), LedgerError> {
        // Signature and envelope-hash fields are fixed width, so placeholder
        // values produce the exact final encoded length without invoking a signer.
        let envelope_bytes = Self::envelope_from_candidate(candidate).encode()?;
        if envelope_bytes.len() > self.profile.max_envelope_bytes {
            return Err(LedgerError::OversizedEnvelope(envelope_bytes.len()));
        }
        Ok(())
    }

    /// Execute the invariant admission path shared by every adapter.
    fn final_admit(
        &mut self,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, LedgerError> {
        if self.degraded {
            return Err(LedgerError::Degraded);
        }

        let envelope = match self.materialize_candidate(&candidate) {
            Ok(envelope) => envelope,
            Err(reason) => return Ok(AdmissionOutcome::Rejected { reason }),
        };
        self.final_admit_materialized(envelope)
    }

    /// Decode and commit the exact canonical envelope supplied by replication.
    fn final_admit_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
    ) -> Result<AdmissionOutcome, LedgerError> {
        if self.degraded {
            return Err(LedgerError::Degraded);
        }
        let envelope = AdmittedBlockEnvelope::decode(envelope_bytes)?;
        if envelope.canonical_bytes()? != envelope_bytes {
            return Err(LedgerError::NonCanonicalEnvelope);
        }
        self.final_admit_materialized(envelope)
    }

    /// The sole implementation containing the authoritative append-plus-fsync.
    fn final_admit_materialized(
        &mut self,
        envelope: AdmittedBlockEnvelope,
    ) -> Result<AdmissionOutcome, LedgerError> {
        let envelope_bytes = envelope.canonical_bytes()?;
        if envelope_bytes.len() > self.profile.max_envelope_bytes {
            return Ok(AdmissionOutcome::Rejected {
                reason: format!(
                    "envelope size {} exceeds profile limit {}",
                    envelope_bytes.len(),
                    self.profile.max_envelope_bytes
                ),
            });
        }

        // Resolve exact lost-response retries before checking the now-advanced parent.
        // The same verified envelope history is the sole input for reconstructing
        // semantic parent state; an Oxigraph projection never determines verdicts.
        let authoritative_replay = self.journal.verified_replay()?;
        if let Some(existing) = authoritative_replay
            .envelopes
            .iter()
            .find(|existing| existing.envelope_hash == envelope.envelope_hash)
            .cloned()
        {
            if existing != envelope {
                return Err(LedgerError::TamperedEnvelope(
                    "two different envelopes claim the same identity".to_string(),
                ));
            }
            if !self
                .projections
                .envelope_index
                .contains_key(&envelope.envelope_hash)
            {
                self.replay()?;
            }
            return Ok(AdmissionOutcome::Committed {
                envelope: Box::new(existing),
            });
        }

        if let Ok(index) = usize::try_from(envelope.index) {
            if let Some(existing) = self.projections.chain.get(index) {
                return Ok(AdmissionOutcome::Rejected {
                    reason: format!(
                        "divergent committed envelope at index {}: local {}, received {}",
                        envelope.index,
                        hex::encode(existing.envelope_hash),
                        hex::encode(envelope.envelope_hash)
                    ),
                });
            }
        }

        if let Err(reason) = self.validate_envelope_position(&envelope) {
            return Ok(AdmissionOutcome::Rejected { reason });
        }

        let staged_privacy_state = match envelope.admission_kind {
            AdmissionKind::OrdinaryProvenanceV1 => {
                let parent_quads =
                    match asserted_public_state_quads(&authoritative_replay.envelopes) {
                        Ok(quads) => quads,
                        Err(error) => {
                            self.degraded = true;
                            return Err(LedgerError::SemanticIncapacity(error.to_string()));
                        }
                    };
                let candidate_quads =
                    match projected_public_quads(envelope.index, &envelope.public_provenance) {
                        Ok(quads) => quads,
                        Err(error) => {
                            return Ok(AdmissionOutcome::Rejected {
                                reason: error.to_string(),
                            });
                        }
                    };
                match self
                    .semantic_package
                    .validate_transition(&parent_quads, &candidate_quads)
                {
                    Ok(SemanticAdmissionVerdict::Conforms) => {}
                    Ok(SemanticAdmissionVerdict::Rejected { reason }) => {
                        return Ok(AdmissionOutcome::Rejected { reason });
                    }
                    Err(error) => {
                        self.degraded = true;
                        return Err(LedgerError::SemanticIncapacity(error.to_string()));
                    }
                }
                None
            }
            AdmissionKind::PrivacyControlV1 => {
                let (authoritative_privacy_state, parent_prefix_hash) =
                    match rebuild_privacy_projection(&authoritative_replay.envelopes, &self.profile)
                    {
                        Ok(rebuilt) => rebuilt,
                        Err(error) => {
                            self.degraded = true;
                            return Err(error);
                        }
                    };
                if authoritative_privacy_state != self.projections.privacy_state {
                    self.degraded = true;
                    return Err(LedgerError::PrivacyIncapacity(
                        "effective privacy projection diverges from verified journal replay"
                            .to_string(),
                    ));
                }
                match stage_privacy_envelope(
                    &self.profile,
                    &authoritative_privacy_state,
                    parent_prefix_hash,
                    &envelope,
                ) {
                    Ok(staged) => Some(staged),
                    Err(error) => {
                        return Ok(AdmissionOutcome::Rejected {
                            reason: error.to_string(),
                        });
                    }
                }
            }
        };

        let staged_bridge_state = if let Some(origin) = &envelope.bridge_origin_evidence {
            let target = self.profile.bridge_target_profile.as_ref().ok_or_else(|| {
                LedgerError::InvalidProfile(
                    "bridge import exists without an active target verifier".to_string(),
                )
            })?;
            let authoritative =
                match LedgerProjections::rebuild(&authoritative_replay.envelopes, &self.profile) {
                    Ok(projections) => projections,
                    Err(error) => {
                        self.degraded = true;
                        return Err(error);
                    }
                };
            if authoritative.bridge_state != self.projections.bridge_state {
                self.degraded = true;
                return Err(LedgerError::ProjectionFailed {
                    envelope_hash: "bridge-state".to_string(),
                    reason: "effective bridge projection diverges from verified journal replay"
                        .to_string(),
                });
            }
            let verified = match target.verify_origin(origin, &envelope.public_provenance) {
                Ok(verified) => verified,
                Err(error) => {
                    return Ok(AdmissionOutcome::Rejected {
                        reason: error.to_string(),
                    });
                }
            };
            let parent_ledger_prefix_hash =
                match ledger_prefix_hash_for_envelopes(&authoritative_replay.envelopes) {
                    Ok(prefix) => prefix,
                    Err(error) => {
                        self.degraded = true;
                        return Err(error);
                    }
                };
            let target_ledger_prefix_hash = crate::network::convergence::extend_ledger_prefix_hash(
                parent_ledger_prefix_hash,
                envelope.index,
                &envelope_bytes,
            );
            let mut state = authoritative.bridge_state;
            if let Err(error) = state.insert_verified(
                verified,
                envelope.index,
                envelope.envelope_hash,
                target_ledger_prefix_hash,
            ) {
                return Ok(AdmissionOutcome::Rejected {
                    reason: error.to_string(),
                });
            }
            Some(state)
        } else {
            None
        };

        // This is the only call that can make a block authoritative.
        if let Err(error) = self.journal.append(&envelope) {
            if self.journal.requires_replay() {
                self.degraded = true;
            }
            return Err(error);
        }

        if self.fail_next_projection {
            self.fail_next_projection = false;
            self.degraded = true;
            return Err(LedgerError::ProjectionFailed {
                envelope_hash: hex::encode(envelope.envelope_hash),
                reason: "injected projection failure".to_string(),
            });
        }
        if let Err(error) =
            self.projections
                .apply(&envelope, staged_privacy_state, staged_bridge_state)
        {
            self.degraded = true;
            return Err(LedgerError::ProjectionFailed {
                envelope_hash: hex::encode(envelope.envelope_hash),
                reason: error.to_string(),
            });
        }

        if self.fail_next_response {
            self.fail_next_response = false;
            return Err(LedgerError::ResponseLost {
                envelope_hash: hex::encode(envelope.envelope_hash),
            });
        }

        Ok(AdmissionOutcome::Committed {
            envelope: Box::new(envelope),
        })
    }

    fn final_admission(&mut self) -> FinalAdmission<'_> {
        FinalAdmission { ledger: self }
    }

    fn rebuild_verified_state(
        &mut self,
        replay: &VerifiedJournalReplay,
    ) -> Result<(), LedgerError> {
        let projections = LedgerProjections::rebuild(&replay.envelopes, &self.profile)?;
        let parent_quads = match asserted_public_state_quads(&replay.envelopes) {
            Ok(quads) => quads,
            Err(error) => {
                self.degraded = true;
                return Err(LedgerError::SemanticIncapacity(error.to_string()));
            }
        };
        if let Err(error) = self
            .semantic_package
            .validate_committed_state(&parent_quads)
        {
            self.degraded = true;
            return Err(LedgerError::SemanticIncapacity(error.to_string()));
        }
        self.projections = projections;
        self.journal.clear_replay_requirement();
        self.degraded = false;
        Ok(())
    }

    /// Rebuild every projection from verified journal history without readmission.
    pub fn replay(&mut self) -> Result<VerifiedJournalReplay, LedgerError> {
        let replay = self.journal.verified_replay()?;
        if replay.has_torn_tail {
            self.degraded = true;
            return Err(LedgerError::CorruptJournal(
                "incomplete journal tail requires explicit repair".to_string(),
            ));
        }
        self.rebuild_verified_state(&replay)?;
        Ok(replay)
    }

    /// Explicitly discard an incomplete final journal frame and rebuild views.
    ///
    /// A torn tail is never removed implicitly during open or replay because
    /// the bytes may represent a damaged durable frame. Operators must choose
    /// this recovery action after inspecting the verified replay boundary.
    pub fn repair_torn_tail(&mut self) -> Result<VerifiedJournalReplay, LedgerError> {
        let replay = self.journal.verified_replay()?;
        if !replay.has_torn_tail {
            return Err(LedgerError::CorruptJournal(
                "journal has no incomplete tail to repair".to_string(),
            ));
        }
        self.journal.truncate_to(replay.valid_bytes)?;
        let repaired = self.journal.verified_replay()?;
        self.rebuild_verified_state(&repaired)?;
        Ok(repaired)
    }

    /// Whether a post-commit projection failure currently blocks new writes.
    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    /// Inject one deterministic ledger commit-boundary failure.
    #[cfg(any(feature = "privacy-conformance", feature = "bridge-conformance"))]
    #[doc(hidden)]
    pub fn inject_next_commit_failpoint(&mut self, failpoint: LedgerCommitFailpoint) {
        match failpoint {
            LedgerCommitFailpoint::BeforeJournalWrite => self.journal.fail_next_write = true,
            LedgerCommitFailpoint::BeforeJournalFsync => self.journal.fail_next_fsync = true,
            LedgerCommitFailpoint::AfterJournalCommitBeforeProjection => {
                self.fail_next_projection = true;
            }
            LedgerCommitFailpoint::AfterProjectionBeforeResponse => {
                self.fail_next_response = true;
            }
        }
    }

    /// Inject a journal write failure for the next candidate.
    #[cfg(test)]
    fn inject_next_journal_write_failure(&mut self) {
        self.journal.inject_next_write_failure();
    }

    /// Inject a journal synchronization failure for the next candidate.
    #[cfg(test)]
    fn inject_next_journal_fsync_failure(&mut self) {
        self.journal.inject_next_fsync_failure();
    }

    /// Inject a projection failure after the next durable journal commit.
    #[cfg(test)]
    fn inject_next_projection_failure(&mut self) {
        self.fail_next_projection = true;
    }

    /// Inject a lost response after the next durable commit and projection update.
    #[cfg(test)]
    fn inject_next_response_failure(&mut self) {
        self.fail_next_response = true;
    }

    fn materialize_candidate(
        &self,
        candidate: &AdmissionCandidate,
    ) -> Result<AdmittedBlockEnvelope, String> {
        candidate
            .verify_signed_proposal()
            .map_err(|error| error.to_string())?;
        let mut envelope = Self::envelope_from_candidate(candidate);
        envelope.envelope_hash = envelope
            .compute_envelope_hash()
            .map_err(|error| error.to_string())?;
        envelope
            .validate_shape()
            .map_err(|error| error.to_string())?;
        Ok(envelope)
    }

    fn envelope_from_candidate(candidate: &AdmissionCandidate) -> AdmittedBlockEnvelope {
        AdmittedBlockEnvelope {
            version: codec_version(
                candidate.admission_kind,
                candidate.bridge_export_declaration.is_some(),
                candidate.bridge_origin_evidence.is_some(),
            ),
            admission_kind: candidate.admission_kind,
            network_id: candidate.network_id.clone(),
            profile_id: candidate.profile_id.clone(),
            ontology_package_id: candidate.ontology_package_id.clone(),
            ontology_package_version: candidate.ontology_package_version.clone(),
            ontology_package_hash: candidate.ontology_package_hash.clone(),
            semantic_execution_profile_id: candidate.semantic_execution_profile_id.clone(),
            state_commitment_scheme: candidate.state_commitment_scheme,
            index: candidate.index,
            previous_envelope_hash: candidate.previous_envelope_hash,
            previous_state_commitment: candidate.previous_state_commitment,
            timestamp_millis: candidate.timestamp_millis,
            public_provenance: candidate.public_provenance.clone(),
            encrypted_payload: candidate.encrypted_payload.clone(),
            privacy_control: candidate.privacy_control.clone(),
            bridge_export_declaration: candidate.bridge_export_declaration.clone(),
            bridge_origin_evidence: candidate.bridge_origin_evidence.clone(),
            post_state_commitment: candidate.post_state_commitment,
            proposer_public_key: candidate.proposer_public_key,
            proposal_digest: candidate.proposal_digest,
            proposer_signature: candidate.proposer_signature,
            envelope_hash: [0; 32],
        }
    }

    fn validate_envelope_position(&self, envelope: &AdmittedBlockEnvelope) -> Result<(), String> {
        validate_envelope_against_profile(envelope, &self.profile)
            .map_err(|error| error.to_string())?;
        let tip = self.tip();
        let expected_index = tip.index.map_or(0, |index| index + 1);
        if envelope.index != expected_index {
            return Err(format!(
                "wrong parent index: expected {expected_index}, got {}",
                envelope.index
            ));
        }
        if envelope.previous_envelope_hash != tip.envelope_hash {
            return Err("wrong parent envelope hash".to_string());
        }
        if envelope.previous_state_commitment != tip.state_commitment {
            return Err("wrong parent state commitment".to_string());
        }
        let expected_post_state = match envelope.admission_kind {
            AdmissionKind::OrdinaryProvenanceV1 => self
                .projections
                .staged_post_state_commitment(
                    envelope.index,
                    &envelope.public_provenance,
                    self.profile.state_commitment_bounds(),
                )
                .map_err(|error| error.to_string())?,
            AdmissionKind::PrivacyControlV1 => tip.state_commitment,
        };
        if envelope.post_state_commitment != expected_post_state {
            return Err(
                "post-state commitment is not canonical for staged public state".to_string(),
            );
        }
        Ok(())
    }
}

/// The current position used to build the next candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerTip {
    /// `None` before the first ordinary envelope.
    pub index: Option<u64>,
    /// Hash of the current tip, or zero at virtual genesis.
    pub envelope_hash: LedgerHash,
    /// State commitment of the current tip, or the empty-state commitment at virtual genesis.
    pub state_commitment: LedgerHash,
}

fn validate_envelope_against_profile(
    envelope: &AdmittedBlockEnvelope,
    profile: &LedgerProfile,
) -> Result<(), LedgerError> {
    envelope.validate_shape()?;
    if envelope.network_id != profile.network_id {
        return Err(LedgerError::MalformedEnvelope(
            "network_id does not match ledger profile".to_string(),
        ));
    }
    if envelope.profile_id != profile.profile_id {
        return Err(LedgerError::MalformedEnvelope(
            "profile_id does not match ledger profile".to_string(),
        ));
    }
    if envelope.state_commitment_scheme != profile.state_commitment_scheme {
        return Err(LedgerError::MalformedEnvelope(
            "state commitment scheme does not match ledger profile".to_string(),
        ));
    }
    if envelope.ontology_package_id != profile.ontology_package_id
        || envelope.ontology_package_version != profile.ontology_package_version
        || envelope.ontology_package_hash != profile.ontology_package_hash
        || envelope.semantic_execution_profile_id != profile.semantic_execution_profile_id
    {
        return Err(LedgerError::MalformedEnvelope(
            "semantic package identity does not match ledger profile".to_string(),
        ));
    }
    if envelope.admission_kind == AdmissionKind::PrivacyControlV1
        && profile.privacy_profile().is_none()
    {
        return Err(LedgerError::MalformedEnvelope(
            "PrivacyControlV1 is inactive for this ledger profile".to_string(),
        ));
    }
    if let Some(declaration_bytes) = envelope.bridge_export_declaration.as_deref() {
        let source = profile.bridge_source_profile.as_ref().ok_or_else(|| {
            LedgerError::MalformedEnvelope(
                "bridge export declaration is not enabled by the ledger profile".to_string(),
            )
        })?;
        let declaration = BridgeExportDeclarationV1::decode(declaration_bytes)
            .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
        declaration
            .validate_source(source, envelope.index, &envelope.public_provenance)
            .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
    }
    if let Some(origin) = envelope.bridge_origin_evidence.as_deref() {
        let target = profile.bridge_target_profile.as_ref().ok_or_else(|| {
            LedgerError::MalformedEnvelope(
                "bridge origin evidence is not enabled by the ledger profile".to_string(),
            )
        })?;
        target
            .verify_origin(origin, &envelope.public_provenance)
            .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
    }
    let bytes = envelope.canonical_bytes()?;
    if bytes.len() > profile.max_envelope_bytes {
        return Err(LedgerError::OversizedEnvelope(bytes.len()));
    }
    Ok(())
}

fn stage_privacy_envelope(
    profile: &LedgerProfile,
    parent_state: &EffectivePrivacyState,
    parent_prefix_hash: LedgerHash,
    envelope: &AdmittedBlockEnvelope,
) -> Result<EffectivePrivacyState, PrivacyError> {
    let privacy_profile = profile.privacy_profile().ok_or_else(|| {
        PrivacyError::InvalidProfile(
            "privacy lifecycle is not active for this ledger profile".to_string(),
        )
    })?;
    let transition_bytes = envelope.privacy_control.as_deref().ok_or_else(|| {
        PrivacyError::Malformed("privacy envelope has no control transition".to_string())
    })?;
    let transition = PrivacyControlTransition::decode(transition_bytes)?;
    let expected_anchor = PrivacyAdmissionAnchor::new(
        profile.network_id.clone(),
        profile.profile_id.clone(),
        privacy_profile.network_profile_content_hash(),
        envelope.index,
        parent_state
            .revision()
            .checked_add(1)
            .ok_or_else(|| PrivacyError::Rejected("privacy revision overflow".to_string()))?,
        parent_prefix_hash,
        (envelope.index != 0).then_some(envelope.previous_envelope_hash),
    )?;
    parent_state.stage_transition(privacy_profile, &expected_anchor, &transition)
}

fn rebuild_privacy_projection(
    envelopes: &[AdmittedBlockEnvelope],
    profile: &LedgerProfile,
) -> Result<(EffectivePrivacyState, LedgerHash), LedgerError> {
    let mut state = EffectivePrivacyState::new();
    let mut prefix_hash = crate::network::convergence::genesis_ledger_prefix_hash();
    for envelope in envelopes {
        if envelope.admission_kind == AdmissionKind::PrivacyControlV1 {
            state = stage_privacy_envelope(profile, &state, prefix_hash, envelope)
                .map_err(|error| LedgerError::PrivacyIncapacity(error.to_string()))?;
        }
        prefix_hash = crate::network::convergence::extend_ledger_prefix_hash(
            prefix_hash,
            envelope.index,
            &envelope.canonical_bytes()?,
        );
    }
    Ok((state, prefix_hash))
}

fn ledger_prefix_hash_for_envelopes(
    envelopes: &[AdmittedBlockEnvelope],
) -> Result<LedgerHash, LedgerError> {
    let mut prefix_hash = crate::network::convergence::genesis_ledger_prefix_hash();
    for envelope in envelopes {
        prefix_hash = crate::network::convergence::extend_ledger_prefix_hash(
            prefix_hash,
            envelope.index,
            &envelope.canonical_bytes()?,
        );
    }
    Ok(prefix_hash)
}

fn validate_activated_semantic_package(
    profile: &LedgerProfile,
    semantic_package: &ActivatedSemanticPackage,
) -> Result<(), LedgerError> {
    if semantic_package.package_id() != profile.ontology_package_id
        || semantic_package.package_version() != profile.ontology_package_version
        || semantic_package.package_hash() != profile.ontology_package_hash
        || semantic_package.semantic_execution_profile_id() != profile.semantic_execution_profile_id
    {
        return Err(LedgerError::InvalidProfile(
            "activated semantic package does not match ledger profile binding".to_string(),
        ));
    }
    Ok(())
}

struct CommonAdmissionFields<'a> {
    admission_kind: AdmissionKind,
    state_commitment_scheme: StateCommitmentScheme,
    network_id: &'a str,
    profile_id: &'a str,
    ontology_package_id: &'a str,
    ontology_package_version: &'a str,
    ontology_package_hash: &'a str,
    semantic_execution_profile_id: &'a str,
    timestamp_millis: u64,
    public_provenance: &'a [u8],
    encrypted_payload: Option<&'a [u8]>,
    privacy_control: Option<&'a [u8]>,
    bridge_export_declaration: Option<&'a [u8]>,
    bridge_origin_evidence: Option<&'a [u8]>,
}

fn validate_common_fields(fields: &CommonAdmissionFields<'_>) -> Result<(), LedgerError> {
    if fields.state_commitment_scheme != StateCommitmentScheme::Rdfc10Sha256NQuadsV1 {
        return Err(LedgerError::MalformedEnvelope(
            "unsupported state commitment scheme".to_string(),
        ));
    }
    validate_text(fields.network_id, "network_id", MAX_TEXT_BYTES)?;
    validate_text(fields.profile_id, "profile_id", MAX_TEXT_BYTES)?;
    validate_text(
        fields.ontology_package_id,
        "ontology_package_id",
        MAX_TEXT_BYTES,
    )?;
    validate_text(
        fields.ontology_package_version,
        "ontology_package_version",
        MAX_TEXT_BYTES,
    )?;
    validate_text(
        fields.ontology_package_hash,
        "ontology_package_hash",
        MAX_TEXT_BYTES,
    )?;
    validate_text(
        fields.semantic_execution_profile_id,
        "semantic_execution_profile_id",
        MAX_TEXT_BYTES,
    )?;
    if fields.semantic_execution_profile_id != SEMANTIC_EXECUTION_PROFILE_V1 {
        return Err(LedgerError::MalformedEnvelope(
            "unsupported semantic execution profile".to_string(),
        ));
    }
    if fields.timestamp_millis == 0 {
        return Err(LedgerError::MalformedEnvelope(
            "timestamp_millis must be nonzero".to_string(),
        ));
    }
    match fields.admission_kind {
        AdmissionKind::OrdinaryProvenanceV1 => {
            validate_public_provenance(fields.public_provenance)?;
            if fields.encrypted_payload.is_some() || fields.privacy_control.is_some() {
                return Err(LedgerError::MalformedEnvelope(
                    "OrdinaryProvenanceV1 forbids encrypted_payload and privacy_control"
                        .to_string(),
                ));
            }
            if let Some(declaration) = fields.bridge_export_declaration {
                BridgeExportDeclarationV1::decode(declaration)
                    .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
            }
            if fields.bridge_export_declaration.is_some() && fields.bridge_origin_evidence.is_some()
            {
                return Err(LedgerError::MalformedEnvelope(
                    "ordinary admission cannot be both a bridge export and import".to_string(),
                ));
            }
            if let Some(origin) = fields.bridge_origin_evidence {
                BridgeOriginEvidenceV1::decode(origin)
                    .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
            }
        }
        AdmissionKind::PrivacyControlV1 => {
            if !fields.public_provenance.is_empty()
                || fields.encrypted_payload.is_some()
                || fields.bridge_export_declaration.is_some()
                || fields.bridge_origin_evidence.is_some()
            {
                return Err(LedgerError::MalformedEnvelope(
                    "PrivacyControlV1 is control-only and forbids RDF, encrypted payload, or bridge export bytes".to_string(),
                ));
            }
            let privacy_control = fields.privacy_control.ok_or_else(|| {
                LedgerError::MalformedEnvelope(
                    "PrivacyControlV1 requires exactly one canonical transition".to_string(),
                )
            })?;
            if privacy_control.len() > MAX_PRIVACY_TRANSITION_BYTES {
                return Err(LedgerError::MalformedEnvelope(format!(
                    "privacy transition exceeds {MAX_PRIVACY_TRANSITION_BYTES} bytes"
                )));
            }
            PrivacyControlTransition::decode(privacy_control)
                .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
        }
    }
    Ok(())
}

fn validate_public_provenance(public_provenance: &[u8]) -> Result<(), LedgerError> {
    if public_provenance.is_empty() || public_provenance.len() > MAX_PAYLOAD_BYTES {
        return Err(LedgerError::MalformedCandidate(format!(
            "public provenance must be between 1 and {MAX_PAYLOAD_BYTES} bytes"
        )));
    }
    parse_public_provenance_store(public_provenance).map_err(LedgerError::MalformedCandidate)?;
    Ok(())
}

fn parse_public_provenance_store(public_provenance: &[u8]) -> Result<Store, String> {
    let store = Store::new().map_err(|error| error.to_string())?;
    store
        .load_from_reader(RdfFormat::Turtle, Cursor::new(public_provenance))
        .map_err(|error| format!("public provenance is not valid Turtle: {error}"))?;
    match store.iter().next() {
        None => Err("public provenance must contain at least one RDF statement".to_string()),
        Some(Ok(_)) => Ok(store),
        Some(Err(error)) => Err(format!("public provenance read failed: {error}")),
    }
}

fn projected_public_quads(index: u64, public_provenance: &[u8]) -> Result<Vec<Quad>, LedgerError> {
    let parsed_store =
        parse_public_provenance_store(public_provenance).map_err(LedgerError::MalformedEnvelope)?;
    let graph_name = NamedNode::new(format!("http://provchain.org/block/{index}"))
        .map_err(|error| LedgerError::MalformedEnvelope(error.to_string()))?;
    let mut scoped_labels = BTreeMap::new();
    let mut projected_quads = Vec::new();
    for result in parsed_store.iter() {
        let quad = result.map_err(|error| {
            LedgerError::MalformedEnvelope(format!("public provenance read failed: {error}"))
        })?;
        for identifier in blank_node_ids(&quad) {
            scoped_labels
                .entry(identifier.clone())
                .or_insert_with(|| scoped_blank_node_label(index, &identifier));
        }
        projected_quads.push(Quad::new(
            remap_subject(&quad.subject, &scoped_labels)?,
            quad.predicate,
            remap_term(&quad.object, &scoped_labels)?,
            graph_name.clone(),
        ));
    }
    Ok(projected_quads)
}

fn asserted_public_state_quads(
    envelopes: &[AdmittedBlockEnvelope],
) -> Result<Vec<Quad>, LedgerError> {
    let mut public_state = Vec::new();
    for envelope in envelopes {
        if envelope.admission_kind == AdmissionKind::OrdinaryProvenanceV1 {
            public_state.extend(projected_public_quads(
                envelope.index,
                &envelope.public_provenance,
            )?);
        }
    }
    Ok(public_state)
}

fn scoped_blank_node_label(index: u64, identifier: &str) -> String {
    format!(
        "provchain-block-{index}-{}",
        hex::encode(identifier.as_bytes())
    )
}

fn calculate_payload_state_commitment(
    index: u64,
    public_provenance: &[u8],
    bounds: StateCommitmentBounds,
) -> Result<LedgerHash, LedgerError> {
    let quads = projected_public_quads(index, public_provenance)?;
    calculate_public_state_commitment(&quads, bounds)
}

fn calculate_public_state_commitment(
    quads: &[Quad],
    bounds: StateCommitmentBounds,
) -> Result<LedgerHash, LedgerError> {
    let canonical_nquads = canonicalize_public_dataset(quads, bounds)?;
    Ok(hash_parts(POST_STATE_DOMAIN, &[&canonical_nquads]))
}

/// Return the scheme-defined commitment of the empty public provenance state.
pub fn genesis_state_commitment() -> LedgerHash {
    hash_parts(POST_STATE_DOMAIN, &[&[]])
}

fn canonicalize_public_dataset(
    quads: &[Quad],
    bounds: StateCommitmentBounds,
) -> Result<Vec<u8>, LedgerError> {
    let state = CanonicalizationState::new(quads, bounds)?;
    let mut canonical_issuer = IdentifierIssuer::new("c14n");
    let mut shared_hash_groups = Vec::new();
    for (hash, identifiers) in &state.first_degree_hashes.iter().fold(
        BTreeMap::<String, Vec<String>>::new(),
        |mut groups, (id, hash)| {
            groups.entry(hash.clone()).or_default().push(id.clone());
            groups
        },
    ) {
        if identifiers.len() == 1 {
            canonical_issuer.issue(&identifiers[0]);
        } else {
            shared_hash_groups.push((hash.clone(), identifiers.clone()));
        }
    }

    let mut work_budget = bounds.max_canonicalization_work;
    for (_, identifiers) in shared_hash_groups {
        let mut hash_path_list = Vec::new();
        for identifier in identifiers {
            if canonical_issuer.contains(&identifier) {
                continue;
            }
            let mut temporary_issuer = IdentifierIssuer::new("b");
            temporary_issuer.issue(&identifier);
            let (hash, temporary_issuer) = hash_n_degree_quads(
                &state,
                &identifier,
                temporary_issuer,
                &canonical_issuer,
                &mut work_budget,
            )?;
            hash_path_list.push((hash, temporary_issuer));
        }
        hash_path_list.sort_by(|left, right| left.0.cmp(&right.0));
        for (_, temporary_issuer) in hash_path_list {
            for identifier in temporary_issuer.order {
                canonical_issuer.issue(&identifier);
            }
        }
    }

    if state
        .quads_by_blank_node
        .keys()
        .any(|identifier| !canonical_issuer.contains(identifier))
    {
        return Err(LedgerError::ProtocolBoundExceeded(
            "canonicalization did not assign every blank node".to_string(),
        ));
    }

    let labels = canonical_issuer.issued;
    let mut canonical_quads = Vec::with_capacity(state.quads.len());
    for quad in &state.quads {
        canonical_quads.push(canonical_quad_bytes(quad, &labels)?);
    }
    canonical_quads.sort();
    Ok(canonical_quads.into_iter().flatten().collect())
}

struct CanonicalizationState {
    quads: Vec<Quad>,
    quads_by_blank_node: BTreeMap<String, Vec<usize>>,
    first_degree_hashes: BTreeMap<String, String>,
}

impl CanonicalizationState {
    fn new(quads: &[Quad], bounds: StateCommitmentBounds) -> Result<Self, LedgerError> {
        if quads.len() > bounds.max_public_state_quads {
            return Err(LedgerError::ProtocolBoundExceeded(format!(
                "public state contains more than {} quads",
                bounds.max_public_state_quads
            )));
        }
        let quads = quads.to_vec();
        let mut quads_by_blank_node = BTreeMap::<String, Vec<usize>>::new();
        for (index, quad) in quads.iter().enumerate() {
            for identifier in blank_node_ids(quad) {
                quads_by_blank_node
                    .entry(identifier)
                    .or_default()
                    .push(index);
            }
        }
        if quads_by_blank_node.len() > bounds.max_public_state_blank_nodes {
            return Err(LedgerError::ProtocolBoundExceeded(format!(
                "public state contains more than {} blank nodes",
                bounds.max_public_state_blank_nodes
            )));
        }
        let mut first_degree_hashes = BTreeMap::new();
        for identifier in quads_by_blank_node.keys() {
            first_degree_hashes.insert(
                identifier.clone(),
                hash_first_degree_quads(&quads, &quads_by_blank_node, identifier)?,
            );
        }
        Ok(Self {
            quads,
            quads_by_blank_node,
            first_degree_hashes,
        })
    }
}

#[derive(Clone)]
struct IdentifierIssuer {
    prefix: &'static str,
    counter: usize,
    issued: BTreeMap<String, String>,
    order: Vec<String>,
}

impl IdentifierIssuer {
    fn new(prefix: &'static str) -> Self {
        Self {
            prefix,
            counter: 0,
            issued: BTreeMap::new(),
            order: Vec::new(),
        }
    }

    fn contains(&self, identifier: &str) -> bool {
        self.issued.contains_key(identifier)
    }

    fn get(&self, identifier: &str) -> Option<&str> {
        self.issued.get(identifier).map(String::as_str)
    }

    fn issue(&mut self, identifier: &str) -> String {
        if let Some(issued) = self.issued.get(identifier) {
            return issued.clone();
        }
        let issued = format!("{}{}", self.prefix, self.counter);
        self.counter += 1;
        self.issued.insert(identifier.to_string(), issued.clone());
        self.order.push(identifier.to_string());
        issued
    }
}

fn blank_node_ids(quad: &Quad) -> BTreeSet<String> {
    let mut identifiers = BTreeSet::new();
    if let Subject::BlankNode(blank_node) = &quad.subject {
        identifiers.insert(blank_node.as_str().to_string());
    }
    if let Term::BlankNode(blank_node) = &quad.object {
        identifiers.insert(blank_node.as_str().to_string());
    }
    if let GraphName::BlankNode(blank_node) = &quad.graph_name {
        identifiers.insert(blank_node.as_str().to_string());
    }
    identifiers
}

fn hash_first_degree_quads(
    quads: &[Quad],
    quads_by_blank_node: &BTreeMap<String, Vec<usize>>,
    identifier: &str,
) -> Result<String, LedgerError> {
    let mut nquads = Vec::new();
    for index in quads_by_blank_node
        .get(identifier)
        .ok_or_else(|| LedgerError::MalformedEnvelope("unknown blank node".to_string()))?
    {
        let quad = &quads[*index];
        let labels = blank_node_ids(quad)
            .into_iter()
            .map(|blank_node| {
                let label = if blank_node == identifier { "a" } else { "z" };
                (blank_node, label.to_string())
            })
            .collect::<BTreeMap<_, _>>();
        nquads.push(canonical_quad_bytes(quad, &labels)?);
    }
    nquads.sort();
    Ok(sha256_hex(
        &nquads.into_iter().flatten().collect::<Vec<_>>(),
    ))
}

fn hash_related_blank_node(
    state: &CanonicalizationState,
    related: &str,
    quad: &Quad,
    issuer: &IdentifierIssuer,
    canonical_issuer: &IdentifierIssuer,
    position: &str,
) -> Result<String, LedgerError> {
    let mut input = position.to_string();
    if position != "g" {
        input.push('<');
        input.push_str(quad.predicate.as_str());
        input.push('>');
    }
    if let Some(identifier) = canonical_issuer
        .get(related)
        .or_else(|| issuer.get(related))
    {
        input.push_str("_:");
        input.push_str(identifier);
    } else {
        input.push_str(
            state
                .first_degree_hashes
                .get(related)
                .ok_or_else(|| LedgerError::MalformedEnvelope("unknown blank node".to_string()))?,
        );
    }
    Ok(sha256_hex(input.as_bytes()))
}

fn related_blank_nodes(quad: &Quad) -> Vec<(&str, &str)> {
    let mut related = Vec::new();
    if let Subject::BlankNode(blank_node) = &quad.subject {
        related.push((blank_node.as_str(), "s"));
    }
    if let Term::BlankNode(blank_node) = &quad.object {
        related.push((blank_node.as_str(), "o"));
    }
    if let GraphName::BlankNode(blank_node) = &quad.graph_name {
        related.push((blank_node.as_str(), "g"));
    }
    related
}

fn hash_n_degree_quads(
    state: &CanonicalizationState,
    identifier: &str,
    mut issuer: IdentifierIssuer,
    canonical_issuer: &IdentifierIssuer,
    work_budget: &mut usize,
) -> Result<(String, IdentifierIssuer), LedgerError> {
    let mut hash_to_related = BTreeMap::<String, Vec<String>>::new();
    for index in state
        .quads_by_blank_node
        .get(identifier)
        .ok_or_else(|| LedgerError::MalformedEnvelope("unknown blank node".to_string()))?
    {
        let quad = &state.quads[*index];
        for (related, position) in related_blank_nodes(quad) {
            if related != identifier {
                let hash = hash_related_blank_node(
                    state,
                    related,
                    quad,
                    &issuer,
                    canonical_issuer,
                    position,
                )?;
                hash_to_related
                    .entry(hash)
                    .or_default()
                    .push(related.to_string());
            }
        }
    }

    let mut data_to_hash = String::new();
    for (related_hash, mut related_identifiers) in hash_to_related {
        data_to_hash.push_str(&related_hash);
        related_identifiers.sort();
        let mut chosen_path: Option<(String, IdentifierIssuer)> = None;
        for permutation in permutations(&related_identifiers) {
            if *work_budget == 0 {
                return Err(LedgerError::ProtocolBoundExceeded(
                    "RDFC-1.0 structural work budget exhausted".to_string(),
                ));
            }
            *work_budget -= 1;
            let mut issuer_copy = issuer.clone();
            let mut path = String::new();
            let mut recursion_list = Vec::new();
            for related in permutation {
                if let Some(canonical_identifier) = canonical_issuer.get(&related) {
                    path.push_str("_:");
                    path.push_str(canonical_identifier);
                } else {
                    if !issuer_copy.contains(&related) {
                        recursion_list.push(related.clone());
                    }
                    path.push_str("_:");
                    path.push_str(&issuer_copy.issue(&related));
                }
            }
            for related in recursion_list {
                let (hash, returned_issuer) = hash_n_degree_quads(
                    state,
                    &related,
                    issuer_copy,
                    canonical_issuer,
                    work_budget,
                )?;
                issuer_copy = returned_issuer;
                path.push_str("_:");
                path.push_str(issuer_copy.get(&related).ok_or_else(|| {
                    LedgerError::MalformedEnvelope(
                        "temporary blank-node issuer lost an identifier".to_string(),
                    )
                })?);
                path.push('<');
                path.push_str(&hash);
                path.push('>');
            }
            let is_better_path = match chosen_path.as_ref() {
                None => true,
                Some((chosen, _)) => path < *chosen,
            };
            if is_better_path {
                chosen_path = Some((path, issuer_copy));
            }
        }
        let (chosen_path, chosen_issuer) = chosen_path.ok_or_else(|| {
            LedgerError::ProtocolBoundExceeded("no RDFC-1.0 permutation was evaluated".to_string())
        })?;
        data_to_hash.push_str(&chosen_path);
        issuer = chosen_issuer;
    }
    Ok((sha256_hex(data_to_hash.as_bytes()), issuer))
}

fn permutations(values: &[String]) -> Vec<Vec<String>> {
    fn visit(start: usize, values: &mut [String], output: &mut Vec<Vec<String>>) {
        if start == values.len() {
            output.push(values.to_vec());
            return;
        }
        for index in start..values.len() {
            values.swap(start, index);
            visit(start + 1, values, output);
            values.swap(start, index);
        }
    }

    let mut values = values.to_vec();
    let mut output = Vec::new();
    visit(0, &mut values, &mut output);
    output
}

fn canonical_quad_bytes(
    quad: &Quad,
    labels: &BTreeMap<String, String>,
) -> Result<Vec<u8>, LedgerError> {
    let quad = Quad::new(
        remap_subject(&quad.subject, labels)?,
        quad.predicate.clone(),
        remap_term(&quad.object, labels)?,
        remap_graph_name(&quad.graph_name, labels)?,
    );
    let mut serializer = RdfSerializer::from_format(RdfFormat::NQuads).for_writer(Vec::new());
    serializer.serialize_quad(&quad).map_err(|error| {
        LedgerError::MalformedEnvelope(format!("N-Quads serialization failed: {error}"))
    })?;
    serializer.finish().map_err(|error| {
        LedgerError::MalformedEnvelope(format!("N-Quads serialization failed: {error}"))
    })
}

fn remap_subject(
    subject: &Subject,
    labels: &BTreeMap<String, String>,
) -> Result<Subject, LedgerError> {
    match subject {
        Subject::NamedNode(node) => Ok(Subject::NamedNode(node.clone())),
        Subject::BlankNode(node) => Ok(Subject::BlankNode(remap_blank_node(node, labels))),
        _ => Err(LedgerError::ProtocolBoundExceeded(
            "RDF-star terms are outside the Issue #2 state scheme".to_string(),
        )),
    }
}

fn remap_term(term: &Term, labels: &BTreeMap<String, String>) -> Result<Term, LedgerError> {
    match term {
        Term::NamedNode(node) => Ok(Term::NamedNode(node.clone())),
        Term::BlankNode(node) => Ok(Term::BlankNode(remap_blank_node(node, labels))),
        Term::Literal(literal) => Ok(Term::Literal(literal.clone())),
        _ => Err(LedgerError::ProtocolBoundExceeded(
            "RDF-star terms are outside the Issue #2 state scheme".to_string(),
        )),
    }
}

fn remap_graph_name(
    graph_name: &GraphName,
    labels: &BTreeMap<String, String>,
) -> Result<GraphName, LedgerError> {
    match graph_name {
        GraphName::NamedNode(node) => Ok(GraphName::NamedNode(node.clone())),
        GraphName::BlankNode(node) => Ok(GraphName::BlankNode(remap_blank_node(node, labels))),
        GraphName::DefaultGraph => Ok(GraphName::DefaultGraph),
    }
}

fn remap_blank_node(blank_node: &BlankNode, labels: &BTreeMap<String, String>) -> BlankNode {
    BlankNode::new_unchecked(
        labels
            .get(blank_node.as_str())
            .map(String::as_str)
            .unwrap_or_else(|| blank_node.as_str()),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn candidate_body_from_envelope(envelope: &AdmittedBlockEnvelope) -> Result<Vec<u8>, LedgerError> {
    let mut writer = CanonicalWriter::new(PROPOSAL_MAGIC);
    writer.u16(envelope.version);
    writer.u8(envelope.admission_kind as u8);
    writer.string(&envelope.network_id, MAX_TEXT_BYTES)?;
    writer.string(&envelope.profile_id, MAX_TEXT_BYTES)?;
    writer.string(&envelope.ontology_package_id, MAX_TEXT_BYTES)?;
    writer.string(&envelope.ontology_package_version, MAX_TEXT_BYTES)?;
    writer.string(&envelope.ontology_package_hash, MAX_TEXT_BYTES)?;
    writer.string(&envelope.semantic_execution_profile_id, MAX_TEXT_BYTES)?;
    writer.u8(envelope.state_commitment_scheme as u8);
    writer.u64(envelope.index);
    writer.fixed(&envelope.previous_envelope_hash);
    writer.fixed(&envelope.previous_state_commitment);
    writer.u64(envelope.timestamp_millis);
    writer.bytes(&envelope.public_provenance, MAX_PAYLOAD_BYTES)?;
    writer.optional_bytes(envelope.encrypted_payload.as_deref(), MAX_PAYLOAD_BYTES)?;
    if envelope.version == PRIVACY_ENVELOPE_VERSION {
        writer.optional_bytes(
            envelope.privacy_control.as_deref(),
            MAX_PRIVACY_TRANSITION_BYTES,
        )?;
    }
    if envelope.version == BRIDGE_EXPORT_ENVELOPE_VERSION {
        writer.bytes(
            envelope
                .bridge_export_declaration
                .as_deref()
                .ok_or_else(|| {
                    LedgerError::MalformedEnvelope(
                        "bridge export envelope is missing its declaration".to_string(),
                    )
                })?,
            MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
        )?;
    }
    if envelope.version == BRIDGE_IMPORT_ENVELOPE_VERSION {
        writer.bytes(
            envelope.bridge_origin_evidence.as_deref().ok_or_else(|| {
                LedgerError::MalformedEnvelope(
                    "bridge import envelope is missing origin evidence".to_string(),
                )
            })?,
            MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
        )?;
    }
    writer.fixed(&envelope.post_state_commitment);
    writer.fixed(&envelope.proposer_public_key);
    Ok(writer.finish())
}

fn codec_version(
    admission_kind: AdmissionKind,
    has_bridge_export: bool,
    has_bridge_import: bool,
) -> u16 {
    match (admission_kind, has_bridge_export, has_bridge_import) {
        (AdmissionKind::OrdinaryProvenanceV1, false, false) => ORDINARY_ENVELOPE_VERSION,
        (AdmissionKind::OrdinaryProvenanceV1, true, false) => BRIDGE_EXPORT_ENVELOPE_VERSION,
        (AdmissionKind::OrdinaryProvenanceV1, false, true) => BRIDGE_IMPORT_ENVELOPE_VERSION,
        (AdmissionKind::OrdinaryProvenanceV1, true, true) => BRIDGE_IMPORT_ENVELOPE_VERSION,
        (AdmissionKind::PrivacyControlV1, _, _) => PRIVACY_ENVELOPE_VERSION,
    }
}

fn bridge_candidate_error(error: BridgeError) -> LedgerError {
    LedgerError::MalformedCandidate(error.to_string())
}

fn validate_codec_version(
    version: u16,
    admission_kind: AdmissionKind,
    has_bridge_export: bool,
    has_bridge_import: bool,
) -> Result<(), LedgerError> {
    if has_bridge_export && has_bridge_import
        || version != codec_version(admission_kind, has_bridge_export, has_bridge_import)
    {
        return Err(LedgerError::MalformedEnvelope(format!(
            "envelope version {version} does not support admission kind {}",
            admission_kind as u8
        )));
    }
    Ok(())
}

fn hash_parts(domain: &[u8], parts: &[&[u8]]) -> LedgerHash {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

fn validate_text(value: &str, field: &str, max_bytes: usize) -> Result<(), LedgerError> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > max_bytes
        || value.chars().any(|character| character.is_control())
    {
        return Err(LedgerError::MalformedEnvelope(format!(
            "{field} is empty, bounded, trimmed, printable text"
        )));
    }
    Ok(())
}

struct CanonicalWriter {
    bytes: Vec<u8>,
}

impl CanonicalWriter {
    fn new(magic: &[u8]) -> Self {
        Self {
            bytes: magic.to_vec(),
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn fixed<const N: usize>(&mut self, value: &[u8; N]) {
        self.bytes.extend_from_slice(value);
    }

    fn string(&mut self, value: &str, max_bytes: usize) -> Result<(), LedgerError> {
        self.bytes(value.as_bytes(), max_bytes)
    }

    fn bytes(&mut self, value: &[u8], max_bytes: usize) -> Result<(), LedgerError> {
        if value.len() > max_bytes || value.len() > u32::MAX as usize {
            return Err(LedgerError::OversizedEnvelope(value.len()));
        }
        self.bytes
            .extend_from_slice(&(value.len() as u32).to_be_bytes());
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn optional_bytes(
        &mut self,
        value: Option<&[u8]>,
        max_bytes: usize,
    ) -> Result<(), LedgerError> {
        match value {
            Some(value) => {
                self.u8(1);
                self.bytes(value, max_bytes)?;
            }
            None => self.u8(0),
        }
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct CanonicalReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn magic(&mut self, expected: &[u8]) -> Result<(), LedgerError> {
        let actual = self.take(expected.len())?;
        if actual != expected {
            return Err(LedgerError::MalformedEnvelope(
                "unexpected codec magic".to_string(),
            ));
        }
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, LedgerError> {
        Ok(*self.take(1)?.first().ok_or_else(|| {
            LedgerError::MalformedEnvelope("unexpected end of envelope".to_string())
        })?)
    }

    fn u16(&mut self) -> Result<u16, LedgerError> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, LedgerError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], LedgerError> {
        self.take(N)?
            .try_into()
            .map_err(|_| LedgerError::MalformedEnvelope("unexpected end of envelope".to_string()))
    }

    fn string(&mut self, max_bytes: usize, field: &str) -> Result<String, LedgerError> {
        let bytes = self.bytes(max_bytes, field)?;
        String::from_utf8(bytes)
            .map_err(|_| LedgerError::MalformedEnvelope(format!("{field} is not valid UTF-8")))
    }

    fn bytes(&mut self, max_bytes: usize, field: &str) -> Result<Vec<u8>, LedgerError> {
        let length = u32::from_be_bytes(self.fixed()?) as usize;
        if length > max_bytes {
            return Err(LedgerError::OversizedEnvelope(length));
        }
        self.take(length)
            .map(|bytes| bytes.to_vec())
            .map_err(|_| LedgerError::MalformedEnvelope(format!("invalid {field} length")))
    }

    fn optional_bytes(
        &mut self,
        max_bytes: usize,
        field: &str,
    ) -> Result<Option<Vec<u8>>, LedgerError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.bytes(max_bytes, field)?)),
            flag => Err(LedgerError::MalformedEnvelope(format!(
                "invalid optional {field} flag {flag}"
            ))),
        }
    }

    fn finish(&self) -> Result<(), LedgerError> {
        if self.offset != self.bytes.len() {
            return Err(LedgerError::NonCanonicalEnvelope);
        }
        Ok(())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], LedgerError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| LedgerError::MalformedEnvelope("length overflow".to_string()))?;
        if end > self.bytes.len() {
            return Err(LedgerError::MalformedEnvelope(
                "unexpected end of envelope".to_string(),
            ));
        }
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }
}

const ORDINARY_ENVELOPE_VERSION: u16 = 2;
const PRIVACY_ENVELOPE_VERSION: u16 = 3;
const BRIDGE_EXPORT_ENVELOPE_VERSION: u16 = 4;
const BRIDGE_IMPORT_ENVELOPE_VERSION: u16 = 5;
const MAX_TEXT_BYTES: usize = 4096;
/// Maximum bytes in one public or encrypted candidate payload.
pub const MAX_PAYLOAD_BYTES: usize = 262_144;
const MAX_PRIVACY_TRANSITION_BYTES: usize = crate::privacy::MAX_CREATE_PROTECTED_OBJECT_BYTES;
/// Maximum bytes in one canonical admitted envelope.
pub const MAX_ENVELOPE_BYTES: usize = 1_048_576;
const FRAME_LENGTH_BYTES: usize = 4;
const FRAME_CHECKSUM_BYTES: usize = 32;
const JOURNAL_MAGIC: &[u8] = b"PROVCHAIN_LEDGER_JOURNAL_V1";
const ENVELOPE_MAGIC: &[u8] = b"PROVCHAIN_ADMITTED_ENVELOPE_V2";
const PROPOSAL_MAGIC: &[u8] = b"PROVCHAIN_PROPOSAL_V2";
const PROPOSAL_DIGEST_DOMAIN: &[u8] = b"provchain/ordinary-proposal-digest/v1";
const ENVELOPE_HASH_DOMAIN: &[u8] = b"provchain/admitted-envelope-hash/v1";
const POST_STATE_DOMAIN: &[u8] = b"provchain/public-state-commitment/v1";
const JOURNAL_FRAME_DOMAIN: &[u8] = b"provchain/ledger-journal-frame/v1";
const MAX_PUBLIC_STATE_QUADS: usize = 65_536;
const MAX_CANONICALIZATION_BLANK_NODES: usize = 8;
const MAX_CANONICALIZATION_WORK: usize = 250_000;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::semantic_admission::activated_test_package;
    use tempfile::tempdir;

    fn profile() -> LedgerProfile {
        let package = activated_test_package();
        LedgerProfile::new("test-network", "test-profile").with_semantic_package(
            package.package_id(),
            package.package_version(),
            package.package_hash(),
        )
    }

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[9; 32])
    }

    fn payload() -> &'static [u8] {
        br#"@prefix ex: <http://example.org/> .
ex:item ex:value "one" ."#
    }

    #[derive(Clone, Copy)]
    enum TestIngress {
        Local,
        Api,
        Batch,
        Synchronization,
        Replication,
        Proposal,
    }

    fn submit(
        node: &mut Ledger,
        ingress: TestIngress,
        candidate: AdmissionCandidate,
    ) -> Result<AdmissionOutcome, LedgerError> {
        match ingress {
            TestIngress::Local => node.local_adapter().submit(candidate),
            TestIngress::Api => node.api_adapter().submit(candidate),
            TestIngress::Batch => node.batch_adapter().submit(candidate),
            TestIngress::Synchronization => node.synchronization_adapter().submit(candidate),
            TestIngress::Replication => node.replication_adapter().submit(candidate),
            TestIngress::Proposal => node.proposal_adapter().submit(candidate),
        }
    }

    #[test]
    fn candidate_signature_and_envelope_codec_are_deterministic() {
        let candidate = AdmissionCandidate::new_ordinary(
            &profile(),
            0,
            [0; 32],
            genesis_state_commitment(),
            1_700_000_000_000,
            payload(),
        )
        .unwrap()
        .sign(&key())
        .unwrap();
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let outcome = submit(&mut node, TestIngress::Local, candidate).unwrap();
        let envelope = match outcome {
            AdmissionOutcome::Committed { envelope } => *envelope,
            AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
        };
        let first = envelope.canonical_bytes().unwrap();
        let second = envelope.canonical_bytes().unwrap();
        assert_eq!(first, second);
        assert_eq!(AdmittedBlockEnvelope::decode(&first).unwrap(), envelope);
    }

    #[test]
    fn legacy_bridge_origin_adapter_rejects_import_evidence() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let mut candidate = AdmissionCandidate::new_ordinary(
            &profile(),
            0,
            [0; 32],
            genesis_state_commitment(),
            1_700_000_000_000,
            payload(),
        )
        .unwrap();
        candidate.bridge_origin_evidence = Some(vec![0x11]);

        assert!(matches!(
            node.bridge_origin_adapter().submit(candidate),
            Err(LedgerError::MalformedCandidate(reason))
                if reason.contains("PoA Proposal Coordinator")
        ));
        assert_eq!(node.journal_frame_count().unwrap(), 0);
    }

    #[test]
    fn envelope_decode_rejects_stale_proposal_evidence() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();
        let envelope = match submit(&mut node, TestIngress::Replication, candidate).unwrap() {
            AdmissionOutcome::Committed { envelope } => *envelope,
            AdmissionOutcome::Rejected { reason } => panic!("unexpected rejection: {reason}"),
        };

        let mut tampered = envelope.clone();
        tampered.public_provenance = br#"@prefix ex: <http://example.org/> .
ex:item ex:value "two" ."#
            .to_vec();
        tampered.envelope_hash = tampered.compute_envelope_hash().unwrap();
        let bytes = tampered.encode().unwrap();

        assert!(matches!(
            AdmittedBlockEnvelope::decode(&bytes),
            Err(LedgerError::TamperedEnvelope(_))
        ));
    }

    #[test]
    fn injected_journal_failures_do_not_append_frames() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();

        node.inject_next_journal_write_failure();
        assert!(submit(&mut node, TestIngress::Api, candidate.clone()).is_err());
        assert_eq!(node.journal_frame_count().unwrap(), 0);

        node.inject_next_journal_fsync_failure();
        assert!(submit(&mut node, TestIngress::Batch, candidate).is_err());
        assert_eq!(node.journal_frame_count().unwrap(), 0);
    }

    #[test]
    fn exact_lost_response_retry_does_not_append_again() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();
        node.inject_next_response_failure();
        assert!(matches!(
            submit(&mut node, TestIngress::Proposal, candidate.clone()),
            Err(LedgerError::ResponseLost { .. })
        ));
        assert_eq!(node.journal_frame_count().unwrap(), 1);
        assert!(matches!(
            submit(&mut node, TestIngress::Synchronization, candidate).unwrap(),
            AdmissionOutcome::Committed { .. }
        ));
        assert_eq!(node.journal_frame_count().unwrap(), 1);
    }

    #[test]
    fn replay_rebuilds_oxigraph_after_projection_failure() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();
        node.inject_next_projection_failure();
        assert!(matches!(
            submit(&mut node, TestIngress::Replication, candidate),
            Err(LedgerError::ProjectionFailed { .. })
        ));
        assert!(node.is_degraded());
        assert_eq!(node.journal_frame_count().unwrap(), 1);
        assert!(matches!(
            node.effective_privacy_state(),
            Err(LedgerError::Degraded)
        ));
        node.replay().unwrap();
        assert!(!node.is_degraded());
        assert!(node.effective_privacy_state().is_ok());
        assert_eq!(node.committed_envelopes().len(), 1);
        assert_eq!(node.rdf_store().len().unwrap(), 1);
    }

    #[test]
    fn torn_tail_requires_explicit_recovery() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();
        submit(&mut node, TestIngress::Local, candidate).unwrap();
        let path = node.journal_path().to_path_buf();
        drop(node);
        let mut file = OpenOptions::new().append(true).open(path).unwrap();
        file.write_all(&[0, 0, 0, 100, 1, 2, 3]).unwrap();
        drop(file);

        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        assert!(node.is_degraded());
        assert_eq!(node.journal_frame_count().unwrap(), 1);
        assert_eq!(node.committed_envelopes().len(), 1);
        assert!(matches!(node.replay(), Err(LedgerError::CorruptJournal(_))));
        node.repair_torn_tail().unwrap();
        assert!(!node.is_degraded());
        assert_eq!(node.journal_frame_count().unwrap(), 1);
        assert_eq!(node.committed_envelopes().len(), 1);
    }

    #[test]
    fn wrong_parent_and_profile_are_rejected_without_append() {
        let ledger = tempdir().unwrap();
        let mut node =
            Ledger::open_in_dir(ledger.path(), profile(), activated_test_package()).unwrap();
        let mut candidate = node
            .create_ordinary_candidate(payload(), 1_700_000_000_000, &key())
            .unwrap();
        candidate.previous_envelope_hash = [3; 32];
        assert!(matches!(
            submit(&mut node, TestIngress::Proposal, candidate),
            Ok(AdmissionOutcome::Rejected { .. })
        ));
        assert_eq!(node.journal_frame_count().unwrap(), 0);
    }
}
