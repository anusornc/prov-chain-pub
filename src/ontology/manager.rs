//! Ontology manager for loading and managing multiple domain ontologies

use crate::storage::rdf_store::RDFStore;
use oxigraph::model::NamedNode;
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

/// Manages multiple ontologies for different domains
pub struct OntologyManager {
    pub rdf_store: RDFStore,
    pub loaded_ontologies: HashMap<String, LoadedOntology>,
}

/// Represents a loaded ontology with its metadata
pub struct LoadedOntology {
    pub ontology_uri: String,
    pub shacl_shapes_graph: Option<String>,
    pub rules_graph: Option<String>,
    pub domain: String,
}

impl OntologyManager {
    /// Create a new ontology manager
    pub fn new() -> Result<Self> {
        Ok(OntologyManager {
            rdf_store: RDFStore::new(),
            loaded_ontologies: HashMap::new(),
        })
    }

    /// Load a core ontology from file
    pub fn load_core_ontology<P: AsRef<Path>>(&mut self, ontology_path: P) -> Result<()> {
        let ontology_data = std::fs::read_to_string(ontology_path)?;
        let graph_name = NamedNode::new("http://provchain.org/ontology/core")?;
        self.rdf_store.load_ontology(&ontology_data, &graph_name);
        
        self.loaded_ontologies.insert("core".to_string(), LoadedOntology {
            ontology_uri: graph_name.as_str().to_string(),
            shacl_shapes_graph: None,
            rules_graph: None,
            domain: "core".to_string(),
        });
        
        Ok(())
    }

    /// Load a domain ontology from file
    pub fn load_domain_ontology<P: AsRef<Path>>(&mut self, domain: &str, ontology_path: P) -> Result<()> {
        let ontology_data = std::fs::read_to_string(ontology_path)?;
        let graph_name = NamedNode::new(format!("http://provchain.org/ontology/{}", domain))?;
        self.rdf_store.load_ontology(&ontology_data, &graph_name);
        
        self.loaded_ontologies.insert(domain.to_string(), LoadedOntology {
            ontology_uri: graph_name.as_str().to_string(),
            shacl_shapes_graph: None,
            rules_graph: None,
            domain: domain.to_string(),
        });
        
        Ok(())
    }

    /// Check if an ontology is loaded for a domain
    pub fn is_ontology_loaded(&self, domain: &str) -> bool {
        self.loaded_ontologies.contains_key(domain)
    }

    /// Get the ontology URI for a domain
    pub fn get_ontology_uri(&self, domain: &str) -> Option<&String> {
        self.loaded_ontologies.get(domain).map(|loaded| &loaded.ontology_uri)
    }
}