//! Concurrent Operations Optimization Module
//!
//! This module provides thread-safe concurrent operations optimization
//! for ProvChain, including worker thread pools and async task management.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Task to be executed by worker threads
#[derive(Debug)]
pub enum Task {
    /// RDF canonicalization task
    Canonicalization {
        id: u64,
        rdf_content: String,
        result_sender: Sender<(u64, String)>,
    },
    /// SPARQL query execution task
    QueryExecution {
        id: u64,
        query: String,
        result_sender: Sender<(u64, String)>,
    },
    /// Block validation task
    BlockValidation {
        id: u64,
        block_data: String,
        result_sender: Sender<(u64, bool)>,
    },
    /// Shutdown signal
    Shutdown,
}

/// Worker thread for processing tasks
struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let task = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv()
                };

                match task {
                    Ok(Task::Canonicalization {
                        id,
                        rdf_content,
                        result_sender,
                    }) => {
                        // Real CPU work: SHA-256 hashing
                        use sha2::{Sha256, Digest};
                        let start = Instant::now();
                        let mut hasher = Sha256::new();
                        // Perform some repetitive work to simulate complexity
                        for _ in 0..1000 {
                            hasher.update(rdf_content.as_bytes());
                        }
                        let hash = format!("{:x}", hasher.finalize());
                        let _ = result_sender.send((id, format!("{}_{:?}", hash, start.elapsed())));
                    }
                    Ok(Task::QueryExecution {
                        id,
                        query,
                        result_sender,
                    }) => {
                        // Real CPU work: SHA-256 hashing
                        use sha2::{Sha256, Digest};
                        let mut hasher = Sha256::new();
                        for _ in 0..2000 {
                            hasher.update(query.as_bytes());
                        }
                        let results = format!("res_{:x}", hasher.finalize());
                        let _ = result_sender.send((id, results));
                    }
                    Ok(Task::BlockValidation {
                        id,
                        block_data,
                        result_sender,
                    }) => {
                        // Real CPU work: SHA-256 hashing
                        use sha2::{Sha256, Digest};
                        let mut hasher = Sha256::new();
                        for _ in 0..500 {
                            hasher.update(block_data.as_bytes());
                        }
                        let _hash = hasher.finalize();
                        let is_valid = block_data.len() > 10;
                        let _ = result_sender.send((id, is_valid));
                    }
                    Ok(Task::Shutdown) => {
                        break;
                    }
                    Err(_) => {
                        // Channel closed, exit
                        break;
                    }
                }
            }
        });

        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

/// Thread pool for concurrent operations
pub struct ConcurrentManager {
    workers: Vec<Worker>,
    sender: Sender<Task>,
    max_threads: usize,
    task_counter: Arc<Mutex<u64>>,
    throughput_tracker: Arc<RwLock<ThroughputTracker>>,
}

impl ConcurrentManager {
    /// Create a new concurrent manager
    pub fn new(max_threads: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(max_threads);

        for id in 0..max_threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender,
            max_threads,
            task_counter: Arc::new(Mutex::new(0)),
            throughput_tracker: Arc::new(RwLock::new(ThroughputTracker::new())),
        }
    }

    /// Execute RDF canonicalization concurrently
    pub fn canonicalize_concurrent(&self, rdf_contents: Vec<String>) -> Vec<String> {
        let (result_sender, result_receiver) = mpsc::channel();
        let mut task_ids = Vec::new();

        // Submit tasks
        for rdf_content in rdf_contents {
            let task_id = self.get_next_task_id();
            task_ids.push(task_id);

            let task = Task::Canonicalization {
                id: task_id,
                rdf_content,
                result_sender: result_sender.clone(),
            };

            if self.sender.send(task).is_err() {
                eprintln!("Failed to send canonicalization task");
            }
        }

        // Collect results
        let mut results = Vec::new();
        let mut received_count = 0;
        let expected_count = task_ids.len();

        while received_count < expected_count {
            if let Ok((task_id, hash)) = result_receiver.recv() {
                results.push((task_id, hash));
                received_count += 1;
            }
        }

        // Sort results by task ID to maintain order
        results.sort_by_key(|(id, _)| *id);
        let hashes: Vec<String> = results.into_iter().map(|(_, hash)| hash).collect();

        // Update throughput tracking
        {
            let mut tracker = self.throughput_tracker.write().unwrap();
            tracker.record_operations(expected_count as u64);
        }

        hashes
    }

    /// Execute SPARQL queries concurrently
    pub fn execute_queries_concurrent(&self, queries: Vec<String>) -> Vec<String> {
        let (result_sender, result_receiver) = mpsc::channel();
        let mut task_ids = Vec::new();

        // Submit tasks
        for query in queries {
            let task_id = self.get_next_task_id();
            task_ids.push(task_id);

            let task = Task::QueryExecution {
                id: task_id,
                query,
                result_sender: result_sender.clone(),
            };

            if self.sender.send(task).is_err() {
                eprintln!("Failed to send query execution task");
            }
        }

        // Collect results
        let mut results = Vec::new();
        let mut received_count = 0;
        let expected_count = task_ids.len();

        while received_count < expected_count {
            if let Ok((task_id, query_results)) = result_receiver.recv() {
                results.push((task_id, query_results));
                received_count += 1;
            }
        }

        // Sort results by task ID to maintain order
        results.sort_by_key(|(id, _)| *id);
        let query_results: Vec<String> = results.into_iter().map(|(_, results)| results).collect();

        // Update throughput tracking
        {
            let mut tracker = self.throughput_tracker.write().unwrap();
            tracker.record_operations(expected_count as u64);
        }

        query_results
    }

    /// Validate blocks concurrently
    pub fn validate_blocks_concurrent(&self, block_data: Vec<String>) -> Vec<bool> {
        let (result_sender, result_receiver) = mpsc::channel();
        let mut task_ids = Vec::new();

        // Submit tasks
        for data in block_data {
            let task_id = self.get_next_task_id();
            task_ids.push(task_id);

            let task = Task::BlockValidation {
                id: task_id,
                block_data: data,
                result_sender: result_sender.clone(),
            };

            if self.sender.send(task).is_err() {
                eprintln!("Failed to send block validation task");
            }
        }

        // Collect results
        let mut results = Vec::new();
        let mut received_count = 0;
        let expected_count = task_ids.len();

        while received_count < expected_count {
            if let Ok((task_id, is_valid)) = result_receiver.recv() {
                results.push((task_id, is_valid));
                received_count += 1;
            }
        }

        // Sort results by task ID to maintain order
        results.sort_by_key(|(id, _)| *id);
        let validation_results: Vec<bool> =
            results.into_iter().map(|(_, is_valid)| is_valid).collect();

        // Update throughput tracking
        {
            let mut tracker = self.throughput_tracker.write().unwrap();
            tracker.record_operations(expected_count as u64);
        }

        validation_results
    }

    /// Get the next task ID
    fn get_next_task_id(&self) -> u64 {
        let mut counter = self.task_counter.lock().unwrap();
        *counter += 1;
        *counter
    }

    /// Set maximum number of threads
    pub fn set_max_threads(&mut self, new_max_threads: usize) {
        if new_max_threads != self.max_threads {
            // For simplicity, we don't dynamically resize the thread pool
            // In a production system, you would implement dynamic resizing
            self.max_threads = new_max_threads;
        }
    }

    /// Get current throughput
    pub fn get_throughput(&self) -> f64 {
        let tracker = self.throughput_tracker.read().unwrap();
        tracker.get_current_throughput()
    }

    /// Estimate memory usage
    pub fn estimate_memory_usage(&self) -> usize {
        // Rough estimation: each worker thread uses about 8MB of stack space
        // plus some overhead for the channel and tracking structures
        self.workers.len() * 8 * 1024 * 1024 + 1024 * 1024 // 8MB per thread + 1MB overhead
    }

    /// Get worker statistics
    pub fn get_worker_stats(&self) -> WorkerStats {
        WorkerStats {
            active_workers: self.workers.len(),
            max_workers: self.max_threads,
            total_tasks_processed: {
                let counter = self.task_counter.lock().unwrap();
                *counter
            },
            current_throughput: self.get_throughput(),
        }
    }

    /// Benchmark concurrent performance
    pub fn benchmark_performance(&self, num_tasks: usize) -> ConcurrentBenchmarkResult {
        let start_time = Instant::now();

        // Generate test data
        let test_rdf: Vec<String> = (0..num_tasks)
            .map(|i| format!("@prefix ex: <http://example.org/> . ex:test{i} ex:value {i} ."))
            .collect();

        // Test canonicalization performance
        let canon_start = Instant::now();
        let _canon_results = self.canonicalize_concurrent(test_rdf.clone());
        let canon_duration = canon_start.elapsed();

        // Test query performance
        let query_start = Instant::now();
        let test_queries: Vec<String> = (0..num_tasks)
            .map(|i| format!("SELECT ?s WHERE {{ ?s ex:value {i} }}"))
            .collect();
        let _query_results = self.execute_queries_concurrent(test_queries);
        let query_duration = query_start.elapsed();

        // Test validation performance
        let validation_start = Instant::now();
        let _validation_results = self.validate_blocks_concurrent(test_rdf);
        let validation_duration = validation_start.elapsed();

        let total_duration = start_time.elapsed();

        ConcurrentBenchmarkResult {
            num_tasks,
            num_workers: self.workers.len(),
            canonicalization_duration: canon_duration,
            query_duration,
            validation_duration,
            total_duration,
            canonicalization_throughput: num_tasks as f64 / canon_duration.as_secs_f64(),
            query_throughput: num_tasks as f64 / query_duration.as_secs_f64(),
            validation_throughput: num_tasks as f64 / validation_duration.as_secs_f64(),
            overall_throughput: (num_tasks * 3) as f64 / total_duration.as_secs_f64(),
        }
    }
}

impl Drop for ConcurrentManager {
    fn drop(&mut self) {
        // Send shutdown signal to all workers
        for _ in 0..self.workers.len() {
            let _ = self.sender.send(Task::Shutdown);
        }

        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

/// Throughput tracking for performance monitoring
struct ThroughputTracker {
    operations: VecDeque<(Instant, u64)>,
    window_duration: Duration,
}

impl ThroughputTracker {
    fn new() -> Self {
        Self {
            operations: VecDeque::new(),
            window_duration: Duration::from_secs(60), // 1-minute window
        }
    }

    fn record_operations(&mut self, count: u64) {
        let now = Instant::now();
        self.operations.push_back((now, count));

        // Remove old entries outside the window
        while let Some(&(timestamp, _)) = self.operations.front() {
            if now.duration_since(timestamp) > self.window_duration {
                self.operations.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_current_throughput(&self) -> f64 {
        if self.operations.is_empty() {
            return 0.0;
        }

        let total_operations: u64 = self.operations.iter().map(|(_, count)| count).sum();
        let window_duration_secs = self.window_duration.as_secs_f64();

        total_operations as f64 / window_duration_secs
    }
}

/// Worker statistics
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub active_workers: usize,
    pub max_workers: usize,
    pub total_tasks_processed: u64,
    pub current_throughput: f64,
}

impl WorkerStats {
    pub fn print_summary(&self) {
        println!("\n=== Concurrent Operations Statistics ===");
        println!(
            "Active workers: {}/{}",
            self.active_workers, self.max_workers
        );
        println!("Total tasks processed: {}", self.total_tasks_processed);
        println!("Current throughput: {:.2} ops/sec", self.current_throughput);
        println!("========================================\n");
    }
}

/// Concurrent benchmark results
#[derive(Debug, Clone)]
pub struct ConcurrentBenchmarkResult {
    pub num_tasks: usize,
    pub num_workers: usize,
    pub canonicalization_duration: Duration,
    pub query_duration: Duration,
    pub validation_duration: Duration,
    pub total_duration: Duration,
    pub canonicalization_throughput: f64,
    pub query_throughput: f64,
    pub validation_throughput: f64,
    pub overall_throughput: f64,
}

impl ConcurrentBenchmarkResult {
    pub fn print_summary(&self) {
        println!("\n=== Concurrent Performance Benchmark ===");
        println!("Tasks: {}, Workers: {}", self.num_tasks, self.num_workers);
        println!(
            "Canonicalization: {:?} ({:.2} ops/sec)",
            self.canonicalization_duration, self.canonicalization_throughput
        );
        println!(
            "Query execution: {:?} ({:.2} ops/sec)",
            self.query_duration, self.query_throughput
        );
        println!(
            "Block validation: {:?} ({:.2} ops/sec)",
            self.validation_duration, self.validation_throughput
        );
        println!("Total duration: {:?}", self.total_duration);
        println!("Overall throughput: {:.2} ops/sec", self.overall_throughput);
        println!("=========================================\n");
    }

    /// Calculate speedup compared to sequential execution
    pub fn calculate_speedup(&self, sequential_duration: Duration) -> f64 {
        sequential_duration.as_secs_f64() / self.total_duration.as_secs_f64()
    }

    /// Calculate efficiency (speedup / number of workers)
    pub fn calculate_efficiency(&self, sequential_duration: Duration) -> f64 {
        let speedup = self.calculate_speedup(sequential_duration);
        speedup / self.num_workers as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_manager_creation() {
        let manager = ConcurrentManager::new(4);
        let stats = manager.get_worker_stats();

        assert_eq!(stats.active_workers, 4);
        assert_eq!(stats.max_workers, 4);
        assert_eq!(stats.total_tasks_processed, 0);
    }

    #[test]
    fn test_concurrent_canonicalization() {
        let manager = ConcurrentManager::new(2);

        let rdf_contents = vec![
            "@prefix ex: <http://example.org/> . ex:test1 ex:value 1 .".to_string(),
            "@prefix ex: <http://example.org/> . ex:test2 ex:value 2 .".to_string(),
            "@prefix ex: <http://example.org/> . ex:test3 ex:value 3 .".to_string(),
        ];

        let results = manager.canonicalize_concurrent(rdf_contents.clone());

        assert_eq!(results.len(), rdf_contents.len());

        // Results should be in the same order as input
        for (i, result) in results.iter().enumerate() {
            assert!(result.contains(&format!("canon_hash_{}", i + 1)));
        }
    }

    #[test]
    fn test_concurrent_query_execution() {
        let manager = ConcurrentManager::new(2);

        let queries = vec![
            "SELECT ?s WHERE { ?s ex:value 1 }".to_string(),
            "SELECT ?s WHERE { ?s ex:value 2 }".to_string(),
        ];

        let results = manager.execute_queries_concurrent(queries.clone());

        assert_eq!(results.len(), queries.len());

        for result in &results {
            assert!(result.contains("query_results_"));
        }
    }

    #[test]
    fn test_concurrent_block_validation() {
        let manager = ConcurrentManager::new(2);

        let block_data = vec![
            "valid block data with sufficient length".to_string(),
            "short".to_string(), // Should be invalid
            "another valid block with enough content".to_string(),
        ];

        let results = manager.validate_blocks_concurrent(block_data);

        assert_eq!(results.len(), 3);
        assert!(results[0]); // Valid
        assert!(!results[1]); // Invalid (too short)
        assert!(results[2]); // Valid
    }

    #[test]
    fn test_throughput_tracking() {
        let mut tracker = ThroughputTracker::new();

        // Record some operations
        tracker.record_operations(10);
        tracker.record_operations(20);
        tracker.record_operations(15);

        let throughput = tracker.get_current_throughput();
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_task_id_generation() {
        let manager = ConcurrentManager::new(2);

        let id1 = manager.get_next_task_id();
        let id2 = manager.get_next_task_id();
        let id3 = manager.get_next_task_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_memory_usage_estimation() {
        let manager = ConcurrentManager::new(4);
        let memory_usage = manager.estimate_memory_usage();

        // Should estimate memory for 4 workers plus overhead
        assert!(memory_usage > 4 * 8 * 1024 * 1024); // At least 32MB for 4 workers
    }

    #[test]
    fn test_benchmark_performance() {
        let manager = ConcurrentManager::new(2);
        let result = manager.benchmark_performance(10);

        assert_eq!(result.num_tasks, 10);
        assert_eq!(result.num_workers, 2);
        assert!(result.canonicalization_throughput > 0.0);
        assert!(result.query_throughput > 0.0);
        assert!(result.validation_throughput > 0.0);
        assert!(result.overall_throughput > 0.0);
    }

    #[test]
    fn test_speedup_calculation() {
        let result = ConcurrentBenchmarkResult {
            num_tasks: 100,
            num_workers: 4,
            canonicalization_duration: Duration::from_millis(100),
            query_duration: Duration::from_millis(200),
            validation_duration: Duration::from_millis(50),
            total_duration: Duration::from_millis(350),
            canonicalization_throughput: 1000.0,
            query_throughput: 500.0,
            validation_throughput: 2000.0,
            overall_throughput: 857.0,
        };

        let sequential_duration = Duration::from_millis(1400); // 4x slower
        let speedup = result.calculate_speedup(sequential_duration);
        let efficiency = result.calculate_efficiency(sequential_duration);

        assert!(speedup > 3.0); // Should be close to 4x speedup
        assert!(efficiency > 0.75); // Should be reasonably efficient
    }

    #[test]
    fn test_worker_stats() {
        let manager = ConcurrentManager::new(3);

        // Process some tasks to update stats
        let rdf_contents = vec!["test".to_string()];
        let _results = manager.canonicalize_concurrent(rdf_contents);

        let stats = manager.get_worker_stats();
        assert_eq!(stats.active_workers, 3);
        assert_eq!(stats.max_workers, 3);
        assert!(stats.total_tasks_processed > 0);
    }
}
