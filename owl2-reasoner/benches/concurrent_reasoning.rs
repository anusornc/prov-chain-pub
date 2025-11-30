//! Concurrent Reasoning Stress Tests
//!
//! Tests the performance and behavior of multiple reasoning tasks
//! running simultaneously, focusing on thread safety and resource contention.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

// Include our test data generation utilities
mod memory_profiler;
// mod test_data_generator; // Temporarily removed due to missing module

use memory_profiler::{measure_performance, PerformanceResults};
// use test_data_generator::{
//     generate_medium_ontology, generate_ontology_with_size, generate_small_ontology,
// };

/// Test concurrent consistency checking
fn bench_concurrent_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_consistency");

    let thread_counts = vec![1, 2, 4, 8];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_consistency", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                                barrier.wait(); // Synchronize start

                                // Perform consistency check
                                let result = reasoner.is_consistent().unwrap();
                                completed_operations.fetch_add(1, Ordering::Relaxed);
                                black_box(result)
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Test concurrent satisfiability checking
fn bench_concurrent_satisfiability(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_satisfiability");

    let thread_counts = vec![1, 2, 4, 8];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_satisfiability", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                                // Get a class for satisfiability testing
                                if let Some(first_class) = reasoner.ontology.classes().iter().next()
                                {
                                    let class_iri = first_class.iri().clone();

                                    barrier.wait(); // Synchronize start

                                    // Perform satisfiability check
                                    let result = reasoner.is_class_satisfiable(&class_iri).unwrap();
                                    completed_operations.fetch_add(1, Ordering::Relaxed);
                                    black_box(result)
                                } else {
                                    black_box(false)
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Test mixed concurrent operations
fn bench_concurrent_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_mixed");

    let thread_counts = vec![2, 4, 8];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("mixed_operations", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|thread_id| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                                // Different threads perform different operations
                                barrier.wait(); // Synchronize start

                                match thread_id % 3 {
                                    0 => {
                                        // Consistency checking
                                        let result = reasoner.is_consistent().unwrap();
                                        completed_operations.fetch_add(1, Ordering::Relaxed);
                                        black_box(result)
                                    }
                                    1 => {
                                        // Satisfiability checking
                                        if let Some(first_class) =
                                            reasoner.ontology.classes().iter().next()
                                        {
                                            let class_iri = first_class.iri().clone();
                                            let result =
                                                reasoner.is_class_satisfiable(&class_iri).unwrap();
                                            completed_operations.fetch_add(1, Ordering::Relaxed);
                                            black_box(result)
                                        } else {
                                            black_box(false)
                                        }
                                    }
                                    _ => {
                                        // Multiple satisfiability checks
                                        if let Some(first_class) =
                                            reasoner.ontology.classes().iter().next()
                                        {
                                            let class_iri = first_class.iri().clone();
                                            for _ in 0..3 {
                                                let result = reasoner
                                                    .is_class_satisfiable(&class_iri)
                                                    .unwrap();
                                                black_box(result);
                                            }
                                            completed_operations.fetch_add(1, Ordering::Relaxed);
                                        }
                                        black_box(true)
                                    }
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Test resource contention under load
fn bench_resource_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_contention");

    let thread_counts = vec![4, 8, 16];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("contention_test", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    // Create a shared ontology (simulating shared resources)
                    let shared_ontology = Arc::new(owl2_reasoner::Ontology::new());
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let ontology = shared_ontology.clone();
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                let reasoner =
                                    owl2_reasoner::SimpleReasoner::new((*ontology).clone());

                                barrier.wait(); // Synchronize start

                                // Perform reasoning on shared data
                                let result = reasoner.is_consistent().unwrap();
                                completed_operations.fetch_add(1, Ordering::Relaxed);
                                black_box(result)
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Test memory allocation under concurrent load
fn bench_concurrent_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_memory_allocation");

    let thread_counts = vec![2, 4, 8];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("memory_allocation", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                // Each thread creates its own reasoner to test allocation
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                                barrier.wait(); // Synchronize start

                                // Perform classification (memory intensive)
                                reasoner.classify().unwrap();
                                completed_operations.fetch_add(1, Ordering::Relaxed);
                                black_box(())
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Test cache behavior under concurrent access
#[allow(dead_code)]
fn bench_concurrent_cache_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_cache_access");

    let thread_counts = vec![2, 4, 8];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("cache_access", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                // Maintain an isolated reasoner per thread to avoid sharing
                                // non-Send state while still exercising cache-heavy workloads.
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                                let classes: Vec<_> = reasoner
                                    .ontology
                                    .classes()
                                    .iter()
                                    .take(10)
                                    .map(|class| class.iri().clone())
                                    .collect();

                                // Warm the local cache prior to synchronization
                                for class_iri in &classes {
                                    let _ = reasoner.is_class_satisfiable(class_iri).unwrap();
                                }

                                barrier.wait(); // Synchronize start

                                // Perform cached operations
                                for class_iri in &classes {
                                    let result = reasoner.is_class_satisfiable(class_iri).unwrap();
                                    black_box(result);
                                }
                                completed_operations.fetch_add(1, Ordering::Relaxed);
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Stress test with high concurrency
#[allow(dead_code)]
fn bench_high_concurrency_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_concurrency_stress");
    group.measurement_time(Duration::from_secs(30));

    let thread_counts = vec![16, 32];

    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("stress_test", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let barrier = Arc::new(Barrier::new(thread_count));
                    let completed_operations = Arc::new(AtomicUsize::new(0));

                    let start_time = Instant::now();

                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let barrier = barrier.clone();
                            let completed_operations = completed_operations.clone();

                            thread::spawn(move || {
                                // Create smaller ontologies for stress testing
                                let ontology = owl2_reasoner::Ontology::new();
                                let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                                barrier.wait(); // Synchronize start

                                // Perform multiple operations rapidly
                                for _ in 0..5 {
                                    let result = reasoner.is_consistent().unwrap();
                                    black_box(result);
                                }
                                completed_operations.fetch_add(1, Ordering::Relaxed);
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    let duration = start_time.elapsed();
                    let total_operations = completed_operations.load(Ordering::Relaxed);
                    black_box((duration, total_operations))
                })
            },
        );
    }

    group.finish();
}

/// Comprehensive concurrent reasoning analysis
#[allow(dead_code)]
fn run_concurrent_analysis() -> PerformanceResults {
    let mut results = PerformanceResults::new();

    println!("=== Running Concurrent Reasoning Analysis ===");

    // Test different thread counts
    let thread_counts = vec![1, 2, 4, 8, 16];

    for thread_count in thread_counts {
        println!("Testing {} concurrent threads...", thread_count);

        let barrier = Arc::new(Barrier::new(thread_count));
        let start_time = Instant::now();

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let barrier = barrier.clone();

                thread::spawn(move || {
                    let ontology = owl2_reasoner::Ontology::new();
                    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                    barrier.wait(); // Synchronize start

                    // Different operations for different threads
                    match thread_id % 3 {
                        0 => {
                            let (_, measurement) = measure_performance(
                                &format!("consistency_{}", thread_count),
                                || reasoner.is_consistent().unwrap(),
                            );
                            measurement
                        }
                        1 => {
                            if let Some(first_class) = reasoner.ontology.classes().iter().next() {
                                let class_iri = first_class.iri().clone();
                                let (_, measurement) = measure_performance(
                                    &format!("satisfiability_{}", thread_count),
                                    || reasoner.is_class_satisfiable(&class_iri).unwrap(),
                                );
                                measurement
                            } else {
                                // Create a dummy measurement
                                memory_profiler::PerformanceMeasurement {
                                    operation_name: format!("satisfiability_{}", thread_count),
                                    duration_ms: 0.0,
                                    memory_before: Default::default(),
                                    memory_after: Default::default(),
                                    memory_delta: Default::default(),
                                    allocator_before: Default::default(),
                                    allocator_after: Default::default(),
                                    allocator_delta: Default::default(),
                                }
                            }
                        }
                        _ => {
                            let (_, measurement) = measure_performance(
                                &format!("classification_{}", thread_count),
                                || reasoner.classify().unwrap(),
                            );
                            measurement
                        }
                    }
                })
            })
            .collect();

        let mut thread_measurements = Vec::new();
        for handle in handles {
            thread_measurements.push(handle.join().unwrap());
        }

        let total_duration = start_time.elapsed();

        // Add a summary measurement for this thread count
        let summary_measurement = memory_profiler::PerformanceMeasurement {
            operation_name: format!("concurrent_test_{}_threads", thread_count),
            duration_ms: total_duration.as_millis() as f64,
            memory_before: Default::default(),
            memory_after: Default::default(),
            memory_delta: Default::default(),
            allocator_before: Default::default(),
            allocator_after: Default::default(),
            allocator_delta: Default::default(),
        };
        results.add_measurement(summary_measurement);

        // Add individual thread measurements
        for measurement in thread_measurements {
            results.add_measurement(measurement);
        }
    }

    results.complete();
    results
}

/// Analyze concurrent reasoning performance
#[allow(dead_code)]
fn analyze_concurrent_performance() {
    println!("\n=== Concurrent Reasoning Performance Analysis ===");
    let results = run_concurrent_analysis();

    println!("{}", results.generate_summary());

    // Calculate scalability metrics
    let mut concurrent_measurements = Vec::new();
    for measurement in &results.measurements {
        if measurement.operation_name.contains("concurrent_test") {
            concurrent_measurements.push(measurement);
        }
    }

    if concurrent_measurements.len() > 1 {
        println!("\n=== Scalability Analysis ===");

        // Sort by thread count
        concurrent_measurements.sort_by_key(|m| {
            m.operation_name
                .split('_')
                .nth(2)
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0)
        });

        for (i, measurement) in concurrent_measurements.iter().enumerate() {
            if i > 0 {
                let prev_duration = concurrent_measurements[i - 1].duration_ms;
                let current_duration = measurement.duration_ms;
                let scaling_ratio = current_duration / prev_duration;
                let thread_count = measurement.operation_name.split('_').nth(2).unwrap_or("0");

                println!(
                    "Thread {}: {:.2}ms (scaling ratio: {:.2}x)",
                    thread_count, current_duration, scaling_ratio
                );
            }
        }
    }

    println!("\n=== Memory Usage Report ===");
    println!(
        "{}",
        memory_profiler::utils::generate_memory_report(&results.measurements)
    );
}

/// Test thread safety and race conditions
#[allow(dead_code)]
fn bench_thread_safety(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_safety");

    group.bench_function("race_condition_test", |b| {
        b.iter(|| {
            use std::sync::Mutex;

            let results = Arc::new(Mutex::new(Vec::new()));
            let thread_count = 8;
            let barrier = Arc::new(Barrier::new(thread_count));

            let handles: Vec<_> = (0..thread_count)
                .map(|_| {
                    let results = results.clone();
                    let barrier = barrier.clone();

                    thread::spawn(move || {
                        let ontology = owl2_reasoner::Ontology::new();
                        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

                        barrier.wait();

                        // Perform reasoning and store results
                        let result = reasoner.is_consistent().unwrap();
                        let mut results = results.lock().unwrap();
                        results.push(result);
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            let final_results = results.lock().unwrap();
            assert_eq!(final_results.len(), thread_count);
            black_box(final_results.clone())
        })
    });

    group.finish();
}

criterion_group!(
    concurrent_benchmarks,
    bench_concurrent_consistency,
    bench_concurrent_satisfiability,
    bench_concurrent_mixed_operations,
    bench_resource_contention,
    bench_concurrent_memory_allocation
);

criterion_main!(concurrent_benchmarks);
