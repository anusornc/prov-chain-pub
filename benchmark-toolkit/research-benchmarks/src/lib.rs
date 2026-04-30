//! ProvChain Research Benchmarks Library
//!
//! This library provides benchmarking tools for comparing ProvChain-Org
//! with other systems like Neo4j and future competitive baselines.

pub mod adapters;
pub mod core;
pub mod dataset;
pub mod neo4j_client;
pub mod workloads;

// Re-export commonly used types
pub use adapters::fabric::{
    FabricAdapter, FabricBatchRequest, FabricBatchResponse, FabricConfig, FabricPolicyCheckRequest,
    FabricPolicyCheckResponse, FabricRecord, FabricRecordPayload, FabricRecordPolicy,
    FabricSubmitResponse,
};
pub use adapters::fluree::{FlureeAdapter, FlureeConfig};
pub use adapters::geth::{GethAdapter, GethConfig, GethTransactionRequest};
pub use adapters::neo4j::Neo4jTraceAdapter;
pub use adapters::provchain::{
    ProvChainAdapter, ProvChainPolicyCheckRequest, ProvChainPolicyCheckResponse,
};
pub use core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
pub use core::result::{
    BenchmarkFamily, BenchmarkResult, BenchmarkSummary, CapabilityPath, FairnessLabel, MetricType,
    MetricUnit, SystemSummary, TraceQueryResult,
};
pub use neo4j_client::{Neo4jBenchmarkResult, Neo4jClient, Neo4jClientConfig};
pub use workloads::provchain_queries::{
    aggregation_by_producer_query, entity_lookup_query, multi_hop_query,
};
pub use workloads::trace_query::{
    default_trace_query_scenarios, parse_batch_ids, TraceQueryKind, TraceQueryScenario,
};

use std::collections::HashMap;

/// Helper functions for benchmark calculations
pub mod utils {
    use super::*;

    /// Calculate average duration from a list of results
    pub fn average_duration(results: &[BenchmarkResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        results.iter().map(|r| r.duration_ms).sum::<f64>() / results.len() as f64
    }

    /// Calculate average operations per second
    pub fn average_ops_per_sec(results: &[BenchmarkResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        results.iter().map(|r| r.operations_per_second).sum::<f64>() / results.len() as f64
    }

    /// Calculate success rate as a percentage
    pub fn success_rate(results: &[BenchmarkResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let success_count = results.iter().filter(|r| r.success).count();
        (success_count as f64 / results.len() as f64) * 100.0
    }

    /// Group results by test name
    pub fn group_by_test(results: &[BenchmarkResult]) -> HashMap<String, Vec<&BenchmarkResult>> {
        let mut grouped: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in results {
            let key = format!("{}:{}", result.scenario, result.test_name);
            grouped.entry(key).or_default().push(result);
        }
        grouped
    }

    /// Calculate statistics for a group of results
    pub fn calculate_stats(results: &[&BenchmarkResult]) -> BenchmarkStats {
        if results.is_empty() {
            return BenchmarkStats::default();
        }

        let mut durations: Vec<f64> = results.iter().map(|r| r.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = durations.len();
        let sum: f64 = durations.iter().sum();
        let mean = sum / count as f64;

        let min = *durations.first().unwrap();
        let max = *durations.last().unwrap();

        let median = if count % 2 == 0 {
            (durations[count / 2 - 1] + durations[count / 2]) / 2.0
        } else {
            durations[count / 2]
        };

        // Standard deviation
        let variance: f64 =
            durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        // 95th percentile
        let p95_idx = ((count as f64) * 0.95) as usize;
        let p95 = durations[p95_idx.min(count - 1)];

        BenchmarkStats {
            count,
            min,
            max,
            mean,
            median,
            std_dev,
            p95,
        }
    }
}

/// Statistical metrics for benchmark results
#[derive(Debug, Clone, Copy, Default)]
pub struct BenchmarkStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub p95: f64,
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_benchmark_result_new() {
        let result = BenchmarkResult::new(
            "System",
            BenchmarkFamily::TraceQuery,
            MetricType::QueryLatencyMs,
            "Scenario",
            "Test",
            0,
            100.0,
        );
        assert_eq!(result.system, "System");
        assert_eq!(result.duration_ms, 100.0);
        assert_eq!(result.operations_per_second, 10.0); // 1000/100
    }

    #[test]
    fn test_average_duration() {
        let results = vec![
            BenchmarkResult::new(
                "S",
                BenchmarkFamily::TraceQuery,
                MetricType::QueryLatencyMs,
                "Sc",
                "T",
                0,
                100.0,
            ),
            BenchmarkResult::new(
                "S",
                BenchmarkFamily::TraceQuery,
                MetricType::QueryLatencyMs,
                "Sc",
                "T",
                1,
                200.0,
            ),
            BenchmarkResult::new(
                "S",
                BenchmarkFamily::TraceQuery,
                MetricType::QueryLatencyMs,
                "Sc",
                "T",
                2,
                300.0,
            ),
        ];
        assert_eq!(average_duration(&results), 200.0);
    }

    #[test]
    fn test_calculate_stats() {
        let results: Vec<BenchmarkResult> = (0..10)
            .map(|i| {
                BenchmarkResult::new(
                    "S",
                    BenchmarkFamily::TraceQuery,
                    MetricType::QueryLatencyMs,
                    "Sc",
                    "T",
                    i,
                    (i as f64) * 10.0,
                )
            })
            .collect();

        let refs: Vec<&BenchmarkResult> = results.iter().collect();
        let stats = calculate_stats(&refs);

        assert_eq!(stats.count, 10);
        assert_eq!(stats.min, 0.0);
        assert_eq!(stats.max, 90.0);
    }
}
