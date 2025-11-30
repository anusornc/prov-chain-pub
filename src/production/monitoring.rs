//! Monitoring and observability for production deployment

use crate::production::ProductionError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    pub prometheus_enabled: bool,
    /// Prometheus metrics port
    pub prometheus_port: u16,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Jaeger endpoint for tracing
    pub jaeger_endpoint: Option<String>,
    /// Metrics collection interval in seconds
    pub metrics_interval: u64,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Enable structured logging
    pub structured_logging: bool,
    /// Custom metrics configuration
    pub custom_metrics: Vec<CustomMetricConfig>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            prometheus_port: 9090,
            tracing_enabled: true,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            metrics_interval: 30,
            log_level: "info".to_string(),
            structured_logging: true,
            custom_metrics: vec![
                CustomMetricConfig {
                    name: "blockchain_blocks_total".to_string(),
                    metric_type: MetricType::Counter,
                    description: "Total number of blocks in the blockchain".to_string(),
                    labels: vec!["chain_id".to_string()],
                },
                CustomMetricConfig {
                    name: "rdf_triples_total".to_string(),
                    metric_type: MetricType::Counter,
                    description: "Total number of RDF triples stored".to_string(),
                    labels: vec!["graph_name".to_string()],
                },
                CustomMetricConfig {
                    name: "api_request_duration_seconds".to_string(),
                    metric_type: MetricType::Histogram,
                    description: "API request duration in seconds".to_string(),
                    labels: vec![
                        "method".to_string(),
                        "endpoint".to_string(),
                        "status".to_string(),
                    ],
                },
                CustomMetricConfig {
                    name: "system_memory_usage_bytes".to_string(),
                    metric_type: MetricType::Gauge,
                    description: "System memory usage in bytes".to_string(),
                    labels: vec!["type".to_string()],
                },
            ],
        }
    }
}

/// Custom metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub labels: Vec<String>,
}

/// Metric types supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// System metrics collected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_usage_bytes: u64,
    pub disk_total_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub open_file_descriptors: u64,
    pub uptime_seconds: u64,
}

/// Application metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub blockchain_blocks: u64,
    pub rdf_triples: u64,
    pub active_connections: u64,
    pub api_requests_total: u64,
    pub api_requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub cache_hit_rate_percent: f64,
}

/// Monitoring manager
pub struct MonitoringManager {
    config: MonitoringConfig,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    app_metrics: Arc<RwLock<ApplicationMetrics>>,
    metrics_history: Arc<RwLock<Vec<(SystemMetrics, ApplicationMetrics)>>>,
    start_time: Instant,
}

impl MonitoringManager {
    /// Create a new monitoring manager
    pub fn new(config: MonitoringConfig) -> Result<Self, ProductionError> {
        let system_metrics = Arc::new(RwLock::new(SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_total_bytes: 0,
            disk_usage_bytes: 0,
            disk_total_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            open_file_descriptors: 0,
            uptime_seconds: 0,
        }));

        let app_metrics = Arc::new(RwLock::new(ApplicationMetrics {
            timestamp: chrono::Utc::now(),
            blockchain_blocks: 0,
            rdf_triples: 0,
            active_connections: 0,
            api_requests_total: 0,
            api_requests_per_second: 0.0,
            average_response_time_ms: 0.0,
            error_rate_percent: 0.0,
            cache_hit_rate_percent: 0.0,
        }));

        Ok(Self {
            config,
            system_metrics,
            app_metrics,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        })
    }

    /// Start monitoring services
    pub async fn start(&mut self) -> Result<(), ProductionError> {
        tracing::info!("Starting monitoring services");

        // Initialize Prometheus metrics if enabled
        if self.config.prometheus_enabled {
            self.initialize_prometheus().await?;
        }

        // Initialize distributed tracing if enabled
        if self.config.tracing_enabled {
            self.initialize_tracing().await?;
        }

        // Start metrics collection loop
        self.start_metrics_collection().await?;

        tracing::info!("Monitoring services started successfully");
        Ok(())
    }

    /// Initialize Prometheus metrics
    async fn initialize_prometheus(&self) -> Result<(), ProductionError> {
        tracing::info!(
            "Initializing Prometheus metrics on port {}",
            self.config.prometheus_port
        );

        // Register custom metrics
        for metric_config in &self.config.custom_metrics {
            tracing::debug!("Registering metric: {}", metric_config.name);
        }

        Ok(())
    }

    /// Initialize distributed tracing
    async fn initialize_tracing(&self) -> Result<(), ProductionError> {
        if let Some(jaeger_endpoint) = &self.config.jaeger_endpoint {
            tracing::info!(
                "Initializing distributed tracing with Jaeger endpoint: {}",
                jaeger_endpoint
            );
        }
        Ok(())
    }

    /// Start metrics collection loop
    async fn start_metrics_collection(&self) -> Result<(), ProductionError> {
        let system_metrics = Arc::clone(&self.system_metrics);
        let app_metrics = Arc::clone(&self.app_metrics);
        let metrics_history = Arc::clone(&self.metrics_history);
        let interval = Duration::from_secs(self.config.metrics_interval);
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Collect system metrics
                let sys_metrics = Self::collect_system_metrics(start_time).await;
                let app_metrics_data = Self::collect_application_metrics().await;

                // Update current metrics
                {
                    let mut sys_guard = system_metrics.write().await;
                    *sys_guard = sys_metrics.clone();
                }
                {
                    let mut app_guard = app_metrics.write().await;
                    *app_guard = app_metrics_data.clone();
                }

                // Store in history (keep last 1000 entries)
                {
                    let mut history = metrics_history.write().await;
                    history.push((sys_metrics, app_metrics_data));
                    if history.len() > 1000 {
                        history.remove(0);
                    }
                }
            }
        });

        Ok(())
    }

    /// Collect system metrics
    async fn collect_system_metrics(start_time: Instant) -> SystemMetrics {
        // Simplified implementation - in production would use proper system monitoring
        SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: 0.0,   // Would be collected from system
            memory_usage_bytes: 0,    // Would be collected from system
            memory_total_bytes: 0,    // Would be collected from system
            disk_usage_bytes: 0,      // Would be collected from system
            disk_total_bytes: 0,      // Would be collected from system
            network_rx_bytes: 0,      // Would be collected from system
            network_tx_bytes: 0,      // Would be collected from system
            open_file_descriptors: 0, // Would need platform-specific implementation
            uptime_seconds: start_time.elapsed().as_secs(),
        }
    }

    /// Collect application metrics
    async fn collect_application_metrics() -> ApplicationMetrics {
        // In a real implementation, these would be collected from the actual application
        ApplicationMetrics {
            timestamp: chrono::Utc::now(),
            blockchain_blocks: 0,          // Would be collected from blockchain
            rdf_triples: 0,                // Would be collected from RDF store
            active_connections: 0,         // Would be collected from web server
            api_requests_total: 0,         // Would be collected from request counter
            api_requests_per_second: 0.0,  // Calculated from request rate
            average_response_time_ms: 0.0, // Calculated from response times
            error_rate_percent: 0.0,       // Calculated from error count
            cache_hit_rate_percent: 0.0,   // Would be collected from cache
        }
    }

    /// Get current system status
    pub async fn status(&self) -> String {
        let sys_metrics = self.system_metrics.read().await;
        let app_metrics = self.app_metrics.read().await;

        format!(
            "CPU: {:.1}%, Memory: {:.1}%, Uptime: {}s, Blocks: {}, API RPS: {:.1}",
            sys_metrics.cpu_usage_percent,
            (sys_metrics.memory_usage_bytes as f64 / sys_metrics.memory_total_bytes as f64) * 100.0,
            sys_metrics.uptime_seconds,
            app_metrics.blockchain_blocks,
            app_metrics.api_requests_per_second
        )
    }

    /// Get detailed metrics
    pub async fn get_metrics(&self) -> (SystemMetrics, ApplicationMetrics) {
        let sys_metrics = self.system_metrics.read().await.clone();
        let app_metrics = self.app_metrics.read().await.clone();
        (sys_metrics, app_metrics)
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self) -> Vec<(SystemMetrics, ApplicationMetrics)> {
        self.metrics_history.read().await.clone()
    }

    /// Generate Prometheus configuration
    pub fn generate_prometheus_config(&self) -> String {
        format!(
            r#"global:
  scrape_interval: {}s
  evaluation_interval: {}s

rule_files:
  - "provchain_rules.yml"

scrape_configs:
  - job_name: 'provchain'
    static_configs:
      - targets: ['localhost:{}']
    metrics_path: /metrics
    scrape_interval: {}s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

# Custom recording rules
recording_rules:
  - name: provchain.rules
    rules:
      - record: provchain:api_request_rate_5m
        expr: rate(api_request_duration_seconds_count[5m])
      
      - record: provchain:error_rate_5m
        expr: rate(api_request_duration_seconds_count[5m])
      
      - record: provchain:response_time_95th
        expr: histogram_quantile(0.95, rate(api_request_duration_seconds_bucket[5m]))

# Alerting rules
alerting_rules:
  - name: provchain.alerts
    rules:
      - alert: HighErrorRate
        expr: provchain:error_rate_5m > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} for the last 5 minutes"

      - alert: HighResponseTime
        expr: provchain:response_time_95th > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time detected"
          description: "95th percentile response time is {{ $value }}s"

      - alert: HighMemoryUsage
        expr: (system_memory_usage_bytes / system_memory_total_bytes) > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanizePercentage }}"

      - alert: HighCPUUsage
        expr: system_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}%"
"#,
            self.config.metrics_interval,
            self.config.metrics_interval,
            self.config.prometheus_port,
            self.config.metrics_interval
        )
    }

    /// Generate Grafana dashboard configuration
    pub fn generate_grafana_dashboard(&self) -> String {
        r#"{
  "dashboard": {
    "id": null,
    "title": "ProvChain Monitoring Dashboard",
    "tags": ["provchain", "blockchain", "rdf"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "System Overview",
        "type": "stat",
        "targets": [
          {
            "expr": "system_cpu_usage_percent",
            "legendFormat": "CPU Usage %"
          },
          {
            "expr": "(system_memory_usage_bytes / system_memory_total_bytes) * 100",
            "legendFormat": "Memory Usage %"
          },
          {
            "expr": "system_uptime_seconds",
            "legendFormat": "Uptime (s)"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Blockchain Metrics",
        "type": "graph",
        "targets": [
          {
            "expr": "blockchain_blocks_total",
            "legendFormat": "Total Blocks"
          },
          {
            "expr": "rdf_triples_total",
            "legendFormat": "RDF Triples"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "API Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(api_request_duration_seconds_count[5m])",
            "legendFormat": "Requests/sec"
          },
          {
            "expr": "histogram_quantile(0.95, rate(api_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile response time"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(api_request_duration_seconds_count{status=~\"5..\"}[5m]) / rate(api_request_duration_seconds_count[5m])",
            "legendFormat": "Error Rate"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}"#.to_string()
    }

    /// Stop monitoring services
    pub async fn stop(&mut self) -> Result<(), ProductionError> {
        tracing::info!("Stopping monitoring services");
        // In a real implementation, this would stop background tasks
        Ok(())
    }
}

/// Metrics recorder for application events
pub struct MetricsRecorder {
    request_count: Arc<RwLock<u64>>,
    error_count: Arc<RwLock<u64>>,
    response_times: Arc<RwLock<Vec<f64>>>,
}

impl MetricsRecorder {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
            response_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record an API request
    pub async fn record_request(&self, duration_ms: f64, is_error: bool) {
        {
            let mut count = self.request_count.write().await;
            *count += 1;
        }

        if is_error {
            let mut errors = self.error_count.write().await;
            *errors += 1;
        }

        {
            let mut times = self.response_times.write().await;
            times.push(duration_ms);
            // Keep only last 1000 response times
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    /// Get current metrics
    pub async fn get_stats(&self) -> (u64, u64, f64) {
        let request_count = *self.request_count.read().await;
        let error_count = *self.error_count.read().await;
        let response_times = self.response_times.read().await;

        let avg_response_time = if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        };

        (request_count, error_count, avg_response_time)
    }
}

impl Default for MetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}
