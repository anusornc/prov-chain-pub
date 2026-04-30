use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FabricConfig {
    pub gateway_url: String,
    pub channel: String,
    pub chaincode: String,
}

impl Default for FabricConfig {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:8800".to_string(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        }
    }
}

pub struct FabricAdapter {
    config: FabricConfig,
    client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricRecordPayload {
    pub entity_id: String,
    pub entity_type: String,
    pub event_type: String,
    pub timestamp: String,
    pub actor_id: String,
    pub location_id: Option<String>,
    #[serde(default)]
    pub previous_record_ids: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricRecordPolicy {
    pub visibility: String,
    pub owner_org: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricRecord {
    pub record_id: String,
    pub payload: FabricRecordPayload,
    pub policy: FabricRecordPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricBatchRequest {
    pub records: Vec<FabricRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricSubmitResponse {
    pub success: bool,
    pub tx_id: Option<String>,
    #[serde(default)]
    pub submit_latency_ms: f64,
    #[serde(default)]
    pub commit_latency_ms: f64,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricBatchResponse {
    pub success: bool,
    #[serde(default)]
    pub submitted: usize,
    #[serde(default)]
    pub committed: usize,
    #[serde(default)]
    pub submit_latency_ms: f64,
    #[serde(default)]
    pub commit_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricPolicyCheckRequest {
    pub record_id: String,
    pub actor_org: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricPolicyCheckResponse {
    pub authorized: bool,
    #[serde(default)]
    pub policy_latency_ms: f64,
}

impl FabricAdapter {
    pub fn new(config: FabricConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.config.gateway_url))
            .send()
            .await
            .context("failed to call Fabric gateway health endpoint")?;
        Ok(response.status().is_success())
    }

    pub async fn submit_record(&self, record: &FabricRecord) -> Result<FabricSubmitResponse> {
        self.post_json("ledger/records", record)
            .await
            .context("failed to submit Fabric record")
    }

    pub async fn submit_batch(&self, records: Vec<FabricRecord>) -> Result<FabricBatchResponse> {
        self.post_json("ledger/records/batch", &FabricBatchRequest { records })
            .await
            .context("failed to submit Fabric record batch")
    }

    pub async fn check_policy(
        &self,
        request: &FabricPolicyCheckRequest,
    ) -> Result<FabricPolicyCheckResponse> {
        self.post_json("policy/check", request)
            .await
            .context("failed to check Fabric policy")
    }

    async fn post_json<T, R>(&self, path: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: for<'de> Deserialize<'de>,
    {
        let url = format!(
            "{}/{}",
            self.config.gateway_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("failed to call Fabric gateway endpoint {url}"))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("Fabric gateway request failed: {status} body={text}");
        }

        serde_json::from_str(&text)
            .with_context(|| format!("failed to decode Fabric gateway response: {text}"))
    }
}

impl BenchmarkAdapter for FabricAdapter {
    fn system_name(&self) -> &'static str {
        "Hyperledger Fabric"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: false,
            supports_ledger_write: true,
            supports_semantic_pipeline: false,
            supports_native_rdf: false,
            supports_native_jsonld: false,
            supports_native_shacl: false,
            supports_finality_measurement: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    fn sample_record() -> FabricRecord {
        FabricRecord {
            record_id: "record-001".to_string(),
            payload: FabricRecordPayload {
                entity_id: "BATCH001".to_string(),
                entity_type: "ProductBatch".to_string(),
                event_type: "Produced".to_string(),
                timestamp: "2026-04-24T00:00:00Z".to_string(),
                actor_id: "producer-001".to_string(),
                location_id: Some("site-001".to_string()),
                previous_record_ids: Vec::new(),
                attributes: HashMap::new(),
            },
            policy: FabricRecordPolicy {
                visibility: "public".to_string(),
                owner_org: "Org1MSP".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn health_check_returns_true_for_healthy_gateway() {
        let mut server = Server::new_async().await;
        let _health = server
            .mock("GET", "/health")
            .with_status(200)
            .with_body(r#"{"status":"ok","channel":"provchain","chaincode":"traceability"}"#)
            .create_async()
            .await;

        let adapter = FabricAdapter::new(FabricConfig {
            gateway_url: server.url(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        });

        assert!(adapter
            .health_check()
            .await
            .expect("health check should pass"));
    }

    #[tokio::test]
    async fn submit_record_decodes_commit_metrics() {
        let mut server = Server::new_async().await;
        let _submit = server
            .mock("POST", "/ledger/records")
            .match_header("content-type", Matcher::Regex("application/json.*".to_string()))
            .with_status(200)
            .with_body(
                r#"{"success":true,"tx_id":"tx-1","submit_latency_ms":12.3,"commit_latency_ms":45.6,"block_number":7}"#,
            )
            .create_async()
            .await;

        let adapter = FabricAdapter::new(FabricConfig {
            gateway_url: server.url(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        });

        let response = adapter
            .submit_record(&sample_record())
            .await
            .expect("submit should succeed");

        assert!(response.success);
        assert_eq!(response.tx_id.as_deref(), Some("tx-1"));
        assert_eq!(response.block_number, Some(7));
        assert_eq!(response.submit_latency_ms, 12.3);
        assert_eq!(response.commit_latency_ms, 45.6);
    }

    #[tokio::test]
    async fn submit_record_error_includes_response_body() {
        let mut server = Server::new_async().await;
        let _submit = server
            .mock("POST", "/ledger/records")
            .with_status(500)
            .with_body(r#"{"error":"endorsement failed"}"#)
            .create_async()
            .await;

        let adapter = FabricAdapter::new(FabricConfig {
            gateway_url: server.url(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        });

        let error = adapter
            .submit_record(&sample_record())
            .await
            .expect_err("submit should fail");

        assert!(format!("{error:#}").contains("endorsement failed"));
    }

    #[tokio::test]
    async fn submit_batch_decodes_counts_and_latencies() {
        let mut server = Server::new_async().await;
        let _submit = server
            .mock("POST", "/ledger/records/batch")
            .with_status(200)
            .with_body(
                r#"{"success":true,"submitted":2,"committed":2,"submit_latency_ms":20.0,"commit_latency_ms":70.0}"#,
            )
            .create_async()
            .await;

        let adapter = FabricAdapter::new(FabricConfig {
            gateway_url: server.url(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        });

        let response = adapter
            .submit_batch(vec![sample_record(), sample_record()])
            .await
            .expect("batch submit should succeed");

        assert!(response.success);
        assert_eq!(response.submitted, 2);
        assert_eq!(response.committed, 2);
        assert_eq!(response.commit_latency_ms, 70.0);
    }

    #[tokio::test]
    async fn check_policy_decodes_authorization_result() {
        let mut server = Server::new_async().await;
        let _policy = server
            .mock("POST", "/policy/check")
            .with_status(200)
            .with_body(r#"{"authorized":false,"policy_latency_ms":1.2}"#)
            .create_async()
            .await;

        let adapter = FabricAdapter::new(FabricConfig {
            gateway_url: server.url(),
            channel: "provchain".to_string(),
            chaincode: "traceability".to_string(),
        });

        let response = adapter
            .check_policy(&FabricPolicyCheckRequest {
                record_id: "record-001".to_string(),
                actor_org: "Org2MSP".to_string(),
                action: "read".to_string(),
            })
            .await
            .expect("policy check should succeed");

        assert!(!response.authorized);
        assert_eq!(response.policy_latency_ms, 1.2);
    }
}
