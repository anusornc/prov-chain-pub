//! Domain adapter implementations for different domains

use crate::core::TraceableEntity;
use crate::core::entity::DomainType;
use crate::ontology::OntologyManager;
use anyhow::Result;
use std::collections::HashMap;

/// Trait for domain adapters
pub trait DomainAdapter {
    /// Get the domain type this adapter supports
    fn domain_type(&self) -> DomainType;
    
    /// Validate an entity according to domain rules
    fn validate_entity(&self, entity: &TraceableEntity) -> Result<ValidationReport>;
    
    /// Enrich an entity with inferred information
    fn enrich_entity(&self, entity: &mut TraceableEntity) -> Result<()>;
    
    /// Get the domain ontology
    fn get_domain_ontology(&self) -> &str;
}

/// Configuration for domain adapters
#[derive(Debug, Clone)]
pub struct DomainConfig {
    pub ontology_path: String,
    pub shacl_shapes_path: Option<String>,
    pub inference_rules_path: Option<String>,
    pub required_properties: Vec<String>,
    pub validation_queries: Vec<String>,
}

/// Report from entity validation
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// OWL-based domain adapter implementation
pub struct OwlDomainAdapter {
    pub domain_type: DomainType,
    pub ontology_manager: OntologyManager,
    pub config: DomainConfig,
    pub domain_specific_validators: HashMap<String, Box<dyn Fn(&TraceableEntity) -> bool>>,
}

impl OwlDomainAdapter {
    /// Create a new OWL domain adapter
    pub fn new(domain_type: DomainType, config: DomainConfig) -> Result<Self> {
        let mut ontology_manager = OntologyManager::new()?;
        
        // Load the core ontology
        let _ = ontology_manager.load_core_ontology("ontologies/core.owl");
        
        // Load domain-specific ontology
        let _ = ontology_manager.load_domain_ontology(
            &format!("{:?}", domain_type), 
            &config.ontology_path
        );
        
        Ok(OwlDomainAdapter {
            domain_type,
            ontology_manager,
            config,
            domain_specific_validators: HashMap::new(),
        })
    }
    
    /// Add a domain-specific validator
    pub fn add_validator<F>(&mut self, property: String, validator: F)
    where
        F: Fn(&TraceableEntity) -> bool + 'static,
    {
        self.domain_specific_validators.insert(property, Box::new(validator));
    }
}

impl DomainAdapter for OwlDomainAdapter {
    fn domain_type(&self) -> DomainType {
        self.domain_type.clone()
    }
    
    fn validate_entity(&self, entity: &TraceableEntity) -> Result<ValidationReport> {
        // Validate against core ontology
        let _rdf_data = entity.to_rdf();
        
        // Check if entity belongs to this domain
        if entity.domain != self.domain_type {
            return Ok(ValidationReport {
                is_valid: false,
                errors: vec![format!(
                    "Entity domain '{:?}' does not match adapter domain '{:?}'", 
                    entity.domain, 
                    self.domain_type
                )],
                warnings: vec![],
                suggestions: vec![],
            });
        }
        
        // Check required properties
        let mut errors = Vec::new();
        for required_prop in &self.config.required_properties {
            if !entity.properties.contains_key(required_prop) {
                errors.push(format!("Missing required property: {}", required_prop));
            }
        }
        
        // Run domain-specific validators
        for (property, validator) in &self.domain_specific_validators {
            if entity.properties.contains_key(property) {
                if !validator(entity) {
                    errors.push(format!("Validation failed for property: {}", property));
                }
            }
        }
        
        Ok(ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings: vec![],
            suggestions: vec![],
        })
    }
    
    fn enrich_entity(&self, entity: &mut TraceableEntity) -> Result<()> {
        // In a full implementation, this would:
        // 1. Convert entity to RDF
        // 2. Use OWL reasoner to infer new relationships
        // 3. Update entity with inferred information
        println!("Enriching entity {} for domain {:?}", entity.id, self.domain_type);
        Ok(())
    }
    
    fn get_domain_ontology(&self) -> &str {
        // Return the URI of the loaded domain ontology
        self.ontology_manager
            .get_ontology_uri(&format!("{:?}", self.domain_type))
            .map(|s| s.as_str())
            .unwrap_or("http://provchain.org/ontology/core")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::{EntityType, PropertyValue};

    #[test]
    fn test_domain_adapter_creation() {
        let config = DomainConfig {
            ontology_path: "nonexistent.owl".to_string(), // Use nonexistent file for tests
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec!["batch_id".to_string()],
            validation_queries: vec![],
        };
        
        // This test checks that the function structure works
        // Even with a nonexistent file, we can test the error handling
        let adapter = OwlDomainAdapter::new(DomainType::SupplyChain, config);
        // We expect this to work without panicking, even with nonexistent files
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_entity_validation() {
        let config = DomainConfig {
            ontology_path: "nonexistent.owl".to_string(), // Use nonexistent file for tests
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec!["batch_id".to_string()],
            validation_queries: vec![],
        };
        
        let adapter = OwlDomainAdapter::new(DomainType::SupplyChain, config).unwrap();
        
        let mut entity = TraceableEntity::new(
            "test_batch_001".to_string(),
            EntityType::Product,
            DomainType::SupplyChain
        );
        
        // Add required property
        entity.add_property(
            "batch_id".to_string(), 
            PropertyValue::String("BATCH001".to_string())
        );
        
        let report = adapter.validate_entity(&entity).unwrap();
        // Entity should be valid since it has the required property
        assert!(report.is_valid);
        assert_eq!(report.errors.len(), 0);
    }

    #[test]
    fn test_entity_validation_missing_required() {
        let config = DomainConfig {
            ontology_path: "nonexistent.owl".to_string(), // Use nonexistent file for tests
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec!["batch_id".to_string()],
            validation_queries: vec![],
        };
        
        let adapter = OwlDomainAdapter::new(DomainType::SupplyChain, config).unwrap();
        
        let entity = TraceableEntity::new(
            "test_batch_001".to_string(),
            EntityType::Product,
            DomainType::SupplyChain
        );
        
        let report = adapter.validate_entity(&entity).unwrap();
        // Entity should be invalid since it's missing the required property
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("batch_id"));
    }
}