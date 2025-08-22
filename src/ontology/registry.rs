//! Ontology registry for managing available ontologies

use std::collections::HashMap;

/// Registry of available ontologies
pub struct OntologyRegistry {
    pub available_ontologies: HashMap<String, OntologyInfo>,
}

/// Information about an ontology
pub struct OntologyInfo {
    pub name: String,
    pub description: String,
    pub path: String,
    pub version: String,
    pub domain: String,
}

impl OntologyRegistry {
    /// Create a new ontology registry
    pub fn new() -> Self {
        OntologyRegistry {
            available_ontologies: HashMap::new(),
        }
    }

    /// Register a new ontology
    pub fn register_ontology(&mut self, name: String, info: OntologyInfo) {
        self.available_ontologies.insert(name, info);
    }

    /// Get information about an ontology
    pub fn get_ontology_info(&self, name: &str) -> Option<&OntologyInfo> {
        self.available_ontologies.get(name)
    }

    /// List all available ontologies
    pub fn list_ontologies(&self) -> Vec<&String> {
        self.available_ontologies.keys().collect()
    }

    /// List ontologies for a specific domain
    pub fn list_domain_ontologies(&self, domain: &str) -> Vec<&String> {
        self.available_ontologies
            .iter()
            .filter(|(_, info)| info.domain == domain)
            .map(|(name, _)| name)
            .collect()
    }
}