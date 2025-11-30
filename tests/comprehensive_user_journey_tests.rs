//! Comprehensive User Journey Tests
//!
//! This test suite provides comprehensive E2E testing for complex user journeys,
//! performance benchmarks, and edge case handling.

use anyhow::Result;
use provchain_org::core::blockchain::Blockchain;
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Comprehensive test data for complex scenarios
const COMPLEX_SUPPLY_CHAIN_DATA: &str = r#"
@prefix : <http://example.org/> .
@prefix tc: <http://provchain.org/trace#> .

# Complex multi-step supply chain
:batch001 tc:product "Organic Coffee Beans" ;
          tc:origin "Farm ABC, Colombia" ;
          tc:currentLocation "Warehouse XYZ, USA" ;
          tc:status "In Transit" ;
          tc:batchId "BATCH001" ;
          tc:productionDate "2024-01-15" ;
          tc:certification "Organic" ;
          tc:environmentalData :env001 .

:env001 tc:temperature "22.5" ;
        tc:humidity "65.0" ;
        tc:co2Level "400" .

:batch002 tc:product "Fair Trade Cocoa" ;
          tc:origin "Farm DEF, Ecuador" ;
          tc:currentLocation "Processing Plant" ;
          tc:status "Processing" ;
          tc:batchId "BATCH002" ;
          tc:productionDate "2024-01-20" ;
          tc:certification "Fair Trade" .

# Complex relationships
:batch001 tc:processedInto :product001 .
:product001 tc:product "Premium Coffee Powder" ;
            tc:manufacturingDate "2024-02-01" ;
            tc:location "Factory USA" ;
            tc:qualityGrade "Premium" .
"#;

/// Find an available port for testing
async fn find_available_port() -> Result<u16> {
    use std::net::TcpListener;

    // Try to bind to port 0 to get an available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let port = addr.port();
    drop(listener); // Release the port

    // Wait a bit to ensure the port is released
    sleep(Duration::from_millis(100)).await;

    Ok(port)
}

/// Test helper for complex scenarios
async fn setup_test_environment() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    let mut blockchain = Blockchain::new();

    // Add complex test data
    let _ = blockchain.add_block(COMPLEX_SUPPLY_CHAIN_DATA.to_string());

    // Find an available port
    let port = find_available_port().await?;
    let mut config = provchain_org::config::Config::default();
    config.web.port = port;
    let server = provchain_org::web::server::create_web_server(blockchain, Some(config)).await?;
    let actual_port = server.port();

    let handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    sleep(Duration::from_millis(3000)).await;
    Ok((actual_port, handle))
}

/// Comprehensive E2E test for complex supply chain traceability
#[tokio::test]
#[ignore]
async fn test_complex_supply_chain_traceability() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // First authenticate to get a token
    let login_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    if !login_response.status().is_success() {
        println!("Login failed with status: {}", login_response.status());
        let error_text = login_response.text().await?;
        println!("Login error: {}", error_text);
        panic!("Authentication failed");
    }

    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Test complex traceability query with authentication
    let query = r#"
    PREFIX tc: <http://provchain.org/trace#>
    SELECT ?product ?origin ?status WHERE {
        GRAPH ?g {
            ?batch tc:product ?product ;
                   tc:origin ?origin ;
                   tc:status ?status .
            FILTER(?product = "Organic Coffee Beans")
        }
    }
    "#;

    let response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": query,
            "format": "json"
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        println!("SPARQL query failed with status: {}", response.status());
        let error_text = response.text().await?;
        println!("SPARQL error: {}", error_text);
        panic!("SPARQL query failed");
    }

    let results: serde_json::Value = response.json().await?;
    println!("Query results: {}", serde_json::to_string_pretty(&results)?);

    // Check if we have results (may be empty if no matching data)
    // The structure is results.results.bindings due to nested results
    assert!(results["results"]["results"]["bindings"].is_array());

    // Verify we got the expected data
    let bindings = &results["results"]["results"]["bindings"];
    if let Some(bindings_array) = bindings.as_array() {
        if !bindings_array.is_empty() {
            println!("Found {} results", bindings_array.len());
            // Verify the first result has the expected fields
            if let Some(first_result) = bindings_array.first() {
                assert!(first_result["product"].is_string());
                assert!(first_result["origin"].is_string());
                assert!(first_result["status"].is_string());
            }
        }
    }

    Ok(())
}

/// Performance benchmark for complex SPARQL queries
#[tokio::test]
#[ignore]
async fn test_performance_benchmark() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // First authenticate to get a token
    let login_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    assert!(
        login_response.status().is_success(),
        "Authentication should succeed"
    );
    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    let start = Instant::now();

    // Complex query with joins and filters (using GRAPH pattern)
    let complex_query = r#"
    PREFIX tc: <http://provchain.org/trace#>
    SELECT ?product ?origin ?status WHERE {
        GRAPH ?g {
            ?batch tc:product ?product ;
                   tc:origin ?origin ;
                   tc:status ?status .
            ?batch tc:certification ?cert .
            FILTER(?cert = "Organic" && ?status = "In Transit")
        }
    }
    "#;

    let response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": complex_query,
            "format": "json"
        }))
        .send()
        .await?;

    let duration = start.elapsed();
    assert!(
        response.status().is_success(),
        "SPARQL query should succeed"
    );
    assert!(
        duration < Duration::from_secs(5),
        "Query should complete within 5 seconds"
    );

    Ok(())
}

/// Edge case testing for error handling
#[tokio::test]
#[ignore]
async fn test_edge_cases() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // First authenticate to get a token
    let login_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    assert!(
        login_response.status().is_success(),
        "Authentication should succeed"
    );
    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Test invalid SPARQL query with authentication
    let invalid_query = "INVALID SPARQL SYNTAX";
    let response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": invalid_query,
            "format": "json"
        }))
        .send()
        .await?;

    assert!(
        response.status().is_client_error(),
        "Invalid SPARQL should return client error"
    );
    Ok(())
}

/// Performance benchmark for blockchain operations
#[tokio::test]
#[ignore]
async fn test_blockchain_performance() -> Result<()> {
    let mut blockchain = Blockchain::new();

    let start = Instant::now();
    for i in 0..1000 {
        let _ = blockchain.add_block(format!("Test data {}", i));
    }
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_secs(30),
        "1000 blocks should be added within 30 seconds"
    );
    assert!(
        blockchain.is_valid(),
        "Blockchain should remain valid after adding 1000 blocks"
    );

    Ok(())
}
