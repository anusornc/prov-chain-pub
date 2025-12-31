//! Performance Benchmark Tests
//!
//! Comprehensive performance testing for realistic scenarios and stress testing

use anyhow::Result;
use provchain_org::core::blockchain::Blockchain;
use std::time::{Duration, Instant};

/// Test blockchain performance under realistic load
#[test]
#[ignore]
fn test_blockchain_performance_realistic_load() -> Result<()> {
    let start = Instant::now();
    let mut blockchain = Blockchain::new();

    // Simulate realistic supply chain data
    for i in 0..1000 {
        let realistic_data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:04} tc:product "Product {}" ;
                   tc:origin "Farm {}" ;
                   tc:batchId "BATCH{:04}" ;
                   tc:timestamp "2024-01-{:02}T10:00:00Z" ;
                   tc:status "In Transit" ;
                   tc:temperature "22.5" ;
                   tc:humidity "65.0" ;
                   tc:location "Warehouse {}" .
        "#,
            i,
            i % 100,
            i % 50,
            i,
            (i % 28) + 1,
            i % 10
        );

        let _ = blockchain.add_block(realistic_data);

        // Validate every 100 blocks to ensure integrity
        if i % 100 == 0 {
            assert!(
                blockchain.is_valid(),
                "Blockchain should remain valid at block {}",
                i
            );
        }
    }

    let duration = start.elapsed();
    println!("Added 1000 realistic blocks in {:?}", duration);

    // Performance requirements
    assert!(
        duration < Duration::from_secs(60),
        "Should add 1000 blocks within 60 seconds"
    );
    assert!(blockchain.is_valid(), "Final blockchain should be valid");
    assert_eq!(blockchain.chain.len(), 1001); // Genesis + 1000 blocks

    Ok(())
}

/// Test SPARQL query performance with large datasets
#[test]
fn test_sparql_query_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Add substantial test data
    for i in 0..500 {
        let data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:04} tc:product "Product Type {}" ;
                   tc:origin "Origin {}" ;
                   tc:status "Status {}" ;
                   tc:certification "Cert {}" ;
                   tc:quality "Grade {}" .
        "#,
            i,
            i % 10,
            i % 20,
            i % 5,
            i % 3,
            i % 4
        );

        let _ = blockchain.add_block(data);
    }

    // Test simple query performance
    let start = Instant::now();
    let simple_query = r#"
    PREFIX tc: <http://provchain.org/trace#>
    SELECT ?product WHERE {
        ?batch tc:product ?product .
    } LIMIT 100
    "#;

    let _results = blockchain.rdf_store.query(simple_query);
    let simple_duration = start.elapsed();

    // Test complex query performance
    let start = Instant::now();
    let complex_query = r#"
    PREFIX tc: <http://provchain.org/trace#>
    SELECT ?product ?origin ?status WHERE {
        ?batch tc:product ?product ;
               tc:origin ?origin ;
               tc:status ?status ;
               tc:certification ?cert .
        FILTER(?cert = "Cert 1" && CONTAINS(?product, "Type 1"))
    }
    "#;

    let _results = blockchain.rdf_store.query(complex_query);
    let complex_duration = start.elapsed();

    println!(
        "Simple query: {:?}, Complex query: {:?}",
        simple_duration, complex_duration
    );

    // Performance requirements
    assert!(
        simple_duration < Duration::from_secs(5),
        "Simple queries should complete within 5 seconds"
    );
    assert!(
        complex_duration < Duration::from_secs(10),
        "Complex queries should complete within 10 seconds"
    );

    Ok(())
}

/// Test RDF canonicalization performance
#[test]
fn test_rdf_canonicalization_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Add data with complex RDF structures including blank nodes
    let complex_rdf = r#"
    @prefix : <http://example.org/> .
    @prefix tc: <http://provchain.org/trace#> .
    
    :batch001 tc:product "Complex Product" ;
              tc:origin "Complex Farm" ;
              tc:hasEnvironmentalData [
                  tc:temperature "22.5" ;
                  tc:humidity "65.0" ;
                  tc:co2Level "400"
              ] ;
              tc:hasQualityCheck [
                  tc:inspector "John Doe" ;
                  tc:grade "A" ;
                  tc:timestamp "2024-01-01T10:00:00Z"
              ] .
    "#;

    let _ = blockchain.add_block(complex_rdf.to_string());

    // Test canonicalization performance
    let graph_name = oxigraph::model::NamedNode::new("http://provchain.org/block/1").unwrap();

    let start = Instant::now();
    for _ in 0..100 {
        let _hash = blockchain.rdf_store.canonicalize_graph(&graph_name);
    }
    let duration = start.elapsed();

    println!("100 canonicalizations took {:?}", duration);

    // Performance requirement
    assert!(
        duration < Duration::from_secs(10),
        "100 canonicalizations should complete within 10 seconds"
    );

    Ok(())
}

/// Test concurrent access performance
#[test]
#[ignore]
fn test_concurrent_access_performance() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    let mut blockchain = Blockchain::new();

    // Add initial data
    for i in 0..100 {
        let data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:03} tc:product "Product {}" ;
                   tc:origin "Farm {}" .
        "#,
            i, i, i
        );
        let _ = blockchain.add_block(data);
    }

    let blockchain = Arc::new(blockchain);
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn multiple threads for concurrent read access
    for i in 0..10 {
        let blockchain_clone = Arc::clone(&blockchain);
        let handle = thread::spawn(move || -> Result<()> {
            let query = format!(
                r#"
            PREFIX tc: <http://provchain.org/trace#>
            SELECT ?product WHERE {{
                ?batch tc:product ?product .
                FILTER(CONTAINS(?product, "Product {}"))
            }}
            "#,
                i * 10
            );

            // Perform multiple queries
            for _ in 0..50 {
                let _results = blockchain_clone.rdf_store.query(&query);
            }

            Ok(())
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }

    let duration = start.elapsed();
    println!(
        "Concurrent access (10 threads, 50 queries each) took {:?}",
        duration
    );

    // Performance requirement
    assert!(
        duration < Duration::from_secs(30),
        "Concurrent access should complete within 30 seconds"
    );

    Ok(())
}

/// Test memory usage under load
#[test]
#[ignore]
fn test_memory_usage_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Add progressively larger blocks to test memory efficiency
    for i in 0..200 {
        let large_description = "A".repeat(1000); // 1KB description
        let data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:04} tc:product "Product {}" ;
                   tc:origin "Farm {}" ;
                   tc:description "{}" ;
                   tc:metadata "Additional metadata for batch {}" .
        "#,
            i, i, i, large_description, i
        );

        let _ = blockchain.add_block(data);

        // Validate periodically to ensure memory isn't causing corruption
        if i % 50 == 0 {
            assert!(
                blockchain.is_valid(),
                "Blockchain should remain valid under memory pressure"
            );
        }
    }

    // Final validation
    assert!(
        blockchain.is_valid(),
        "Blockchain should be valid after memory stress test"
    );
    assert_eq!(blockchain.chain.len(), 201); // Genesis + 200 blocks

    Ok(())
}

/// Test blockchain validation performance with large chains
#[test]
#[ignore]
fn test_validation_performance_large_chain() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Create a large blockchain
    for i in 0..500 {
        let data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:04} tc:product "Product {}" ;
                   tc:origin "Farm {}" ;
                   tc:timestamp "2024-01-{:02}T10:00:00Z" .
        "#,
            i,
            i,
            i,
            (i % 28) + 1
        );

        let _ = blockchain.add_block(data);
    }

    // Test validation performance
    let start = Instant::now();
    let is_valid = blockchain.is_valid();
    let duration = start.elapsed();

    println!("Validating 500-block chain took {:?}", duration);

    assert!(is_valid, "Large blockchain should be valid");
    assert!(
        duration < Duration::from_secs(30),
        "Validation should complete within 30 seconds"
    );

    Ok(())
}

/// Test performance degradation over time
#[test]
#[ignore]
fn test_performance_degradation() -> Result<()> {
    let mut blockchain = Blockchain::new();
    let mut add_times = Vec::new();
    let mut validation_times = Vec::new();

    // Measure performance at different blockchain sizes
    for batch in 0..5 {
        let batch_start = Instant::now();

        // Add 100 blocks
        for i in 0..100 {
            let block_index = batch * 100 + i;
            let data = format!(
                r#"
            @prefix : <http://example.org/> .
            @prefix tc: <http://provchain.org/trace#> .
            
            :batch{:04} tc:product "Product {}" ;
                       tc:origin "Farm {}" .
            "#,
                block_index, block_index, block_index
            );

            let _ = blockchain.add_block(data);
        }

        let add_time = batch_start.elapsed();
        add_times.push(add_time);

        // Measure validation time
        let validation_start = Instant::now();
        assert!(blockchain.is_valid());
        let validation_time = validation_start.elapsed();
        validation_times.push(validation_time);

        println!(
            "Batch {}: Add time {:?}, Validation time {:?}",
            batch, add_time, validation_time
        );
    }

    // Check that performance doesn't degrade too much
    let first_add_time = add_times[0];
    let last_add_time = add_times[add_times.len() - 1];
    let add_degradation = last_add_time.as_millis() as f64 / first_add_time.as_millis() as f64;

    let first_validation_time = validation_times[0];
    let last_validation_time = validation_times[validation_times.len() - 1];
    let validation_degradation =
        last_validation_time.as_millis() as f64 / first_validation_time.as_millis() as f64;

    println!("Add time degradation: {:.2}x", add_degradation);
    println!(
        "Validation time degradation: {:.2}x",
        validation_degradation
    );

    // Performance should not degrade more than 10x
    assert!(
        add_degradation < 10.0,
        "Add time degradation should be less than 10x"
    );
    assert!(
        validation_degradation < 10.0,
        "Validation time degradation should be less than 10x"
    );

    Ok(())
}

/// Test query performance with complex filters
#[test]
#[ignore]
fn test_complex_query_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Add diverse test data
    let products = ["Coffee", "Cocoa", "Tea", "Sugar", "Vanilla"];
    let origins = ["Colombia", "Ecuador", "India", "Brazil", "Madagascar"];
    let statuses = ["Harvested", "Processing", "In Transit", "Delivered"];

    for i in 0..1000 {
        let data = format!(
            r#"
        @prefix : <http://example.org/> .
        @prefix tc: <http://provchain.org/trace#> .
        
        :batch{:04} tc:product "{}" ;
                   tc:origin "{}" ;
                   tc:status "{}" ;
                   tc:timestamp "2024-{:02}-{:02}T10:00:00Z" ;
                   tc:quality "{}" .
        "#,
            i,
            products[i % products.len()],
            origins[i % origins.len()],
            statuses[i % statuses.len()],
            (i % 12) + 1,
            (i % 28) + 1,
            if i % 3 == 0 { "Premium" } else { "Standard" }
        );

        let _ = blockchain.add_block(data);
    }

    // Test various complex queries
    let queries = vec![
        // Simple filter
        r#"
        PREFIX tc: <http://provchain.org/trace#>
        SELECT ?batch WHERE {
            ?batch tc:product "Coffee" .
        }
        "#,
        // Multiple filters
        r#"
        PREFIX tc: <http://provchain.org/trace#>
        SELECT ?batch WHERE {
            ?batch tc:product "Coffee" ;
                   tc:origin "Colombia" ;
                   tc:status "In Transit" .
        }
        "#,
        // Complex filter with CONTAINS
        r#"
        PREFIX tc: <http://provchain.org/trace#>
        SELECT ?batch ?product WHERE {
            ?batch tc:product ?product ;
                   tc:quality "Premium" .
            FILTER(CONTAINS(?product, "Co"))
        }
        "#,
        // Aggregation query
        r#"
        PREFIX tc: <http://provchain.org/trace#>
        SELECT ?product (COUNT(?batch) as ?count) WHERE {
            ?batch tc:product ?product .
        }
        GROUP BY ?product
        "#,
    ];

    for (i, query) in queries.iter().enumerate() {
        let start = Instant::now();
        let _results = blockchain.rdf_store.query(query);
        let duration = start.elapsed();

        println!("Query {} took {:?}", i + 1, duration);
        assert!(
            duration < Duration::from_secs(15),
            "Query {} should complete within 15 seconds",
            i + 1
        );
    }

    Ok(())
}

/// Benchmark RDF canonicalization algorithms
#[test]
fn test_canonicalization_algorithm_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    // Add data with varying complexity
    let simple_rdf = r#"
    @prefix : <http://example.org/> .
    @prefix tc: <http://provchain.org/trace#> .
    
    :batch001 tc:product "Simple Product" ;
              tc:origin "Simple Farm" .
    "#;

    let complex_rdf = r#"
    @prefix : <http://example.org/> .
    @prefix tc: <http://provchain.org/trace#> .
    
    :batch002 tc:product "Complex Product" ;
              tc:origin "Complex Farm" ;
              tc:hasEnvironmentalData [
                  tc:temperature "22.5" ;
                  tc:humidity "65.0" ;
                  tc:co2Level "400" ;
                  tc:recordedBy [
                      tc:sensor "TempSensor001" ;
                      tc:calibrationDate "2024-01-01"
                  ]
              ] .
    "#;

    let _ = blockchain.add_block(simple_rdf.to_string());
    let _ = blockchain.add_block(complex_rdf.to_string());

    // Test performance of different canonicalization approaches
    let simple_graph = oxigraph::model::NamedNode::new("http://provchain.org/block/1").unwrap();
    let complex_graph = oxigraph::model::NamedNode::new("http://provchain.org/block/2").unwrap();

    // Test simple graph canonicalization
    let start = Instant::now();
    for _ in 0..100 {
        let _hash = blockchain.rdf_store.canonicalize_graph(&simple_graph);
    }
    let simple_duration = start.elapsed();

    // Test complex graph canonicalization
    let start = Instant::now();
    for _ in 0..100 {
        let _hash = blockchain.rdf_store.canonicalize_graph(&complex_graph);
    }
    let complex_duration = start.elapsed();

    println!("Simple canonicalization (100x): {:?}", simple_duration);
    println!("Complex canonicalization (100x): {:?}", complex_duration);

    // Performance requirements
    assert!(
        simple_duration < Duration::from_secs(5),
        "Simple canonicalization should be fast"
    );
    assert!(
        complex_duration < Duration::from_secs(15),
        "Complex canonicalization should complete within reasonable time"
    );

    Ok(())
}
