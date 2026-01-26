use provchain_org::blockchain::Blockchain;
use std::fs;

#[test]
fn test_blockchain_with_minimal_test_data() {
    let mut blockchain = Blockchain::new();
    
    // Read and add minimal test data to blockchain
    let turtle_data = fs::read_to_string("tests/data/minimal_test_data.ttl")
        .expect("Failed to read minimal_test_data.ttl");
    
    let _ = blockchain.add_block(turtle_data);
    
    // Verify blockchain is valid
    assert!(blockchain.is_valid(), "Blockchain should be valid after adding minimal test data");
    assert_eq!(blockchain.chain.len(), 2, "Should have genesis block + 1 data block");
}

#[test]
fn test_blockchain_with_complete_supply_chain_data() {
    let mut blockchain = Blockchain::new();
    
    // Read and add complete supply chain test data to blockchain
    let turtle_data = fs::read_to_string("tests/data/complete_supply_chain_test.ttl")
        .expect("Failed to read complete_supply_chain_test.ttl");
    
    let _ = blockchain.add_block(turtle_data);
    
    // Verify blockchain is valid
    assert!(blockchain.is_valid(), "Blockchain should be valid after adding supply chain data");
    assert_eq!(blockchain.chain.len(), 2, "Should have genesis block + 1 data block");
    
    // Query the RDF data in the blockchain
    let query = r#"PREFIX trace: <http://example.org/trace#>
        SELECT ?batch ?batchId
        FROM <http://provchain.org/block/1>
        WHERE {
            ?batch a trace:ProductBatch ;
                   trace:hasBatchID ?batchId .
        }
    "#;
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = blockchain.rdf_store.query(query) {
        let results: Vec<_> = solutions.collect();
        assert!(!results.is_empty(), "Should find product batches in the blockchain");
    } else {
        panic!("SPARQL query failed");
    }
}

#[test]
fn test_blockchain_with_both_test_files() {
    let mut blockchain = Blockchain::new();
    
    // Add minimal test data first
    let minimal_data = fs::read_to_string("tests/data/minimal_test_data.ttl")
        .expect("Failed to read minimal_test_data.ttl");
    let _ = blockchain.add_block(minimal_data);
    
    // Add complete supply chain data second
    let supply_chain_data = fs::read_to_string("tests/data/complete_supply_chain_test.ttl")
        .expect("Failed to read complete_supply_chain_test.ttl");
    let _ = blockchain.add_block(supply_chain_data);
    
    // Verify blockchain is valid
    assert!(blockchain.is_valid(), "Blockchain should be valid with both test data files");
    assert_eq!(blockchain.chain.len(), 3, "Should have genesis block + 2 data blocks");
    
    // Query data from first block (minimal test data)
    let query1 = r#"PREFIX pc: <http://provchain.org/>
        SELECT ?value
        FROM <http://provchain.org/block/1>
        WHERE {
            pc:testData pc:hasValue ?value .
        }
    "#;
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = blockchain.rdf_store.query(query1) {
        let results: Vec<_> = solutions.collect();
        assert_eq!(results.len(), 1, "Should find test data in block 1");
    } else {
        panic!("SPARQL query for block 1 failed");
    }
    
    // Query data from second block (supply chain data)
    let query2 = r#"PREFIX trace: <http://example.org/trace#>
        SELECT ?batch
        FROM <http://provchain.org/block/2>
        WHERE {
            ?batch a trace:ProductBatch .
        }
    "#;
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = blockchain.rdf_store.query(query2) {
        let results: Vec<_> = solutions.collect();
        assert!(!results.is_empty(), "Should find product batches in block 2");
    } else {
        panic!("SPARQL query for block 2 failed");
    }
}
