use provchain_org::rdf_store::RDFStore;
use oxigraph::model::NamedNode;
use oxigraph::sparql::QueryResults;
use std::fs;

#[test]
fn test_minimal_test_data_file() {
    let mut store = RDFStore::new();
    
    // Read the minimal test data file
    let turtle_data = fs::read_to_string("tests/data/minimal_test_data.ttl")
        .expect("Failed to read minimal_test_data.ttl");
    
    let graph_name = NamedNode::new("http://provchain.org/test_graph").unwrap();
    store.add_rdf_to_graph(&turtle_data, &graph_name);

    // Query for the test data
    let query = r#"PREFIX pc: <http://provchain.org/>
        SELECT ?value
        FROM <http://provchain.org/test_graph>
        WHERE {
            pc:testData pc:hasValue ?value .
        }
    "#;

    if let QueryResults::Solutions(solutions) = store.query(query) {
        let results: Vec<_> = solutions.collect();
        assert_eq!(results.len(), 1, "Should find exactly one test value");
        let solution = results[0].as_ref().unwrap();
        assert_eq!(solution.get("value").unwrap().to_string(), "\"Updated with ProvChain namespace\"");
    } else {
        panic!("SPARQL query failed");
    }
}

#[test]
fn test_complete_supply_chain_test_file() {
    let mut store = RDFStore::new();
    
    // Read the complete supply chain test data file
    let turtle_data = fs::read_to_string("tests/data/complete_supply_chain_test.ttl")
        .expect("Failed to read complete_supply_chain_test.ttl");
    
    let graph_name = NamedNode::new("http://provchain.org/test_graph").unwrap();
    store.add_rdf_to_graph(&turtle_data, &graph_name);

    // Query for milk batch
    let query = r#"PREFIX trace: <http://example.org/trace#>
        PREFIX prov: <http://www.w3.org/ns/prov#>
        SELECT ?batch ?batchId
        FROM <http://provchain.org/test_graph>
        WHERE {
            ?batch a trace:ProductBatch ;
                   trace:hasBatchID ?batchId .
        }
    "#;

    if let QueryResults::Solutions(solutions) = store.query(query) {
        let results: Vec<_> = solutions.collect();
        assert!(!results.is_empty(), "Should find at least one product batch");
        
        // Check if we can find the specific milk batch
        let mut found_milk_batch = false;
        for result in results {
            if let Ok(solution) = result {
                if let Some(batch_id) = solution.get("batchId") {
                    if batch_id.to_string().contains("MB-2025-001") {
                        found_milk_batch = true;
                        break;
                    }
                }
            }
        }
        assert!(found_milk_batch, "Should find the milk batch MB-2025-001");
    } else {
        panic!("SPARQL query failed");
    }
}

#[test]
fn test_supply_chain_provenance_relationships() {
    let mut store = RDFStore::new();
    
    // Read the complete supply chain test data file
    let turtle_data = fs::read_to_string("tests/data/complete_supply_chain_test.ttl")
        .expect("Failed to read complete_supply_chain_test.ttl");
    
    let graph_name = NamedNode::new("http://provchain.org/test_graph").unwrap();
    store.add_rdf_to_graph(&turtle_data, &graph_name);

    // Query for provenance relationships
    let query = r#"PREFIX trace: <http://example.org/trace#>
        PREFIX prov: <http://www.w3.org/ns/prov#>
        SELECT ?product ?source
        FROM <http://provchain.org/test_graph>
        WHERE {
            ?product trace:lotDerivedFrom ?source .
        }
    "#;

    if let QueryResults::Solutions(solutions) = store.query(query) {
        let results: Vec<_> = solutions.collect();
        assert!(!results.is_empty(), "Should find at least one derivation relationship");
    } else {
        panic!("SPARQL query failed");
    }
}
