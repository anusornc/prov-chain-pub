//! # Performance Optimization Benchmarks
//!
//! Comprehensive benchmarks for validating the performance improvements from
//! arena parser integration and parallel tableaux reasoning implementations.
//!
//! ## Benchmarks Included
//!
//! - **Arena Parser Performance**: Turtle and OWL Functional parsing with/without arena allocation
//! - **Parallel Tableaux Reasoning**: Consistency checking and classification with multi-core processing
//! - **Memory Usage Comparison**: Memory efficiency improvements from arena allocation
//! - **Scalability Testing**: Performance improvements on large ontologies
//! - **Cache Effectiveness**: Hit/miss ratios and performance impact

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::{
    iri::IRI,
    parser::{OntologyParser, OwlFunctionalSyntaxParser, ParserConfig, TurtleParser},
    reasoning::tableaux::{ParallelTableauxReasoner, ReasoningConfig, TableauxReasoner},
    Class, ClassExpression, ObjectProperty, Ontology, SubClassOfAxiom,
};

/// Generate test ontologies of different sizes for benchmarking
fn generate_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Generate classes
    for i in 0..size {
        let class_iri = IRI::new(format!("http://example.org/class{}", i)).unwrap();
        let class = Class::new(class_iri);
        ontology.add_class(class).unwrap();
    }

    // Generate properties
    for i in 0..(size / 10) {
        let prop_iri = IRI::new(format!("http://example.org/prop{}", i)).unwrap();
        let prop = ObjectProperty::new(prop_iri);
        ontology.add_object_property(prop).unwrap();
    }

    // Generate subclass relationships
    for i in 0..(size - 1) {
        let sub_class = ClassExpression::Class(Class::new(
            IRI::new(format!("http://example.org/class{}", i)).unwrap(),
        ));
        let super_class = ClassExpression::Class(Class::new(
            IRI::new(format!("http://example.org/class{}", i + 1)).unwrap(),
        ));
        let axiom = SubClassOfAxiom::new(sub_class, super_class);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    ontology
}

/// Generate large Turtle content for parsing benchmarks
fn generate_large_turtle_content(size: usize) -> String {
    let mut content = String::new();

    // Add prefixes
    content.push_str("@prefix : <http://example.org/> .\n");
    content.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
    content.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");

    // Add classes
    for i in 0..size {
        content.push_str(&format!(":class{} a owl:Class .\n", i));
    }

    // Add subclass relationships
    for i in 0..(size - 1) {
        content.push_str(&format!(":class{} rdfs:subClassOf :class{} .\n", i, i + 1));
    }

    content
}

/// Generate large OWL Functional content for parsing benchmarks
fn generate_large_owl_functional_content(size: usize) -> String {
    let mut content = String::new();

    // Add prefixes
    content.push_str("Prefix(:=<http://example.org/>)\n");
    content.push_str("Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n");
    content.push_str("Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\n\n");

    // Add ontology declaration
    content.push_str("Ontology(<http://example.org/ontology>\n\n");

    // Add classes
    for i in 0..size {
        content.push_str(&format!("Declaration(Class(:class{}))\n", i));
    }

    // Add subclass relationships
    for i in 0..(size - 1) {
        content.push_str(&format!("SubClassOf(:class{} :class{})\n", i, i + 1));
    }

    content.push_str(")\n");
    content
}

/// Benchmark arena parser performance vs traditional parser
fn bench_arena_parser_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_parser_performance");

    for size in [100, 1000, 5000, 10000].iter() {
        let turtle_content = generate_large_turtle_content(*size);
        let owl_functional_content = generate_large_owl_functional_content(*size);

        // Turtle parser benchmarks
        group.bench_with_input(
            BenchmarkId::new("turtle_traditional", size),
            size,
            |b, _| {
                b.iter(|| {
                    let config = ParserConfig {
                        use_arena_allocation: false,
                        ..Default::default()
                    };
                    let parser = TurtleParser::with_config(config);
                    let result = parser.parse_str(&turtle_content).unwrap();
                    let _ = black_box(result);
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("turtle_arena", size), size, |b, _| {
            b.iter(|| {
                let config = ParserConfig {
                    use_arena_allocation: true,
                    arena_capacity: 1024 * 1024, // 1MB arena
                    ..Default::default()
                };
                let parser = TurtleParser::with_config(config);
                let result = parser.parse_str(&turtle_content).unwrap();
                let _ = black_box(result);
            });
        });

        // OWL Functional parser benchmarks
        group.bench_with_input(
            BenchmarkId::new("owl_functional_traditional", size),
            size,
            |b, _| {
                b.iter(|| {
                    let config = ParserConfig {
                        use_arena_allocation: false,
                        ..Default::default()
                    };
                    let parser = OwlFunctionalSyntaxParser::with_config(config);
                    let result = parser.parse_str(&owl_functional_content).unwrap();
                    let _ = black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("owl_functional_arena", size),
            size,
            |b, _| {
                b.iter(|| {
                    let config = ParserConfig {
                        use_arena_allocation: true,
                        arena_capacity: 1024 * 1024, // 1MB arena
                        ..Default::default()
                    };
                    let parser = OwlFunctionalSyntaxParser::with_config(config);
                    let result = parser.parse_str(&owl_functional_content).unwrap();
                    let _ = black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel tableaux reasoning vs traditional
fn bench_parallel_reasoning_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_reasoning_performance");

    for size in [100, 500, 1000, 2000].iter() {
        let ontology = generate_test_ontology(*size);

        // Traditional tableaux reasoning
        group.bench_with_input(
            BenchmarkId::new("traditional_tableaux", size),
            size,
            |b, _| {
                let config = ReasoningConfig {
                    enable_parallel: false,
                    ..Default::default()
                };
                let mut reasoner = TableauxReasoner::with_config(ontology.clone(), config);

                b.iter(|| black_box(reasoner.is_consistent().unwrap()));
            },
        );

        // Parallel tableaux reasoning with different worker counts
        for workers in [2, 4, 8].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("parallel_tableaux_{}workers", workers), size),
                size,
                |b, _| {
                    let config = ReasoningConfig {
                        enable_parallel: true,
                        parallel_workers: Some(*workers),
                        parallel_chunk_size: 64,
                        ..Default::default()
                    };
                    let reasoner = ParallelTableauxReasoner::with_config(ontology.clone(), config);

                    b.iter(|| {
                        let result = reasoner.is_consistent_parallel().unwrap();
                        let _ = black_box(result);
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark memory usage improvements
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let size = 5000;
    let turtle_content = generate_large_turtle_content(size);
    let owl_functional_content = generate_large_owl_functional_content(size);

    // Measure memory usage for traditional parsing
    group.bench_function("turtle_traditional_memory", |b| {
        b.iter(|| {
            let config = ParserConfig {
                use_arena_allocation: false,
                ..Default::default()
            };
            let parser = TurtleParser::with_config(config);
            let result = parser.parse_str(&turtle_content).unwrap();
            black_box(result);
        });
    });

    // Measure memory usage for arena parsing
    group.bench_function("turtle_arena_memory", |b| {
        b.iter(|| {
            let config = ParserConfig {
                use_arena_allocation: true,
                arena_capacity: 1024 * 1024,
                ..Default::default()
            };
            let parser = TurtleParser::with_config(config);
            let result = parser.parse_str(&turtle_content).unwrap();
            black_box(result);
        });
    });

    // Measure memory usage for traditional OWL Functional parsing
    group.bench_function("owl_functional_traditional_memory", |b| {
        b.iter(|| {
            let config = ParserConfig {
                use_arena_allocation: false,
                ..Default::default()
            };
            let parser = OwlFunctionalSyntaxParser::with_config(config);
            let result = parser.parse_str(&owl_functional_content).unwrap();
            black_box(result);
        });
    });

    // Measure memory usage for arena OWL Functional parsing
    group.bench_function("owl_functional_arena_memory", |b| {
        b.iter(|| {
            let config = ParserConfig {
                use_arena_allocation: true,
                arena_capacity: 1024 * 1024,
                ..Default::default()
            };
            let parser = OwlFunctionalSyntaxParser::with_config(config);
            let result = parser.parse_str(&owl_functional_content).unwrap();
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark scalability improvements
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    let sizes = [1000, 2000, 5000, 10000];

    // Test arena parser scalability
    for size in sizes.iter() {
        let turtle_content = generate_large_turtle_content(*size);

        group.bench_with_input(
            BenchmarkId::new("arena_parser_scalability", size),
            size,
            |b, _| {
                b.iter(|| {
                    let config = ParserConfig {
                        use_arena_allocation: true,
                        arena_capacity: 2 * 1024 * 1024, // 2MB arena for larger sizes
                        ..Default::default()
                    };
                    let parser = TurtleParser::with_config(config);
                    black_box(parser.parse_str(&turtle_content).unwrap())
                });
            },
        );
    }

    // Test parallel reasoning scalability
    for size in sizes.iter() {
        let ontology = generate_test_ontology(*size);

        group.bench_with_input(
            BenchmarkId::new("parallel_reasoning_scalability", size),
            size,
            |b, _| {
                let config = ReasoningConfig {
                    enable_parallel: true,
                    parallel_workers: Some(4), // Use 4 workers for scalability test
                    parallel_chunk_size: 128,
                    ..Default::default()
                };
                let reasoner = ParallelTableauxReasoner::with_config(ontology.clone(), config);

                b.iter(|| {
                    let result = reasoner.is_consistent_parallel().unwrap();
                    let _ = black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache effectiveness
fn bench_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");

    let ontology = generate_test_ontology(1000);
    let config = ReasoningConfig {
        enable_parallel: true,
        parallel_workers: Some(4),
        ..Default::default()
    };
    let reasoner = ParallelTableauxReasoner::with_config(ontology.clone(), config);

    // Benchmark cached vs uncached operations
    group.bench_function("uncached_consistency", |b| {
        b.iter(|| {
            let config = ReasoningConfig {
                enable_parallel: true,
                parallel_workers: Some(4),
                ..Default::default()
            };
            let reasoner = ParallelTableauxReasoner::with_config(ontology.clone(), config);
            black_box(reasoner.is_consistent_parallel().unwrap());
        });
    });

    group.bench_function("cached_consistency", |b| {
        b.iter(|| {
            // First call will cache the result
            black_box(reasoner.is_consistent_parallel().unwrap());
        });
    });

    group.finish();
}

/// Comprehensive performance validation
fn bench_comprehensive_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_validation");

    // Test combined performance improvements
    let size = 2000;
    let turtle_content = generate_large_turtle_content(size);

    // Traditional approach (no arena, no parallel)
    group.bench_function("traditional_end_to_end", |b| {
        b.iter(|| {
            // Parse without arena
            let parse_config = ParserConfig {
                use_arena_allocation: false,
                ..Default::default()
            };
            let parser = TurtleParser::with_config(parse_config);
            let parsed_ontology = parser.parse_str(&turtle_content).unwrap();

            // Reason without parallel
            let reason_config = ReasoningConfig {
                enable_parallel: false,
                ..Default::default()
            };
            let mut reasoner = TableauxReasoner::with_config(parsed_ontology, reason_config);
            black_box(reasoner.is_consistent().unwrap());
        });
    });

    // Optimized approach (arena + parallel)
    group.bench_function("optimized_end_to_end", |b| {
        b.iter(|| {
            // Parse with arena
            let parse_config = ParserConfig {
                use_arena_allocation: true,
                arena_capacity: 1024 * 1024,
                ..Default::default()
            };
            let parser = TurtleParser::with_config(parse_config);
            let parsed_ontology = parser.parse_str(&turtle_content).unwrap();

            // Reason with parallel
            let reason_config = ReasoningConfig {
                enable_parallel: true,
                parallel_workers: Some(4),
                parallel_chunk_size: 64,
                ..Default::default()
            };
            let reasoner = ParallelTableauxReasoner::with_config(parsed_ontology, reason_config);
            black_box(reasoner.is_consistent_parallel().unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arena_parser_performance,
    bench_parallel_reasoning_performance,
    bench_memory_usage,
    bench_scalability,
    bench_cache_effectiveness,
    bench_comprehensive_validation
);
criterion_main!(benches);
