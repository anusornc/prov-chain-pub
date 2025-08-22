//! Domain manager for loading and managing domain plugins
//!
//! This module provides the domain manager that loads and manages
//! domain plugins for the universal traceability platform.

use crate::domain::plugin::{DomainPlugin, DomainConfig, ValidationResult, ProcessedEntity, EntityData};
// use crate::domain::adapters::OwlDomainAdapter;
use anyhow::{Result, Context};
use std::collections::HashMap;
use tracing::{info, warn};

/// Domain manager for loading and managing domain plugins
pub struct DomainManager {
    /// Registered domain plugins
    pub plugins: HashMap<String, Box<dyn DomainPlugin>>,
    /// Currently active domain
    pub active_domain: Option<String>,
}

impl DomainManager {
    /// Create a new domain manager
    pub fn new() -> Self {
        DomainManager {
            plugins: HashMap::new(),
            active_domain: None,
        }
    }
    
    /// Register a domain plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn DomainPlugin>) -> Result<()> {
        let domain_id = plugin.domain_id().to_string();
        info!("Registering domain plugin: {}", domain_id);
        self.plugins.insert(domain_id, plugin);
        Ok(())
    }
    
    /// Load domain plugins from configuration
    pub fn load_from_config(&mut self, config_path: &str) -> Result<()> {
        info!("Loading domain plugins from config: {}", config_path);
        
        let config: serde_yaml::Value = serde_yaml::from_reader(
            std::fs::File::open(config_path)?
        ).context("Failed to parse domain configuration")?;
        
        if let Some(domains) = config.get("domains") {
            if let Some(mapping) = domains.as_mapping() {
                for (domain_id, domain_config) in mapping {
                    if let Some(domain_id_str) = domain_id.as_str() {
                        let enabled = domain_config.get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        
                        if enabled {
                            self.load_domain_plugin(domain_id_str, domain_config)?;
                        }
                    }
                }
            }
        }
        
        // Set default domain
        if let Some(default_domain) = config.get("default_domain") {
            if let Some(domain_id) = default_domain.as_str() {
                self.set_active_domain(domain_id)?;
            }
        }
        
        info!("Loaded {} domain plugins", self.plugins.len());
        Ok(())
    }
    
    /// Load a single domain plugin
    pub fn load_domain_plugin(&mut self, domain_id: &str, config: &serde_yaml::Value) -> Result<()> {
        info!("Loading domain plugin: {}", domain_id);
        
        // Create domain plugin based on configuration
        let plugin = self.create_domain_plugin(domain_id, config)?;
        self.register_plugin(plugin)?;
        
        Ok(())
    }
    
    /// Create domain plugin based on configuration
    fn create_domain_plugin(&self, domain_id: &str, config: &serde_yaml::Value) -> Result<Box<dyn DomainPlugin>> {
        match domain_id {
            "supplychain" => {
                // Create OWL adapter for supply chain domain
                let mut domain_config = config.clone();
                if let Some(mapping) = domain_config.as_mapping_mut() {
                    mapping.insert(
                        serde_yaml::Value::String("domain_id".to_string()),
                        serde_yaml::Value::String("supplychain".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("name".to_string()),
                        serde_yaml::Value::String("Supply Chain Traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("description".to_string()),
                        serde_yaml::Value::String("General supply chain and manufacturing traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("domain_ontology_path".to_string()),
                        serde_yaml::Value::String("ontologies/supply-chain.owl".to_string())
                    );
                }
                Err(anyhow::anyhow!("OwlDomainAdapter not yet implemented"))
            },
            "healthcare" => {
                // Create OWL adapter for healthcare domain
                let mut domain_config = config.clone();
                if let Some(mapping) = domain_config.as_mapping_mut() {
                    mapping.insert(
                        serde_yaml::Value::String("domain_id".to_string()),
                        serde_yaml::Value::String("healthcare".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("name".to_string()),
                        serde_yaml::Value::String("Healthcare Traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("description".to_string()),
                        serde_yaml::Value::String("Healthcare and medical traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("domain_ontology_path".to_string()),
                        serde_yaml::Value::String("ontologies/healthcare.owl".to_string())
                    );
                }
                Err(anyhow::anyhow!("OwlDomainAdapter not yet implemented"))
            },
            "pharmaceutical" => {
                // Create OWL adapter for pharmaceutical domain
                let mut domain_config = config.clone();
                if let Some(mapping) = domain_config.as_mapping_mut() {
                    mapping.insert(
                        serde_yaml::Value::String("domain_id".to_string()),
                        serde_yaml::Value::String("pharmaceutical".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("name".to_string()),
                        serde_yaml::Value::String("Pharmaceutical Traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("description".to_string()),
                        serde_yaml::Value::String("Pharmaceutical and drug traceability".to_string())
                    );
                    mapping.insert(
                        serde_yaml::Value::String("domain_ontology_path".to_string()),
                        serde_yaml::Value::String("ontologies/pharmaceutical.owl".to_string())
                    );
                }
                Err(anyhow::anyhow!("OwlDomainAdapter not yet implemented"))
            },
            _ => {
                // Try to load as external plugin or generic OWL adapter
                self.load_external_plugin(domain_id, config)
            }
        }
    }
    
    /// Load external plugin from shared library or create generic OWL adapter
    fn load_external_plugin(&self, domain_id: &str, config: &serde_yaml::Value) -> Result<Box<dyn DomainPlugin>> {
        let plugin_path = format!("plugins/{}_plugin.so", domain_id);
        warn!("External plugin loading not yet implemented: {}", plugin_path);
        
        // For now, create a generic OWL adapter
        let mut domain_config = config.clone();
        if let Some(mapping) = domain_config.as_mapping_mut() {
            mapping.insert(
                serde_yaml::Value::String("domain_id".to_string()),
                serde_yaml::Value::String(domain_id.to_string())
            );
            mapping.insert(
                serde_yaml::Value::String("name".to_string()),
                serde_yaml::Value::String(format!("{} Domain", domain_id))
            );
            mapping.insert(
                serde_yaml::Value::String("description".to_string()),
                serde_yaml::Value::String(format!("{} traceability domain", domain_id))
            );
            mapping.insert(
                serde_yaml::Value::String("domain_ontology_path".to_string()),
                serde_yaml::Value::String(format!("ontologies/{}.owl", domain_id))
            );
        }
        
        Err(anyhow::anyhow!("OwlDomainAdapter not yet implemented"))
    }
    
    /// Set active domain
    pub fn set_active_domain(&mut self, domain_id: &str) -> Result<()> {
        if self.plugins.contains_key(domain_id) {
            info!("Setting active domain to: {}", domain_id);
            self.active_domain = Some(domain_id.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Domain {} not registered", domain_id))
        }
    }
    
    /// Get active domain
    pub fn get_active_domain(&self) -> Option<&Box<dyn DomainPlugin>> {
        if let Some(ref domain_id) = self.active_domain {
            self.plugins.get(domain_id)
        } else {
            None
        }
    }
    
    /// Validate entity for active domain
    pub fn validate_entity_for_active_domain(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if let Some(domain) = self.get_active_domain() {
            domain.validate_entity(entity_data)
        } else {
            // Use generic validation
            self.generic_validate(entity_data)
        }
    }
    
    /// Generic validation for entities not tied to specific domain
    fn generic_validate(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Basic validation for generic traceable entities
        if entity_data.entity_id.is_empty() {
            return Ok(ValidationResult::Invalid("Entity ID is required".to_string()));
        }
        
        if entity_data.entity_type.is_empty() {
            return Ok(ValidationResult::Invalid("Entity type is required".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Process entity data for active domain
    pub fn process_entity_for_active_domain(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        if let Some(domain) = self.get_active_domain() {
            domain.process_entity(entity_data)
        } else {
            // Use generic processing
            self.generic_process(entity_data)
        }
    }
    
    /// Generic processing for entities not tied to specific domain
    fn generic_process(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        // Basic processing for generic traceable entities
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: entity_data.data.clone(),
            domain_context: "generic".to_string(),
        })
    }
}

/// Generic domain adapter for domains without specific implementations
pub struct GenericDomainAdapter {
    config: DomainConfig,
    validation_rules: HashMap<String, String>,
    domain_properties: Vec<String>,
}

impl GenericDomainAdapter {
    /// Create a new generic domain adapter
    pub fn new(domain_id: &str) -> Self {
        let config = DomainConfig {
            domain_id: domain_id.to_string(),
            name: format!("Generic {} Domain", domain_id),
            description: format!("Generic domain adapter for {}", domain_id),
            core_ontology_path: "ontologies/core.owl".to_string(),
            domain_ontology_path: format!("ontologies/{}.owl", domain_id),
            ontology_path: format!("ontologies/{}.owl", domain_id),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: Vec::new(),
            validation_queries: Vec::new(),
            enabled: true,
            priority: 1,
            custom_properties: HashMap::new(),
        };
        
        GenericDomainAdapter {
            config,
            validation_rules: HashMap::new(),
            domain_properties: Vec::new(),
        }
    }
}

impl DomainPlugin for GenericDomainAdapter {
    fn domain_id(&self) -> &str {
        &self.config.domain_id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    fn is_valid_entity_type(&self, _entity_type: &str) -> bool {
        // Accept any entity type in generic domain
        true
    }
    
    fn validation_rules(&self) -> &HashMap<String, String> {
        &self.validation_rules
    }
    
    fn domain_properties(&self) -> &Vec<String> {
        &self.domain_properties
    }
    
    fn initialize(&mut self, _config: &DomainConfig) -> Result<()> {
        // Nothing to initialize for generic domain
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Nothing to shutdown for generic domain
        Ok(())
    }
    
    fn validate_entity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Use generic validation
        if entity_data.entity_id.is_empty() {
            return Ok(ValidationResult::Invalid("Entity ID is required".to_string()));
        }
        
        if entity_data.entity_type.is_empty() {
            return Ok(ValidationResult::Invalid("Entity type is required".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        // Use generic processing
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: entity_data.data.clone(),
            domain_context: self.config.domain_id.clone(),
        })
    }
}