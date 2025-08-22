//! Semantic web functionality for the ProvChainOrg platform
//!
//! This module provides semantic web implementations including
//! ontology management, SHACL validation, and SPARQL processing.
//!
//! ## Key Components
//! - `owl_reasoner`: Basic OWL reasoner with foundational OWL features
//! - `owl2_enhanced_reasoner`: Enhanced OWL reasoner with full OWL2 feature support
//! - `owl2_integration`: Integration with owl2_rs library for advanced OWL2 features
//! - `owl2_demo`: Demonstration of owl2_rs integration
//! - `shacl_validator`: SHACL validation for data conformance
//!
//! ## Implementation Status
//! This module implements the enhanced OWL2 features as planned in
//! REVISED_IMPLEMENTATION_PLAN.md and addresses the issues identified
//! in our debugging session.

pub mod owl_reasoner;
pub mod owl2_enhanced_reasoner;
pub mod owl2_integration;
pub mod owl2_demo;
pub mod shacl_validator;

// Re-exports for convenience
pub use owl_reasoner::{OwlReasoner, OwlReasonerConfig, ValidationResult};
pub use owl2_enhanced_reasoner::{Owl2EnhancedReasoner, QualifiedCardinalityRestriction, InferredGraph};
pub use owl2_integration::test_owl2_integration;
pub use shacl_validator::ShaclValidator;
