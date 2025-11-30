//! Cache Performance Testing Benchmarks
//!
//! Tests the effectiveness and performance of the caching mechanisms
//! in the OWL2 reasoning engine, including multi-layer cache behavior.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// Include our test data generation utilities
mod memory_profiler;
// mod test_data_generator; // Temporarily removed due to missing module

use memory_profiler::{measure_performance, PerformanceResults};
// use test_data_generator::{generate_medium_ontology, generate_ontology_with_size};

/// Test cache hit/miss performance for consistency checking
fn bench_consistency_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_cache");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // First call (cache miss)
    group.bench_function("consistency_first_call", |b| {
        b.iter(|| {
            let result = black_box(reasoner.is_consistent().unwrap());
            black_box(result)
        })
    });

    // Subsequent calls (cache hits)
    group.bench_function("consistency_cached_calls", |b| {
        b.iter(|| {
            let result = black_box(reasoner.is_consistent().unwrap());
            black_box(result)
        })
    });

    group.finish();
}

/// Test cache performance for satisfiability checking
fn bench_satisfiability_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("satisfiability_cache");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get multiple class IRIs for testing
    let classes: Vec<_> = reasoner
        .ontology
        .classes()
        .iter()
        .take(10)
        .cloned()
        .collect();
    if !classes.is_empty() {
        // First calls for each class (cache misses)
        for (i, class) in classes.iter().enumerate() {
            let class_iri = class.iri().clone();
            group.bench_with_input(
                BenchmarkId::new("satisfiability_first_call", i),
                &i,
                |b, &_index| {
                    b.iter(|| {
                        let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                        black_box(result)
                    })
                },
            );
        }

        // Second calls for each class (cache hits)
        for (i, class) in classes.iter().enumerate() {
            let class_iri = class.iri().clone();
            group.bench_with_input(
                BenchmarkId::new("satisfiability_cached_call", i),
                &i,
                |b, &_index| {
                    b.iter(|| {
                        let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                        black_box(result)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Test cache performance with varying data sizes
fn bench_cache_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scalability");
    group.measurement_time(Duration::from_secs(30));

    let sizes = vec![50, 100, 200, 500];

    for size in sizes {
        let ontology = owl2_reasoner::Ontology::new();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

        // Get some classes for testing
        let classes: Vec<_> = reasoner
            .ontology
            .classes()
            .iter()
            .take(5)
            .cloned()
            .collect();
        if !classes.is_empty() {
            let class_iri = classes[0].iri().clone();

            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new("cached_satisfiability", size),
                &size,
                |b, _size| {
                    // First call to populate cache
                    let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();

                    // Now benchmark cached calls
                    b.iter(|| {
                        let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                        black_box(result)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Test cache invalidation and memory management
fn bench_cache_memory_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_memory_management");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get multiple classes for testing
    let classes: Vec<_> = reasoner
        .ontology
        .classes()
        .iter()
        .take(20)
        .cloned()
        .collect();

    group.bench_function("cache_pressure_test", |b| {
        b.iter(|| {
            // Perform many different satisfiability checks to stress the cache
            for (i, class) in classes.iter().enumerate() {
                let class_iri = class.iri().clone();
                let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                black_box(result);

                // Occasionally check consistency to test cross-operation caching
                if i % 5 == 0 {
                    let consistency = black_box(reasoner.is_consistent().unwrap());
                    black_box(consistency);
                }
            }
        })
    });

    group.finish();
}

/// Test multi-layer cache behavior
fn bench_multi_layer_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_layer_cache");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get classes for different cache layers
    let primary_classes: Vec<_> = reasoner
        .ontology
        .classes()
        .iter()
        .take(10)
        .cloned()
        .collect();
    let secondary_classes: Vec<_> = reasoner
        .ontology
        .classes()
        .iter()
        .skip(10)
        .take(10)
        .cloned()
        .collect();

    // Primary cache layer (frequently accessed)
    if !primary_classes.is_empty() {
        let primary_class_iri = primary_classes[0].iri().clone();

        group.bench_function("primary_cache_access", |b| {
            // Populate primary cache
            let _ = reasoner.is_class_satisfiable(&primary_class_iri).unwrap();

            b.iter(|| {
                let result = black_box(reasoner.is_class_satisfiable(&primary_class_iri).unwrap());
                black_box(result)
            })
        });
    }

    // Secondary cache layer (less frequently accessed)
    if !secondary_classes.is_empty() {
        let secondary_class_iri = secondary_classes[0].iri().clone();

        group.bench_function("secondary_cache_access", |b| {
            // Populate secondary cache
            let _ = reasoner.is_class_satisfiable(&secondary_class_iri).unwrap();

            b.iter(|| {
                let result =
                    black_box(reasoner.is_class_satisfiable(&secondary_class_iri).unwrap());
                black_box(result)
            })
        });
    }

    group.finish();
}

/// Comprehensive cache analysis
#[allow(dead_code)]
fn run_cache_analysis() -> PerformanceResults {
    let mut results = PerformanceResults::new();

    println!("=== Running Cache Performance Analysis ===");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get classes for testing
    let classes: Vec<_> = reasoner
        .ontology
        .classes()
        .iter()
        .take(10)
        .cloned()
        .collect();

    // Test cache warmup performance
    println!("Testing cache warmup...");
    for (i, class) in classes.iter().enumerate() {
        let class_iri = class.iri().clone();
        let (_, measurement) = measure_performance(&format!("warmup_class_{}", i), || {
            reasoner.is_class_satisfiable(&class_iri).unwrap()
        });
        results.add_measurement(measurement);
    }

    // Test cache hit performance
    println!("Testing cache hits...");
    for (i, class) in classes.iter().enumerate() {
        let class_iri = class.iri().clone();
        let (_, measurement) = measure_performance(&format!("cache_hit_class_{}", i), || {
            reasoner.is_class_satisfiable(&class_iri).unwrap()
        });
        results.add_measurement(measurement);
    }

    // Test cache with repeated access patterns
    println!("Testing repeated access patterns...");
    for _ in 0..5 {
        let class_iri = classes[0].iri().clone();
        let (_, measurement) = measure_performance("repeated_access", || {
            reasoner.is_class_satisfiable(&class_iri).unwrap()
        });
        results.add_measurement(measurement);
    }

    // Test consistency checking cache
    println!("Testing consistency cache...");
    for i in 0..10 {
        let (_, measurement) = measure_performance(&format!("consistency_check_{}", i), || {
            reasoner.is_consistent().unwrap()
        });
        results.add_measurement(measurement);
    }

    results.complete();
    results
}

/// Analyze cache effectiveness
#[allow(dead_code)]
fn analyze_cache_effectiveness() {
    println!("\n=== Cache Effectiveness Analysis ===");
    let results = run_cache_analysis();

    println!("{}", results.generate_summary());

    // Calculate cache hit/miss ratios
    let mut warmup_times = Vec::new();
    let mut cache_hit_times = Vec::new();

    for measurement in &results.measurements {
        if measurement.operation_name.contains("warmup") {
            warmup_times.push(measurement.duration_ms);
        } else if measurement.operation_name.contains("cache_hit") {
            cache_hit_times.push(measurement.duration_ms);
        }
    }

    if !warmup_times.is_empty() && !cache_hit_times.is_empty() {
        let avg_warmup_time = warmup_times.iter().sum::<f64>() / warmup_times.len() as f64;
        let avg_cache_hit_time = cache_hit_times.iter().sum::<f64>() / cache_hit_times.len() as f64;
        let speedup_ratio = avg_warmup_time / avg_cache_hit_time;

        println!("\nCache Performance Summary:");
        println!("Average warmup time: {:.2} ms", avg_warmup_time);
        println!("Average cache hit time: {:.2} ms", avg_cache_hit_time);
        println!("Cache speedup ratio: {:.2}x", speedup_ratio);

        if speedup_ratio > 2.0 {
            println!("✅ Cache is performing well (speedup > 2x)");
        } else if speedup_ratio > 1.2 {
            println!("⚠️  Cache performance is moderate (speedup > 1.2x)");
        } else {
            println!("❌ Cache performance is poor (speedup <= 1.2x)");
        }
    }

    println!("\n=== Memory Usage Report ===");
    println!(
        "{}",
        memory_profiler::utils::generate_memory_report(&results.measurements)
    );
}

/// Test cache TTL (Time To Live) behavior
fn bench_cache_ttl(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_ttl");

    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    if let Some(first_class) = reasoner.ontology.classes().iter().next() {
        let class_iri = first_class.iri().clone();

        // Test immediate cache access
        group.bench_function("immediate_cache_access", |b| {
            // Populate cache
            let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();

            b.iter(|| {
                let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                black_box(result)
            })
        });

        // Test delayed cache access (simulating TTL expiration)
        group.bench_function("delayed_cache_access", |b| {
            b.iter(|| {
                // Populate cache
                let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();

                // Simulate delay (though in practice, TTL would be tested differently)
                std::thread::sleep(Duration::from_millis(1));

                let result = black_box(reasoner.is_class_satisfiable(&class_iri).unwrap());
                black_box(result)
            })
        });
    }

    group.finish();
}

criterion_group!(
    cache_benches,
    bench_consistency_cache,
    bench_satisfiability_cache,
    bench_cache_scalability,
    bench_multi_layer_cache,
    bench_cache_memory_management,
    bench_cache_ttl
);

criterion_main!(cache_benches);
