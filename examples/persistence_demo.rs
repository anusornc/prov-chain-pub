use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::{RDFStore as RdfStore, StorageConfig};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== ProvChain Persistence Demo ===\n");

    // 1. Setup storage directory
    let storage_dir = Path::new("data/rdf_store");
    if storage_dir.exists() {
        println!("Cleaning existing demo directory...");
        std::fs::remove_dir_all(storage_dir)?;
    }

    // 2. Create storage configuration
    let config = StorageConfig {
        data_dir: storage_dir.to_path_buf(),
        enable_backup: true,
        backup_interval_hours: 24,
        max_backup_files: 3,
        enable_compression: false,
        enable_encryption: false,
        cache_size: 1000,
        warm_cache_on_startup: true,
    };

    println!("1. Creating persistent RDF store...");
    let mut store = RdfStore::new_persistent_with_config(config)?;

    // 3. Add supply chain data
    println!("2. Adding supply chain data...");
    let supply_chain_data = r#"
        <http://example.org/milk_batch_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/MilkBatch> .
        <http://example.org/milk_batch_001> <http://provchain.org/batchId> "BATCH001" .
        <http://example.org/milk_batch_001> <http://provchain.org/producedBy> <http://example.org/farm_organic_dairy> .
        <http://example.org/milk_batch_001> <http://provchain.org/productionDate> "2024-01-15" .
        <http://example.org/milk_batch_001> <http://provchain.org/quantity> "1000" .
        <http://example.org/milk_batch_001> <http://provchain.org/unit> "liters" .
        
        <http://example.org/cheese_batch_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/CheeseBatch> .
        <http://example.org/cheese_batch_001> <http://provchain.org/batchId> "CHEESE001" .
        <http://example.org/cheese_batch_001> <http://provchain.org/madeFrom> <http://example.org/milk_batch_001> .
        <http://example.org/cheese_batch_001> <http://provchain.org/producedBy> <http://example.org/cheese_factory_001> .
        <http://example.org/cheese_batch_001> <http://provchain.org/productionDate> "2024-01-20" .
        <http://example.org/cheese_batch_001> <http://provchain.org/quantity> "50" .
        <http://example.org/cheese_batch_001> <http://provchain.org/unit> "kg" .
        
        <http://example.org/farm_organic_dairy> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/Farm> .
        <http://example.org/farm_organic_dairy> <http://provchain.org/name> "Organic Dairy Farm" .
        <http://example.org/farm_organic_dairy> <http://provchain.org/location> "Wisconsin, USA" .
        <http://example.org/farm_organic_dairy> <http://provchain.org/certification> "USDA Organic" .
        
        <http://example.org/cheese_factory_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/ProcessingFacility> .
        <http://example.org/cheese_factory_001> <http://provchain.org/name> "Artisan Cheese Factory" .
        <http://example.org/cheese_factory_001> <http://provchain.org/location> "Vermont, USA" .
        <http://example.org/cheese_factory_001> <http://provchain.org/certification> "FDA Approved" .
    "#;

    store.load_data_from_string(supply_chain_data)?;
    println!("   ✓ Added {} triples to store", store.store.len()?);

    // 4. Save to disk
    println!("3. Saving data to disk...");
    store.save_to_disk()?;
    println!("   ✓ Data saved to {}", storage_dir.display());

    // 5. Create blockchain with persistent storage
    println!("4. Creating persistent blockchain...");
    let mut blockchain = Blockchain::new_persistent(storage_dir)?;

    // 6. Add blockchain blocks
    println!("5. Adding blockchain blocks...");
    let production_data = r#"
        <http://example.org/production_event_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/ProductionEvent> .
        <http://example.org/production_event_001> <http://provchain.org/product> <http://example.org/milk_batch_001> .
        <http://example.org/production_event_001> <http://provchain.org/timestamp> "2024-01-15T08:00:00Z" .
        <http://example.org/production_event_001> <http://provchain.org/location> <http://example.org/farm_organic_dairy> .
        <http://example.org/production_event_001> <http://provchain.org/operator> "John Farmer" .
    "#;

    let _ = blockchain.add_block(production_data.to_string());

    let processing_data = r#"
        <http://example.org/processing_event_001> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/ProcessingEvent> .
        <http://example.org/processing_event_001> <http://provchain.org/input> <http://example.org/milk_batch_001> .
        <http://example.org/processing_event_001> <http://provchain.org/output> <http://example.org/cheese_batch_001> .
        <http://example.org/processing_event_001> <http://provchain.org/timestamp> "2024-01-20T10:00:00Z" .
        <http://example.org/processing_event_001> <http://provchain.org/location> <http://example.org/cheese_factory_001> .
        <http://example.org/processing_event_001> <http://provchain.org/operator> "Jane Cheesemaker" .
    "#;

    let _ = blockchain.add_block(processing_data.to_string());

    println!("   ✓ Added {} blocks to blockchain", blockchain.chain.len());

    // 7. Save blockchain state
    println!("6. Saving blockchain state...");
    blockchain.rdf_store.save_to_disk()?;

    // 8. Create backup
    println!("7. Creating backup...");
    let backup_info = store.create_backup()?;
    println!("   ✓ Backup created at {}", backup_info.path.display());

    // 9. List backups
    println!("8. Available backups:");
    let backups = store.list_backups()?;
    for backup in backups {
        println!(
            "   - {} ({} bytes)",
            backup.path.display(),
            backup.size_bytes
        );
    }

    // 10. Load from disk (simulating restart)
    println!("9. Loading data from disk (simulating restart)...");
    let new_store = RdfStore::new_persistent(storage_dir)?;
    // Note: load_from_disk is called automatically in new_persistent

    println!("   ✓ Loaded {} triples from disk", new_store.store.len()?);

    // 11. Verify data integrity
    println!("10. Verifying data integrity...");
    let integrity_report = new_store.check_integrity()?;
    if integrity_report.errors.is_empty() {
        println!("   ✓ Data integrity verified");
    } else {
        println!("   ⚠ Data integrity issues found:");
        for error in &integrity_report.errors {
            println!("     - {}", error);
        }
    }

    // 12. Perform trace query
    println!("11. Performing trace query...");
    let trace_query = r#"
        SELECT ?product ?batchId ?producer ?date WHERE {
            ?product a <http://provchain.org/MilkBatch> ;
                     <http://provchain.org/batchId> ?batchId ;
                     <http://provchain.org/producedBy> ?producer ;
                     <http://provchain.org/productionDate> ?date .
        }
    "#;

    let results = new_store.query(trace_query);
    let result_count = match results {
        oxigraph::sparql::QueryResults::Solutions(solutions) => solutions.count(),
        _ => 0,
    };
    println!("   ✓ Found {} products matching trace query", result_count);

    // 13. Get storage statistics
    println!("12. Storage statistics:");
    let stats = new_store.get_storage_stats()?;
    println!("   - Total triples: {}", stats.quad_count);
    if let Some(disk_usage) = stats.disk_usage_bytes {
        println!("   - Storage size: {} bytes", disk_usage);
    }
    println!("   - Backup count: {}", stats.backup_count);

    // 14. Performance benchmark
    println!("13. Performance benchmark...");
    let start = std::time::Instant::now();
    let _ = new_store.query("SELECT * WHERE { ?s ?p ?o } LIMIT 100");
    let query_time = start.elapsed();
    println!("   ✓ Query executed in {:?}", query_time);

    println!("\n=== Demo completed successfully! ===");
    println!("All data has been persisted to: {}", storage_dir.display());

    Ok(())
}
