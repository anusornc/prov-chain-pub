//! Exact source export and target import for the bounded bridge.
//!
//! Source proofs require convergence-backed receipts. Target trust is activated
//! independently, imports pass through target PoA and universal Final Admission,
//! and Effective Bridge State is derived only from verified journal replay.

use std::collections::{BTreeMap, BTreeSet};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::ledger::{AdmissionKind, AdmittedBlockEnvelope, LedgerHash, MAX_PAYLOAD_BYTES};
use crate::network::membership::{
    validate_profile_authorities, MemberRole, MemberStatus, SignedMembershipManifest,
};
use crate::network::profile::NetworkProfile;

/// Maximum canonical Bridge Export Declaration size.
pub const MAX_BRIDGE_EXPORT_DECLARATION_BYTES: usize = 1_024;
/// Maximum canonical source Network Profile artifact size.
pub const MAX_BRIDGE_SOURCE_PROFILE_BYTES: usize = 32_768;
/// Maximum canonical signed source Membership Manifest artifact size.
pub const MAX_BRIDGE_SIGNED_MANIFEST_BYTES: usize = 65_536;
/// Maximum canonical declaration-bearing source envelope size.
pub const MAX_BRIDGE_SOURCE_ENVELOPE_BYTES: usize = 393_216;
/// Maximum canonical bridge Commit Receipt size.
pub const MAX_BRIDGE_COMMIT_RECEIPT_BYTES: usize = 4_096;
/// Exact number of receipt signers in the bounded reference topology.
pub const REQUIRED_BRIDGE_RECEIPTS: usize = 3;
/// Maximum complete Bridge Proof Bundle size fixed by ADR 0037.
pub const MAX_BRIDGE_PROOF_BUNDLE_BYTES: usize = 503_883;
/// Maximum complete Bridge Origin Evidence size fixed by ADR 0037.
pub const MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES: usize = 503_968;

const BRIDGE_MAGIC: &[u8; 4] = b"BCV1";
const BRIDGE_EXPORT_CORE_TAG: u8 = 0x01;
const BRIDGE_EXPORT_DECLARATION_TAG: u8 = 0x02;
const BRIDGE_COMMIT_RECEIPT_CORE_TAG: u8 = 0x03;
const BRIDGE_COMMIT_RECEIPT_TAG: u8 = 0x04;
const BRIDGE_PROOF_BUNDLE_TAG: u8 = 0x10;
const BRIDGE_ORIGIN_EVIDENCE_TAG: u8 = 0x11;
const ORDINARY_PROVENANCE_V1_TAG: u8 = 0x01;
const EXACT_PUBLIC_PAYLOAD_V1_TAG: u8 = 0x01;
const STRICT_ED25519_V1_TAG: u8 = 0x01;
const TRANSFER_ID_DOMAIN: &[u8] = b"provchain/bridge/transfer-id/v1";
const RECEIPT_SIGNATURE_DOMAIN: &[u8] = b"provchain/bridge/commit-receipt-signature/v1";
const MAX_IDENTIFIER_BYTES: usize = 128;

/// A nonzero public identifier for one exact ledger history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LedgerInstanceId32([u8; 32]);

impl LedgerInstanceId32 {
    /// Validate one exact raw identifier.
    pub fn new(bytes: [u8; 32]) -> Result<Self, BridgeError> {
        if bytes == [0; 32] {
            return Err(BridgeError::Malformed(
                "ledger instance identifier must be nonzero".to_string(),
            ));
        }
        Ok(Self(bytes))
    }

    /// Return the exact raw identifier bytes.
    pub fn bytes(self) -> [u8; 32] {
        self.0
    }
}

/// One exact target permitted by a source bridge profile.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeExportTargetV1 {
    network_id: String,
    ledger_instance_id: LedgerInstanceId32,
}

impl BridgeExportTargetV1 {
    /// Bind a target network name and exact ledger instance.
    pub fn new(
        network_id: impl Into<String>,
        ledger_instance_id: LedgerInstanceId32,
    ) -> Result<Self, BridgeError> {
        let network_id = network_id.into();
        validate_identifier(&network_id, "target network_id")?;
        Ok(Self {
            network_id,
            ledger_instance_id,
        })
    }

    /// Target network identifier.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Target ledger-instance identifier.
    pub fn ledger_instance_id(&self) -> LedgerInstanceId32 {
        self.ledger_instance_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct BridgeTrustedReceiptNodeV1 {
    node_id: String,
    identity_public_key: String,
}

/// Static source trust embedded in the target Network Profile.
///
/// This record is configuration, not proof-carried authority. Runtime
/// activation reconstructs it from independently supplied source artifacts and
/// rejects any difference before a target node exposes import capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BridgeSourceTrustProfileV1 {
    target_ledger_instance_id: String,
    source_network_id: String,
    source_ledger_instance_id: String,
    source_profile_id: String,
    source_profile_content_hash: String,
    source_governance_root: String,
    source_manifest_id: String,
    source_manifest_version: u64,
    source_manifest_digest: String,
    source_receipt_nodes: Vec<BridgeTrustedReceiptNodeV1>,
    source_authority_keys: Vec<String>,
    ontology_package_id: String,
    ontology_package_version: String,
    ontology_package_hash: String,
    semantic_execution_profile_id: String,
    state_commitment_scheme: String,
    source_admission_kind: String,
    bridge_suite: String,
    copy_mode: String,
    receipt_policy: String,
}

impl BridgeSourceTrustProfileV1 {
    /// Pin one exact source contract into a named target ledger instance.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` if the target network identifier is invalid, the
    /// ledger instance identifier is zero, the source profile fails validation,
    /// or the source manifest does not verify under the governance root.
    pub fn new(
        target_network_id: &str,
        target_ledger_instance_id: [u8; 32],
        source_profile: &NetworkProfile,
        source_manifest: &SignedMembershipManifest,
        source_governance_root: &VerifyingKey,
    ) -> Result<Self, BridgeError> {
        validate_identifier(target_network_id, "target network_id")?;
        let target_ledger_instance_id = LedgerInstanceId32::new(target_ledger_instance_id)?;
        source_profile
            .validate()
            .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?;
        source_manifest
            .verify(source_governance_root)
            .map_err(|_| {
                BridgeError::InvalidProfile(
                    "source Membership Manifest governance signature is invalid".to_string(),
                )
            })?;
        let source_bridge = source_profile.bridge.as_ref().ok_or_else(|| {
            BridgeError::InvalidProfile(
                "source Network Profile has no outbound bridge rule".to_string(),
            )
        })?;
        if source_bridge.outbound_rule().target_network_id() != target_network_id
            || source_bridge
                .outbound_rule()
                .target_ledger_instance_id()
                .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?
                != target_ledger_instance_id.bytes()
        {
            return Err(BridgeError::InvalidProfile(
                "source outbound rule does not name this exact target ledger".to_string(),
            ));
        }
        let source_ledger_instance_id = LedgerInstanceId32::new(
            source_bridge
                .source_ledger_instance_id()
                .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
        )?;
        let target = BridgeExportTargetV1::new(target_network_id, target_ledger_instance_id)?;
        let source = BridgeSourceProfileV1::from_network_profile(
            source_profile,
            source_manifest,
            source_ledger_instance_id,
            target,
        )?;
        let authorities: BTreeSet<_> = source.authority_keys.iter().copied().collect();
        let mut source_receipt_nodes = source_manifest
            .manifest
            .members
            .iter()
            .filter(|member| {
                member.status == MemberStatus::Active
                    && member.roles.contains(&MemberRole::Peer)
                    && member.roles.contains(&MemberRole::Validator)
                    && member
                        .validator_public_key
                        .is_some_and(|key| authorities.contains(&key))
            })
            .map(|member| BridgeTrustedReceiptNodeV1 {
                node_id: member.node_id.to_string(),
                identity_public_key: hex::encode(member.identity_public_key),
            })
            .collect::<Vec<_>>();
        source_receipt_nodes.sort();
        let trust = Self {
            target_ledger_instance_id: hex::encode(target_ledger_instance_id.bytes()),
            source_network_id: source.network_id.clone(),
            source_ledger_instance_id: hex::encode(source_ledger_instance_id.bytes()),
            source_profile_id: source.profile_id.clone(),
            source_profile_content_hash: hex::encode(source.content_hash()?),
            source_governance_root: hex::encode(source_governance_root.to_bytes()),
            source_manifest_id: source.manifest_id.clone(),
            source_manifest_version: source.manifest_version,
            source_manifest_digest: hex::encode(source.manifest_digest),
            source_receipt_nodes,
            source_authority_keys: source.authority_keys.iter().map(hex::encode).collect(),
            ontology_package_id: source.ontology_package_id.clone(),
            ontology_package_version: source.ontology_package_version.clone(),
            ontology_package_hash: source.ontology_package_hash.clone(),
            semantic_execution_profile_id: source.semantic_execution_profile_id.clone(),
            state_commitment_scheme: "Rdfc10Sha256NQuadsV1".to_string(),
            source_admission_kind: "OrdinaryProvenanceV1".to_string(),
            bridge_suite: "ProvChainBridgeSuiteV1".to_string(),
            copy_mode: "ExactPublicPayloadV1".to_string(),
            receipt_policy: "AllThreeCommitReceiptsV1".to_string(),
        };
        trust.validate()?;
        Ok(trust)
    }

    pub(crate) fn validate(&self) -> Result<(), BridgeError> {
        for (value, field) in [
            (&self.source_network_id, "trusted source network_id"),
            (&self.source_profile_id, "trusted source profile_id"),
            (&self.source_manifest_id, "trusted source manifest_id"),
            (
                &self.ontology_package_id,
                "trusted source ontology_package_id",
            ),
            (
                &self.semantic_execution_profile_id,
                "trusted source semantic_execution_profile_id",
            ),
        ] {
            validate_identifier(value, field)?;
        }
        if self.source_manifest_version == 0 {
            return Err(BridgeError::InvalidProfile(
                "trusted source manifest version must be nonzero".to_string(),
            ));
        }
        for (value, field) in [
            (&self.target_ledger_instance_id, "target ledger instance"),
            (&self.source_ledger_instance_id, "source ledger instance"),
            (&self.source_profile_content_hash, "source profile hash"),
            (&self.source_governance_root, "source governance root"),
            (&self.source_manifest_digest, "source manifest digest"),
        ] {
            nonzero_hash(&hex32(value, field)?, field)?;
        }
        if self.source_receipt_nodes.len() != REQUIRED_BRIDGE_RECEIPTS
            || self
                .source_receipt_nodes
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.source_authority_keys.len() != REQUIRED_BRIDGE_RECEIPTS
        {
            return Err(BridgeError::InvalidProfile(
                "target trust requires exactly three ordered source receipt nodes and authorities"
                    .to_string(),
            ));
        }
        let mut identity_keys = BTreeSet::new();
        for node in &self.source_receipt_nodes {
            let node_id = Uuid::parse_str(&node.node_id).map_err(|_| {
                BridgeError::InvalidProfile("invalid trusted source node identity".to_string())
            })?;
            if node_id.to_string() != node.node_id {
                return Err(BridgeError::InvalidProfile(
                    "trusted source node identity is not canonical".to_string(),
                ));
            }
            let key = hex32(&node.identity_public_key, "source Node Identity Key")?;
            VerifyingKey::from_bytes(&key).map_err(|_| {
                BridgeError::InvalidProfile("invalid source Node Identity Key".to_string())
            })?;
            if !identity_keys.insert(key) {
                return Err(BridgeError::InvalidProfile(
                    "source Node Identity Keys must be unique".to_string(),
                ));
            }
        }
        let mut authority_keys = BTreeSet::new();
        for key in &self.source_authority_keys {
            let key = hex32(key, "source authority key")?;
            VerifyingKey::from_bytes(&key).map_err(|_| {
                BridgeError::InvalidProfile("invalid source authority key".to_string())
            })?;
            if !authority_keys.insert(key) || identity_keys.contains(&key) {
                return Err(BridgeError::InvalidProfile(
                    "source authority keys must be unique and role-separated".to_string(),
                ));
            }
        }
        if self.ontology_package_version.is_empty()
            || self.ontology_package_hash.len() != 64
            || hex32(&self.ontology_package_hash, "source ontology package hash").is_err()
            || self.state_commitment_scheme != "Rdfc10Sha256NQuadsV1"
            || self.source_admission_kind != "OrdinaryProvenanceV1"
            || self.bridge_suite != "ProvChainBridgeSuiteV1"
            || self.copy_mode != "ExactPublicPayloadV1"
            || self.receipt_policy != "AllThreeCommitReceiptsV1"
        {
            return Err(BridgeError::InvalidProfile(
                "unsupported target bridge source trust contract".to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_target_profile(
        &self,
        target_profile: &NetworkProfile,
    ) -> Result<(), BridgeError> {
        self.validate()?;
        if self.ontology_package_id != target_profile.semantic.ontology_package_id
            || self.ontology_package_version != target_profile.semantic.ontology_package_version
            || self.ontology_package_hash != target_profile.semantic.ontology_package_hash
            || self.semantic_execution_profile_id
                != target_profile.semantic.semantic_execution_profile_id
        {
            return Err(BridgeError::InvalidProfile(
                "source and target semantic contracts are not identical".to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn target_ledger_instance_id(&self) -> Result<LedgerInstanceId32, BridgeError> {
        LedgerInstanceId32::new(hex32(
            &self.target_ledger_instance_id,
            "target ledger instance",
        )?)
    }

    pub(crate) fn canonical_config_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        self.validate()?;
        let mut receipt_nodes = Vec::with_capacity(REQUIRED_BRIDGE_RECEIPTS * 2);
        for node in &self.source_receipt_nodes {
            receipt_nodes.push(node.node_id.as_bytes().to_vec());
            receipt_nodes
                .push(hex32(&node.identity_public_key, "source Node Identity Key")?.to_vec());
        }
        let authority_keys = self
            .source_authority_keys
            .iter()
            .map(|key| Ok(hex32(key, "source authority key")?.to_vec()))
            .collect::<Result<Vec<_>, BridgeError>>()?;
        let mut bytes = b"PROVCHAIN_BRIDGE_SOURCE_TRUST_BINDING_V1".to_vec();
        for value in [
            hex32(&self.target_ledger_instance_id, "target ledger instance")?.to_vec(),
            self.source_network_id.as_bytes().to_vec(),
            hex32(&self.source_ledger_instance_id, "source ledger instance")?.to_vec(),
            self.source_profile_id.as_bytes().to_vec(),
            hex32(&self.source_profile_content_hash, "source profile hash")?.to_vec(),
            hex32(&self.source_governance_root, "source governance root")?.to_vec(),
            self.source_manifest_id.as_bytes().to_vec(),
            self.source_manifest_version.to_be_bytes().to_vec(),
            hex32(&self.source_manifest_digest, "source manifest digest")?.to_vec(),
            encode_sequence(&receipt_nodes)?,
            encode_sequence(&authority_keys)?,
            self.ontology_package_id.as_bytes().to_vec(),
            self.ontology_package_version.as_bytes().to_vec(),
            self.ontology_package_hash.as_bytes().to_vec(),
            self.semantic_execution_profile_id.as_bytes().to_vec(),
            self.state_commitment_scheme.as_bytes().to_vec(),
            self.source_admission_kind.as_bytes().to_vec(),
            self.bridge_suite.as_bytes().to_vec(),
            self.copy_mode.as_bytes().to_vec(),
            self.receipt_policy.as_bytes().to_vec(),
        ] {
            let len = u32::try_from(value.len()).map_err(|_| {
                BridgeError::InvalidProfile("target trust field exceeds u32".to_string())
            })?;
            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(&value);
        }
        Ok(bytes)
    }

    fn matches_source(
        &self,
        target_network_id: &str,
        source_profile: &NetworkProfile,
        source_manifest: &SignedMembershipManifest,
        source_governance_root: &VerifyingKey,
    ) -> Result<(), BridgeError> {
        let expected = Self::new(
            target_network_id,
            self.target_ledger_instance_id()?.bytes(),
            source_profile,
            source_manifest,
            source_governance_root,
        )?;
        if expected != *self {
            return Err(BridgeError::InvalidProfile(
                "activated source artifacts do not match target-pinned bridge trust".to_string(),
            ));
        }
        Ok(())
    }
}

/// Validated view over one exact canonical source Network Profile.
///
/// The source ledger identity and outbound rule are fields of that Network
/// Profile itself. Proof-carried bytes never activate this view; `ReferenceNode`
/// compares it with its independently activated context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSourceProfileV1 {
    network_profile_bytes: Vec<u8>,
    network_id: String,
    ledger_instance_id: LedgerInstanceId32,
    profile_id: String,
    manifest_id: String,
    manifest_version: u64,
    manifest_digest: [u8; 32],
    authority_keys: Vec<[u8; 32]>,
    ontology_package_id: String,
    ontology_package_version: String,
    ontology_package_hash: String,
    semantic_execution_profile_id: String,
    target: BridgeExportTargetV1,
}

impl BridgeSourceProfileV1 {
    /// Snapshot an exact already specified source profile and outbound rule.
    pub fn from_network_profile(
        profile: &NetworkProfile,
        manifest: &SignedMembershipManifest,
        ledger_instance_id: LedgerInstanceId32,
        target: BridgeExportTargetV1,
    ) -> Result<Self, BridgeError> {
        profile
            .validate()
            .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?;
        manifest
            .validate_structure()
            .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?;
        let binding = profile.membership.as_ref().ok_or_else(|| {
            BridgeError::InvalidProfile(
                "source bridge profile requires a Membership Manifest binding".to_string(),
            )
        })?;
        let bridge_binding = profile.bridge.as_ref().ok_or_else(|| {
            BridgeError::InvalidProfile(
                "source Network Profile has no outbound bridge rule".to_string(),
            )
        })?;
        let bound_source_ledger = LedgerInstanceId32::new(
            bridge_binding
                .source_ledger_instance_id()
                .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
        )?;
        let bound_target = BridgeExportTargetV1::new(
            bridge_binding.outbound_rule().target_network_id(),
            LedgerInstanceId32::new(
                bridge_binding
                    .outbound_rule()
                    .target_ledger_instance_id()
                    .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
            )?,
        )?;
        if bound_source_ledger != ledger_instance_id || bound_target != target {
            return Err(BridgeError::InvalidProfile(
                "requested ledger or target does not match the canonical Network Profile bridge rule"
                    .to_string(),
            ));
        }
        if manifest.manifest.network_id != profile.network_id
            || manifest.manifest.network_profile_id != profile.profile_id
            || manifest.manifest.manifest_id != binding.manifest_id
            || manifest.manifest.version != binding.manifest_version
            || manifest.digest() != decode_hash32(&binding.manifest_digest, "manifest digest")?
        {
            return Err(BridgeError::InvalidProfile(
                "source Membership Manifest does not match the Network Profile binding".to_string(),
            ));
        }
        let authority_set = validate_profile_authorities(profile, &manifest.manifest)
            .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?;
        if profile.consensus.consensus_type != "poa"
            || profile.consensus.authority_keys.len() != REQUIRED_BRIDGE_RECEIPTS
            || authority_set.len() != REQUIRED_BRIDGE_RECEIPTS
        {
            return Err(BridgeError::InvalidProfile(format!(
                "source bridge requires exactly {REQUIRED_BRIDGE_RECEIPTS} active PoA authorities"
            )));
        }
        let authority_keys = profile
            .consensus
            .authority_keys
            .iter()
            .map(|key| decode_hash32(key, "authority key"))
            .collect::<Result<Vec<_>, _>>()?;
        let expected_receipt_nodes = manifest
            .manifest
            .members
            .iter()
            .filter(|member| {
                member.status == MemberStatus::Active
                    && member.roles.contains(&MemberRole::Peer)
                    && member.roles.contains(&MemberRole::Validator)
                    && member
                        .validator_public_key
                        .is_some_and(|key| authority_set.contains(&key))
            })
            .count();
        if expected_receipt_nodes != REQUIRED_BRIDGE_RECEIPTS {
            return Err(BridgeError::InvalidProfile(format!(
                "source bridge requires exactly {REQUIRED_BRIDGE_RECEIPTS} pinned receipt nodes"
            )));
        }
        validate_identifier(&profile.network_id, "source network_id")?;
        validate_identifier(&profile.profile_id, "source profile_id")?;
        validate_identifier(&binding.manifest_id, "source manifest_id")?;
        validate_identifier(
            &profile.semantic.ontology_package_id,
            "source ontology_package_id",
        )?;
        validate_identifier(
            &profile.semantic.semantic_execution_profile_id,
            "source semantic_execution_profile_id",
        )?;
        let source = Self {
            network_profile_bytes: profile
                .canonical_bytes()
                .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
            network_id: profile.network_id.clone(),
            ledger_instance_id,
            profile_id: profile.profile_id.clone(),
            manifest_id: binding.manifest_id.clone(),
            manifest_version: binding.manifest_version,
            manifest_digest: manifest.digest(),
            authority_keys,
            ontology_package_id: profile.semantic.ontology_package_id.clone(),
            ontology_package_version: profile.semantic.ontology_package_version.clone(),
            ontology_package_hash: profile.semantic.ontology_package_hash.clone(),
            semantic_execution_profile_id: profile.semantic.semantic_execution_profile_id.clone(),
            target,
        };
        let source_profile_bytes = source.canonical_bytes()?;
        if source_profile_bytes.len() > MAX_BRIDGE_SOURCE_PROFILE_BYTES {
            return Err(BridgeError::Oversized {
                record: "source profile",
                bytes: source_profile_bytes.len(),
                limit: MAX_BRIDGE_SOURCE_PROFILE_BYTES,
            });
        }
        Ok(source)
    }

    /// Source network identifier.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Exact source ledger instance.
    pub fn ledger_instance_id(&self) -> LedgerInstanceId32 {
        self.ledger_instance_id
    }

    /// Source Network Profile identifier.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Exact profile-bound manifest digest.
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }

    /// Stable manifest lineage selected by the source profile.
    pub fn manifest_id(&self) -> &str {
        &self.manifest_id
    }

    /// Exact active manifest version selected by the source profile.
    pub fn manifest_version(&self) -> u64 {
        self.manifest_version
    }

    /// The only target supported by this bounded source-profile slice.
    pub fn target(&self) -> &BridgeExportTargetV1 {
        &self.target
    }

    /// Exact canonical source Network Profile bytes embedded in a proof bundle.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        Ok(self.network_profile_bytes.clone())
    }

    /// SHA-256 identity of the exact canonical profile artifact.
    pub fn content_hash(&self) -> Result<[u8; 32], BridgeError> {
        Ok(Sha256::digest(self.canonical_bytes()?).into())
    }

    /// Reject a configured source view that differs from the live governed artifacts.
    pub(crate) fn matches_active_context(
        &self,
        network: &NetworkProfile,
        manifest: &SignedMembershipManifest,
    ) -> Result<(), BridgeError> {
        let rebuilt = Self::from_network_profile(
            network,
            manifest,
            self.ledger_instance_id,
            self.target.clone(),
        )?;
        if rebuilt != *self {
            return Err(BridgeError::InvalidProfile(
                "source bridge profile does not match the independently activated context"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_source_envelope(
        &self,
        envelope: &AdmittedBlockEnvelope,
        manifest: &SignedMembershipManifest,
    ) -> Result<(), BridgeError> {
        if envelope.admission_kind != AdmissionKind::OrdinaryProvenanceV1
            || envelope.encrypted_payload.is_some()
            || envelope.privacy_control.is_some()
            || envelope.bridge_origin_evidence.is_some()
            || envelope.network_id != self.network_id
            || envelope.profile_id != self.profile_id
            || envelope.ontology_package_id != self.ontology_package_id
            || envelope.ontology_package_version != self.ontology_package_version
            || envelope.ontology_package_hash != self.ontology_package_hash
            || envelope.semantic_execution_profile_id != self.semantic_execution_profile_id
        {
            return Err(BridgeError::Rejected(
                "source envelope does not satisfy the exact ordinary semantic contract".to_string(),
            ));
        }
        let scheduled_index = (envelope.index % self.authority_keys.len() as u64) as usize;
        let scheduled_key = self.authority_keys[scheduled_index];
        if envelope.proposer_public_key != scheduled_key {
            return Err(BridgeError::Rejected(
                "source envelope was not signed by its scheduled PoA authority".to_string(),
            ));
        }
        let scheduled_member = manifest.manifest.members.iter().any(|member| {
            member.status == MemberStatus::Active
                && member.roles.contains(&MemberRole::Validator)
                && member.validator_public_key == Some(scheduled_key)
        });
        if !scheduled_member {
            return Err(BridgeError::Rejected(
                "scheduled source authority is not active in the pinned manifest".to_string(),
            ));
        }
        Ok(())
    }
}

/// The exact ten-field declaration core committed by source Final Admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExportCoreV1 {
    source_network_id: String,
    source_ledger_instance_id: LedgerInstanceId32,
    source_profile_id: String,
    source_profile_content_hash: [u8; 32],
    source_position: u64,
    target: BridgeExportTargetV1,
    public_payload_sha256: [u8; 32],
}

impl BridgeExportCoreV1 {
    /// Build the canonical declaration core from active source facts.
    pub fn new(
        source: &BridgeSourceProfileV1,
        source_position: u64,
        target: BridgeExportTargetV1,
        public_payload: &[u8],
    ) -> Result<Self, BridgeError> {
        if &target != source.target() {
            return Err(BridgeError::Rejected(
                "target is not the exact outbound profile binding".to_string(),
            ));
        }
        if public_payload.is_empty() || public_payload.len() > MAX_PAYLOAD_BYTES {
            return Err(BridgeError::Rejected(format!(
                "public payload must contain between 1 and {MAX_PAYLOAD_BYTES} bytes"
            )));
        }
        Ok(Self {
            source_network_id: source.network_id.clone(),
            source_ledger_instance_id: source.ledger_instance_id,
            source_profile_id: source.profile_id.clone(),
            source_profile_content_hash: source.content_hash()?,
            source_position,
            target,
            public_payload_sha256: Sha256::digest(public_payload).into(),
        })
    }

    /// Intended source ledger position.
    pub fn source_position(&self) -> u64 {
        self.source_position
    }

    /// Exact target network and ledger instance.
    pub fn target(&self) -> &BridgeExportTargetV1 {
        &self.target
    }

    /// Digest of the complete exact public payload bytes.
    pub fn public_payload_sha256(&self) -> [u8; 32] {
        self.public_payload_sha256
    }

    /// Exact source profile content hash.
    pub fn source_profile_content_hash(&self) -> [u8; 32] {
        self.source_profile_content_hash
    }

    /// Exact source ledger instance.
    pub fn source_ledger_instance_id(&self) -> LedgerInstanceId32 {
        self.source_ledger_instance_id
    }

    /// Canonical ten-field core bytes.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_EXPORT_CORE_TAG,
            &[
                self.source_network_id.as_bytes().to_vec(),
                self.source_ledger_instance_id.0.to_vec(),
                self.source_profile_id.as_bytes().to_vec(),
                self.source_profile_content_hash.to_vec(),
                self.source_position.to_be_bytes().to_vec(),
                self.target.network_id.as_bytes().to_vec(),
                self.target.ledger_instance_id.0.to_vec(),
                vec![ORDINARY_PROVENANCE_V1_TAG],
                vec![EXACT_PUBLIC_PAYLOAD_V1_TAG],
                self.public_payload_sha256.to_vec(),
            ],
        )
    }

    fn decode(bytes: &[u8]) -> Result<Self, BridgeError> {
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_EXPORT_CORE_TAG, 10)?;
        let source_network_id = decode_identifier(&fields[0], "source network_id")?;
        let source_ledger_instance_id =
            LedgerInstanceId32::new(array(&fields[1], "source ledger id")?)?;
        let source_profile_id = decode_identifier(&fields[2], "source profile_id")?;
        let source_profile_content_hash = nonzero_hash(&fields[3], "source profile hash")?;
        let source_position = decode_u64(&fields[4], "source position")?;
        let target_network_id = decode_identifier(&fields[5], "target network_id")?;
        let target_ledger_instance_id =
            LedgerInstanceId32::new(array(&fields[6], "target ledger id")?)?;
        if fields[7].as_slice() != [ORDINARY_PROVENANCE_V1_TAG] {
            return Err(BridgeError::Malformed(
                "source admission kind is not OrdinaryProvenanceV1".to_string(),
            ));
        }
        if fields[8].as_slice() != [EXACT_PUBLIC_PAYLOAD_V1_TAG] {
            return Err(BridgeError::Malformed(
                "copy mode is not ExactPublicPayloadV1".to_string(),
            ));
        }
        let public_payload_sha256 = nonzero_hash(&fields[9], "public payload digest")?;
        Ok(Self {
            source_network_id,
            source_ledger_instance_id,
            source_profile_id,
            source_profile_content_hash,
            source_position,
            target: BridgeExportTargetV1::new(target_network_id, target_ledger_instance_id)?,
            public_payload_sha256,
        })
    }

    /// Match every declared core fact to the active source and exact payload.
    pub(crate) fn validate_source(
        &self,
        source: &BridgeSourceProfileV1,
        position: u64,
        public_payload: &[u8],
    ) -> Result<(), BridgeError> {
        let expected = Self::new(source, position, self.target.clone(), public_payload)?;
        if expected != *self {
            return Err(BridgeError::Rejected(
                "bridge declaration does not match the exact source profile, position, target, or payload"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Canonical source export declaration and derived transfer identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExportDeclarationV1 {
    core: BridgeExportCoreV1,
    transfer_id: [u8; 32],
}

impl BridgeExportDeclarationV1 {
    /// Construct a declaration whose transfer ID is derived, never supplied.
    pub fn new(core: BridgeExportCoreV1) -> Result<Self, BridgeError> {
        let transfer_id = domain_hash(TRANSFER_ID_DOMAIN, &core.canonical_bytes()?);
        Ok(Self { core, transfer_id })
    }

    /// Decode one strict canonical declaration.
    pub fn decode(bytes: &[u8]) -> Result<Self, BridgeError> {
        if bytes.len() > MAX_BRIDGE_EXPORT_DECLARATION_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge export declaration",
                bytes: bytes.len(),
                limit: MAX_BRIDGE_EXPORT_DECLARATION_BYTES,
            });
        }
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_EXPORT_DECLARATION_TAG, 2)?;
        let core = BridgeExportCoreV1::decode(&fields[0])?;
        let transfer_id = nonzero_hash(&fields[1], "transfer id")?;
        let declaration = Self::new(core)?;
        if declaration.transfer_id != transfer_id || declaration.canonical_bytes()? != bytes {
            return Err(BridgeError::NonCanonical);
        }
        Ok(declaration)
    }

    /// Exact canonical declaration core.
    pub fn core(&self) -> &BridgeExportCoreV1 {
        &self.core
    }

    /// Derived deterministic Bridge Transfer ID.
    pub fn transfer_id(&self) -> [u8; 32] {
        self.transfer_id
    }

    /// Exact canonical declaration bytes committed in the source envelope.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_EXPORT_DECLARATION_TAG,
            &[self.core.canonical_bytes()?, self.transfer_id.to_vec()],
        )
    }

    /// Match the complete declaration and derived transfer ID to active source facts.
    pub(crate) fn validate_source(
        &self,
        source: &BridgeSourceProfileV1,
        position: u64,
        public_payload: &[u8],
    ) -> Result<(), BridgeError> {
        self.core
            .validate_source(source, position, public_payload)?;
        if domain_hash(TRANSFER_ID_DOMAIN, &self.core.canonical_bytes()?) != self.transfer_id {
            return Err(BridgeError::Rejected(
                "bridge transfer ID is not canonical for the declaration core".to_string(),
            ));
        }
        Ok(())
    }
}

/// Exact unsigned facts attested by one source convergence node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommitReceiptCoreV1 {
    network_id: String,
    ledger_instance_id: LedgerInstanceId32,
    profile_id: String,
    profile_content_hash: [u8; 32],
    manifest_id: String,
    manifest_version: u64,
    manifest_digest: [u8; 32],
    node_id: String,
    ledger_position: u64,
    envelope_hash: LedgerHash,
    ledger_prefix_hash: LedgerHash,
}

impl BridgeCommitReceiptCoreV1 {
    fn new(
        source: &BridgeSourceProfileV1,
        manifest: &SignedMembershipManifest,
        node_id: Uuid,
        ledger_position: u64,
        envelope_hash: LedgerHash,
        ledger_prefix_hash: LedgerHash,
    ) -> Result<Self, BridgeError> {
        validate_manifest_binding(source, manifest)?;
        let node_id = node_id.to_string();
        validate_identifier(&node_id, "receipt node_id")?;
        if envelope_hash == [0; 32] || ledger_prefix_hash == [0; 32] {
            return Err(BridgeError::Malformed(
                "receipt envelope and prefix hashes must be nonzero".to_string(),
            ));
        }
        Ok(Self {
            network_id: source.network_id.clone(),
            ledger_instance_id: source.ledger_instance_id,
            profile_id: source.profile_id.clone(),
            profile_content_hash: source.content_hash()?,
            manifest_id: source.manifest_id.clone(),
            manifest_version: source.manifest_version,
            manifest_digest: source.manifest_digest,
            node_id,
            ledger_position,
            envelope_hash,
            ledger_prefix_hash,
        })
    }

    fn decode(bytes: &[u8]) -> Result<Self, BridgeError> {
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_COMMIT_RECEIPT_CORE_TAG, 11)?;
        let core = Self {
            network_id: decode_identifier(&fields[0], "receipt network_id")?,
            ledger_instance_id: LedgerInstanceId32::new(array(
                &fields[1],
                "receipt ledger instance id",
            )?)?,
            profile_id: decode_identifier(&fields[2], "receipt profile_id")?,
            profile_content_hash: nonzero_hash(&fields[3], "receipt profile hash")?,
            manifest_id: decode_identifier(&fields[4], "receipt manifest_id")?,
            manifest_version: decode_nonzero_u64(&fields[5], "receipt manifest version")?,
            manifest_digest: nonzero_hash(&fields[6], "receipt manifest hash")?,
            node_id: decode_identifier(&fields[7], "receipt node_id")?,
            ledger_position: decode_u64(&fields[8], "receipt ledger position")?,
            envelope_hash: nonzero_hash(&fields[9], "receipt envelope hash")?,
            ledger_prefix_hash: nonzero_hash(&fields[10], "receipt prefix hash")?,
        };
        Uuid::parse_str(&core.node_id).map_err(|_| {
            BridgeError::Malformed("receipt node_id is not a canonical UUID".to_string())
        })?;
        if core.canonical_bytes()? != bytes {
            return Err(BridgeError::NonCanonical);
        }
        Ok(core)
    }

    /// Logical source-node identity represented as canonical lowercase UUID text.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Exact source ledger position attested by this node.
    pub fn ledger_position(&self) -> u64 {
        self.ledger_position
    }

    /// Exact committed source envelope identity.
    pub fn envelope_hash(&self) -> LedgerHash {
        self.envelope_hash
    }

    /// Exact ordered source ledger prefix ending at this envelope.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.ledger_prefix_hash
    }

    /// Encode the exact eleven-field receipt core.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_COMMIT_RECEIPT_CORE_TAG,
            &[
                self.network_id.as_bytes().to_vec(),
                self.ledger_instance_id.0.to_vec(),
                self.profile_id.as_bytes().to_vec(),
                self.profile_content_hash.to_vec(),
                self.manifest_id.as_bytes().to_vec(),
                self.manifest_version.to_be_bytes().to_vec(),
                self.manifest_digest.to_vec(),
                self.node_id.as_bytes().to_vec(),
                self.ledger_position.to_be_bytes().to_vec(),
                self.envelope_hash.to_vec(),
                self.ledger_prefix_hash.to_vec(),
            ],
        )
    }
}

/// One deterministic StrictEd25519V1 source commit receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommitReceiptV1 {
    core: BridgeCommitReceiptCoreV1,
    signature: [u8; 64],
}

impl BridgeCommitReceiptV1 {
    /// Sign exact locally durable source facts with the pinned Node Identity Key.
    pub(crate) fn sign(
        source: &BridgeSourceProfileV1,
        manifest: &SignedMembershipManifest,
        node_id: Uuid,
        ledger_position: u64,
        envelope_hash: LedgerHash,
        ledger_prefix_hash: LedgerHash,
        identity_key: &SigningKey,
    ) -> Result<Self, BridgeError> {
        let identity_public_key = expected_receipt_key(source, manifest, node_id)?;
        if identity_key.verifying_key().to_bytes() != identity_public_key {
            return Err(BridgeError::Rejected(
                "local Node Identity Key does not match the pinned receipt node".to_string(),
            ));
        }
        let core = BridgeCommitReceiptCoreV1::new(
            source,
            manifest,
            node_id,
            ledger_position,
            envelope_hash,
            ledger_prefix_hash,
        )?;
        let message = domain_hash(RECEIPT_SIGNATURE_DOMAIN, &core.canonical_bytes()?);
        let signature = identity_key.sign(&message).to_bytes();
        Ok(Self { core, signature })
    }

    /// Decode one closed canonical receipt without trusting its signer claims.
    pub fn decode(bytes: &[u8]) -> Result<Self, BridgeError> {
        if bytes.len() > MAX_BRIDGE_COMMIT_RECEIPT_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge commit receipt",
                bytes: bytes.len(),
                limit: MAX_BRIDGE_COMMIT_RECEIPT_BYTES,
            });
        }
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_COMMIT_RECEIPT_TAG, 3)?;
        let core = BridgeCommitReceiptCoreV1::decode(&fields[0])?;
        if fields[1].as_slice() != [STRICT_ED25519_V1_TAG] {
            return Err(BridgeError::Malformed(
                "receipt signature scheme is not StrictEd25519V1".to_string(),
            ));
        }
        let signature = array(&fields[2], "receipt signature")?;
        if signature == [0; 64] {
            return Err(BridgeError::Malformed(
                "receipt signature must be nonzero".to_string(),
            ));
        }
        let receipt = Self { core, signature };
        if receipt.canonical_bytes()? != bytes {
            return Err(BridgeError::NonCanonical);
        }
        Ok(receipt)
    }

    /// Exact facts signed by the source node.
    pub fn core(&self) -> &BridgeCommitReceiptCoreV1 {
        &self.core
    }

    /// Raw deterministic StrictEd25519V1 signature bytes.
    pub fn signature(&self) -> [u8; 64] {
        self.signature
    }

    /// Encode this receipt as the closed three-field bridge record.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_COMMIT_RECEIPT_TAG,
            &[
                self.core.canonical_bytes()?,
                vec![STRICT_ED25519_V1_TAG],
                self.signature.to_vec(),
            ],
        )
    }

    fn verify(
        &self,
        source: &BridgeSourceProfileV1,
        manifest: &SignedMembershipManifest,
        ledger_position: u64,
        envelope_hash: LedgerHash,
        ledger_prefix_hash: LedgerHash,
    ) -> Result<(), BridgeError> {
        let node_id = Uuid::parse_str(&self.core.node_id).map_err(|_| {
            BridgeError::Malformed("receipt node_id is not a canonical UUID".to_string())
        })?;
        let expected = BridgeCommitReceiptCoreV1::new(
            source,
            manifest,
            node_id,
            ledger_position,
            envelope_hash,
            ledger_prefix_hash,
        )?;
        if expected != self.core {
            return Err(BridgeError::Rejected(
                "receipt facts do not match the exact source profile, manifest, envelope, or prefix"
                    .to_string(),
            ));
        }
        let public_key = expected_receipt_key(source, manifest, node_id)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| BridgeError::Malformed("invalid receipt public key".to_string()))?;
        let signature = Signature::from_bytes(&self.signature);
        let message = domain_hash(RECEIPT_SIGNATURE_DOMAIN, &self.core.canonical_bytes()?);
        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|_| {
                BridgeError::Rejected("StrictEd25519V1 receipt signature is invalid".to_string())
            })
    }
}

/// Exact source proof bytes that become exportable only after all three receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeProofBundleV1 {
    transfer_id: [u8; 32],
    source_profile_bytes: Vec<u8>,
    signed_manifest_bytes: Vec<u8>,
    source_envelope_bytes: Vec<u8>,
    receipts: Vec<BridgeCommitReceiptV1>,
}

impl BridgeProofBundleV1 {
    /// Assemble source evidence only after all exact profile, envelope, and receipt checks pass.
    pub(crate) fn new(
        source: &BridgeSourceProfileV1,
        manifest: &SignedMembershipManifest,
        source_envelope_bytes: Vec<u8>,
        ledger_prefix_hash: LedgerHash,
        receipts: Vec<BridgeCommitReceiptV1>,
    ) -> Result<Self, BridgeError> {
        validate_manifest_binding(source, manifest)?;
        if source_envelope_bytes.len() > MAX_BRIDGE_SOURCE_ENVELOPE_BYTES {
            return Err(BridgeError::Oversized {
                record: "source envelope",
                bytes: source_envelope_bytes.len(),
                limit: MAX_BRIDGE_SOURCE_ENVELOPE_BYTES,
            });
        }
        let envelope = AdmittedBlockEnvelope::decode(&source_envelope_bytes)
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        if envelope
            .canonical_bytes()
            .map_err(|error| BridgeError::Malformed(error.to_string()))?
            != source_envelope_bytes
        {
            return Err(BridgeError::NonCanonical);
        }
        source.validate_source_envelope(&envelope, manifest)?;
        let declaration_bytes = envelope
            .bridge_export_declaration
            .as_deref()
            .ok_or_else(|| {
                BridgeError::Rejected(
                    "source envelope has no bridge export declaration".to_string(),
                )
            })?;
        let declaration = BridgeExportDeclarationV1::decode(declaration_bytes)?;
        declaration.validate_source(source, envelope.index, &envelope.public_provenance)?;
        validate_receipt_set(
            source,
            manifest,
            envelope.index,
            envelope.envelope_hash,
            ledger_prefix_hash,
            &receipts,
        )?;
        let source_profile_bytes = source.canonical_bytes()?;
        let signed_manifest_bytes = manifest.canonical_bytes();
        if signed_manifest_bytes.len() > MAX_BRIDGE_SIGNED_MANIFEST_BYTES {
            return Err(BridgeError::Oversized {
                record: "signed manifest",
                bytes: signed_manifest_bytes.len(),
                limit: MAX_BRIDGE_SIGNED_MANIFEST_BYTES,
            });
        }
        let bundle = Self {
            transfer_id: declaration.transfer_id(),
            source_profile_bytes,
            signed_manifest_bytes,
            source_envelope_bytes,
            receipts,
        };
        let encoded_len = bundle.canonical_bytes()?.len();
        if encoded_len > MAX_BRIDGE_PROOF_BUNDLE_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge proof bundle",
                bytes: encoded_len,
                limit: MAX_BRIDGE_PROOF_BUNDLE_BYTES,
            });
        }
        Ok(bundle)
    }

    /// Decode and verify a proof against independently activated source trust.
    ///
    /// The proof-carried profile, manifest, and signer claims never create trust:
    /// they must be byte-identical to the caller's active source artifacts, and
    /// the manifest must verify under the caller's pinned Governance Trust Root.
    pub fn decode_verified(
        bytes: &[u8],
        source: &BridgeSourceProfileV1,
        manifest: &SignedMembershipManifest,
        governance_root: &VerifyingKey,
    ) -> Result<Self, BridgeError> {
        manifest.verify(governance_root).map_err(|_| {
            BridgeError::Rejected(
                "source Membership Manifest governance signature is invalid".to_string(),
            )
        })?;
        validate_manifest_binding(source, manifest)?;
        let bundle = Self::decode_structure(bytes)?;
        if bundle.source_profile_bytes != source.canonical_bytes()?
            || bundle.signed_manifest_bytes != manifest.canonical_bytes()
        {
            return Err(BridgeError::Rejected(
                "proof artifacts do not equal the independently activated source profile and manifest"
                    .to_string(),
            ));
        }
        let envelope = AdmittedBlockEnvelope::decode(&bundle.source_envelope_bytes)
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        source.validate_source_envelope(&envelope, manifest)?;
        let declaration = BridgeExportDeclarationV1::decode(
            envelope
                .bridge_export_declaration
                .as_deref()
                .ok_or_else(|| {
                    BridgeError::Malformed("proof envelope has no export declaration".to_string())
                })?,
        )?;
        declaration.validate_source(source, envelope.index, &envelope.public_provenance)?;
        let ledger_prefix_hash = bundle
            .receipts
            .first()
            .ok_or_else(|| BridgeError::Malformed("proof has no receipts".to_string()))?
            .core
            .ledger_prefix_hash;
        if envelope.index != 0 {
            return Err(BridgeError::Rejected(
                "five-field bridge proof cannot establish a non-genesis source parent Ledger Prefix"
                    .to_string(),
            ));
        }
        let expected_ledger_prefix_hash = crate::network::convergence::extend_ledger_prefix_hash(
            crate::network::convergence::genesis_ledger_prefix_hash(),
            envelope.index,
            &bundle.source_envelope_bytes,
        );
        if ledger_prefix_hash != expected_ledger_prefix_hash {
            return Err(BridgeError::Rejected(
                "source receipt prefix does not equal the independently recomputed source prefix transition"
                    .to_string(),
            ));
        }
        validate_receipt_set(
            source,
            manifest,
            envelope.index,
            envelope.envelope_hash,
            ledger_prefix_hash,
            &bundle.receipts,
        )?;
        Ok(bundle)
    }

    fn decode_structure(bytes: &[u8]) -> Result<Self, BridgeError> {
        if bytes.len() > MAX_BRIDGE_PROOF_BUNDLE_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge proof bundle",
                bytes: bytes.len(),
                limit: MAX_BRIDGE_PROOF_BUNDLE_BYTES,
            });
        }
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_PROOF_BUNDLE_TAG, 5)?;
        let transfer_id = nonzero_hash(&fields[0], "proof transfer id")?;
        if fields[1].is_empty() || fields[1].len() > MAX_BRIDGE_SOURCE_PROFILE_BYTES {
            return Err(BridgeError::Malformed(
                "invalid source profile artifact size".to_string(),
            ));
        }
        if fields[2].is_empty() || fields[2].len() > MAX_BRIDGE_SIGNED_MANIFEST_BYTES {
            return Err(BridgeError::Malformed(
                "invalid signed manifest artifact size".to_string(),
            ));
        }
        if fields[3].is_empty() || fields[3].len() > MAX_BRIDGE_SOURCE_ENVELOPE_BYTES {
            return Err(BridgeError::Malformed(
                "invalid source envelope artifact size".to_string(),
            ));
        }
        let envelope = AdmittedBlockEnvelope::decode(&fields[3])
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        let declaration = BridgeExportDeclarationV1::decode(
            envelope
                .bridge_export_declaration
                .as_deref()
                .ok_or_else(|| {
                    BridgeError::Malformed("proof envelope has no export declaration".to_string())
                })?,
        )?;
        if declaration.transfer_id() != transfer_id {
            return Err(BridgeError::Rejected(
                "proof transfer ID does not match its source declaration".to_string(),
            ));
        }
        let source_profile_hash: [u8; 32] = Sha256::digest(&fields[1]).into();
        if source_profile_hash != declaration.core.source_profile_content_hash {
            return Err(BridgeError::Rejected(
                "source profile artifact does not match the export declaration".to_string(),
            ));
        }
        let receipts = decode_receipt_sequence(&fields[4])?;
        ensure_receipts_sorted(&receipts)?;
        validate_structural_receipt_agreement(&declaration, &envelope, &receipts)?;
        let bundle = Self {
            transfer_id,
            source_profile_bytes: fields[1].clone(),
            signed_manifest_bytes: fields[2].clone(),
            source_envelope_bytes: fields[3].clone(),
            receipts,
        };
        if bundle.canonical_bytes()? != bytes {
            return Err(BridgeError::NonCanonical);
        }
        Ok(bundle)
    }

    /// Derived transfer identity carried by this proof.
    pub fn transfer_id(&self) -> [u8; 32] {
        self.transfer_id
    }

    /// Exact canonical source Network Profile bytes.
    pub fn source_profile_bytes(&self) -> &[u8] {
        &self.source_profile_bytes
    }

    /// Exact canonical signed Membership Manifest bytes.
    pub fn signed_manifest_bytes(&self) -> &[u8] {
        &self.signed_manifest_bytes
    }

    /// Exact declaration-bearing source Admitted Block Envelope bytes.
    pub fn source_envelope_bytes(&self) -> &[u8] {
        &self.source_envelope_bytes
    }

    /// Exactly three receipts in canonical node-identity order.
    pub fn receipts(&self) -> &[BridgeCommitReceiptV1] {
        &self.receipts
    }

    /// Encode the exact five-field proof bundle.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_PROOF_BUNDLE_TAG,
            &[
                self.transfer_id.to_vec(),
                self.source_profile_bytes.clone(),
                self.signed_manifest_bytes.clone(),
                self.source_envelope_bytes.clone(),
                encode_receipt_sequence(&self.receipts)?,
            ],
        )
    }

    fn inspect_transfer(bytes: &[u8]) -> Result<([u8; 32], [u8; 32]), BridgeError> {
        if bytes.len() > MAX_BRIDGE_PROOF_BUNDLE_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge proof bundle",
                bytes: bytes.len(),
                limit: MAX_BRIDGE_PROOF_BUNDLE_BYTES,
            });
        }
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_PROOF_BUNDLE_TAG, 5)?;
        Ok((
            nonzero_hash(&fields[0], "proof transfer id")?,
            Sha256::digest(bytes).into(),
        ))
    }
}

/// Independently activated target verifier for one exact source trust contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTargetProfileV1 {
    target_network_id: String,
    target_profile_id: String,
    target_profile_hash: [u8; 32],
    target_ledger_instance_id: LedgerInstanceId32,
    source: BridgeSourceProfileV1,
    source_manifest: SignedMembershipManifest,
    source_governance_root: VerifyingKey,
}

impl BridgeTargetProfileV1 {
    /// Activate proof verification from target-pinned trust and separately supplied source artifacts.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` when either profile fails validation, the target
    /// profile pins no source trust, the supplied source artifacts disagree
    /// with the pinned trust, or the source profile carries no bridge rule.
    pub fn activate(
        target_profile: &NetworkProfile,
        source_profile: &NetworkProfile,
        source_manifest: &SignedMembershipManifest,
        source_governance_root: VerifyingKey,
    ) -> Result<Self, BridgeError> {
        target_profile
            .validate()
            .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?;
        let trust = target_profile.bridge_source_trust.as_ref().ok_or_else(|| {
            BridgeError::InvalidProfile(
                "target Network Profile has no pinned source bridge trust".to_string(),
            )
        })?;
        trust.validate_target_profile(target_profile)?;
        trust.matches_source(
            &target_profile.network_id,
            source_profile,
            source_manifest,
            &source_governance_root,
        )?;
        let source_bridge = source_profile.bridge.as_ref().ok_or_else(|| {
            BridgeError::InvalidProfile("source profile has no bridge rule".to_string())
        })?;
        let source = BridgeSourceProfileV1::from_network_profile(
            source_profile,
            source_manifest,
            LedgerInstanceId32::new(
                source_bridge
                    .source_ledger_instance_id()
                    .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
            )?,
            BridgeExportTargetV1::new(
                &target_profile.network_id,
                trust.target_ledger_instance_id()?,
            )?,
        )?;
        Ok(Self {
            target_network_id: target_profile.network_id.clone(),
            target_profile_id: target_profile.profile_id.clone(),
            target_profile_hash: Sha256::digest(
                target_profile
                    .canonical_bytes()
                    .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
            )
            .into(),
            target_ledger_instance_id: trust.target_ledger_instance_id()?,
            source,
            source_manifest: source_manifest.clone(),
            source_governance_root,
        })
    }

    /// Whether this activated target contract matches the given target
    /// network/profile identity and semantic-package identity fields.
    pub(crate) fn matches_target(
        &self,
        network_id: &str,
        profile_id: &str,
        ontology_package_id: &str,
        ontology_package_version: &str,
        ontology_package_hash: &str,
        semantic_execution_profile_id: &str,
    ) -> bool {
        self.target_network_id == network_id
            && self.target_profile_id == profile_id
            && self.source.ontology_package_id == ontology_package_id
            && self.source.ontology_package_version == ontology_package_version
            && self.source.ontology_package_hash == ontology_package_hash
            && self.source.semantic_execution_profile_id == semantic_execution_profile_id
    }

    pub(crate) fn matches_network_profile(
        &self,
        target_profile: &NetworkProfile,
    ) -> Result<bool, BridgeError> {
        let target_profile_hash: [u8; 32] = Sha256::digest(
            target_profile
                .canonical_bytes()
                .map_err(|error| BridgeError::InvalidProfile(error.to_string()))?,
        )
        .into();
        Ok(self.target_network_id == target_profile.network_id
            && self.target_profile_id == target_profile.profile_id
            && self.target_profile_hash == target_profile_hash)
    }

    /// Exact target ledger instance selected by the governed source rule.
    pub fn target_ledger_instance_id(&self) -> LedgerInstanceId32 {
        self.target_ledger_instance_id
    }

    fn verify_import(
        &self,
        proof_bytes: &[u8],
        public_payload: &[u8],
    ) -> Result<VerifiedBridgeImportV1, BridgeError> {
        let proof = BridgeProofBundleV1::decode_verified(
            proof_bytes,
            &self.source,
            &self.source_manifest,
            &self.source_governance_root,
        )?;
        let source_envelope = AdmittedBlockEnvelope::decode(proof.source_envelope_bytes())
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        if public_payload != source_envelope.public_provenance {
            return Err(BridgeError::Rejected(
                "target payload is not the exact complete source public payload".to_string(),
            ));
        }
        let declaration = BridgeExportDeclarationV1::decode(
            source_envelope
                .bridge_export_declaration
                .as_deref()
                .ok_or_else(|| BridgeError::Malformed("missing source declaration".to_string()))?,
        )?;
        if declaration.core.target.network_id != self.target_network_id
            || declaration.core.target.ledger_instance_id != self.target_ledger_instance_id
        {
            return Err(BridgeError::Rejected(
                "proof does not name this exact target ledger instance".to_string(),
            ));
        }
        let source_ledger_prefix_hash = proof
            .receipts()
            .first()
            .ok_or_else(|| BridgeError::Malformed("proof has no source receipts".to_string()))?
            .core()
            .ledger_prefix_hash();
        let origin = BridgeOriginEvidenceV1::new(proof.transfer_id(), proof_bytes.to_vec())?;
        Ok(VerifiedBridgeImportV1 {
            transfer_id: proof.transfer_id(),
            proof_hash: Sha256::digest(proof_bytes).into(),
            payload_hash: Sha256::digest(public_payload).into(),
            source_network_id: self.source.network_id.clone(),
            source_ledger_instance_id: self.source.ledger_instance_id.bytes(),
            source_position: source_envelope.index,
            source_envelope_hash: source_envelope.envelope_hash,
            source_ledger_prefix_hash,
            target_network_id: self.target_network_id.clone(),
            target_ledger_instance_id: self.target_ledger_instance_id.bytes(),
            origin_bytes: origin.canonical_bytes()?,
        })
    }

    /// Re-verify committed origin evidence against the pinned contract.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` when the evidence bytes fail to decode, the
    /// embedded proof fails closed verification, or the evidence identity
    /// differs from the verified proof identity.
    pub(crate) fn verify_origin(
        &self,
        origin_bytes: &[u8],
        public_payload: &[u8],
    ) -> Result<VerifiedBridgeImportV1, BridgeError> {
        let origin = BridgeOriginEvidenceV1::decode(origin_bytes)?;
        let verified = self.verify_import(&origin.proof_bytes, public_payload)?;
        if verified.transfer_id != origin.transfer_id || verified.proof_hash != origin.proof_sha256
        {
            return Err(BridgeError::Rejected(
                "origin evidence does not equal the verified proof identity".to_string(),
            ));
        }
        Ok(verified)
    }
}

/// Exact three-field bridge origin evidence committed by target Final Admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOriginEvidenceV1 {
    transfer_id: [u8; 32],
    proof_sha256: [u8; 32],
    proof_bytes: Vec<u8>,
}

impl BridgeOriginEvidenceV1 {
    fn new(transfer_id: [u8; 32], proof_bytes: Vec<u8>) -> Result<Self, BridgeError> {
        nonzero_hash(&transfer_id, "origin transfer id")?;
        if proof_bytes.is_empty() || proof_bytes.len() > MAX_BRIDGE_PROOF_BUNDLE_BYTES {
            return Err(BridgeError::Malformed(
                "invalid origin proof size".to_string(),
            ));
        }
        let origin = Self {
            transfer_id,
            proof_sha256: Sha256::digest(&proof_bytes).into(),
            proof_bytes,
        };
        if origin.canonical_bytes()?.len() > MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge origin evidence",
                bytes: origin.canonical_bytes()?.len(),
                limit: MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
            });
        }
        Ok(origin)
    }

    /// Decode the canonical origin evidence committed in a target envelope.
    pub fn decode(bytes: &[u8]) -> Result<Self, BridgeError> {
        if bytes.len() > MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES {
            return Err(BridgeError::Oversized {
                record: "bridge origin evidence",
                bytes: bytes.len(),
                limit: MAX_BRIDGE_ORIGIN_EVIDENCE_BYTES,
            });
        }
        let fields = decode_fields(bytes, BRIDGE_MAGIC, BRIDGE_ORIGIN_EVIDENCE_TAG, 3)?;
        let origin = Self {
            transfer_id: nonzero_hash(&fields[0], "origin transfer id")?,
            proof_sha256: nonzero_hash(&fields[1], "origin proof hash")?,
            proof_bytes: fields[2].clone(),
        };
        if Sha256::digest(&origin.proof_bytes).as_slice() != origin.proof_sha256
            || BridgeProofBundleV1::inspect_transfer(&origin.proof_bytes)?.0 != origin.transfer_id
            || origin.canonical_bytes()? != bytes
        {
            return Err(BridgeError::NonCanonical);
        }
        Ok(origin)
    }

    /// Transfer identity carried unchanged from the verified proof.
    pub fn transfer_id(&self) -> [u8; 32] {
        self.transfer_id
    }

    /// SHA-256 identity of the complete embedded proof bytes.
    pub fn proof_sha256(&self) -> [u8; 32] {
        self.proof_sha256
    }

    /// Complete exact source proof retained as target origin evidence.
    pub fn proof_bytes(&self) -> &[u8] {
        &self.proof_bytes
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, BridgeError> {
        encode_fields(
            BRIDGE_MAGIC,
            BRIDGE_ORIGIN_EVIDENCE_TAG,
            &[
                self.transfer_id.to_vec(),
                self.proof_sha256.to_vec(),
                self.proof_bytes.clone(),
            ],
        )
    }
}

/// One fully verified import candidate produced by strict proof verification.
///
/// Carries the exact identities bound by the verified five-field proof bundle;
/// used to construct the committed origin evidence and journal projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedBridgeImportV1 {
    pub(crate) transfer_id: [u8; 32],
    pub(crate) proof_hash: [u8; 32],
    pub(crate) payload_hash: [u8; 32],
    pub(crate) source_network_id: String,
    pub(crate) source_ledger_instance_id: [u8; 32],
    pub(crate) source_position: u64,
    pub(crate) source_envelope_hash: [u8; 32],
    pub(crate) source_ledger_prefix_hash: [u8; 32],
    pub(crate) target_network_id: String,
    pub(crate) target_ledger_instance_id: [u8; 32],
    pub(crate) origin_bytes: Vec<u8>,
}

/// Durable target reference returned for a committed bridge import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeImportReferenceV1 {
    transfer_id: [u8; 32],
    proof_hash: [u8; 32],
    payload_hash: [u8; 32],
    source_network_id: String,
    source_ledger_instance_id: [u8; 32],
    source_position: u64,
    source_envelope_hash: [u8; 32],
    source_ledger_prefix_hash: [u8; 32],
    target_network_id: String,
    target_ledger_instance_id: [u8; 32],
    target_position: u64,
    target_envelope_hash: [u8; 32],
    target_ledger_prefix_hash: [u8; 32],
}

impl BridgeImportReferenceV1 {
    /// Source-derived transfer identity.
    pub fn transfer_id(&self) -> [u8; 32] {
        self.transfer_id
    }

    /// SHA-256 identity of the exact source proof.
    pub fn proof_hash(&self) -> [u8; 32] {
        self.proof_hash
    }

    /// SHA-256 identity of the exact copied public payload.
    pub fn payload_hash(&self) -> [u8; 32] {
        self.payload_hash
    }

    /// Source network named by the verified declaration.
    pub fn source_network_id(&self) -> &str {
        &self.source_network_id
    }

    /// Source ledger instance named by the verified declaration.
    pub fn source_ledger_instance_id(&self) -> [u8; 32] {
        self.source_ledger_instance_id
    }

    /// Source journal position named by the verified declaration and receipts.
    pub fn source_position(&self) -> u64 {
        self.source_position
    }

    /// Authenticated source envelope identity.
    pub fn source_envelope_hash(&self) -> [u8; 32] {
        self.source_envelope_hash
    }

    /// Authenticated source ledger-prefix identity attested by all three receipts.
    pub fn source_ledger_prefix_hash(&self) -> [u8; 32] {
        self.source_ledger_prefix_hash
    }

    /// Target network that committed the imported envelope.
    pub fn target_network_id(&self) -> &str {
        &self.target_network_id
    }

    /// Exact target ledger instance that accepted the import.
    pub fn target_ledger_instance_id(&self) -> [u8; 32] {
        self.target_ledger_instance_id
    }

    /// Position of the target envelope derived from the authoritative journal.
    pub fn target_position(&self) -> u64 {
        self.target_position
    }

    /// Authenticated target envelope identity.
    pub fn target_envelope_hash(&self) -> [u8; 32] {
        self.target_envelope_hash
    }

    /// Ordered target-ledger prefix ending at the imported envelope.
    pub fn target_ledger_prefix_hash(&self) -> [u8; 32] {
        self.target_ledger_prefix_hash
    }
}

/// Journal-derived externally observable outcome for a bridge import attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeImportOutcomeV1 {
    /// The proof passed target consensus and Final Admission; this transfer ID
    /// now names one committed target envelope.
    Imported {
        /// Exact target-journal reference reconstructed from Effective Bridge State.
        reference: BridgeImportReferenceV1,
    },
    /// Exact proof, payload, target, and reference were already committed.
    AlreadyImported {
        /// Existing target-journal reference for this exact proof and payload.
        reference: BridgeImportReferenceV1,
    },
    /// The transfer ID already names different evidence, payload, target, or reference.
    ReplayConflict,
}

/// Journal-replay classification of one import attempt before any append.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BridgeImportPreflightV1 {
    /// Verified candidate whose transfer ID has no prior journal usage.
    Ready(VerifiedBridgeImportV1),
    /// This exact proof, payload, and target are already committed.
    AlreadyImported(BridgeImportReferenceV1),
    /// The transfer ID already names different evidence, payload, or target.
    ReplayConflict,
}

/// Effective Bridge State rebuilt only from verified target journal envelopes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectiveBridgeStateV1 {
    imports: BTreeMap<[u8; 32], BridgeImportReferenceV1>,
}

impl EffectiveBridgeStateV1 {
    /// Number of distinct committed transfer IDs.
    pub fn len(&self) -> usize {
        self.imports.len()
    }

    /// Whether no bridge import has been committed.
    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }

    /// Resolve one committed transfer reference by ID.
    pub fn get(&self, transfer_id: &[u8; 32]) -> Option<&BridgeImportReferenceV1> {
        self.imports.get(transfer_id)
    }

    /// Classify one import attempt against journal-replayed state.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` when the proof cannot be inspected or a ready
    /// candidate fails closed verification.
    pub(crate) fn classify(
        &self,
        target: &BridgeTargetProfileV1,
        proof_bytes: &[u8],
        public_payload: &[u8],
    ) -> Result<BridgeImportPreflightV1, BridgeError> {
        let (transfer_id, proof_hash) = BridgeProofBundleV1::inspect_transfer(proof_bytes)?;
        let payload_hash: [u8; 32] = Sha256::digest(public_payload).into();
        if let Some(existing) = self.imports.get(&transfer_id) {
            return if existing.proof_hash == proof_hash
                && existing.payload_hash == payload_hash
                && existing.target_ledger_instance_id == target.target_ledger_instance_id.bytes()
            {
                Ok(BridgeImportPreflightV1::AlreadyImported(existing.clone()))
            } else {
                Ok(BridgeImportPreflightV1::ReplayConflict)
            };
        }
        Ok(BridgeImportPreflightV1::Ready(
            target.verify_import(proof_bytes, public_payload)?,
        ))
    }

    /// Record one committed import into projected state.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` when the journal already holds this transfer ID.
    pub(crate) fn insert_verified(
        &mut self,
        verified: VerifiedBridgeImportV1,
        target_position: u64,
        target_envelope_hash: [u8; 32],
        target_ledger_prefix_hash: [u8; 32],
    ) -> Result<BridgeImportReferenceV1, BridgeError> {
        if self.imports.contains_key(&verified.transfer_id) {
            return Err(BridgeError::Rejected(
                "target journal contains a duplicate bridge transfer ID".to_string(),
            ));
        }
        let reference = BridgeImportReferenceV1 {
            transfer_id: verified.transfer_id,
            proof_hash: verified.proof_hash,
            payload_hash: verified.payload_hash,
            source_network_id: verified.source_network_id,
            source_ledger_instance_id: verified.source_ledger_instance_id,
            source_position: verified.source_position,
            source_envelope_hash: verified.source_envelope_hash,
            source_ledger_prefix_hash: verified.source_ledger_prefix_hash,
            target_network_id: verified.target_network_id,
            target_ledger_instance_id: verified.target_ledger_instance_id,
            target_position,
            target_envelope_hash,
            target_ledger_prefix_hash,
        };
        self.imports
            .insert(reference.transfer_id, reference.clone());
        Ok(reference)
    }
}

/// Reproducible, source-only export product; it grants no target admission authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExportArtifactV1 {
    declaration_bytes: Vec<u8>,
    public_payload: Vec<u8>,
    proof_bundle: BridgeProofBundleV1,
    reproducibility: BridgeExportReproducibilityV1,
}

impl BridgeExportArtifactV1 {
    /// Materialize the exact source export and its derived reproducibility facts.
    pub(crate) fn new(proof_bundle: BridgeProofBundleV1) -> Result<Self, BridgeError> {
        let envelope = AdmittedBlockEnvelope::decode(proof_bundle.source_envelope_bytes())
            .map_err(|error| BridgeError::Malformed(error.to_string()))?;
        let declaration_bytes = envelope.bridge_export_declaration.clone().ok_or_else(|| {
            BridgeError::Malformed("proof envelope has no export declaration".to_string())
        })?;
        let declaration = BridgeExportDeclarationV1::decode(&declaration_bytes)?;
        let receipt = proof_bundle
            .receipts
            .first()
            .ok_or_else(|| BridgeError::Malformed("proof has no receipts".to_string()))?;
        let reproducibility = BridgeExportReproducibilityV1 {
            transfer_id: declaration.transfer_id(),
            source_position: envelope.index,
            declaration_sha256: Sha256::digest(&declaration_bytes).into(),
            public_payload_sha256: Sha256::digest(&envelope.public_provenance).into(),
            source_profile_sha256: Sha256::digest(proof_bundle.source_profile_bytes()).into(),
            signed_manifest_sha256: Sha256::digest(proof_bundle.signed_manifest_bytes()).into(),
            source_envelope_sha256: Sha256::digest(proof_bundle.source_envelope_bytes()).into(),
            source_envelope_hash: envelope.envelope_hash,
            ledger_prefix_hash: receipt.core.ledger_prefix_hash,
            manifest_digest: receipt.core.manifest_digest,
            proof_bundle_sha256: Sha256::digest(proof_bundle.canonical_bytes()?).into(),
        };
        Ok(Self {
            declaration_bytes,
            public_payload: envelope.public_provenance,
            proof_bundle,
            reproducibility,
        })
    }

    /// Exact declaration bytes already committed by source Final Admission.
    pub fn declaration_bytes(&self) -> &[u8] {
        &self.declaration_bytes
    }

    /// Exact complete public payload; no mapping, subset, or reformatting is allowed.
    pub fn public_payload(&self) -> &[u8] {
        &self.public_payload
    }

    /// Complete canonical source proof.
    pub fn proof_bundle(&self) -> &BridgeProofBundleV1 {
        &self.proof_bundle
    }

    /// Immutable hashes and source position needed to reproduce and audit this export.
    pub fn reproducibility(&self) -> &BridgeExportReproducibilityV1 {
        &self.reproducibility
    }

    /// Reject any requested export bytes that are not the complete committed payload.
    pub fn require_exact_payload(&self, requested: &[u8]) -> Result<(), BridgeError> {
        if requested != self.public_payload {
            return Err(BridgeError::Rejected(
                "changed, mapped, redacted, subset, or reformatted payload is forbidden"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Derived, immutable metadata for reproducing one exact source export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeExportReproducibilityV1 {
    transfer_id: [u8; 32],
    source_position: u64,
    declaration_sha256: [u8; 32],
    public_payload_sha256: [u8; 32],
    source_profile_sha256: [u8; 32],
    signed_manifest_sha256: [u8; 32],
    source_envelope_sha256: [u8; 32],
    source_envelope_hash: [u8; 32],
    ledger_prefix_hash: [u8; 32],
    manifest_digest: [u8; 32],
    proof_bundle_sha256: [u8; 32],
}

impl BridgeExportReproducibilityV1 {
    /// Transfer identity derived from the exact declaration core.
    pub fn transfer_id(&self) -> [u8; 32] {
        self.transfer_id
    }

    /// Exact source ledger position.
    pub fn source_position(&self) -> u64 {
        self.source_position
    }

    /// SHA-256 of the exact committed declaration bytes.
    pub fn declaration_sha256(&self) -> [u8; 32] {
        self.declaration_sha256
    }

    /// SHA-256 of the exact complete public payload bytes.
    pub fn public_payload_sha256(&self) -> [u8; 32] {
        self.public_payload_sha256
    }

    /// Canonical source Network Profile content hash.
    pub fn source_profile_sha256(&self) -> [u8; 32] {
        self.source_profile_sha256
    }

    /// SHA-256 of the complete signed Membership Manifest artifact.
    pub fn signed_manifest_sha256(&self) -> [u8; 32] {
        self.signed_manifest_sha256
    }

    /// Plain SHA-256 of the exact source envelope artifact.
    pub fn source_envelope_sha256(&self) -> [u8; 32] {
        self.source_envelope_sha256
    }

    /// Domain-separated committed source Envelope Hash.
    pub fn source_envelope_hash(&self) -> [u8; 32] {
        self.source_envelope_hash
    }

    /// Domain-separated source Ledger Prefix Hash attested by all receipts.
    pub fn ledger_prefix_hash(&self) -> [u8; 32] {
        self.ledger_prefix_hash
    }

    /// Domain-separated canonical Membership Manifest digest.
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }

    /// SHA-256 of the complete canonical proof-bundle bytes.
    pub fn proof_bundle_sha256(&self) -> [u8; 32] {
        self.proof_bundle_sha256
    }
}

fn validate_manifest_binding(
    source: &BridgeSourceProfileV1,
    manifest: &SignedMembershipManifest,
) -> Result<(), BridgeError> {
    if manifest.manifest.network_id != source.network_id
        || manifest.manifest.network_profile_id != source.profile_id
        || manifest.manifest.manifest_id != source.manifest_id
        || manifest.manifest.version != source.manifest_version
        || manifest.digest() != source.manifest_digest
    {
        return Err(BridgeError::Rejected(
            "manifest does not match the source bridge profile".to_string(),
        ));
    }
    Ok(())
}

fn expected_receipt_key(
    source: &BridgeSourceProfileV1,
    manifest: &SignedMembershipManifest,
    node_id: Uuid,
) -> Result<[u8; 32], BridgeError> {
    validate_manifest_binding(source, manifest)?;
    let member = manifest
        .manifest
        .members
        .iter()
        .find(|member| member.node_id == node_id)
        .ok_or_else(|| BridgeError::Rejected("unknown receipt signer".to_string()))?;
    if member.status != MemberStatus::Active
        || !member.roles.contains(&MemberRole::Peer)
        || !member.roles.contains(&MemberRole::Validator)
        || !member
            .validator_public_key
            .is_some_and(|key| source.authority_keys.contains(&key))
    {
        return Err(BridgeError::Rejected(
            "receipt signer is not one of the three pinned active source nodes".to_string(),
        ));
    }
    Ok(member.identity_public_key)
}

fn validate_receipt_set(
    source: &BridgeSourceProfileV1,
    manifest: &SignedMembershipManifest,
    ledger_position: u64,
    envelope_hash: LedgerHash,
    ledger_prefix_hash: LedgerHash,
    receipts: &[BridgeCommitReceiptV1],
) -> Result<(), BridgeError> {
    if receipts.len() != REQUIRED_BRIDGE_RECEIPTS {
        return Err(BridgeError::Rejected(format!(
            "source export requires exactly {REQUIRED_BRIDGE_RECEIPTS} receipts"
        )));
    }
    ensure_receipts_sorted(receipts)?;
    let expected_nodes = expected_receipt_node_ids(source, manifest)?;
    let actual_nodes: Vec<_> = receipts
        .iter()
        .map(|receipt| receipt.core.node_id.clone())
        .collect();
    if actual_nodes != expected_nodes {
        return Err(BridgeError::Rejected(
            "receipt signers do not equal the three pinned source nodes".to_string(),
        ));
    }
    for receipt in receipts {
        receipt.verify(
            source,
            manifest,
            ledger_position,
            envelope_hash,
            ledger_prefix_hash,
        )?;
    }
    Ok(())
}

fn expected_receipt_node_ids(
    source: &BridgeSourceProfileV1,
    manifest: &SignedMembershipManifest,
) -> Result<Vec<String>, BridgeError> {
    validate_manifest_binding(source, manifest)?;
    let mut nodes = Vec::new();
    for member in &manifest.manifest.members {
        if member.status == MemberStatus::Active
            && member.roles.contains(&MemberRole::Peer)
            && member.roles.contains(&MemberRole::Validator)
            && member
                .validator_public_key
                .is_some_and(|key| source.authority_keys.contains(&key))
        {
            nodes.push(member.node_id.to_string());
        }
    }
    nodes.sort();
    if nodes.len() != REQUIRED_BRIDGE_RECEIPTS {
        return Err(BridgeError::InvalidProfile(
            "source profile does not resolve exactly three receipt nodes".to_string(),
        ));
    }
    Ok(nodes)
}

fn ensure_receipts_sorted(receipts: &[BridgeCommitReceiptV1]) -> Result<(), BridgeError> {
    if receipts.len() != REQUIRED_BRIDGE_RECEIPTS
        || receipts
            .windows(2)
            .any(|pair| pair[0].core.node_id >= pair[1].core.node_id)
    {
        return Err(BridgeError::Malformed(
            "receipt sequence must contain exactly three distinct receipts in node order"
                .to_string(),
        ));
    }
    Ok(())
}

fn encode_receipt_sequence(receipts: &[BridgeCommitReceiptV1]) -> Result<Vec<u8>, BridgeError> {
    ensure_receipts_sorted(receipts)?;
    let mut bytes = Vec::new();
    for receipt in receipts {
        let receipt = receipt.canonical_bytes()?;
        let len = u32::try_from(receipt.len())
            .map_err(|_| BridgeError::Malformed("receipt length exceeds u32".to_string()))?;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&receipt);
    }
    Ok(bytes)
}

fn decode_receipt_sequence(bytes: &[u8]) -> Result<Vec<BridgeCommitReceiptV1>, BridgeError> {
    let mut offset = 0usize;
    let mut receipts = Vec::with_capacity(REQUIRED_BRIDGE_RECEIPTS);
    while offset < bytes.len() && receipts.len() < REQUIRED_BRIDGE_RECEIPTS {
        let length_end = offset
            .checked_add(4)
            .ok_or_else(|| BridgeError::Malformed("receipt length overflow".to_string()))?;
        let length = u32::from_be_bytes(array(
            bytes.get(offset..length_end).ok_or_else(|| {
                BridgeError::Malformed("truncated receipt sequence length".to_string())
            })?,
            "receipt length",
        )?) as usize;
        if length == 0 || length > MAX_BRIDGE_COMMIT_RECEIPT_BYTES {
            return Err(BridgeError::Malformed(
                "invalid receipt sequence length".to_string(),
            ));
        }
        offset = length_end;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| BridgeError::Malformed("receipt offset overflow".to_string()))?;
        let receipt = bytes.get(offset..end).ok_or_else(|| {
            BridgeError::Malformed("truncated receipt sequence value".to_string())
        })?;
        receipts.push(BridgeCommitReceiptV1::decode(receipt)?);
        offset = end;
    }
    if offset != bytes.len() || receipts.len() != REQUIRED_BRIDGE_RECEIPTS {
        return Err(BridgeError::Malformed(
            "receipt sequence must contain exactly three receipts and no trailing bytes"
                .to_string(),
        ));
    }
    Ok(receipts)
}

fn validate_structural_receipt_agreement(
    declaration: &BridgeExportDeclarationV1,
    envelope: &AdmittedBlockEnvelope,
    receipts: &[BridgeCommitReceiptV1],
) -> Result<(), BridgeError> {
    let first = receipts
        .first()
        .ok_or_else(|| BridgeError::Malformed("proof has no bridge commit receipts".to_string()))?;
    for receipt in receipts {
        let core = &receipt.core;
        if core.network_id != declaration.core.source_network_id
            || core.ledger_instance_id != declaration.core.source_ledger_instance_id
            || core.profile_id != declaration.core.source_profile_id
            || core.profile_content_hash != declaration.core.source_profile_content_hash
            || core.ledger_position != envelope.index
            || core.envelope_hash != envelope.envelope_hash
            || core.manifest_id != first.core.manifest_id
            || core.manifest_version != first.core.manifest_version
            || core.manifest_digest != first.core.manifest_digest
            || core.ledger_prefix_hash != first.core.ledger_prefix_hash
        {
            return Err(BridgeError::Rejected(
                "proof receipts disagree with the declaration, envelope, manifest, or prefix"
                    .to_string(),
            ));
        }
    }
    Ok(())
}

/// Bounded bridge codec, trust, export, or import failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// A profile cannot activate the bounded source bridge slice.
    #[error("invalid bridge profile: {0}")]
    InvalidProfile(String),
    /// Bytes do not satisfy the closed canonical codec.
    #[error("malformed bridge record: {0}")]
    Malformed(String),
    /// Bytes decode but are not the unique canonical representation.
    #[error("non-canonical bridge record")]
    NonCanonical,
    /// A deterministic source bridge invariant rejected the request.
    #[error("bridge request rejected: {0}")]
    Rejected(String),
    /// One deterministic record bound was exceeded.
    #[error("{record} is oversized: {bytes} bytes exceeds {limit}")]
    Oversized {
        /// Record family.
        record: &'static str,
        /// Observed encoded size.
        bytes: usize,
        /// Fixed maximum.
        limit: usize,
    },
}

fn encode_fields(
    magic: &[u8; 4],
    record_tag: u8,
    fields: &[Vec<u8>],
) -> Result<Vec<u8>, BridgeError> {
    let field_count = u8::try_from(fields.len())
        .map_err(|_| BridgeError::Malformed("field count exceeds u8".to_string()))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(magic);
    bytes.push(record_tag);
    bytes.push(field_count);
    for (offset, value) in fields.iter().enumerate() {
        let field_tag = u8::try_from(offset + 1)
            .map_err(|_| BridgeError::Malformed("field tag exceeds u8".to_string()))?;
        let value_len = u32::try_from(value.len())
            .map_err(|_| BridgeError::Malformed("field length exceeds u32".to_string()))?;
        bytes.push(field_tag);
        bytes.extend_from_slice(&value_len.to_be_bytes());
        bytes.extend_from_slice(value);
    }
    Ok(bytes)
}

fn encode_sequence(values: &[Vec<u8>]) -> Result<Vec<u8>, BridgeError> {
    let mut bytes = Vec::new();
    let count = u32::try_from(values.len())
        .map_err(|_| BridgeError::Malformed("sequence count exceeds u32".to_string()))?;
    bytes.extend_from_slice(&count.to_be_bytes());
    for value in values {
        let len = u32::try_from(value.len())
            .map_err(|_| BridgeError::Malformed("sequence value exceeds u32".to_string()))?;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(value);
    }
    Ok(bytes)
}

fn decode_fields(
    bytes: &[u8],
    magic: &[u8; 4],
    record_tag: u8,
    field_count: u8,
) -> Result<Vec<Vec<u8>>, BridgeError> {
    if bytes.len() < 6 || &bytes[..4] != magic {
        return Err(BridgeError::Malformed("wrong record magic".to_string()));
    }
    if bytes[4] != record_tag || bytes[5] != field_count {
        return Err(BridgeError::Malformed(
            "wrong record tag or field count".to_string(),
        ));
    }
    let mut offset = 6usize;
    let mut fields = Vec::with_capacity(usize::from(field_count));
    for expected_tag in 1..=field_count {
        let tag = *bytes
            .get(offset)
            .ok_or_else(|| BridgeError::Malformed("truncated field tag".to_string()))?;
        offset = offset
            .checked_add(1)
            .ok_or_else(|| BridgeError::Malformed("field offset overflow".to_string()))?;
        if tag != expected_tag {
            return Err(BridgeError::Malformed(
                "unknown, duplicate, missing, or unordered field".to_string(),
            ));
        }
        let length_end = offset
            .checked_add(4)
            .ok_or_else(|| BridgeError::Malformed("field length overflow".to_string()))?;
        let length_bytes = bytes
            .get(offset..length_end)
            .ok_or_else(|| BridgeError::Malformed("truncated field length".to_string()))?;
        let length = u32::from_be_bytes(
            length_bytes
                .try_into()
                .map_err(|_| BridgeError::Malformed("invalid field length".to_string()))?,
        );
        offset = length_end;
        let end = offset
            .checked_add(length as usize)
            .ok_or_else(|| BridgeError::Malformed("field value overflow".to_string()))?;
        let value = bytes
            .get(offset..end)
            .ok_or_else(|| BridgeError::Malformed("truncated field value".to_string()))?;
        fields.push(value.to_vec());
        offset = end;
    }
    if offset != bytes.len() {
        return Err(BridgeError::Malformed(
            "trailing bytes after bridge record".to_string(),
        ));
    }
    Ok(fields)
}

fn domain_hash(label: &[u8], record: &[u8]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update((label.len() as u16).to_be_bytes());
    digest.update(label);
    digest.update((record.len() as u32).to_be_bytes());
    digest.update(record);
    digest.finalize().into()
}

fn validate_identifier(value: &str, field: &str) -> Result<(), BridgeError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > MAX_IDENTIFIER_BYTES
        || !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit()
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-'))
        })
    {
        return Err(BridgeError::Malformed(format!(
            "{field} is not a canonical bridge identifier"
        )));
    }
    Ok(())
}

fn decode_identifier(bytes: &[u8], field: &str) -> Result<String, BridgeError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| BridgeError::Malformed(format!("{field} is not ASCII")))?
        .to_string();
    validate_identifier(&value, field)?;
    Ok(value)
}

fn decode_hash32(value: &str, field: &str) -> Result<[u8; 32], BridgeError> {
    let decoded = hex::decode(value)
        .map_err(|_| BridgeError::InvalidProfile(format!("{field} is not lowercase hex")))?;
    if value != hex::encode(&decoded) {
        return Err(BridgeError::InvalidProfile(format!(
            "{field} is not canonical lowercase hex"
        )));
    }
    array(&decoded, field)
}

fn hex32(value: &str, field: &str) -> Result<[u8; 32], BridgeError> {
    decode_hash32(value, field)
}

fn array<const N: usize>(bytes: &[u8], field: &str) -> Result<[u8; N], BridgeError> {
    bytes
        .try_into()
        .map_err(|_| BridgeError::Malformed(format!("{field} must contain {N} bytes")))
}

fn nonzero_hash(bytes: &[u8], field: &str) -> Result<[u8; 32], BridgeError> {
    let hash = array(bytes, field)?;
    if hash == [0; 32] {
        return Err(BridgeError::Malformed(format!("{field} must be nonzero")));
    }
    Ok(hash)
}

fn decode_u64(bytes: &[u8], field: &str) -> Result<u64, BridgeError> {
    Ok(u64::from_be_bytes(array(bytes, field)?))
}

fn decode_nonzero_u64(bytes: &[u8], field: &str) -> Result<u64, BridgeError> {
    let value = decode_u64(bytes, field)?;
    if value == 0 {
        return Err(BridgeError::Malformed(format!("{field} must be nonzero")));
    }
    Ok(value)
}
