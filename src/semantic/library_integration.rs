//! Integration with the owl2-reasoner library
//!
//! This module provides a high-level interface to the owl2-reasoner library
//! for ontology consistency checking.

use anyhow::{Context, Result};
use owl2_reasoner::parser::TurtleParser;
use owl2_reasoner::OntologyParser;
use owl2_reasoner::reasoning::tableaux::{ReasoningConfig, TableauxReasoner};
use std::fs;
use tracing::info;

/// Check consistency of an ontology file using the TableauxReasoner
pub fn check_consistency(ontology_path: &str) -> Result<bool> {
    info!("Checking consistency of ontology: {}", ontology_path);

    // Read the ontology file
    let ontology_content = fs::read_to_string(ontology_path)
        .with_context(|| format!("Failed to read ontology file: {}", ontology_path))?;

    // Parse the ontology
    let parser = TurtleParser::new();
    let ontology = parser.parse_str(&ontology_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse ontology: {}", e))?;

    // Configure reasoner
    let config = ReasoningConfig {
        max_depth: 1000,
        debug: false,
        incremental: true,
        timeout: Some(30000), // 30 seconds
        enable_parallel: false,
        parallel_workers: None,
        parallel_chunk_size: 64,
    };

    // Create reasoner
    // Note: with_config returns Self directly, not Result
    let mut reasoner = TableauxReasoner::with_config(ontology, config);

    // Check consistency
    let is_consistent = reasoner.is_consistent()
        .map_err(|e| anyhow::anyhow!("Reasoning failed: {}", e))?;

    Ok(is_consistent)
}

/// Validate an ontology file using the AcademicValidator
pub fn validate_ontology(ontology_path: &str) -> Result<owl2_reasoner::AcademicValidationReport> {
    info!("Validating ontology: {}", ontology_path);

    // Read the ontology file
    let ontology_content = fs::read_to_string(ontology_path)
        .with_context(|| format!("Failed to read ontology file: {}", ontology_path))?;

    // Parse the ontology
    let parser = TurtleParser::new();
    let ontology = parser.parse_str(&ontology_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse ontology: {}", e))?;

    // Validate
    let validator = owl2_reasoner::AcademicValidator::new();
    let report = validator.validate(&ontology)
        .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

    Ok(report)
}
