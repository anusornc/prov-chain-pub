//! Simple benchmark runner for OWL2 Reasoner
//!
//! This file runs basic benchmarks for the OWL2 reasoning system.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::Class;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::SimpleReasoner;

fn benchmark_suite(c: &mut Criterion) {
    println!("Running OWL2 Reasoner Benchmark Suite...");
    println!("==========================================");

    // Basic reasoning benchmarks
    bench_consistency_checking(c);
    bench_ontology_creation(c);
    bench_class_operations(c);

    println!("==========================================");
    println!("Benchmark suite completed!");
}

fn bench_consistency_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_checking");
    group.measurement_time(std::time::Duration::from_millis(500));
    group.warm_up_time(std::time::Duration::from_millis(200));

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(BenchmarkId::new("consistency", size), size, |b, _| {
            b.iter(|| {
                let result = reasoner.is_consistent();
                let _ = black_box(result); // Handle the Result properly
            })
        });
    }

    group.finish();
}

fn bench_ontology_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_creation");
    group.measurement_time(std::time::Duration::from_millis(500));
    group.warm_up_time(std::time::Duration::from_millis(200));

    for size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_ontology", size),
            size,
            |b, size| {
                b.iter(|| {
                    let ontology = create_hierarchy_ontology(*size);
                    black_box(ontology);
                })
            },
        );
    }

    group.finish();
}

fn bench_class_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_operations");
    group.measurement_time(std::time::Duration::from_millis(500));
    group.warm_up_time(std::time::Duration::from_millis(200));

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("add_classes", size), size, |b, size| {
            b.iter(|| {
                let mut ontology = Ontology::new();
                for i in 0..*size {
                    let iri = IRI::new(format!("http://example.org/class{}", i)).unwrap();
                    let class = Class::new(iri);
                    let _ = ontology.add_class(class);
                }
                black_box(ontology);
            })
        });
    }

    group.finish();
}

fn create_hierarchy_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut classes = Vec::new();

    // Create classes
    for i in 0..size {
        let iri = IRI::new(format!("http://example.org/class{}", i)).unwrap();
        let class = Class::new(iri);
        ontology.add_class(class.clone()).unwrap();
        classes.push(class);
    }

    // Create hierarchical relationships
    for i in 1..classes.len().min(size) {
        let parent_idx = (i - 1) / 2;
        let subclass_axiom = SubClassOfAxiom::new(
            ClassExpression::Class(classes[i].clone()),
            ClassExpression::Class(classes[parent_idx].clone()),
        );
        ontology.add_subclass_axiom(subclass_axiom).unwrap();
    }

    ontology
}

criterion_group!(benches, benchmark_suite);
criterion_main!(benches);
