//! Transaction processing functionality
//!
//! This module contains transaction processing, validation, and blockchain integration.

#[allow(clippy::module_inception)]
pub mod blockchain;
#[allow(clippy::module_inception)]
pub mod transaction;

// Re-exports for convenience
pub use blockchain::TransactionBlockchain;
pub use transaction::Transaction;
