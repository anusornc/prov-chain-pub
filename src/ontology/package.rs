//! Ontology package manifest for shared-ontology permissioned networks
//!
//! This module defines the deployable semantic package artifact that network
//! participants are expected to share when they join the same traceability network.

use crate::ontology::{OntologyConfig, OntologyError, ValidationMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Deployable ontology package manifest shared across a permissioned network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OntologyPackageManifest {
    /// Stable package identifier shared across the network.
    pub package_id: String,
    /// Semantic version of the package.
    pub package_version: String,
    /// Foundational provenance ontology used by the package.
    pub core_ontology_path: String,
    /// Network- or domain-specific ontology extending the provenance core.
    pub domain_ontology_path: String,
    /// Core SHACL shapes applied across the network.
    pub core_shacl_path: String,
    /// Domain-specific SHACL shapes for this package.
    pub domain_shacl_path: String,
    /// Optional mapping files to external standards such as GS1/EPCIS.
    pub mappings: Vec<String>,
    /// Validation mode enforced by the package.
    pub validation_mode: String,
    /// Optional expected package hash. If present, validation checks it.
    pub package_hash: Option<String>,
}

impl Default for OntologyPackageManifest {
    fn default() -> Self {
        Self {
            package_id: "provchain.shared-ontology.default".to_string(),
            package_version: "0.1.0".to_string(),
            core_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            domain_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            core_shacl_path: "src/semantic/shapes/core.shacl.ttl".to_string(),
            domain_shacl_path: "src/semantic/shapes/core.shacl.ttl".to_string(),
            mappings: vec![],
            validation_mode: "strict".to_string(),
            package_hash: None,
        }
    }
}

impl OntologyPackageManifest {
    /// Load a package manifest from a TOML file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Save a package manifest to a TOML file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Validate manifest structure and referenced files.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.package_id.trim().is_empty() {
            anyhow::bail!("Ontology package ID cannot be empty");
        }

        if self.package_version.trim().is_empty() {
            anyhow::bail!("Ontology package version cannot be empty");
        }

        if self.validation_mode != "strict" {
            anyhow::bail!(
                "Unsupported ontology package validation mode: {}",
                self.validation_mode
            );
        }

        for path in self.required_paths() {
            if !Path::new(path).exists() {
                anyhow::bail!("Ontology package path does not exist: {}", path);
            }
        }

        let computed_hash = self.compute_package_hash()?;
        if let Some(expected_hash) = &self.package_hash {
            if expected_hash != &computed_hash {
                anyhow::bail!(
                    "Ontology package hash mismatch: expected {}, computed {}",
                    expected_hash,
                    computed_hash
                );
            }
        }

        Ok(())
    }

    /// Compute a deterministic hash from manifest metadata and referenced file contents.
    pub fn compute_package_hash(&self) -> anyhow::Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(self.package_id.as_bytes());
        hasher.update(self.package_version.as_bytes());
        hasher.update(self.validation_mode.as_bytes());

        for path in self.required_paths() {
            hasher.update(path.as_bytes());
            let content = fs::read(path)?;
            hasher.update(content);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Compute the package hash without requiring it to be embedded in the manifest.
    pub fn resolved_package_hash(&self) -> anyhow::Result<String> {
        self.compute_package_hash()
    }

    /// Convert the package manifest into the production ontology runtime config.
    pub fn to_ontology_config(&self) -> Result<OntologyConfig, OntologyError> {
        self.validate()
            .map_err(|e| OntologyError::OntologyParseError {
                path: self.domain_ontology_path.clone(),
                message: e.to_string(),
            })?;

        Ok(OntologyConfig {
            domain_ontology_path: self.domain_ontology_path.clone(),
            core_ontology_path: self.core_ontology_path.clone(),
            domain_shacl_path: self.domain_shacl_path.clone(),
            core_shacl_path: self.core_shacl_path.clone(),
            validation_mode: ValidationMode::Strict,
            ontology_hash: self.resolved_package_hash().map_err(|e| {
                OntologyError::OntologyParseError {
                    path: self.domain_ontology_path.clone(),
                    message: e.to_string(),
                }
            })?,
        })
    }

    fn required_paths(&self) -> Vec<&str> {
        let mut paths = vec![
            self.core_ontology_path.as_str(),
            self.domain_ontology_path.as_str(),
            self.core_shacl_path.as_str(),
            self.domain_shacl_path.as_str(),
        ];
        for mapping in &self.mappings {
            paths.push(mapping.as_str());
        }
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_file(path: &Path, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        writeln!(file, "{contents}").unwrap();
    }

    #[test]
    fn test_manifest_hash_is_stable() {
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
            package_id: "provchain.test".to_string(),
            package_version: "1.0.0".to_string(),
            core_ontology_path: core.to_string_lossy().to_string(),
            domain_ontology_path: domain.to_string_lossy().to_string(),
            core_shacl_path: core_shape.to_string_lossy().to_string(),
            domain_shacl_path: domain_shape.to_string_lossy().to_string(),
            ..OntologyPackageManifest::default()
        };

        let first = manifest.compute_package_hash().unwrap();
        let second = manifest.compute_package_hash().unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn test_manifest_converts_to_ontology_config() {
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
            package_id: "provchain.test".to_string(),
            package_version: "1.0.0".to_string(),
            core_ontology_path: core.to_string_lossy().to_string(),
            domain_ontology_path: domain.to_string_lossy().to_string(),
            core_shacl_path: core_shape.to_string_lossy().to_string(),
            domain_shacl_path: domain_shape.to_string_lossy().to_string(),
            ..OntologyPackageManifest::default()
        };

        let ontology_config = manifest.to_ontology_config().unwrap();

        assert_eq!(
            ontology_config.domain_ontology_path,
            domain.to_string_lossy()
        );
        assert!(!ontology_config.ontology_hash.is_empty());
    }
}
