//! Criterion.rs benchmarks for RDF canonicalization performance
//!
//! Benchmarks different RDF canonicalization algorithms and complexity scenarios

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use provchain_org::core::blockchain::Blockchain;
use std::time::Duration;

/// Benchmark RDF canonicalization with different graph complexities
fn bench_canonicalization_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonicalization_complexity");
    group.measurement_time(Duration::from_secs(15));

    let complexity_scenarios = vec![
        ("minimal", generate_minimal_rdf_graphs(10)),
        ("simple", generate_simple_rdf_graphs(10)),
        ("moderate", generate_moderate_rdf_graphs(10)),
        ("complex", generate_complex_rdf_graphs(10)),
        ("pathological", generate_pathological_rdf_graphs(5)), // Fewer for pathological cases
    ];

    for (complexity_name, test_graphs) in complexity_scenarios {
        group.throughput(Throughput::Elements(test_graphs.len() as u64));
        group.bench_function(BenchmarkId::new("complexity", complexity_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_graphs.clone())
                },
                |(mut blockchain, graphs)| {
                    for graph_data in graphs {
                        let _ = blockchain.add_block(black_box(graph_data));
                    }
                    black_box(blockchain)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark canonicalization with different blank node patterns
fn bench_blank_node_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("blank_node_patterns");

    let blank_node_scenarios = vec![
        ("no_blanks", generate_no_blank_nodes(15)),
        ("simple_blanks", generate_simple_blank_nodes(15)),
        ("nested_blanks", generate_nested_blank_nodes(10)),
        ("circular_blanks", generate_circular_blank_nodes(8)),
        ("isomorphic_blanks", generate_isomorphic_blank_nodes(5)),
    ];

    for (pattern_name, test_graphs) in blank_node_scenarios {
        group.bench_function(BenchmarkId::new("blank_nodes", pattern_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_graphs.clone())
                },
                |(mut blockchain, graphs)| {
                    for graph_data in graphs {
                        let _ = blockchain.add_block(black_box(graph_data));
                    }
                    black_box(blockchain)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark canonicalization scaling with graph size
fn bench_canonicalization_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonicalization_scaling");
    group.measurement_time(Duration::from_secs(20));

    // Test different graph sizes
    for &size in [10, 25, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("graph_size", size),
            &size,
            |b, &graph_size| {
                let test_graph = generate_scaling_test_graph(graph_size);
                b.iter_batched(
                    || {
                        let blockchain = Blockchain::new();
                        (blockchain, test_graph.clone())
                    },
                    |(mut blockchain, graph_data)| {
                        let _ = blockchain.add_block(black_box(graph_data));
                        black_box(blockchain)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark supply chain specific canonicalization patterns
fn bench_supply_chain_canonicalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("supply_chain_canonicalization");

    let supply_chain_scenarios = vec![
        ("linear_trace", generate_linear_supply_chain(20)),
        ("branched_trace", generate_branched_supply_chain(15)),
        ("merged_trace", generate_merged_supply_chain(12)),
        ("complex_provenance", generate_complex_provenance_chain(10)),
    ];

    for (scenario_name, test_chains) in supply_chain_scenarios {
        group.bench_function(BenchmarkId::new("supply_chain", scenario_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_chains.clone())
                },
                |(mut blockchain, chains)| {
                    for chain_data in chains {
                        let _ = blockchain.add_block(black_box(chain_data));
                    }
                    black_box(blockchain)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// Graph generation functions

fn generate_minimal_rdf_graphs(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
ex:subject{} ex:predicate{} "value{}" .
"#,
                i, i, i
            )
        })
        .collect()
}

fn generate_simple_rdf_graphs(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:subject{} rdf:type ex:Type{} .
ex:subject{} ex:hasProperty "value{}" .
ex:subject{} ex:relatedTo ex:object{} .
"#,
                i,
                i,
                i,
                i,
                i,
                (i + 1) % count
            )
        })
        .collect()
}

fn generate_moderate_rdf_graphs(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:entity{} rdf:type ex:ComplexType .
ex:entity{} ex:hasChild ex:child{} .
ex:entity{} ex:hasChild ex:child{} .
ex:child{} ex:parentOf ex:entity{} .
ex:child{} ex:siblingOf ex:child{} .
_:blank{} ex:describes ex:entity{} .
_:blank{} ex:hasValue "complex value {}" .
"#,
                i,
                i,
                i,
                (i + 1) % count,
                i,
                (i + 2) % count,
                (i + 1) % count,
                (i + 2) % count,
                i,
                i,
                i,
                i,
                i
            )
        })
        .collect()
}

fn generate_complex_rdf_graphs(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

_:root{} rdf:type ex:ComplexStructure .
_:root{} ex:hasComponent _:comp{} .
_:root{} ex:hasComponent _:comp{} .
_:comp{} ex:connectsTo _:comp{} .
_:comp{} ex:connectsTo _:comp{} .
_:comp{} ex:hasProperty "prop{}" .
_:comp{} ex:hasProperty "prop{}" .
ex:named{} ex:references _:root{} .
ex:named{} ex:hasComplexity "high" .
"#,
                i,
                i,
                i,
                (i + 1) % count,
                i,
                (i + 1) % count,
                (i + 2) % count,
                (i + 1) % count,
                (i + 2) % count,
                i,
                (i + 1) % count,
                i,
                i,
                i,
                i,
                i
            )
        })
        .collect()
}

fn generate_pathological_rdf_graphs(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let mut graph = format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

"#
            );

            // Create a highly interconnected graph with many blank nodes
            for j in 0..10 {
                for k in 0..10 {
                    graph.push_str(&format!(
                        "_:b{}{} ex:connectsTo _:b{}{} .\n",
                        i,
                        j,
                        i,
                        (k + 1) % 10
                    ));
                }
            }

            // Add some named nodes that reference the blank node structure
            for j in 0..5 {
                graph.push_str(&format!(
                    "ex:anchor{}{} ex:references _:b{}{} .\n",
                    i, j, i, j
                ));
            }

            graph
        })
        .collect()
}

fn generate_no_blank_nodes(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:subject{} rdf:type ex:Type .
ex:subject{} ex:property "value{}" .
ex:subject{} ex:relatedTo ex:object{} .
ex:object{} ex:backRef ex:subject{} .
"#,
                i,
                i,
                i,
                i,
                (i + 1) % count,
                (i + 1) % count,
                i
            )
        })
        .collect()
}

fn generate_simple_blank_nodes(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .

ex:subject{} ex:hasBlankProperty _:blank{} .
_:blank{} ex:value "blank value {}" .
_:blank{} ex:type "BlankType" .
"#,
                i, i, i, i, i
            )
        })
        .collect()
}

fn generate_nested_blank_nodes(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .

ex:root{} ex:hasChild _:child{} .
_:child{} ex:hasGrandchild _:grandchild{} .
_:grandchild{} ex:hasValue "nested{}" .
_:child{} ex:siblingOf _:sibling{} .
_:sibling{} ex:backToRoot ex:root{} .
"#,
                i, i, i, i, i, i, i, i, i, i
            )
        })
        .collect()
}

fn generate_circular_blank_nodes(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .

_:a{} ex:pointsTo _:b{} .
_:b{} ex:pointsTo _:c{} .
_:c{} ex:pointsTo _:a{} .
_:a{} ex:hasValue "circular{}" .
ex:anchor{} ex:references _:a{} .
"#,
                i, i, i, i, i, i, i, i, i, i
            )
        })
        .collect()
}

fn generate_isomorphic_blank_nodes(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .

_:struct1_{} ex:hasA _:a1_{} .
_:struct1_{} ex:hasB _:b1_{} .
_:a1_{} ex:connectsTo _:b1_{} .

_:struct2_{} ex:hasA _:a2_{} .
_:struct2_{} ex:hasB _:b2_{} .
_:a2_{} ex:connectsTo _:b2_{} .

ex:container{} ex:contains _:struct1_{} .
ex:container{} ex:contains _:struct2_{} .
"#,
                i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i
            )
        })
        .collect()
}

fn generate_scaling_test_graph(size: usize) -> String {
    let mut graph = String::from(
        r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

"#,
    );

    for i in 0..size {
        graph.push_str(&format!("ex:node{} rdf:type ex:ScalingNode .\n", i));
        graph.push_str(&format!("ex:node{} ex:hasValue \"value{}\" .\n", i, i));

        if i > 0 {
            graph.push_str(&format!("ex:node{} ex:connectsTo ex:node{} .\n", i, i - 1));
        }

        if i < size - 1 {
            graph.push_str(&format!("ex:node{} ex:pointsTo ex:node{} .\n", i, i + 1));
        }
    }

    graph
}

fn generate_linear_supply_chain(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:step{} a trace:SupplyChainStep ;
    trace:stepNumber {} ;
    prov:wasInformedBy trace:step{} ;
    trace:hasProduct trace:product{} .

trace:product{} a trace:Product ;
    trace:hasID "PROD{:06}" .
"#,
                i,
                i,
                if i > 0 { i - 1 } else { 0 },
                i,
                i,
                i
            )
        })
        .collect()
}

fn generate_branched_supply_chain(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            if i % 3 == 0 {
                // Branching point
                format!(
                    r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:branch{} a trace:BranchingProcess ;
    prov:used trace:input{} ;
    prov:generated trace:output{}A ;
    prov:generated trace:output{}B .

trace:output{}A a trace:Product .
trace:output{}B a trace:Product .
"#,
                    i, i, i, i, i, i
                )
            } else {
                // Regular step
                format!(
                    r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:step{} a trace:ProcessingStep ;
    prov:used trace:input{} ;
    prov:generated trace:output{} .
"#,
                    i, i, i
                )
            }
        })
        .collect()
}

fn generate_merged_supply_chain(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            if i % 4 == 0 && i > 0 {
                // Merging point
                format!(
                    r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:merge{} a trace:MergingProcess ;
    prov:used trace:inputA{} ;
    prov:used trace:inputB{} ;
    prov:generated trace:merged{} .

trace:merged{} a trace:CompositeProduct .
"#,
                    i, i, i, i, i
                )
            } else {
                format!(
                    r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:step{} a trace:LinearStep ;
    prov:used trace:input{} ;
    prov:generated trace:output{} .
"#,
                    i, i, i
                )
            }
        })
        .collect()
}

fn generate_complex_provenance_chain(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:activity{} a prov:Activity ;
    prov:startedAtTime "2025-08-08T{:02}:00:00Z" ;
    prov:endedAtTime "2025-08-08T{:02}:30:00Z" ;
    prov:wasAssociatedWith trace:agent{} ;
    prov:used trace:entity{} ;
    prov:generated trace:result{} .

trace:agent{} a prov:Agent ;
    trace:hasRole "processor{}" .

trace:entity{} a prov:Entity ;
    prov:wasDerivedFrom trace:source{} .

trace:result{} a prov:Entity ;
    prov:wasGeneratedBy trace:activity{} .

_:provenance{} prov:hadPrimarySource trace:entity{} ;
    prov:wasQuotedFrom trace:source{} .
"#,
                i,
                i % 24,
                (i + 1) % 24,
                i,
                i,
                i,
                i,
                i,
                i,
                i,
                i,
                i,
                i,
                i,
                i
            )
        })
        .collect()
}

criterion_group!(
    benches,
    bench_canonicalization_complexity,
    bench_blank_node_patterns,
    bench_canonicalization_scaling,
    bench_supply_chain_canonicalization
);

criterion_main!(benches);
