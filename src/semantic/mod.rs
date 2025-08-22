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
//! - `simple_owl2_test`: Simple test of owl2_rs integration
//! - `shacl_validator`: SHACL validation for data conformance
//!
//! ## Implementation Status
//! This module implements the enhanced OWL2 features as planned in
//! REVISED_IMPLEMENTATION_PLAN.md and addresses the issues identified
//! in our debugging session.

pub mod owl_reasoner;
pub mod owl2_enhanced_reasoner;
pub mod owl2_integration;
pub mod owl2_traceability;
pub mod simple_owl2_test;
pub mod shacl_validator;

// Re-exports for convenience
pub use owl_reasoner::{OwlReasoner, OwlReasonerConfig, ValidationResult};
pub use owl2_enhanced_reasoner::{Owl2EnhancedReasoner, QualifiedCardinalityRestriction, InferredGraph};
pub use owl2_integration::test_owl2_integration;
pub use owl2_traceability::Owl2EnhancedTraceability;
pub use simple_owl2_test::simple_owl2_integration_test;
pub use shacl_validator::ShaclValidator;
