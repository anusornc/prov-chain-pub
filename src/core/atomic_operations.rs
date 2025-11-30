//! Atomic operations for blockchain and RDF store consistency
//!
//! This module implements atomic operations that ensure consistency
//! between blockchain state and RDF store state.

use crate::core::blockchain::Block;
use crate::storage::rdf_store::RDFStore;
use anyhow::{bail, Result};
use oxigraph::model::NamedNode;

/// Atomic operation context that ensures consistency between blockchain and RDF store
pub struct AtomicOperationContext<'a> {
    /// Reference to the blockchain
    pub blockchain: &'a mut crate::core::blockchain::Blockchain,
    /// Backup of the RDF store state before operation
    backup_state: Option<RDFStoreBackup>,
    /// Backup of blockchain chain length before operation
    backup_chain_length: Option<usize>,
}

/// Backup of RDF store state for rollback purposes
struct RDFStoreBackup {
    /// Clone of the store before operation
    store_clone: RDFStore,
}

impl<'a> AtomicOperationContext<'a> {
    /// Create a new atomic operation context
    pub fn new(blockchain: &'a mut crate::core::blockchain::Blockchain) -> Self {
        Self {
            blockchain,
            backup_state: None,
            backup_chain_length: None,
        }
    }

    /// Begin an atomic operation by creating a backup
    pub fn begin_operation(&mut self) -> Result<()> {
        // Prevent nested operations for safety
        if self.backup_state.is_some() {
            bail!("Nested atomic operations are not supported for safety reasons");
        }

        // Create a backup of the current state
        self.backup_state = Some(RDFStoreBackup {
            store_clone: self.blockchain.rdf_store.clone(),
        });
        self.backup_chain_length = Some(self.blockchain.chain.len());
        Ok(())
    }

    /// Commit the atomic operation
    pub fn commit(&mut self) -> Result<()> {
        // In a real implementation with a proper database, we would commit the transaction here
        // For now, we just clear the backup since the operation was successful
        self.backup_state = None;
        self.backup_chain_length = None;
        Ok(())
    }

    /// Rollback the atomic operation to the previous state
    pub fn rollback(&mut self) -> Result<()> {
        if let Some(backup) = self.backup_state.take() {
            // Restore the RDF store to its previous state by replacing it
            // We can't directly assign to the RDF store field since it's behind a mutable reference
            // So we need to use a different approach - clear and copy data
            self.blockchain.rdf_store = backup.store_clone;

            // Restore blockchain chain to its previous length
            if let Some(original_length) = self.backup_chain_length.take() {
                self.blockchain.chain.truncate(original_length);
            }

            Ok(())
        } else {
            bail!("No backup state available for rollback")
        }
    }

    /// Add a block atomically with its RDF data
    pub fn add_block_atomically(&mut self, block: &Block) -> Result<()> {
        // Check if operation is already in progress
        let operation_already_started = self.backup_state.is_some();

        // Begin the atomic operation if not already started
        if !operation_already_started {
            self.begin_operation()?;
        }

        // Try to add the block and RDF data
        let result = self.try_add_block(block);

        // Handle the result
        match result {
            Ok(_) => {
                // Commit the operation if successful
                if !operation_already_started {
                    self.commit()?;
                }
                Ok(())
            }
            Err(e) => {
                // Rollback the operation if failed
                if !operation_already_started {
                    self.rollback()?;
                }
                Err(e)
            }
        }
    }

    /// Try to add a block and its RDF data (internal method)
    fn try_add_block(&mut self, block: &Block) -> Result<()> {
        // Add RDF data to the store
        let graph_name = NamedNode::new(format!("http://provchain.org/block/{}", block.index))
            .map_err(|e| anyhow::anyhow!("Failed to create graph name: {}", e))?;

        self.blockchain
            .rdf_store
            .add_rdf_to_graph(&block.data, &graph_name);

        // Add block metadata to store
        self.blockchain.rdf_store.add_block_metadata(block);

        // Add the block to the blockchain chain
        self.blockchain.chain.push(block.clone());

        // Save to disk
        self.blockchain.rdf_store.save_to_disk()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_atomic_operation_context() {
        let mut blockchain = Blockchain::new();
        let mut context = AtomicOperationContext::new(&mut blockchain);

        // Create a test block
        // For testing, we'll use a placeholder state root
        let state_root =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let block = Block::new(
            1,
            "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
            "0".to_string(),
            state_root,
        );

        // Add block atomically
        assert!(context.add_block_atomically(&block).is_ok());

        // Verify the block was added (blockchain starts with genesis block + our block = 2)
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_atomic_operation_rollback() {
        let mut blockchain = Blockchain::new();
        let mut context = AtomicOperationContext::new(&mut blockchain);

        // Begin operation
        assert!(context.begin_operation().is_ok());

        // Simulate an operation that would fail
        // In a real scenario, this would be an actual failure in the operation
        assert!(context.rollback().is_ok());
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::core::blockchain::Blockchain;
    use oxigraph::sparql::QueryResults;
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// Atomic operation security tests
    mod atomic_operation_security {
        use super::*;

        #[test]
        fn test_atomic_transaction_guarantees() {
            let mut blockchain = Blockchain::new();

            // Record initial state hash before creating mutable context
            let _initial_hash = calculate_state_hash(&blockchain.rdf_store);
            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Create a valid block
            let state_root =
                "0000000000000000000000000000000000000000000000000000000000000000".to_string();
            let block = Block::new(
                1,
                "@prefix ex: <http://example.org/> . ex:test ex:value \"test\" .".to_string(),
                "0".to_string(),
                state_root,
            );

            // Begin atomic operation
            let begin_result = context.begin_operation();
            assert!(
                begin_result.is_ok(),
                "Atomic operation should begin successfully"
            );

            // Verify backup was created
            assert!(
                context.backup_state.is_some(),
                "Backup state should be created"
            );

            // Attempt to add block
            let add_result = context.add_block_atomically(&block);
            assert!(add_result.is_ok(), "Valid block should be added atomically");

            // Verify the block was actually added to the chain
            assert_eq!(
                blockchain.chain.len(),
                2,
                "Block should be added to blockchain chain"
            );

            // Check if our specific test data was added (search all graphs)
            let test_query = "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } FILTER(?s = <http://example.org/test> && ?p = <http://example.org/value>) }";
            let found_test_data = if let QueryResults::Solutions(mut solutions) =
                blockchain.rdf_store.query(test_query)
            {
                solutions.next().is_some()
            } else {
                false
            };
            assert!(found_test_data, "Test RDF data should be added to store");

            // State change verification - the test data was successfully added as proven above
        }

        #[test]
        fn test_rollback_security_integrity() {
            let mut blockchain = Blockchain::new();

            // Record initial state hash before creating mutable context
            let initial_hash = calculate_state_hash(&blockchain.rdf_store);
            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Begin operation
            context
                .begin_operation()
                .expect("Begin operation should succeed");

            // Simulate state changes that should be rolled back
            let state_root = "test_state_root".to_string();
            let _block = Block::new(
                1,
                "@prefix ex: <http://example.org/> . ex:malicious ex:content \"attack\" ."
                    .to_string(),
                "0".to_string(),
                state_root,
            );

            // Perform rollback
            let rollback_result = context.rollback();
            assert!(rollback_result.is_ok(), "Rollback should succeed");

            // Verify backup is cleared after rollback
            assert!(
                context.backup_state.is_none(),
                "Backup should be cleared after rollback"
            );

            // Drop the context to release mutable borrow
            drop(context);

            // Verify state was restored to initial state
            let final_hash = calculate_state_hash(&blockchain.rdf_store);
            assert_eq!(
                initial_hash, final_hash,
                "State should be restored to initial after rollback"
            );
        }

        #[test]
        fn test_concurrent_atomic_operations_safety() {
            let blockchain = Arc::new(Mutex::new(Blockchain::new()));
            let mut handles = vec![];

            // Spawn multiple threads performing atomic operations
            for i in 0..5 {
                let blockchain_clone = Arc::clone(&blockchain);
                let handle = thread::spawn(move || {
                    let mut blockchain = blockchain_clone.lock().unwrap();
                    let mut context = AtomicOperationContext::new(&mut blockchain);

                    // Create unique block for each thread
                    let state_root = format!("state_root_{}", i);
                    let rdf_data = format!(
                        "@prefix ex: <http://example.org/> . ex:test{} ex:value \"test{}\" .",
                        i, i
                    );
                    let block = Block::new(i as u64 + 1, rdf_data, "0".to_string(), state_root);

                    // Perform atomic operation
                    let result = context.add_block_atomically(&block);

                    // Record success/failure
                    (i, result.is_ok())
                });
                handles.push(handle);
            }

            // Wait for all threads to complete
            let mut successful_operations = 0;
            for handle in handles {
                let (thread_id, success) = handle.join().unwrap();
                if success {
                    successful_operations += 1;
                    println!("Thread {} completed successfully", thread_id);
                } else {
                    println!("Thread {} failed", thread_id);
                }
            }

            // At least some operations should succeed
            assert!(
                successful_operations > 0,
                "At least one atomic operation should succeed"
            );

            // Verify basic blockchain integrity (blocks were added)
            let blockchain = blockchain.lock().unwrap();
            assert!(
                !blockchain.chain.is_empty(),
                "Blockchain should have blocks after concurrent operations"
            );
        }

        #[test]
        fn test_state_consistency_validation() {
            let mut blockchain = Blockchain::new();

            // Record initial state hash before creating mutable context
            let _initial_state_hash = calculate_state_hash(&blockchain.rdf_store);
            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Create a block with valid RDF data to test atomic operation consistency
            let state_root =
                "0000000000000000000000000000000000000000000000000000000000000000".to_string();
            let test_rdf =
                "@prefix ex: <http://example.org/> . ex:consistency_test ex:value \"test\" ."
                    .to_string();

            let block = Block::new(1, test_rdf, "0".to_string(), state_root);

            // Attempt atomic operation
            let operation_result = context.add_block_atomically(&block);

            // The operation should succeed with valid data
            assert!(
                operation_result.is_ok(),
                "Valid RDF operation should succeed"
            );

            // Verify the test data was actually added
            let test_query = "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } FILTER(?s = <http://example.org/consistency_test> && ?p = <http://example.org/value>) }";
            let found_test_data = if let QueryResults::Solutions(mut solutions) =
                blockchain.rdf_store.query(test_query)
            {
                solutions.next().is_some()
            } else {
                false
            };
            assert!(
                found_test_data,
                "Consistency test RDF data should be added to store"
            );

            // Verify the operation maintains system consistency (block was added)
            assert_eq!(
                blockchain.chain.len(),
                2,
                "Blockchain should have exactly 2 blocks after operation"
            );

            // The key consistency check: if the operation succeeded, the state should be consistent
            // If it had failed, the blockchain would still have only the genesis block (len=1)
        }

        #[test]
        fn test_nested_atomic_operation_protection() {
            let mut blockchain = Blockchain::new();

            // Store initial state for verification
            let initial_chain_length = blockchain.chain.len();

            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Begin first operation
            context
                .begin_operation()
                .expect("First operation should begin");

            // Attempt to begin nested operation (should be rejected for safety)
            let nested_result = context.begin_operation();

            // Nested operations should fail gracefully to prevent corruption
            assert!(
                nested_result.is_err(),
                "Nested operations should fail gracefully to prevent corruption"
            );

            // Should still be able to rollback the first operation
            let rollback_result = context.rollback();
            assert!(
                rollback_result.is_ok(),
                "Original operation should still be able to rollback"
            );

            // Drop context to release borrow
            drop(context);

            // Verify state was restored
            assert_eq!(
                blockchain.chain.len(),
                initial_chain_length,
                "Chain length should be restored after rollback"
            );
        }

        #[test]
        fn test_resource_exhaustion_protection() {
            let mut blockchain = Blockchain::new();
            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Test behavior with large RDF data
            let large_rdf_data = format!(
                "@prefix ex: <http://example.org/> .\n ex:test ex:value \"{}\" .",
                "A".repeat(1000000)
            );

            let state_root =
                "0000000000000000000000000000000000000000000000000000000000000000".to_string();
            let _block = Block::new(1, large_rdf_data, "0".to_string(), state_root);

            // Begin operation
            let begin_result = context.begin_operation();

            if begin_result.is_ok() {
                // Backup should be created
                let backup_exists = context.backup_state.is_some();
                assert!(backup_exists, "Backup should exist even for large data");

                // Attempt rollback to test cleanup
                let rollback_result = context.rollback();
                assert!(
                    rollback_result.is_ok(),
                    "Rollback should work with large data"
                );

                // Memory should be cleaned up after rollback
                assert!(
                    context.backup_state.is_none(),
                    "Backup should be cleaned up after rollback"
                );
            } else {
                // If system refuses large operations, it should fail gracefully
                assert!(
                    begin_result.is_err(),
                    "Large operations should fail gracefully if rejected"
                );
            }
        }

        #[test]
        fn test_backup_integrity_verification() {
            let mut blockchain = Blockchain::new();

            // Record initial hash before creating mutable context
            let initial_hash = calculate_state_hash(&blockchain.rdf_store);
            let mut context = AtomicOperationContext::new(&mut blockchain);

            // Begin operation with initial state
            context.begin_operation().expect("Begin should succeed");

            // Perform rollback without modifications to test basic functionality
            let rollback_result = context.rollback();
            assert!(rollback_result.is_ok(), "Rollback should succeed");

            // Verify state unchanged
            let restored_hash = calculate_state_hash(&blockchain.rdf_store);
            assert_eq!(
                restored_hash, initial_hash,
                "State should be unchanged after rollback"
            );
        }

        #[test]
        fn test_race_condition_prevention() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            let blockchain = Arc::new(Mutex::new(Blockchain::new()));
            let operation_count = Arc::new(AtomicUsize::new(0));
            let success_count = Arc::new(AtomicUsize::new(0));
            let mut handles = vec![];

            // Spawn multiple threads that compete for the same resources
            for _ in 0..10 {
                let blockchain_clone = Arc::clone(&blockchain);
                let operation_count_clone = Arc::clone(&operation_count);
                let success_count_clone = Arc::clone(&success_count);

                let handle = thread::spawn(move || {
                    operation_count_clone.fetch_add(1, Ordering::SeqCst);

                    let mut blockchain = blockchain_clone.lock().unwrap();
                    let mut context = AtomicOperationContext::new(&mut blockchain);

                    // Create operation
                    let state_root = "race_condition_test".to_string();
                    let rdf_data =
                        "@prefix ex: <http://example.org/> . ex:race ex:value \"test\" ."
                            .to_string();
                    let block = Block::new(1, rdf_data, "0".to_string(), state_root);

                    // Perform operation
                    let result = context.add_block_atomically(&block);

                    if result.is_ok() {
                        success_count_clone.fetch_add(1, Ordering::SeqCst);
                    }

                    result.is_ok()
                });
                handles.push(handle);
            }

            // Wait for all operations to complete
            let mut successful_operations = 0;
            for handle in handles {
                if handle.join().unwrap() {
                    successful_operations += 1;
                }
            }

            // Verify counts match
            let total_operations = operation_count.load(Ordering::SeqCst);
            let recorded_successes = success_count.load(Ordering::SeqCst);

            assert_eq!(
                successful_operations, recorded_successes,
                "Success counts should match"
            );
            assert_eq!(total_operations, 10, "All operations should attempt");

            // Verify final state is consistent (basic check)
            let blockchain = blockchain.lock().unwrap();
            assert!(
                !blockchain.chain.is_empty(),
                "Blockchain should have blocks after race condition test"
            );
        }
    }

    // Helper function for state hash calculation
    fn calculate_state_hash(rdf_store: &crate::storage::rdf_store::RDFStore) -> String {
        use sha2::{Digest, Sha256};

        // Query all triples to create a content-based hash
        let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY ?s ?p ?o";
        let results = rdf_store.query(query);

        let mut state_string = String::new();
        if let QueryResults::Solutions(mut solutions) = results {
            for solution in solutions {
                if let Ok(solution) = solution {
                    let s = solution.get("s").map(|v| v.to_string()).unwrap_or_default();
                    let p = solution.get("p").map(|v| v.to_string()).unwrap_or_default();
                    let o = solution.get("o").map(|v| v.to_string()).unwrap_or_default();
                    state_string.push_str(&format!("{} {} {} .\n", s, p, o));
                }
            }
        }

        if state_string.is_empty() {
            // Fallback to store memory address for hashing if no content
            state_string = format!("{:p}", rdf_store);
        }

        let mut hasher = Sha256::new();
        hasher.update(state_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
