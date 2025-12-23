//! Criterion.rs benchmarks for enhanced supply chain traceability optimization
//!
//! Benchmarks the performance improvements from SSSP-inspired frontier reduction
//! and pivot selection algorithms for supply chain traceability queries
//!
//! ## Performance Optimization Strategy
//!
//! These benchmarks use **adaptive configuration** to prevent hanging on large inputs:
//! - **Sample sizes scale down** as complexity increases (50 → 25 → 15 → 10)
//! - **Measurement time reduces** for larger datasets (20s → 15s → 12s → 10s)
//!
//! This ensures benchmarks complete in reasonable time while still providing meaningful
//! performance metrics across different scales.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use provchain_org::core::blockchain::Blockchain;
use provchain_org::trace_optimization::EnhancedTraceabilitySystem;
use std::time::Duration;

/// Benchmark enhanced trace performance with different optimization levels
fn bench_enhanced_trace_optimization_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("enhanced_trace_optimization");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(50);

    // Create a blockchain with realistic supply chain data
    let blockchain = create_supply_chain_blockchain(500);

    let optimization_scenarios = vec![
        ("no_optimization", 0u8),
        ("frontier_reduction", 1u8),
        ("pivot_selection", 2u8),
    ];

    for (scenario_name, optimization_level) in optimization_scenarios {
        group.bench_function(BenchmarkId::new("optimization_level", scenario_name), |b| {
            let system = EnhancedTraceabilitySystem::new(&blockchain);
            b.iter(|| {
                let result = system.enhanced_trace(black_box("001"), black_box(optimization_level));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark trace performance with different graph complexities
fn bench_trace_complexity_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("trace_complexity_scaling");

    let complexities = vec![
        ("linear_chain", 100usize),
        ("branched_chain", 200usize),
        ("merged_chain", 150usize),
        ("complex_network", 250usize),
    ];

    for (complexity_name, chain_size) in complexities {
        // Adaptive configuration based on chain size
        let (measurement_time, sample_size) = match chain_size {
            0..=150 => (Duration::from_secs(20), 30),
            151..=200 => (Duration::from_secs(15), 25),
            _ => (Duration::from_secs(12), 20), // 250
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);
        group.throughput(Throughput::Elements(chain_size as u64));

        group.bench_with_input(
            BenchmarkId::new("chain_complexity", complexity_name),
            &chain_size,
            |b, &_chain_size| {
                let blockchain = create_complex_supply_chain(_chain_size);
                let system = EnhancedTraceabilitySystem::new(&blockchain);

                b.iter(|| {
                    let result = system.enhanced_trace(black_box("001"), black_box(2));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark frontier reduction effectiveness
fn bench_frontier_reduction_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("frontier_reduction_effectiveness");

    let frontier_sizes = vec![50, 100, 200, 500, 1000];

    for &frontier_size in &frontier_sizes {
        // Adaptive configuration based on frontier size
        let (measurement_time, sample_size) = match frontier_size {
            0..=100 => (Duration::from_secs(15), 50),
            101..=200 => (Duration::from_secs(12), 30),
            201..=500 => (Duration::from_secs(10), 20),
            _ => (Duration::from_secs(10), 15), // 1000
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        group.bench_with_input(
            BenchmarkId::new("frontier_size", frontier_size),
            &frontier_size,
            |b, &_frontier_size| {
                let blockchain = create_supply_chain_blockchain(_frontier_size);
                let system = EnhancedTraceabilitySystem::new(&blockchain);

                b.iter(|| {
                    let result = system.enhanced_trace(black_box("001"), black_box(1));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark pivot selection performance
fn bench_pivot_selection_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pivot_selection_performance");

    let chain_sizes = vec![100, 200, 500];

    for &chain_size in &chain_sizes {
        // Adaptive configuration based on chain size
        let (measurement_time, sample_size) = match chain_size {
            0..=200 => (Duration::from_secs(15), 50),
            _ => (Duration::from_secs(12), 30), // 500
        };

        group.measurement_time(measurement_time);
        group.sample_size(sample_size);

        group.bench_with_input(
            BenchmarkId::new("chain_size", chain_size),
            &chain_size,
            |b, &_chain_size| {
                let blockchain = create_complex_supply_chain(_chain_size);
                let system = EnhancedTraceabilitySystem::new(&blockchain);

                b.iter(|| {
                    let result = system.enhanced_trace(black_box("001"), black_box(2));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark trace performance comparison: baseline vs optimized
fn bench_trace_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("trace_performance_comparison");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    // Create a large, complex supply chain
    let blockchain = create_complex_supply_chain(300);

    group.bench_function("baseline_trace", |b| {
        let system = EnhancedTraceabilitySystem::new(&blockchain);
        b.iter(|| {
            let result = system.enhanced_trace(black_box("001"), black_box(0));
            black_box(result)
        });
    });

    group.bench_function("optimized_trace", |b| {
        let system = EnhancedTraceabilitySystem::new(&blockchain);
        b.iter(|| {
            let result = system.enhanced_trace(black_box("001"), black_box(2));
            black_box(result)
        });
    });

    group.finish();
}

// Helper functions for creating test data

fn create_supply_chain_blockchain(size: usize) -> Blockchain {
    let mut blockchain = Blockchain::new();

    for i in 0..size {
        let data = format!(
            "@prefix : <http://example.org/> .\n\
            :batch{:04} a :ProductBatch ;\n\
                       :product \"Product {}\" ;\n\
                       :origin \"Farm {}\" ;\n\
                       :batchId \"BATCH{:04}\" .",
            i,
            i % 50,
            i % 20,
            i
        );

        let _ = blockchain.add_block(data);
    }

    blockchain
}

fn create_complex_supply_chain(size: usize) -> Blockchain {
    let mut blockchain = Blockchain::new();

    // Create a more complex supply chain with multiple stages
    for i in 0..size {
        let stage = i % 5; // 5 stages: farm, processing, transport, distribution, retail

        let data = match stage {
            0 => format!(
                "@prefix : <http://example.org/> .\n\
                :batch{:04} a :ProductBatch ;\n\
                           :product \"Raw Product {}\" ;\n\
                           :origin \"Farm {}\" ;\n\
                           :batchId \"BATCH{:04}\" .",
                i,
                i,
                i % 20,
                i
            ),

            1 => format!(
                "@prefix : <http://example.org/> .\n\
                :batch{:04} a :ProcessedBatch ;\n\
                           :product \"Processed Product {}\" ;\n\
                           :originBatch :batch{:04} ;\n\
                           :batchId \"BATCH{:04}\" .",
                i,
                i,
                (i + size - 1) % size,
                i
            ),

            2 => format!(
                "@prefix : <http://example.org/> .\n\
                :batch{:04} a :TransportedBatch ;\n\
                           :product \"Transported Product {}\" ;\n\
                           :originBatch :batch{:04} ;\n\
                           :batchId \"BATCH{:04}\" ;\n\
                           :transportVehicle \"TRUCK-{}\" .",
                i,
                i,
                (i + size - 1) % size,
                i,
                i % 50
            ),

            3 => format!(
                "@prefix : <http://example.org/> .\n\
                :batch{:04} a :DistributedBatch ;\n\
                           :product \"Distributed Product {}\" ;\n\
                           :originBatch :batch{:04} ;\n\
                           :batchId \"BATCH{:04}\" ;\n\
                           :warehouse \"WAREHOUSE-{}\" .",
                i,
                i,
                (i + size - 1) % size,
                i,
                i % 20
            ),

            _ => format!(
                "@prefix : <http://example.org/> .\n\
                :batch{:04} a :RetailBatch ;\n\
                           :product \"Retail Product {}\" ;\n\
                           :originBatch :batch{:04} ;\n\
                           :batchId \"BATCH{:04}\" ;\n\
                           :retailLocation \"STORE-{}\" .",
                i,
                i,
                (i + size - 1) % size,
                i,
                i % 50
            ),
        };

        let _ = blockchain.add_block(data);
    }

    blockchain
}

criterion_group!(
    benches,
    bench_enhanced_trace_optimization_levels,
    bench_trace_complexity_scaling,
    bench_frontier_reduction_effectiveness,
    bench_pivot_selection_performance,
    bench_trace_performance_comparison
);

criterion_main!(benches);
