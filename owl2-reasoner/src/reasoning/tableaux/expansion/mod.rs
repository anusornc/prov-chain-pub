//! Tableaux Rule Expansion
//!
//! Implements the core rule expansion logic for the tableaux reasoning algorithm.
//! This module manages the application of tableaux rules to expand the model
//! and derive new consequences from the ontology.

pub mod axiom_rules;
pub mod class_rules;
pub mod constraint_rules;
pub mod context;
pub mod engine;
pub mod property_rules;
pub mod types;

// Re-export public types for backward compatibility
pub use axiom_rules::*;
pub use class_rules::*;
pub use constraint_rules::*;
pub use context::*;
pub use engine::*;
pub use property_rules::*;
pub use types::*;
