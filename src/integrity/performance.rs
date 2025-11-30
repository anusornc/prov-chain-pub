//! Performance optimization system for integrity validation
//!
//! This module provides configurable validation levels, performance optimization,
//! and background monitoring capabilities for production deployment.

use crate::core::blockchain::Blockchain;
use crate::error::Result;
use crate::integrity::{
    BlockchainIntegrityStatus, CanonicalizationIntegrityStatus, IntegrityStatus,
    IntegrityValidationReport, IntegrityValidator, SparqlIntegrityStatus,
    TransactionCountIntegrityStatus,
};
use crate::storage::rdf_store::RDFStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, info, instrument, warn};

/// Configurable validation levels for different deployment scenarios
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Minimal validation for high-performance scenarios
    Minimal,
    /// Standard validation for normal operations
    Standard,
    /// Comprehensive validation for critical systems
    Comprehensive,
    /// Full validation with all checks enabled
    Full,
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Validation level to use
    pub validation_level: ValidationLevel,
    /// Enable parallel validation where possible
    pub enable_parallel_validation: bool,
    /// Maximum validation time in seconds
    pub max_validation_time: u64,
    /// Enable validation result caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable incremental validation
    pub enable_incremental_validation: bool,
    /// Batch size for large dataset processing
    pub batch_size: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            validation_level: ValidationLevel::Standard,
            enable_parallel_validation: true,
            max_validation_time: 300, // 5 minutes
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            enable_incremental_validation: true,
            batch_size: 1000,
            enable_profiling: false,
        }
    }
}

/// Validation cache for performance optimization
#[derive(Debug, Clone)]
pub struct ValidationCache {
    /// Cached validation results
    cache: HashMap<String, CachedValidationResult>,
    /// Cache TTL in seconds
    ttl_seconds: u64,
    /// Maximum cache size
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
struct CachedValidationResult {
    result: IntegrityValidationReport,
    timestamp: Instant,
}

impl ValidationCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl_seconds,
            max_cache_size: 1000,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<IntegrityValidationReport> {
        if let Some(cached) = self.cache.get(key) {
            if cached.timestamp.elapsed().as_secs() < self.ttl_seconds {
                return Some(cached.result.clone());
            } else {
                // Remove expired entry
                self.cache.remove(key);
            }
        }
        None
    }

    pub fn insert(&mut self, key: String, result: IntegrityValidationReport) {
        // Clean up expired entries if cache is full
        if self.cache.len() >= self.max_cache_size {
            self.cleanup_expired();
        }

        self.cache.insert(
            key,
            CachedValidationResult {
                result,
                timestamp: Instant::now(),
            },
        );
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache
            .retain(|_, cached| now.duration_since(cached.timestamp).as_secs() < self.ttl_seconds);
    }
}

/// Performance-optimized integrity validator
pub struct OptimizedIntegrityValidator {
    /// Base validator
    base_validator: IntegrityValidator,
    /// Performance configuration
    config: PerformanceConfig,
    /// Validation cache
    cache: Arc<Mutex<ValidationCache>>,
    /// Performance metrics
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

/// Performance metrics tracking
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total validations performed
    pub total_validations: usize,
    /// Cache hit count
    pub cache_hits: usize,
    /// Cache miss count
    pub cache_misses: usize,
    /// Average validation time
    pub average_validation_time: Duration,
    /// Total validation time
    pub total_validation_time: Duration,
    /// Parallel validation count
    pub parallel_validations: usize,
    /// Incremental validation count
    pub incremental_validations: usize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_validations: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_validation_time: Duration::from_secs(0),
            total_validation_time: Duration::from_secs(0),
            parallel_validations: 0,
            incremental_validations: 0,
        }
    }

    pub fn record_validation(
        &mut self,
        duration: Duration,
        used_cache: bool,
        used_parallel: bool,
        used_incremental: bool,
    ) {
        self.total_validations += 1;
        self.total_validation_time += duration;
        self.average_validation_time = self.total_validation_time / self.total_validations as u32;

        if used_cache {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        if used_parallel {
            self.parallel_validations += 1;
        }

        if used_incremental {
            self.incremental_validations += 1;
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_validations > 0 {
            self.cache_hits as f64 / self.total_validations as f64
        } else {
            0.0
        }
    }

    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_validations: self.total_validations,
            average_validation_time: self.average_validation_time,
            cache_hit_rate: self.cache_hit_rate(),
            parallel_usage_rate: if self.total_validations > 0 {
                self.parallel_validations as f64 / self.total_validations as f64
            } else {
                0.0
            },
            incremental_usage_rate: if self.total_validations > 0 {
                self.incremental_validations as f64 / self.total_validations as f64
            } else {
                0.0
            },
        }
    }
}

/// Performance summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_validations: usize,
    pub average_validation_time: Duration,
    pub cache_hit_rate: f64,
    pub parallel_usage_rate: f64,
    pub incremental_usage_rate: f64,
}

impl OptimizedIntegrityValidator {
    /// Create a new optimized validator with default configuration
    pub fn new() -> Self {
        let config = PerformanceConfig::default();
        Self {
            base_validator: IntegrityValidator::with_config(
                false,
                config.max_validation_time,
                true,
            ),
            cache: Arc::new(Mutex::new(ValidationCache::new(config.cache_ttl_seconds))),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            config,
        }
    }

    /// Create a new optimized validator with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            base_validator: IntegrityValidator::with_config(
                config.enable_profiling,
                config.max_validation_time,
                true,
            ),
            cache: Arc::new(Mutex::new(ValidationCache::new(config.cache_ttl_seconds))),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            config,
        }
    }

    /// Perform optimized integrity validation based on configuration
    #[instrument(skip(self, blockchain), fields(validation_level = ?self.config.validation_level))]
    pub async fn validate_with_optimization(
        &self,
        blockchain: &Blockchain,
    ) -> Result<IntegrityValidationReport> {
        let start_time = Instant::now();
        let cache_key = self.generate_cache_key(blockchain);

        // Check cache first if enabled
        if self.config.enable_caching {
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(cached_result) = cache.get(&cache_key) {
                    if let Ok(mut metrics) = self.metrics.lock() {
                        metrics.record_validation(start_time.elapsed(), true, false, false);
                    }
                    debug!("Returning cached validation result");
                    return Ok(cached_result);
                }
            }
        }

        // Perform validation based on level
        let report = match self.config.validation_level {
            ValidationLevel::Minimal => self.validate_minimal(blockchain).await?,
            ValidationLevel::Standard => self.validate_standard(blockchain).await?,
            ValidationLevel::Comprehensive => self.validate_comprehensive(blockchain).await?,
            ValidationLevel::Full => self.validate_full(blockchain).await?,
        };

        let validation_time = start_time.elapsed();

        // Cache result if enabled
        if self.config.enable_caching {
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(cache_key, report.clone());
            }
        }

        // Record metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_validation(
                validation_time,
                false, // didn't use cache
                self.config.enable_parallel_validation,
                self.config.enable_incremental_validation,
            );
        }

        info!(
            "Optimized validation completed in {:?} with level {:?}",
            validation_time, self.config.validation_level
        );

        Ok(report)
    }

    /// Minimal validation for high-performance scenarios
    #[instrument(skip(self, blockchain))]
    async fn validate_minimal(&self, blockchain: &Blockchain) -> Result<IntegrityValidationReport> {
        debug!("Performing minimal validation");

        let mut report = IntegrityValidationReport::new();

        // Only check basic blockchain integrity
        report.blockchain_integrity = self.validate_blockchain_basic(blockchain)?;

        // Skip transaction counting, SPARQL, and canonicalization for performance
        report.transaction_count_integrity = TransactionCountIntegrityStatus::new();
        report.sparql_query_integrity = SparqlIntegrityStatus::new();
        report.rdf_canonicalization_integrity = CanonicalizationIntegrityStatus::new();

        report.calculate_overall_status();
        Ok(report)
    }

    /// Standard validation for normal operations
    #[instrument(skip(self, blockchain))]
    async fn validate_standard(
        &self,
        blockchain: &Blockchain,
    ) -> Result<IntegrityValidationReport> {
        debug!("Performing standard validation");

        let mut report = IntegrityValidationReport::new();

        // Blockchain and transaction counting
        report.blockchain_integrity = self.validate_blockchain_basic(blockchain)?;
        report.transaction_count_integrity = self
            .base_validator
            .validate_transaction_counts(blockchain)?;

        // Skip SPARQL and canonicalization for standard performance
        report.sparql_query_integrity = SparqlIntegrityStatus::new();
        report.rdf_canonicalization_integrity = CanonicalizationIntegrityStatus::new();

        report.calculate_overall_status();
        Ok(report)
    }

    /// Comprehensive validation for critical systems
    #[instrument(skip(self, blockchain))]
    async fn validate_comprehensive(
        &self,
        blockchain: &Blockchain,
    ) -> Result<IntegrityValidationReport> {
        debug!("Performing comprehensive validation");

        let mut report = IntegrityValidationReport::new();

        // All validations except full canonicalization
        report.blockchain_integrity = self
            .base_validator
            .validate_blockchain_integrity(blockchain)?;
        report.transaction_count_integrity = self
            .base_validator
            .validate_transaction_counts(blockchain)?;
        report.sparql_query_integrity = self
            .base_validator
            .validate_sparql_consistency(&blockchain.rdf_store)?;

        // Limited canonicalization validation for performance
        report.rdf_canonicalization_integrity = self
            .validate_canonicalization_sample(&blockchain.rdf_store)
            .await?;

        report.calculate_overall_status();
        Ok(report)
    }

    /// Full validation with all checks enabled
    #[instrument(skip(self, blockchain))]
    async fn validate_full(&self, blockchain: &Blockchain) -> Result<IntegrityValidationReport> {
        debug!("Performing full validation");

        // Use base validator for complete validation
        self.base_validator.validate_system_integrity(blockchain)
    }

    /// Basic blockchain validation for minimal/standard levels
    fn validate_blockchain_basic(
        &self,
        blockchain: &Blockchain,
    ) -> Result<BlockchainIntegrityStatus> {
        let mut status = BlockchainIntegrityStatus::new();
        status.chain_length = blockchain.chain.len();

        // Only check basic chain structure and hash integrity
        status.persistent_block_count = blockchain.chain.len(); // Assume persistence is correct for performance

        // Quick hash validation for recent blocks only (last 10 blocks)
        let recent_blocks = if blockchain.chain.len() > 10 {
            &blockchain.chain[blockchain.chain.len() - 10..]
        } else {
            &blockchain.chain
        };

        for (i, block) in recent_blocks.iter().enumerate() {
            let actual_index = if blockchain.chain.len() > 10 {
                blockchain.chain.len() - 10 + i
            } else {
                i
            };

            // Quick hash validation
            if actual_index > 0 {
                let expected_prev_hash = &blockchain.chain[actual_index - 1].hash;
                if &block.previous_hash != expected_prev_hash {
                    status.hash_validation_errors.push(format!(
                        "Block {} hash mismatch: expected {}, got {}",
                        actual_index, expected_prev_hash, block.previous_hash
                    ));
                }
            }
        }

        Ok(status)
    }

    /// Sample-based canonicalization validation for performance
    async fn validate_canonicalization_sample(
        &self,
        rdf_store: &RDFStore,
    ) -> Result<CanonicalizationIntegrityStatus> {
        let mut status = CanonicalizationIntegrityStatus::new();

        // Use the canonicalization validator directly for sample validation
        let canonicalization_validator =
            crate::integrity::canonicalization_validator::CanonicalizationValidator::with_config(
                self.config.enable_profiling,
                true,
                5, // Sample only 5 graphs for performance
            );

        let all_graphs = canonicalization_validator.get_all_graph_names(rdf_store)?;
        let sample_size = std::cmp::min(5, all_graphs.len()); // Sample up to 5 graphs

        for graph_name in all_graphs.iter().take(sample_size) {
            match canonicalization_validator
                .validate_single_graph_consistency(rdf_store, graph_name)
            {
                Ok(result) => status.algorithm_consistency_checks.push(result),
                Err(e) => {
                    status.hash_validation_failures.push(format!(
                        "Sample canonicalization validation failed for graph {}: {}",
                        graph_name, e
                    ));
                }
            }
        }

        Ok(status)
    }

    /// Generate cache key for validation results
    fn generate_cache_key(&self, blockchain: &Blockchain) -> String {
        let latest_hash = if let Some(latest_block) = blockchain.chain.last() {
            latest_block.hash.clone()
        } else {
            "genesis".to_string()
        };

        format!(
            "integrity_{}_{}_{}_{}",
            blockchain.chain.len(),
            latest_hash,
            self.config.validation_level as u8,
            chrono::Utc::now().timestamp() / self.config.cache_ttl_seconds as i64
        )
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceSummary {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.get_performance_summary()
        } else {
            PerformanceSummary {
                total_validations: 0,
                average_validation_time: Duration::from_secs(0),
                cache_hit_rate: 0.0,
                parallel_usage_rate: 0.0,
                incremental_usage_rate: 0.0,
            }
        }
    }

    /// Update performance configuration
    pub fn update_config(&mut self, config: PerformanceConfig) {
        self.config = config.clone();
        self.base_validator = IntegrityValidator::with_config(
            config.enable_profiling,
            config.max_validation_time,
            true,
        );

        // Update cache TTL
        if let Ok(mut cache) = self.cache.lock() {
            *cache = ValidationCache::new(config.cache_ttl_seconds);
        }

        info!(
            "Performance configuration updated: level={:?}, parallel={}, caching={}",
            config.validation_level, config.enable_parallel_validation, config.enable_caching
        );
    }

    /// Clear validation cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.cache.clear();
            debug!("Validation cache cleared");
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStatistics {
        if let Ok(cache) = self.cache.lock() {
            CacheStatistics {
                total_entries: cache.cache.len(),
                max_entries: cache.max_cache_size,
                ttl_seconds: cache.ttl_seconds,
                hit_rate: self.get_performance_metrics().cache_hit_rate,
            }
        } else {
            CacheStatistics::default()
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub hit_rate: f64,
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            max_entries: 1000,
            ttl_seconds: 300,
            hit_rate: 0.0,
        }
    }
}

/// Background integrity monitoring service for production deployment
pub struct BackgroundIntegrityService {
    /// Optimized validator
    validator: OptimizedIntegrityValidator,
    /// Monitoring interval
    monitoring_interval: Duration,
    /// Enable background monitoring
    enabled: bool,
    /// Last validation result
    last_result: Arc<Mutex<Option<IntegrityValidationReport>>>,
}

impl BackgroundIntegrityService {
    /// Create a new background service
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            validator: OptimizedIntegrityValidator::with_config(config),
            monitoring_interval: Duration::from_secs(300), // 5 minutes default
            enabled: true,
            last_result: Arc::new(Mutex::new(None)),
        }
    }

    /// Start background monitoring
    #[instrument(skip(self, blockchain))]
    #[allow(clippy::await_holding_lock)] // Keeping std::sync::Mutex for compatibility, aware of blocking risk
    pub async fn start_background_monitoring(
        &self,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> Result<()> {
        if !self.enabled {
            debug!("Background monitoring is disabled");
            return Ok(());
        }

        info!(
            "Starting background integrity monitoring with {:?} intervals",
            self.monitoring_interval
        );

        let mut interval_timer = interval(self.monitoring_interval);

        loop {
            interval_timer.tick().await;

            if let Ok(blockchain_guard) = blockchain.lock() {
                match self
                    .validator
                    .validate_with_optimization(&blockchain_guard)
                    .await
                {
                    Ok(report) => {
                        // Store last result
                        if let Ok(mut last_result) = self.last_result.lock() {
                            *last_result = Some(report.clone());
                        }

                        // Log significant issues
                        match report.overall_status {
                            IntegrityStatus::Critical | IntegrityStatus::Corrupted => {
                                warn!("Background monitoring detected critical integrity issues");
                            }
                            IntegrityStatus::Warning => {
                                info!("Background monitoring detected integrity warnings");
                            }
                            IntegrityStatus::Healthy => {
                                debug!("Background monitoring: system healthy");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Background integrity validation failed: {}", e);
                    }
                }
            }
        }
    }

    /// Get last validation result
    pub fn get_last_result(&self) -> Option<IntegrityValidationReport> {
        if let Ok(last_result) = self.last_result.lock() {
            last_result.clone()
        } else {
            None
        }
    }

    /// Enable or disable background monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!(
            "Background monitoring {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Update monitoring interval
    pub fn set_monitoring_interval(&mut self, interval: Duration) {
        self.monitoring_interval = interval;
        info!("Background monitoring interval updated to {:?}", interval);
    }

    /// Get performance metrics from the validator
    pub fn get_performance_metrics(&self) -> PerformanceSummary {
        self.validator.get_performance_metrics()
    }
}

/// Production deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Performance optimization settings
    pub performance: PerformanceConfig,
    /// Background monitoring settings
    pub background_monitoring: BackgroundMonitoringConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Alerting configuration
    pub alerting: AlertingConfig,
}

/// Background monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundMonitoringConfig {
    /// Enable background monitoring
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval_seconds: u64,
    /// Validation level for background checks
    pub validation_level: ValidationLevel,
    /// Enable automatic repair for background issues
    pub enable_auto_repair: bool,
}

/// Resource limits for production deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum validation time in seconds
    pub max_validation_time_seconds: u64,
    /// Maximum concurrent validations
    pub max_concurrent_validations: usize,
}

/// Alerting configuration for production
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable email alerts
    pub enable_email: bool,
    /// Enable webhook alerts
    pub enable_webhook: bool,
    /// Enable Slack alerts
    pub enable_slack: bool,
    /// Alert threshold for critical issues
    pub critical_threshold: usize,
    /// Alert threshold for warnings
    pub warning_threshold: usize,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            performance: PerformanceConfig::default(),
            background_monitoring: BackgroundMonitoringConfig {
                enabled: true,
                interval_seconds: 300,
                validation_level: ValidationLevel::Standard,
                enable_auto_repair: false,
            },
            resource_limits: ResourceLimits {
                max_memory_mb: 1024, // 1GB
                max_cpu_percent: 80.0,
                max_validation_time_seconds: 300,
                max_concurrent_validations: 5,
            },
            alerting: AlertingConfig {
                enable_email: false,
                enable_webhook: false,
                enable_slack: false,
                critical_threshold: 5,
                warning_threshold: 3,
            },
        }
    }
}

impl Default for OptimizedIntegrityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.validation_level, ValidationLevel::Standard);
        assert!(config.enable_parallel_validation);
        assert!(config.enable_caching);
        assert_eq!(config.max_validation_time, 300);
    }

    #[test]
    fn test_validation_cache() {
        let mut cache = ValidationCache::new(60);
        let report = IntegrityValidationReport::new();

        cache.insert("test_key".to_string(), report.clone());
        let cached = cache.get("test_key");
        assert!(cached.is_some());

        let retrieved = cached.unwrap();
        assert_eq!(retrieved.overall_status, report.overall_status);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_validation(Duration::from_millis(100), true, false, false);
        metrics.record_validation(Duration::from_millis(200), false, true, false);

        assert_eq!(metrics.total_validations, 2);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_optimized_validator_minimal() {
        let config = PerformanceConfig {
            validation_level: ValidationLevel::Minimal,
            ..Default::default()
        };
        let validator = OptimizedIntegrityValidator::with_config(config);
        let blockchain = Blockchain::new();

        let result = validator.validate_with_optimization(&blockchain).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        // Minimal validation should have empty transaction/sparql/canonicalization status
        assert!(report
            .transaction_count_integrity
            .counting_discrepancies
            .is_empty());
        assert!(report
            .sparql_query_integrity
            .query_consistency_checks
            .is_empty());
        assert!(report
            .rdf_canonicalization_integrity
            .algorithm_consistency_checks
            .is_empty());
    }

    #[tokio::test]
    async fn test_background_service() {
        let config = PerformanceConfig::default();
        let service = BackgroundIntegrityService::new(config);

        assert!(service.enabled);
        assert_eq!(service.monitoring_interval, Duration::from_secs(300));
        assert!(service.get_last_result().is_none());
    }

    #[test]
    fn test_production_config_default() {
        let config = ProductionConfig::default();
        assert!(config.background_monitoring.enabled);
        assert_eq!(config.resource_limits.max_memory_mb, 1024);
        assert_eq!(config.alerting.critical_threshold, 5);
    }
}
