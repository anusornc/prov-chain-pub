//! Performance Metrics Collection Module
//!
//! This module provides comprehensive performance metrics collection,
//! monitoring, and reporting capabilities for ProvChain.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Performance metrics collector
pub struct MetricsCollector {
    /// Collection interval
    _interval: Duration,
    /// Total operations processed
    total_operations: u64,
    /// Operation timing history
    operation_timings: VecDeque<(Instant, Duration)>,
    /// Memory usage samples
    memory_samples: VecDeque<(Instant, u64)>,
    /// Custom metrics
    custom_metrics: HashMap<String, f64>,
    /// Start time for uptime calculation
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(interval: Duration) -> Self {
        Self {
            _interval: interval,
            total_operations: 0,
            operation_timings: VecDeque::new(),
            memory_samples: VecDeque::new(),
            custom_metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    /// Record an operation timing
    pub fn record_operation(&mut self, duration: Duration) {
        self.total_operations += 1;
        self.operation_timings.push_back((Instant::now(), duration));

        // Keep only recent timings (last hour)
        let cutoff = Instant::now() - Duration::from_secs(3600);
        while let Some(&(timestamp, _)) = self.operation_timings.front() {
            if timestamp < cutoff {
                self.operation_timings.pop_front();
            } else {
                break;
            }
        }
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, memory_mb: u64) {
        self.memory_samples.push_back((Instant::now(), memory_mb));

        // Keep only recent samples (last hour)
        let cutoff = Instant::now() - Duration::from_secs(3600);
        while let Some(&(timestamp, _)) = self.memory_samples.front() {
            if timestamp < cutoff {
                self.memory_samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Set a custom metric
    pub fn set_custom_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }

    /// Get total operations processed
    pub fn get_total_operations(&self) -> u64 {
        self.total_operations
    }

    /// Get average operation time
    pub fn get_average_operation_time(&self) -> Duration {
        if self.operation_timings.is_empty() {
            Duration::from_secs(0)
        } else {
            let total: Duration = self
                .operation_timings
                .iter()
                .map(|(_, duration)| *duration)
                .sum();
            total / self.operation_timings.len() as u32
        }
    }

    /// Get operations per second
    pub fn get_operations_per_second(&self) -> f64 {
        if self.operation_timings.is_empty() {
            0.0
        } else {
            let window_duration = Duration::from_secs(60); // 1 minute window
            let cutoff = Instant::now() - window_duration;
            let recent_ops = self
                .operation_timings
                .iter()
                .filter(|(timestamp, _)| *timestamp >= cutoff)
                .count();

            recent_ops as f64 / window_duration.as_secs_f64()
        }
    }

    /// Get current memory usage
    pub fn get_current_memory_usage(&self) -> u64 {
        self.memory_samples
            .back()
            .map(|(_, memory)| *memory)
            .unwrap_or(0)
    }

    /// Get average memory usage
    pub fn get_average_memory_usage(&self) -> u64 {
        if self.memory_samples.is_empty() {
            0
        } else {
            let total: u64 = self.memory_samples.iter().map(|(_, memory)| *memory).sum();
            total / self.memory_samples.len() as u64
        }
    }

    /// Get uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get all metrics as a report
    pub fn generate_report(&self) -> MetricsReport {
        MetricsReport {
            uptime: self.get_uptime(),
            total_operations: self.total_operations,
            operations_per_second: self.get_operations_per_second(),
            average_operation_time: self.get_average_operation_time(),
            current_memory_usage: self.get_current_memory_usage(),
            average_memory_usage: self.get_average_memory_usage(),
            custom_metrics: self.custom_metrics.clone(),
        }
    }
}

/// Performance metrics report
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub uptime: Duration,
    pub total_operations: u64,
    pub operations_per_second: f64,
    pub average_operation_time: Duration,
    pub current_memory_usage: u64,
    pub average_memory_usage: u64,
    pub custom_metrics: HashMap<String, f64>,
}

impl MetricsReport {
    pub fn print_summary(&self) {
        println!("\n=== Performance Metrics Report ===");
        println!("Uptime: {:?}", self.uptime);
        println!("Total operations: {}", self.total_operations);
        println!("Operations per second: {:.2}", self.operations_per_second);
        println!("Average operation time: {:?}", self.average_operation_time);
        println!("Current memory usage: {} MB", self.current_memory_usage);
        println!("Average memory usage: {} MB", self.average_memory_usage);

        if !self.custom_metrics.is_empty() {
            println!("Custom metrics:");
            for (name, value) in &self.custom_metrics {
                println!("  {name}: {value:.2}");
            }
        }
        println!("==================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        assert_eq!(collector.get_total_operations(), 0);
        assert_eq!(
            collector.get_average_operation_time(),
            Duration::from_secs(0)
        );
        assert_eq!(collector.get_operations_per_second(), 0.0);
    }

    #[test]
    fn test_operation_recording() {
        let mut collector = MetricsCollector::new(Duration::from_secs(60));

        collector.record_operation(Duration::from_millis(100));
        collector.record_operation(Duration::from_millis(200));

        assert_eq!(collector.get_total_operations(), 2);
        assert_eq!(
            collector.get_average_operation_time(),
            Duration::from_millis(150)
        );
    }

    #[test]
    fn test_memory_recording() {
        let mut collector = MetricsCollector::new(Duration::from_secs(60));

        collector.record_memory_usage(100);
        collector.record_memory_usage(150);
        collector.record_memory_usage(200);

        assert_eq!(collector.get_current_memory_usage(), 200);
        assert_eq!(collector.get_average_memory_usage(), 150);
    }

    #[test]
    fn test_custom_metrics() {
        let mut collector = MetricsCollector::new(Duration::from_secs(60));

        collector.set_custom_metric("cache_hit_rate".to_string(), 0.85);
        collector.set_custom_metric("error_rate".to_string(), 0.02);

        let report = collector.generate_report();
        assert_eq!(report.custom_metrics.get("cache_hit_rate"), Some(&0.85));
        assert_eq!(report.custom_metrics.get("error_rate"), Some(&0.02));
    }

    #[test]
    fn test_uptime_calculation() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        let uptime = collector.get_uptime();

        // Should be a very small duration since we just created it
        assert!(uptime < Duration::from_secs(1));
    }

    #[test]
    fn test_metrics_report_generation() {
        let mut collector = MetricsCollector::new(Duration::from_secs(60));

        collector.record_operation(Duration::from_millis(50));
        collector.record_memory_usage(128);
        collector.set_custom_metric("test_metric".to_string(), 42.0);

        let report = collector.generate_report();

        assert_eq!(report.total_operations, 1);
        assert_eq!(report.average_operation_time, Duration::from_millis(50));
        assert_eq!(report.current_memory_usage, 128);
        assert_eq!(report.custom_metrics.get("test_metric"), Some(&42.0));
    }
}
