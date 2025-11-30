//! Benchmark memory helper with allocator statistics.
//!
//! This module restores the memory profiling API expected by the benchmark
//! suite and augments it with allocation counters and byte deltas so we can
//! correlate performance regressions with allocator activity.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use sysinfo::System as SysInfoSystem;

/// Tracking allocator that instruments allocations for the bench crate.
#[derive(Debug)]
struct TrackingAllocator {
    total_allocated_bytes: AtomicU64,
    total_deallocated_bytes: AtomicU64,
    current_allocated_bytes: AtomicU64,
    peak_allocated_bytes: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
}

#[allow(dead_code)]
impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            total_allocated_bytes: AtomicU64::new(0),
            total_deallocated_bytes: AtomicU64::new(0),
            current_allocated_bytes: AtomicU64::new(0),
            peak_allocated_bytes: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }

    fn snapshot(&self) -> AllocatorStats {
        AllocatorStats {
            total_allocated_bytes: self.total_allocated_bytes.load(Ordering::Relaxed),
            total_deallocated_bytes: self.total_deallocated_bytes.load(Ordering::Relaxed),
            current_allocated_bytes: self.current_allocated_bytes.load(Ordering::Relaxed),
            peak_allocated_bytes: self.peak_allocated_bytes.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
        }
    }

    fn record_alloc(&self, size: usize) {
        let size = size as u64;
        self.total_allocated_bytes
            .fetch_add(size, Ordering::Relaxed);
        let current = self
            .current_allocated_bytes
            .fetch_add(size, Ordering::Relaxed)
            .saturating_add(size);

        // Update peak if current usage exceeded previous maximum.
        loop {
            let peak = self.peak_allocated_bytes.load(Ordering::Relaxed);
            if current <= peak {
                break;
            }
            if self
                .peak_allocated_bytes
                .compare_exchange(peak, current, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        self.allocation_count.fetch_add(1, Ordering::Relaxed);
    }

    fn record_dealloc(&self, size: usize) {
        let size = size as u64;
        self.total_deallocated_bytes
            .fetch_add(size, Ordering::Relaxed);
        self.current_allocated_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(size))
            })
            .ok();
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            self.record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.record_dealloc(layout.size());
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            if new_size > old_size {
                self.record_alloc(new_size - old_size);
            } else {
                self.record_dealloc(old_size - new_size);
            }
        }
        new_ptr
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

/// Snapshot of process memory usage in megabytes.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MemorySnapshot {
    pub total_memory_mb: f64,
    pub used_memory_mb: f64,
}

#[allow(dead_code)]
impl MemorySnapshot {
    /// Capture a best-effort snapshot of the current process memory usage.
    pub fn capture() -> Self {
        let mut system = SysInfoSystem::new_all();
        system.refresh_memory();

        // `sysinfo` reports memory in kibibytes; convert to megabytes.
        let total_memory_mb = system.total_memory() as f64 / 1024.0;
        let used_memory_mb = system.used_memory() as f64 / 1024.0;

        Self {
            total_memory_mb,
            used_memory_mb,
        }
    }
}

/// Difference between two memory snapshots.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MemoryDelta {
    pub total_delta_mb: f64,
    pub used_delta_mb: f64,
}

#[allow(dead_code)]
impl MemoryDelta {
    fn from_snapshots(before: &MemorySnapshot, after: &MemorySnapshot) -> Self {
        Self {
            total_delta_mb: after.total_memory_mb - before.total_memory_mb,
            used_delta_mb: after.used_memory_mb - before.used_memory_mb,
        }
    }
}

/// Allocator statistics captured before and after a benchmarked operation.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AllocatorStats {
    pub total_allocated_bytes: u64,
    pub total_deallocated_bytes: u64,
    pub current_allocated_bytes: u64,
    pub peak_allocated_bytes: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

#[allow(dead_code)]
impl AllocatorStats {
    fn capture() -> Self {
        GLOBAL_ALLOCATOR.snapshot()
    }
}

/// Delta between two allocator snapshots.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AllocatorDelta {
    pub allocated_bytes: i64,
    pub deallocated_bytes: i64,
    pub current_bytes: i64,
    pub allocation_count: i64,
    pub deallocation_count: i64,
}

#[allow(dead_code)]
impl AllocatorDelta {
    fn from_stats(before: &AllocatorStats, after: &AllocatorStats) -> Self {
        Self {
            allocated_bytes: after.total_allocated_bytes as i64
                - before.total_allocated_bytes as i64,
            deallocated_bytes: after.total_deallocated_bytes as i64
                - before.total_deallocated_bytes as i64,
            current_bytes: after.current_allocated_bytes as i64
                - before.current_allocated_bytes as i64,
            allocation_count: after.allocation_count as i64 - before.allocation_count as i64,
            deallocation_count: after.deallocation_count as i64 - before.deallocation_count as i64,
        }
    }
}

/// Individual performance measurement captured during a benchmark helper run.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PerformanceMeasurement {
    pub operation_name: String,
    pub duration_ms: f64,
    pub memory_before: MemorySnapshot,
    pub memory_after: MemorySnapshot,
    pub memory_delta: MemoryDelta,
    pub allocator_before: AllocatorStats,
    pub allocator_after: AllocatorStats,
    pub allocator_delta: AllocatorDelta,
}

/// Aggregate performance results for a benchmark scenario.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct PerformanceResults {
    pub measurements: Vec<PerformanceMeasurement>,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
}

#[allow(dead_code)]
impl PerformanceResults {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            started_at: Some(Instant::now()),
            completed_at: None,
        }
    }

    pub fn add_measurement(&mut self, measurement: PerformanceMeasurement) {
        self.measurements.push(measurement);
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Instant::now());
    }

    /// Produce a textual summary of the captured measurements.
    pub fn generate_summary(&self) -> String {
        if self.measurements.is_empty() {
            return "No performance measurements captured.".to_string();
        }

        let total_duration: f64 = self.measurements.iter().map(|m| m.duration_ms).sum();
        let average_duration = total_duration / self.measurements.len() as f64;
        let max_duration = self
            .measurements
            .iter()
            .map(|m| m.duration_ms)
            .fold(f64::MIN, f64::max);
        let min_duration = self
            .measurements
            .iter()
            .map(|m| m.duration_ms)
            .fold(f64::MAX, f64::min);

        let total_alloc_delta: i64 = self
            .measurements
            .iter()
            .map(|m| m.allocator_delta.allocated_bytes)
            .sum();
        let total_dealloc_delta: i64 = self
            .measurements
            .iter()
            .map(|m| m.allocator_delta.deallocated_bytes)
            .sum();
        let total_alloc_calls: i64 = self
            .measurements
            .iter()
            .map(|m| m.allocator_delta.allocation_count)
            .sum();
        let total_dealloc_calls: i64 = self
            .measurements
            .iter()
            .map(|m| m.allocator_delta.deallocation_count)
            .sum();

        let elapsed_ms = self
            .started_at
            .zip(self.completed_at)
            .map(|(start, end)| (end - start).as_secs_f64() * 1_000.0)
            .unwrap_or_default();

        format!(
            "Performance Summary:\n- Measurements: {}\n- Total duration: {:.2} ms\n- Average duration: {:.2} ms\n- Fastest: {:.2} ms\n- Slowest: {:.2} ms\n- Elapsed wall time: {:.2} ms\n- Allocated delta: {} bytes\n- Deallocated delta: {} bytes\n- Allocation calls: {}\n- Deallocation calls: {}",
            self.measurements.len(),
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            elapsed_ms,
            total_alloc_delta,
            total_dealloc_delta,
            total_alloc_calls,
            total_dealloc_calls
        )
    }
}

/// Execute `operation` while capturing duration, allocator, and memory data.
#[allow(dead_code)]
pub fn measure_performance<F, R>(
    operation_name: &str,
    mut operation: F,
) -> (R, PerformanceMeasurement)
where
    F: FnMut() -> R,
{
    let memory_before = MemorySnapshot::capture();
    let allocator_before = AllocatorStats::capture();
    let start = Instant::now();
    let result = operation();
    let duration_ms = start.elapsed().as_secs_f64() * 1_000.0;
    let memory_after = MemorySnapshot::capture();
    let allocator_after = AllocatorStats::capture();

    let memory_delta = MemoryDelta::from_snapshots(&memory_before, &memory_after);
    let allocator_delta = AllocatorDelta::from_stats(&allocator_before, &allocator_after);

    let measurement = PerformanceMeasurement {
        operation_name: operation_name.to_string(),
        duration_ms,
        memory_before,
        memory_after,
        memory_delta,
        allocator_before,
        allocator_after,
        allocator_delta,
    };

    (result, measurement)
}

/// Helper utilities for reporting.
pub mod utils {
    use super::PerformanceMeasurement;

    /// Generate a Markdown report of memory and allocation deltas captured in performance measurements.
    #[allow(dead_code)]
    pub fn generate_memory_report(measurements: &[PerformanceMeasurement]) -> String {
        if measurements.is_empty() {
            return "No memory measurements captured.".to_string();
        }

        let mut report = String::from("# Memory Usage Measurements\n\n");
        for measurement in measurements {
            report.push_str(&format!(
                "## {}\n- Duration: {:.2} ms\n- Used memory delta: {:.2} MB\n- Total memory delta: {:.2} MB\n- Allocated bytes: {}\n- Deallocated bytes: {}\n- Allocation calls: {}\n- Deallocation calls: {}\n\n",
                measurement.operation_name,
                measurement.duration_ms,
                measurement.memory_delta.used_delta_mb,
                measurement.memory_delta.total_delta_mb,
                measurement.allocator_delta.allocated_bytes,
                measurement.allocator_delta.deallocated_bytes,
                measurement.allocator_delta.allocation_count,
                measurement.allocator_delta.deallocation_count
            ));
        }

        report
    }
}
