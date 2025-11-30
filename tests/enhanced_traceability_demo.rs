//! Enhanced Traceability System Demonstration
//!
//! This test demonstrates the performance improvements from SSSP-inspired frontier reduction
//! and pivot selection algorithms for supply chain traceability queries

use provchain_org::core::blockchain::Blockchain;
use provchain_org::trace_optimization::EnhancedTraceabilitySystem;
use std::time::Instant;

/// Demonstrate enhanced traceability performance improvements
fn main() {
    println!("=== Enhanced Supply Chain Traceability System Demo ===\n");

    // Create a blockchain with realistic supply chain data
    let blockchain = create_supply_chain_blockchain(100);
    println!("Created blockchain with {} blocks", blockchain.chain.len());

    // Create enhanced traceability system
    let system = EnhancedTraceabilitySystem::new(&blockchain);

    // Demonstrate different optimization levels
    println!("--- Performance Comparison ---");

    // No optimization (baseline) - optimization level 0
    let start = Instant::now();
    let result_baseline = system.enhanced_trace("001", 0);
    let duration_baseline = start.elapsed();
    println!(
        "Baseline trace (no optimization): {:?} ({} entities explored)",
        duration_baseline, result_baseline.entities_explored
    );

    // Frontier reduction optimization - optimization level 1
    let start = Instant::now();
    let result_frontier = system.enhanced_trace("001", 1);
    let duration_frontier = start.elapsed();
    println!(
        "Frontier reduction trace: {:?} ({} entities explored)",
        duration_frontier, result_frontier.entities_explored
    );

    // Pivot selection optimization - optimization level 2
    let start = Instant::now();
    let result_pivot = system.enhanced_trace("001", 2);
    let duration_pivot = start.elapsed();
    println!(
        "Pivot selection trace: {:?} ({} entities explored)",
        duration_pivot, result_pivot.entities_explored
    );

    // Show sample results
    println!("\n--- Sample Trace Results ---");
    if !result_baseline.path.is_empty() {
        let sample_results: Vec<_> = result_baseline.path.iter().take(3).collect();
        for event in sample_results {
            println!(
                "  - {} -> {} -> {}",
                event.source.as_ref().unwrap_or(&"origin".to_string()),
                event.relationship,
                event.entity
            );
        }

        if result_baseline.path.len() > 3 {
            println!(
                "  ... and {} more trace events",
                result_baseline.path.len() - 3
            );
        }
    } else {
        println!("  No trace events found (this is expected with our simple test data)");
    }

    println!("\n=== Demo Complete ===");
    println!("Note: Performance improvements are more pronounced with complex supply chain data.");
    println!("See benchmarks for detailed performance analysis.");
}

/// Create a realistic supply chain blockchain for testing
fn create_supply_chain_blockchain(size: usize) -> Blockchain {
    let mut blockchain = Blockchain::new();

    // Add genesis block
    let _ = blockchain.add_block(
        "@prefix ex: <http://example.org/> . ex:genesis ex:type \"Genesis Block\".".to_string(),
    );

    for i in 0..size {
        let data = format!(
            "@prefix : <http://example.org/> .
            @prefix tc: <http://provchain.org/trace#> .
            @prefix prov: <http://www.w3.org/ns/prov#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            
            :batch{:04} a tc:ProductBatch ;
                       tc:product \"Product {}\" ;
                       tc:origin \"Farm {}\" ;
                       tc:batchId \"BATCH{:04}\" ;
                       tc:timestamp \"2024-01-{:02}T10:00:00Z\"^^xsd:dateTime ;
                       tc:status \"In Transit\" .
            
            :farmer{} a tc:Farmer ;
                     rdfs:label \"Farmer {}\" .
            
            :activity{:04} a tc:HarvestActivity ;
                          tc:recordedAt \"2024-01-{:02}T08:00:00Z\"^^xsd:dateTime ;
                          prov:wasAssociatedWith :farmer{} ;
                          prov:used :batch{:04} .",
            i,
            i % 50,
            i % 20,
            i,
            (i % 28) + 1,
            i,
            i,
            i,
            (i % 28) + 1,
            i,
            i
        );

        let _ = blockchain.add_block(data);
    }

    blockchain
}
