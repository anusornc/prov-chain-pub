//! Benchmark suite for the OWL2 Reasoner
//!
//! This module contains comprehensive benchmarks for all major components
//! of the OWL2 reasoning system using the criterion benchmarking framework.

mod parser_bench;
mod query_bench;
mod reasoning_bench;
mod scalability_bench;
mod tableaux_benchmarks;

pub use parser_bench::*;
pub use query_bench::*;
pub use reasoning_bench::*;
pub use scalability_bench::*;
pub use tableaux_benchmarks::*;
