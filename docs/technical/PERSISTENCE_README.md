# ProvChain Persistence System

This document describes the complete persistence system for Oxigraph data and blockchain state in ProvChain.

## Overview

The persistence system provides:
- **Persistent Storage**: RocksDB-backed storage for RDF data
- **Memory Cache**: Efficient in-memory caching for fast queries
- **Backup/Restore**: Automated backup and restore functionality
- **Data Integrity**: Verification and validation tools
- **Performance Optimization**: Configurable caching and indexing

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │────│   Memory Cache   │────│   RocksDB       │
│   Layer         │    │   (Oxigraph)     │    │   Storage       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │   Backup System  │
                    │   (File-based)   │
                    └──────────────────┘
```

## Quick Start

### Basic Usage

```rust
use provchain_org::rdf_store::{RDFStore, StorageConfig};

// Create persistent store
let mut store = RDFStore::new_persistent("data/storage")?;

// Add data
store.load_from_string(rdf_data, "text/turtle")?;

// Save to disk
store.save_to_disk()?;

// Load from disk
store.load_from_disk()?;

// Query data
let results = store.query_sparql("SELECT * WHERE { ?s ?p ?o }")?;
```

### Configuration

```rust
let config = StorageConfig {
    data_dir: PathBuf::from("data/storage"),
    enable_backup: true,
    backup_interval_hours: 24,
    max_backup_files: 5,
    enable_compression: false,
    cache_size: 1000,
    warm_cache_on_startup: true,
};
```

## Features

### 1. Persistent Storage
- **RocksDB Backend**: High-performance key-value storage
- **ACID Transactions**: Atomic operations for data consistency
- **Compression**: Optional compression for storage efficiency
- **Encryption**: Optional encryption for security

### 2. Memory Cache
- **LRU Cache**: Least Recently Used eviction policy
- **Warm Cache**: Pre-load cache on startup
- **Statistics**: Cache hit/miss tracking
- **Configurable Size**: Memory usage limits

### 3. Backup System
- **Automatic Backups**: Scheduled backup creation
- **Multiple Versions**: Keep configurable number of backups
- **Compression**: Optional backup compression
- **Restore**: Point-in-time restore capability

### 4. Data Integrity
- **Checksum Verification**: Detect data corruption
- **Schema Validation**: Validate RDF data structure
- **Consistency Checks**: Ensure referential integrity
- **Error Reporting**: Detailed integrity reports

## API Reference

### RDFStore Methods

#### `new_persistent(path: impl AsRef<Path>) -> Result<Self>`
Create a new persistent RDF store with default configuration.

#### `new_persistent_with_config(config: StorageConfig) -> Result<Self>`
Create a new persistent RDF store with custom configuration.

#### `save_to_disk(&mut self) -> Result<()>`
Save all data to disk.

#### `load_from_disk(&mut self) -> Result<()>`
Load all data from disk.

#### `create_backup(&self) -> Result<BackupInfo>`
Create a new backup.

#### `restore_from_backup(backup_path: &Path, target_path: &Path) -> Result<Self>`
Restore from a backup.

#### `check_integrity(&self) -> Result<IntegrityReport>`
Verify data integrity.

#### `get_storage_stats(&self) -> Result<StorageStats>`
Get storage statistics.

### StorageConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `data_dir` | `PathBuf` | Required | Base directory for storage |
| `enable_backup` | `bool` | `true` | Enable automatic backups |
| `backup_interval_hours` | `u32` | `24` | Backup interval in hours |
| `max_backup_files` | `u32` | `5` | Maximum backup files to keep |
| `enable_compression` | `bool` | `false` | Enable compression |
| `enable_encryption` | `bool` | `false` | Enable encryption |
| `cache_size` | `usize` | `1000` | Cache size in entries |
| `warm_cache_on_startup` | `bool` | `true` | Pre-load cache on startup |

## Usage Examples

### Basic Persistence

```rust
use provchain_org::rdf_store::RDFStore;

let mut store = RDFStore::new_persistent("data/my_store")?;

// Add some data
let data = r#"
    @prefix ex: <http://example.org/> .
    ex:product1 ex:name "Milk" ;
                ex:batch "BATCH001" .
"#;

store.load_from_string(data, "text/turtle")?;
store.save_to_disk()?;

// Later, or after restart
let mut store = RDFStore::new_persistent("data/my_store")?;
store.load_from_disk()?;
```

### Supply Chain Traceability

```rust
use provchain_org::rdf_store::{RDFStore, StorageConfig};
use provchain_org::blockchain::Blockchain;

// Configure for supply chain use
let config = StorageConfig {
    data_dir: "supply_chain_data".into(),
    enable_backup: true,
    max_backup_files: 10,
    cache_size: 5000,
    ..Default::default()
};

// Create persistent blockchain
let mut blockchain = Blockchain::new_persistent_with_config(config)?;

// Add production data
let production_rdf = r#"
    @prefix ex: <http://example.org/> .
    ex:milk_batch_001 ex:producedBy ex:farm_001 ;
                      ex:productionDate "2024-01-15" ;
                      ex:quantity "1000L" .
"#;

blockchain.add_block(production_rdf);
blockchain.rdf_store.save_to_disk()?;

// Query trace data
let results = blockchain.rdf_store.query_sparql(
    "SELECT ?batch ?producer WHERE { ?batch ex:producedBy ?producer }"
)?;
```

### Backup and Restore

```rust
use provchain_org::rdf_store::RDFStore;

let mut store = RDFStore::new_persistent("data/production")?;

// Create backup
let backup_info = store.create_backup()?;
println!("Backup created: {}", backup_info.path.display());

// List backups
let backups = store.list_backups()?;
for backup in backups {
    println!("Available backup: {}", backup.name);
}

// Restore from backup
let restored = RDFStore::restore_from_backup(
    &backup_info.path,
    "data/restored"
)?;
```

## Performance Tuning

### Cache Configuration
```rust
let config = StorageConfig {
    cache_size: 10000,        // Larger cache for better performance
    warm_cache_on_startup: true,
    ..Default::default()
};
```

### Backup Strategy
```rust
let config = StorageConfig {
    enable_backup: true,
    backup_interval_hours: 6,  // More frequent backups
    max_backup_files: 20,      // Keep more versions
    enable_compression: true,  // Save disk space
    ..Default::default()
};
```

## Testing

Run the persistence tests:
```bash
cargo test --test persistence_integration_tests
```

Run the demo:
```bash
cargo run --example persistence_demo
```

## Migration

### From In-Memory to Persistent
```rust
// Old in-memory store
let old_store = RDFStore::new();

// Migrate to persistent
let mut new_store = RDFStore::new_persistent("data/migrated")?;
new_store.store = old_store.store;
new_store.save_to_disk()?;
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Ensure the data directory has write permissions
   - Check file system permissions

2. **Out of Memory**
   - Reduce cache_size in configuration
   - Monitor memory usage with get_storage_stats()

3. **Corrupted Data**
   - Use check_integrity() to detect issues
   - Restore from backup if needed

4. **Slow Performance**
   - Increase cache_size
   - Enable compression for large datasets
   - Consider SSD storage

### Debug Mode
Enable debug logging:
```rust
env_logger::init();
let mut store = RDFStore::new_persistent("data/debug")?;
```

## Security Considerations

- **Encryption**: Enable encryption for sensitive data
- **Access Control**: Implement file system permissions
- **Backup Security**: Secure backup storage locations
- **Audit Trail**: Log all persistence operations

## Best Practices

1. **Regular Backups**: Enable automatic backups
2. **Integrity Checks**: Run periodic integrity verification
3. **Monitoring**: Track storage statistics
4. **Testing**: Test restore procedures regularly
5. **Documentation**: Document your data schema

## Support

For issues and questions:
- Check the test files for usage examples
- Review the API documentation
- Run the demo for hands-on experience
