#[cfg(test)]
mod tests {
    use anyhow::Result;
    use provchain_org::semantic::owl2_enhanced_reasoner::{Owl2EnhancedReasoner, ValidationResult};
    use provchain_org::semantic::owl_reasoner::OwlReasonerConfig;

    #[test]
    fn test_enhanced_owl2_feature_processing() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;
        config.enable_property_chain_inference = true;
        config.enable_qualified_cardinality_validation = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process OWL2 features
        reasoner.process_owl2_features()?;

        // For now, we'll just check that the method doesn't panic
        // In a real implementation, we'd check the internal state

        Ok(())
    }

    #[test]
    fn test_has_key_constraint_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process hasKey axioms
        reasoner.process_has_key_axioms()?;

        // For now, we'll just check that the method doesn't panic
        // In a real implementation, we'd check the internal state

        Ok(())
    }

    #[test]
    fn test_property_chain_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_property_chain_inference = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process property chain axioms
        reasoner.process_property_chain_axioms()?;

        // For now, we'll just check that the method doesn't panic
        // In a real implementation, we'd check the internal state

        Ok(())
    }

    #[test]
    fn test_qualified_cardinality_extraction() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_qualified_cardinality_validation = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process qualified cardinality restrictions
        reasoner.process_qualified_cardinality_restrictions()?;

        // For now, we'll just check that the method doesn't panic
        // In a real implementation, we'd check the internal state

        Ok(())
    }

    #[test]
    fn test_owl2_entity_validation() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_has_key_validation = true;
        config.enable_qualified_cardinality_validation = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process OWL2 features
        reasoner.process_owl2_features()?;

        // Test entity validation (should be valid when no conflicting entities exist)
        let entity_data = "test entity data";
        let result = reasoner.validate_entity_uniqueness(entity_data)?;
        assert_eq!(result, ValidationResult::Valid);

        // Test qualified cardinality validation (should be valid when no constraints violated)
        let cardinality_result = reasoner.validate_qualified_cardinality(entity_data)?;
        assert_eq!(cardinality_result, ValidationResult::Valid);

        Ok(())
    }

    #[test]
    fn test_owl2_property_chain_inference() -> Result<()> {
        let mut config = OwlReasonerConfig::default();
        config.ontology_path = "src/semantic/ontologies/test-owl2.owl".to_string();
        config.process_owl2_features = true;
        config.enable_property_chain_inference = true;

        let mut reasoner = Owl2EnhancedReasoner::new(config)?;

        // Process OWL2 features
        reasoner.process_owl2_features()?;

        // Test property chain inference
        let graph_data = "test graph data";
        let inferred_graph = reasoner.apply_property_chain_inference(graph_data)?;

        // Should return an inferred graph (may be empty if no inferences can be made)
        assert_eq!(inferred_graph.triples().len(), 0); // Empty for now in our simple test

        Ok(())
    }
}
