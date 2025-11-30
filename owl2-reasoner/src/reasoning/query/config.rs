//! Query configuration and statistics for OWL2 ontologies
//!
//! Contains configuration options, performance statistics, and query optimization settings.

use std::num::NonZeroUsize;
use std::time::Duration;

/// Query engine configuration with optimization options
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Maximum number of results to return
    pub max_results: Option<usize>,
    /// Enable query result caching
    pub enable_caching: bool,
    /// Enable reasoning for query evaluation
    pub enable_reasoning: bool,
    /// Enable parallel query execution
    pub enable_parallel: bool,
    /// Cache size for query results
    pub cache_size: Option<NonZeroUsize>,
    /// Timeout for query execution
    pub timeout: Option<Duration>,
    /// Enable query optimization
    pub enable_optimization: bool,
    /// Maximum memory usage in bytes
    pub max_memory: Option<usize>,
    /// Batch size for parallel processing
    pub batch_size: usize,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_results: Some(10000),
            enable_caching: true,
            enable_reasoning: true,
            enable_parallel: true,
            cache_size: NonZeroUsize::new(1000),
            timeout: Some(Duration::from_secs(30)),
            enable_optimization: true,
            max_memory: Some(100 * 1024 * 1024), // 100MB
            batch_size: 100,
        }
    }
}

impl QueryConfig {
    /// Create a new query configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Enable or disable query caching
    pub fn with_caching(mut self, enable: bool) -> Self {
        self.enable_caching = enable;
        self
    }

    /// Enable or disable reasoning
    pub fn with_reasoning(mut self, enable: bool) -> Self {
        self.enable_reasoning = enable;
        self
    }

    /// Enable or disable parallel execution
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.enable_parallel = enable;
        self
    }

    /// Set the cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = NonZeroUsize::new(size);
        self
    }

    /// Set the query timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable or disable query optimization
    pub fn with_optimization(mut self, enable: bool) -> Self {
        self.enable_optimization = enable;
        self
    }

    /// Set the maximum memory usage
    pub fn with_max_memory(mut self, memory: usize) -> Self {
        self.max_memory = Some(memory);
        self
    }

    /// Set the batch size for parallel processing
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Disable all optimizations for testing
    pub fn no_optimization() -> Self {
        Self {
            max_results: None,
            enable_caching: false,
            enable_reasoning: false,
            enable_parallel: false,
            cache_size: None,
            timeout: None,
            enable_optimization: false,
            max_memory: None,
            batch_size: 1,
        }
    }

    /// Check if caching should be used
    pub fn should_cache(&self) -> bool {
        self.enable_caching && self.cache_size.is_some()
    }

    /// Check if parallel execution should be used
    pub fn should_use_parallel(&self) -> bool {
        self.enable_parallel && self.batch_size > 1
    }

    /// Check if reasoning should be used
    pub fn should_use_reasoning(&self) -> bool {
        self.enable_reasoning
    }

    /// Check if optimization should be used
    pub fn should_optimize(&self) -> bool {
        self.enable_optimization
    }

    /// Get the effective cache size
    pub fn effective_cache_size(&self) -> usize {
        self.cache_size.map(|sz| sz.get()).unwrap_or(0)
    }

    /// Get the effective max results
    pub fn effective_max_results(&self) -> usize {
        self.max_results.unwrap_or(usize::MAX)
    }

    /// Check if a timeout is set
    pub fn has_timeout(&self) -> bool {
        self.timeout.is_some()
    }

    /// Check if memory limit is set
    pub fn has_memory_limit(&self) -> bool {
        self.max_memory.is_some()
    }
}

/// Query engine performance statistics
#[derive(Debug, Clone, Default)]
pub struct QueryEngineStats {
    /// Total number of queries executed
    pub total_queries: u64,
    /// Total number of successful queries
    pub successful_queries: u64,
    /// Total number of failed queries
    pub failed_queries: u64,
    /// Total query execution time in milliseconds
    pub total_time_ms: u64,
    /// Average query execution time in milliseconds
    pub average_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// Number of parallel executions
    pub parallel_executions: u64,
    /// Number of reasoning operations
    pub reasoning_operations: u64,
}

impl QueryEngineStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful query execution
    pub fn record_success(&mut self, time_ms: u64) {
        self.total_queries += 1;
        self.successful_queries += 1;
        self.total_time_ms += time_ms;
        self.update_average_time();
    }

    /// Record a failed query execution
    pub fn record_failure(&mut self, time_ms: u64) {
        self.total_queries += 1;
        self.failed_queries += 1;
        self.total_time_ms += time_ms;
        self.update_average_time();
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
        self.update_cache_hit_rate();
    }

    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
        self.update_cache_hit_rate();
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, usage: u64) {
        self.memory_usage = usage;
        if usage > self.peak_memory_usage {
            self.peak_memory_usage = usage;
        }
    }

    /// Record a parallel execution
    pub fn record_parallel_execution(&mut self) {
        self.parallel_executions += 1;
    }

    /// Record a reasoning operation
    pub fn record_reasoning_operation(&mut self) {
        self.reasoning_operations += 1;
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64
        }
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.failed_queries as f64 / self.total_queries as f64
        }
    }

    /// Calculate queries per second
    pub fn queries_per_second(&self) -> f64 {
        if self.total_time_ms == 0 {
            0.0
        } else {
            (self.total_queries as f64 * 1000.0) / self.total_time_ms as f64
        }
    }

    /// Calculate parallel execution rate
    pub fn parallel_execution_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.parallel_executions as f64 / self.total_queries as f64
        }
    }

    /// Calculate reasoning operation rate
    pub fn reasoning_operation_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.reasoning_operations as f64 / self.total_queries as f64
        }
    }

    /// Update average execution time
    fn update_average_time(&mut self) {
        if self.total_queries > 0 {
            self.average_time_ms = self.total_time_ms as f64 / self.total_queries as f64;
        }
    }

    /// Update cache hit rate
    fn update_cache_hit_rate(&mut self) {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hit_rate = self.cache_hits as f64 / total_requests as f64;
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get cache statistics summary
    pub fn cache_summary(&self) -> String {
        format!(
            "Cache: {:.1}% hit rate ({} hits, {} misses)",
            self.cache_hit_rate * 100.0,
            self.cache_hits,
            self.cache_misses
        )
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Performance: {:.1} QPS, {:.1}ms avg, {:.1}% success rate",
            self.queries_per_second(),
            self.average_time_ms,
            self.success_rate() * 100.0
        )
    }

    /// Get memory summary
    pub fn memory_summary(&self) -> String {
        format!(
            "Memory: {} MB current, {} MB peak",
            self.memory_usage / (1024 * 1024),
            self.peak_memory_usage / (1024 * 1024)
        )
    }

    /// Get comprehensive summary
    pub fn summary(&self) -> String {
        format!(
            "QueryEngineStats: {} total, {} successful, {} failed | {} | {} | {:.1}% parallel, {:.1}% reasoning",
            self.total_queries,
            self.successful_queries,
            self.failed_queries,
            self.performance_summary(),
            self.cache_summary(),
            self.parallel_execution_rate() * 100.0,
            self.reasoning_operation_rate() * 100.0
        )
    }
}
