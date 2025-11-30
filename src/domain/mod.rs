//! Domain management for universal traceability platform
//!
//! This module provides domain management capabilities for the
//! universal traceability platform, including plugin interfaces,
//! domain managers, and domain-specific adapters.

pub mod adapters;
pub mod manager;
pub mod plugin;

// Re-exports for convenience
pub use manager::DomainManager;
pub use plugin::{DomainConfig, DomainPlugin, EntityData, ProcessedEntity, ValidationResult};
