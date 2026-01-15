//! Query performance benchmarks

use criterion::{black_box, BenchmarkId, Criterion};
use owl2_reasoner::entities::{Class, NamedIndividual};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::query::{PatternTerm, QueryEngine, QueryPattern, TriplePattern};

/// Benchmark query engine creation
pub fn bench_query_engine_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_engine_creation");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_query_ontology(*size);

        group.bench_with_input(BenchmarkId::new("create_engine", size), size, |b, _| {
            b.iter(|| {
                let engine = QueryEngine::new(black_box(ontology.clone()));
                black_box(engine);
            })
        });
    }

    group.finish();
}

/// Benchmark simple SPARQL queries
pub fn bench_simple_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_queries");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_query_ontology(*size);
        let engine = QueryEngine::new(ontology);
        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("s".into()),
            predicate: PatternTerm::Variable("p".into()),
            object: PatternTerm::Variable("o".into()),
        }]);

        group.bench_with_input(BenchmarkId::new("simple_select", size), size, |b, _| {
            b.iter(|| {
                let result = engine.execute(black_box(&pattern));
                let _ = black_box(result);
            })
        });
    }

    group.finish();
}

/// Benchmark class type queries
pub fn bench_class_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_queries");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_query_ontology(*size);
        let engine = QueryEngine::new(ontology);
        let rdf_type = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let class0 = IRI::new("http://example.org/Class0").unwrap();
        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("s".into()),
            predicate: PatternTerm::IRI(rdf_type),
            object: PatternTerm::IRI(class0),
        }]);

        group.bench_with_input(BenchmarkId::new("class_type_query", size), size, |b, _| {
            b.iter(|| {
                let result = engine.execute(black_box(&pattern));
                let _ = black_box(result);
            })
        });
    }

    group.finish();
}

/// Benchmark subclass queries
pub fn bench_subclass_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("subclass_queries");

    for size in [10, 50, 100, 500].iter() {
        let ontology = create_query_ontology(*size);
        let engine = QueryEngine::new(ontology);
        let rdfs_subclass = IRI::new("http://www.w3.org/2000/01/rdf-schema#subClassOf").unwrap();
        let class0 = IRI::new("http://example.org/Class0").unwrap();
        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("s".into()),
            predicate: PatternTerm::IRI(rdfs_subclass),
            object: PatternTerm::IRI(class0),
        }]);

        group.bench_with_input(BenchmarkId::new("subclass_query", size), size, |b, _| {
            b.iter(|| {
                let result = engine.execute(black_box(&pattern));
                let _ = black_box(result);
            })
        });
    }

    group.finish();
}

/// Helper function to create an ontology for query benchmarking
fn create_query_ontology(size: usize) -> Ontology {
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
