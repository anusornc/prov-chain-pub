use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::axioms::*;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::query::*;
use owl2_reasoner::{Class, NamedIndividual, ObjectProperty};
use std::sync::Arc;

fn create_union_test_ontology(size: usize, union_branches: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create test classes for union patterns
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
    let _type_prop = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();

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

    // Add individuals and assertions distributed across union branches
    for i in 0..size {
        let individual_iri = IRI::new(format!("http://example.org/person{}", i)).unwrap();
        let individual = NamedIndividual::new(Arc::new(individual_iri.clone()));

        // Distribute individuals across different types to create union scenarios
        let branch_index = i % union_branches;
        let target_class = match branch_index {
            0 => &person_class,
            1 => &employee_class,
            2 => &manager_class,
            _ => &department_class,
        };

        // Add individual
        ontology.add_named_individual(individual).unwrap();

        // Add type assertion
        ontology
            .add_class_assertion(ClassAssertionAxiom::new(
                Arc::new(individual_iri.clone()),
                ClassExpression::Class(Class::new(Arc::new(target_class.clone()))),
            ))
            .unwrap();

        // Add some property assertions for more complex queries
        if i > 0 {
            let works_for_iri =
                IRI::new(format!("http://example.org/department{}", i % 10)).unwrap();
            let works_for_individual = NamedIndividual::new(Arc::new(works_for_iri.clone()));
            ontology.add_named_individual(works_for_individual).unwrap();

            ontology
                .add_property_assertion(PropertyAssertionAxiom::new(
                    Arc::new(individual_iri.clone()),
                    Arc::new(works_for_prop.clone()),
                    Arc::new(works_for_iri),
                ))
                .unwrap();
        }
    }

    ontology
}

fn create_union_query_pattern(union_branches: usize) -> QueryPattern {
    // Helper function to create a basic pattern for a class
    let create_class_pattern = |class_iri: &str| -> QueryPattern {
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new(class_iri).unwrap()),
        }])
    };

    let class_iris = vec![
        "http://example.org/Person",
        "http://example.org/Employee",
        "http://example.org/Manager",
        "http://example.org/Department",
    ];

    // Create nested union from multiple patterns
    let patterns: Vec<QueryPattern> = (0..union_branches)
        .map(|i| {
            let class_iri = class_iris[i % class_iris.len()];
            create_class_pattern(class_iri)
        })
        .collect();

    // Reduce to nested union: Union { left: A, right: Union { left: B, right: C } }
    patterns
        .into_iter()
        .reduce(|left, right| QueryPattern::Union {
            left: Box::new(left),
            right: Box::new(right),
        })
        .unwrap_or_else(|| create_class_pattern("http://example.org/Person"))
}

fn bench_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_query_execution");

    // Test different data sizes
    for size in [1000, 5000, 10000].iter() {
        // Test different union complexities
        for branches in [2, 4, 8].iter() {
            let ontology = create_union_test_ontology(*size, *branches);
            let query_pattern = create_union_query_pattern(*branches);

            // Sequential execution
            group.bench_with_input(
                BenchmarkId::new("sequential", format!("size{}_branches{}", size, branches)),
                &(&ontology, &query_pattern),
                |b, (ontology, pattern)| {
                    b.iter(|| {
                        let engine = QueryEngine::with_config(
                            (*ontology).clone(),
                            QueryConfig {
                                enable_reasoning: false,
                                max_results: None,
                                timeout: None,
                                enable_caching: false,
                                cache_size: None,
                                enable_parallel: false,
                                enable_optimization: false,
                                max_memory: None,
                                batch_size: 100,
                            },
                        );

                        black_box(engine.execute(black_box(pattern)).unwrap());
                    });
                },
            );

            // Parallel execution
            group.bench_with_input(
                BenchmarkId::new("parallel", format!("size{}_branches{}", size, branches)),
                &(&ontology, &query_pattern),
                |b, (ontology, pattern)| {
                    b.iter(|| {
                        let engine = QueryEngine::with_config(
                            (*ontology).clone(),
                            QueryConfig {
                                enable_reasoning: false,
                                max_results: None,
                                timeout: None,
                                enable_caching: false,
                                cache_size: None,
                                enable_parallel: true,
                                enable_optimization: true,
                                max_memory: Some(1024 * 1024),
                                batch_size: 100,
                            },
                        );

                        black_box(engine.execute(black_box(pattern)).unwrap());
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_parallel_thread_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_thread_scaling");

    let ontology = create_union_test_ontology(10000, 8);
    let query_pattern = create_union_query_pattern(8);

    // Test different thread counts
    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &(threads, &ontology, &query_pattern),
            |b, (threads, ontology, pattern)| {
                b.iter(|| {
                    let engine = QueryEngine::with_config(
                        (*ontology).clone(),
                        QueryConfig {
                            enable_reasoning: false,
                            max_results: None,
                            timeout: None,
                            enable_caching: false,
                            cache_size: None,
                            enable_parallel: true,
                            enable_optimization: true,
                            max_memory: Some(1024 * 1024),
                            batch_size: 100,
                        },
                    );

                    black_box(engine.execute(black_box(pattern)).unwrap());
                });
            },
        );
    }
    group.finish();
}

fn bench_parallel_threshold_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_threshold");

    let ontology = create_union_test_ontology(5000, 6);

    // Test different union sizes against thresholds
    for union_size in [1, 2, 3, 6].iter() {
        let query_pattern = create_union_query_pattern(*union_size);

        // Below threshold (should be sequential)
        group.bench_with_input(
            BenchmarkId::new("below_threshold", union_size),
            &(&ontology, &query_pattern),
            |b, (ontology, pattern)| {
                b.iter(|| {
                    let engine = QueryEngine::with_config(
                        (*ontology).clone(),
                        QueryConfig {
                            enable_reasoning: false,
                            max_results: None,
                            timeout: None,
                            enable_caching: false,
                            cache_size: None,
                            enable_parallel: true,
                            enable_optimization: true,
                            max_memory: Some(1024 * 1024),
                            batch_size: 100,
                        },
                    );

                    black_box(engine.execute(black_box(pattern)).unwrap());
                });
            },
        );

        // Above threshold (should be parallel)
        group.bench_with_input(
            BenchmarkId::new("above_threshold", union_size),
            &(&ontology, &query_pattern),
            |b, (ontology, pattern)| {
                b.iter(|| {
                    let engine = QueryEngine::with_config(
                        (*ontology).clone(),
                        QueryConfig {
                            enable_reasoning: false,
                            max_results: None,
                            timeout: None,
                            enable_caching: false,
                            cache_size: None,
                            enable_parallel: true,
                            enable_optimization: true,
                            max_memory: Some(1024 * 1024),
                            batch_size: 100,
                        },
                    );

                    black_box(engine.execute(black_box(pattern)).unwrap());
                });
            },
        );
    }
    group.finish();
}

fn bench_memory_pool_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool_effectiveness");

    let ontology = create_union_test_ontology(5000, 4);
    let query_pattern = create_union_query_pattern(4);

    // With memory pool
    group.bench_function("with_memory_pool", |b| {
        b.iter(|| {
            let engine = QueryEngine::with_config(
                ontology.clone(),
                QueryConfig {
                    enable_reasoning: false,
                    max_results: None,
                    timeout: None,
                    enable_caching: false,
                    cache_size: None,
                    enable_parallel: true,
                    enable_optimization: true,
                    max_memory: Some(1024 * 1024),
                    batch_size: 100,
                },
            );

            black_box(
                engine
                    .execute(black_box(&query_pattern))
                    .unwrap(),
            );
        });
    });

    // Without memory pool
    group.bench_function("without_memory_pool", |b| {
        b.iter(|| {
            let engine = QueryEngine::with_config(
                ontology.clone(),
                QueryConfig {
                    enable_reasoning: false,
                    max_results: None,
                    timeout: None,
                    enable_caching: false,
                    cache_size: None,
                    enable_parallel: true,
                    enable_optimization: true,
                    max_memory: Some(1024 * 1024),
                    batch_size: 100,
                },
            );

            black_box(
                engine
                    .execute(black_box(&query_pattern))
                    .unwrap(),
            );
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_vs_parallel,
    bench_parallel_thread_scaling,
    bench_parallel_threshold_effectiveness,
    bench_memory_pool_effectiveness
);
criterion_main!(benches);
