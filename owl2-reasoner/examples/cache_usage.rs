//! Cache usage examples for the OWL2 Reasoner
//!
//! This example demonstrates various ways to use the configurable caching system
//! in your applications.

use owl2_reasoner::cache::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OWL2 Reasoner Cache Usage Examples ===\n");

    // Example 1: Basic cache usage
    println!("1. Basic Cache Usage");
    println!("   Creating a simple cache with default settings...");

    let simple_cache = BoundedCache::<String, i32>::new(1000);

    simple_cache.insert("key1".to_string(), 42)?;
    simple_cache.insert("key2".to_string(), 100)?;

    if let Some(value) = simple_cache.get(&"key1".to_string())? {
        println!("   Retrieved value: {}", value);
    }

    println!("   Cache size: {}", simple_cache.len()?);
    println!();

    // Example 2: Advanced configuration with builder
    println!("2. Advanced Configuration with Builder");
    println!("   Creating a cache with custom settings...");

    let advanced_cache = BoundedCache::<String, f64>::from_builder(
        CacheConfigBuilder::new()
            .max_size(500)
            .enable_stats(true)
            .enable_memory_pressure(true)
            .memory_pressure_threshold(0.75)
            .cleanup_interval(std::time::Duration::from_secs(30)),
    );

    advanced_cache.insert("pi".to_string(), std::f64::consts::PI)?;
    advanced_cache.insert("e".to_string(), std::f64::consts::E)?;

    println!("   Cache stats: {:?}", advanced_cache.stats());
    println!();

    // Example 3: Different eviction strategies
    println!("3. Different Eviction Strategies");

    // LRU Strategy (default)
    let lru_cache = BoundedCache::<String, String>::new(3);
    lru_cache.insert("a".to_string(), "value_a".to_string())?;
    lru_cache.insert("b".to_string(), "value_b".to_string())?;
    lru_cache.insert("c".to_string(), "value_c".to_string())?;

    // Access 'a' to make it recently used
    let _ = lru_cache.get(&"a".to_string())?;

    // Add 'd' - should evict 'b' (least recently used)
    lru_cache.insert("d".to_string(), "value_d".to_string())?;

    println!("   LRU cache after eviction: {} items", lru_cache.len()?);

    // LFU Strategy
    let lfu_cache = BoundedCache::<String, String, LfuStrategy>::with_strategy(
        CacheConfigBuilder::new().max_size(3).build(),
        LfuStrategy::new().min_access_count(2),
    );

    lfu_cache.insert("x".to_string(), "value_x".to_string())?;
    lfu_cache.insert("y".to_string(), "value_y".to_string())?;
    lfu_cache.insert("z".to_string(), "value_z".to_string())?;

    // Access 'x' multiple times to make it frequently used
    for _ in 0..3 {
        let _ = lfu_cache.get(&"x".to_string())?;
    }

    // Add 'w' - should evict least frequently used
    lfu_cache.insert("w".to_string(), "value_w".to_string())?;

    println!("   LFU cache after eviction: {} items", lfu_cache.len()?);
    println!();

    // Example 4: Custom eviction strategy
    println!("4. Custom Eviction Strategy");

    #[derive(Debug, Clone, Default)]
    struct TimeBasedEviction {
        max_age_seconds: u64,
    }

    impl EvictionStrategy for TimeBasedEviction {
        fn should_evict<K, V>(&self, _key: &K, _value: &V, metadata: &CacheMetadata) -> bool
        where
            K: std::hash::Hash + Eq + std::fmt::Debug + ?Sized,
            V: Clone + std::fmt::Debug,
        {
            let age = metadata.last_accessed.elapsed().as_secs();
            age > self.max_age_seconds
        }

        fn name(&self) -> &'static str {
            "TimeBased"
        }
    }

    let time_cache = BoundedCache::<String, Vec<i32>, TimeBasedEviction>::with_strategy(
        CacheConfigBuilder::new().max_size(100).build(),
        TimeBasedEviction { max_age_seconds: 1 },
    );

    time_cache.insert("numbers".to_string(), vec![1, 2, 3, 4, 5])?;

    // Wait for eviction
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Force eviction by adding new items
    for i in 0..10 {
        time_cache.insert(format!("key_{}", i), vec![i])?;
    }

    println!("   Time-based cache size: {}", time_cache.len()?);
    println!();

    // Example 5: Cache in a real application context
    println!("5. Real-world Application: Ontology Entity Caching");

    #[derive(Debug, Clone)]
    struct EntityInfo {
        name: String,
        description: String,
        _created_at: std::time::SystemTime,
    }

    let entity_cache = BoundedCache::<String, EntityInfo>::from_builder(
        CacheConfigBuilder::new()
            .max_size(10_000)
            .enable_stats(true)
            .enable_memory_pressure(true)
            .memory_pressure_threshold(0.8),
    );

    // Simulate loading entities
    let entities = vec![
        ("Person", "Human being"),
        ("Organization", "Formal group of people"),
        ("Location", "Geographical place"),
        ("Event", "Something that happens"),
        ("Product", "Commercial item"),
    ];

    for (name, desc) in entities {
        let info = EntityInfo {
            name: name.to_string(),
            description: desc.to_string(),
            _created_at: std::time::SystemTime::now(),
        };

        entity_cache.insert(name.to_string(), info)?;
    }

    // Retrieve some entities
    if let Some(person) = entity_cache.get(&"Person".to_string())? {
        println!(
            "   Retrieved entity: {} - {}",
            person.name, person.description
        );
    }

    if let Some(org) = entity_cache.get(&"Organization".to_string())? {
        println!("   Retrieved entity: {} - {}", org.name, org.description);
    }

    // Show cache statistics
    let stats = entity_cache.stats();
    println!("   Cache hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("   Total operations: {}", stats.hits + stats.misses);
    println!("   Current size: {}", stats.current_size);
    println!();

    // Example 6: Cache performance comparison
    println!("6. Performance Comparison");

    // Run separate benchmarks per strategy (avoid closure type unification issues)
    // LRU
    {
        let cache = BoundedCache::<usize, String, LruStrategy>::new(1000);
        let start = std::time::Instant::now();
        for i in 0..2000 {
            let _ = cache.insert(i, format!("item_{}", i));
        }
        for i in 0..500 {
            let _ = cache.get(&i);
        }
        println!("   LRU strategy: {:?}", start.elapsed());
    }

    // LFU
    {
        let cache = BoundedCache::<usize, String, LfuStrategy>::with_strategy(
            CacheConfigBuilder::new().max_size(1000).build(),
            LfuStrategy::new().min_access_count(3),
        );
        let start = std::time::Instant::now();
        for i in 0..2000 {
            let _ = cache.insert(i, format!("item_{}", i));
        }
        for i in 0..500 {
            let _ = cache.get(&i);
        }
        println!("   LFU strategy: {:?}", start.elapsed());
    }

    // FIFO
    {
        let cache = BoundedCache::<usize, String, FifoStrategy>::with_strategy(
            CacheConfigBuilder::new().max_size(1000).build(),
            FifoStrategy::new(),
        );
        let start = std::time::Instant::now();
        for i in 0..2000 {
            let _ = cache.insert(i, format!("item_{}", i));
        }
        for i in 0..500 {
            let _ = cache.get(&i);
        }
        println!("   FIFO strategy: {:?}", start.elapsed());
    }

    // Random
    {
        let cache = BoundedCache::<usize, String, RandomStrategy>::with_strategy(
            CacheConfigBuilder::new().max_size(1000).build(),
            RandomStrategy::new(),
        );
        let start = std::time::Instant::now();
        for i in 0..2000 {
            let _ = cache.insert(i, format!("item_{}", i));
        }
        for i in 0..500 {
            let _ = cache.get(&i);
        }
        println!("   Random strategy: {:?}", start.elapsed());
    }

    println!("\n=== Cache Usage Examples Complete ===");
    Ok(())
}
