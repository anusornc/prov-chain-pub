//! Ontology management module
//!
//! This module provides flexible ontology loading and management
//! capabilities that decouple ontology loading from the core
//! blockchain implementation.

pub mod config;
pub mod manager;
pub mod loader;

// Re-export commonly used items
pub use config::{OntologyConfig, DomainOntologyConfig, OntologyResolutionStrategy};
pub use manager::OntologyManager;
pub use loader::{load_ontology_config, load_ontology_config_from_toml, load_ontology_config_from_env};