//! End-to-End API Workflow Tests
//!
//! This test suite validates complete API workflows including data ingestion,
//! query processing, traceability pipelines, and real-time updates.

use anyhow::Result;
use provchain_org::{config::Config, core::blockchain::Blockchain, web::server::create_web_server};
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test helper to start a test web server
async fn start_test_server() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    use std::net::TcpListener;

    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener); // Release the port for the server to use

    let blockchain = Blockchain::new();
    let mut config = Config::default();
    config.web.port = port;

    let server = create_web_server(blockchain, Some(config))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create server: {:?}", e))?;

    let handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(3000)).await;

    Ok((port, handle))
}

/// Test helper to authenticate and get token
async fn get_auth_token(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let auth_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await?;

    let auth_data: serde_json::Value = auth_response.json().await?;
    Ok(auth_data["token"].as_str().unwrap_or("").to_string())
}

/// Test helper to create sample RDF data
#[allow(dead_code)]
fn create_sample_rdf_data() -> String {
    r#"
@prefix tc: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:batch001 a tc:ProductBatch ;
    tc:hasBatchID "BATCH001" ;
    tc:product "Organic Coffee Beans" ;
    tc:origin "Farm ABC, Colombia" ;
    tc:currentLocation "Warehouse XYZ, USA" ;
    tc:status "In Transit" ;
    tc:producedAt "2024-01-15T08:00:00Z"^^xsd:dateTime .

:farmer001 a tc:Farmer ;
    tc:name "John Doe" ;
    tc:location "Farm ABC, Colombia" .

:batch001 prov:wasAttributedTo :farmer001 .

:transport001 a tc:TransportActivity ;
    tc:recordedAt "2024-01-16T10:00:00Z"^^xsd:dateTime ;
    prov:used :batch001 ;
    tc:hasCondition :condition001 .

:condition001 a tc:EnvironmentalCondition ;
    tc:hasTemperature "22.5"^^xsd:decimal ;
    tc:hasHumidity "65.0"^^xsd:decimal .
"#
    .to_string()
}

#[tokio::test]
#[ignore]
async fn test_complete_data_ingestion_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Complete Data Ingestion Pipeline on {}", base_url);

    // Step 1: Authentication
    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;
    assert!(!token.is_empty(), "Should receive authentication token");

    // Step 2: Get initial blockchain state
    let initial_status = client
        .get(format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let initial_data: serde_json::Value = initial_status.json().await?;
    let initial_height = initial_data["height"].as_u64().unwrap();

    // Step 3: Ingest RDF data via multiple triples
    let rdf_triples = vec![
        json!({
            "subject": "http://example.org/batch123",
            "predicate": "http://provchain.org/trace#hasBatchID",
            "object": "BATCH123",
            "graph_name": "supply_chain"
        }),
        json!({
            "subject": "http://example.org/batch123",
            "predicate": "http://provchain.org/trace#product",
            "object": "Organic Tomatoes",
            "graph_name": "supply_chain"
        }),
        json!({
            "subject": "http://example.org/batch123",
            "predicate": "http://provchain.org/trace#origin",
            "object": "Farm XYZ",
            "graph_name": "supply_chain"
        }),
        json!({
            "subject": "http://example.org/batch123",
            "predicate": "http://provchain.org/trace#status",
            "object": "Harvested",
            "graph_name": "supply_chain"
        }),
    ];

    let ingestion_start = Instant::now();
    for triple in rdf_triples {
        let response = client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&triple)
            .send()
            .await?;

        assert!(
            response.status().is_success(),
            "Should successfully ingest triple"
        );

        // Small delay to ensure proper ordering
        sleep(Duration::from_millis(100)).await;
    }
    let ingestion_duration = ingestion_start.elapsed();

    // Step 4: Verify data appears in blockchain
    let final_status = client
        .get(format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let final_data: serde_json::Value = final_status.json().await?;
    let final_height = final_data["height"].as_u64().unwrap();

    assert!(
        final_height > initial_height,
        "Blockchain height should increase"
    );
    assert_eq!(final_height - initial_height, 4, "Should add 4 new blocks");

    // Step 5: Verify data integrity via SPARQL query
    let verification_query = json!({
        "query": r#"
            SELECT ?property ?value WHERE {
                GRAPH ?g {
                    <http://example.org/batch123> ?property ?value .
                }
            }
            ORDER BY ?property
        "#,
        "format": "json"
    });

    let query_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&verification_query)
        .send()
        .await?;

    assert!(
        query_response.status().is_success(),
        "Should execute verification query"
    );
    let query_data: serde_json::Value = query_response.json().await?;
    assert_eq!(
        query_data["result_count"].as_u64().unwrap(),
        4,
        "Should find all 4 properties"
    );

    println!(
        "✓ Data Ingestion Pipeline completed in {:?}",
        ingestion_duration
    );
    println!("  Blocks added: {}", final_height - initial_height);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_sparql_query_processing_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing SPARQL Query Processing Pipeline on {}", base_url);

    // Setup: Add test data
    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    let test_data = vec![
        json!({
            "subject": "http://example.org/product1",
            "predicate": "http://provchain.org/trace#hasTemperature",
            "object": "25.0",
            "graph_name": "environmental"
        }),
        json!({
            "subject": "http://example.org/product2",
            "predicate": "http://provchain.org/trace#hasTemperature",
            "object": "30.0",
            "graph_name": "environmental"
        }),
        json!({
            "subject": "http://example.org/product3",
            "predicate": "http://provchain.org/trace#hasTemperature",
            "object": "20.0",
            "graph_name": "environmental"
        }),
    ];

    for data in test_data {
        client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&data)
            .send()
            .await?;
    }

    sleep(Duration::from_millis(1000)).await;

    // Test 1: Simple SELECT query
    let simple_query = json!({
        "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10",
        "format": "json"
    });

    let simple_start = Instant::now();
    let simple_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&simple_query)
        .send()
        .await?;
    let simple_duration = simple_start.elapsed();

    assert!(
        simple_response.status().is_success(),
        "Should execute simple query"
    );
    let simple_data: serde_json::Value = simple_response.json().await?;
    assert!(
        simple_data["result_count"].as_u64().unwrap() > 0,
        "Should return results"
    );

    // Test 2: Filtered query with FILTER
    let filtered_query = json!({
        "query": r#"
            SELECT ?product ?temp WHERE {
                GRAPH ?g {
                    ?product <http://provchain.org/trace#hasTemperature> ?temp .
                }
                FILTER(?temp > "22.0")
            }
            ORDER BY DESC(?temp)
        "#,
        "format": "json"
    });

    let filtered_start = Instant::now();
    let filtered_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&filtered_query)
        .send()
        .await?;
    let filtered_duration = filtered_start.elapsed();

    assert!(
        filtered_response.status().is_success(),
        "Should execute filtered query"
    );
    let filtered_data: serde_json::Value = filtered_response.json().await?;
    assert_eq!(
        filtered_data["result_count"].as_u64().unwrap(),
        2,
        "Should filter correctly"
    );

    // Test 3: Aggregation query
    let aggregation_query = json!({
        "query": r#"
            SELECT (COUNT(?product) as ?count) (AVG(?temp) as ?avg_temp) WHERE {
                GRAPH ?g {
                    ?product <http://provchain.org/trace#hasTemperature> ?temp .
                }
            }
        "#,
        "format": "json"
    });

    let agg_start = Instant::now();
    let agg_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&aggregation_query)
        .send()
        .await?;
    let agg_duration = agg_start.elapsed();

    assert!(
        agg_response.status().is_success(),
        "Should execute aggregation query"
    );
    let agg_data: serde_json::Value = agg_response.json().await?;
    assert_eq!(
        agg_data["result_count"].as_u64().unwrap(),
        1,
        "Should return aggregation result"
    );

    // Test 4: Complex query with multiple patterns
    let complex_query = json!({
        "query": r#"
            SELECT ?product ?temp ?graph WHERE {
                GRAPH ?graph {
                    ?product <http://provchain.org/trace#hasTemperature> ?temp .
                }
                FILTER(?temp >= "20.0" && ?temp <= "30.0")
            }
            ORDER BY ?temp
        "#,
        "format": "json"
    });

    let complex_start = Instant::now();
    let complex_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&complex_query)
        .send()
        .await?;
    let complex_duration = complex_start.elapsed();

    assert!(
        complex_response.status().is_success(),
        "Should execute complex query"
    );
    let complex_data: serde_json::Value = complex_response.json().await?;
    assert_eq!(
        complex_data["result_count"].as_u64().unwrap(),
        3,
        "Should return all matching results"
    );

    println!("✓ SPARQL Query Processing Pipeline completed");
    println!("  Simple query: {:?}", simple_duration);
    println!("  Filtered query: {:?}", filtered_duration);
    println!("  Aggregation query: {:?}", agg_duration);
    println!("  Complex query: {:?}", complex_duration);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_product_traceability_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Product Traceability Pipeline on {}", base_url);

    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    // Step 1: Create complete supply chain data
    let supply_chain_data = vec![
        // Product batch
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasBatchID",
            "object": "BATCH456",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#product",
            "object": "Premium Coffee",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#origin",
            "object": "Mountain Farm, Colombia",
            "graph_name": "traceability"
        }),
        // Supply chain events
        json!({
            "subject": "http://example.org/event1",
            "predicate": "http://provchain.org/trace#batch",
            "object": "http://example.org/batch456",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event1",
            "predicate": "http://provchain.org/trace#actor",
            "object": "Farmer Carlos",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event1",
            "predicate": "http://provchain.org/trace#action",
            "object": "Harvested",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event1",
            "predicate": "http://provchain.org/trace#timestamp",
            "object": "2024-01-15T08:00:00Z",
            "graph_name": "traceability"
        }),
        // Processing event
        json!({
            "subject": "http://example.org/event2",
            "predicate": "http://provchain.org/trace#batch",
            "object": "http://example.org/batch456",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event2",
            "predicate": "http://provchain.org/trace#actor",
            "object": "Processing Plant A",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event2",
            "predicate": "http://provchain.org/trace#action",
            "object": "Processed",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/event2",
            "predicate": "http://provchain.org/trace#timestamp",
            "object": "2024-01-20T14:30:00Z",
            "graph_name": "traceability"
        }),
        // Environmental conditions
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasTemperature",
            "object": "23.5",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasHumidity",
            "object": "68.0",
            "graph_name": "traceability"
        }),
        // Certifications
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#certification",
            "object": "Organic",
            "graph_name": "traceability"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#certification",
            "object": "Fair Trade",
            "graph_name": "traceability"
        }),
    ];

    // Ingest all supply chain data
    for data in supply_chain_data {
        let response = client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&data)
            .send()
            .await?;
        assert!(
            response.status().is_success(),
            "Should ingest supply chain data"
        );
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_millis(1000)).await;

    // Step 2: Test traceability query
    let trace_start = Instant::now();
    let trace_response = client
        .get(format!(
            "{}/api/products/trace?batch_id=BATCH456",
            base_url
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let trace_duration = trace_start.elapsed();

    assert!(
        trace_response.status().is_success(),
        "Should execute traceability query"
    );
    let trace_data: serde_json::Value = trace_response.json().await?;

    // Verify traceability response structure
    assert_eq!(
        trace_data["batch_id"], "BATCH456",
        "Should return correct batch ID"
    );
    assert!(
        trace_data["product_name"]
            .as_str()
            .unwrap()
            .contains("Premium Coffee"),
        "Should return product name"
    );
    assert!(
        trace_data["origin"]
            .as_str()
            .unwrap()
            .contains("Mountain Farm"),
        "Should return origin"
    );

    // Step 3: Test comprehensive traceability query via SPARQL
    // Simplified query to avoid syntax errors
    let comprehensive_query = json!({
        "query": r#"
            SELECT ?s ?p ?o WHERE {
                GRAPH ?g {
                    ?s ?p ?o .
                    FILTER(CONTAINS(STR(?s), "batch456"))
                }
            }
            LIMIT 20
        "#,
        "format": "json"
    });

    let comprehensive_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&comprehensive_query)
        .send()
        .await?;

    assert!(
        comprehensive_response.status().is_success(),
        "Should execute comprehensive query"
    );
    let comprehensive_data: serde_json::Value = comprehensive_response.json().await?;
    assert!(
        comprehensive_data["result_count"].as_u64().unwrap() > 0,
        "Should return comprehensive results"
    );

    // Step 4: Test timeline reconstruction - simplified to work with multi-graph structure
    let timeline_query = json!({
        "query": r#"
            SELECT ?s ?p ?o WHERE {
                GRAPH ?g {
                    ?s ?p ?o .
                    FILTER(CONTAINS(STR(?s), "event"))
                }
            }
            LIMIT 20
        "#,
        "format": "json"
    });

    let timeline_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&timeline_query)
        .send()
        .await?;

    assert!(
        timeline_response.status().is_success(),
        "Should execute timeline query"
    );
    let timeline_data: serde_json::Value = timeline_response.json().await?;
    // We expect at least some event data
    assert!(
        timeline_data["result_count"].as_u64().unwrap() > 0,
        "Should return event data"
    );

    println!(
        "✓ Product Traceability Pipeline completed in {:?}",
        trace_duration
    );
    println!(
        "  Comprehensive query results: {}",
        comprehensive_data["result_count"]
    );
    println!("  Timeline events: {}", timeline_data["result_count"]);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_blockchain_validation_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Blockchain Validation Pipeline on {}", base_url);

    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    // Step 1: Get initial blockchain state
    let initial_validation = client
        .get(format!("{}/api/blockchain/validate", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        initial_validation.status().is_success(),
        "Should validate initial blockchain"
    );
    let initial_data: serde_json::Value = initial_validation.json().await?;
    assert_eq!(
        initial_data["is_valid"], true,
        "Initial blockchain should be valid"
    );

    // Step 2: Add multiple blocks
    let test_blocks = ["Test block 1 data",
        "Test block 2 data",
        "Test block 3 data",
        "Test block 4 data",
        "Test block 5 data"];

    for (i, block_data) in test_blocks.iter().enumerate() {
        let triple_data = json!({
            "subject": format!("http://example.org/test{}", i),
            "predicate": "http://provchain.org/trace#data",
            "object": block_data,
            "graph_name": "validation_test"
        });

        let response = client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&triple_data)
            .send()
            .await?;

        assert!(response.status().is_success(), "Should add test block");
        sleep(Duration::from_millis(100)).await;
    }

    // Step 3: Validate blockchain after additions
    let validation_start = Instant::now();
    let final_validation = client
        .get(format!("{}/api/blockchain/validate", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let validation_duration = validation_start.elapsed();

    assert!(
        final_validation.status().is_success(),
        "Should validate final blockchain"
    );
    let final_data: serde_json::Value = final_validation.json().await?;
    assert_eq!(
        final_data["is_valid"], true,
        "Final blockchain should be valid"
    );

    // Step 4: Verify block structure
    let blocks_response = client
        .get(format!("{}/api/blockchain/blocks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        blocks_response.status().is_success(),
        "Should retrieve blocks"
    );
    let blocks_data: serde_json::Value = blocks_response.json().await?;
    let blocks = blocks_data.as_array().unwrap();

    // Verify block chain integrity
    for i in 1..blocks.len() {
        let current_block = &blocks[i];
        let previous_block = &blocks[i - 1];

        let current_prev_hash = current_block["previous_hash"].as_str().unwrap();
        let previous_hash = previous_block["hash"].as_str().unwrap();

        assert_eq!(
            current_prev_hash, previous_hash,
            "Block chain should be properly linked"
        );
        assert_eq!(
            current_block["index"].as_u64().unwrap(),
            i as u64,
            "Block indices should be sequential"
        );
    }

    // Step 5: Test individual block retrieval
    for i in 0..blocks.len().min(3) {
        let block_response = client
            .get(format!("{}/api/blockchain/blocks/{}", base_url, i))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        assert!(
            block_response.status().is_success(),
            "Should retrieve individual block"
        );
        let block_data: serde_json::Value = block_response.json().await?;
        assert_eq!(
            block_data["index"].as_u64().unwrap(),
            i as u64,
            "Should return correct block"
        );
    }

    println!(
        "✓ Blockchain Validation Pipeline completed in {:?}",
        validation_duration
    );
    println!("  Total blocks validated: {}", blocks.len());
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_concurrent_api_operations() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Concurrent API Operations on {}", base_url);

    // Create multiple clients for concurrent operations
    let num_clients = 5;
    let operations_per_client = 3;

    let mut handles = Vec::new();

    for client_id in 0..num_clients {
        let base_url = base_url.clone();
        let handle = tokio::spawn(async move {
            let client = Client::new();
            let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

            let mut results = Vec::new();

            for op_id in 0..operations_per_client {
                // Add data
                let add_start = Instant::now();
                let triple_data = json!({
                    "subject": format!("http://example.org/concurrent_{}_{}", client_id, op_id),
                    "predicate": "http://provchain.org/trace#clientId",
                    "object": format!("client_{}", client_id),
                    "graph_name": "concurrent_test"
                });

                let add_response = client
                    .post(format!("{}/api/blockchain/add-triple", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&triple_data)
                    .send()
                    .await?;
                let add_duration = add_start.elapsed();

                assert!(
                    add_response.status().is_success(),
                    "Should add data concurrently"
                );

                // Query data
                let query_start = Instant::now();
                let query_data = json!({
                    "query": format!("SELECT * WHERE {{ ?s <http://provchain.org/trace#clientId> \"client_{}\" }}", client_id),
                    "format": "json"
                });

                let query_response = client
                    .post(format!("{}/api/sparql/query", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&query_data)
                    .send()
                    .await?;
                let query_duration = query_start.elapsed();

                assert!(
                    query_response.status().is_success(),
                    "Should query data concurrently"
                );

                results.push((add_duration, query_duration));

                // Small delay to prevent overwhelming the server
                sleep(Duration::from_millis(50)).await;
            }

            Ok::<Vec<(Duration, Duration)>, anyhow::Error>(results)
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    let overall_start = Instant::now();
    let mut all_results = Vec::new();

    for handle in handles {
        let client_results = handle.await??;
        all_results.extend(client_results);
    }

    let overall_duration = overall_start.elapsed();

    // Analyze performance
    let total_operations = all_results.len();
    let avg_add_time: Duration =
        all_results.iter().map(|(add, _)| *add).sum::<Duration>() / total_operations as u32;
    let avg_query_time: Duration = all_results
        .iter()
        .map(|(_, query)| *query)
        .sum::<Duration>()
        / total_operations as u32;

    // Verify final state
    let client = Client::new();
    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    let final_status = client
        .get(format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let final_data: serde_json::Value = final_status.json().await?;
    let final_height = final_data["height"].as_u64().unwrap();

    // Should have added blocks from all concurrent operations
    assert!(
        final_height > 1,
        "Should have added blocks from concurrent operations"
    );

    // Performance assertions
    assert!(
        overall_duration < Duration::from_secs(60),
        "Concurrent operations should complete within 60 seconds"
    );
    assert!(
        avg_add_time < Duration::from_secs(5),
        "Average add time should be reasonable"
    );
    assert!(
        avg_query_time < Duration::from_secs(2),
        "Average query time should be reasonable"
    );

    println!(
        "✓ Concurrent API Operations completed in {:?}",
        overall_duration
    );
    println!("  Total operations: {}", total_operations);
    println!("  Average add time: {:?}", avg_add_time);
    println!("  Average query time: {:?}", avg_query_time);
    println!("  Final blockchain height: {}", final_height);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_error_handling_and_recovery_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!(
        "Testing Error Handling and Recovery Pipeline on {}",
        base_url
    );

    // Test 1: Invalid authentication
    let invalid_auth_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "invalid_user",
            "password": "invalid_password"
        }))
        .send()
        .await?;

    assert_eq!(
        invalid_auth_response.status(),
        401,
        "Should reject invalid credentials"
    );

    // Test 2: Unauthorized access
    let unauthorized_response = client
        .get(format!("{}/api/blockchain/status", base_url))
        .send()
        .await?;

    assert_eq!(
        unauthorized_response.status(),
        401,
        "Should require authentication"
    );

    // Test 3: Get valid token for subsequent tests
    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    // Test 4: Malformed request body
    let malformed_response = client
        .post(format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "invalid_field": "invalid_value"
        }))
        .send()
        .await?;

    assert!(
        malformed_response.status().is_client_error(),
        "Should reject malformed requests"
    );

    // Test 5: Invalid SPARQL syntax
    let invalid_sparql_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": "INVALID SPARQL SYNTAX HERE",
            "format": "json"
        }))
        .send()
        .await?;

    assert!(
        invalid_sparql_response.status().is_client_error(),
        "Should reject invalid SPARQL"
    );

    // Test 6: Non-existent resource
    let not_found_response = client
        .get(format!("{}/api/blockchain/blocks/99999", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert_eq!(
        not_found_response.status(),
        404,
        "Should return 404 for non-existent blocks"
    );

    // Test 7: Recovery - valid operations should still work after errors
    let recovery_data = json!({
        "subject": "http://example.org/recovery_test",
        "predicate": "http://provchain.org/trace#status",
        "object": "Recovery Test Successful",
        "graph_name": "recovery"
    });

    let recovery_response = client
        .post(format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&recovery_data)
        .send()
        .await?;

    assert!(
        recovery_response.status().is_success(),
        "Should recover and process valid requests"
    );

    // Test 8: System stability check
    let stability_response = client
        .get(format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        stability_response.status().is_success(),
        "System should remain stable after errors"
    );

    // Test 9: Verify recovery data was actually added
    let verification_query = json!({
        "query": "SELECT * WHERE { GRAPH ?g { <http://example.org/recovery_test> ?p ?o } }",
        "format": "json"
    });

    let verification_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&verification_query)
        .send()
        .await?;

    assert!(
        verification_response.status().is_success(),
        "Should verify recovery data"
    );
    let verification_data: serde_json::Value = verification_response.json().await?;
    assert!(
        verification_data["result_count"].as_u64().unwrap() > 0,
        "Should find recovery data"
    );

    println!("✓ Error Handling and Recovery Pipeline completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_performance_benchmarking_pipeline() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Performance Benchmarking Pipeline on {}", base_url);

    let token = get_auth_token(&client, &base_url, "admin", "admin123").await?;

    // Benchmark 1: Single triple insertion performance
    let single_insert_times = Vec::new();
    let mut insert_times = single_insert_times;

    for i in 0..10 {
        let start = Instant::now();
        let response = client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "subject": format!("http://example.org/perf_test_{}", i),
                "predicate": "http://provchain.org/trace#performance",
                "object": format!("Performance test {}", i),
                "graph_name": "performance"
            }))
            .send()
            .await?;
        let duration = start.elapsed();

        assert!(
            response.status().is_success(),
            "Should insert triple successfully"
        );
        insert_times.push(duration);

        sleep(Duration::from_millis(100)).await;
    }

    let avg_insert_time: Duration =
        insert_times.iter().sum::<Duration>() / insert_times.len() as u32;
    let max_insert_time = insert_times.iter().max().unwrap();
    let min_insert_time = insert_times.iter().min().unwrap();

    // Benchmark 2: Query performance with increasing data size
    let mut query_times = Vec::new();

    for limit in [10, 50, 100, 500] {
        let start = Instant::now();
        let response = client
            .post(format!("{}/api/sparql/query", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "query": format!("SELECT * WHERE {{ ?s ?p ?o }} LIMIT {}", limit),
                "format": "json"
            }))
            .send()
            .await?;
        let duration = start.elapsed();

        assert!(
            response.status().is_success(),
            "Should execute query successfully"
        );
        query_times.push((limit, duration));
    }

    // Benchmark 3: Blockchain validation performance
    let validation_start = Instant::now();
    let validation_response = client
        .get(format!("{}/api/blockchain/validate", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let validation_duration = validation_start.elapsed();

    assert!(
        validation_response.status().is_success(),
        "Should validate blockchain"
    );

    // Benchmark 4: Block retrieval performance
    let block_retrieval_start = Instant::now();
    let blocks_response = client
        .get(format!("{}/api/blockchain/blocks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let block_retrieval_duration = block_retrieval_start.elapsed();

    assert!(
        blocks_response.status().is_success(),
        "Should retrieve blocks"
    );

    // Performance assertions
    assert!(
        avg_insert_time < Duration::from_secs(2),
        "Average insert time should be under 2 seconds"
    );
    assert!(
        validation_duration < Duration::from_secs(5),
        "Validation should complete within 5 seconds"
    );
    assert!(
        block_retrieval_duration < Duration::from_secs(3),
        "Block retrieval should be fast"
    );

    // Query performance should scale reasonably
    for (limit, duration) in &query_times {
        assert!(
            duration < &Duration::from_secs(10),
            "Query with limit {} should complete within 10 seconds",
            limit
        );
    }

    println!("✓ Performance Benchmarking Pipeline completed");
    println!("  Average insert time: {:?}", avg_insert_time);
    println!("  Min insert time: {:?}", min_insert_time);
    println!("  Max insert time: {:?}", max_insert_time);
    println!("  Validation time: {:?}", validation_duration);
    println!("  Block retrieval time: {:?}", block_retrieval_duration);

    for (limit, duration) in query_times {
        println!("  Query (LIMIT {}): {:?}", limit, duration);
    }

    Ok(())
}
