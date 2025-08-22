//! Domain management for universal traceability platform
//!
//! This module provides domain management capabilities for the
//! universal traceability platform, including plugin interfaces,
//! domain managers, and domain-specific adapters.

pub mod plugin;
pub mod manager;
pub mod adapters;

// Re-exports for convenience
pub use plugin::{DomainPlugin, DomainConfig, ValidationResult, ProcessedEntity, EntityData};
pub use manager::DomainManager;