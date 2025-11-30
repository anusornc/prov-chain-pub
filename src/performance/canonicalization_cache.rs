//! RDF Canonicalization Cache Module
//!
//! This module provides caching for RDF canonicalization operations to improve
//! performance by avoiding redundant hash calculations for identical graphs.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache entry for canonicalization results
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The canonical hash
    hash: String,
    /// When this entry was created
    created_at: Instant,
    /// How many times this entry has been accessed
    access_count: u64,
    /// Last access time
    last_accessed: Instant,
}

impl CacheEntry {
    fn new(hash: String) -> Self {
        let now = Instant::now();
        Self {
            hash,
            created_at: now,
            access_count: 1,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> &str {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.hash
    }
}

/// LRU cache for RDF canonicalization results
pub struct CanonicalizationCache {
    /// Maximum number of entries to cache
    max_size: usize,
    /// Cache storage
    cache: HashMap<String, CacheEntry>,
    /// Access order for LRU eviction
    access_order: Vec<String>,
    /// Cache statistics
    hits: u64,
    misses: u64,
    /// Total time saved by cache hits
    time_saved: Duration,
}

impl CanonicalizationCache {
    /// Create a new canonicalization cache
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            cache: HashMap::new(),
            access_order: Vec::new(),
            hits: 0,
            misses: 0,
            time_saved: Duration::new(0, 0),
        }
    }

    /// Get cached canonicalization result or compute and cache it
    pub fn get_or_compute<F>(&mut self, rdf_content: &str, compute_fn: F) -> String
    where
        F: FnOnce(&str) -> (String, Duration),
    {
        let cache_key = self.generate_cache_key(rdf_content);

        if let Some(entry) = self.cache.get_mut(&cache_key) {
            // Cache hit
            self.hits += 1;
            let hash = entry.access().to_string();
            self.update_access_order(&cache_key);

            // Estimate time saved (average canonicalization time)
            self.time_saved += Duration::from_millis(50); // Conservative estimate

            hash
        } else {
            // Cache miss - compute the result
            self.misses += 1;
            let (hash, _computation_time) = compute_fn(rdf_content);

            // Add to cache
            self.insert(cache_key, hash.clone());

            hash
        }
    }

    /// Insert a new entry into the cache
    fn insert(&mut self, key: String, hash: String) {
        // Remove oldest entry if cache is full
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        let entry = CacheEntry::new(hash);
        self.cache.insert(key.clone(), entry);
        self.access_order.push(key);
    }

    /// Evict the least recently used entry
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self.access_order.first().cloned() {
            self.cache.remove(&oldest_key);
            self.access_order.retain(|k| k != &oldest_key);
        }
    }

    /// Update access order for LRU tracking
    fn update_access_order(&mut self, key: &str) {
        self.access_order.retain(|k| k != key);
        self.access_order.push(key.to_string());
    }

    /// Generate a cache key from RDF content
    fn generate_cache_key(&self, rdf_content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rdf_content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cache hit rate
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.hits = 0;
        self.misses = 0;
        self.time_saved = Duration::new(0, 0);
    }

    /// Resize the cache
    pub fn resize(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;

        // Evict entries if new size is smaller
        while self.cache.len() > self.max_size {
            self.evict_lru();
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.get_hit_rate(),
            size: self.cache.len(),
            max_size: self.max_size,
            time_saved: self.time_saved,
        }
    }

    /// Estimate memory usage in bytes
    pub fn estimate_memory_usage(&self) -> usize {
        let entry_size = std::mem::size_of::<CacheEntry>() + 64; // Hash string + overhead
        let key_size = 64; // SHA256 hex string
        let access_order_size = self.access_order.len() * 64;

        self.cache.len() * (entry_size + key_size) + access_order_size
    }

    /// Get entries sorted by access frequency
    pub fn get_most_accessed_entries(&self, limit: usize) -> Vec<(String, u64)> {
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|(key, entry)| (key.clone(), entry.access_count))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(limit).collect()
    }

    /// Remove expired entries (older than specified duration)
    pub fn remove_expired(&mut self, max_age: Duration) {
        let now = Instant::now();
        let expired_keys: Vec<_> = self
            .cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > max_age)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
    }

    /// Precompute and cache canonicalization for common RDF patterns
    pub fn precompute_common_patterns(&mut self) {
        let common_patterns = vec![
            // Empty graph
            "",
            // Simple triple
            r#"<http://example.org/subject> <http://example.org/predicate> <http://example.org/object> ."#,
            // Basic supply chain pattern
            r#"
            @prefix trace: <http://provchain.org/trace#> .
            <http://example.org/batch1> a trace:ProductBatch .
            "#,
            // Blank node pattern
            r#"
            @prefix trace: <http://provchain.org/trace#> .
            _:batch a trace:ProductBatch .
            "#,
        ];

        for pattern in common_patterns {
            let cache_key = self.generate_cache_key(pattern);
            if !self.cache.contains_key(&cache_key) {
                // Simulate canonicalization (in real usage, this would call the actual function)
                let mut hasher = Sha256::new();
                hasher.update(pattern.as_bytes());
                let hash = format!("{:x}", hasher.finalize());

                self.insert(cache_key, hash);
            }
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub max_size: usize,
    pub time_saved: Duration,
}

impl CacheStats {
    pub fn print_summary(&self) {
        println!("\n=== Canonicalization Cache Statistics ===");
        println!("Cache hits: {}", self.hits);
        println!("Cache misses: {}", self.misses);
        println!("Hit rate: {:.2}%", self.hit_rate * 100.0);
        println!("Current size: {}/{}", self.size, self.max_size);
        println!("Time saved: {:?}", self.time_saved);
        println!("=========================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = CanonicalizationCache::new(3);

        // Test cache miss and insertion
        let result1 = cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(result1, "hash_test1");
        assert_eq!(cache.size(), 1);
        assert_eq!(cache.get_hit_rate(), 0.0); // First access is always a miss

        // Test cache hit
        let result2 = cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(result2, "hash_test1");
        assert_eq!(cache.size(), 1);
        assert!(cache.get_hit_rate() > 0.0); // Should have a hit now
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = CanonicalizationCache::new(2);

        // Fill cache to capacity
        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 2);

        // Add third item, should evict first
        cache.get_or_compute("test3", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 2);

        // test1 should be evicted, test2 and test3 should remain
        let stats = cache.get_stats();
        assert_eq!(stats.misses, 3); // All initial accesses are misses
    }

    #[test]
    fn test_cache_resize() {
        let mut cache = CanonicalizationCache::new(3);

        // Fill cache
        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test3", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 3);

        // Resize to smaller size
        cache.resize(2);
        assert_eq!(cache.size(), 2);
        assert_eq!(cache.max_size, 2);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CanonicalizationCache::new(3);

        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });

        assert_eq!(cache.size(), 2);
        assert!(cache.get_stats().misses > 0);

        cache.clear();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get_stats().misses, 0);
        assert_eq!(cache.get_stats().hits, 0);
    }

    #[test]
    fn test_cache_key_generation() {
        let cache = CanonicalizationCache::new(10);

        let key1 = cache.generate_cache_key("test content");
        let key2 = cache.generate_cache_key("test content");
        let key3 = cache.generate_cache_key("different content");

        assert_eq!(key1, key2); // Same content should generate same key
        assert_ne!(key1, key3); // Different content should generate different key
        assert_eq!(key1.len(), 64); // SHA256 hex string length
    }

    #[test]
    fn test_most_accessed_entries() {
        let mut cache = CanonicalizationCache::new(5);

        // Add entries with different access patterns
        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test3", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });

        // Access test1 multiple times
        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });

        // Access test2 once more
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });

        let most_accessed = cache.get_most_accessed_entries(2);
        assert_eq!(most_accessed.len(), 2);

        // test1 should be most accessed (3 times), test2 second (2 times)
        assert!(most_accessed[0].1 >= most_accessed[1].1);
    }

    #[test]
    fn test_memory_usage_estimation() {
        let mut cache = CanonicalizationCache::new(10);

        let initial_memory = cache.estimate_memory_usage();

        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        cache.get_or_compute("test2", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });

        let memory_with_entries = cache.estimate_memory_usage();
        assert!(memory_with_entries > initial_memory);
    }

    #[test]
    fn test_precompute_common_patterns() {
        let mut cache = CanonicalizationCache::new(10);

        let initial_size = cache.size();
        cache.precompute_common_patterns();

        assert!(cache.size() > initial_size);

        // Should have cached some common patterns
        let stats = cache.get_stats();
        assert!(stats.size > 0);
    }

    #[test]
    fn test_expired_entry_removal() {
        let mut cache = CanonicalizationCache::new(10);

        cache.get_or_compute("test1", |content| {
            (format!("hash_{content}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 1);

        // Remove entries older than 0 seconds (should remove all)
        cache.remove_expired(Duration::from_secs(0));
        assert_eq!(cache.size(), 0);
    }
}
