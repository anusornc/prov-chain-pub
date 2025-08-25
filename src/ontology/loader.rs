//! Ontology configuration loader
//!
//! This module provides utilities for loading ontology configurations
//! from various sources including TOML files, environment variables,
//! and command-line arguments.

use crate::ontology::config::OntologyConfig;
use anyhow::Result;
use std::path::Path;

/// Load ontology configuration from TOML file
pub fn load_ontology_config_from_toml(config_path: &str) -> Result<OntologyConfig> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;
    
    let config: OntologyConfig = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse TOML config: {}", e))?;
    
    Ok(config)
}

/// Load ontology configuration from environment variables
pub fn load_ontology_config_from_env() -> OntologyConfig {
    let mut config = OntologyConfig::default();
    
    // Override with environment variables if present
    if let Ok(main_path) = std::env::var("ONTOLGY_MAIN_PATH") {
        config.main_ontology_path = main_path;
    }
    
    if let Ok(auto_load) = std::env::var("ONTOLGY_AUTO_LOAD") {
        if let Ok(auto_load_bool) = auto_load.parse::<bool>() {
            config.auto_load = auto_load_bool;
        }
    }
    
    if let Ok(validate_data) = std::env::var("ONTOLGY_VALIDATE_DATA") {
        if let Ok(validate_bool) = validate_data.parse::<bool>() {
            config.validate_data = validate_bool;
        }
    }
    
    config
}

/// Load ontology configuration with fallback priority:
/// 1. Command-line config file
/// 2. Environment variables  
/// 3. Default config file (config/ontology.toml)
/// 4. Built-in defaults
pub fn load_ontology_config(config_file: Option<&str>) -> Result<OntologyConfig> {
    // Try to load from specified config file
    if let Some(file_path) = config_file {
        if Path::new(file_path).exists() {
            return load_ontology_config_from_toml(file_path);
        } else {
            return Err(anyhow::anyhow!("Config file not found: {}", file_path));
        }
    }
    
    // Try to load from environment variables
    let env_config = load_ontology_config_from_env();
    
    // Try to load from default config file
    let default_config_path = "config/ontology.toml";
    if Path::new(default_config_path).exists() {
        match load_ontology_config_from_toml(default_config_path) {
            Ok(mut config) => {
                // Merge with environment config (env overrides file)
                merge_configs(&mut config, &env_config);
                return Ok(config);
            },
            Err(e) => {
                eprintln!("Warning: Failed to load default config file: {}", e);
            }
        }
    }
    
    // Fall back to environment config or defaults
    Ok(env_config)
}

/// Merge two configurations, with env_config overriding file_config
fn merge_configs(file_config: &mut OntologyConfig, env_config: &OntologyConfig) {
    // Override non-default values from environment
    if !env_config.main_ontology_path.is_empty() {
        file_config.main_ontology_path = env_config.main_ontology_path.clone();
    }
    
    if !env_config.main_ontology_graph.is_empty() {
        file_config.main_ontology_graph = env_config.main_ontology_graph.clone();
    }
    
    // Merge namespace mappings
    for (prefix, uri) in &env_config.namespace_mappings {
        file_config.namespace_mappings.insert(prefix.clone(), uri.clone());
    }
    
    // Note: For simplicity, we don't merge domain ontologies from env vars
    // In a full implementation, you might want to do this too
}