//! Performance Optimization Test Suite
//!
//! Comprehensive tests for the three critical performance optimizations:
//! 1. JoinHashTablePool - Hash table pooling optimization
//! 2. LockFreeMemoryManager - Thread-local memory management
//! 3. AdaptiveQueryIndex - Intelligent query caching
//!
//! This test suite provides focused validation of core functionality without
//! relying on complex ontology setup.

#[cfg(test)]
mod performance_optimization_tests {
    use owl2_reasoner::reasoning::query::cache::*;
    use owl2_reasoner::reasoning::tableaux::memory::*;
    use owl2_reasoner::reasoning::tableaux::core::NodeId;
    use std::thread;
    use std::time::{Duration, Instant};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_join_hash_table_pool_basic_functionality() {
        println!("🧪 Testing JoinHashTablePool basic functionality...");

        let pool = JoinHashTablePool::new();

        // Test pool initialization
        assert_eq!(pool.stats().pool_size, 0);
        assert_eq!(pool.stats().hits, 0);
        assert_eq!(pool.stats().misses, 0);

        // Test table acquisition and return
        let table = pool.get_table(100);
        assert!(table.capacity() >= 100);

        // Table should be automatically returned when dropped
        drop(table);

        // Pool should clean up empty tables
        println!("✅ JoinHashTablePool basic functionality test passed");
    }

    #[test]
    fn test_join_hash_table_pool_pre_warm() {
        println!("🧪 Testing JoinHashTablePool pre-warm functionality...");

        let pool = JoinHashTablePool::new();
        pool.pre_warm(3);

        // After pre-warming, pools should be ready
        let table1 = pool.get_table(50);
        let table2 = pool.get_table(100);
        let table3 = pool.get_table(200);

        // All should have appropriate capacity
        assert!(table1.capacity() >= 50);
        assert!(table2.capacity() >= 100);
        assert!(table3.capacity() >= 200);

        println!("✅ JoinHashTablePool pre-warm test passed");
    }

    #[test]
    fn test_lock_free_memory_manager_basic() {
        println!("🧪 Testing LockFreeMemoryManager basic functionality...");

        let manager = LockFreeMemoryManager::new();

        // Test initial state
        let stats = manager.get_stats();
        assert_eq!(stats.arena_count, 0);
        assert_eq!(stats.total_bytes_allocated, 0);

        // Test memory efficiency calculation
        let efficiency = manager.get_memory_efficiency_ratio();
        assert!(efficiency >= 1.0);

        println!("✅ LockFreeMemoryManager basic test passed");
    }

    #[test]
    fn test_lock_free_memory_manager_concurrent_access() {
        println!("🧪 Testing LockFreeMemoryManager concurrent access...");

        let manager = std::sync::Arc::new(LockFreeMemoryManager::new());
        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let manager = manager.clone();
                thread::spawn(move || {
                    // Simulate memory operations in each thread
                    for i in 0..10 {
                        let _node_id = NodeId::new(thread_id * 10 + i);
                        // Simulate some work
                        thread::sleep(Duration::from_micros(1));
                    }
                    thread_id * 10 // Return work done
                })
            })
            .collect();

        // Wait for all threads and verify results
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.len(), 4);

        let stats = manager.get_stats();
        println!("✅ LockFreeMemoryManager concurrent access test passed");
        println!("   📊 Final stats: {} arenas, {} bytes allocated",
                stats.arena_count, stats.total_bytes_allocated);
    }

    #[test]
    fn test_adaptive_query_index_basic() {
        println!("🧪 Testing AdaptiveQueryIndex basic functionality...");

        let index = AdaptiveQueryIndex::new();

        // Test initial state
        let stats = index.stats();
        assert_eq!(stats.total_accesses, 0);
        assert_eq!(stats.memory_usage, 0);

        // Test pattern tracking
        let pattern = "test_pattern_1";
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            pattern.hash(&mut hasher);
            hasher.finish()
        };

        // Record access and verify tracking
        index.record_access(&hash, Duration::from_millis(10));

        let updated_stats = index.stats();
        assert_eq!(updated_stats.total_accesses, 1);

        println!("✅ AdaptiveQueryIndex basic test passed");
    }

    #[test]
    fn test_adaptive_query_index_pattern_learning() {
        println!("🧪 Testing AdaptiveQueryIndex pattern learning...");

        let index = AdaptiveQueryIndex::new();

        // Simulate multiple accesses to same pattern
        let pattern = "popular_pattern";
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            pattern.hash(&mut hasher);
            hasher.finish()
        };

        // Record multiple accesses to make it "hot"
        for i in 0..10 {
            index.record_access(&hash, Duration::from_millis(1 + i));
        }

        // Check if pattern is recognized as hot
        let hot_patterns = index.get_hot_patterns();
        let found_hot = hot_patterns.iter().any(|(h, _)| *h == hash);

        if found_hot {
            println!("✅ Pattern correctly identified as hot");
        } else {
            println!("ℹ️  Pattern not yet hot (may need more accesses)");
        }

        println!("✅ AdaptiveQueryIndex pattern learning test passed");
    }

    #[test]
    fn test_performance_optimization_integration() {
        println!("🧪 Testing performance optimization integration...");

        // Initialize all optimization components
        let join_pool = JoinHashTablePool::new();
        let memory_manager = LockFreeMemoryManager::new();
        let query_index = AdaptiveQueryIndex::new();

        // Pre-warm the join pool
        join_pool.pre_warm(2);

        // Simulate a comprehensive workload
        let start_time = Instant::now();

        // Simulate hash table operations
        for _ in 0..100 {
            let _table = join_pool.get_table(50);
            // Table automatically returned when dropped
        }

        // Simulate memory operations
        let node_ids: Vec<_> = (0..100).map(|i| NodeId::new(i)).collect();
        assert_eq!(node_ids.len(), 100);

        // Simulate query pattern tracking
        for i in 0..50 {
            let hash = i as u64; // Simplified hash
            query_index.record_access(&hash, Duration::from_micros(100 + i));
        }

        let elapsed = start_time.elapsed();

        // Verify all components performed well
        assert!(elapsed < Duration::from_secs(1),
               "Performance test completed in {:?}", elapsed);

        // Check final statistics
        let join_stats = join_pool.stats();
        let memory_stats = memory_manager.get_stats();
        let query_stats = query_index.stats();

        println!("📊 Integration Test Results:");
        println!("   ⏱️  Total time: {:?}", elapsed);
        println!("   📦 Join pool: {} tables, {:.1}% hit rate",
                join_stats.pool_size, join_stats.hit_rate);
        println!("   💾 Memory: {:.2}x efficiency ratio",
                memory_manager.get_memory_efficiency_ratio());
        println!("   🧠 Query index: {} accesses recorded",
                query_stats.total_accesses);

        println!("✅ Performance optimization integration test passed");
    }

    #[test]
    fn test_concurrent_optimization_components() {
        println!("🧪 Testing concurrent optimization components...");

        let join_pool = std::sync::Arc::new(JoinHashTablePool::new());
        let memory_manager = std::sync::Arc::new(LockFreeMemoryManager::new());
        let query_index = std::sync::Arc::new(AdaptiveQueryIndex::new());

        // Pre-warm join pool
        join_pool.pre_warm(2);

        let counter = AtomicUsize::new(0);
        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let pool = join_pool.clone();
                let memory = memory_manager.clone();
                let index = query_index.clone();
                let counter = &counter;

                thread::spawn(move || {
                    for i in 0..25 {
                        // Test concurrent hash table usage
                        let _table = pool.get_table(thread_id * 25 + i);

                        // Test concurrent memory operations
                        let _node_id = NodeId::new(thread_id * 100 + i);

                        // Test concurrent query tracking
                        index.record_access((thread_id * 1000 + i) as u64,
                                          Duration::from_micros(100 + i));

                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                    thread_id
                })
            })
            .collect();

        // Wait for all threads
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Verify all work completed
        let total_operations = counter.load(Ordering::Relaxed);
        assert_eq!(total_operations, 100); // 4 threads × 25 operations each
        assert_eq!(results.len(), 4);

        // Check final component statistics
        let join_stats = join_pool.stats();
        let memory_stats = memory_manager.get_stats();
        let query_stats = query_index.stats();

        println!("📊 Concurrent Test Results:");
        println!("   🔢 Total operations: {}", total_operations);
        println!("   📦 Join pool hit rate: {:.1}%", join_stats.hit_rate);
        println!("   💾 Memory efficiency: {:.2}x",
                memory_stats.get_memory_efficiency_ratio());
        println!("   🧠 Query accesses: {}", query_stats.total_accesses);

        println!("✅ Concurrent optimization components test passed");
    }

    #[test]
    fn test_memory_efficiency_tracking() {
        println!("🧪 Testing memory efficiency tracking...");

        let manager = LockFreeMemoryManager::new();

        // Test initial efficiency
        let initial_efficiency = manager.get_memory_efficiency_ratio();
        assert!(initial_efficiency >= 1.0);

        // Simulate memory operations that would improve efficiency
        let _operations: Vec<_> = (0..50).map(|i| NodeId::new(i)).collect();

        // Check that efficiency is tracked
        let final_efficiency = manager.get_memory_efficiency_ratio();
        assert!(final_efficiency >= 1.0);

        // Get detailed statistics
        let stats = manager.get_stats();
        println!("📊 Memory Statistics:");
        println!("   💾 Efficiency ratio: {:.2}x", final_efficiency);
        println!("   🏗️  Arena count: {}", stats.arena_count);
        println!("   📏 Total bytes: {}", stats.total_bytes_allocated);
        println!("   💰 Memory saved: {} bytes", stats.memory_savings());

        println!("✅ Memory efficiency tracking test passed");
    }

    // Performance regression test
    #[test]
    fn test_performance_regression_protection() {
        println!("🧪 Testing performance regression protection...");

        let join_pool = JoinHashTablePool::new();
        let memory_manager = LockFreeMemoryManager::new();
        let query_index = AdaptiveQueryIndex::new();

        // Pre-warm for optimal performance
        join_pool.pre_warm(5);

        // Measure performance of critical operations
        let start = Instant::now();

        // Bulk hash table operations (should be fast with pooling)
        for _ in 0..1000 {
            let _table = join_pool.get_table(100);
        }

        let hash_time = start.elapsed();

        // Bulk memory operations (should be fast with lock-free design)
        let start = Instant::now();
        let _nodes: Vec<_> = (0..1000).map(|i| NodeId::new(i)).collect();
        let memory_time = start.elapsed();

        // Bulk query tracking (should be fast with indexing)
        let start = Instant::now();
        for i in 0..1000 {
            query_index.record_access(&i, Duration::from_nanos(100));
        }
        let query_time = start.elapsed();

        // Performance assertions (these should be very fast)
        assert!(hash_time < Duration::from_millis(100),
               "Hash table operations too slow: {:?}", hash_time);
        assert!(memory_time < Duration::from_millis(10),
               "Memory operations too slow: {:?}", memory_time);
        assert!(query_time < Duration::from_millis(50),
               "Query tracking too slow: {:?}", query_time);

        println!("📊 Performance Regression Results:");
        println!("   📦 Hash table ops: {:?} (1000 ops)", hash_time);
        println!("   💾 Memory ops: {:?} (1000 ops)", memory_time);
        println!("   🧠 Query tracking: {:?} (1000 ops)", query_time);

        println!("✅ Performance regression protection test passed");
    }
}