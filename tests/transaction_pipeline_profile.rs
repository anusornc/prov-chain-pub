//! Transaction Pipeline Profiling Test
//!
//! This test profiles each step of the transaction submission pipeline
//! to identify bottlenecks affecting TPS performance.

use provchain_org::core::blockchain::Blockchain;
use std::time::Instant;

/// Generate test RDF data for a single transaction
fn generate_test_transaction(index: usize) -> String {
    format!(
        r#"@prefix ex: <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:product_{} a ex:Product ;
    ex:batchId "batch_{}" ;
    ex:timestamp "{}"^^xsd:dateTime ;
    ex:temperature 4.5 ;
    ex:location "warehouse_a" ."#,
        index,
        index,
        chrono::Utc::now().to_rfc3339()
    )
}

/// Profile single transaction submission with detailed timing
#[test]
fn profile_single_transaction() {
    println!("\n=== Single Transaction Submission Profile ===\n");

    let mut blockchain = Blockchain::new();
    let test_data = generate_test_transaction(0);

    // Profile complete transaction
    let start = Instant::now();
    let result = blockchain.add_block(test_data);
    let total_time = start.elapsed();

    println!("Result: {:?}", result);
    println!("Total Time: {:?}", total_time);
    println!("Estimated TPS: {:.2}", 1.0 / total_time.as_secs_f64());
}

/// Profile single transaction with batching (flush_interval=100)
#[test]
fn profile_single_transaction_batched() {
    println!("\n=== Single Transaction Submission Profile (Batched, flush_interval=100) ===\n");

    use provchain_org::storage::rdf_store::StorageConfig;
    use std::path::PathBuf;

    let config = StorageConfig {
        data_dir: PathBuf::from("./data/test_batched"),
        enable_backup: false,
        backup_interval_hours: 24,
        max_backup_files: 7,
        enable_compression: false,
        enable_encryption: false,
        cache_size: 1000,
        warm_cache_on_startup: false,
        flush_interval: 100, // Batch 100 blocks before flushing
    };

    let mut blockchain = Blockchain::new_persistent_with_config(config).unwrap();
    let test_data = generate_test_transaction(0);

    // Profile complete transaction
    let start = Instant::now();
    let result = blockchain.add_block(test_data);
    let total_time = start.elapsed();

    println!("Result: {:?}", result);
    println!("Total Time: {:?}", total_time);
    println!("Estimated TPS: {:.2}", 1.0 / total_time.as_secs_f64());
}

/// Profile batch transaction submission
#[test]
fn profile_batch_transactions() {
    println!("\n=== Batch Transaction Profile (100 transactions) ===\n");

    let batch_size = 100;
    let mut blockchain = Blockchain::new();

    let overall_start = Instant::now();

    for i in 0..batch_size {
        let test_data = generate_test_transaction(i);
        let _ = blockchain.add_block(test_data);
    }

    let overall_time = overall_start.elapsed();

    println!("Total transactions: {}", batch_size);
    println!("Total Time: {:?}", overall_time);
    println!(
        "Actual TPS: {:.2}",
        batch_size as f64 / overall_time.as_secs_f64()
    );
    println!(
        "Avg per transaction: {:?}",
        overall_time / batch_size as u32
    );
}

/// Profile batch transaction submission with batching (flush_interval=100)
#[test]
fn profile_batch_transactions_batched() {
    println!("\n=== Batch Transaction Profile (100 transactions, flush_interval=100) ===\n");

    use provchain_org::storage::rdf_store::StorageConfig;
    use std::path::PathBuf;

    let batch_size = 100;

    let config = StorageConfig {
        data_dir: PathBuf::from("./data/test_batched_batch"),
        enable_backup: false,
        backup_interval_hours: 24,
        max_backup_files: 7,
        enable_compression: false,
        enable_encryption: false,
        cache_size: 1000,
        warm_cache_on_startup: false,
        flush_interval: 100, // Batch 100 blocks before flushing - only one flush at the end
    };

    let mut blockchain = Blockchain::new_persistent_with_config(config).unwrap();

    let overall_start = Instant::now();

    for i in 0..batch_size {
        let test_data = generate_test_transaction(i);
        let _ = blockchain.add_block(test_data);
    }

    let overall_time = overall_start.elapsed();

    println!("Total transactions: {}", batch_size);
    println!("Total Time: {:?}", overall_time);
    println!(
        "Actual TPS: {:.2}",
        batch_size as f64 / overall_time.as_secs_f64()
    );
    println!(
        "Avg per transaction: {:?}",
        overall_time / batch_size as u32
    );
}
