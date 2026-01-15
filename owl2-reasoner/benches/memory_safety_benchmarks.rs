//! Performance Benchmarks for Memory Safety Features
//!
//! This benchmark suite measures the performance impact of memory safety features
//! and validates that they don't significantly degrade system performance.
//!
//! Note: Many benchmarks that depended on test helper modules (test_helpers,
//! test_memory_guard) have been removed because those modules are not part of
//! the public API.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use owl2_reasoner::memory::*;

/// Benchmark memory stats collection performance
fn bench_memory_stats_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_stats_collection");

    group.bench_function("get_memory_stats", |b| {
        b.iter(|| black_box(get_memory_stats()))
    });

    group.bench_function("get_memory_pressure_level", |b| {
        b.iter(|| black_box(get_memory_pressure_level()))
    });

    group.bench_function("is_under_memory_pressure", |b| {
        b.iter(|| black_box(is_under_memory_pressure()))
    });

    group.bench_function("detect_memory_leaks", |b| {
        b.iter(|| black_box(detect_memory_leaks()))
    });

    group.finish();
}

criterion_group!(benches, bench_memory_stats_collection);
criterion_main!(benches);
