#[cfg(test)]
mod tests {
    use provchain_org::domain::DomainManager;
    // use provchain_org::domain::adapters::{SupplyChainAdapter, HealthcareAdapter, PharmaceuticalAdapter};
    use anyhow::Result;

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
        let config = Value::default();
        let adapter = SupplyChainAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "supplychain");
        assert_eq!(adapter.name(), "Supply Chain Traceability");
        assert_eq!(adapter.description(), "General supply chain and manufacturing traceability");
        Ok(())
    }

    #[test]
    fn test_healthcare_adapter_creation() -> Result<()> {
        let config = Value::default();
        let adapter = HealthcareAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "healthcare");
        assert_eq!(adapter.name(), "Healthcare Traceability");
        assert_eq!(adapter.description(), "Healthcare and medical traceability");
        Ok(())
    }

    #[test]
    fn test_pharmaceutical_adapter_creation() -> Result<()> {
        let config = Value::default();
        let adapter = PharmaceuticalAdapter::from_config(&config)?;
        assert_eq!(adapter.domain_id(), "pharmaceutical");
        assert_eq!(adapter.name(), "Pharmaceutical Traceability");
        assert_eq!(adapter.description(), "Pharmaceutical and drug traceability");
        Ok(())
    }

    #[test]
    fn test_domain_registration() -> Result<()> {
        let mut manager = DomainManager::new();

        let config = Value::default();
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
    */
}
