//! Persistent Storage with WAL (Write-Ahead Log)
//!
//! This module provides durable, atomic storage for blockchain data with:
//! - Write-Ahead Logging (WAL) for crash recovery
//! - Atomic block writes with checksums
//! - Separate chain index from RDF data
//! - RocksDB backend for Oxigraph

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tracing::{debug, info, warn};

/// Version of the persistence format
const PERSISTENCE_VERSION: u32 = 1;

/// Magic bytes for WAL file validation
const WAL_MAGIC: &[u8] = b"PROVCHAIN_WAL\x01";
const DEFAULT_SYNC_INTERVAL: u64 = 1;

/// Entry types for WAL
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum WalEntryType {
    /// Block data entry
    Block = 0x01,
    /// Chain metadata entry
    ChainMetadata = 0x02,
    /// Checkpoint entry
    Checkpoint = 0x03,
    /// Delete marker (for future use)
    Delete = 0x04,
}

/// A single WAL entry with checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Entry type
    pub entry_type: WalEntryType,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Timestamp (nanoseconds since epoch)
    pub timestamp: u128,
    /// Actual data
    pub data: Vec<u8>,
    /// CRC32 checksum of data
    pub checksum: u32,
}

impl WalEntry {
    /// Create a new WAL entry
    pub fn new(entry_type: WalEntryType, sequence: u64, data: Vec<u8>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let checksum = Self::calculate_checksum(&data);

        Self {
            entry_type,
            sequence,
            timestamp,
            data,
            checksum,
        }
    }

    /// Calculate CRC32 checksum of data
    fn calculate_checksum(data: &[u8]) -> u32 {
        crc32fast::hash(data)
    }

    /// Verify entry integrity
    pub fn verify(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }

    /// Serialize entry to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let data = bincode::serialize(self).context("Failed to serialize WAL entry")?;
        Ok(data)
    }

    /// Deserialize entry from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).context("Failed to deserialize WAL entry")
    }
}

/// Block metadata for chain index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Block index
    pub index: u64,
    /// Block hash
    pub hash: String,
    /// Previous block hash
    pub previous_hash: String,
    /// Block timestamp
    pub timestamp: String,
    /// Validator public key
    pub validator: String,
    /// Block signature
    pub signature: String,
    /// State root hash
    pub state_root: String,
    /// RDF data graph URI
    pub data_graph_uri: String,
    /// Encrypted data flag
    pub has_encrypted_data: bool,
    /// Size of RDF data in bytes
    pub data_size: u64,
}

/// Chain metadata checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCheckpoint {
    /// Block height (number of blocks)
    pub block_height: u64,
    /// Hash of the latest block
    pub latest_block_hash: String,
    /// Merkle root of all block hashes
    pub chain_merkle_root: String,
    /// Timestamp of checkpoint
    pub checkpoint_time: u128,
}

/// Persistent storage with WAL
#[derive(Debug)]
pub struct PersistentStorage {
    /// Data directory
    data_dir: PathBuf,
    /// WAL file path
    wal_path: PathBuf,
    /// Chain index file path
    chain_index_path: PathBuf,
    /// RDF data directory
    rdf_data_dir: PathBuf,
    /// Current WAL sequence number
    sequence: AtomicU64,
    /// WAL file handle (buffered)
    wal_file: Mutex<File>,
    /// Sync interval (fsync every N entries)
    sync_interval: u64,
    /// Entries since last sync
    entries_since_sync: AtomicU64,
    /// Chain index fsync interval. Defaults to the WAL sync interval.
    chain_index_sync_interval: u64,
    /// Chain index entries since last fsync.
    chain_index_entries_since_sync: AtomicU64,
}

impl PersistentStorage {
    /// Open or create persistent storage at the given directory
    pub fn open<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let (wal_sync_interval, chain_index_sync_interval) = Self::configured_sync_intervals();
        Self::open_with_sync_intervals(data_dir, wal_sync_interval, chain_index_sync_interval)
    }

    /// Open storage with an explicit fsync interval.
    ///
    /// The default public `open` path keeps the conservative one-block fsync
    /// behavior unless `PROVCHAIN_WAL_SYNC_INTERVAL` is set to a positive value.
    pub fn open_with_sync_interval<P: AsRef<Path>>(
        data_dir: P,
        sync_interval: u64,
    ) -> Result<Self> {
        Self::open_with_sync_intervals(data_dir, sync_interval, sync_interval)
    }

    /// Open storage with explicit WAL and chain-index fsync intervals.
    pub fn open_with_sync_intervals<P: AsRef<Path>>(
        data_dir: P,
        wal_sync_interval: u64,
        chain_index_sync_interval: u64,
    ) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let wal_sync_interval = wal_sync_interval.max(1);
        let chain_index_sync_interval = chain_index_sync_interval.max(1);

        // Create directories
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {}", data_dir.display()))?;

        let rdf_data_dir = data_dir.join("rdf");
        fs::create_dir_all(&rdf_data_dir).with_context(|| {
            format!("Failed to create RDF directory: {}", rdf_data_dir.display())
        })?;

        let wal_path = data_dir.join("wal.dat");
        let chain_index_path = data_dir.join("chain.index");

        // Open WAL file (append mode)
        let wal_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&wal_path)
            .with_context(|| format!("Failed to open WAL file: {}", wal_path.display()))?;

        let storage = Self {
            data_dir,
            wal_path,
            chain_index_path,
            rdf_data_dir,
            sequence: AtomicU64::new(0),
            wal_file: Mutex::new(wal_file),
            sync_interval: wal_sync_interval,
            entries_since_sync: AtomicU64::new(0),
            chain_index_sync_interval,
            chain_index_entries_since_sync: AtomicU64::new(0),
        };

        // Initialize or recover
        storage.initialize_or_recover()?;
        info!(
            "Persistent storage fsync intervals configured: wal={}, chain_index={}",
            wal_sync_interval, chain_index_sync_interval
        );

        Ok(storage)
    }

    fn configured_sync_intervals() -> (u64, u64) {
        let wal_sync_interval = Self::configured_positive_interval(
            "PROVCHAIN_WAL_SYNC_INTERVAL",
            DEFAULT_SYNC_INTERVAL,
        );
        let chain_index_sync_interval = Self::configured_positive_interval(
            "PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL",
            wal_sync_interval,
        );
        (wal_sync_interval, chain_index_sync_interval)
    }

    fn configured_positive_interval(name: &str, default: u64) -> u64 {
        match std::env::var(name) {
            Ok(value) => match value.parse::<u64>() {
                Ok(interval) if interval > 0 => interval,
                _ => {
                    warn!(
                        "Ignoring invalid {}='{}'; using default {}",
                        name, value, default
                    );
                    default
                }
            },
            Err(_) => default,
        }
    }

    /// Initialize new storage or recover existing
    fn initialize_or_recover(&self) -> Result<()> {
        // Check if WAL exists and has data
        let wal_size = fs::metadata(&self.wal_path)?.len();

        if wal_size == 0 {
            // New WAL - write header
            self.write_wal_header()?;
            info!("Initialized new WAL at {}", self.wal_path.display());
        } else {
            // Recover from existing WAL
            self.recover()?;
        }

        Ok(())
    }

    /// Write WAL header
    fn write_wal_header(&self) -> Result<()> {
        let mut file = self
            .wal_file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WAL file"))?;

        // Write magic bytes
        file.write_all(WAL_MAGIC)?;

        // Write version
        file.write_all(&PERSISTENCE_VERSION.to_le_bytes())?;

        // Write sequence counter (starts at 0)
        file.write_all(&0u64.to_le_bytes())?;

        file.sync_all().context("Failed to sync WAL header")?;

        Ok(())
    }

    /// Recover from WAL after crash
    fn recover(&self) -> Result<()> {
        info!("Recovering from WAL: {}", self.wal_path.display());

        let file = File::open(&self.wal_path).context("Failed to open WAL for recovery")?;
        let mut reader = BufReader::new(file);

        // Verify magic bytes
        let mut magic = [0u8; 14];
        reader.read_exact(&mut magic)?;
        if &magic != WAL_MAGIC {
            return Err(anyhow::anyhow!("Invalid WAL file format"));
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != PERSISTENCE_VERSION {
            warn!(
                "WAL version mismatch: expected {}, found {}",
                PERSISTENCE_VERSION, version
            );
        }

        // Read entries and find highest sequence number
        let mut highest_sequence: u64 = 0;
        let mut entry_count: u64 = 0;
        let mut corrupt_entries: u64 = 0;

        loop {
            // Read entry length (8 bytes)
            let mut len_bytes = [0u8; 8];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(_) => break, // EOF or error
            }
            let entry_len = u64::from_le_bytes(len_bytes) as usize;

            if entry_len == 0 || entry_len > 100_000_000 {
                // Invalid entry size, likely corruption
                corrupt_entries += 1;
                warn!(
                    "Invalid entry size: {} at position {:?}",
                    entry_len,
                    reader.stream_position()
                );
                break;
            }

            // Read entry data
            let mut entry_data = vec![0u8; entry_len];
            if let Err(e) = reader.read_exact(&mut entry_data) {
                warn!("Failed to read entry data: {}. Truncated WAL?", e);
                corrupt_entries += 1;
                break;
            }

            // Parse and verify entry
            match WalEntry::from_bytes(&entry_data) {
                Ok(entry) => {
                    if entry.verify() {
                        highest_sequence = highest_sequence.max(entry.sequence);
                        entry_count += 1;
                    } else {
                        corrupt_entries += 1;
                        warn!("Checksum mismatch for entry at sequence {}", entry.sequence);
                    }
                }
                Err(e) => {
                    corrupt_entries += 1;
                    warn!("Failed to parse entry: {}", e);
                }
            }
        }

        // Set sequence counter
        self.sequence.store(highest_sequence + 1, Ordering::SeqCst);

        info!(
            "WAL recovery complete: {} entries, {} corrupt, next sequence: {}",
            entry_count,
            corrupt_entries,
            highest_sequence + 1
        );

        // Rebuild chain index if needed
        self.rebuild_chain_index()?;

        Ok(())
    }

    /// Rebuild chain index from WAL
    fn rebuild_chain_index(&self) -> Result<()> {
        info!("Rebuilding chain index...");

        let mut blocks: Vec<BlockMetadata> = Vec::new();

        // Read all block entries from WAL
        self.read_entries(|entry| {
            if entry.entry_type == WalEntryType::Block {
                if let Ok(metadata) = bincode::deserialize::<BlockMetadata>(&entry.data) {
                    blocks.push(metadata);
                }
            }
            Ok(())
        })?;

        // Sort by index
        blocks.sort_by_key(|b| b.index);

        // Write chain index
        self.write_chain_index(&blocks)?;

        info!("Chain index rebuilt: {} blocks", blocks.len());

        Ok(())
    }

    /// Read all entries from WAL, calling the callback for each
    fn read_entries<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(&WalEntry) -> Result<()>,
    {
        let file = File::open(&self.wal_path)?;
        let mut reader = BufReader::new(file);

        // Skip header
        let mut header_buf = [0u8; 14 + 4 + 8]; // magic + version + sequence
        reader.read_exact(&mut header_buf)?;

        // Read entries
        loop {
            let mut len_bytes = [0u8; 8];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(_) => break,
            }
            let entry_len = u64::from_le_bytes(len_bytes) as usize;

            if entry_len == 0 || entry_len > 100_000_000 {
                break;
            }

            let mut entry_data = vec![0u8; entry_len];
            if reader.read_exact(&mut entry_data).is_err() {
                break;
            }

            if let Ok(entry) = WalEntry::from_bytes(&entry_data) {
                if entry.verify() {
                    callback(&entry)?;
                }
            }
        }

        Ok(())
    }

    /// Write an entry to the WAL
    fn write_entry(&self, entry: &WalEntry) -> Result<()> {
        let mut file = self
            .wal_file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WAL file"))?;

        let entry_bytes = entry.to_bytes()?;
        let entry_len = entry_bytes.len() as u64;

        // Write entry length
        file.write_all(&entry_len.to_le_bytes())?;

        // Write entry data
        file.write_all(&entry_bytes)?;

        // Sync if needed
        let entries = self.entries_since_sync.fetch_add(1, Ordering::SeqCst) + 1;
        if entries >= self.sync_interval {
            file.sync_all().context("Failed to sync WAL")?;
            self.entries_since_sync.store(0, Ordering::SeqCst);
            debug!("WAL synced after {} entries", entries);
        }

        Ok(())
    }

    /// Force sync WAL to disk
    pub fn sync(&self) -> Result<()> {
        self.sync_wal()?;
        self.sync_chain_index()?;
        Ok(())
    }

    fn sync_wal(&self) -> Result<()> {
        let file = self
            .wal_file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WAL file"))?;

        file.sync_all().context("Failed to sync WAL")?;

        self.entries_since_sync.store(0, Ordering::SeqCst);

        Ok(())
    }

    fn sync_chain_index(&self) -> Result<()> {
        if !self.chain_index_path.exists() {
            self.chain_index_entries_since_sync
                .store(0, Ordering::SeqCst);
            return Ok(());
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&self.chain_index_path)
            .with_context(|| {
                format!(
                    "Failed to open chain index for sync: {}",
                    self.chain_index_path.display()
                )
            })?;

        file.sync_all().context("Failed to sync chain index")?;
        self.chain_index_entries_since_sync
            .store(0, Ordering::SeqCst);

        Ok(())
    }

    /// Store a block atomically
    pub fn store_block(&self, metadata: &BlockMetadata, rdf_data: &str) -> Result<()> {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);

        // 1. Write RDF data to separate file first
        let rdf_file_path = self
            .rdf_data_dir
            .join(format!("block_{}.ttl", metadata.index));
        fs::write(&rdf_file_path, rdf_data)
            .with_context(|| format!("Failed to write RDF data to {}", rdf_file_path.display()))?;

        // 2. Create WAL entry for block metadata
        let metadata_bytes =
            bincode::serialize(metadata).context("Failed to serialize block metadata")?;

        let entry = WalEntry::new(WalEntryType::Block, sequence, metadata_bytes);

        // 3. Write to WAL (this is the atomic commit point). `write_entry`
        // handles fsync according to the configured interval.
        self.write_entry(&entry)?;

        // 4. Update chain index. Crash recovery can rebuild this from WAL if
        // the index is behind the latest entries.
        self.append_to_chain_index(metadata)?;

        info!(
            "Block {} persisted: hash={}, rdf_file={}",
            metadata.index,
            &metadata.hash[..16.min(metadata.hash.len())],
            rdf_file_path.display()
        );

        Ok(())
    }

    /// Load RDF data for a block
    pub fn load_block_data(&self, index: u64) -> Result<String> {
        let rdf_file_path = self.rdf_data_dir.join(format!("block_{}.ttl", index));

        fs::read_to_string(&rdf_file_path).with_context(|| {
            format!(
                "Failed to read RDF data for block {} from {}",
                index,
                rdf_file_path.display()
            )
        })
    }

    /// Load all blocks from chain index
    pub fn load_all_blocks(&self) -> Result<Vec<BlockMetadata>> {
        if !self.chain_index_path.exists() {
            return Ok(Vec::new());
        }

        let index_data =
            fs::read_to_string(&self.chain_index_path).context("Failed to read chain index")?;

        let mut blocks = Vec::new();
        for line in index_data.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let metadata: BlockMetadata = bincode::deserialize(
                &hex::decode(line).context("Failed to decode chain index entry")?,
            )
            .context("Failed to deserialize block metadata")?;

            blocks.push(metadata);
        }

        Ok(blocks)
    }

    /// Write chain index file
    fn write_chain_index(&self, blocks: &[BlockMetadata]) -> Result<()> {
        let mut index_data = String::new();

        for block in blocks {
            let encoded = hex::encode(
                bincode::serialize(block).context("Failed to serialize block metadata")?,
            );
            index_data.push_str(&encoded);
            index_data.push('\n');
        }

        // Write to temp file first, then rename for atomicity
        let temp_path = self.chain_index_path.with_extension("tmp");
        fs::write(&temp_path, index_data)
            .with_context(|| format!("Failed to write chain index to {}", temp_path.display()))?;

        fs::rename(&temp_path, &self.chain_index_path)
            .with_context(|| format!("Failed to rename chain index file"))?;

        Ok(())
    }

    /// Append single block to chain index
    fn append_to_chain_index(&self, metadata: &BlockMetadata) -> Result<()> {
        let encoded = hex::encode(
            bincode::serialize(metadata).context("Failed to serialize block metadata")?,
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.chain_index_path)?;

        writeln!(file, "{}", encoded)?;

        let entries = self
            .chain_index_entries_since_sync
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        if entries >= self.chain_index_sync_interval {
            file.sync_all().context("Failed to sync chain index")?;
            self.chain_index_entries_since_sync
                .store(0, Ordering::SeqCst);
            debug!("Chain index synced after {} entries", entries);
        }

        Ok(())
    }

    /// Create a checkpoint
    pub fn create_checkpoint(&self, checkpoint: &ChainCheckpoint) -> Result<()> {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);

        let checkpoint_bytes =
            bincode::serialize(checkpoint).context("Failed to serialize checkpoint")?;

        let entry = WalEntry::new(WalEntryType::Checkpoint, sequence, checkpoint_bytes);

        self.write_entry(&entry)?;
        self.sync()?;

        info!(
            "Checkpoint created at block {}, merkle_root={}",
            checkpoint.block_height,
            &checkpoint.chain_merkle_root[..16.min(checkpoint.chain_merkle_root.len())]
        );

        Ok(())
    }

    /// Get data directory path
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get RDF data directory path
    pub fn rdf_data_dir(&self) -> &Path {
        &self.rdf_data_dir
    }

    /// Compact WAL (remove old entries, keeping only necessary data)
    pub fn compact_wal(&self) -> Result<()> {
        info!("Compacting WAL...");

        // This is a simplified compaction - in production, you'd want to:
        // 1. Write all current blocks to a new WAL
        // 2. Atomically swap files
        // 3. For now, we just rotate the WAL

        self.sync()?;

        // Rename current WAL to backup
        let backup_path = self.wal_path.with_extension("wal.bak");
        fs::rename(&self.wal_path, &backup_path)?;

        // Create new WAL
        {
            let mut file = self
                .wal_file
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock WAL file"))?;
            *file = OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(&self.wal_path)?;
        }

        self.write_wal_header()?;

        // Rebuild chain index in new WAL
        let blocks = self.load_all_blocks()?;
        for block in blocks {
            let metadata_bytes = bincode::serialize(&block)?;
            let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
            let entry = WalEntry::new(WalEntryType::Block, sequence, metadata_bytes);
            self.write_entry(&entry)?;
        }

        self.sync()?;

        // Remove old backup
        fs::remove_file(&backup_path)?;

        info!("WAL compaction complete");

        Ok(())
    }
}

impl Drop for PersistentStorage {
    fn drop(&mut self) {
        if let Err(error) = self.sync() {
            warn!("Failed to sync persistent storage during drop: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wal_entry_serialization() {
        let entry = WalEntry::new(WalEntryType::Block, 1, b"test data".to_vec());

        let bytes = entry.to_bytes().unwrap();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.entry_type, entry.entry_type);
        assert_eq!(recovered.sequence, entry.sequence);
        assert_eq!(recovered.data, entry.data);
        assert!(recovered.verify());
    }

    #[test]
    fn test_persistent_storage_basic() {
        let temp_dir = tempdir().unwrap();
        let storage = PersistentStorage::open(temp_dir.path()).unwrap();

        // Store a block
        let metadata = BlockMetadata {
            index: 0,
            hash: "abc123".to_string(),
            previous_hash: "000000".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            validator: "validator1".to_string(),
            signature: "sig123".to_string(),
            state_root: "state123".to_string(),
            data_graph_uri: "http://example.org/block/0".to_string(),
            has_encrypted_data: false,
            data_size: 100,
        };

        storage
            .store_block(
                &metadata,
                "@prefix ex: <http://example.org/> .\nex:test ex:value 1 .",
            )
            .unwrap();

        // Load blocks
        let blocks = storage.load_all_blocks().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].index, 0);

        // Load RDF data
        let rdf_data = storage.load_block_data(0).unwrap();
        assert!(rdf_data.contains("ex:test"));
    }

    #[test]
    fn test_persistent_storage_batched_sync_interval_keeps_index_readable() {
        let temp_dir = tempdir().unwrap();
        let storage =
            PersistentStorage::open_with_sync_intervals(temp_dir.path(), 100, 50).unwrap();

        assert_eq!(storage.sync_interval, 100);
        assert_eq!(storage.chain_index_sync_interval, 50);

        for index in 0..3 {
            let metadata = BlockMetadata {
                index,
                hash: format!("hash{}", index),
                previous_hash: format!("hash{}", index.saturating_sub(1)),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                validator: "validator1".to_string(),
                signature: format!("sig{}", index),
                state_root: format!("state{}", index),
                data_graph_uri: format!("http://example.org/block/{}", index),
                has_encrypted_data: false,
                data_size: 100,
            };

            storage
                .store_block(
                    &metadata,
                    &format!(
                        "@prefix ex: <http://example.org/> .\nex:block{} ex:index {} .",
                        index, index
                    ),
                )
                .unwrap();
        }

        assert_eq!(storage.entries_since_sync.load(Ordering::SeqCst), 3);
        assert_eq!(
            storage
                .chain_index_entries_since_sync
                .load(Ordering::SeqCst),
            3
        );

        let blocks = storage.load_all_blocks().unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[2].index, 2);

        storage.sync().unwrap();
        assert_eq!(storage.entries_since_sync.load(Ordering::SeqCst), 0);
        assert_eq!(
            storage
                .chain_index_entries_since_sync
                .load(Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn test_recovery_rebuilds_missing_chain_index_from_wal() {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("rebuild_index_test");

        {
            let storage = PersistentStorage::open_with_sync_interval(&data_dir, 1).unwrap();

            for index in 0..3 {
                let metadata = BlockMetadata {
                    index,
                    hash: format!("hash{}", index),
                    previous_hash: format!("hash{}", index.saturating_sub(1)),
                    timestamp: "2024-01-01T00:00:00Z".to_string(),
                    validator: "validator1".to_string(),
                    signature: format!("sig{}", index),
                    state_root: format!("state{}", index),
                    data_graph_uri: format!("http://example.org/block/{}", index),
                    has_encrypted_data: false,
                    data_size: 100,
                };

                storage
                    .store_block(
                        &metadata,
                        &format!(
                            "@prefix ex: <http://example.org/> .\nex:block{} ex:index {} .",
                            index, index
                        ),
                    )
                    .unwrap();
            }
        }

        std::fs::remove_file(data_dir.join("chain.index")).unwrap();

        let storage = PersistentStorage::open_with_sync_interval(&data_dir, 1).unwrap();
        let blocks = storage.load_all_blocks().unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[2].index, 2);
    }

    #[test]
    fn test_storage_recovery() {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("recovery_test");

        // Create storage and store blocks
        {
            let storage = PersistentStorage::open(&data_dir).unwrap();

            for i in 0..5 {
                let metadata = BlockMetadata {
                    index: i,
                    hash: format!("hash{}", i),
                    previous_hash: format!("hash{}", i.saturating_sub(1)),
                    timestamp: "2024-01-01T00:00:00Z".to_string(),
                    validator: "validator1".to_string(),
                    signature: format!("sig{}", i),
                    state_root: format!("state{}", i),
                    data_graph_uri: format!("http://example.org/block/{}", i),
                    has_encrypted_data: false,
                    data_size: 100,
                };

                storage
                    .store_block(
                        &metadata,
                        &format!(
                            "@prefix ex: <http://example.org/> .\nex:block{} ex:index {} .",
                            i, i
                        ),
                    )
                    .unwrap();
            }
        }

        // Re-open (simulates crash recovery)
        {
            let storage = PersistentStorage::open(&data_dir).unwrap();
            let blocks = storage.load_all_blocks().unwrap();

            assert_eq!(blocks.len(), 5);
            for (i, block) in blocks.iter().enumerate() {
                assert_eq!(block.index, i as u64);
            }
        }
    }
}
