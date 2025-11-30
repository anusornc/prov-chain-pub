//! Basic Performance Benchmark Suite
//!
//! This benchmark suite provides basic performance measurements for the OWL2 Reasoner.
//!
//! ## Purpose
//!
//! This is a general performance benchmark that measures basic operations:
//!
//! 1. **Response time measurement** - Basic operation timing
//! 2. **Memory usage estimation** - Basic entity size tracking
//! 3. **Cache effectiveness** - Basic hit rate measurement
//! 4. **Arc sharing analysis** - Basic deduplication tracking
//!
//! ## Methodology
//!
//! Uses simplified measurements for basic performance tracking and estimation.
//! Results are estimates and may vary based on actual implementation details.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::{Class, NamedIndividual, ObjectProperty};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::SimpleReasoner;
// TODO: Replace with alternative size calculation approach
use std::time::Instant;

/// Main benchmark suite that tests basic performance
fn performance_validation_suite(c: &mut Criterion) {
    println!("📊 OWL2 Reasoner Basic Performance Benchmark");
    println!("==========================================");
    println!("📈 Measuring Basic Performance Metrics");
    println!();

    // Note: Configure individual benchmark groups for faster execution

    // Measure basic performance metrics
    measure_response_times(c);
    measure_memory_usage(c);
    measure_cache_effectiveness(c);
    measure_arc_sharing(c);

    // Run comprehensive benchmark
    comprehensive_performance_benchmark(c);

    println!("==========================================");
    println!("✅ Performance Benchmark Complete!");
}

/// Measure basic response times for reasoning operations
fn measure_response_times(c: &mut Criterion) {
    println!("⏱️  MEASUREMENT 1: Response Times");
    println!("   Measuring basic operation timing");

    let mut group = c.benchmark_group("response_time_measurement");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for size in [10, 50, 100, 250].iter() {
        let ontology = create_test_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(BenchmarkId::new("response_time", size), size, |b, _| {
            b.iter(|| {
                let start = Instant::now();

                // Perform basic reasoning operations
                let _consistency = reasoner.is_consistent();
                let _ = reasoner.get_cache_stats();

                // Test subclass reasoning for a few classes
                let classes: Vec<_> = reasoner
                    .ontology
                    .classes()
                    .iter()
                    .take(5)
                    .cloned()
                    .collect();
                for i in 0..classes.len().min(3) {
                    for j in 0..classes.len().min(3) {
                        if i != j {
                            let _ = reasoner.is_subclass_of(classes[i].iri(), classes[j].iri());
                        }
                    }
                }

                let duration = start.elapsed();
                let duration_ms = duration.as_nanos() as f64 / 1_000_000.0;

                // Record measurement (no assertions - just measurement)
                black_box(duration_ms);
            })
        });
    }

    group.finish();
    println!("   ✅ Response time measurement complete");
}

/// Measure basic memory usage estimation
fn measure_memory_usage(c: &mut Criterion) {
    println!("💾 MEASUREMENT 2: Memory Usage");
    println!("   Measuring basic entity size estimation");

    let mut group = c.benchmark_group("memory_usage_measurement");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for size in [10, 50, 100, 250].iter() {
        group.bench_with_input(BenchmarkId::new("entity_memory", size), size, |b, size| {
            b.iter(|| {
                let ontology = create_test_ontology(*size);

                // Use EntitySizeCalculator for basic estimation
                let mut total_entity_bytes = 0;
                let mut entity_count = 0;

                // Estimate class sizes
                for class in ontology.classes() {
                    total_entity_bytes += std::mem::size_of_val(class);
                    entity_count += 1;
                }

                // Estimate property sizes
                for prop in ontology.object_properties() {
                    total_entity_bytes += std::mem::size_of_val(prop);
                    entity_count += 1;
                }

                // Estimate axiom sizes
                for axiom in ontology.subclass_axioms() {
                    total_entity_bytes += std::mem::size_of_val(axiom);
                    entity_count += 1;
                }

                let memory_per_entity_bytes = if entity_count > 0 {
                    total_entity_bytes / entity_count
                } else {
                    0
                };

                let memory_per_entity_kb = memory_per_entity_bytes as f64 / 1024.0;

                // Record measurement (no assertions - just estimation)
                black_box(memory_per_entity_kb);
            })
        });
    }

    group.finish();
    println!("   ✅ Memory usage measurement complete");
}

/// Measure basic cache effectiveness
fn measure_cache_effectiveness(c: &mut Criterion) {
    println!("🎯 MEASUREMENT 3: Cache Effectiveness");
    println!("   Measuring basic cache hit rates");

    let mut group = c.benchmark_group("cache_effectiveness_measurement");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for size in [10, 50, 100].iter() {
        let ontology = create_test_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology.clone());

        // Warm up caches first
        let _ = reasoner.warm_up_caches();

        group.bench_with_input(BenchmarkId::new("cache_efficiency", size), size, |b, _| {
            b.iter(|| {
                // Reset cache stats for clean measurement
                let _ = reasoner.reset_cache_stats();

                // Perform operations that use cache
                for _ in 0..10 {
                    let _ = reasoner.is_consistent();

                    let classes: Vec<_> = reasoner
                        .ontology
                        .classes()
                        .iter()
                        .take(5)
                        .cloned()
                        .collect();
                    for i in 0..classes.len().min(3) {
                        for j in 0..classes.len().min(3) {
                            if i != j {
                                let _ = reasoner.is_subclass_of(classes[i].iri(), classes[j].iri());
                            }
                        }
                    }

                    for class in classes.iter().take(3) {
                        let _ = reasoner.is_class_satisfiable(class.iri());
                    }
                }

                let hit_rate = reasoner
                    .get_cache_stats()
                    .map(|s| s.hit_rate())
                    .unwrap_or(0.0);

                // Record measurement (no assertions - just measurement)
                black_box(hit_rate);
            })
        });
    }

    group.finish();
    println!("   ✅ Cache effectiveness measurement complete");
}

/// Measure basic Arc sharing analysis
fn measure_arc_sharing(c: &mut Criterion) {
    println!("🔗 MEASUREMENT 4: Arc Sharing Analysis");
    println!("   Measuring basic deduplication metrics");

    let mut group = c.benchmark_group("arc_sharing_measurement");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("arc_sharing", size), size, |b, _| {
            b.iter(|| {
                let ontology = create_test_ontology(*size);

                // Analyze basic IRI deduplication
                use std::collections::HashMap;
                let mut iri_references = HashMap::new();

                // Count class IRI references
                for class in ontology.classes() {
                    let iri_str = class.iri().as_str();
                    *iri_references.entry(iri_str).or_insert(0) += 1;
                }

                // Count property IRI references
                for prop in ontology.object_properties() {
                    let iri_str = prop.iri().as_str();
                    *iri_references.entry(iri_str).or_insert(0) += 1;
                }

                // Calculate basic sharing ratio
                let total_references: usize = iri_references.values().sum();
                let shared_references: usize = iri_references
                    .values()
                    .filter(|&&count| count > 1)
                    .map(|&count| count - 1)
                    .sum();

                let sharing_ratio = if total_references > 0 {
                    shared_references as f64 / total_references as f64
                } else {
                    0.0
                };

                // Record measurement (no assertions - just measurement)
                black_box(sharing_ratio);
            })
        });
    }

    group.finish();
    println!("   ✅ Arc sharing measurement complete");
}

/// Comprehensive benchmark: All measurements in one integrated test
fn comprehensive_performance_benchmark(c: &mut Criterion) {
    println!("🎯 COMPREHENSIVE BENCHMARK: All Performance Metrics");

    let mut group = c.benchmark_group("comprehensive_benchmark");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for size in [25, 75, 150].iter() {
        group.bench_with_input(BenchmarkId::new("full_benchmark", size), size, |b, _| {
            b.iter(|| {
                let start_time = Instant::now();

                // Create test ontology
                let ontology = create_test_ontology(*size);
                let reasoner = SimpleReasoner::new(ontology.clone());

                // MEASUREMENT 1: Response time
                let response_start = Instant::now();
                let _consistency = reasoner.is_consistent();
                let _ = reasoner.get_cache_stats();
                let response_time = response_start.elapsed().as_nanos() as f64 / 1_000_000.0;

                // MEASUREMENT 2: Memory usage estimation
                let mut total_bytes = 0;
                let mut count = 0;

                for class in reasoner.ontology.classes() {
                    total_bytes += std::mem::size_of_val(class);
                    count += 1;
                }

                let memory_per_entity_kb = (total_bytes / count.max(1)) as f64 / 1024.0;

                // MEASUREMENT 3: Cache effectiveness
                let _ = reasoner.reset_cache_stats();
                let _result = reasoner.warm_up_caches();

                // Perform cache operations
                for _ in 0..5 {
                    let _ = reasoner.is_consistent();
                    let classes: Vec<_> = reasoner
                        .ontology
                        .classes()
                        .iter()
                        .take(3)
                        .cloned()
                        .collect();
                    for class in classes.iter().take(2) {
                        let _ = reasoner.is_class_satisfiable(class.iri());
                    }
                }

                let hit_rate = reasoner
                    .get_cache_stats()
                    .map(|s| s.hit_rate())
                    .unwrap_or(0.0);

                // MEASUREMENT 4: Basic Arc sharing analysis
                use std::collections::HashMap;
                let mut iri_references = HashMap::new();
                for class in reasoner.ontology.classes() {
                    let iri_str = class.iri().as_str();
                    *iri_references.entry(iri_str).or_insert(0) += 1;
                }

                let total_references: usize = iri_references.values().sum();
                let shared_references: usize = iri_references
                    .values()
                    .filter(|&&count| count > 1)
                    .map(|&count| count - 1)
                    .sum();
                let sharing_ratio = if total_references > 0 {
                    shared_references as f64 / total_references as f64
                } else {
                    0.0
                };

                let total_time = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;

                // Record all measurements
                black_box((
                    response_time,
                    memory_per_entity_kb,
                    hit_rate,
                    sharing_ratio,
                    total_time,
                ));
            })
        });
    }

    group.finish();
    println!("   ✅ Comprehensive benchmark complete - ALL METRICS RECORDED!");
}

/// Helper function to create a standardized test ontology
fn create_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create classes with shared IRIs where possible
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }

    // Create object properties
    for i in 0..(size / 5).max(1) {
        let iri = IRI::new(format!("http://example.org/hasProperty{}", i)).unwrap();
        let prop = ObjectProperty::new(iri);
        ontology.add_object_property(prop).unwrap();
    }

    // Create subclass relationships to enable reasoning tests
    for i in 1..classes.len().min(size) {
        let parent_idx = (i - 1) / 3; // Create reasonable hierarchy
        if parent_idx < classes.len() {
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(classes[i].clone()),
                ClassExpression::Class(classes[parent_idx].clone()),
            );
            ontology.add_subclass_axiom(subclass_axiom).unwrap();
        }
    }

    // Add some individuals for completeness
    for i in 0..(size / 2) {
        let iri = IRI::new(format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(iri);
        ontology.add_named_individual(individual).unwrap();
    }

    ontology
}

criterion_group!(benches, performance_validation_suite);
criterion_main!(benches);
