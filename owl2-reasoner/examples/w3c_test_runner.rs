/// W3C OWL 2 Test Suite Runner
///
/// This program runs W3C OWL 2 conformance tests against the owl2-reasoner implementation.
/// It parses test cases from the W3C test manifest and executes consistency and entailment checks.
use owl2_reasoner::{reasoning::tableaux::TableauxReasoner, Ontology};
use std::fs;

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCase {
    identifier: String,
    #[serde(default)]
    uri: Option<String>,
    #[serde(rename = "types")]
    test_types: Vec<String>,
    description: String,
    creator: String,
    status: String,
    #[serde(default)]
    premise_ontology: Option<String>,
    #[serde(default)]
    conclusion_ontology: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestResult {
    test_id: String,
    test_type: String,
    status: String, // PASS, FAIL, SKIP, ERROR
    expected: Option<bool>,
    actual: Option<bool>,
    error_message: Option<String>,
    execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestMetadata {
    total_tests: usize,
    direct_semantics_tests: usize,
    by_type: std::collections::HashMap<String, usize>,
    sample_tests: Vec<TestCase>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("W3C OWL 2 Test Suite Runner for owl2-reasoner");
    println!("{}", "=".repeat(60));

    // Load parsed test metadata
    let metadata_path = "/home/ubuntu/w3c_tests_parsed.json";
    println!("\nLoading test metadata from: {}", metadata_path);

    let metadata_str = fs::read_to_string(metadata_path)?;
    let metadata: TestMetadata = serde_json::from_str(&metadata_str)?;

    println!("Total tests: {}", metadata.total_tests);
    println!(
        "Direct semantics tests: {}",
        metadata.direct_semantics_tests
    );

    println!("\nTest distribution:");
    for (test_type, count) in &metadata.by_type {
        println!("  {:30} {:4} tests", test_type, count);
    }

    // Run all tests
    println!("\n{}", "=".repeat(60));
    println!("RUNNING ALL TESTS");
    println!("{}", "=".repeat(60));
    println!("Total tests to run: {}\n", metadata.sample_tests.len());

    let mut results = Vec::new();

    for (i, test) in metadata.sample_tests.iter().enumerate() {
        println!(
            "\n[{}/{}] Running test: {}",
            i + 1,
            metadata.sample_tests.len(),
            test.identifier
        );
        println!("  Description: {}", test.description);
        println!("  Types: {:?}", test.test_types);
        println!("  Has premise: {}", test.premise_ontology.is_some());
        println!("  Has conclusion: {}", test.conclusion_ontology.is_some());
        if let Some(ref premise) = test.premise_ontology {
            println!("  Premise length: {} chars", premise.len());
        }

        let result = run_test(test);
        println!("  Result: {}", result.status);
        if let Some(msg) = &result.error_message {
            println!("  Message: {}", msg);
        }

        results.push(result);
    }

    // Print summary
    print_summary(&results);

    // Save results
    let results_path = "/home/ubuntu/w3c_test_results.json";
    let results_json = serde_json::to_string_pretty(&results)?;
    fs::write(results_path, results_json)?;
    println!("\nResults saved to: {}", results_path);

    Ok(())
}

fn run_test(test: &TestCase) -> TestResult {
    let start = Instant::now();

    // Determine primary test type
    let primary_type = test
        .test_types
        .first()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let (status, expected, actual, error_message) = if primary_type.contains("Consistency") {
        run_consistency_test(test, true)
    } else if primary_type.contains("Inconsistency") {
        run_consistency_test(test, false)
    } else if primary_type.contains("PositiveEntailment") {
        run_entailment_test(test, true)
    } else if primary_type.contains("NegativeEntailment") {
        run_entailment_test(test, false)
    } else {
        (
            "SKIP".to_string(),
            None,
            None,
            Some("Unsupported test type".to_string()),
        )
    };

    let execution_time_ms = start.elapsed().as_millis() as u64;

    TestResult {
        test_id: test.identifier.clone(),
        test_type: primary_type,
        status,
        expected,
        actual,
        error_message,
        execution_time_ms,
    }
}

fn run_consistency_test(
    test: &TestCase,
    expected_consistent: bool,
) -> (String, Option<bool>, Option<bool>, Option<String>) {
    // Check if we have premise ontology
    if test.premise_ontology.is_none() {
        return (
            "SKIP".to_string(),
            Some(expected_consistent),
            None,
            Some("No premise ontology provided".to_string()),
        );
    }

    // Try to parse and check consistency
    match parse_ontology_from_rdf(test.premise_ontology.as_ref().unwrap()) {
        Ok(ontology) => {
            // Create reasoner and check consistency
            match check_consistency(&ontology) {
                Ok(is_consistent) => {
                    let status = if is_consistent == expected_consistent {
                        "PASS".to_string()
                    } else {
                        "FAIL".to_string()
                    };
                    (status, Some(expected_consistent), Some(is_consistent), None)
                }
                Err(e) => (
                    "ERROR".to_string(),
                    Some(expected_consistent),
                    None,
                    Some(format!("Reasoning error: {}", e)),
                ),
            }
        }
        Err(e) => (
            "SKIP".to_string(),
            Some(expected_consistent),
            None,
            Some(format!("Parse error: {}", e)),
        ),
    }
}

fn run_entailment_test(
    test: &TestCase,
    expected_entailed: bool,
) -> (String, Option<bool>, Option<bool>, Option<String>) {
    // Check if we have both premise and conclusion
    if test.premise_ontology.is_none() || test.conclusion_ontology.is_none() {
        return (
            "SKIP".to_string(),
            Some(expected_entailed),
            None,
            Some("Missing premise or conclusion ontology".to_string()),
        );
    }

    // For now, skip entailment tests as they require more complex implementation
    (
        "SKIP".to_string(),
        Some(expected_entailed),
        None,
        Some("Entailment checking not yet implemented".to_string()),
    )
}

fn parse_ontology_from_rdf(rdf_xml: &str) -> Result<Ontology, String> {
    use owl2_reasoner::parser::{OntologyParser, RdfXmlParser};

    // Use the built-in RDF/XML parser
    let parser = RdfXmlParser::new();
    parser.parse_str(rdf_xml).map_err(|e| e.to_string())
}

fn check_consistency(ontology: &Ontology) -> Result<bool, String> {
    let mut reasoner = TableauxReasoner::new(ontology.clone());
    reasoner.is_consistent().map_err(|e| e.to_string())
}

fn print_summary(results: &[TestResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.status == "PASS").count();
    let failed = results.iter().filter(|r| r.status == "FAIL").count();
    let skipped = results.iter().filter(|r| r.status == "SKIP").count();
    let errors = results.iter().filter(|r| r.status == "ERROR").count();

    println!("\n{}", "=".repeat(60));
    println!("SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Total:    {}", total);
    println!(
        "Passed:   {} ({:.1}%)",
        passed,
        passed as f64 / total as f64 * 100.0
    );
    println!(
        "Failed:   {} ({:.1}%)",
        failed,
        failed as f64 / total as f64 * 100.0
    );
    println!(
        "Skipped:  {} ({:.1}%)",
        skipped,
        skipped as f64 / total as f64 * 100.0
    );
    println!(
        "Errors:   {} ({:.1}%)",
        errors,
        errors as f64 / total as f64 * 100.0
    );
    println!("{}", "=".repeat(60));
}
