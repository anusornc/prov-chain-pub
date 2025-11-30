//! Error handling for ProvChain-Org
//!
//! This module provides comprehensive error types and handling for the entire application.

use thiserror::Error;

/// Main error type for ProvChain operations
#[derive(Error, Debug)]
pub enum ProvChainError {
    /// Blockchain-related errors
    #[error("Blockchain error: {0}")]
    Blockchain(#[from] BlockchainError),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Ontology-related errors
    #[error("Ontology error: {0}")]
    Ontology(#[from] OntologyError),

    /// Transaction-related errors
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    /// Web/API-related errors
    #[error("Web error: {0}")]
    Web(#[from] WebError),

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Oxigraph errors
    #[error("Oxigraph error: {0}")]
    Oxigraph(#[from] oxigraph::store::StorageError),

    /// Oxigraph IRI parse errors
    #[error("IRI parse error: {0}")]
    IriParse(#[from] oxigraph::model::IriParseError),

    /// Anyhow errors (for compatibility)
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

/// Blockchain-specific errors
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Chain validation failed: {0}")]
    ValidationFailed(String),

    #[error("Genesis block creation failed: {0}")]
    GenesisCreationFailed(String),

    #[error("Block hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Invalid chain state: {0}")]
    InvalidChainState(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Ontology initialization failed: {0}")]
    OntologyInitializationFailed(String),
}

/// Storage-specific errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("RDF parsing failed: {0}")]
    RdfParsingFailed(String),

    #[error("SPARQL query error: {0}")]
    SparqlError(String),

    #[error("Backup operation failed: {0}")]
    BackupFailed(String),

    #[error("Restore operation failed: {0}")]
    RestoreFailed(String),

    #[error("Data corruption detected: {0}")]
    DataCorruption(String),

    #[error("Storage capacity exceeded: {0}")]
    CapacityExceeded(String),
}

/// Network-specific errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection timeout: {0}")]
    Timeout(String),

    #[error("Peer connection failed: {0}")]
    PeerConnectionFailed(String),

    #[error("Message serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Invalid peer address: {0}")]
    InvalidPeerAddress(String),

    #[error("Network discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Sync operation failed: {0}")]
    SyncFailed(String),
}

/// Cryptographic errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Hash calculation failed: {0}")]
    HashCalculationFailed(String),
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),

    #[error("SHACL validation failed: {0}")]
    ShaclValidationFailed(String),

    #[error("Entity validation failed: {0}")]
    EntityValidationFailed(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    #[error("Missing configuration parameter: {0}")]
    MissingParameter(String),

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

/// Ontology-specific errors
#[derive(Error, Debug)]
pub enum OntologyError {
    #[error("Ontology loading failed: {0}")]
    LoadingFailed(String),

    #[error("OWL parsing error: {0}")]
    OwlParsingError(String),

    #[error("Reasoning failed: {0}")]
    ReasoningFailed(String),

    #[error("Namespace resolution failed: {0}")]
    NamespaceResolutionFailed(String),

    #[error("Ontology validation failed: {0}")]
    ValidationFailed(String),
}

/// Transaction-specific errors
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Transaction signing failed: {0}")]
    SigningFailed(String),

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Transaction already exists: {0}")]
    AlreadyExists(String),

    #[error("Transaction not found: {0}")]
    NotFound(String),

    #[error("Invalid transaction state: {0}")]
    InvalidState(String),
}

/// Web/API-specific errors
#[derive(Error, Debug)]
pub enum WebError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

/// Result type alias for ProvChain operations
pub type Result<T> = std::result::Result<T, ProvChainError>;

/// Result type alias for blockchain operations
pub type BlockchainResult<T> = std::result::Result<T, BlockchainError>;

/// Result type alias for storage operations
pub type StorageResult<T> = std::result::Result<T, StorageError>;

/// Result type alias for network operations
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// Result type alias for crypto operations
pub type CryptoResult<T> = std::result::Result<T, CryptoError>;

/// Result type alias for validation operations
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

/// Helper trait for converting errors to ProvChainError
pub trait IntoProvChainError<T> {
    fn into_provchain_error(self) -> Result<T>;
}

impl<T, E> IntoProvChainError<T> for std::result::Result<T, E>
where
    E: Into<ProvChainError>,
{
    fn into_provchain_error(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

/// Helper macro for creating custom errors
#[macro_export]
macro_rules! provchain_error {
    ($msg:expr) => {
        $crate::error::ProvChainError::Custom($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::ProvChainError::Custom(format!($fmt, $($arg)*))
    };
}

/// Helper macro for blockchain errors
#[macro_export]
macro_rules! blockchain_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::BlockchainError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::BlockchainError::$variant(format!($fmt, $($arg)*))
    };
}

/// Helper macro for storage errors
#[macro_export]
macro_rules! storage_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::StorageError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::StorageError::$variant(format!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let blockchain_err = BlockchainError::InvalidBlock("test".to_string());
        let provchain_err = ProvChainError::Blockchain(blockchain_err);

        assert!(matches!(provchain_err, ProvChainError::Blockchain(_)));
    }

    #[test]
    fn test_error_display() {
        let err = ProvChainError::Custom("test error".to_string());
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_error_macros() {
        let err = provchain_error!("test error");
        assert!(matches!(err, ProvChainError::Custom(_)));

        let err = blockchain_error!(InvalidBlock, "test block");
        assert!(matches!(err, BlockchainError::InvalidBlock(_)));
    }
}
