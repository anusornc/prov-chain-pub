//! Network profile model for shared-ontology permissioned deployments.
//!
//! A network profile is the network-wide contract that nodes are expected to match
//! before joining the same permissioned traceability network.

use crate::ontology::package::OntologyPackageManifest;
use crate::utils::config::NodeConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Semantic contract metadata exchanged by nodes during discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticContractInfo {
    pub network_profile_id: String,
    pub consensus_type: String,
    pub ontology_package_id: String,
    pub ontology_package_version: String,
    pub ontology_package_hash: String,
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
#[serde(default)]
pub struct SemanticProfile {
    pub ontology_package_id: String,
    pub ontology_package_version: String,
    pub ontology_package_hash: String,
    pub validation_mode: String,
}

impl Default for NetworkProfile {
    fn default() -> Self {
        Self {
            profile_id: "provchain.default".to_string(),
            network_id: "provchain-org-default".to_string(),
            consensus: ConsensusProfile::default(),
            semantic: SemanticProfile::default(),
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
            validation_mode: "strict".to_string(),
        }
    }
}

impl NetworkProfile {
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

        if self.semantic.validation_mode != "strict" {
            anyhow::bail!(
                "Unsupported network profile validation mode: {}",
                self.semantic.validation_mode
            );
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
            validation_mode: self.semantic.validation_mode.clone(),
        }
    }
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

    #[test]
    fn test_network_profile_matches_node_config() {
        let node_config = NodeConfig::default();
        let profile = NetworkProfile {
            semantic: SemanticProfile {
                ontology_package_hash: "placeholder".to_string(),
                ..SemanticProfile::default()
            },
            ..NetworkProfile::default()
        };
        assert!(profile.validate_node_config(&node_config).is_ok());
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
