//! Main benchmark runner for OWL2 Reasoner
//!
//! This file runs all benchmarks for the OWL2 reasoning system.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::{ClassExpression, SubClassOfAxiom};
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::parser::OntologyParser;
use owl2_reasoner::reasoning::query::{PatternTerm, QueryEngine, QueryPattern, TriplePattern};
use owl2_reasoner::reasoning::SimpleReasoner;

fn benchmark_suite(c: &mut Criterion) {
    println!("Running OWL2 Reasoner Benchmark Suite...");
    println!("==========================================");

    // Reasoning benchmarks
    bench_consistency_checking(c);
    bench_class_satisfiability(c);
    bench_cache_operations(c);

    // Parser benchmarks
    bench_turtle_parsing(c);

    // Query benchmarks
    bench_query_engine_creation(c);
    bench_simple_queries(c);

    // Memory benchmarks
    bench_ontology_memory_usage(c);

    println!("==========================================");
    println!("Benchmark suite completed!");
}

fn bench_consistency_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_checking");

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(
            BenchmarkId::new("simple_consistency", size),
            size,
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

fn bench_class_satisfiability(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_satisfiability");

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);

        if let Some(first_class) = reasoner.ontology.classes().iter().next() {
            group.bench_with_input(
                BenchmarkId::new("class_satisfiability", size),
                size,
                |b, _| {
                    b.iter(|| {
                        let result = reasoner.is_class_satisfiable(first_class.iri());
                        let _ = black_box(result);
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);
        let reasoner = SimpleReasoner::new(ontology);

        group.bench_with_input(BenchmarkId::new("cache_clear", size), size, |b, _| {
            b.iter(|| {
                let _ = reasoner.clear_caches();
                black_box(());
            })
        });
    }

    group.finish();
}

fn bench_turtle_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("turtle_parsing");

    let small_turtle = r#"
        @prefix : <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        
        :Person a owl:Class .
        :Student a owl:Class ; rdfs:subClassOf :Person .
        :John a :Student .
    "#;

    group.bench_with_input(
        BenchmarkId::new("parse_turtle", "small"),
        &small_turtle,
        |b, content| {
            b.iter(|| {
                let parser = owl2_reasoner::parser::turtle::TurtleParser::new();
                let result = parser.parse_str(black_box(content));
                let _ = black_box(result);
            })
        },
    );

    group.finish();
}

fn bench_query_engine_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_engine_creation");

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);

        group.bench_with_input(BenchmarkId::new("create_engine", size), size, |b, _| {
            b.iter(|| {
                let engine = QueryEngine::new(black_box(ontology.clone()));
                black_box(engine);
            })
        });
    }

    group.finish();
}

fn bench_simple_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_queries");

    for size in [10, 50, 100].iter() {
        let ontology = create_hierarchy_ontology(*size);
        let mut engine = QueryEngine::new(ontology);

        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("s".into()),
            predicate: PatternTerm::Variable("p".into()),
            object: PatternTerm::Variable("o".into()),
        }]);

        group.bench_with_input(BenchmarkId::new("simple_select", size), size, |b, _| {
            b.iter(|| {
                let result = engine.execute_query(black_box(&pattern));
                let _ = black_box(result);
            })
        });
    }

    group.finish();
}

fn bench_ontology_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology_memory");

    for size in [100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_ontology", size),
            size,
            |b, size| {
                b.iter(|| {
                    let ontology = create_memory_intensive_ontology(*size);
                    black_box(ontology);
                })
            },
        );
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

fn create_memory_intensive_ontology(size: usize) -> Ontology {
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
    for i in 0..size * 2 {
        let iri = IRI::new(format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(iri);
        ontology.add_named_individual(individual).unwrap();
    }

    ontology
}

criterion_group!(benches, benchmark_suite);
criterion_main!(benches);
