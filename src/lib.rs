pub mod error;
pub mod validation;
pub mod core;
pub mod transaction;
pub mod storage;
pub mod semantic;
pub mod utils;

pub mod trace_optimization;
pub mod governance;
pub mod demo;
pub mod demo_runner;
pub mod uht_demo;
pub mod wallet;
pub mod web;
pub mod analytics;
pub mod knowledge_graph;
pub mod network;
pub mod performance;
pub mod production;
pub mod universal_demo;

pub mod domain;
pub mod ontology;

// Re-export common error types
pub use error::{ProvChainError, Result};
