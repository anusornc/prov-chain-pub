use oxigraph::model::NamedNode;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::RDFStore;

#[test]
fn test_rdf_canonicalization_with_blank_nodes() {
    let mut rdf_store = RDFStore::new();

    // Add RDF data with blank nodes to test canonicalization
    let rdf_data = r#"
        @prefix ex: <http://example.org/> .
        _:b1 ex:name "Alice" .
        _:b1 ex:age 30 .
        _:b2 ex:name "Bob" .
        _:b2 ex:knows _:b1 .
    "#;

    let graph_name = NamedNode::new("http://example.org/test").unwrap();
    rdf_store.add_rdf_to_graph(rdf_data, &graph_name);

    // Get the canonical hash
    let hash1 = rdf_store.canonicalize_graph(&graph_name);

    // Add the same data again with different blank node identifiers
    let rdf_data2 = r#"
        @prefix ex: <http://example.org/> .
        _:x ex:name "Alice" .
        _:x ex:age 30 .
        _:y ex:name "Bob" .
        _:y ex:knows _:x .
    "#;

    let graph_name2 = NamedNode::new("http://example.org/test2").unwrap();
    rdf_store.add_rdf_to_graph(rdf_data2, &graph_name2);

    // Get the canonical hash for the second graph
    let hash2 = rdf_store.canonicalize_graph(&graph_name2);

    // The hashes should be the same because the RDF content is semantically identical
    assert_eq!(hash1, hash2, "Canonical hashes should be identical for semantically equivalent RDF with different blank node identifiers");
}

#[test]
fn test_blockchain_with_rdf_canonicalization() {
    let mut bc = Blockchain::new();

    // First verify the genesis block is valid
    assert!(bc.is_valid(), "Genesis blockchain should be valid");

    // Add a block with simple RDF data
    let simple_rdf_data = r#"@prefix ex: <http://example.org/> . ex:test ex:value "simple" ."#;
    let _ = bc.add_block(simple_rdf_data.to_string());

    // Verify we have 2 blocks (genesis + our block)
    assert_eq!(bc.chain.len(), 2, "Should have 2 blocks");

    // Verify the blockchain is valid
    assert!(
        bc.is_valid(),
        "Blockchain should be valid after adding simple RDF block"
    );

    // Test RDF canonicalization directly by comparing canonical hashes
    let graph_name1 = oxigraph::model::NamedNode::new("http://provchain.org/block/1").unwrap();
    let canonical_hash1 = bc.rdf_store.canonicalize_graph(&graph_name1);

    // Create another RDF store with the same data
    let mut rdf_store2 = provchain_org::storage::rdf_store::RDFStore::new();
    let graph_name2 = oxigraph::model::NamedNode::new("http://example.org/test").unwrap();
    rdf_store2.add_rdf_to_graph(simple_rdf_data, &graph_name2);
    let canonical_hash2 = rdf_store2.canonicalize_graph(&graph_name2);

    // The canonical hashes should be identical for the same RDF content
    assert_eq!(
        canonical_hash1, canonical_hash2,
        "Canonical hashes should be identical for identical RDF content"
    );

    // Test with blank nodes - create two RDF stores with semantically equivalent data
    let mut rdf_store3 = provchain_org::storage::rdf_store::RDFStore::new();
    let rdf_data1 =
        r#"@prefix ex: <http://example.org/> . _:b1 ex:name "Alice" . _:b1 ex:age 30 ."#;
    let graph_name3 = oxigraph::model::NamedNode::new("http://example.org/test3").unwrap();
    rdf_store3.add_rdf_to_graph(rdf_data1, &graph_name3);

    let mut rdf_store4 = provchain_org::storage::rdf_store::RDFStore::new();
    let rdf_data2 = r#"@prefix ex: <http://example.org/> . _:x ex:name "Alice" . _:x ex:age 30 ."#;
    let graph_name4 = oxigraph::model::NamedNode::new("http://example.org/test4").unwrap();
    rdf_store4.add_rdf_to_graph(rdf_data2, &graph_name4);

    let canonical_hash3 = rdf_store3.canonicalize_graph(&graph_name3);
    let canonical_hash4 = rdf_store4.canonicalize_graph(&graph_name4);

    // The canonical hashes should be identical due to blank node canonicalization
    assert_eq!(canonical_hash3, canonical_hash4,
        "Canonical hashes should be identical for semantically equivalent RDF with different blank node identifiers");
}

#[test]
fn test_magic_placeholders_in_canonicalization() {
    let mut rdf_store = RDFStore::new();

    // Test that the Magic_S and Magic_O placeholders work correctly
    let rdf_data = r#"
        @prefix ex: <http://example.org/> .
        _:subject ex:predicate "object" .
        ex:namedSubject ex:predicate _:object .
    "#;

    let graph_name = NamedNode::new("http://example.org/magic_test").unwrap();
    rdf_store.add_rdf_to_graph(rdf_data, &graph_name);

    let hash = rdf_store.canonicalize_graph(&graph_name);

    // The hash should be deterministic and not empty
    assert!(!hash.is_empty(), "Canonical hash should not be empty");
    assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 characters long");
}
