//! Semantic web functionality for the ProvChainOrg platform
//!
//! This module provides semantic web implementations including
//! ontology management, SHACL validation, and SPARQL processing.
//!
//! Production note:
//! The production semantic path is implemented in `crate::ontology` and backed by
//! the SPACL `owl2-reasoner` dependency. Modules in `crate::semantic` are retained
//! primarily for demos, experimentation, and migration support.
//!
//! ## Key Components
//! - `owl_reasoner`: Legacy OWL reasoner retained for experimentation
//! - `owl2_enhanced_reasoner`: Experimental OWL2 feature exploration
//! - `owl2_integration`: Basic integration with owl2-reasoner library
//! - `owl2_traceability`: Legacy traceability experiment using owl2-reasoner
//! - `enhanced_owl2_demo`: Demo of enhanced OWL2 features with hasKey support
//! - `simple_owl2_test`: Simple test of owl2-reasoner integration
//! - `shacl_validator`: Basic SHACL validation for experimental workflows
//!
//! ## Implementation Status
//! This module implements the enhanced OWL2 features as planned in
//! REVISED_IMPLEMENTATION_PLAN.md and addresses the issues identified
//! in our debugging session.

#[cfg(test)]
pub mod debug_ontology;
pub mod enhanced_owl2_demo;
pub mod gs1_epcis;
pub mod library_integration;
pub mod owl2_enhanced_reasoner;
pub mod owl2_integration;
pub mod owl2_traceability;
pub mod owl_reasoner;
pub mod shacl_validator;
pub mod simple_owl2_test;

// Re-exports for convenience
pub use enhanced_owl2_demo::run_enhanced_owl2_demo;
pub use gs1_epcis::{
    biz_steps, create_epcis_document, dispositions, generate_uht_supply_chain_events, namespaces,
    EpcisEventBuilder, EpcisEventType,
};
pub use owl2_enhanced_reasoner::{
    InferredGraph, Owl2EnhancedReasoner, QualifiedCardinalityRestriction,
};
pub use owl2_integration::test_owl2_integration;
pub use owl2_traceability::Owl2EnhancedTraceability;
pub use owl_reasoner::{OwlReasoner, OwlReasonerConfig, ValidationResult};
pub use shacl_validator::ShaclValidator;
pub use simple_owl2_test::simple_owl2_integration_test;
