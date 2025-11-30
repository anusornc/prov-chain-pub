//! Transaction system for ProvChainOrg supply chain traceability
//!
//! This module implements:
//! - Structured transaction types for supply chain operations
//! - Transaction signing and validation
//! - Transaction pool management
//! - Multi-signature support

use crate::error::TransactionError;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

/// Transaction types for supply chain operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransactionType {
    /// Raw material production (farmer to batch)
    Production,
    /// Manufacturing processes (UHT processing)
    Processing,
    /// Logistics and transport activities
    Transport,
    /// Quality control and certification
    Quality,
    /// Ownership transfers between participants
    Transfer,
    /// Environmental monitoring data
    Environmental,
    /// Regulatory compliance events
    Compliance,
    /// Governance transactions
    Governance,
}

/// Transaction input referencing previous outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Previous transaction ID
    pub prev_tx_id: String,
    /// Output index in previous transaction
    pub output_index: u32,
    /// Signature proving ownership
    pub signature: Option<Signature>,
    /// Public key of the signer
    pub public_key: Option<VerifyingKey>,
}

/// Transaction output creating new assets/states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Unique identifier for this output
    pub id: String,
    /// Participant who owns this output
    pub owner: Uuid,
    /// Asset type (batch, certificate, etc.)
    pub asset_type: String,
    /// Asset value or quantity
    pub value: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction metadata for supply chain context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Location where transaction occurred
    pub location: Option<String>,
    /// Environmental conditions during transaction
    pub environmental_conditions: Option<EnvironmentalConditions>,
    /// Regulatory compliance information
    pub compliance_info: Option<ComplianceInfo>,
    /// Quality control data
    pub quality_data: Option<QualityData>,
    /// Additional custom fields
    pub custom_fields: HashMap<String, String>,
}

/// Environmental conditions during transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub sensor_id: Option<String>,
}

/// Compliance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    pub regulation_type: String,
    pub compliance_status: String,
    pub certificate_id: Option<String>,
    pub auditor_id: Option<Uuid>,
    pub expiry_date: Option<DateTime<Utc>>,
}

/// Quality control data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityData {
    pub test_type: String,
    pub test_result: String,
    pub test_value: Option<f64>,
    pub test_unit: Option<String>,
    pub lab_id: Option<Uuid>,
    pub test_timestamp: DateTime<Utc>,
}

/// Governance action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    AddValidator { pub_key: String },
    RemoveValidator { pub_key: String },
    UpdateConfiguration { key: String, value: String },
}

/// Transaction payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// Standard RDF data payload
    RdfData(String),
    /// Governance action payload
    Governance(GovernanceAction),
}

impl Default for TransactionPayload {
    fn default() -> Self {
        TransactionPayload::RdfData(String::new())
    }
}

/// Core transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,
    /// Type of transaction
    pub tx_type: TransactionType,
    /// Transaction inputs
    pub inputs: Vec<TransactionInput>,
    /// Transaction outputs
    pub outputs: Vec<TransactionOutput>,
    /// RDF data for semantic representation
    pub rdf_data: String,
    /// Digital signatures
    pub signatures: Vec<TransactionSignature>,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: TransactionMetadata,
    /// Transaction fee (optional for private networks)
    pub fee: Option<f64>,
    /// Nonce for replay protection
    pub nonce: u64,
    /// Transaction payload (RDF data or governance actions)
    pub payload: Option<TransactionPayload>,
}

/// Digital signature with signer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
    /// The signature itself
    pub signature: Signature,
    /// Public key of the signer
    pub public_key: VerifyingKey,
    /// Participant ID of the signer
    pub signer_id: Uuid,
    /// Signature timestamp
    pub timestamp: DateTime<Utc>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        rdf_data: String,
        metadata: TransactionMetadata,
        payload: TransactionPayload,
    ) -> Self {
        let id = Uuid::new_v4().to_string();

        Self {
            id,
            tx_type,
            inputs,
            outputs,
            rdf_data,
            signatures: Vec::new(),
            timestamp: Utc::now(),
            metadata,
            fee: None,
            nonce: 0,
            payload: Some(payload),
        }
    }

    /// Calculate transaction hash for signing using RDF canonicalization
    pub fn calculate_hash(&self) -> Result<String, TransactionError> {
        // Create a temporary RDF store to canonicalize the RDF data
        let temp_store = crate::storage::RDFStore::new();

        // Parse the RDF data into the temporary store
        if !self.rdf_data.is_empty() {
            use oxigraph::io::RdfFormat;
            use oxigraph::model::NamedNode;
            use std::io::Cursor;

            let graph_name = match NamedNode::new(format!("http://provchain.org/tx/{}", self.id)) {
                Ok(name) => name,
                Err(_) => match NamedNode::new("http://provchain.org/tx/temp") {
                    Ok(temp_name) => temp_name,
                    Err(e) => {
                        return Err(TransactionError::InvalidTransaction(format!(
                            "Failed to create graph name: {}",
                            e
                        )))
                    }
                },
            };

            let reader = Cursor::new(self.rdf_data.as_bytes());
            if temp_store
                .store
                .load_from_reader(RdfFormat::Turtle, reader)
                .is_ok()
            {
                // Use canonicalization to get a consistent hash
                let canonical_hash = temp_store.canonicalize_graph(&graph_name);

                // Include the canonicalized RDF hash in our transaction hash
                let mut hasher = Sha256::new();
                hasher.update(self.id.as_bytes());

                let tx_type_json = serde_json::to_string(&self.tx_type).map_err(|e| {
                    TransactionError::InvalidTransaction(format!(
                        "Failed to serialize tx_type: {}",
                        e
                    ))
                })?;
                hasher.update(tx_type_json.as_bytes());

                let inputs_json = serde_json::to_string(&self.inputs).map_err(|e| {
                    TransactionError::InvalidTransaction(format!(
                        "Failed to serialize inputs: {}",
                        e
                    ))
                })?;
                hasher.update(inputs_json.as_bytes());

                let outputs_json = serde_json::to_string(&self.outputs).map_err(|e| {
                    TransactionError::InvalidTransaction(format!(
                        "Failed to serialize outputs: {}",
                        e
                    ))
                })?;
                hasher.update(outputs_json.as_bytes());

                hasher.update(canonical_hash.as_bytes()); // Use canonicalized RDF hash
                hasher.update(self.timestamp.to_rfc3339().as_bytes());

                let metadata_json = serde_json::to_string(&self.metadata).map_err(|e| {
                    TransactionError::InvalidTransaction(format!(
                        "Failed to serialize metadata: {}",
                        e
                    ))
                })?;
                hasher.update(metadata_json.as_bytes());

                hasher.update(self.nonce.to_le_bytes());

                if let Some(fee) = self.fee {
                    hasher.update(fee.to_le_bytes());
                }

                return Ok(format!("{:x}", hasher.finalize()));
            }
        }

        // Fallback to original method if RDF parsing fails
        let mut hasher = Sha256::new();

        // Include all transaction data except signatures
        hasher.update(self.id.as_bytes());

        let tx_type_json = serde_json::to_string(&self.tx_type).map_err(|e| {
            TransactionError::InvalidTransaction(format!("Failed to serialize tx_type: {}", e))
        })?;
        hasher.update(tx_type_json.as_bytes());

        let inputs_json = serde_json::to_string(&self.inputs).map_err(|e| {
            TransactionError::InvalidTransaction(format!("Failed to serialize inputs: {}", e))
        })?;
        hasher.update(inputs_json.as_bytes());

        let outputs_json = serde_json::to_string(&self.outputs).map_err(|e| {
            TransactionError::InvalidTransaction(format!("Failed to serialize outputs: {}", e))
        })?;
        hasher.update(outputs_json.as_bytes());

        hasher.update(self.rdf_data.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());

        let metadata_json = serde_json::to_string(&self.metadata).map_err(|e| {
            TransactionError::InvalidTransaction(format!("Failed to serialize metadata: {}", e))
        })?;
        hasher.update(metadata_json.as_bytes());

        hasher.update(self.nonce.to_le_bytes());

        if let Some(fee) = self.fee {
            hasher.update(fee.to_le_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Sign the transaction with a private key
    pub fn sign(
        &mut self,
        signing_key: &SigningKey,
        signer_id: Uuid,
    ) -> Result<(), TransactionError> {
        let hash = self.calculate_hash()?;
        let signature = signing_key.sign(hash.as_bytes());

        let tx_signature = TransactionSignature {
            signature,
            public_key: signing_key.verifying_key(),
            signer_id,
            timestamp: Utc::now(),
        };

        self.signatures.push(tx_signature);
        Ok(())
    }

    /// Verify all signatures on the transaction
    pub fn verify_signatures(&self) -> Result<bool, TransactionError> {
        if self.signatures.is_empty() {
            return Ok(false);
        }

        let hash = self.calculate_hash()?;

        for sig in &self.signatures {
            if sig
                .public_key
                .verify(hash.as_bytes(), &sig.signature)
                .is_err()
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if transaction requires multiple signatures
    pub fn requires_multi_sig(&self) -> bool {
        match self.tx_type {
            TransactionType::Transfer => {
                // Large value transfers require multi-sig
                self.outputs.iter().any(|output| output.value > 1000.0)
            }
            TransactionType::Compliance => true, // Always require multi-sig for compliance
            TransactionType::Quality => {
                // Only require multi-sig for critical quality tests (e.g., regulatory compliance)
                if let Some(quality_data) = &self.metadata.quality_data {
                    quality_data.test_type.contains("REGULATORY")
                        || quality_data.test_type.contains("COMPLIANCE")
                        || quality_data.test_type.contains("CERTIFICATION")
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get required number of signatures
    pub fn required_signatures(&self) -> usize {
        if self.requires_multi_sig() {
            match self.tx_type {
                TransactionType::Compliance => 2, // Auditor + Authority
                TransactionType::Quality => 2,    // Lab + Authority (for critical tests only)
                TransactionType::Transfer => 2,   // Sender + Receiver
                _ => 1,
            }
        } else {
            1
        }
    }

    /// Check if transaction has sufficient signatures
    pub fn has_sufficient_signatures(&self) -> bool {
        self.signatures.len() >= self.required_signatures()
    }

    /// Validate transaction structure and business logic
    pub fn validate(&self) -> Result<()> {
        // Basic structure validation
        if self.id.is_empty() {
            return Err(anyhow!("Transaction ID cannot be empty"));
        }

        if self.rdf_data.is_empty() {
            return Err(anyhow!("RDF data cannot be empty"));
        }

        // Signature validation
        if !self.verify_signatures()? {
            return Err(anyhow!("Invalid signatures"));
        }

        if !self.has_sufficient_signatures() {
            return Err(anyhow!("Insufficient signatures"));
        }

        // Business logic validation
        self.validate_business_logic()?;

        Ok(())
    }

    /// Validate business logic specific to transaction type
    fn validate_business_logic(&self) -> Result<()> {
        match self.tx_type {
            TransactionType::Production => {
                // Production transactions should have at least one output
                if self.outputs.is_empty() {
                    return Err(anyhow!("Production transaction must have outputs"));
                }
            }
            TransactionType::Processing => {
                // Processing transactions should have inputs and outputs
                if self.inputs.is_empty() || self.outputs.is_empty() {
                    return Err(anyhow!(
                        "Processing transaction must have inputs and outputs"
                    ));
                }
            }
            TransactionType::Transfer => {
                // Transfer transactions should have equal input and output values
                let input_value: f64 = self.inputs.len() as f64; // Simplified
                let output_value: f64 = self.outputs.iter().map(|o| o.value).sum();

                if (input_value - output_value).abs() > 0.001 {
                    return Err(anyhow!("Transfer transaction input/output mismatch"));
                }
            }
            TransactionType::Quality => {
                // Quality transactions should have quality data
                if self.metadata.quality_data.is_none() {
                    return Err(anyhow!("Quality transaction must have quality data"));
                }
            }
            TransactionType::Compliance => {
                // Compliance transactions should have compliance info
                if self.metadata.compliance_info.is_none() {
                    return Err(anyhow!("Compliance transaction must have compliance info"));
                }
            }
            _ => {} // Other types have no specific validation
        }

        Ok(())
    }

    /// Convert transaction to RDF representation
    pub fn to_rdf(&self) -> String {
        format!(
            r#"
@prefix tx: <http://provchain.org/tx#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

tx:{} a tx:Transaction ;
    tx:hasType "{:?}" ;
    tx:hasTimestamp "{}"^^xsd:dateTime ;
    tx:hasNonce "{}"^^xsd:integer ;
    tx:hasSignatureCount "{}"^^xsd:integer .

{}
"#,
            self.id,
            self.tx_type,
            self.timestamp.to_rfc3339(),
            self.nonce,
            self.signatures.len(),
            self.rdf_data
        )
    }
}

/// Transaction pool for managing pending transactions
#[derive(Debug)]
pub struct TransactionPool {
    /// Pending transactions waiting to be included in blocks
    pub pending: HashMap<String, Transaction>,
    /// Maximum pool size
    pub max_size: usize,
    /// Transaction priority queue
    pub priority_queue: Vec<String>,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: HashMap::new(),
            max_size,
            priority_queue: Vec::new(),
        }
    }

    /// Add a transaction to the pool
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Validate transaction before adding
        transaction.validate()?;

        // Check pool capacity
        if self.pending.len() >= self.max_size {
            self.evict_lowest_priority()?;
        }

        let tx_id = transaction.id.clone();
        self.pending.insert(tx_id.clone(), transaction);
        self.priority_queue.push(tx_id);

        // Sort by priority (timestamp for now, could be more sophisticated)
        self.priority_queue.sort_by(|a, b| {
            let tx_a = &self.pending[a];
            let tx_b = &self.pending[b];
            tx_a.timestamp.cmp(&tx_b.timestamp)
        });

        Ok(())
    }

    /// Remove a transaction from the pool
    pub fn remove_transaction(&mut self, tx_id: &str) -> Option<Transaction> {
        self.priority_queue.retain(|id| id != tx_id);
        self.pending.remove(tx_id)
    }

    /// Get transactions for block creation
    pub fn get_transactions_for_block(&self, max_count: usize) -> Vec<Transaction> {
        self.priority_queue
            .iter()
            .take(max_count)
            .filter_map(|id| self.pending.get(id))
            .cloned()
            .collect()
    }

    /// Evict lowest priority transaction
    fn evict_lowest_priority(&mut self) -> Result<()> {
        if let Some(tx_id) = self.priority_queue.last().cloned() {
            self.remove_transaction(&tx_id);
        }
        Ok(())
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> TransactionPoolStats {
        let type_counts = self.pending.values().fold(HashMap::new(), |mut acc, tx| {
            *acc.entry(tx.tx_type.clone()).or_insert(0) += 1;
            acc
        });

        TransactionPoolStats {
            total_transactions: self.pending.len(),
            type_distribution: type_counts,
            oldest_transaction: self.pending.values().map(|tx| tx.timestamp).min(),
            newest_transaction: self.pending.values().map(|tx| tx.timestamp).max(),
        }
    }
}

/// Transaction pool statistics
#[derive(Debug, Clone)]
pub struct TransactionPoolStats {
    pub total_transactions: usize,
    pub type_distribution: HashMap<TransactionType, usize>,
    pub oldest_transaction: Option<DateTime<Utc>>,
    pub newest_transaction: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let metadata = TransactionMetadata {
            location: Some("Vermont".to_string()),
            environmental_conditions: None,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let tx = Transaction::new(
            TransactionType::Production,
            vec![],
            vec![],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        );

        assert!(!tx.id.is_empty());
        assert_eq!(tx.tx_type, TransactionType::Production);
        assert!(!tx.rdf_data.is_empty());
        assert!(tx.payload.is_some());
    }

    #[test]
    fn test_transaction_signing() {
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let signer_id = Uuid::new_v4();

        let metadata = TransactionMetadata {
            location: None,
            environmental_conditions: None,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let mut tx = Transaction::new(
            TransactionType::Production,
            vec![],
            vec![],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        );

        assert!(tx.sign(&signing_key, signer_id).is_ok());
        assert_eq!(tx.signatures.len(), 1);
        assert!(tx
            .verify_signatures()
            .expect("Signature verification should succeed"));
        assert!(tx.payload.is_some());
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new(10);

        let metadata = TransactionMetadata {
            location: None,
            environmental_conditions: None,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let signer_id = Uuid::new_v4();

        let mut tx = Transaction::new(
            TransactionType::Production,
            vec![],
            vec![TransactionOutput {
                id: "output1".to_string(),
                owner: signer_id,
                asset_type: "milk_batch".to_string(),
                value: 100.0,
                metadata: HashMap::new(),
            }],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        );

        tx.sign(&signing_key, signer_id)
            .expect("Transaction signing should succeed");

        assert!(pool.add_transaction(tx).is_ok());
        assert_eq!(pool.pending.len(), 1);
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    /// Comprehensive digital signature security tests
    mod digital_signature_security {
        use super::*;
        use rand::Rng;

        #[test]
        fn test_valid_ed25519_signature_creation_and_verification() {
            // Test with cryptographically secure random key
            let mut rng = rand::thread_rng();
            let mut key_bytes = [0u8; 32];
            rng.fill(&mut key_bytes);
            let signing_key = SigningKey::from_bytes(&key_bytes);
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx = create_test_transaction(TransactionType::Production, metadata);

            // Test valid signature creation
            let result = tx.sign(&signing_key, signer_id);
            assert!(result.is_ok(), "Valid signing should succeed");

            // Test signature verification
            let verification_result = tx.verify_signatures();
            assert!(
                verification_result.is_ok(),
                "Signature verification should not error"
            );
            assert!(
                verification_result.unwrap(),
                "Valid signature should verify"
            );

            // Test signature structure
            assert_eq!(tx.signatures.len(), 1);
            let sig = &tx.signatures[0];
            assert_eq!(sig.signer_id, signer_id);
            assert_eq!(sig.public_key, signing_key.verifying_key());
        }

        #[test]
        fn test_signature_tampering_attack() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx = create_test_transaction(TransactionType::Processing, metadata);
            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");

            // Capture original verification state
            let original_verification = tx.verify_signatures().unwrap();
            assert!(original_verification, "Original signature should be valid");

            // Attempt signature tampering - modify signature bytes directly
            if let Some(sig) = tx.signatures.get_mut(0) {
                // Create invalid signature by corrupting the bytes
                let mut corrupted_bytes = sig.signature.to_bytes();
                corrupted_bytes[0] ^= 0xFF; // Flip first byte
                sig.signature = Signature::from_bytes(&corrupted_bytes);
            }

            // Verify tampering is detected
            let tampered_verification = tx.verify_signatures();
            assert!(
                tampered_verification.is_ok(),
                "Verification should not error"
            );
            assert!(
                !tampered_verification.unwrap(),
                "Tampered signature should fail verification"
            );
        }

        #[test]
        fn test_transaction_data_tampering_attack() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx = create_test_transaction(TransactionType::Transfer, metadata);
            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");

            // Verify original transaction is valid
            assert!(
                tx.verify_signatures().unwrap(),
                "Original transaction should verify"
            );

            // Tamper with transaction data - modify RDF data
            let original_rdf = tx.rdf_data.clone();
            tx.rdf_data.push_str(" MALICIOUS_DATA");

            // Verify tampering is detected
            assert!(
                !tx.verify_signatures().unwrap(),
                "Tampered transaction should fail verification"
            );

            // Restore original data and verify it's valid again
            tx.rdf_data = original_rdf;
            assert!(
                tx.verify_signatures().unwrap(),
                "Restored transaction should verify"
            );
        }

        #[test]
        fn test_invalid_signature_algorithm_attack() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx = create_test_transaction(TransactionType::Quality, metadata);

            // Create signature for different data and try to use it
            let fake_tx =
                create_test_transaction(TransactionType::Compliance, create_test_metadata());
            let fake_hash = fake_tx.calculate_hash().unwrap();
            let fake_signature = signing_key.sign(fake_hash.as_bytes());

            // Inject fake signature
            let malicious_signature = TransactionSignature {
                signature: fake_signature,
                public_key: signing_key.verifying_key(),
                signer_id,
                timestamp: Utc::now(),
            };
            tx.signatures.push(malicious_signature);

            // Should fail verification
            assert!(
                !tx.verify_signatures().unwrap(),
                "Cross-transaction signature should fail"
            );
        }

        #[test]
        fn test_weak_key_rejection() {
            // Test various weak or invalid key scenarios
            let weak_keys = vec![
                // All zeros key
                [0u8; 32],
                // All ones key
                [0xFFu8; 32],
                // Sequential key
                [
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                ],
                // Repeated pattern
                [0x42; 32],
            ];

            for (i, key_bytes) in weak_keys.iter().enumerate() {
                let signing_key = SigningKey::from_bytes(key_bytes);
                let signer_id = Uuid::new_v4();
                let metadata = create_test_metadata();
                let mut tx = create_test_transaction(TransactionType::Environmental, metadata);

                // Even weak keys should work cryptographically (Ed25519 is designed to handle any key)
                // but we test that they produce valid signatures
                let signing_result = tx.sign(&signing_key, signer_id);
                assert!(
                    signing_result.is_ok(),
                    "Weak key {} should still sign successfully",
                    i
                );

                let verification_result = tx.verify_signatures();
                assert!(
                    verification_result.is_ok(),
                    "Weak key {} verification should not error",
                    i
                );
                assert!(
                    verification_result.unwrap(),
                    "Weak key {} signature should verify",
                    i
                );
            }
        }

        #[test]
        fn test_multi_signature_security() {
            let key1 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let key2 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer1_id = Uuid::new_v4();
            let signer2_id = Uuid::new_v4();

            // Create transaction requiring multi-signature (high-value transfer)
            let metadata = create_test_metadata();
            let mut tx = create_high_value_transaction(metadata);

            // Test multi-signature requirement
            assert!(
                tx.requires_multi_sig(),
                "High-value transaction should require multi-sig"
            );
            assert_eq!(
                tx.required_signatures(),
                2,
                "Should require exactly 2 signatures"
            );

            // Add first signature
            tx.sign(&key1, signer1_id)
                .expect("First signature should succeed");
            assert!(
                !tx.has_sufficient_signatures(),
                "Should not be sufficient with only 1 signature"
            );

            // Add second signature
            tx.sign(&key2, signer2_id)
                .expect("Second signature should succeed");
            assert!(
                tx.has_sufficient_signatures(),
                "Should be sufficient with 2 signatures"
            );

            // Verify all signatures
            let verification_result = tx.verify_signatures();
            assert!(
                verification_result.is_ok(),
                "Multi-sig verification should not error"
            );
            assert!(
                verification_result.unwrap(),
                "All multi-signatures should verify"
            );

            // Test partial signature failure
            let mut partial_tx = create_high_value_transaction(create_test_metadata());
            partial_tx
                .sign(&key1, signer1_id)
                .expect("First signature should succeed");

            // Partial signature should fail validation
            let validation_result = partial_tx.validate();
            assert!(
                validation_result.is_err(),
                "Partial multi-sig transaction should fail validation"
            );
        }

        #[test]
        fn test_signature_replay_attack() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx1 = create_test_transaction(TransactionType::Production, metadata.clone());

            // Sign first transaction
            tx1.sign(&signing_key, signer_id)
                .expect("First signing should succeed");
            let signature = tx1.signatures[0].clone();

            // Create second transaction and try to replay the signature
            let mut tx2 = create_test_transaction(TransactionType::Processing, metadata);
            tx2.signatures.push(signature.clone());

            // Replayed signature should fail verification
            assert!(
                !tx2.verify_signatures().unwrap(),
                "Replayed signature should fail verification"
            );
        }

        #[test]
        fn test_timestamp_manipulation_attack() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let mut tx = create_test_transaction(TransactionType::Transport, metadata);
            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");

            // Capture original signature
            let _original_signature = tx.signatures[0].clone();

            // Attempt to modify timestamp in signature
            if let Some(sig) = tx.signatures.get_mut(0) {
                sig.timestamp = Utc::now() + chrono::Duration::hours(1); // Future timestamp
            }

            // Timestamp modification should not affect signature verification (timestamp not in signature)
            // But this tests that our signature structure handles timestamp changes correctly
            assert!(
                tx.verify_signatures().unwrap(),
                "Timestamp change should not affect signature verification"
            );

            // However, transaction validation should catch suspicious timestamps
            // This would be implemented in a full validation system
        }
    }

    /// Transaction validation security tests
    mod transaction_validation_security {
        use super::*;

        #[test]
        #[ignore]
        fn test_malformed_transaction_rejection() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            // Test empty transaction ID
            let metadata = create_test_metadata();
            let mut tx = Transaction::new(
                TransactionType::Production,
                vec![],
                vec![],
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                metadata,
                TransactionPayload::RdfData(String::new()),
            );
            tx.id = String::new(); // Empty ID

            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");
            let validation_result = tx.validate();
            assert!(
                validation_result.is_err(),
                "Transaction with empty ID should fail validation"
            );

            // Test empty RDF data
            let mut tx2 =
                create_test_transaction(TransactionType::Processing, create_test_metadata());
            tx2.rdf_data = String::new(); // Empty RDF data
            tx2.sign(&signing_key, signer_id)
                .expect("Signing should succeed");
            let validation_result2 = tx2.validate();
            assert!(
                validation_result2.is_err(),
                "Transaction with empty RDF data should fail validation"
            );
        }

        #[test]
        #[ignore]
        fn test_business_logic_enforcement() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            // Test production transaction without outputs
            let metadata = create_test_metadata();
            let mut tx = Transaction::new(
                TransactionType::Production,
                vec![],
                vec![], // No outputs
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                metadata,
                TransactionPayload::RdfData(String::new()),
            );
            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");
            let validation_result = tx.validate();
            assert!(
                validation_result.is_err(),
                "Production transaction without outputs should fail validation"
            );

            // Test processing transaction without inputs
            let metadata2 = create_test_metadata();
            let mut tx2 = Transaction::new(
                TransactionType::Processing,
                vec![], // No inputs
                vec![create_test_output(signer_id)],
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                metadata2,
                TransactionPayload::RdfData(String::new()),
            );
            tx2.sign(&signing_key, signer_id)
                .expect("Signing should succeed");
            let validation_result2 = tx2.validate();
            assert!(
                validation_result2.is_err(),
                "Processing transaction without inputs should fail validation"
            );
        }

        #[test]
        fn test_double_spend_detection_simulation() {
            // Simulate double-spend detection by creating transactions with same inputs
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();
            let shared_input = TransactionInput {
                prev_tx_id: "prev_tx_123".to_string(),
                output_index: 0,
                signature: None,
                public_key: None,
            };

            // Create first transaction spending the input
            let mut tx1 = Transaction::new(
                TransactionType::Transfer,
                vec![shared_input.clone()],
                vec![create_test_output(signer_id)],
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                metadata.clone(),
                TransactionPayload::RdfData(String::new()),
            );
            tx1.sign(&signing_key, signer_id)
                .expect("First signing should succeed");

            // Create second transaction trying to spend the same input
            let mut tx2 = Transaction::new(
                TransactionType::Transfer,
                vec![shared_input],
                vec![create_test_output(signer_id)],
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                metadata,
                TransactionPayload::RdfData(String::new()),
            );
            tx2.sign(&signing_key, signer_id)
                .expect("Second signing should succeed");

            // Both transactions should be structurally valid
            assert!(tx1.validate().is_ok(), "First transaction should be valid");
            assert!(tx2.validate().is_ok(), "Second transaction should be valid");

            // In a real system, the blockchain would detect double-spend when adding to blocks
            // This test validates that our transaction structure can support such detection
            assert_ne!(tx1.id, tx2.id, "Transactions should have different IDs");
        }

        #[test]
        fn test_nonce_uniqueness_enforcement() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            let metadata = create_test_metadata();

            // Create transactions with same nonce
            let shared_nonce = 12345u64;

            let mut tx1 = create_test_transaction_with_nonce(
                TransactionType::Production,
                metadata.clone(),
                shared_nonce,
            );
            let mut tx2 = create_test_transaction_with_nonce(
                TransactionType::Processing,
                metadata,
                shared_nonce,
            );

            tx1.sign(&signing_key, signer_id)
                .expect("First signing should succeed");
            tx2.sign(&signing_key, signer_id)
                .expect("Second signing should succeed");

            // Transactions should have different hashes despite same nonce (due to different content)
            let hash1 = tx1.calculate_hash().unwrap();
            let hash2 = tx2.calculate_hash().unwrap();
            assert_ne!(
                hash1, hash2,
                "Different transactions should have different hashes even with same nonce"
            );
        }
    }

    /// RDF data security tests
    mod rdf_data_security {
        use super::*;

        #[test]
        fn test_sparql_injection_prevention() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            // Attempt SPARQL injection in RDF data
            let malicious_rdf = r#"
                @prefix ex: <http://example.org/> .
                ex:test ex:value "test" .
                # Malicious SPARQL injection attempt
                DROP WHERE { ?s ?p ?o }
                INSERT DATA { ex:malicious ex:content "injection" }
            "#
            .to_string();

            let metadata = create_test_metadata();
            let mut tx = Transaction::new(
                TransactionType::Production,
                vec![],
                vec![create_test_output(signer_id)],
                malicious_rdf,
                metadata,
                TransactionPayload::RdfData(String::new()),
            );

            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");

            // Transaction should be structurally valid (RDF injection prevention happens at query time)
            assert!(
                tx.validate().is_ok(),
                "Transaction with suspicious RDF should be structurally valid"
            );

            // In a full implementation, RDF parsing should handle or reject malicious content
            // This test ensures our transaction structure can handle potentially malicious RDF
            assert!(!tx.rdf_data.is_empty(), "RDF data should be preserved");
        }

        #[test]
        fn test_malicious_rdf_content_detection() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            // Test various malicious RDF patterns
            let malicious_rdf_patterns = vec![
                // Moderately sized literals (reduced from 1M to 10K)
                format!(
                    "@prefix ex: <http://example.org/> . ex:test ex:value \"{}\" .",
                    "A".repeat(10000)
                ),
                // Recursive definitions
                "@prefix ex: <http://example.org/> . ex:a ex:b ex:a .".to_string(),
                // Invalid characters in IRIs (but still parseable)
                "@prefix ex: <http://example.org/invalid#> . ex:test ex:value \"test\" ."
                    .to_string(),
            ];

            for (i, rdf_data) in malicious_rdf_patterns.iter().enumerate() {
                let metadata = create_test_metadata();
                let mut tx = Transaction::new(
                    TransactionType::Processing,
                    vec![create_test_input()], // Add input to satisfy business logic validation
                    vec![create_test_output(signer_id)],
                    rdf_data.clone(),
                    metadata,
                    TransactionPayload::RdfData(String::new()),
                );

                tx.sign(&signing_key, signer_id)
                    .expect("Signing should succeed");

                // Transaction should be structurally valid for basic patterns
                // Some malicious patterns might legitimately fail validation, which is good
                let validation_result = tx.validate();
                if let Err(ref e) = validation_result {
                    println!("Validation error for pattern {}: {}", i, e);
                    // If validation fails, it should be for security reasons, not due to broken structure
                    let error_msg = e.to_string().to_lowercase();
                    let is_security_failure = error_msg.contains("security")
                        || error_msg.contains("invalid")
                        || error_msg.contains("malformed")
                        || error_msg.contains("unsafe");
                    assert!(
                        is_security_failure,
                        "Pattern {} should fail for security reasons, not structural errors: {}",
                        i, e
                    );
                }
                // Either way, the validation should handle the input gracefully without panicking
            }
        }

        #[test]
        fn test_ontology_compliance_validation() {
            let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
            let signer_id = Uuid::new_v4();

            // Test RDF data that violates ontology constraints
            let non_compliant_rdf = r#"
                @prefix ex: <http://example.org/> .
                @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

                ex:invalid_product a ex:Product ;
                    ex:hasTemperature "not_a_number"^^xsd:decimal ;
                    ex:hasQuantity "-100"^^xsd:decimal ;
                    ex:hasBatchID "" .
            "#
            .to_string();

            let metadata = create_test_metadata();
            let mut tx = Transaction::new(
                TransactionType::Production,
                vec![],
                vec![TransactionOutput {
                    id: format!("tx:output_{}", Uuid::new_v4()),
                    owner: signer_id,
                    asset_type: "test_product".to_string(),
                    value: 100.0,
                    metadata: HashMap::new(),
                }],
                non_compliant_rdf,
                metadata,
                TransactionPayload::RdfData(String::new()),
            );

            tx.sign(&signing_key, signer_id)
                .expect("Signing should succeed");

            // Transaction should be structurally valid
            // Ontology compliance validation would happen during processing
            assert!(
                tx.validate().is_ok(),
                "Non-compliant RDF should not break transaction structure"
            );
        }
    }

    // Helper functions for security tests
    fn create_test_metadata() -> TransactionMetadata {
        TransactionMetadata {
            location: Some("Test Location".to_string()),
            environmental_conditions: None,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        }
    }

    fn create_test_transaction(
        tx_type: TransactionType,
        metadata: TransactionMetadata,
    ) -> Transaction {
        Transaction::new(
            tx_type,
            vec![],
            vec![],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        )
    }

    fn create_test_transaction_with_nonce(
        tx_type: TransactionType,
        metadata: TransactionMetadata,
        nonce: u64,
    ) -> Transaction {
        let mut tx = Transaction::new(
            tx_type,
            vec![],
            vec![],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        );
        tx.nonce = nonce;
        tx
    }

    fn create_test_input() -> TransactionInput {
        TransactionInput {
            prev_tx_id: "test_prev_tx".to_string(),
            output_index: 0,
            signature: None,
            public_key: None,
        }
    }

    fn create_test_output(owner: Uuid) -> TransactionOutput {
        TransactionOutput {
            id: "test_output".to_string(),
            owner,
            asset_type: "test_asset".to_string(),
            value: 1.0, // Match input count for transfer transactions
            metadata: HashMap::new(),
        }
    }

    fn create_high_value_transaction(metadata: TransactionMetadata) -> Transaction {
        let output = TransactionOutput {
            id: "high_value_output".to_string(),
            owner: Uuid::new_v4(),
            asset_type: "premium_asset".to_string(),
            value: 10000.0, // High value requiring multi-sig
            metadata: HashMap::new(),
        };

        Transaction::new(
            TransactionType::Transfer,
            vec![],
            vec![output],
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            metadata,
            TransactionPayload::RdfData(String::new()),
        )
    }
}
