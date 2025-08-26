#[cfg(test)]
mod tests {
    use provchain_org::domain::{DomainManager, ValidationResult, EntityData};
    // use provchain_org::domain::adapters::{SupplyChainAdapter, HealthcareAdapter, PharmaceuticalAdapter};
    use anyhow::Result;
    use std::collections::HashMap;
    
    #[test]
    fn test_domain_manager_creation() -> Result<()> {
        let manager = DomainManager::new();
        assert_eq!(manager.plugins.len(), 0);
        assert!(manager.active_domain.is_none());
        Ok(())
    }
    
    /*
    #[test]
    fn test_supply_chain_adapter_creation() -> Result<()> {
        let config = serde_yaml::Value::default();
        let adapter = SupplyChainAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "supplychain");
        assert_eq!(adapter.name(), "Supply Chain Traceability");
        assert_eq!(adapter.description(), "General supply chain and manufacturing traceability");
        Ok(())
    }
    
    #[test]
    fn test_healthcare_adapter_creation() -> Result<()> {
        let config = serde_yaml::Value::default();
        let adapter = HealthcareAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "healthcare");
        assert_eq!(adapter.name(), "Healthcare Traceability");
        assert_eq!(adapter.description(), "Healthcare and medical traceability");
        Ok(())
    }
    
    #[test]
    fn test_pharmaceutical_adapter_creation() -> Result<()> {
        let config = serde_yaml::Value::default();
        let adapter = PharmaceuticalAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "pharmaceutical");
        assert_eq!(adapter.name(), "Pharmaceutical Traceability");
        assert_eq!(adapter.description(), "Pharmaceutical and drug traceability");
        Ok(())
    }
    
    #[test]
    fn test_domain_registration() -> Result<()> {
        let mut manager = DomainManager::new();
        
        let config = serde_yaml::Value::default();
        let supply_chain_adapter = Box::new(SupplyChainAdapter::from_config(&config)?);
        let healthcare_adapter = Box::new(HealthcareAdapter::from_config(&config)?);
        let pharmaceutical_adapter = Box::new(PharmaceuticalAdapter::from_config(&config)?);
        
        manager.register_plugin(supply_chain_adapter)?;
        manager.register_plugin(healthcare_adapter)?;
        manager.register_plugin(pharmaceutical_adapter)?;
        
        assert_eq!(manager.plugins.len(), 3);
        assert!(manager.plugins.contains_key("supplychain"));
        assert!(manager.plugins.contains_key("healthcare"));
        assert!(manager.plugins.contains_key("pharmaceutical"));
        
        Ok(())
    }
    
    #[test]
    fn test_domain_activation() -> Result<()> {
        let mut manager = DomainManager::new();
        
        let config = serde_yaml::Value::default();
        let supply_chain_adapter = Box::new(SupplyChainAdapter::from_config(&config)?);
        manager.register_plugin(supply_chain_adapter)?;
        
        // Initially no active domain
        assert!(manager.active_domain.is_none());
        assert!(manager.get_active_domain().is_none());
        
        // Set active domain
        manager.set_active_domain("supplychain")?;
        assert_eq!(manager.active_domain, Some("supplychain".to_string()));
        assert!(manager.get_active_domain().is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_entity_validation() -> Result<()> {
        let mut manager = DomainManager::new();
        
        let config = serde_yaml::Value::default();
        let supply_chain_adapter = Box::new(SupplyChainAdapter::from_config(&config)?);
        manager.register_plugin(supply_chain_adapter)?;
        manager.set_active_domain("supplychain")?;
        
        // Create test entity data
        let mut properties = HashMap::new();
        properties.insert("hasBatchID".to_string(), "BATCH001".to_string());
        properties.insert("originFarm".to_string(), "Farm A".to_string());
        
        let entity_data = EntityData::new(
            "test_batch_001".to_string(),
            "ProductBatch".to_string(),
            "test data".to_string(),
            properties,
        );
        
        // Validate entity
        let result = manager.validate_entity_for_active_domain(&entity_data)?;
        assert_eq!(result, ValidationResult::Valid);
        
        Ok(())
    }
    
    #[test]
    fn test_entity_processing() -> Result<()> {
        let mut manager = DomainManager::new();
        
        let config = serde_yaml::Value::default();
        let supply_chain_adapter = Box::new(SupplyChainAdapter::from_config(&config)?);
        manager.register_plugin(supply_chain_adapter)?;
        manager.set_active_domain("supplychain")?;
        
        // Create test entity data
        let mut properties = HashMap::new();
        properties.insert("hasBatchID".to_string(), "BATCH001".to_string());
        properties.insert("originFarm".to_string(), "Farm A".to_string());
        
        let entity_data = EntityData::new(
            "test_batch_001".to_string(),
            "ProductBatch".to_string(),
            "test data".to_string(),
            properties,
        );
        
        // Process entity
        let result = manager.process_entity_for_active_domain(&entity_data)?;
        assert_eq!(result.entity_id, "test_batch_001");
        assert_eq!(result.entity_type, "ProductBatch");
        assert_eq!(result.domain_context, "supplychain");
        assert!(!result.processed_data.is_empty());
        
        Ok(())
    }
    */
    
    #[test]
    fn test_generic_validation() -> Result<()> {
        let manager = DomainManager::new();
        
        // Create test entity data
        let mut properties = HashMap::new();
        properties.insert("testProperty".to_string(), "testValue".to_string());
        
        let entity_data = EntityData::new(
            "test_entity_001".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            properties,
        );
        
        // Validate entity with no active domain (should use generic validation)
        let result = manager.validate_entity_for_active_domain(&entity_data)?;
        assert_eq!(result, ValidationResult::Valid);
        
        // Test with missing entity ID
        let entity_data_no_id = EntityData::new(
            "".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            HashMap::new(),
        );
        
        let result = manager.validate_entity_for_active_domain(&entity_data_no_id)?;
        assert_eq!(result, ValidationResult::Invalid("Entity ID is required".to_string()));
        
        // Test with missing entity type
        let entity_data_no_type = EntityData::new(
            "test_entity_001".to_string(),
            "".to_string(),
            "test data".to_string(),
            HashMap::new(),
        );
        
        let result = manager.validate_entity_for_active_domain(&entity_data_no_type)?;
        assert_eq!(result, ValidationResult::Invalid("Entity type is required".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_generic_processing() -> Result<()> {
        let manager = DomainManager::new();
        
        // Create test entity data
        let mut properties = HashMap::new();
        properties.insert("testProperty".to_string(), "testValue".to_string());
        
        let entity_data = EntityData::new(
            "test_entity_001".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            properties,
        );
        
        // Process entity with no active domain (should use generic processing)
        let result = manager.process_entity_for_active_domain(&entity_data)?;
        assert_eq!(result.entity_id, "test_entity_001");
        assert_eq!(result.entity_type, "TestEntity");
        assert_eq!(result.domain_context, "generic");
        assert_eq!(result.processed_data, "test data");
        
        Ok(())
    }
}