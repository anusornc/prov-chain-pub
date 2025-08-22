#[cfg(test)]
mod tests {
    use anyhow::Result;
    use provchain_org::semantic::{OwlReasonerConfig, OwlReasoner, Owl2EnhancedReasoner, ValidationResult};
    
    #[test]
    fn test_enhanced_owl2_feature_processing() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;
        config.enable_property_chain_inference = true;
        config.enable_qualified_cardinality_validation = true;
        
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Test that OWL2 feature processing doesn't crash
        let result = reasoner.process_owl2_features();
        assert!(result.is_ok());
        
        // Test that hasKey processing doesn't crash
        let has_key_result = reasoner.process_has_key_axioms();
        assert!(has_key_result.is_ok());
        
        // Test that property chain processing doesn't crash
        let chain_result = reasoner.process_property_chain_axioms();
        assert!(chain_result.is_ok());
        
        // Test that qualified cardinality processing doesn't crash
        let cardinality_result = reasoner.process_qualified_cardinality_restrictions();
        assert!(cardinality_result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_owl2_entity_validation() -> Result<()> {
        let config = OwlReasonerConfig::default();
        let reasoner = OwlReasoner::new(config)?;
        
        // Test entity uniqueness validation
        let entity_data = "test entity data";
        let result = reasoner.validate_entity_uniqueness(entity_data)?;
        // Should be valid when no hasKey constraints are defined
        assert_eq!(result, ValidationResult::Valid);
        
        // Test qualified cardinality validation
        let cardinality_result = reasoner.validate_qualified_cardinality(entity_data)?;
        // Should be valid when no qualified cardinality restrictions are defined
        assert_eq!(cardinality_result, ValidationResult::Valid);
        
        Ok(())
    }
    
    #[test]
    fn test_owl2_property_chain_inference() -> Result<()> {
        let config = OwlReasonerConfig::default();
        let mut reasoner = OwlReasoner::new(config)?;
        
        // Test property chain inference
        let graph_data = "test graph data";
        let result = reasoner.apply_property_chain_inference(graph_data)?;
        // Should return empty inferred graph when no property chains are defined
        assert_eq!(result.triples().len(), 0);
        
        Ok(())
    }
}