//! Performance utilities and optimizations
//!
//! This module provides various utility functions and optimizations
//! for improving performance across the OWL2 reasoner.

pub mod iri;
pub mod smallvec;

// Re-export commonly used utilities for convenience
pub use iri::*;
pub use smallvec::*;

/// Pre-allocate collections with sensible defaults
pub fn preallocate_vec<T>(size_hint: usize) -> Vec<T> {
    Vec::with_capacity(size_hint.max(8))
}

/// Optimized string interning for frequently used strings
pub struct StringInterner {
    map: std::collections::HashMap<String, std::sync::Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> std::sync::Arc<str> {
        self.map
            .entry(s.to_string())
            .or_insert_with(|| std::sync::Arc::from(s))
            .clone()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
