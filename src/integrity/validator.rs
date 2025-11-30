//! Main integrity validation coordinator
//!
//! This module provides the central IntegrityValidator that orchestrates
//! all integrity validation activities across the ProvChainOrg system.

use crate::core::blockchain::Blockchain;
use crate::error::Result;
use crate::integrity::blockchain_validator::BlockchainIntegrityValidator;
use crate::integrity::canonicalization_validator::CanonicalizationValidator;
use crate::integrity::sparql_validator::SparqlConsistencyValidator;
use crate::integrity::transaction_counter::TransactionCountValidator;
#[cfg(test)]
use crate::integrity::IntegrityStatus;
use crate::integrity::{
    BlockchainIntegrityStatus, CanonicalizationIntegrityStatus, IntegrityRecommendation,
    IntegrityValidationReport, RecommendationSeverity, SparqlIntegrityStatus,
    TransactionCountIntegrityStatus,
};
use crate::storage::rdf_store::RDFStore;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

/// Main integrity validation coordinator
pub struct IntegrityValidator {
    /// Enable detailed logging during validation
    pub verbose_logging: bool,
    /// Maximum time to spend on validation (in seconds)
    pub max_validation_time: u64,
    /// Enable automatic repair suggestions
    pub enable_repair_suggestions: bool,
}

impl IntegrityValidator {
    /// Create a new integrity validator with default settings
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            max_validation_time: 300, // 5 minutes default
            enable_repair_suggestions: true,
        }
    }

    /// Create a new integrity validator with custom settings
    pub fn with_config(verbose: bool, max_time: u64, enable_repair: bool) -> Self {
        Self {
            verbose_logging: verbose,
            max_validation_time: max_time,
            enable_repair_suggestions: enable_repair,
        }
    }

    /// Perform comprehensive system integrity validation
    #[instrument(skip(self, blockchain), fields(chain_length = blockchain.chain.len()))]
    pub fn validate_system_integrity(
        &self,
        blockchain: &Blockchain,
    ) -> Result<IntegrityValidationReport> {
        let start_time = Instant::now();
        info!("Starting comprehensive system integrity validation");

        let mut report = IntegrityValidationReport::new();

        // Phase 1: Validate blockchain integrity
        debug!("Phase 1: Validating blockchain integrity");
        match self.validate_blockchain_integrity(blockchain) {
            Ok(blockchain_status) => {
                report.blockchain_integrity = blockchain_status;
                if self.verbose_logging {
                    info!(
                        "Blockchain integrity validation completed: {} blocks checked",
                        report.blockchain_integrity.chain_length
                    );
                }
            }
            Err(e) => {
                error!("Blockchain integrity validation failed: {}", e);
                report
                    .blockchain_integrity
                    .reconstruction_errors
                    .push(format!("Validation failed: {}", e));
            }
        }

        // Phase 2: Validate transaction counting
        debug!("Phase 2: Validating transaction counting");
        match self.validate_transaction_counts(blockchain) {
            Ok(transaction_status) => {
                report.transaction_count_integrity = transaction_status;
                if self.verbose_logging {
                    info!(
                        "Transaction count validation completed: {} reported vs {} actual",
                        report
                            .transaction_count_integrity
                            .reported_total_transactions,
                        report.transaction_count_integrity.actual_rdf_triple_count
                    );
                }
            }
            Err(e) => {
                error!("Transaction count validation failed: {}", e);
                report
                    .transaction_count_integrity
                    .counting_discrepancies
                    .push(format!("Validation failed: {}", e));
            }
        }

        // Phase 3: Validate SPARQL consistency
        debug!("Phase 3: Validating SPARQL consistency");
        match self.validate_sparql_consistency(&blockchain.rdf_store) {
            Ok(sparql_status) => {
                report.sparql_query_integrity = sparql_status;
                if self.verbose_logging {
                    info!(
                        "SPARQL consistency validation completed: {} queries checked",
                        report.sparql_query_integrity.query_consistency_checks.len()
                    );
                }
            }
            Err(e) => {
                error!("SPARQL consistency validation failed: {}", e);
                report
                    .sparql_query_integrity
                    .graph_accessibility_issues
                    .push(format!("Validation failed: {}", e));
            }
        }

        // Phase 4: Validate canonicalization integrity
        debug!("Phase 4: Validating canonicalization integrity");
        match self.validate_canonicalization_integrity(&blockchain.rdf_store) {
            Ok(canonicalization_status) => {
                report.rdf_canonicalization_integrity = canonicalization_status;
                if self.verbose_logging {
                    info!(
                        "Canonicalization integrity validation completed: {} algorithms checked",
                        report
                            .rdf_canonicalization_integrity
                            .algorithm_consistency_checks
                            .len()
                    );
                }
            }
            Err(e) => {
                error!("Canonicalization integrity validation failed: {}", e);
                report
                    .rdf_canonicalization_integrity
                    .hash_validation_failures
                    .push(format!("Validation failed: {}", e));
            }
        }

        // Phase 5: Calculate overall status and generate recommendations
        debug!("Phase 5: Calculating overall status and generating recommendations");
        report.calculate_overall_status();

        if self.enable_repair_suggestions {
            self.generate_recommendations(&mut report);
        }

        let validation_time = start_time.elapsed();
        info!(
            "System integrity validation completed in {:?} with status: {:?}",
            validation_time, report.overall_status
        );

        // Check if validation took too long
        if validation_time.as_secs() > self.max_validation_time {
            warn!(
                "Integrity validation took longer than expected: {:?} > {}s",
                validation_time, self.max_validation_time
            );
            report.add_recommendation(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "Performance".to_string(),
                description: "Integrity validation is taking longer than expected".to_string(),
                action_required: "Consider optimizing validation queries or increasing timeout"
                    .to_string(),
                auto_fixable: false,
            });
        }

        Ok(report)
    }

    /// Validate blockchain integrity
    #[instrument(skip(self, blockchain))]
    pub fn validate_blockchain_integrity(
        &self,
        blockchain: &Blockchain,
    ) -> Result<BlockchainIntegrityStatus> {
        let mut status = BlockchainIntegrityStatus::new();
        status.chain_length = blockchain.chain.len();

        // Get persistent block count from RDF store
        status.persistent_block_count = self.count_persistent_blocks(&blockchain.rdf_store)?;

        // Check for missing blocks
        status.missing_blocks = self.detect_missing_blocks(blockchain)?;

        // Validate block hash integrity
        status.hash_validation_errors = self.validate_block_hash_integrity(blockchain)?;

        // Check for corrupted blocks
        status.corrupted_blocks = self.detect_corrupted_blocks(blockchain)?;

        // Validate chain reconstruction
        status.reconstruction_errors = self.validate_chain_reconstruction(blockchain)?;

        if self.verbose_logging {
            debug!("Blockchain integrity status: chain_length={}, persistent_count={}, missing={}, corrupted={}",
                   status.chain_length, status.persistent_block_count,
                   status.missing_blocks.len(), status.corrupted_blocks.len());
        }

        Ok(status)
    }

    /// Validate transaction counting accuracy
    #[instrument(skip(self, blockchain))]
    pub fn validate_transaction_counts(
        &self,
        blockchain: &Blockchain,
    ) -> Result<TransactionCountIntegrityStatus> {
        let mut status = TransactionCountIntegrityStatus::new();

        // Current reported count (simplified - each block = 1 transaction)
        status.reported_total_transactions = blockchain.chain.len();

        // Count actual RDF triples across all graphs
        status.actual_rdf_triple_count = self.count_actual_rdf_triples(&blockchain.rdf_store)?;

        // Validate per-block transaction counts
        status.per_block_transaction_counts = self.validate_per_block_counts(blockchain)?;

        // Identify counting discrepancies
        if status.reported_total_transactions != status.actual_rdf_triple_count {
            status.counting_discrepancies.push(format!(
                "Total transaction count mismatch: reported {} vs actual RDF triples {}",
                status.reported_total_transactions, status.actual_rdf_triple_count
            ));
        }

        // Check for blocks with parsing errors
        for (block_index, details) in &status.per_block_transaction_counts {
            if !details.rdf_parsing_errors.is_empty() {
                status.counting_discrepancies.push(format!(
                    "Block {} has RDF parsing errors: {:?}",
                    block_index, details.rdf_parsing_errors
                ));
            }
        }

        if self.verbose_logging {
            debug!(
                "Transaction count validation: reported={}, actual={}, discrepancies={}",
                status.reported_total_transactions,
                status.actual_rdf_triple_count,
                status.counting_discrepancies.len()
            );
        }

        Ok(status)
    }

    /// Validate SPARQL query consistency
    #[instrument(skip(self, rdf_store))]
    pub fn validate_sparql_consistency(
        &self,
        rdf_store: &RDFStore,
    ) -> Result<SparqlIntegrityStatus> {
        let mut status = SparqlIntegrityStatus::new();

        // Test standard queries for consistency
        let test_queries = self.get_standard_test_queries();

        for query in test_queries {
            match self.validate_query_consistency(rdf_store, &query) {
                Ok(result) => status.query_consistency_checks.push(result),
                Err(e) => {
                    status
                        .graph_accessibility_issues
                        .push(format!("Query validation failed: {}", e));
                }
            }
        }

        // Check graph accessibility
        status
            .graph_accessibility_issues
            .extend(self.validate_graph_accessibility(rdf_store)?);

        // Cross-validate canonicalization with queries
        status
            .canonicalization_query_mismatches
            .extend(self.cross_validate_canonicalization_queries(rdf_store)?);

        if self.verbose_logging {
            debug!("SPARQL consistency validation: {} queries checked, {} accessibility issues, {} canonicalization mismatches",
                   status.query_consistency_checks.len(),
                   status.graph_accessibility_issues.len(),
                   status.canonicalization_query_mismatches.len());
        }

        Ok(status)
    }

    /// Validate RDF canonicalization integrity
    #[instrument(skip(self, rdf_store))]
    pub fn validate_canonicalization_integrity(
        &self,
        rdf_store: &RDFStore,
    ) -> Result<CanonicalizationIntegrityStatus> {
        let mut status = CanonicalizationIntegrityStatus::new();

        // Get all named graphs for testing
        let graph_names = self.get_all_graph_names(rdf_store)?;

        for graph_name in graph_names {
            // Skip genesis block and blockchain metadata graphs from consistency checks
            if graph_name == "http://provchain.org/block/0"
                || graph_name == "http://provchain.org/blockchain"
            {
                if self.verbose_logging {
                    debug!(
                        "Skipping canonicalization consistency check for special graph: {}",
                        graph_name
                    );
                }
                continue;
            }

            match self.validate_canonicalization_consistency(rdf_store, &graph_name) {
                Ok(result) => status.algorithm_consistency_checks.push(result),
                Err(e) => {
                    status.hash_validation_failures.push(format!(
                        "Canonicalization validation failed for graph {}: {}",
                        graph_name, e
                    ));
                }
            }
        }

        // Check blank node handling
        status
            .blank_node_handling_issues
            .extend(self.validate_blank_node_handling(rdf_store)?);

        if self.verbose_logging {
            debug!("Canonicalization integrity validation: {} graphs checked, {} consistency checks, {} blank node issues", 
                   status.algorithm_consistency_checks.len(),
                   status.algorithm_consistency_checks.len(),
                   status.blank_node_handling_issues.len());
        }

        Ok(status)
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(&self, report: &mut IntegrityValidationReport) {
        // Blockchain integrity recommendations
        if !report.blockchain_integrity.missing_blocks.is_empty() {
            report.add_recommendation(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Blockchain".to_string(),
                description: format!(
                    "Missing blocks detected: {:?}",
                    report.blockchain_integrity.missing_blocks
                ),
                action_required: "Restore missing blocks from backup or resync from network"
                    .to_string(),
                auto_fixable: false,
            });
        }

        if !report.blockchain_integrity.corrupted_blocks.is_empty() {
            report.add_recommendation(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Blockchain".to_string(),
                description: format!(
                    "Corrupted blocks detected: {:?}",
                    report.blockchain_integrity.corrupted_blocks
                ),
                action_required: "Restore corrupted blocks from backup or resync from network"
                    .to_string(),
                auto_fixable: false,
            });
        }

        // Transaction count recommendations
        let count_discrepancy = report
            .transaction_count_integrity
            .reported_total_transactions
            .abs_diff(report.transaction_count_integrity.actual_rdf_triple_count);

        if count_discrepancy > 0 {
            let severity = if count_discrepancy > 10 {
                RecommendationSeverity::Critical
            } else {
                RecommendationSeverity::Warning
            };

            report.add_recommendation(IntegrityRecommendation {
                severity,
                category: "Transaction Counting".to_string(),
                description: format!(
                    "Transaction count mismatch: {} discrepancy",
                    count_discrepancy
                ),
                action_required:
                    "Update transaction counting logic to parse RDF content accurately".to_string(),
                auto_fixable: true,
            });
        }

        // SPARQL consistency recommendations
        if !report
            .sparql_query_integrity
            .graph_accessibility_issues
            .is_empty()
        {
            report.add_recommendation(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "SPARQL".to_string(),
                description: "Graph accessibility issues detected".to_string(),
                action_required: "Check RDF store integrity and graph naming consistency"
                    .to_string(),
                auto_fixable: false,
            });
        }

        // Canonicalization recommendations
        let canonicalization_recommendations: Vec<_> = report
            .rdf_canonicalization_integrity
            .algorithm_consistency_checks
            .iter()
            .filter(|check| !check.hashes_match)
            .map(|check| IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Canonicalization".to_string(),
                description: format!("Hash mismatch in graph {}", check.graph_name),
                action_required: "Investigate canonicalization algorithm consistency".to_string(),
                auto_fixable: false,
            })
            .collect();

        for recommendation in canonicalization_recommendations {
            report.add_recommendation(recommendation);
        }

        // Performance recommendations
        if report.blockchain_integrity.chain_length > 1000 {
            report.add_recommendation(IntegrityRecommendation {
                severity: RecommendationSeverity::Info,
                category: "Performance".to_string(),
                description: "Large blockchain detected".to_string(),
                action_required:
                    "Consider implementing incremental validation for better performance"
                        .to_string(),
                auto_fixable: false,
            });
        }
    }

    // Helper methods for validation - Phase 2 implementations using BlockchainIntegrityValidator

    fn count_persistent_blocks(&self, rdf_store: &RDFStore) -> Result<usize> {
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.count_persistent_blocks(rdf_store)
    }

    fn detect_missing_blocks(&self, blockchain: &Blockchain) -> Result<Vec<u64>> {
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.detect_missing_blocks(blockchain)
    }

    fn validate_block_hash_integrity(&self, blockchain: &Blockchain) -> Result<Vec<String>> {
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.validate_block_hash_integrity(blockchain)
    }

    fn detect_corrupted_blocks(&self, blockchain: &Blockchain) -> Result<Vec<u64>> {
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.detect_corrupted_blocks(blockchain)
    }

    fn validate_chain_reconstruction(&self, blockchain: &Blockchain) -> Result<Vec<String>> {
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.validate_chain_reconstruction(blockchain)
    }

    fn count_actual_rdf_triples(&self, rdf_store: &RDFStore) -> Result<usize> {
        let validator =
            TransactionCountValidator::with_config(self.verbose_logging, true, 1024 * 1024);
        validator.count_actual_rdf_triples(rdf_store)
    }

    fn validate_per_block_counts(
        &self,
        blockchain: &Blockchain,
    ) -> Result<std::collections::HashMap<u64, crate::integrity::TransactionCountDetails>> {
        let validator =
            TransactionCountValidator::with_config(self.verbose_logging, true, 1024 * 1024);
        validator.count_actual_transactions_per_block(blockchain)
    }

    fn get_standard_test_queries(&self) -> Vec<String> {
        let validator =
            SparqlConsistencyValidator::with_config(self.verbose_logging, 30, Vec::new());
        validator.test_queries
    }

    fn validate_query_consistency(
        &self,
        rdf_store: &RDFStore,
        query: &str,
    ) -> Result<crate::integrity::QueryConsistencyResult> {
        let validator =
            SparqlConsistencyValidator::with_config(self.verbose_logging, 30, Vec::new());
        validator.validate_single_query_consistency(rdf_store, query)
    }

    fn validate_graph_accessibility(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let validator =
            SparqlConsistencyValidator::with_config(self.verbose_logging, 30, Vec::new());
        validator.validate_graph_accessibility(rdf_store)
    }

    fn cross_validate_canonicalization_queries(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let validator =
            SparqlConsistencyValidator::with_config(self.verbose_logging, 30, Vec::new());
        validator.cross_validate_canonicalization_queries(rdf_store)
    }

    fn get_all_graph_names(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);
        validator.get_all_graph_names(rdf_store)
    }

    fn validate_canonicalization_consistency(
        &self,
        rdf_store: &RDFStore,
        graph_name: &str,
    ) -> Result<crate::integrity::CanonicalizationConsistencyResult> {
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);
        validator.validate_single_graph_consistency(rdf_store, graph_name)
    }

    fn validate_blank_node_handling(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);
        validator.validate_blank_node_handling(rdf_store)
    }
}

impl Default for IntegrityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_integrity_validator_creation() {
        let validator = IntegrityValidator::new();
        assert!(!validator.verbose_logging);
        assert_eq!(validator.max_validation_time, 300);
        assert!(validator.enable_repair_suggestions);
    }

    #[test]
    fn test_integrity_validator_with_config() {
        let validator = IntegrityValidator::with_config(true, 600, false);
        assert!(validator.verbose_logging);
        assert_eq!(validator.max_validation_time, 600);
        assert!(!validator.enable_repair_suggestions);
    }

    #[test]
    fn test_validate_system_integrity_basic() {
        let validator = IntegrityValidator::new();
        let blockchain = Blockchain::new();

        let result = validator.validate_system_integrity(&blockchain);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.overall_status, IntegrityStatus::Healthy);
    }
}
