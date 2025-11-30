//! Memory optimization utilities
//!
//! This module provides tools for optimizing memory usage throughout the application,
//! including object pooling, memory-efficient data structures, and garbage collection hints.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Memory pool for reusing expensive objects
pub struct ObjectPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ObjectPool<T>
where
    T: Send + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(Mutex::new(Vec::new())),
            factory: Box::new(factory),
            max_size,
        }
    }

    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().unwrap();
        let object = objects.pop().unwrap_or_else(|| (self.factory)());

        PooledObject {
            object: Some(object),
            pool: Arc::clone(&self.objects),
            max_size: self.max_size,
        }
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.objects.lock().unwrap().len()
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.objects.lock().unwrap().clear();
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Get a reference to the pooled object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }

    /// Get a mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < self.max_size {
                pool.push(object);
            }
            // If pool is full, object is dropped
        }
    }
}

/// Memory usage tracker
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated_bytes: u64,
    pub peak_bytes: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub last_updated: Instant,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            allocated_bytes: 0,
            peak_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
            last_updated: Instant::now(),
        }
    }
}

/// Memory tracker for monitoring memory usage
pub struct MemoryTracker {
    stats: Arc<Mutex<MemoryStats>>,
    tracking_enabled: bool,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(MemoryStats::default())),
            tracking_enabled: true,
        }
    }

    /// Record memory allocation
    pub fn record_allocation(&self, size: u64) {
        if !self.tracking_enabled {
            return;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.allocated_bytes += size;
        stats.allocation_count += 1;
        stats.peak_bytes = stats.peak_bytes.max(stats.allocated_bytes);
        stats.last_updated = Instant::now();
    }

    /// Record memory deallocation
    pub fn record_deallocation(&self, size: u64) {
        if !self.tracking_enabled {
            return;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.allocated_bytes = stats.allocated_bytes.saturating_sub(size);
        stats.deallocation_count += 1;
        stats.last_updated = Instant::now();
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = MemoryStats::default();
    }

    /// Enable or disable tracking
    pub fn set_tracking_enabled(&mut self, enabled: bool) {
        self.tracking_enabled = enabled;
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory-efficient string interning
pub struct StringInterner {
    strings: Arc<Mutex<HashMap<String, Arc<str>>>>,
    max_entries: usize,
}

impl StringInterner {
    pub fn new(max_entries: usize) -> Self {
        Self {
            strings: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
        }
    }

    /// Intern a string, returning a shared reference
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut strings = self.strings.lock().unwrap();

        if let Some(interned) = strings.get(s) {
            return Arc::clone(interned);
        }

        // Check if we need to evict old entries
        if strings.len() >= self.max_entries {
            // Simple eviction: remove oldest entries
            // In a production system, you might use LRU or other strategies
            strings.clear();
        }

        let interned: Arc<str> = Arc::from(s);
        strings.insert(s.to_string(), Arc::clone(&interned));
        interned
    }

    /// Get current number of interned strings
    pub fn size(&self) -> usize {
        self.strings.lock().unwrap().len()
    }

    /// Clear all interned strings
    pub fn clear(&self) {
        self.strings.lock().unwrap().clear();
    }
}

/// Memory-efficient buffer pool for reusing byte buffers
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<u8>>>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::new())),
            buffer_size,
            max_buffers,
        }
    }

    /// Get a buffer from the pool
    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        buffers
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();

        let mut buffers = self.buffers.lock().unwrap();
        if buffers.len() < self.max_buffers {
            buffers.push(buffer);
        }
        // If pool is full, buffer is dropped
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let buffers = self.buffers.lock().unwrap();
        (buffers.len(), self.buffer_size, self.max_buffers)
    }
}

/// Memory optimization utilities
pub struct MemoryOptimizer;

impl MemoryOptimizer {
    /// Suggest garbage collection (hint to the runtime)
    pub fn suggest_gc() {
        // In Rust, we don't have explicit GC, but we can drop large objects
        // and suggest that the allocator reclaim memory
        std::hint::black_box(());
    }

    /// Calculate memory usage of a data structure (approximate)
    pub fn estimate_size<T>(data: &T) -> usize {
        std::mem::size_of_val(data)
    }

    /// Create a memory-efficient clone strategy
    pub fn efficient_clone<T: Clone>(data: &T, threshold_size: usize) -> T {
        let size = Self::estimate_size(data);

        if size > threshold_size {
            // For large objects, consider using Arc or other shared ownership
            eprintln!("Warning: Cloning large object ({} bytes)", size);
        }

        data.clone()
    }

    /// Memory usage report
    pub fn memory_report() -> String {
        // In a real implementation, you might use system calls to get actual memory usage
        "Memory Report:\n\
             - Process memory usage: Not available in safe Rust\n\
             - Heap allocations: Tracked by custom allocator if available\n\
             - Recommendation: Use memory profiling tools like valgrind or heaptrack"
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(Vec::<u8>::new, 5);

        // Get an object
        let mut obj1 = pool.get();
        obj1.get_mut().push(1);

        // Pool should be empty
        assert_eq!(pool.size(), 0);

        // Drop the object (returns to pool)
        drop(obj1);

        // Pool should have one object
        assert_eq!(pool.size(), 1);

        // Get another object (should reuse the returned one)
        let obj2 = pool.get();
        // Note: The returned object still contains the data from before
        // In a real implementation, you might want to clear objects when returning to pool
        assert!(!obj2.get().is_empty()); // Should have some data
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker.record_allocation(100);
        tracker.record_allocation(200);

        let stats = tracker.get_stats();
        assert_eq!(stats.allocated_bytes, 300);
        assert_eq!(stats.allocation_count, 2);

        tracker.record_deallocation(100);
        let stats = tracker.get_stats();
        assert_eq!(stats.allocated_bytes, 200);
        assert_eq!(stats.deallocation_count, 1);
    }

    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new(100);

        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");

        // Should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(interner.size(), 1);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(1024, 5);

        let buffer1 = pool.get_buffer();
        assert_eq!(buffer1.capacity(), 1024);

        pool.return_buffer(buffer1);

        let (pool_size, buffer_size, max_buffers) = pool.stats();
        assert_eq!(pool_size, 1);
        assert_eq!(buffer_size, 1024);
        assert_eq!(max_buffers, 5);
    }

    #[test]
    fn test_memory_optimizer() {
        let data = vec![1, 2, 3, 4, 5];
        let size = MemoryOptimizer::estimate_size(&data);
        assert!(size > 0);

        let cloned = MemoryOptimizer::efficient_clone(&data, 1000);
        assert_eq!(data, cloned);

        let report = MemoryOptimizer::memory_report();
        assert!(report.contains("Memory Report"));
    }
}
