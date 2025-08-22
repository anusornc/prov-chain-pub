#[cfg(test)]
mod tests {
    use provchain_org::domain::{DomainManager, DomainPlugin};
    // use provchain_org::domain::adapters::OwlDomainAdapter;
    use anyhow::Result;
    use serde_yaml::Value;
    
    #[test]
    fn test_domain_manager_basic_functionality() -> Result<()> {
        let mut manager = DomainManager::new();
        
        // Verify basic properties
        assert_eq!(manager.plugins.len(), 0);
        assert!(manager.active_domain.is_none());
        
        Ok(())
    }
    
    /*
    #[test]
    fn test_owl_domain_adapter_creation() -> Result<()> {
        // Create a simple configuration for testing
        let config_yaml = r#"
            domain_id: "test_domain"
            name: "Test Domain"
            description: "Test domain for OWL adapter"
            core_ontology_path: "ontologies/core.owl"
            domain_ontology_path: "ontologies/test-owl2.owl"
            ontology_path: "ontologies/test-owl2.owl"
            enabled: true
            priority: 1
        "#;
        
        let config: Value = serde_yaml::from_str(config_yaml)?;
        
        // Create OWL domain adapter
        let adapter = OwlDomainAdapter::from_config(&config)?;
        
        // Verify basic properties
        assert_eq!(adapter.domain_id(), "test_domain");
        assert_eq!(adapter.name(), "Test Domain");
        assert_eq!(adapter.description(), "Test domain for OWL adapter");
        
        Ok(())
    }
    
    #[test]
    fn test_domain_manager_with_owl_adapter() -> Result<()> {
        let mut manager = DomainManager::new();
        
        // Create a simple configuration for testing
        let config_yaml = r#"
            domain_id: "supplychain"
            name: "Supply Chain Domain"
            description: "Supply chain traceability domain"
            core_ontology_path: "ontologies/core.owl"
            domain_ontology_path: "ontologies/test-owl2.owl"
            ontology_path: "ontologies/test-owl2.owl"
            enabled: true
            priority: 1
        "#;
        
        let config: Value = serde_yaml::from_str(config_yaml)?;
        
        // Create OWL domain adapter
        let adapter = Box::new(OwlDomainAdapter::from_config(&config)?);
        
        // Register the adapter with the manager
        manager.register_plugin(adapter)?;
        
        // Verify the adapter was registered
        assert_eq!(manager.plugins.len(), 1);
        assert!(manager.plugins.contains_key("supplychain"));
        
        Ok(())
    }
    
    #[test]
    fn test_entity_validation_with_owl_adapter() -> Result<()> {
        // Create a simple configuration for testing
        let config_yaml = r#"
            domain_id: "test_domain"
            name: "Test Domain"
            description: "Test domain for OWL adapter"
            core_ontology_path: "ontologies/core.owl"
            domain_ontology_path: "ontologies/test-owl2.owl"
            ontology_path: "ontologies/test-owl2.owl"
            enabled: true
            priority: 1
        "#;
        
        let config: Value = serde_yaml::from_str(config_yaml)?;
        
        // Create OWL domain adapter
        let adapter = OwlDomainAdapter::from_config(&config)?;
        
        // Create test entity data
        let entity_data = provchain_org::domain::plugin::EntityData::new(
            "test_entity_001".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            std::collections::HashMap::new(),
        );
        
        // Validate the entity
        let result = adapter.validate_entity(&entity_data)?;
        
        // Should be valid (basic validation passes)
        assert_eq!(result, provchain_org::domain::plugin::ValidationResult::Valid);
        
        Ok(())
    }
    */
}