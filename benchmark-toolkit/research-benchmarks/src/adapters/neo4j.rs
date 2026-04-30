use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
use crate::core::result::TraceQueryResult;
use crate::neo4j_client::Neo4jClient;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

pub struct Neo4jTraceAdapter<'a> {
    client: &'a Neo4jClient,
}

impl<'a> Neo4jTraceAdapter<'a> {
    pub fn new(client: &'a Neo4jClient) -> Self {
        Self { client }
    }

    pub async fn load_dataset_turtle(&self, dataset_path: &Path) -> Result<Duration> {
        self.client.clear_all_data().await?;
        self.client.load_turtle_data(dataset_path).await
    }
}

#[async_trait]
impl BenchmarkAdapter for Neo4jTraceAdapter<'_> {
    fn system_name(&self) -> &'static str {
        "Neo4j"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: true,
            supports_ledger_write: true,
            supports_semantic_pipeline: false,
            supports_native_rdf: false,
            supports_native_jsonld: false,
            supports_native_shacl: false,
            supports_finality_measurement: false,
        }
    }
}

#[async_trait]
impl TraceQueryAdapter for Neo4jTraceAdapter<'_> {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult> {
        let result = self.client.query_product_by_batch_id(id).await?;
        Ok(
            TraceQueryResult::new("Neo4j", "Simple Product Lookup", result.duration_ms)
                .with_record_count(result.record_count),
        )
    }

    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult> {
        let result = self.client.query_multi_hop_traceability(id, hops).await?;
        Ok(TraceQueryResult::new(
            "Neo4j",
            format!("Multi-hop Traceability ({} hops)", hops),
            result.duration_ms,
        )
        .with_record_count(result.record_count))
    }

    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult> {
        let result = self.client.query_aggregation_by_producer().await?;
        Ok(
            TraceQueryResult::new("Neo4j", "Aggregation by Producer", result.duration_ms)
                .with_record_count(result.record_count),
        )
    }
}
