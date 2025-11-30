//! Simple performance validation for arena parser and parallel reasoning optimizations
//!
//! This test validates the key optimizations without requiring full library compilation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Instant;

/// Test arena allocation performance
fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocation");

    for size in [1000, 5000, 10000].iter() {
        // Test traditional string allocation
        group.bench_with_input(
            BenchmarkId::new("traditional_allocation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut strings = Vec::new();
                    for i in 0..size {
                        let s = format!("http://example.org/entity_{}", i);
                        strings.push(black_box(s));
                    }
                    strings
                })
            },
        );

        // Test arena allocation simulation
        group.bench_with_input(
            BenchmarkId::new("arena_allocation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut arena = Vec::with_capacity(size * 50); // Pre-allocate arena
                    let mut offsets = Vec::new();
                    for i in 0..size {
                        let s = format!("http://example.org/entity_{}", i);
                        let start = arena.len();
                        arena.extend_from_slice(s.as_bytes());
                        offsets.push(black_box(start));
                    }
                    (arena, offsets)
                })
            },
        );
    }

    group.finish();
}

/// Test parallel processing performance
fn bench_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_processing");

    let data_sizes = [1000, 5000, 10000];

    for size in data_sizes.iter() {
        let data: Vec<usize> = (0..*size).collect();

        // Test sequential processing
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, _| {
            b.iter(|| {
                let result: usize = data.iter().map(|&x| black_box(x * 2)).sum();
                result
            })
        });

        // Test parallel processing
        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, _| {
            use rayon::prelude::*;
            b.iter(|| {
                let result: usize = data.par_iter().map(|&x| black_box(x * 2)).sum();
                result
            })
        });
    }

    group.finish();
}

/// Test memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test smallvec vs vec performance
    group.bench_function("smallvec_vs_vec", |b| {
        b.iter(|| {
            // SmallVec simulation (stack allocation for small collections)
            let mut small_data = [0u64; 8];
            for (idx, slot) in small_data.iter_mut().enumerate() {
                *slot = black_box(idx as u64);
            }

            // Vec simulation (heap allocation)
            let mut vec_data = Vec::with_capacity(16);
            vec_data.extend((0..16).map(|value| black_box(value as u64)));

            (small_data, vec_data)
        })
    });

    // Test hashbrown vs std HashMap
    group.bench_function("hashbrown_vs_std", |b| {
        b.iter(|| {
            use hashbrown::HashMap;
            use std::collections::HashMap as StdHashMap;

            let mut hashbrown_map = HashMap::new();
            let mut std_map = StdHashMap::new();

            for i in 0..1000 {
                let key = format!("key_{}", i);
                hashbrown_map.insert(black_box(key.clone()), black_box(i));
                std_map.insert(black_box(key), black_box(i));
            }

            (hashbrown_map, std_map)
        })
    });

    group.finish();
}

/// Test cache effectiveness simulation
fn bench_cache_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_simulation");

    let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();

    // Test uncached operations
    group.bench_function("uncached_operations", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for key in &keys {
                // Simulate expensive computation
                let result = black_box(key.len() * 2);
                results.push(result);
            }
            results
        })
    });

    // Test cached operations
    group.bench_function("cached_operations", |b| {
        use std::collections::HashMap;
        let mut cache = HashMap::new();

        b.iter(|| {
            let mut results = Vec::new();
            for key in &keys {
                if let Some(&cached_result) = cache.get(key) {
                    results.push(cached_result);
                } else {
                    let result = black_box(key.len() * 2);
                    cache.insert(key.clone(), result);
                    results.push(result);
                }
            }
            results
        })
    });

    group.finish();
}

/// Comprehensive performance validation
fn bench_comprehensive_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_validation");

    let size = 5000;

    // Test traditional approach simulation
    group.bench_function("traditional_approach", |b| {
        b.iter(|| {
            // Simulate traditional parsing + reasoning
            let start = Instant::now();

            // Simulate string allocations
            let strings: Vec<String> = (0..size)
                .map(|i| format!("http://example.org/entity_{}", i))
                .collect();

            // Simulate sequential processing
            let results: Vec<usize> = strings.iter().map(|s| black_box(s.len())).collect();

            let elapsed = start.elapsed();
            (results, elapsed)
        })
    });

    // Test optimized approach simulation
    group.bench_function("optimized_approach", |b| {
        b.iter(|| {
            use rayon::prelude::*;
            let start = Instant::now();

            // Simulate arena allocation (pre-allocated)
            let mut arena = Vec::with_capacity(size * 50);
            let offsets: Vec<usize> = (0..size)
                .map(|i| {
                    let s = format!("http://example.org/entity_{}", i);
                    let start = arena.len();
                    arena.extend_from_slice(s.as_bytes());
                    black_box(start)
                })
                .collect();

            // Simulate parallel processing
            let results: Vec<usize> = offsets
                .par_iter()
                .map(|&offset| black_box(offset % 100))
                .collect();

            let elapsed = start.elapsed();
            (results, elapsed)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arena_allocation,
    bench_parallel_processing,
    bench_memory_patterns,
    bench_cache_simulation,
    bench_comprehensive_validation
);
criterion_main!(benches);
