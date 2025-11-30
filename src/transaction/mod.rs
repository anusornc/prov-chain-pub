//! Transaction processing functionality
//!
//! This module contains transaction processing, validation, and blockchain integration.

pub mod blockchain;
pub mod transaction;

// Re-exports for convenience
pub use blockchain::TransactionBlockchain;
pub use transaction::Transaction;
