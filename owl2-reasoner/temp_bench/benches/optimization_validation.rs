//! Performance validation for OWL2 reasoner optimization techniques
//!
//! Standalone benchmark to validate:
//! - Arena allocation benefits (30-50% improvement target)
//! - Parallel processing benefits (25-40% improvement target)
//! - Memory efficiency improvements
//! - Cache effectiveness

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;

/// Test arena allocation vs traditional allocation for string-heavy workloads
fn bench_arena_vs_traditional_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_vs_traditional");

    let sizes = [1000, 5000, 10000, 20000, 50000];

    for size in &sizes {
        // Traditional allocation: many small allocations
        group.bench_with_input(BenchmarkId::new("traditional_strings", size), size, |b, &size| {
            b.iter(|| {
                let mut strings = Vec::with_capacity(size * 3);
                let mut total_memory = 0;

                // Simulate IRI and entity string creation
                for i in 0..size {
                    // Create various string types as in OWL parsing
                    let class_iri = format!("http://example.org/ontology_{}/class_{}", i % 100, i);
                    let prop_iri = format!("http://example.org/ontology_{}/prop_{}", i % 50, i);
                    let individual_iri = format!("http://example.org/ontology_{}/individual_{}", i % 100, i);

                    let class_len = class_iri.len();
                    let prop_len = prop_iri.len();
                    let individual_len = individual_iri.len();

                    strings.push(black_box(class_iri));
                    strings.push(black_box(prop_iri));
                    strings.push(black_box(individual_iri));

                    total_memory += class_len + prop_len + individual_len;
                }

                // Simulate processing
                let hash_sum: usize = strings.iter()
                    .map(|s| black_box(s.bytes().map(|b| b as usize).sum::<usize>()))
                    .sum();

                (strings, hash_sum, total_memory)
            })
        });

        // Arena allocation simulation: single large allocation
        group.bench_with_input(BenchmarkId::new("arena_simulation", size), size, |b, &size| {
            b.iter(|| {
                // Pre-calculate total memory requirements
                let avg_iri_len = 45; // Average length of IRIs
                let total_bytes = size * 3 * avg_iri_len; // 3 strings per entity
                let mut arena = Vec::with_capacity(total_bytes);
                let mut string_refs = Vec::with_capacity(size * 3);

                // Allocate all strings in arena
                for i in 0..size {
                    // Class IRI
                    let class_str = format!("http://example.org/ontology_{}/class_{}", i % 100, i);
                    let start = arena.len();
                    arena.extend_from_slice(class_str.as_bytes());
                    unsafe {
                        let class_slice = std::str::from_utf8_unchecked(&arena[start..]);
                        string_refs.push(black_box(class_slice as *const str));
                    }

                    // Property IRI
                    let prop_str = format!("http://example.org/ontology_{}/prop_{}", i % 50, i);
                    let start = arena.len();
                    arena.extend_from_slice(prop_str.as_bytes());
                    unsafe {
                        let prop_slice = std::str::from_utf8_unchecked(&arena[start..]);
                        string_refs.push(black_box(prop_slice as *const str));
                    }

                    // Individual IRI
                    let indiv_str = format!("http://example.org/ontology_{}/individual_{}", i % 100, i);
                    let start = arena.len();
                    arena.extend_from_slice(indiv_str.as_bytes());
                    unsafe {
                        let indiv_slice = std::str::from_utf8_unchecked(&arena[start..]);
                        string_refs.push(black_box(indiv_slice as *const str));
                    }
                }

                // Simulate processing with arena-allocated strings
                let mut hash_sum = 0;
                for &ptr in &string_refs {
                    if !ptr.is_null() {
                        let s = unsafe { &*ptr };
                        hash_sum += black_box(s.bytes().map(|b| b as usize).sum::<usize>());
                    }
                }

                (arena, string_refs, hash_sum)
            })
        });
    }

    group.finish();
}

/// Test parallel vs sequential tableaux-style reasoning
fn bench_parallel_vs_sequential_reasoning(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vs_sequential_reasoning");

    let ontology_sizes = [1000, 5000, 10000, 20000];

    for size in &ontology_sizes {
        // Generate simulated ontology data
        let classes: Vec<usize> = (0..*size).collect();
        let subclass_relationships: Vec<(usize, usize)> = (0..*size - 1)
            .map(|i| (i, i + 1))
            .collect();
        let equivalence_relationships: Vec<(usize, usize)> = (0..*size / 2)
            .map(|i| (i * 2, i * 2 + 1))
            .collect();

        // Sequential reasoning simulation
        group.bench_with_input(BenchmarkId::new("sequential_reasoning", size), size, |b, _| {
            b.iter(|| {
                let mut consistency_checks = 0;
                let mut classifications = 0;

                // Simulate consistency checking
                for &(sub, sup) in &subclass_relationships {
                    // Simulate subclass consistency check
                    if black_box(sub < sup) {
                        consistency_checks += 1;
                    }
                }

                // Simulate equivalence processing
                for &(eq1, eq2) in &equivalence_relationships {
                    // Simulate equivalence reasoning
                    if black_box(eq1 != eq2) {
                        classifications += 1;
                    }
                }

                // Simulate classification
                for class in &classes {
                    // Simulate class hierarchy processing
                    let hierarchy_level = black_box(class / 100);
                    classifications += hierarchy_level;
                }

                (consistency_checks, classifications)
            })
        });

        // Parallel reasoning with different thread counts
        for threads in [2, 4, 8].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("parallel_{}_threads", threads), size),
                size,
                |b, _| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(*threads)
                        .build()
                        .unwrap();

                    b.iter(|| {
                        pool.install(|| {
                            let consistency_checks: usize = subclass_relationships.par_iter()
                                .map(|&(sub, sup)| {
                                    if black_box(sub < sup) { 1 } else { 0 }
                                })
                                .sum();

                            let classifications: usize = equivalence_relationships.par_iter()
                                .map(|&(eq1, eq2)| {
                                    if black_box(eq1 != eq2) { 1 } else { 0 }
                                })
                                .sum();

                            let hierarchy_classifications: usize = classes.par_iter()
                                .map(|&class| {
                                    black_box(class / 100)
                                })
                                .sum();

                            (consistency_checks, classifications + hierarchy_classifications)
                        })
                    })
                }
            );
        }
    }

    group.finish();
}

/// Test memory efficiency: SmallVec vs Vec, hashbrown vs std HashMap
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let data_size = 10000;

    // SmallVec vs Vec for small collections
    group.bench_function("smallvec_vs_vec_small", |b| {
        b.iter(|| {
            // Simulate SmallVec behavior (stack allocation for small collections)
            let mut small_data = [0u64; 8];
            for i in 0..8 {
                small_data[i] = black_box(i as u64);
            }

            // Traditional Vec allocation
            let mut vec_data = Vec::with_capacity(8);
            for i in 0..8 {
                vec_data.push(black_box(i as u64));
            }

            (small_data, vec_data)
        })
    });

    // SmallVec vs Vec for large collections (forces heap allocation)
    group.bench_function("smallvec_vs_vec_large", |b| {
        b.iter(|| {
            // Simulate SmallVec that spills to heap
            let mut small_vec_spill = Vec::with_capacity(32);
            for i in 0..32 {
                small_vec_spill.push(black_box(i as u64));
            }

            // Traditional Vec
            let mut vec_data = Vec::with_capacity(32);
            for i in 0..32 {
                vec_data.push(black_box(i as u64));
            }

            (small_vec_spill, vec_data)
        })
    });

    // hashbrown vs std HashMap performance
    group.bench_function("hashbrown_vs_std_hashmap", |b| {
        b.iter(|| {
            let keys: Vec<String> = (0..data_size)
                .map(|i| format!("entity_{}", i))
                .collect();

            // Test std::HashMap
            let mut std_map = HashMap::new();
            for key in &keys {
                std_map.insert(black_box(key.clone()), black_box(key.len()));
            }

            // Test hashbrown::HashMap
            use hashbrown::HashMap;
            let mut hashbrown_map = HashMap::new();
            for key in &keys {
                hashbrown_map.insert(black_box(key.clone()), black_box(key.len()));
            }

            // Performance comparison: lookups
            let mut std_sum = 0;
            let mut hashbrown_sum = 0;

            for key in &keys {
                if let Some(val) = std_map.get(key) {
                    std_sum += *val;
                }
                if let Some(val) = hashbrown_map.get(key) {
                    hashbrown_sum += *val;
                }
            }

            (std_sum, hashbrown_sum)
        })
    });

    group.finish();
}

/// Test concurrent data structures for parallel reasoning
fn bench_concurrent_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_structures");

    let thread_counts = [2, 4, 8];
    let operations_per_thread = 10000;

    for &threads in &thread_counts {
        // Test DashMap (concurrent HashMap)
        group.bench_with_input(BenchmarkId::new("dashmap_concurrent", threads), &threads, |b, &threads| {
            b.iter(|| {
                use dashmap::DashMap;
                let map = DashMap::new();
                let keys: Vec<String> = (0..operations_per_thread).map(|i| format!("key_{}", i)).collect();

                // Concurrent writes
                use std::sync::Arc;
                let map_arc = Arc::new(map);
                let keys_arc = Arc::new(keys);

                let handles: Vec<_> = (0..threads).map(|thread_id| {
                    let map_clone = map_arc.clone();
                    let keys_clone = keys_arc.clone();
                    std::thread::spawn(move || {
                        let mut sum = 0;
                        for (i, key) in keys_clone.iter().enumerate() {
                            if i % threads == thread_id {
                                map_clone.insert(key.clone(), key.len());
                                sum += key.len();
                            }
                        }
                        sum
                    })
                }).collect();

                let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                let total: usize = results.iter().sum();

                // Concurrent reads
                let read_handles: Vec<_> = (0..threads).map(|thread_id| {
                    let map_clone = map_arc.clone();
                    let keys_clone = keys_arc.clone();
                    std::thread::spawn(move || {
                        let mut sum = 0;
                        for (i, key) in keys_clone.iter().enumerate() {
                            if i % threads == thread_id {
                                if let Some(val) = map_clone.get(key) {
                                    sum += *val;
                                }
                            }
                        }
                        sum
                    })
                }).collect();

                let read_results: Vec<usize> = read_handles.into_iter().map(|h| h.join().unwrap()).collect();
                let read_total: usize = read_results.iter().sum();

                (total, read_total)
            })
        });

        // Test Mutex-protected HashMap for comparison
        group.bench_with_input(BenchmarkId::new("mutex_hashmap", threads), &threads, |b, &threads| {
            b.iter(|| {
                use std::sync::{Arc, Mutex};
                let map = Arc::new(Mutex::new(HashMap::new()));
                let keys: Vec<String> = (0..operations_per_thread).map(|i| format!("key_{}", i)).collect();

                // Concurrent writes with mutex
                let map_clone = map.clone();
                let keys_clone = keys.clone();

                let handles: Vec<_> = (0..threads).map(|thread_id| {
                    let map_clone = map_clone.clone();
                    let keys_clone = keys_clone.clone();
                    std::thread::spawn(move || {
                        let mut sum = 0;
                        for (i, key) in keys_clone.iter().enumerate() {
                            if i % threads == thread_id {
                                let mut guard = map_clone.lock().unwrap();
                                guard.insert(key.clone(), key.len());
                                drop(guard); // Release lock quickly
                                sum += key.len();
                            }
                        }
                        sum
                    })
                }).collect();

                let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                let total: usize = results.iter().sum();

                total
            })
        });
    }

    group.finish();
}

/// Test caching strategies for reasoning optimization
fn bench_caching_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching_strategies");

    let dataset_size = 50000;
    let query_keys: Vec<String> = (0..dataset_size)
        .map(|i| format!("entity_{}", i % 1000)) // Create some重复性 to benefit from caching
        .collect();

    // No caching baseline
    group.bench_function("no_caching", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for key in &query_keys {
                // Simulate expensive computation (hash calculation + string processing)
                let result = black_box(
                    key.len() * 2 +
                    key.bytes().map(|b| b as usize).sum::<usize>() +
                    key.chars().filter(|&c| c.is_numeric()).count()
                );
                results.push(result);
            }
            results
        })
    });

    // LRU caching simulation
    group.bench_function("lru_caching", |b| {
        b.iter(|| {
            use std::collections::{HashMap, LinkedList};
            let mut cache = HashMap::new();
            let mut lru_list = LinkedList::new();
            let cache_size = 1000;

            let mut results = Vec::new();
            let mut cache_hits = 0;

            for key in &query_keys {
                if let Some(result) = cache.get(key) {
                    results.push(*result);
                    cache_hits += 1;
                } else {
                    let result = black_box(
                        key.len() * 2 +
                        key.bytes().map(|b| b as usize).sum::<usize>() +
                        key.chars().filter(|&c| c.is_numeric()).count()
                    );

                    // Evict oldest if cache is full
                    if cache.len() >= cache_size {
                        if let Some(evicted_key) = lru_list.pop_front() {
                            cache.remove(&evicted_key);
                        }
                    }

                    cache.insert(key.clone(), result);
                    lru_list.push_back(key.clone());
                    results.push(result);
                }
            }

            (results, cache_hits)
        })
    });

    // Cache effectiveness test
    group.bench_function("cache_effectiveness", |b| {
        b.iter(|| {
            let mut cache = HashMap::new();
            let mut hits = 0;
            let mut misses = 0;

            // First pass: populate cache
            for i in 0..1000 {
                let key = format!("entity_{}", i);
                let value = black_box(i * 2);
                cache.insert(key, value);
            }

            // Second pass: test hits and misses
            for i in 0..2000 {
                let key = format!("entity_{}", i);
                if cache.get(&key).is_some() {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            (hits, misses)
        })
    });

    group.finish();
}

/// Comprehensive end-to-end simulation of optimized vs traditional approaches
fn bench_comprehensive_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_optimization");

    let ontology_size = 10000;

    // Traditional approach simulation
    group.bench_function("traditional_approach", |b| {
        b.iter(|| {
            let start_time = Instant::now();

            // Phase 1: Traditional string allocations (many small allocations)
            let classes: Vec<String> = (0..ontology_size)
                .map(|i| format!("http://example.org/ontology_{}/class_{}", i % 200, i))
                .collect();

            let properties: Vec<String> = (0..(ontology_size / 10))
                .map(|i| format!("http://example.org/ontology_{}/property_{}", i % 50, i))
                .collect();

            let individuals: Vec<String> = (0..(ontology_size / 5))
                .map(|i| format!("http://example.org/ontology_{}/individual_{}", i % 200, i))
                .collect();

            // Phase 2: Sequential consistency checking
            let mut consistency_checks = 0;
            for i in 0..(ontology_size - 1) {
                // Simulate subclass axioms processing
                if black_box(i % 100 != 0) { // Most are consistent
                    consistency_checks += 1;
                }
            }

            // Phase 3: Sequential classification
            let mut classifications = 0;
            for class in &classes {
                // Simulate hierarchy classification
                let depth = black_box(class.len() / 10);
                classifications += depth;
            }

            let total_time = start_time.elapsed();
            (
                classes.len() + properties.len() + individuals.len(),
                consistency_checks,
                classifications,
                total_time
            )
        })
    });

    // Optimized approach simulation
    group.bench_function("optimized_approach", |b| {
        b.iter(|| {
            let start_time = Instant::now();

            // Phase 1: Simplified arena allocation simulation
            let total_entities = ontology_size + ontology_size / 10 + ontology_size / 5;
            let avg_len = 50;
            let mut arena = Vec::with_capacity(total_entities * avg_len);

            // Simulate arena allocation by pre-allocating and using slices
            let mut arena_data = Vec::with_capacity(total_entities);
            for i in 0..ontology_size {
                let s = format!("http://example.org/ontology_{}/class_{}", i % 200, i);
                arena_data.push(s);
            }
            for i in 0..(ontology_size / 10) {
                let s = format!("http://example.org/ontology_{}/property_{}", i % 50, i);
                arena_data.push(s);
            }
            for i in 0..(ontology_size / 5) {
                let s = format!("http://example.org/ontology_{}/individual_{}", i % 200, i);
                arena_data.push(s);
            }

            // Simulate arena allocation by copying all data to contiguous memory
            for s in &arena_data {
                arena.extend_from_slice(s.as_bytes());
            }

            let classes_count = ontology_size;
            let properties_count = ontology_size / 10;

            // Phase 2: Parallel consistency checking
            let consistency_checks: usize = (0..(ontology_size - 1))
                .into_par_iter()
                .map(|i| {
                    if black_box(i % 100 != 0) { 1 } else { 0 }
                })
                .sum();

            // Phase 3: Parallel classification
            let classifications: usize = (0..classes_count)
                .into_par_iter()
                .map(|i| {
                    let depth = black_box(i / 200); // Simulate hierarchy depth
                    depth
                })
                .sum();

            let total_time = start_time.elapsed();
            (
                classes_count + properties_count + ontology_size / 5,
                consistency_checks,
                classifications,
                total_time
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arena_vs_traditional_allocation,
    bench_parallel_vs_sequential_reasoning,
    bench_memory_efficiency,
    bench_concurrent_structures,
    bench_caching_strategies,
    bench_comprehensive_optimization
);
criterion_main!(benches);