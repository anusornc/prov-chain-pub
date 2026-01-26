use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ontology Manager Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    /// Path to the main ontology file
    pub main_ontology_path: String,
    
    /// Graph IRI for the main ontology
    pub main_ontology_graph: String,
    
    /// Domain-specific ontology configurations
    pub domain_ontologies: HashMap<String, DomainOntologyConfig>,
    
    /// Whether to automatically load ontologies on startup
    pub auto_load: bool,
    
    /// Whether to validate RDF data against ontologies
    pub validate_data: bool,
    
    /// Default namespace mappings
    pub namespace_mappings: HashMap<String, String>,
    
    /// Ontology resolution strategy
    pub resolution_strategy: OntologyResolutionStrategy,
}

/// Configuration for domain-specific ontologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainOntologyConfig {
    /// Path to the domain ontology file
    pub path: String,
    
    /// Graph IRI for the domain ontology
    pub graph_iri: String,
    
    /// Whether this domain is enabled
    pub enabled: bool,
    
    /// Priority for this domain (higher = loaded first)
    pub priority: u32,
}

/// Strategy for resolving ontologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OntologyResolutionStrategy {
    /// Load from file system
    FileSystem,
    
    /// Load from HTTP/HTTPS URLs
    Http,
    
    /// Load from embedded resources
    Embedded,
    
    /// Auto-detect (try multiple strategies)
    Auto,
}

impl Default for OntologyConfig {
    fn default() -> Self {
        Self {
            main_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            main_ontology_graph: "http://provchain.org/ontology/core".to_string(),
            domain_ontologies: HashMap::new(),
            auto_load: true,
            validate_data: false,
            namespace_mappings: {
                let mut mappings = HashMap::new();
                mappings.insert("core".to_string(), "http://provchain.org/core#".to_string());
                mappings.insert("prov".to_string(), "http://www.w3.org/ns/prov#".to_string());
                mappings.insert("xsd".to_string(), "http://www.w3.org/2001/XMLSchema#".to_string());
                mappings.insert("rdfs".to_string(), "http://www.w3.org/2000/01/rdf-schema#".to_string());
                mappings.insert("owl".to_string(), "http://www.w3.org/2002/07/owl#".to_string());
                mappings
            },
            resolution_strategy: OntologyResolutionStrategy::FileSystem,
        }
    }
}