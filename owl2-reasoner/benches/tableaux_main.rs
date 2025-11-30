//! Main Tableaux Benchmark Suite
//!
//! This benchmark suite comprehensively tests the performance of the new modular
//! tableaux reasoning engine, including individual component benchmarks and
//! integration tests.

use criterion::{criterion_group, criterion_main, Criterion};

// Import all tableaux benchmark functions
mod tableaux_benchmarks;
use tableaux_benchmarks::*;

fn tableaux_benchmark_suite(c: &mut Criterion) {
    println!("🚀 Running Tableaux Module Performance Benchmarks");
    println!("=================================================");

    // Core tableaux reasoner benchmarks
    bench_tableaux_core(c);

    // Graph operations benchmarks
    bench_tableaux_graph(c);

    // Memory management benchmarks
    bench_tableaux_memory(c);

    // Blocking strategy benchmarks
    bench_tableaux_blocking(c);

    // Dependency management benchmarks
    bench_tableaux_dependency(c);

    // Expansion rule benchmarks
    bench_tableaux_expansion(c);

    // Integration benchmarks
    bench_tableaux_integration(c);

    // Memory usage benchmarks
    bench_tableaux_memory_usage(c);

    // Cache performance benchmarks
    bench_tableaux_caching(c);

    println!("✅ All tableaux benchmarks completed successfully!");
}

criterion_group!(benches, tableaux_benchmark_suite);
criterion_main!(benches);
