//! Parser performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::parser::turtle::TurtleParser;
use owl2_reasoner::parser::OntologyParser;

/// Benchmark Turtle parsing performance
pub fn bench_turtle_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("turtle_parsing");

    // Small ontology
    let small_turtle = r#"
        @prefix : <http://example.org/> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .
        
        :Person a owl:Class .
        :Student a owl:Class ;
            rdfs:subClassOf :Person .
        :Professor a owl:Class ;
            rdfs:subClassOf :Person .
        
        :John a :Student .
        :Mary a :Professor .
    "#;

    // Medium ontology
    let medium_turtle = generate_medium_turtle();

    // Large ontology
    let large_turtle = generate_large_turtle();

    let test_cases = vec![
        ("small", small_turtle),
        ("medium", &medium_turtle),
        ("large", &large_turtle),
    ];

    for (name, content) in test_cases {
        let parser = TurtleParser::new();
        group.bench_with_input(
            BenchmarkId::new("parse_turtle", name),
            &content,
            |b, content| {
                b.iter(|| {
                    let result = parser.parse_str(black_box(content));
                    let _ = black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Helper function to generate medium-sized Turtle content
fn generate_medium_turtle() -> String {
    let mut content = String::new();
    content.push_str("@prefix : <http://example.org/> .\n");
    content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\n");

    // Create 50 classes
    for i in 0..50 {
        content.push_str(&format!(":Class{} a owl:Class .\n", i));
    }

    // Create hierarchical relationships
    for i in 1..50 {
        let parent = (i - 1) / 2;
        content.push_str(&format!(":Class{} rdfs:subClassOf :Class{} .\n", i, parent));
    }

    // Create 100 individuals
    for i in 0..100 {
        let class = i % 50;
        content.push_str(&format!(":Individual{} a :Class{} .\n", i, class));
    }

    content
}

/// Helper function to generate large Turtle content
fn generate_large_turtle() -> String {
    let mut content = String::new();
    content.push_str("@prefix : <http://example.org/> .\n");
    content.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\n");

    // Create 500 classes
    for i in 0..500 {
        content.push_str(&format!(":Class{} a owl:Class .\n", i));
    }

    // Create hierarchical relationships
    for i in 1..500 {
        let parent = (i - 1) / 2;
        content.push_str(&format!(":Class{} rdfs:subClassOf :Class{} .\n", i, parent));
    }

    // Create 1000 individuals
    for i in 0..1000 {
        let class = i % 500;
        content.push_str(&format!(":Individual{} a :Class{} .\n", i, class));
    }

    content
}

criterion_group!(benches, bench_turtle_parsing);
criterion_main!(benches);
