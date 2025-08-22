#[cfg(test)]
mod tests {
    use provchain_org::semantic::owl_reasoner::{OwlReasoner, OwlReasonerConfig};
    use anyhow::Result;
    
    #[test]
    fn test_owl2_feature_processing_with_test_ontology() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;
        config.enable_property_chain_inference = true;
        config.enable_qualified_cardinality_validation = true;
        
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Process OWL2 features
        let result = reasoner.process_owl2_features();
        assert!(result.is_ok());
        
        // Check that hasKey constraints were processed
        assert!(!reasoner.has_key_constraints.is_empty());
        
        // Check that property chains were processed
        assert!(!reasoner.property_chains.is_empty());
        
        // Check that qualified cardinality restrictions were processed
        assert!(!reasoner.qualified_cardinality_restrictions.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_has_key_constraint_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;
        
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Process hasKey axioms
        let result = reasoner.process_has_key_axioms();
        assert!(result.is_ok());
        
        // Check specific hasKey constraints
        // Batch should have batchId as key
        assert!(reasoner.has_key_constraints.contains_key("http://provchain.org/test-owl2#Batch"));
        if let Some(keys) = reasoner.has_key_constraints.get("http://provchain.org/test-owl2#Batch") {
            assert!(keys.contains(&"http://provchain.org/test-owl2#batchId".to_string()));
        }
        
        // ProductBatch should have productId and batchNumber as composite key
        assert!(reasoner.has_key_constraints.contains_key("http://provchain.org/test-owl2#ProductBatch"));
        if let Some(keys) = reasoner.has_key_constraints.get("http://provchain.org/test-owl2#ProductBatch") {
            assert!(keys.contains(&"http://provchain.org/test-owl2#productId".to_string()));
            assert!(keys.contains(&"http://provchain.org/test-owl2#batchNumber".to_string()));
        }
        
        Ok(())
    }
    
    #[test]
    fn test_property_chain_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_property_chain_inference = true;
        
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Process property chain axioms
        let result = reasoner.process_property_chain_axioms();
        assert!(result.is_ok());
        
        // Check property chains
        assert!(reasoner.property_chains.contains_key("http://provchain.org/test-owl2#transitivelySuppliedTo"));
        
        Ok(())
    }
    
    #[test]
    fn test_qualified_cardinality_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_qualified_cardinality_validation = true;
        
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Process qualified cardinality restrictions
        let result = reasoner.process_qualified_cardinality_restrictions();
        assert!(result.is_ok());
        
        // Check that we extracted some qualified cardinality restrictions
        assert!(!reasoner.qualified_cardinality_restrictions.is_empty());
        
        Ok(())
    }
}