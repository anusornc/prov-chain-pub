//! Performance Benchmarks for Memory Safety Features
//!
//! This benchmark suite measures the performance impact of memory safety features
//! and validates that they don't significantly degrade system performance.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use owl2_reasoner::memory::*;
use owl2_reasoner::test_helpers::MemorySafeTestConfig;
use owl2_reasoner::test_memory_guard::MemoryGuard;

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

// /// Benchmark memory guard overhead (disabled - requires complex implementation)
// // fn bench_memory_guard_overhead(c: &mut Criterion) {
//     let mut group = c.benchmark_group("memory_guard_overhead");
//
//     group.bench_function("guard_creation_default", |b| {
//         b.iter(|| {
//             let guard = MemoryGuard::new();
//             black_box(guard)
//         })
//     });
//
//     group.bench_function("guard_creation_custom", |b| {
//         b.iter(|| {
//             let config = MemorySafeTestConfig {
//                 max_memory_mb: 100,
//                 timeout_seconds: 60,
//             };
//             let guard = MemoryGuard::with_config(config);
//             black_box(guard)
//         })
//     });
//
//     group.bench_function("guard_check_memory", |b| {
//         let guard = MemoryGuard::new();
//         guard.start_monitoring();
//
//         b.iter(|| {
//             let _ = black_box(guard.check_memory());
//         })
//     });
//
//     group.bench_function("guard_memory_usage_percent", |b| {
//         let guard = MemoryGuard::new();
//         guard.start_monitoring();
//
//         b.iter(|| black_box(guard.memory_usage_percent()))
//     });
//
//     group.finish();
// }
//
// /// Benchmark memory monitor performance
// fn bench_memory_monitor_performance(c: &mut Criterion) {
//     let mut group = c.benchmark_group("memory_monitor_performance");
//
//     group.bench_function("monitor_creation_default", |b| {
//         b.iter(|| {
//             let config = MemoryMonitorConfig::default();
//             let monitor = MemoryMonitor::new(config);
//             black_box(monitor)
//         })
//     });
//
//     group.bench_function("monitor_get_stats", |b| {
//         let config = MemoryMonitorConfig::default();
//         let monitor = MemoryMonitor::new(config);
//
//         b.iter(|| black_box(monitor.get_stats()))
//     });
//
//     group.bench_function("monitor_check_and_cleanup", |b| {
//         let config = MemoryMonitorConfig {
//             max_memory: 1024 * 1024 * 1024, // 1GB
//             pressure_threshold: 0.8,
//             cleanup_interval: Duration::from_secs(300),
//             auto_cleanup: true,
//         };
//         let monitor = MemoryMonitor::new(config);
//
//         b.iter(|| {
//             let _ = black_box(monitor.check_and_cleanup());
//         })
//     });
//
//     group.finish();
// }
//
// /// Benchmark cache operations with memory safety
// fn bench_cache_with_memory_safety(c: &mut Criterion) {
//     let mut group = c.benchmark_group("cache_with_memory_safety");
//
//     // Setup test data
//     let test_iris: Vec<String> = (0..1000)
//         .map(|i| format!("http://example.org/test/{}", i))
//         .collect();
//
//     group.bench_function("cache_operations_with_guard", |b| {
//         let guard = MemoryGuard::new();
//         guard.start_monitoring();
//
//         b.iter(|| {
//             // Simulate cache operations
//             for iri_str in &test_iris[..100] {
//                 let _ = IRI::new(iri_str.as_str());
//             }
//
//             let _ = guard.check_memory();
//         })
//     });
//
//     group.bench_function("cache_operations_without_guard", |b| {
//         b.iter(|| {
//             // Same operations without memory guard
//             for iri_str in &test_iris[..100] {
//                 let _ = IRI::new(iri_str.as_str());
//             }
//         })
//     });
//
//     group.finish();
// }
//
// /// Benchmark ontology operations with memory safety
// fn bench_ontology_with_memory_safety(c: &mut Criterion) {
//     let mut group = c.benchmark_group("ontology_with_memory_safety");
//
//     let sizes = vec![100, 500, 1000];
//
//     for size in sizes {
//         group.bench_with_input(
//             BenchmarkId::new("ontology_creation_with_guard", size),
//             &size,
//             |b, &size| {
//                 b.iter(|| {
//                     let guard = MemoryGuard::with_config(MemorySafeTestConfig {
//                         max_memory_mb: 200,
//                         timeout_seconds: 120,
//                     });
//                     guard.start_monitoring();
//
//                     let mut ontology = Ontology::new();
//
//                     for i in 0..size {
//                         let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                         let class = Class::new(Arc::new(iri));
//                         let _ = ontology.add_class(class);
//
//                         if i % 100 == 0 {
//                             let _ = guard.check_memory();
//                         }
//                     }
//
//                     black_box(ontology)
//                 })
//             },
//         );
//
//         group.bench_with_input(
//             BenchmarkId::new("ontology_creation_without_guard", size),
//             &size,
//             |b, &size| {
//                 b.iter(|| {
//                     let mut ontology = Ontology::new();
//
//                     for i in 0..size {
//                         let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                         let class = Class::new(Arc::new(iri));
//                         let _ = ontology.add_class(class);
//                     }
//
//                     black_box(ontology)
//                 })
//             },
//         );
//     }
//
//     group.finish();
// }
//
// /// Benchmark memory cleanup performance
// fn bench_memory_cleanup_performance(c: &mut Criterion) {
//     let mut group = c.benchmark_group("memory_cleanup_performance");
//
//     group.bench_function("force_memory_cleanup", |b| {
//         b.iter(|| {
//             let _ = black_box(force_memory_cleanup());
//         })
//     });
//
//     group.bench_function("clear_global_iri_cache", |b| {
//         b.iter(|| {
//             let _ = black_box(clear_global_iri_cache());
//         })
//     });
//
//     group.bench_function("global_cache_stats", |b| {
//         b.iter(|| black_box(global_cache_stats()))
//     });
//
//     group.finish();
// }
//
// /// Benchmark memory leak detection
// fn bench_memory_leak_detection(c: &mut Criterion) {
//     let mut group = c.benchmark_group("memory_leak_detection");
//
//     group.bench_function("leak_detection_baseline", |b| {
//         b.iter(|| black_box(detect_memory_leaks()))
//     });
//
//     group.bench_function("leak_detection_with_allocations", |b| {
//         b.iter(|| {
//             // Create some allocations to detect
//             let _allocation1: Vec<u8> = vec![1; 1024 * 1024]; // 1MB
//             let _allocation2: Vec<u8> = vec![2; 512 * 1024]; // 512KB
//
//             let report = black_box(detect_memory_leaks());
//
//             // Clean up
//             drop(_allocation1);
//             drop(_allocation2);
//
//             report
//         })
//     });
//
//     group.finish();
// }
//
// /// Benchmark concurrent memory operations
// fn bench_concurrent_memory_operations(c: &mut Criterion) {
//     let mut group = c.benchmark_group("concurrent_memory_operations");
//
//     group.bench_function("concurrent_stats_access", |b| {
//         b.iter(|| {
//             // use std::sync::Arc; // Not used
//             use std::thread;
//
//             let handles: Vec<_> = (0..4)
//                 .map(|_| {
//                     thread::spawn(|| {
//                         for _ in 0..100 {
//                             black_box(get_memory_stats());
//                             black_box(get_memory_pressure_level());
//                         }
//                     })
//                 })
//                 .collect();
//
//             for handle in handles {
//                 handle.join().unwrap();
//             }
//         })
//     });
//
//     group.bench_function("concurrent_guard_operations", |b| {
//         b.iter(|| {
//             // use std::sync::Arc; // Not used
//             use std::thread;
//
//             let handles: Vec<_> = (0..4)
//                 .map(|thread_id| {
//                     thread::spawn(move || {
//                         let guard = MemoryGuard::new();
//                         guard.start_monitoring();
//
//                         for i in 0..50 {
//                             let _allocation: Vec<u8> = vec![(thread_id + i) as u8; 1024];
//                             let _ = guard.check_memory();
//                         }
//
//                         let report = guard.stop_monitoring();
//                         black_box(report)
//                     })
//                 })
//                 .collect();
//
//             for handle in handles {
//                 handle.join().unwrap();
//             }
//         })
//     });
//
//     group.finish();
// }
//
// /// Benchmark memory safety overhead in realistic scenarios
// fn bench_realistic_memory_safety_overhead(c: &mut Criterion) {
//     let mut group = c.benchmark_group("realistic_memory_safety_overhead");
//
//     // Simulate a realistic ontology processing scenario
//     group.bench_function("realistic_scenario_with_safety", |b| {
//         b.iter(|| {
//             let guard = MemoryGuard::with_config(MemorySafeTestConfig {
//                 max_memory_mb: 500,
//                 timeout_seconds: 300,
//             });
//             guard.start_monitoring();
//
//             // Simulate ontology loading and processing
//             let mut ontology = Ontology::new();
//
//             // Create classes
//             for i in 0..200 {
//                 let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                 let class = Class::new(Arc::new(iri));
//                 let _ = ontology.add_class(class);
//             }
//
//             // Create properties
//             for i in 0..50 {
//                 let iri = IRI::new(&format!("http://example.org/prop{}", i)).unwrap();
//                 let prop = owl2_reasoner::ObjectProperty::new(Arc::new(iri));
//                 let _ = ontology.add_object_property(prop);
//             }
//
//             // Create subclass relationships
//             for i in 1..200 {
//                 let subclass_iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                 let superclass_iri =
//                     IRI::new(&format!("http://example.org/class{}", i / 2)).unwrap();
//
//                 let subclass = owl2_reasoner::ClassExpression::Class(owl2_reasoner::Class::new(
//                     Arc::new(subclass_iri),
//                 ));
//                 let superclass = owl2_reasoner::ClassExpression::Class(owl2_reasoner::Class::new(
//                     Arc::new(superclass_iri),
//                 ));
//
//                 let axiom = owl2_reasoner::SubClassOfAxiom::new(subclass, superclass);
//                 let _ = ontology.add_subclass_axiom(axiom);
//
//                 if i % 20 == 0 {
//                     let _ = guard.check_memory();
//                 }
//             }
//
//             // Simulate reasoning
//             let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
//             let _ = reasoner.is_consistent();
//
//             let report = guard.stop_monitoring();
//             black_box(report)
//         })
//     });
//
//     group.bench_function("realistic_scenario_without_safety", |b| {
//         b.iter(|| {
//             // Same scenario without memory safety
//             let mut ontology = Ontology::new();
//
//             // Create classes
//             for i in 0..200 {
//                 let iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                 let class = Class::new(Arc::new(iri));
//                 let _ = ontology.add_class(class);
//             }
//
//             // Create properties
//             for i in 0..50 {
//                 let iri = IRI::new(&format!("http://example.org/prop{}", i)).unwrap();
//                 let prop = owl2_reasoner::ObjectProperty::new(Arc::new(iri));
//                 let _ = ontology.add_object_property(prop);
//             }
//
//             // Create subclass relationships
//             for i in 1..200 {
//                 let subclass_iri = IRI::new(&format!("http://example.org/class{}", i)).unwrap();
//                 let superclass_iri =
//                     IRI::new(&format!("http://example.org/class{}", i / 2)).unwrap();
//
//                 let subclass = owl2_reasoner::ClassExpression::Class(owl2_reasoner::Class::new(
//                     Arc::new(subclass_iri),
//                 ));
//                 let superclass = owl2_reasoner::ClassExpression::Class(owl2_reasoner::Class::new(
//                     Arc::new(superclass_iri),
//                 ));
//
//                 let axiom = owl2_reasoner::SubClassOfAxiom::new(subclass, superclass);
//                 let _ = ontology.add_subclass_axiom(axiom);
//             }
//
//             // Simulate reasoning
//             let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
//             let _ = reasoner.is_consistent();
//         })
//     });
//
//     group.finish();
// }
//
// criterion_group!(
//     benches,
//     bench_memory_stats_collection,
//     bench_memory_guard_overhead,
//     bench_memory_monitor_performance,
//     bench_cache_with_memory_safety,
//     bench_ontology_with_memory_safety,
//     bench_memory_cleanup_performance,
//     bench_memory_leak_detection,
//     bench_concurrent_memory_operations,
//     bench_realistic_memory_safety_overhead
// );
//
// criterion_main!(benches);

// Simple working benchmark for MemoryGuard
fn bench_simple_memory_guard(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_memory_guard");

    group.bench_function("guard_creation", |b| {
        b.iter(|| {
            let guard = MemoryGuard::new();
            black_box(guard.is_enabled())
        })
    });

    group.bench_function("guard_with_config", |b| {
        b.iter(|| {
            let config = MemorySafeTestConfig {
                max_memory_mb: 100,
                timeout_seconds: 60,
            };
            let guard = MemoryGuard::with_config(config);
            black_box(guard.is_enabled())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_stats_collection,
    bench_simple_memory_guard
);

criterion_main!(benches);
