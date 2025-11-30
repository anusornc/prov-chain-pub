//! Comprehensive benchmark tests for performance optimizations
//!
//! Validates the performance improvements from:
//! - JoinHashTablePool for reusable hash join operations
//! - LockFreeMemoryManager for thread-local arena allocation
//! - AdaptiveQueryIndex for intelligent query caching

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use owl2_reasoner::reasoning::query::cache::*;
use owl2_reasoner::reasoning::query::types::*;
use owl2_reasoner::reasoning::tableaux::memory::*;
use owl2_reasoner::axioms::*;
use owl2_reasoner::entities::*;
use owl2_reasoner::iri::IRI;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Benchmark JoinHashTablePool performance
fn benchmark_join_hash_table_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("join_hash_table_pool");

    // Test different binding sizes
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        // Baseline: Create new HashMap each time
        group.bench_with_input(
            BenchmarkId::new("baseline", size),
            size,
            |b, &size| {
                let bindings = create_test_bindings(size);
                let common_vars = vec_string(&["?x".to_string(), "?y".to_string()]);

                b.iter(|| {
                    // Traditional approach: create new HashMap each time
                    let mut hash_table: HashMap<Vec<QueryValue>, Vec<usize>> = HashMap::new();
                    for (idx, binding) in bindings.iter().enumerate() {
                        let key = extract_join_key(binding, &common_vars);
                        hash_table.entry(key).or_default().push(idx);
                    }

                    // Simulate probe phase
                    let mut results = 0;
                    for binding in &bindings {
                        let key = extract_join_key(binding, &common_vars);
                        if let Some(indices) = hash_table.get(&key) {
                            results += indices.len();
                        }
                    }
                    black_box(results);
                });
            },
        );

        // Optimized: Use JoinHashTablePool
        group.bench_with_input(
            BenchmarkId::new("optimized", size),
            size,
            |b, &size| {
                let bindings = create_test_bindings(size);
                let common_vars = vec_string(&["?x".to_string(), "?y".to_string()]);
                let pool = JoinHashTablePool::new();
                pool.pre_warm(5); // Pre-warm pool

                b.iter(|| {
                    let mut hash_table = pool.get_table(size);
                    hash_table.build_from_bindings(&bindings, &common_vars);

                    // Simulate probe phase
                    let mut results = 0;
                    for binding in &bindings {
                        let key = extract_join_key(binding, &common_vars);
                        if let Some(indices) = hash_table.get_indices(&key) {
                            results += indices.len();
                        }
                    }
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark LockFreeMemoryManager performance
fn benchmark_lock_free_memory_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_free_memory_manager");

    // Test different allocation counts
    for count in [1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        // Baseline: Traditional mutex-based memory manager
        group.bench_with_input(
            BenchmarkId::new("baseline", count),
            count,
            |b, &count| {
                let memory_manager = MemoryManager::new();
                let nodes = create_test_nodes(count);

                b.iter(|| {
                    let mut handles = Vec::new();
                    for chunk in nodes.chunks(100) {
                        let manager = &memory_manager;
                        let chunk = chunk.to_vec();

                        let handle = std::thread::spawn(move || {
                            let mut results = 0;
                            for node in chunk {
                                if manager.allocate_node(node.clone()).is_ok() {
                                    results += 1;
                                }
                            }
                            results
                        });
                        handles.push(handle);
                    }

                    let mut total = 0;
                    for handle in handles {
                        total += handle.join().unwrap();
                    }
                    black_box(total);
                });
            },
        );

        // Optimized: Lock-free memory manager
        group.bench_with_input(
            BenchmarkId::new("optimized", count),
            count,
            |b, &count| {
                let memory_manager = LockFreeMemoryManager::new();
                let nodes = create_test_nodes(count);

                b.iter(|| {
                    let mut handles = Vec::new();
                    for chunk in nodes.chunks(100) {
                        let manager = &memory_manager;
                        let chunk = chunk.to_vec();

                        let handle = std::thread::spawn(move || {
                            let mut results = 0;
                            for node in chunk {
                                if manager.allocate_node(node.clone()).is_ok() {
                                    results += 1;
                                }
                            }
                            results
                        });
                        handles.push(handle);
                    }

                    let mut total = 0;
                    for handle in handles {
                        total += handle.join().unwrap();
                    }
                    black_box(total);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark AdaptiveQueryIndex performance
fn benchmark_adaptive_query_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_query_index");

    // Test different numbers of unique queries
    for unique_queries in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*unique_queries as u64));

        // Baseline: Linear scan through query cache
        group.bench_with_input(
            BenchmarkId::new("baseline", unique_queries),
            unique_queries,
            |b, &unique_queries| {
                let queries = create_test_queries(unique_queries);
                let query_cache: HashMap<u64, CompiledPattern> = HashMap::new();

                b.iter(|| {
                    let mut hits = 0;
                    for query in &queries {
                        let pattern_hash = compute_pattern_hash(query);
                        if query_cache.contains_key(&pattern_hash) {
                            hits += 1;
                        }
                    }
                    black_box(hits);
                });
            },
        );

        // Optimized: AdaptiveQueryIndex
        group.bench_with_input(
            BenchmarkId::new("optimized", unique_queries),
            unique_queries,
            |b, &unique_queries| {
                let queries = create_test_queries(unique_queries);
                let index = AdaptiveQueryIndex::new();

                // Pre-populate some queries
                for query in queries.iter().take(unique_queries / 2) {
                    index.get_or_create(query);
                }

                b.iter(|| {
                    let mut hits = 0;
                    for query in &queries {
                        if index.get_or_create(query).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark QueryPatternPredictor performance
fn benchmark_query_pattern_predictor(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_pattern_predictor");

    // Test different sequence lengths
    for sequence_length in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*sequence_length as u64));

        group.bench_with_input(
            BenchmarkId::new("prediction_accuracy", sequence_length),
            sequence_length,
            |b, &sequence_length| {
                let predictor = QueryPatternPredictor::new();
                let query_patterns = create_test_query_patterns(sequence_length);

                // Train the predictor
                for pattern in &query_patterns {
                    predictor.record_query(pattern, Duration::from_millis(1));
                }

                b.iter(|| {
                    let current_pattern = &query_patterns[query_patterns.len() - 1];
                    let predictions = predictor.predict_next_queries(current_pattern, 5);
                    black_box(predictions);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hot_patterns", sequence_length),
            sequence_length,
            |b, &sequence_length| {
                let predictor = QueryPatternPredictor::new();
                let query_patterns = create_test_query_patterns(sequence_length);

                // Train the predictor
                for pattern in &query_patterns {
                    predictor.record_query(pattern, Duration::from_millis(1));
                }

                b.iter(|| {
                    let hot_patterns = predictor.get_hot_patterns(1.0);
                    black_box(hot_patterns);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory efficiency comparison
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    group.bench_function("traditional_allocation", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for i in 0..1000 {
                let node = TableauxNode::new(crate::reasoning::core::NodeId::new(i));
                allocations.push(Box::new(node));
            }
            black_box(allocations.len());
        });
    });

    group.bench_function("lock_free_arena_allocation", |b| {
        let memory_manager = LockFreeMemoryManager::new();

        b.iter(|| {
            let mut allocations = Vec::new();
            for i in 0..1000 {
                let node = TableauxNode::new(crate::reasoning::core::NodeId::new(i));
                if let Ok(arena_node) = memory_manager.allocate_node(node) {
                    allocations.push(arena_node);
                }
            }
            black_box(allocations.len());
        });
    });

    group.finish();
}

/// Benchmark cache hit rates
fn benchmark_cache_hit_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rates");

    // Create a workload with repeated queries
    let repeated_queries = create_repeated_queries(1000);

    // Baseline: No adaptive caching
    group.bench_function("no_adaptive_caching", |b| {
        let cache: HashMap<u64, CompiledPattern> = HashMap::new();

        b.iter(|| {
            let mut hits = 0;
            for query in &repeated_queries {
                let pattern_hash = compute_pattern_hash(query);
                if cache.contains_key(&pattern_hash) {
                    hits += 1;
                }
            }
            black_box(hits);
        });
    });

    // Optimized: AdaptiveQueryIndex
    group.bench_function("adaptive_caching", |b| {
        let index = AdaptiveQueryIndex::new();

        b.iter(|| {
            let mut hits = 0;
            for query in &repeated_queries {
                if index.get_or_create(query).is_some() {
                    hits += 1;
                }
            }
            black_box(hits);
        });
    });

    group.finish();
}

// Helper functions for benchmark data generation

fn create_test_bindings(count: usize) -> Vec<QueryBinding> {
    let mut bindings = Vec::with_capacity(count);

    for i in 0..count {
        let mut binding = QueryBinding::new();
        binding.add_binding(
            "?x".to_string(),
            QueryValue::IRI(IRI::new(&format!("http://example.org/obj{}", i)).unwrap()),
        );
        binding.add_binding(
            "?y".to_string(),
            QueryValue::IRI(IRI::new(&format!("http://example.org/type{}", i % 10)).unwrap()),
        );
        binding.add_binding(
            "?z".to_string(),
            QueryValue::Literal(format!("value{}", i % 100)),
        );
        bindings.push(binding);
    }

    bindings
}

fn extract_join_key(binding: &QueryBinding, vars: &[String]) -> Vec<QueryValue> {
    vars.iter()
        .map(|var| {
            binding.get_value(var)
                .cloned()
                .unwrap_or(QueryValue::Literal("".to_string()))
        })
        .collect()
}

fn create_test_nodes(count: usize) -> Vec<TableauxNode> {
    (0..count)
        .map(|i| TableauxNode::new(crate::reasoning::core::NodeId::new(i)))
        .collect()
}

fn create_test_queries(count: usize) -> Vec<QueryPattern> {
    (0..count)
        .map(|i| {
            QueryPattern::BasicGraphPattern(vec![
                TriplePattern::new(
                    PatternTerm::Variable(format!("?s{}", i)),
                    PatternTerm::IRI(IRI::new("http://example.org/predicate").unwrap()),
                    PatternTerm::Variable(format!("?o{}", i)),
                ),
            ])
        })
        .collect()
}

fn create_test_query_patterns(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("query_pattern_{}", i % 20)) // 20 unique patterns
        .collect()
}

fn create_repeated_queries(count: usize) -> Vec<QueryPattern> {
    // Create queries with 20% repetition rate
    let unique_patterns = create_test_queries(count / 5);
    let mut repeated_queries = Vec::with_capacity(count);

    for i in 0..count {
        repeated_queries.push(unique_patterns[i % unique_patterns.len()].clone());
    }

    repeated_queries
}

fn compute_pattern_hash(pattern: &QueryPattern) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    pattern.hash(&mut hasher);
    hasher.finish()
}

fn vec_string(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| s.to_string()).collect()
}

criterion_group!(
    benches,
    benchmark_join_hash_table_pool,
    benchmark_lock_free_memory_manager,
    benchmark_adaptive_query_index,
    benchmark_query_pattern_predictor,
    benchmark_memory_efficiency,
    benchmark_cache_hit_rates
);

criterion_main!(benches);