//! Storage and persistence functionality
//!
//! This module contains storage implementations, persistence, backup, and caching.

pub mod rdf_store;
pub mod rdf_store_safe;

// Re-exports for convenience
pub use rdf_store::RDFStore;
pub use rdf_store_safe::SafeRDFOperations;
