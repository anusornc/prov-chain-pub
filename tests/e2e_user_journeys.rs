#![cfg(feature = "e2e")]
//! End-to-End User Journey Tests
//!
//! This test suite validates complete user workflows from browser interactions
//! through to blockchain storage and retrieval, simulating real-world usage patterns.

use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use provchain_org::{blockchain::Blockchain, rdf_store::RDFStore, web::server::create_web_server};
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test helper to start a test web server
async fn start_test_server() -> anyhow::Result<(u16, tokio::task::JoinHandle<()>)> {
    let blockchain = Blockchain::new();
    let server = create_web_server(blockchain, Some(0))
        .await
        .map_err(|e| anyhow::Error::from(e))
        .with_context(|| "Failed to create web server")?;
    let server = std::sync::Arc::new(server);
    let port = server.port();

    let server_clone = server.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(500)).await;

    Ok((port, handle))
}

/// Test helper to create browser instance
fn create_browser() -> Result<Browser> {
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .window_size(Some((1920, 1080)))
        .build()?;

    Browser::new(options)
}

/// Test helper to authenticate user
async fn authenticate_user(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let auth_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": username,
            "password": password
        }))
        .send()
        .await?;

    let auth_data: serde_json::Value = auth_response.json().await?;
    Ok(auth_data["token"].as_str().unwrap_or("").to_string())
}

#[tokio::test]
async fn test_supply_chain_manager_complete_journey() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Supply Chain Manager Journey on {}", base_url);

    // Step 1: Authentication
    let token = authenticate_user(&client, &base_url, "manager", "password").await?;
    assert!(!token.is_empty(), "Should receive authentication token");

    // Step 2: Add new product batch via API
    let new_batch_data = json!({
        "subject": "http://example.org/batch123",
        "predicate": "http://provchain.org/trace#hasBatchID",
        "object": "BATCH123",
        "graph_name": "supply_chain"
    });

    let add_response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&new_batch_data)
        .send()
        .await?;

    assert!(
        add_response.status().is_success(),
        "Should successfully add batch data"
    );

    // Step 3: Verify batch appears in blockchain
    let blocks_response = client
        .get(&format!("{}/api/blockchain/blocks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        blocks_response.status().is_success(),
        "Should retrieve blocks"
    );
    let blocks_data: serde_json::Value = blocks_response.json().await?;
    assert!(
        blocks_data.as_array().unwrap().len() > 1,
        "Should have added new block"
    );

    // Step 4: Query for the batch using SPARQL
    let sparql_query = json!({
        "query": "SELECT ?s ?p ?o WHERE { ?s <http://provchain.org/trace#hasBatchID> \"BATCH123\" . ?s ?p ?o }",
        "format": "json"
    });

    let query_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&sparql_query)
        .send()
        .await?;

    assert!(
        query_response.status().is_success(),
        "Should execute SPARQL query"
    );
    let query_data: serde_json::Value = query_response.json().await?;
    assert!(
        query_data["result_count"].as_u64().unwrap() > 0,
        "Should find the added batch"
    );

    // Step 5: Trace the product
    let trace_response = client
        .get(&format!(
            "{}/api/products/trace?batch_id=BATCH123",
            base_url
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        trace_response.status().is_success(),
        "Should trace product successfully"
    );
    let trace_data: serde_json::Value = trace_response.json().await?;
    assert_eq!(
        trace_data["batch_id"], "BATCH123",
        "Should return correct batch ID"
    );

    println!("✓ Supply Chain Manager journey completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_quality_auditor_complete_journey() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Quality Auditor Journey on {}", base_url);

    // Step 1: Authentication as auditor
    let token = authenticate_user(&client, &base_url, "auditor", "password").await?;
    assert!(!token.is_empty(), "Should receive authentication token");

    // Step 2: Add sample quality data first
    let quality_data = vec![
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasQualityCheck",
            "object": "Passed",
            "graph_name": "quality_control"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasTemperature",
            "object": "4.2",
            "graph_name": "quality_control"
        }),
        json!({
            "subject": "http://example.org/batch456",
            "predicate": "http://provchain.org/trace#hasCertification",
            "object": "Organic",
            "graph_name": "quality_control"
        }),
    ];

    for data in quality_data {
        let response = client
            .post(&format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&data)
            .send()
            .await?;
        assert!(response.status().is_success(), "Should add quality data");
    }

    // Step 3: Execute compliance query
    let compliance_query = json!({
        "query": r#"
            SELECT ?batch ?quality ?temp ?cert WHERE {
                ?batch <http://provchain.org/trace#hasQualityCheck> ?quality .
                ?batch <http://provchain.org/trace#hasTemperature> ?temp .
                ?batch <http://provchain.org/trace#hasCertification> ?cert .
                FILTER(?temp < "5.0")
            }
        "#,
        "format": "json"
    });

    let query_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&compliance_query)
        .send()
        .await?;

    assert!(
        query_response.status().is_success(),
        "Should execute compliance query"
    );
    let query_data: serde_json::Value = query_response.json().await?;
    assert!(
        query_data["result_count"].as_u64().unwrap() > 0,
        "Should find compliant batches"
    );

    // Step 4: Validate blockchain integrity
    let validation_response = client
        .get(&format!("{}/api/blockchain/validate", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        validation_response.status().is_success(),
        "Should validate blockchain"
    );
    let validation_data: serde_json::Value = validation_response.json().await?;
    assert_eq!(validation_data["valid"], true, "Blockchain should be valid");

    // Step 5: Generate audit report (via complex query)
    let audit_query = json!({
        "query": r#"
            SELECT ?batch (COUNT(?property) as ?property_count) WHERE {
                ?batch ?property ?value .
                FILTER(STRSTARTS(STR(?batch), "http://example.org/batch"))
            }
            GROUP BY ?batch
            ORDER BY DESC(?property_count)
        "#,
        "format": "json"
    });

    let audit_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&audit_query)
        .send()
        .await?;

    assert!(
        audit_response.status().is_success(),
        "Should generate audit report"
    );

    println!("✓ Quality Auditor journey completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_consumer_public_access_journey() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Consumer Public Access Journey on {}", base_url);

    // Step 1: Access public health endpoint (no auth required)
    let health_response = client.get(&format!("{}/health", base_url)).send().await?;

    assert!(
        health_response.status().is_success(),
        "Should access health endpoint"
    );

    // Step 2: Try to access protected endpoint without auth (should fail)
    let protected_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .send()
        .await?;

    assert_eq!(
        protected_response.status(),
        401,
        "Should require authentication"
    );

    // Step 3: Authenticate as consumer
    let token = authenticate_user(&client, &base_url, "consumer", "password").await?;
    assert!(!token.is_empty(), "Should receive authentication token");

    // Step 4: Access blockchain status with auth
    let status_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        status_response.status().is_success(),
        "Should access blockchain status"
    );
    let status_data: serde_json::Value = status_response.json().await?;
    assert!(
        status_data["height"].as_u64().unwrap() >= 1,
        "Should have blockchain height"
    );

    // Step 5: Search for product information
    let product_query = json!({
        "query": "SELECT ?product ?origin ?status WHERE { ?batch <http://provchain.org/trace#product> ?product . ?batch <http://provchain.org/trace#origin> ?origin . ?batch <http://provchain.org/trace#status> ?status }",
        "format": "json"
    });

    let search_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&product_query)
        .send()
        .await?;

    assert!(
        search_response.status().is_success(),
        "Should search for products"
    );

    println!("✓ Consumer Public Access journey completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_administrator_system_management_journey() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!(
        "Testing Administrator System Management Journey on {}",
        base_url
    );

    // Step 1: Authentication as admin
    let token = authenticate_user(&client, &base_url, "admin", "password").await?;
    assert!(!token.is_empty(), "Should receive authentication token");

    // Step 2: Monitor system health
    let health_start = Instant::now();
    let status_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let health_duration = health_start.elapsed();

    assert!(
        status_response.status().is_success(),
        "Should get system status"
    );
    assert!(
        health_duration < Duration::from_secs(1),
        "Health check should be fast"
    );

    let status_data: serde_json::Value = status_response.json().await?;
    assert!(
        status_data["height"].as_u64().unwrap() >= 1,
        "Should have blockchain data"
    );

    // Step 3: Validate blockchain integrity
    let validation_start = Instant::now();
    let validation_response = client
        .get(&format!("{}/api/blockchain/validate", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let validation_duration = validation_start.elapsed();

    assert!(
        validation_response.status().is_success(),
        "Should validate blockchain"
    );
    assert!(
        validation_duration < Duration::from_secs(5),
        "Validation should complete quickly"
    );

    let validation_data: serde_json::Value = validation_response.json().await?;
    assert_eq!(validation_data["valid"], true, "Blockchain should be valid");

    // Step 4: Retrieve all blocks for audit
    let blocks_response = client
        .get(&format!("{}/api/blockchain/blocks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        blocks_response.status().is_success(),
        "Should retrieve all blocks"
    );
    let blocks_data: serde_json::Value = blocks_response.json().await?;
    let blocks = blocks_data.as_array().unwrap();
    assert!(blocks.len() >= 1, "Should have at least genesis block");

    // Step 5: Verify block integrity
    for (i, block) in blocks.iter().enumerate() {
        assert!(
            block["hash"].as_str().unwrap().len() > 0,
            "Block should have hash"
        );
        assert!(
            block["index"].as_u64().unwrap() == i as u64,
            "Block index should be sequential"
        );

        if i > 0 {
            let prev_hash = blocks[i - 1]["hash"].as_str().unwrap();
            let current_prev_hash = block["previous_hash"].as_str().unwrap();
            assert_eq!(prev_hash, current_prev_hash, "Block chain should be linked");
        }
    }

    // Step 6: Execute system-wide analytics query
    let analytics_query = json!({
        "query": r#"
            SELECT (COUNT(DISTINCT ?s) as ?total_subjects) 
                   (COUNT(?triple) as ?total_triples)
                   (COUNT(DISTINCT ?g) as ?total_graphs) WHERE {
                GRAPH ?g { ?s ?p ?o }
                BIND(CONCAT(STR(?s), STR(?p), STR(?o)) as ?triple)
            }
        "#,
        "format": "json"
    });

    let analytics_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&analytics_query)
        .send()
        .await?;

    assert!(
        analytics_response.status().is_success(),
        "Should execute analytics query"
    );
    let analytics_data: serde_json::Value = analytics_response.json().await?;
    assert!(
        analytics_data["result_count"].as_u64().unwrap() >= 0,
        "Should return analytics results"
    );

    println!("✓ Administrator System Management journey completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_browser_ui_complete_workflow() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Browser UI Complete Workflow on {}", base_url);

    let browser = create_browser()?;
    let tab = browser.new_tab()?;

    // Step 1: Navigate to application
    tab.navigate_to(&base_url)?;
    tab.wait_for_element("nav.navbar")?;

    // Step 2: Verify dashboard loads
    let dashboard_element = tab.wait_for_element("#dashboard")?;
    assert!(
        dashboard_element.get_description()?.contains("Dashboard"),
        "Should load dashboard"
    );

    // Step 3: Test navigation
    tab.click_element("a[data-section='blocks']")?;
    tab.wait_for_element("#blocks.content-section.active")?;

    tab.click_element("a[data-section='traceability']")?;
    tab.wait_for_element("#traceability.content-section.active")?;

    tab.click_element("a[data-section='sparql']")?;
    tab.wait_for_element("#sparql.content-section.active")?;

    // Step 4: Test login modal
    tab.click_element("#loginBtn")?;
    tab.wait_for_element("#loginModal")?;

    // Fill login form
    tab.type_into_element("#loginUsername", "testuser")?;
    tab.type_into_element("#loginPassword", "testpass")?;
    tab.click_element("#loginForm button[type='submit']")?;

    // Wait for login to process (may show error, but tests UI interaction)
    sleep(Duration::from_millis(1000)).await;

    // Step 5: Test SPARQL interface
    tab.click_element("a[data-section='sparql']")?;
    tab.wait_for_element("#sparqlQuery")?;

    // Enter a simple query
    tab.type_into_element("#sparqlQuery", "SELECT * WHERE { ?s ?p ?o } LIMIT 10")?;
    tab.click_element("#executeQuery")?;

    // Wait for query execution
    sleep(Duration::from_millis(2000)).await;

    // Step 6: Test traceability search
    tab.click_element("a[data-section='traceability']")?;
    tab.wait_for_element("#batchId")?;

    tab.type_into_element("#batchId", "BATCH001")?;
    tab.click_element("#traceProduct")?;

    // Wait for trace results
    sleep(Duration::from_millis(2000)).await;

    // Step 7: Test transaction form
    tab.click_element("a[data-section='transactions']")?;
    tab.wait_for_element("#addTripleForm")?;

    tab.type_into_element("#subject", ":testSubject")?;
    tab.type_into_element("#predicate", ":testPredicate")?;
    tab.type_into_element("#object", "Test Object")?;

    // Note: Don't submit as it requires authentication, but test form interaction

    println!("✓ Browser UI Complete Workflow completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_user_operations() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Concurrent User Operations on {}", base_url);

    // Create multiple clients for concurrent operations
    let clients: Vec<Client> = (0..5).map(|_| Client::new()).collect();
    let mut handles = Vec::new();

    // Spawn concurrent user operations
    for (i, client) in clients.into_iter().enumerate() {
        let base_url = base_url.clone();
        let handle = tokio::spawn(async move {
            let username = format!("user{}", i);

            // Authenticate
            let token = authenticate_user(&client, &base_url, &username, "password").await?;

            // Add data
            let data = json!({
                "subject": format!("http://example.org/concurrent_batch_{}", i),
                "predicate": "http://provchain.org/trace#hasBatchID",
                "object": format!("CONCURRENT_BATCH_{}", i),
                "graph_name": "concurrent_test"
            });

            let add_response = client
                .post(&format!("{}/api/blockchain/add-triple", base_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&data)
                .send()
                .await?;

            assert!(
                add_response.status().is_success(),
                "Should add data concurrently"
            );

            // Query data
            let query = json!({
                "query": format!("SELECT * WHERE {{ ?s <http://provchain.org/trace#hasBatchID> \"CONCURRENT_BATCH_{}\" }}", i),
                "format": "json"
            });

            let query_response = client
                .post(&format!("{}/api/sparql/query", base_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&query)
                .send()
                .await?;

            assert!(
                query_response.status().is_success(),
                "Should query data concurrently"
            );

            Ok::<(), anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    let start = Instant::now();
    for handle in handles {
        handle.await??;
    }
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_secs(30),
        "Concurrent operations should complete within 30 seconds"
    );

    // Verify final blockchain state
    let client = Client::new();
    let token = authenticate_user(&client, &base_url, "admin", "password").await?;

    let blocks_response = client
        .get(&format!("{}/api/blockchain/blocks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let blocks_data: serde_json::Value = blocks_response.json().await?;
    let blocks = blocks_data.as_array().unwrap();

    // Should have genesis block + 5 concurrent additions
    assert!(
        blocks.len() >= 6,
        "Should have added blocks from concurrent operations"
    );

    println!("✓ Concurrent User Operations completed in {:?}", duration);
    Ok(())
}

#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    println!("Testing Error Handling and Recovery on {}", base_url);

    // Test 1: Invalid authentication
    let invalid_auth_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "invalid",
            "password": "invalid"
        }))
        .send()
        .await?;

    assert_eq!(
        invalid_auth_response.status(),
        401,
        "Should reject invalid credentials"
    );

    // Test 2: Malformed requests
    let malformed_response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .json(&json!({
            "invalid": "data"
        }))
        .send()
        .await?;

    assert!(
        malformed_response.status().is_client_error(),
        "Should reject malformed requests"
    );

    // Test 3: Invalid SPARQL queries
    let token = authenticate_user(&client, &base_url, "user", "password").await?;

    let invalid_query_response = client
        .post(&format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": "INVALID SPARQL SYNTAX",
            "format": "json"
        }))
        .send()
        .await?;

    assert!(
        invalid_query_response.status().is_client_error(),
        "Should reject invalid SPARQL"
    );

    // Test 4: Recovery after errors - valid operations should still work
    let valid_data = json!({
        "subject": "http://example.org/recovery_test",
        "predicate": "http://provchain.org/trace#status",
        "object": "Recovery Test",
        "graph_name": "recovery"
    });

    let recovery_response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&valid_data)
        .send()
        .await?;

    assert!(
        recovery_response.status().is_success(),
        "Should recover and process valid requests"
    );

    // Test 5: System should remain stable
    let status_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(
        status_response.status().is_success(),
        "System should remain stable after errors"
    );

    println!("✓ Error Handling and Recovery completed successfully");
    Ok(())
}
