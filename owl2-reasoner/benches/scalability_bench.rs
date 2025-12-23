//! Scalability benchmarks
//!
//! ## Performance Optimization Strategy
//!
//! These benchmarks use **adaptive configuration** to prevent hanging on large inputs:
//! - **Sample sizes scale down** as ontology size increases (100 → 50 → 20 → 10)
//! - **Measurement time reduces** for larger ontologies (20s → 15s → 12s → 10s)
//!
//! This ensures benchmarks complete in reasonable time while still providing meaningful
//! performance metrics across different scales.

use criterion::{black_box, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::SimpleReasoner;
use std::time::Duration;

/// Benchmark handling of large ontologies
pub fn bench_large_ontology_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_ontology_handling");

    for size in [1000, 5000, 10000, 20000].iter() {
        // Adaptive configuration based on ontology size
        let (measurement_time, sample_size) = match *size {
            0..=1000 => (Duration::from_secs(20), 100),
            1001..=5000 => (Duration::from_secs(15), 50),
            5001..=10000 => (Duration::from_secs(12), 20),
            _ => (Duration::from_secs(10), 10), // 20000
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        group.bench_with_input(
            BenchmarkId::new("large_ontology_reasoning", size),
            size,
            |b, size| {
                b.iter(|| {
                    let ontology = create_large_ontology(*size);
                    let reasoner = SimpleReasoner::new(ontology);
                    let result = reasoner.is_consistent();
                    let _ = black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark ontology loading performance
pub fn bench_ontology_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_loading");

    for size in [1000, 5000, 10000, 20000].iter() {
        // Adaptive configuration based on ontology size
        let (measurement_time, sample_size) = match *size {
            0..=1000 => (Duration::from_secs(15), 100),
            1001..=5000 => (Duration::from_secs(12), 50),
            5001..=10000 => (Duration::from_secs(10), 20),
            _ => (Duration::from_secs(8), 10), // 20000
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        group.bench_with_input(
            BenchmarkId::new("load_large_ontology", size),
            size,
            |b, size| {
                b.iter(|| {
                    let ontology = create_large_ontology(*size);
                    black_box(ontology);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark deep hierarchy reasoning
pub fn bench_deep_hierarchy_reasoning(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_hierarchy");

    for depth in [10, 50, 100, 200].iter() {
        // Adaptive configuration based on hierarchy depth
        let (measurement_time, sample_size) = match *depth {
            0..=50 => (Duration::from_secs(15), 100),
            51..=100 => (Duration::from_secs(12), 50),
            _ => (Duration::from_secs(10), 30), // 200
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        let ontology = create_deep_hierarchy(*depth);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(
            BenchmarkId::new("deep_consistency", depth),
            depth,
            |b, _| {
                b.iter(|| {
                    let result = reasoner.is_consistent();
                    let _ = black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark wide hierarchy reasoning
pub fn bench_wide_hierarchy_reasoning(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_hierarchy");

    for width in [10, 50, 100, 200].iter() {
        // Adaptive configuration based on hierarchy width
        let (measurement_time, sample_size) = match *width {
            0..=50 => (Duration::from_secs(15), 100),
            51..=100 => (Duration::from_secs(12), 50),
            _ => (Duration::from_secs(10), 30), // 200
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        let ontology = create_wide_hierarchy(*width);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(
            BenchmarkId::new("wide_consistency", width),
            width,
            |b, _| {
                b.iter(|| {
                    let result = reasoner.is_consistent();
                    let _ = black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent access scenarios
pub fn bench_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let mut group = c.benchmark_group("concurrent_access");

    for thread_count in [2, 4, 8].iter() {
        // Adaptive configuration based on thread count
        let (measurement_time, sample_size) = match *thread_count {
            0..=4 => (Duration::from_secs(15), 50),
            _ => (Duration::from_secs(12), 30), // 8
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        group.bench_with_input(
            BenchmarkId::new("concurrent_reasoning", thread_count),
            thread_count,
            |b, thread_count| {
                b.iter(|| {
                    let ontology = create_large_ontology(1000);
                    let arc_ontology = Arc::new(ontology);
                    let mut handles = Vec::new();

                    for _ in 0..*thread_count {
                        let ontology_clone = arc_ontology.clone();
                        let handle = thread::spawn(move || {
                            let reasoner = SimpleReasoner::new((*ontology_clone).clone());
                            let _ = reasoner.is_consistent();
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = handle.join();
                    }

                    black_box(());
                })
            },
        );
    }

    group.finish();
}

/// Helper function to create a large ontology
fn create_large_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create classes
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }

    // Create individuals
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(iri);
        ontology.add_named_individual(individual).unwrap();
    }

    // Create a network of subclass relationships
    for i in 1..classes.len() {
        let parent_idx = (i - 1) % (size / 10 + 1); // Create multiple hierarchies
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i].clone()),
            ClassExpression::Class(classes[parent_idx].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }

    ontology
}

/// Helper function to create a deep hierarchy
fn create_deep_hierarchy(depth: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create classes in a deep hierarchy
    for i in 0..depth {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }

    // Create linear subclass relationships
    for i in 1..classes.len() {
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i].clone()),
            ClassExpression::Class(classes[i - 1].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }

    ontology
}

/// Helper function to create a wide hierarchy
fn create_wide_hierarchy(width: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create root class
    let root_iri = IRI::new("http://example.org/Root").unwrap();
    let root_class = Class::new(root_iri);
    ontology.add_class(root_class.clone()).unwrap();
    classes.push(root_class);

    // Create many direct subclasses
    for i in 0..width {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);

        // Create subclass relationship to root
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i + 1].clone()),
            ClassExpression::Class(classes[0].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }

    ontology
}
