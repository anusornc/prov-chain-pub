//! Comprehensive User Journey Tests
//!
//! This test suite provides comprehensive E2E testing for complex user journeys,
//! performance benchmarks, and edge case handling.

use anyhow::Result;
use provchain_org::core::blockchain::Blockchain;
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const TEST_JWT_SECRET: &str = "test-jwt-secret-key-min-32-chars-for-comprehensive-journeys";
const TEST_BOOTSTRAP_TOKEN: &str = "test-bootstrap-token-for-comprehensive-journeys";
const ADMIN_USERNAME: &str = "adminroot";
const ADMIN_PASSWORD: &str = "AdminRootPassword123!";

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
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);
    std::env::set_var("PROVCHAIN_BOOTSTRAP_TOKEN", TEST_BOOTSTRAP_TOKEN);

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

async fn get_auth_token(client: &Client, base_url: &str) -> Result<String> {
    let login_response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": ADMIN_USERNAME,
            "password": ADMIN_PASSWORD
        }))
        .send()
        .await?;

    if login_response.status() == StatusCode::OK {
        let auth_data: serde_json::Value = login_response.json().await?;
        return Ok(auth_data["token"].as_str().unwrap_or("").to_string());
    }

    if login_response.status() == StatusCode::UNAUTHORIZED {
        let bootstrap_response = client
            .post(format!("{}/auth/bootstrap", base_url))
            .json(&json!({
                "username": ADMIN_USERNAME,
                "password": ADMIN_PASSWORD,
                "bootstrap_token": TEST_BOOTSTRAP_TOKEN
            }))
            .send()
            .await?;

        assert!(
            bootstrap_response.status() == StatusCode::OK
                || bootstrap_response.status() == StatusCode::CONFLICT,
            "bootstrap should succeed or report conflict, got {}",
            bootstrap_response.status()
        );

        let retry_response = client
            .post(format!("{}/auth/login", base_url))
            .json(&json!({
                "username": ADMIN_USERNAME,
                "password": ADMIN_PASSWORD
            }))
            .send()
            .await?;

        assert_eq!(
            retry_response.status(),
            StatusCode::OK,
            "login after bootstrap should succeed"
        );

        let auth_data: serde_json::Value = retry_response.json().await?;
        return Ok(auth_data["token"].as_str().unwrap_or("").to_string());
    }

    let auth_data: serde_json::Value = login_response.json().await?;
    Ok(auth_data["token"].as_str().unwrap_or("").to_string())
}

/// Comprehensive E2E test for complex supply chain traceability
#[tokio::test]
async fn test_complex_supply_chain_traceability() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    let token = get_auth_token(&client, &base_url).await?;

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
    assert!(
        results["result_count"].as_u64().unwrap_or(0) > 0,
        "should return at least one matching traceability result"
    );

    Ok(())
}

/// Performance benchmark for complex SPARQL queries
#[tokio::test]
async fn test_performance_benchmark() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    let token = get_auth_token(&client, &base_url).await?;

    let start = Instant::now();

    // Use a deterministic query shape that still exercises multiple predicates
    // without assuming all patterns live in the same named graph.
    let complex_query = r#"
    PREFIX : <http://example.org/>
    PREFIX tc: <http://provchain.org/trace#>
    SELECT ?product ?origin ?status WHERE {
        GRAPH ?g1 { :batch001 tc:product ?product . }
        GRAPH ?g2 { :batch001 tc:origin ?origin . }
        GRAPH ?g3 { :batch001 tc:status ?status . }
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
    let results: serde_json::Value = response.json().await?;
    assert!(
        results["result_count"].as_u64().unwrap_or(0) > 0,
        "query should return at least one result"
    );
    assert!(
        duration < Duration::from_secs(5),
        "Query should complete within 5 seconds"
    );

    Ok(())
}

/// Edge case testing for error handling
#[tokio::test]
async fn test_edge_cases() -> Result<()> {
    let (port, _server_handle) = setup_test_environment().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    let token = get_auth_token(&client, &base_url).await?;

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
