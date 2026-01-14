//! HTTP handlers for REST API endpoints

use crate::core::blockchain::Blockchain;
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
}

impl AppState {
    pub fn new(blockchain: Blockchain) -> Self {
        Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            network_peers: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn with_peers(blockchain: Blockchain, peer_count: u64) -> Self {
        Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            network_peers: Arc::new(std::sync::atomic::AtomicU64::new(peer_count)),
        }
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
