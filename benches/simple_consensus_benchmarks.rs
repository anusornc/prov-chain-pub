//! Simple Criterion.rs benchmarks for consensus algorithms
//!
//! This provides basic statistical benchmarking for ProvChain consensus performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use provchain_org::core::blockchain::Blockchain;
use std::time::Duration;

/// Benchmark basic block creation performance
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

/// Benchmark SPARQL query performance
fn bench_sparql_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparql_queries");

    // Setup blockchain with test data
    let mut blockchain = Blockchain::new();
    let test_data = generate_supply_chain_data(20);
    for block_data in test_data {
        let _ = blockchain.add_block(block_data);
    }

    let queries = vec![
        (
            "simple_select",
            "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10",
        ),
        (
            "count_query",
            "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }",
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
    for chain_length in [10, 25, 50, 100].iter() {
        group.throughput(Throughput::Elements(*chain_length as u64));
        group.bench_with_input(
            BenchmarkId::new("scaling", chain_length),
            chain_length,
            |b, &size| {
                b.iter_batched(
                    || {
                        let blockchain = Blockchain::new();
                        let test_data = generate_test_rdf_data(size);
                        (blockchain, test_data)
                    },
                    |(mut blockchain, data)| {
                        for block_data in data {
                            let _ = blockchain.add_block(black_box(block_data));
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

/// Benchmark blockchain validation performance
fn bench_validation_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_performance");

    // Test validation on chains of different lengths
    for chain_length in [10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("validation", chain_length),
            chain_length,
            |b, &size| {
                // Pre-build blockchain
                let mut blockchain = Blockchain::new();
                let test_data = generate_test_rdf_data(size);
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

// Data generation functions

fn generate_test_rdf_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                r#"
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .

ex:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{}" ;
    trace:productType "Coffee" .
"#,
                i, i
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

trace:batch{} a trace:ProductBatch ;
    trace:hasBatchID "BATCH{}" ;
    trace:productType "Coffee" ;
    prov:wasAttributedTo trace:farmer{} .

trace:farmer{} a trace:Farmer .
"#,
                i,
                i,
                i % 10,
                i % 10
            )
        })
        .collect()
}

criterion_group!(
    benches,
    bench_block_creation,
    bench_sparql_queries,
    bench_blockchain_scaling,
    bench_validation_performance
);

criterion_main!(benches);
