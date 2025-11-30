//! Cache Management for OWL2 Profile Validation
//!
//! This module implements advanced caching strategies for profile validation results,
//! including multi-level caching, compression, and intelligent eviction policies.

#![allow(dead_code)]

use crate::error::OwlResult;
use crate::profiles::common::Owl2Profile;
use crate::profiles::common::ProfileValidationResult;
use dashmap::DashMap;
use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Cache priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePriority {
    /// Low priority (easily evicted)
    Low = 1,
    /// Medium priority
    Medium = 2,
    /// High priority (rarely evicted)
    High = 3,
    /// Critical priority (never evicted)
    Critical = 4,
}

/// Profile cache configuration parameters
#[derive(Debug, Clone)]
pub struct ProfileCacheConfig {
    /// Maximum entries in primary cache
    primary_cache_size: usize,
    /// Maximum entries in hot cache
    _hot_cache_size: usize,
    /// Maximum entries in compressed cache
    compressed_cache_size: usize,
    /// Time-to-live for cache entries
    ttl_duration: Duration,
    /// Compression threshold (entries larger than this get compressed)
    compression_threshold: usize,
    /// Hot cache promotion threshold (access count)
    hot_cache_threshold: usize,
}

/// Cache statistics and performance metrics
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Number of cache evictions
    pub evictions: usize,
    /// Number of compressed cache hits
    compressed_hits: usize,
    /// Number of hot cache hits
    hot_hits: usize,
    /// Total memory used by cache
    total_memory_bytes: usize,
    /// Memory saved by compression
    compressed_memory_saved: usize,
    /// Average access time
    average_access_time_ns: u64,
    /// Cache hit rate
    pub hit_rate: f64,
}

impl Default for ProfileCacheConfig {
    fn default() -> Self {
        Self {
            primary_cache_size: 1000,
            _hot_cache_size: 100,
            compressed_cache_size: 500,
            ttl_duration: Duration::from_secs(3600), // 1 hour
            compression_threshold: 1024,             // 1KB
            hot_cache_threshold: 5,
        }
    }
}

/// Advanced cache management system
pub struct AdvancedCacheManager {
    primary_cache: LruCache<Owl2Profile, ProfileValidationResult>,
    hot_cache: DashMap<Owl2Profile, ProfileValidationResult>,
    compressed_cache: HashMap<Owl2Profile, Vec<u8>>,
    invalidation_tokens: HashSet<String>,
    cache_stats: CacheStatistics,
    config: ProfileCacheConfig,
}

impl AdvancedCacheManager {
    /// Create a new advanced cache manager
    pub fn new() -> OwlResult<Self> {
        let config = ProfileCacheConfig::default();
        Self::with_config(config)
    }

    /// Create a new advanced cache manager with custom configuration
    pub fn with_config(config: ProfileCacheConfig) -> OwlResult<Self> {
        let primary_cache_size = std::num::NonZeroUsize::new(config.primary_cache_size)
            .ok_or_else(|| crate::error::OwlError::ConfigError {
                parameter: "primary_cache_size".to_string(),
                message: "Cache size must be greater than zero".to_string(),
            })?;

        Ok(Self {
            primary_cache: LruCache::new(primary_cache_size),
            hot_cache: DashMap::new(),
            compressed_cache: HashMap::new(),
            invalidation_tokens: HashSet::new(),
            cache_stats: CacheStatistics::default(),
            config,
        })
    }

    /// Get a cached validation result
    pub fn get(&mut self, profile: &Owl2Profile) -> Option<ProfileValidationResult> {
        let start_time = Instant::now();

        // Check hot cache first (fastest)
        if let Some(result) = self.hot_cache.get(profile).map(|r| r.clone()) {
            self.cache_stats.hot_hits += 1;
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check primary cache
        if let Some(result) = self.primary_cache.get(profile).cloned() {
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check compressed cache (slower but memory efficient)
        if let Some(compressed) = self.compressed_cache.get(profile) {
            if let Ok(result) = self.decompress_result(compressed) {
                self.cache_stats.compressed_hits += 1;
                self.cache_stats.hits += 1;
                self.update_access_time(start_time);
                return Some(result);
            }
        }

        self.cache_stats.misses += 1;
        None
    }

    /// Put a validation result in the cache
    pub fn put(&mut self, profile: Owl2Profile, result: ProfileValidationResult) {
        // Determine cache strategy based on result size and access patterns
        let result_size = std::mem::size_of_val(&result);

        if result_size > self.config.compression_threshold {
            // Compress and store in compressed cache
            if let Ok(compressed) = self.compress_result(&result) {
                self.compressed_cache.insert(profile, compressed);
            }
        } else {
            // Store in primary cache
            self.primary_cache.put(profile, result);
        }

        self.update_stats();
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.primary_cache.clear();
        self.hot_cache.clear();
        self.compressed_cache.clear();
        self.invalidation_tokens.clear();
        self.cache_stats = CacheStatistics::default();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &CacheStatistics {
        &self.cache_stats
    }

    /// Get cache configuration
    pub fn get_config(&self) -> &ProfileCacheConfig {
        &self.config
    }

    /// Compress a validation result
    fn compress_result(&self, result: &ProfileValidationResult) -> OwlResult<Vec<u8>> {
        // Use JSON serialization as fallback since bincode doesn't work with IRI
        serde_json::to_vec(result).map_err(|e| {
            crate::error::OwlError::SerializationError(format!(
                "Failed to compress validation result: {}",
                e
            ))
        })
    }

    /// Decompress a validation result
    fn decompress_result(&self, compressed: &[u8]) -> OwlResult<ProfileValidationResult> {
        serde_json::from_slice(compressed).map_err(|e| {
            crate::error::OwlError::SerializationError(format!(
                "Failed to decompress validation result: {}",
                e
            ))
        })
    }

    /// Update cache access time statistics
    fn update_access_time(&mut self, start_time: Instant) {
        let access_time = start_time.elapsed().as_nanos() as u64;
        self.cache_stats.average_access_time_ns =
            (self.cache_stats.average_access_time_ns + access_time) / 2;
    }

    /// Update cache statistics
    fn update_stats(&mut self) {
        self.cache_stats.hit_rate = if self.cache_stats.hits + self.cache_stats.misses > 0 {
            self.cache_stats.hits as f64 / (self.cache_stats.hits + self.cache_stats.misses) as f64
        } else {
            0.0
        };
    }
}

/// Simple cache manager for legacy compatibility
pub struct ProfileCache {
    cache: DashMap<Owl2Profile, ProfileValidationResult>,
    stats: std::sync::RwLock<CacheStatistics>,
}

impl Default for ProfileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileCache {
    /// Create a new profile cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            stats: std::sync::RwLock::new(CacheStatistics::default()),
        }
    }

    /// Get a cached result
    pub fn get(&self, profile: &Owl2Profile) -> Option<ProfileValidationResult> {
        if let Some(result) = self.cache.get(profile) {
            match self.stats.write() {
                Ok(mut stats) => {
                    stats.hits += 1;
                    Some(result.clone())
                }
                Err(_) => {
                    // Lock is poisoned - return result without updating stats
                    Some(result.clone())
                }
            }
        } else {
            match self.stats.write() {
                Ok(mut stats) => {
                    stats.misses += 1;
                    None
                }
                Err(_) => {
                    // Lock is poisoned - return None without updating stats
                    None
                }
            }
        }
    }

    /// Put a result in cache
    pub fn put(&self, profile: Owl2Profile, result: ProfileValidationResult) {
        self.cache.insert(profile, result);
    }

    /// Clear cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStatistics {
        match self.stats.read() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                // Lock is poisoned - extract stats from the poisoned lock
                poisoned.into_inner().clone()
            }
        }
    }
}
