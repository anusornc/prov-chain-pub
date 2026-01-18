use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::*;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::query::*;
use owl2_reasoner::{Class, NamedIndividual, ObjectProperty};
use std::sync::Arc;
use std::time::Duration;

fn create_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create test classes
    let person_class = IRI::new("http://example.org/Person").unwrap();
    let employee_class = IRI::new("http://example.org/Employee").unwrap();
    let manager_class = IRI::new("http://example.org/Manager").unwrap();
    let department_class = IRI::new("http://example.org/Department").unwrap();

    // Add class declarations
    ontology
        .add_class(Class::new(Arc::new(person_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(employee_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(manager_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(department_class.clone())))
        .unwrap();

    // Create test properties
    let works_for_prop = IRI::new("http://example.org/worksFor").unwrap();
    let manages_prop = IRI::new("http://example.org/manages").unwrap();
    let reports_to_prop = IRI::new("http://example.org/reportsTo").unwrap();

    // Add property declarations
    ontology
        .add_object_property(ObjectProperty::new(Arc::new(works_for_prop.clone())))
        .unwrap();
    ontology
        .add_object_property(ObjectProperty::new(Arc::new(manages_prop.clone())))
        .unwrap();
    ontology
        .add_object_property(ObjectProperty::new(Arc::new(reports_to_prop.clone())))
        .unwrap();

    // Add individuals and assertions
    for i in 0..size {
        let individual_iri = IRI::new(format!("http://example.org/person{}", i)).unwrap();
        let individual = NamedIndividual::new(Arc::new(individual_iri.clone()));
        ontology.add_named_individual(individual).unwrap();

        // Add class assertions (mix of types)
        if i % 3 == 0 {
            let class_expr = ClassExpression::Class(Class::new(Arc::new(person_class.clone())));
            let axiom = ClassAssertionAxiom::new(Arc::new(individual_iri.clone()), class_expr);
            ontology.add_class_assertion(axiom).unwrap();
        }
        if i % 4 == 0 {
            let class_expr = ClassExpression::Class(Class::new(Arc::new(employee_class.clone())));
            let axiom = ClassAssertionAxiom::new(Arc::new(individual_iri.clone()), class_expr);
            ontology.add_class_assertion(axiom).unwrap();
        }
        if i % 10 == 0 {
            let class_expr = ClassExpression::Class(Class::new(Arc::new(manager_class.clone())));
            let axiom = ClassAssertionAxiom::new(Arc::new(individual_iri.clone()), class_expr);
            ontology.add_class_assertion(axiom).unwrap();
        }

        // Add property assertions
        if i > 0 && i % 5 == 0 {
            let department_iri =
                IRI::new(format!("http://example.org/department{}", i / 5)).unwrap();
            let department_individual = NamedIndividual::new(Arc::new(department_iri.clone()));
            ontology
                .add_named_individual(department_individual)
                .unwrap();

            let department_class_expr =
                ClassExpression::Class(Class::new(Arc::new(department_class.clone())));
            let dept_axiom =
                ClassAssertionAxiom::new(Arc::new(department_iri.clone()), department_class_expr);
            ontology.add_class_assertion(dept_axiom).unwrap();

            let prop_axiom = PropertyAssertionAxiom::new(
                Arc::new(individual_iri.clone()),
                Arc::new(works_for_prop.clone()),
                Arc::new(department_iri),
            );
            ontology.add_property_assertion(prop_axiom).unwrap();
        }

        if i > 10 && i % 7 == 0 {
            let manager_iri = IRI::new(format!("http://example.org/person{}", i - 5)).unwrap();
            let prop_axiom = PropertyAssertionAxiom::new(
                Arc::new(individual_iri.clone()),
                Arc::new(reports_to_prop.clone()),
                Arc::new(manager_iri),
            );
            ontology.add_property_assertion(prop_axiom).unwrap();
        }
    }

    ontology
}

fn create_query_patterns() -> Vec<QueryPattern> {
    vec![
        // Simple type query
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Person").unwrap()),
        }]),
        // Property query
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?emp".to_string()),
            predicate: PatternTerm::IRI(IRI::new("http://example.org/worksFor").unwrap()),
            object: PatternTerm::Variable("?dept".to_string()),
        }]),
        // Multi-triple pattern with join
        QueryPattern::BasicGraphPattern(vec![
            TriplePattern {
                subject: PatternTerm::Variable("?emp".to_string()),
                predicate: PatternTerm::IRI(
                    IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                ),
                object: PatternTerm::IRI(IRI::new("http://example.org/Employee").unwrap()),
            },
            TriplePattern {
                subject: PatternTerm::Variable("?emp".to_string()),
                predicate: PatternTerm::IRI(IRI::new("http://example.org/worksFor").unwrap()),
                object: PatternTerm::Variable("?dept".to_string()),
            },
        ]),
        // Complex multi-triple pattern
        QueryPattern::BasicGraphPattern(vec![
            TriplePattern {
                subject: PatternTerm::Variable("?person".to_string()),
                predicate: PatternTerm::IRI(
                    IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                ),
                object: PatternTerm::IRI(IRI::new("http://example.org/Person").unwrap()),
            },
            TriplePattern {
                subject: PatternTerm::Variable("?person".to_string()),
                predicate: PatternTerm::IRI(IRI::new("http://example.org/reportsTo").unwrap()),
                object: PatternTerm::Variable("?manager".to_string()),
            },
            TriplePattern {
                subject: PatternTerm::Variable("?manager".to_string()),
                predicate: PatternTerm::IRI(
                    IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                ),
                object: PatternTerm::IRI(IRI::new("http://example.org/Manager").unwrap()),
            },
        ]),
        // Variable predicate query
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?subject".to_string()),
            predicate: PatternTerm::Variable("?predicate".to_string()),
            object: PatternTerm::Variable("?object".to_string()),
        }]),
    ]
}

fn benchmark_query_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_performance");

    let sizes = vec![100, 500, 1000, 5000];
    let query_patterns = create_query_patterns();

    for size in sizes {
        let ontology = create_test_ontology(size);

        // Test with optimizations enabled
        let config_optimized = QueryConfig {
            enable_reasoning: false,
            max_results: Some(1000),
            timeout: Some(Duration::from_millis(30000)),
            enable_caching: true,
            cache_size: Some(std::num::NonZeroUsize::new(1000).unwrap()),
            enable_parallel: false,
            enable_optimization: true,
            max_memory: Some(1024 * 1024),
            batch_size: 100,
        };

        // Test with optimizations disabled
        let config_baseline = QueryConfig {
            enable_reasoning: false,
            max_results: Some(1000),
            timeout: Some(Duration::from_millis(30000)),
            enable_caching: false,
            cache_size: None,
            enable_parallel: false,
            enable_optimization: false,
            max_memory: Some(1024 * 1024),
            batch_size: 100,
        };

        for (i, pattern) in query_patterns.iter().enumerate() {
            group.bench_with_input(
                BenchmarkId::new("optimized", format!("size{}_query{}", size, i)),
                &(ontology.clone(), pattern.clone(), config_optimized.clone()),
                |b, (ont, pat, conf)| {
                    b.iter(|| {
                        let engine = QueryEngine::with_config(ont.clone(), conf.clone());
                        let result = engine.execute(black_box(pat)).unwrap();
                        black_box(result)
                    })
                },
            );

            group.bench_with_input(
                BenchmarkId::new("baseline", format!("size{}_query{}", size, i)),
                &(ontology.clone(), pattern.clone(), config_baseline.clone()),
                |b, (ont, pat, conf)| {
                    b.iter(|| {
                        let engine = QueryEngine::with_config(ont.clone(), conf.clone());
                        let result = engine.execute(black_box(pat)).unwrap();
                        black_box(result)
                    })
                },
            );
        }
    }

    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    let ontology = create_test_ontology(1000);
    let pattern = &create_query_patterns()[0]; // Use simple type query

    let config_with_cache = QueryConfig {
        enable_reasoning: false,
        max_results: Some(1000),
        timeout: Some(Duration::from_millis(30000)),
        enable_caching: true,
        cache_size: Some(std::num::NonZeroUsize::new(1000).unwrap()),
        enable_parallel: false,
        enable_optimization: true,
        max_memory: Some(1024 * 1024),
        batch_size: 100,
    };

    let config_without_cache = QueryConfig {
        enable_reasoning: false,
        max_results: Some(1000),
        timeout: Some(Duration::from_millis(30000)),
        enable_caching: false,
        cache_size: None,
        enable_parallel: false,
        enable_optimization: false,
        max_memory: Some(1024 * 1024),
        batch_size: 100,
    };

    // First query (cache miss)
    group.bench_function("first_query_with_cache", |b| {
        b.iter(|| {
            let engine = QueryEngine::with_config(ontology.clone(), config_with_cache.clone());
            let result = engine.execute(black_box(pattern)).unwrap();
            black_box(result)
        })
    });

    // Repeated query (cache hit)
    group.bench_function("repeated_query_with_cache", |b| {
        b.iter(|| {
            let engine = QueryEngine::with_config(ontology.clone(), config_with_cache.clone());
            // Execute once to populate cache
            let _ = engine.execute(pattern).unwrap();
            // Execute again (should hit cache)
            let result = engine.execute(black_box(pattern)).unwrap();
            black_box(result)
        })
    });

    // Same query without cache
    group.bench_function("repeated_query_without_cache", |b| {
        b.iter(|| {
            let engine = QueryEngine::with_config(ontology.clone(), config_without_cache.clone());
            let result = engine.execute(black_box(pattern)).unwrap();
            black_box(result)
        })
    });

    group.finish();
}

fn benchmark_index_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_performance");

    let sizes = vec![100, 1000, 5000];

    for size in sizes {
        let ontology = create_test_ontology(size);

        // Query with specific type (should use index)
        let indexed_query = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Employee").unwrap()),
        }]);

        // Query with variable type (should not use index efficiently)
        let non_indexed_query = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::Variable("?type".to_string()),
        }]);

        group.bench_with_input(
            BenchmarkId::new("indexed_query", size),
            &(ontology.clone(), indexed_query),
            |b, (ont, query)| {
                b.iter(|| {
                    let engine = QueryEngine::new(ont.clone());
                    let result = engine.execute(black_box(query)).unwrap();
                    black_box(result)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("non_indexed_query", size),
            &(ontology.clone(), non_indexed_query),
            |b, (ont, query)| {
                b.iter(|| {
                    let engine = QueryEngine::new(ont.clone());
                    let result = engine.execute(black_box(query)).unwrap();
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_pattern_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_compilation");

    let ontology = create_test_ontology(1000);
    let pattern = &create_query_patterns()[2]; // Multi-triple pattern

    // Test pattern compilation caching
    group.bench_function("pattern_compilation_with_cache", |b| {
        b.iter(|| {
            let engine = QueryEngine::new(ontology.clone());
            // Execute multiple times to test pattern caching
            for _ in 0..10 {
                let result = engine.execute(black_box(pattern)).unwrap();
                black_box(result);
            }
        })
    });

    // Test without pattern caching (create new engine each time)
    group.bench_function("pattern_compilation_without_cache", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let engine = QueryEngine::new(ontology.clone());
                let result = engine.execute(black_box(pattern)).unwrap();
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_query_performance,
    benchmark_cache_performance,
    benchmark_index_performance,
    benchmark_pattern_compilation
);
criterion_main!(benches);
