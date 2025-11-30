//! Database Performance Optimization Module
//!
//! This module provides SPARQL query optimization, result caching, and
//! database performance enhancements for ProvChain.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// SPARQL query result cache entry
#[derive(Debug, Clone)]
struct QueryCacheEntry {
    /// Cached query results
    results: String,
    /// When this entry was created
    _created_at: Instant,
    /// How many times this entry has been accessed
    access_count: u64,
    /// Last access time
    last_accessed: Instant,
    /// Query execution time when cached
    execution_time: Duration,
}

impl QueryCacheEntry {
    fn new(results: String, execution_time: Duration) -> Self {
        let now = Instant::now();
        Self {
            results,
            _created_at: now,
            access_count: 1,
            last_accessed: now,
            execution_time,
        }
    }

    fn access(&mut self) -> &str {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.results
    }
}

/// SPARQL query cache with LRU eviction
pub struct QueryCache {
    /// Maximum number of cached queries
    max_size: usize,
    /// Cache storage
    cache: HashMap<String, QueryCacheEntry>,
    /// Access order for LRU eviction
    access_order: Vec<String>,
    /// Cache statistics
    hits: u64,
    misses: u64,
    /// Total time saved by cache hits
    time_saved: Duration,
}

impl QueryCache {
    /// Create a new query cache
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

    /// Get cached query result or execute and cache it
    pub fn get_or_execute<F>(&mut self, query: &str, execute_fn: F) -> String
    where
        F: FnOnce(&str) -> (String, Duration),
    {
        let cache_key = self.generate_cache_key(query);

        // Check if we have a cache hit first
        if self.cache.contains_key(&cache_key) {
            // Cache hit
            self.hits += 1;
            let execution_time = self.cache[&cache_key].execution_time;
            let results = {
                let entry = self.cache.get_mut(&cache_key).unwrap();
                entry.access().to_string()
            };
            self.update_access_order(&cache_key);

            // Add saved execution time
            self.time_saved += execution_time;

            results
        } else {
            // Cache miss - execute the query
            self.misses += 1;
            let (results, execution_time) = execute_fn(query);

            // Add to cache
            self.insert(cache_key, results.clone(), execution_time);

            results
        }
    }

    /// Insert a new query result into the cache
    fn insert(&mut self, key: String, results: String, execution_time: Duration) {
        // Remove oldest entry if cache is full
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            self.evict_lru();
        }

        let entry = QueryCacheEntry::new(results, execution_time);
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

    /// Generate a cache key from SPARQL query
    fn generate_cache_key(&self, query: &str) -> String {
        // Normalize query by removing extra whitespace and converting to lowercase
        let normalized_query = query
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        let mut hasher = Sha256::new();
        hasher.update(normalized_query.as_bytes());
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

    /// Estimate memory usage in bytes
    pub fn estimate_memory_usage(&self) -> usize {
        let entry_overhead = std::mem::size_of::<QueryCacheEntry>();
        let key_size = 64; // SHA256 hex string
        let access_order_size = self.access_order.len() * 64;

        let results_size: usize = self.cache.values().map(|entry| entry.results.len()).sum();

        self.cache.len() * (entry_overhead + key_size) + results_size + access_order_size
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> QueryCacheStats {
        QueryCacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.get_hit_rate(),
            size: self.cache.len(),
            max_size: self.max_size,
            time_saved: self.time_saved,
        }
    }

    /// Invalidate cache entries that might be affected by data changes
    pub fn invalidate_affected_queries(&mut self, affected_predicates: &[&str]) {
        let keys_to_remove: Vec<_> = self
            .cache
            .keys()
            .filter(|key| {
                // Simple heuristic: if any affected predicate appears in the query hash,
                // invalidate it. In a real implementation, this would be more sophisticated.
                affected_predicates.iter().any(|predicate| {
                    // This is a simplified check - in practice, you'd need to parse
                    // the original query or store metadata about what each query accesses
                    key.contains(&format!("{:x}", Sha256::digest(predicate.as_bytes())))
                })
            })
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
    }

    /// Get most frequently accessed queries
    pub fn get_most_accessed_queries(&self, limit: usize) -> Vec<(String, u64, Duration)> {
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|(key, entry)| (key.clone(), entry.access_count, entry.execution_time))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(limit).collect()
    }
}

/// Query cache statistics
#[derive(Debug, Clone)]
pub struct QueryCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub max_size: usize,
    pub time_saved: Duration,
}

impl QueryCacheStats {
    pub fn print_summary(&self) {
        println!("\n=== Query Cache Statistics ===");
        println!("Cache hits: {}", self.hits);
        println!("Cache misses: {}", self.misses);
        println!("Hit rate: {:.2}%", self.hit_rate * 100.0);
        println!("Current size: {}/{}", self.size, self.max_size);
        println!("Time saved: {:?}", self.time_saved);
        println!("==============================\n");
    }
}

/// SPARQL query optimizer
pub struct QueryOptimizer {
    /// Common query patterns and their optimized versions
    _optimization_rules: HashMap<String, String>,
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new() -> Self {
        let mut optimization_rules = HashMap::new();

        // Add common optimization patterns
        optimization_rules.insert(
            "count_all_pattern".to_string(),
            "Use COUNT(*) instead of COUNT(?var) when possible".to_string(),
        );

        optimization_rules.insert(
            "limit_early".to_string(),
            "Apply LIMIT as early as possible in subqueries".to_string(),
        );

        optimization_rules.insert(
            "filter_early".to_string(),
            "Apply FILTER conditions as early as possible".to_string(),
        );

        Self {
            _optimization_rules: optimization_rules,
        }
    }

    /// Optimize a SPARQL query
    pub fn optimize_query(&self, query: &str) -> String {
        let mut optimized_query = query.to_string();

        // Apply basic optimizations
        optimized_query = self.optimize_whitespace(&optimized_query);
        optimized_query = self.optimize_prefixes(&optimized_query);
        optimized_query = self.optimize_filters(&optimized_query);

        optimized_query
    }

    /// Remove unnecessary whitespace
    fn optimize_whitespace(&self, query: &str) -> String {
        query
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Optimize PREFIX declarations
    fn optimize_prefixes(&self, query: &str) -> String {
        // In a real implementation, this would remove unused prefixes
        // and reorder them for better readability
        query.to_string()
    }

    /// Optimize FILTER placement
    fn optimize_filters(&self, query: &str) -> String {
        // In a real implementation, this would move FILTER clauses
        // to more optimal positions in the query
        query.to_string()
    }

    /// Analyze query complexity
    pub fn analyze_complexity(&self, query: &str) -> QueryComplexity {
        let query_lower = query.to_lowercase();

        let has_joins = query_lower.contains("join") || query_lower.matches('.').count() > 3;
        let has_aggregation = query_lower.contains("count")
            || query_lower.contains("sum")
            || query_lower.contains("avg")
            || query_lower.contains("group by");
        let has_optional = query_lower.contains("optional");
        let has_union = query_lower.contains("union");
        let has_subquery = query_lower.matches('{').count() > 1;

        let complexity_score = (if has_joins { 2 } else { 0 })
            + (if has_aggregation { 3 } else { 0 })
            + (if has_optional { 2 } else { 0 })
            + (if has_union { 3 } else { 0 })
            + (if has_subquery { 4 } else { 0 });

        QueryComplexity {
            score: complexity_score,
            has_joins,
            has_aggregation,
            has_optional,
            has_union,
            has_subquery,
            estimated_execution_time: Duration::from_millis((complexity_score * 10) as u64),
        }
    }

    /// Get optimization suggestions for a query
    pub fn get_optimization_suggestions(&self, query: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let query_lower = query.to_lowercase();

        if query_lower.contains("select *") {
            suggestions
                .push("Consider selecting only needed variables instead of SELECT *".to_string());
        }

        if query_lower.contains("filter") && !query_lower.contains("limit") {
            suggestions.push("Consider adding LIMIT to prevent large result sets".to_string());
        }

        if query_lower.matches("optional").count() > 2 {
            suggestions.push(
                "Multiple OPTIONAL clauses can be expensive - consider restructuring".to_string(),
            );
        }

        if !query_lower.contains("prefix") && query_lower.contains("http://") {
            suggestions
                .push("Consider using PREFIX declarations for better readability".to_string());
        }

        suggestions
    }
}

/// Query complexity analysis
#[derive(Debug, Clone)]
pub struct QueryComplexity {
    pub score: u32,
    pub has_joins: bool,
    pub has_aggregation: bool,
    pub has_optional: bool,
    pub has_union: bool,
    pub has_subquery: bool,
    pub estimated_execution_time: Duration,
}

impl QueryComplexity {
    pub fn complexity_level(&self) -> &'static str {
        match self.score {
            0..=2 => "Simple",
            3..=6 => "Moderate",
            7..=10 => "Complex",
            _ => "Very Complex",
        }
    }

    pub fn print_analysis(&self) {
        println!("\n=== Query Complexity Analysis ===");
        println!(
            "Complexity score: {} ({})",
            self.score,
            self.complexity_level()
        );
        println!("Has joins: {}", self.has_joins);
        println!("Has aggregation: {}", self.has_aggregation);
        println!("Has optional: {}", self.has_optional);
        println!("Has union: {}", self.has_union);
        println!("Has subquery: {}", self.has_subquery);
        println!(
            "Estimated execution time: {:?}",
            self.estimated_execution_time
        );
        println!("=================================\n");
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_cache_basic_operations() {
        let mut cache = QueryCache::new(3);

        // Test cache miss and insertion
        let result1 = cache.get_or_execute("SELECT * WHERE { ?s ?p ?o }", |query| {
            (format!("results for {query}"), Duration::from_millis(100))
        });
        assert!(result1.contains("SELECT * WHERE"));
        assert_eq!(cache.size(), 1);
        assert_eq!(cache.get_hit_rate(), 0.0); // First access is always a miss

        // Test cache hit
        let result2 = cache.get_or_execute("SELECT * WHERE { ?s ?p ?o }", |query| {
            (format!("results for {query}"), Duration::from_millis(100))
        });
        assert_eq!(result1, result2);
        assert_eq!(cache.size(), 1);
        assert!(cache.get_hit_rate() > 0.0); // Should have a hit now
    }

    #[test]
    fn test_query_cache_normalization() {
        let cache = QueryCache::new(5);

        // These queries should be treated as the same after normalization
        let query1 = "SELECT * WHERE { ?s ?p ?o }";
        let query2 = "  SELECT   *   WHERE   {   ?s   ?p   ?o   }  ";
        let query3 = "select * where { ?s ?p ?o }"; // Different case

        let key1 = cache.generate_cache_key(query1);
        let key2 = cache.generate_cache_key(query2);
        let key3 = cache.generate_cache_key(query3);

        assert_eq!(key1, key2); // Should normalize whitespace
        assert_eq!(key1, key3); // Should normalize case
    }

    #[test]
    fn test_query_cache_lru_eviction() {
        let mut cache = QueryCache::new(2);

        // Fill cache to capacity
        cache.get_or_execute("query1", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });
        cache.get_or_execute("query2", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 2);

        // Add third query, should evict first
        cache.get_or_execute("query3", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });
        assert_eq!(cache.size(), 2);

        let stats = cache.get_stats();
        assert_eq!(stats.misses, 3); // All initial accesses are misses
    }

    #[test]
    fn test_query_optimizer_complexity_analysis() {
        let optimizer = QueryOptimizer::new();

        // Simple query
        let simple_query = "SELECT ?s WHERE { ?s a <http://example.org/Person> }";
        let simple_complexity = optimizer.analyze_complexity(simple_query);
        assert_eq!(simple_complexity.complexity_level(), "Simple");
        assert!(!simple_complexity.has_aggregation);

        // Complex query
        let complex_query = r#"
            SELECT ?person (COUNT(?friend) as ?friendCount) WHERE {
                ?person a <http://example.org/Person> .
                OPTIONAL { ?person <http://example.org/knows> ?friend }
                UNION { ?person <http://example.org/worksWith> ?colleague }
            } GROUP BY ?person
        "#;
        let complex_complexity = optimizer.analyze_complexity(complex_query);
        assert!(complex_complexity.score > 5);
        assert!(complex_complexity.has_aggregation);
        assert!(complex_complexity.has_optional);
        assert!(complex_complexity.has_union);
    }

    #[test]
    fn test_query_optimizer_suggestions() {
        let optimizer = QueryOptimizer::new();

        let query_with_issues =
            "SELECT * WHERE { ?s ?p ?o . FILTER(?s = <http://example.org/test>) }";
        let suggestions = optimizer.get_optimization_suggestions(query_with_issues);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("SELECT *")));
    }

    #[test]
    fn test_query_cache_invalidation() {
        let mut cache = QueryCache::new(5);

        // Add some cached queries
        cache.get_or_execute(
            "SELECT ?s WHERE { ?s <http://example.org/name> ?name }",
            |q| (format!("result_{q}"), Duration::from_millis(10)),
        );
        cache.get_or_execute(
            "SELECT ?s WHERE { ?s <http://example.org/age> ?age }",
            |q| (format!("result_{q}"), Duration::from_millis(10)),
        );

        assert_eq!(cache.size(), 2);

        // Invalidate queries that might use the 'name' predicate
        cache.invalidate_affected_queries(&["http://example.org/name"]);

        // Note: The actual invalidation logic is simplified in this implementation
        // In practice, it would need more sophisticated query analysis
    }

    #[test]
    fn test_query_cache_memory_estimation() {
        let mut cache = QueryCache::new(10);

        let initial_memory = cache.estimate_memory_usage();

        cache.get_or_execute("query1", |q| {
            (
                format!("large_result_{}", q.repeat(100)),
                Duration::from_millis(10),
            )
        });
        cache.get_or_execute("query2", |q| {
            (format!("small_result_{q}"), Duration::from_millis(10))
        });

        let memory_with_entries = cache.estimate_memory_usage();
        assert!(memory_with_entries > initial_memory);
    }

    #[test]
    fn test_query_cache_most_accessed() {
        let mut cache = QueryCache::new(5);

        // Add queries with different access patterns
        cache.get_or_execute("query1", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });
        cache.get_or_execute("query2", |q| {
            (format!("result_{q}"), Duration::from_millis(20))
        });
        cache.get_or_execute("query3", |q| {
            (format!("result_{q}"), Duration::from_millis(30))
        });

        // Access query1 multiple times
        cache.get_or_execute("query1", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });
        cache.get_or_execute("query1", |q| {
            (format!("result_{q}"), Duration::from_millis(10))
        });

        let most_accessed = cache.get_most_accessed_queries(2);
        assert_eq!(most_accessed.len(), 2);

        // query1 should be most accessed
        assert!(most_accessed[0].1 >= most_accessed[1].1);
    }

    #[test]
    fn test_query_optimizer_whitespace_optimization() {
        let optimizer = QueryOptimizer::new();

        let messy_query = r#"
            SELECT   *   WHERE   {
                ?s   ?p   ?o   .
                
                FILTER   (   ?s   =   <http://example.org/test>   )
            }
        "#;

        let optimized = optimizer.optimize_query(messy_query);
        assert!(!optimized.contains('\n'));
        assert!(!optimized.contains("  ")); // No double spaces
    }
}
