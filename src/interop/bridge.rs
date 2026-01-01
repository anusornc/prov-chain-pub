//! Cross-chain Bridge for Data Interchange
//!
//! This module implements the foundation for transferring data and assets between
//! independent ProvChain networks or compatible ledgers.
//!
//! It follows a "Lock-and-Mint" or "Burn-and-Release" pattern:
//! 1. Data is "locked" on the source chain (recorded in a block with a specific flag).
//! 2. A cryptographic proof (SPV or similar) is generated.
//! 3. The proof is submitted to the destination chain.
//! 4. The destination chain verifies the proof and "mints" or replicates the data.

use crate::core::blockchain::Blockchain;
use crate::ontology::ShaclValidator;
use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey, SigningKey};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents a message/data payload to be transferred across chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    /// ID of the source network
    pub source_network_id: String,
    /// ID of the destination network
    pub target_network_id: String,
    /// Unique ID of this transfer
    pub transfer_id: Uuid,
    /// The RDF data or asset identifier being transferred
    pub payload: String,
    /// Timestamp of initiation
    pub timestamp: String,
}

/// A proof that a specific event happened on the source chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainProof {
    /// The message that was emitted
    pub message: CrossChainMessage,
    /// The block header containing the message
    pub block_index: u64,
    /// Merkle proof or State Root proving inclusion (simplified here as just the root)
    pub state_root: String,
    /// Signature of the source chain authorities attesting to this block
    pub authority_signatures: Vec<(String, Vec<u8>)>, // (AuthorityID, SignatureBytes)
}

/// Manages cross-chain operations
pub struct BridgeManager {
    /// Reference to the local blockchain
    blockchain: Arc<RwLock<Blockchain>>,
    /// Trusted authorities of foreign chains (NetworkID -> List of PublicKeys)
    trusted_foreign_authorities: Arc<RwLock<std::collections::HashMap<String, Vec<VerifyingKey>>>>,
    /// Optional SHACL validator for incoming data
    pub shacl_validator: Option<ShaclValidator>,
}

impl BridgeManager {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self {
            blockchain,
            trusted_foreign_authorities: Arc::new(RwLock::new(std::collections::HashMap::new())),
            shacl_validator: None,
        }
    }

    /// Set the SHACL validator for this bridge
    pub fn set_validator(&mut self, validator: ShaclValidator) {
        self.shacl_validator = Some(validator);
    }

    /// Register a trusted authority for a foreign network
    pub async fn add_trusted_authority(&self, network_id: &str, public_key_bytes: &[u8]) -> Result<()> {
        let key = VerifyingKey::from_bytes(public_key_bytes.try_into().map_err(|_| anyhow!("Invalid key length"))?)
            .map_err(|_| anyhow!("Invalid key"))?;
        
        let mut authorities = self.trusted_foreign_authorities.write().await;
        authorities.entry(network_id.to_string()).or_default().push(key);
        Ok(())
    }

    /// Generate a proof for a local transaction to be sent elsewhere
    pub async fn export_proof(&self, block_index: u64, transfer_id: Uuid, signing_key: &SigningKey) -> Result<CrossChainProof> {
        let blockchain = self.blockchain.read().await;
        
        // Find the block
        let block = blockchain.chain.iter().find(|b| b.index == block_index)
            .ok_or_else(|| anyhow!("Block {} not found", block_index))?;

        // Reconstruct what we sign: index|timestamp|state_root|payload
        let signed_data = format!(
            "{}|{}|{}|{}",
            block.index,
            block.timestamp,
            block.state_root,
            block.data
        );
        let signature = signing_key.sign(signed_data.as_bytes());
        
        // Construct the message object
        let message = CrossChainMessage {
            source_network_id: "local-net".to_string(), // Should come from config
            target_network_id: "foreign-net".to_string(),
            transfer_id,
            payload: block.data.clone(),
            timestamp: block.timestamp.clone(),
        };

        Ok(CrossChainProof {
            message,
            block_index: block.index,
            state_root: block.state_root.clone(),
            authority_signatures: vec![("primary-authority".to_string(), signature.to_bytes().to_vec())],
        })
    }

    /// Verify and ingest a proof from a foreign chain
    pub async fn import_proof(&self, proof: &CrossChainProof) -> Result<bool> {
        // 1. Check if we trust the source network
        let authorities = self.trusted_foreign_authorities.read().await;
        let trusted_keys = authorities.get(&proof.message.source_network_id)
            .ok_or_else(|| anyhow!("Unknown source network: {}", proof.message.source_network_id))?;

        if trusted_keys.is_empty() {
            return Err(anyhow!("No trusted authorities for network {}", proof.message.source_network_id));
        }

        // 2. Verify signatures
        // Reconstruct what was signed. In our implementation, we sign:
        // index|timestamp|state_root|payload
        let signed_data = format!(
            "{}|{}|{}|{}",
            proof.block_index,
            proof.message.timestamp,
            proof.state_root,
            proof.message.payload
        );
        let signed_bytes = signed_data.as_bytes();

        let mut valid_signature_found = false;

        for (_auth_id, sig_bytes) in &proof.authority_signatures {
            let signature = Signature::from_bytes(sig_bytes.as_slice().try_into().map_err(|_| anyhow!("Invalid signature length"))?);
            
            // Check if any of our trusted keys for this network match this signature
            for trusted_key in trusted_keys {
                if trusted_key.verify(signed_bytes, &signature).is_ok() {
                    valid_signature_found = true;
                    break;
                }
            }
            
            if valid_signature_found {
                break;
            }
        }

        if valid_signature_found {
            // 3. SHACL Validation (if configured)
            if let Some(validator) = &self.shacl_validator {
                tracing::info!("Validating cross-chain payload with SHACL...");
                // Note: We assume payload is RDF Turtle data here. 
                // If it's not (e.g. JSON), validation might fail or need parsing adjustment.
                match validator.validate_transaction(&proof.message.payload) {
                    Ok(report) => {
                        if !report.is_valid {
                            tracing::warn!("❌ SHACL Validation Failed for cross-chain transfer: {:?}", report.violations);
                            return Ok(false); // Reject invalid data
                        }
                        tracing::info!("✅ SHACL Validation Passed");
                    },
                    Err(e) => {
                        tracing::error!("SHACL Validation Error: {}", e);
                        // Decide policy: fail closed?
                        return Err(anyhow!("Validation error: {}", e));
                    }
                }
            }

            // 4. Process the payload (Mint/Unlock)
            // In a complete implementation, this would trigger an atomic operation
            // to add the cross-chain data to the local state.
            
            tracing::info!(
                "✅ Successfully verified cross-chain transfer {} from {}", 
                proof.message.transfer_id, 
                proof.message.source_network_id
            );
            Ok(true)
        } else {
            tracing::warn!(
                "❌ Failed to verify cross-chain transfer {} from {}: No valid signatures", 
                proof.message.transfer_id, 
                proof.message.source_network_id
            );
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;
    
    #[tokio::test]
    async fn test_bridge_structure() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let bridge = BridgeManager::new(blockchain);
        
        // Setup a dummy foreign key
        let _key_bytes = [0u8; 32]; // Invalid key but sufficient for structure test if we don't parse it deeply
        // actually ed25519 needs valid point.
        // We'll skip adding authority and just check instantiation
        
        assert!(bridge.trusted_foreign_authorities.read().await.is_empty());
    }
}
