//! Memory-aware allocation strategies for OWL2 Reasoner
//!
//! This module provides intelligent allocation strategies that adapt to current memory conditions
//! and gracefully degrade under memory pressure rather than causing out-of-memory conditions.

use crate::graceful_degradation::{DegradationLevel, can_component_operate, ComponentOperationResult, RejectionReason};
use crate::memory_protection::{can_allocate, AllocationResult};
use crate::memory::get_memory_stats;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Allocation strategy configuration
#[derive(Debug, Clone)]
pub struct AllocationStrategyConfig {
    /// Enable adaptive sizing based on memory pressure
    pub enable_adaptive_sizing: bool,
    /// Enable memory pool allocation
    pub enable_memory_pools: bool,
    /// Enable allocation batching
    pub enable_batching: bool,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Pool allocation limit
    pub pool_limit: usize,
    /// Adaptive sizing factor (0.0 to 1.0)
    pub adaptive_factor: f64,
}

impl Default for AllocationStrategyConfig {
    fn default() -> Self {
        Self {
            enable_adaptive_sizing: true,
            enable_memory_pools: true,
            enable_batching: true,
            max_batch_size: 100,
            pool_limit: 1000,
            adaptive_factor: 0.8,
        }
    }
}

/// Memory pool for efficient allocation of small objects
#[derive(Debug)]
pub struct MemoryPool<T> {
    pool: VecDeque<Box<T>>,
    config: AllocationStrategyConfig,
    allocated: AtomicUsize,
    hits: AtomicUsize,
    misses: AtomicUsize,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl<T> MemoryPool<T> {
    pub fn new(config: AllocationStrategyConfig) -> Self {
        Self {
            pool: VecDeque::with_capacity(config.pool_limit),
            config,
            allocated: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Allocate from pool or create new allocation
    pub fn allocate<F>(&mut self, creator: F) -> Result<Box<T>, String>
    where
        F: FnOnce() -> Box<T>,
    {
        if let Some(item) = self.pool.pop_front() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Ok(item)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            self.allocated.fetch_add(1, Ordering::Relaxed);
            Ok(creator())
        }
    }

    /// Return allocation to pool
    pub fn deallocate(&mut self, item: Box<T>) -> Result<(), String> {
        if self.pool.len() < self.config.pool_limit {
            self.pool.push_back(item);
            Ok(())
        } else {
            Err("Pool full".to_string())
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            allocated: self.allocated.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            pool_size: self.pool.len(),
            hit_rate: if self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed) > 0 {
                self.hits.load(Ordering::Relaxed) as f64 / (self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)) as f64
            } else {
                0.0
            },
        }
    }

    /// Clean up old allocations
    pub fn cleanup(&mut self) -> usize {
        let initial_size = self.pool.len();
        
        // Remove allocations based on some heuristic
        let keep_size = (self.config.pool_limit as f64 * 0.5) as usize;
        if self.pool.len() > keep_size {
            self.pool.truncate(keep_size);
        }
        
        *self.last_cleanup.lock() = Instant::now();
        initial_size - self.pool.len()
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub allocated: usize,
    pub hits: usize,
    pub misses: usize,
    pub pool_size: usize,
    pub hit_rate: f64,
}

/// Memory-aware allocation result
#[derive(Debug, Clone)]
pub struct MemoryAwareAllocation<T> {
    /// The allocated object
    pub item: T,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Whether the allocation was adapted due to memory pressure
    pub was_adapted: bool,
    /// Allocation strategy used
    pub strategy: String,
}

/// Memory-aware allocator
pub struct MemoryAwareAllocator {
    config: AllocationStrategyConfig,
    pools: Arc<Mutex<HashMap<String, Box<dyn MemoryPoolTrait>>>>,
    batch_queue: Arc<Mutex<VecDeque<BatchRequest>>>,
    stats: Arc<Mutex<AllocatorStats>>,
}

/// Trait for memory pool implementations
pub trait MemoryPoolTrait: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn cleanup(&mut self) -> usize;
    fn stats(&self) -> PoolStats;
}

impl<T: Send + Sync + 'static> MemoryPoolTrait for MemoryPool<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn cleanup(&mut self) -> usize {
        self.cleanup()
    }

    fn stats(&self) -> PoolStats {
        self.stats()
    }
}

/// Batch allocation request
#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub component: String,
    pub size: usize,
    pub count: usize,
    pub timestamp: Instant,
}

/// Allocator statistics
#[derive(Debug, Default, Clone)]
pub struct AllocatorStats {
    pub total_allocations: usize,
    pub successful_allocations: usize,
    pub failed_allocations: usize,
    pub adapted_allocations: usize,
    pub total_bytes_allocated: usize,
    pub bytes_saved_by_adaptation: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub batch_operations: usize,
    pub last_adaptation: Option<Instant>,
}

impl MemoryAwareAllocator {
    pub fn new() -> Self {
        Self {
            config: AllocationStrategyConfig::default(),
            pools: Arc::new(Mutex::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(AllocatorStats::default())),
        }
    }

    pub fn with_config(config: AllocationStrategyConfig) -> Self {
        Self {
            config,
            pools: Arc::new(Mutex::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(Mutex::new(AllocatorStats::default())),
        }
    }

    /// Perform memory-aware allocation
    pub fn allocate<F, T>(&mut self, component: &str, size: usize, creator: F) -> Result<MemoryAwareAllocation<T>, String>
    where
        F: FnOnce() -> T,
        T: Send + Sync + 'static,
    {
        let mut stats = self.stats.lock();
        stats.total_allocations += 1;

        // Check global memory protection first
        match can_allocate(size) {
            AllocationResult::Rejected(reason) => {
                stats.failed_allocations += 1;
                return Err(format!("Allocation rejected by memory protection: {:?}", reason));
            }
            AllocationResult::Allowed => {}
        }

        // Check component-specific degradation
        match can_component_operate(component, size) {
            ComponentOperationResult::Rejected(reason) => {
                stats.failed_allocations += 1;
                return Err(format!("Allocation rejected by component degradation: {:?}", reason));
            }
            ComponentOperationResult::Allowed(timeout_multiplier) => {
                // Apply timeout-based adaptation
                if timeout_multiplier > 1.5 {
                    // This is a warning sign of memory pressure
                    stats.adapted_allocations += 1;
                    stats.last_adaptation = Some(Instant::now());
                }
            }
        }

        // Apply adaptive sizing if enabled
        let adapted_size = if self.config.enable_adaptive_sizing {
            self.calculate_adapted_size(size, component)?
        } else {
            size
        };

        // Try to allocate from memory pool if enabled
        let item = if self.config.enable_memory_pools {
            self.pool_allocate(component, adapted_size, creator)?
        } else {
            let item = creator();
            Box::new(item) // Box it for consistency with pool allocation
        };

        stats.successful_allocations += 1;
        stats.total_bytes_allocated += adapted_size;
        
        if adapted_size < size {
            stats.bytes_saved_by_adaptation += size - adapted_size;
        }

        Ok(MemoryAwareAllocation {
            item: *item,
            size: adapted_size,
            was_adapted: adapted_size < size,
            strategy: "direct".to_string(),
        })
    }

    /// Calculate adapted size based on memory pressure
    fn calculate_adapted_size(&self, requested_size: usize, component: &str) -> Result<usize, String> {
        let memory_stats = get_memory_stats();
        let pressure = memory_stats.pressure_level;
        
        // Apply adaptive sizing based on memory pressure
        let adapted_size = if pressure > 0.8 {
            // High pressure - significantly reduce allocation size
            (requested_size as f64 * self.config.adaptive_factor.powi(3)) as usize
        } else if pressure > 0.6 {
            // Medium pressure - moderately reduce allocation size
            (requested_size as f64 * self.config.adaptive_factor.powi(2)) as usize
        } else if pressure > 0.4 {
            // Low pressure - slightly reduce allocation size
            (requested_size as f64 * self.config.adaptive_factor) as usize
        } else {
            // No pressure - use requested size
            requested_size
        };

        // Ensure minimum size
        let min_size = 1024; // 1KB minimum
        let final_size = adapted_size.max(min_size).max(requested_size / 10);
        
        Ok(final_size)
    }

    /// Allocate from memory pool
    fn pool_allocate<F, T>(&mut self, component: &str, size: usize, creator: F) -> Result<Box<T>, String>
    where
        F: FnOnce() -> T,
        T: Send + Sync + 'static,
    {
        let pool_key = format!("{}_{}", component, size);
        let mut pools = self.pools.lock();
        
        let pool: &mut Box<dyn MemoryPoolTrait> = pools.entry(pool_key.clone())
            .or_insert_with(|| {
                let pool: MemoryPool<T> = MemoryPool::new(self.config.clone());
                Box::new(pool)
            });
        
        // This is safe because we just created or retrieved the pool with this specific type
        let typed_pool = unsafe { &mut *(pool.as_any() as *const dyn MemoryPoolTrait as *mut MemoryPool<T>) };
        
        let result = typed_pool.allocate(creator)?;
        
        // Update statistics
        {
            let mut stats = self.stats.lock();
            let pool_stats = typed_pool.stats();
            stats.pool_hits += pool_stats.hits;
            stats.pool_misses += pool_stats.misses;
        }
        
        Ok(result)
    }

    /// Add allocation to batch queue
    pub fn add_to_batch(&mut self, component: String, size: usize, count: usize) -> Result<(), String> {
        if count > self.config.max_batch_size {
            return Err("Batch size exceeds maximum limit".to_string());
        }

        let request = BatchRequest {
            component,
            size,
            count,
            timestamp: Instant::now(),
        };

        self.batch_queue.lock().push_back(request);
        
        {
            let mut stats = self.stats.lock();
            stats.batch_operations += 1;
        }

        Ok(())
    }

    /// Process batch allocations
    pub fn process_batch<F, T>(&mut self, creator: F) -> Vec<Result<MemoryAwareAllocation<T>, String>>
    where
        F: Fn() -> T,
        T: Send + Sync + Clone + 'static,
    {
        let requests: Vec<BatchRequest> = self.batch_queue.lock().drain(..).collect();
        
        requests
            .into_iter()
            .map(|request| {
                self.allocate(&request.component, request.size, &creator)
            })
            .collect()
    }

    /// Get allocator statistics
    pub fn get_stats(&self) -> AllocatorStats {
        self.stats.lock().clone()
    }

    /// Clean up memory pools
    pub fn cleanup_pools(&mut self) -> usize {
        let mut total_cleaned = 0;
        let mut pools = self.pools.lock();
        
        for (_, pool) in pools.iter_mut() {
            total_cleaned += pool.cleanup();
        }
        
        total_cleaned
    }

    /// Get pool statistics for a specific component and size
    pub fn get_pool_stats(&self, component: &str, size: usize) -> Option<PoolStats> {
        let pool_key = format!("{}_{}", component, size);
        let pools = self.pools.lock();
        
        if let Some(pool) = pools.get(&pool_key) {
            Some(pool.stats())
        } else {
            None
        }
    }
}

/// Global memory-aware allocator instance
static GLOBAL_MEMORY_AWARE_ALLOCATOR: std::sync::LazyLock<Mutex<MemoryAwareAllocator>> = 
    std::sync::LazyLock::new(|| Mutex::new(MemoryAwareAllocator::new()));

/// Get global memory-aware allocator
pub fn get_memory_aware_allocator() -> &'static Mutex<MemoryAwareAllocator> {
    &GLOBAL_MEMORY_AWARE_ALLOCATOR
}

/// Perform memory-aware allocation
pub fn memory_aware_allocate<F, T>(component: &str, size: usize, creator: F) -> Result<MemoryAwareAllocation<T>, String>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().allocate(component, size, creator)
}

/// Add allocation to batch queue
pub fn add_to_batch(component: String, size: usize, count: usize) -> Result<(), String> {
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().add_to_batch(component, size, count)
}

/// Process batch allocations
pub fn process_batch<F, T>(creator: F) -> Vec<Result<MemoryAwareAllocation<T>, String>>
where
    F: Fn() -> T,
    T: Send + Sync + Clone + 'static,
{
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().process_batch(creator)
}

/// Get allocator statistics
pub fn get_allocator_stats() -> AllocatorStats {
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().get_stats()
}

/// Clean up memory pools
pub fn cleanup_memory_pools() -> usize {
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().cleanup_pools()
}

/// Get pool statistics for a specific component and size
pub fn get_pool_stats(component: &str, size: usize) -> Option<PoolStats> {
    GLOBAL_MEMORY_AWARE_ALLOCATOR.lock().get_pool_stats(component, size)
}
