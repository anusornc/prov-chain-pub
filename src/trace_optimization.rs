//! Trace optimization module implementing SSSP-inspired concepts
//! for enhanced supply chain traceability in ProvChainOrg
//!
//! This module applies frontier reduction and pivot selection concepts
//! from single-source shortest path algorithms to optimize RDF graph
//! traversal for supply chain traceability queries.

use crate::core::blockchain::Blockchain;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Represents a frontier in traceability graph traversal
#[derive(Debug, Clone)]
pub struct TraceFrontier {
    /// Current entities being explored
    pub current_entities: HashSet<String>,
    /// Entities already visited to avoid cycles
    pub visited_entities: HashSet<String>,
    /// Distance from origin (in hops)
    pub boundary_distance: u32,
    /// Connectivity scores for frontier prioritization
    pub connectivity_scores: HashMap<String, f64>,
}

impl TraceFrontier {
    /// Create a new trace frontier starting from a seed entity
    pub fn new(seed_entity: &str) -> Self {
        let mut current_entities = HashSet::new();
        current_entities.insert(seed_entity.to_string());

        TraceFrontier {
            current_entities,
            visited_entities: HashSet::new(),
            boundary_distance: 0,
            connectivity_scores: HashMap::new(),
        }
    }

    /// Check if frontier is empty
    pub fn is_empty(&self) -> bool {
        self.current_entities.is_empty()
    }

    /// Get size of current frontier
    pub fn size(&self) -> usize {
        self.current_entities.len()
    }

    /// Add entities to the frontier
    pub fn add_entities(&mut self, entities: Vec<String>) {
        for entity in entities {
            self.current_entities.insert(entity);
        }
    }

    /// Mark entities as visited
    pub fn mark_visited(&mut self, entities: &[String]) {
        for entity in entities {
            self.visited_entities.insert(entity.clone());
        }
    }
}

/// Trace pivot selector for identifying key traceability entities
pub struct TracePivotSelector<'a> {
    pub blockchain: &'a Blockchain,
}

impl<'a> TracePivotSelector<'a> {
    /// Create a new pivot selector
    pub fn new(blockchain: &'a Blockchain) -> Self {
        TracePivotSelector { blockchain }
    }

    /// Find pivot entities in the current frontier
    /// Pivots are entities with high connectivity to unexplored parts of the graph
    pub fn find_pivots(&self, frontier: &TraceFrontier, _target_entity: &str) -> HashSet<String> {
        let mut pivots = HashSet::new();
        let mut scores = HashMap::new();

        // Calculate connectivity score for each entity in frontier
        for entity in &frontier.current_entities {
            if frontier.visited_entities.contains(entity) {
                continue;
            }

            let connectivity = self.calculate_connectivity_score(entity, frontier);
            scores.insert(entity.clone(), connectivity);

            // Select as pivot if connectivity exceeds threshold
            if connectivity > 2.0 {
                pivots.insert(entity.clone());
            }
        }

        // If no high-connectivity pivots found, select top 20% by score
        if pivots.is_empty() && !scores.is_empty() {
            let mut sorted_scores: Vec<_> = scores.iter().collect();
            sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            let pivot_count = (sorted_scores.len() as f64 * 0.2).ceil() as usize;
            for (entity, _) in sorted_scores.into_iter().take(pivot_count) {
                pivots.insert(entity.clone());
            }
        }

        pivots
    }

    /// Calculate connectivity score for an entity
    /// Higher scores indicate entities that connect to many unexplored nodes
    fn calculate_connectivity_score(&self, entity: &str, frontier: &TraceFrontier) -> f64 {
        let mut score = 0.0;

        // SPARQL query to find connected entities
        let query = format!(
            r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            
            SELECT ?connected WHERE {{
                {{ <{}> ?p ?connected . }}
                UNION
                {{ ?connected ?p <{}> . }}
                FILTER(isIRI(?connected))
            }}
            "#,
            entity, entity
        );

        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            self.blockchain.rdf_store.query(&query)
        {
            let mut connected_count = 0;
            for sol in solutions.flatten() {
                if let Some(connected_term) = sol.get("connected") {
                    let connected_str = connected_term.to_string();
                    // Count only unvisited entities
                    if !frontier.visited_entities.contains(&connected_str) {
                        connected_count += 1;
                    }
                }
            }
            score = connected_count as f64;
        }

        score
    }
}

/// Enhanced trace result with optimization metadata
#[derive(Debug, Clone, serde::Serialize)]
pub struct EnhancedTraceResult {
    /// The trace path through the supply chain
    pub path: Vec<TraceEvent>,
    /// Whether optimization was applied
    pub optimized: bool,
    /// Performance improvement factor (if measured)
    pub performance_improvement: Option<f64>,
    /// Number of entities explored (for complexity analysis)
    pub entities_explored: usize,
    /// Time taken for trace operation
    pub execution_time_ms: u128,
}

/// Represents a single event in the trace path
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceEvent {
    /// The entity involved in this trace event
    pub entity: String,
    /// The relationship/predicate connecting to this entity
    pub relationship: String,
    /// The source entity that led to this event
    pub source: Option<String>,
    /// Timestamp of the event
    pub timestamp: Option<String>,
    /// Additional metadata about the entity
    pub metadata: HashMap<String, String>,
}

/// Enhanced traceability system applying SSSP concepts
pub struct EnhancedTraceabilitySystem<'a> {
    pub blockchain: &'a Blockchain,
}

impl<'a> EnhancedTraceabilitySystem<'a> {
    /// Create a new enhanced traceability system
    pub fn new(blockchain: &'a Blockchain) -> Self {
        EnhancedTraceabilitySystem { blockchain }
    }

    /// Enhanced trace function applying frontier reduction and pivot selection
    pub fn enhanced_trace(&self, batch_id: &str, optimization_level: u8) -> EnhancedTraceResult {
        let start_time = Instant::now();
        let mut trace_path = Vec::new();
        let mut entities_explored = 0;

        // Initialize frontier with the target batch
        let mut frontier = TraceFrontier::new(&format!("http://example.org/batch{}", batch_id));
        let mut visited = HashSet::new();

        // Create pivot selector
        let pivot_selector = TracePivotSelector::new(self.blockchain);

        // Maximum trace depth to prevent infinite loops
        const MAX_TRACE_DEPTH: usize = 50;
        let mut current_depth = 0;

        while !frontier.is_empty() && current_depth < MAX_TRACE_DEPTH {
            current_depth += 1;

            // Apply frontier reduction if optimization level > 0
            if optimization_level > 0 && frontier.size() > 10 {
                frontier = self.reduce_frontier(&frontier, optimization_level);
            }

            // Find and prioritize pivots if optimization level > 1
            if optimization_level > 1 && current_depth % 3 == 0 {
                let pivots = pivot_selector.find_pivots(&frontier, batch_id);
                frontier = self.prioritize_pivots(frontier, &pivots);
            }

            // Explore current frontier
            let (new_events, next_entities) = self.explore_frontier(&frontier, &visited);
            trace_path.extend(new_events);
            entities_explored += next_entities.len();

            // Update visited set
            visited.extend(frontier.current_entities.iter().cloned());

            // Update frontier for next iteration
            frontier = self.update_frontier(next_entities, visited.clone());
        }

        let execution_time = start_time.elapsed();

        EnhancedTraceResult {
            path: trace_path,
            optimized: optimization_level > 0,
            performance_improvement: None, // Would be calculated in benchmarking
            entities_explored,
            execution_time_ms: execution_time.as_millis(),
        }
    }

    /// Apply frontier reduction to limit the number of entities being explored
    fn reduce_frontier(&self, frontier: &TraceFrontier, optimization_level: u8) -> TraceFrontier {
        let mut reduced = frontier.clone();

        // Reduction factor based on optimization level
        let reduction_factor = match optimization_level {
            1 => 0.8, // Reduce by 20%
            2 => 0.6, // Reduce by 40%
            _ => 0.5, // Reduce by 50%
        };

        let target_size = (frontier.current_entities.len() as f64 * reduction_factor) as usize;

        if target_size < frontier.current_entities.len() && target_size > 0 {
            // Sort entities by connectivity score (higher scores first)
            let mut scored_entities: Vec<_> = frontier
                .current_entities
                .iter()
                .map(|entity| {
                    let score = frontier.connectivity_scores.get(entity).unwrap_or(&0.0);
                    (entity, score)
                })
                .collect();

            scored_entities.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            // Keep top entities based on target size
            reduced.current_entities = scored_entities
                .into_iter()
                .take(target_size)
                .map(|(entity, _)| entity.clone())
                .collect();
        }

        reduced
    }

    /// Prioritize pivot entities in the frontier
    fn prioritize_pivots(
        &self,
        mut frontier: TraceFrontier,
        pivots: &HashSet<String>,
    ) -> TraceFrontier {
        // Boost connectivity scores for pivot entities
        for pivot in pivots {
            if let Some(score) = frontier.connectivity_scores.get_mut(pivot) {
                *score *= 2.0; // Double the score for pivots
            } else {
                frontier.connectivity_scores.insert(pivot.clone(), 10.0);
            }
        }

        frontier
    }

    /// Explore the current frontier and return new trace events
    fn explore_frontier(
        &self,
        frontier: &TraceFrontier,
        visited: &HashSet<String>,
    ) -> (Vec<TraceEvent>, Vec<String>) {
        let mut new_events = Vec::new();
        let mut next_entities = Vec::new();

        for entity in &frontier.current_entities {
            if visited.contains(entity) {
                continue;
            }

            // Query for relationships from this entity
            let outgoing_query = format!(
                r#"
                PREFIX trace: <http://provchain.org/trace#>
                PREFIX prov: <http://www.w3.org/ns/prov#>
                
                SELECT ?target ?predicate ?timestamp WHERE {{
                    <{}> ?predicate ?target .
                    OPTIONAL {{ <{}> trace:recordedAt ?timestamp . }}
                    FILTER(isIRI(?target))
                }}
                "#,
                entity, entity
            );

            if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                self.blockchain.rdf_store.query(&outgoing_query)
            {
                for sol in solutions.flatten() {
                    if let Some(target_term) = sol.get("target") {
                        let target_str = target_term.to_string();

                        // Skip already visited entities
                        if visited.contains(&target_str) {
                            continue;
                        }

                        // Add to next entities for exploration
                        next_entities.push(target_str.clone());

                        // Create trace event
                        let relationship = sol
                            .get("predicate")
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        let timestamp = sol.get("timestamp").map(|t| t.to_string());

                        let mut metadata = HashMap::new();
                        if let Some(ts) = &timestamp {
                            metadata.insert("timestamp".to_string(), ts.clone());
                        }

                        new_events.push(TraceEvent {
                            entity: target_str,
                            relationship,
                            source: Some(entity.clone()),
                            timestamp,
                            metadata,
                        });
                    }
                }
            }
        }

        (new_events, next_entities)
    }

    /// Update frontier for next iteration
    fn update_frontier(
        &self,
        next_entities: Vec<String>,
        visited: HashSet<String>,
    ) -> TraceFrontier {
        let mut new_frontier = TraceFrontier {
            current_entities: next_entities.into_iter().collect(),
            visited_entities: visited,
            boundary_distance: 0, // Would be incremented in a full implementation
            connectivity_scores: HashMap::new(),
        };

        // Calculate connectivity scores for new frontier entities
        for entity in &new_frontier.current_entities {
            let score = self.calculate_entity_connectivity(entity);
            new_frontier
                .connectivity_scores
                .insert(entity.clone(), score);
        }

        new_frontier
    }

    /// Calculate connectivity score for an entity
    fn calculate_entity_connectivity(&self, entity: &str) -> f64 {
        let query = format!(
            r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            
            SELECT (COUNT(?connected) as ?count) WHERE {{
                {{ <{}> ?p ?connected . }}
                UNION
                {{ ?connected ?p <{}> . }}
                FILTER(isIRI(?connected))
            }}
            "#,
            entity, entity
        );

        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            self.blockchain.rdf_store.query(&query)
        {
            for sol in solutions.flatten() {
                if let Some(count_term) = sol.get("count") {
                    if let Ok(count_str) = count_term.to_string().parse::<f64>() {
                        return count_str;
                    }
                }
            }
        }

        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_trace_frontier_creation() {
        let frontier = TraceFrontier::new("http://example.org/batch001");
        assert_eq!(frontier.current_entities.len(), 1);
        assert!(frontier
            .current_entities
            .contains("http://example.org/batch001"));
        assert_eq!(frontier.boundary_distance, 0);
    }

    #[test]
    fn test_frontier_reduction() {
        // This would require a more complex setup with actual blockchain data
        // For now, we'll just verify the structure
        let blockchain = Blockchain::new();
        let system = EnhancedTraceabilitySystem::new(&blockchain);

        let mut frontier = TraceFrontier::new("http://example.org/batch001");
        let entities: Vec<String> = (0..20)
            .map(|i| format!("http://example.org/entity{}", i))
            .collect();
        frontier.add_entities(entities);

        let reduced = system.reduce_frontier(&frontier, 2);
        assert!(reduced.current_entities.len() < frontier.current_entities.len());
    }

    #[test]
    fn test_enhanced_trace_execution() {
        let blockchain = Blockchain::new();
        let system = EnhancedTraceabilitySystem::new(&blockchain);

        // Test with optimization level 0 (no optimization)
        let result = system.enhanced_trace("001", 0);
        assert!(!result.optimized);

        // Test with optimization level 1 (basic optimization)
        let result = system.enhanced_trace("001", 1);
        assert!(result.optimized);
    }
}
