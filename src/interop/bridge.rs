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
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const DEFAULT_DEMO_NETWORK_ID: &str = "local-net";

fn cross_chain_signature_payload(
    message: &CrossChainMessage,
    block_index: u64,
    state_root: &str,
) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        block_index,
        message.timestamp,
        state_root,
        message.source_network_id,
        message.target_network_id,
        message.transfer_id,
        message.payload
    )
}

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
    /// Network ID represented by this bridge instance
    local_network_id: String,
    /// Default destination network ID for exported proofs
    default_target_network_id: String,
    /// Trusted authorities of foreign chains (NetworkID -> List of PublicKeys)
    trusted_foreign_authorities: Arc<RwLock<HashMap<String, Vec<VerifyingKey>>>>,
    /// Transfer IDs already admitted to this destination chain
    accepted_transfers: Arc<RwLock<HashSet<Uuid>>>,
    /// Optional SHACL validator for incoming data
    pub shacl_validator: Option<ShaclValidator>,
}

impl BridgeManager {
    /// Create a demo-compatible bridge with local defaults.
    ///
    /// Production callers should use [`BridgeManager::with_network_ids`] so
    /// source and target network IDs come from the deployed network profile
    /// rather than test/demo defaults.
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self {
            blockchain,
            local_network_id: DEFAULT_DEMO_NETWORK_ID.to_string(),
            default_target_network_id: DEFAULT_DEMO_NETWORK_ID.to_string(),
            trusted_foreign_authorities: Arc::new(RwLock::new(HashMap::new())),
            accepted_transfers: Arc::new(RwLock::new(HashSet::new())),
            shacl_validator: None,
        }
    }

    /// Create a bridge bound to explicit source and default target network IDs.
    pub fn with_network_ids(
        blockchain: Arc<RwLock<Blockchain>>,
        local_network_id: impl Into<String>,
        default_target_network_id: impl Into<String>,
    ) -> Result<Self> {
        let local_network_id = local_network_id.into();
        let default_target_network_id = default_target_network_id.into();
        if local_network_id.trim().is_empty() {
            return Err(anyhow!("Bridge local network ID cannot be empty"));
        }
        if default_target_network_id.trim().is_empty() {
            return Err(anyhow!("Bridge default target network ID cannot be empty"));
        }

        Ok(Self {
            blockchain,
            local_network_id,
            default_target_network_id,
            trusted_foreign_authorities: Arc::new(RwLock::new(HashMap::new())),
            accepted_transfers: Arc::new(RwLock::new(HashSet::new())),
            shacl_validator: None,
        })
    }

    /// Set the SHACL validator for this bridge
    pub fn set_validator(&mut self, validator: ShaclValidator) {
        self.shacl_validator = Some(validator);
    }

    /// Register a trusted authority for a foreign network
    pub async fn add_trusted_authority(
        &self,
        network_id: &str,
        public_key_bytes: &[u8],
    ) -> Result<()> {
        let key = VerifyingKey::from_bytes(
            public_key_bytes
                .try_into()
                .map_err(|_| anyhow!("Invalid key length"))?,
        )
        .map_err(|_| anyhow!("Invalid key"))?;

        let mut authorities = self.trusted_foreign_authorities.write().await;
        authorities
            .entry(network_id.to_string())
            .or_default()
            .push(key);
        Ok(())
    }

    /// Generate a proof for a local transaction to be sent elsewhere
    pub async fn export_proof(
        &self,
        block_index: u64,
        transfer_id: Uuid,
        signing_key: &SigningKey,
    ) -> Result<CrossChainProof> {
        self.export_proof_to_network(
            block_index,
            transfer_id,
            &self.default_target_network_id,
            signing_key,
        )
        .await
    }

    /// Generate a proof for a local transaction to be sent to a named network.
    pub async fn export_proof_to_network(
        &self,
        block_index: u64,
        transfer_id: Uuid,
        target_network_id: &str,
        signing_key: &SigningKey,
    ) -> Result<CrossChainProof> {
        if target_network_id.trim().is_empty() {
            return Err(anyhow!("Target network ID cannot be empty"));
        }

        let blockchain = self.blockchain.read().await;

        // Find the block
        let block = blockchain
            .chain
            .iter()
            .find(|b| b.index == block_index)
            .ok_or_else(|| anyhow!("Block {} not found", block_index))?;

        // Construct the message object
        let message = CrossChainMessage {
            source_network_id: self.local_network_id.clone(),
            target_network_id: target_network_id.to_string(),
            transfer_id,
            payload: block.data.clone(),
            timestamp: block.timestamp.clone(),
        };
        let signed_data = cross_chain_signature_payload(&message, block.index, &block.state_root);
        let signature = signing_key.sign(signed_data.as_bytes());

        Ok(CrossChainProof {
            message,
            block_index: block.index,
            state_root: block.state_root.clone(),
            authority_signatures: vec![(
                "primary-authority".to_string(),
                signature.to_bytes().to_vec(),
            )],
        })
    }

    /// Verify and ingest a proof from a foreign chain
    pub async fn import_proof(&self, proof: &CrossChainProof) -> Result<bool> {
        if proof.message.target_network_id != self.local_network_id {
            return Err(anyhow!(
                "Proof target network '{}' does not match local bridge network '{}'",
                proof.message.target_network_id,
                self.local_network_id
            ));
        }

        // 1. Check if we trust the source network
        let authorities = self.trusted_foreign_authorities.read().await;
        let trusted_keys = authorities
            .get(&proof.message.source_network_id)
            .ok_or_else(|| {
                anyhow!(
                    "Unknown source network: {}",
                    proof.message.source_network_id
                )
            })?;

        if trusted_keys.is_empty() {
            return Err(anyhow!(
                "No trusted authorities for network {}",
                proof.message.source_network_id
            ));
        }

        // 2. Verify signatures
        // Reconstruct what was signed. The signature binds both block material
        // and envelope fields so attackers cannot replay a valid proof by
        // mutating transfer/source/target identity after export.
        let signed_data =
            cross_chain_signature_payload(&proof.message, proof.block_index, &proof.state_root);
        let signed_bytes = signed_data.as_bytes();

        let mut valid_signature_found = false;

        for (_auth_id, sig_bytes) in &proof.authority_signatures {
            let signature = Signature::from_bytes(
                sig_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow!("Invalid signature length"))?,
            );

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
        drop(authorities);

        if valid_signature_found {
            // 3. SHACL Validation (if configured)
            if let Some(validator) = &self.shacl_validator {
                tracing::info!("Validating cross-chain payload with SHACL...");
                // Note: We assume payload is RDF Turtle data here.
                // If it's not (e.g. JSON), validation might fail or need parsing adjustment.
                match validator.validate_transaction(&proof.message.payload) {
                    Ok(report) => {
                        if !report.is_valid {
                            tracing::warn!(
                                "❌ SHACL Validation Failed for cross-chain transfer: {:?}",
                                report.violations
                            );
                            return Ok(false); // Reject invalid data
                        }
                        tracing::info!("✅ SHACL Validation Passed");
                    }
                    Err(e) => {
                        tracing::error!("SHACL Validation Error: {}", e);
                        // Decide policy: fail closed?
                        return Err(anyhow!("Validation error: {}", e));
                    }
                }
            }

            // 4. Process the payload (Mint/Unlock)
            // Accepted payloads enter the destination ledger through the normal
            // block-admission path so ontology/semantic validation and integrity
            // checks remain the single production gate.
            {
                let mut accepted_transfers = self.accepted_transfers.write().await;
                if !accepted_transfers.insert(proof.message.transfer_id) {
                    tracing::warn!(
                        "Rejecting replayed cross-chain transfer {} from {}",
                        proof.message.transfer_id,
                        proof.message.source_network_id
                    );
                    return Ok(false);
                }
            }

            if let Err(error) = self
                .blockchain
                .write()
                .await
                .add_block(proof.message.payload.clone())
            {
                self.accepted_transfers
                    .write()
                    .await
                    .remove(&proof.message.transfer_id);
                return Err(error.into());
            }

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

        assert!(bridge.trusted_foreign_authorities.read().await.is_empty());
        assert_eq!(bridge.local_network_id, DEFAULT_DEMO_NETWORK_ID);
    }

    #[tokio::test]
    async fn import_proof_commits_payload_and_rejects_replay() {
        let source_chain = Arc::new(RwLock::new(Blockchain::new()));
        let source_bridge =
            BridgeManager::with_network_ids(source_chain.clone(), "source-net", "dest-net")
                .unwrap();
        let dest_chain = Arc::new(RwLock::new(Blockchain::new()));
        let dest_bridge =
            BridgeManager::with_network_ids(dest_chain.clone(), "dest-net", "source-net").unwrap();

        let payload = r#"@prefix ex: <http://example.com/> .
ex:transfer1 ex:status "locked" ."#;
        source_chain
            .write()
            .await
            .add_block(payload.to_string())
            .expect("source block should be admitted");

        let signing_key = SigningKey::from_bytes(&[2u8; 32]);
        let transfer_id = Uuid::new_v4();
        let proof = source_bridge
            .export_proof(1, transfer_id, &signing_key)
            .await
            .expect("proof export should succeed");

        dest_bridge
            .add_trusted_authority("source-net", signing_key.verifying_key().as_bytes())
            .await
            .expect("trusted authority should be registered");

        let accepted = dest_bridge
            .import_proof(&proof)
            .await
            .expect("valid proof should import");
        assert!(accepted);
        assert_eq!(dest_chain.read().await.chain.len(), 2);

        let replay = dest_bridge
            .import_proof(&proof)
            .await
            .expect("duplicate proof should be handled without panicking");
        assert!(!replay, "duplicate transfer IDs must not be re-admitted");
        assert_eq!(dest_chain.read().await.chain.len(), 2);
    }

    #[tokio::test]
    async fn import_proof_rejects_mutated_unsigned_transfer_identity() {
        let source_chain = Arc::new(RwLock::new(Blockchain::new()));
        let source_bridge =
            BridgeManager::with_network_ids(source_chain.clone(), "source-net", "dest-net")
                .unwrap();
        let dest_chain = Arc::new(RwLock::new(Blockchain::new()));
        let dest_bridge =
            BridgeManager::with_network_ids(dest_chain.clone(), "dest-net", "source-net").unwrap();

        let payload = r#"@prefix ex: <http://example.com/> .
ex:transfer2 ex:status "locked" ."#;
        source_chain
            .write()
            .await
            .add_block(payload.to_string())
            .expect("source block should be admitted");

        let signing_key = SigningKey::from_bytes(&[3u8; 32]);
        let proof = source_bridge
            .export_proof(1, Uuid::new_v4(), &signing_key)
            .await
            .expect("proof export should succeed");

        dest_bridge
            .add_trusted_authority("source-net", signing_key.verifying_key().as_bytes())
            .await
            .expect("trusted authority should be registered");

        let mut tampered_proof = proof;
        tampered_proof.message.transfer_id = Uuid::new_v4();

        let accepted = dest_bridge
            .import_proof(&tampered_proof)
            .await
            .expect("tampered transfer identity should be rejected cleanly");

        assert!(
            !accepted,
            "mutating transfer ID must invalidate the signed proof"
        );
        assert_eq!(dest_chain.read().await.chain.len(), 1);
    }
}
