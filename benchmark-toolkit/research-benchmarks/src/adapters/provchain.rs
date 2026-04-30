use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
use crate::core::result::TraceQueryResult;
use crate::dataset::normalize_turtle;
use crate::workloads::provchain_queries::{
    aggregation_by_producer_query, entity_lookup_query, multi_hop_query,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rio_api::parser::TriplesParser;
use rio_turtle::TurtleParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct ProvChainAdapter {
    base_url: String,
    client: Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvChainPolicyCheckRequest {
    pub record_id: String,
    pub actor_org: String,
    pub action: String,
    pub owner_org: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProvChainPolicyCheckResponse {
    pub authorized: bool,
    #[serde(default)]
    pub policy_latency_ms: f64,
    #[serde(default)]
    pub policy_engine: String,
    #[serde(default)]
    pub evaluated_by: String,
    #[serde(default)]
    pub user_role: String,
}

#[derive(Debug, Clone)]
pub struct ProvChainBatchWriteTiming {
    pub total_duration: Duration,
    pub auth_duration: Duration,
    pub submit_loop_duration: Duration,
    pub transaction_count: usize,
    pub block_count: usize,
    pub server_timing_totals_ms: HashMap<String, f64>,
    pub server_timing_samples: usize,
}

#[derive(Debug, Clone)]
pub struct ProvChainDatasetLoadTiming {
    pub total_duration: Duration,
    pub read_duration: Duration,
    pub normalize_duration: Duration,
    pub parse_duration: Duration,
    pub auth_duration: Duration,
    pub submit_loop_duration: Duration,
    pub triple_count: usize,
    pub dataset_bytes: usize,
    pub block_count: usize,
    pub import_mode: String,
    pub server_timing_totals_ms: HashMap<String, f64>,
    pub server_timing_samples: usize,
}

impl ProvChainAdapter {
    const BENCHMARK_ADMIN_USERNAME: &'static str = "adminroot";
    const BENCHMARK_ADMIN_PASSWORD: &'static str = "AdminRootPassword123!";

    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .context("failed to call ProvChain health endpoint")?;
        Ok(response.status().is_success())
    }

    pub async fn load_dataset_turtle(&self, dataset_path: &Path) -> Result<Duration> {
        Ok(self
            .load_dataset_turtle_timed(dataset_path)
            .await?
            .total_duration)
    }

    pub async fn load_dataset_turtle_timed(
        &self,
        dataset_path: &Path,
    ) -> Result<ProvChainDatasetLoadTiming> {
        match std::env::var("PROVCHAIN_IMPORT_MODE")
            .unwrap_or_else(|_| "bulk-turtle-single-block".to_string())
            .as_str()
        {
            "legacy-per-triple" => self.load_dataset_turtle_legacy_timed(dataset_path).await,
            "bulk-turtle-single-block" | "bulk" | "bulk-turtle" => {
                self.load_dataset_turtle_bulk_timed(dataset_path).await
            }
            mode => anyhow::bail!(
                "Unsupported PROVCHAIN_IMPORT_MODE='{}'. Use bulk-turtle-single-block or legacy-per-triple.",
                mode
            ),
        }
    }

    pub async fn load_dataset_turtle_bulk_timed(
        &self,
        dataset_path: &Path,
    ) -> Result<ProvChainDatasetLoadTiming> {
        let auth_start = Instant::now();
        let auth_token = self.authenticate_demo_user().await?;
        let auth_duration = auth_start.elapsed();

        let total_start = Instant::now();
        let read_start = Instant::now();
        let raw_content = fs::read_to_string(dataset_path)
            .with_context(|| format!("failed to read dataset {:?}", dataset_path))?;
        let read_duration = read_start.elapsed();

        let normalize_start = Instant::now();
        let content = normalize_turtle(&raw_content);
        let normalize_duration = normalize_start.elapsed();

        let parse_start = Instant::now();
        let triples = Self::parse_turtle_triples(&content)?;
        let parse_duration = parse_start.elapsed();

        let submit_start = Instant::now();
        let triple_count = triples.len();
        let mut server_timing_totals_ms = HashMap::new();
        let mut server_timing_samples = 0;

        let response = self
            .client
            .post(format!("{}/api/datasets/import-turtle", self.base_url))
            .bearer_auth(&auth_token)
            .json(&serde_json::json!({
                "turtle_data": content
            }))
            .send()
            .await
            .context("failed to bulk import Turtle dataset into ProvChain")?;

        if !response.status().is_success() {
            anyhow::bail!("ProvChain bulk import failed: {}", response.status());
        }
        let response_json = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        if collect_server_timings(&response_json, &mut server_timing_totals_ms) {
            server_timing_samples += 1;
        }
        let block_count = response_json
            .get("block_count")
            .and_then(|value| value.as_u64())
            .unwrap_or(1) as usize;
        let import_mode = response_json
            .get("import_mode")
            .and_then(|value| value.as_str())
            .unwrap_or("bulk_turtle_single_block")
            .to_string();

        Ok(ProvChainDatasetLoadTiming {
            total_duration: total_start.elapsed(),
            read_duration,
            normalize_duration,
            parse_duration,
            auth_duration,
            submit_loop_duration: submit_start.elapsed(),
            triple_count,
            dataset_bytes: raw_content.len(),
            block_count,
            import_mode,
            server_timing_totals_ms,
            server_timing_samples,
        })
    }

    pub async fn load_dataset_turtle_legacy_timed(
        &self,
        dataset_path: &Path,
    ) -> Result<ProvChainDatasetLoadTiming> {
        let auth_start = Instant::now();
        let auth_token = self.authenticate_demo_user().await?;
        let auth_duration = auth_start.elapsed();

        let total_start = Instant::now();
        let read_start = Instant::now();
        let raw_content = fs::read_to_string(dataset_path)
            .with_context(|| format!("failed to read dataset {:?}", dataset_path))?;
        let read_duration = read_start.elapsed();

        let normalize_start = Instant::now();
        let content = normalize_turtle(&raw_content);
        let normalize_duration = normalize_start.elapsed();

        let parse_start = Instant::now();
        let triples = Self::parse_turtle_triples(&content)?;
        let parse_duration = parse_start.elapsed();

        let submit_start = Instant::now();
        let triple_count = triples.len();
        let mut server_timing_totals_ms = HashMap::new();
        let mut server_timing_samples = 0;

        for triple in triples {
            let response = self
                .client
                .post(format!("{}/api/blockchain/add-triple", self.base_url))
                .bearer_auth(&auth_token)
                .json(&triple)
                .send()
                .await
                .context("failed to load dataset triple into ProvChain")?;

            if !response.status().is_success() {
                anyhow::bail!("ProvChain import failed: {}", response.status());
            }
            let response_json = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| serde_json::json!({}));
            if collect_server_timings(&response_json, &mut server_timing_totals_ms) {
                server_timing_samples += 1;
            }
        }

        Ok(ProvChainDatasetLoadTiming {
            total_duration: total_start.elapsed(),
            read_duration,
            normalize_duration,
            parse_duration,
            auth_duration,
            submit_loop_duration: submit_start.elapsed(),
            triple_count,
            dataset_bytes: raw_content.len(),
            block_count: triple_count,
            import_mode: "legacy_per_triple".to_string(),
            server_timing_totals_ms,
            server_timing_samples,
        })
    }

    pub async fn write_transaction_batch(&self, count: usize) -> Result<Duration> {
        Ok(self
            .write_transaction_batch_timed(count)
            .await?
            .total_duration)
    }

    pub async fn write_transaction_batch_timed(
        &self,
        count: usize,
    ) -> Result<ProvChainBatchWriteTiming> {
        let total_start = Instant::now();
        let auth_start = Instant::now();
        let auth_token = self.authenticate_demo_user().await?;
        let auth_duration = auth_start.elapsed();
        let submit_start = Instant::now();
        let mut server_timing_totals_ms = HashMap::new();
        let mut server_timing_samples = 0;

        for batch_id in 1..=count {
            let response = self
                .client
                .post(format!("{}/api/blockchain/add-triple", self.base_url))
                .bearer_auth(&auth_token)
                .json(&serde_json::json!({
                    "subject": format!("http://example.org/benchmark/write/BATCH{:06}", batch_id),
                    "predicate": "http://provchain.org/trace#timestamp",
                    "object": Utc::now().to_rfc3339()
                }))
                .send()
                .await
                .with_context(|| format!("failed to submit ProvChain write {}", batch_id))?;

            if !response.status().is_success() {
                anyhow::bail!(
                    "ProvChain write submission failed at {} with status {}",
                    batch_id,
                    response.status()
                );
            }
            let response_json = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| serde_json::json!({}));
            if collect_server_timings(&response_json, &mut server_timing_totals_ms) {
                server_timing_samples += 1;
            }
        }

        Ok(ProvChainBatchWriteTiming {
            total_duration: total_start.elapsed(),
            auth_duration,
            submit_loop_duration: submit_start.elapsed(),
            transaction_count: count,
            block_count: count,
            server_timing_totals_ms,
            server_timing_samples,
        })
    }

    pub async fn write_transaction_batch_as_single_block_timed(
        &self,
        count: usize,
    ) -> Result<ProvChainBatchWriteTiming> {
        let total_start = Instant::now();
        let auth_start = Instant::now();
        let auth_token = self.authenticate_demo_user().await?;
        let auth_duration = auth_start.elapsed();

        let nonce = Utc::now().timestamp_micros();
        let triples = (1..=count)
            .map(|batch_id| {
                serde_json::json!({
                    "subject": format!(
                        "http://example.org/benchmark/write/BLOCKBATCH{:06}/{}",
                        batch_id, nonce
                    ),
                    "predicate": "http://provchain.org/trace#timestamp",
                    "object": Utc::now().to_rfc3339()
                })
            })
            .collect::<Vec<_>>();

        let submit_start = Instant::now();
        let response = self
            .client
            .post(format!("{}/api/blockchain/add-triples", self.base_url))
            .bearer_auth(&auth_token)
            .json(&serde_json::json!({ "triples": triples }))
            .send()
            .await
            .context("failed to submit ProvChain single-block batch write")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "ProvChain single-block batch write failed with status {}",
                response.status()
            );
        }

        let mut server_timing_totals_ms = HashMap::new();
        let response_json = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        let server_timing_samples = usize::from(collect_server_timings(
            &response_json,
            &mut server_timing_totals_ms,
        ));

        Ok(ProvChainBatchWriteTiming {
            total_duration: total_start.elapsed(),
            auth_duration,
            submit_loop_duration: submit_start.elapsed(),
            transaction_count: count,
            block_count: 1,
            server_timing_totals_ms,
            server_timing_samples,
        })
    }

    pub async fn check_policy(
        &self,
        request: &ProvChainPolicyCheckRequest,
    ) -> Result<ProvChainPolicyCheckResponse> {
        let auth_token = self.authenticate_demo_user().await?;
        self.check_policy_with_token(&auth_token, request).await
    }

    pub async fn authenticate_benchmark_user(&self) -> Result<String> {
        self.authenticate_demo_user().await
    }

    pub async fn check_policy_with_token(
        &self,
        auth_token: &str,
        request: &ProvChainPolicyCheckRequest,
    ) -> Result<ProvChainPolicyCheckResponse> {
        let response = self
            .client
            .post(format!("{}/api/policy/check", self.base_url))
            .bearer_auth(auth_token)
            .json(request)
            .send()
            .await
            .context("failed to call ProvChain policy endpoint")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("ProvChain policy check failed: {status} body={text}");
        }

        serde_json::from_str(&text)
            .with_context(|| format!("failed to decode ProvChain policy response: {text}"))
    }

    async fn execute_sparql(
        &self,
        scenario: impl Into<String>,
        query: String,
    ) -> Result<TraceQueryResult> {
        let scenario = scenario.into();
        let auth_token = self.authenticate_demo_user().await?;
        let start = Instant::now();
        let response = self
            .client
            .post(format!("{}/api/sparql/query", self.base_url))
            .bearer_auth(&auth_token)
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .with_context(|| format!("failed to send ProvChain SPARQL request for {scenario}"))?;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let status = response.status();

        if !status.is_success() {
            let response_body = response.text().await.unwrap_or_default();
            return Ok(
                TraceQueryResult::new("ProvChain-Org", scenario, duration_ms)
                    .with_error(format!("http status {status}: {response_body}")),
            );
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));

        let record_count = response_json
            .get("results")
            .and_then(|results| results.get("bindings"))
            .and_then(|bindings| bindings.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(
            TraceQueryResult::new("ProvChain-Org", scenario, duration_ms)
                .with_record_count(record_count),
        )
    }

    async fn authenticate_demo_user(&self) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": Self::BENCHMARK_ADMIN_USERNAME,
                "password": Self::BENCHMARK_ADMIN_PASSWORD
            }))
            .send()
            .await
            .context("failed to authenticate against ProvChain")?;

        if response.status().is_success() {
            let payload: serde_json::Value = response
                .json()
                .await
                .context("failed to decode ProvChain auth response")?;

            return payload
                .get("token")
                .and_then(|token| token.as_str())
                .map(ToOwned::to_owned)
                .context("ProvChain auth response missing token");
        }

        self.bootstrap_demo_admin().await?;

        let retry = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": Self::BENCHMARK_ADMIN_USERNAME,
                "password": Self::BENCHMARK_ADMIN_PASSWORD
            }))
            .send()
            .await
            .context("failed to authenticate against ProvChain after bootstrap")?;

        if !retry.status().is_success() {
            anyhow::bail!("ProvChain authentication failed: {}", retry.status());
        }

        let payload: serde_json::Value = retry
            .json()
            .await
            .context("failed to decode ProvChain auth response after bootstrap")?;

        payload
            .get("token")
            .and_then(|token| token.as_str())
            .map(ToOwned::to_owned)
            .context("ProvChain auth response missing token")
    }

    async fn bootstrap_demo_admin(&self) -> Result<()> {
        let bootstrap_token = std::env::var("PROVCHAIN_BOOTSTRAP_TOKEN")
            .context("PROVCHAIN_BOOTSTRAP_TOKEN not set for benchmark bootstrap")?;

        let response = self
            .client
            .post(format!("{}/auth/bootstrap", self.base_url))
            .json(&serde_json::json!({
                "username": Self::BENCHMARK_ADMIN_USERNAME,
                "password": Self::BENCHMARK_ADMIN_PASSWORD,
                "bootstrap_token": bootstrap_token
            }))
            .send()
            .await
            .context("failed to bootstrap ProvChain admin")?;

        if !(response.status().is_success() || response.status() == reqwest::StatusCode::CONFLICT) {
            anyhow::bail!("ProvChain bootstrap failed: {}", response.status());
        }

        Ok(())
    }

    fn parse_turtle_triples(content: &str) -> Result<Vec<serde_json::Value>> {
        let mut parser = TurtleParser::new(Cursor::new(content.as_bytes()), None);
        let mut triples = Vec::new();

        parser
            .parse_all(&mut |triple| {
                let subject = match &triple.subject {
                    rio_api::model::Subject::NamedNode(node) => node.iri.to_string(),
                    rio_api::model::Subject::BlankNode(node) => format!("_:{}", node.id),
                    rio_api::model::Subject::Triple(_) => return Ok(()),
                };

                let object = match &triple.object {
                    rio_api::model::Term::NamedNode(node) => node.iri.to_string(),
                    rio_api::model::Term::BlankNode(node) => format!("_:{}", node.id),
                    rio_api::model::Term::Literal(literal) => match literal {
                        rio_api::model::Literal::Simple { value } => value.to_string(),
                        rio_api::model::Literal::LanguageTaggedString { value, language: _ } => {
                            value.to_string()
                        }
                        rio_api::model::Literal::Typed { value, datatype: _ } => value.to_string(),
                    },
                    rio_api::model::Term::Triple(_) => return Ok(()),
                };

                triples.push(serde_json::json!({
                    "subject": subject,
                    "predicate": triple.predicate.iri,
                    "object": object
                }));

                Ok(()) as Result<(), rio_turtle::TurtleError>
            })
            .map_err(|e| anyhow::anyhow!("ProvChain Turtle parsing error: {:?}", e))?;

        Ok(triples)
    }
}

fn collect_server_timings(
    response_json: &serde_json::Value,
    totals_ms: &mut HashMap<String, f64>,
) -> bool {
    let Some(timings) = response_json
        .get("timings_ms")
        .and_then(|value| value.as_object())
    else {
        return false;
    };

    for (key, value) in timings {
        if let Some(ms) = value.as_f64() {
            *totals_ms.entry(key.clone()).or_insert(0.0) += ms;
        }
    }
    true
}

#[async_trait]
impl BenchmarkAdapter for ProvChainAdapter {
    fn system_name(&self) -> &'static str {
        "ProvChain-Org"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: true,
            supports_ledger_write: true,
            supports_semantic_pipeline: true,
            supports_native_rdf: true,
            supports_native_jsonld: true,
            supports_native_shacl: true,
            supports_finality_measurement: true,
        }
    }
}

#[async_trait]
impl TraceQueryAdapter for ProvChainAdapter {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult> {
        let query = entity_lookup_query(id);
        self.execute_sparql("Simple Product Lookup", query).await
    }

    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult> {
        let query = multi_hop_query(id);
        self.execute_sparql(&format!("Multi-hop Traceability ({} hops)", hops), query)
            .await
    }

    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult> {
        let query = aggregation_by_producer_query();
        self.execute_sparql("Aggregation by Producer", query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workloads::provchain_queries::{entity_lookup_query, multi_hop_query};
    use mockito::{Matcher, Server};

    #[test]
    fn entity_lookup_query_has_terminated_triples() {
        let query = entity_lookup_query("BATCH001");
        assert!(query.contains("GRAPH ?g1 { ?product a ex:Product . }"));
        assert!(query.contains("GRAPH ?g2 { ?product ex:batchId \"BATCH001\" . }"));
    }

    #[test]
    fn multi_hop_query_selects_bound_variables() {
        let query = multi_hop_query("BATCH001");
        assert!(query.contains("SELECT ?product ?tx1 ?tx2 ?tx3"));
        assert!(query.contains("GRAPH ?g2 { ?product trace:hasTransaction ?tx1 . }"));
        assert!(query.contains("OPTIONAL { GRAPH ?g3 { ?tx1 trace:nextTransaction ?tx2 . } }"));
        assert!(query.contains("OPTIONAL { GRAPH ?g4 { ?tx2 trace:nextTransaction ?tx3 . } }"));
    }

    #[tokio::test]
    async fn bulk_turtle_import_posts_dataset_once() {
        let mut server = Server::new_async().await;
        let _login = server
            .mock("POST", "/auth/login")
            .with_status(200)
            .with_body(r#"{"token":"benchmark-token"}"#)
            .create_async()
            .await;
        let _import = server
            .mock("POST", "/api/datasets/import-turtle")
            .match_header("authorization", "Bearer benchmark-token")
            .match_body(Matcher::Regex("ex:batch1".to_string()))
            .with_status(200)
            .with_body(
                r#"{
                    "success": true,
                    "import_mode": "bulk_turtle_single_block",
                    "triple_count": 1,
                    "block_count": 1,
                    "timings_ms": {
                        "handler_total": 12.5,
                        "block_admission": 11.0
                    }
                }"#,
            )
            .create_async()
            .await;

        let path = std::env::temp_dir().join(format!(
            "provchain-bulk-import-{}.ttl",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::write(
            &path,
            "@prefix ex: <http://example.com/> .\nex:batch1 ex:status \"ok\" .\n",
        )
        .expect("write temp Turtle dataset");

        let adapter = ProvChainAdapter::new(server.url());
        let timing = adapter
            .load_dataset_turtle_bulk_timed(&path)
            .await
            .expect("bulk Turtle import should succeed");

        assert_eq!(timing.triple_count, 1);
        assert_eq!(timing.block_count, 1);
        assert_eq!(timing.import_mode, "bulk_turtle_single_block");
        assert_eq!(timing.server_timing_samples, 1);
        assert_eq!(
            timing.server_timing_totals_ms.get("handler_total").copied(),
            Some(12.5)
        );

        let _ = fs::remove_file(path);
    }
}
