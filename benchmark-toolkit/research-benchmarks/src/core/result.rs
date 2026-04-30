use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BenchmarkFamily {
    LedgerWrite,
    TraceQuery,
    Semantic,
    GovernancePolicy,
}

impl BenchmarkFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LedgerWrite => "ledger-write",
            Self::TraceQuery => "trace-query",
            Self::Semantic => "semantic",
            Self::GovernancePolicy => "governance-policy",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FairnessLabel {
    NativeComparable,
    SecondaryBaseline,
    PublicChainBaseline,
    ExternalizedSemanticPipeline,
    IndexedQueryStack,
    CrossModelWithCaveat,
    NotComparable,
}

impl FairnessLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeComparable => "native-comparable",
            Self::SecondaryBaseline => "secondary-baseline",
            Self::PublicChainBaseline => "public-chain-baseline",
            Self::ExternalizedSemanticPipeline => "externalized-semantic-pipeline",
            Self::IndexedQueryStack => "indexed-query-stack",
            Self::CrossModelWithCaveat => "cross-model-with-caveat",
            Self::NotComparable => "not-comparable",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityPath {
    Native,
    NativeRdfPath,
    PublicChainSmartContract,
    TranslatedGraphModel,
    ExternalSemanticPipeline,
    IndexedQueryStack,
    SecondaryTransactionalBaseline,
}

impl CapabilityPath {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::NativeRdfPath => "native-rdf-path",
            Self::PublicChainSmartContract => "public-chain-smart-contract",
            Self::TranslatedGraphModel => "translated-graph-model",
            Self::ExternalSemanticPipeline => "external-semantic-pipeline",
            Self::IndexedQueryStack => "indexed-query-stack",
            Self::SecondaryTransactionalBaseline => "secondary-transactional-baseline",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    QueryLatencyMs,
    QueryOpsPerSec,
    SubmitLatencyMs,
    CommitLatencyMs,
    ConfirmationLatencyMs,
    GasUsed,
    GasUsedPerRecord,
    ValidationLatencyMs,
    MappingLatencyMs,
    LoadLatencyMs,
    DatasetReadLatencyMs,
    DatasetNormalizeLatencyMs,
    DatasetParseLatencyMs,
    AuthenticationLatencyMs,
    ClientSubmitLoopLatencyMs,
    PolicyCheckLatencyMs,
    AuthorizedReadLatencyMs,
    UnauthorizedRejectionLatencyMs,
    AuthorizedWriteLatencyMs,
    RejectedWriteLatencyMs,
}

impl MetricType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QueryLatencyMs => "query-latency-ms",
            Self::QueryOpsPerSec => "query-ops-per-sec",
            Self::SubmitLatencyMs => "submit-latency-ms",
            Self::CommitLatencyMs => "commit-latency-ms",
            Self::ConfirmationLatencyMs => "confirmation-latency-ms",
            Self::GasUsed => "gas-used",
            Self::GasUsedPerRecord => "gas-used-per-record",
            Self::ValidationLatencyMs => "validation-latency-ms",
            Self::MappingLatencyMs => "mapping-latency-ms",
            Self::LoadLatencyMs => "load-latency-ms",
            Self::DatasetReadLatencyMs => "dataset-read-latency-ms",
            Self::DatasetNormalizeLatencyMs => "dataset-normalize-latency-ms",
            Self::DatasetParseLatencyMs => "dataset-parse-latency-ms",
            Self::AuthenticationLatencyMs => "authentication-latency-ms",
            Self::ClientSubmitLoopLatencyMs => "client-submit-loop-latency-ms",
            Self::PolicyCheckLatencyMs => "policy-check-latency-ms",
            Self::AuthorizedReadLatencyMs => "authorized-read-latency-ms",
            Self::UnauthorizedRejectionLatencyMs => "unauthorized-rejection-latency-ms",
            Self::AuthorizedWriteLatencyMs => "authorized-write-latency-ms",
            Self::RejectedWriteLatencyMs => "rejected-write-latency-ms",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricUnit {
    Milliseconds,
    OperationsPerSecond,
    Gas,
}

impl MetricUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Milliseconds => "ms",
            Self::OperationsPerSecond => "ops/sec",
            Self::Gas => "gas",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceQueryResult {
    pub family: BenchmarkFamily,
    pub system: String,
    pub scenario: String,
    pub duration_ms: f64,
    pub success: bool,
    pub record_count: usize,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TraceQueryResult {
    pub fn new(system: impl Into<String>, scenario: impl Into<String>, duration_ms: f64) -> Self {
        Self {
            family: BenchmarkFamily::TraceQuery,
            system: system.into(),
            scenario: scenario.into(),
            duration_ms,
            success: true,
            record_count: 0,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_record_count(mut self, record_count: usize) -> Self {
        self.record_count = record_count;
        self
    }

    pub fn with_error(mut self, error_message: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error_message.into());
        self
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Standard benchmark result structure used across all systems
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkResult {
    pub family: BenchmarkFamily,
    pub fairness_label: String,
    pub capability_path: String,
    pub metric_type: MetricType,
    pub unit: String,
    pub system: String,
    pub scenario: String,
    pub test_name: String,
    pub iteration: usize,
    pub duration_ms: f64,
    pub operations_per_second: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BenchmarkResult {
    pub fn new(
        system: impl Into<String>,
        family: BenchmarkFamily,
        metric_type: MetricType,
        scenario: impl Into<String>,
        test_name: impl Into<String>,
        iteration: usize,
        duration_ms: f64,
    ) -> Self {
        let ops_per_sec = if duration_ms > 0.0 {
            1000.0 / duration_ms
        } else {
            0.0
        };

        Self {
            family,
            fairness_label: FairnessLabel::NativeComparable.as_str().to_string(),
            capability_path: CapabilityPath::Native.as_str().to_string(),
            metric_type,
            unit: MetricUnit::Milliseconds.as_str().to_string(),
            system: system.into(),
            scenario: scenario.into(),
            test_name: test_name.into(),
            iteration,
            duration_ms,
            operations_per_second: ops_per_sec,
            success: true,
            error_message: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn from_trace_result(
        trace_result: TraceQueryResult,
        iteration: usize,
        scenario: impl Into<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        let mut merged = trace_result.metadata;
        merged.extend(metadata);
        if trace_result.record_count > 0 {
            merged.insert("record_count".to_string(), trace_result.record_count.into());
        }

        Self {
            family: trace_result.family,
            fairness_label: FairnessLabel::NativeComparable.as_str().to_string(),
            capability_path: CapabilityPath::Native.as_str().to_string(),
            metric_type: MetricType::QueryLatencyMs,
            unit: MetricUnit::Milliseconds.as_str().to_string(),
            system: trace_result.system,
            scenario: scenario.into(),
            test_name: trace_result.scenario,
            iteration,
            duration_ms: trace_result.duration_ms,
            operations_per_second: 1000.0 / trace_result.duration_ms.max(1.0),
            success: trace_result.success,
            error_message: trace_result.error_message,
            timestamp: Utc::now(),
            metadata: merged,
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_metric_unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit.as_str().to_string();
        self
    }

    pub fn with_fairness_label(mut self, label: FairnessLabel) -> Self {
        self.fairness_label = label.as_str().to_string();
        self
    }

    pub fn with_capability_path(mut self, path: CapabilityPath) -> Self {
        self.capability_path = path.as_str().to_string();
        self
    }
}

/// Benchmark summary for comparing systems
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub family: BenchmarkFamily,
    pub scenario: String,
    pub test_name: String,
    pub fairness_label: String,
    pub comparison_status: String,
    pub systems: Vec<SystemSummary>,
    pub winner: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSummary {
    pub system: String,
    pub fairness_label: String,
    pub capability_path: String,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub ops_per_sec: f64,
    pub success_rate: f64,
    pub total_runs: usize,
    pub successful_runs: usize,
}
