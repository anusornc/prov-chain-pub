//! Real-time integrity monitoring system
//!
//! This module provides real-time monitoring of system integrity,
//! including alerting and notification capabilities for integrity issues.

use crate::core::blockchain::Blockchain;
use crate::error::Result;
use crate::integrity::{
    IntegrityStatus, IntegrityValidationReport, IntegrityValidator, RecommendationSeverity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, error, info, instrument, warn};

/// Performance metrics collector for monitoring system performance
#[derive(Debug, Clone)]
pub struct PerformanceMetricsCollector {
    /// Validation performance history
    pub validation_times: VecDeque<Duration>,
    /// Memory usage tracking
    pub memory_usage_history: VecDeque<usize>,
    /// Throughput metrics
    pub throughput_history: VecDeque<f64>,
    /// Error rate tracking
    pub error_rates: VecDeque<f64>,
    /// Maximum history size
    pub max_history_size: usize,
}

impl Default for PerformanceMetricsCollector {
    fn default() -> Self {
        Self {
            validation_times: VecDeque::new(),
            memory_usage_history: VecDeque::new(),
            throughput_history: VecDeque::new(),
            error_rates: VecDeque::new(),
            max_history_size: 1000,
        }
    }
}

impl PerformanceMetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_validation_time(&mut self, duration: Duration) {
        self.validation_times.push_back(duration);
        if self.validation_times.len() > self.max_history_size {
            self.validation_times.pop_front();
        }
    }

    pub fn record_memory_usage(&mut self, usage: usize) {
        self.memory_usage_history.push_back(usage);
        if self.memory_usage_history.len() > self.max_history_size {
            self.memory_usage_history.pop_front();
        }
    }

    pub fn get_average_validation_time(&self) -> Duration {
        if self.validation_times.is_empty() {
            Duration::from_secs(0)
        } else {
            let total: Duration = self.validation_times.iter().sum();
            total / self.validation_times.len() as u32
        }
    }

    pub fn get_performance_trend(&self) -> PerformanceTrend {
        if self.validation_times.len() < 10 {
            return PerformanceTrend::Stable;
        }

        let recent_avg = self.validation_times.iter().rev().take(5).sum::<Duration>() / 5;
        let older_avg = self
            .validation_times
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .sum::<Duration>()
            / 5;

        if recent_avg > older_avg * 2 {
            PerformanceTrend::Degrading
        } else if recent_avg < older_avg / 2 {
            PerformanceTrend::Improving
        } else {
            PerformanceTrend::Stable
        }
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

/// Alert manager for handling different notification channels
#[derive(Debug, Clone)]
pub struct AlertManager {
    /// Email notification configuration
    pub email_config: Option<EmailConfig>,
    /// Webhook notification configuration
    pub webhook_config: Option<WebhookConfig>,
    /// Slack notification configuration
    pub slack_config: Option<SlackConfig>,
    /// Alert history
    pub alert_history: VecDeque<IntegrityAlert>,
    /// Maximum alert history size
    pub max_history_size: usize,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self {
            email_config: None,
            webhook_config: None,
            slack_config: None,
            alert_history: VecDeque::new(),
            max_history_size: 1000,
        }
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure_email(&mut self, config: EmailConfig) {
        self.email_config = Some(config);
    }

    pub fn configure_webhook(&mut self, config: WebhookConfig) {
        self.webhook_config = Some(config);
    }

    pub fn configure_slack(&mut self, config: SlackConfig) {
        self.slack_config = Some(config);
    }

    pub async fn send_alert(&mut self, alert: IntegrityAlert) -> Result<()> {
        // Store alert in history
        self.alert_history.push_back(alert.clone());
        if self.alert_history.len() > self.max_history_size {
            self.alert_history.pop_front();
        }

        // Send notifications based on configuration
        if let Some(_email_config) = &self.email_config {
            self.send_email_alert(&alert).await?;
        }

        if let Some(_webhook_config) = &self.webhook_config {
            self.send_webhook_alert(&alert).await?;
        }

        if let Some(_slack_config) = &self.slack_config {
            self.send_slack_alert(&alert).await?;
        }

        Ok(())
    }

    async fn send_email_alert(&self, alert: &IntegrityAlert) -> Result<()> {
        // TODO: Implement email notification
        debug!("Email alert would be sent: {}", alert.message);
        Ok(())
    }

    async fn send_webhook_alert(&self, alert: &IntegrityAlert) -> Result<()> {
        // TODO: Implement webhook notification
        debug!("Webhook alert would be sent: {}", alert.message);
        Ok(())
    }

    async fn send_slack_alert(&self, alert: &IntegrityAlert) -> Result<()> {
        // TODO: Implement Slack notification
        debug!("Slack alert would be sent: {}", alert.message);
        Ok(())
    }
}

/// Historical monitoring data storage
#[derive(Debug, Clone)]
pub struct MonitoringHistory {
    /// Historical integrity reports
    pub reports: VecDeque<IntegrityValidationReport>,
    /// Performance metrics over time
    pub performance_metrics: VecDeque<PerformanceSnapshot>,
    /// Alert history
    pub alerts: VecDeque<IntegrityAlert>,
    /// Maximum history size
    pub max_history_size: usize,
}

impl Default for MonitoringHistory {
    fn default() -> Self {
        Self {
            reports: VecDeque::new(),
            performance_metrics: VecDeque::new(),
            alerts: VecDeque::new(),
            max_history_size: 10000,
        }
    }
}

impl MonitoringHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_report(&mut self, report: IntegrityValidationReport) {
        self.reports.push_back(report);
        if self.reports.len() > self.max_history_size {
            self.reports.pop_front();
        }
    }

    pub fn add_performance_snapshot(&mut self, snapshot: PerformanceSnapshot) {
        self.performance_metrics.push_back(snapshot);
        if self.performance_metrics.len() > self.max_history_size {
            self.performance_metrics.pop_front();
        }
    }

    pub fn get_health_trend(&self, window_size: usize) -> HealthTrend {
        if self.reports.len() < window_size {
            return HealthTrend::Insufficient;
        }

        let recent_reports: Vec<_> = self.reports.iter().rev().take(window_size).collect();
        let healthy_count = recent_reports
            .iter()
            .filter(|r| matches!(r.overall_status, IntegrityStatus::Healthy))
            .count();

        let health_percentage = (healthy_count as f64 / window_size as f64) * 100.0;

        if health_percentage >= 90.0 {
            HealthTrend::Excellent
        } else if health_percentage >= 75.0 {
            HealthTrend::Good
        } else if health_percentage >= 50.0 {
            HealthTrend::Concerning
        } else {
            HealthTrend::Critical
        }
    }
}

/// Performance snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub validation_time: Duration,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub throughput: f64,
}

/// Health trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthTrend {
    Excellent,
    Good,
    Concerning,
    Critical,
    Insufficient,
}

/// Email notification configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub recipients: Vec<String>,
}

/// Webhook notification configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: u64,
}

/// Slack notification configuration
#[derive(Debug, Clone)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
    pub username: String,
}

/// Real-time monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEvent {
    IntegrityCheckCompleted {
        timestamp: DateTime<Utc>,
        status: IntegrityStatus,
        duration: Duration,
    },
    AlertTriggered {
        timestamp: DateTime<Utc>,
        alert_type: AlertType,
        message: String,
    },
    PerformanceThresholdExceeded {
        timestamp: DateTime<Utc>,
        metric: String,
        value: f64,
        threshold: f64,
    },
    SystemRecovered {
        timestamp: DateTime<Utc>,
        previous_status: IntegrityStatus,
    },
}

/// Integrity alert
#[derive(Debug, Clone)]
pub struct IntegrityAlert {
    pub alert_type: AlertType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub report_summary: crate::integrity::IntegrityReportSummary,
    pub monitoring_stats: MonitoringStatistics,
}

/// Alert types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    Critical,
    Warning,
    Recovery,
    MonitoringFailure,
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitoringStatistics {
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub warning_checks: usize,
    pub critical_checks: usize,
    pub corrupted_checks: usize,
    pub failed_checks: usize,
    pub total_check_time: Duration,
    pub last_check_time: Option<Instant>,
}

impl MonitoringStatistics {
    pub fn new() -> Self {
        Self {
            total_checks: 0,
            healthy_checks: 0,
            warning_checks: 0,
            critical_checks: 0,
            corrupted_checks: 0,
            failed_checks: 0,
            total_check_time: Duration::from_secs(0),
            last_check_time: None,
        }
    }

    pub fn average_check_time(&self) -> Duration {
        if self.total_checks > 0 {
            self.total_check_time / self.total_checks as u32
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn health_percentage(&self) -> f64 {
        if self.total_checks > 0 {
            (self.healthy_checks as f64 / self.total_checks as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn has_recent_issues(&self) -> bool {
        self.critical_checks > 0 || self.corrupted_checks > 0
    }
}

/// Real-time integrity monitoring system with advanced alerting and metrics
pub struct IntegrityMonitor {
    /// Enable detailed monitoring logging
    pub verbose_logging: bool,
    /// Monitoring interval in seconds
    pub monitoring_interval_seconds: u64,
    /// Enable automatic alerting
    pub alerting_enabled: bool,
    /// Threshold for critical alerts
    pub critical_alert_threshold: usize,
    /// Warning alert threshold
    pub warning_alert_threshold: usize,
    /// Integrity validator instance
    validator: IntegrityValidator,
    /// Performance metrics collector
    metrics_collector: Arc<Mutex<PerformanceMetricsCollector>>,
    /// Alert manager for notifications
    alert_manager: Arc<Mutex<AlertManager>>,
    /// Historical monitoring data
    monitoring_history: Arc<Mutex<MonitoringHistory>>,
    /// Real-time event broadcaster
    event_broadcaster: Arc<broadcast::Sender<MonitoringEvent>>,
}

impl IntegrityMonitor {
    /// Create a new integrity monitor
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            verbose_logging: false,
            monitoring_interval_seconds: 300, // 5 minutes default
            alerting_enabled: true,
            critical_alert_threshold: 5,
            warning_alert_threshold: 3,
            validator: IntegrityValidator::new(),
            metrics_collector: Arc::new(Mutex::new(PerformanceMetricsCollector::new())),
            alert_manager: Arc::new(Mutex::new(AlertManager::new())),
            monitoring_history: Arc::new(Mutex::new(MonitoringHistory::new())),
            event_broadcaster: Arc::new(event_tx),
        }
    }

    /// Create a monitor with custom configuration
    pub fn with_config(
        verbose: bool,
        interval_seconds: u64,
        alerting: bool,
        threshold: usize,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            verbose_logging: verbose,
            monitoring_interval_seconds: interval_seconds,
            alerting_enabled: alerting,
            critical_alert_threshold: threshold,
            warning_alert_threshold: threshold / 2,
            validator: IntegrityValidator::with_config(verbose, 60, true), // 1 minute timeout for monitoring
            metrics_collector: Arc::new(Mutex::new(PerformanceMetricsCollector::new())),
            alert_manager: Arc::new(Mutex::new(AlertManager::new())),
            monitoring_history: Arc::new(Mutex::new(MonitoringHistory::new())),
            event_broadcaster: Arc::new(event_tx),
        }
    }

    /// Start continuous integrity monitoring
    #[instrument(skip(self, blockchain))]
    pub async fn start_monitoring(&self, blockchain: &Blockchain) -> Result<()> {
        if self.verbose_logging {
            info!(
                "Starting integrity monitoring with {} second intervals",
                self.monitoring_interval_seconds
            );
        }

        let mut monitoring_interval =
            interval(Duration::from_secs(self.monitoring_interval_seconds));
        let mut monitoring_stats = MonitoringStatistics::new();

        loop {
            monitoring_interval.tick().await;

            match self
                .perform_monitoring_check(blockchain, &mut monitoring_stats)
                .await
            {
                Ok(report) => {
                    self.process_monitoring_report(&report, &mut monitoring_stats)
                        .await?;
                }
                Err(e) => {
                    error!("Monitoring check failed: {}", e);
                    monitoring_stats.failed_checks += 1;

                    if self.alerting_enabled {
                        self.send_monitoring_failure_alert(&e).await?;
                    }
                }
            }
        }
    }

    /// Perform a single monitoring check with enhanced metrics collection
    #[instrument(skip(self, blockchain, stats))]
    async fn perform_monitoring_check(
        &self,
        blockchain: &Blockchain,
        stats: &mut MonitoringStatistics,
    ) -> Result<IntegrityValidationReport> {
        let start_time = Instant::now();

        if self.verbose_logging {
            debug!(
                "Performing integrity monitoring check #{}",
                stats.total_checks + 1
            );
        }

        // Collect memory usage before validation
        let memory_before = self.get_memory_usage();

        // Run integrity validation
        let report = self.validator.validate_system_integrity(blockchain)?;

        // Calculate performance metrics
        let check_duration = start_time.elapsed();
        let memory_after = self.get_memory_usage();
        let memory_delta = memory_after.saturating_sub(memory_before);

        // Update statistics
        stats.total_checks += 1;
        stats.total_check_time += check_duration;
        stats.last_check_time = Some(Instant::now());

        // Record performance metrics
        if let Ok(mut collector) = self.metrics_collector.lock() {
            collector.record_validation_time(check_duration);
            collector.record_memory_usage(memory_after);

            // Calculate throughput (checks per second)
            let throughput = if stats.total_checks > 0 && stats.total_check_time.as_secs() > 0 {
                stats.total_checks as f64 / stats.total_check_time.as_secs_f64()
            } else {
                0.0
            };
            collector.throughput_history.push_back(throughput);
            if collector.throughput_history.len() > collector.max_history_size {
                collector.throughput_history.pop_front();
            }
        }

        // Store report in history
        if let Ok(mut history) = self.monitoring_history.lock() {
            history.add_report(report.clone());

            // Create performance snapshot
            let snapshot = PerformanceSnapshot {
                timestamp: Utc::now(),
                validation_time: check_duration,
                memory_usage: memory_after,
                cpu_usage: self.get_cpu_usage(),
                throughput: if stats.total_checks > 0 && stats.total_check_time.as_secs() > 0 {
                    stats.total_checks as f64 / stats.total_check_time.as_secs_f64()
                } else {
                    0.0
                },
            };
            history.add_performance_snapshot(snapshot);
        }

        // Broadcast monitoring event
        let event = MonitoringEvent::IntegrityCheckCompleted {
            timestamp: Utc::now(),
            status: report.overall_status.clone(),
            duration: check_duration,
        };
        let _ = self.event_broadcaster.send(event);

        // Check for performance threshold violations
        self.check_performance_thresholds(check_duration, memory_delta)
            .await?;

        if self.verbose_logging {
            debug!(
                "Monitoring check completed in {:?} with status: {:?}, memory delta: {} bytes",
                check_duration, report.overall_status, memory_delta
            );
        }

        Ok(report)
    }

    /// Process monitoring report and handle alerts
    #[instrument(skip(self, report, stats))]
    async fn process_monitoring_report(
        &self,
        report: &IntegrityValidationReport,
        stats: &mut MonitoringStatistics,
    ) -> Result<()> {
        // Update statistics
        match report.overall_status {
            IntegrityStatus::Healthy => stats.healthy_checks += 1,
            IntegrityStatus::Warning => stats.warning_checks += 1,
            IntegrityStatus::Critical => stats.critical_checks += 1,
            IntegrityStatus::Corrupted => stats.corrupted_checks += 1,
        }

        // Handle alerting
        if self.alerting_enabled {
            self.handle_monitoring_alerts(report, stats).await?;
        }

        // Log monitoring results
        if self.verbose_logging {
            info!(
                "Monitoring report: status={:?}, issues={}, recommendations={}",
                report.overall_status,
                report.count_total_issues(),
                report.recommendations.len()
            );
        }

        Ok(())
    }

    /// Handle monitoring alerts based on report
    #[instrument(skip(self, report, stats))]
    async fn handle_monitoring_alerts(
        &self,
        report: &IntegrityValidationReport,
        stats: &MonitoringStatistics,
    ) -> Result<()> {
        match report.overall_status {
            IntegrityStatus::Critical | IntegrityStatus::Corrupted => {
                self.send_critical_alert(report, stats).await?;
            }
            IntegrityStatus::Warning => {
                if report
                    .recommendations
                    .iter()
                    .any(|r| matches!(r.severity, RecommendationSeverity::Critical))
                {
                    self.send_warning_alert(report, stats).await?;
                }
            }
            IntegrityStatus::Healthy => {
                // Check if we're recovering from previous issues
                if stats.critical_checks > 0 || stats.corrupted_checks > 0 {
                    self.send_recovery_alert(report, stats).await?;
                }
            }
        }

        Ok(())
    }

    /// Send critical integrity alert
    #[instrument(skip(self, report, stats))]
    async fn send_critical_alert(
        &self,
        report: &IntegrityValidationReport,
        stats: &MonitoringStatistics,
    ) -> Result<()> {
        if self.verbose_logging {
            warn!("Sending critical integrity alert");
        }

        // TODO: Phase 7 Implementation
        // - Send email/SMS alerts to administrators
        // - Log to external monitoring systems
        // - Trigger automated backup procedures
        // - Escalate to on-call personnel if configured

        let alert = IntegrityAlert {
            alert_type: AlertType::Critical,
            timestamp: chrono::Utc::now(),
            message: format!(
                "Critical integrity issues detected: {} total issues",
                report.count_total_issues()
            ),
            report_summary: report.get_summary(),
            monitoring_stats: stats.clone(),
        };

        debug!("Critical alert prepared: {}", alert.message);
        Ok(())
    }

    /// Send warning integrity alert
    #[instrument(skip(self, report, stats))]
    async fn send_warning_alert(
        &self,
        report: &IntegrityValidationReport,
        stats: &MonitoringStatistics,
    ) -> Result<()> {
        if self.verbose_logging {
            info!("Sending warning integrity alert");
        }

        // TODO: Phase 7 Implementation
        // - Send warning notifications
        // - Log to monitoring dashboard
        // - Schedule follow-up checks

        let alert = IntegrityAlert {
            alert_type: AlertType::Warning,
            timestamp: chrono::Utc::now(),
            message: format!(
                "Integrity warnings detected: {} recommendations",
                report.recommendations.len()
            ),
            report_summary: report.get_summary(),
            monitoring_stats: stats.clone(),
        };

        debug!("Warning alert prepared: {}", alert.message);
        Ok(())
    }

    /// Send recovery alert
    #[instrument(skip(self, report, stats))]
    async fn send_recovery_alert(
        &self,
        report: &IntegrityValidationReport,
        stats: &MonitoringStatistics,
    ) -> Result<()> {
        if self.verbose_logging {
            info!("Sending recovery alert - system integrity restored");
        }

        // TODO: Phase 7 Implementation
        // - Send recovery notifications
        // - Update monitoring dashboard
        // - Log recovery metrics

        let alert = IntegrityAlert {
            alert_type: AlertType::Recovery,
            timestamp: chrono::Utc::now(),
            message: "System integrity has been restored to healthy status".to_string(),
            report_summary: report.get_summary(),
            monitoring_stats: stats.clone(),
        };

        debug!("Recovery alert prepared: {}", alert.message);
        Ok(())
    }

    /// Send monitoring failure alert
    #[instrument(skip(self, error))]
    async fn send_monitoring_failure_alert(
        &self,
        error: &crate::error::ProvChainError,
    ) -> Result<()> {
        if self.verbose_logging {
            error!("Sending monitoring failure alert");
        }

        // TODO: Phase 7 Implementation
        // - Alert about monitoring system failures
        // - Escalate monitoring issues
        // - Trigger fallback monitoring procedures

        debug!("Monitoring failure alert prepared for error: {}", error);
        Ok(())
    }

    /// Get current monitoring statistics
    pub fn get_monitoring_statistics(&self) -> MonitoringStatistics {
        // TODO: Phase 7 Implementation
        // - Return current monitoring statistics
        // - Include performance metrics
        // - Provide trend analysis

        MonitoringStatistics::new()
    }

    /// Perform on-demand integrity check
    #[instrument(skip(self, blockchain))]
    pub async fn perform_on_demand_check(
        &self,
        blockchain: &Blockchain,
    ) -> Result<IntegrityValidationReport> {
        if self.verbose_logging {
            info!("Performing on-demand integrity check");
        }

        let report = self.validator.validate_system_integrity(blockchain)?;

        if self.alerting_enabled && !matches!(report.overall_status, IntegrityStatus::Healthy) {
            let temp_stats = MonitoringStatistics::new();
            self.handle_monitoring_alerts(&report, &temp_stats).await?;
        }

        Ok(report)
    }

    /// Configure monitoring thresholds
    pub fn configure_thresholds(&mut self, critical_threshold: usize, warning_threshold: usize) {
        self.critical_alert_threshold = critical_threshold;
        // TODO: Add warning threshold configuration

        if self.verbose_logging {
            info!(
                "Updated monitoring thresholds: critical={}, warning={}",
                critical_threshold, warning_threshold
            );
        }
    }

    /// Enable or disable alerting
    pub fn set_alerting_enabled(&mut self, enabled: bool) {
        self.alerting_enabled = enabled;

        if self.verbose_logging {
            info!("Alerting {}", if enabled { "enabled" } else { "disabled" });
        }
    }

    /// Get current memory usage in bytes
    fn get_memory_usage(&self) -> usize {
        // Simple memory usage estimation
        // In a production system, this would use system APIs
        std::mem::size_of::<Self>() + 1024 * 1024 // Base estimate + 1MB
    }

    /// Get current CPU usage percentage
    fn get_cpu_usage(&self) -> f64 {
        // Simple CPU usage estimation
        // In a production system, this would use system APIs
        0.0 // Placeholder
    }

    /// Check for performance threshold violations
    async fn check_performance_thresholds(
        &self,
        duration: Duration,
        memory_delta: usize,
    ) -> Result<()> {
        // Performance thresholds
        let max_validation_time = Duration::from_secs(30); // 30 seconds max
        let max_memory_delta = 100 * 1024 * 1024; // 100MB max delta

        if duration > max_validation_time {
            let event = MonitoringEvent::PerformanceThresholdExceeded {
                timestamp: Utc::now(),
                metric: "validation_time".to_string(),
                value: duration.as_secs_f64(),
                threshold: max_validation_time.as_secs_f64(),
            };
            let _ = self.event_broadcaster.send(event);

            if self.verbose_logging {
                warn!(
                    "Validation time threshold exceeded: {:?} > {:?}",
                    duration, max_validation_time
                );
            }
        }

        if memory_delta > max_memory_delta {
            let event = MonitoringEvent::PerformanceThresholdExceeded {
                timestamp: Utc::now(),
                metric: "memory_delta".to_string(),
                value: memory_delta as f64,
                threshold: max_memory_delta as f64,
            };
            let _ = self.event_broadcaster.send(event);

            if self.verbose_logging {
                warn!(
                    "Memory delta threshold exceeded: {} bytes > {} bytes",
                    memory_delta, max_memory_delta
                );
            }
        }

        Ok(())
    }

    /// Get event broadcaster receiver for real-time monitoring
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<MonitoringEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Get comprehensive monitoring dashboard data
    pub async fn get_dashboard_data(&self) -> Result<MonitoringDashboardData> {
        let mut dashboard_data = MonitoringDashboardData::new();

        // Get current statistics
        dashboard_data.current_stats = self.get_monitoring_statistics();

        // Get performance metrics
        if let Ok(collector) = self.metrics_collector.lock() {
            dashboard_data.average_validation_time = collector.get_average_validation_time();
            dashboard_data.performance_trend = collector.get_performance_trend();
            dashboard_data.recent_validation_times = collector
                .validation_times
                .iter()
                .rev()
                .take(10)
                .cloned()
                .collect();
            dashboard_data.recent_throughput = collector
                .throughput_history
                .iter()
                .rev()
                .take(10)
                .cloned()
                .collect();
        }

        // Get historical data
        if let Ok(history) = self.monitoring_history.lock() {
            dashboard_data.health_trend = history.get_health_trend(20);
            dashboard_data.recent_reports = history.reports.iter().rev().take(5).cloned().collect();
            dashboard_data.recent_alerts = history.alerts.iter().rev().take(10).cloned().collect();
        }

        // Get alert statistics
        if let Ok(alert_manager) = self.alert_manager.lock() {
            dashboard_data.total_alerts = alert_manager.alert_history.len();
            dashboard_data.recent_alert_types = alert_manager
                .alert_history
                .iter()
                .rev()
                .take(10)
                .map(|a| a.alert_type.clone())
                .collect();
        }

        Ok(dashboard_data)
    }

    /// Configure alert channels
    pub async fn configure_alerts(
        &self,
        email_config: Option<EmailConfig>,
        webhook_config: Option<WebhookConfig>,
        slack_config: Option<SlackConfig>,
    ) -> Result<()> {
        if let Ok(mut alert_manager) = self.alert_manager.lock() {
            if let Some(config) = email_config {
                alert_manager.configure_email(config);
            }
            if let Some(config) = webhook_config {
                alert_manager.configure_webhook(config);
            }
            if let Some(config) = slack_config {
                alert_manager.configure_slack(config);
            }
        }

        if self.verbose_logging {
            info!("Alert channels configured successfully");
        }

        Ok(())
    }

    /// Generate monitoring report for external systems
    pub async fn generate_monitoring_report(&self) -> Result<MonitoringReport> {
        let dashboard_data = self.get_dashboard_data().await?;

        let report = MonitoringReport {
            timestamp: Utc::now(),
            system_health: dashboard_data.health_trend,
            performance_summary: PerformanceSummary {
                average_validation_time: dashboard_data.average_validation_time,
                performance_trend: dashboard_data.performance_trend,
                health_percentage: dashboard_data.current_stats.health_percentage(),
                total_checks: dashboard_data.current_stats.total_checks,
            },
            alert_summary: AlertSummary {
                total_alerts: dashboard_data.total_alerts,
                critical_alerts: dashboard_data
                    .recent_alert_types
                    .iter()
                    .filter(|&t| matches!(t, AlertType::Critical))
                    .count(),
                warning_alerts: dashboard_data
                    .recent_alert_types
                    .iter()
                    .filter(|&t| matches!(t, AlertType::Warning))
                    .count(),
                recovery_alerts: dashboard_data
                    .recent_alert_types
                    .iter()
                    .filter(|&t| matches!(t, AlertType::Recovery))
                    .count(),
            },
            recommendations: self.generate_recommendations(&dashboard_data).await,
        };

        Ok(report)
    }

    /// Generate recommendations based on monitoring data
    async fn generate_recommendations(
        &self,
        dashboard_data: &MonitoringDashboardData,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if matches!(
            dashboard_data.performance_trend,
            PerformanceTrend::Degrading
        ) {
            recommendations.push("Performance is degrading. Consider optimizing validation algorithms or increasing system resources.".to_string());
        }

        // Health recommendations
        match dashboard_data.health_trend {
            HealthTrend::Critical => {
                recommendations
                    .push("System health is critical. Immediate attention required.".to_string());
            }
            HealthTrend::Concerning => {
                recommendations.push("System health is concerning. Monitor closely and consider preventive measures.".to_string());
            }
            _ => {}
        }

        // Alert frequency recommendations
        if dashboard_data.total_alerts > 50 {
            recommendations.push(
                "High alert frequency detected. Review alert thresholds and system stability."
                    .to_string(),
            );
        }

        // Validation time recommendations
        if dashboard_data.average_validation_time > Duration::from_secs(10) {
            recommendations.push("Validation times are high. Consider performance optimization or incremental validation.".to_string());
        }

        recommendations
    }
}

/// Monitoring dashboard data structure
#[derive(Debug, Clone)]
pub struct MonitoringDashboardData {
    pub current_stats: MonitoringStatistics,
    pub average_validation_time: Duration,
    pub performance_trend: PerformanceTrend,
    pub health_trend: HealthTrend,
    pub recent_validation_times: Vec<Duration>,
    pub recent_throughput: Vec<f64>,
    pub recent_reports: Vec<IntegrityValidationReport>,
    pub recent_alerts: Vec<IntegrityAlert>,
    pub total_alerts: usize,
    pub recent_alert_types: Vec<AlertType>,
}

impl Default for MonitoringDashboardData {
    fn default() -> Self {
        Self {
            current_stats: MonitoringStatistics::new(),
            average_validation_time: Duration::from_secs(0),
            performance_trend: PerformanceTrend::Stable,
            health_trend: HealthTrend::Insufficient,
            recent_validation_times: Vec::new(),
            recent_throughput: Vec::new(),
            recent_reports: Vec::new(),
            recent_alerts: Vec::new(),
            total_alerts: 0,
            recent_alert_types: Vec::new(),
        }
    }
}

impl MonitoringDashboardData {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Comprehensive monitoring report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub timestamp: DateTime<Utc>,
    pub system_health: HealthTrend,
    pub performance_summary: PerformanceSummary,
    pub alert_summary: AlertSummary,
    pub recommendations: Vec<String>,
}

/// Performance summary for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub average_validation_time: Duration,
    pub performance_trend: PerformanceTrend,
    pub health_percentage: f64,
    pub total_checks: usize,
}

/// Alert summary for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub recovery_alerts: usize,
}

impl Default for IntegrityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MonitoringStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_integrity_monitor_creation() {
        let monitor = IntegrityMonitor::new();
        assert!(!monitor.verbose_logging);
        assert_eq!(monitor.monitoring_interval_seconds, 300);
        assert!(monitor.alerting_enabled);
        assert_eq!(monitor.critical_alert_threshold, 5);
        assert_eq!(monitor.warning_alert_threshold, 3);
    }

    #[test]
    fn test_integrity_monitor_with_config() {
        let monitor = IntegrityMonitor::with_config(true, 60, false, 10);
        assert!(monitor.verbose_logging);
        assert_eq!(monitor.monitoring_interval_seconds, 60);
        assert!(!monitor.alerting_enabled);
        assert_eq!(monitor.critical_alert_threshold, 10);
        assert_eq!(monitor.warning_alert_threshold, 5);
    }

    #[test]
    fn test_monitoring_statistics_creation() {
        let stats = MonitoringStatistics::new();
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.healthy_checks, 0);
        assert_eq!(stats.health_percentage(), 0.0);
        assert!(!stats.has_recent_issues());
    }

    #[test]
    fn test_monitoring_statistics_calculations() {
        let mut stats = MonitoringStatistics::new();
        stats.total_checks = 10;
        stats.healthy_checks = 8;
        stats.warning_checks = 2;
        stats.total_check_time = Duration::from_secs(100);

        assert_eq!(stats.health_percentage(), 80.0);
        assert_eq!(stats.average_check_time(), Duration::from_secs(10));
        assert!(!stats.has_recent_issues());
    }

    #[test]
    fn test_performance_metrics_collector() {
        let mut collector = PerformanceMetricsCollector::new();
        collector.record_validation_time(Duration::from_millis(100));
        collector.record_validation_time(Duration::from_millis(150));

        assert_eq!(collector.validation_times.len(), 2);
        assert_eq!(
            collector.get_average_validation_time(),
            Duration::from_millis(125)
        );
    }

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::new();
        assert!(manager.email_config.is_none());
        assert!(manager.webhook_config.is_none());
        assert!(manager.slack_config.is_none());
        assert_eq!(manager.alert_history.len(), 0);
    }

    #[test]
    fn test_monitoring_history() {
        let mut history = MonitoringHistory::new();
        let report = IntegrityValidationReport::new();

        history.add_report(report);
        assert_eq!(history.reports.len(), 1);

        let trend = history.get_health_trend(5);
        assert_eq!(trend, HealthTrend::Insufficient);
    }

    #[tokio::test]
    async fn test_perform_on_demand_check() {
        let monitor = IntegrityMonitor::new();
        let blockchain = Blockchain::new();

        let result = monitor.perform_on_demand_check(&blockchain).await;
        assert!(result.is_ok());

        let report = result.unwrap();

        // For a new blockchain with loaded ontology but no transactions,
        // the system should be Healthy (ontology triples don't count as transactions)
        assert_eq!(report.overall_status, IntegrityStatus::Healthy);

        // Verify that the blockchain has genesis block and has ontology data
        assert_eq!(
            report.blockchain_integrity.chain_length, 1,
            "Should have genesis block"
        );
        assert!(
            report.transaction_count_integrity.actual_rdf_triple_count > 0,
            "Ontology should be loaded"
        );
        assert_eq!(
            report
                .transaction_count_integrity
                .reported_total_transactions,
            1,
            "Should have genesis block transaction"
        );
    }
}
