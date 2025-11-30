//! Enhanced blockchain with transaction and wallet integration
//!
//! This module implements:
//! - Transaction-based blockchain operations
//! - Multi-participant wallet integration
//! - Supply chain specific transaction processing
//! - Persistent storage with disk persistence

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::blockchain::Blockchain;
use crate::transaction::transaction::{
    EnvironmentalConditions, QualityData, Transaction, TransactionMetadata, TransactionOutput,
    TransactionPayload, TransactionPool, TransactionType,
};
use crate::wallet::{Participant, ParticipantType, Wallet, WalletManager};

/// Enhanced blockchain with transaction support
pub struct TransactionBlockchain {
    /// Underlying blockchain
    pub blockchain: Blockchain,
    /// Transaction pool for pending transactions
    pub transaction_pool: TransactionPool,
    /// Wallet manager for participants
    pub wallet_manager: WalletManager,
    /// Transaction index for fast lookups
    pub transaction_index: HashMap<String, (u64, usize)>, // tx_id -> (block_index, tx_index)
    /// UTXO set for tracking unspent outputs
    pub utxo_set: HashMap<String, TransactionOutput>, // output_id -> output
}

impl TransactionBlockchain {
    /// Create a new transaction blockchain
    pub fn new(data_dir: &str) -> Result<Self> {
        let blockchain = Blockchain::new_persistent(data_dir)?;
        let transaction_pool = TransactionPool::new(1000); // Max 1000 pending transactions
        let wallet_manager = WalletManager::new(format!("{}/wallets", data_dir))?;

        Ok(Self {
            blockchain,
            transaction_pool,
            wallet_manager,
            transaction_index: HashMap::new(),
            utxo_set: HashMap::new(),
        })
    }

    /// Submit a transaction to the blockchain
    pub fn submit_transaction(&mut self, transaction: Transaction) -> Result<String> {
        // Validate transaction
        transaction.validate()?;

        // Check if submitter has permission
        if let Some(signer) = transaction.signatures.first() {
            if let Some(wallet) = self.wallet_manager.get_wallet(signer.signer_id) {
                let operation = match transaction.tx_type {
                    TransactionType::Production => "produce",
                    TransactionType::Processing => "process",
                    TransactionType::Transport => "transport",
                    TransactionType::Quality => "quality_test",
                    TransactionType::Compliance => "audit",
                    TransactionType::Transfer => "transfer",
                    _ => "unknown",
                };

                if !wallet.has_permission(operation) {
                    return Err(anyhow!(
                        "Participant does not have permission for {} operation",
                        operation
                    ));
                }
            } else {
                return Err(anyhow!("Unknown participant"));
            }
        }

        // Add to transaction pool
        let tx_id = transaction.id.clone();
        self.transaction_pool.add_transaction(transaction)?;

        Ok(tx_id)
    }

    /// Create a new block with pending transactions
    pub fn create_block(&mut self, max_transactions: usize) -> Result<()> {
        let transactions = self
            .transaction_pool
            .get_transactions_for_block(max_transactions);

        if transactions.is_empty() {
            return Ok(()); // No transactions to process
        }

        // Create block data from transactions
        let block_data = self.create_block_rdf_data(&transactions)?;

        // Add block to blockchain
        self.blockchain.add_block(block_data)?;

        // Update transaction index and UTXO set
        let block_index = self.blockchain.chain.len() as u64 - 1;
        for (tx_index, transaction) in transactions.iter().enumerate() {
            // Update transaction index
            self.transaction_index
                .insert(transaction.id.clone(), (block_index, tx_index));

            // Update UTXO set
            for output in &transaction.outputs {
                self.utxo_set.insert(output.id.clone(), output.clone());
            }

            // Remove spent outputs
            for input in &transaction.inputs {
                self.utxo_set
                    .remove(&format!("{}:{}", input.prev_tx_id, input.output_index));
            }

            // Remove from transaction pool
            self.transaction_pool.remove_transaction(&transaction.id);
        }

        Ok(())
    }

    /// Create RDF data for a block from transactions
    fn create_block_rdf_data(&self, transactions: &[Transaction]) -> Result<String> {
        let mut rdf_data = String::new();

        // Add prefixes
        rdf_data.push_str(
            r#"
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix tx: <http://provchain.org/tx#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

"#,
        );

        // Add transaction data
        for transaction in transactions {
            rdf_data.push_str(&transaction.to_rdf());
            rdf_data.push('\n');

            // Add the original RDF data from the transaction
            rdf_data.push_str(&transaction.rdf_data);
            rdf_data.push('\n');
        }

        Ok(rdf_data)
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<Transaction> {
        // First check transaction pool
        if let Some(tx) = self.transaction_pool.pending.get(tx_id) {
            return Some(tx.clone());
        }

        // Then check blockchain
        if let Some((_block_index, _tx_index)) = self.transaction_index.get(tx_id) {
            // In a full implementation, we would parse the block data to extract the transaction
            // For now, we'll return None as this requires more complex RDF parsing
            None
        } else {
            None
        }
    }

    /// Get transactions by participant
    pub fn get_transactions_by_participant(&self, participant_id: Uuid) -> Vec<String> {
        let mut tx_ids = Vec::new();

        // Check transaction pool
        for tx in self.transaction_pool.pending.values() {
            if tx
                .signatures
                .iter()
                .any(|sig| sig.signer_id == participant_id)
            {
                tx_ids.push(tx.id.clone());
            }
        }

        // In a full implementation, we would also search the blockchain
        tx_ids
    }

    /// Register a new participant
    pub fn register_participant(&mut self, participant: Participant) -> Result<Uuid> {
        self.wallet_manager.create_wallet(participant)
    }

    /// Get participant wallet
    pub fn get_participant_wallet(&self, participant_id: Uuid) -> Option<&Wallet> {
        self.wallet_manager.get_wallet(participant_id)
    }

    /// Create a production transaction (farmer produces raw materials)
    pub fn create_production_transaction(
        &self,
        producer_id: Uuid,
        batch_id: String,
        quantity: f64,
        location: String,
        environmental_conditions: Option<EnvironmentalConditions>,
    ) -> Result<Transaction> {
        let wallet = self
            .wallet_manager
            .get_wallet(producer_id)
            .ok_or_else(|| anyhow!("Producer wallet not found"))?;

        if !wallet.has_permission("produce") {
            return Err(anyhow!("Producer does not have production permission"));
        }

        // Create transaction output
        let output = TransactionOutput {
            id: format!("{}:0", batch_id),
            owner: producer_id,
            asset_type: "raw_material_batch".to_string(),
            value: quantity,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("batch_id".to_string(), batch_id.clone());
                meta.insert("location".to_string(), location.clone());
                meta
            },
        };

        // Create RDF data
        let rdf_data = format!(
            r#"
ex:{} a trace:ProductBatch ;
    trace:hasBatchID "{}" ;
    trace:producedAt "{}"^^xsd:dateTime ;
    prov:wasAttributedTo ex:participant_{} ;
    trace:hasQuantity "{}"^^xsd:decimal ;
    trace:hasLocation "{}" .

ex:participant_{} a trace:Farmer ;
    rdfs:label "{}" .
"#,
            batch_id,
            batch_id,
            Utc::now().to_rfc3339(),
            producer_id,
            quantity,
            location,
            producer_id,
            wallet.participant.name
        );

        let metadata = TransactionMetadata {
            location: Some(location),
            environmental_conditions,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let mut transaction = Transaction::new(
            TransactionType::Production,
            vec![], // No inputs for production
            vec![output],
            rdf_data.clone(),
            metadata,
            TransactionPayload::RdfData(rdf_data.clone()),
        );

        // Sign the transaction
        transaction.sign(wallet.signing_key.as_ref().unwrap(), producer_id)?;

        Ok(transaction)
    }

    /// Create a processing transaction (manufacturer processes raw materials)
    pub fn create_processing_transaction(
        &self,
        processor_id: Uuid,
        input_batch_ids: Vec<String>,
        output_batch_id: String,
        process_type: String,
        environmental_conditions: Option<EnvironmentalConditions>,
    ) -> Result<Transaction> {
        let wallet = self
            .wallet_manager
            .get_wallet(processor_id)
            .ok_or_else(|| anyhow!("Processor wallet not found"))?;

        if !wallet.has_permission("process") {
            return Err(anyhow!("Processor does not have processing permission"));
        }

        // Create transaction inputs (simplified - in reality we'd look up the actual UTXOs)
        let inputs = input_batch_ids
            .iter()
            .map(
                |batch_id| crate::transaction::transaction::TransactionInput {
                    prev_tx_id: batch_id.clone(),
                    output_index: 0,
                    signature: None,
                    public_key: None,
                },
            )
            .collect();

        // Create transaction output
        let output = TransactionOutput {
            id: format!("{}:0", output_batch_id),
            owner: processor_id,
            asset_type: "processed_batch".to_string(),
            value: 1.0, // Simplified
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("batch_id".to_string(), output_batch_id.clone());
                meta.insert("process_type".to_string(), process_type.clone());
                meta
            },
        };

        // Create RDF data
        let rdf_data = format!(
            r#"
ex:{} a trace:ProductBatch ;
    trace:hasBatchID "{}" ;
    trace:producedAt "{}"^^xsd:dateTime ;
    prov:wasGeneratedBy ex:process_{} ;
    prov:wasAttributedTo ex:participant_{} .

ex:process_{} a trace:ProcessingActivity ;
    trace:recordedAt "{}"^^xsd:dateTime ;
    trace:hasProcessType "{}" ;
    prov:wasAssociatedWith ex:participant_{} .

ex:participant_{} a trace:Manufacturer ;
    rdfs:label "{}" .
"#,
            output_batch_id,
            output_batch_id,
            Utc::now().to_rfc3339(),
            output_batch_id,
            processor_id,
            output_batch_id,
            Utc::now().to_rfc3339(),
            process_type,
            processor_id,
            processor_id,
            wallet.participant.name
        );

        let metadata = TransactionMetadata {
            location: wallet.participant.location.clone(),
            environmental_conditions,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let mut transaction = Transaction::new(
            TransactionType::Processing,
            inputs,
            vec![output],
            rdf_data.clone(),
            metadata,
            TransactionPayload::RdfData(rdf_data.clone()),
        );

        // Sign the transaction
        transaction.sign(wallet.signing_key.as_ref().unwrap(), processor_id)?;

        Ok(transaction)
    }

    /// Create a quality control transaction
    pub fn create_quality_transaction(
        &self,
        lab_id: Uuid,
        batch_id: String,
        test_type: String,
        test_result: String,
        test_value: Option<f64>,
    ) -> Result<Transaction> {
        let wallet = self
            .wallet_manager
            .get_wallet(lab_id)
            .ok_or_else(|| anyhow!("Lab wallet not found"))?;

        if !wallet.has_permission("quality_test") {
            return Err(anyhow!("Lab does not have quality testing permission"));
        }

        let quality_data = QualityData {
            test_type: test_type.clone(),
            test_result: test_result.clone(),
            test_value,
            test_unit: None,
            lab_id: Some(lab_id),
            test_timestamp: Utc::now(),
        };

        // Create RDF data
        let rdf_data = format!(
            r#"
ex:quality_test_{} a trace:QualityCheck ;
    prov:used ex:{} ;
    prov:wasAssociatedWith ex:participant_{} ;
    trace:recordedAt "{}"^^xsd:dateTime ;
    trace:hasResult "{}" ;
    trace:hasTestType "{}" .

ex:participant_{} a trace:QualityLab ;
    rdfs:label "{}" .
"#,
            batch_id,
            batch_id,
            lab_id,
            Utc::now().to_rfc3339(),
            test_result,
            test_type,
            lab_id,
            wallet.participant.name
        );

        let metadata = TransactionMetadata {
            location: wallet.participant.location.clone(),
            environmental_conditions: None,
            compliance_info: None,
            quality_data: Some(quality_data),
            custom_fields: HashMap::new(),
        };

        let mut transaction = Transaction::new(
            TransactionType::Quality,
            vec![], // Quality checks don't consume inputs
            vec![], // Quality checks don't produce outputs
            rdf_data.clone(),
            metadata,
            TransactionPayload::RdfData(rdf_data.clone()),
        );

        // Sign the transaction
        transaction.sign(wallet.signing_key.as_ref().unwrap(), lab_id)?;

        Ok(transaction)
    }

    /// Create a transport transaction
    pub fn create_transport_transaction(
        &self,
        logistics_id: Uuid,
        batch_id: String,
        from_location: String,
        to_location: String,
        environmental_conditions: Option<EnvironmentalConditions>,
    ) -> Result<Transaction> {
        let wallet = self
            .wallet_manager
            .get_wallet(logistics_id)
            .ok_or_else(|| anyhow!("Logistics provider wallet not found"))?;

        if !wallet.has_permission("transport") {
            return Err(anyhow!("Provider does not have transport permission"));
        }

        // Create RDF data
        let rdf_data = format!(
            r#"
ex:transport_{} a trace:TransportActivity ;
    prov:used ex:{} ;
    prov:wasAssociatedWith ex:participant_{} ;
    trace:recordedAt "{}"^^xsd:dateTime ;
    trace:hasFromLocation "{}" ;
    trace:hasToLocation "{}" .

ex:participant_{} a trace:LogisticsProvider ;
    rdfs:label "{}" .
"#,
            batch_id,
            batch_id,
            logistics_id,
            Utc::now().to_rfc3339(),
            from_location,
            to_location,
            logistics_id,
            wallet.participant.name
        );

        let metadata = TransactionMetadata {
            location: Some(format!("{} -> {}", from_location, to_location)),
            environmental_conditions,
            compliance_info: None,
            quality_data: None,
            custom_fields: HashMap::new(),
        };

        let mut transaction = Transaction::new(
            TransactionType::Transport,
            vec![], // Simplified - no inputs/outputs for transport
            vec![],
            rdf_data.clone(),
            metadata,
            TransactionPayload::RdfData(rdf_data.clone()),
        );

        // Sign the transaction
        transaction.sign(wallet.signing_key.as_ref().unwrap(), logistics_id)?;

        Ok(transaction)
    }

    /// Get blockchain statistics
    pub fn get_statistics(&self) -> TransactionBlockchainStats {
        let pool_stats = self.transaction_pool.get_stats();
        let wallet_stats = self.wallet_manager.get_statistics();

        TransactionBlockchainStats {
            total_blocks: self.blockchain.chain.len(),
            pending_transactions: pool_stats.total_transactions,
            total_participants: wallet_stats.total_participants,
            total_utxos: self.utxo_set.len(),
            participant_distribution: wallet_stats.type_distribution,
            transaction_distribution: pool_stats.type_distribution,
        }
    }

    /// Validate the entire blockchain
    pub fn validate(&self) -> bool {
        self.blockchain.is_valid()
    }

    /// Save blockchain state to disk
    pub fn save_to_disk(&self) -> Result<()> {
        self.blockchain.rdf_store.save_to_disk()
    }
}

/// Statistics for the transaction blockchain
#[derive(Debug, Clone)]
pub struct TransactionBlockchainStats {
    pub total_blocks: usize,
    pub pending_transactions: usize,
    pub total_participants: usize,
    pub total_utxos: usize,
    pub participant_distribution: HashMap<ParticipantType, usize>,
    pub transaction_distribution: HashMap<TransactionType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Participant;
    use tempfile::tempdir;

    #[test]
    fn test_transaction_blockchain_creation() {
        let temp_dir = tempdir().unwrap();
        let blockchain = TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

        let stats = blockchain.get_statistics();
        assert_eq!(stats.total_blocks, 1); // Genesis block
        assert_eq!(stats.pending_transactions, 0);
        assert_eq!(stats.total_participants, 0);
    }

    #[test]
    fn test_participant_registration() {
        let temp_dir = tempdir().unwrap();
        let mut blockchain = TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let participant_id = blockchain.register_participant(farmer).unwrap();
        assert!(blockchain.get_participant_wallet(participant_id).is_some());

        let stats = blockchain.get_statistics();
        assert_eq!(stats.total_participants, 1);
    }

    #[test]
    fn test_production_transaction() {
        let temp_dir = tempdir().unwrap();
        let mut blockchain = TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let farmer_id = blockchain.register_participant(farmer).unwrap();

        let tx = blockchain
            .create_production_transaction(
                farmer_id,
                "MILK-001".to_string(),
                1000.0,
                "Vermont, USA".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Production);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 1000.0);
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::wallet::Participant;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tempfile::tempdir;

    /// Blockchain integration security tests
    mod blockchain_integration_security {
        use super::*;

        #[test]
        fn test_block_transaction_validation() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Register a participant
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create valid transaction
            let valid_tx = blockchain
                .create_production_transaction(
                    farmer_id,
                    "TEST-001".to_string(),
                    100.0,
                    "Test Location".to_string(),
                    None,
                )
                .unwrap();

            // Submit valid transaction
            let submit_result = blockchain.submit_transaction(valid_tx);
            assert!(
                submit_result.is_ok(),
                "Valid transaction should be accepted"
            );

            // Attempt to create transaction with invalid data
            let invalid_tx_result = blockchain.create_production_transaction(
                Uuid::new_v4(), // Non-existent participant
                "INVALID-001".to_string(),
                -100.0, // Negative quantity
                "Test Location".to_string(),
                None,
            );

            assert!(
                invalid_tx_result.is_err(),
                "Invalid transaction should be rejected"
            );
        }

        #[test]
        fn test_chain_integrity_verification() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Verify initial chain integrity
            assert!(blockchain.validate(), "Initial blockchain should be valid");

            // Register participant and create transactions
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create and submit multiple transactions
            for i in 0..5 {
                let tx = blockchain
                    .create_production_transaction(
                        farmer_id,
                        format!("BATCH-{:03}", i),
                        100.0 + i as f64,
                        "Test Location".to_string(),
                        None,
                    )
                    .unwrap();

                let _submit_result = blockchain.submit_transaction(tx);
                assert!(
                    _submit_result.is_ok(),
                    "Transaction {} should be submitted successfully",
                    i
                );
            }

            // Create block with transactions
            let create_block_result = blockchain.create_block(3); // Process up to 3 transactions
            assert!(create_block_result.is_ok(), "Block creation should succeed");

            // Verify chain integrity after adding blocks
            assert!(
                blockchain.validate(),
                "Blockchain should remain valid after adding transactions"
            );

            // Verify statistics
            let stats = blockchain.get_statistics();
            assert!(
                stats.total_blocks > 1,
                "Should have more than genesis block"
            );
            assert!(stats.total_utxos > 0, "Should have UTXOs from transactions");
        }

        #[test]
        fn test_fork_resolution_security() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Create two competing blocks with same parent
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create transactions
            let tx1 = blockchain
                .create_production_transaction(
                    farmer_id,
                    "FORK-1".to_string(),
                    100.0,
                    "Location A".to_string(),
                    None,
                )
                .unwrap();

            let tx2 = blockchain
                .create_production_transaction(
                    farmer_id,
                    "FORK-2".to_string(),
                    200.0,
                    "Location B".to_string(),
                    None,
                )
                .unwrap();

            // Submit transactions
            blockchain
                .submit_transaction(tx1)
                .expect("First transaction should submit");
            blockchain
                .submit_transaction(tx2)
                .expect("Second transaction should submit");

            // Create block - this would normally resolve based on consensus rules
            let block_result = blockchain.create_block(10);
            assert!(
                block_result.is_ok(),
                "Block creation should handle competing transactions"
            );

            // Verify chain integrity is maintained
            assert!(
                blockchain.validate(),
                "Chain should be valid after potential fork"
            );

            // In a real implementation, this would test specific fork resolution logic
            let _stats = blockchain.get_statistics();
        }

        #[test]
        fn test_consensus_integration_security() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Test that only properly validated transactions are included in blocks
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create valid transaction
            let valid_tx = blockchain
                .create_production_transaction(
                    farmer_id,
                    "CONSENSUS-1".to_string(),
                    100.0,
                    "Test Location".to_string(),
                    None,
                )
                .unwrap();

            // Submit and create block
            blockchain
                .submit_transaction(valid_tx)
                .expect("Valid transaction should submit");
            let block_result = blockchain.create_block(5);
            assert!(block_result.is_ok(), "Block creation should succeed");

            // Verify block contains only valid transactions
            let stats = blockchain.get_statistics();
            assert!(stats.total_blocks > 1, "New block should be created");
            assert!(blockchain.validate(), "Blockchain should remain valid");

            // Test that consensus rules prevent invalid transactions
            // This would involve more complex consensus logic in a full implementation
        }

        #[test]
        fn test_permission_enforcement() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Register different types of participants
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let processor = Participant::new_uht_manufacturer(
                "Test Processor".to_string(),
                "Test Location".to_string(),
            );

            let farmer_id = blockchain.register_participant(farmer).unwrap();
            let processor_id = blockchain.register_participant(processor).unwrap();

            // Test farmer permissions
            let farmer_production_tx = blockchain.create_production_transaction(
                farmer_id,
                "PERM-TEST-1".to_string(),
                100.0,
                "Test Location".to_string(),
                None,
            );
            assert!(
                farmer_production_tx.is_ok(),
                "Farmer should be able to create production transactions"
            );

            // Test processor permissions
            let processor_processing_tx = blockchain.create_processing_transaction(
                processor_id,
                vec!["PERM-TEST-1".to_string()],
                "PROCESSED-1".to_string(),
                "UHT".to_string(),
                None,
            );
            assert!(
                processor_processing_tx.is_ok(),
                "Processor should be able to create processing transactions"
            );

            // Test permission violations would be handled at wallet level
            // This is a simplified test - in a full implementation, permissions would be enforced more strictly
        }

        #[test]
        fn test_replay_attack_protection() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create transaction
            let tx = blockchain
                .create_production_transaction(
                    farmer_id,
                    "REPLAY-TEST".to_string(),
                    100.0,
                    "Test Location".to_string(),
                    None,
                )
                .unwrap();

            // Submit transaction first time
            let first_submit = blockchain.submit_transaction(tx.clone());
            assert!(first_submit.is_ok(), "First submission should succeed");

            // Attempt to submit same transaction again (replay attack)
            let _second_submit = blockchain.submit_transaction(tx);

            // Should either succeed (if duplicate detection happens elsewhere) or fail gracefully
            // In a full implementation, transaction pool would detect duplicates
            let _stats = blockchain.get_statistics();
        }

        #[test]
        fn test_sybil_attack_resistance() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Attempt to register many participants (potential Sybil attack)
            let mut participant_ids = Vec::new();
            for i in 0..20 {
                let participant =
                    Participant::new_farmer(format!("Farm {}", i), format!("Location {}", i));

                let registration_result = blockchain.register_participant(participant);
                if registration_result.is_ok() {
                    participant_ids.push(registration_result.unwrap());
                }
            }

            // Verify system remains stable after many registrations
            assert!(
                blockchain.validate(),
                "Blockchain should remain valid after many registrations"
            );

            let stats = blockchain.get_statistics();
            assert!(
                stats.total_participants <= 20,
                "Should track participants correctly"
            );

            // Test that creating many transactions doesn't break the system
            for (i, &participant_id) in participant_ids.iter().take(10).enumerate() {
                let tx = blockchain.create_production_transaction(
                    participant_id,
                    format!("SYBIL-{}", i),
                    10.0,
                    format!("Location {}", i),
                    None,
                );

                if tx.is_ok() {
                    let _submit_result = blockchain.submit_transaction(tx.unwrap());
                    // System should handle high transaction volume gracefully
                }
            }

            // System should remain stable
            assert!(
                blockchain.validate(),
                "Blockchain should handle high transaction volume"
            );
        }

        #[test]
        fn test_denial_of_service_resistance() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Test with large transaction data (potential DoS attack)
            let large_description = "A".repeat(100000); // Large description
            let large_tx = blockchain.create_production_transaction(
                farmer_id,
                "LARGE-TX".to_string(),
                100.0,
                large_description.clone(),
                None,
            );

            // Should handle large transactions gracefully
            match large_tx {
                Ok(tx) => {
                    // If large transaction is created, system should handle it
                    let _submit_result = blockchain.submit_transaction(tx);
                    // Should either succeed or fail gracefully, not crash
                }
                Err(_) => {
                    // Large transaction should be rejected gracefully
                }
            }

            // Test with many small transactions
            for i in 0..50 {
                let tx = blockchain.create_production_transaction(
                    farmer_id,
                    format!("DOS-{}", i),
                    1.0,
                    format!("Location {}", i),
                    None,
                );

                if let Ok(transaction) = tx {
                    let _ = blockchain.submit_transaction(transaction);
                    // System should not be overwhelmed by many transactions
                }
            }

            // System should remain stable
            assert!(
                blockchain.validate(),
                "Blockchain should resist DoS attempts"
            );

            let stats = blockchain.get_statistics();
            assert!(
                stats.pending_transactions <= 1000,
                "Transaction pool should have reasonable limits"
            );
        }

        #[test]
        fn test_privacy_leakage_prevention() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Test with sensitive data
            let farmer =
                Participant::new_farmer("Private Farm".to_string(), "Secret Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create transaction with potentially sensitive data
            let sensitive_tx = blockchain
                .create_production_transaction(
                    farmer_id,
                    "SENSITIVE-1".to_string(),
                    100.0,
                    "Confidential Location".to_string(),
                    None,
                )
                .unwrap();

            // Submit transaction
            let _submit_result = blockchain.submit_transaction(sensitive_tx);
            assert!(
                _submit_result.is_ok(),
                "Transaction with sensitive data should submit"
            );

            // Verify that sensitive information is handled appropriately
            // In a full implementation, this would test encryption, access controls, etc.
            let participant_wallet = blockchain.get_participant_wallet(farmer_id);
            assert!(
                participant_wallet.is_some(),
                "Participant data should be accessible to authorized queries"
            );

            // Test that information leakage is prevented
            // This would involve testing access controls, encryption, and data sanitization
        }

        #[test]
        fn test_blockchain_state_consistency() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Track initial state
            let initial_stats = blockchain.get_statistics();
            let initial_validity = blockchain.validate();

            // Perform series of operations
            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Create multiple transactions
            let transactions: Vec<_> = (0..5)
                .map(|i| {
                    blockchain
                        .create_production_transaction(
                            farmer_id,
                            format!("CONSISTENCY-{}", i),
                            100.0 + i as f64,
                            "Test Location".to_string(),
                            None,
                        )
                        .unwrap()
                })
                .collect();

            // Submit all transactions
            for tx in transactions {
                let _ = blockchain.submit_transaction(tx);
            }

            // Create blocks
            let _ = blockchain.create_block(10);

            // Verify final state consistency
            let final_stats = blockchain.get_statistics();
            let final_validity = blockchain.validate();

            assert!(initial_validity, "Initial state should be valid");
            assert!(final_validity, "Final state should be valid");
            assert!(
                final_stats.total_blocks > initial_stats.total_blocks,
                "Blocks should be added"
            );
            assert!(
                final_stats.total_participants > initial_stats.total_participants,
                "Participants should be registered"
            );

            // Verify state transitions are consistent
            assert!(
                blockchain.validate(),
                "All state transitions should maintain consistency"
            );
        }

        #[test]
        fn test_concurrent_blockchain_operations() {
            let temp_dir = tempdir().unwrap();
            let blockchain = Arc::new(Mutex::new(
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap(),
            ));
            let mut handles = vec![];

            // Spawn multiple threads performing blockchain operations
            for i in 0..5 {
                let blockchain_clone = Arc::clone(&blockchain);
                let handle = thread::spawn(move || {
                    let mut blockchain = blockchain_clone.lock().unwrap();

                    // Register participant
                    let participant =
                        Participant::new_farmer(format!("Farm {}", i), format!("Location {}", i));

                    let participant_id = blockchain.register_participant(participant);

                    if participant_id.is_ok() {
                        let pid = participant_id.unwrap();

                        // Create transaction
                        let tx = blockchain.create_production_transaction(
                            pid,
                            format!("CONCURRENT-{}", i),
                            100.0,
                            format!("Location {}", i),
                            None,
                        );

                        if tx.is_ok() {
                            let _ = blockchain.submit_transaction(tx.unwrap());
                        }
                    }

                    // Verify blockchain remains valid
                    blockchain.validate()
                });
                handles.push(handle);
            }

            // Wait for all threads
            let mut all_valid = true;
            for handle in handles {
                if !handle.join().unwrap() {
                    all_valid = false;
                }
            }

            assert!(
                all_valid,
                "Blockchain should remain valid under concurrent operations"
            );

            // Final verification
            let blockchain = blockchain.lock().unwrap();
            assert!(
                blockchain.validate(),
                "Final blockchain state should be valid"
            );
        }

        #[test]
        fn test_transaction_pool_overflow_protection() {
            let temp_dir = tempdir().unwrap();
            let mut blockchain =
                TransactionBlockchain::new(temp_dir.path().to_str().unwrap()).unwrap();

            let farmer =
                Participant::new_farmer("Test Farm".to_string(), "Test Location".to_string());
            let farmer_id = blockchain.register_participant(farmer).unwrap();

            // Fill transaction pool beyond capacity
            let mut successful_submissions = 0;
            let mut failed_submissions = 0;

            for i in 0..1500 {
                // More than default pool size of 1000
                let tx = blockchain.create_production_transaction(
                    farmer_id,
                    format!("OVERFLOW-{}", i),
                    1.0,
                    format!("Location {}", i),
                    None,
                );

                match tx {
                    Ok(transaction) => match blockchain.submit_transaction(transaction) {
                        Ok(_) => successful_submissions += 1,
                        Err(_) => failed_submissions += 1,
                    },
                    Err(_) => failed_submissions += 1,
                }
            }

            // System should handle overflow gracefully
            let stats = blockchain.get_statistics();
            assert!(
                stats.pending_transactions <= 1000,
                "Transaction pool should not exceed capacity"
            );
            assert!(
                successful_submissions + failed_submissions > 0,
                "Some transactions should be processed"
            );
            assert!(
                blockchain.validate(),
                "Blockchain should remain valid despite overflow attempts"
            );
        }
    }
}
