use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct GethConfig {
    pub rpc_url: String,
}

impl Default for GethConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
        }
    }
}

pub struct GethAdapter {
    config: GethConfig,
    client: Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GethReceipt {
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<String>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    #[serde(rename = "effectiveGasPrice")]
    pub effective_gas_price: Option<String>,
}

impl GethReceipt {
    pub fn is_success(&self) -> bool {
        self.status.as_deref() == Some("0x1")
    }
}

#[derive(Debug, Clone)]
pub struct GethSubmitResult {
    pub tx_hash: String,
    pub submit_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GethTransactionRequest {
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GethConfirmationResult {
    pub receipt: GethReceipt,
    pub confirmation_latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    method: &'a str,
    params: Value,
    id: u64,
}

impl GethAdapter {
    pub fn new(config: GethConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(self.client_version().await.is_ok())
    }

    pub async fn client_version(&self) -> Result<String> {
        self.rpc("web3_clientVersion", serde_json::json!([]))
            .await
            .context("failed to call Geth client version")
    }

    pub async fn chain_id(&self) -> Result<String> {
        self.rpc("eth_chainId", serde_json::json!([]))
            .await
            .context("failed to call Geth chain id")
    }

    pub async fn accounts(&self) -> Result<Vec<String>> {
        self.rpc("eth_accounts", serde_json::json!([]))
            .await
            .context("failed to list Geth accounts")
    }

    pub async fn web3_sha3(&self, hex_data: &str) -> Result<String> {
        self.rpc("web3_sha3", serde_json::json!([hex_data]))
            .await
            .context("failed to call Geth web3_sha3")
    }

    pub async fn validate_contract_address(&self, address: &str) -> Result<bool> {
        let code: String = self
            .rpc("eth_getCode", serde_json::json!([address, "latest"]))
            .await
            .with_context(|| format!("failed to validate Geth contract address {address}"))?;
        let code = code.trim();
        Ok(!code.is_empty() && code != "0x" && code != "0x0")
    }

    pub async fn submit_raw_transaction(&self, raw_tx: &str) -> Result<GethSubmitResult> {
        let start = Instant::now();
        let tx_hash: String = self
            .rpc("eth_sendRawTransaction", serde_json::json!([raw_tx]))
            .await
            .context("failed to submit Geth raw transaction")?;
        Ok(GethSubmitResult {
            tx_hash,
            submit_latency_ms: elapsed_ms(start),
        })
    }

    pub async fn send_transaction(
        &self,
        request: &GethTransactionRequest,
    ) -> Result<GethSubmitResult> {
        let start = Instant::now();
        let tx_hash: String = self
            .rpc("eth_sendTransaction", serde_json::json!([request]))
            .await
            .context("failed to submit Geth transaction")?;
        Ok(GethSubmitResult {
            tx_hash,
            submit_latency_ms: elapsed_ms(start),
        })
    }

    pub async fn transaction_receipt(&self, tx_hash: &str) -> Result<Option<GethReceipt>> {
        let method = "eth_getTransactionReceipt";
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&JsonRpcRequest {
                jsonrpc: "2.0",
                method,
                params: serde_json::json!([tx_hash]),
                id: 1,
            })
            .send()
            .await
            .with_context(|| format!("failed to fetch Geth transaction receipt {tx_hash}"))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("Geth RPC method {method} returned HTTP {status}");
        }

        let payload: JsonRpcResponse<Option<GethReceipt>> = response
            .json()
            .await
            .with_context(|| format!("failed to decode Geth RPC response for {method}"))?;

        if let Some(error) = payload.error {
            let message = error.message.to_ascii_lowercase();
            if message.contains("not found") || message.contains("indexing is in progress") {
                return Ok(None);
            }
            anyhow::bail!(
                "Geth RPC method {method} failed: code={} message={}",
                error.code,
                error.message
            );
        }

        Ok(payload.result.flatten())
    }

    pub async fn wait_for_receipt(
        &self,
        tx_hash: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<GethConfirmationResult> {
        let start = Instant::now();
        loop {
            if let Some(receipt) = self.transaction_receipt(tx_hash).await? {
                return Ok(GethConfirmationResult {
                    success: receipt.is_success(),
                    receipt,
                    confirmation_latency_ms: elapsed_ms(start),
                });
            }

            if start.elapsed() >= timeout {
                anyhow::bail!("timed out waiting for Geth transaction receipt {tx_hash}");
            }

            sleep(poll_interval).await;
        }
    }

    async fn rpc<T>(&self, method: &str, params: Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&JsonRpcRequest {
                jsonrpc: "2.0",
                method,
                params,
                id: 1,
            })
            .send()
            .await
            .with_context(|| format!("failed to call Geth RPC method {method}"))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("Geth RPC method {method} returned HTTP {status}");
        }

        let payload: JsonRpcResponse<T> = response
            .json()
            .await
            .with_context(|| format!("failed to decode Geth RPC response for {method}"))?;

        if let Some(error) = payload.error {
            anyhow::bail!(
                "Geth RPC method {method} failed: code={} message={}",
                error.code,
                error.message
            );
        }

        payload
            .result
            .with_context(|| format!("Geth RPC method {method} returned no result"))
    }
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

impl BenchmarkAdapter for GethAdapter {
    fn system_name(&self) -> &'static str {
        "Go Ethereum (Geth)"
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
    use mockito::Server;

    fn adapter(server: &Server) -> GethAdapter {
        GethAdapter::new(GethConfig {
            rpc_url: server.url(),
        })
    }

    #[tokio::test]
    async fn health_check_decodes_client_version() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"result":"Geth/v1.13.15-stable/linux-amd64/go1.21"}"#,
            )
            .create_async()
            .await;

        assert!(adapter(&server)
            .health_check()
            .await
            .expect("health check should decode client version"));
    }

    #[tokio::test]
    async fn validate_contract_address_requires_code() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0x6080604052"}"#)
            .create_async()
            .await;

        assert!(adapter(&server)
            .validate_contract_address("0x1111111111111111111111111111111111111111")
            .await
            .expect("contract code should validate"));
    }

    #[tokio::test]
    async fn submit_raw_transaction_returns_hash_and_latency() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xabc123"}"#)
            .create_async()
            .await;

        let result = adapter(&server)
            .submit_raw_transaction("0xdeadbeef")
            .await
            .expect("raw transaction should submit");

        assert_eq!(result.tx_hash, "0xabc123");
        assert!(result.submit_latency_ms >= 0.0);
    }

    #[tokio::test]
    async fn accounts_decodes_unlocked_sender_list() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"result":["0x1111111111111111111111111111111111111111"]}"#,
            )
            .create_async()
            .await;

        let accounts = adapter(&server)
            .accounts()
            .await
            .expect("accounts should decode");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0], "0x1111111111111111111111111111111111111111");
    }

    #[tokio::test]
    async fn send_transaction_returns_hash_and_latency() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"jsonrpc":"2.0","id":1,"result":"0xabc123"}"#)
            .create_async()
            .await;

        let result = adapter(&server)
            .send_transaction(&GethTransactionRequest {
                from: "0x1111111111111111111111111111111111111111".to_string(),
                to: Some("0x2222222222222222222222222222222222222222".to_string()),
                data: "0x12345678".to_string(),
                gas: Some("0x100000".to_string()),
            })
            .await
            .expect("transaction should submit");

        assert_eq!(result.tx_hash, "0xabc123");
        assert!(result.submit_latency_ms >= 0.0);
    }

    #[tokio::test]
    async fn wait_for_receipt_decodes_confirmation_metadata() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"result":{"transactionHash":"0xabc123","blockNumber":"0x2a","status":"0x1","gasUsed":"0x5208","effectiveGasPrice":"0x3b9aca00"}}"#,
            )
            .create_async()
            .await;

        let result = adapter(&server)
            .wait_for_receipt("0xabc123", Duration::from_secs(1), Duration::from_millis(1))
            .await
            .expect("receipt should confirm");

        assert!(result.success);
        assert_eq!(result.receipt.transaction_hash, "0xabc123");
        assert_eq!(result.receipt.block_number.as_deref(), Some("0x2a"));
        assert_eq!(result.receipt.gas_used.as_deref(), Some("0x5208"));
        assert_eq!(
            result.receipt.effective_gas_price.as_deref(),
            Some("0x3b9aca00")
        );
        assert!(result.confirmation_latency_ms >= 0.0);
    }

    #[tokio::test]
    async fn transaction_receipt_treats_indexing_progress_as_pending() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"transaction indexing is in progress"}}"#,
            )
            .create_async()
            .await;

        let result = adapter(&server)
            .transaction_receipt("0xabc123")
            .await
            .expect("indexing progress should be treated as pending");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn transaction_receipt_treats_not_found_as_pending() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"not found"}}"#)
            .create_async()
            .await;

        let result = adapter(&server)
            .transaction_receipt("0xabc123")
            .await
            .expect("not found should be treated as pending");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn failed_receipt_is_returned_as_unsuccessful_confirmation() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"result":{"transactionHash":"0xdef456","blockNumber":"0x2b","status":"0x0","gasUsed":"0x5208"}}"#,
            )
            .create_async()
            .await;

        let result = adapter(&server)
            .wait_for_receipt("0xdef456", Duration::from_secs(1), Duration::from_millis(1))
            .await
            .expect("failed receipt should still be decoded");

        assert!(!result.success);
        assert_eq!(result.receipt.status.as_deref(), Some("0x0"));
    }
}
