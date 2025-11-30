//! Core blockchain functionality
//!
//! This module contains the core blockchain implementation including
//! block structure, state management, and atomic operations.

pub mod atomic_operations;
pub mod blockchain;
pub mod entity;

// Re-exports for convenience
pub use atomic_operations::AtomicOperationContext;
pub use blockchain::Blockchain;
pub use entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
