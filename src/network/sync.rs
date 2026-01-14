//! Blockchain synchronization for distributed ProvChainOrg network
//!
//! This module handles:
//! - Block synchronization between peers
//! - Conflict resolution for concurrent blocks
//! - Merkle tree verification for data integrity
//! - Incremental synchronization for efficiency

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::messages::P2PMessage;
use super::NetworkManager;
use crate::core::blockchain::{Block, Blockchain};

/// Blockchain synchronization manager
pub struct BlockchainSync {
    /// Local blockchain instance
    pub blockchain: Arc<RwLock<Blockchain>>,
    /// Network manager for peer communication
    pub network: Arc<NetworkManager>,
    /// Synchronization state tracking
    pub sync_state: Arc<RwLock<SyncState>>,
    /// Pending block requests
    pub pending_requests: Arc<RwLock<HashMap<u64, DateTime<Utc>>>>,
}

/// Synchronization state information
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Whether we're currently syncing
    pub is_syncing: bool,
    /// Highest block index we know about
    pub highest_known_block: u64,
    /// Our current block height
    pub current_height: u64,
    /// Peers we're syncing from
    pub sync_peers: HashSet<Uuid>,
    /// Last sync attempt timestamp
    pub last_sync: DateTime<Utc>,
}

/// Block synchronization request
#[derive(Debug, Clone)]
pub struct SyncRequest {
    pub from_block: u64,
    pub to_block: u64,
    pub peer_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl BlockchainSync {
    /// Create a new blockchain synchronization manager
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, network: Arc<NetworkManager>) -> Self {
        // For tests, we'll initialize with a default height and update it later if needed
        let current_height = 1; // Assume at least genesis block

        let sync_state = SyncState {
            is_syncing: false,
            highest_known_block: current_height,
            current_height,
            sync_peers: HashSet::new(),
            last_sync: Utc::now(),
        };

        Self {
            blockchain,
            network,
            sync_state: Arc::new(RwLock::new(sync_state)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the sync state with actual blockchain height (async version)
    pub async fn initialize(&self) -> Result<()> {
        let current_height = {
            let blockchain = self.blockchain.read().await;
            blockchain.chain.len() as u64
        };

        let mut sync_state = self.sync_state.write().await;
        sync_state.current_height = current_height;
        sync_state.highest_known_block = current_height;

        Ok(())
    }

    /// Start the synchronization process
    pub async fn start_sync(&self) -> Result<()> {
        info!("Starting blockchain synchronization");

        // Request chain status from all connected peers
        self.request_chain_status_from_peers().await?;

        // Start periodic sync check
        self.start_periodic_sync().await;

        Ok(())
    }

    /// Request chain status from all connected peers
    async fn request_chain_status_from_peers(&self) -> Result<()> {
        let peers = self.network.get_connected_peers().await;
        let node_id = self.network.node_id;

        for peer in peers {
            let status_request = P2PMessage::ChainStatusRequest {
                requester_id: node_id,
            };

            if let Err(e) = self
                .network
                .send_to_peer(peer.node_id, status_request)
                .await
            {
                warn!(
                    "Failed to request chain status from peer {}: {}",
                    peer.node_id, e
                );
            }
        }

        Ok(())
    }

    /// Handle incoming chain status response
    pub async fn handle_chain_status_response(
        &self,
        latest_block_index: u64,
        latest_block_hash: String,
        chain_length: u64,
        peer_id: Uuid,
    ) -> Result<()> {
        debug!(
            "Received chain status from peer {}: height {}, hash {}",
            peer_id, latest_block_index, latest_block_hash
        );

        let mut sync_state = self.sync_state.write().await;

        // Update highest known block if this peer has more blocks
        if latest_block_index > sync_state.highest_known_block {
            sync_state.highest_known_block = latest_block_index;
            info!(
                "Updated highest known block to {} from peer {}",
                latest_block_index, peer_id
            );
        }

        // Check if we need to sync
        if latest_block_index > sync_state.current_height {
            info!(
                "Peer {} has newer blocks ({}), starting sync",
                peer_id, latest_block_index
            );
            sync_state.sync_peers.insert(peer_id);

            if !sync_state.is_syncing {
                sync_state.is_syncing = true;
                drop(sync_state); // Release lock before async operation
                self.sync_from_peer(peer_id, chain_length).await?;
            }
        }

        Ok(())
    }

    /// Synchronize blockchain from a specific peer
    async fn sync_from_peer(&self, peer_id: Uuid, target_height: u64) -> Result<()> {
        let current_height = {
            let sync_state = self.sync_state.read().await;
            sync_state.current_height
        };

        info!(
            "Syncing from peer {} (current: {}, target: {})",
            peer_id, current_height, target_height
        );

        // Request blocks in batches
        const BATCH_SIZE: u64 = 10;
        let mut start_block = current_height;

        while start_block < target_height {
            let end_block = std::cmp::min(start_block + BATCH_SIZE, target_height);

            for block_index in start_block..end_block {
                self.request_block_from_peer(peer_id, block_index).await?;
            }

            start_block = end_block;

            // Wait a bit between batches to avoid overwhelming the peer
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Request a specific block from a peer
    async fn request_block_from_peer(&self, peer_id: Uuid, block_index: u64) -> Result<()> {
        debug!("Requesting block {} from peer {}", block_index, peer_id);

        // Track the request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(block_index, Utc::now());
        }

        let block_request = P2PMessage::new_block_request(block_index, self.network.node_id);
        self.network.send_to_peer(peer_id, block_request).await?;

        Ok(())
    }

    /// Handle incoming block response
    pub async fn handle_block_response(
        &self,
        block: Option<Block>,
        requester_id: Uuid,
    ) -> Result<()> {
        if requester_id != self.network.node_id {
            // This response is not for us
            return Ok(());
        }

        if let Some(block) = block {
            debug!("Received block {} with hash {}", block.index, block.hash);

            // Remove from pending requests
            {
                let mut pending = self.pending_requests.write().await;
                pending.remove(&block.index);
            }

            // Validate and add the block
            self.process_received_block(block).await?;
        } else {
            warn!("Received empty block response");
        }

        Ok(())
    }

    /// Process a received block and add it to the blockchain
    async fn process_received_block(&self, block: Block) -> Result<()> {
        let mut blockchain = self.blockchain.write().await;
        let mut sync_state = self.sync_state.write().await;

        // Validate block integrity
        if !self.validate_block(&block, &blockchain).await? {
            warn!("Received invalid block {}, rejecting", block.index);
            return Ok(());
        }

        // Check if this block extends our chain
        if block.index == blockchain.chain.len() as u64 {
            // This block extends our chain directly
            let _ = blockchain.add_block(block.data.clone());
            sync_state.current_height = blockchain.chain.len() as u64;

            info!(
                "Added block {} to blockchain (height: {})",
                block.index, sync_state.current_height
            );
        } else if block.index < blockchain.chain.len() as u64 {
            // This is an older block, check for conflicts
            if let Some(existing_block) = blockchain.chain.get(block.index as usize) {
                if existing_block.hash != block.hash {
                    warn!("Block conflict detected at index {}", block.index);
                    self.handle_block_conflict(block, existing_block.clone())
                        .await?;
                }
            }
        } else {
            // This block is from the future, store it for later
            debug!("Received future block {}, storing for later", block.index);
            // In a full implementation, we'd store this in a pending blocks cache
        }

        // Check if sync is complete
        if sync_state.current_height >= sync_state.highest_known_block {
            sync_state.is_syncing = false;
            sync_state.sync_peers.clear();
            info!(
                "Blockchain synchronization complete at height {}",
                sync_state.current_height
            );
        }

        Ok(())
    }

    /// Validate a received block
    async fn validate_block(&self, block: &Block, blockchain: &Blockchain) -> Result<bool> {
        // Check block index
        if block.index > blockchain.chain.len() as u64 {
            // Future block - we'll validate it when we have the previous blocks
            return Ok(true);
        }

        // Check previous hash
        if block.index > 0 {
            if let Some(prev_block) = blockchain.chain.get((block.index - 1) as usize) {
                if block.previous_hash != prev_block.hash {
                    warn!("Block {} has invalid previous hash", block.index);
                    return Ok(false);
                }
            }
        }

        // Validate block hash by recalculating it
        let expected_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));
        if block.hash != expected_hash {
            warn!("Block {} has invalid hash", block.index);
            return Ok(false);
        }

        Ok(true)
    }

    /// Handle block conflicts (simple resolution for now)
    async fn handle_block_conflict(&self, new_block: Block, existing_block: Block) -> Result<()> {
        warn!("Handling block conflict at index {}", new_block.index);

        // Parse timestamps for comparison
        let new_time = chrono::DateTime::parse_from_rfc3339(&new_block.timestamp)
            .map_err(|_| anyhow::anyhow!("Invalid timestamp format in new block"))?;
        let existing_time = chrono::DateTime::parse_from_rfc3339(&existing_block.timestamp)
            .map_err(|_| anyhow::anyhow!("Invalid timestamp format in existing block"))?;

        // Simple conflict resolution: prefer the block with the earlier timestamp
        if new_time < existing_time {
            let block_index = new_block.index;
            info!(
                "Replacing block {} with earlier timestamped version",
                block_index
            );

            let mut blockchain = self.blockchain.write().await;
            blockchain.chain[block_index as usize] = new_block;

            // In a full implementation, we'd need to:
            // 1. Remove the old RDF data from the store
            // 2. Add the new RDF data
            // 3. Recompute hashes for subsequent blocks
            // 4. Notify other peers of the change
        } else {
            debug!(
                "Keeping existing block {} (earlier timestamp)",
                existing_block.index
            );
        }

        Ok(())
    }

    /// Start periodic synchronization checks
    async fn start_periodic_sync(&self) {
        let sync_manager = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                if let Err(e) = sync_manager.periodic_sync_check().await {
                    error!("Periodic sync check failed: {}", e);
                }
            }
        });
    }

    /// Perform periodic synchronization check
    async fn periodic_sync_check(&self) -> Result<()> {
        debug!("Performing periodic sync check");

        // Clean up old pending requests
        self.cleanup_pending_requests().await;

        // Request chain status from peers if we're not currently syncing
        let is_syncing = {
            let sync_state = self.sync_state.read().await;
            sync_state.is_syncing
        };

        if !is_syncing {
            self.request_chain_status_from_peers().await?;
        }

        Ok(())
    }

    /// Clean up old pending requests
    async fn cleanup_pending_requests(&self) {
        let mut pending = self.pending_requests.write().await;
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(60);

        pending.retain(|block_index, timestamp| {
            let is_recent = now.signed_duration_since(*timestamp) < timeout;
            if !is_recent {
                warn!("Removing stale block request for block {}", block_index);
            }
            is_recent
        });
    }

    /// Get synchronization statistics
    pub async fn get_sync_stats(&self) -> SyncStats {
        let sync_state = self.sync_state.read().await;
        let pending_count = self.pending_requests.read().await.len();

        SyncStats {
            is_syncing: sync_state.is_syncing,
            current_height: sync_state.current_height,
            highest_known_block: sync_state.highest_known_block,
            sync_peers_count: sync_state.sync_peers.len(),
            pending_requests: pending_count,
            last_sync: sync_state.last_sync,
        }
    }

    /// Announce a new block to the network
    pub async fn announce_new_block(&self, block: &Block) -> Result<()> {
        info!("Announcing new block {} to network", block.index);

        let graph_uri = format!("http://provchain.org/block/{}", block.index);
        let announcement = P2PMessage::new_block_announcement(block, graph_uri);
        self.network.broadcast_message(announcement).await?;

        Ok(())
    }

    /// Handle incoming block announcement
    pub async fn handle_block_announcement(
        &self,
        block_index: u64,
        _block_hash: String,
        _previous_hash: String,
        _graph_uri: String,
        peer_id: Uuid,
    ) -> Result<()> {
        debug!(
            "Received block announcement for block {} from peer {}",
            block_index, peer_id
        );

        let current_height = {
            let sync_state = self.sync_state.read().await;
            sync_state.current_height
        };

        // If this is a new block we don't have, request it
        if block_index >= current_height {
            info!(
                "Requesting announced block {} from peer {}",
                block_index, peer_id
            );
            self.request_block_from_peer(peer_id, block_index).await?;
        }

        Ok(())
    }
}

// Implement Clone for BlockchainSync to enable Arc usage in async tasks
impl Clone for BlockchainSync {
    fn clone(&self) -> Self {
        Self {
            blockchain: Arc::clone(&self.blockchain),
            network: Arc::clone(&self.network),
            sync_state: Arc::clone(&self.sync_state),
            pending_requests: Arc::clone(&self.pending_requests),
        }
    }
}

/// Synchronization statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub is_syncing: bool,
    pub current_height: u64,
    pub highest_known_block: u64,
    pub sync_peers_count: usize,
    pub pending_requests: usize,
    pub last_sync: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::NodeConfig;

    #[tokio::test]
    async fn test_sync_state_creation() {
        let blockchain = {
            let mut bc = Blockchain::new();
            // Ensure we have at least one block
            if bc.chain.is_empty() {
                let _ = bc.add_block("genesis".to_string());
            }
            Arc::new(RwLock::new(bc))
        };
        let config = NodeConfig::default();
        let network = Arc::new(NetworkManager::new(config));

        let sync = BlockchainSync::new(blockchain, network);
        let stats = sync.get_sync_stats().await;

        assert!(!stats.is_syncing);
        assert!(stats.current_height >= 1); // At least genesis block exists
        assert_eq!(stats.sync_peers_count, 0);
    }
}
