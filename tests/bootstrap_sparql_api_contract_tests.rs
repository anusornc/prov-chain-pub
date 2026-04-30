//! Black-box API contract tests for bootstrap, login, and authenticated SPARQL access.

use anyhow::Result;
use provchain_org::{
    config::Config,
    core::blockchain::Blockchain,
    web::{
        models::{AuthResponse, SparqlQueryResponse},
        server::create_web_server,
    },
};
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::{
    net::TcpListener,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tokio::time::sleep;

const TEST_JWT_SECRET: &str = "test-jwt-secret-key-min-32-chars-for-api-contract";
const TEST_BOOTSTRAP_TOKEN: &str = "test-bootstrap-token-for-api-contract";
const ADMIN_USERNAME: &str = "adminroot";
const ADMIN_PASSWORD: &str = "AdminRootPassword123!";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

async fn start_test_server() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);
    std::env::set_var("PROVCHAIN_BOOTSTRAP_TOKEN", TEST_BOOTSTRAP_TOKEN);

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let blockchain = Blockchain::new();
    let mut config = Config::default();
    config.web.port = port;

    let server = create_web_server(blockchain, Some(config)).await?;
    let handle = tokio::spawn(async move {
        if let Err(error) = server.start().await {
            eprintln!("Server error: {}", error);
        }
    });

    sleep(Duration::from_millis(1500)).await;

    Ok((port, handle))
}

async fn bootstrap_admin(client: &Client, base_url: &str) -> Result<AuthResponse> {
    let response = client
        .post(format!("{}/auth/bootstrap", base_url))
        .json(&json!({
            "username": ADMIN_USERNAME,
            "password": ADMIN_PASSWORD,
            "bootstrap_token": TEST_BOOTSTRAP_TOKEN
        }))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "bootstrap should succeed"
    );

    Ok(response.json::<AuthResponse>().await?)
}

async fn login_admin(client: &Client, base_url: &str) -> Result<AuthResponse> {
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&json!({
            "username": ADMIN_USERNAME,
            "password": ADMIN_PASSWORD
        }))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK, "login should succeed");

    Ok(response.json::<AuthResponse>().await?)
}

#[tokio::test]
async fn test_bootstrap_login_and_prefixed_sparql_query_contract() -> Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = Client::new();

    let bootstrap_response = bootstrap_admin(&client, &base_url).await?;
    assert!(!bootstrap_response.token.is_empty());
    assert_eq!(bootstrap_response.user_role, "admin");

    let login_response = login_admin(&client, &base_url).await?;
    assert!(!login_response.token.is_empty());
    assert_eq!(login_response.user_role, "admin");

    let add_triple_response = client
        .post(format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", login_response.token))
        .json(&json!({
            "subject": "http://example.org/product-001",
            "predicate": "http://example.org/name",
            "object": "Product 001"
        }))
        .send()
        .await?;

    assert_eq!(
        add_triple_response.status(),
        StatusCode::OK,
        "authenticated triple creation should succeed"
    );

    let query_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", login_response.token))
        .json(&json!({
            "query": r#"
                PREFIX ex: <http://example.org/>
                SELECT ?value WHERE {
                    GRAPH ?g {
                        <http://example.org/product-001> ex:name ?value .
                    }
                }
            "#,
            "format": "json"
        }))
        .send()
        .await?;

    assert_eq!(
        query_response.status(),
        StatusCode::OK,
        "authenticated prefixed SPARQL query should succeed"
    );

    let query_payload = query_response.json::<SparqlQueryResponse>().await?;
    assert_eq!(query_payload.result_count, 1);

    let value = query_payload.results["results"]["bindings"][0]["value"]["value"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(value, "Product 001");

    Ok(())
}

#[tokio::test]
async fn test_sparql_query_requires_authentication() -> Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = Client::new();

    let bootstrap_response = bootstrap_admin(&client, &base_url).await?;
    assert!(!bootstrap_response.token.is_empty());

    let query_response = client
        .post(format!("{}/api/sparql/query", base_url))
        .json(&json!({
            "query": "SELECT ?s WHERE { ?s ?p ?o }",
            "format": "json"
        }))
        .send()
        .await?;

    assert_eq!(
        query_response.status(),
        StatusCode::UNAUTHORIZED,
        "SPARQL query without bearer token must be rejected"
    );

    Ok(())
}
