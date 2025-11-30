//! Knowledge Graph module for Phase 3
//!
//! This module provides advanced knowledge graph construction, entity linking,
//! and graph analytics capabilities for the provenance blockchain system.

pub mod builder;
pub mod entity_linking;
pub mod graph_db;

use anyhow::Result;
use petgraph::Graph;
use std::collections::HashMap;

/// Represents an entity in the knowledge graph
#[derive(Debug, Clone)]
pub struct KnowledgeEntity {
    pub uri: String,
    pub entity_type: String,
    pub label: Option<String>,
    pub properties: HashMap<String, String>,
    pub confidence_score: f64,
}

/// Represents a relationship between entities
#[derive(Debug, Clone)]
pub struct KnowledgeRelationship {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence_score: f64,
    pub temporal_info: Option<chrono::DateTime<chrono::Utc>>,
}

/// Main knowledge graph structure
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub entities: HashMap<String, KnowledgeEntity>,
    pub relationships: Vec<KnowledgeRelationship>,
    pub graph: Graph<String, String>,
    pub entity_index: HashMap<String, petgraph::graph::NodeIndex>,
}

impl KnowledgeGraph {
    /// Create a new empty knowledge graph
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            relationships: Vec::new(),
            graph: Graph::new(),
            entity_index: HashMap::new(),
        }
    }

    /// Add an entity to the knowledge graph
    pub fn add_entity(&mut self, entity: KnowledgeEntity) -> Result<()> {
        let node_index = self.graph.add_node(entity.uri.clone());
        self.entity_index.insert(entity.uri.clone(), node_index);
        self.entities.insert(entity.uri.clone(), entity);
        Ok(())
    }

    /// Add a relationship to the knowledge graph
    pub fn add_relationship(&mut self, relationship: KnowledgeRelationship) -> Result<()> {
        // Ensure both entities exist
        if !self.entities.contains_key(&relationship.subject) {
            let entity = KnowledgeEntity {
                uri: relationship.subject.clone(),
                entity_type: "Unknown".to_string(),
                label: None,
                properties: HashMap::new(),
                confidence_score: 0.5,
            };
            self.add_entity(entity)?;
        }

        if !self.entities.contains_key(&relationship.object) {
            let entity = KnowledgeEntity {
                uri: relationship.object.clone(),
                entity_type: "Unknown".to_string(),
                label: None,
                properties: HashMap::new(),
                confidence_score: 0.5,
            };
            self.add_entity(entity)?;
        }

        // Add edge to graph
        if let (Some(&subject_idx), Some(&object_idx)) = (
            self.entity_index.get(&relationship.subject),
            self.entity_index.get(&relationship.object),
        ) {
            self.graph
                .add_edge(subject_idx, object_idx, relationship.predicate.clone());
        }

        self.relationships.push(relationship);
        Ok(())
    }

    /// Get entities by type
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<&KnowledgeEntity> {
        self.entities
            .values()
            .filter(|entity| entity.entity_type == entity_type)
            .collect()
    }

    /// Get relationships for an entity
    pub fn get_entity_relationships(&self, entity_uri: &str) -> Vec<&KnowledgeRelationship> {
        self.relationships
            .iter()
            .filter(|rel| rel.subject == entity_uri || rel.object == entity_uri)
            .collect()
    }

    /// Calculate graph statistics
    pub fn get_statistics(&self) -> KnowledgeGraphStats {
        let mut entity_type_counts = HashMap::new();
        for entity in self.entities.values() {
            *entity_type_counts
                .entry(entity.entity_type.clone())
                .or_insert(0) += 1;
        }

        KnowledgeGraphStats {
            total_entities: self.entities.len(),
            total_relationships: self.relationships.len(),
            entity_type_counts,
            average_confidence: self
                .entities
                .values()
                .map(|e| e.confidence_score)
                .sum::<f64>()
                / self.entities.len() as f64,
        }
    }
}

/// Statistics about the knowledge graph
#[derive(Debug, serde::Serialize)]
pub struct KnowledgeGraphStats {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub entity_type_counts: HashMap<String, usize>,
    pub average_confidence: f64,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}
