//! Configuration management for ProvChainOrg distributed nodes
//!
//! This module handles node configuration including:
//! - Node identity and network parameters
//! - Authority settings for consensus
//! - Bootstrap peer configuration
//! - Storage and networking options

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Complete node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique identifier for this node
    pub node_id: Uuid,

    /// Network configuration
    pub network: NetworkConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Ontology configuration
    pub ontology: Option<OntologyConfig>,
}

/// Network-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network identifier (must match across all nodes)
    pub network_id: String,

    /// Port to listen for incoming peer connections
    pub listen_port: u16,

    /// Address to bind to (usually "0.0.0.0" for all interfaces)
    pub bind_address: String,

    /// List of bootstrap peers to connect to on startup
    pub known_peers: Vec<String>,

    /// Maximum number of peer connections
    pub max_peers: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Ping interval for connection health checks (seconds)
    pub ping_interval: u64,
}

/// Consensus-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus protocol type ("poa" or "pbft")
    pub consensus_type: String,

    /// Whether this node is an authority (can create blocks)
    pub is_authority: bool,

    /// Authority private key file path (if authority)
    pub authority_key_file: Option<String>,

    /// List of known authority public keys
    pub authority_keys: Vec<String>,

    /// Block creation interval for authorities (seconds)
    pub block_interval: u64,

    /// Maximum block size in bytes
    pub max_block_size: usize,
}

/// Storage-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory for persistent storage
    pub data_dir: String,

    /// Whether to use persistent storage (vs in-memory)
    pub persistent: bool,

    /// RDF store type ("oxigraph" for now)
    pub store_type: String,

    /// Maximum cache size in MB
    pub cache_size_mb: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log output format (json, pretty)
    pub format: String,

    /// Log file path (optional, logs to stdout if not specified)
    pub file: Option<String>,
}

/// Ontology configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    /// Path to the ontology file
    pub path: String,

    /// Graph name for the ontology
    pub graph_name: String,

    /// Whether to automatically load the ontology on startup
    pub auto_load: bool,

    /// Whether to validate RDF data against the ontology
    pub validate_data: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: Uuid::new_v4(),
            network: NetworkConfig::default(),
            consensus: ConsensusConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            ontology: Some(OntologyConfig::default()),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_id: "provchain-org-default".to_string(),
            listen_port: 8080,
            bind_address: "0.0.0.0".to_string(),
            known_peers: vec![],
            max_peers: 50,
            connection_timeout: 30,
            ping_interval: 30,
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            consensus_type: "poa".to_string(),
            is_authority: false,
            authority_key_file: None,
            authority_keys: vec![],
            block_interval: 10,
            max_block_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            persistent: true,
            store_type: "oxigraph".to_string(),
            cache_size_mb: 100,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file: None,
        }
    }
}

impl Default for OntologyConfig {
    fn default() -> Self {
        Self {
            path: "ontologies/generic_core.owl".to_string(),
            graph_name: "http://provchain.org/ontology".to_string(),
            auto_load: true,
            validate_data: false,
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: NodeConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load configuration from environment variables and command line args
    pub fn load_from_env() -> Result<Self> {
        let mut config = NodeConfig::default();

        // Override with environment variables
        if let Ok(network_id) = std::env::var("PROVCHAIN_NETWORK_ID") {
            config.network.network_id = network_id;
        }

        if let Ok(port) = std::env::var("PROVCHAIN_PORT") {
            config.network.listen_port = port.parse()?;
        }

        if let Ok(peers) = std::env::var("PROVCHAIN_PEERS") {
            config.network.known_peers = peers.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(is_authority) = std::env::var("PROVCHAIN_AUTHORITY") {
            config.consensus.is_authority = is_authority.parse().unwrap_or(false);
        }

        if let Ok(data_dir) = std::env::var("PROVCHAIN_DATA_DIR") {
            config.storage.data_dir = data_dir;
        }

        if let Ok(log_level) = std::env::var("PROVCHAIN_LOG_LEVEL") {
            config.logging.level = log_level;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate network configuration
        if self.network.network_id.is_empty() {
            anyhow::bail!("Network ID cannot be empty");
        }

        if self.network.listen_port == 0 {
            anyhow::bail!("Listen port must be greater than 0");
        }

        if self.network.max_peers == 0 {
            anyhow::bail!("Max peers must be greater than 0");
        }

        // Validate consensus configuration
        let valid_consensus_types = ["poa", "pbft"];
        if !valid_consensus_types.contains(&self.consensus.consensus_type.as_str()) {
            anyhow::bail!("Invalid consensus type: {}", self.consensus.consensus_type);
        }

        if self.consensus.is_authority && self.consensus.authority_key_file.is_none() {
            anyhow::bail!("Authority nodes must specify a key file");
        }

        if self.consensus.block_interval == 0 {
            anyhow::bail!("Block interval must be greater than 0");
        }

        // Validate storage configuration
        if self.storage.data_dir.is_empty() {
            anyhow::bail!("Data directory cannot be empty");
        }

        // Validate logging configuration
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            anyhow::bail!("Invalid log level: {}", self.logging.level);
        }

        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            anyhow::bail!("Invalid log format: {}", self.logging.format);
        }

        Ok(())
    }

    /// Get the full listen address
    pub fn listen_address(&self) -> String {
        format!("{}:{}", self.network.bind_address, self.network.listen_port)
    }

    /// Get the public address for this node
    pub fn public_address(&self) -> String {
        // In a real implementation, this might determine the external IP
        format!("127.0.0.1:{}", self.network.listen_port)
    }

    /// Create data directory if it doesn't exist
    pub fn ensure_data_dir(&self) -> Result<()> {
        if !Path::new(&self.storage.data_dir).exists() {
            fs::create_dir_all(&self.storage.data_dir)?;
        }
        Ok(())
    }

    /// Generate a new node ID and save to config file
    pub fn regenerate_node_id(&mut self) {
        self.node_id = Uuid::new_v4();
    }
}

/// Create a default configuration file
pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let config = NodeConfig::default();
    config.save_to_file(path)?;
    Ok(())
}

/// Load configuration with fallback priority:
/// 1. Command line config file
/// 2. Environment variables
/// 3. Default config file
/// 4. Built-in defaults
pub fn load_config(config_file: Option<&str>) -> Result<NodeConfig> {
    // Try to load from specified config file
    if let Some(file_path) = config_file {
        if Path::new(file_path).exists() {
            return NodeConfig::load_from_file(file_path);
        } else {
            anyhow::bail!("Config file not found: {}", file_path);
        }
    }

    // Try to load from default config file
    let default_config_path = "config.toml";
    if Path::new(default_config_path).exists() {
        let mut config = NodeConfig::load_from_file(default_config_path)?;

        // Override with environment variables
        let env_config = NodeConfig::load_from_env()?;
        merge_configs(&mut config, env_config);

        return Ok(config);
    }

    // Fall back to environment variables and defaults
    NodeConfig::load_from_env()
}

/// Merge environment config into file config
fn merge_configs(base: &mut NodeConfig, env: NodeConfig) {
    // Only override non-default values from environment
    if env.network.network_id != NetworkConfig::default().network_id {
        base.network.network_id = env.network.network_id;
    }

    if env.network.listen_port != NetworkConfig::default().listen_port {
        base.network.listen_port = env.network.listen_port;
    }

    if !env.network.known_peers.is_empty() {
        base.network.known_peers = env.network.known_peers;
    }

    if env.consensus.is_authority != ConsensusConfig::default().is_authority {
        base.consensus.is_authority = env.consensus.is_authority;
    }

    if env.storage.data_dir != StorageConfig::default().data_dir {
        base.storage.data_dir = env.storage.data_dir;
    }

    if env.logging.level != LoggingConfig::default().level {
        base.logging.level = env.logging.level;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.network.network_id, "provchain-org-default");
        assert_eq!(config.network.listen_port, 8080);
        assert!(!config.consensus.is_authority);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NodeConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid network ID
        config.network.network_id = String::new();
        assert!(config.validate().is_err());

        // Reset and test invalid port
        config = NodeConfig::default();
        config.network.listen_port = 0;
        assert!(config.validate().is_err());

        // Reset and test authority without key file
        config = NodeConfig::default();
        config.consensus.is_authority = true;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_operations() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let config = NodeConfig::default();

        // Save config
        config.save_to_file(&config_path).unwrap();
        assert!(config_path.exists());

        // Load config
        let loaded_config = NodeConfig::load_from_file(&config_path).unwrap();
        assert_eq!(config.node_id, loaded_config.node_id);
        assert_eq!(config.network.network_id, loaded_config.network.network_id);
    }

    #[test]
    fn test_address_methods() {
        let config = NodeConfig::default();
        assert_eq!(config.listen_address(), "0.0.0.0:8080");
        assert_eq!(config.public_address(), "127.0.0.1:8080");
    }
}
