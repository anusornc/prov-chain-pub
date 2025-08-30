use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::StorageConfig;
use tempfile::tempdir;

#[test]
fn test_backup_restore_functionality() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("test_data");
    
    // Create storage configuration
    let config = StorageConfig {
        data_dir: data_dir.clone(),
        enable_backup: true,
        backup_interval_hours: 24,
        max_backup_files: 3,
        enable_compression: false,
        enable_encryption: false,
        cache_size: 100,
        warm_cache_on_startup: true,
    };
    
    // Create a persistent blockchain
    let mut blockchain = Blockchain::new_persistent_with_config(config.clone()).unwrap();
    
    // Add some test data
    let test_data1 = r#"
        @prefix ex: <http://example.org/> .
        ex:product1 ex:name "Test Product 1" .
        ex:product1 ex:batch "BATCH001" .
    "#.to_string();
    
    let _ = blockchain.add_block(test_data1);
    
    let test_data2 = r#"
        @prefix ex: <http://example.org/> .
        ex:product2 ex:name "Test Product 2" .
        ex:product2 ex:batch "BATCH002" .
    "#.to_string();
    
    let _ = blockchain.add_block(test_data2);
    
    // Create a backup
    let backup_info = blockchain.create_backup("test_backup_001".to_string()).unwrap();
    assert!(backup_info.path.exists());
    
    // Create a new temporary directory for restore
    let restore_dir = tempdir().unwrap();
    let restore_path = restore_dir.path().join("restored_data");
    
    // Restore from backup
    let restored_blockchain = Blockchain::restore_from_backup(&backup_info.path, &restore_path).unwrap();
    
    // Verify we can query the restored data (basic check)
    let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 1";
    let results = restored_blockchain.rdf_store.query(query);
    let has_data = match results {
        oxigraph::sparql::QueryResults::Solutions(solutions) => solutions.count() > 0,
        _ => false,
    };
    assert!(has_data);
    
    // Verify backup info is valid
    assert!(backup_info.size_bytes > 0);
    assert!(backup_info.path.exists());
}
