//! SPARQL query consistency validation
//!
//! This module validates SPARQL query consistency across the RDF store,
//! ensuring that queries return expected results and that all graphs
//! are accessible and consistent.

use crate::error::Result;
use crate::integrity::{
    IntegrityRecommendation, QueryConsistencyResult, RecommendationSeverity, SparqlIntegrityStatus,
};
use crate::storage::rdf_store::RDFStore;
use tracing::{debug, error, info, instrument, warn};

/// SPARQL query consistency validator
pub struct SparqlConsistencyValidator {
    /// Enable detailed validation logging
    pub verbose_logging: bool,
    /// Maximum query execution time (in seconds)
    pub max_query_time: u64,
    /// Test queries to validate consistency
    pub test_queries: Vec<String>,
}

impl SparqlConsistencyValidator {
    /// Create a new SPARQL consistency validator
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            max_query_time: 30,
            test_queries: Self::default_test_queries(),
        }
    }

    /// Create a validator with custom configuration
    pub fn with_config(verbose: bool, max_time: u64, queries: Vec<String>) -> Self {
        Self {
            verbose_logging: verbose,
            max_query_time: max_time,
            test_queries: queries,
        }
    }

    /// Get default test queries for consistency validation
    fn default_test_queries() -> Vec<String> {
        vec![
            // Basic triple count
            "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }".to_string(),

            // Graph enumeration
            "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } }".to_string(),

            // Block metadata validation
            "PREFIX prov: <http://provchain.org/> SELECT ?block ?index WHERE { GRAPH <http://provchain.org/blockchain> { ?block prov:hasIndex ?index } }".to_string(),

            // Data graph accessibility
            "SELECT ?g (COUNT(*) as ?triples) WHERE { GRAPH ?g { ?s ?p ?o } FILTER(STRSTARTS(STR(?g), \"http://provchain.org/block/\")) } GROUP BY ?g".to_string(),
        ]
    }

    /// Validate query result consistency
    #[instrument(skip(self, rdf_store, test_queries))]
    pub fn validate_query_result_consistency(
        &self,
        rdf_store: &RDFStore,
        test_queries: &[String],
    ) -> Result<Vec<QueryConsistencyResult>> {
        let mut results = Vec::new();

        if self.verbose_logging {
            info!(
                "Validating consistency for {} test queries",
                test_queries.len()
            );
        }

        for query in test_queries {
            match self.validate_single_query_consistency(rdf_store, query) {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Query validation failed for '{}': {}", query, e);
                    results.push(QueryConsistencyResult {
                        query: query.clone(),
                        expected_result_count: 0,
                        actual_result_count: 0,
                        missing_graphs: Vec::new(),
                        inaccessible_data: vec![format!("Query execution failed: {}", e)],
                    });
                }
            }
        }

        debug!(
            "Query consistency validation completed for {} queries",
            results.len()
        );
        Ok(results)
    }

    /// Validate consistency of a single query
    #[instrument(skip(self, rdf_store, query))]
    pub fn validate_single_query_consistency(
        &self,
        rdf_store: &RDFStore,
        query: &str,
    ) -> Result<QueryConsistencyResult> {
        if self.verbose_logging {
            debug!("Validating query: {}", query);
        }

        let mut result = QueryConsistencyResult {
            query: query.to_string(),
            expected_result_count: 0,
            actual_result_count: 0,
            missing_graphs: Vec::new(),
            inaccessible_data: Vec::new(),
        };

        // Execute the query and measure results
        match rdf_store.query(query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut count = 0;
                for solution_result in solutions {
                    match solution_result {
                        Ok(_) => count += 1,
                        Err(e) => {
                            result
                                .inaccessible_data
                                .push(format!("Solution error: {}", e));
                        }
                    }
                }
                result.actual_result_count = count;

                if self.verbose_logging {
                    debug!("Query returned {} solutions", count);
                }
            }
            oxigraph::sparql::QueryResults::Boolean(boolean_result) => {
                result.actual_result_count = if boolean_result { 1 } else { 0 };

                if self.verbose_logging {
                    debug!("Query returned boolean: {}", boolean_result);
                }
            }
            oxigraph::sparql::QueryResults::Graph(graph_results) => {
                let mut count = 0;
                for triple_result in graph_results {
                    match triple_result {
                        Ok(_) => count += 1,
                        Err(e) => {
                            result
                                .inaccessible_data
                                .push(format!("Triple error: {}", e));
                        }
                    }
                }
                result.actual_result_count = count;

                if self.verbose_logging {
                    debug!("Query returned {} triples", count);
                }
            }
        }

        // For specific query types, calculate expected results
        result.expected_result_count = self.calculate_expected_result_count(rdf_store, query)?;

        // Check for missing graphs if query references specific graphs
        result.missing_graphs = self.detect_missing_graphs_in_query(rdf_store, query)?;

        // Validate query performance
        let performance_result = self.test_query_performance(rdf_store, query)?;
        if performance_result.timed_out {
            result.inaccessible_data.push("Query timed out".to_string());
        }

        if self.verbose_logging {
            debug!(
                "Query validation completed: expected={}, actual={}, missing_graphs={}, issues={}",
                result.expected_result_count,
                result.actual_result_count,
                result.missing_graphs.len(),
                result.inaccessible_data.len()
            );
        }

        Ok(result)
    }

    /// Validate graph accessibility
    #[instrument(skip(self, rdf_store))]
    pub fn validate_graph_accessibility(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let mut accessibility_issues = Vec::new();

        if self.verbose_logging {
            info!("Validating graph accessibility");
        }

        // Enumerate all named graphs
        let graph_enumeration_query = "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } }";
        let mut discovered_graphs = Vec::new();

        match rdf_store.query(graph_enumeration_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution_result in solutions {
                    match solution_result {
                        Ok(solution) => {
                            if let Some(oxigraph::model::Term::NamedNode(graph_node)) =
                                solution.get("g")
                            {
                                discovered_graphs.push(graph_node.as_str().to_string());
                            }
                        }
                        Err(e) => {
                            accessibility_issues.push(format!("Error enumerating graphs: {}", e));
                        }
                    }
                }
            }
            _ => {
                accessibility_issues
                    .push("Graph enumeration query returned unexpected result type".to_string());
            }
        }

        if self.verbose_logging {
            debug!("Discovered {} graphs", discovered_graphs.len());
        }

        // Test accessibility of each discovered graph
        for graph_name in &discovered_graphs {
            let accessibility_query = format!("ASK {{ GRAPH <{}> {{ ?s ?p ?o }} }}", graph_name);

            match rdf_store.query(&accessibility_query) {
                oxigraph::sparql::QueryResults::Boolean(accessible) => {
                    if !accessible {
                        accessibility_issues
                            .push(format!("Graph {} is not accessible", graph_name));
                    }
                }
                _ => {
                    accessibility_issues.push(format!(
                        "Failed to test accessibility of graph {}",
                        graph_name
                    ));
                }
            }
        }

        // Validate graph naming consistency for ProvChain graphs
        for graph_name in &discovered_graphs {
            if graph_name.starts_with("http://provchain.org/block/") {
                // Extract block index from graph name
                if let Some(index_str) = graph_name.strip_prefix("http://provchain.org/block/") {
                    if index_str.parse::<u64>().is_err() {
                        accessibility_issues
                            .push(format!("Invalid block graph naming: {}", graph_name));
                    }
                }
            } else if graph_name == "http://provchain.org/blockchain" {
                // Validate blockchain metadata graph has expected structure
                let metadata_query = format!(
                    "ASK {{ GRAPH <{}> {{ ?block <http://provchain.org/hasIndex> ?index }} }}",
                    graph_name
                );

                match rdf_store.query(&metadata_query) {
                    oxigraph::sparql::QueryResults::Boolean(has_metadata) => {
                        if !has_metadata {
                            accessibility_issues.push(
                                "Blockchain metadata graph missing expected structure".to_string(),
                            );
                        }
                    }
                    _ => {
                        accessibility_issues.push(
                            "Failed to validate blockchain metadata graph structure".to_string(),
                        );
                    }
                }
            }
        }

        // Check for orphaned or unreachable graphs by comparing with expected block graphs
        let expected_blockchain_graphs = self.get_expected_blockchain_graphs(rdf_store)?;
        for expected_graph in &expected_blockchain_graphs {
            if !discovered_graphs.contains(expected_graph) {
                accessibility_issues.push(format!("Expected graph {} is missing", expected_graph));
            }
        }

        debug!(
            "Graph accessibility validation completed with {} issues",
            accessibility_issues.len()
        );
        Ok(accessibility_issues)
    }

    /// Cross-validate query results against raw storage
    #[instrument(skip(self, rdf_store))]
    pub fn cross_validate_query_results(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let mut validation_mismatches = Vec::new();

        if self.verbose_logging {
            info!("Cross-validating query results against raw storage");
        }

        // Test 1: Validate total triple count
        let count_query = "SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }";
        match rdf_store.query(count_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut query_count = 0;
                for solution in solutions.flatten() {
                    if let Some(count_term) = solution.get("count") {
                        if let oxigraph::model::Term::Literal(literal) = count_term {
                            if let Ok(count) = literal.value().parse::<usize>() {
                                query_count = count;
                                break;
                            }
                        }
                    }
                }

                // Compare with direct store iteration
                let mut direct_count = 0;
                for quad_result in rdf_store.store.iter() {
                    if quad_result.is_ok() {
                        direct_count += 1;
                    }
                }

                if query_count != direct_count {
                    validation_mismatches.push(format!(
                        "Total triple count mismatch: query={}, direct={}",
                        query_count, direct_count
                    ));
                }
            }
            _ => {
                validation_mismatches.push("Failed to execute total count query".to_string());
            }
        }

        // Test 2: Validate graph count consistency
        let graph_count_query =
            "SELECT (COUNT(DISTINCT ?g) as ?count) WHERE { GRAPH ?g { ?s ?p ?o } }";
        match rdf_store.query(graph_count_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut query_graph_count = 0;
                for solution in solutions.flatten() {
                    if let Some(count_term) = solution.get("count") {
                        if let oxigraph::model::Term::Literal(literal) = count_term {
                            if let Ok(count) = literal.value().parse::<usize>() {
                                query_graph_count = count;
                                break;
                            }
                        }
                    }
                }

                // Count distinct graphs through direct iteration
                let mut direct_graphs = std::collections::HashSet::new();
                for quad_result in rdf_store.store.iter() {
                    if let Ok(quad) = quad_result {
                        match &quad.graph_name {
                            oxigraph::model::GraphName::NamedNode(node) => {
                                direct_graphs.insert(node.as_str().to_string());
                            }
                            _ => {} // Skip default graph
                        }
                    }
                }

                if query_graph_count != direct_graphs.len() {
                    validation_mismatches.push(format!(
                        "Graph count mismatch: query={}, direct={}",
                        query_graph_count,
                        direct_graphs.len()
                    ));
                }
            }
            _ => {
                validation_mismatches.push("Failed to execute graph count query".to_string());
            }
        }

        // Test 3: Validate specific graph content consistency
        let graph_enum_query = "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } }";
        let mut discovered_graphs = Vec::new();

        match rdf_store.query(graph_enum_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions.flatten() {
                    if let Some(graph_term) = solution.get("g") {
                        if let oxigraph::model::Term::NamedNode(graph_node) = graph_term {
                            discovered_graphs.push(graph_node.as_str().to_string());
                        }
                    }
                }
            }
            _ => {
                validation_mismatches.push("Failed to enumerate graphs for validation".to_string());
            }
        }

        // For each discovered graph, validate triple count consistency
        for graph_name in discovered_graphs {
            let graph_triple_query = format!(
                "SELECT (COUNT(*) as ?count) WHERE {{ GRAPH <{}> {{ ?s ?p ?o }} }}",
                graph_name
            );

            match rdf_store.query(&graph_triple_query) {
                oxigraph::sparql::QueryResults::Solutions(solutions) => {
                    let mut query_triple_count = 0;
                    for solution in solutions.flatten() {
                        if let Some(count_term) = solution.get("count") {
                            if let oxigraph::model::Term::Literal(literal) = count_term {
                                if let Ok(count) = literal.value().parse::<usize>() {
                                    query_triple_count = count;
                                    break;
                                }
                            }
                        }
                    }

                    // Count triples in this graph through direct iteration
                    let mut direct_triple_count = 0;
                    if let Ok(graph_node) = oxigraph::model::NamedNode::new(&graph_name) {
                        for quad_result in rdf_store.store.quads_for_pattern(
                            None,
                            None,
                            None,
                            Some((&graph_node).into()),
                        ) {
                            if quad_result.is_ok() {
                                direct_triple_count += 1;
                            }
                        }
                    }

                    if query_triple_count != direct_triple_count {
                        validation_mismatches.push(format!(
                            "Graph {} triple count mismatch: query={}, direct={}",
                            graph_name, query_triple_count, direct_triple_count
                        ));
                    }
                }
                _ => {
                    validation_mismatches
                        .push(format!("Failed to count triples in graph {}", graph_name));
                }
            }
        }

        debug!(
            "Cross-validation completed with {} mismatches",
            validation_mismatches.len()
        );
        Ok(validation_mismatches)
    }

    /// Cross-validate canonicalization with queries
    #[instrument(skip(self, rdf_store))]
    pub fn cross_validate_canonicalization_queries(
        &self,
        rdf_store: &RDFStore,
    ) -> Result<Vec<String>> {
        let mut canonicalization_mismatches = Vec::new();

        if self.verbose_logging {
            info!("Cross-validating canonicalization with query results");
        }

        // Get all named graphs for canonicalization testing
        let graph_enumeration_query = "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } }";
        let mut discovered_graphs = Vec::new();

        match rdf_store.query(graph_enumeration_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions.flatten() {
                    if let Some(graph_term) = solution.get("g") {
                        if let oxigraph::model::Term::NamedNode(graph_node) = graph_term {
                            discovered_graphs.push(graph_node.as_str().to_string());
                        }
                    }
                }
            }
            _ => {
                canonicalization_mismatches
                    .push("Failed to enumerate graphs for canonicalization validation".to_string());
                return Ok(canonicalization_mismatches);
            }
        }

        // For each graph, validate that canonicalization results are consistent with query access
        for graph_name in discovered_graphs {
            if let Ok(graph_node) = oxigraph::model::NamedNode::new(&graph_name) {
                // Test 1: Validate that canonicalized graphs are queryable
                let graph_accessibility_query =
                    format!("ASK {{ GRAPH <{}> {{ ?s ?p ?o }} }}", graph_name);
                match rdf_store.query(&graph_accessibility_query) {
                    oxigraph::sparql::QueryResults::Boolean(accessible) => {
                        if !accessible {
                            canonicalization_mismatches.push(format!(
                                "Graph {} is not accessible via SPARQL but exists for canonicalization", 
                                graph_name
                            ));
                            continue;
                        }
                    }
                    _ => {
                        canonicalization_mismatches.push(format!(
                            "Failed to test accessibility of graph {} for canonicalization validation", 
                            graph_name
                        ));
                        continue;
                    }
                }

                // Test 2: Compare query-based triple enumeration with canonicalization input
                let triple_enumeration_query = format!(
                    "SELECT ?s ?p ?o WHERE {{ GRAPH <{}> {{ ?s ?p ?o }} }}",
                    graph_name
                );

                let mut query_triples = Vec::new();
                match rdf_store.query(&triple_enumeration_query) {
                    oxigraph::sparql::QueryResults::Solutions(solutions) => {
                        for solution in solutions.flatten() {
                            if let (Some(s), Some(p), Some(o)) =
                                (solution.get("s"), solution.get("p"), solution.get("o"))
                            {
                                query_triples.push((s.clone(), p.clone(), o.clone()));
                            }
                        }
                    }
                    _ => {
                        canonicalization_mismatches.push(format!(
                            "Failed to enumerate triples in graph {} for canonicalization validation", 
                            graph_name
                        ));
                        continue;
                    }
                }

                // Test 3: Validate blank node handling consistency
                let blank_node_query = format!(
                    "SELECT ?s ?p ?o WHERE {{ GRAPH <{}> {{ ?s ?p ?o }} FILTER(isBlank(?s) || isBlank(?o)) }}", 
                    graph_name
                );

                let mut has_blank_nodes = false;
                match rdf_store.query(&blank_node_query) {
                    oxigraph::sparql::QueryResults::Solutions(solutions) => {
                        for solution_result in solutions {
                            if solution_result.is_ok() {
                                has_blank_nodes = true;
                                break;
                            }
                        }
                    }
                    _ => {
                        canonicalization_mismatches.push(format!(
                            "Failed to check blank nodes in graph {} for canonicalization validation", 
                            graph_name
                        ));
                        continue;
                    }
                }

                // Test 4: Verify graph hash consistency with query-accessible content
                // Generate a simple hash based on query results for comparison
                let mut query_content_hash = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;

                for (s, p, o) in &query_triples {
                    query_content_hash.write(s.to_string().as_bytes());
                    query_content_hash.write(p.to_string().as_bytes());
                    query_content_hash.write(o.to_string().as_bytes());
                }
                let query_hash = query_content_hash.finish();

                // Get canonicalization hash for comparison
                let canonical_hash = rdf_store.canonicalize_graph(&graph_node);

                // Note: We can't directly compare these hashes as they use different algorithms,
                // but we can validate that both methods see the same content structure
                if query_triples.is_empty() && !canonical_hash.is_empty() {
                    canonicalization_mismatches.push(format!(
                        "Graph {} appears empty via SPARQL but has canonicalization hash: {}",
                        graph_name, canonical_hash
                    ));
                } else if !query_triples.is_empty() && canonical_hash.is_empty() {
                    canonicalization_mismatches.push(format!(
                        "Graph {} has {} triples via SPARQL but empty canonicalization hash",
                        graph_name,
                        query_triples.len()
                    ));
                }

                if self.verbose_logging {
                    debug!("Graph {} validation: {} triples, blank_nodes={}, query_hash={}, canonical_hash={}", 
                           graph_name, query_triples.len(), has_blank_nodes, query_hash, canonical_hash);
                }
            }
        }

        debug!(
            "Canonicalization cross-validation completed with {} mismatches",
            canonicalization_mismatches.len()
        );
        Ok(canonicalization_mismatches)
    }

    /// Test query performance and timeout handling
    #[instrument(skip(self, rdf_store, query))]
    pub fn test_query_performance(
        &self,
        rdf_store: &RDFStore,
        query: &str,
    ) -> Result<QueryPerformanceResult> {
        if self.verbose_logging {
            debug!("Testing performance for query: {}", query);
        }

        let start_time = std::time::Instant::now();
        let mut result_count = 0;
        let mut timed_out = false;

        // Execute query with timeout monitoring (without panic handling due to UnwindSafe issues)
        let query_results = rdf_store.query(query);
        let execution_time = start_time.elapsed();

        // Check for timeout
        if execution_time.as_secs() > self.max_query_time {
            timed_out = true;
            if self.verbose_logging {
                warn!("Query timed out after {:?}", execution_time);
            }
        }

        // Count results
        match query_results {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution_result in solutions {
                    if solution_result.is_ok() {
                        result_count += 1;
                    }

                    // Check for timeout during result iteration
                    if start_time.elapsed().as_secs() > self.max_query_time {
                        timed_out = true;
                        break;
                    }
                }
            }
            oxigraph::sparql::QueryResults::Boolean(_) => {
                result_count = 1;
            }
            oxigraph::sparql::QueryResults::Graph(graph_results) => {
                for triple_result in graph_results {
                    if triple_result.is_ok() {
                        result_count += 1;
                    }

                    // Check for timeout during result iteration
                    if start_time.elapsed().as_secs() > self.max_query_time {
                        timed_out = true;
                        break;
                    }
                }
            }
        }

        // Estimate memory usage (simplified approach)
        let estimated_memory_usage = result_count * 256; // Rough estimate: 256 bytes per result

        let performance_result = QueryPerformanceResult {
            query: query.to_string(),
            execution_time_ms: execution_time.as_millis(),
            result_count,
            memory_usage_bytes: estimated_memory_usage,
            timed_out,
        };

        if self.verbose_logging {
            debug!(
                "Query performance: {}ms, {} results, {}KB memory, timed_out={}",
                performance_result.execution_time_ms,
                performance_result.result_count,
                performance_result.memory_usage_bytes / 1024,
                performance_result.timed_out
            );
        }

        Ok(performance_result)
    }

    /// Generate SPARQL consistency recommendations
    pub fn generate_recommendations(
        &self,
        status: &SparqlIntegrityStatus,
    ) -> Vec<IntegrityRecommendation> {
        let mut recommendations = Vec::new();

        // Graph accessibility issues recommendations
        if !status.graph_accessibility_issues.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "SPARQL Consistency".to_string(),
                description: format!(
                    "Found {} graph accessibility issues",
                    status.graph_accessibility_issues.len()
                ),
                action_required: "Review graph naming and accessibility patterns".to_string(),
                auto_fixable: false,
            });
        }

        // Canonicalization query mismatches recommendations
        if !status.canonicalization_query_mismatches.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "SPARQL Consistency".to_string(),
                description: format!("Found {} canonicalization-query mismatches", status.canonicalization_query_mismatches.len()),
                action_required: "Investigate consistency between canonicalization and query results".to_string(),
                auto_fixable: false,
            });
        }

        // Query consistency issues recommendations
        let failed_queries = status
            .query_consistency_checks
            .iter()
            .filter(|check| {
                check.expected_result_count != check.actual_result_count
                    || !check.missing_graphs.is_empty()
                    || !check.inaccessible_data.is_empty()
            })
            .count();

        if failed_queries > 0 {
            let severity = if failed_queries > status.query_consistency_checks.len() / 2 {
                RecommendationSeverity::Critical
            } else {
                RecommendationSeverity::Warning
            };

            recommendations.push(IntegrityRecommendation {
                severity,
                category: "SPARQL Consistency".to_string(),
                description: format!(
                    "Found consistency issues in {} out of {} test queries",
                    failed_queries,
                    status.query_consistency_checks.len()
                ),
                action_required: "Review query execution logic and data consistency".to_string(),
                auto_fixable: false,
            });
        }

        // Missing graphs recommendations
        let total_missing_graphs: usize = status
            .query_consistency_checks
            .iter()
            .map(|check| check.missing_graphs.len())
            .sum();

        if total_missing_graphs > 0 {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "SPARQL Consistency".to_string(),
                description: format!(
                    "Found {} missing graphs across test queries",
                    total_missing_graphs
                ),
                action_required: "Restore missing graphs or update query expectations".to_string(),
                auto_fixable: false,
            });
        }

        // Inaccessible data recommendations
        let total_inaccessible_data: usize = status
            .query_consistency_checks
            .iter()
            .map(|check| check.inaccessible_data.len())
            .sum();

        if total_inaccessible_data > 0 {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "SPARQL Consistency".to_string(),
                description: format!(
                    "Found {} inaccessible data issues across test queries",
                    total_inaccessible_data
                ),
                action_required: "Review data accessibility and query permissions".to_string(),
                auto_fixable: false,
            });
        }

        recommendations
    }

    /// Get SPARQL validation statistics
    pub fn get_validation_statistics(
        &self,
        status: &SparqlIntegrityStatus,
    ) -> SparqlValidationStatistics {
        let total_queries = status.query_consistency_checks.len();
        let successful_queries = status
            .query_consistency_checks
            .iter()
            .filter(|check| {
                check.expected_result_count == check.actual_result_count
                    && check.missing_graphs.is_empty()
                    && check.inaccessible_data.is_empty()
            })
            .count();

        let total_missing_graphs: usize = status
            .query_consistency_checks
            .iter()
            .map(|check| check.missing_graphs.len())
            .sum();

        let total_inaccessible_data: usize = status
            .query_consistency_checks
            .iter()
            .map(|check| check.inaccessible_data.len())
            .sum();

        SparqlValidationStatistics {
            total_queries,
            successful_queries,
            failed_queries: total_queries - successful_queries,
            total_missing_graphs,
            total_inaccessible_data,
            graph_accessibility_issues: status.graph_accessibility_issues.len(),
            canonicalization_mismatches: status.canonicalization_query_mismatches.len(),
            success_rate: if total_queries > 0 {
                (successful_queries as f64 / total_queries as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Calculate expected result count for a query based on store state
    fn calculate_expected_result_count(&self, rdf_store: &RDFStore, query: &str) -> Result<usize> {
        // For basic count queries, we can estimate based on store size
        if query.contains("COUNT(*)") && query.contains("?s ?p ?o") && !query.contains("GRAPH") {
            // Total triple count query
            match rdf_store.store.len() {
                Ok(count) => return Ok(count),
                Err(_) => return Ok(0),
            }
        }

        // For graph enumeration queries
        if query.contains("SELECT DISTINCT ?g") && query.contains("GRAPH ?g") {
            let graph_count_query =
                "SELECT (COUNT(DISTINCT ?g) as ?count) WHERE { GRAPH ?g { ?s ?p ?o } }";
            if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                rdf_store.query(graph_count_query)
            {
                for solution in solutions.flatten() {
                    if let Some(count_term) = solution.get("count") {
                        if let oxigraph::model::Term::Literal(literal) = count_term {
                            if let Ok(count) = literal.value().parse::<usize>() {
                                return Ok(count);
                            }
                        }
                    }
                }
            }
        }

        // For block metadata queries
        if query.contains("prov:hasIndex") && query.contains("http://provchain.org/blockchain") {
            let block_count_query = "SELECT (COUNT(*) as ?count) WHERE { GRAPH <http://provchain.org/blockchain> { ?block <http://provchain.org/hasIndex> ?index } }";
            if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                rdf_store.query(block_count_query)
            {
                for solution in solutions.flatten() {
                    if let Some(count_term) = solution.get("count") {
                        if let oxigraph::model::Term::Literal(literal) = count_term {
                            if let Ok(count) = literal.value().parse::<usize>() {
                                return Ok(count);
                            }
                        }
                    }
                }
            }
        }

        // For block data graph queries
        if query.contains("http://provchain.org/block/") && query.contains("GROUP BY ?g") {
            // Count distinct block graphs
            let block_graph_query = "SELECT (COUNT(DISTINCT ?g) as ?count) WHERE { GRAPH ?g { ?s ?p ?o } FILTER(STRSTARTS(STR(?g), \"http://provchain.org/block/\")) }";
            if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                rdf_store.query(block_graph_query)
            {
                for solution in solutions.flatten() {
                    if let Some(count_term) = solution.get("count") {
                        if let oxigraph::model::Term::Literal(literal) = count_term {
                            if let Ok(count) = literal.value().parse::<usize>() {
                                return Ok(count);
                            }
                        }
                    }
                }
            }
        }

        // Default: return 0 for unknown query patterns
        Ok(0)
    }

    /// Detect missing graphs referenced in a query
    fn detect_missing_graphs_in_query(
        &self,
        rdf_store: &RDFStore,
        query: &str,
    ) -> Result<Vec<String>> {
        let mut missing_graphs = Vec::new();

        // Extract explicit graph references from the query
        let graph_references = self.extract_graph_references(query);

        for graph_uri in graph_references {
            // Check if the graph exists and has content
            let check_query = format!("ASK {{ GRAPH <{}> {{ ?s ?p ?o }} }}", graph_uri);
            match rdf_store.query(&check_query) {
                oxigraph::sparql::QueryResults::Boolean(exists) => {
                    if !exists {
                        missing_graphs.push(graph_uri);
                    }
                }
                _ => {
                    missing_graphs.push(format!("Failed to check graph: {}", graph_uri));
                }
            }
        }

        Ok(missing_graphs)
    }

    /// Extract graph references from a SPARQL query
    fn extract_graph_references(&self, query: &str) -> Vec<String> {
        let mut graph_refs = Vec::new();

        // Simple regex-like extraction for GRAPH <uri> patterns
        let _query_lower = query.to_lowercase();
        let chars: Vec<char> = query.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for "graph <" pattern
            if i + 6 < chars.len() {
                let slice: String = chars[i..i + 6].iter().collect::<String>().to_lowercase();
                if slice == "graph " {
                    // Skip to find the opening <
                    let mut j = i + 6;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }

                    if j < chars.len() && chars[j] == '<' {
                        // Extract URI until closing >
                        j += 1; // Skip opening <
                        let start = j;
                        while j < chars.len() && chars[j] != '>' {
                            j += 1;
                        }

                        if j < chars.len() && chars[j] == '>' {
                            let uri: String = chars[start..j].iter().collect();
                            graph_refs.push(uri);
                        }
                    }
                }
            }
            i += 1;
        }

        graph_refs
    }

    /// Get expected blockchain graphs based on metadata
    fn get_expected_blockchain_graphs(&self, rdf_store: &RDFStore) -> Result<Vec<String>> {
        let mut expected_graphs = Vec::new();

        // Always expect the blockchain metadata graph
        expected_graphs.push("http://provchain.org/blockchain".to_string());

        // Query for all block indices to determine expected block graphs
        let block_query = "SELECT ?index WHERE { GRAPH <http://provchain.org/blockchain> { ?block <http://provchain.org/hasIndex> ?index } } ORDER BY ?index";

        match rdf_store.query(block_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions.flatten() {
                    if let Some(index_term) = solution.get("index") {
                        if let oxigraph::model::Term::Literal(literal) = index_term {
                            if let Ok(index) = literal.value().parse::<u64>() {
                                expected_graphs
                                    .push(format!("http://provchain.org/block/{}", index));
                            }
                        }
                    }
                }
            }
            _ => {
                warn!("Failed to query block indices for expected graphs");
            }
        }

        Ok(expected_graphs)
    }
}

/// Query performance test result
#[derive(Debug, Clone)]
pub struct QueryPerformanceResult {
    pub query: String,
    pub execution_time_ms: u128,
    pub result_count: usize,
    pub memory_usage_bytes: usize,
    pub timed_out: bool,
}

/// SPARQL validation statistics
#[derive(Debug, Clone)]
pub struct SparqlValidationStatistics {
    pub total_queries: usize,
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub total_missing_graphs: usize,
    pub total_inaccessible_data: usize,
    pub graph_accessibility_issues: usize,
    pub canonicalization_mismatches: usize,
    pub success_rate: f64,
}

impl Default for SparqlConsistencyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::rdf_store::RDFStore;

    #[test]
    fn test_sparql_validator_creation() {
        let validator = SparqlConsistencyValidator::new();
        assert!(!validator.verbose_logging);
        assert_eq!(validator.max_query_time, 30);
        assert!(!validator.test_queries.is_empty());
    }

    #[test]
    fn test_sparql_validator_with_config() {
        let custom_queries = vec!["SELECT * WHERE { ?s ?p ?o }".to_string()];
        let validator = SparqlConsistencyValidator::with_config(true, 60, custom_queries.clone());
        assert!(validator.verbose_logging);
        assert_eq!(validator.max_query_time, 60);
        assert_eq!(validator.test_queries, custom_queries);
    }

    #[test]
    fn test_default_test_queries() {
        let queries = SparqlConsistencyValidator::default_test_queries();
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.contains("COUNT")));
        assert!(queries.iter().any(|q| q.contains("GRAPH")));
    }

    #[test]
    fn test_validate_query_result_consistency_basic() {
        let validator = SparqlConsistencyValidator::new();
        let rdf_store = RDFStore::new();
        let test_queries = vec!["SELECT (COUNT(*) as ?count) WHERE { ?s ?p ?o }".to_string()];

        let result = validator.validate_query_result_consistency(&rdf_store, &test_queries);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
