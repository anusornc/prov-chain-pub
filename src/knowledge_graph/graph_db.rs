//! Enhanced Graph Database Operations
//!
//! This module provides advanced graph operations, indexing,
//! and analytics capabilities for the knowledge graph.

use super::{KnowledgeEntity, KnowledgeGraph, KnowledgeRelationship};
use anyhow::Result;
use ndarray::Array2;
use petgraph::algo::{connected_components, dijkstra, is_cyclic_directed};
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

/// Enhanced graph database with advanced operations
pub struct GraphDatabase {
    knowledge_graph: KnowledgeGraph,
    indexes: GraphIndexes,
    embeddings: Option<GraphEmbeddings>,
}

impl GraphDatabase {
    /// Create a new graph database
    pub fn new(knowledge_graph: KnowledgeGraph) -> Self {
        let mut db = Self {
            knowledge_graph,
            indexes: GraphIndexes::new(),
            embeddings: None,
        };

        db.rebuild_indexes();
        db
    }

    /// Rebuild all indexes for efficient querying
    pub fn rebuild_indexes(&mut self) {
        self.indexes = GraphIndexes::new();

        // Build entity type index
        for (uri, entity) in &self.knowledge_graph.entities {
            self.indexes
                .entity_type_index
                .entry(entity.entity_type.clone())
                .or_default()
                .push(uri.clone());
        }

        // Build property index
        for (uri, entity) in &self.knowledge_graph.entities {
            for (property, value) in &entity.properties {
                self.indexes
                    .property_index
                    .entry(property.clone())
                    .or_default()
                    .entry(value.clone())
                    .or_default()
                    .push(uri.clone());
            }
        }

        // Build relationship index
        for relationship in &self.knowledge_graph.relationships {
            self.indexes
                .relationship_index
                .entry(relationship.predicate.clone())
                .or_default()
                .push((relationship.subject.clone(), relationship.object.clone()));
        }
    }

    /// Find shortest path between two entities
    pub fn find_shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let from_idx = self.knowledge_graph.entity_index.get(from)?;
        let to_idx = self.knowledge_graph.entity_index.get(to)?;

        petgraph::algo::astar(
            &self.knowledge_graph.graph,
            *from_idx,
            |finish| finish == *to_idx,
            |_| 1,
            |_| 0,
        )
        .map(|(_cost, path)| {
            path.into_iter()
                .map(|idx| self.knowledge_graph.graph[idx].clone())
                .collect()
        })
    }

    /// Find all paths between two entities up to a maximum length
    pub fn find_all_paths(&self, from: &str, to: &str, max_length: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![from.to_string()];
        let mut visited = HashSet::new();

        self.dfs_paths(
            from,
            to,
            &mut current_path,
            &mut visited,
            &mut paths,
            max_length,
        );
        paths
    }

    /// Depth-first search for finding paths
    fn dfs_paths(
        &self,
        current: &str,
        target: &str,
        current_path: &mut Vec<String>,
        visited: &mut HashSet<String>,
        paths: &mut Vec<Vec<String>>,
        max_length: usize,
    ) {
        if current_path.len() > max_length {
            return;
        }

        if current == target {
            paths.push(current_path.clone());
            return;
        }

        visited.insert(current.to_string());

        // Find outgoing relationships
        for relationship in &self.knowledge_graph.relationships {
            if relationship.subject == current && !visited.contains(&relationship.object) {
                current_path.push(relationship.object.clone());
                self.dfs_paths(
                    &relationship.object,
                    target,
                    current_path,
                    visited,
                    paths,
                    max_length,
                );
                current_path.pop();
            }
        }

        visited.remove(current);
    }

    /// Find entities within a certain distance from a given entity
    pub fn find_neighbors(&self, entity_uri: &str, max_distance: usize) -> Vec<(String, usize)> {
        if let Some(&start_idx) = self.knowledge_graph.entity_index.get(entity_uri) {
            let distances = dijkstra(&self.knowledge_graph.graph, start_idx, None, |_| 1);

            distances
                .into_iter()
                .filter_map(|(node_idx, distance)| {
                    if distance <= max_distance && distance > 0 {
                        // Find entity URI for this node index
                        for (uri, &idx) in &self.knowledge_graph.entity_index {
                            if idx == node_idx {
                                return Some((uri.clone(), distance));
                            }
                        }
                    }
                    None
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate centrality measures for entities
    pub fn calculate_centrality(&self) -> HashMap<String, CentralityMeasures> {
        let mut centrality_map = HashMap::new();
        let graph = &self.knowledge_graph.graph;

        for (uri, &node_idx) in &self.knowledge_graph.entity_index {
            let degree = graph.edges(node_idx).count();
            let in_degree = graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .count();
            let out_degree = graph
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
                .count();

            // Calculate betweenness centrality (simplified)
            let betweenness = self.calculate_betweenness_centrality(node_idx);

            centrality_map.insert(
                uri.clone(),
                CentralityMeasures {
                    degree_centrality: degree as f64,
                    in_degree_centrality: in_degree as f64,
                    out_degree_centrality: out_degree as f64,
                    betweenness_centrality: betweenness,
                    closeness_centrality: self.calculate_closeness_centrality(node_idx),
                },
            );
        }

        centrality_map
    }

    /// Calculate betweenness centrality for a node (simplified implementation)
    fn calculate_betweenness_centrality(&self, node_idx: NodeIndex) -> f64 {
        // Simplified implementation - in practice would use proper algorithm
        let neighbors = self.knowledge_graph.graph.neighbors(node_idx).count();
        neighbors as f64 / (self.knowledge_graph.entities.len() as f64).max(1.0)
    }

    /// Calculate closeness centrality for a node
    fn calculate_closeness_centrality(&self, node_idx: NodeIndex) -> f64 {
        let distances = dijkstra(&self.knowledge_graph.graph, node_idx, None, |_| 1);
        let total_distance: usize = distances.values().sum();

        if total_distance > 0 {
            (distances.len() - 1) as f64 / total_distance as f64
        } else {
            0.0
        }
    }

    /// Detect communities in the graph using simple connected components
    pub fn detect_communities(&self) -> Vec<Vec<String>> {
        let components = connected_components(&self.knowledge_graph.graph);
        let mut communities = vec![Vec::new(); components];

        for (uri, &node_idx) in &self.knowledge_graph.entity_index {
            if let Some(_component_id) = self.knowledge_graph.graph.node_weight(node_idx) {
                // Find which component this node belongs to
                let mut component_idx = 0;
                let mut current_count = 0;

                for (idx, _) in self.knowledge_graph.graph.node_indices().enumerate() {
                    if petgraph::graph::NodeIndex::new(idx) == node_idx {
                        component_idx = current_count % components;
                        break;
                    }
                    current_count += 1;
                }

                if component_idx < communities.len() {
                    communities[component_idx].push(uri.clone());
                }
            }
        }

        communities.into_iter().filter(|c| !c.is_empty()).collect()
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&self) -> bool {
        is_cyclic_directed(&self.knowledge_graph.graph)
    }

    /// Generate graph embeddings using simple node2vec-like approach
    pub fn generate_embeddings(&mut self, dimensions: usize) -> Result<()> {
        let num_nodes = self.knowledge_graph.entities.len();
        let mut embeddings = Array2::zeros((num_nodes, dimensions));

        // Simple random walk-based embedding generation
        for (i, (_uri, &node_idx)) in self.knowledge_graph.entity_index.iter().enumerate() {
            let walks = self.generate_random_walks(node_idx, 10, 80); // 10 walks of length 80
            let embedding = self.compute_embedding_from_walks(&walks, dimensions);

            for (j, &value) in embedding.iter().enumerate() {
                if j < dimensions {
                    embeddings[[i, j]] = value;
                }
            }
        }

        self.embeddings = Some(GraphEmbeddings {
            embeddings,
            entity_to_index: self
                .knowledge_graph
                .entity_index
                .iter()
                .enumerate()
                .map(|(i, (uri, _))| (uri.clone(), i))
                .collect(),
        });

        Ok(())
    }

    /// Generate random walks from a starting node
    fn generate_random_walks(
        &self,
        start_node: NodeIndex,
        num_walks: usize,
        walk_length: usize,
    ) -> Vec<Vec<NodeIndex>> {
        let mut walks = Vec::new();

        for _ in 0..num_walks {
            let mut walk = vec![start_node];
            let mut current = start_node;

            for _ in 1..walk_length {
                let neighbors: Vec<_> = self.knowledge_graph.graph.neighbors(current).collect();
                if neighbors.is_empty() {
                    break;
                }

                // Simple random selection (in practice would use proper random number generation)
                let next_idx = neighbors.len() % neighbors.len();
                current = neighbors[next_idx];
                walk.push(current);
            }

            walks.push(walk);
        }

        walks
    }

    /// Compute embedding from random walks (simplified)
    fn compute_embedding_from_walks(
        &self,
        walks: &[Vec<NodeIndex>],
        dimensions: usize,
    ) -> Vec<f64> {
        let mut embedding = vec![0.0; dimensions];

        // Simple approach: use walk statistics as features
        for (i, walk) in walks.iter().enumerate() {
            if i < dimensions {
                embedding[i] = walk.len() as f64 / 80.0; // Normalize by max walk length
            }
        }

        // Fill remaining dimensions with derived features
        for i in walks.len()..dimensions {
            embedding[i] = (i as f64).sin(); // Simple derived feature
        }

        embedding
    }

    /// Find similar entities based on embeddings
    pub fn find_similar_entities(&self, entity_uri: &str, top_k: usize) -> Vec<(String, f64)> {
        if let Some(ref embeddings) = self.embeddings {
            if let Some(&entity_idx) = embeddings.entity_to_index.get(entity_uri) {
                let entity_embedding = embeddings.embeddings.row(entity_idx);
                let mut similarities = Vec::new();

                for (other_uri, &other_idx) in &embeddings.entity_to_index {
                    if other_uri != entity_uri {
                        let other_embedding = embeddings.embeddings.row(other_idx);
                        let similarity = cosine_similarity(
                            &entity_embedding.to_vec(),
                            &other_embedding.to_vec(),
                        );
                        similarities.push((other_uri.clone(), similarity));
                    }
                }

                similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                similarities.into_iter().take(top_k).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Query entities by type with optional filters
    pub fn query_entities_by_type(
        &self,
        entity_type: &str,
        filters: Option<&HashMap<String, String>>,
    ) -> Vec<&KnowledgeEntity> {
        if let Some(entity_uris) = self.indexes.entity_type_index.get(entity_type) {
            entity_uris
                .iter()
                .filter_map(|uri| self.knowledge_graph.entities.get(uri))
                .filter(|entity| {
                    if let Some(filters) = filters {
                        filters
                            .iter()
                            .all(|(key, value)| entity.properties.get(key) == Some(value))
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get graph statistics
    pub fn get_graph_statistics(&self) -> GraphStatistics {
        let centrality = self.calculate_centrality();
        let communities = self.detect_communities();

        GraphStatistics {
            num_entities: self.knowledge_graph.entities.len(),
            num_relationships: self.knowledge_graph.relationships.len(),
            num_entity_types: self.indexes.entity_type_index.len(),
            num_communities: communities.len(),
            has_cycles: self.has_cycles(),
            average_degree: centrality
                .values()
                .map(|c| c.degree_centrality)
                .sum::<f64>()
                / centrality.len() as f64,
            density: self.calculate_graph_density(),
        }
    }

    /// Calculate graph density
    fn calculate_graph_density(&self) -> f64 {
        let num_nodes = self.knowledge_graph.entities.len() as f64;
        let num_edges = self.knowledge_graph.relationships.len() as f64;

        if num_nodes > 1.0 {
            num_edges / (num_nodes * (num_nodes - 1.0))
        } else {
            0.0
        }
    }

    /// Get entities from the knowledge graph
    pub fn get_entities(&self) -> &HashMap<String, KnowledgeEntity> {
        &self.knowledge_graph.entities
    }

    /// Get relationships from the knowledge graph
    pub fn get_relationships(&self) -> &Vec<KnowledgeRelationship> {
        &self.knowledge_graph.relationships
    }
}

/// Graph indexes for efficient querying
#[derive(Debug)]
pub struct GraphIndexes {
    pub entity_type_index: HashMap<String, Vec<String>>,
    pub property_index: HashMap<String, HashMap<String, Vec<String>>>,
    pub relationship_index: HashMap<String, Vec<(String, String)>>,
}

impl GraphIndexes {
    fn new() -> Self {
        Self {
            entity_type_index: HashMap::new(),
            property_index: HashMap::new(),
            relationship_index: HashMap::new(),
        }
    }
}

/// Graph embeddings for similarity analysis
#[derive(Debug)]
pub struct GraphEmbeddings {
    pub embeddings: Array2<f64>,
    pub entity_to_index: HashMap<String, usize>,
}

/// Centrality measures for an entity
#[derive(Debug, Clone)]
pub struct CentralityMeasures {
    pub degree_centrality: f64,
    pub in_degree_centrality: f64,
    pub out_degree_centrality: f64,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
}

/// Graph statistics
#[derive(Debug, serde::Serialize)]
pub struct GraphStatistics {
    pub num_entities: usize,
    pub num_relationships: usize,
    pub num_entity_types: usize,
    pub num_communities: usize,
    pub has_cycles: bool,
    pub average_degree: f64,
    pub density: f64,
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(vec1: &[f64], vec2: &[f64]) -> f64 {
    if vec1.len() != vec2.len() {
        return 0.0;
    }

    let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot_product / (norm1 * norm2)
    }
}
