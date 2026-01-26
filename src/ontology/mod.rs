//! Ontology Management Module
//!
//! This module provides domain-specific ontology management and SHACL validation
//! for the ProvChainOrg blockchain system. It enables CLI-based ontology selection
//! at startup with strict validation that blocks invalid transactions.

pub mod domain_manager;
pub mod error;
pub mod shacl_validator;

pub use domain_manager::{DomainConfig, OntologyManager};
pub use error::{ConsistencyError, OntologyError, ShapeViolation, ValidationError};
pub use shacl_validator::{ShaclConstraint, ShaclProperty, ShaclShape, ShaclValidator};

use crate::config::Config;
use std::path::Path;

/// Ontology configuration for domain-specific blockchain validation
#[derive(Debug, Clone)]
pub struct OntologyConfig {
    /// Path to the domain-specific ontology file (e.g., "src/semantic/ontologies/uht_manufacturing.owl")
    pub domain_ontology_path: String,
    /// Path to the core ontology file (default: "src/semantic/ontologies/generic_core.owl")
    pub core_ontology_path: String,
    /// Path to domain-specific SHACL shapes (e.g., "src/semantic/shapes/uht_manufacturing.shacl.ttl")
    pub domain_shacl_path: String,
    /// Path to core SHACL shapes (default: "src/semantic/shapes/core.shacl.ttl")
    pub core_shacl_path: String,
    /// Validation mode - currently only Strict is supported
    pub validation_mode: ValidationMode,
    /// Hash of the ontology for network consistency checking
    pub ontology_hash: String,
}

/// Validation mode for SHACL constraint checking
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ValidationMode {
    /// Block invalid transactions (default and only supported mode)
    #[default]
    Strict,
}

impl OntologyConfig {
    /// Create a new ontology configuration from CLI parameters and config file
    pub fn new(domain_ontology: Option<String>, config: &Config) -> Result<Self, OntologyError> {
        let domain_ontology_path = domain_ontology
            .or_else(|| {
                config
                    .ontology_config
                    .as_ref()
                    .map(|c| c.domain_ontology_path.clone())
            })
            .unwrap_or_else(|| "src/semantic/ontologies/generic_core.owl".to_string());

        // Validate that the ontology file exists
        if !Path::new(&domain_ontology_path).exists() {
            return Err(OntologyError::OntologyNotFound {
                path: domain_ontology_path,
            });
        }

        // Derive domain-specific paths from the ontology file
        let domain_name = Self::extract_domain_name(&domain_ontology_path)?;
        let domain_shacl_path = format!("src/semantic/shapes/{}.shacl.ttl", domain_name);

        // Generate ontology hash for consistency checking
        let ontology_hash = Self::generate_ontology_hash(&domain_ontology_path)?;

        Ok(OntologyConfig {
            domain_ontology_path,
            core_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            domain_shacl_path,
            core_shacl_path: "src/semantic/shapes/core.shacl.ttl".to_string(),
            validation_mode: ValidationMode::Strict,
            ontology_hash,
        })
    }

    /// Extract domain name from ontology file path
    /// e.g., "src/semantic/ontologies/uht_manufacturing.owl" -> "uht_manufacturing"
    fn extract_domain_name(ontology_path: &str) -> Result<String, OntologyError> {
        let path = Path::new(ontology_path);
        let filename = path
            .file_stem()
            .ok_or_else(|| OntologyError::InvalidOntologyPath {
                path: ontology_path.to_string(),
                reason: "Cannot extract filename".to_string(),
            })?;

        Ok(filename.to_string_lossy().to_string())
    }

    /// Generate a hash of the ontology file for network consistency checking
    fn generate_ontology_hash(ontology_path: &str) -> Result<String, OntologyError> {
        use std::collections::hash_map::DefaultHasher;
        use std::fs;
        use std::hash::{Hash, Hasher};

        let content =
            fs::read_to_string(ontology_path).map_err(|e| OntologyError::OntologyLoadError {
                path: ontology_path.to_string(),
                source: Box::new(e),
            })?;

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Get the domain name from this configuration
    pub fn domain_name(&self) -> Result<String, OntologyError> {
        Self::extract_domain_name(&self.domain_ontology_path)
    }

    /// Validate that all required files exist
    pub fn validate_files(&self) -> Result<(), OntologyError> {
        let files = [
            &self.domain_ontology_path,
            &self.core_ontology_path,
            &self.domain_shacl_path,
            &self.core_shacl_path,
        ];

        for file_path in &files {
            if !Path::new(file_path).exists() {
                return Err(OntologyError::OntologyNotFound {
                    path: file_path.to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_domain_name() {
        assert_eq!(
            OntologyConfig::extract_domain_name("src/semantic/ontologies/uht_manufacturing.owl").unwrap(),
            "uht_manufacturing"
        );
        assert_eq!(
            OntologyConfig::extract_domain_name("pharmaceutical.owl").unwrap(),
            "pharmaceutical"
        );
    }

    #[test]
    fn test_ontology_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let ontology_path = temp_dir.path().join("test_ontology.owl");

        // Create a test ontology file
        let mut file = fs::File::create(&ontology_path).unwrap();
        writeln!(file, "@prefix owl: <http://www.w3.org/2002/07/owl#> .").unwrap();

        let config = Config::default();
        let ontology_config =
            OntologyConfig::new(Some(ontology_path.to_string_lossy().to_string()), &config);

        assert!(ontology_config.is_ok());
        let config = ontology_config.unwrap();
        assert_eq!(config.validation_mode, ValidationMode::Strict);
        assert!(!config.ontology_hash.is_empty());
    }

    #[test]
    fn test_ontology_not_found() {
        let config = Config::default();
        let result = OntologyConfig::new(Some("nonexistent/ontology.owl".to_string()), &config);

        assert!(result.is_err());
        match result.unwrap_err() {
            OntologyError::OntologyNotFound { path } => {
                assert_eq!(path, "nonexistent/ontology.owl");
            }
            _ => panic!("Expected OntologyNotFound error"),
        }
    }
}
