//! Multi-protocol Consensus Mechanism
//! 
//! This module implements a configurable consensus layer supporting:
//! - Proof-of-Authority (PoA)
//! - PBFT (Simplified Simulation)
//! 
//! It uses a trait-based approach to allow switching protocols via configuration.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::messages::P2PMessage;
use super::{MessageHandler, NetworkManager};
use crate::core::blockchain::{Block, Blockchain};
use crate::security::keys::generate_signing_key;
use crate::utils::config::ConsensusConfig;

/// Authority state tracking (Shared by PoA and potentially others)
#[derive(Debug, Clone)]
pub struct AuthorityState {
    pub current_round: u64,
    pub current_authority: Option<Uuid>,
    pub last_block_time: DateTime<Utc>,
    pub authority_performance: HashMap<Uuid, AuthorityPerformance>,
    pub authority_rotation_order: Vec<Uuid>,
    pub current_authority_index: usize,
}

/// Authority performance metrics
#[derive(Debug, Clone)]
pub struct AuthorityPerformance {
    pub blocks_created: u64,
    pub missed_slots: u64,
    pub last_activity: DateTime<Utc>,
    pub reputation: f64,
}

/// Block creation proposal
#[derive(Debug, Clone)]
pub struct BlockProposal {
    pub block: Block,
    pub signature: Signature,
    pub authority_key: VerifyingKey,
    pub timestamp: DateTime<Utc>,
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
    pub protocol_type: String, // Added field
}

/// Consensus Protocol Trait
#[async_trait]
pub trait ConsensusProtocol: Send + Sync + MessageHandler {
    async fn start(&self) -> Result<()>;
    async fn create_block(&self) -> Result<Block>;
    async fn validate_block_proposal(&self, proposal: &BlockProposal) -> Result<bool>;
    async fn get_stats(&self) -> ConsensusStats;
    async fn add_authority(&self, authority_id: Uuid, public_key: VerifyingKey) -> Result<()>;
    async fn remove_authority(&self, authority_id: Uuid) -> Result<()>;
}

/// Main Consensus Manager holding the active protocol
#[derive(Clone)]
pub struct ConsensusManager {
    protocol: Arc<dyn ConsensusProtocol>,
}

impl ConsensusManager {
    pub async fn new(
        config: ConsensusConfig,
        network: Arc<NetworkManager>,
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<Self> {
        let protocol: Arc<dyn ConsensusProtocol> = match config.consensus_type.as_str() {
            "pbft" => Arc::new(PbftConsensus::new(config, network, blockchain).await?),
            _ => Arc::new(ProofOfAuthority::new(config, network, blockchain).await?),
        };

        Ok(Self { protocol })
    }

    pub async fn start(&self) -> Result<()> {
        self.protocol.start().await
    }

    pub async fn create_block(&self) -> Result<Block> {
        self.protocol.create_block().await
    }

    pub async fn validate_block_proposal(&self, proposal: &BlockProposal) -> Result<bool> {
        self.protocol.validate_block_proposal(proposal).await
    }

    pub async fn get_consensus_stats(&self) -> ConsensusStats {
        self.protocol.get_stats().await
    }

    pub async fn add_authority(&self, authority_id: Uuid, public_key: VerifyingKey) -> Result<()> {
        self.protocol.add_authority(authority_id, public_key).await
    }

    pub async fn remove_authority(&self, authority_id: Uuid) -> Result<()> {
        self.protocol.remove_authority(authority_id).await
    }
}

impl MessageHandler for ConsensusManager {
    fn handle_message(&self, peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>> {
        self.protocol.handle_message(peer_id, message)
    }
}

// ------------------------------------------------------------------------------------------------
// Proof of Authority Implementation
// ------------------------------------------------------------------------------------------------

pub struct ProofOfAuthority {
    pub config: ConsensusConfig,
    pub authority_keypair: Option<SigningKey>,
    pub authority_keys: Arc<RwLock<HashMap<Uuid, VerifyingKey>>>,
    pub network: Arc<NetworkManager>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub authority_state: Arc<RwLock<AuthorityState>>,
}

impl ProofOfAuthority {
    pub async fn new(
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

        for key_str in &config.authority_keys {
            if let Ok(key_bytes) = hex::decode(key_str) {
                if key_bytes.len() == 32 {
                    if let Ok(public_key) = VerifyingKey::from_bytes(&key_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid key length"))?) {
                        let authority_id = Uuid::new_v4();
                        authority_keys.write().await.insert(authority_id, public_key);
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

    fn load_or_generate_keypair(key_file: &Option<String>) -> Result<SigningKey> {
        if let Some(file_path) = key_file {
            // Try to load existing keypair
            if std::path::Path::new(file_path).exists() {
                let key_data = std::fs::read(file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read authority key file: {}", e))?;
                
                if key_data.len() == 32 {
                    let keypair = SigningKey::from_bytes(&key_data.try_into().map_err(|_| anyhow::anyhow!("Invalid key length"))?);
                    info!("Loaded authority keypair from {}", file_path);
                    return Ok(keypair);
                } else {
                    return Err(anyhow::anyhow!("Authority key file must be 32 bytes"));
                }
            }

            // Generate new keypair and save it
            let keypair = generate_signing_key()
                .map_err(|e| anyhow::anyhow!("Failed to generate authority key: {}", e))?;
            std::fs::write(file_path, keypair.to_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to write authority key file: {}", e))?;
            info!("Generated new authority keypair and saved to {}", file_path);
            Ok(keypair)
        } else {
            // Generate ephemeral keypair using CSPRNG
            generate_signing_key()
                .map_err(|e| anyhow::anyhow!("Failed to generate ephemeral key: {}", e))
        }
    }

    async fn start_authority_duties(self: Arc<Self>) -> Result<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                self.config.block_interval,
            ));
            loop {
                interval.tick().await;
                if let Err(e) = self.try_create_block().await {
                    error!("Failed to create block: {}", e);
                }
            }
        });
        Ok(())
    }

    async fn start_authority_monitoring(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.update_authority_performance().await {
                    error!("Failed to update authority performance: {}", e);
                }
            }
        });
    }

    async fn try_create_block(&self) -> Result<()> {
        if self.should_create_block().await? {
            debug!("It's our turn to create a block");
            self.create_and_propose_block_internal().await?;
        }
        Ok(())
    }

    async fn should_create_block(&self) -> Result<bool> {
        // Acquire both locks atomically to prevent race condition
        let (authority_state, authority_keys) = tokio::join!(
            self.authority_state.read(),
            self.authority_keys.read()
        );

        let now = Utc::now();
        let time_since_last = now.signed_duration_since(authority_state.last_block_time);
        if time_since_last < Duration::seconds(self.config.block_interval as i64) { return Ok(false); }
        if authority_keys.is_empty() { return Ok(false); }

        let our_authority_id = if let Some(keypair) = &self.authority_keypair {
            let public_key = keypair.verifying_key();
            authority_keys.iter().find_map(|(id, key)| if key == &public_key { Some(*id) } else { None })
        } else { None };

        if let Some(our_id) = our_authority_id {
            if let Some(current_authority) = authority_state.current_authority {
                Ok(current_authority == our_id)
            } else {
                if !authority_state.authority_rotation_order.is_empty() {
                    Ok(authority_state.authority_rotation_order[0] == our_id)
                } else {
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }

    async fn create_and_propose_block_internal(&self) -> Result<()> {
        let keypair = self.authority_keypair.as_ref().ok_or_else(|| anyhow::anyhow!("No authority keypair"))?;
        let block = self.create_block().await?;
        let block_data = self.serialize_block_for_signing(&block)?;
        let signature = keypair.sign(&block_data);
        
        let proposal = BlockProposal {
            block: block.clone(),
            signature,
            authority_key: keypair.verifying_key(),
            timestamp: Utc::now(),
        };

        {
            let mut blockchain = self.blockchain.write().await;
            let mut signed_block = block.clone();
            signed_block.signature = hex::encode(signature.to_bytes());
            blockchain.submit_signed_block(signed_block)?;
        }

        self.broadcast_block_proposal(proposal).await?;

        // Update state
        let mut authority_state = self.authority_state.write().await;
        authority_state.last_block_time = Utc::now();
        authority_state.current_round += 1;
        
        // Simple rotation
        if !authority_state.authority_rotation_order.is_empty() {
            authority_state.current_authority_index = (authority_state.current_authority_index + 1) 
                % authority_state.authority_rotation_order.len();
             if authority_state.current_authority_index < authority_state.authority_rotation_order.len() {
                authority_state.current_authority = Some(authority_state.authority_rotation_order[authority_state.current_authority_index]);
             }
        }
        
        // Stats
        if let Some(current_authority) = authority_state.current_authority {
            if let Some(performance) = authority_state.authority_performance.get_mut(&current_authority) {
                performance.blocks_created += 1;
                performance.last_activity = Utc::now();
            }
        }

        info!("PoA: Successfully created block {}", block.index);
        Ok(())
    }

    fn serialize_block_for_signing(&self, block: &Block) -> Result<Vec<u8>> {
        let data = format!("{}
{}
{}
{}", block.index, block.timestamp, block.previous_hash, block.data);
        Ok(data.into_bytes())
    }

    async fn broadcast_block_proposal(&self, proposal: BlockProposal) -> Result<()> {
        let announcement = P2PMessage::new_block_announcement(
            &proposal.block,
            format!("http://provchain.org/block/{}", proposal.block.index),
        );
        self.network.broadcast_message(announcement).await?;
        Ok(())
    }
    
    async fn update_authority_performance(&self) -> Result<()> {
        let mut authority_state = self.authority_state.write().await;
        let now = Utc::now();
        let authority_keys = self.authority_keys.read().await;
        for authority_id in authority_keys.keys() {
            let performance = authority_state.authority_performance.entry(*authority_id).or_insert_with(|| AuthorityPerformance {
                blocks_created: 0,
                missed_slots: 0,
                last_activity: now,
                reputation: 1.0,
            });
            let total_slots = performance.blocks_created + performance.missed_slots;
            if total_slots > 0 {
                performance.reputation = performance.blocks_created as f64 / total_slots as f64;
            }
        }
        Ok(())
    }

    async fn validate_block_structure(&self, block: &Block, blockchain: &Blockchain) -> Result<bool> {
        let expected_index = blockchain.chain.len() as u64;
        if block.index != expected_index { return Ok(false); }
        if let Some(prev_block) = blockchain.chain.last() {
            if block.previous_hash != prev_block.hash { return Ok(false); }
        } else if block.previous_hash != "0".repeat(64) { return Ok(false); }
        let expected_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));
        if block.hash != expected_hash { return Ok(false); }
        Ok(true)
    }

    async fn validate_block_timing(&self, proposal: &BlockProposal) -> Result<bool> {
        let authority_state = self.authority_state.read().await;
        let block_time = chrono::DateTime::parse_from_rfc3339(&proposal.block.timestamp)
            .map_err(|_| anyhow::anyhow!("Invalid timestamp"))?
            .with_timezone(&Utc);
        let now = Utc::now();
        let time_diff = block_time.signed_duration_since(now);
        if time_diff > Duration::seconds(30) { return Ok(false); }
        
        let time_since_last = block_time.signed_duration_since(authority_state.last_block_time);
        if time_since_last < Duration::seconds(self.config.block_interval as i64 / 2) { return Ok(false); }
        Ok(true)
    }
}

#[async_trait]
impl ConsensusProtocol for ProofOfAuthority {
    async fn start(&self) -> Result<()> {
        info!("Starting Proof-of-Authority consensus");
        // We can't use self directly in spawn because we are behind an Arc trait object in the manager
        // But here we are just implementing the trait.
        // The issue is: how to get a 'static reference or Arc<Self> to spawn?
        // We will assume start() is called when we have an Arc.
        // HACK: We can't clone 'self' here easily if it's just &self.
        // We might need to restructure start() to not rely on 'self' in the background task if possible,
        // OR rely on the caller to handle the spawning.
        
        // Actually, since we are moving to a trait object, 'self' in start is just a reference.
        // The common pattern is to have an inner struct wrapped in Arc that is cheap to clone.
        // Let's modify ProofOfAuthority to be a cheap clone wrapper around an Inner struct?
        // Or just Clone the Arc fields. 
        
        // Since ProofOfAuthority has Arc fields, cloning it is cheap!
        // But we can't clone &self to Self if we don't know the concrete type? 
        // We do know it here. 
        
        // Wait, async_trait creates a BoxFuture.
        // We can create a clone of our fields and move them into the task.
        
        if self.config.is_authority {
             // We need to run background tasks.
             // We can construct a new instance of Self (since it's just Arcs) and move it.
             let clone = Self {
                 config: self.config.clone(),
                 authority_keypair: self.authority_keypair.clone(), // This one is not Arc... SigningKey is small copy? No. 
                 // SigningKey is Copy? No. It's Clone.
                 authority_keys: self.authority_keys.clone(),
                 network: self.network.clone(),
                 blockchain: self.blockchain.clone(),
                 authority_state: self.authority_state.clone(),
             };
             let arc_clone = Arc::new(clone);
             arc_clone.start_authority_duties().await?;
             // arc_clone.start_authority_monitoring().await; // Reuse same clone? no, move occurs.
        }
        
        // Monitoring
        let clone_mon = Self {
                 config: self.config.clone(),
                 authority_keypair: self.authority_keypair.clone(),
                 authority_keys: self.authority_keys.clone(),
                 network: self.network.clone(),
                 blockchain: self.blockchain.clone(),
                 authority_state: self.authority_state.clone(),
        };
        let arc_clone_mon = Arc::new(clone_mon);
        arc_clone_mon.start_authority_monitoring().await;

        Ok(())
    }

    async fn create_block(&self) -> Result<Block> {
        let blockchain = self.blockchain.read().await;
        let previous_block = blockchain.chain.last();
        let (previous_hash, index) = if let Some(prev) = previous_block {
            (prev.hash.clone(), prev.index + 1)
        } else {
            ("0".repeat(64), 0)
        };
        let state_root = self.blockchain.read().await.rdf_store.calculate_state_root();
        let rdf_data = format!(
            "<http://provchain.org/block/{}> <http://provchain.org/timestamp> \"{}\" .\n<http://provchain.org/block/{}> <http://provchain.org/authority> \"{}\" .",
            index, Utc::now().to_rfc3339(), index, self.network.node_id
        );
        Ok(Block::new(index, rdf_data, previous_hash, state_root, self.network.node_id.to_string()))
    }

    async fn validate_block_proposal(&self, proposal: &BlockProposal) -> Result<bool> {
        debug!("PoA: Validating block proposal {}", proposal.block.index);
        let authority_keys = self.authority_keys.read().await;
        let is_known = authority_keys.values().any(|key| *key == proposal.authority_key);
        if !is_known { warn!("Unknown authority"); return Ok(false); }
        
        let block_data = self.serialize_block_for_signing(&proposal.block)?;
        if proposal.authority_key.verify(&block_data, &proposal.signature).is_err() {
            warn!("Invalid signature"); return Ok(false);
        }
        
        let blockchain = self.blockchain.read().await;
        if !self.validate_block_structure(&proposal.block, &blockchain).await? {
            warn!("Invalid structure"); return Ok(false);
        }
        
        if !self.validate_block_timing(proposal).await? {
            warn!("Invalid timing"); return Ok(false);
        }
        Ok(true)
    }

    async fn get_stats(&self) -> ConsensusStats {
        let authority_state = self.authority_state.read().await;
        let authority_keys = self.authority_keys.read().await;
        ConsensusStats {
            is_authority: self.config.is_authority,
            current_round: authority_state.current_round,
            total_authorities: authority_keys.len(),
            last_block_time: authority_state.last_block_time,
            block_interval: self.config.block_interval,
            authority_performance: authority_state.authority_performance.clone(),
            protocol_type: "PoA".to_string(),
        }
    }

    async fn add_authority(&self, authority_id: Uuid, public_key: VerifyingKey) -> Result<()> {
        self.authority_keys.write().await.insert(authority_id, public_key);
        Ok(())
    }

    async fn remove_authority(&self, authority_id: Uuid) -> Result<()> {
        self.authority_keys.write().await.remove(&authority_id);
        Ok(())
    }
}

impl MessageHandler for ProofOfAuthority {
    fn handle_message(&self, _peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>> {
        match message {
            P2PMessage::BlockAnnouncement { block_index, .. } => {
                info!("PoA: Received block announcement for {}", block_index);
                // In real impl, trigger fetch
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

// ------------------------------------------------------------------------------------------------
// PBFT Implementation
// ------------------------------------------------------------------------------------------------

/// PBFT phases
#[derive(Debug, Clone, PartialEq)]
pub enum PbftPhase {
    Idle,
    PrePrepare,
    Prepare,
    Commit,
    Executed,
}

/// PBFT view state
#[derive(Debug, Clone)]
pub struct PbftViewState {
    pub view: u64,
    pub primary: Option<Uuid>,
    pub sequence: u64,
}

/// PBFT message types with Ed25519 signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PbftMessage {
    PrePrepare {
        view: u64,
        sequence: u64,
        block_hash: String,
        block: Block,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    },
    Prepare {
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    },
    Commit {
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    },
    ViewChange {
        new_view: u64,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    },
}

impl PbftMessage {
    /// Get the message signature
    pub fn get_signature(&self) -> Option<&Signature> {
        match self {
            PbftMessage::PrePrepare { signature, .. } => Some(signature),
            PbftMessage::Prepare { signature, .. } => Some(signature),
            PbftMessage::Commit { signature, .. } => Some(signature),
            PbftMessage::ViewChange { signature, .. } => Some(signature),
        }
    }

    /// Get the sender's public key
    pub fn get_public_key(&self) -> &VerifyingKey {
        match self {
            PbftMessage::PrePrepare { public_key, .. } => public_key,
            PbftMessage::Prepare { public_key, .. } => public_key,
            PbftMessage::Commit { public_key, .. } => public_key,
            PbftMessage::ViewChange { public_key, .. } => public_key,
        }
    }

    /// Get the message sender
    pub fn get_sender(&self) -> Uuid {
        match self {
            PbftMessage::PrePrepare { sender, .. } => *sender,
            PbftMessage::Prepare { sender, .. } => *sender,
            PbftMessage::Commit { sender, .. } => *sender,
            PbftMessage::ViewChange { sender, .. } => *sender,
        }
    }

    /// Get bytes to sign for each message type
    fn get_signing_bytes(&self) -> Vec<u8> {
        match self {
            PbftMessage::PrePrepare { view, sequence, block_hash, sender, .. } => {
                format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
                    .into_bytes()
            }
            PbftMessage::Prepare { view, sequence, block_hash, sender, .. } => {
                format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
                    .into_bytes()
            }
            PbftMessage::Commit { view, sequence, block_hash, sender, .. } => {
                format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
                    .into_bytes()
            }
            PbftMessage::ViewChange { new_view, sender, .. } => {
                format!("{}-{}", new_view, sender).into_bytes()
            }
        }
    }

    /// Verify the message signature
    pub fn verify_signature(&self) -> bool {
        let public_key = self.get_public_key();
        let Some(signature) = self.get_signature() else {
            return false;
        };
        let message_bytes = self.get_signing_bytes();
        public_key.verify(&message_bytes, signature).is_ok()
    }

    /// Create a signed PrePrepare message
    pub fn create_pre_prepare(
        view: u64,
        sequence: u64,
        block_hash: String,
        block: Block,
        sender: Uuid,
        keypair: &SigningKey,
    ) -> Self {
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
            .into_bytes();
        let signature = keypair.sign(&message_bytes);
        let public_key = keypair.verifying_key();

        PbftMessage::PrePrepare {
            view,
            sequence,
            block_hash,
            block,
            sender,
            signature,
            public_key,
        }
    }

    /// Create a signed Prepare message
    pub fn create_prepare(
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        keypair: &SigningKey,
    ) -> Self {
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
            .into_bytes();
        let signature = keypair.sign(&message_bytes);
        let public_key = keypair.verifying_key();

        PbftMessage::Prepare {
            view,
            sequence,
            block_hash,
            sender,
            signature,
            public_key,
        }
    }

    /// Create a signed Commit message
    pub fn create_commit(
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        keypair: &SigningKey,
    ) -> Self {
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender)
            .into_bytes();
        let signature = keypair.sign(&message_bytes);
        let public_key = keypair.verifying_key();

        PbftMessage::Commit {
            view,
            sequence,
            block_hash,
            sender,
            signature,
            public_key,
        }
    }

    /// Create a signed ViewChange message
    pub fn create_view_change(
        new_view: u64,
        sender: Uuid,
        keypair: &SigningKey,
    ) -> Self {
        let message_bytes = format!("{}-{}", new_view, sender).into_bytes();
        let signature = keypair.sign(&message_bytes);
        let public_key = keypair.verifying_key();

        PbftMessage::ViewChange {
            new_view,
            sender,
            signature,
            public_key,
        }
    }
}

/// PBFT consensus state
#[derive(Debug, Clone)]
pub struct PbftState {
    pub view: u64,
    pub sequence: u64,
    pub phase: PbftPhase,
    pub current_block: Option<Block>,
    pub current_block_hash: Option<String>,
    pub prepare_certificates: HashMap<(u64, u64, String), HashMap<Uuid, PbftMessage>>, // (view, sequence, block_hash) -> senders
    pub commit_certificates: HashMap<(u64, u64, String), HashMap<Uuid, PbftMessage>>,   // (view, sequence, block_hash) -> senders
    pub logged_prepared: HashSet<(u64, u64, String)>,  // (view, sequence, block_hash)
    pub logged_committed: HashSet<(u64, u64, String)>, // (view, sequence, block_hash)
}

impl Default for PbftState {
    fn default() -> Self {
        Self {
            view: 0,
            sequence: 0,
            phase: PbftPhase::Idle,
            current_block: None,
            current_block_hash: None,
            prepare_certificates: HashMap::new(),
            commit_certificates: HashMap::new(),
            logged_prepared: HashSet::new(),
            logged_committed: HashSet::new(),
        }
    }
}

pub struct PbftConsensus {
    pub config: ConsensusConfig,
    pub network: Arc<NetworkManager>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub node_id: Uuid,
    pub authority_keys: Arc<RwLock<HashMap<Uuid, VerifyingKey>>>,
    pub pbft_state: Arc<RwLock<PbftState>>,
    pub authority_keypair: Option<SigningKey>,
    pub is_primary: bool,
}

impl PbftConsensus {
    pub async fn new(
        config: ConsensusConfig,
        network: Arc<NetworkManager>,
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> Result<Self> {
        let node_id = network.node_id;
        let authority_keypair = if config.is_authority {
            Some(generate_signing_key()
                .map_err(|e| anyhow::anyhow!("Failed to generate authority key: {}", e))?)
        } else {
            None
        };

        let mut authority_keys = HashMap::new();
        for key_str in &config.authority_keys {
            if let Ok(key_bytes) = hex::decode(key_str) {
                if key_bytes.len() == 32 {
                    if let Ok(public_key) = VerifyingKey::from_bytes(&key_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid key length"))?) {
                        let authority_id = Uuid::new_v4();
                        authority_keys.insert(authority_id, public_key);
                    }
                }
            }
        }

        // Determine if we're primary (simple deterministic selection based on node ID)
        let is_primary = authority_keys.is_empty() ||
            authority_keys.keys().min().map(|min_id| *min_id == node_id).unwrap_or(false);

        Ok(Self {
            config,
            network,
            blockchain,
            node_id,
            authority_keys: Arc::new(RwLock::new(authority_keys)),
            pbft_state: Arc::new(RwLock::new(PbftState::default())),
            authority_keypair,
            is_primary,
        })
    }

    /// Calculate the required quorum for PBFT (2f + 1 where f = (n-1)/3)
    fn required_quorum(&self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        // For n nodes, we need at least 2f+1 consensus where f = floor((n-1)/3)
        let f = (n.saturating_sub(1)) / 3;
        (2 * f) + 1
    }

    /// Check if we are the primary for the current view
    async fn is_primary_for_view(&self, view: u64) -> bool {
        let authority_keys = self.authority_keys.read().await;
        if authority_keys.is_empty() {
            return true; // Single node mode
        }

        // Deterministic primary selection: primary is view % n-th authority
        let mut sorted_ids: Vec<Uuid> = authority_keys.keys().copied().collect();
        sorted_ids.sort();
        let primary_index = (view as usize) % sorted_ids.len();
        sorted_ids.get(primary_index).map(|id| *id == self.node_id).unwrap_or(false)
    }

    /// Send a PREPARE message to all nodes
    async fn send_prepare(&self, view: u64, sequence: u64, block_hash: String) -> Result<()> {
        let keypair = self.authority_keypair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No signing key available"))?;
        
        let prepare_msg = PbftMessage::create_prepare(
            view,
            sequence,
            block_hash.clone(),
            self.node_id,
            keypair,
        );

        // Broadcast to network
        self.broadcast_pbft_message(prepare_msg.clone()).await?;

        // Log that we sent prepare
        let mut state = self.pbft_state.write().await;
        let key = (view, sequence, block_hash);
        state.logged_prepared.insert(key);

        debug!("PBFT: Sent PREPARE for view={}, sequence={}", view, sequence);
        Ok(())
    }

    /// Send a COMMIT message to all nodes
    async fn send_commit(&self, view: u64, sequence: u64, block_hash: String) -> Result<()> {
        let keypair = self.authority_keypair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No signing key available"))?;
        
        let commit_msg = PbftMessage::create_commit(
            view,
            sequence,
            block_hash.clone(),
            self.node_id,
            keypair,
        );

        // Broadcast to network
        self.broadcast_pbft_message(commit_msg.clone()).await?;

        // Log that we sent commit
        let mut state = self.pbft_state.write().await;
        let key = (view, sequence, block_hash);
        state.logged_committed.insert(key);

        debug!("PBFT: Sent COMMIT for view={}, sequence={}", view, sequence);
        Ok(())
    }

    /// Broadcast a PBFT message to all nodes
    async fn broadcast_pbft_message(&self, message: PbftMessage) -> Result<()> {
        // In a real implementation, this would send the message via P2P network
        // For now, we just log it
        debug!("PBFT: Broadcasting message: {:?}", message);
        Ok(())
    }

    /// Validate a block's structure
    async fn validate_block_structure(&self, block: &Block) -> Result<bool> {
        let blockchain = self.blockchain.read().await;
        let expected_index = blockchain.chain.len() as u64;
        if block.index != expected_index {
            return Ok(false);
        }
        if let Some(prev_block) = blockchain.chain.last() {
            if block.previous_hash != prev_block.hash {
                return Ok(false);
            }
        } else if block.previous_hash != "0".repeat(64) {
            return Ok(false);
        }

        let expected_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));
        if block.hash != expected_hash {
            return Ok(false);
        }
        Ok(true)
    }
}

#[async_trait]
impl ConsensusProtocol for PbftConsensus {
    async fn start(&self) -> Result<()> {
        info!("Starting PBFT Consensus");
        let state = self.pbft_state.read().await;
        let authority_keys = self.authority_keys.read().await;
        info!("PBFT: Node {}, View={}, Primary={}, Authorities={}",
            self.node_id, state.view, self.is_primary, authority_keys.len());

        // Start PBFT consensus loop if we're an authority
        if self.config.is_authority {
            let clone = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                    clone.config.block_interval,
                ));
                loop {
                    interval.tick().await;
                    if let Err(e) = clone.try_pbft_round().await {
                        error!("PBFT round error: {}", e);
                    }
                }
            });
        }
        Ok(())
    }

    async fn create_block(&self) -> Result<Block> {
        let blockchain = self.blockchain.read().await;
        let previous_block = blockchain.chain.last();
        let (previous_hash, index) = if let Some(prev) = previous_block {
            (prev.hash.clone(), prev.index + 1)
        } else {
            ("0".repeat(64), 0)
        };
        let state_root = self.blockchain.read().await.rdf_store.calculate_state_root();

        let rdf_data = format!(
            "<http://provchain.org/block/{}> <http://provchain.org/consensus> \"PBFT\" .",
            index
        );

        Ok(Block::new(index, rdf_data, previous_hash, state_root, self.network.node_id.to_string()))
    }

    async fn validate_block_proposal(&self, proposal: &BlockProposal) -> Result<bool> {
        let state = self.pbft_state.read().await;
        let authority_keys = self.authority_keys.read().await;

        // 1. Check if we're in the correct view
        if proposal.block.index < state.sequence {
            debug!("PBFT: Old proposal, index {} < current sequence {}",
                proposal.block.index, state.sequence);
            return Ok(false);
        }

        // 2. Validate block structure
        if !self.validate_block_structure(&proposal.block).await? {
            debug!("PBFT: Invalid block structure");
            return Ok(false);
        }

        // 3. Verify the authority that signed the block is known
        let is_known_authority = authority_keys.values().any(|key| *key == proposal.authority_key);
        if !is_known_authority {
            debug!("PBFT: Unknown authority in proposal");
            return Ok(false);
        }

        // 4. Verify signature
        let block_data = format!("{}{}{}{}",
            proposal.block.index,
            proposal.block.timestamp,
            proposal.block.previous_hash,
            proposal.block.data
        );
        if proposal.authority_key.verify(block_data.as_bytes(), &proposal.signature).is_err() {
            debug!("PBFT: Invalid signature");
            return Ok(false);
        }

        // 5. Check sequence number
        if proposal.block.index <= state.sequence {
            debug!("PBFT: Sequence number too low");
            return Ok(false);
        }

        debug!("PBFT: Block proposal {} validated successfully", proposal.block.index);
        Ok(true)
    }

    async fn get_stats(&self) -> ConsensusStats {
        let state = self.pbft_state.read().await;
        let authority_keys = self.authority_keys.read().await;
        ConsensusStats {
            is_authority: self.config.is_authority,
            current_round: state.view,
            total_authorities: authority_keys.len(),
            last_block_time: Utc::now(),
            block_interval: self.config.block_interval,
            authority_performance: HashMap::new(),
            protocol_type: "PBFT".to_string(),
        }
    }

    async fn add_authority(&self, id: Uuid, key: VerifyingKey) -> Result<()> {
        self.authority_keys.write().await.insert(id, key);
        info!("PBFT: Added authority {}", id);
        Ok(())
    }

    async fn remove_authority(&self, id: Uuid) -> Result<()> {
        self.authority_keys.write().await.remove(&id);
        info!("PBFT: Removed authority {}", id);
        Ok(())
    }
}

impl PbftConsensus {
    /// Try to execute a PBFT consensus round
    async fn try_pbft_round(&self) -> Result<()> {
        let state = self.pbft_state.read().await;

        // Only primary initiates the pre-prepare phase
        if !self.is_primary_for_view(state.view).await {
            return Ok(());
        }

        // Check if we should create a new block
        if state.phase != PbftPhase::Idle {
            return Ok(());
        }

        drop(state);

        // Phase 1: Pre-Prepare (only primary)
        if let Err(e) = self.send_pre_prepare().await {
            error!("Failed to send pre-prepare: {}", e);
        }

        Ok(())
    }

    /// Send PRE-PREPARE message (primary only)
    async fn send_pre_prepare(&self) -> Result<()> {
        let block = self.create_block().await?;
        let block_hash = block.hash.clone();

        let state = self.pbft_state.read().await;
        let view = state.view;
        let sequence = state.sequence;
        drop(state);

        let keypair = self.authority_keypair.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No signing key available"))?;

        let pre_prepare_msg = PbftMessage::create_pre_prepare(
            view,
            sequence,
            block_hash.clone(),
            block.clone(),
            self.node_id,
            keypair,
        );

        // Update state
        {
            let mut state = self.pbft_state.write().await;
            state.phase = PbftPhase::PrePrepare;
            state.current_block = Some(block.clone());
            state.current_block_hash = Some(block_hash.clone());
        }

        // Broadcast PRE-PREPARE
        self.broadcast_pbft_message(pre_prepare_msg).await?;

        // Auto-advance to PREPARE phase for primary
        self.send_prepare(view, sequence, block_hash).await?;

        info!("PBFT: Primary sent PRE-PREPARE for sequence {}", sequence);
        Ok(())
    }

    /// Handle incoming PBFT message
    pub async fn handle_pbft_message(&self, message: PbftMessage) -> Result<()> {
        match message {
            PbftMessage::PrePrepare { view, sequence, block_hash, block, sender, signature, public_key } => {
                self.handle_pre_prepare(view, sequence, block_hash, block, sender, signature, public_key).await?;
            }
            PbftMessage::Prepare { view, sequence, block_hash, sender, signature, public_key } => {
                self.handle_prepare(view, sequence, block_hash, sender, signature, public_key).await?;
            }
            PbftMessage::Commit { view, sequence, block_hash, sender, signature, public_key } => {
                self.handle_commit(view, sequence, block_hash, sender, signature, public_key).await?;
            }
            PbftMessage::ViewChange { new_view, sender, .. } => {
                self.handle_view_change(new_view, sender).await?;
            }
        }
        Ok(())
    }

    /// Handle PRE-PREPARE message
    async fn handle_pre_prepare(
        &self,
        view: u64,
        sequence: u64,
        block_hash: String,
        block: Block,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    ) -> Result<()> {
        let state = self.pbft_state.read().await;

        // Only accept PRE-PREPARE from primary
        if !self.is_primary_for_view(view).await || sender != self.node_id {
            return Ok(());
        }

        // Verify signature
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender).into_bytes();
        if public_key.verify(&message_bytes, &signature).is_err() {
            warn!("PBFT: Invalid signature on PRE-PREPARE message from {}", sender);
            return Ok(());
        }

        // Check view and sequence
        if view < state.view || sequence <= state.sequence {
            return Ok(());
        }

        // Validate the block
        if !self.validate_block_structure(&block).await? {
            warn!("PBFT: Invalid block in PRE-PREPARE");
            return Ok(());
        }

        drop(state);

        // Update state
        {
            let mut state = self.pbft_state.write().await;
            state.phase = PbftPhase::PrePrepare;
            state.current_block = Some(block.clone());
            state.current_block_hash = Some(block_hash.clone());
        }

        // Send PREPARE
        self.send_prepare(view, sequence, block_hash).await?;

        debug!("PBFT: Handled PRE-PREPARE for sequence {}", sequence);
        Ok(())
    }

    /// Handle PREPARE message
    async fn handle_prepare(
        &self,
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    ) -> Result<()> {
        let mut state = self.pbft_state.write().await;

        // Only accept PREPARE for current view/sequence
        if view != state.view || sequence != state.sequence {
            return Ok(());
        }

        // Verify signature
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender).into_bytes();
        if public_key.verify(&message_bytes, &signature).is_err() {
            warn!("PBFT: Invalid signature on PREPARE message from {}", sender);
            return Ok(());
        }

        // Store the PREPARE certificate
        let key = (view, sequence, block_hash.clone());
        state.prepare_certificates
            .entry(key.clone())
            .or_insert_with(HashMap::new)
            .insert(sender, PbftMessage::Prepare {
                view,
                sequence,
                block_hash: block_hash.clone(),
                sender,
                signature,
                public_key,
            });

        // Check if we have enough PREPARE messages (2f+1)
        let authority_count = self.authority_keys.read().await.len();
        let required = self.required_quorum(authority_count);
        let prepare_count = state.prepare_certificates.get(&key).map(|m| m.len()).unwrap_or(0);

        if prepare_count >= required && !state.logged_prepared.contains(&key) {
            state.logged_prepared.insert(key.clone());
            state.phase = PbftPhase::Commit;

            // Advance to COMMIT phase
            drop(state);
            self.send_commit(view, sequence, block_hash).await?;

            info!("PBFT: Reached quorum for PREPARE, moving to COMMIT phase");
        }

        Ok(())
    }

    /// Handle COMMIT message
    async fn handle_commit(
        &self,
        view: u64,
        sequence: u64,
        block_hash: String,
        sender: Uuid,
        signature: Signature,
        public_key: VerifyingKey,
    ) -> Result<()> {
        let mut state = self.pbft_state.write().await;

        // Only accept COMMIT for current view/sequence
        if view != state.view || sequence != state.sequence {
            return Ok(());
        }

        // Verify signature
        let message_bytes = format!("{}-{}-{}-{}", view, sequence, block_hash, sender).into_bytes();
        if public_key.verify(&message_bytes, &signature).is_err() {
            warn!("PBFT: Invalid signature on COMMIT message from {}", sender);
            return Ok(());
        }

        // Store the COMMIT certificate
        let key = (view, sequence, block_hash.clone());
        state.commit_certificates
            .entry(key.clone())
            .or_insert_with(HashMap::new)
            .insert(sender, PbftMessage::Commit {
                view,
                sequence,
                block_hash: block_hash.clone(),
                sender,
                signature,
                public_key,
            });

        // Check if we have enough COMMIT messages (2f+1)
        let authority_count = self.authority_keys.read().await.len();
        let required = self.required_quorum(authority_count);
        let commit_count = state.commit_certificates.get(&key).map(|m| m.len()).unwrap_or(0);

        if commit_count >= required && !state.logged_committed.contains(&key) {
            state.logged_committed.insert(key.clone());
            state.phase = PbftPhase::Executed;

            // Execute the block
            if let Some(block) = state.current_block.take() {
                drop(state);
                self.execute_block(block).await?;
            } else {
                drop(state);
            }

            info!("PBFT: Reached quorum for COMMIT, executed block");
        }

        Ok(())
    }

    /// Handle VIEW CHANGE message
    async fn handle_view_change(&self, new_view: u64, sender: Uuid) -> Result<()> {
        let mut state = self.pbft_state.write().await;

        if new_view > state.view {
            info!("PBFT: View change from {} to {} (triggered by {})", state.view, new_view, sender);
            state.view = new_view;
            state.phase = PbftPhase::Idle;
            state.current_block = None;
            state.current_block_hash = None;
        }

        Ok(())
    }

    /// Execute a committed block
    async fn execute_block(&self, block: Block) -> Result<()> {
        // Add block to blockchain
        let mut blockchain = self.blockchain.write().await;
        blockchain.submit_signed_block(block.clone())?;

        // Update sequence number
        let mut state = self.pbft_state.write().await;
        state.sequence += 1;
        state.phase = PbftPhase::Idle;
        state.current_block = None;
        state.current_block_hash = None;

        info!("PBFT: Executed block {} at sequence {}", block.index, state.sequence);
        Ok(())
    }
}

// Need to implement Clone for PbftConsensus for tokio::spawn
impl Clone for PbftConsensus {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            network: self.network.clone(),
            blockchain: self.blockchain.clone(),
            node_id: self.node_id,
            authority_keys: self.authority_keys.clone(),
            pbft_state: self.pbft_state.clone(),
            authority_keypair: self.authority_keypair.clone(),
            is_primary: self.is_primary,
        }
    }
}

impl MessageHandler for PbftConsensus {
    fn handle_message(&self, _peer_id: Uuid, message: P2PMessage) -> Result<Option<P2PMessage>> {
        // In a real implementation, this would deserialize PBFT messages from P2PMessage
        // For now, we handle the basic message types
        match message {
            P2PMessage::BlockAnnouncement { block_index, .. } => {
                debug!("PBFT: Received block announcement for {}", block_index);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::NodeConfig;

    #[tokio::test]
    async fn test_consensus_manager_creation() {
        let config = ConsensusConfig::default(); // defaults to poa
        let node_config = NodeConfig::default();
        let network = Arc::new(NetworkManager::new(node_config));
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));

        let consensus = ConsensusManager::new(config, network, blockchain)
            .await
            .unwrap();
        let stats = consensus.get_consensus_stats().await;

        assert_eq!(stats.protocol_type, "PoA");
    }
    
    #[tokio::test]
    async fn test_pbft_switching() {
        let mut config = ConsensusConfig::default();
        config.consensus_type = "pbft".to_string();
        
        let node_config = NodeConfig::default();
        let network = Arc::new(NetworkManager::new(node_config));
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));

        let consensus = ConsensusManager::new(config, network, blockchain)
            .await
            .unwrap();
        let stats = consensus.get_consensus_stats().await;

        assert_eq!(stats.protocol_type, "PBFT");
    }
}