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
    pub message_handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>>,
    /// Channel sender for incoming messages
    pub message_sender: tokio::sync::mpsc::Sender<(Uuid, P2PMessage)>,
}

/// Trait for handling incoming network messages
pub trait MessageHandler {
    fn handle_message(&self, peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>>;
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NodeConfig) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(Uuid, P2PMessage)>(100);
        let handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>> =
            Arc::new(RwLock::new(Vec::new()));
        let handlers_clone = Arc::clone(&handlers);

        // Spawn message processor
        tokio::spawn(async move {
            while let Some((peer_id, message)) = rx.recv().await {
                let handlers = handlers_clone.read().await;
                for handler in handlers.iter() {
                    if let Ok(Some(_response)) = handler.handle_message(peer_id, message.clone()) {
                        // TODO: Send response back (requires access to network/peers)
                        // For now, we just handle the message
                        tracing::debug!(
                            "Handler generated response, but sending not implemented in loop"
                        );
                    }
                }
            }
        });

        Self {
            node_id: config.node_id,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: handlers,
            message_sender: tx,
        }
    }

    /// Start the network manager (listen for connections and connect to peers)
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting network manager for node {}", self.node_id);

        // Start WebSocket server for incoming connections
        self.start_server().await?;

        // Connect to known peers
        self.connect_to_known_peers().await?;

        Ok(())
    }

    /// Start WebSocket server for incoming peer connections
    async fn start_server(&self) -> Result<()> {
        let listen_addr = self.config.listen_address();
        let message_handler = self.create_message_handler();

        // Create connection handler to store new peers
        let peers = Arc::clone(&self.peers);
        let connection_handler =
            Arc::new(move |connection: crate::network::peer::PeerConnection| {
                let peer_id = connection.info.node_id;
                tracing::info!("New peer connected: {}", peer_id);

                // We need to store the connection in the peers map
                // Since this is a sync closure, we can't await the lock.
                // We spawn a task to do it.
                let peers_clone = Arc::clone(&peers);
                tokio::spawn(async move {
                    peers_clone.write().await.insert(peer_id, connection);
                });
            });

        let server = crate::network::peer::PeerServer::new(
            &listen_addr,
            message_handler,
            connection_handler,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                tracing::error!("WebSocket server error: {}", e);
            }
        });

        tracing::info!("WebSocket server started on {}", listen_addr);
        Ok(())
    }

    /// Connect to known peers from configuration
    async fn connect_to_known_peers(&self) -> Result<()> {
        for peer_addr in &self.config.network.known_peers {
            tracing::info!("Attempting to connect to peer: {}", peer_addr);

            let message_handler = self.create_message_handler();
            let peer_addr_clone = peer_addr.clone();
            let peers = Arc::clone(&self.peers);

            tokio::spawn(async move {
                match crate::network::peer::PeerClient::connect(&peer_addr_clone, message_handler)
                    .await
                {
                    Ok(connection) => {
                        tracing::info!("Successfully connected to peer {}", peer_addr_clone);
                        let peer_id = connection.info.node_id;
                        peers.write().await.insert(peer_id, connection);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to peer {}: {}", peer_addr_clone, e);
                    }
                }
            });
        }
        Ok(())
    }

    /// Create a message handler closure
    fn create_message_handler(&self) -> Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync> {
        let tx = self.message_sender.clone();
        Arc::new(move |peer_id, message| {
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = tx.send((peer_id, message)).await {
                    tracing::error!("Failed to send message to internal channel: {}", e);
                }
            });
        })
    }

    /// Add a message handler
    pub async fn add_message_handler(&self, handler: Box<dyn MessageHandler + Send + Sync>) {
        self.message_handlers.write().await.push(handler);
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

        let handlers = self.message_handlers.read().await;
        for handler in handlers.iter() {
            if let Some(response) = handler.handle_message(peer_id, message.clone())? {
                self.send_to_peer(peer_id, response).await?;
            }
        }

        Ok(())
    }
}
