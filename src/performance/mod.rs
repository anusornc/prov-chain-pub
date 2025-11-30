//! ProvChain Performance & Scalability Module
//!
//! This module provides comprehensive performance optimization features including:
//! - RDF canonicalization caching and optimization
//! - Database performance enhancements
//! - Concurrent operations optimization
//! - Horizontal scaling capabilities
//! - Storage optimization features

pub mod canonicalization_cache;
pub mod concurrent_operations;
pub mod database_optimization;
pub mod memory_optimization;
pub mod metrics;
pub mod scaling;
pub mod storage_optimization;

use std::collections::HashMap;
use std::time::Duration;

/// Performance configuration for ProvChain
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable RDF canonicalization caching
    pub enable_canonicalization_cache: bool,
    /// Maximum cache size for canonical hashes
    pub max_cache_size: usize,
    /// Enable SPARQL query result caching
    pub enable_query_cache: bool,
    /// Maximum number of cached query results
    pub max_query_cache_size: usize,
    /// Enable concurrent operations optimization
    pub enable_concurrent_optimization: bool,
    /// Maximum number of worker threads
    pub max_worker_threads: usize,
    /// Enable storage compression
    pub enable_storage_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_canonicalization_cache: true,
            max_cache_size: 10000,
            enable_query_cache: true,
            max_query_cache_size: 1000,
            enable_concurrent_optimization: true,
            max_worker_threads: num_cpus::get(),
            enable_storage_compression: true,
            compression_level: 6,
            enable_performance_monitoring: true,
            metrics_interval: Duration::from_secs(60),
        }
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Cache hit rate for canonicalization
    pub canonicalization_cache_hit_rate: f64,
    /// Cache hit rate for SPARQL queries
    pub query_cache_hit_rate: f64,
    /// Average block processing time
    pub avg_block_processing_time: Duration,
    /// Average SPARQL query time
    pub avg_query_time: Duration,
    /// Current memory usage in MB
    pub memory_usage_mb: u64,
    /// Storage compression ratio
    pub compression_ratio: f64,
    /// Concurrent operations throughput
    pub concurrent_throughput: f64,
    /// Total operations processed
    pub total_operations: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            canonicalization_cache_hit_rate: 0.0,
            query_cache_hit_rate: 0.0,
            avg_block_processing_time: Duration::new(0, 0),
            avg_query_time: Duration::new(0, 0),
            memory_usage_mb: 0,
            compression_ratio: 1.0,
            concurrent_throughput: 0.0,
            total_operations: 0,
        }
    }
}

impl PerformanceMetrics {
    /// Print a summary of performance metrics
    pub fn print_summary(&self) {
        println!("\n=== ProvChain Performance Metrics ===");
        println!(
            "Canonicalization cache hit rate: {:.2}%",
            self.canonicalization_cache_hit_rate * 100.0
        );
        println!(
            "Query cache hit rate: {:.2}%",
            self.query_cache_hit_rate * 100.0
        );
        println!(
            "Average block processing time: {:?}",
            self.avg_block_processing_time
        );
        println!("Average query time: {:?}", self.avg_query_time);
        println!("Memory usage: {} MB", self.memory_usage_mb);
        println!("Storage compression ratio: {:.2}:1", self.compression_ratio);
        println!(
            "Concurrent throughput: {:.2} ops/sec",
            self.concurrent_throughput
        );
        println!("Total operations processed: {}", self.total_operations);
        println!("=====================================\n");
    }

    /// Calculate overall performance score (0-100)
    pub fn calculate_performance_score(&self) -> f64 {
        let cache_score = (self.canonicalization_cache_hit_rate + self.query_cache_hit_rate) * 25.0;
        let speed_score = if self.avg_block_processing_time.as_millis() < 100 {
            25.0
        } else {
            25.0 * (100.0 / self.avg_block_processing_time.as_millis() as f64)
        };
        let memory_score = if self.memory_usage_mb < 100 {
            25.0
        } else {
            25.0 * (100.0 / self.memory_usage_mb as f64)
        };
        let compression_score = (self.compression_ratio - 1.0) * 25.0;

        (cache_score + speed_score + memory_score + compression_score).min(100.0)
    }
}

/// Performance optimization manager
pub struct PerformanceManager {
    config: PerformanceConfig,
    metrics: PerformanceMetrics,
    canonicalization_cache: canonicalization_cache::CanonicalizationCache,
    query_cache: database_optimization::QueryCache,
    concurrent_manager: concurrent_operations::ConcurrentManager,
    storage_optimizer: storage_optimization::StorageOptimizer,
    metrics_collector: metrics::MetricsCollector,
}

impl PerformanceManager {
    /// Create a new performance manager with default configuration
    pub fn new() -> Self {
        let config = PerformanceConfig::default();
        Self::with_config(config)
    }

    /// Create a new performance manager with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            canonicalization_cache: canonicalization_cache::CanonicalizationCache::new(
                config.max_cache_size,
            ),
            query_cache: database_optimization::QueryCache::new(config.max_query_cache_size),
            concurrent_manager: concurrent_operations::ConcurrentManager::new(
                config.max_worker_threads,
            ),
            storage_optimizer: storage_optimization::StorageOptimizer::new(
                config.compression_level,
            ),
            metrics_collector: metrics::MetricsCollector::new(config.metrics_interval),
            metrics: PerformanceMetrics::default(),
            config,
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Update performance metrics
    pub fn update_metrics(&mut self) {
        self.metrics.canonicalization_cache_hit_rate = self.canonicalization_cache.get_hit_rate();
        self.metrics.query_cache_hit_rate = self.query_cache.get_hit_rate();
        self.metrics.memory_usage_mb = self.estimate_memory_usage();
        self.metrics.compression_ratio = self.storage_optimizer.get_compression_ratio();
        self.metrics.concurrent_throughput = self.concurrent_manager.get_throughput();
        self.metrics.total_operations = self.metrics_collector.get_total_operations();
    }

    /// Estimate current memory usage
    fn estimate_memory_usage(&self) -> u64 {
        let cache_memory = self.canonicalization_cache.estimate_memory_usage();
        let query_cache_memory = self.query_cache.estimate_memory_usage();
        let concurrent_memory = self.concurrent_manager.estimate_memory_usage();

        (cache_memory + query_cache_memory + concurrent_memory) as u64 / 1024 / 1024
        // Convert to MB
    }

    /// Get performance configuration
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// Update performance configuration
    pub fn update_config(&mut self, config: PerformanceConfig) {
        self.config = config;
        // Update component configurations
        self.canonicalization_cache
            .resize(self.config.max_cache_size);
        self.query_cache.resize(self.config.max_query_cache_size);
        self.concurrent_manager
            .set_max_threads(self.config.max_worker_threads);
        self.storage_optimizer
            .set_compression_level(self.config.compression_level);
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.canonicalization_cache.clear();
        self.query_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert(
            "canonicalization_cache_hit_rate".to_string(),
            self.canonicalization_cache.get_hit_rate(),
        );
        stats.insert(
            "canonicalization_cache_size".to_string(),
            self.canonicalization_cache.size() as f64,
        );
        stats.insert(
            "query_cache_hit_rate".to_string(),
            self.query_cache.get_hit_rate(),
        );
        stats.insert(
            "query_cache_size".to_string(),
            self.query_cache.size() as f64,
        );
        stats
    }
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.enable_canonicalization_cache);
        assert!(config.enable_query_cache);
        assert!(config.enable_concurrent_optimization);
        assert!(config.enable_storage_compression);
        assert!(config.enable_performance_monitoring);
        assert!(config.max_cache_size > 0);
        assert!(config.max_worker_threads > 0);
    }

    #[test]
    fn test_performance_metrics_score() {
        let mut metrics = PerformanceMetrics::default();
        metrics.canonicalization_cache_hit_rate = 0.8;
        metrics.query_cache_hit_rate = 0.9;
        metrics.avg_block_processing_time = Duration::from_millis(50);
        metrics.memory_usage_mb = 50;
        metrics.compression_ratio = 2.0;

        let score = metrics.calculate_performance_score();
        assert!(score > 80.0); // Should be a high score with good metrics
    }

    #[test]
    fn test_performance_manager_creation() {
        let manager = PerformanceManager::new();
        assert!(manager.get_config().enable_canonicalization_cache);

        let custom_config = PerformanceConfig {
            max_cache_size: 5000,
            max_worker_threads: 4,
            ..Default::default()
        };

        let custom_manager = PerformanceManager::with_config(custom_config);
        assert_eq!(custom_manager.get_config().max_cache_size, 5000);
        assert_eq!(custom_manager.get_config().max_worker_threads, 4);
    }

    #[test]
    fn test_cache_stats() {
        let manager = PerformanceManager::new();
        let stats = manager.get_cache_stats();

        assert!(stats.contains_key("canonicalization_cache_hit_rate"));
        assert!(stats.contains_key("canonicalization_cache_size"));
        assert!(stats.contains_key("query_cache_hit_rate"));
        assert!(stats.contains_key("query_cache_size"));
    }
}
