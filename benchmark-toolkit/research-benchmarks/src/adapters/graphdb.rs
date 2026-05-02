use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
use crate::core::result::TraceQueryResult;
use crate::dataset::normalize_turtle;
use crate::workloads::provchain_queries::{
    aggregation_by_producer_query, entity_lookup_query, multi_hop_query,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const DEFAULT_GRAPH_IRI: &str = "http://provchain.org/benchmark/graphdb/default-graph";
const MULTIPART_BOUNDARY: &str = "provchain-graphdb-repository-config";

#[derive(Debug, Clone)]
pub struct GraphDbConfig {
    pub base_url: String,
    pub repository: String,
    pub timeout_secs: u64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub graph_iri: String,
}

impl Default for GraphDbConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:17200".to_string(),
            repository: "provchain_smoke".to_string(),
            timeout_secs: 30,
            username: None,
            password: None,
            graph_iri: DEFAULT_GRAPH_IRI.to_string(),
        }
    }
}

impl GraphDbConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("GRAPHDB_URL")
                .unwrap_or_else(|_| "http://localhost:17200".to_string()),
            repository: std::env::var("GRAPHDB_REPOSITORY")
                .unwrap_or_else(|_| "provchain_smoke".to_string()),
            timeout_secs: std::env::var("GRAPHDB_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(30),
            username: std::env::var("GRAPHDB_USERNAME").ok(),
            password: std::env::var("GRAPHDB_PASSWORD").ok(),
            graph_iri: std::env::var("GRAPHDB_GRAPH_IRI")
                .unwrap_or_else(|_| DEFAULT_GRAPH_IRI.to_string()),
        }
    }
}

pub struct GraphDbAdapter {
    config: GraphDbConfig,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct GraphDbTurtleLoadTiming {
    pub total_duration: Duration,
    pub read_duration: Duration,
    pub ingest_duration: Duration,
    pub dataset_bytes: u64,
}

impl GraphDbAdapter {
    pub fn new(config: GraphDbConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("GraphDB reqwest client should be constructible");

        Self { config, client }
    }

    pub fn config(&self) -> &GraphDbConfig {
        &self.config
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .apply_auth(self.client.get(self.url("/rest/repositories")))
            .send()
            .await
            .context("failed to call GraphDB repositories endpoint")?;
        Ok(response.status().is_success())
    }

    pub async fn reset_repository(&self) -> Result<()> {
        validate_repository_id(&self.config.repository)?;
        self.delete_repository().await?;
        self.create_repository().await
    }

    pub async fn create_repository(&self) -> Result<()> {
        validate_repository_id(&self.config.repository)?;
        let config_ttl = repository_config_ttl(&self.config.repository);
        let body = multipart_config_body(&config_ttl);
        let response = self
            .apply_auth(
                self.client
                    .post(self.url("/rest/repositories"))
                    .header(
                        "Content-Type",
                        format!("multipart/form-data; boundary={MULTIPART_BOUNDARY}"),
                    )
                    .body(body),
            )
            .send()
            .await
            .context("failed to create GraphDB repository")?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if status.is_success() {
            return Ok(());
        }
        anyhow::bail!("GraphDB repository create failed: {status} body={body}");
    }

    pub async fn delete_repository(&self) -> Result<()> {
        validate_repository_id(&self.config.repository)?;
        let response = self
            .apply_auth(
                self.client
                    .delete(self.url(&format!("/rest/repositories/{}", self.config.repository))),
            )
            .send()
            .await
            .context("failed to delete GraphDB repository")?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 404 {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GraphDB repository delete failed: {status} body={body}");
    }

    pub async fn load_turtle(&self, dataset_path: &Path) -> Result<Duration> {
        Ok(self.load_turtle_timed(dataset_path).await?.total_duration)
    }

    pub async fn load_turtle_timed(&self, dataset_path: &Path) -> Result<GraphDbTurtleLoadTiming> {
        let total_start = Instant::now();

        let read_start = Instant::now();
        let raw_content = fs::read_to_string(dataset_path)
            .with_context(|| format!("failed to read GraphDB Turtle dataset {:?}", dataset_path))?;
        let content = normalize_turtle(&raw_content);
        let read_duration = read_start.elapsed();
        let dataset_bytes = raw_content.len() as u64;

        let ingest_start = Instant::now();
        let context = graphdb_context_param(&self.config.graph_iri);
        let response = self
            .apply_auth(
                self.client
                    .post(self.repository_url("/statements"))
                    .query(&[("context", context.as_str())])
                    .header("Content-Type", "text/turtle")
                    .body(content),
            )
            .send()
            .await
            .context("failed to load Turtle into GraphDB")?;
        let ingest_duration = ingest_start.elapsed();

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("GraphDB Turtle load failed: {status} body={body}");
        }

        Ok(GraphDbTurtleLoadTiming {
            total_duration: total_start.elapsed(),
            read_duration,
            ingest_duration,
            dataset_bytes,
        })
    }

    async fn execute_sparql(
        &self,
        scenario: impl Into<String>,
        query: String,
    ) -> Result<TraceQueryResult> {
        let scenario = scenario.into();
        let start = Instant::now();
        let response = self
            .apply_auth(
                self.client
                    .post(self.repository_url(""))
                    .header("Accept", "application/sparql-results+json")
                    .form(&[("query", query.as_str())]),
            )
            .send()
            .await
            .with_context(|| format!("failed to send GraphDB query for {scenario}"))?;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let status = response.status();
        let payload: Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));

        if !status.is_success() {
            return Ok(TraceQueryResult::new("GraphDB", scenario, duration_ms)
                .with_error(format!("http status {status}: {payload}")));
        }

        let record_count = payload
            .get("results")
            .and_then(|results| results.get("bindings"))
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_else(|| {
                payload
                    .get("boolean")
                    .and_then(Value::as_bool)
                    .map(|value| usize::from(value))
                    .unwrap_or(0)
            });

        Ok(TraceQueryResult::new("GraphDB", scenario, duration_ms)
            .with_record_count(record_count)
            .with_metadata("graph_iri", self.config.graph_iri.clone())
            .with_metadata("repository", self.config.repository.clone()))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    fn repository_url(&self, suffix: &str) -> String {
        self.url(&format!(
            "/repositories/{}{}",
            self.config.repository, suffix
        ))
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match (&self.config.username, &self.config.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            (Some(username), None) => request.basic_auth(username, Option::<String>::None),
            _ => request,
        }
    }
}

#[async_trait]
impl BenchmarkAdapter for GraphDbAdapter {
    fn system_name(&self) -> &'static str {
        "GraphDB"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: true,
            supports_ledger_write: true,
            supports_semantic_pipeline: false,
            supports_native_rdf: true,
            supports_native_jsonld: false,
            supports_native_shacl: false,
            supports_finality_measurement: false,
        }
    }
}

#[async_trait]
impl TraceQueryAdapter for GraphDbAdapter {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult> {
        self.execute_sparql("Simple Product Lookup", entity_lookup_query(id))
            .await
    }

    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult> {
        let query = multi_hop_query(id);
        self.execute_sparql(format!("Multi-hop Traceability ({} hops)", hops), query)
            .await
    }

    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult> {
        self.execute_sparql("Aggregation by Producer", aggregation_by_producer_query())
            .await
    }
}

fn validate_repository_id(repository: &str) -> Result<()> {
    if repository.is_empty() {
        anyhow::bail!("GraphDB repository id cannot be empty");
    }
    if repository
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Ok(());
    }
    anyhow::bail!("GraphDB repository id contains unsupported characters: {repository}");
}

fn graphdb_context_param(graph_iri: &str) -> String {
    let trimmed = graph_iri.trim();
    if trimmed.starts_with('<') || trimmed.starts_with("_:") {
        trimmed.to_string()
    } else {
        format!("<{trimmed}>")
    }
}

fn repository_config_ttl(repository: &str) -> String {
    format!(
        r#"@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rep: <http://www.openrdf.org/config/repository#> .
@prefix sr: <http://www.openrdf.org/config/repository/sail#> .
@prefix sail: <http://www.openrdf.org/config/sail#> .
@prefix graphdb: <http://www.ontotext.com/trree/graphdb#> .

[] a rep:Repository ;
    rep:repositoryID "{repository}" ;
    rdfs:label "ProvChain GraphDB benchmark repository" ;
    rep:repositoryImpl [
        rep:repositoryType "graphdb:SailRepository" ;
        sr:sailImpl [
            sail:sailType "graphdb:Sail" ;
            graphdb:ruleset "empty" ;
            graphdb:storage-folder "storage" ;
            graphdb:enable-context-index "true" ;
            graphdb:enablePredicateList "true" ;
            graphdb:enable-literal-index "true" ;
            graphdb:in-memory-literal-properties "true"
        ]
    ] .
"#
    )
}

fn multipart_config_body(config_ttl: &str) -> Vec<u8> {
    format!(
        "--{MULTIPART_BOUNDARY}\r\n\
Content-Disposition: form-data; name=\"config\"; filename=\"repository-config.ttl\"\r\n\
Content-Type: text/turtle\r\n\r\n\
{config_ttl}\r\n\
--{MULTIPART_BOUNDARY}--\r\n"
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    fn test_adapter(server: &Server) -> GraphDbAdapter {
        GraphDbAdapter::new(GraphDbConfig {
            base_url: server.url(),
            repository: "provchain_smoke".to_string(),
            timeout_secs: 5,
            username: None,
            password: None,
            graph_iri: "http://example.org/benchmark/graph".to_string(),
        })
    }

    #[tokio::test]
    async fn graphdb_health_check_uses_repositories_endpoint() {
        let mut server = Server::new_async().await;
        let _repositories = server
            .mock("GET", "/rest/repositories")
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        assert!(adapter
            .health_check()
            .await
            .expect("health check should pass"));
    }

    #[tokio::test]
    async fn graphdb_reset_deletes_and_creates_repository() {
        let mut server = Server::new_async().await;
        let _delete = server
            .mock("DELETE", "/rest/repositories/provchain_smoke")
            .with_status(200)
            .create_async()
            .await;
        let _create = server
            .mock("POST", "/rest/repositories")
            .match_header(
                "content-type",
                Matcher::Regex("multipart/form-data; boundary=.*".to_string()),
            )
            .match_body(Matcher::Regex(
                "repositoryID \"provchain_smoke\"".to_string(),
            ))
            .with_status(201)
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        adapter
            .reset_repository()
            .await
            .expect("repository reset should delete and recreate repository");
    }

    #[test]
    fn graphdb_repository_id_rejects_path_like_values() {
        let error = validate_repository_id("../private")
            .expect_err("path-like repository ids should be rejected");
        assert!(error.to_string().contains("unsupported characters"));
    }

    #[test]
    fn graphdb_context_param_wraps_plain_iri_as_ntriples_resource() {
        assert_eq!(
            graphdb_context_param("http://example.org/benchmark/graph"),
            "<http://example.org/benchmark/graph>"
        );
        assert_eq!(
            graphdb_context_param("<http://example.org/benchmark/graph>"),
            "<http://example.org/benchmark/graph>"
        );
        assert_eq!(
            graphdb_context_param("_:benchmarkGraph"),
            "_:benchmarkGraph"
        );
    }

    #[tokio::test]
    async fn graphdb_load_turtle_posts_context_and_turtle_body() {
        let mut server = Server::new_async().await;
        let _load = server
            .mock("POST", "/repositories/provchain_smoke/statements")
            .match_query(Matcher::UrlEncoded(
                "context".to_string(),
                "<http://example.org/benchmark/graph>".to_string(),
            ))
            .match_header("content-type", Matcher::Regex("text/turtle.*".to_string()))
            .match_body(Matcher::Regex(
                "<http://example.org/supplychain/Producer/Farm001>".to_string(),
            ))
            .match_body(Matcher::Regex("ex:batchId \"BATCH001\"".to_string()))
            .with_status(204)
            .create_async()
            .await;

        let dataset_path =
            std::env::temp_dir().join(format!("graphdb-turtle-test-{}.ttl", std::process::id()));
        fs::write(
            &dataset_path,
            r#"@prefix ex: <http://example.org/supplychain/> .
ex:Producer/Farm001 a ex:Producer .
ex:Product/BATCH001 ex:batchId "BATCH001" ;
    ex:hasProducer ex:Producer/Farm001 .
"#,
        )
        .expect("test dataset should be writable");

        let adapter = test_adapter(&server);
        let timing = adapter
            .load_turtle_timed(&dataset_path)
            .await
            .expect("Turtle load should succeed");
        assert!(timing.dataset_bytes > 0);
        let _ = fs::remove_file(dataset_path);
    }

    #[tokio::test]
    async fn graphdb_entity_lookup_sends_sparql_and_counts_bindings() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("POST", "/repositories/provchain_smoke")
            .match_header(
                "content-type",
                Matcher::Regex("application/x-www-form-urlencoded.*".to_string()),
            )
            .match_body(Matcher::Regex("BATCH001".to_string()))
            .with_status(200)
            .with_header("content-type", "application/sparql-results+json")
            .with_body(
                r#"{
                  "head": {"vars": ["product"]},
                  "results": {"bindings": [
                    {"product": {"type": "uri", "value": "http://example.org/Product/BATCH001"}},
                    {"product": {"type": "uri", "value": "http://example.org/Product/BATCH001-copy"}}
                  ]}
                }"#,
            )
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        let result = adapter
            .entity_lookup("BATCH001")
            .await
            .expect("GraphDB query should return a TraceQueryResult");

        assert_eq!(result.system, "GraphDB");
        assert_eq!(result.scenario, "Simple Product Lookup");
        assert_eq!(result.record_count, 2);
        assert!(result.success);
        assert_eq!(
            result.metadata.get("repository").and_then(Value::as_str),
            Some("provchain_smoke")
        );
    }

    #[tokio::test]
    async fn graphdb_multi_hop_uses_trace_query_sparql_shape() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("POST", "/repositories/provchain_smoke")
            .match_body(Matcher::Regex("BATCH017".to_string()))
            .match_body(Matcher::Regex("GRAPH".to_string()))
            .match_body(Matcher::Regex("nextTransaction".to_string()))
            .with_status(200)
            .with_header("content-type", "application/sparql-results+json")
            .with_body(
                r#"{
                  "head": {"vars": ["product", "tx1"]},
                  "results": {"bindings": [
                    {"product": {"type": "uri", "value": "http://example.org/Product/BATCH017"}}
                  ]}
                }"#,
            )
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        let result = adapter
            .trace_multi_hop("BATCH017", 10)
            .await
            .expect("GraphDB multi-hop query should complete");

        assert_eq!(result.system, "GraphDB");
        assert_eq!(result.scenario, "Multi-hop Traceability (10 hops)");
        assert_eq!(result.record_count, 1);
    }

    #[tokio::test]
    async fn graphdb_aggregation_uses_grouped_sparql_shape() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("POST", "/repositories/provchain_smoke")
            .match_body(Matcher::Regex("GROUP".to_string()))
            .match_body(Matcher::Regex("producer".to_string()))
            .match_body(Matcher::Regex("quantity".to_string()))
            .with_status(200)
            .with_header("content-type", "application/sparql-results+json")
            .with_body(
                r#"{
                  "head": {"vars": ["producer", "total"]},
                  "results": {"bindings": [
                    {"producer": {"type": "uri", "value": "http://example.org/Producer/Farm001"}}
                  ]}
                }"#,
            )
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        let result = adapter
            .aggregation_by_producer()
            .await
            .expect("GraphDB aggregation query should complete");

        assert_eq!(result.system, "GraphDB");
        assert_eq!(result.scenario, "Aggregation by Producer");
        assert_eq!(result.record_count, 1);
    }

    #[tokio::test]
    async fn graphdb_query_error_includes_response_body() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("POST", "/repositories/provchain_smoke")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message":"bad sparql"}"#)
            .create_async()
            .await;

        let adapter = test_adapter(&server);
        let result = adapter
            .aggregation_by_producer()
            .await
            .expect("query transport should complete");

        assert!(!result.success);
        assert!(result
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("bad sparql"));
    }
}
