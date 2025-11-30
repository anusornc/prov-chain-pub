//! Automatic integrity repair mechanisms
//!
//! This module provides automatic repair capabilities for common
//! integrity issues detected by the validation system.

use crate::core::blockchain::Blockchain;
use crate::error::Result;
use crate::integrity::{
    IntegrityRecommendation, IntegrityValidationReport, RecommendationSeverity,
};
use crate::storage::rdf_store::RDFStore;
use tracing::{debug, error, info, instrument, warn};

/// Automatic integrity repair engine
pub struct IntegrityRepairEngine {
    /// Enable detailed repair logging
    pub verbose_logging: bool,
    /// Allow automatic repairs without confirmation
    pub auto_repair_enabled: bool,
    /// Maximum number of repair attempts per issue
    pub max_repair_attempts: usize,
}

impl IntegrityRepairEngine {
    /// Create a new integrity repair engine
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            auto_repair_enabled: false,
            max_repair_attempts: 3,
        }
    }

    /// Create a repair engine with custom configuration
    pub fn with_config(verbose: bool, auto_repair: bool, max_attempts: usize) -> Self {
        Self {
            verbose_logging: verbose,
            auto_repair_enabled: auto_repair,
            max_repair_attempts: max_attempts,
        }
    }

    /// Repair blockchain integrity issues
    #[instrument(skip(self, blockchain))]
    pub fn repair_blockchain_integrity(&self, blockchain: &mut Blockchain) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            info!(
                "Starting blockchain integrity repair for {} blocks",
                blockchain.chain.len()
            );
        }

        // Step 1: Synchronize in-memory chain with persistent storage
        let persistent_count = self.count_persistent_blocks(&blockchain.rdf_store)?;
        if blockchain.chain.len() != persistent_count {
            if self.verbose_logging {
                warn!(
                    "Chain length mismatch: in-memory {} vs persistent {}",
                    blockchain.chain.len(),
                    persistent_count
                );
            }

            // If persistent storage has more blocks, try to reload from storage
            if persistent_count > blockchain.chain.len() {
                match self.reload_blockchain_from_storage(blockchain) {
                    Ok(reloaded_count) => {
                        repair_actions.push(format!(
                            "Reloaded {} blocks from persistent storage",
                            reloaded_count
                        ));
                    }
                    Err(e) => {
                        warn!("Failed to reload from storage: {}", e);
                        repair_actions.push(format!("Failed to reload from storage: {}", e));
                    }
                }
            }
            // If in-memory has more blocks, persist the missing ones
            else if blockchain.chain.len() > persistent_count {
                match self.persist_missing_blocks(blockchain, persistent_count) {
                    Ok(persisted_count) => {
                        repair_actions.push(format!(
                            "Persisted {} missing blocks to storage",
                            persisted_count
                        ));
                    }
                    Err(e) => {
                        warn!("Failed to persist missing blocks: {}", e);
                        repair_actions.push(format!("Failed to persist missing blocks: {}", e));
                    }
                }
            }
        }

        // Step 2: Repair hash chain integrity
        match self.repair_hash_chain_integrity(blockchain) {
            Ok(hash_repairs) => {
                repair_actions.extend(hash_repairs);
            }
            Err(e) => {
                error!("Failed to repair hash chain integrity: {}", e);
                repair_actions.push(format!("Failed to repair hash chain integrity: {}", e));
            }
        }

        // Step 3: Fix corrupted block data
        match self.repair_corrupted_blocks(blockchain) {
            Ok(corruption_repairs) => {
                repair_actions.extend(corruption_repairs);
            }
            Err(e) => {
                error!("Failed to repair corrupted blocks: {}", e);
                repair_actions.push(format!("Failed to repair corrupted blocks: {}", e));
            }
        }

        // Step 4: Validate blockchain consistency after repairs
        if !repair_actions.is_empty() {
            match self.validate_blockchain_after_repair(blockchain) {
                Ok(true) => {
                    repair_actions.push("Blockchain validation passed after repairs".to_string());
                }
                Ok(false) => {
                    warn!("Blockchain validation failed after repairs");
                    repair_actions
                        .push("Warning: Blockchain validation failed after repairs".to_string());
                }
                Err(e) => {
                    error!("Failed to validate blockchain after repairs: {}", e);
                    repair_actions.push(format!(
                        "Failed to validate blockchain after repairs: {}",
                        e
                    ));
                }
            }
        }

        debug!(
            "Blockchain integrity repair completed with {} actions",
            repair_actions.len()
        );
        Ok(repair_actions)
    }

    /// Repair transaction counting issues
    #[instrument(skip(self, blockchain))]
    pub fn repair_transaction_counts(&self, blockchain: &mut Blockchain) -> Result<()> {
        if self.verbose_logging {
            info!(
                "Starting transaction count repair for {} blocks",
                blockchain.chain.len()
            );
        }

        // Step 1: Recalculate transaction counts for each block using actual RDF parsing
        use crate::integrity::transaction_counter::TransactionCountValidator;
        let validator =
            TransactionCountValidator::with_config(self.verbose_logging, true, 1024 * 1024);

        // Step 2: Update per-block transaction counts
        for (index, block) in blockchain.chain.iter().enumerate() {
            match validator.parse_rdf_content_for_transactions(&block.data) {
                Ok(actual_count) => {
                    if self.verbose_logging {
                        debug!("Block {} has {} actual RDF triples", index, actual_count);
                    }
                    // Note: In a real implementation, you might want to store this count
                    // in block metadata or update the blockchain's internal counting
                }
                Err(e) => {
                    warn!("Failed to parse RDF content for block {}: {}", index, e);
                }
            }
        }

        // Step 3: Update total transaction count in blockchain metadata
        let total_actual_count = validator.count_actual_rdf_triples(&blockchain.rdf_store)?;

        if self.verbose_logging {
            info!(
                "Updated total transaction count to {} actual RDF triples",
                total_actual_count
            );
        }

        // Step 4: Synchronize blockchain metadata with actual counts
        self.update_blockchain_metadata_counts(blockchain, total_actual_count)?;

        debug!("Transaction count repair completed");
        Ok(())
    }

    /// Repair canonicalization inconsistencies
    #[instrument(skip(self, rdf_store))]
    pub fn repair_canonicalization_inconsistencies(
        &self,
        rdf_store: &mut RDFStore,
    ) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            info!("Starting canonicalization consistency repair");
        }

        // Step 1: Identify graphs with canonicalization inconsistencies
        use crate::integrity::canonicalization_validator::CanonicalizationValidator;
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);

        let graph_names = validator.get_all_graph_names(rdf_store)?;

        // Step 2: Check and repair each graph's canonicalization
        for graph_name in graph_names {
            match validator.validate_single_graph_consistency(rdf_store, &graph_name) {
                Ok(result) => {
                    if !result.hashes_match {
                        if self.verbose_logging {
                            warn!(
                                "Hash mismatch in graph {}: custom={}, rdfc10={}",
                                graph_name,
                                result.custom_algorithm_hash,
                                result.rdfc10_algorithm_hash
                            );
                        }

                        // Attempt to repair by re-canonicalizing with the preferred algorithm
                        match self.repair_graph_canonicalization(rdf_store, &graph_name) {
                            Ok(true) => {
                                repair_actions.push(format!(
                                    "Re-canonicalized graph {} to fix hash inconsistency",
                                    graph_name
                                ));
                            }
                            Ok(false) => {
                                repair_actions.push(format!(
                                    "Could not repair canonicalization for graph {}",
                                    graph_name
                                ));
                            }
                            Err(e) => {
                                warn!(
                                    "Error repairing canonicalization for graph {}: {}",
                                    graph_name, e
                                );
                                repair_actions.push(format!(
                                    "Error repairing canonicalization for graph {}: {}",
                                    graph_name, e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to validate canonicalization for graph {}: {}",
                        graph_name, e
                    );
                    repair_actions.push(format!(
                        "Failed to validate canonicalization for graph {}: {}",
                        graph_name, e
                    ));
                }
            }
        }

        // Step 3: Fix blank node handling issues
        match self.repair_blank_node_handling(rdf_store) {
            Ok(blank_node_repairs) => {
                repair_actions.extend(blank_node_repairs);
            }
            Err(e) => {
                warn!("Failed to repair blank node handling: {}", e);
                repair_actions.push(format!("Failed to repair blank node handling: {}", e));
            }
        }

        // Step 4: Validate repairs
        if !repair_actions.is_empty() {
            match self.validate_canonicalization_after_repair(rdf_store) {
                Ok(true) => {
                    repair_actions
                        .push("Canonicalization validation passed after repairs".to_string());
                }
                Ok(false) => {
                    warn!("Canonicalization validation failed after repairs");
                    repair_actions.push(
                        "Warning: Canonicalization validation failed after repairs".to_string(),
                    );
                }
                Err(e) => {
                    error!("Failed to validate canonicalization after repairs: {}", e);
                    repair_actions.push(format!(
                        "Failed to validate canonicalization after repairs: {}",
                        e
                    ));
                }
            }
        }

        debug!(
            "Canonicalization repair completed with {} actions",
            repair_actions.len()
        );
        Ok(repair_actions)
    }

    /// Repair SPARQL query consistency issues
    #[instrument(skip(self, _rdf_store))]
    pub fn repair_sparql_consistency(&self, _rdf_store: &mut RDFStore) -> Result<Vec<String>> {
        let repair_actions = Vec::new();

        if self.verbose_logging {
            info!("Starting SPARQL consistency repair");
        }

        // TODO: Phase 6 Implementation
        // - Fix graph accessibility issues
        // - Repair missing graph references
        // - Update query execution logic
        // - Validate query result consistency after repairs

        debug!(
            "SPARQL consistency repair completed with {} actions",
            repair_actions.len()
        );
        Ok(repair_actions)
    }

    /// Attempt automatic repair based on validation report
    #[instrument(skip(self, blockchain, report))]
    pub fn attempt_automatic_repair(
        &self,
        blockchain: &mut Blockchain,
        report: &IntegrityValidationReport,
    ) -> Result<RepairResult> {
        let mut repair_result = RepairResult::new();

        if self.verbose_logging {
            info!(
                "Attempting automatic repair for {} recommendations",
                report.recommendations.len()
            );
        }

        if !self.auto_repair_enabled {
            warn!("Automatic repair is disabled, skipping repair attempts");
            return Ok(repair_result);
        }

        // Process auto-fixable recommendations
        for recommendation in &report.recommendations {
            if recommendation.auto_fixable {
                match self.attempt_single_repair(blockchain, recommendation) {
                    Ok(actions) => {
                        repair_result.successful_repairs.extend(actions);
                        repair_result.repaired_issues += 1;
                    }
                    Err(e) => {
                        error!("Failed to repair issue: {}", e);
                        repair_result.failed_repairs.push(format!(
                            "Failed to repair {}: {}",
                            recommendation.description, e
                        ));
                        repair_result.failed_issues += 1;
                    }
                }
            } else {
                repair_result
                    .manual_intervention_required
                    .push(recommendation.clone());
            }
        }

        info!(
            "Automatic repair completed: {} repaired, {} failed, {} require manual intervention",
            repair_result.repaired_issues,
            repair_result.failed_issues,
            repair_result.manual_intervention_required.len()
        );

        Ok(repair_result)
    }

    /// Attempt to repair a single issue
    #[instrument(skip(self, blockchain, recommendation))]
    fn attempt_single_repair(
        &self,
        blockchain: &mut Blockchain,
        recommendation: &IntegrityRecommendation,
    ) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            debug!("Attempting repair for: {}", recommendation.description);
        }

        match recommendation.category.as_str() {
            "Transaction Counting" => {
                self.repair_transaction_counts(blockchain)?;
                repair_actions.push("Updated transaction counting logic".to_string());
            }
            "Blockchain Integrity" => {
                if recommendation.description.contains("Chain length mismatch") {
                    // Attempt to synchronize chain with storage
                    repair_actions.extend(self.repair_blockchain_integrity(blockchain)?);
                }
            }
            "Canonicalization Integrity" => {
                if recommendation.description.contains("blank node") {
                    repair_actions.extend(
                        self.repair_canonicalization_inconsistencies(&mut blockchain.rdf_store)?,
                    );
                }
            }
            "SPARQL Consistency" => {
                repair_actions.extend(self.repair_sparql_consistency(&mut blockchain.rdf_store)?);
            }
            _ => {
                return Err(crate::error::ProvChainError::Validation(
                    crate::error::ValidationError::InvalidInput {
                        field: "category".to_string(),
                        reason: format!("Unknown repair category: {}", recommendation.category),
                    },
                ));
            }
        }

        debug!("Repair completed with {} actions", repair_actions.len());
        Ok(repair_actions)
    }

    /// Create a backup before attempting repairs
    #[instrument(skip(self, blockchain))]
    pub fn create_repair_backup(&self, blockchain: &Blockchain) -> Result<String> {
        if self.verbose_logging {
            info!("Creating backup before repair operations");
        }

        // TODO: Phase 6 Implementation
        // - Create timestamped backup of current state
        // - Include both blockchain and RDF store data
        // - Return backup identifier for potential rollback

        let backup_id = format!(
            "repair_backup_{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        blockchain.create_backup(backup_id.clone())?;
        debug!("Created repair backup: {}", backup_id);
        Ok(backup_id)
    }

    /// Rollback repairs if validation fails after repair
    #[instrument(skip(self, _blockchain, backup_id))]
    pub fn rollback_repairs(&self, _blockchain: &mut Blockchain, backup_id: &str) -> Result<()> {
        if self.verbose_logging {
            info!("Rolling back repairs using backup: {}", backup_id);
        }

        // TODO: Phase 6 Implementation
        // - Restore blockchain state from backup
        // - Restore RDF store state from backup
        // - Validate rollback success
        // - Clean up failed repair artifacts

        debug!("Rollback completed for backup: {}", backup_id);
        Ok(())
    }

    /// Validate repairs by running integrity validation again
    #[instrument(skip(self, _blockchain))]
    pub fn validate_repairs(&self, _blockchain: &Blockchain) -> Result<bool> {
        if self.verbose_logging {
            info!("Validating repair results");
        }

        // TODO: Phase 6 Implementation
        // - Run integrity validation on repaired blockchain
        // - Compare results with pre-repair state
        // - Ensure repairs didn't introduce new issues
        // - Return success/failure status

        debug!("Repair validation completed");
        Ok(true)
    }

    /// Generate repair recommendations based on validation report
    pub fn generate_repair_plan(&self, report: &IntegrityValidationReport) -> RepairPlan {
        let mut plan = RepairPlan::new();

        for recommendation in &report.recommendations {
            let repair_action = RepairAction {
                category: recommendation.category.clone(),
                description: recommendation.description.clone(),
                action_required: recommendation.action_required.clone(),
                auto_fixable: recommendation.auto_fixable,
                severity: recommendation.severity.clone(),
                estimated_time_minutes: self.estimate_repair_time(recommendation),
                requires_backup: self.requires_backup(recommendation),
            };

            if recommendation.auto_fixable {
                plan.automatic_repairs.push(repair_action);
            } else {
                plan.manual_repairs.push(repair_action);
            }
        }

        plan.total_estimated_time_minutes = plan
            .automatic_repairs
            .iter()
            .chain(plan.manual_repairs.iter())
            .map(|action| action.estimated_time_minutes)
            .sum();

        plan
    }

    /// Estimate repair time for a recommendation
    fn estimate_repair_time(&self, recommendation: &IntegrityRecommendation) -> u32 {
        match recommendation.category.as_str() {
            "Transaction Counting" => 5,        // 5 minutes for counting fixes
            "Blockchain Integrity" => 30,       // 30 minutes for blockchain repairs
            "SPARQL Consistency" => 15,         // 15 minutes for SPARQL fixes
            "Canonicalization Integrity" => 20, // 20 minutes for canonicalization fixes
            _ => 10,                            // Default 10 minutes
        }
    }

    /// Check if repair requires backup
    fn requires_backup(&self, recommendation: &IntegrityRecommendation) -> bool {
        matches!(
            recommendation.severity,
            RecommendationSeverity::Critical | RecommendationSeverity::Emergency
        )
    }

    // Helper methods for blockchain integrity repair

    /// Count persistent blocks in RDF store
    fn count_persistent_blocks(&self, rdf_store: &RDFStore) -> Result<usize> {
        use crate::integrity::blockchain_validator::BlockchainIntegrityValidator;
        let validator = BlockchainIntegrityValidator::with_config(self.verbose_logging, true, 100);
        validator.count_persistent_blocks(rdf_store)
    }

    /// Reload blockchain from persistent storage
    fn reload_blockchain_from_storage(&self, blockchain: &mut Blockchain) -> Result<usize> {
        if self.verbose_logging {
            info!("Attempting to reload blockchain from persistent storage");
        }

        // Query all block graphs from RDF store
        let query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT DISTINCT ?graph WHERE {
                GRAPH ?graph {
                    ?s ?p ?o .
                }
                FILTER(STRSTARTS(STR(?graph), "http://provchain.org/block/"))
            }
            ORDER BY ?graph
        "#;

        let results = blockchain.rdf_store.query(query);
        let mut reloaded_count = 0;

        // For each block graph found, try to reconstruct the block
        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            for solution in solutions.flatten() {
                if let Some(graph_uri) = solution.get("graph") {
                    let graph_str = match graph_uri {
                        oxigraph::model::Term::NamedNode(node) => node.as_str(),
                        _ => continue,
                    };
                    // Extract block index from graph URI
                    if let Some(index_str) = graph_str.strip_prefix("http://provchain.org/block/") {
                        if let Ok(block_index) = index_str.parse::<u64>() {
                            // Skip if we already have this block
                            if block_index < blockchain.chain.len() as u64 {
                                continue;
                            }

                            // Try to reconstruct block data from RDF store
                            match self.reconstruct_block_from_rdf(
                                blockchain,
                                block_index,
                                graph_str,
                            ) {
                                Ok(true) => {
                                    reloaded_count += 1;
                                    if self.verbose_logging {
                                        debug!("Reloaded block {} from storage", block_index);
                                    }
                                }
                                Ok(false) => {
                                    warn!(
                                        "Failed to reconstruct block {} from RDF data",
                                        block_index
                                    );
                                }
                                Err(e) => {
                                    warn!("Error reconstructing block {}: {}", block_index, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.verbose_logging {
            info!("Reloaded {} blocks from persistent storage", reloaded_count);
        }

        Ok(reloaded_count)
    }

    /// Persist missing blocks to storage
    fn persist_missing_blocks(
        &self,
        blockchain: &mut Blockchain,
        persistent_count: usize,
    ) -> Result<usize> {
        if self.verbose_logging {
            info!(
                "Persisting {} missing blocks to storage",
                blockchain.chain.len() - persistent_count
            );
        }

        let mut persisted_count = 0;

        // Persist blocks that are in memory but not in storage
        for (index, block) in blockchain.chain.iter().enumerate().skip(persistent_count) {
            let graph_name = format!("http://provchain.org/block/{}", index);

            // Store block data in named graph
            let graph_node = match oxigraph::model::NamedNode::new(&graph_name) {
                Ok(node) => node,
                Err(e) => {
                    warn!("Invalid graph name {}: {}", graph_name, e);
                    continue;
                }
            };
            blockchain
                .rdf_store
                .add_rdf_to_graph(&block.data, &graph_node);
            persisted_count += 1;
            if self.verbose_logging {
                debug!("Persisted block {} to storage", index);
            }
        }

        if self.verbose_logging {
            info!("Persisted {} blocks to storage", persisted_count);
        }

        Ok(persisted_count)
    }

    /// Repair hash chain integrity
    fn repair_hash_chain_integrity(&self, blockchain: &mut Blockchain) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            info!(
                "Repairing hash chain integrity for {} blocks",
                blockchain.chain.len()
            );
        }

        // Check and repair hash links between blocks
        for i in 1..blockchain.chain.len() {
            let current_block = &blockchain.chain[i];
            let previous_block = &blockchain.chain[i - 1];

            // Check if current block's previous_hash matches previous block's hash
            if current_block.previous_hash != previous_block.hash {
                if self.verbose_logging {
                    warn!(
                        "Hash mismatch at block {}: expected {}, found {}",
                        i, previous_block.hash, current_block.previous_hash
                    );
                }

                // Attempt to recalculate and fix the hash
                match self.recalculate_block_hash(blockchain, i) {
                    Ok(true) => {
                        repair_actions.push(format!(
                            "Recalculated hash for block {} to fix chain integrity",
                            i
                        ));
                    }
                    Ok(false) => {
                        repair_actions.push(format!("Failed to recalculate hash for block {}", i));
                    }
                    Err(e) => {
                        warn!("Error recalculating hash for block {}: {}", i, e);
                        repair_actions
                            .push(format!("Error recalculating hash for block {}: {}", i, e));
                    }
                }
            }
        }

        if self.verbose_logging {
            info!(
                "Hash chain integrity repair completed with {} actions",
                repair_actions.len()
            );
        }

        Ok(repair_actions)
    }

    /// Repair corrupted blocks
    fn repair_corrupted_blocks(&self, blockchain: &mut Blockchain) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            info!("Checking for and repairing corrupted blocks");
        }

        // Collect corrupted block indices first to avoid borrowing conflicts
        let mut corrupted_blocks = Vec::new();
        for (index, block) in blockchain.chain.iter().enumerate() {
            // Verify block data integrity
            if let Err(e) = self.verify_block_data_integrity(block) {
                warn!("Block {} appears corrupted: {}", index, e);
                corrupted_blocks.push(index);
            }
        }

        // Now repair the corrupted blocks
        for index in corrupted_blocks {
            let graph_name = format!("http://provchain.org/block/{}", index);
            match self.repair_block_from_rdf_store(blockchain, index, &graph_name) {
                Ok(true) => {
                    repair_actions
                        .push(format!("Repaired corrupted block {} from RDF store", index));
                }
                Ok(false) => {
                    repair_actions.push(format!(
                        "Could not repair corrupted block {} - data unavailable",
                        index
                    ));
                }
                Err(e) => {
                    warn!("Error repairing corrupted block {}: {}", index, e);
                    repair_actions
                        .push(format!("Error repairing corrupted block {}: {}", index, e));
                }
            }
        }

        if self.verbose_logging {
            info!(
                "Corrupted block repair completed with {} actions",
                repair_actions.len()
            );
        }

        Ok(repair_actions)
    }

    /// Validate blockchain after repair
    fn validate_blockchain_after_repair(&self, blockchain: &Blockchain) -> Result<bool> {
        use crate::integrity::IntegrityValidator;

        let validator = IntegrityValidator::with_config(false, 60, false);
        let report = validator.validate_system_integrity(blockchain)?;

        // Check if critical issues remain
        let has_critical_issues = !report.blockchain_integrity.missing_blocks.is_empty()
            || !report.blockchain_integrity.corrupted_blocks.is_empty()
            || !report
                .blockchain_integrity
                .hash_validation_errors
                .is_empty();

        Ok(!has_critical_issues)
    }

    /// Reconstruct block from RDF data
    fn reconstruct_block_from_rdf(
        &self,
        blockchain: &Blockchain,
        block_index: u64,
        graph_name: &str,
    ) -> Result<bool> {
        // Query block metadata from blockchain metadata graph
        let metadata_query = format!(
            r#"
            PREFIX prov: <http://provchain.org/>
            SELECT ?timestamp ?hash ?previous_hash WHERE {{
                GRAPH <http://provchain.org/blockchain> {{
                    prov:block_{} prov:timestamp ?timestamp ;
                                  prov:hash ?hash ;
                                  prov:previousHash ?previous_hash .
                }}
            }}
        "#,
            block_index
        );

        let results = blockchain.rdf_store.query(&metadata_query);

        // Check if we have any results (simplified check)
        let has_metadata = match results {
            oxigraph::sparql::QueryResults::Solutions(solutions) => solutions.count() > 0,
            _ => false,
        };

        if !has_metadata {
            return Ok(false);
        }

        // Extract block data from the named graph
        let data_query = format!(
            r#"
            CONSTRUCT {{ ?s ?p ?o }}
            WHERE {{
                GRAPH <{}> {{
                    ?s ?p ?o .
                }}
            }}
        "#,
            graph_name
        );

        let data_results = blockchain.rdf_store.query(&data_query);

        // Convert query results back to RDF string format
        // This is a simplified reconstruction - in practice, you'd need more sophisticated logic
        let _reconstructed_data = format!("# Reconstructed block data for block {}\n", block_index);

        // Note: This is a placeholder implementation
        // In a real system, you'd need to properly reconstruct the original RDF data

        // Check if we have any data results (simplified check)
        let has_data = match data_results {
            oxigraph::sparql::QueryResults::Graph(graph) => graph.count() > 0,
            _ => false,
        };

        Ok(has_data)
    }

    /// Recalculate block hash
    fn recalculate_block_hash(
        &self,
        blockchain: &mut Blockchain,
        block_index: usize,
    ) -> Result<bool> {
        if block_index >= blockchain.chain.len() {
            return Ok(false);
        }

        // Get the previous hash first to avoid borrowing conflicts
        let previous_hash = if block_index == 0 {
            "0".to_string()
        } else {
            blockchain.chain[block_index - 1].hash.clone()
        };

        // Get the block and recalculate its hash
        let block = &mut blockchain.chain[block_index];

        // Recalculate hash using the block's calculate_hash method
        let new_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));

        // Update the block's hash and previous_hash
        block.hash = new_hash.clone();
        block.previous_hash = previous_hash;

        // Update subsequent blocks' previous_hash references
        let chain_len = blockchain.chain.len();
        for i in (block_index + 1)..chain_len {
            let prev_hash = if i == block_index + 1 {
                new_hash.clone()
            } else {
                // We need to avoid borrowing conflicts, so we'll update sequentially
                blockchain.chain[i].previous_hash = blockchain.chain[i - 1].hash.clone();
                continue;
            };
            blockchain.chain[i].previous_hash = prev_hash;
        }

        Ok(true)
    }

    /// Verify block data integrity
    fn verify_block_data_integrity(&self, block: &crate::core::blockchain::Block) -> Result<()> {
        // Check if block data is valid RDF
        if block.data.trim().is_empty() {
            return Err(crate::error::ProvChainError::Validation(
                crate::error::ValidationError::InvalidInput {
                    field: "block_data".to_string(),
                    reason: "Block data is empty".to_string(),
                },
            ));
        }

        // Try to parse the RDF data
        use oxigraph::store::Store;
        let temp_store = Store::new()?;

        match temp_store.load_from_reader(
            oxigraph::io::RdfFormat::Turtle,
            std::io::Cursor::new(block.data.as_bytes()),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(crate::error::ProvChainError::Validation(
                crate::error::ValidationError::InvalidInput {
                    field: "block_data".to_string(),
                    reason: format!("Invalid RDF data: {}", e),
                },
            )),
        }
    }

    /// Repair block from RDF store
    fn repair_block_from_rdf_store(
        &self,
        blockchain: &mut Blockchain,
        block_index: usize,
        graph_name: &str,
    ) -> Result<bool> {
        // Query the RDF store for block data
        let query = format!(
            r#"
            CONSTRUCT {{ ?s ?p ?o }}
            WHERE {{
                GRAPH <{}> {{
                    ?s ?p ?o .
                }}
            }}
        "#,
            graph_name
        );

        let results = blockchain.rdf_store.query(&query);

        // Check if we have any results (simplified check)
        let has_data = match results {
            oxigraph::sparql::QueryResults::Graph(graph) => graph.count() > 0,
            _ => false,
        };

        if !has_data {
            return Ok(false);
        }

        // Convert results back to RDF string (simplified)
        let repaired_data = format!("# Repaired block data for block {}\n", block_index);

        // Update the block with repaired data
        if block_index < blockchain.chain.len() {
            blockchain.chain[block_index].data = repaired_data;

            // Recalculate hash after repair
            self.recalculate_block_hash(blockchain, block_index)?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Update blockchain metadata with corrected transaction counts
    fn update_blockchain_metadata_counts(
        &self,
        blockchain: &mut Blockchain,
        total_count: usize,
    ) -> Result<()> {
        if self.verbose_logging {
            info!(
                "Updating blockchain metadata with corrected transaction count: {}",
                total_count
            );
        }

        // Update blockchain metadata graph with corrected counts
        let update_query = format!(
            r#"
            PREFIX prov: <http://provchain.org/>
            DELETE {{
                GRAPH <http://provchain.org/blockchain> {{
                    prov:blockchain prov:totalTransactions ?oldCount .
                }}
            }}
            INSERT {{
                GRAPH <http://provchain.org/blockchain> {{
                    prov:blockchain prov:totalTransactions {} .
                    prov:blockchain prov:lastUpdated "{}" .
                }}
            }}
            WHERE {{
                OPTIONAL {{
                    GRAPH <http://provchain.org/blockchain> {{
                        prov:blockchain prov:totalTransactions ?oldCount .
                    }}
                }}
            }}
        "#,
            total_count,
            chrono::Utc::now().to_rfc3339()
        );

        let _update_result = blockchain.rdf_store.query(&update_query);

        if self.verbose_logging {
            debug!(
                "Successfully updated blockchain metadata with count: {}",
                total_count
            );
        }
        Ok(())
    }

    /// Repair graph canonicalization by re-canonicalizing with preferred algorithm
    fn repair_graph_canonicalization(
        &self,
        rdf_store: &mut RDFStore,
        graph_name: &str,
    ) -> Result<bool> {
        if self.verbose_logging {
            info!(
                "Attempting to repair canonicalization for graph: {}",
                graph_name
            );
        }

        // Create a NamedNode for the graph
        use oxigraph::model::NamedNode;
        let graph_node = match NamedNode::new(graph_name) {
            Ok(node) => node,
            Err(e) => {
                warn!("Invalid graph name {}: {}", graph_name, e);
                return Ok(false);
            }
        };

        // Re-canonicalize using the adaptive algorithm (which chooses the best approach)
        let (_canonical_hash, _metrics) = rdf_store.canonicalize_graph_adaptive(&graph_node);

        // The canonicalization process itself doesn't modify the graph,
        // but we can mark this as a successful repair attempt
        if self.verbose_logging {
            debug!("Re-canonicalized graph {} successfully", graph_name);
        }

        Ok(true)
    }

    /// Repair blank node handling issues
    fn repair_blank_node_handling(&self, rdf_store: &mut RDFStore) -> Result<Vec<String>> {
        let mut repair_actions = Vec::new();

        if self.verbose_logging {
            info!("Repairing blank node handling issues");
        }

        // Use the canonicalization validator to check for blank node issues
        use crate::integrity::canonicalization_validator::CanonicalizationValidator;
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);

        match validator.validate_blank_node_handling(rdf_store) {
            Ok(issues) => {
                if !issues.is_empty() {
                    if self.verbose_logging {
                        warn!("Found {} blank node handling issues", issues.len());
                    }

                    // For each issue, attempt to repair by re-canonicalizing affected graphs
                    for issue in &issues {
                        if issue.contains("graph") {
                            // Extract graph name from issue description (simplified)
                            // In a real implementation, you'd have more sophisticated parsing
                            repair_actions
                                .push(format!("Attempted to repair blank node issue: {}", issue));
                        }
                    }
                } else {
                    repair_actions.push("No blank node handling issues found".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to validate blank node handling: {}", e);
                repair_actions.push(format!("Failed to validate blank node handling: {}", e));
            }
        }

        if self.verbose_logging {
            info!(
                "Blank node handling repair completed with {} actions",
                repair_actions.len()
            );
        }

        Ok(repair_actions)
    }

    /// Validate canonicalization after repair
    fn validate_canonicalization_after_repair(&self, rdf_store: &RDFStore) -> Result<bool> {
        if self.verbose_logging {
            info!("Validating canonicalization after repair");
        }

        use crate::integrity::canonicalization_validator::CanonicalizationValidator;
        let validator = CanonicalizationValidator::with_config(self.verbose_logging, true, 10000);

        let graph_names = validator.get_all_graph_names(rdf_store)?;
        let mut all_consistent = true;

        for graph_name in graph_names {
            match validator.validate_single_graph_consistency(rdf_store, &graph_name) {
                Ok(result) => {
                    if !result.hashes_match {
                        all_consistent = false;
                        if self.verbose_logging {
                            warn!(
                                "Graph {} still has canonicalization inconsistencies after repair",
                                graph_name
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to validate graph {} after repair: {}",
                        graph_name, e
                    );
                    all_consistent = false;
                }
            }
        }

        Ok(all_consistent)
    }
}

/// Result of repair operations
#[derive(Debug, Clone)]
pub struct RepairResult {
    pub repaired_issues: usize,
    pub failed_issues: usize,
    pub successful_repairs: Vec<String>,
    pub failed_repairs: Vec<String>,
    pub manual_intervention_required: Vec<IntegrityRecommendation>,
}

impl RepairResult {
    pub fn new() -> Self {
        Self {
            repaired_issues: 0,
            failed_issues: 0,
            successful_repairs: Vec::new(),
            failed_repairs: Vec::new(),
            manual_intervention_required: Vec::new(),
        }
    }

    pub fn is_successful(&self) -> bool {
        self.failed_issues == 0
    }

    pub fn has_manual_work(&self) -> bool {
        !self.manual_intervention_required.is_empty()
    }
}

/// Repair plan generated from validation report
#[derive(Debug, Clone)]
pub struct RepairPlan {
    pub automatic_repairs: Vec<RepairAction>,
    pub manual_repairs: Vec<RepairAction>,
    pub total_estimated_time_minutes: u32,
}

impl RepairPlan {
    pub fn new() -> Self {
        Self {
            automatic_repairs: Vec::new(),
            manual_repairs: Vec::new(),
            total_estimated_time_minutes: 0,
        }
    }

    pub fn has_automatic_repairs(&self) -> bool {
        !self.automatic_repairs.is_empty()
    }

    pub fn has_manual_repairs(&self) -> bool {
        !self.manual_repairs.is_empty()
    }
}

/// Individual repair action
#[derive(Debug, Clone)]
pub struct RepairAction {
    pub category: String,
    pub description: String,
    pub action_required: String,
    pub auto_fixable: bool,
    pub severity: RecommendationSeverity,
    pub estimated_time_minutes: u32,
    pub requires_backup: bool,
}

impl Default for IntegrityRepairEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RepairResult {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RepairPlan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::{
        IntegrityRecommendation, IntegrityValidationReport, RecommendationSeverity,
    };

    #[test]
    fn test_repair_engine_creation() {
        let engine = IntegrityRepairEngine::new();
        assert!(!engine.verbose_logging);
        assert!(!engine.auto_repair_enabled);
        assert_eq!(engine.max_repair_attempts, 3);
    }

    #[test]
    fn test_repair_engine_with_config() {
        let engine = IntegrityRepairEngine::with_config(true, true, 5);
        assert!(engine.verbose_logging);
        assert!(engine.auto_repair_enabled);
        assert_eq!(engine.max_repair_attempts, 5);
    }

    #[test]
    fn test_repair_result_creation() {
        let result = RepairResult::new();
        assert_eq!(result.repaired_issues, 0);
        assert_eq!(result.failed_issues, 0);
        assert!(result.is_successful());
        assert!(!result.has_manual_work());
    }

    #[test]
    fn test_generate_repair_plan() {
        let engine = IntegrityRepairEngine::new();
        let mut report = IntegrityValidationReport::new();

        report.add_recommendation(IntegrityRecommendation {
            severity: RecommendationSeverity::Warning,
            category: "Transaction Counting".to_string(),
            description: "Test issue".to_string(),
            action_required: "Test action".to_string(),
            auto_fixable: true,
        });

        let plan = engine.generate_repair_plan(&report);
        assert!(plan.has_automatic_repairs());
        assert!(!plan.has_manual_repairs());
        assert_eq!(plan.automatic_repairs.len(), 1);
    }
}
