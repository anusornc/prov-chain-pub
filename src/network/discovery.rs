//! Peer discovery protocol for GraphChain distributed network
//!
//! This module implements peer discovery mechanisms including:
//! - Bootstrap peer connection
//! - Peer list exchange
//! - Network topology maintenance

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::messages::{P2PMessage, PeerInfo};
use super::peer::PeerClient;

/// Peer discovery manager
pub struct PeerDiscovery {
    /// Known peers in the network
    pub known_peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    /// Bootstrap peers (seed nodes)
    pub bootstrap_peers: Vec<String>,
    /// This node's information
    pub local_node_info: PeerInfo,
}

impl PeerDiscovery {
    /// Create a new peer discovery manager
    pub fn new(local_node_info: PeerInfo, bootstrap_peers: Vec<String>) -> Self {
        Self {
            known_peers: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_peers,
            local_node_info,
        }
    }

    /// Start peer discovery process
    pub async fn start_discovery(&self) -> Result<()> {
        info!(
            "Starting peer discovery for node {}",
            self.local_node_info.node_id
        );

        // Connect to bootstrap peers
        self.connect_to_bootstrap_peers().await?;

        // Start periodic peer discovery
        self.start_periodic_discovery().await;

        Ok(())
    }

    /// Connect to bootstrap peers to join the network
    async fn connect_to_bootstrap_peers(&self) -> Result<()> {
        for bootstrap_addr in &self.bootstrap_peers {
            info!("Connecting to bootstrap peer: {}", bootstrap_addr);

            match self.connect_to_peer(bootstrap_addr).await {
                Ok(_) => {
                    info!(
                        "Successfully connected to bootstrap peer: {}",
                        bootstrap_addr
                    );
                    // Request peer list from bootstrap peer
                    self.request_peer_list_from_bootstrap(bootstrap_addr)
                        .await?;
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to bootstrap peer {}: {}",
                        bootstrap_addr, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Connect to a specific peer
    async fn connect_to_peer(&self, peer_address: &str) -> Result<()> {
        // Create a dummy message handler for discovery connections
        let message_handler = Arc::new(|_peer_id: Uuid, _message: P2PMessage| {
            // Handle discovery messages here
        });

        let _connection = PeerClient::connect(peer_address, message_handler).await?;

        // Send peer discovery message
        let discovery_message = P2PMessage::new_peer_discovery(
            self.local_node_info.node_id,
            self.local_node_info.port,
            self.local_node_info.network_id.clone(),
        );

        // In a real implementation, we would send this through the connection
        debug!("Would send discovery message: {:?}", discovery_message);

        Ok(())
    }

    /// Request peer list from a bootstrap peer
    async fn request_peer_list_from_bootstrap(&self, _bootstrap_addr: &str) -> Result<()> {
        // In a real implementation, this would send a peer list request
        // and handle the response to populate known_peers
        debug!("Requesting peer list from bootstrap peer");
        Ok(())
    }

    /// Start periodic peer discovery to maintain network connectivity
    async fn start_periodic_discovery(&self) {
        let known_peers = Arc::clone(&self.known_peers);
        let local_info = self.local_node_info.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                debug!(
                    "Running periodic peer discovery for node {}",
                    local_info.node_id
                );

                // Check health of known peers
                Self::check_peer_health(&known_peers).await;

                // Attempt to discover new peers
                Self::discover_new_peers(&known_peers, &local_info).await;
            }
        });
    }

    /// Check health of known peers and remove stale ones
    async fn check_peer_health(known_peers: &Arc<RwLock<HashMap<Uuid, PeerInfo>>>) {
        let mut peers = known_peers.write().await;
        let now = chrono::Utc::now();
        let stale_threshold = chrono::Duration::minutes(5);

        // Remove peers that haven't been seen recently
        peers.retain(|peer_id, peer_info| {
            let is_stale = now.signed_duration_since(peer_info.last_seen) > stale_threshold;
            if is_stale {
                info!("Removing stale peer: {}", peer_id);
            }
            !is_stale
        });

        debug!("Active peers after health check: {}", peers.len());
    }

    /// Attempt to discover new peers through existing connections
    async fn discover_new_peers(
        known_peers: &Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
        _local_info: &PeerInfo,
    ) {
        let peers = known_peers.read().await;

        // In a real implementation, this would:
        // 1. Send peer discovery requests to known peers
        // 2. Process responses to learn about new peers
        // 3. Attempt connections to new peers

        debug!(
            "Attempting to discover new peers through {} known peers",
            peers.len()
        );
    }

    /// Add a newly discovered peer
    pub async fn add_discovered_peer(&self, peer_info: PeerInfo) -> Result<()> {
        let mut peers = self.known_peers.write().await;

        // Check if peer is already known
        if let std::collections::hash_map::Entry::Vacant(e) = peers.entry(peer_info.node_id) {
            // Add new peer
            info!(
                "Discovered new peer: {} at {}",
                peer_info.node_id,
                peer_info.full_address()
            );
            e.insert(peer_info);
        } else {
            // Update existing peer info
            if let Some(existing_peer) = peers.get_mut(&peer_info.node_id) {
                existing_peer.update_last_seen();
                debug!("Updated existing peer: {}", peer_info.node_id);
            }
        }

        Ok(())
    }

    /// Get list of known peers
    pub async fn get_known_peers(&self) -> Vec<PeerInfo> {
        let peers = self.known_peers.read().await;
        peers.values().cloned().collect()
    }

    /// Handle incoming peer discovery message
    pub async fn handle_peer_discovery(
        &self,
        peer_discovery: P2PMessage,
    ) -> Result<Option<P2PMessage>> {
        match peer_discovery {
            P2PMessage::PeerDiscovery {
                node_id,
                listen_port,
                network_id,
                ..
            } => {
                // Validate network ID
                if network_id != self.local_node_info.network_id {
                    warn!("Peer {} has different network ID: {}", node_id, network_id);
                    return Ok(Some(P2PMessage::new_error(
                        super::messages::ErrorCode::NetworkMismatch,
                        "Network ID mismatch".to_string(),
                    )));
                }

                // Add peer to known peers
                let peer_info = PeerInfo::new(
                    node_id,
                    "unknown".to_string(), // Address would be extracted from connection
                    listen_port,
                    network_id,
                    false, // Authority status would be determined separately
                );

                self.add_discovered_peer(peer_info).await?;

                // Respond with our peer list
                let known_peers = self.get_known_peers().await;
                Ok(Some(P2PMessage::PeerList {
                    peers: known_peers,
                    timestamp: chrono::Utc::now(),
                }))
            }
            P2PMessage::PeerList { peers, .. } => {
                // Process received peer list
                for peer_info in peers {
                    if peer_info.node_id != self.local_node_info.node_id {
                        self.add_discovered_peer(peer_info).await?;
                    }
                }
                Ok(None)
            }
            _ => {
                // Not a discovery message
                Ok(None)
            }
        }
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let peers = self.known_peers.read().await;
        let authority_count = peers.values().filter(|p| p.is_authority).count();

        NetworkStats {
            total_peers: peers.len(),
            authority_peers: authority_count,
            regular_peers: peers.len() - authority_count,
            bootstrap_peers: self.bootstrap_peers.len(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_peers: usize,
    pub authority_peers: usize,
    pub regular_peers: usize,
    pub bootstrap_peers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_discovery_creation() {
        let local_info = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8080,
            "test-network".to_string(),
            false,
        );

        let bootstrap_peers = vec!["127.0.0.1:8081".to_string()];
        let discovery = PeerDiscovery::new(local_info, bootstrap_peers);

        assert_eq!(discovery.bootstrap_peers.len(), 1);
        assert_eq!(discovery.get_known_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_discovered_peer() {
        let local_info = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8080,
            "test-network".to_string(),
            false,
        );

        let discovery = PeerDiscovery::new(local_info, vec![]);

        let peer_info = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8081,
            "test-network".to_string(),
            false,
        );

        discovery.add_discovered_peer(peer_info).await.unwrap();
        assert_eq!(discovery.get_known_peers().await.len(), 1);
    }

    #[tokio::test]
    async fn test_network_stats() {
        let local_info = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8080,
            "test-network".to_string(),
            false,
        );

        let discovery = PeerDiscovery::new(local_info, vec!["127.0.0.1:8081".to_string()]);

        // Add a regular peer
        let regular_peer = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8081,
            "test-network".to_string(),
            false,
        );
        discovery.add_discovered_peer(regular_peer).await.unwrap();

        // Add an authority peer
        let authority_peer = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8082,
            "test-network".to_string(),
            true,
        );
        discovery.add_discovered_peer(authority_peer).await.unwrap();

        let stats = discovery.get_network_stats().await;
        assert_eq!(stats.total_peers, 2);
        assert_eq!(stats.authority_peers, 1);
        assert_eq!(stats.regular_peers, 1);
        assert_eq!(stats.bootstrap_peers, 1);
    }
}
