//! Criterion.rs benchmarks for overall blockchain performance
//!
//! Comprehensive performance benchmarks covering all aspects of ProvChain

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use provchain_org::core::blockchain::Blockchain;
use std::time::Duration;

/// Benchmark blockchain throughput with different transaction loads
fn bench_blockchain_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("blockchain_throughput");
    group.measurement_time(Duration::from_secs(15));

    // Test different transaction batch sizes
    for &batch_size in [1, 5, 10, 25, 50, 100].iter() {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_processing", batch_size),
            &batch_size,
            |b, &size| {
                let test_transactions = generate_transaction_batch(size);
                b.iter_batched(
                    || {
                        let blockchain = Blockchain::new();
                        (blockchain, test_transactions.clone())
                    },
                    |(mut blockchain, transactions)| {
                        for tx in transactions {
                            let _ = blockchain.add_block(black_box(tx));
                        }
                        black_box(blockchain)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark SPARQL query performance on growing blockchain
fn bench_query_performance_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_performance_scaling");
    group.measurement_time(Duration::from_secs(20));

    // Test query performance on blockchains of different sizes
    for &chain_size in [10, 50, 100, 200, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("chain_size", chain_size),
            &chain_size,
            |b, &size| {
                // Setup blockchain with data
                let mut blockchain = Blockchain::new();
                let test_data = generate_supply_chain_data(size);
                for block_data in test_data {
                    let _ = blockchain.add_block(block_data);
                }

                let complex_query = r#"
                    PREFIX trace: <http://provchain.org/trace#>
                    PREFIX prov: <http://www.w3.org/ns/prov#>
                    SELECT ?batch ?farmer ?product WHERE {
                        ?batch a trace:ProductBatch .
                        ?batch prov:wasAttributedTo ?farmer .
                        ?batch trace:hasProduct ?product .
                        ?farmer trace:hasLocation ?location .
                    }
                "#;

                b.iter(|| {
                    let results = blockchain.rdf_store.query(black_box(complex_query));
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark data complexity performance
fn bench_data_complexity_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_complexity_performance");

    // Test memory usage with different data types
    let memory_scenarios = vec![
        ("minimal_rdf", generate_minimal_rdf_data(100)),
        ("complex_rdf", generate_complex_rdf_data(50)),
        ("supply_chain", generate_supply_chain_data(75)),
        ("mixed_data", generate_mixed_data_types(60)),
    ];

    for (scenario_name, test_data) in memory_scenarios {
        group.bench_function(BenchmarkId::new("memory_usage", scenario_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_data.clone())
                },
                |(mut blockchain, data)| {
                    for block_data in data {
                        let _ = blockchain.add_block(black_box(block_data));
                    }

                    // Perform some operations to test memory efficiency
                    let _query_result = blockchain
                        .rdf_store
                        .query("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5");

                    black_box(blockchain)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark blockchain validation performance
fn bench_validation_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_performance");

    // Test validation on chains of different lengths
    for &chain_length in [10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("validation", chain_length),
            &chain_length,
            |b, &size| {
                // Pre-build blockchain
                let mut blockchain = Blockchain::new();
                let test_data = generate_transaction_batch(size);
                for block_data in test_data {
                    let _ = blockchain.add_block(block_data);
                }

                b.iter(|| {
                    let is_valid = blockchain.is_valid();
                    black_box(is_valid)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent operations simulation
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    group.bench_function("mixed_operations", |b| {
        b.iter_batched(
            || {
                let mut blockchain = Blockchain::new();
                // Pre-populate with some data
                let initial_data = generate_transaction_batch(20);
                for block_data in initial_data {
                    let _ = blockchain.add_block(block_data);
                }
                blockchain
            },
            |mut blockchain| {
                // Simulate concurrent operations

                // Add new blocks
                let new_blocks = generate_transaction_batch(5);
                for block_data in new_blocks {
                    let _ = blockchain.add_block(black_box(block_data));
                }

                // Perform queries
                let _query1 = blockchain
                    .rdf_store
                    .query("SELECT ?s WHERE { ?s a ?type } LIMIT 3");

                let _query2 = blockchain
                    .rdf_store
                    .query("SELECT COUNT(*) WHERE { ?s ?p ?o }");

                // Validate blockchain
                let _is_valid = blockchain.is_valid();

                black_box(blockchain)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark different RDF data complexity levels
fn bench_rdf_complexity_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdf_complexity_impact");

    let complexity_levels = vec![
        ("simple_triples", generate_simple_triples(30)),
        ("with_blank_nodes", generate_blank_node_data(25)),
        ("nested_structures", generate_nested_structures(20)),
        ("ontology_heavy", generate_ontology_heavy_data(15)),
    ];

    for (complexity_name, test_data) in complexity_levels {
        group.bench_function(BenchmarkId::new("complexity", complexity_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_data.clone())
                },
                |(mut blockchain, data)| {
                    for block_data in data {
                        let _ = blockchain.add_block(black_box(block_data));
                    }
                    black_box(blockchain)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark hash computation performance
fn bench_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_computation");

    // Test hash computation with different data sizes
    for &data_size in [100, 500, 1000, 5000].iter() {
        let test_data = generate_large_rdf_block(data_size);

        group.throughput(Throughput::Bytes(test_data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("hash_size", data_size),
            &test_data,
            |b, data| {
                b.iter_batched(
                    || {
                        let blockchain = Blockchain::new();
                        (blockchain, data.clone())
                    },
                    |(mut blockchain, block_data)| {
                        let _ = blockchain.add_block(black_box(block_data));
                        // Get the hash of the last block
                        let last_block = blockchain.chain.last().unwrap();
                        let _hash =
                            last_block.calculate_hash_with_store(Some(&blockchain.rdf_store));
                        black_box(blockchain)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// Data generation functions

fn generate_transaction_batch(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:transaction{} a trace:Transaction ;
    trace:hasID "TX{:08}" ;
    trace:hasTimestamp "2025-08-08T{:02}:{:02}:00Z"^^xsd:dateTime ;
    trace:hasValue "{}" ;
    trace:fromAccount "account{}" ;
    trace:toAccount "account{}" .
"#,
                i,
                i,
                i % 24,
                i % 60,
                i * 100,
                i % 1000,
                (i + 1) % 1000
            )
        })
        .collect()
}

fn generate_supply_chain_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix geo: <http://www.w3.org/2003/01/geo/wgs84_pos#> .

trace:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{:06}" ;
    trace:productType "Coffee" ;
    prov:wasAttributedTo trace:farmer{} ;
    trace:hasProduct trace:product{} .

trace:farmer{} a trace:Farmer ;
    trace:hasName "Farmer {}" ;
    trace:hasLocation trace:location{} .

trace:location{} a trace:GeographicLocation ;
    geo:lat "{:.6}" ;
    geo:long "{:.6}" .

trace:product{} a trace:Product ;
    trace:hasWeight "{}" ;
    trace:hasQuality "Grade A" .
"#,
                i,
                i,
                i % 100,
                i % 100,
                i % 100,
                i % 100,
                i % 100,
                i % 100,
                i % 100,
                40.0 + (i as f64 % 10.0),
                -74.0 + (i as f64 % 10.0),
                (i % 100) + 50
            )
        })
        .collect()
}

fn generate_minimal_rdf_data(size: usize) -> Vec<String> {
    (0..size)
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

fn generate_complex_rdf_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

ex:entity{} rdf:type ex:ComplexType ;
    rdfs:label "Complex Entity {}" ;
    ex:hasProperty _:blank{} ;
    ex:relatedTo ex:entity{} .

_:blank{} ex:nestedProperty "nested value {}" ;
    ex:connectsTo _:blank{} .

ex:collection{} rdf:type rdf:Bag ;
    rdf:_1 ex:entity{} ;
    rdf:_2 "literal value {}" .
"#,
                i,
                i,
                i,
                (i + 1) % size,
                i,
                (i + 1) % size,
                i,
                i,
                i,
                i
            )
        })
        .collect()
}

fn generate_mixed_data_types(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| match i % 3 {
            0 => format!(
                r#"
@prefix ex: <http://example.org/> .
ex:simple{} ex:hasValue "simple{}" .
"#,
                i, i
            ),
            1 => format!(
                r#"
@prefix trace: <http://provchain.org/trace#> .
trace:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{:06}" .
"#,
                i, i
            ),
            _ => format!(
                r#"
@prefix ex: <http://example.org/> .
_:complex{} ex:hasNested _:nested{} .
_:nested{} ex:value "complex{}" .
"#,
                i, i, i, i
            ),
        })
        .collect()
}

fn generate_simple_triples(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
ex:s{} ex:p{} ex:o{} .
ex:s{} ex:type "SimpleType" .
"#,
                i, i, i, i
            )
        })
        .collect()
}

fn generate_blank_node_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
ex:root{} ex:hasBlank _:b{} .
_:b{} ex:property "value{}" .
_:b{} ex:connectsTo _:b{} .
"#,
                i,
                i,
                i,
                i,
                i,
                (i + 1) % size
            )
        })
        .collect()
}

fn generate_nested_structures(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:container{} rdf:type ex:Container .
ex:container{} ex:contains ex:item{} .
ex:item{} ex:hasSubItem ex:subitem{} .
ex:subitem{} ex:hasProperty "nested property {}" .
ex:subitem{} ex:backRef ex:container{} .
"#,
                i, i, i, i, i, i, i, i, i
            )
        })
        .collect()
}

fn generate_ontology_heavy_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

ex:Class{} rdf:type owl:Class ;
    rdfs:subClassOf ex:SuperClass{} ;
    rdfs:label "Class {}" ;
    rdfs:comment "This is a complex ontological class" .

ex:instance{} rdf:type ex:Class{} ;
    ex:hasComplexProperty ex:propertyValue{} .

ex:propertyValue{} rdf:type ex:PropertyType ;
    rdfs:domain ex:Class{} ;
    rdfs:range ex:ValueType .
"#,
                i,
                i % 10,
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

fn generate_large_rdf_block(target_size: usize) -> String {
    let mut block = String::from(
        r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

"#,
    );

    let mut current_size = block.len();
    let mut i = 0;

    while current_size < target_size {
        let triple = format!(
            "ex:subject{} ex:predicate{} \"Large data value with lots of text to increase size {}\" .\n",
            i, i, i
        );
        block.push_str(&triple);
        current_size += triple.len();
        i += 1;
    }

    block
}

criterion_group!(
    benches,
    bench_blockchain_throughput,
    bench_query_performance_scaling,
    bench_data_complexity_performance,
    bench_validation_performance,
    bench_concurrent_operations,
    bench_rdf_complexity_impact,
    bench_hash_computation
);

criterion_main!(benches);
