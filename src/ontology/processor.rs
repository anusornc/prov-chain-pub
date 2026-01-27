//! # DEPRECATED: Ontology processor (Not Used)
//!
//! **This module is deprecated and not used in the research codebase.**
//!
//! For actual ontology processing, the codebase uses:
//! - `src/semantic/owl_reasoner.rs` - OWL reasoning implementation
//! - `src/semantic/shacl_validator.rs` - SHACL validation
//! - `src/semantic/owl2_enhanced_reasoner.rs` - Enhanced OWL2 reasoning
//! - `src/domain/` modules - Domain-specific validation adapters
//!
//! This file is retained for historical reference only and should not be used
//! in research or production code.

use crate::storage::rdf_store::RDFStore;
use oxigraph::model::NamedNode;
use anyhow::Result;

/// # DEPRECATED: Ontology Processor
///
/// **DO NOT USE** - This struct is deprecated and not maintained.
///
/// Use the actual implementations:
/// - `OwlReasoner` in `src/semantic/owl_reasoner.rs`
/// - `ShaclValidator` in `src/semantic/shacl_validator.rs`
/// - `Owl2EnhancedReasoner` in `src/semantic/owl2_enhanced_reasoner.rs`
#[deprecated(note = "Use OwlReasoner, ShaclValidator, or Owl2EnhancedReasoner instead")]
pub struct OntologyProcessor {
    pub rdf_store: RDFStore,
}

#[allow(deprecated)]
impl OntologyProcessor {
    /// Create a new ontology processor
    ///
    /// **DEPRECATED**: Use the actual reasoner implementations instead.
    #[deprecated(note = "Use OwlReasoner::new(), ShaclValidator::new(), or Owl2EnhancedReasoner::new() instead")]
    pub fn new() -> Self {
        OntologyProcessor {
            rdf_store: RDFStore::new(),
        }
    }

    /// Load an ontology into the processor
    ///
    /// **DEPRECATED**: Use RDFStore::load_ontology() directly or the reasoner implementations.
    #[deprecated(note = "Use RDFStore::load_ontology() directly")]
    pub fn load_ontology(&mut self, ontology_data: &str, graph_name: &str) -> Result<()> {
        let graph_node = NamedNode::new(graph_name)?;
        self.rdf_store.load_ontology(ontology_data, &graph_node);
        Ok(())
    }

    /// Validate entity data against an ontology
    ///
    /// **DEPRECATED**: This is a stub implementation.
    /// Use `ShaclValidator::validate_graph()` for actual validation.
    #[deprecated(note = "Use ShaclValidator::validate_graph() for SHACL validation")]
    pub fn validate_entity(&self, _entity_data: &str, ontology_graph: &str) -> Result<bool> {
        tracing::warn!(
            "Deprecated OntologyProcessor::validate_entity() called with ontology: {}",
            ontology_graph
        );
        tracing::warn!("Use ShaclValidator::validate_graph() for actual validation");
        Ok(true)
    }

    /// Perform reasoning to infer new relationships
    ///
    /// **DEPRECATED**: This is a stub implementation.
    /// Use `OwlReasoner::infer_relationships()` for actual reasoning.
    #[deprecated(note = "Use OwlReasoner::infer_relationships() for OWL reasoning")]
    pub fn infer_relationships(&self, entity_data: &str, ontology_graph: &str) -> Result<String> {
        tracing::warn!(
            "Deprecated OntologyProcessor::infer_relationships() called with ontology: {}",
            ontology_graph
        );
        tracing::warn!("Use OwlReasoner::infer_relationships() for actual reasoning");
        Ok(entity_data.to_string())
    }
}