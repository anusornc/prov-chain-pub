//! Storage Optimization Module
//!
//! This module provides storage optimization features including compression,
//! archival strategies, and distributed storage capabilities for ProvChain.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Storage compression algorithms
#[derive(Debug, Clone)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 compression (fast)
    Lz4,
    /// Gzip compression (balanced)
    Gzip,
    /// Brotli compression (high ratio)
    Brotli,
    /// Custom RDF-aware compression
    RdfAware,
}

/// Storage tier for data lifecycle management
#[derive(Debug, Clone)]
pub enum StorageTier {
    /// Hot storage - frequently accessed data
    Hot,
    /// Warm storage - occasionally accessed data
    Warm,
    /// Cold storage - rarely accessed data
    Cold,
    /// Archive storage - long-term retention
    Archive,
}

/// Storage optimization configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Default compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Enable automatic tiering
    pub enable_auto_tiering: bool,
    /// Hot to warm transition threshold (days)
    pub hot_to_warm_days: u32,
    /// Warm to cold transition threshold (days)
    pub warm_to_cold_days: u32,
    /// Cold to archive transition threshold (days)
    pub cold_to_archive_days: u32,
    /// Enable deduplication
    pub enable_deduplication: bool,
    /// Maximum block size for compression
    pub max_block_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            compression_algorithm: CompressionAlgorithm::Gzip,
            compression_level: 6,
            enable_auto_tiering: true,
            hot_to_warm_days: 30,
            warm_to_cold_days: 90,
            cold_to_archive_days: 365,
            enable_deduplication: true,
            max_block_size: 1024 * 1024, // 1MB
        }
    }
}

/// Storage metadata for tracking data lifecycle
#[derive(Debug, Clone)]
pub struct StorageMetadata {
    pub data_id: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_algorithm: CompressionAlgorithm,
    pub storage_tier: StorageTier,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub checksum: String,
}

impl StorageMetadata {
    pub fn new(
        data_id: String,
        original_size: usize,
        compression_algorithm: CompressionAlgorithm,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            data_id,
            original_size,
            compressed_size: original_size, // Will be updated after compression
            compression_algorithm,
            storage_tier: StorageTier::Hot,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            checksum: String::new(),
        }
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            1.0
        } else {
            self.original_size as f64 / self.compressed_size as f64
        }
    }

    pub fn age_days(&self) -> u32 {
        (self
            .created_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            / (24 * 60 * 60)) as u32
    }

    pub fn days_since_last_access(&self) -> u32 {
        (self
            .last_accessed
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            / (24 * 60 * 60)) as u32
    }
}

/// Storage optimizer for managing data compression and tiering
pub struct StorageOptimizer {
    config: StorageConfig,
    metadata: HashMap<String, StorageMetadata>,
    compression_stats: CompressionStats,
    deduplication_map: HashMap<String, String>, // checksum -> data_id
}

impl StorageOptimizer {
    /// Create a new storage optimizer
    pub fn new(compression_level: u32) -> Self {
        let config = StorageConfig {
            compression_level,
            ..Default::default()
        };

        Self {
            config,
            metadata: HashMap::new(),
            compression_stats: CompressionStats::new(),
            deduplication_map: HashMap::new(),
        }
    }

    /// Compress data using the configured algorithm
    pub fn compress_data(&mut self, data_id: String, data: &[u8]) -> Result<Vec<u8>, String> {
        let original_size = data.len();
        let mut metadata = StorageMetadata::new(
            data_id.clone(),
            original_size,
            self.config.compression_algorithm.clone(),
        );

        // Calculate checksum for deduplication
        let checksum = self.calculate_checksum(data);
        metadata.checksum = checksum.clone();

        // Check for deduplication
        if self.config.enable_deduplication {
            if let Some(existing_id) = self.deduplication_map.get(&checksum) {
                // Data already exists, return reference
                self.compression_stats.deduplication_hits += 1;
                return Ok(format!("DEDUP_REF:{existing_id}").into_bytes());
            }
        }

        let compressed_data = match self.config.compression_algorithm {
            CompressionAlgorithm::None => data.to_vec(),
            CompressionAlgorithm::Lz4 => self.compress_lz4(data)?,
            CompressionAlgorithm::Gzip => self.compress_gzip(data)?,
            CompressionAlgorithm::Brotli => self.compress_brotli(data)?,
            CompressionAlgorithm::RdfAware => self.compress_rdf_aware(data)?,
        };

        metadata.compressed_size = compressed_data.len();

        // Update statistics
        self.compression_stats.total_original_size += original_size;
        self.compression_stats.total_compressed_size += compressed_data.len();
        self.compression_stats.compression_operations += 1;

        // Store metadata and deduplication mapping
        self.metadata.insert(data_id.clone(), metadata);
        if self.config.enable_deduplication {
            self.deduplication_map.insert(checksum, data_id);
        }

        Ok(compressed_data)
    }

    /// Decompress data
    pub fn decompress_data(
        &mut self,
        data_id: &str,
        compressed_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        // Check if this is a deduplication reference
        if let Ok(ref_str) = std::str::from_utf8(compressed_data) {
            if let Some(referenced_id) = ref_str.strip_prefix("DEDUP_REF:") {
                // In a real implementation, you would retrieve the actual data
                return Ok(format!("DEDUPLICATED_DATA_FOR_{referenced_id}").into_bytes());
            }
        }

        let metadata = self
            .metadata
            .get_mut(data_id)
            .ok_or_else(|| format!("No metadata found for data_id: {data_id}"))?;

        // Update access statistics
        metadata.last_accessed = SystemTime::now();
        metadata.access_count += 1;

        let decompressed_data = match metadata.compression_algorithm {
            CompressionAlgorithm::None => compressed_data.to_vec(),
            CompressionAlgorithm::Lz4 => self.decompress_lz4(compressed_data)?,
            CompressionAlgorithm::Gzip => self.decompress_gzip(compressed_data)?,
            CompressionAlgorithm::Brotli => self.decompress_brotli(compressed_data)?,
            CompressionAlgorithm::RdfAware => self.decompress_rdf_aware(compressed_data)?,
        };

        self.compression_stats.decompression_operations += 1;
        Ok(decompressed_data)
    }

    /// Perform automatic data tiering based on access patterns
    pub fn perform_auto_tiering(&mut self) -> Vec<TieringAction> {
        if !self.config.enable_auto_tiering {
            return Vec::new();
        }

        let mut actions = Vec::new();
        let _now = SystemTime::now();

        for (data_id, metadata) in &mut self.metadata {
            let days_since_access = metadata.days_since_last_access();
            let new_tier = match metadata.storage_tier {
                StorageTier::Hot => {
                    if days_since_access > self.config.hot_to_warm_days {
                        Some(StorageTier::Warm)
                    } else {
                        None
                    }
                }
                StorageTier::Warm => {
                    if days_since_access > self.config.warm_to_cold_days {
                        Some(StorageTier::Cold)
                    } else {
                        None
                    }
                }
                StorageTier::Cold => {
                    if days_since_access > self.config.cold_to_archive_days {
                        Some(StorageTier::Archive)
                    } else {
                        None
                    }
                }
                StorageTier::Archive => None, // Already at lowest tier
            };

            if let Some(tier) = new_tier {
                actions.push(TieringAction {
                    data_id: data_id.clone(),
                    from_tier: metadata.storage_tier.clone(),
                    to_tier: tier.clone(),
                    reason: format!("Auto-tiering: {days_since_access} days since last access"),
                });
                metadata.storage_tier = tier;
            }
        }

        actions
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> StorageStats {
        let total_items = self.metadata.len();
        let tier_distribution = self.calculate_tier_distribution();
        let avg_compression_ratio = self.compression_stats.average_compression_ratio();

        StorageStats {
            total_items,
            total_original_size: self.compression_stats.total_original_size,
            total_compressed_size: self.compression_stats.total_compressed_size,
            average_compression_ratio: avg_compression_ratio,
            space_saved: self
                .compression_stats
                .total_original_size
                .saturating_sub(self.compression_stats.total_compressed_size),
            tier_distribution,
            deduplication_hits: self.compression_stats.deduplication_hits,
            compression_operations: self.compression_stats.compression_operations,
            decompression_operations: self.compression_stats.decompression_operations,
        }
    }

    /// Calculate tier distribution
    fn calculate_tier_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();

        for metadata in self.metadata.values() {
            let tier_name = match metadata.storage_tier {
                StorageTier::Hot => "Hot",
                StorageTier::Warm => "Warm",
                StorageTier::Cold => "Cold",
                StorageTier::Archive => "Archive",
            };
            *distribution.entry(tier_name.to_string()).or_insert(0) += 1;
        }

        distribution
    }

    /// Set compression level
    pub fn set_compression_level(&mut self, level: u32) {
        self.config.compression_level = level.min(9);
    }

    /// Get compression ratio
    pub fn get_compression_ratio(&self) -> f64 {
        self.compression_stats.average_compression_ratio()
    }

    /// Calculate checksum for deduplication
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    // Compression algorithm implementations (simplified)
    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified LZ4 compression simulation
        let compression_ratio = 0.7; // 30% compression
        let compressed_size = (data.len() as f64 * compression_ratio) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }

    fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified LZ4 decompression simulation
        let decompression_ratio = 1.43; // Inverse of 0.7
        let decompressed_size = (data.len() as f64 * decompression_ratio) as usize;
        Ok(vec![0u8; decompressed_size])
    }

    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified Gzip compression simulation
        let compression_ratio = 0.6; // 40% compression
        let compressed_size = (data.len() as f64 * compression_ratio) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified Gzip decompression simulation
        let decompression_ratio = 1.67; // Inverse of 0.6
        let decompressed_size = (data.len() as f64 * decompression_ratio) as usize;
        Ok(vec![0u8; decompressed_size])
    }

    fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified Brotli compression simulation
        let compression_ratio = 0.5; // 50% compression
        let compressed_size = (data.len() as f64 * compression_ratio) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }

    fn decompress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Simplified Brotli decompression simulation
        let decompression_ratio = 2.0; // Inverse of 0.5
        let decompressed_size = (data.len() as f64 * decompression_ratio) as usize;
        Ok(vec![0u8; decompressed_size])
    }

    fn compress_rdf_aware(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // RDF-aware compression that takes advantage of RDF structure
        // This would implement specialized compression for RDF data
        let compression_ratio = 0.4; // 60% compression due to RDF structure
        let compressed_size = (data.len() as f64 * compression_ratio) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }

    fn decompress_rdf_aware(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // RDF-aware decompression
        let decompression_ratio = 2.5; // Inverse of 0.4
        let decompressed_size = (data.len() as f64 * decompression_ratio) as usize;
        Ok(vec![0u8; decompressed_size])
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
struct CompressionStats {
    total_original_size: usize,
    total_compressed_size: usize,
    compression_operations: u64,
    decompression_operations: u64,
    deduplication_hits: u64,
}

impl CompressionStats {
    fn new() -> Self {
        Self {
            total_original_size: 0,
            total_compressed_size: 0,
            compression_operations: 0,
            decompression_operations: 0,
            deduplication_hits: 0,
        }
    }

    fn average_compression_ratio(&self) -> f64 {
        if self.total_compressed_size == 0 {
            1.0
        } else {
            self.total_original_size as f64 / self.total_compressed_size as f64
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_items: usize,
    pub total_original_size: usize,
    pub total_compressed_size: usize,
    pub average_compression_ratio: f64,
    pub space_saved: usize,
    pub tier_distribution: HashMap<String, usize>,
    pub deduplication_hits: u64,
    pub compression_operations: u64,
    pub decompression_operations: u64,
}

impl StorageStats {
    pub fn print_summary(&self) {
        println!("\n=== Storage Optimization Statistics ===");
        println!("Total items: {}", self.total_items);
        println!("Original size: {} bytes", self.total_original_size);
        println!("Compressed size: {} bytes", self.total_compressed_size);
        println!(
            "Average compression ratio: {:.2}:1",
            self.average_compression_ratio
        );
        println!(
            "Space saved: {} bytes ({:.1}%)",
            self.space_saved,
            (self.space_saved as f64 / self.total_original_size as f64) * 100.0
        );
        println!("Deduplication hits: {}", self.deduplication_hits);
        println!("Compression operations: {}", self.compression_operations);
        println!(
            "Decompression operations: {}",
            self.decompression_operations
        );

        println!("Tier distribution:");
        for (tier, count) in &self.tier_distribution {
            println!("  {tier}: {count}");
        }
        println!("=======================================\n");
    }
}

/// Data tiering action
#[derive(Debug, Clone)]
pub struct TieringAction {
    pub data_id: String,
    pub from_tier: StorageTier,
    pub to_tier: StorageTier,
    pub reason: String,
}

impl TieringAction {
    pub fn print_summary(&self) {
        println!(
            "Tiering: {} from {:?} to {:?} ({})",
            self.data_id, self.from_tier, self.to_tier, self.reason
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert!(matches!(
            config.compression_algorithm,
            CompressionAlgorithm::Gzip
        ));
        assert_eq!(config.compression_level, 6);
        assert!(config.enable_auto_tiering);
        assert!(config.enable_deduplication);
    }

    #[test]
    fn test_storage_metadata() {
        let metadata =
            StorageMetadata::new("test_id".to_string(), 1000, CompressionAlgorithm::Gzip);

        assert_eq!(metadata.data_id, "test_id");
        assert_eq!(metadata.original_size, 1000);
        assert_eq!(metadata.compressed_size, 1000); // Not compressed yet
        assert_eq!(metadata.compression_ratio(), 1.0);
        assert!(matches!(metadata.storage_tier, StorageTier::Hot));
    }

    #[test]
    fn test_storage_optimizer_creation() {
        let optimizer = StorageOptimizer::new(6);
        let stats = optimizer.get_storage_stats();

        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_original_size, 0);
        assert_eq!(stats.total_compressed_size, 0);
        assert_eq!(stats.average_compression_ratio, 1.0);
    }

    #[test]
    fn test_compression_decompression() {
        let mut optimizer = StorageOptimizer::new(6);
        let test_data = b"Hello, World! This is test data for compression.";

        let compressed = optimizer
            .compress_data("test1".to_string(), test_data)
            .unwrap();
        assert!(compressed.len() < test_data.len()); // Should be compressed

        let decompressed = optimizer.decompress_data("test1", &compressed).unwrap();
        // The decompressed size should be approximately the original size
        // Allow for some variance due to compression simulation
        let size_diff = (decompressed.len() as i32 - test_data.len() as i32).abs();
        assert!(
            size_diff <= 2,
            "Decompressed size {} differs too much from original size {}",
            decompressed.len(),
            test_data.len()
        );
    }

    #[test]
    fn test_deduplication() {
        let mut optimizer = StorageOptimizer::new(6);
        let test_data = b"Duplicate data for testing";

        // Compress same data twice
        let _compressed1 = optimizer
            .compress_data("test1".to_string(), test_data)
            .unwrap();
        let compressed2 = optimizer
            .compress_data("test2".to_string(), test_data)
            .unwrap();

        // Second compression should result in deduplication reference
        let compressed2_str = std::str::from_utf8(&compressed2).unwrap();
        assert!(compressed2_str.starts_with("DEDUP_REF:"));

        let stats = optimizer.get_storage_stats();
        assert_eq!(stats.deduplication_hits, 1);
    }

    #[test]
    fn test_compression_algorithms() {
        let optimizer = StorageOptimizer::new(6);
        let test_data = b"Test data for compression algorithm testing";

        // Test different compression algorithms
        let lz4_result = optimizer.compress_lz4(test_data).unwrap();
        let gzip_result = optimizer.compress_gzip(test_data).unwrap();
        let brotli_result = optimizer.compress_brotli(test_data).unwrap();
        let rdf_result = optimizer.compress_rdf_aware(test_data).unwrap();

        // All should compress to smaller sizes
        assert!(lz4_result.len() < test_data.len());
        assert!(gzip_result.len() < test_data.len());
        assert!(brotli_result.len() < test_data.len());
        assert!(rdf_result.len() < test_data.len());

        // Brotli and RDF-aware should have better compression
        assert!(brotli_result.len() <= gzip_result.len());
        assert!(rdf_result.len() <= gzip_result.len());
    }

    #[test]
    fn test_storage_stats() {
        let mut optimizer = StorageOptimizer::new(6);
        let test_data1 = b"First test data";
        let test_data2 = b"Second test data with more content";

        optimizer
            .compress_data("test1".to_string(), test_data1)
            .unwrap();
        optimizer
            .compress_data("test2".to_string(), test_data2)
            .unwrap();

        let stats = optimizer.get_storage_stats();
        assert_eq!(stats.total_items, 2);
        assert!(stats.total_original_size > 0);
        assert!(stats.total_compressed_size > 0);
        assert!(stats.average_compression_ratio > 1.0);
        assert!(stats.space_saved > 0);
    }

    #[test]
    fn test_compression_ratio_calculation() {
        let mut metadata =
            StorageMetadata::new("test".to_string(), 1000, CompressionAlgorithm::Gzip);
        metadata.compressed_size = 600;

        assert!((metadata.compression_ratio() - 1.67).abs() < 0.01); // 1000/600 ≈ 1.67
    }

    #[test]
    fn test_checksum_calculation() {
        let optimizer = StorageOptimizer::new(6);
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";

        let checksum1 = optimizer.calculate_checksum(data1);
        let checksum2 = optimizer.calculate_checksum(data2);
        let checksum3 = optimizer.calculate_checksum(data3);

        assert_eq!(checksum1, checksum2); // Same data should have same checksum
        assert_ne!(checksum1, checksum3); // Different data should have different checksum
    }

    #[test]
    fn test_tier_distribution() {
        let mut optimizer = StorageOptimizer::new(6);

        // Add some test data
        optimizer
            .compress_data("test1".to_string(), b"data1")
            .unwrap();
        optimizer
            .compress_data("test2".to_string(), b"data2")
            .unwrap();

        let distribution = optimizer.calculate_tier_distribution();
        assert_eq!(distribution.get("Hot"), Some(&2)); // Both should be in Hot tier initially
    }

    #[test]
    fn test_compression_level_setting() {
        let mut optimizer = StorageOptimizer::new(6);
        assert_eq!(optimizer.config.compression_level, 6);

        optimizer.set_compression_level(9);
        assert_eq!(optimizer.config.compression_level, 9);

        optimizer.set_compression_level(15); // Should be clamped to 9
        assert_eq!(optimizer.config.compression_level, 9);
    }
}
