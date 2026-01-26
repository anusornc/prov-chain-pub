#[cfg(test)]
mod tests {
    use crate::core::TraceableEntity;
    use crate::core::entity::{EntityType, DomainType};
    use crate::domain::{DomainConfig, OwlDomainAdapter, DomainAdapter};
    use std::fs;

    #[test]
    fn test_ontology_files_exist() {
        // Check that our ontology files were created
        assert!(fs::metadata("src/semantic/ontologies/generic_core.owl").is_ok());
        assert!(fs::metadata("src/semantic/ontologies/healthcare.owl").is_ok());
        assert!(fs::metadata("src/semantic/ontologies/pharmaceutical.owl").is_ok());
        assert!(fs::metadata("src/semantic/ontologies/automotive.owl").is_ok());
        assert!(fs::metadata("src/semantic/ontologies/digital_assets.owl").is_ok());
    }

    #[test]
    fn test_domain_adapter_creation() {
        let config = DomainConfig {
            ontology_path: "src/semantic/ontologies/healthcare.owl".to_string(),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec!["patientID".to_string()],
            validation_queries: vec![],
        };
        
        let adapter = OwlDomainAdapter::new(DomainType::Healthcare, config);
        // This should work even without loading the actual ontology in tests
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_entity_domain_validation() {
        let config = DomainConfig {
            ontology_path: "src/semantic/ontologies/healthcare.owl".to_string(),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec![],
            validation_queries: vec![],
        };
        
        let adapter = OwlDomainAdapter::new(DomainType::Healthcare, config).unwrap();
        
        // Create an entity with the wrong domain
        let entity = TraceableEntity::new(
            "test_entity".to_string(),
            EntityType::Product,
            DomainType::SupplyChain  // Wrong domain
        );
        
        let report = adapter.validate_entity(&entity).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("domain")));
    }
}