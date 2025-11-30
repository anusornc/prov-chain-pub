//! Data Integrity Validation and Correction System
//!
//! This module provides comprehensive integrity validation for ProvChainOrg's blockchain,
//! transaction counting, SPARQL queries, and RDF canonicalization mechanisms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod blockchain_validator;
pub mod canonicalization_validator;
pub mod monitor;
pub mod performance;
pub mod repair;
pub mod sparql_validator;
pub mod transaction_counter;
pub mod validator;

// Re-export main types for convenience
pub use blockchain_validator::BlockchainIntegrityValidator;
pub use canonicalization_validator::CanonicalizationValidator;
pub use monitor::IntegrityMonitor;
pub use performance::{
    BackgroundIntegrityService, OptimizedIntegrityValidator, PerformanceConfig, ProductionConfig,
    ValidationLevel,
};
pub use repair::IntegrityRepairEngine;
pub use sparql_validator::SparqlConsistencyValidator;
pub use transaction_counter::TransactionCountValidator;
pub use validator::IntegrityValidator;

/// Core integrity validation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityValidationReport {
    pub timestamp: DateTime<Utc>,
    pub blockchain_integrity: BlockchainIntegrityStatus,
    pub transaction_count_integrity: TransactionCountIntegrityStatus,
    pub sparql_query_integrity: SparqlIntegrityStatus,
    pub rdf_canonicalization_integrity: CanonicalizationIntegrityStatus,
    pub overall_status: IntegrityStatus,
    pub recommendations: Vec<IntegrityRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainIntegrityStatus {
    pub chain_length: usize,
    pub persistent_block_count: usize,
    pub missing_blocks: Vec<u64>,
    pub corrupted_blocks: Vec<u64>,
    pub hash_validation_errors: Vec<String>,
    pub reconstruction_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCountIntegrityStatus {
    pub reported_total_transactions: usize,
    pub actual_rdf_triple_count: usize,
    pub per_block_transaction_counts: HashMap<u64, TransactionCountDetails>,
    pub counting_discrepancies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCountDetails {
    pub block_index: u64,
    pub reported_count: usize,
    pub actual_triple_count: usize,
    pub rdf_parsing_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparqlIntegrityStatus {
    pub query_consistency_checks: Vec<QueryConsistencyResult>,
    pub graph_accessibility_issues: Vec<String>,
    pub canonicalization_query_mismatches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConsistencyResult {
    pub query: String,
    pub expected_result_count: usize,
    pub actual_result_count: usize,
    pub missing_graphs: Vec<String>,
    pub inaccessible_data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizationIntegrityStatus {
    pub algorithm_consistency_checks: Vec<CanonicalizationConsistencyResult>,
    pub blank_node_handling_issues: Vec<String>,
    pub hash_validation_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizationConsistencyResult {
    pub graph_name: String,
    pub custom_algorithm_hash: String,
    pub rdfc10_algorithm_hash: String,
    pub hashes_match: bool,
    pub complexity: crate::storage::rdf_store::GraphComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrityStatus {
    Healthy,
    Warning,
    Critical,
    Corrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityRecommendation {
    pub severity: RecommendationSeverity,
    pub category: String,
    pub description: String,
    pub action_required: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl IntegrityValidationReport {
    /// Create a new empty integrity validation report
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            blockchain_integrity: BlockchainIntegrityStatus::new(),
            transaction_count_integrity: TransactionCountIntegrityStatus::new(),
            sparql_query_integrity: SparqlIntegrityStatus::new(),
            rdf_canonicalization_integrity: CanonicalizationIntegrityStatus::new(),
            overall_status: IntegrityStatus::Healthy,
            recommendations: Vec::new(),
        }
    }

    /// Calculate overall integrity status based on component statuses
    pub fn calculate_overall_status(&mut self) {
        let mut has_critical = false;
        let mut has_warning = false;

        // Check blockchain integrity
        if !self.blockchain_integrity.missing_blocks.is_empty()
            || !self.blockchain_integrity.corrupted_blocks.is_empty()
            || !self.blockchain_integrity.hash_validation_errors.is_empty()
        {
            has_critical = true;
        }

        if !self.blockchain_integrity.reconstruction_errors.is_empty() {
            has_warning = true;
        }

        // Check transaction count integrity
        if !self
            .transaction_count_integrity
            .counting_discrepancies
            .is_empty()
        {
            has_warning = true;
        }

        // Special case: When there are no transactions, ontology triples shouldn't count as discrepancy
        let effective_triple_count =
            if self.transaction_count_integrity.reported_total_transactions == 0 {
                // When no transactions exist, expect 0 transaction-related triples
                0
            } else {
                // When transactions exist, compare against actual RDF triple count
                self.transaction_count_integrity.actual_rdf_triple_count
            };

        let total_discrepancy = self
            .transaction_count_integrity
            .reported_total_transactions
            .abs_diff(effective_triple_count);

        if total_discrepancy > 0 {
            if total_discrepancy > 10 {
                has_critical = true;
            } else {
                has_warning = true;
            }
        }

        // Check SPARQL integrity
        if !self
            .sparql_query_integrity
            .graph_accessibility_issues
            .is_empty()
            || !self
                .sparql_query_integrity
                .canonicalization_query_mismatches
                .is_empty()
        {
            has_warning = true;
        }

        for check in &self.sparql_query_integrity.query_consistency_checks {
            if check.expected_result_count != check.actual_result_count {
                has_warning = true;
            }
            if !check.missing_graphs.is_empty() || !check.inaccessible_data.is_empty() {
                has_critical = true;
            }
        }

        // Check canonicalization integrity
        if !self
            .rdf_canonicalization_integrity
            .hash_validation_failures
            .is_empty()
        {
            has_critical = true;
        }

        if !self
            .rdf_canonicalization_integrity
            .blank_node_handling_issues
            .is_empty()
        {
            has_warning = true;
        }

        for check in &self
            .rdf_canonicalization_integrity
            .algorithm_consistency_checks
        {
            if !check.hashes_match {
                has_critical = true;
            }
        }

        // Set overall status
        self.overall_status = if has_critical {
            IntegrityStatus::Critical
        } else if has_warning {
            IntegrityStatus::Warning
        } else {
            IntegrityStatus::Healthy
        };
    }

    /// Add a recommendation to the report
    pub fn add_recommendation(&mut self, recommendation: IntegrityRecommendation) {
        self.recommendations.push(recommendation);
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> IntegrityReportSummary {
        IntegrityReportSummary {
            overall_status: self.overall_status.clone(),
            total_issues: self.count_total_issues(),
            critical_issues: self.count_critical_issues(),
            warning_issues: self.count_warning_issues(),
            auto_fixable_issues: self.count_auto_fixable_issues(),
            timestamp: self.timestamp,
        }
    }

    fn count_total_issues(&self) -> usize {
        self.blockchain_integrity.missing_blocks.len()
            + self.blockchain_integrity.corrupted_blocks.len()
            + self.blockchain_integrity.hash_validation_errors.len()
            + self.blockchain_integrity.reconstruction_errors.len()
            + self
                .transaction_count_integrity
                .counting_discrepancies
                .len()
            + self.sparql_query_integrity.graph_accessibility_issues.len()
            + self
                .sparql_query_integrity
                .canonicalization_query_mismatches
                .len()
            + self
                .rdf_canonicalization_integrity
                .hash_validation_failures
                .len()
            + self
                .rdf_canonicalization_integrity
                .blank_node_handling_issues
                .len()
    }

    fn count_critical_issues(&self) -> usize {
        self.recommendations
            .iter()
            .filter(|r| {
                matches!(
                    r.severity,
                    RecommendationSeverity::Critical | RecommendationSeverity::Emergency
                )
            })
            .count()
    }

    fn count_warning_issues(&self) -> usize {
        self.recommendations
            .iter()
            .filter(|r| matches!(r.severity, RecommendationSeverity::Warning))
            .count()
    }

    fn count_auto_fixable_issues(&self) -> usize {
        self.recommendations
            .iter()
            .filter(|r| r.auto_fixable)
            .count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReportSummary {
    pub overall_status: IntegrityStatus,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub warning_issues: usize,
    pub auto_fixable_issues: usize,
    pub timestamp: DateTime<Utc>,
}

impl BlockchainIntegrityStatus {
    pub fn new() -> Self {
        Self {
            chain_length: 0,
            persistent_block_count: 0,
            missing_blocks: Vec::new(),
            corrupted_blocks: Vec::new(),
            hash_validation_errors: Vec::new(),
            reconstruction_errors: Vec::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.missing_blocks.is_empty()
            && self.corrupted_blocks.is_empty()
            && self.hash_validation_errors.is_empty()
            && self.reconstruction_errors.is_empty()
            && self.chain_length == self.persistent_block_count
    }
}

impl TransactionCountIntegrityStatus {
    pub fn new() -> Self {
        Self {
            reported_total_transactions: 0,
            actual_rdf_triple_count: 0,
            per_block_transaction_counts: HashMap::new(),
            counting_discrepancies: Vec::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.counting_discrepancies.is_empty()
            && self.reported_total_transactions == self.actual_rdf_triple_count
    }
}

impl SparqlIntegrityStatus {
    pub fn new() -> Self {
        Self {
            query_consistency_checks: Vec::new(),
            graph_accessibility_issues: Vec::new(),
            canonicalization_query_mismatches: Vec::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.graph_accessibility_issues.is_empty()
            && self.canonicalization_query_mismatches.is_empty()
            && self.query_consistency_checks.iter().all(|check| {
                check.expected_result_count == check.actual_result_count
                    && check.missing_graphs.is_empty()
                    && check.inaccessible_data.is_empty()
            })
    }
}

impl CanonicalizationIntegrityStatus {
    pub fn new() -> Self {
        Self {
            algorithm_consistency_checks: Vec::new(),
            blank_node_handling_issues: Vec::new(),
            hash_validation_failures: Vec::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.blank_node_handling_issues.is_empty()
            && self.hash_validation_failures.is_empty()
            && self
                .algorithm_consistency_checks
                .iter()
                .all(|check| check.hashes_match)
    }
}

impl Default for IntegrityValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BlockchainIntegrityStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TransactionCountIntegrityStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SparqlIntegrityStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CanonicalizationIntegrityStatus {
    fn default() -> Self {
        Self::new()
    }
}
