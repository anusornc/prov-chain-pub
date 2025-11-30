//! Configuration management for ProvChainOrg

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
    pub web: WebConfig,
    pub ontology_config: Option<OntologyConfigFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_id: String,
    pub listen_port: u16,
    pub bind_address: String,
    pub known_peers: Vec<String>,
    pub max_peers: u32,
    pub connection_timeout: u64,
    pub ping_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub is_authority: bool,
    pub authority_key_file: Option<String>,
    pub authority_keys: Vec<String>,
    pub block_interval: u64,
    pub max_block_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub persistent: bool,
    pub store_type: String,
    pub cache_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            domain_ontology_path: "ontologies/generic_core.owl".to_string(),
            core_ontology_path: Some("ontologies/generic_core.owl".to_string()),
            domain_shacl_path: Some("shapes/core.shacl.ttl".to_string()),
            core_shacl_path: Some("shapes/core.shacl.ttl".to_string()),
            validation_enabled: Some(true),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // Get CORS origins from environment variable or use minimal defaults
        let default_origins = if cfg!(debug_assertions) {
            // Development mode - allow common dev ports
            vec![
                "http://localhost:5173".to_string(),
                "http://localhost:5174".to_string(),
                "http://localhost:5175".to_string(),
            ]
        } else {
            // Production mode - use environment variable or restrictive default
            std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://yourdomain.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        Self {
            network: NetworkConfig {
                network_id: "provchain-org-default".to_string(),
                listen_port: 8080,
                bind_address: "0.0.0.0".to_string(),
                known_peers: vec![],
                max_peers: 50,
                connection_timeout: 30,
                ping_interval: 30,
            },
            consensus: ConsensusConfig {
                is_authority: false,
                authority_key_file: None,
                authority_keys: vec![],
                block_interval: 10,
                max_block_size: 1048576,
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
                persistent: true,
                store_type: "oxigraph".to_string(),
                cache_size_mb: 100,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
            },
            web: WebConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                jwt_secret: "".to_string(), // Must be set via JWT_SECRET environment variable
                cors: CorsConfig {
                    enabled: true,
                    allowed_origins: default_origins,
                    allowed_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "OPTIONS".to_string(),
                    ],
                    allowed_headers: vec![
                        "Authorization".to_string(),
                        "Content-Type".to_string(),
                        "Accept".to_string(),
                    ],
                    allow_credentials: true,
                    max_age: Some(3600),
                },
            },
            ontology_config: None,
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
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.listen_port, 8080);
        assert!(config.web.cors.enabled);
        assert!(config
            .web
            .cors
            .allowed_origins
            .contains(&"http://localhost:5173".to_string()));
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
            "ontologies/generic_core.owl"
        );
        assert_eq!(ontology_config.validation_enabled, Some(true));
    }
}
