//! Semantic web functionality for the ProvChainOrg platform
//!
//! This module provides semantic web implementations including
//! ontology management, SHACL validation, and SPARQL processing.
//!
//! ## Key Components
//! - `owl_reasoner`: Basic OWL reasoner with foundational OWL features
//! - `owl2_enhanced_reasoner`: Enhanced OWL reasoner with full OWL2 feature support
//! - `owl2_integration`: Basic integration with owl2_rs library
//! - `owl2_traceability`: Enhanced traceability using owl2_rs for OWL2 reasoning
//! - `enhanced_owl2_demo`: Demo of enhanced OWL2 features with hasKey support
//! - `simple_owl2_test`: Simple test of owl2_rs integration
//! - `shacl_validator`: SHACL validation for data conformance
//!
//! ## Implementation Status
//! This module implements the enhanced OWL2 features as planned in
//! REVISED_IMPLEMENTATION_PLAN.md and addresses the issues identified
//! in our debugging session.

#[cfg(test)]
pub mod debug_ontology;
pub mod enhanced_owl2_demo;
pub mod owl2_enhanced_reasoner;
pub mod owl2_integration;
pub mod owl2_traceability;
pub mod owl_reasoner;
pub mod shacl_validator;
pub mod simple_owl2_test;

// Re-exports for convenience
pub use owl2_enhanced_reasoner::{
    InferredGraph, Owl2EnhancedReasoner, QualifiedCardinalityRestriction,
};
pub use owl2_integration::test_owl2_integration;
pub use owl2_traceability::Owl2EnhancedTraceability;
pub use owl_reasoner::{OwlReasoner, OwlReasonerConfig, ValidationResult};
// pub use enhanced_owl2_demo::run_enhanced_owl2_demo;
pub use shacl_validator::ShaclValidator;
pub use simple_owl2_test::simple_owl2_integration_test;
