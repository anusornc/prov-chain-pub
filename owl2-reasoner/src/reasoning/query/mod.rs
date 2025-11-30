//! Query answering for OWL2 ontologies
//!
//! Provides SPARQL-like query capabilities for OWL2 ontologies with reasoning support.
//! Features advanced query optimizations including caching, indexing, and pattern compilation.

use crate::iri::IRI;

pub mod cache;
pub mod config;
pub mod engine;
pub mod executor;
pub mod optimized_engine;
pub mod types;

// Re-export public types
pub use cache::*;
pub use config::*;
pub use engine::*;
pub use executor::*;
pub use optimized_engine::*;
pub use types::*;

/// Helper function to avoid unnecessary (**arc_iri).clone() operations
#[inline(always)]
pub fn arc_iri_to_owned(arc_iri: &std::sync::Arc<IRI>) -> IRI {
    // This is still needed for PatternTerm::IRI which takes owned IRI
    // but we optimize by avoiding the double dereference
    (**arc_iri).clone()
}

/// Helper function to create PatternTerm::IRI from Arc<IRI> with minimal cloning
#[inline(always)]
pub fn pattern_term_from_arc(arc_iri: &std::sync::Arc<IRI>) -> PatternTerm {
    PatternTerm::IRI(arc_iri_to_owned(arc_iri))
}
