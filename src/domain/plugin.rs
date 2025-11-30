//! Domain plugin interface for universal traceability platform
//!
//! This module defines the trait for domain-specific plugins
//! that extend the generic traceability system with domain-specific
//! validation and processing capabilities.

use anyhow::Result;
use std::collections::HashMap;

/// Trait for domain-specific plugins
pub trait DomainPlugin: Send + Sync {
    /// Unique identifier for the domain
    fn domain_id(&self) -> &str;

    /// Human readable name
    fn name(&self) -> &str;

    /// Description of the domain
    fn description(&self) -> &str;

    /// Check if an entity type is valid for this domain
    fn is_valid_entity_type(&self, entity_type: &str) -> bool;

    /// Get domain-specific validation rules
    fn validation_rules(&self) -> &HashMap<String, String>;

    /// Get domain-specific properties
    fn domain_properties(&self) -> &Vec<String>;

    /// Initialize the domain with configuration
    fn initialize(&mut self, config: &DomainConfig) -> Result<()>;

    /// Shutdown and cleanup
    fn shutdown(&mut self) -> Result<()>;

    /// Validate entity data for this domain
    fn validate_entity(&self, entity_data: &EntityData) -> Result<ValidationResult>;

    /// Process entity data for this domain
    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity>;
}

/// Configuration for a domain
#[derive(Debug, Clone)]
pub struct DomainConfig {
    pub domain_id: String,
    pub name: String,
    pub description: String,
    pub core_ontology_path: String,
    pub domain_ontology_path: String,
    pub ontology_path: String,
    pub shacl_shapes_path: Option<String>,
    pub inference_rules_path: Option<String>,
    pub required_properties: Vec<String>,
    pub validation_queries: Vec<String>,
    pub enabled: bool,
    pub priority: u32,
    pub custom_properties: HashMap<String, String>,
}

impl Default for DomainConfig {
    fn default() -> Self {
        DomainConfig {
            domain_id: "generic".to_string(),
            name: "Generic Traceability".to_string(),
            description: "Generic traceability domain".to_string(),
            core_ontology_path: "ontologies/generic_core.owl".to_string(),
            domain_ontology_path: "ontologies/generic.owl".to_string(),
            ontology_path: "ontologies/core.owl".to_string(),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: Vec::new(),
            validation_queries: Vec::new(),
            enabled: true,
            priority: 1,
            custom_properties: HashMap::new(),
        }
    }
}

/// Validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Warning(String),
}

/// Processed entity data
#[derive(Debug, Clone)]
pub struct ProcessedEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub processed_data: String,
    pub domain_context: String,
}

/// Entity data for validation and processing
#[derive(Debug, Clone)]
pub struct EntityData {
    pub entity_id: String,
    pub entity_type: String,
    pub data: String,
    pub properties: HashMap<String, String>,
}

impl EntityData {
    /// Create a new entity data
    pub fn new(
        entity_id: String,
        entity_type: String,
        data: String,
        properties: HashMap<String, String>,
    ) -> Self {
        EntityData {
            entity_id,
            entity_type,
            data,
            properties,
        }
    }
}
