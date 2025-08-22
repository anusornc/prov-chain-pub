//! Ontology management for universal traceability platform
//! 
//! This module provides ontology loading, management, and processing capabilities
//! for multiple domains using OWL ontologies.

pub mod manager;
pub mod processor;
pub mod registry;

#[cfg(test)]
mod test;

// Re-exports for convenience
pub use manager::OntologyManager;
pub use processor::OntologyProcessor;
pub use registry::OntologyRegistry;