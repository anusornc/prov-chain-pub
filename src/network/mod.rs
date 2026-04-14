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
pub mod profile;
pub mod sync;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::discovery::PeerDiscovery;
use self::messages::{ErrorCode, P2PMessage, PeerInfo};
use self::peer::PeerConnection;
use self::profile::SemanticContractInfo;
use crate::utils::config::NodeConfig;

/// Network manager for handling all P2P operations
pub struct NetworkManager {
    /// Unique identifier for this node
    pub node_id: Uuid,
    /// Node configuration
    pub config: NodeConfig,
    /// Connected peers
    pub peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    /// Mapping from discovered logical node IDs to stable transport connection IDs
    pub logical_peers: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    /// Peer discovery and semantic compatibility state
    pub discovery: Arc<PeerDiscovery>,
    /// Network event handlers
    pub message_handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>>,
    /// Channel sender for incoming messages
    pub message_sender: tokio::sync::mpsc::Sender<(Uuid, P2PMessage)>,
}

/// Trait for handling incoming network messages
pub trait MessageHandler {
    fn handle_message(&self, peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>>;
}

enum RekeyOutcome {
    Updated,
    DuplicateLogicalPeer { existing_transport_id: Uuid },
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NodeConfig) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(Uuid, P2PMessage)>(100);
        let peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let logical_peers: Arc<RwLock<HashMap<Uuid, Uuid>>> = Arc::new(RwLock::new(HashMap::new()));
        let handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>> =
            Arc::new(RwLock::new(Vec::new()));
        let handlers_clone = Arc::clone(&handlers);
        let peers_clone = Arc::clone(&peers);
        let logical_peers_clone = Arc::clone(&logical_peers);
        let local_peer_info = Self::build_local_peer_info(&config);
        let discovery = Arc::new(PeerDiscovery::new(
            local_peer_info,
            config.network.known_peers.clone(),
        ));
        let discovery_clone = Arc::clone(&discovery);

        // Spawn message processor
        tokio::spawn(async move {
            while let Some((peer_id, message)) = rx.recv().await {
                Self::process_discovery_message(
                    Arc::clone(&peers_clone),
                    Arc::clone(&logical_peers_clone),
                    Arc::clone(&discovery_clone),
                    peer_id,
                    message.clone(),
                )
                .await;

                let handler_peer_id =
                    Self::resolve_handler_peer_id(Arc::clone(&peers_clone), peer_id)
                        .await
                        .unwrap_or(peer_id);

                let handlers = handlers_clone.read().await;
                for handler in handlers.iter() {
                    if let Ok(Some(response)) =
                        handler.handle_message(handler_peer_id, message.clone())
                    {
                        tracing::debug!(
                            "Handler generated response for peer {}: {:?}",
                            handler_peer_id,
                            response.message_type()
                        );
                        Self::send_response_to_peer(Arc::clone(&peers_clone), peer_id, response)
                            .await;
                    }
                }
            }
        });

        Self {
            node_id: config.node_id,
            config,
            peers,
            logical_peers,
            discovery,
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
        let discovery = Arc::clone(&self.discovery);
        let connection_handler =
            Arc::new(move |connection: crate::network::peer::PeerConnection| {
                let transport_id = connection.transport_id;
                tracing::info!("New peer transport connected: {}", transport_id);

                let peers_clone = Arc::clone(&peers);
                let discovery_clone = Arc::clone(&discovery);
                tokio::spawn(async move {
                    if let Err(e) = connection
                        .send_message(Self::build_peer_discovery_message(
                            &discovery_clone.local_node_info,
                        ))
                        .await
                    {
                        tracing::warn!(
                            "Failed to send initial discovery message to transport {}: {}",
                            transport_id,
                            e
                        );
                    }
                    peers_clone.write().await.insert(transport_id, connection);
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
            let discovery = Arc::clone(&self.discovery);

            tokio::spawn(async move {
                match crate::network::peer::PeerClient::connect(&peer_addr_clone, message_handler)
                    .await
                {
                    Ok(connection) => {
                        tracing::info!("Successfully connected to peer {}", peer_addr_clone);
                        if let Err(e) = connection
                            .send_message(Self::build_peer_discovery_message(
                                &discovery.local_node_info,
                            ))
                            .await
                        {
                            tracing::warn!(
                                "Failed to send discovery message to peer {}: {}",
                                peer_addr_clone,
                                e
                            );
                        }
                        let transport_id = connection.transport_id;
                        peers.write().await.insert(transport_id, connection);
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
        let resolved_peer_id = {
            let logical_peers = self.logical_peers.read().await;
            logical_peers.get(&peer_id).copied().unwrap_or(peer_id)
        };

        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(&resolved_peer_id) {
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

        Self::process_discovery_message(
            Arc::clone(&self.peers),
            Arc::clone(&self.logical_peers),
            Arc::clone(&self.discovery),
            peer_id,
            message.clone(),
        )
        .await;

        let handler_peer_id = Self::resolve_handler_peer_id(Arc::clone(&self.peers), peer_id)
            .await
            .unwrap_or(peer_id);

        let handlers = self.message_handlers.read().await;
        for handler in handlers.iter() {
            if let Some(response) = handler.handle_message(handler_peer_id, message.clone())? {
                self.send_to_peer(peer_id, response).await?;
            }
        }

        Ok(())
    }

    fn build_local_peer_info(config: &NodeConfig) -> PeerInfo {
        let public_address = config.public_address();
        let mut parts = public_address.split(':');
        let address = parts.next().unwrap_or("127.0.0.1").to_string();
        let port = parts
            .next()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(config.network.listen_port);

        let mut peer_info = PeerInfo::new(
            config.node_id,
            address,
            port,
            config.network.network_id.clone(),
            config.consensus.is_authority,
        );

        match SemanticContractInfo::load_from_node_config(config) {
            Ok(Some(contract)) => peer_info = peer_info.with_semantic_contract(contract),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(
                    "Failed to load semantic contract info for local node {}: {}",
                    config.node_id,
                    e
                );
            }
        }

        peer_info
    }

    fn build_peer_discovery_message(local_node_info: &PeerInfo) -> P2PMessage {
        match &local_node_info.semantic_contract {
            Some(contract) => P2PMessage::new_peer_discovery_with_contract(
                local_node_info.node_id,
                local_node_info.port,
                local_node_info.network_id.clone(),
                Some(contract.clone()),
            ),
            None => P2PMessage::new_peer_discovery(
                local_node_info.node_id,
                local_node_info.port,
                local_node_info.network_id.clone(),
            ),
        }
    }

    async fn process_discovery_message(
        peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        logical_peers: Arc<RwLock<HashMap<Uuid, Uuid>>>,
        discovery: Arc<PeerDiscovery>,
        peer_id: Uuid,
        message: P2PMessage,
    ) {
        let is_discovery = matches!(
            message,
            P2PMessage::PeerDiscovery { .. } | P2PMessage::PeerList { .. }
        );
        if !is_discovery {
            return;
        }

        let discovered_identity = match &message {
            P2PMessage::PeerDiscovery {
                node_id,
                listen_port,
                network_id,
                semantic_contract,
                ..
            } => Some((
                *node_id,
                *listen_port,
                network_id.clone(),
                semantic_contract.clone(),
            )),
            _ => None,
        };

        match discovery.handle_peer_discovery(message).await {
            Ok(Some(response)) => {
                let should_disconnect = matches!(
                    &response,
                    P2PMessage::Error {
                        error_code: self::messages::ErrorCode::NetworkMismatch
                            | self::messages::ErrorCode::SemanticMismatch,
                        ..
                    }
                );

                Self::send_response_to_peer(Arc::clone(&peers), peer_id, response).await;

                if should_disconnect {
                    Self::disconnect_peer(Arc::clone(&peers), Arc::clone(&logical_peers), peer_id)
                        .await;
                } else if let Some((
                    discovered_node_id,
                    listen_port,
                    network_id,
                    semantic_contract,
                )) = discovered_identity
                {
                    match Self::rekey_connected_peer(
                        Arc::clone(&peers),
                        Arc::clone(&logical_peers),
                        peer_id,
                        discovered_node_id,
                        listen_port,
                        &network_id,
                        semantic_contract,
                    )
                    .await
                    {
                        RekeyOutcome::Updated => {}
                        RekeyOutcome::DuplicateLogicalPeer {
                            existing_transport_id,
                        } => {
                            tracing::warn!(
                                "Rejecting newer transport {} for logical peer {} because transport {} is already active",
                                peer_id,
                                discovered_node_id,
                                existing_transport_id
                            );
                            Self::send_response_to_peer(
                                Arc::clone(&peers),
                                peer_id,
                                P2PMessage::new_error(
                                    ErrorCode::DuplicateLogicalPeer,
                                    format!(
                                        "Logical peer {} is already connected on transport {}",
                                        discovered_node_id, existing_transport_id
                                    ),
                                ),
                            )
                            .await;
                            Self::disconnect_peer(
                                Arc::clone(&peers),
                                Arc::clone(&logical_peers),
                                peer_id,
                            )
                            .await;
                        }
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!("Discovery handling failed for peer {}: {}", peer_id, e);
                Self::disconnect_peer(peers, logical_peers, peer_id).await;
            }
        }
    }

    async fn send_response_to_peer(
        peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        peer_id: Uuid,
        response: P2PMessage,
    ) {
        let sender = {
            let peers_guard = peers.read().await;
            peers_guard.get(&peer_id).map(|peer| peer.sender.clone())
        };

        let Some(sender) = sender else {
            return;
        };

        if let Err(e) = sender.send(response).await {
            tracing::warn!("Failed to send response to peer {}: {}", peer_id, e);
        }
    }

    async fn disconnect_peer(
        peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        logical_peers: Arc<RwLock<HashMap<Uuid, Uuid>>>,
        peer_id: Uuid,
    ) {
        let removed = peers.write().await.remove(&peer_id);
        if let Some(connection) = removed {
            logical_peers
                .write()
                .await
                .retain(|_, transport_id| *transport_id != peer_id);
            connection.task_handle.abort();
            tracing::warn!("Disconnected incompatible peer transport {}", peer_id);
        }
    }

    async fn resolve_handler_peer_id(
        peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        transport_peer_id: Uuid,
    ) -> Option<Uuid> {
        let logical_peer_id_handle = {
            let peers_guard = peers.read().await;
            peers_guard
                .get(&transport_peer_id)
                .map(|peer| Arc::clone(&peer.current_peer_id))
        }?;

        let logical_peer_id = *logical_peer_id_handle.read().await;
        Some(logical_peer_id)
    }

    async fn rekey_connected_peer(
        peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
        logical_peers: Arc<RwLock<HashMap<Uuid, Uuid>>>,
        transport_peer_id: Uuid,
        discovered_node_id: Uuid,
        listen_port: u16,
        network_id: &str,
        semantic_contract: Option<SemanticContractInfo>,
    ) -> RekeyOutcome {
        {
            let logical_peers_guard = logical_peers.read().await;
            if let Some(existing_transport_id) = logical_peers_guard.get(&discovered_node_id) {
                if *existing_transport_id != transport_peer_id {
                    return RekeyOutcome::DuplicateLogicalPeer {
                        existing_transport_id: *existing_transport_id,
                    };
                }
            }
        }

        let logical_id_to_remove = {
            let mut peers_guard = peers.write().await;
            peers_guard.get_mut(&transport_peer_id).map(|connection| {
                let previous_logical_id = connection.info.node_id;
                connection.info.node_id = discovered_node_id;
                connection.info.port = listen_port;
                connection.info.network_id = network_id.to_string();
                connection.info.semantic_contract = semantic_contract;
                (previous_logical_id, connection.current_peer_id.clone())
            })
        };

        if let Some((previous_logical_id, logical_id_handle)) = logical_id_to_remove {
            *logical_id_handle.write().await = discovered_node_id;
            let mut logical_peers_guard = logical_peers.write().await;
            if previous_logical_id != discovered_node_id {
                logical_peers_guard.remove(&previous_logical_id);
            }
            logical_peers_guard.insert(discovered_node_id, transport_peer_id);
        }

        RekeyOutcome::Updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::NodeConfig;
    use chrono::Utc;
    use tokio::sync::{mpsc, RwLock as TokioRwLock};

    #[tokio::test]
    async fn test_network_manager_default_discovery_has_no_semantic_contract() {
        let manager = NetworkManager::new(NodeConfig::default());
        assert!(manager
            .discovery
            .local_node_info
            .semantic_contract
            .is_none());
        assert_eq!(
            manager.discovery.local_node_info.network_id,
            "provchain-org-default"
        );
    }

    #[tokio::test]
    async fn test_network_manager_loads_semantic_contract_from_node_config() {
        let node_config = NodeConfig::load_from_file("config/config.toml").unwrap();
        let manager = NetworkManager::new(node_config);
        let contract = manager
            .discovery
            .local_node_info
            .semantic_contract
            .as_ref()
            .unwrap();

        assert_eq!(contract.network_profile_id, "provchain.default");
        assert_eq!(
            contract.ontology_package_id,
            "provchain.shared-ontology.default"
        );
        assert_eq!(
            contract.ontology_package_hash,
            "720781c9daf1935b8cb2929d7448240767dc205b35236eeed1fc3819af4c28fc"
        );
    }

    fn test_peer_connection(
        transport_id: Uuid,
        logical_id: Uuid,
    ) -> (PeerConnection, mpsc::Receiver<P2PMessage>) {
        let (sender, receiver) = mpsc::channel(8);
        let connection = PeerConnection {
            transport_id,
            info: PeerInfo {
                node_id: logical_id,
                address: "127.0.0.1".to_string(),
                port: 8080,
                network_id: "test-network".to_string(),
                semantic_contract: None,
                last_seen: Utc::now(),
                is_authority: false,
            },
            current_peer_id: Arc::new(TokioRwLock::new(logical_id)),
            sender,
            task_handle: tokio::spawn(async {}),
        };

        (connection, receiver)
    }

    #[tokio::test]
    async fn test_rekey_connected_peer_preserves_transport_id_and_updates_logical_index() {
        let transport_id = Uuid::new_v4();
        let provisional_id = Uuid::new_v4();
        let discovered_id = Uuid::new_v4();
        let peers = Arc::new(RwLock::new(HashMap::new()));
        let logical_peers = Arc::new(RwLock::new(HashMap::new()));
        let (connection, _receiver) = test_peer_connection(transport_id, provisional_id);
        peers.write().await.insert(transport_id, connection);

        NetworkManager::rekey_connected_peer(
            Arc::clone(&peers),
            Arc::clone(&logical_peers),
            transport_id,
            discovered_id,
            9090,
            "aligned-network",
            None,
        )
        .await;

        let peers_guard = peers.read().await;
        let updated_connection = peers_guard.get(&transport_id).unwrap();
        assert_eq!(updated_connection.info.node_id, discovered_id);
        assert_eq!(updated_connection.info.port, 9090);
        assert_eq!(updated_connection.info.network_id, "aligned-network");
        assert_eq!(
            *updated_connection.current_peer_id.read().await,
            discovered_id
        );
        drop(peers_guard);

        let logical_guard = logical_peers.read().await;
        assert_eq!(logical_guard.get(&discovered_id), Some(&transport_id));
        assert!(!logical_guard.contains_key(&provisional_id));
    }

    #[tokio::test]
    async fn test_rekey_connected_peer_rejects_newer_duplicate_logical_peer() {
        let existing_transport_id = Uuid::new_v4();
        let conflicting_transport_id = Uuid::new_v4();
        let existing_logical_id = Uuid::new_v4();
        let peers = Arc::new(RwLock::new(HashMap::new()));
        let logical_peers = Arc::new(RwLock::new(HashMap::new()));

        let (existing_connection, _existing_receiver) =
            test_peer_connection(existing_transport_id, existing_logical_id);
        let (conflicting_connection, _conflicting_receiver) =
            test_peer_connection(conflicting_transport_id, Uuid::new_v4());

        peers
            .write()
            .await
            .insert(existing_transport_id, existing_connection);
        peers
            .write()
            .await
            .insert(conflicting_transport_id, conflicting_connection);
        logical_peers
            .write()
            .await
            .insert(existing_logical_id, existing_transport_id);

        let outcome = NetworkManager::rekey_connected_peer(
            Arc::clone(&peers),
            Arc::clone(&logical_peers),
            conflicting_transport_id,
            existing_logical_id,
            9091,
            "aligned-network",
            None,
        )
        .await;

        match outcome {
            RekeyOutcome::DuplicateLogicalPeer {
                existing_transport_id: active_transport_id,
            } => assert_eq!(active_transport_id, existing_transport_id),
            RekeyOutcome::Updated => panic!("duplicate logical peer should not be accepted"),
        }

        let peers_guard = peers.read().await;
        assert_eq!(
            peers_guard
                .get(&existing_transport_id)
                .unwrap()
                .info
                .node_id,
            existing_logical_id
        );
        assert_ne!(
            peers_guard
                .get(&conflicting_transport_id)
                .unwrap()
                .info
                .node_id,
            existing_logical_id
        );
        drop(peers_guard);

        let logical_guard = logical_peers.read().await;
        assert_eq!(
            logical_guard.get(&existing_logical_id),
            Some(&existing_transport_id)
        );
        assert_eq!(logical_guard.len(), 1);
    }

    #[tokio::test]
    async fn test_send_to_peer_resolves_logical_node_id_to_transport_connection() {
        let manager = NetworkManager::new(NodeConfig::default());
        let transport_id = Uuid::new_v4();
        let logical_id = Uuid::new_v4();
        let (connection, mut receiver) = test_peer_connection(transport_id, logical_id);

        manager.peers.write().await.insert(transport_id, connection);
        manager
            .logical_peers
            .write()
            .await
            .insert(logical_id, transport_id);

        let ping = P2PMessage::new_ping(manager.node_id);
        manager
            .send_to_peer(logical_id, ping.clone())
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.message_type(), ping.message_type());
    }
}
