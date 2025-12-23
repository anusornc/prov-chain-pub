//! OWL2 Validation Framework
//!
//! This module provides validation infrastructure for the OWL2 reasoner.

pub mod academic_validation;
// pub mod benchmark_suite;
pub mod competition_framework;
pub mod compliance_reporter;
pub mod enterprise_validation;
pub mod execution_engine;
pub mod memory_profiler;
pub mod oaei_integration;
pub mod performance_profiler;
pub mod realtime_monitor;
pub mod w3c_test_suite;

use crate::OwlResult;
use log::info;

/// Main validation framework coordinator
pub struct ValidationFramework {
    test_suite: w3c_test_suite::W3CTestSuite,
}

impl ValidationFramework {
    /// Create a new validation framework instance
    pub fn new() -> OwlResult<Self> {
        Ok(Self {
            test_suite: w3c_test_suite::W3CTestSuite::new()?,
        })
    }

    /// Run basic validation
    pub fn run_basic_validation(&mut self) -> OwlResult<ValidationReport> {
        info!("Running basic validation...");

        let mut report = ValidationReport::new();

        // W3C Compliance Validation
        let w3c_results = self.test_suite.run_basic_tests()?;
        report.w3c_compliance_score = w3c_results.overall_score;

        Ok(report)
    }
}

/// Basic validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub w3c_compliance_score: f64,
    pub validation_passed: bool,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            w3c_compliance_score: 0.0,
            validation_passed: false,
        }
    }
}

/// Re-export commonly used validation types
pub use w3c_test_suite::ComplianceReport;

#[cfg(test)]
mod validation_test;
