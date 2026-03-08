//! Domain adapters for specific domains.
//!
//! These adapters extend the generic traceability system with
//! lightweight domain-specific validation and enrichment rules.

pub mod healthcare;
pub mod owl_adapter;
pub mod pharmaceutical;
pub mod supply_chain;

pub use healthcare::HealthcareAdapter;
pub use pharmaceutical::PharmaceuticalAdapter;
pub use supply_chain::SupplyChainAdapter;
