//! Contract tests for the exact SPARQL queries used by the benchmark toolkit.

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

#[path = "../benchmark-toolkit/research-benchmarks/src/workloads/provchain_queries.rs"]
mod provchain_queries;

use provchain_queries::{
    aggregation_by_producer_query, entity_lookup_query, multi_hop_query,
};

const TEST_JWT_SECRET: &str = "test-jwt-secret-key-min-32-chars-for-benchmark-queries";
const TEST_BOOTSTRAP_TOKEN: &str = "test-bootstrap-token-for-benchmark-queries";
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

    assert_eq!(response.status(), StatusCode::OK);
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

    assert_eq!(response.status(), StatusCode::OK);
    Ok(response.json::<AuthResponse>().await?)
}

async fn add_triple(
    client: &Client,
    base_url: &str,
    token: &str,
    subject: &str,
    predicate: &str,
    object: &str,
) -> Result<()> {
    let response = client
        .post(format!("{}/api/blockchain/add-triple", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "subject": subject,
            "predicate": predicate,
            "object": object
        }))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

async fn query_sparql(
    client: &Client,
    base_url: &str,
    token: &str,
    query: String,
) -> Result<SparqlQueryResponse> {
    let response = client
        .post(format!("{}/api/sparql/query", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "query": query,
            "format": "json"
        }))
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;
    assert_eq!(
        status,
        StatusCode::OK,
        "benchmark query should execute successfully: {}",
        body
    );

    Ok(serde_json::from_str::<SparqlQueryResponse>(&body)?)
}

async fn seed_minimal_benchmark_graph(client: &Client, base_url: &str, token: &str) -> Result<()> {
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Product/BATCH001",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
        "http://example.org/supplychain/Product",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Product/BATCH001",
        "http://example.org/supplychain/batchId",
        "BATCH001",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Product/BATCH001",
        "http://example.org/traceability#hasTransaction",
        "http://example.org/supplychain/Transaction/TX001",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Transaction/TX001",
        "http://example.org/traceability#nextTransaction",
        "http://example.org/supplychain/Transaction/TX002",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Transaction/TX002",
        "http://example.org/traceability#nextTransaction",
        "http://example.org/supplychain/Transaction/TX003",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Product/BATCH001",
        "http://example.org/traceability#hasProducer",
        "http://example.org/supplychain/Producer/Farm001",
    )
    .await?;
    add_triple(
        client,
        base_url,
        token,
        "http://example.org/supplychain/Transaction/TX001",
        "http://example.org/traceability#quantity",
        "155.0",
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_benchmark_entity_lookup_query_contract() -> Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = Client::new();

    bootstrap_admin(&client, &base_url).await?;
    let auth = login_admin(&client, &base_url).await?;
    seed_minimal_benchmark_graph(&client, &base_url, &auth.token).await?;

    let response =
        query_sparql(&client, &base_url, &auth.token, entity_lookup_query("BATCH001")).await?;
    assert_eq!(response.result_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_benchmark_multi_hop_query_contract() -> Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = Client::new();

    bootstrap_admin(&client, &base_url).await?;
    let auth = login_admin(&client, &base_url).await?;
    seed_minimal_benchmark_graph(&client, &base_url, &auth.token).await?;

    let response =
        query_sparql(&client, &base_url, &auth.token, multi_hop_query("BATCH001")).await?;
    assert!(response.result_count >= 1);

    Ok(())
}

#[tokio::test]
async fn test_benchmark_aggregation_query_contract() -> Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = Client::new();

    bootstrap_admin(&client, &base_url).await?;
    let auth = login_admin(&client, &base_url).await?;
    seed_minimal_benchmark_graph(&client, &base_url, &auth.token).await?;

    let response =
        query_sparql(&client, &base_url, &auth.token, aggregation_by_producer_query()).await?;
    assert_eq!(response.result_count, 1);

    Ok(())
}
