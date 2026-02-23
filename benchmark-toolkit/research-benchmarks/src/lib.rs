//! ProvChain Research Benchmarks Library
//!
//! This library provides benchmarking tools for comparing ProvChain-Org
//! with other systems like Neo4j and Apache Jena Fuseki.

pub mod jena_client;
pub mod neo4j_client;

// Re-export commonly used types
pub use jena_client::{JenaClient, JenaClientConfig, JenaBenchmarkResult, QueryExecutionResult};
pub use neo4j_client::{Neo4jClient, Neo4jClientConfig, Neo4jBenchmarkResult};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard benchmark result structure used across all systems
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkResult {
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
    /// Create a new benchmark result
    pub fn new(
        system: impl Into<String>,
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

    /// Mark the result as failed with an error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Benchmark summary for comparing systems
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub scenario: String,
    pub provchain_avg_ms: f64,
    pub neo4j_avg_ms: f64,
    pub jena_avg_ms: f64,
    pub provchain_ops_per_sec: f64,
    pub neo4j_ops_per_sec: f64,
    pub jena_ops_per_sec: f64,
    pub improvement_percent: f64,
    pub winner: String,
}

impl Default for BenchmarkSummary {
    fn default() -> Self {
        Self {
            scenario: String::new(),
            provchain_avg_ms: 0.0,
            neo4j_avg_ms: 0.0,
            jena_avg_ms: 0.0,
            provchain_ops_per_sec: 0.0,
            neo4j_ops_per_sec: 0.0,
            jena_ops_per_sec: 0.0,
            improvement_percent: 0.0,
            winner: "Unknown".to_string(),
        }
    }
}

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
        let variance: f64 = durations
            .iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f64>() / count as f64;
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
    use super::*;
    use super::utils::*;

    #[test]
    fn test_benchmark_result_new() {
        let result = BenchmarkResult::new("System", "Scenario", "Test", 0, 100.0);
        assert_eq!(result.system, "System");
        assert_eq!(result.duration_ms, 100.0);
        assert_eq!(result.operations_per_second, 10.0); // 1000/100
    }

    #[test]
    fn test_average_duration() {
        let results = vec![
            BenchmarkResult::new("S", "Sc", "T", 0, 100.0),
            BenchmarkResult::new("S", "Sc", "T", 1, 200.0),
            BenchmarkResult::new("S", "Sc", "T", 2, 300.0),
        ];
        assert_eq!(average_duration(&results), 200.0);
    }

    #[test]
    fn test_calculate_stats() {
        let results: Vec<BenchmarkResult> = (0..10)
            .map(|i| BenchmarkResult::new("S", "Sc", "T", i, (i as f64) * 10.0))
            .collect();
        
        let refs: Vec<&BenchmarkResult> = results.iter().collect();
        let stats = calculate_stats(&refs);
        
        assert_eq!(stats.count, 10);
        assert_eq!(stats.min, 0.0);
        assert_eq!(stats.max, 90.0);
    }
}
