pub mod config;
pub mod core;
pub mod error;
pub mod integrity;
pub mod semantic;
pub mod storage;
pub mod transaction;
pub mod utils;
pub mod validation;

pub mod analytics;
pub mod demo;
pub mod demo_runner;
pub mod governance;
pub mod knowledge_graph;
pub mod network;
pub mod performance;
pub mod production;
pub mod trace_optimization;
pub mod uht_demo;
pub mod universal_demo;
pub mod wallet;
pub mod web;

pub mod domain;
pub mod interop;
pub mod ontology;
pub mod security;

// Re-export common error types
pub use error::{ProvChainError, Result};
