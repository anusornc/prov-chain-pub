//! Transaction system for ProvChainOrg supply chain traceability
//! 
//! This module implements:
//! - Structured transaction types for supply chain operations
//! - Transaction signing and validation
//! - Transaction pool management
//! - Multi-signature support

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use crate::error::{TransactionError };

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
            use oxigraph::model::NamedNode;
            use std::io::Cursor;
            use oxigraph::io::RdfFormat;
            
            let graph_name = match NamedNode::new(format!("http://provchain.org/tx/{}", self.id)) {
                Ok(name) => name,
                Err(_) => {
                    match NamedNode::new("http://provchain.org/tx/temp") {
                        Ok(temp_name) => temp_name,
                        Err(e) => return Err(TransactionError::InvalidTransaction(
                            format!("Failed to create graph name: {}", e)
                        )),
                    }
                }
            };
            
            let reader = Cursor::new(self.rdf_data.as_bytes());
            if temp_store.store.load_from_reader(RdfFormat::Turtle, reader).is_ok() {
                // Use canonicalization to get a consistent hash
                let canonical_hash = temp_store.canonicalize_graph(&graph_name);
                
                // Include the canonicalized RDF hash in our transaction hash
                let mut hasher = Sha256::new();
                hasher.update(self.id.as_bytes());
                
                let tx_type_json = serde_json::to_string(&self.tx_type)
                    .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize tx_type: {}", e)))?;
                hasher.update(tx_type_json.as_bytes());
                
                let inputs_json = serde_json::to_string(&self.inputs)
                    .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize inputs: {}", e)))?;
                hasher.update(inputs_json.as_bytes());
                
                let outputs_json = serde_json::to_string(&self.outputs)
                    .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize outputs: {}", e)))?;
                hasher.update(outputs_json.as_bytes());
                
                hasher.update(canonical_hash.as_bytes()); // Use canonicalized RDF hash
                hasher.update(self.timestamp.to_rfc3339().as_bytes());
                
                let metadata_json = serde_json::to_string(&self.metadata)
                    .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize metadata: {}", e)))?;
                hasher.update(metadata_json.as_bytes());
                
                hasher.update(&self.nonce.to_le_bytes());
                
                if let Some(fee) = self.fee {
                    hasher.update(&fee.to_le_bytes());
                }
                
                return Ok(format!("{:x}", hasher.finalize()));
            }
        }
        
        // Fallback to original method if RDF parsing fails
        let mut hasher = Sha256::new();
        
        // Include all transaction data except signatures
        hasher.update(self.id.as_bytes());
        
        let tx_type_json = serde_json::to_string(&self.tx_type)
            .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize tx_type: {}", e)))?;
        hasher.update(tx_type_json.as_bytes());
        
        let inputs_json = serde_json::to_string(&self.inputs)
            .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize inputs: {}", e)))?;
        hasher.update(inputs_json.as_bytes());
        
        let outputs_json = serde_json::to_string(&self.outputs)
            .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize outputs: {}", e)))?;
        hasher.update(outputs_json.as_bytes());
        
        hasher.update(self.rdf_data.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        
        let metadata_json = serde_json::to_string(&self.metadata)
            .map_err(|e| TransactionError::InvalidTransaction(format!("Failed to serialize metadata: {}", e)))?;
        hasher.update(metadata_json.as_bytes());
        
        hasher.update(&self.nonce.to_le_bytes());
        
        if let Some(fee) = self.fee {
            hasher.update(&fee.to_le_bytes());
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Sign the transaction with a private key
    pub fn sign(&mut self, signing_key: &SigningKey, signer_id: Uuid) -> Result<(), TransactionError> {
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
            if sig.public_key.verify(hash.as_bytes(), &sig.signature).is_err() {
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
            },
            TransactionType::Compliance => true, // Always require multi-sig for compliance
            TransactionType::Quality => {
                // Only require multi-sig for critical quality tests (e.g., regulatory compliance)
                if let Some(quality_data) = &self.metadata.quality_data {
                    quality_data.test_type.contains("REGULATORY") || 
                    quality_data.test_type.contains("COMPLIANCE") ||
                    quality_data.test_type.contains("CERTIFICATION")
                } else {
                    false
                }
            },
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
            },
            TransactionType::Processing => {
                // Processing transactions should have inputs and outputs
                if self.inputs.is_empty() || self.outputs.is_empty() {
                    return Err(anyhow!("Processing transaction must have inputs and outputs"));
                }
            },
            TransactionType::Transfer => {
                // Transfer transactions should have equal input and output values
                let input_value: f64 = self.inputs.len() as f64; // Simplified
                let output_value: f64 = self.outputs.iter().map(|o| o.value).sum();
                
                if (input_value - output_value).abs() > 0.001 {
                    return Err(anyhow!("Transfer transaction input/output mismatch"));
                }
            },
            TransactionType::Quality => {
                // Quality transactions should have quality data
                if self.metadata.quality_data.is_none() {
                    return Err(anyhow!("Quality transaction must have quality data"));
                }
            },
            TransactionType::Compliance => {
                // Compliance transactions should have compliance info
                if self.metadata.compliance_info.is_none() {
                    return Err(anyhow!("Compliance transaction must have compliance info"));
                }
            },
            _ => {}, // Other types have no specific validation
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
        let type_counts = self.pending.values()
            .fold(HashMap::new(), |mut acc, tx| {
                *acc.entry(tx.tx_type.clone()).or_insert(0) += 1;
                acc
            });

        TransactionPoolStats {
            total_transactions: self.pending.len(),
            type_distribution: type_counts,
            oldest_transaction: self.pending.values()
                .map(|tx| tx.timestamp)
                .min(),
            newest_transaction: self.pending.values()
                .map(|tx| tx.timestamp)
                .max(),
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
        assert!(tx.verify_signatures().expect("Signature verification should succeed"));
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

        tx.sign(&signing_key, signer_id).expect("Transaction signing should succeed");
        
        assert!(pool.add_transaction(tx).is_ok());
        assert_eq!(pool.pending.len(), 1);
    }
}
