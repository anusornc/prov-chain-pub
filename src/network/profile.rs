//! Network profile model for shared-ontology permissioned deployments.
//!
//! A network profile is the network-wide contract that nodes are expected to match
//! before joining the same permissioned traceability network.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::bridge::{
    BridgeSourceTrustProfileV1, MAX_BRIDGE_COMMIT_RECEIPT_BYTES,
    MAX_BRIDGE_EXPORT_DECLARATION_BYTES, MAX_BRIDGE_PROOF_BUNDLE_BYTES,
    MAX_BRIDGE_SIGNED_MANIFEST_BYTES, MAX_BRIDGE_SOURCE_ENVELOPE_BYTES,
    MAX_BRIDGE_SOURCE_PROFILE_BYTES, REQUIRED_BRIDGE_RECEIPTS,
};
use crate::network::membership::{validate_profile_authorities, SignedMembershipManifest};
use crate::ontology::package::{OntologyPackageManifest, SEMANTIC_EXECUTION_PROFILE_V1};
#[cfg(feature = "privacy-conformance")]
use crate::privacy::PrivacyLifecycleConformanceProfile;
use crate::privacy::{ActivatedPrivacyLifecycleProfile, PrivacyLifecycleProfile};
use crate::utils::config::NodeConfig;

/// Semantic contract metadata exchanged by nodes during discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticContractInfo {
    pub network_profile_id: String,
    pub consensus_type: String,
    pub ontology_package_id: String,
    pub ontology_package_version: String,
    pub ontology_package_hash: String,
    /// Exact bounded semantic execution profile activated by the package.
    pub semantic_execution_profile_id: String,
    pub validation_mode: String,
}

/// Shared network-wide profile for a permissioned deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkProfile {
    /// Stable profile identifier for governance and rollout.
    pub profile_id: String,
    /// Network identifier that all participating nodes must share.
    pub network_id: String,
    /// Consensus contract shared by the network.
    pub consensus: ConsensusProfile,
    /// Semantic contract shared by the network.
    pub semantic: SemanticProfile,
    /// Exact governance-authenticated membership contract for reference nodes.
    pub membership: Option<MembershipProfile>,
    /// Optional profile-bound participant/key lifecycle contract.
    pub privacy: Option<PrivacyLifecycleProfile>,
    /// Optional genesis-bound source ledger identity and outbound bridge rule.
    pub bridge: Option<SourceBridgeProfileV1>,
    /// Exact independently governed source trust accepted by this target profile.
    pub bridge_source_trust: Option<BridgeSourceTrustProfileV1>,
}

/// Consensus settings that must match across the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsensusProfile {
    pub consensus_type: String,
    pub authority_keys: Vec<String>,
    pub block_interval: u64,
    pub max_block_size: usize,
}

/// Semantic package identity and compatibility information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProfile {
    pub ontology_package_id: String,
    pub ontology_package_version: String,
    pub ontology_package_hash: String,
    /// Exact bounded semantic execution profile required by the network.
    pub semantic_execution_profile_id: String,
    pub validation_mode: String,
}

/// Identity, version, and digest of the manifest activated by this profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MembershipProfile {
    /// Stable manifest identifier.
    pub manifest_id: String,
    /// Monotonically increasing manifest version selected by governance.
    pub manifest_version: u64,
    /// Lowercase SHA-256 identity of the canonical manifest bytes.
    pub manifest_digest: String,
}

/// Genesis-bound source identity and the one outbound rule supported by bridge v1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceBridgeProfileV1 {
    source_ledger_instance_id: String,
    outbound_rule: OutboundBridgeRuleV1,
}

/// Closed outbound authorization embedded in the canonical source Network Profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OutboundBridgeRuleV1 {
    target_network_id: String,
    target_ledger_instance_id: String,
    bridge_suite: String,
    copy_mode: String,
    state_commitment_scheme: String,
    source_admission_kind: String,
    max_public_payload_bytes: u64,
    max_export_declaration_bytes: u64,
    max_source_profile_bytes: u64,
    max_signed_manifest_bytes: u64,
    max_source_envelope_bytes: u64,
    max_commit_receipt_bytes: u64,
    required_commit_receipts: u64,
    max_proof_bundle_bytes: u64,
    max_target_origin_evidence_bytes: u64,
    max_target_non_payload_bytes: u64,
    max_target_envelope_bytes: u64,
}

impl SourceBridgeProfileV1 {
    /// Create the exact bounded source identity and outbound authorization fixed by ADR 0037.
    pub fn new(
        source_ledger_instance_id: [u8; 32],
        target_network_id: impl Into<String>,
        target_ledger_instance_id: [u8; 32],
    ) -> anyhow::Result<Self> {
        let profile = Self {
            source_ledger_instance_id: hex::encode(source_ledger_instance_id),
            outbound_rule: OutboundBridgeRuleV1 {
                target_network_id: target_network_id.into(),
                target_ledger_instance_id: hex::encode(target_ledger_instance_id),
                bridge_suite: "ProvChainBridgeSuiteV1".to_string(),
                copy_mode: "ExactPublicPayloadV1".to_string(),
                state_commitment_scheme: "Rdfc10Sha256NQuadsV1".to_string(),
                source_admission_kind: "OrdinaryProvenanceV1".to_string(),
                max_public_payload_bytes: 262_144,
                max_export_declaration_bytes: MAX_BRIDGE_EXPORT_DECLARATION_BYTES as u64,
                max_source_profile_bytes: MAX_BRIDGE_SOURCE_PROFILE_BYTES as u64,
                max_signed_manifest_bytes: MAX_BRIDGE_SIGNED_MANIFEST_BYTES as u64,
                max_source_envelope_bytes: MAX_BRIDGE_SOURCE_ENVELOPE_BYTES as u64,
                max_commit_receipt_bytes: MAX_BRIDGE_COMMIT_RECEIPT_BYTES as u64,
                required_commit_receipts: REQUIRED_BRIDGE_RECEIPTS as u64,
                max_proof_bundle_bytes: MAX_BRIDGE_PROOF_BUNDLE_BYTES as u64,
                max_target_origin_evidence_bytes: 503_968,
                max_target_non_payload_bytes: 262_187,
                max_target_envelope_bytes: 1_048_576,
            },
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Exact raw source ledger-instance identifier.
    pub fn source_ledger_instance_id(&self) -> anyhow::Result<[u8; 32]> {
        decode_profile_id32(
            &self.source_ledger_instance_id,
            "source bridge ledger instance",
        )
    }

    /// Closed outbound authorization carried by this profile.
    pub fn outbound_rule(&self) -> &OutboundBridgeRuleV1 {
        &self.outbound_rule
    }

    fn validate(&self) -> anyhow::Result<()> {
        let source = self.source_ledger_instance_id()?;
        if source == [0; 32] {
            anyhow::bail!("source bridge ledger instance must be nonzero");
        }
        self.outbound_rule.validate()
    }
}

impl OutboundBridgeRuleV1 {
    /// Exact target network identifier.
    pub fn target_network_id(&self) -> &str {
        &self.target_network_id
    }

    /// Exact raw target ledger-instance identifier.
    pub fn target_ledger_instance_id(&self) -> anyhow::Result<[u8; 32]> {
        decode_profile_id32(
            &self.target_ledger_instance_id,
            "target bridge ledger instance",
        )
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.target_network_id.is_empty()
            || self.target_network_id.trim() != self.target_network_id
            || self.target_network_id.len() > 128
            || !self.target_network_id.is_ascii()
            || self
                .target_network_id
                .bytes()
                .any(|byte| byte.is_ascii_control())
        {
            anyhow::bail!("target bridge network identifier is invalid");
        }
        let target = self.target_ledger_instance_id()?;
        if target == [0; 32] {
            anyhow::bail!("target bridge ledger instance must be nonzero");
        }
        if self.bridge_suite != "ProvChainBridgeSuiteV1"
            || self.copy_mode != "ExactPublicPayloadV1"
            || self.state_commitment_scheme != "Rdfc10Sha256NQuadsV1"
            || self.source_admission_kind != "OrdinaryProvenanceV1"
            || self.max_public_payload_bytes != 262_144
            || self.max_export_declaration_bytes != MAX_BRIDGE_EXPORT_DECLARATION_BYTES as u64
            || self.max_source_profile_bytes != MAX_BRIDGE_SOURCE_PROFILE_BYTES as u64
            || self.max_signed_manifest_bytes != MAX_BRIDGE_SIGNED_MANIFEST_BYTES as u64
            || self.max_source_envelope_bytes != MAX_BRIDGE_SOURCE_ENVELOPE_BYTES as u64
            || self.max_commit_receipt_bytes != MAX_BRIDGE_COMMIT_RECEIPT_BYTES as u64
            || self.required_commit_receipts != REQUIRED_BRIDGE_RECEIPTS as u64
            || self.max_proof_bundle_bytes != MAX_BRIDGE_PROOF_BUNDLE_BYTES as u64
            || self.max_target_origin_evidence_bytes != 503_968
            || self.max_target_non_payload_bytes != 262_187
            || self.max_target_envelope_bytes != 1_048_576
        {
            anyhow::bail!("unsupported outbound bridge rule");
        }
        Ok(())
    }
}

impl Default for NetworkProfile {
    fn default() -> Self {
        Self {
            profile_id: "provchain.default".to_string(),
            network_id: "provchain-org-default".to_string(),
            consensus: ConsensusProfile::default(),
            semantic: SemanticProfile::default(),
            membership: None,
            privacy: None,
            bridge: None,
            bridge_source_trust: None,
        }
    }
}

impl Default for ConsensusProfile {
    fn default() -> Self {
        Self {
            consensus_type: "poa".to_string(),
            authority_keys: vec![],
            block_interval: 10,
            max_block_size: 1024 * 1024,
        }
    }
}

impl Default for SemanticProfile {
    fn default() -> Self {
        Self {
            ontology_package_id: "provchain.shared-ontology.default".to_string(),
            ontology_package_version: "0.1.0".to_string(),
            ontology_package_hash: String::new(),
            semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
            validation_mode: "strict".to_string(),
        }
    }
}

impl NetworkProfile {
    /// Verify the complete governance/profile context and produce the only
    /// lifecycle binding accepted by the durable ledger.
    pub fn activate_privacy_lifecycle(
        &self,
        signed_manifest: &SignedMembershipManifest,
        governance_root: &ed25519_dalek::VerifyingKey,
    ) -> anyhow::Result<Option<ActivatedPrivacyLifecycleProfile>> {
        let Some(_) = self.verify_privacy_lifecycle_context(signed_manifest, governance_root)?
        else {
            return Ok(None);
        };
        anyhow::bail!(
            "PrivacyControlV1 activation remains unavailable until the complete Issue #10 evidence gate is recorded"
        )
    }

    /// Verify the feature-gated conformance slice without making it deployable.
    ///
    /// Reference-node startup rejects this capability. It exists only so the
    /// closed codecs, reducer, Final Admission, journal, replay, and custody
    /// behavior can accumulate evidence before indivisible activation.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn privacy_lifecycle_conformance_slice(
        &self,
        signed_manifest: &SignedMembershipManifest,
        governance_root: &ed25519_dalek::VerifyingKey,
    ) -> anyhow::Result<Option<PrivacyLifecycleConformanceProfile>> {
        Ok(self
            .verify_privacy_lifecycle_context(signed_manifest, governance_root)?
            .map(PrivacyLifecycleConformanceProfile::new))
    }

    fn verify_privacy_lifecycle_context(
        &self,
        signed_manifest: &SignedMembershipManifest,
        governance_root: &ed25519_dalek::VerifyingKey,
    ) -> anyhow::Result<Option<PrivacyLifecycleProfile>> {
        self.validate()?;
        let Some(privacy) = &self.privacy else {
            return Ok(None);
        };
        signed_manifest.verify(governance_root)?;
        let binding = self
            .membership
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Privacy lifecycle requires a Membership Manifest"))?;
        let manifest = &signed_manifest.manifest;
        if manifest.network_id != self.network_id
            || manifest.network_profile_id != self.profile_id
            || manifest.manifest_id != binding.manifest_id
            || manifest.version != binding.manifest_version
            || hex::encode(signed_manifest.digest()) != binding.manifest_digest
        {
            anyhow::bail!(
                "Privacy lifecycle Membership Manifest does not match the Network Profile binding"
            );
        }
        validate_profile_authorities(self, manifest)?;

        let governance_public_key = governance_root.to_bytes();
        let mut expected_network_role_keys = BTreeSet::from([governance_public_key]);
        for member in &manifest.members {
            if member.identity_public_key == governance_public_key
                || member.validator_public_key == Some(governance_public_key)
            {
                anyhow::bail!("Membership governance root reuses a member key role");
            }
            expected_network_role_keys.insert(member.identity_public_key);
            if let Some(validator_public_key) = member.validator_public_key {
                expected_network_role_keys.insert(validator_public_key);
            }
        }
        if expected_network_role_keys.contains(&privacy.bootstrap_public_key()) {
            anyhow::bail!(
                "Privacy bootstrap governance key reuses a verified Network Profile key role"
            );
        }
        if privacy.network_role_public_keys() != &expected_network_role_keys {
            anyhow::bail!(
                "Privacy lifecycle Network Profile role-key set does not match verified governance and membership"
            );
        }
        Ok(Some(privacy.clone()))
    }

    /// Bind the lifecycle profile to the canonical hash of this complete
    /// Network Profile, excluding only the hash field itself.
    pub fn with_derived_privacy_content_hash(mut self) -> anyhow::Result<Self> {
        if self.privacy.is_none() {
            anyhow::bail!("Network profile has no privacy lifecycle binding");
        }
        let content_hash = self.derived_content_hash()?;
        self.privacy
            .as_mut()
            .expect("privacy presence checked above")
            .bind_network_profile_content_hash(content_hash);
        self.validate()?;
        Ok(self)
    }

    /// Derive the canonical complete-profile identity used by privacy anchors.
    ///
    /// The preimage deliberately excludes only the embedded content-hash field
    /// to avoid a self-reference. It includes the privacy bootstrap binding and
    /// the sorted public keys reserved for all other verified network roles.
    pub fn derived_content_hash(&self) -> anyhow::Result<[u8; 32]> {
        Ok(Sha256::digest(self.canonical_bytes()?).into())
    }

    /// Encode the exact canonical Network Profile artifact used by bridge proofs.
    ///
    /// The encoding excludes only the privacy profile's self-referential
    /// `network_profile_content_hash`, matching [`Self::derived_content_hash`].
    /// A present bridge contract is a trailing extension; profiles without one
    /// retain their pre-bridge canonical bytes and content identity.
    pub fn canonical_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = b"PROVCHAIN_NETWORK_PROFILE_CONTENT_V1".to_vec();
        push_profile_text(&mut bytes, &self.profile_id)?;
        push_profile_text(&mut bytes, &self.network_id)?;
        push_profile_text(&mut bytes, &self.consensus.consensus_type)?;
        push_profile_len(&mut bytes, self.consensus.authority_keys.len())?;
        for authority_key in &self.consensus.authority_keys {
            push_profile_text(&mut bytes, authority_key)?;
        }
        bytes.extend_from_slice(&self.consensus.block_interval.to_be_bytes());
        bytes.extend_from_slice(
            &u64::try_from(self.consensus.max_block_size)
                .map_err(|_| anyhow::anyhow!("Network profile max_block_size exceeds u64"))?
                .to_be_bytes(),
        );
        push_profile_text(&mut bytes, &self.semantic.ontology_package_id)?;
        push_profile_text(&mut bytes, &self.semantic.ontology_package_version)?;
        push_profile_text(&mut bytes, &self.semantic.ontology_package_hash)?;
        push_profile_text(&mut bytes, &self.semantic.semantic_execution_profile_id)?;
        push_profile_text(&mut bytes, &self.semantic.validation_mode)?;
        match &self.membership {
            Some(membership) => {
                bytes.push(1);
                push_profile_text(&mut bytes, &membership.manifest_id)?;
                bytes.extend_from_slice(&membership.manifest_version.to_be_bytes());
                push_profile_text(&mut bytes, &membership.manifest_digest)?;
            }
            None => bytes.push(0),
        }
        match &self.privacy {
            Some(privacy) => {
                bytes.push(1);
                push_profile_text(&mut bytes, crate::privacy::PRIVACY_CONTROL_V1)?;
                push_profile_text(&mut bytes, crate::privacy::CANONICAL_PRIVACY_ENCODING_V1)?;
                bytes.extend_from_slice(&privacy.bootstrap_public_key());
                bytes.extend_from_slice(&privacy.bootstrap_fingerprint());
                push_profile_len(&mut bytes, privacy.network_role_public_keys().len())?;
                for public_key in privacy.network_role_public_keys() {
                    bytes.extend_from_slice(public_key);
                }
            }
            None => bytes.push(0),
        }
        if let Some(bridge) = &self.bridge {
            bytes.push(1);
            bytes.extend_from_slice(&bridge.source_ledger_instance_id()?);
            let rule = bridge.outbound_rule();
            push_profile_text(&mut bytes, rule.target_network_id())?;
            bytes.extend_from_slice(&rule.target_ledger_instance_id()?);
            push_profile_text(&mut bytes, &rule.bridge_suite)?;
            push_profile_text(&mut bytes, &rule.copy_mode)?;
            push_profile_text(&mut bytes, &rule.state_commitment_scheme)?;
            push_profile_text(&mut bytes, &rule.source_admission_kind)?;
            for bound in [
                rule.max_public_payload_bytes,
                rule.max_export_declaration_bytes,
                rule.max_source_profile_bytes,
                rule.max_signed_manifest_bytes,
                rule.max_source_envelope_bytes,
                rule.max_commit_receipt_bytes,
                rule.required_commit_receipts,
                rule.max_proof_bundle_bytes,
                rule.max_target_origin_evidence_bytes,
                rule.max_target_non_payload_bytes,
                rule.max_target_envelope_bytes,
            ] {
                bytes.extend_from_slice(&bound.to_be_bytes());
            }
        }
        if let Some(trust) = &self.bridge_source_trust {
            bytes.push(2);
            let trust_bytes = trust
                .canonical_config_bytes()
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            push_profile_len(&mut bytes, trust_bytes.len())?;
            bytes.extend_from_slice(&trust_bytes);
        }
        Ok(bytes)
    }

    /// Load and validate a network profile from TOML.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let profile: Self = toml::from_str(&content)?;
        profile.validate()?;
        Ok(profile)
    }

    /// Save a network profile to TOML.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Validate profile structure.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.profile_id.trim().is_empty() {
            anyhow::bail!("Network profile ID cannot be empty");
        }

        if self.network_id.trim().is_empty() {
            anyhow::bail!("Network profile network_id cannot be empty");
        }

        if !matches!(self.consensus.consensus_type.as_str(), "poa" | "pbft") {
            anyhow::bail!(
                "Unsupported network profile consensus type: {}",
                self.consensus.consensus_type
            );
        }

        if self.consensus.block_interval == 0 {
            anyhow::bail!("Network profile block_interval must be greater than 0");
        }

        if self.consensus.max_block_size == 0 {
            anyhow::bail!("Network profile max_block_size must be greater than 0");
        }

        if self.semantic.ontology_package_id.trim().is_empty() {
            anyhow::bail!("Network profile ontology_package_id cannot be empty");
        }

        if self.semantic.ontology_package_version.trim().is_empty() {
            anyhow::bail!("Network profile ontology_package_version cannot be empty");
        }

        if self.semantic.ontology_package_hash.trim().is_empty() {
            anyhow::bail!("Network profile ontology_package_hash cannot be empty");
        }

        if self.semantic.semantic_execution_profile_id != SEMANTIC_EXECUTION_PROFILE_V1 {
            anyhow::bail!(
                "Unsupported network semantic execution profile: {}",
                self.semantic.semantic_execution_profile_id
            );
        }

        if self.semantic.validation_mode != "strict" {
            anyhow::bail!(
                "Unsupported network profile validation mode: {}",
                self.semantic.validation_mode
            );
        }

        if let Some(membership) = &self.membership {
            if membership.manifest_id.trim().is_empty() {
                anyhow::bail!("Membership manifest ID cannot be empty");
            }
            if membership.manifest_version == 0 {
                anyhow::bail!("Membership manifest version must be greater than 0");
            }
            let digest = hex::decode(&membership.manifest_digest)
                .map_err(|_| anyhow::anyhow!("Membership manifest digest must be lowercase hex"))?;
            if digest.len() != 32 || membership.manifest_digest != hex::encode(&digest) {
                anyhow::bail!("Membership manifest digest must be 32 lowercase hex bytes");
            }
        }

        if let Some(privacy) = &self.privacy {
            privacy.validate()?;
            let bootstrap_key_hex = hex::encode(privacy.bootstrap_public_key());
            if self.consensus.authority_keys.contains(&bootstrap_key_hex) {
                anyhow::bail!("Privacy bootstrap governance key cannot reuse a PoA authority key");
            }
            let derived_content_hash = self.derived_content_hash()?;
            if privacy.network_profile_content_hash() != derived_content_hash {
                anyhow::bail!(
                    "Privacy lifecycle content hash does not match the canonical Network Profile"
                );
            }
        }

        if let Some(bridge) = &self.bridge {
            bridge.validate()?;
            if self.membership.is_none() {
                anyhow::bail!("Source bridge profile requires a Membership Manifest binding");
            }
            if self.consensus.consensus_type != "poa" {
                anyhow::bail!("Source bridge profile requires PoA consensus");
            }
            let canonical_len = self.canonical_bytes()?.len();
            if canonical_len > MAX_BRIDGE_SOURCE_PROFILE_BYTES {
                anyhow::bail!(
                    "Canonical source Network Profile exceeds {MAX_BRIDGE_SOURCE_PROFILE_BYTES} bytes"
                );
            }
        }

        if let Some(trust) = &self.bridge_source_trust {
            if self.bridge.is_some() {
                anyhow::bail!("Source and target bridge roles are mutually exclusive");
            }
            if self.membership.is_none() || self.consensus.consensus_type != "poa" {
                anyhow::bail!(
                    "Target bridge trust requires a Membership Manifest and PoA consensus"
                );
            }
            trust
                .validate_target_profile(self)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        }

        Ok(())
    }

    /// Validate that a node-local config is compatible with the network profile.
    pub fn validate_node_config(&self, node_config: &NodeConfig) -> anyhow::Result<()> {
        self.validate()?;

        if node_config.network.network_id != self.network_id {
            anyhow::bail!(
                "Node network_id '{}' does not match network profile '{}'",
                node_config.network.network_id,
                self.network_id
            );
        }

        if node_config.consensus.consensus_type != self.consensus.consensus_type {
            anyhow::bail!(
                "Node consensus_type '{}' does not match network profile '{}'",
                node_config.consensus.consensus_type,
                self.consensus.consensus_type
            );
        }

        if node_config.consensus.block_interval != self.consensus.block_interval {
            anyhow::bail!(
                "Node block_interval '{}' does not match network profile '{}'",
                node_config.consensus.block_interval,
                self.consensus.block_interval
            );
        }

        if node_config.consensus.max_block_size != self.consensus.max_block_size {
            anyhow::bail!(
                "Node max_block_size '{}' does not match network profile '{}'",
                node_config.consensus.max_block_size,
                self.consensus.max_block_size
            );
        }

        if !self.consensus.authority_keys.is_empty()
            && node_config.consensus.authority_keys != self.consensus.authority_keys
        {
            anyhow::bail!("Node authority_keys do not match the network profile");
        }

        if self.semantic.validation_mode == "strict" {
            let Some(ontology) = &node_config.ontology else {
                anyhow::bail!(
                    "Network profile '{}' requires an ontology package, but node config has no ontology section",
                    self.profile_id
                );
            };

            if ontology.package_manifest_path.is_none() {
                anyhow::bail!(
                    "Network profile '{}' requires an ontology package manifest",
                    self.profile_id
                );
            }

            if !ontology.auto_load {
                anyhow::bail!(
                    "Network profile '{}' requires ontology.auto_load=true",
                    self.profile_id
                );
            }

            if !ontology.validate_data {
                anyhow::bail!(
                    "Network profile '{}' requires ontology.validate_data=true",
                    self.profile_id
                );
            }
        }

        Ok(())
    }

    /// Validate that an ontology package manifest matches the network semantic contract.
    pub fn validate_manifest(&self, manifest: &OntologyPackageManifest) -> anyhow::Result<()> {
        self.validate()?;
        manifest.validate()?;

        if manifest.package_id != self.semantic.ontology_package_id {
            anyhow::bail!(
                "Ontology package ID '{}' does not match network profile '{}'",
                manifest.package_id,
                self.semantic.ontology_package_id
            );
        }

        if manifest.package_version != self.semantic.ontology_package_version {
            anyhow::bail!(
                "Ontology package version '{}' does not match network profile '{}'",
                manifest.package_version,
                self.semantic.ontology_package_version
            );
        }

        if manifest.validation_mode != self.semantic.validation_mode {
            anyhow::bail!(
                "Ontology package validation mode '{}' does not match network profile '{}'",
                manifest.validation_mode,
                self.semantic.validation_mode
            );
        }

        if manifest.semantic_execution_profile_id != self.semantic.semantic_execution_profile_id {
            anyhow::bail!(
                "Ontology package semantic execution profile '{}' does not match network profile '{}'",
                manifest.semantic_execution_profile_id,
                self.semantic.semantic_execution_profile_id
            );
        }

        let manifest_hash = manifest.resolved_package_hash()?;
        if manifest_hash != self.semantic.ontology_package_hash {
            anyhow::bail!(
                "Ontology package hash '{}' does not match network profile '{}'",
                manifest_hash,
                self.semantic.ontology_package_hash
            );
        }

        Ok(())
    }

    /// Convert the network profile into discovery metadata for semantic compatibility checks.
    pub fn semantic_contract_info(&self) -> SemanticContractInfo {
        SemanticContractInfo {
            network_profile_id: self.profile_id.clone(),
            consensus_type: self.consensus.consensus_type.clone(),
            ontology_package_id: self.semantic.ontology_package_id.clone(),
            ontology_package_version: self.semantic.ontology_package_version.clone(),
            ontology_package_hash: self.semantic.ontology_package_hash.clone(),
            semantic_execution_profile_id: self.semantic.semantic_execution_profile_id.clone(),
            validation_mode: self.semantic.validation_mode.clone(),
        }
    }
}

fn push_profile_len(bytes: &mut Vec<u8>, length: usize) -> anyhow::Result<()> {
    bytes.extend_from_slice(
        &u32::try_from(length)
            .map_err(|_| anyhow::anyhow!("Network profile collection exceeds u32"))?
            .to_be_bytes(),
    );
    Ok(())
}

fn push_profile_text(bytes: &mut Vec<u8>, value: &str) -> anyhow::Result<()> {
    push_profile_len(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn decode_profile_id32(value: &str, field: &str) -> anyhow::Result<[u8; 32]> {
    let decoded =
        hex::decode(value).map_err(|_| anyhow::anyhow!("{field} must be lowercase hexadecimal"))?;
    if value != hex::encode(&decoded) {
        anyhow::bail!("{field} must use lowercase hexadecimal");
    }
    decoded
        .try_into()
        .map_err(|_| anyhow::anyhow!("{field} must contain exactly 32 bytes"))
}

impl SemanticContractInfo {
    /// Validate semantic contract metadata before it is exchanged over the network.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.network_profile_id.trim().is_empty() {
            anyhow::bail!("Semantic contract network_profile_id cannot be empty");
        }

        if !matches!(self.consensus_type.as_str(), "poa" | "pbft") {
            anyhow::bail!(
                "Unsupported semantic contract consensus type: {}",
                self.consensus_type
            );
        }

        if self.ontology_package_id.trim().is_empty() {
            anyhow::bail!("Semantic contract ontology_package_id cannot be empty");
        }

        if self.ontology_package_version.trim().is_empty() {
            anyhow::bail!("Semantic contract ontology_package_version cannot be empty");
        }

        if self.ontology_package_hash.trim().is_empty() {
            anyhow::bail!("Semantic contract ontology_package_hash cannot be empty");
        }

        if self.semantic_execution_profile_id != SEMANTIC_EXECUTION_PROFILE_V1 {
            anyhow::bail!(
                "Unsupported semantic contract execution profile: {}",
                self.semantic_execution_profile_id
            );
        }

        if self.validation_mode != "strict" {
            anyhow::bail!(
                "Unsupported semantic contract validation mode: {}",
                self.validation_mode
            );
        }

        Ok(())
    }

    /// Load semantic contract metadata from the node's declared network profile and ontology package.
    pub fn load_from_node_config(node_config: &NodeConfig) -> anyhow::Result<Option<Self>> {
        let Some(profile_path) = &node_config.network_profile_path else {
            return Ok(None);
        };

        let profile = NetworkProfile::load_from_file(profile_path)?;
        profile.validate_node_config(node_config)?;

        if let Some(ontology) = &node_config.ontology {
            if let Some(manifest_path) = &ontology.package_manifest_path {
                let manifest = OntologyPackageManifest::load_from_file(manifest_path)?;
                profile.validate_manifest(&manifest)?;
            }
        }

        Ok(Some(profile.semantic_contract_info()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::package::OntologyPackageManifest;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_file(path: &Path, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        writeln!(file, "{contents}").unwrap();
    }

    fn strict_test_profile() -> NetworkProfile {
        NetworkProfile {
            semantic: SemanticProfile {
                ontology_package_hash: "placeholder".to_string(),
                ..SemanticProfile::default()
            },
            ..NetworkProfile::default()
        }
    }

    fn strict_node_config_with_manifest() -> NodeConfig {
        let mut node_config = NodeConfig::default();
        let ontology = node_config
            .ontology
            .as_mut()
            .expect("default node config should include ontology settings");
        ontology.package_manifest_path = Some("config/ontology_package.toml".to_string());
        ontology.validate_data = true;
        node_config
    }

    #[test]
    fn test_network_profile_matches_node_config() {
        let node_config = strict_node_config_with_manifest();
        let profile = strict_test_profile();
        assert!(profile.validate_node_config(&node_config).is_ok());
    }

    #[test]
    fn test_strict_network_profile_requires_ontology_section() {
        let node_config = NodeConfig {
            ontology: None,
            ..NodeConfig::default()
        };
        let profile = strict_test_profile();

        let error = profile.validate_node_config(&node_config).unwrap_err();
        assert!(
            error.to_string().contains("has no ontology section"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_strict_network_profile_requires_ontology_manifest() {
        let mut node_config = strict_node_config_with_manifest();
        node_config
            .ontology
            .as_mut()
            .expect("test config should include ontology settings")
            .package_manifest_path = None;
        let profile = strict_test_profile();

        let error = profile.validate_node_config(&node_config).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires an ontology package manifest"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_strict_network_profile_requires_ontology_auto_load() {
        let mut node_config = strict_node_config_with_manifest();
        let ontology = node_config
            .ontology
            .as_mut()
            .expect("test config should include ontology settings");
        ontology.auto_load = false;
        let profile = strict_test_profile();

        let error = profile.validate_node_config(&node_config).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires ontology.auto_load=true"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_strict_network_profile_requires_ontology_validate_data() {
        let mut node_config = strict_node_config_with_manifest();
        let ontology = node_config
            .ontology
            .as_mut()
            .expect("test config should include ontology settings");
        ontology.validate_data = false;
        let profile = strict_test_profile();

        let error = profile.validate_node_config(&node_config).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires ontology.validate_data=true"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_network_profile_matches_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let core = temp_dir.path().join("core.owl");
        let domain = temp_dir.path().join("domain.owl");
        let core_shape = temp_dir.path().join("core.shacl.ttl");
        let domain_shape = temp_dir.path().join("domain.shacl.ttl");

        create_file(&core, "@prefix owl: <http://www.w3.org/2002/07/owl#> .");
        create_file(&domain, "@prefix ex: <http://example.com#> .");
        create_file(&core_shape, "@prefix sh: <http://www.w3.org/ns/shacl#> .");
        create_file(&domain_shape, "@prefix sh: <http://www.w3.org/ns/shacl#> .");

        let manifest = OntologyPackageManifest {
            package_id: "provchain.shared-ontology.default".to_string(),
            package_version: "0.1.0".to_string(),
            core_ontology_path: core.to_string_lossy().to_string(),
            domain_ontology_path: domain.to_string_lossy().to_string(),
            core_shacl_path: core_shape.to_string_lossy().to_string(),
            domain_shacl_path: domain_shape.to_string_lossy().to_string(),
            ..OntologyPackageManifest::default()
        };

        let profile = NetworkProfile {
            semantic: SemanticProfile {
                ontology_package_hash: manifest.resolved_package_hash().unwrap(),
                ..SemanticProfile::default()
            },
            ..NetworkProfile::default()
        };

        assert!(profile.validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_semantic_contract_info_from_profile() {
        let profile = NetworkProfile {
            semantic: SemanticProfile {
                ontology_package_hash: "hash123".to_string(),
                ..SemanticProfile::default()
            },
            ..NetworkProfile::default()
        };

        let contract = profile.semantic_contract_info();
        assert_eq!(contract.network_profile_id, "provchain.default");
        assert_eq!(contract.ontology_package_hash, "hash123");
        assert!(contract.validate().is_ok());
    }
}
