//! OWL2 Profile Validation Module
//!
//! This module implements validation for the three OWL2 profiles:
//! - EL: Expressive Logic (EL++) profile
//! - QL: Query Language (OWL2 QL) profile
//! - RL: Rule Language (OWL2 RL) profile
//!
//! Each profile has specific restrictions on which OWL2 constructs are allowed.
//! This module provides efficient validation algorithms inspired by owl2_rs.

pub mod cache;
pub mod common;
pub mod el;
pub mod ql;
pub mod rl;

// Re-export commonly used types for backward compatibility
pub use common::*;

// Main exports for the profiles module
pub use crate::profiles::common::{
    OntologyStats, OptimizationHint, OptimizationType, Owl2Profile, Owl2ProfileValidator,
    ProfileAnalysisReport, ProfileValidationResult, ProfileValidator, ProfileViolation,
    ProfileViolationType, ValidationStatistics, ViolationSeverity,
};

// Re-export cache types
pub use crate::profiles::cache::{CachePriority, CacheStatistics, ProfileCacheConfig};
