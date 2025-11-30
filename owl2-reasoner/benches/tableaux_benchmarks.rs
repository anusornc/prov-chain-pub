//! Tableaux Module Performance Benchmarks
//!
//! Comprehensive benchmark suite for the new modular tableaux reasoning engine.
//! This benchmarks individual components and their integration to ensure
//! optimal performance across all tableaux modules.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use owl2_reasoner::axioms::*;
use owl2_reasoner::entities::*;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::tableaux::dependency::{
    ChoiceType, DependencySource, DependencyType,
};
use owl2_reasoner::reasoning::tableaux::graph::GraphChangeLog;
use owl2_reasoner::reasoning::tableaux::memory::MemoryChangeLog;
use owl2_reasoner::reasoning::tableaux::*;
use std::sync::Arc;

/// Benchmark core tableaux reasoner performance
pub fn bench_tableaux_core(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_core");

    for size in [100, 500, 1000, 2000].iter() {
        let ontology = create_test_ontology(*size);

        group.bench_with_input(BenchmarkId::new("reasoner_creation", size), size, |b, _| {
            b.iter(|| {
                let reasoner = TableauxReasoner::new(black_box(ontology.clone()));
                black_box(reasoner);
            })
        });

        let mut reasoner = TableauxReasoner::new(ontology.clone());
        group.bench_with_input(
            BenchmarkId::new("consistency_checking", size),
            size,
            |b, _| {
                b.iter(|| {
                    let result = reasoner.is_consistent();
                    let _ = black_box(result);
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("memory_stats", size), size, |b, _| {
            b.iter(|| {
                let stats = reasoner.get_memory_stats();
                black_box(stats);
            })
        });
    }

    group.finish();
}

/// Benchmark tableaux graph operations
pub fn bench_tableaux_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_graph");

    for node_count in [100, 500, 1000, 2000].iter() {
        let mut graph = TableauxGraph::new();

        // Create nodes
        let _nodes: Vec<NodeId> = (0..*node_count).map(|_| graph.add_node()).collect();

        group.bench_with_input(
            BenchmarkId::new("node_creation", node_count),
            node_count,
            |b, count| {
                b.iter(|| {
                    let mut test_graph = TableauxGraph::new();
                    for _ in 0..*count {
                        test_graph.add_node();
                    }
                    black_box(test_graph);
                })
            },
        );

        let property_iri = IRI::new("http://example.org/hasProperty").unwrap();

        group.bench_with_input(
            BenchmarkId::new("edge_addition", node_count),
            node_count,
            |b, _| {
                b.iter(|| {
                    let mut test_graph = TableauxGraph::new();
                    let node1 = test_graph.add_node();
                    let node2 = test_graph.add_node();
                    test_graph.add_edge(node1, black_box(&property_iri), node2);
                    black_box(test_graph);
                })
            },
        );

        // Create graph with edges for traversal benchmark
        let mut traversal_graph = TableauxGraph::new();
        let traversal_nodes: Vec<NodeId> = (0..*node_count)
            .map(|_| traversal_graph.add_node())
            .collect();

        for i in 0..(node_count - 1) {
            traversal_graph.add_edge(traversal_nodes[i], &property_iri, traversal_nodes[i + 1]);
        }

        group.bench_with_input(
            BenchmarkId::new("graph_traversal", node_count),
            node_count,
            |b, _| {
                b.iter(|| {
                    for node_id in &traversal_nodes {
                        if let Some(node) = traversal_graph.get_node(*node_id) {
                            let concept_count = node.concepts_iter().count();
                            black_box(concept_count);
                        }
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory management performance
pub fn bench_tableaux_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_memory");

    for allocation_count in [1000, 5000, 10000, 20000].iter() {
        group.bench_with_input(
            BenchmarkId::new("arena_allocation", allocation_count),
            allocation_count,
            |b, count| {
                b.iter(|| {
                    let memory_manager = MemoryManager::new();
                    // Memory manager doesn't need explicit arena creation in this version

                    for i in 0..*count {
                        let node = TableauxNode::new(NodeId::new(i));
                        // Simulate arena allocation (concept would normally be allocated in arena)
                        black_box(node);
                    }

                    // Get memory stats without arena reference
                    let stats = memory_manager.get_memory_stats();
                    let _ = black_box(stats);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("memory_manager_creation", allocation_count),
            allocation_count,
            |b, _| {
                b.iter(|| {
                    let manager = MemoryManager::new();
                    black_box(manager);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark blocking strategies performance
pub fn bench_tableaux_blocking(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_blocking");

    let strategies = [
        BlockingStrategy::Equality,
        BlockingStrategy::Subset,
        BlockingStrategy::Optimized,
    ];

    for strategy in strategies.iter() {
        for node_count in [100, 500, 1000].iter() {
            let blocking_manager = BlockingManager::new(strategy.clone());
            let mut graph = TableauxGraph::new();

            // Create test nodes
            let nodes: Vec<NodeId> = (0..*node_count).map(|_| graph.add_node()).collect();

            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", strategy), node_count),
                node_count,
                |b, _| {
                    b.iter(|| {
                        let should_block = blocking_manager.is_blocked(nodes[0]);
                        black_box(should_block);
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark dependency management performance
pub fn bench_tableaux_dependency(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_dependency");

    for dependency_count in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("dependency_creation", dependency_count),
            dependency_count,
            |b, count| {
                b.iter(|| {
                    let mut manager = DependencyManager::new();

                    for i in 0..*count {
                        let choice_point =
                            manager.create_choice_point(NodeId::new(i), ChoiceType::Disjunction);
                        let dependency = Dependency::new(
                            NodeId::new(i + 1),
                            DependencySource::ChoicePoint(choice_point.id),
                            DependencyType::Subclass,
                        );
                        manager.add_dependency(dependency);
                    }

                    black_box(manager);
                })
            },
        );

        let mut dependency_manager = DependencyManager::new();
        for i in 0..*dependency_count {
            let choice_point =
                dependency_manager.create_choice_point(NodeId::new(i), ChoiceType::Disjunction);
            let dependency = Dependency::new(
                NodeId::new(i + 1),
                DependencySource::ChoicePoint(choice_point.id),
                DependencyType::Subclass,
            );
            dependency_manager.add_dependency(dependency);
        }

        group.bench_with_input(
            BenchmarkId::new("dependency_backtracking", dependency_count),
            dependency_count,
            |b, _| {
                b.iter(|| {
                    let dependencies =
                        dependency_manager.get_dependencies(NodeId::new(*dependency_count / 2));
                    black_box(dependencies.len());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark expansion rules performance
pub fn bench_tableaux_expansion(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_expansion");

    for complexity in [10, 50, 100, 200].iter() {
        let _ontology = create_complex_ontology(*complexity);
        let mut graph = TableauxGraph::new();
        let mut memory_manager = MemoryManager::new();
        let _expansion_engine = ExpansionEngine::new();

        group.bench_with_input(
            BenchmarkId::new("expansion_engine_creation", complexity),
            complexity,
            |b, _| {
                b.iter(|| {
                    let engine = ExpansionEngine::new();
                    black_box(engine);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rule_application", complexity),
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut test_engine = ExpansionEngine::new();
                    let max_depth = 50;
                    let mut graph_change_log = GraphChangeLog::new();
                    let mut memory_change_log = MemoryChangeLog::new();
                    let result = test_engine.expand(
                        &mut graph,
                        &mut memory_manager,
                        max_depth,
                        &mut graph_change_log,
                        &mut memory_change_log,
                    );
                    let _ = black_box(result);
                })
            },
        );

        let rules = ExpansionRules::new();
        group.bench_with_input(
            BenchmarkId::new("rule_selection", complexity),
            complexity,
            |b, _| {
                b.iter(|| {
                    let mut test_engine = ExpansionEngine::new();
                    let context = &mut test_engine.context;
                    if let Some(rule) = rules.get_next_rule(context) {
                        black_box(rule);
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark integration performance - full tableaux reasoning pipeline
pub fn bench_tableaux_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_integration");

    for ontology_size in [100, 500, 1000, 2000].iter() {
        let ontology = create_comprehensive_test_ontology(*ontology_size);

        group.throughput(Throughput::Elements(*ontology_size as u64));

        group.bench_with_input(
            BenchmarkId::new("full_reasoning_pipeline", ontology_size),
            ontology_size,
            |b, _size| {
                b.iter(|| {
                    let mut reasoner = TableauxReasoner::with_config(
                        black_box(ontology.clone()),
                        ReasoningConfig {
                            max_depth: 1000,
                            debug: false,
                            incremental: true,
                            timeout: Some(30000),
                            enable_parallel: false,
                            parallel_workers: None,
                            parallel_chunk_size: 64,
                        },
                    );

                    let is_consistent = reasoner.is_consistent().unwrap();
                    let memory_stats = reasoner.get_memory_stats();

                    black_box((is_consistent, memory_stats));
                })
            },
        );

        // Benchmark with different configurations
        let configs = vec![
            (
                "fast",
                ReasoningConfig {
                    max_depth: 100,
                    debug: false,
                    incremental: false,
                    timeout: Some(5000),
                    enable_parallel: false,
                    parallel_workers: None,
                    parallel_chunk_size: 64,
                },
            ),
            (
                "balanced",
                ReasoningConfig {
                    max_depth: 500,
                    debug: false,
                    incremental: true,
                    timeout: Some(15000),
                    enable_parallel: false,
                    parallel_workers: None,
                    parallel_chunk_size: 64,
                },
            ),
            (
                "thorough",
                ReasoningConfig {
                    max_depth: 2000,
                    debug: true,
                    incremental: true,
                    timeout: Some(60000),
                    enable_parallel: false,
                    parallel_workers: None,
                    parallel_chunk_size: 64,
                },
            ),
        ];

        for (config_name, config) in configs {
            group.bench_with_input(
                BenchmarkId::new(format!("config_{}", config_name), ontology_size),
                ontology_size,
                |b, _| {
                    b.iter(|| {
                        let mut reasoner = TableauxReasoner::with_config(
                            black_box(ontology.clone()),
                            black_box(config.clone()),
                        );

                        let result = reasoner.is_consistent();
                        let _ = black_box(result);
                    })
                },
            );
        }
    }

    group.finish();
}

/// Helper function to create a test ontology
fn create_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create classes
    for i in 0..size {
        let class_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(class_iri);
        ontology.add_class(class).unwrap();
    }

    // Create subclass hierarchy
    for i in 1..(size / 2) {
        let subclass_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let superclass_iri = IRI::new(format!("http://example.org/Class{}", i / 2)).unwrap();

        let subclass = ClassExpression::Class(Class::new(subclass_iri));
        let superclass = ClassExpression::Class(Class::new(superclass_iri));
        let axiom = SubClassOfAxiom::new(subclass, superclass);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    ontology
}

/// Helper function to create a complex ontology with various axiom types
fn create_complex_ontology(complexity: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create classes
    for i in 0..complexity {
        let class_iri = IRI::new(format!("http://example.org/ComplexClass{}", i)).unwrap();
        let class = Class::new(class_iri);
        ontology.add_class(class).unwrap();
    }

    // Create object properties
    for i in 0..(complexity / 10) {
        let prop_iri = IRI::new(format!("http://example.org/hasProperty{}", i)).unwrap();
        let prop = ObjectProperty::new(prop_iri);
        ontology.add_object_property(prop).unwrap();
    }

    // Create complex class expressions
    for i in 0..(complexity / 5) {
        if i + 1 < complexity {
            let class1_iri = IRI::new(format!("http://example.org/ComplexClass{}", i)).unwrap();
            let class2_iri = IRI::new(format!("http://example.org/ComplexClass{}", i + 1)).unwrap();

            let class1 = ClassExpression::Class(Class::new(class1_iri));
            let class2 = ClassExpression::Class(Class::new(class2_iri));

            // Create simple subclass relationships instead of complex expressions
            let subclass_axiom = SubClassOfAxiom::new(class1.clone(), class2.clone());
            ontology.add_subclass_axiom(subclass_axiom).unwrap();
        }
    }

    ontology
}

/// Helper function to create a comprehensive test ontology
fn create_comprehensive_test_ontology(size: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create hierarchical class structure
    for i in 0..size {
        let class_iri = IRI::new(format!("http://example.org/TestClass{}", i)).unwrap();
        let class = Class::new(class_iri);
        ontology.add_class(class).unwrap();
    }

    // Create object properties
    for i in 0..(size / 20) {
        let prop_iri = IRI::new(format!("http://example.org/testProperty{}", i)).unwrap();
        let prop = ObjectProperty::new(prop_iri);
        ontology.add_object_property(prop).unwrap();
    }

    // Create subclass relationships
    for i in 1..(size / 2) {
        let subclass_iri = IRI::new(format!("http://example.org/TestClass{}", i)).unwrap();
        let superclass_iri = IRI::new(format!("http://example.org/TestClass{}", i / 2)).unwrap();

        let subclass = ClassExpression::Class(Class::new(subclass_iri));
        let superclass = ClassExpression::Class(Class::new(superclass_iri));
        let axiom = SubClassOfAxiom::new(subclass, superclass);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    // Create equivalent classes
    for i in 0..(size / 10) {
        if i + 1 < size {
            let class1_iri = IRI::new(format!("http://example.org/TestClass{}", i)).unwrap();
            let class2_iri = IRI::new(format!("http://example.org/TestClass{}", i + 1)).unwrap();

            let _class1 = ClassExpression::Class(Class::new(class1_iri.clone()));
            let _class2 = ClassExpression::Class(Class::new(class2_iri.clone()));

            let equiv_axiom =
                EquivalentClassesAxiom::new(vec![Arc::new(class1_iri), Arc::new(class2_iri)]);
            ontology.add_equivalent_classes_axiom(equiv_axiom).unwrap();
        }
    }

    ontology
}

/// Benchmark memory usage across different tableaux operations
pub fn bench_tableaux_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_memory_usage");

    group.measurement_time(std::time::Duration::from_secs(10));

    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("reasoner_memory_footprint", size),
            size,
            |b, size| {
                b.iter(|| {
                    let ontology = create_comprehensive_test_ontology(*size);
                    let mut reasoner = TableauxReasoner::new(ontology);

                    // Perform reasoning to populate memory structures
                    let _is_consistent = reasoner.is_consistent().unwrap();
                    let memory_stats = reasoner.get_memory_stats();

                    black_box(memory_stats);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("graph_memory_usage", size),
            size,
            |b, size| {
                b.iter(|| {
                    let mut graph = TableauxGraph::new();
                    let property_iri = IRI::new("http://example.org/testProperty").unwrap();

                    // Create many nodes and edges
                    for _i in 0..*size {
                        let node1 = graph.add_node();
                        let node2 = graph.add_node();
                        graph.add_edge(node1, &property_iri, node2);
                    }

                    black_box(graph);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark cache performance
pub fn bench_tableaux_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("tableaux_caching");

    for cache_size in [100, 500, 1000].iter() {
        let mut reasoner = TableauxReasoner::new(create_test_ontology(*cache_size));

        // Warm up cache
        for _ in 0..*cache_size {
            let _ = reasoner.is_consistent();
        }

        group.bench_with_input(
            BenchmarkId::new("cache_hit_performance", cache_size),
            cache_size,
            |b, _| {
                b.iter(|| {
                    let result = reasoner.is_consistent();
                    let _ = black_box(result);
                })
            },
        );

        // Benchmark cache clear performance
        group.bench_with_input(
            BenchmarkId::new("cache_clear_performance", cache_size),
            cache_size,
            |b, _| {
                b.iter(|| {
                    let mut test_reasoner =
                        TableauxReasoner::new(create_test_ontology(*cache_size));
                    test_reasoner.clear_cache();
                    black_box(test_reasoner);
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    tableaux_benchmarks,
    bench_tableaux_core,
    bench_tableaux_graph,
    bench_tableaux_memory,
    bench_tableaux_blocking,
    bench_tableaux_dependency,
    bench_tableaux_expansion,
    bench_tableaux_integration,
    bench_tableaux_memory_usage,
    bench_tableaux_caching
);

criterion_main!(tableaux_benchmarks);
