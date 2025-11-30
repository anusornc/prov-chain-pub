//! Network module for distributed ProvChainOrg implementation
//!
//! This module provides P2P networking capabilities including:
//! - Peer discovery and connection management
//! - Message protocol for blockchain synchronization
//! - WebSocket-based communication between nodes
//! - Blockchain synchronization and consensus

pub mod consensus;
pub mod discovery;
pub mod messages;
pub mod peer;
pub mod sync;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::messages::{P2PMessage, PeerInfo};
use self::peer::PeerConnection;
use crate::utils::config::NodeConfig;

/// Network manager for handling all P2P operations
pub struct NetworkManager {
    /// Unique identifier for this node
    pub node_id: Uuid,
    /// Node configuration
    pub config: NodeConfig,
    /// Connected peers
    pub peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    /// Network event handlers
    pub message_handlers: Vec<Box<dyn MessageHandler + Send + Sync>>,
}

/// Trait for handling incoming network messages
pub trait MessageHandler {
    fn handle_message(&self, peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>>;
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NodeConfig) -> Self {
        Self {
            node_id: config.node_id,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Vec::new(),
        }
    }

    /// Start the network manager (listen for connections and connect to peers)
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting network manager for node {}", self.node_id);

        // Start WebSocket server for incoming connections
        self.start_server().await?;

        // Connect to known peers
        self.connect_to_known_peers().await?;

        Ok(())
    }

    /// Start WebSocket server for incoming peer connections
    async fn start_server(&self) -> Result<()> {
        // Implementation will be added in peer.rs
        tracing::info!(
            "WebSocket server starting on port {}",
            self.config.network.listen_port
        );
        Ok(())
    }

    /// Connect to known peers from configuration
    async fn connect_to_known_peers(&self) -> Result<()> {
        for peer_addr in &self.config.network.known_peers {
            tracing::info!("Attempting to connect to peer: {}", peer_addr);
            // Implementation will be added in peer.rs
        }
        Ok(())
    }

    /// Add a message handler
    pub fn add_message_handler(&mut self, handler: Box<dyn MessageHandler + Send + Sync>) {
        self.message_handlers.push(handler);
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast_message(&self, message: P2PMessage) -> Result<()> {
        let peers = self.peers.read().await;
        for (peer_id, peer) in peers.iter() {
            if let Err(e) = peer.send_message(message.clone()).await {
                tracing::warn!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }
        Ok(())
    }

    /// Send a message to a specific peer
    pub async fn send_to_peer(&self, peer_id: Uuid, message: P2PMessage) -> Result<()> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(&peer_id) {
            peer.send_message(message).await?;
        } else {
            anyhow::bail!("Peer {} not found", peer_id);
        }
        Ok(())
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().map(|p| p.info.clone()).collect()
    }

    /// Handle incoming message from a peer
    pub async fn handle_incoming_message(&self, peer_id: Uuid, message: P2PMessage) -> Result<()> {
        tracing::debug!("Received message from peer {}: {:?}", peer_id, message);

        for handler in &self.message_handlers {
            if let Some(response) = handler.handle_message(peer_id, message.clone())? {
                self.send_to_peer(peer_id, response).await?;
            }
        }

        Ok(())
    }
}
