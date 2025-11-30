//! Production deployment module for enterprise-grade features
//!
//! This module provides:
//! - Container orchestration support
//! - Monitoring and observability
//! - Security hardening
//! - Compliance framework

pub mod compliance;
pub mod container;
pub mod deployment;
pub mod monitoring;
pub mod security;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProductionError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    #[error("Security error: {0}")]
    Security(String),
    #[error("Compliance error: {0}")]
    Compliance(String),
    #[error("Deployment error: {0}")]
    Deployment(String),
}

/// Production configuration for the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Environment (development, staging, production)
    pub environment: String,
    /// Application version
    pub version: String,
    /// Monitoring configuration
    pub monitoring: monitoring::MonitoringConfig,
    /// Security configuration
    pub security: security::SecurityConfig,
    /// Compliance configuration
    pub compliance: compliance::ComplianceConfig,
    /// Container configuration
    pub container: container::ContainerConfig,
    /// Deployment configuration
    pub deployment: deployment::DeploymentConfig,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            monitoring: monitoring::MonitoringConfig::default(),
            security: security::SecurityConfig::default(),
            compliance: compliance::ComplianceConfig::default(),
            container: container::ContainerConfig::default(),
            deployment: deployment::DeploymentConfig::default(),
        }
    }
}

/// Production manager for coordinating all production features
pub struct ProductionManager {
    config: ProductionConfig,
    monitoring: monitoring::MonitoringManager,
    security: security::SecurityManager,
    compliance: compliance::ComplianceManager,
}

impl ProductionManager {
    /// Create a new production manager
    pub fn new(config: ProductionConfig) -> Result<Self, ProductionError> {
        let monitoring = monitoring::MonitoringManager::new(config.monitoring.clone())?;
        let security = security::SecurityManager::new(config.security.clone())?;
        let compliance = compliance::ComplianceManager::new(config.compliance.clone())?;

        Ok(Self {
            config,
            monitoring,
            security,
            compliance,
        })
    }

    /// Initialize production environment
    pub async fn initialize(&mut self) -> Result<(), ProductionError> {
        tracing::info!(
            "Initializing production environment: {}",
            self.config.environment
        );

        // Initialize monitoring
        self.monitoring.start().await?;
        tracing::info!("Monitoring system initialized");

        // Initialize security
        self.security.initialize().await?;
        tracing::info!("Security system initialized");

        // Initialize compliance
        self.compliance.initialize().await?;
        tracing::info!("Compliance system initialized");

        tracing::info!("Production environment initialized successfully");
        Ok(())
    }

    /// Get system health status
    pub async fn health_check(&self) -> HashMap<String, String> {
        let mut health = HashMap::new();

        health.insert("environment".to_string(), self.config.environment.clone());
        health.insert("version".to_string(), self.config.version.clone());
        health.insert("monitoring".to_string(), self.monitoring.status().await);
        health.insert("security".to_string(), self.security.status().await);
        health.insert("compliance".to_string(), self.compliance.status().await);

        health
    }

    /// Shutdown production environment
    pub async fn shutdown(&mut self) -> Result<(), ProductionError> {
        tracing::info!("Shutting down production environment");

        self.monitoring.stop().await?;
        self.security.shutdown().await?;
        self.compliance.shutdown().await?;

        tracing::info!("Production environment shutdown complete");
        Ok(())
    }
}
