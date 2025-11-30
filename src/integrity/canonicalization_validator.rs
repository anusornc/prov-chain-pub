//! RDF canonicalization integrity validation
//!
//! This module validates RDF canonicalization integrity by comparing
//! different canonicalization algorithms and ensuring consistent
//! hash generation for equivalent RDF graphs.

use crate::error::Result;
use crate::integrity::{
    CanonicalizationConsistencyResult, CanonicalizationIntegrityStatus, IntegrityRecommendation,
    RecommendationSeverity,
};
use crate::storage::rdf_store::RDFStore;
use oxigraph::model::NamedNode;
use tracing::{debug, error, info, instrument, warn};

/// RDF canonicalization integrity validator
pub struct CanonicalizationValidator {
    /// Enable detailed validation logging
    pub verbose_logging: bool,
    /// Test both custom and RDFC-1.0 algorithms
    pub test_both_algorithms: bool,
    /// Maximum graph size to test (in triples)
    pub max_graph_size: usize,
}

impl CanonicalizationValidator {
    /// Create a new canonicalization validator
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            test_both_algorithms: true,
            max_graph_size: 10000,
        }
    }

    /// Create a validator with custom configuration
    pub fn with_config(verbose: bool, test_both: bool, max_size: usize) -> Self {
        Self {
            verbose_logging: verbose,
            test_both_algorithms: test_both,
            max_graph_size: max_size,
        }
    }

    /// Validate canonicalization algorithm consistency
    #[instrument(skip(self, rdf_store, graph_names))]
    pub fn validate_algorithm_consistency(
        &self,
        rdf_store: &RDFStore,
        graph_names: &[String],
    ) -> Result<Vec<CanonicalizationConsistencyResult>> {
        let mut results = Vec::new();

        if self.verbose_logging {
            info!(
                "Validating canonicalization consistency for {} graphs",
                graph_names.len()
            );
        }

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

            match self.validate_single_graph_consistency(rdf_store, graph_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!(
                        "Canonicalization validation failed for graph '{}': {}",
                        graph_name, e
                    );
                    results.push(CanonicalizationConsistencyResult {
                        graph_name: graph_name.clone(),
                        custom_algorithm_hash: String::new(),
                        rdfc10_algorithm_hash: String::new(),
                        hashes_match: false,
                        complexity: crate::storage::rdf_store::GraphComplexity::Simple,
                    });
                }
            }
        }

        debug!(
            "Canonicalization consistency validation completed for {} graphs",
            results.len()
        );
        Ok(results)
    }

    /// Validate canonicalization consistency for a single graph
    #[instrument(skip(self, rdf_store, graph_name))]
    pub fn validate_single_graph_consistency(
        &self,
        rdf_store: &RDFStore,
        graph_name: &str,
    ) -> Result<CanonicalizationConsistencyResult> {
        if self.verbose_logging {
            debug!(
                "Validating canonicalization consistency for graph: {}",
                graph_name
            );
        }

        // Create NamedNode from graph_name
        let graph_node = NamedNode::new(graph_name).map_err(|e| {
            crate::error::ProvChainError::Validation(crate::error::ValidationError::InvalidInput {
                field: "graph_name".to_string(),
                reason: format!("Invalid graph name '{}': {}", graph_name, e),
            })
        })?;

        // Get graph complexity analysis
        let complexity = rdf_store.analyze_graph_complexity(&graph_node);

        if self.verbose_logging {
            debug!("Graph '{}' complexity: {:?}", graph_name, complexity);
        }

        // Run custom canonicalization algorithm
        let custom_hash = rdf_store.canonicalize_graph(&graph_node);

        // Run RDFC-1.0 canonicalization algorithm
        let rdfc10_hash = rdf_store.canonicalize_graph_rdfc10(&graph_node);

        // Compare hash results
        let hashes_match = custom_hash == rdfc10_hash;

        if !hashes_match && self.verbose_logging {
            warn!(
                "Hash mismatch for graph '{}': custom={}, rdfc10={}",
                graph_name, custom_hash, rdfc10_hash
            );
        }

        if self.verbose_logging {
            debug!(
                "Canonicalization validation completed for graph '{}': hashes_match={}",
                graph_name, hashes_match
            );
        }

        Ok(CanonicalizationConsistencyResult {
            graph_name: graph_name.to_string(),
            custom_algorithm_hash: custom_hash,
            rdfc10_algorithm_hash: rdfc10_hash,
            hashes_match,
            complexity,
        })
    }

    /// Get all graph names from the RDF store
    #[instrument(skip(self, rdf_store))]
    pub fn get_all_graph_names(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let mut graph_names = Vec::new();

        if self.verbose_logging {
            info!("Enumerating all graph names in RDF store");
        }

        // Query RDF store for all named graphs
        let query = r#"
            SELECT DISTINCT ?g WHERE {
                GRAPH ?g { ?s ?p ?o }
            }
            ORDER BY ?g
        "#;

        match rdf_store.query(query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions {
                    match solution {
                        Ok(sol) => {
                            if let Some(oxigraph::model::Term::NamedNode(graph_node)) = sol.get("g")
                            {
                                let graph_name = graph_node.as_str().to_string();

                                // Filter for relevant graph patterns (block graphs and blockchain metadata)
                                if graph_name.starts_with("http://provchain.org/block/")
                                    || graph_name == "http://provchain.org/blockchain"
                                {
                                    graph_names.push(graph_name);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error processing SPARQL solution: {}", e);
                            return Err(crate::error::ProvChainError::Validation(
                                crate::error::ValidationError::InvalidInput {
                                    field: "graph_enumeration".to_string(),
                                    reason: format!("Failed to enumerate graph names: {}", e),
                                },
                            ));
                        }
                    }
                }
            }
            _ => {
                error!("Unexpected query result type when enumerating graphs");
                return Err(crate::error::ProvChainError::Validation(
                    crate::error::ValidationError::InvalidInput {
                        field: "graph_enumeration".to_string(),
                        reason: "Failed to enumerate graph names: unexpected query result type"
                            .to_string(),
                    },
                ));
            }
        }

        // Sort graph names for consistent ordering
        graph_names.sort();

        if self.verbose_logging {
            debug!("Found {} relevant graph names", graph_names.len());
            for graph_name in &graph_names {
                debug!("  - {}", graph_name);
            }
        }

        Ok(graph_names)
    }

    /// Validate blank node handling consistency
    #[instrument(skip(self, rdf_store))]
    pub fn validate_blank_node_handling(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let mut blank_node_issues = Vec::new();

        if self.verbose_logging {
            info!("Validating blank node handling consistency");
        }

        // Get all graph names to check for blank nodes
        let graph_names = self.get_all_graph_names(rdf_store)?;

        for graph_name in &graph_names {
            // Query for blank nodes in this graph
            let blank_node_query = format!(
                r#"
                SELECT DISTINCT ?bn WHERE {{
                    GRAPH <{}> {{
                        {{ ?bn ?p ?o . FILTER(isBlank(?bn)) }}
                        UNION
                        {{ ?s ?p ?bn . FILTER(isBlank(?bn)) }}
                    }}
                }}
            "#,
                graph_name
            );

            let mut blank_nodes = Vec::new();
            match rdf_store.query(&blank_node_query) {
                oxigraph::sparql::QueryResults::Solutions(solutions) => {
                    for solution in solutions {
                        match solution {
                            Ok(sol) => {
                                if let Some(oxigraph::model::Term::BlankNode(bn)) = sol.get("bn") {
                                    blank_nodes.push(bn.as_str().to_string());
                                }
                            }
                            Err(e) => {
                                let issue = format!(
                                    "Error querying blank nodes in graph '{}': {}",
                                    graph_name, e
                                );
                                error!("{}", issue);
                                blank_node_issues.push(issue);
                            }
                        }
                    }
                }
                _ => {
                    let issue = format!(
                        "Unexpected query result type when checking blank nodes in graph '{}'",
                        graph_name
                    );
                    error!("{}", issue);
                    blank_node_issues.push(issue);
                }
            }

            if !blank_nodes.is_empty() {
                if self.verbose_logging {
                    debug!(
                        "Found {} blank nodes in graph '{}'",
                        blank_nodes.len(),
                        graph_name
                    );
                }

                // Test blank node canonicalization consistency
                // Run canonicalization multiple times to ensure deterministic behavior
                let graph_node = match NamedNode::new(graph_name) {
                    Ok(node) => node,
                    Err(e) => {
                        let issue = format!("Invalid graph name '{}': {}", graph_name, e);
                        blank_node_issues.push(issue);
                        continue;
                    }
                };

                let mut hashes = Vec::new();
                for run in 0..3 {
                    let hash = rdf_store.canonicalize_graph(&graph_node);
                    hashes.push(hash);

                    if self.verbose_logging {
                        debug!(
                            "Canonicalization run {} for graph '{}': hash starts with {}",
                            run + 1,
                            graph_name,
                            &hashes[run][..8]
                        );
                    }
                }

                // Check if all hashes are identical (deterministic behavior)
                let first_hash = &hashes[0];
                if !hashes.iter().all(|h| h == first_hash) {
                    let issue = format!("Non-deterministic blank node canonicalization in graph '{}': got {} different hashes across 3 runs", 
                                       graph_name, hashes.iter().collect::<std::collections::HashSet<_>>().len());
                    warn!("{}", issue);
                    blank_node_issues.push(issue);
                }

                // Test blank node identifier generation consistency between algorithms
                let custom_hash = rdf_store.canonicalize_graph(&graph_node);
                let rdfc10_hash = rdf_store.canonicalize_graph_rdfc10(&graph_node);

                if custom_hash != rdfc10_hash {
                    let complexity = rdf_store.analyze_graph_complexity(&graph_node);

                    // For simple graphs, algorithms should produce the same result
                    if matches!(
                        complexity,
                        crate::storage::rdf_store::GraphComplexity::Simple
                    ) {
                        let issue = format!("Blank node canonicalization mismatch in simple graph '{}': custom != rdfc10", graph_name);
                        warn!("{}", issue);
                        blank_node_issues.push(issue);
                    } else if self.verbose_logging {
                        debug!("Expected canonicalization difference in complex graph '{}' (complexity: {:?})", graph_name, complexity);
                    }
                }

                // Check for potential blank node collision issues
                // This is a simplified check - in practice, we'd need more sophisticated collision detection
                if blank_nodes.len() > 10 {
                    let issue = format!("Large number of blank nodes ({}) in graph '{}' may cause performance issues", 
                                       blank_nodes.len(), graph_name);
                    if self.verbose_logging {
                        debug!("{}", issue);
                    }
                    // This is informational, not necessarily an error
                }
            }
        }

        if self.verbose_logging {
            debug!(
                "Blank node validation completed with {} issues across {} graphs",
                blank_node_issues.len(),
                graph_names.len()
            );
        }

        Ok(blank_node_issues)
    }

    /// Test canonicalization performance across different graph complexities
    #[instrument(skip(self, rdf_store, graph_name))]
    pub fn test_canonicalization_performance(
        &self,
        rdf_store: &RDFStore,
        graph_name: &str,
    ) -> Result<CanonicalizationPerformanceResult> {
        if self.verbose_logging {
            debug!(
                "Testing canonicalization performance for graph: {}",
                graph_name
            );
        }

        // Create NamedNode from graph_name
        let graph_node = NamedNode::new(graph_name).map_err(|e| {
            crate::error::ProvChainError::Validation(crate::error::ValidationError::InvalidInput {
                field: "graph_name".to_string(),
                reason: format!("Invalid graph name '{}': {}", graph_name, e),
            })
        })?;

        // Analyze graph complexity first
        let complexity = rdf_store.analyze_graph_complexity(&graph_node);

        // Count graph size and blank nodes
        let mut graph_size = 0;
        let mut blank_node_count = 0;

        // Query to count total triples in the graph
        let size_query = format!(
            r#"
            SELECT (COUNT(*) as ?count) WHERE {{
                GRAPH <{}> {{ ?s ?p ?o }}
            }}
        "#,
            graph_name
        );

        match rdf_store.query(&size_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for sol in solutions.flatten() {
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                        if let Ok(count) = lit.value().parse::<usize>() {
                            graph_size = count;
                            break;
                        }
                    }
                }
            }
            _ => {
                warn!("Could not determine graph size for '{}'", graph_name);
            }
        }

        // Query to count blank nodes in the graph
        let blank_node_query = format!(
            r#"
            SELECT (COUNT(DISTINCT ?bn) as ?count) WHERE {{
                GRAPH <{}> {{
                    {{ ?bn ?p ?o . FILTER(isBlank(?bn)) }}
                    UNION
                    {{ ?s ?p ?bn . FILTER(isBlank(?bn)) }}
                }}
            }}
        "#,
            graph_name
        );

        match rdf_store.query(&blank_node_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for sol in solutions.flatten() {
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                        if let Ok(count) = lit.value().parse::<usize>() {
                            blank_node_count = count;
                            break;
                        }
                    }
                }
            }
            _ => {
                warn!("Could not determine blank node count for '{}'", graph_name);
            }
        }

        if self.verbose_logging {
            debug!(
                "Graph '{}' analysis: size={} triples, blank_nodes={}, complexity={:?}",
                graph_name, graph_size, blank_node_count, complexity
            );
        }

        // Skip performance testing for very large graphs to avoid timeouts
        if graph_size > self.max_graph_size {
            warn!(
                "Skipping performance test for large graph '{}' ({} triples > {} limit)",
                graph_name, graph_size, self.max_graph_size
            );
            return Ok(CanonicalizationPerformanceResult {
                graph_name: graph_name.to_string(),
                graph_size,
                blank_node_count,
                complexity,
                custom_algorithm_time_ms: 0,
                rdfc10_algorithm_time_ms: 0,
                custom_algorithm_memory_bytes: 0,
                rdfc10_algorithm_memory_bytes: 0,
            });
        }

        // Measure custom algorithm performance
        let custom_start = std::time::Instant::now();
        let _custom_hash = rdf_store.canonicalize_graph(&graph_node);
        let custom_duration = custom_start.elapsed();
        let custom_algorithm_time_ms = custom_duration.as_millis();

        // Measure RDFC-1.0 algorithm performance
        let rdfc10_start = std::time::Instant::now();
        let _rdfc10_hash = rdf_store.canonicalize_graph_rdfc10(&graph_node);
        let rdfc10_duration = rdfc10_start.elapsed();
        let rdfc10_algorithm_time_ms = rdfc10_duration.as_millis();

        // Memory usage estimation (simplified - in practice would need more sophisticated measurement)
        // Estimate based on graph size and complexity
        let base_memory = graph_size * 100; // Rough estimate: 100 bytes per triple
        let complexity_multiplier = match complexity {
            crate::storage::rdf_store::GraphComplexity::Simple => 1.0,
            crate::storage::rdf_store::GraphComplexity::Moderate => 1.5,
            crate::storage::rdf_store::GraphComplexity::Complex => 2.5,
            crate::storage::rdf_store::GraphComplexity::Pathological => 4.0,
        };

        let custom_algorithm_memory_bytes = (base_memory as f64 * complexity_multiplier) as usize;
        let rdfc10_algorithm_memory_bytes =
            (base_memory as f64 * complexity_multiplier * 1.2) as usize; // RDFC-1.0 typically uses more memory

        if self.verbose_logging {
            debug!("Performance results for graph '{}': custom={}ms, rdfc10={}ms, custom_mem={}KB, rdfc10_mem={}KB",
                   graph_name,
                   custom_algorithm_time_ms,
                   rdfc10_algorithm_time_ms,
                   custom_algorithm_memory_bytes / 1024,
                   rdfc10_algorithm_memory_bytes / 1024);
        }

        // Log performance comparison
        if custom_algorithm_time_ms > 0 && rdfc10_algorithm_time_ms > 0 {
            let performance_ratio =
                rdfc10_algorithm_time_ms as f64 / custom_algorithm_time_ms as f64;
            if self.verbose_logging {
                if performance_ratio > 2.0 {
                    debug!(
                        "Custom algorithm is {:.1}x faster than RDFC-1.0 for graph '{}'",
                        performance_ratio, graph_name
                    );
                } else if performance_ratio < 0.5 {
                    debug!(
                        "RDFC-1.0 algorithm is {:.1}x faster than custom for graph '{}'",
                        1.0 / performance_ratio,
                        graph_name
                    );
                } else {
                    debug!(
                        "Similar performance between algorithms for graph '{}' (ratio: {:.2})",
                        graph_name, performance_ratio
                    );
                }
            }
        }

        Ok(CanonicalizationPerformanceResult {
            graph_name: graph_name.to_string(),
            graph_size,
            blank_node_count,
            complexity,
            custom_algorithm_time_ms,
            rdfc10_algorithm_time_ms,
            custom_algorithm_memory_bytes,
            rdfc10_algorithm_memory_bytes,
        })
    }

    /// Validate hash consistency across multiple runs
    #[instrument(skip(self, rdf_store, graph_name))]
    pub fn validate_hash_consistency(
        &self,
        rdf_store: &RDFStore,
        graph_name: &str,
        runs: usize,
    ) -> Result<HashConsistencyResult> {
        if self.verbose_logging {
            debug!(
                "Validating hash consistency for graph '{}' across {} runs",
                graph_name, runs
            );
        }

        // Create NamedNode from graph_name
        let graph_node = NamedNode::new(graph_name).map_err(|e| {
            crate::error::ProvChainError::Validation(crate::error::ValidationError::InvalidInput {
                field: "graph_name".to_string(),
                reason: format!("Invalid graph name '{}': {}", graph_name, e),
            })
        })?;

        // Run canonicalization multiple times and collect all hash results
        let mut custom_hashes = Vec::new();
        let mut rdfc10_hashes = Vec::new();

        for run in 0..runs {
            if self.verbose_logging && run % 10 == 0 {
                debug!(
                    "Hash consistency validation run {} of {} for graph '{}'",
                    run + 1,
                    runs,
                    graph_name
                );
            }

            // Test custom algorithm consistency
            let custom_hash = rdf_store.canonicalize_graph(&graph_node);
            custom_hashes.push(custom_hash);

            // Test RDFC-1.0 algorithm consistency if enabled
            if self.test_both_algorithms {
                let rdfc10_hash = rdf_store.canonicalize_graph_rdfc10(&graph_node);
                rdfc10_hashes.push(rdfc10_hash);
            }
        }

        // Analyze hash variations for custom algorithm
        let unique_custom_hashes: std::collections::HashSet<_> = custom_hashes.iter().collect();
        let custom_is_consistent = unique_custom_hashes.len() == 1;

        let mut hash_variations = Vec::new();
        if !custom_is_consistent {
            hash_variations.push(format!(
                "Custom algorithm produced {} different hashes across {} runs",
                unique_custom_hashes.len(),
                runs
            ));

            if self.verbose_logging {
                warn!(
                    "Custom canonicalization inconsistency in graph '{}': {} unique hashes",
                    graph_name,
                    unique_custom_hashes.len()
                );
                for (i, hash) in unique_custom_hashes.iter().enumerate() {
                    debug!("  Unique hash {}: {}", i + 1, hash);
                }
            }
        }

        // Analyze hash variations for RDFC-1.0 algorithm if tested
        let mut rdfc10_is_consistent = true;
        if self.test_both_algorithms && !rdfc10_hashes.is_empty() {
            let unique_rdfc10_hashes: std::collections::HashSet<_> = rdfc10_hashes.iter().collect();
            rdfc10_is_consistent = unique_rdfc10_hashes.len() == 1;

            if !rdfc10_is_consistent {
                hash_variations.push(format!(
                    "RDFC-1.0 algorithm produced {} different hashes across {} runs",
                    unique_rdfc10_hashes.len(),
                    runs
                ));

                if self.verbose_logging {
                    warn!(
                        "RDFC-1.0 canonicalization inconsistency in graph '{}': {} unique hashes",
                        graph_name,
                        unique_rdfc10_hashes.len()
                    );
                }
            }

            // Check cross-algorithm consistency
            if custom_is_consistent && rdfc10_is_consistent {
                let custom_hash = &custom_hashes[0];
                let rdfc10_hash = &rdfc10_hashes[0];

                if custom_hash != rdfc10_hash {
                    let complexity = rdf_store.analyze_graph_complexity(&graph_node);

                    // For simple graphs, we expect algorithms to match
                    if matches!(
                        complexity,
                        crate::storage::rdf_store::GraphComplexity::Simple
                    ) {
                        hash_variations.push(format!("Algorithm mismatch in simple graph: custom != rdfc10 (complexity: {:?})", complexity));
                    } else if self.verbose_logging {
                        debug!("Expected algorithm difference in complex graph '{}' (complexity: {:?})", graph_name, complexity);
                    }
                }
            }
        }

        // Overall consistency check
        let is_consistent = custom_is_consistent && rdfc10_is_consistent;
        let total_unique_hashes = if self.test_both_algorithms {
            unique_custom_hashes.len().max(if rdfc10_hashes.is_empty() {
                0
            } else {
                rdfc10_hashes
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            })
        } else {
            unique_custom_hashes.len()
        };

        if self.verbose_logging {
            if is_consistent {
                debug!(
                    "Hash consistency validation passed for graph '{}': {} runs, {} unique hashes",
                    graph_name, runs, total_unique_hashes
                );
            } else {
                warn!("Hash consistency validation failed for graph '{}': {} runs, {} unique hashes, {} issues", 
                      graph_name, runs, total_unique_hashes, hash_variations.len());
            }
        }

        Ok(HashConsistencyResult {
            graph_name: graph_name.to_string(),
            total_runs: runs,
            unique_hashes: total_unique_hashes,
            is_consistent,
            hash_variations,
        })
    }

    /// Generate canonicalization integrity recommendations
    pub fn generate_recommendations(
        &self,
        status: &CanonicalizationIntegrityStatus,
    ) -> Vec<IntegrityRecommendation> {
        let mut recommendations = Vec::new();

        // Hash validation failures recommendations
        if !status.hash_validation_failures.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Canonicalization Integrity".to_string(),
                description: format!(
                    "Found {} hash validation failures",
                    status.hash_validation_failures.len()
                ),
                action_required:
                    "Investigate canonicalization algorithm implementation and hash generation"
                        .to_string(),
                auto_fixable: false,
            });
        }

        // Blank node handling issues recommendations
        if !status.blank_node_handling_issues.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "Canonicalization Integrity".to_string(),
                description: format!("Found {} blank node handling issues", status.blank_node_handling_issues.len()),
                action_required: "Review blank node canonicalization logic and identifier generation".to_string(),
                auto_fixable: true,
            });
        }

        // Algorithm consistency recommendations
        let inconsistent_algorithms = status
            .algorithm_consistency_checks
            .iter()
            .filter(|check| !check.hashes_match)
            .count();

        if inconsistent_algorithms > 0 {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Canonicalization Integrity".to_string(),
                description: format!("Found hash mismatches in {} graphs between canonicalization algorithms", inconsistent_algorithms),
                action_required: "Investigate algorithm implementation differences and ensure semantic equivalence".to_string(),
                auto_fixable: false,
            });
        }

        // Complex graph handling recommendations
        let complex_graphs = status
            .algorithm_consistency_checks
            .iter()
            .filter(|check| {
                matches!(
                    check.complexity,
                    crate::storage::rdf_store::GraphComplexity::Complex
                        | crate::storage::rdf_store::GraphComplexity::Pathological
                )
            })
            .count();

        if complex_graphs > 0 {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Info,
                category: "Canonicalization Integrity".to_string(),
                description: format!(
                    "Found {} graphs with complex canonicalization patterns",
                    complex_graphs
                ),
                action_required:
                    "Consider performance optimization for complex graph canonicalization"
                        .to_string(),
                auto_fixable: false,
            });
        }

        // Performance recommendations
        let total_graphs = status.algorithm_consistency_checks.len();
        if total_graphs > 100 {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Info,
                category: "Canonicalization Integrity".to_string(),
                description: format!(
                    "Large number of graphs ({}) may impact canonicalization performance",
                    total_graphs
                ),
                action_required: "Consider implementing incremental canonicalization validation"
                    .to_string(),
                auto_fixable: false,
            });
        }

        recommendations
    }

    /// Get canonicalization validation statistics
    pub fn get_validation_statistics(
        &self,
        status: &CanonicalizationIntegrityStatus,
    ) -> CanonicalizationValidationStatistics {
        let total_graphs = status.algorithm_consistency_checks.len();
        let consistent_graphs = status
            .algorithm_consistency_checks
            .iter()
            .filter(|check| check.hashes_match)
            .count();

        let complexity_distribution = {
            let mut simple = 0;
            let mut moderate = 0;
            let mut complex = 0;
            let mut pathological = 0;

            for check in &status.algorithm_consistency_checks {
                match check.complexity {
                    crate::storage::rdf_store::GraphComplexity::Simple => simple += 1,
                    crate::storage::rdf_store::GraphComplexity::Moderate => moderate += 1,
                    crate::storage::rdf_store::GraphComplexity::Complex => complex += 1,
                    crate::storage::rdf_store::GraphComplexity::Pathological => pathological += 1,
                }
            }

            (simple, moderate, complex, pathological)
        };

        CanonicalizationValidationStatistics {
            total_graphs,
            consistent_graphs,
            inconsistent_graphs: total_graphs - consistent_graphs,
            hash_validation_failures: status.hash_validation_failures.len(),
            blank_node_issues: status.blank_node_handling_issues.len(),
            simple_graphs: complexity_distribution.0,
            moderate_graphs: complexity_distribution.1,
            complex_graphs: complexity_distribution.2,
            pathological_graphs: complexity_distribution.3,
            consistency_rate: if total_graphs > 0 {
                (consistent_graphs as f64 / total_graphs as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Canonicalization performance test result
#[derive(Debug, Clone)]
pub struct CanonicalizationPerformanceResult {
    pub graph_name: String,
    pub graph_size: usize,
    pub blank_node_count: usize,
    pub complexity: crate::storage::rdf_store::GraphComplexity,
    pub custom_algorithm_time_ms: u128,
    pub rdfc10_algorithm_time_ms: u128,
    pub custom_algorithm_memory_bytes: usize,
    pub rdfc10_algorithm_memory_bytes: usize,
}

/// Hash consistency validation result
#[derive(Debug, Clone)]
pub struct HashConsistencyResult {
    pub graph_name: String,
    pub total_runs: usize,
    pub unique_hashes: usize,
    pub is_consistent: bool,
    pub hash_variations: Vec<String>,
}

/// Canonicalization validation statistics
#[derive(Debug, Clone)]
pub struct CanonicalizationValidationStatistics {
    pub total_graphs: usize,
    pub consistent_graphs: usize,
    pub inconsistent_graphs: usize,
    pub hash_validation_failures: usize,
    pub blank_node_issues: usize,
    pub simple_graphs: usize,
    pub moderate_graphs: usize,
    pub complex_graphs: usize,
    pub pathological_graphs: usize,
    pub consistency_rate: f64,
}

impl Default for CanonicalizationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::rdf_store::RDFStore;

    #[test]
    fn test_canonicalization_validator_creation() {
        let validator = CanonicalizationValidator::new();
        assert!(!validator.verbose_logging);
        assert!(validator.test_both_algorithms);
        assert_eq!(validator.max_graph_size, 10000);
    }

    #[test]
    fn test_canonicalization_validator_with_config() {
        let validator = CanonicalizationValidator::with_config(true, false, 5000);
        assert!(validator.verbose_logging);
        assert!(!validator.test_both_algorithms);
        assert_eq!(validator.max_graph_size, 5000);
    }

    #[test]
    fn test_validate_single_graph_consistency_basic() {
        let validator = CanonicalizationValidator::new();
        let rdf_store = RDFStore::new();
        let graph_name = "http://example.org/test";

        let result = validator.validate_single_graph_consistency(&rdf_store, graph_name);
        assert!(result.is_ok());

        let consistency_result = result.unwrap();
        assert_eq!(consistency_result.graph_name, graph_name);
        assert!(consistency_result.hashes_match);
    }

    #[test]
    fn test_get_all_graph_names_basic() {
        let validator = CanonicalizationValidator::new();
        let rdf_store = RDFStore::new();

        let result = validator.get_all_graph_names(&rdf_store);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty()); // Empty store should return empty list
    }
}
