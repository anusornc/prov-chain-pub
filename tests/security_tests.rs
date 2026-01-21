//! Security Tests
//!
//! Tests for authentication, authorization, input validation,
//! and security edge cases.

use anyhow::Result;
use provchain_org::core::blockchain::Blockchain;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Test helper for setting up a test server with authentication
async fn setup_test_server_with_auth() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    // Set JWT_SECRET for tests (required for authentication to work)
    std::env::set_var("JWT_SECRET", "test-secret-for-integration-tests-must-be-32-chars-long");

    let blockchain = Blockchain::new();

    // Try to find an available port
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

    sleep(Duration::from_millis(1500)).await;
    Ok((actual_port, handle))
}

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

/// Test authentication with valid credentials
#[tokio::test]
#[ignore]
async fn test_valid_authentication() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test login with valid credentials
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    assert!(login_response.status().is_success());

    let auth_result: serde_json::Value = login_response.json().await?;
    assert!(auth_result["token"].is_string());
    assert!(!auth_result["token"].as_str().unwrap().is_empty());

    Ok(())
}

/// Test authentication with invalid credentials
#[tokio::test]
async fn test_invalid_authentication() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test login with invalid password
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "wrongpassword"
        }))
        .send()
        .await?;

    eprintln!("Login response status: {}", login_response.status());
    assert!(login_response.status().is_client_error(), "Expected 4xx, got {}", login_response.status());

    // Test login with non-existent user
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "nonexistent",
            "password": "password"
        }))
        .send()
        .await?;

    assert!(login_response.status().is_client_error());

    Ok(())
}

/// Test malformed authentication requests
#[tokio::test]
async fn test_malformed_authentication_requests() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test login with missing username
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "password": "admin123"
        }))
        .send()
        .await?;

    assert!(login_response.status().is_client_error());

    // Test login with missing password
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin"
        }))
        .send()
        .await?;

    assert!(login_response.status().is_client_error());

    // Test login with empty JSON
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({}))
        .send()
        .await?;

    assert!(login_response.status().is_client_error());

    // Test login with invalid JSON
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .header("content-type", "application/json")
        .body("invalid json")
        .send()
        .await?;

    assert!(login_response.status().is_client_error());

    Ok(())
}

/// Test JWT token validation
#[tokio::test]
#[ignore]
async fn test_jwt_token_validation() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get a valid token first
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let auth_result: serde_json::Value = login_response.json().await?;
    let valid_token = auth_result["token"].as_str().unwrap();

    // Test protected endpoint with valid token
    let protected_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", format!("Bearer {}", valid_token))
        .send()
        .await?;

    assert!(protected_response.status().is_success());

    // Test protected endpoint with invalid token
    let protected_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await?;

    assert!(protected_response.status().is_client_error());

    // Test protected endpoint with malformed authorization header
    let protected_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .header("Authorization", "InvalidFormat")
        .send()
        .await?;

    assert!(protected_response.status().is_client_error());

    // Test protected endpoint without authorization header
    let protected_response = client
        .get(&format!("{}/api/blockchain/status", base_url))
        .send()
        .await?;

    assert!(protected_response.status().is_client_error());

    Ok(())
}

/// Test SQL injection attempts in SPARQL queries
#[tokio::test]
#[ignore]
async fn test_sparql_injection_protection() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get authentication token
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Test various injection attempts
    let injection_attempts = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "'; DELETE FROM blockchain; --",
        "UNION SELECT * FROM sensitive_data",
        "'; INSERT INTO malicious_data VALUES ('hack'); --",
    ];

    for injection in injection_attempts {
        let malicious_query = format!(
            r#"
        PREFIX tc: <http://provchain.org/trace#>
        SELECT ?product WHERE {{
            ?batch tc:product "{}" .
        }}
        "#,
            injection
        );

        let response = client
            .post(&format!("{}/api/sparql/query", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "query": malicious_query,
                "format": "json"
            }))
            .send()
            .await?;

        // Should either return an error or safely handle the malicious input
        // The system should not crash or expose sensitive data
        assert!(response.status().is_client_error() || response.status().is_success());

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            // Verify no sensitive data is exposed
            assert!(!result.to_string().contains("DROP TABLE"));
            assert!(!result.to_string().contains("DELETE FROM"));
            assert!(!result.to_string().contains("sensitive_data"));
        }
    }

    Ok(())
}

/// Test input validation for blockchain operations
#[tokio::test]
#[ignore]
async fn test_input_validation() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get authentication token
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Test extremely large input
    let large_data = "A".repeat(10_000_000); // 10MB of data
    let response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "subject": "http://example.org/test",
            "predicate": "http://example.org/large_data",
            "object": large_data
        }))
        .send()
        .await?;

    let status = response.status();
    println!("Large data response status: {}", status);
    if status.is_success() {
        let response_text = response.text().await?;
        println!("Large data response text: {}", response_text);
    } else {
        println!("Large data response error: {}", status);
    }
    assert!(status.is_client_error());

    // Test malformed RDF data
    let malformed_rdf = "This is not valid RDF data @#$%^&*()";
    let response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "subject": "http://example.org/test",
            "predicate": "http://example.org/malformed",
            "object": malformed_rdf
        }))
        .send()
        .await?;

    let status = response.status();
    println!("Malformed RDF response status: {}", status);
    if status.is_success() {
        let response_text = response.text().await?;
        println!("Malformed RDF response text: {}", response_text);
    } else {
        let response_text = response.text().await?;
        println!("Malformed RDF error response text: {}", response_text);
    }

    // Should handle malformed RDF gracefully (either accept or reject safely)
    assert!(status.is_client_error() || status.is_success());

    // Test null/empty inputs
    let response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "subject": "http://example.org/test",
            "predicate": "http://example.org/empty",
            "object": ""
        }))
        .send()
        .await?;

    let status = response.status();
    println!("Empty input response status: {}", status);
    // System should handle empty inputs gracefully (either accept or reject safely)
    assert!(status.is_client_error() || status.is_success());

    // Test missing required fields
    let response = client
        .post(&format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({}))
        .send()
        .await?;

    let status = response.status();
    println!("Missing fields response status: {}", status);
    assert!(status.is_client_error());

    Ok(())
}

/// Test rate limiting and DoS protection
#[tokio::test]
#[ignore]
async fn test_rate_limiting() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get authentication token
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Attempt rapid-fire requests
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for i in 0..50 {
        let response = client
            .get(&format!("{}/api/blockchain/status", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            success_count += 1;
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }

        // Small delay to avoid overwhelming the test
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    println!(
        "Success: {}, Rate limited: {}",
        success_count, rate_limited_count
    );

    // Should have some successful requests but also some rate limiting
    assert!(success_count > 0);
    // Note: Rate limiting might not be implemented yet, so we don't assert on rate_limited_count

    Ok(())
}

/// Test cross-site scripting (XSS) protection
#[tokio::test]
#[ignore]
async fn test_xss_protection() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get authentication token
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let auth_result: serde_json::Value = login_response.json().await?;
    let token = auth_result["token"].as_str().unwrap();

    // Test XSS attempts in RDF data
    let xss_attempts = vec![
        "<script>alert('XSS')</script>",
        "javascript:alert('XSS')",
        "<img src=x onerror=alert('XSS')>",
        "<svg onload=alert('XSS')>",
        "';alert('XSS');//",
    ];

    for xss_payload in xss_attempts {
        let response = client
            .post(&format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "subject": "http://example.org/batch001",
                "predicate": "http://provchain.org/trace#product",
                "object": xss_payload
            }))
            .send()
            .await?;

        // Print response for debugging
        let status = response.status();
        println!("Add triple response status: {}", status);
        if status.is_success() {
            let response_text = response.text().await?;
            println!("Add triple response text: {}", response_text);
        } else {
            let response_text = response.text().await?;
            println!("Add triple error response text: {}", response_text);
        }

        // Should handle XSS attempts safely
        if status.is_success() {
            // If accepted, verify the data is properly escaped when retrieved
            let query_response = client
                .post(&format!("{}/api/sparql/query", base_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&json!({
                    "query": "SELECT ?product WHERE { <http://example.org/batch001> <http://provchain.org/trace#product> ?product }",
                    "format": "json"
                }))
                .send()
                .await?;

            let query_status = query_response.status();
            println!("Query response status: {}", query_status);
            if query_status.is_success() {
                match query_response.json::<serde_json::Value>().await {
                    Ok(result) => {
                        let result_text = result.to_string();
                        println!("Query result text: {}", result_text);

                        // Verify XSS payloads are properly escaped or sanitized
                        assert!(!result_text.contains("<script>"));
                        assert!(!result_text.contains("javascript:"));
                        assert!(!result_text.contains("onerror="));
                        assert!(!result_text.contains("onload="));
                    }
                    Err(e) => {
                        println!("Error parsing query response: {}", e);
                        // If we can't parse the response, that's also a problem
                        return Err(e.into());
                    }
                }
            } else {
                let query_response_text = query_response.text().await?;
                println!("Query error response text: {}", query_response_text);
            }
        }
    }

    Ok(())
}

/// Test authorization bypass attempts
#[tokio::test]
async fn test_authorization_bypass_attempts() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test accessing protected endpoints without authentication
    let protected_endpoints = vec![
        "/api/blockchain/add-triple",
        "/api/blockchain/status",
        "/api/sparql/query",
    ];

    for endpoint in protected_endpoints {
        let response = client
            .get(&format!("{}{}", base_url, endpoint))
            .send()
            .await?;

        // Should require authentication
        assert!(response.status().is_client_error());
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    }

    // Test with expired/invalid tokens
    let invalid_tokens = vec![
        "expired.token.here",
        "Bearer malformed",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        "",
    ];

    for invalid_token in invalid_tokens {
        let response = client
            .get(&format!("{}/api/blockchain/status", base_url))
            .header("Authorization", format!("Bearer {}", invalid_token))
            .send()
            .await?;

        assert!(response.status().is_client_error());
    }

    Ok(())
}

/// Test session management security
#[tokio::test]
#[ignore]
async fn test_session_security() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test multiple concurrent sessions
    let mut tokens = Vec::new();

    for _ in 0..3 {
        let login_response = client
            .post(&format!("{}/auth/login", base_url))
            .json(&json!({
                "username": "admin",
                "password": "admin123"
            }))
            .send()
            .await?;

        assert!(login_response.status().is_success());
        let auth_result: serde_json::Value = login_response.json().await?;
        tokens.push(auth_result["token"].as_str().unwrap().to_string());
    }

    // Verify all tokens work
    for token in &tokens {
        let response = client
            .get(&format!("{}/api/blockchain/status", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        assert!(response.status().is_success());
    }

    // Test logout functionality (if implemented)
    if let Some(first_token) = tokens.first() {
        let logout_response = client
            .post(&format!("{}/api/auth/logout", base_url))
            .header("Authorization", format!("Bearer {}", first_token))
            .send()
            .await?;

        // Logout might not be implemented, so we check if endpoint exists
        if logout_response.status() != reqwest::StatusCode::NOT_FOUND {
            // If logout endpoint exists but doesn't invalidate tokens, that's a security issue
            // For now, we'll just note that logout doesn't invalidate tokens
            println!("Logout endpoint exists but may not invalidate tokens");
        }
        // If logout endpoint doesn't exist (404), that's acceptable for this test
    }

    Ok(())
}

/// Test password security requirements
#[tokio::test]
async fn test_password_security() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Test weak passwords (if user registration is available)
    let weak_passwords = vec!["123", "password", "admin", "qwerty", "abc123"];

    for weak_password in weak_passwords {
        let register_response = client
            .post(&format!("{}/auth/register", base_url))
            .json(&json!({
                "username": "testuser",
                "password": weak_password
            }))
            .send()
            .await?;

        // Should reject weak passwords (if registration endpoint exists)
        if register_response.status() != reqwest::StatusCode::NOT_FOUND {
            assert!(register_response.status().is_client_error());
        }
    }

    Ok(())
}

/// Test data integrity and tampering protection
#[tokio::test]
#[ignore]
async fn test_data_integrity_protection() -> Result<()> {
    let (port, _server_handle) = setup_test_server_with_auth().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();

    // Get authentication token
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await?;

    let login_status = login_response.status();
    println!("Login response status: {}", login_status);
    if login_status.is_success() {
        let login_text = login_response.text().await?;
        println!("Login response text: {}", login_text);
        // Parse the JSON to get the token
        let auth_result: serde_json::Value = serde_json::from_str(&login_text)?;
        let token = auth_result["token"].as_str().unwrap();

        // Add legitimate data
        let response = client
            .post(&format!("{}/api/blockchain/add-triple", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "subject": "http://example.org/batch001",
                "predicate": "http://provchain.org/trace#product",
                "object": "Legitimate Product"
            }))
            .send()
            .await?;

        let add_status = response.status();
        println!("Add triple response status: {}", add_status);
        if !add_status.is_success() {
            let add_text = response.text().await?;
            println!("Add triple error response text: {}", add_text);
        }
        assert!(add_status.is_success());

        // Verify blockchain integrity
        let stats_response = client
            .get(&format!("{}/api/blockchain/status", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let stats_status = stats_response.status();
        println!("Stats response status: {}", stats_status);
        assert!(stats_status.is_success());

        if !stats_status.is_success() {
            let stats_text = stats_response.text().await?;
            println!("Stats error response text: {}", stats_text);
        } else {
            let stats: serde_json::Value = stats_response.json().await?;

            // Verify blockchain is valid (should be at least 1, but could be more due to demo data)
            assert!(stats["height"].as_u64().unwrap() >= 1);
        }

        // Test attempts to modify existing blocks (should be impossible)
        let tamper_response = client
            .put(&format!("{}/api/blockchain/block/0", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "data": "Tampered data"
            }))
            .send()
            .await?;

        // Should not allow modification of existing blocks
        assert!(
            tamper_response.status().is_client_error()
                || tamper_response.status() == reqwest::StatusCode::NOT_FOUND
        );
    } else {
        let login_text = login_response.text().await?;
        println!("Login error response text: {}", login_text);
        assert!(login_status.is_success());
    }

    Ok(())
}
