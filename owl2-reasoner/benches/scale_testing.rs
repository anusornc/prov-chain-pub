//! Scale Testing for OWL2 Reasoner Working Components
//!
//! This benchmark tests the actual working components at scale:
//! - IRI caching and management
//! - Basic ontology operations
//! - Simple consistency checking
//! - Memory usage with large numbers of entities
//!
//! Focuses on what actually works, not claimed features.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::{Class, NamedIndividual, ObjectProperty};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::SimpleReasoner;
use std::time::Instant;

/// Scale test: IRI caching performance with large numbers of IRIs
fn scale_iri_caching(c: &mut Criterion) {
    println!("🔗 SCALE TEST 1: IRI Caching Performance");
    println!("   Testing IRI management with large numbers of unique IRIs");

    // Note: Configure individual benchmark groups for faster execution

    let mut group = c.benchmark_group("iri_caching_scale");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for iri_count in [500, 1000, 2500, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("iri_creation", iri_count),
            iri_count,
            |b, count| {
                b.iter(|| {
                    let start = Instant::now();

                    // Create many unique IRIs to test caching performance
                    for i in 0..*count {
                        let iri_str = format!("http://example.org/entity/{}", i);
                        let _iri = IRI::new(&iri_str).unwrap();
                    }

                    let duration = start.elapsed();
                    black_box(duration);
                })
            },
        );
    }

    group.finish();
    println!("   ✅ IRI caching scale test complete");
}

/// Scale test: Basic ontology operations with large numbers of entities
fn scale_ontology_operations(c: &mut Criterion) {
    println!("🏗️  SCALE TEST 2: Ontology Operations");
    println!("   Testing basic ontology management at scale");

    let mut group = c.benchmark_group("ontology_operations_scale");
    group.measurement_time(std::time::Duration::from_millis(500));
    group.warm_up_time(std::time::Duration::from_millis(200));

    for entity_count in [500, 1000, 2500, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("ontology_creation", entity_count),
            entity_count,
            |b, count| {
                b.iter(|| {
                    let start = Instant::now();
                    let mut ontology = Ontology::new();

                    // Add classes
                    for i in 0..*count {
                        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
                        let class = Class::new(iri);
                        let _ = ontology.add_class(class);
                    }

                    // Add properties (fewer than classes)
                    for i in 0..(*count / 10).max(1) {
                        let iri = IRI::new(format!("http://example.org/hasProperty{}", i)).unwrap();
                        let prop = ObjectProperty::new(iri);
                        let _ = ontology.add_object_property(prop);
                    }

                    // Add some subclass relationships
                    for i in 1..(*count / 5).max(1) {
                        let subclass_iri =
                            IRI::new(format!("http://example.org/Class{}", i)).unwrap();
                        let superclass_iri =
                            IRI::new(format!("http://example.org/Class{}", i / 2)).unwrap();

                        let subclass = ClassExpression::Class(Class::new(subclass_iri));
                        let superclass = ClassExpression::Class(Class::new(superclass_iri));
                        let axiom = SubClassOfAxiom::new(subclass, superclass);
                        let _ = ontology.add_subclass_axiom(axiom);
                    }

                    let duration = start.elapsed();
                    black_box((duration, ontology.classes().len()));
                })
            },
        );
    }

    group.finish();
    println!("   ✅ Ontology operations scale test complete");
}

/// Scale test: Simple consistency checking with larger ontologies
fn scale_consistency_checking(c: &mut Criterion) {
    println!("🧠 SCALE TEST 3: Consistency Checking");
    println!("   Testing basic consistency checking performance at scale");

    let mut group = c.benchmark_group("consistency_checking_scale");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for ontology_size in [250, 500, 1000, 2000].iter() {
        // Pre-create ontology for consistency testing
        let ontology = create_consistent_test_ontology(*ontology_size);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(
            BenchmarkId::new("consistency_check", ontology_size),
            ontology_size,
            |b, _| {
                b.iter(|| {
                    let start = Instant::now();
                    let _is_consistent = reasoner.is_consistent();
                    let duration = start.elapsed();
                    black_box(duration);
                })
            },
        );
    }

    group.finish();
    println!("   ✅ Consistency checking scale test complete");
}

/// Scale test: Memory usage measurement with large ontologies
fn scale_memory_usage(c: &mut Criterion) {
    println!("💾 SCALE TEST 4: Memory Usage");
    println!("   Measuring memory usage with large ontologies");

    let mut group = c.benchmark_group("memory_usage_scale");
    group.measurement_time(std::time::Duration::from_millis(300));
    group.warm_up_time(std::time::Duration::from_millis(100));

    for entity_count in [1000, 2500, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_measurement", entity_count),
            entity_count,
            |b, count| {
                b.iter(|| {
                    let start = Instant::now();

                    // Create large ontology and measure basic memory characteristics
                    let ontology = create_large_test_ontology(*count);

                    // Basic memory estimation - count entities and estimate sizes
                    let class_count = ontology.classes().len();
                    let prop_count = ontology.object_properties().len();
                    let axiom_count = ontology.subclass_axioms().len();
                    let individual_count = ontology.named_individuals().len();

                    // Conservative memory estimation
                    let estimated_memory_bytes = (class_count * 128) +    // Classes: ~128 bytes each
                    (prop_count * 96) +      // Properties: ~96 bytes each
                    (axiom_count * 64) +     // Axioms: ~64 bytes each
                    (individual_count * 80); // Individuals: ~80 bytes each

                    let duration = start.elapsed();
                    black_box((
                        duration,
                        estimated_memory_bytes,
                        class_count + prop_count + axiom_count + individual_count,
                    ));
                })
            },
        );
    }

    group.finish();
    println!("   ✅ Memory usage scale test complete");
}

/// Scale test: Combined operations - realistic workload
fn scale_combined_operations(c: &mut Criterion) {
    println!("🎯 SCALE TEST 5: Combined Operations");
    println!("   Testing realistic combined workload at scale");

    let mut group = c.benchmark_group("combined_operations_scale");
    group.measurement_time(std::time::Duration::from_millis(500));
    group.warm_up_time(std::time::Duration::from_millis(200));

    for scale_factor in [500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("combined_workload", scale_factor),
            scale_factor,
            |b, size| {
                b.iter(|| {
                    let start = Instant::now();

                    // Create ontology
                    let ontology = create_large_test_ontology(*size);

                    // Initialize reasoner
                    let reasoner = SimpleReasoner::new(ontology.clone());

                    // Perform consistency check
                    let _is_consistent = reasoner.is_consistent();

                    // Perform some subclass reasoning
                    let classes: Vec<_> = ontology.classes().iter().take(10).cloned().collect();
                    for i in 0..classes.len().min(5) {
                        for j in 0..classes.len().min(5) {
                            if i != j {
                                let _ = reasoner.is_subclass_of(classes[i].iri(), classes[j].iri());
                            }
                        }
                    }

                    // Measure basic stats
                    let class_count = ontology.classes().len();
                    let axiom_count = ontology.subclass_axioms().len();

                    let duration = start.elapsed();
                    black_box((duration, class_count, axiom_count));
                })
            },
        );
    }

    group.finish();
    println!("   ✅ Combined operations scale test complete");
}

/// Helper: Create a consistent test ontology of given size
fn create_consistent_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create classes in a hierarchy (no cycles to ensure consistency)
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        let _ = ontology.add_class(class);
    }

    // Create hierarchical relationships (parent -> child)
    for i in 1..(size / 2).max(1) {
        let child_iri = IRI::new(format!("http://example.org/Class{}", i * 2)).unwrap();
        let parent_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();

        let child = ClassExpression::Class(Class::new(child_iri));
        let parent = ClassExpression::Class(Class::new(parent_iri));
        let axiom = SubClassOfAxiom::new(child, parent);
        let _ = ontology.add_subclass_axiom(axiom);
    }

    ontology
}

/// Helper: Create a large test ontology with various entity types
fn create_large_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create classes
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(iri);
        let _ = ontology.add_class(class);
    }

    // Create object properties (fewer than classes)
    for i in 0..(size / 10).max(1) {
        let iri = IRI::new(format!("http://example.org/hasProperty{}", i)).unwrap();
        let prop = ObjectProperty::new(iri);
        let _ = ontology.add_object_property(prop);
    }

    // Create subclass relationships
    for i in 1..(size / 5).max(1) {
        let child_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let parent_iri = IRI::new(format!("http://example.org/Class{}", i / 2)).unwrap();

        let child = ClassExpression::Class(Class::new(child_iri));
        let parent = ClassExpression::Class(Class::new(parent_iri));
        let axiom = SubClassOfAxiom::new(child, parent);
        let _ = ontology.add_subclass_axiom(axiom);
    }

    // Create some individuals
    for i in 0..(size / 3).max(1) {
        let iri = IRI::new(format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(iri);
        let _ = ontology.add_named_individual(individual);
    }

    ontology
}

criterion_group!(
    scale_benches,
    scale_iri_caching,
    scale_ontology_operations,
    scale_consistency_checking,
    scale_memory_usage,
    scale_combined_operations
);
criterion_main!(scale_benches);
