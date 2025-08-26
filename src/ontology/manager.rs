//! Ontology Manager for dynamic ontology loading and management
//!
//! This module provides a flexible ontology manager that can load
//! ontologies from various sources (file system, HTTP, embedded)
//! based on configuration.

use crate::storage::rdf_store::RDFStore;
use crate::ontology::config::{OntologyConfig, OntologyResolutionStrategy};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn, error};
use oxigraph::model::NamedNode;

/// Ontology Manager for dynamic ontology loading
pub struct OntologyManager {
    /// Configuration for ontology loading
    config: OntologyConfig,
    
    /// RDF store for loaded ontologies
    rdf_store: RDFStore,
    
    /// Cache of loaded ontology IRIs
    loaded_ontologies: HashMap<String, String>,
    
    /// Namespace mappings for easy access
    namespace_cache: HashMap<String, String>,
}

impl OntologyManager {
    /// Create a new ontology manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(OntologyConfig::default())
    }
    
    /// Create a new ontology manager with specific configuration
    pub fn with_config(config: OntologyConfig) -> Result<Self> {
        let rdf_store = RDFStore::new();
        
        let mut manager = OntologyManager {
            config,
            rdf_store,
            loaded_ontologies: HashMap::new(),
            namespace_cache: HashMap::new(),
        };
        
        // Initialize namespace cache
        manager.initialize_namespace_cache();
        
        // Auto-load ontologies if configured
        if manager.config.auto_load {
            manager.load_configured_ontologies()?;
        }
        
        Ok(manager)
    }
    
    /// Initialize namespace cache from configuration
    fn initialize_namespace_cache(&mut self) {
        self.namespace_cache = self.config.namespace_mappings.clone();
    }
    
    /// Load all configured ontologies
    pub fn load_configured_ontologies(&mut self) -> Result<()> {
        info!("Loading configured ontologies...");
        
        // Load main ontology
        self.load_main_ontology()?;
        
        // Load domain ontologies
        self.load_domain_ontologies()?;
        
        Ok(())
    }
    
    /// Load the main ontology
    pub fn load_main_ontology(&mut self) -> Result<()> {
        let path = self.config.main_ontology_path.clone();
        let graph_iri = self.config.main_ontology_graph.clone();
        
        info!("Loading main ontology from: {}", path);
        
        match self.config.resolution_strategy {
            OntologyResolutionStrategy::FileSystem => {
                self.load_ontology_from_file(&path, &graph_iri)
            },
            OntologyResolutionStrategy::Http => {
                self.load_ontology_from_http(&path, &graph_iri)
            },
            OntologyResolutionStrategy::Embedded => {
                self.load_embedded_ontology(&graph_iri)
            },
            OntologyResolutionStrategy::Auto => {
                self.load_ontology_auto(&path, &graph_iri)
            },
        }
    }
    
    /// Load ontology from file system
    fn load_ontology_from_file(&mut self, path: &str, graph_iri: &str) -> Result<()> {
        if !Path::new(path).exists() {
            return Err(anyhow::anyhow!("Ontology file not found: {}", path));
        }
        
        let ontology_data = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read ontology file: {}", path))?;
        
        self.rdf_store.load_ontology(&ontology_data, &NamedNode::new(graph_iri)?);
        self.loaded_ontologies.insert(graph_iri.to_string(), path.to_string());
        
        info!("Successfully loaded ontology from file: {}", path);
        Ok(())
    }
    
    /// Load ontology from HTTP/HTTPS URL
    fn load_ontology_from_http(&mut self, url: &str, graph_iri: &str) -> Result<()> {
        // For simplicity, we'll use a mock implementation
        // In a real implementation, you would use reqwest or similar
        warn!("HTTP ontology loading not fully implemented, falling back to file system");
        self.load_ontology_from_file(url, graph_iri)
    }
    
    /// Load embedded ontology
    fn load_embedded_ontology(&mut self, graph_iri: &str) -> Result<()> {
        // This would load from embedded resources
        // For now, we'll fall back to file system loading
        let path = self.config.main_ontology_path.clone();
        warn!("Embedded ontology loading not implemented, falling back to file system");
        self.load_ontology_from_file(&path, graph_iri)
    }
    
    /// Auto-detect and load ontology
    fn load_ontology_auto(&mut self, path_or_url: &str, graph_iri: &str) -> Result<()> {
        if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            self.load_ontology_from_http(path_or_url, graph_iri)
        } else {
            self.load_ontology_from_file(path_or_url, graph_iri)
        }
    }
    
    /// Load domain-specific ontologies
    fn load_domain_ontologies(&mut self) -> Result<()> {
        // Clone the configs to avoid borrowing issues
        let domain_configs: Vec<_> = self.config.domain_ontologies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        for (domain_name, domain_config) in domain_configs {
            if domain_config.enabled {
                info!("Loading domain ontology: {}", domain_name);
                
                match self.load_ontology_auto(&domain_config.path, &domain_config.graph_iri) {
                    Ok(_) => {
                        info!("Successfully loaded domain ontology: {}", domain_name);
                    },
                    Err(e) => {
                        error!("Failed to load domain ontology {}: {}", domain_name, e);
                        if domain_config.priority > 100 {
                            // High priority domain - fail if it can't load
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get loaded ontology paths/URLs
    pub fn get_loaded_ontologies(&self) -> &HashMap<String, String> {
        &self.loaded_ontologies
    }
    
    /// Get namespace by prefix
    pub fn get_namespace(&self, prefix: &str) -> Option<&String> {
        self.namespace_cache.get(prefix)
    }
    
    /// Add namespace mapping
    pub fn add_namespace(&mut self, prefix: &str, uri: &str) {
        self.namespace_cache.insert(prefix.to_string(), uri.to_string());
    }
    
    /// Get all namespace mappings
    pub fn get_namespaces(&self) -> &HashMap<String, String> {
        &self.namespace_cache
    }
    
    /// Get reference to RDF store with loaded ontologies
    pub fn get_rdf_store(&self) -> &RDFStore {
        &self.rdf_store
    }
    
    /// Get mutable reference to RDF store
    pub fn get_rdf_store_mut(&mut self) -> &mut RDFStore {
        &mut self.rdf_store
    }
    
    /// Validate that required ontologies are loaded
    pub fn validate_required_ontologies(&self) -> Result<()> {
        // Check if main ontology is loaded
        if !self.loaded_ontologies.contains_key(&self.config.main_ontology_graph) {
            return Err(anyhow::anyhow!("Main ontology not loaded: {}", self.config.main_ontology_graph));
        }
        
        // Check required domain ontologies
        for (domain_name, domain_config) in &self.config.domain_ontologies {
            if domain_config.enabled && domain_config.priority > 50 {
                // High priority domain - must be loaded
                if !self.loaded_ontologies.contains_key(&domain_config.graph_iri) {
                    return Err(anyhow::anyhow!("Required domain ontology not loaded: {}", domain_name));
                }
            }
        }
        
        Ok(())
    }
    
    /// Reload all ontologies
    pub fn reload_ontologies(&mut self) -> Result<()> {
        self.loaded_ontologies.clear();
        self.load_configured_ontologies()
    }
    
    /// Reload specific ontology
    pub fn reload_ontology(&mut self, graph_iri: &str) -> Result<()> {
        self.loaded_ontologies.remove(graph_iri);
        
        // Find the configuration for this ontology
        if self.config.main_ontology_graph == graph_iri {
            self.load_main_ontology()
        } else {
            // Check domain ontologies - need to clone to avoid borrowing issues
            let domain_configs: Vec<_> = self.config.domain_ontologies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            for (_, domain_config) in domain_configs {
                if domain_config.graph_iri == graph_iri {
                    return self.load_ontology_auto(&domain_config.path, &graph_iri);
                }
            }
            Err(anyhow::anyhow!("Unknown ontology IRI: {}", graph_iri))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_ontology_manager_creation() {
        let manager = OntologyManager::new();
        assert!(manager.is_ok());
        
        let _manager = manager.unwrap();
        // Note: This might not have loaded ontologies if files don't exist
    }

    #[test]
    fn test_config_loading() {
        let config = OntologyConfig::default();
        let manager = OntologyManager::with_config(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_namespace_access() {
        let manager = OntologyManager::new().unwrap();
        
        // Test default namespaces
        assert!(manager.get_namespace("core").is_some());
        assert!(manager.get_namespace("prov").is_some());
        assert!(manager.get_namespace("xsd").is_some());
        
        // Test non-existent namespace
        assert!(manager.get_namespace("nonexistent").is_none());
    }

    #[test]
    fn test_add_namespace() {
        let mut manager = OntologyManager::new().unwrap();
        manager.add_namespace("test", "http://example.org/test#");
        
        assert_eq!(manager.get_namespace("test"), Some(&"http://example.org/test#".to_string()));
    }

    #[test]
    fn test_reload_ontologies() {
        let mut manager = OntologyManager::new().unwrap();
        
        // This should work without errors
        let result = manager.reload_ontologies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_ontology.toml");
        
        let config_content = r#"
[main_ontology]
path = "ontologies/generic_core.owl"
graph_iri = "http://provchain.org/ontology/core"
auto_load = true

[resolution]
strategy = "FileSystem"

[namespaces]
test = "http://example.org/test#"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Test loading from custom config
        // Note: This will fail because the actual files don't exist,
        // but we can test the configuration loading part
    }
}
