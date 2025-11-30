//! P2P message protocol for GraphChain distributed communication
//!
//! This module defines all message types used for communication between
//! GraphChain nodes, including blockchain synchronization, peer discovery,
//! and RDF graph exchange.

use crate::core::blockchain::Block;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All possible P2P messages exchanged between GraphChain nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum P2PMessage {
    /// Peer discovery and network management
    PeerDiscovery {
        node_id: Uuid,
        listen_port: u16,
        network_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Response to peer discovery with peer list
    PeerList {
        peers: Vec<PeerInfo>,
        timestamp: DateTime<Utc>,
    },

    /// Announce a new block to the network
    BlockAnnouncement {
        block_index: u64,
        block_hash: String,
        previous_hash: String,
        graph_uri: String,
        timestamp: DateTime<Utc>,
    },

    /// Request a specific block by index
    BlockRequest {
        block_index: u64,
        requester_id: Uuid,
    },

    /// Response with requested block data
    BlockResponse {
        block: Option<Block>,
        requester_id: Uuid,
    },

    /// Request RDF graph data for a specific URI
    GraphRequest {
        graph_uri: String,
        requester_id: Uuid,
    },

    /// Response with RDF graph data
    GraphResponse {
        graph_uri: String,
        rdf_data: Option<String>, // Turtle format, compressed and base64 encoded
        requester_id: Uuid,
    },

    /// Request chain status (latest block info)
    ChainStatusRequest { requester_id: Uuid },

    /// Response with chain status
    ChainStatusResponse {
        latest_block_index: u64,
        latest_block_hash: String,
        chain_length: u64,
        requester_id: Uuid,
    },

    /// Ping message for connection health check
    Ping {
        sender_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Pong response to ping
    Pong {
        sender_id: Uuid,
        original_timestamp: DateTime<Utc>,
        response_timestamp: DateTime<Utc>,
    },

    /// Error message
    Error {
        error_code: ErrorCode,
        message: String,
        timestamp: DateTime<Utc>,
    },
}

/// Information about a peer node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: Uuid,
    pub address: String,
    pub port: u16,
    pub network_id: String,
    pub last_seen: DateTime<Utc>,
    pub is_authority: bool,
}

/// Error codes for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    InvalidMessage,
    BlockNotFound,
    GraphNotFound,
    NetworkMismatch,
    AuthenticationFailed,
    InternalError,
}

impl P2PMessage {
    /// Create a new peer discovery message
    pub fn new_peer_discovery(node_id: Uuid, listen_port: u16, network_id: String) -> Self {
        Self::PeerDiscovery {
            node_id,
            listen_port,
            network_id,
            timestamp: Utc::now(),
        }
    }

    /// Create a new block announcement
    pub fn new_block_announcement(block: &Block, graph_uri: String) -> Self {
        Self::BlockAnnouncement {
            block_index: block.index,
            block_hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            graph_uri,
            timestamp: Utc::now(),
        }
    }

    /// Create a new block request
    pub fn new_block_request(block_index: u64, requester_id: Uuid) -> Self {
        Self::BlockRequest {
            block_index,
            requester_id,
        }
    }

    /// Create a new graph request
    pub fn new_graph_request(graph_uri: String, requester_id: Uuid) -> Self {
        Self::GraphRequest {
            graph_uri,
            requester_id,
        }
    }

    /// Create a new ping message
    pub fn new_ping(sender_id: Uuid) -> Self {
        Self::Ping {
            sender_id,
            timestamp: Utc::now(),
        }
    }

    /// Create a new pong response
    pub fn new_pong(sender_id: Uuid, original_timestamp: DateTime<Utc>) -> Self {
        Self::Pong {
            sender_id,
            original_timestamp,
            response_timestamp: Utc::now(),
        }
    }

    /// Create a new error message
    pub fn new_error(error_code: ErrorCode, message: String) -> Self {
        Self::Error {
            error_code,
            message,
            timestamp: Utc::now(),
        }
    }

    /// Get the message type as a string for logging
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::PeerDiscovery { .. } => "PeerDiscovery",
            Self::PeerList { .. } => "PeerList",
            Self::BlockAnnouncement { .. } => "BlockAnnouncement",
            Self::BlockRequest { .. } => "BlockRequest",
            Self::BlockResponse { .. } => "BlockResponse",
            Self::GraphRequest { .. } => "GraphRequest",
            Self::GraphResponse { .. } => "GraphResponse",
            Self::ChainStatusRequest { .. } => "ChainStatusRequest",
            Self::ChainStatusResponse { .. } => "ChainStatusResponse",
            Self::Ping { .. } => "Ping",
            Self::Pong { .. } => "Pong",
            Self::Error { .. } => "Error",
        }
    }

    /// Serialize message to JSON bytes for network transmission
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let json = serde_json::to_string(self)?;
        Ok(json.into_bytes())
    }

    /// Deserialize message from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let json = std::str::from_utf8(bytes)?;
        let message = serde_json::from_str(json)?;
        Ok(message)
    }

    /// Validate message structure and content
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::PeerDiscovery { network_id, .. } => {
                if network_id.is_empty() {
                    anyhow::bail!("Network ID cannot be empty");
                }
            }
            Self::BlockRequest { block_index, .. } => {
                // Block index validation could be added here
                if *block_index == u64::MAX {
                    anyhow::bail!("Invalid block index");
                }
            }
            Self::GraphRequest { graph_uri, .. } => {
                if graph_uri.is_empty() {
                    anyhow::bail!("Graph URI cannot be empty");
                }
            }
            _ => {} // Other messages don't need special validation
        }
        Ok(())
    }
}

impl PeerInfo {
    /// Create a new peer info
    pub fn new(
        node_id: Uuid,
        address: String,
        port: u16,
        network_id: String,
        is_authority: bool,
    ) -> Self {
        Self {
            node_id,
            address,
            port,
            network_id,
            last_seen: Utc::now(),
            is_authority,
        }
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Get the full address (address:port)
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let node_id = Uuid::new_v4();
        let message = P2PMessage::new_peer_discovery(node_id, 8080, "test-network".to_string());

        // Test serialization
        let bytes = message.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Test deserialization
        let deserialized = P2PMessage::from_bytes(&bytes).unwrap();

        // Verify the message type matches
        assert_eq!(message.message_type(), deserialized.message_type());
    }

    #[test]
    fn test_message_validation() {
        let node_id = Uuid::new_v4();

        // Valid message
        let valid_message =
            P2PMessage::new_peer_discovery(node_id, 8080, "test-network".to_string());
        assert!(valid_message.validate().is_ok());

        // Invalid message with empty network ID
        let invalid_message = P2PMessage::PeerDiscovery {
            node_id,
            listen_port: 8080,
            network_id: String::new(),
            timestamp: Utc::now(),
        };
        assert!(invalid_message.validate().is_err());
    }

    #[test]
    fn test_peer_info() {
        let node_id = Uuid::new_v4();
        let mut peer = PeerInfo::new(
            node_id,
            "127.0.0.1".to_string(),
            8080,
            "test-network".to_string(),
            false,
        );

        assert_eq!(peer.full_address(), "127.0.0.1:8080");

        let original_time = peer.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(1));
        peer.update_last_seen();
        assert!(peer.last_seen > original_time);
    }
}
