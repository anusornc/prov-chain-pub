use crate::core::result::TraceQueryResult;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterCapabilities {
    pub supports_ledger_write: bool,
    pub supports_trace_query: bool,
    pub supports_semantic_pipeline: bool,
    pub supports_native_rdf: bool,
    pub supports_native_jsonld: bool,
    pub supports_native_shacl: bool,
    pub supports_finality_measurement: bool,
}

#[async_trait]
pub trait BenchmarkAdapter {
    fn system_name(&self) -> &'static str;
    fn capabilities(&self) -> AdapterCapabilities;
}

#[async_trait]
pub trait TraceQueryAdapter: BenchmarkAdapter + Send + Sync {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult>;
    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult>;
    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult>;
}
