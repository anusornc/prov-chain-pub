//! HTTP handlers for REST API endpoints

use crate::core::blockchain::Blockchain;
use crate::wallet::WalletManager;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod query;
pub mod transaction;
pub mod utils;

// Re-export common handlers
pub use query::*;
pub use transaction::*;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub network_peers: Arc<std::sync::atomic::AtomicU64>,
    pub wallet_manager: Arc<RwLock<WalletManager>>,
}

impl AppState {
    /// Create a new AppState with the given blockchain
    ///
    /// # Errors
    /// Returns an error if the wallet manager fails to initialize
    pub fn new(blockchain: Blockchain) -> anyhow::Result<Self> {
        // Initialize wallet manager with default path
        let wallet_manager = WalletManager::new("./data/wallets", None)
            .map_err(|e| anyhow::anyhow!("Failed to initialize wallet manager: {}", e))?;

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            network_peers: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            wallet_manager: Arc::new(RwLock::new(wallet_manager)),
        })
    }

    /// Create a new AppState with the given blockchain and peer count
    ///
    /// # Errors
    /// Returns an error if the wallet manager fails to initialize
    pub fn with_peers(blockchain: Blockchain, peer_count: u64) -> anyhow::Result<Self> {
        let wallet_manager = WalletManager::new("./data/wallets", None)
            .map_err(|e| anyhow::anyhow!("Failed to initialize wallet manager: {}", e))?;

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            network_peers: Arc::new(std::sync::atomic::AtomicU64::new(peer_count)),
            wallet_manager: Arc::new(RwLock::new(wallet_manager)),
        })
    }
}

/// Enhanced health check endpoint with security status
pub async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "security": {
            "jwt_secret_configured": std::env::var("JWT_SECRET").is_ok(),
            "rate_limiting_enabled": true,
            "security_headers_enabled": true,
            "environment": if cfg!(debug_assertions) { "development" } else { "production" }
        }
    }))
}
