use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
use crate::core::result::TraceQueryResult;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const EX_BATCH_ID: &str = "http://example.org/supplychain/batchId";
const EX_PRODUCT: &str = "http://example.org/supplychain/product";
const TRACE_DISTRIBUTED_BY: &str = "http://example.org/traceability#distributedBy";
const TRACE_HAS_PRODUCER: &str = "http://example.org/traceability#hasProducer";
const TRACE_PROCESSED_BY: &str = "http://example.org/traceability#processedBy";
const TRACE_QUANTITY: &str = "http://example.org/traceability#quantity";
const TRACE_SOLD_BY: &str = "http://example.org/traceability#soldBy";

#[derive(Debug, Clone)]
pub struct FlureeConfig {
    pub base_url: String,
    pub ledger: String,
}

impl Default for FlureeConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8090".to_string(),
            ledger: "provchain/benchmark".to_string(),
        }
    }
}

pub struct FlureeAdapter {
    config: FlureeConfig,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct FlureeJsonLdLoadTiming {
    pub total_duration: Duration,
    pub ledger_prepare_duration: Duration,
    pub read_duration: Duration,
    pub parse_duration: Duration,
    pub transact_duration: Duration,
}

impl FlureeAdapter {
    pub fn new(config: FlureeConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let fdb_health = format!("{}/fdb/health", self.config.base_url.trim_end_matches('/'));
        let response = self.client.get(&fdb_health).send().await;
        let response = match response {
            Ok(response)
                if response.status().is_success() || response.status().is_redirection() =>
            {
                response
            }
            _ => self
                .client
                .get(format!(
                    "{}/index.html",
                    self.config.base_url.trim_end_matches('/')
                ))
                .send()
                .await
                .context("failed to call Fluree health endpoint")?,
        };
        Ok(response.status().is_success() || response.status().is_redirection())
    }

    fn api_paths(&self, endpoint: &str) -> Vec<String> {
        let base = self.config.base_url.trim_end_matches('/');
        let (network, db) = self.network_and_db();

        if endpoint == "create" {
            return vec![
                format!("{base}/fluree/create"),
                format!("{base}/v1/fluree/create"),
                format!("{base}/fdb/{network}/create"),
                format!("{base}/fdb/{network}/{db}/create"),
                format!("{base}/create"),
            ];
        }

        vec![
            format!("{base}/fluree/{endpoint}"),
            format!("{base}/v1/fluree/{endpoint}"),
            format!("{base}/fdb/{network}/{db}/{endpoint}"),
            format!("{base}/{endpoint}"),
        ]
    }

    fn network_and_db(&self) -> (&str, &str) {
        self.config
            .ledger
            .split_once('/')
            .unwrap_or(("provchain", self.config.ledger.as_str()))
    }

    async fn post_json(
        &self,
        endpoint: &str,
        body: &Value,
        extra_headers: &[(&str, &str)],
    ) -> Result<reqwest::Response> {
        let mut attempted = Vec::new();

        for url in self.api_paths(endpoint) {
            let mut request = self.client.post(&url).json(body);
            for (name, value) in extra_headers {
                request = request.header(*name, *value);
            }

            let response = request
                .send()
                .await
                .with_context(|| format!("failed to call Fluree endpoint {url}"))?;

            let status = response.status();
            if status != StatusCode::NOT_FOUND {
                return Ok(response);
            }

            attempted.push(format!("{url} -> {status}"));
        }

        anyhow::bail!(
            "Fluree endpoint not found for {endpoint}; attempted {}",
            attempted.join(", ")
        );
    }

    async fn ensure_ledger_exists(&self) -> Result<()> {
        let response = self
            .post_json(
                "create",
                &serde_json::json!({
                    "ledger": self.config.ledger,
                }),
                &[],
            )
            .await
            .context("failed to ensure Fluree ledger exists")?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() || status == StatusCode::CONFLICT {
            return Ok(());
        }

        if status == StatusCode::BAD_REQUEST
            && (body.contains("already exists") || body.contains("db/already-exists"))
        {
            return Ok(());
        }

        anyhow::bail!("Fluree ledger creation failed: {status} body={body}");
    }

    pub async fn load_jsonld(&self, dataset_path: &Path) -> Result<Duration> {
        Ok(self
            .load_jsonld_timed(dataset_path)
            .await?
            .transact_duration)
    }

    pub async fn load_jsonld_timed(&self, dataset_path: &Path) -> Result<FlureeJsonLdLoadTiming> {
        let total_start = Instant::now();
        let ledger_start = Instant::now();
        self.ensure_ledger_exists().await?;
        let ledger_prepare_duration = ledger_start.elapsed();

        let read_start = Instant::now();
        let content = fs::read_to_string(dataset_path)
            .with_context(|| format!("failed to read dataset {:?}", dataset_path))?;
        let read_duration = read_start.elapsed();

        let parse_start = Instant::now();
        let payload = serde_json::from_str::<Value>(&content)
            .context("failed to parse Fluree JSON-LD dataset")?;
        let parse_duration = parse_start.elapsed();

        let start = Instant::now();
        let response = self
            .post_json(
                "transact",
                &serde_json::json!({
                    "ledger": self.config.ledger,
                    "insert": payload,
                }),
                &[],
            )
            .await
            .context("failed to load dataset into Fluree")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Fluree load failed: {status} body={body}");
        }
        let transact_duration = start.elapsed();

        Ok(FlureeJsonLdLoadTiming {
            total_duration: total_start.elapsed(),
            ledger_prepare_duration,
            read_duration,
            parse_duration,
            transact_duration,
        })
    }

    async fn execute_query(
        &self,
        scenario: impl Into<String>,
        query: Value,
    ) -> Result<TraceQueryResult> {
        let scenario = scenario.into();
        if let Err(e) = self.ensure_ledger_exists().await {
            return Ok(TraceQueryResult::new("Fluree", scenario, 0.0).with_error(format!("{e:#}")));
        }
        let mut query_body = serde_json::Map::new();
        query_body.insert(
            "from".to_string(),
            Value::String(self.config.ledger.clone()),
        );
        query_body.insert(
            "select".to_string(),
            query.get("select").cloned().unwrap_or(Value::Null),
        );
        query_body.insert(
            "where".to_string(),
            query.get("where").cloned().unwrap_or(Value::Null),
        );
        if let Some(group_by) = query.get("group-by").or_else(|| query.get("groupBy")) {
            query_body.insert("group-by".to_string(), group_by.clone());
        }
        if let Some(opts) = query.get("opts") {
            query_body.insert("opts".to_string(), opts.clone());
        }

        let start = Instant::now();
        let response = self
            .post_json("query", &Value::Object(query_body), &[])
            .await
            .with_context(|| format!("failed to send Fluree query for {scenario}"))?;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let status = response.status();
        let payload: Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!([]));

        if !status.is_success() {
            return Ok(TraceQueryResult::new("Fluree", scenario, duration_ms)
                .with_error(format!("http status {status}: {payload}")));
        }

        let record_count = payload.as_array().map(|rows| rows.len()).unwrap_or(0);

        Ok(TraceQueryResult::new("Fluree", scenario, duration_ms).with_record_count(record_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    #[tokio::test]
    async fn health_check_accepts_root_redirect() {
        let mut server = Server::new_async().await;
        let _health = server
            .mock("GET", "/fdb/health")
            .with_status(302)
            .with_header("location", "/index.html")
            .create_async()
            .await;
        let _root = server
            .mock("GET", "/")
            .with_status(302)
            .with_header("location", "/index.html")
            .create_async()
            .await;
        let _index = server
            .mock("GET", "/index.html")
            .with_status(200)
            .create_async()
            .await;

        let adapter = FlureeAdapter::new(FlureeConfig {
            base_url: server.url(),
            ledger: "provchain/benchmark".to_string(),
        });

        let healthy = adapter
            .health_check()
            .await
            .expect("health check should succeed");
        assert!(healthy);
    }

    #[tokio::test]
    async fn ensure_ledger_exists_falls_back_to_second_api_family() {
        let mut server = Server::new_async().await;

        let _create_fdb_network = server
            .mock("POST", "/fluree/create")
            .match_header(
                "content-type",
                Matcher::Regex("application/json.*".to_string()),
            )
            .match_body(Matcher::PartialJson(serde_json::json!({
                "ledger": "provchain/benchmark"
            })))
            .with_status(200)
            .with_body(r#"{"status":"ok"}"#)
            .create_async()
            .await;

        let adapter = FlureeAdapter::new(FlureeConfig {
            base_url: server.url(),
            ledger: "provchain/benchmark".to_string(),
        });

        adapter
            .ensure_ledger_exists()
            .await
            .expect("ledger creation should use Fluree API family and succeed");
    }

    #[tokio::test]
    async fn ensure_ledger_exists_falls_back_to_legacy_api_family() {
        let mut server = Server::new_async().await;

        let _create_fluree = server
            .mock("POST", "/fluree/create")
            .with_status(404)
            .create_async()
            .await;
        let _create_v1 = server
            .mock("POST", "/v1/fluree/create")
            .with_status(404)
            .create_async()
            .await;
        let _create_fdb = server
            .mock("POST", "/fdb/provchain/create")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "ledger": "provchain/benchmark"
            })))
            .with_status(201)
            .with_body(r#"{"status":"ok"}"#)
            .create_async()
            .await;

        let adapter = FlureeAdapter::new(FlureeConfig {
            base_url: server.url(),
            ledger: "provchain/benchmark".to_string(),
        });

        adapter
            .ensure_ledger_exists()
            .await
            .expect("legacy create fallback should succeed");
    }

    #[tokio::test]
    async fn load_jsonld_wraps_dataset_in_ledger_transaction() {
        let mut server = Server::new_async().await;

        let _create = server
            .mock("POST", "/fluree/create")
            .with_status(201)
            .with_body(r#"{"status":"ok"}"#)
            .create_async()
            .await;
        let _transact = server
            .mock("POST", "/fluree/transact")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "ledger": "provchain/benchmark",
                "insert": [{
                    "@id": "http://example.org/supplychain/Product/BATCH001",
                    "http://example.org/supplychain/batchId": [{"@value": "BATCH001"}]
                }]
            })))
            .with_status(200)
            .with_body(r#"{"status":"ok"}"#)
            .create_async()
            .await;

        let dataset_path =
            std::env::temp_dir().join(format!("fluree-jsonld-test-{}.json", std::process::id()));
        fs::write(
            &dataset_path,
            r#"[{"@id":"http://example.org/supplychain/Product/BATCH001","http://example.org/supplychain/batchId":[{"@value":"BATCH001"}]}]"#,
        )
        .expect("test dataset should be writable");

        let adapter = FlureeAdapter::new(FlureeConfig {
            base_url: server.url(),
            ledger: "provchain/benchmark".to_string(),
        });

        adapter
            .load_jsonld(&dataset_path)
            .await
            .expect("JSON-LD load should be wrapped as a Fluree transaction");
        let _ = fs::remove_file(dataset_path);
    }

    #[tokio::test]
    async fn execute_query_includes_response_body_in_error() {
        let mut server = Server::new_async().await;

        let _create_fdb_network = server
            .mock("POST", "/fluree/create")
            .with_status(200)
            .with_body(r#"{"status":"ok"}"#)
            .create_async()
            .await;
        let _query_fluree = server
            .mock("POST", "/fluree/query")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "from": "provchain/benchmark",
                "select": ["?product"],
                "where": {
                    "@id": "?product",
                    "http://example.org/supplychain/batchId": "BATCH001"
                }
            })))
            .with_status(400)
            .with_body(r#"{"error":"bad query"}"#)
            .create_async()
            .await;

        let adapter = FlureeAdapter::new(FlureeConfig {
            base_url: server.url(),
            ledger: "provchain/benchmark".to_string(),
        });

        let result = adapter
            .entity_lookup("BATCH001")
            .await
            .expect("query call should return TraceQueryResult");

        assert!(!result.success);
        assert!(result
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("bad query"));
    }
}

#[async_trait]
impl BenchmarkAdapter for FlureeAdapter {
    fn system_name(&self) -> &'static str {
        "Fluree"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: true,
            supports_ledger_write: true,
            supports_semantic_pipeline: false,
            supports_native_rdf: false,
            supports_native_jsonld: true,
            supports_native_shacl: false,
            supports_finality_measurement: false,
        }
    }
}

#[async_trait]
impl TraceQueryAdapter for FlureeAdapter {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult> {
        let mut where_clause = serde_json::Map::new();
        where_clause.insert("@id".to_string(), Value::String("?product".to_string()));
        where_clause.insert(EX_BATCH_ID.to_string(), Value::String(id.to_string()));

        let query = serde_json::json!({
            "select": ["?product"],
            "where": Value::Object(where_clause)
        });
        self.execute_query("Simple Product Lookup", query).await
    }

    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult> {
        let mut where_clause = serde_json::Map::new();
        where_clause.insert("@id".to_string(), Value::String("?product".to_string()));
        where_clause.insert(EX_BATCH_ID.to_string(), Value::String(id.to_string()));
        where_clause.insert(
            TRACE_HAS_PRODUCER.to_string(),
            Value::String("?producer".to_string()),
        );
        where_clause.insert(
            TRACE_PROCESSED_BY.to_string(),
            Value::String("?processor".to_string()),
        );
        where_clause.insert(
            TRACE_DISTRIBUTED_BY.to_string(),
            Value::String("?distributor".to_string()),
        );
        where_clause.insert(
            TRACE_SOLD_BY.to_string(),
            Value::String("?retailer".to_string()),
        );

        let query = serde_json::json!({
            "select": ["?product", "?producer", "?processor", "?distributor", "?retailer"],
            "where": Value::Object(where_clause),
            "opts": {
                "maxHops": hops
            }
        });
        self.execute_query(format!("Multi-hop Traceability ({} hops)", hops), query)
            .await
    }

    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult> {
        let mut product_clause = serde_json::Map::new();
        product_clause.insert("@id".to_string(), Value::String("?product".to_string()));
        product_clause.insert(
            TRACE_HAS_PRODUCER.to_string(),
            Value::String("?producer".to_string()),
        );

        let mut transaction_clause = serde_json::Map::new();
        transaction_clause.insert("@id".to_string(), Value::String("?tx".to_string()));
        transaction_clause.insert(
            EX_PRODUCT.to_string(),
            Value::String("?product".to_string()),
        );
        transaction_clause.insert(
            TRACE_QUANTITY.to_string(),
            Value::String("?quantity".to_string()),
        );

        let query = serde_json::json!({
            "select": ["?producer", "(sum ?quantity)"],
            "where": [
                Value::Object(product_clause),
                Value::Object(transaction_clause)
            ],
            "group-by": "?producer"
        });
        self.execute_query("Aggregation by Producer", query).await
    }
}
