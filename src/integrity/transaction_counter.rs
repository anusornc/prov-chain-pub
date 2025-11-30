//! Accurate transaction counting system
//!
//! This module provides accurate transaction counting by parsing RDF content
//! instead of using hardcoded values, addressing the core issue of transaction
//! count mismatches between frontend and backend.

use crate::core::blockchain::{Block, Blockchain};
use crate::error::Result;
use crate::integrity::{
    IntegrityRecommendation, RecommendationSeverity, TransactionCountDetails,
    TransactionCountIntegrityStatus,
};
use crate::storage::rdf_store::RDFStore;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

/// Accurate transaction counting validator
pub struct TransactionCountValidator {
    /// Enable detailed counting logging
    pub verbose_logging: bool,
    /// Parse RDF content to count actual triples
    pub parse_rdf_content: bool,
    /// Maximum RDF content size to parse (in bytes)
    pub max_rdf_parse_size: usize,
}

impl TransactionCountValidator {
    /// Create a new transaction count validator
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            parse_rdf_content: true,
            max_rdf_parse_size: 1024 * 1024, // 1MB default
        }
    }

    /// Create a validator with custom configuration
    pub fn with_config(verbose: bool, parse_rdf: bool, max_parse_size: usize) -> Self {
        Self {
            verbose_logging: verbose,
            parse_rdf_content: parse_rdf,
            max_rdf_parse_size: max_parse_size,
        }
    }

    /// Count actual transactions per block by parsing RDF content
    #[instrument(skip(self, blockchain))]
    pub fn count_actual_transactions_per_block(
        &self,
        blockchain: &Blockchain,
    ) -> Result<HashMap<u64, TransactionCountDetails>> {
        let mut per_block_counts = HashMap::new();

        if self.verbose_logging {
            info!(
                "Counting actual transactions for {} blocks",
                blockchain.chain.len()
            );
        }

        for block in &blockchain.chain {
            let details = self.count_transactions_in_block(block)?;
            per_block_counts.insert(block.index, details);
        }

        debug!(
            "Transaction counting completed for {} blocks",
            per_block_counts.len()
        );
        Ok(per_block_counts)
    }

    /// Count transactions in a single block
    #[instrument(skip(self, block))]
    pub fn count_transactions_in_block(&self, block: &Block) -> Result<TransactionCountDetails> {
        let mut details = TransactionCountDetails {
            block_index: block.index,
            reported_count: 1, // Current hardcoded logic: 1 transaction per block
            actual_triple_count: 0,
            rdf_parsing_errors: Vec::new(),
        };

        if self.parse_rdf_content {
            match self.parse_rdf_content_for_transactions(&block.data) {
                Ok(count) => {
                    details.actual_triple_count = count;
                    if self.verbose_logging {
                        debug!(
                            "Block {}: parsed {} triples from RDF content",
                            block.index, count
                        );
                    }
                }
                Err(e) => {
                    details
                        .rdf_parsing_errors
                        .push(format!("RDF parsing failed: {}", e));
                    error!(
                        "Failed to parse RDF content for block {}: {}",
                        block.index, e
                    );
                }
            }
        }

        Ok(details)
    }

    /// Parse RDF content to count actual triples/transactions
    #[instrument(skip(self, rdf_data))]
    pub fn parse_rdf_content_for_transactions(&self, rdf_data: &str) -> Result<usize> {
        if rdf_data.len() > self.max_rdf_parse_size {
            return Err(crate::error::ProvChainError::Validation(
                crate::error::ValidationError::InvalidInput {
                    field: "rdf_data".to_string(),
                    reason: format!(
                        "RDF content too large: {} bytes > {} bytes limit",
                        rdf_data.len(),
                        self.max_rdf_parse_size
                    ),
                },
            ));
        }

        if rdf_data.trim().is_empty() {
            return Ok(0);
        }

        // Parse RDF content using Oxigraph to count actual triples
        use oxigraph::io::RdfFormat;
        use oxigraph::store::Store;
        use std::io::Cursor;

        let temp_store = Store::new().map_err(|e| {
            crate::error::ProvChainError::Storage(crate::error::StorageError::ConnectionFailed(
                format!("Failed to create temporary store: {}", e),
            ))
        })?;

        // Try to parse as Turtle format first (most common in ProvChainOrg)
        let reader = Cursor::new(rdf_data.as_bytes());
        match temp_store.load_from_reader(RdfFormat::Turtle, reader) {
            Ok(_) => {
                // Successfully parsed as Turtle, count triples
                let triple_count = temp_store.len().unwrap_or(0);

                if self.verbose_logging {
                    debug!(
                        "Successfully parsed {} triples from Turtle RDF content",
                        triple_count
                    );
                }

                Ok(triple_count)
            }
            Err(turtle_error) => {
                // Try N-Triples format as fallback
                let reader = Cursor::new(rdf_data.as_bytes());
                match temp_store.load_from_reader(RdfFormat::NTriples, reader) {
                    Ok(_) => {
                        let triple_count = temp_store.len().unwrap_or(0);

                        if self.verbose_logging {
                            debug!(
                                "Successfully parsed {} triples from N-Triples RDF content",
                                triple_count
                            );
                        }

                        Ok(triple_count)
                    }
                    Err(ntriples_error) => {
                        // Try RDF/XML format as second fallback
                        let reader = Cursor::new(rdf_data.as_bytes());
                        match temp_store.load_from_reader(RdfFormat::RdfXml, reader) {
                            Ok(_) => {
                                let triple_count = temp_store.len().unwrap_or(0);

                                if self.verbose_logging {
                                    debug!(
                                        "Successfully parsed {} triples from RDF/XML content",
                                        triple_count
                                    );
                                }

                                Ok(triple_count)
                            }
                            Err(rdfxml_error) => {
                                // All parsing attempts failed, fall back to line counting
                                if self.verbose_logging {
                                    warn!("RDF parsing failed (Turtle: {}, N-Triples: {}, RDF/XML: {}), falling back to line counting", 
                                          turtle_error, ntriples_error, rdfxml_error);
                                }

                                // Count non-empty, non-comment lines as a fallback
                                let line_count = rdf_data
                                    .lines()
                                    .filter(|line| {
                                        let trimmed = line.trim();
                                        !trimmed.is_empty()
                                            && !trimmed.starts_with('#')
                                            && !trimmed.starts_with("@prefix")
                                            && !trimmed.starts_with("@base")
                                    })
                                    .count();

                                if self.verbose_logging {
                                    debug!(
                                        "Fallback line counting: {} transaction lines",
                                        line_count
                                    );
                                }

                                Ok(line_count)
                            }
                        }
                    }
                }
            }
        }
    }

    /// Count total RDF triples across all graphs in the store
    #[instrument(skip(self, rdf_store))]
    pub fn count_actual_rdf_triples(&self, rdf_store: &RDFStore) -> Result<usize> {
        if self.verbose_logging {
            info!("Counting total RDF triples in store");
        }

        // Method 1: Use store length (includes all quads/triples)
        let total_quads = rdf_store.store.len().unwrap_or(0);

        // Method 2: Query for data triples only (excluding metadata)
        let data_triples_query = r#"
            SELECT (COUNT(*) as ?count) WHERE {
                GRAPH ?g {
                    ?s ?p ?o .
                    FILTER(STRSTARTS(STR(?g), "http://provchain.org/block/"))
                }
            }
        "#;

        let data_triple_count = match rdf_store.query(data_triples_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut count = 0;
                for sol in solutions.flatten() {
                    if let Some(count_term) = sol.get("count") {
                        if let oxigraph::model::Term::Literal(lit) = count_term {
                            if let Ok(parsed_count) = lit.value().parse::<usize>() {
                                count = parsed_count;
                                break;
                            }
                        }
                    }
                }
                count
            }
            _ => {
                if self.verbose_logging {
                    warn!("Failed to query data triples, using total quad count");
                }
                total_quads
            }
        };

        // Method 3: Count triples per block graph for verification
        let per_block_query = r#"
            SELECT ?g (COUNT(*) as ?count) WHERE {
                GRAPH ?g {
                    ?s ?p ?o .
                    FILTER(STRSTARTS(STR(?g), "http://provchain.org/block/"))
                }
            }
            GROUP BY ?g
            ORDER BY ?g
        "#;

        let mut per_block_total = 0;
        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            rdf_store.query(per_block_query)
        {
            for sol in solutions.flatten() {
                if let Some(count_term) = sol.get("count") {
                    if let oxigraph::model::Term::Literal(lit) = count_term {
                        if let Ok(block_count) = lit.value().parse::<usize>() {
                            per_block_total += block_count;
                        }
                    }
                }
            }
        }

        // Use the most accurate count available
        let final_count = if data_triple_count > 0 {
            data_triple_count
        } else if per_block_total > 0 {
            per_block_total
        } else {
            total_quads
        };

        if self.verbose_logging {
            debug!("RDF triple counts - Total quads: {}, Data triples: {}, Per-block total: {}, Final count: {}", 
                   total_quads, data_triple_count, per_block_total, final_count);
        }

        debug!("Total RDF triples in store: {}", final_count);
        Ok(final_count)
    }

    /// Validate transaction count consistency across the blockchain
    #[instrument(skip(self, blockchain))]
    pub fn validate_transaction_count_consistency(
        &self,
        blockchain: &Blockchain,
    ) -> Result<Vec<String>> {
        let mut inconsistencies = Vec::new();

        if self.verbose_logging {
            info!(
                "Validating transaction count consistency for {} blocks",
                blockchain.chain.len()
            );
        }

        // 1. Get per-block transaction counts
        let per_block_counts = self.count_actual_transactions_per_block(blockchain)?;

        // 2. Compare reported counts with actual parsed counts
        for (block_index, details) in &per_block_counts {
            if details.reported_count != details.actual_triple_count {
                inconsistencies.push(format!(
                    "Block {} count mismatch: reported {} vs actual {} triples",
                    block_index, details.reported_count, details.actual_triple_count
                ));
            }

            // Check for RDF parsing errors
            if !details.rdf_parsing_errors.is_empty() {
                inconsistencies.push(format!(
                    "Block {} has RDF parsing errors: {:?}",
                    block_index, details.rdf_parsing_errors
                ));
            }
        }

        // 3. Check for systematic counting errors
        let total_reported: usize = per_block_counts
            .values()
            .map(|details| details.reported_count)
            .sum();
        let total_actual: usize = per_block_counts
            .values()
            .map(|details| details.actual_triple_count)
            .sum();

        if total_reported != total_actual {
            inconsistencies.push(format!(
                "Total count mismatch: {} reported vs {} actual transactions",
                total_reported, total_actual
            ));
        }

        // 4. Validate count consistency with RDF store
        let store_triple_count = self.count_actual_rdf_triples(&blockchain.rdf_store)?;
        if total_actual != store_triple_count {
            inconsistencies.push(format!(
                "Store consistency mismatch: {} block triples vs {} store triples",
                total_actual, store_triple_count
            ));
        }

        // 5. Identify patterns in counting discrepancies
        let blocks_with_discrepancies: Vec<_> = per_block_counts
            .iter()
            .filter(|(_, details)| details.reported_count != details.actual_triple_count)
            .collect();

        if blocks_with_discrepancies.len() > blockchain.chain.len() / 2 {
            inconsistencies.push(format!(
                "Systematic counting error: {}/{} blocks have count discrepancies",
                blocks_with_discrepancies.len(),
                blockchain.chain.len()
            ));
        }

        // Check for consistent over/under counting patterns
        let over_counted: Vec<_> = per_block_counts
            .iter()
            .filter(|(_, details)| details.reported_count > details.actual_triple_count)
            .collect();
        let under_counted: Vec<_> = per_block_counts
            .iter()
            .filter(|(_, details)| details.reported_count < details.actual_triple_count)
            .collect();

        if over_counted.len() > blockchain.chain.len() * 3 / 4 {
            inconsistencies.push(format!(
                "Systematic over-counting pattern: {}/{} blocks report more transactions than actual",
                over_counted.len(), blockchain.chain.len()
            ));
        }

        if under_counted.len() > blockchain.chain.len() * 3 / 4 {
            inconsistencies.push(format!(
                "Systematic under-counting pattern: {}/{} blocks report fewer transactions than actual",
                under_counted.len(), blockchain.chain.len()
            ));
        }

        // 6. Validate count consistency across different access methods
        // Compare direct block parsing with RDF store queries
        for block in &blockchain.chain {
            let block_graph_query = format!(
                r#"
                SELECT (COUNT(*) as ?count) WHERE {{
                    GRAPH <http://provchain.org/block/{}> {{
                        ?s ?p ?o .
                    }}
                }}
            "#,
                block.index
            );

            let store_count = match blockchain.rdf_store.query(&block_graph_query) {
                oxigraph::sparql::QueryResults::Solutions(solutions) => {
                    let mut count = 0;
                    for sol in solutions.flatten() {
                        if let Some(count_term) = sol.get("count") {
                            if let oxigraph::model::Term::Literal(lit) = count_term {
                                if let Ok(parsed_count) = lit.value().parse::<usize>() {
                                    count = parsed_count;
                                    break;
                                }
                            }
                        }
                    }
                    count
                }
                _ => 0,
            };

            if let Some(details) = per_block_counts.get(&block.index) {
                if details.actual_triple_count != store_count {
                    inconsistencies.push(format!(
                        "Block {} access method inconsistency: parsed {} vs store query {}",
                        block.index, details.actual_triple_count, store_count
                    ));
                }
            }
        }

        if self.verbose_logging {
            if inconsistencies.is_empty() {
                info!(
                    "Transaction count consistency validation passed for {} blocks",
                    blockchain.chain.len()
                );
            } else {
                warn!(
                    "Found {} transaction count inconsistencies",
                    inconsistencies.len()
                );
            }
        }

        debug!(
            "Transaction count consistency validation completed with {} inconsistencies",
            inconsistencies.len()
        );
        Ok(inconsistencies)
    }

    /// Generate transaction counting recommendations
    pub fn generate_recommendations(
        &self,
        status: &TransactionCountIntegrityStatus,
    ) -> Vec<IntegrityRecommendation> {
        let mut recommendations = Vec::new();

        // Total count discrepancy recommendations
        let total_discrepancy = status
            .reported_total_transactions
            .abs_diff(status.actual_rdf_triple_count);

        if total_discrepancy > 0 {
            let severity = if total_discrepancy > 10 {
                RecommendationSeverity::Critical
            } else if total_discrepancy > 5 {
                RecommendationSeverity::Warning
            } else {
                RecommendationSeverity::Info
            };

            recommendations.push(IntegrityRecommendation {
                severity,
                category: "Transaction Counting".to_string(),
                description: format!("Total transaction count discrepancy: {} reported vs {} actual (difference: {})",
                                   status.reported_total_transactions, status.actual_rdf_triple_count, total_discrepancy),
                action_required: "Update transaction counting logic to parse RDF content accurately".to_string(),
                auto_fixable: true,
            });
        }

        // Per-block counting errors recommendations
        let blocks_with_errors: Vec<_> = status
            .per_block_transaction_counts
            .iter()
            .filter(|(_, details)| !details.rdf_parsing_errors.is_empty())
            .collect();

        if !blocks_with_errors.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "Transaction Counting".to_string(),
                description: format!(
                    "Found RDF parsing errors in {} blocks",
                    blocks_with_errors.len()
                ),
                action_required: "Review RDF parsing logic and handle edge cases".to_string(),
                auto_fixable: true,
            });
        }

        // Per-block count discrepancies recommendations
        let blocks_with_discrepancies: Vec<_> = status
            .per_block_transaction_counts
            .iter()
            .filter(|(_, details)| details.reported_count != details.actual_triple_count)
            .collect();

        if !blocks_with_discrepancies.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "Transaction Counting".to_string(),
                description: format!(
                    "Found count discrepancies in {} blocks",
                    blocks_with_discrepancies.len()
                ),
                action_required: "Investigate per-block counting logic and RDF content parsing"
                    .to_string(),
                auto_fixable: true,
            });
        }

        // General counting discrepancies recommendations
        if !status.counting_discrepancies.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Info,
                category: "Transaction Counting".to_string(),
                description: format!(
                    "Found {} general counting discrepancies",
                    status.counting_discrepancies.len()
                ),
                action_required: "Review overall transaction counting methodology".to_string(),
                auto_fixable: false,
            });
        }

        recommendations
    }

    /// Get transaction counting statistics
    pub fn get_counting_statistics(
        &self,
        status: &TransactionCountIntegrityStatus,
    ) -> TransactionCountingStatistics {
        let total_blocks = status.per_block_transaction_counts.len();
        let blocks_with_errors = status
            .per_block_transaction_counts
            .iter()
            .filter(|(_, details)| !details.rdf_parsing_errors.is_empty())
            .count();
        let blocks_with_discrepancies = status
            .per_block_transaction_counts
            .iter()
            .filter(|(_, details)| details.reported_count != details.actual_triple_count)
            .count();

        let total_reported: usize = status
            .per_block_transaction_counts
            .values()
            .map(|details| details.reported_count)
            .sum();
        let total_actual: usize = status
            .per_block_transaction_counts
            .values()
            .map(|details| details.actual_triple_count)
            .sum();

        TransactionCountingStatistics {
            total_blocks,
            blocks_with_errors,
            blocks_with_discrepancies,
            total_reported_transactions: total_reported,
            total_actual_transactions: total_actual,
            overall_accuracy: if total_reported > 0 {
                (total_actual as f64 / total_reported as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Transaction counting statistics
#[derive(Debug, Clone)]
pub struct TransactionCountingStatistics {
    pub total_blocks: usize,
    pub blocks_with_errors: usize,
    pub blocks_with_discrepancies: usize,
    pub total_reported_transactions: usize,
    pub total_actual_transactions: usize,
    pub overall_accuracy: f64,
}

impl Default for TransactionCountValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_transaction_count_validator_creation() {
        let validator = TransactionCountValidator::new();
        assert!(!validator.verbose_logging);
        assert!(validator.parse_rdf_content);
        assert_eq!(validator.max_rdf_parse_size, 1024 * 1024);
    }

    #[test]
    fn test_transaction_count_validator_with_config() {
        let validator = TransactionCountValidator::with_config(true, false, 512 * 1024);
        assert!(validator.verbose_logging);
        assert!(!validator.parse_rdf_content);
        assert_eq!(validator.max_rdf_parse_size, 512 * 1024);
    }

    #[test]
    fn test_parse_rdf_content_basic() {
        let validator = TransactionCountValidator::new();
        let rdf_data = "@prefix ex: <http://example.org/> .\nex:subject ex:predicate \"object\" .";

        let result = validator.parse_rdf_content_for_transactions(rdf_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // One non-empty, non-comment line
    }

    #[test]
    fn test_count_transactions_in_block_basic() {
        let validator = TransactionCountValidator::new();
        let blockchain = Blockchain::new();
        let block = &blockchain.chain[0]; // Genesis block

        let result = validator.count_transactions_in_block(block);
        assert!(result.is_ok());

        let details = result.unwrap();
        assert_eq!(details.block_index, 0);
        assert_eq!(details.reported_count, 1);
        assert!(details.rdf_parsing_errors.is_empty());
    }
}
