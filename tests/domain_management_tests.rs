#[cfg(test)]
mod tests {
    use anyhow::Result;
    use provchain_org::domain::adapters::{
        HealthcareAdapter, PharmaceuticalAdapter, SupplyChainAdapter,
    };
    use provchain_org::domain::{DomainManager, DomainPlugin, EntityData, ValidationResult};
    use std::collections::HashMap;

    #[test]
    fn test_domain_manager_creation() -> Result<()> {
        let manager = DomainManager::new();
        assert_eq!(manager.plugins.len(), 0);
        assert!(manager.active_domain.is_none());
        Ok(())
    }

    #[test]
    fn test_concrete_adapter_creation() -> Result<()> {
        let config = serde_yaml::Value::default();

        let supply_chain = SupplyChainAdapter::from_config(&config)?;
        assert_eq!(supply_chain.domain_id(), "supplychain");
        assert_eq!(supply_chain.name(), "Supply Chain Traceability");

        let healthcare = HealthcareAdapter::from_config(&config)?;
        assert_eq!(healthcare.domain_id(), "healthcare");
        assert_eq!(healthcare.name(), "Healthcare Traceability");

        let pharmaceutical = PharmaceuticalAdapter::from_config(&config)?;
        assert_eq!(pharmaceutical.domain_id(), "pharmaceutical");
        assert_eq!(pharmaceutical.name(), "Pharmaceutical Traceability");

        Ok(())
    }

    #[test]
    fn test_domain_manager_loads_supported_domains() -> Result<()> {
        let mut manager = DomainManager::new();
        let config = serde_yaml::Value::default();

        manager.load_domain_plugin("supplychain", &config)?;
        manager.load_domain_plugin("healthcare", &config)?;
        manager.load_domain_plugin("pharmaceutical", &config)?;

        assert_eq!(manager.plugins.len(), 3);
        assert!(manager.plugins.contains_key("supplychain"));
        assert!(manager.plugins.contains_key("healthcare"));
        assert!(manager.plugins.contains_key("pharmaceutical"));

        Ok(())
    }

    #[test]
    fn test_supply_chain_validation_and_processing() -> Result<()> {
        let mut manager = DomainManager::new();
        manager.load_domain_plugin("supplychain", &serde_yaml::Value::default())?;
        manager.set_active_domain("supplychain")?;

        let mut properties = HashMap::new();
        properties.insert("hasBatchID".to_string(), "BATCH001".to_string());
        properties.insert("originFarm".to_string(), "Farm A".to_string());

        let entity_data = EntityData::new(
            "test_batch_001".to_string(),
            "ProductBatch".to_string(),
            "test data".to_string(),
            properties,
        );

        assert_eq!(
            manager.validate_entity_for_active_domain(&entity_data)?,
            ValidationResult::Valid
        );

        let processed = manager.process_entity_for_active_domain(&entity_data)?;
        assert_eq!(processed.domain_context, "supplychain");
        assert!(processed.processed_data.contains("Supply chain context"));

        Ok(())
    }

    #[test]
    fn test_healthcare_and_pharmaceutical_validation() -> Result<()> {
        let mut manager = DomainManager::new();
        let config = serde_yaml::Value::default();
        manager.load_domain_plugin("healthcare", &config)?;
        manager.load_domain_plugin("pharmaceutical", &config)?;

        manager.set_active_domain("healthcare")?;
        let mut healthcare_properties = HashMap::new();
        healthcare_properties.insert("medicalDeviceSerial".to_string(), "MD-1001".to_string());
        let healthcare_entity = EntityData::new(
            "device_001".to_string(),
            "MedicalDevice".to_string(),
            "device data".to_string(),
            healthcare_properties,
        );
        assert_eq!(
            manager.validate_entity_for_active_domain(&healthcare_entity)?,
            ValidationResult::Valid
        );

        manager.set_active_domain("pharmaceutical")?;
        let mut pharmaceutical_properties = HashMap::new();
        pharmaceutical_properties.insert("hasDrugID".to_string(), "DRUG-001".to_string());
        pharmaceutical_properties.insert("hasBatchNumber".to_string(), "LOT-2026-001".to_string());
        let pharmaceutical_entity = EntityData::new(
            "drug_batch_001".to_string(),
            "DrugBatch".to_string(),
            "drug data".to_string(),
            pharmaceutical_properties,
        );
        assert_eq!(
            manager.validate_entity_for_active_domain(&pharmaceutical_entity)?,
            ValidationResult::Valid
        );

        Ok(())
    }

    #[test]
    fn test_generic_validation_and_processing() -> Result<()> {
        let manager = DomainManager::new();

        let mut properties = HashMap::new();
        properties.insert("testProperty".to_string(), "testValue".to_string());

        let entity_data = EntityData::new(
            "test_entity_001".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            properties,
        );

        assert_eq!(
            manager.validate_entity_for_active_domain(&entity_data)?,
            ValidationResult::Valid
        );

        let entity_data_no_id = EntityData::new(
            "".to_string(),
            "TestEntity".to_string(),
            "test data".to_string(),
            HashMap::new(),
        );
        assert_eq!(
            manager.validate_entity_for_active_domain(&entity_data_no_id)?,
            ValidationResult::Invalid("Entity ID is required".to_string())
        );

        let processed = manager.process_entity_for_active_domain(&entity_data)?;
        assert_eq!(processed.domain_context, "generic");
        assert_eq!(processed.processed_data, "test data");

        Ok(())
    }
}
