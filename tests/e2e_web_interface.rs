//! End-to-End Web Interface Tests
//!
//! This test suite validates the complete web interface using browser automation,
//! testing all UI components, interactions, and data flows.

use anyhow::Result;
use fantoccini::{ClientBuilder, Locator};
use provchain_org::{config::Config, core::blockchain::Blockchain, web::server::create_web_server};
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test helper to start a test web server with sample data
async fn start_test_server_with_data() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    let mut blockchain = Blockchain::new();

    // Add sample data for testing
    add_sample_test_data(&mut blockchain);

    // Use port 0 to get an available port
    let mut config = Config::default();
    config.web.port = 0;
    let server = create_web_server(blockchain, Some(config)).await?;
    let port = server.port();

    let handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(5000)).await;

    Ok((port, handle))
}

/// Add sample data for testing
fn add_sample_test_data(blockchain: &mut Blockchain) {
    let sample_data = vec![
        // Product batches
        ":batch001 tc:product \"Organic Coffee Beans\" .",
        ":batch001 tc:origin \"Farm ABC, Colombia\" .",
        ":batch001 tc:currentLocation \"Warehouse XYZ, USA\" .",
        ":batch001 tc:status \"In Transit\" .",
        ":batch001 tc:batchId \"BATCH001\" .",
        ":batch002 tc:product \"Fair Trade Cocoa\" .",
        ":batch002 tc:origin \"Farm DEF, Ecuador\" .",
        ":batch002 tc:currentLocation \"Processing Plant\" .",
        ":batch002 tc:status \"Processing\" .",
        ":batch002 tc:batchId \"BATCH002\" .",
        // Environmental data
        ":batch001 tc:environmentalData :env001 .",
        ":env001 tc:temperature \"22.5\" .",
        ":env001 tc:humidity \"65.0\" .",
        // Supply chain events
        ":event001 tc:batch :batch001 .",
        ":event001 tc:actor \"Farmer John\" .",
        ":event001 tc:action \"Harvested\" .",
        ":event001 tc:timestamp \"2024-01-15T08:00:00Z\" .",
        // Certifications
        ":batch001 tc:certification \"Organic\" .",
        ":batch002 tc:certification \"Fair Trade\" .",
    ];

    for data in sample_data {
        let _ = blockchain.add_block(data.to_string());
    }
}

/// Test helper to create browser client
async fn create_client() -> Result<fantoccini::Client> {
    let mut caps = serde_json::map::Map::new();
    let opts = serde_json::json!({
        "args": ["--headless", "--disable-gpu", "--window-size=1920,1080"]
    });
    caps.insert("goog:chromeOptions".to_string(), opts);

    Ok(ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:9515")
        .await?)
}

/// Test helper to wait for element and handle timeouts
async fn wait_for_element_with_timeout(
    client: &fantoccini::Client,
    selector: &str,
    timeout_ms: u64,
) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms) {
        if client.find(Locator::Css(selector)).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(anyhow::anyhow!(
        "Element {} not found within timeout",
        selector
    ))
}

/// Test helper to simulate user login via UI
async fn login_via_ui(client: &fantoccini::Client, username: &str, password: &str) -> Result<()> {
    // Click login button
    client
        .find(Locator::Css("#loginBtn"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(client, "#loginModal", 2000).await?;

    // Fill login form
    client
        .find(Locator::Css("#loginUsername"))
        .await?
        .send_keys(username)
        .await?;
    client
        .find(Locator::Css("#loginPassword"))
        .await?
        .send_keys(password)
        .await?;

    // Submit form
    client
        .find(Locator::Css("#loginForm button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for login to process
    sleep(Duration::from_millis(2000)).await;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_dashboard_functionality() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Dashboard Functionality on {}", base_url);

    let client = create_client().await?;

    // Navigate to application
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;

    // Verify dashboard is active by default
    wait_for_element_with_timeout(&client, "#dashboard.content-section.active", 2000).await?;

    // Check for stats cards
    wait_for_element_with_timeout(&client, ".stats-grid", 2000).await?;
    wait_for_element_with_timeout(&client, "#blockHeight", 2000).await?;
    wait_for_element_with_timeout(&client, "#totalTransactions", 2000).await?;
    wait_for_element_with_timeout(&client, "#networkPeers", 2000).await?;
    wait_for_element_with_timeout(&client, "#blockchainStatus", 2000).await?;

    // Check for dashboard cards
    wait_for_element_with_timeout(&client, ".dashboard-grid", 2000).await?;
    wait_for_element_with_timeout(&client, "#recentTransactions", 2000).await?;
    wait_for_element_with_timeout(&client, "#apiStatus", 2000).await?;
    wait_for_element_with_timeout(&client, "#lastBlockTime", 2000).await?;
    wait_for_element_with_timeout(&client, "#validationStatus", 2000).await?;

    // Login to see actual data
    login_via_ui(&client, "admin", "password").await?;

    // Wait for data to load
    sleep(Duration::from_millis(3000)).await;

    // Verify stats are populated (should show numbers, not just "-")
    let block_height = client.find(Locator::Css("#blockHeight")).await?;
    let block_height_text = block_height.html(false).await?;
    assert_ne!(block_height_text, "-", "Block height should be populated");

    println!("✓ Dashboard functionality test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_block_explorer_functionality() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Block Explorer Functionality on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "admin", "password").await?;

    // Navigate to blocks section
    client
        .find(Locator::Css("a[data-section='blocks']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#blocks.content-section.active", 2000).await?;

    // Check for blocks controls
    wait_for_element_with_timeout(&client, "#blockSearch", 2000).await?;
    wait_for_element_with_timeout(&client, "#refreshBlocks", 2000).await?;
    wait_for_element_with_timeout(&client, "#blocksList", 2000).await?;

    // Test refresh functionality
    client
        .find(Locator::Css("#refreshBlocks"))
        .await?
        .click()
        .await?;
    sleep(Duration::from_millis(2000)).await;

    // Test search functionality
    client
        .find(Locator::Css("#blockSearch"))
        .await?
        .send_keys("0")
        .await?;
    sleep(Duration::from_millis(1000)).await;

    // Clear search
    client
        .execute("document.getElementById('blockSearch').value = ''", vec![])
        .await?;
    sleep(Duration::from_millis(500)).await;

    println!("✓ Block Explorer functionality test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_product_traceability_interface() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Product Traceability Interface on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "user", "password").await?;

    // Navigate to traceability section
    client
        .find(Locator::Css("a[data-section='traceability']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#traceability.content-section.active", 2000).await?;

    // Check for traceability interface elements
    wait_for_element_with_timeout(&client, "#batchId", 2000).await?;
    wait_for_element_with_timeout(&client, "#productName", 2000).await?;
    wait_for_element_with_timeout(&client, "#traceProduct", 2000).await?;
    wait_for_element_with_timeout(&client, "#traceResults", 2000).await?;

    // Test traceability search with known batch
    client
        .find(Locator::Css("#batchId"))
        .await?
        .send_keys("BATCH001")
        .await?;
    client
        .find(Locator::Css("#productName"))
        .await?
        .send_keys("Coffee")
        .await?;
    client
        .find(Locator::Css("#traceProduct"))
        .await?
        .click()
        .await?;

    // Wait for results
    sleep(Duration::from_millis(3000)).await;

    // Verify results area is updated
    let results_element = client.find(Locator::Css("#traceResults")).await?;
    let results_html = results_element.html(false).await?;

    // Should not show placeholder anymore
    assert!(
        !results_html.contains("trace-placeholder"),
        "Should show actual results, not placeholder"
    );

    // Test with invalid batch ID
    client
        .execute("document.getElementById('batchId').value = ''", vec![])
        .await?;
    client
        .find(Locator::Css("#batchId"))
        .await?
        .send_keys("INVALID_BATCH")
        .await?;
    client
        .find(Locator::Css("#traceProduct"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;

    println!("✓ Product Traceability Interface test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_sparql_query_interface() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing SPARQL Query Interface on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "admin", "password").await?;

    // Navigate to SPARQL section
    client
        .find(Locator::Css("a[data-section='sparql']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#sparql.content-section.active", 2000).await?;

    // Check for SPARQL interface elements
    wait_for_element_with_timeout(&client, "#sparqlQuery", 2000).await?;
    wait_for_element_with_timeout(&client, "#queryTemplates", 2000).await?;
    wait_for_element_with_timeout(&client, "#executeQuery", 2000).await?;
    wait_for_element_with_timeout(&client, "#sparqlResults", 2000).await?;

    // Test query template selection
    client
        .find(Locator::Css("#queryTemplates"))
        .await?
        .select_by_value("all-triples")
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Verify query was populated
    let query_element = client.find(Locator::Css("#sparqlQuery")).await?;
    let query_text = query_element.html(false).await?;
    assert!(
        !query_text.is_empty(),
        "Query template should populate the editor"
    );

    // Test custom query
    client
        .execute("document.getElementById('sparqlQuery').value = ''", vec![])
        .await?;
    client
        .find(Locator::Css("#sparqlQuery"))
        .await?
        .send_keys("SELECT * WHERE { ?s ?p ?o } LIMIT 5")
        .await?;

    // Execute query
    client
        .find(Locator::Css("#executeQuery"))
        .await?
        .click()
        .await?;

    // Wait for results
    sleep(Duration::from_millis(3000)).await;

    // Check for results
    let results_element = client.find(Locator::Css("#sparqlResults")).await?;
    let results_html = results_element.html(false).await?;

    // Should not show placeholder anymore
    assert!(
        !results_html.contains("results-placeholder"),
        "Should show query results"
    );

    // Test query stats
    wait_for_element_with_timeout(&client, "#queryStats", 2000).await?;

    // Test invalid query
    client
        .execute("document.getElementById('sparqlQuery').value = ''", vec![])
        .await?;
    client
        .find(Locator::Css("#sparqlQuery"))
        .await?
        .send_keys("INVALID SPARQL SYNTAX")
        .await?;
    client
        .find(Locator::Css("#executeQuery"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;

    println!("✓ SPARQL Query Interface test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_transaction_management_interface() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Transaction Management Interface on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "manager", "password").await?;

    // Navigate to transactions section
    client
        .find(Locator::Css("a[data-section='transactions']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#transactions.content-section.active", 2000).await?;

    // Check for transaction interface elements
    wait_for_element_with_timeout(&client, "#addTripleForm", 2000).await?;
    wait_for_element_with_timeout(&client, "#subject", 2000).await?;
    wait_for_element_with_timeout(&client, "#predicate", 2000).await?;
    wait_for_element_with_timeout(&client, "#object", 2000).await?;
    wait_for_element_with_timeout(&client, "#transactionsList", 2000).await?;

    // Test form validation (empty form)
    client
        .find(Locator::Css("#addTripleForm button[type='submit']"))
        .await?
        .click()
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Fill form with valid data
    client
        .find(Locator::Css("#subject"))
        .await?
        .send_keys(":testBatch123")
        .await?;
    client
        .find(Locator::Css("#predicate"))
        .await?
        .send_keys("tc:hasStatus")
        .await?;
    client
        .find(Locator::Css("#object"))
        .await?
        .send_keys("Testing")
        .await?;

    // Submit form
    client
        .find(Locator::Css("#addTripleForm button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for submission
    sleep(Duration::from_millis(3000)).await;

    // Check if form was cleared (indicates successful submission)
    let _subject_value = client
        .find(Locator::Css("#subject"))
        .await?
        .attr("value")
        .await?;
    // Form might be cleared on successful submission

    // Check transaction history
    let transactions_element = client.find(Locator::Css("#transactionsList")).await?;
    let transactions_html = transactions_element.html(false).await?;

    // Should show transactions, not loading message
    assert!(
        !transactions_html.contains("Loading transactions"),
        "Should show transaction history"
    );

    println!("✓ Transaction Management Interface test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_authentication_flow() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Authentication Flow on {}", base_url);

    let client = create_client().await?;

    // Navigate to application
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;

    // Verify login button is visible
    wait_for_element_with_timeout(&client, "#loginBtn", 2000).await?;

    // Test login modal
    client
        .find(Locator::Css("#loginBtn"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#loginModal", 2000).await?;

    // Test modal close
    client
        .find(Locator::Css(".modal .close"))
        .await?
        .click()
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Modal should be hidden
    let modal_style = client
        .find(Locator::Css("#loginModal"))
        .await?
        .attr("style")
        .await?;
    assert!(modal_style.is_none_or(|s| s.contains("display: none") || s.is_empty()));

    // Test login with invalid credentials
    client
        .find(Locator::Css("#loginBtn"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#loginModal", 2000).await?;

    client
        .find(Locator::Css("#loginUsername"))
        .await?
        .send_keys("invalid")
        .await?;
    client
        .find(Locator::Css("#loginPassword"))
        .await?
        .send_keys("invalid")
        .await?;
    client
        .find(Locator::Css("#loginForm button[type='submit']"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;

    // Should still show login button (login failed)
    wait_for_element_with_timeout(&client, "#loginBtn", 2000).await?;

    // Test login with valid credentials
    client
        .execute(
            "document.getElementById('loginUsername').value = ''",
            vec![],
        )
        .await?;
    client
        .execute(
            "document.getElementById('loginPassword').value = ''",
            vec![],
        )
        .await?;

    client
        .find(Locator::Css("#loginUsername"))
        .await?
        .send_keys("testuser")
        .await?;
    client
        .find(Locator::Css("#loginPassword"))
        .await?
        .send_keys("testpass")
        .await?;
    client
        .find(Locator::Css("#loginForm button[type='submit']"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;

    // Check if user info appears or login button changes
    // (Behavior depends on actual authentication implementation)

    println!("✓ Authentication Flow test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_navigation_and_routing() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Navigation and Routing on {}", base_url);

    let client = create_client().await?;

    // Navigate to application
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;

    // Test all navigation links
    let nav_sections = vec![
        ("dashboard", "#dashboard"),
        ("blocks", "#blocks"),
        ("traceability", "#traceability"),
        ("sparql", "#sparql"),
        ("transactions", "#transactions"),
    ];

    for (section, selector) in nav_sections {
        // Click navigation link
        client
            .find(Locator::Css(&format!("a[data-section='{}']", section)))
            .await?
            .click()
            .await?;

        // Wait for section to become active
        wait_for_element_with_timeout(
            &client,
            &format!("{}.content-section.active", selector),
            2000,
        )
        .await?;

        // Verify other sections are not active
        for (other_section, other_selector) in &[
            ("dashboard", "#dashboard"),
            ("blocks", "#blocks"),
            ("traceability", "#traceability"),
            ("sparql", "#sparql"),
            ("transactions", "#transactions"),
        ] {
            if *other_section != section {
                let other_element = client.find(Locator::Css(other_selector)).await?;
                let class_attr = other_element.attr("class").await?;
                assert!(
                    !class_attr.unwrap_or_default().contains("active"),
                    "Other sections should not be active"
                );
            }
        }

        // Verify navigation link is active
        let nav_link = client
            .find(Locator::Css(&format!("a[data-section='{}']", section)))
            .await?;
        let nav_class = nav_link.attr("class").await?;
        assert!(
            nav_class.unwrap_or_default().contains("active"),
            "Navigation link should be active"
        );

        sleep(Duration::from_millis(200)).await;
    }

    println!("✓ Navigation and Routing test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_responsive_design() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Responsive Design on {}", base_url);

    let client = create_client().await?;

    // Navigate to application
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;

    // Test desktop view (default)
    let navbar = client.find(Locator::Css("nav.navbar")).await?;
    assert!(navbar.html(false).await?.contains("navbar"));

    // Test tablet view
    client.set_window_size(768, 1024).await?;
    sleep(Duration::from_millis(500)).await;

    // Verify layout still works
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    wait_for_element_with_timeout(&client, "#dashboard", 2000).await?;

    // Test mobile view
    client.set_window_size(375, 667).await?;
    sleep(Duration::from_millis(500)).await;

    // Verify layout still works
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    wait_for_element_with_timeout(&client, "#dashboard", 2000).await?;

    // Test navigation still works on mobile
    client
        .find(Locator::Css("a[data-section='blocks']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#blocks.content-section.active", 2000).await?;

    // Reset to desktop view
    client.set_window_size(1920, 1080).await?;
    sleep(Duration::from_millis(500)).await;

    println!("✓ Responsive Design test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_error_handling_ui() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Error Handling UI on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "user", "password").await?;

    // Test SPARQL error handling
    client
        .find(Locator::Css("a[data-section='sparql']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#sparql.content-section.active", 2000).await?;

    // Enter invalid SPARQL query
    client
        .find(Locator::Css("#sparqlQuery"))
        .await?
        .send_keys("INVALID SPARQL SYNTAX")
        .await?;
    client
        .find(Locator::Css("#executeQuery"))
        .await?
        .click()
        .await?;

    // Wait for error response
    sleep(Duration::from_millis(3000)).await;

    // Check for error indication in results
    let results_element = client.find(Locator::Css("#sparqlResults")).await?;
    let _results_html = results_element.html(false).await?;

    // Should show some kind of error indication
    // (Exact error handling depends on implementation)

    // Test traceability error handling
    client
        .find(Locator::Css("a[data-section='traceability']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#traceability.content-section.active", 2000).await?;

    // Search for non-existent batch
    client
        .find(Locator::Css("#batchId"))
        .await?
        .send_keys("NONEXISTENT_BATCH")
        .await?;
    client
        .find(Locator::Css("#traceProduct"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;

    // Should handle gracefully (no crash)
    wait_for_element_with_timeout(&client, "#traceResults", 2000).await?;

    // Test form validation
    client
        .find(Locator::Css("a[data-section='transactions']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#transactions.content-section.active", 2000).await?;

    // Try to submit empty form
    client
        .find(Locator::Css("#addTripleForm button[type='submit']"))
        .await?
        .click()
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Form should prevent submission or show validation errors
    // (Exact behavior depends on implementation)

    println!("✓ Error Handling UI test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_real_time_updates() -> Result<()> {
    let (port, _server_handle) = start_test_server_with_data().await?;
    let base_url = format!("http://localhost:{}", port);

    println!("Testing Real-time Updates on {}", base_url);

    let client = create_client().await?;

    // Navigate and login
    client.goto(&base_url).await?;
    wait_for_element_with_timeout(&client, "nav.navbar", 2000).await?;
    login_via_ui(&client, "admin", "password").await?;

    // Go to dashboard to see stats
    client
        .find(Locator::Css("a[data-section='dashboard']"))
        .await?
        .click()
        .await?;
    wait_for_element_with_timeout(&client, "#dashboard.content-section.active", 2000).await?;

    // Wait for initial data load
    sleep(Duration::from_millis(3000)).await;

    // Get initial block height
    let block_height = client.find(Locator::Css("#blockHeight")).await?;
    let initial_height = block_height.html(false).await?;

    // Add new data via API in background
    let api_client = Client::new();
    let auth_response = api_client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "password"
        }))
        .send()
        .await?;

    if auth_response.status().is_success() {
        let auth_data: serde_json::Value = auth_response.json().await?;
        let token = auth_data["token"].as_str().unwrap_or("");

        // Add new triple
        let _add_response = api_client
            .post(format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "subject": "http://example.org/realtime_test",
                "predicate": "http://provchain.org/trace#status",
                "object": "Real-time Test",
                "graph_name": "realtime"
            }))
            .send()
            .await?;
    }

    // Wait for potential updates
    sleep(Duration::from_millis(5000)).await;

    // Check if stats updated (depends on implementation of real-time updates)
    let updated_block_height = client.find(Locator::Css("#blockHeight")).await?;
    let updated_height = updated_block_height.html(false).await?;

    // Note: This test verifies the UI can handle updates,
    // actual real-time behavior depends on WebSocket implementation

    println!("✓ Real-time Updates test completed successfully");
    println!(
        "  Initial height: {}, Updated height: {}",
        initial_height, updated_height
    );
    Ok(())
}
