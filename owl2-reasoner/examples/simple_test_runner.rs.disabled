//! Simple OWL2 Test Suite Runner
//!
//! This example demonstrates basic test suite functionality.

use owl2_reasoner::test_suite_simple::{TestSuiteConfig, TestSuiteRunner};
use owl2_reasoner::OwlResult;
use std::path::PathBuf;

fn main() -> OwlResult<()> {
    println!("🧪 Simple OWL2 Test Suite Runner");
    println!("=================================");

    // Create test suite configuration
    let config = TestSuiteConfig {
        test_files: vec![
            PathBuf::from("test_suite/simple_test.ofn"),
            PathBuf::from("test_suite/family_test.ttl"),
            PathBuf::from("test_suite/property_test.rdf"),
            PathBuf::from("test_suite/complex_expressions.ttl"),
            PathBuf::from("test_suite/biomedical_test.ttl"),
            PathBuf::from("test_suite/classification_test.rdf"),
        ],
        timeout_seconds: 30,
    };

    println!("Test files:");
    for test_file in &config.test_files {
        println!("  - {}", test_file.display());
    }
    println!();

    // Create and run test suite
    let runner = TestSuiteRunner::new(config);

    println!("Running test suite...");
    match runner.run_tests() {
        Ok(result) => {
            println!("✅ Test suite completed successfully!");
            println!("{}", runner.get_summary(&result));

            // Display individual test results
            println!("\n📋 Individual Test Results:");
            for detail in &result.details {
                let status = if detail.passed {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                };
                println!(
                    "  {} {} ({:?})",
                    status, detail.test_name, detail.execution_time
                );
                if let Some(error) = &detail.error_message {
                    println!("    Error: {}", error);
                }
            }

            // Overall assessment
            println!("\n🎯 Compliance Assessment:");
            let pass_rate = if result.total_tests > 0 {
                (result.passed_tests as f64 / result.total_tests as f64) * 100.0
            } else {
                0.0
            };

            if pass_rate >= 80.0 {
                println!(
                    "✅ GOOD: {:.1}% pass rate - Test suite is working well!",
                    pass_rate
                );
            } else if pass_rate >= 50.0 {
                println!(
                    "⚠️  FAIR: {:.1}% pass rate - Some issues detected",
                    pass_rate
                );
            } else {
                println!(
                    "❌ POOR: {:.1}% pass rate - Significant issues found",
                    pass_rate
                );
            }
        }
        Err(e) => {
            println!("❌ Test suite execution failed: {}", e);
        }
    }

    Ok(())
}
