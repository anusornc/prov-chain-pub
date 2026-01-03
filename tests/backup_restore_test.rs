use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::{StorageConfig, BackupInfo};
use tempfile::tempdir;
use std::fs;
use std::io::Write;

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
    "#
    .to_string();

    let _ = blockchain.add_block(test_data1);

    let test_data2 = r#"
        @prefix ex: <http://example.org/> .
        ex:product2 ex:name "Test Product 2" .
        ex:product2 ex:batch "BATCH002" .
    "#
    .to_string();

    let _ = blockchain.add_block(test_data2);

    // Create a backup
    let backup_info = blockchain
        .create_backup("test_backup_001".to_string())
        .unwrap();
    assert!(backup_info.path.exists());

    // Create a new temporary directory for restore
    let restore_dir = tempdir().unwrap();
    let restore_path = restore_dir.path().join("restored_data");

    // Restore from backup
    let restored_blockchain =
        Blockchain::restore_from_backup(&backup_info.path, &restore_path).unwrap();

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

#[test]
fn test_backup_restore_with_corrupted_backup() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("test_data");

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

    // Create blockchain and add data
    let mut blockchain = Blockchain::new_persistent_with_config(config.clone()).unwrap();
    let test_data = r#"
        @prefix ex: <http://example.org/> .
        ex:product1 ex:name "Test Product" .
    "#.to_string();
    let _ = blockchain.add_block(test_data);

    // Create backup
    let backup_info = blockchain.create_backup("test_backup".to_string()).unwrap();

    // Corrupt the backup file
    {
        let mut backup_file = fs::File::create(&backup_info.path).unwrap();
        backup_file.write_all(b"CORRUPTED DATA!!!").unwrap();
    }

    // Try to restore from corrupted backup
    let restore_dir = tempdir().unwrap();
    let restore_path = restore_dir.path().join("restored_data");
    let result = Blockchain::restore_from_backup(&backup_info.path, &restore_path);

    // Should fail gracefully with an error
    assert!(result.is_err(), "Restore from corrupted backup should fail");
}

#[test]
fn test_backup_restore_with_missing_backup_file() {
    let temp_dir = tempdir().unwrap();
    let restore_path = temp_dir.path().join("restored_data");
    let non_existent_backup = temp_dir.path().join("non_existent_backup.db");

    // Try to restore from non-existent backup
    let result = Blockchain::restore_from_backup(&non_existent_backup, &restore_path);

    // Should fail with an appropriate error
    assert!(result.is_err(), "Restore from missing backup should fail");
}

#[test]
fn test_backup_list_and_management() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("test_data");

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

    let mut blockchain = Blockchain::new_persistent_with_config(config.clone()).unwrap();

    // Create multiple backups
    let backup1 = blockchain.create_backup("backup_001".to_string());
    let backup2 = blockchain.create_backup("backup_002".to_string());
    let backup3 = blockchain.create_backup("backup_003".to_string());

    assert!(backup1.is_ok());
    assert!(backup2.is_ok());
    assert!(backup3.is_ok());

    // List available backups
    let backups = blockchain.list_backups().expect("Should list backups successfully");
    assert!(backups.len() >= 3, "Should have at least 3 backups");
}

#[test]
fn test_backup_with_max_backups_limit() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("test_data");

    let config = StorageConfig {
        data_dir: data_dir.clone(),
        enable_backup: true,
        backup_interval_hours: 24,
        max_backup_files: 2, // Limit to 2 backups
        enable_compression: false,
        enable_encryption: false,
        cache_size: 100,
        warm_cache_on_startup: true,
    };

    let mut blockchain = Blockchain::new_persistent_with_config(config).unwrap();

    // Add some data
    let test_data = r#"
        @prefix ex: <http://example.org/> .
        ex:product1 ex:name "Test Product" .
    "#.to_string();
    let _ = blockchain.add_block(test_data);

    // Create more backups than max_backup_files
    let _ = blockchain.create_backup("backup_001".to_string()).unwrap();
    let _ = blockchain.create_backup("backup_002".to_string()).unwrap();
    let _ = blockchain.create_backup("backup_003".to_string()).unwrap();

    // Verify only max_backup_files exist
    let backups = blockchain.list_backups().expect("Should list backups successfully");
    assert!(backups.len() <= 2, "Should not exceed max_backup_files limit");
}

#[test]
fn test_backup_restore_with_large_blockchain() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("test_data");

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

    let mut blockchain = Blockchain::new_persistent_with_config(config).unwrap();

    // Add multiple blocks
    for i in 0..10 {
        let test_data = format!(
            r#"
            @prefix ex: <http://example.org/> .
            ex:product{} ex:name "Test Product {}" .
            ex:product{} ex:batch "BATCH{:03}" .
        "#,
            i, i, i, i
        );
        let _ = blockchain.add_block(test_data);
    }

    // Create backup
    let backup_info = blockchain.create_backup("large_backup".to_string()).unwrap();

    // Verify backup is substantial size
    assert!(backup_info.size_bytes > 100, "Backup should contain data");

    // Restore and verify
    let restore_dir = tempdir().unwrap();
    let restore_path = restore_dir.path().join("restored_data");
    let restored_blockchain =
        Blockchain::restore_from_backup(&backup_info.path, &restore_path).unwrap();

    // Verify we have data in restored blockchain
    let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
    let results = restored_blockchain.rdf_store.query(query);
    let count = match results {
        oxigraph::sparql::QueryResults::Solutions(solutions) => solutions.count(),
        _ => 0,
    };
    assert!(count > 0, "Restored blockchain should contain data");
}
