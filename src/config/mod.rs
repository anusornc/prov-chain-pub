//! Configuration management for ProvChainOrg

pub use crate::utils::config::{
    load_config, ConsensusConfig, LoggingConfig, NetworkConfig, NodeConfig, StorageConfig,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
    pub web: WebConfig,
    pub ontology_config: Option<OntologyConfigFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u64>,
}

/// Ontology configuration for TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OntologyConfigFile {
    /// Path to the domain-specific ontology file
    pub domain_ontology_path: String,
    /// Path to the core ontology file (optional, defaults to generic_core.owl)
    pub core_ontology_path: Option<String>,
    /// Path to domain-specific SHACL shapes (optional, auto-derived from ontology)
    pub domain_shacl_path: Option<String>,
    /// Path to core SHACL shapes (optional, defaults to core.shacl.ttl)
    pub core_shacl_path: Option<String>,
    /// Whether validation is enabled (defaults to true)
    pub validation_enabled: Option<bool>,
}

impl Default for OntologyConfigFile {
    fn default() -> Self {
        Self {
            domain_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            core_ontology_path: Some("src/semantic/ontologies/generic_core.owl".to_string()),
            domain_shacl_path: Some("src/semantic/shapes/core.shacl.ttl".to_string()),
            core_shacl_path: Some("src/semantic/shapes/core.shacl.ttl".to_string()),
            validation_enabled: Some(true),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            consensus: ConsensusConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            web: WebConfig::default(),
            ontology_config: None,
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "".to_string(),
            cors: CorsConfig::default(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        let allowed_origins = if cfg!(debug_assertions) {
            vec![
                "http://localhost:5173".to_string(),
                "http://localhost:5174".to_string(),
                "http://localhost:5175".to_string(),
            ]
        } else {
            std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://yourdomain.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        Self {
            enabled: true,
            allowed_origins,
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "Accept".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(3600),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from file or use default
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load config file: {}. Using defaults.",
                    e
                );
                Self::default()
            }
        }
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get CORS configuration for development environment
    pub fn get_development_cors(&self) -> CorsConfig {
        if cfg!(debug_assertions) {
            self.web.cors.clone()
        } else {
            // In production, use environment variables or more restrictive defaults
            let allowed_origins = std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://yourdomain.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            CorsConfig {
                enabled: true,
                allowed_origins,
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
                allowed_headers: vec![
                    "Authorization".to_string(),
                    "Content-Type".to_string(),
                    "Accept".to_string(),
                ],
                allow_credentials: true,
                max_age: Some(3600),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg(debug_assertions)]
    fn test_default_config_debug() {
        let config = Config::default();
        assert_eq!(config.network.listen_port, 8080);
        assert!(config.web.cors.enabled);
        // In debug mode, default origins include localhost dev ports
        assert!(config
            .web
            .cors
            .allowed_origins
            .contains(&"http://localhost:5173".to_string()));
        assert!(config
            .web
            .cors
            .allowed_origins
            .contains(&"http://localhost:5174".to_string()));
        assert!(config
            .web
            .cors
            .allowed_origins
            .contains(&"http://localhost:5175".to_string()));
    }

    #[test]
    fn test_default_config_common() {
        let config = Config::default();
        // These assertions should work in both debug and release mode
        assert_eq!(config.network.listen_port, 8080);
        assert!(config.web.cors.enabled);
        assert!(!config.web.cors.allowed_origins.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.network.listen_port, deserialized.network.listen_port);
    }

    #[test]
    fn test_config_file_operations() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();

        // Save config
        config.save_to_file(temp_file.path()).unwrap();

        // Load config
        let loaded_config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(
            config.network.listen_port,
            loaded_config.network.listen_port
        );
    }

    #[test]
    fn test_ontology_config_default() {
        let ontology_config = OntologyConfigFile::default();
        assert_eq!(
            ontology_config.domain_ontology_path,
            "src/semantic/ontologies/generic_core.owl"
        );
        assert_eq!(ontology_config.validation_enabled, Some(true));
    }

    #[test]
    fn test_partial_config_uses_defaults_for_missing_fields() {
        let partial_toml = r#"
[network]
network_id = "partial-config-network"

[web]
port = 9090
"#;

        let config: Config = toml::from_str(partial_toml).unwrap();

        assert_eq!(config.network.network_id, "partial-config-network");
        assert_eq!(
            config.network.listen_port,
            NetworkConfig::default().listen_port
        );
        assert_eq!(
            config.consensus.consensus_type,
            ConsensusConfig::default().consensus_type
        );
        assert_eq!(config.web.port, 9090);
        assert_eq!(config.web.host, WebConfig::default().host);
    }
}
