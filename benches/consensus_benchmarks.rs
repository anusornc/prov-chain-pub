//! Criterion.rs benchmarks for consensus algorithms
//!
//! This provides statistical benchmarking similar to Elixir's Benchee
//! with detailed performance analysis, confidence intervals, and HTML reports.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use provchain_org::core::blockchain::Blockchain;
use std::time::Duration;

/// Generate test RDF data for benchmarking
fn generate_test_rdf_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

ex:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{:06}" ;
    trace:productType "Coffee" ;
    prov:wasAttributedTo ex:farmer{} .

ex:farmer{} a trace:Farmer ;
    trace:hasLocation ex:location{} .
"#,
                i,
                i,
                i % 100,
                i % 100,
                i
            )
        })
        .collect()
}

/// Benchmark ProvChain block creation performance
fn bench_block_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_creation");

    // Test different block sizes
    for size in [1, 5, 10, 25, 50].iter() {
        let test_data = generate_test_rdf_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("provchain_poa", size), size, |b, _| {
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

/// Benchmark RDF canonicalization performance
fn bench_rdf_canonicalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("rdf_canonicalization");

    // Test different complexity levels
    let test_scenarios = vec![
        ("simple", generate_simple_rdf_blocks(10)),
        ("complex", generate_complex_rdf_blocks(10)),
        ("supply_chain", generate_supply_chain_rdf_blocks(10)),
    ];

    for (scenario_name, test_blocks) in test_scenarios {
        group.bench_function(BenchmarkId::new("canonicalization", scenario_name), |b| {
            b.iter_batched(
                || {
                    let blockchain = Blockchain::new();
                    (blockchain, test_blocks.clone())
                },
                |(mut blockchain, blocks)| {
                    for block_data in blocks {
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

/// Benchmark SPARQL query performance
fn bench_sparql_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparql_queries");

    // Setup blockchain with test data
    let mut blockchain = Blockchain::new();
    let test_data = generate_supply_chain_rdf_blocks(50);
    for block_data in test_data {
        let _ = blockchain.add_block(block_data);
    }

    let queries = vec![
        (
            "simple_select",
            "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10",
        ),
        (
            "batch_trace",
            r#"
            PREFIX trace: <http://provchain.org/trace#>
            SELECT ?batch ?farmer WHERE {
                ?batch a trace:ProductBatch .
                ?batch trace:producedBy ?farmer .
            }
        "#,
        ),
        (
            "complex_join",
            r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            SELECT ?batch ?farmer ?location WHERE {
                ?batch a trace:ProductBatch .
                ?batch prov:wasAttributedTo ?farmer .
                ?farmer trace:hasLocation ?location .
            }
        "#,
        ),
    ];

    for (query_name, query) in queries {
        group.bench_function(BenchmarkId::new("query", query_name), |b| {
            b.iter(|| {
                let results = blockchain.rdf_store.query(black_box(query));
                black_box(results)
            });
        });
    }

    group.finish();
}

/// Benchmark blockchain scaling performance
fn bench_blockchain_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("blockchain_scaling");
    group.measurement_time(Duration::from_secs(10));

    // Test scaling with different chain lengths
    for chain_length in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*chain_length as u64));
        group.bench_with_input(
            BenchmarkId::new("scaling", chain_length),
            chain_length,
            |b, &size| {
                b.iter_batched(
                    || {
                        let mut blockchain = Blockchain::new();
                        let test_data = generate_test_rdf_data(size);

                        // Pre-populate blockchain
                        for (i, block_data) in test_data.iter().enumerate() {
                            if i < size - 1 {
                                let _ = blockchain.add_block(block_data.clone());
                            }
                        }

                        (blockchain, test_data.last().unwrap().clone())
                    },
                    |(mut blockchain, final_block)| {
                        let _ = blockchain.add_block(black_box(final_block));
                        black_box(blockchain)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark consensus algorithm comparison
fn bench_consensus_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_comparison");

    let block_count = 20;
    let test_data = generate_test_rdf_data(block_count);

    // ProvChain PoA (actual implementation)
    group.bench_function("provchain_poa", |b| {
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

    // Simulated PoW (for comparison)
    group.bench_function("simulated_pow", |b| {
        b.iter(|| {
            // Simulate mining delay
            std::thread::sleep(Duration::from_micros(100));
            black_box(())
        });
    });

    // Simulated PoS (for comparison)
    group.bench_function("simulated_pos", |b| {
        b.iter(|| {
            // Simulate validator selection and consensus
            std::thread::sleep(Duration::from_micros(50));
            black_box(())
        });
    });

    group.finish();
}

/// Generate simple RDF blocks for testing
fn generate_simple_rdf_blocks(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
ex:subject{} ex:predicate "value{}" .
"#,
                i, i
            )
        })
        .collect()
}

/// Generate complex RDF blocks with blank nodes
fn generate_complex_rdf_blocks(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

_:b{} ex:connects _:b{} .
_:b{} rdf:type ex:ComplexNode .
ex:root{} ex:hasChild _:b{} .
"#,
                i,
                (i + 1) % count,
                i,
                i,
                i
            )
        })
        .collect()
}

/// Generate supply chain RDF blocks
fn generate_supply_chain_rdf_blocks(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            format!(
                r#"
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

trace:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{:06}" ;
    prov:wasAttributedTo trace:farmer{} .

trace:farmer{} a trace:Farmer .
"#,
                i,
                i,
                i % 100,
                i % 100
            )
        })
        .collect()
}

criterion_group!(
    benches,
    bench_block_creation,
    bench_rdf_canonicalization,
    bench_sparql_queries,
    bench_blockchain_scaling,
    bench_consensus_comparison
);

criterion_main!(benches);
