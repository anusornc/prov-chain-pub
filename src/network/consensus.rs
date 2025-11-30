//! Proof-of-Authority consensus mechanism for ProvChainOrg
//!
//! This module implements:
//! - Ed25519 signature-based authority validation
//! - Authority node management and rotation
//! - Byzantine fault tolerance considerations
//! - Block creation and validation rules

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::messages::P2PMessage;
use super::NetworkManager;
use crate::core::blockchain::{Block, Blockchain};
use crate::utils::config::ConsensusConfig;

/// Proof-of-Authority consensus manager
pub struct ConsensusManager {
    /// Node configuration
    pub config: ConsensusConfig,
    /// Authority keypair (if this node is an authority)
    pub authority_keypair: Option<SigningKey>,
    /// Known authority public keys
    pub authority_keys: Arc<RwLock<HashMap<Uuid, VerifyingKey>>>,
    /// Network manager for communication
    pub network: Arc<NetworkManager>,
    /// Blockchain instance
    pub blockchain: Arc<RwLock<Blockchain>>,
    /// Authority rotation state
    pub authority_state: Arc<RwLock<AuthorityState>>,
}

/// Authority state tracking
#[derive(Debug, Clone)]
pub struct AuthorityState {
    /// Current authority rotation round
    pub current_round: u64,
    /// Authority that should create the next block
    pub current_authority: Option<Uuid>,
    /// Last block creation time
    pub last_block_time: DateTime<Utc>,
    /// Authority performance tracking
    pub authority_performance: HashMap<Uuid, AuthorityPerformance>,
    /// List of authority IDs in rotation order
    pub authority_rotation_order: Vec<Uuid>,
    /// Index of current authority in rotation
    pub current_authority_index: usize,
}

/// Authority performance metrics
#[derive(Debug, Clone)]
pub struct AuthorityPerformance {
    /// Number of blocks created
    pub blocks_created: u64,
    /// Number of missed slots
    pub missed_slots: u64,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Reputation score (0.0 to 1.0)
    pub reputation: f64,
}

/// Block creation proposal
#[derive(Debug, Clone)]
pub struct BlockProposal {
    /// Proposed block
    pub block: Block,
    /// Authority signature
    pub signature: Signature,
    /// Authority public key
    pub authority_key: VerifyingKey,
    /// Proposal timestamp
    pub timestamp: DateTime<Utc>,
}

impl ConsensusManager {
    /// Create a new consensus manager
    pub fn new(
        config: ConsensusConfig,
        network: Arc<NetworkManager>,
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<Self> {
        let authority_keypair = if config.is_authority {
            Some(Self::load_or_generate_keypair(&config.authority_key_file)?)
        } else {
            None
        };

        let authority_keys = Arc::new(RwLock::new(HashMap::new()));

        // Load known authority keys
        for key_str in &config.authority_keys {
            if let Ok(key_bytes) = hex::decode(key_str) {
                if key_bytes.len() == 32 {
                    if let Ok(public_key) = VerifyingKey::from_bytes(&key_bytes.try_into().unwrap())
                    {
                        let authority_id = Uuid::new_v4(); // In practice, this would be derived from the key
                        authority_keys
                            .blocking_write()
                            .insert(authority_id, public_key);
                    }
                }
            }
        }

        let authority_state = AuthorityState {
            current_round: 0,
            current_authority: None,
            last_block_time: Utc::now(),
            authority_performance: HashMap::new(),
            authority_rotation_order: Vec::new(),
            current_authority_index: 0,
        };

        Ok(Self {
            config,
            authority_keypair,
            authority_keys,
            network,
            blockchain,
            authority_state: Arc::new(RwLock::new(authority_state)),
        })
    }

    /// Load or generate authority keypair
    fn load_or_generate_keypair(key_file: &Option<String>) -> Result<SigningKey> {
        if let Some(file_path) = key_file {
            // Try to load existing keypair
            if std::path::Path::new(file_path).exists() {
                let key_data = std::fs::read(file_path)?;
                if key_data.len() == 32 {
                    let keypair = SigningKey::from_bytes(&key_data.try_into().unwrap());
                    info!("Loaded authority keypair from {}", file_path);
                    return Ok(keypair);
                }
            }

            // Generate new keypair and save it
            let keypair = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            std::fs::write(file_path, keypair.to_bytes())?;
            info!("Generated new authority keypair and saved to {}", file_path);
            Ok(keypair)
        } else {
            // Generate ephemeral keypair
            Ok(SigningKey::from_bytes(&rand::random::<[u8; 32]>()))
        }
    }

    /// Start the consensus mechanism
    pub async fn start(&self) -> Result<()> {
        info!("Starting Proof-of-Authority consensus");

        if self.config.is_authority {
            info!("This node is configured as an authority");
            self.start_authority_duties().await?;
        } else {
            info!("This node is a regular peer (not an authority)");
        }

        // Start authority monitoring
        self.start_authority_monitoring().await;

        Ok(())
    }

    /// Start authority duties (block creation)
    async fn start_authority_duties(&self) -> Result<()> {
        let consensus_manager = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                consensus_manager.config.block_interval,
            ));

            loop {
                interval.tick().await;

                if let Err(e) = consensus_manager.try_create_block().await {
                    error!("Failed to create block: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Try to create a new block (if it's our turn)
    async fn try_create_block(&self) -> Result<()> {
        let should_create = self.should_create_block().await?;

        if should_create {
            debug!("It's our turn to create a block");
            self.create_and_propose_block().await?;
        }

        Ok(())
    }

    /// Check if this authority should create the next block
    async fn should_create_block(&self) -> Result<bool> {
        let authority_state = self.authority_state.read().await;
        let now = Utc::now();

        // Check if enough time has passed since the last block
        let time_since_last = now.signed_duration_since(authority_state.last_block_time);
        if time_since_last < Duration::seconds(self.config.block_interval as i64) {
            return Ok(false);
        }

        // Check if it's our turn in the round-robin rotation
        let authority_keys = self.authority_keys.read().await;
        if authority_keys.is_empty() {
            return Ok(false);
        }

        // Get our authority ID
        let our_authority_id = if let Some(keypair) = &self.authority_keypair {
            // In a real implementation, we would derive the ID from the public key
            // For now, we'll use a placeholder approach
            let public_key = keypair.verifying_key();
            // Find our ID in the authority keys
            authority_keys.iter().find_map(
                |(id, key)| {
                    if key == &public_key {
                        Some(*id)
                    } else {
                        None
                    }
                },
            )
        } else {
            None
        };

        if let Some(our_id) = our_authority_id {
            // Check if we're the current authority in rotation
            if let Some(current_authority) = authority_state.current_authority {
                Ok(current_authority == our_id)
            } else {
                // If no current authority is set, check if we're first in rotation
                if !authority_state.authority_rotation_order.is_empty() {
                    Ok(authority_state.authority_rotation_order[0] == our_id)
                } else {
                    // Fallback: any authority can create if rotation order is empty
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Create and propose a new block
    async fn create_and_propose_block(&self) -> Result<()> {
        info!("Creating new block");

        let keypair = self
            .authority_keypair
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authority keypair available"))?;

        // Create the block
        let block = self.create_block().await?;

        // Sign the block
        let block_data = self.serialize_block_for_signing(&block)?;
        let signature = keypair.sign(&block_data);

        let proposal = BlockProposal {
            block: block.clone(),
            signature,
            authority_key: keypair.verifying_key(),
            timestamp: Utc::now(),
        };

        // Add block to our local blockchain using atomic operations
        {
            let mut blockchain = self.blockchain.write().await;
            // Update block signature before submitting
            let mut signed_block = block.clone();
            signed_block.signature = hex::encode(signature.to_bytes());
            blockchain.submit_signed_block(signed_block)?;
        }

        // Broadcast the block to the network
        self.broadcast_block_proposal(proposal).await?;

        // Update authority state for round-robin rotation
        {
            let mut authority_state = self.authority_state.write().await;
            authority_state.last_block_time = Utc::now();
            authority_state.current_round += 1;

            // Update round-robin rotation
            if !authority_state.authority_rotation_order.is_empty() {
                authority_state.current_authority_index = (authority_state.current_authority_index
                    + 1)
                    % authority_state.authority_rotation_order.len();

                if authority_state.current_authority_index
                    < authority_state.authority_rotation_order.len()
                {
                    authority_state.current_authority = Some(
                        authority_state.authority_rotation_order
                            [authority_state.current_authority_index],
                    );
                }
            }

            // Update performance metrics
            if let Some(current_authority) = authority_state.current_authority {
                if let Some(performance) = authority_state
                    .authority_performance
                    .get_mut(&current_authority)
                {
                    performance.blocks_created += 1;
                    performance.last_activity = Utc::now();
                }
            }
        }

        info!("Successfully created and broadcast block {}", block.index);
        Ok(())
    }

    /// Create a new block with pending transactions/data
    async fn create_block(&self) -> Result<Block> {
        let blockchain = self.blockchain.read().await;
        let previous_block = blockchain.chain.last();

        let (previous_hash, index) = if let Some(prev) = previous_block {
            (prev.hash.clone(), prev.index + 1)
        } else {
            ("0".repeat(64), 0)
        };

        // For now, create a block with some sample RDF data
        // In a real implementation, this would collect pending transactions
        // Calculate state root before creating the block
        let state_root = self
            .blockchain
            .read()
            .await
            .rdf_store
            .calculate_state_root();

        let rdf_data = format!(
            "<http://provchain.org/block/{}> <http://provchain.org/timestamp> \"{}\" .\n<http://provchain.org/block/{}> <http://provchain.org/authority> \"{}\" .",
            index,
            Utc::now().to_rfc3339(),
            index,
            self.network.node_id
        );

        Ok(Block::new(
            index,
            rdf_data,
            previous_hash,
            state_root,
            self.network.node_id.to_string(),
        ))
    }

    /// Serialize block data for signing
    fn serialize_block_for_signing(&self, block: &Block) -> Result<Vec<u8>> {
        let data = format!(
            "{}|{}|{}|{}",
            block.index, block.timestamp, block.previous_hash, block.data
        );
        Ok(data.into_bytes())
    }

    /// Broadcast block proposal to the network
    async fn broadcast_block_proposal(&self, proposal: BlockProposal) -> Result<()> {
        let announcement = P2PMessage::new_block_announcement(
            &proposal.block,
            format!("http://provchain.org/block/{}", proposal.block.index),
        );

        self.network.broadcast_message(announcement).await?;
        Ok(())
    }

    /// Validate a block proposal from another authority
    pub async fn validate_block_proposal(&self, proposal: &BlockProposal) -> Result<bool> {
        debug!(
            "Validating block proposal for block {}",
            proposal.block.index
        );

        // Check if the authority is known and authorized
        let authority_keys = self.authority_keys.read().await;
        let is_known_authority = authority_keys
            .values()
            .any(|key| *key == proposal.authority_key);

        if !is_known_authority {
            warn!("Block proposal from unknown authority");
            return Ok(false);
        }

        // Verify the signature
        let block_data = self.serialize_block_for_signing(&proposal.block)?;
        if proposal
            .authority_key
            .verify(&block_data, &proposal.signature)
            .is_err()
        {
            warn!("Invalid signature on block proposal");
            return Ok(false);
        }

        // Validate block structure and content
        let blockchain = self.blockchain.read().await;
        if !self
            .validate_block_structure(&proposal.block, &blockchain)
            .await?
        {
            warn!("Invalid block structure");
            return Ok(false);
        }

        // Check timing constraints
        if !self.validate_block_timing(proposal).await? {
            warn!("Block proposal violates timing constraints");
            return Ok(false);
        }

        debug!("Block proposal validation successful");
        Ok(true)
    }

    /// Validate block structure and content
    async fn validate_block_structure(
        &self,
        block: &Block,
        blockchain: &Blockchain,
    ) -> Result<bool> {
        // Check block index
        let expected_index = blockchain.chain.len() as u64;
        if block.index != expected_index {
            return Ok(false);
        }

        // Check previous hash
        if let Some(prev_block) = blockchain.chain.last() {
            if block.previous_hash != prev_block.hash {
                return Ok(false);
            }
        } else if block.previous_hash != "0".repeat(64) {
            return Ok(false);
        }

        // Validate block hash by recalculating it
        let expected_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));
        if block.hash != expected_hash {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate block timing constraints
    async fn validate_block_timing(&self, proposal: &BlockProposal) -> Result<bool> {
        let authority_state = self.authority_state.read().await;

        // Parse the timestamp from the block
        let block_time = chrono::DateTime::parse_from_rfc3339(&proposal.block.timestamp)
            .map_err(|_| anyhow::anyhow!("Invalid timestamp format"))?
            .with_timezone(&Utc);

        let now = Utc::now();

        // Check if the block is not too far in the future
        let time_diff = block_time.signed_duration_since(now);
        if time_diff > Duration::seconds(30) {
            return Ok(false);
        }

        // Check if enough time has passed since the last block
        let time_since_last = block_time.signed_duration_since(authority_state.last_block_time);
        if time_since_last < Duration::seconds(self.config.block_interval as i64 / 2) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Start monitoring authority performance
    async fn start_authority_monitoring(&self) {
        let consensus_manager = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                if let Err(e) = consensus_manager.update_authority_performance().await {
                    error!("Failed to update authority performance: {}", e);
                }
            }
        });
    }

    /// Update authority performance metrics
    async fn update_authority_performance(&self) -> Result<()> {
        debug!("Updating authority performance metrics");

        let mut authority_state = self.authority_state.write().await;
        let now = Utc::now();

        // Update performance for all known authorities
        let authority_keys = self.authority_keys.read().await;
        for authority_id in authority_keys.keys() {
            let performance = authority_state
                .authority_performance
                .entry(*authority_id)
                .or_insert_with(|| AuthorityPerformance {
                    blocks_created: 0,
                    missed_slots: 0,
                    last_activity: now,
                    reputation: 1.0,
                });

            // Calculate reputation based on performance
            let total_slots = performance.blocks_created + performance.missed_slots;
            if total_slots > 0 {
                performance.reputation = performance.blocks_created as f64 / total_slots as f64;
            }
        }

        Ok(())
    }

    /// Get consensus statistics
    pub async fn get_consensus_stats(&self) -> ConsensusStats {
        let authority_state = self.authority_state.read().await;
        let authority_keys = self.authority_keys.read().await;

        ConsensusStats {
            is_authority: self.config.is_authority,
            current_round: authority_state.current_round,
            total_authorities: authority_keys.len(),
            last_block_time: authority_state.last_block_time,
            block_interval: self.config.block_interval,
            authority_performance: authority_state.authority_performance.clone(),
        }
    }

    /// Add a new authority to the network
    pub async fn add_authority(&self, authority_id: Uuid, public_key: VerifyingKey) -> Result<()> {
        info!("Adding new authority: {}", authority_id);

        let mut authority_keys = self.authority_keys.write().await;
        authority_keys.insert(authority_id, public_key);

        let mut authority_state = self.authority_state.write().await;
        authority_state.authority_performance.insert(
            authority_id,
            AuthorityPerformance {
                blocks_created: 0,
                missed_slots: 0,
                last_activity: Utc::now(),
                reputation: 1.0,
            },
        );

        Ok(())
    }

    /// Remove an authority from the network
    pub async fn remove_authority(&self, authority_id: Uuid) -> Result<()> {
        info!("Removing authority: {}", authority_id);

        let mut authority_keys = self.authority_keys.write().await;
        authority_keys.remove(&authority_id);

        let mut authority_state = self.authority_state.write().await;
        authority_state.authority_performance.remove(&authority_id);

        Ok(())
    }
}

// Implement Clone for ConsensusManager
impl Clone for ConsensusManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            authority_keypair: self.authority_keypair.clone(),
            authority_keys: Arc::clone(&self.authority_keys),
            network: Arc::clone(&self.network),
            blockchain: Arc::clone(&self.blockchain),
            authority_state: Arc::clone(&self.authority_state),
        }
    }
}

/// Consensus statistics
#[derive(Debug, Clone)]
pub struct ConsensusStats {
    pub is_authority: bool,
    pub current_round: u64,
    pub total_authorities: usize,
    pub last_block_time: DateTime<Utc>,
    pub block_interval: u64,
    pub authority_performance: HashMap<Uuid, AuthorityPerformance>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::NodeConfig;

    #[tokio::test]
    async fn test_consensus_manager_creation() {
        let config = ConsensusConfig::default();
        let node_config = NodeConfig::default();
        let network = Arc::new(NetworkManager::new(node_config));
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));

        let consensus = ConsensusManager::new(config, network, blockchain).unwrap();
        let stats = consensus.get_consensus_stats().await;

        assert!(!stats.is_authority);
        assert_eq!(stats.current_round, 0);
        assert_eq!(stats.total_authorities, 0);
    }

    #[tokio::test]
    async fn test_block_creation() {
        let mut config = ConsensusConfig::default();
        config.is_authority = true;

        let node_config = NodeConfig::default();
        let network = Arc::new(NetworkManager::new(node_config));
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));

        let consensus = ConsensusManager::new(config, network, blockchain.clone()).unwrap();
        let block = consensus.create_block().await.unwrap();

        assert_eq!(block.index, 1); // Should be 1 since blockchain starts with genesis block
        assert!(!block.data.is_empty());
        assert!(!block.hash.is_empty());
    }

    #[test]
    fn test_keypair_generation() {
        let keypair = ConsensusManager::load_or_generate_keypair(&None).unwrap();

        // Test signing and verification
        let message = b"test message";
        let signature = keypair.sign(message);
        assert!(keypair.verifying_key().verify(message, &signature).is_ok());
    }
}
