//! OWL2 Configuration for ProvChainOrg
//! 
//! This module provides configuration options for OWL2 feature support.

/// OWL Reasoner configuration with OWL2 features
#[derive(Debug, Clone)]
pub struct OwlReasonerConfig {
    /// Whether reasoning is enabled
    pub enabled: bool,
    /// Path to the ontology file
    pub ontology_path: String,
    /// Whether to perform classification
    pub classify: bool,
    /// Whether to perform consistency checking
    pub check_consistency: bool,
    /// Whether to process OWL2 features
    pub process_owl2_features: bool,
    /// Enable owl:hasKey validation
    pub enable_has_key_validation: bool,
    /// Enable property chain inference
    pub enable_property_chain_inference: bool,
    /// Enable qualified cardinality validation
    pub enable_qualified_cardinality_validation: bool,
}

impl Default for OwlReasonerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            classify: true,
            check_consistency: true,
            process_owl2_features: true,
            enable_has_key_validation: true,
            enable_property_chain_inference: true,
            enable_qualified_cardinality_validation: true,
        }
    }
}

/// Validation result enum
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Warning(String),
}