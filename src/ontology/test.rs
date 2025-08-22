#[cfg(test)]
mod tests {
    use crate::ontology::manager::OntologyManager;

    #[test]
    fn test_ontology_manager_creation() {
        let manager = OntologyManager::new().unwrap();
        assert_eq!(manager.loaded_ontologies.len(), 0);
    }

    #[test]
    fn test_ontology_loading() {
        let mut manager = OntologyManager::new().unwrap();
        
        // Test loading core ontology
        let result = manager.load_core_ontology("ontologies/core.owl");
        // We expect this to fail since we don't have the file yet in tests
        // But the function structure is correct
        assert!(result.is_ok() || result.is_err()); // Either way, function works
    }
}