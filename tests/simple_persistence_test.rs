use provchain_org::core::blockchain::Blockchain;
use tempfile::tempdir;

#[test]
fn test_simple_blockchain_persistence() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("simple_test");

    // Create a persistent blockchain
    let mut blockchain = Blockchain::new_persistent(&data_dir).unwrap();

    // Add a simple block
    let test_data = r#"
        @prefix ex: <http://example.org/> .
        ex:product ex:name "Test Product" .
    "#
    .to_string();

    let _ = blockchain.add_block(test_data);

    // Verify blockchain has the expected number of blocks
    assert_eq!(blockchain.chain.len(), 2); // Genesis + 1 added block

    // Create a new blockchain instance to test loading from disk
    let blockchain2 = Blockchain::new_persistent(&data_dir).unwrap();

    // Verify all blocks were loaded correctly (this was the bug that is now fixed)
    assert_eq!(
        blockchain2.chain.len(), 
        2, 
        "Expected 2 blocks (genesis + 1 added), but found {}. Persistence may not be working correctly.",
        blockchain2.chain.len()
    );

    // Verify block hashes match
    for i in 0..2 {
        assert_eq!(
            blockchain.chain[i].hash, blockchain2.chain[i].hash,
            "Block {} hash mismatch - data corruption detected",
            i
        );
    }

    // Verify block data matches
    assert_eq!(
        blockchain.chain[1].data, blockchain2.chain[1].data,
        "Block data mismatch after persistence"
    );
}

#[test]
fn test_persistence_multiple_blocks() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("multi_block_test");

    // Create and populate blockchain
    {
        let mut blockchain = Blockchain::new_persistent(&data_dir).unwrap();

        // Add multiple blocks
        for i in 1..=5 {
            let test_data = format!(
                r#"
                @prefix ex: <http://example.org/> .
                ex:product{} ex:name "Test Product {}" .
                ex:product{} ex:batch "BATCH{:03}" .
                "#,
                i, i, i, i
            );
            let _ = blockchain.add_block(test_data);
        }

        // Verify 6 blocks (genesis + 5 added)
        assert_eq!(blockchain.chain.len(), 6);
    }

    // Re-open and verify all blocks are loaded
    {
        let blockchain = Blockchain::new_persistent(&data_dir).unwrap();
        assert_eq!(
            blockchain.chain.len(),
            6,
            "All 6 blocks should be loaded from persistent storage"
        );

        // Verify chain integrity
        assert!(
            blockchain.is_valid(),
            "Blockchain should be valid after loading from disk"
        );

        // Verify each block's index
        for (i, block) in blockchain.chain.iter().enumerate() {
            assert_eq!(
                block.index, i as u64,
                "Block at position {} should have index {}",
                i, i
            );
        }
    }
}

#[test]
fn test_persistence_crash_recovery() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("crash_recovery_test");

    // Simulate adding blocks
    let block_hashes: Vec<String> = {
        let mut blockchain = Blockchain::new_persistent(&data_dir).unwrap();
        let mut hashes = Vec::new();

        for i in 1..=3 {
            let test_data = format!(
                r#"@prefix ex: <http://example.org/> . ex:item{} ex:id {} ."#,
                i, i
            );
            let _ = blockchain.add_block(test_data);
            hashes.push(blockchain.chain.last().unwrap().hash.clone());
        }
        hashes
    };

    // Simulate crash recovery by opening a new instance
    let blockchain = Blockchain::new_persistent(&data_dir).unwrap();

    // Verify all blocks recovered
    assert_eq!(
        blockchain.chain.len(),
        4,
        "Should recover genesis + 3 blocks"
    );

    // Verify hashes match
    for (i, expected_hash) in block_hashes.iter().enumerate() {
        let block_index = i + 1; // Skip genesis
        assert_eq!(
            &blockchain.chain[block_index].hash, expected_hash,
            "Block {} hash should match after recovery",
            block_index
        );
    }
}
