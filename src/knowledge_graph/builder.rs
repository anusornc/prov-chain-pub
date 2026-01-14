//! Graph Builder Pipeline for Knowledge Graph Construction
//!
//! This module provides automated RDF graph generation from blockchain data,
//! entity extraction and classification, and relationship discovery.

use super::{KnowledgeEntity, KnowledgeGraph, KnowledgeRelationship};
use crate::storage::rdf_store::RDFStore;
use anyhow::Result;
use chrono::{DateTime, Utc};
use oxigraph::model::*;
use oxigraph::sparql::QueryResults;
use std::collections::HashMap;

/// Graph builder for constructing knowledge graphs from RDF data
pub struct GraphBuilder {
    rdf_store: RDFStore,
    entity_extractors: Vec<Box<dyn EntityExtractor>>,
    relationship_extractors: Vec<Box<dyn RelationshipExtractor>>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new(rdf_store: RDFStore) -> Self {
        let mut builder = Self {
            rdf_store,
            entity_extractors: Vec::new(),
            relationship_extractors: Vec::new(),
        };

        // Register default extractors
        builder.register_default_extractors();
        builder
    }

    /// Register default entity and relationship extractors
    fn register_default_extractors(&mut self) {
        self.entity_extractors.push(Box::new(ProductBatchExtractor));
        self.entity_extractors.push(Box::new(AgentExtractor));
        self.entity_extractors.push(Box::new(ActivityExtractor));

        self.relationship_extractors
            .push(Box::new(ProvenanceRelationshipExtractor));
        self.relationship_extractors
            .push(Box::new(TemporalRelationshipExtractor));
        self.relationship_extractors
            .push(Box::new(SupplyChainRelationshipExtractor));
    }

    /// Build a complete knowledge graph from all blockchain data
    pub fn build_knowledge_graph(&self) -> Result<KnowledgeGraph> {
        let mut kg = KnowledgeGraph::new();

        // Extract entities
        for extractor in &self.entity_extractors {
            let entities = extractor.extract_entities(&self.rdf_store)?;
            for entity in entities {
                kg.add_entity(entity)?;
            }
        }

        // Extract relationships
        for extractor in &self.relationship_extractors {
            let relationships = extractor.extract_relationships(&self.rdf_store)?;
            for relationship in relationships {
                kg.add_relationship(relationship)?;
            }
        }

        Ok(kg)
    }

    /// Update knowledge graph with new block data
    pub fn update_with_block(&self, kg: &mut KnowledgeGraph, block_index: usize) -> Result<()> {
        let graph_name = NamedNode::new(format!("http://provchain.org/block/{block_index}"))?;

        // Extract entities from the specific block
        for extractor in &self.entity_extractors {
            let entities = extractor.extract_entities_from_graph(&self.rdf_store, &graph_name)?;
            for entity in entities {
                kg.add_entity(entity)?;
            }
        }

        // Extract relationships from the specific block
        for extractor in &self.relationship_extractors {
            let relationships =
                extractor.extract_relationships_from_graph(&self.rdf_store, &graph_name)?;
            for relationship in relationships {
                kg.add_relationship(relationship)?;
            }
        }

        Ok(())
    }

    /// Get temporal evolution of the knowledge graph
    pub fn get_temporal_evolution(&self) -> Result<Vec<(DateTime<Utc>, KnowledgeGraph)>> {
        let mut evolution = Vec::new();
        let mut cumulative_kg = KnowledgeGraph::new();

        // Query for all blocks with timestamps
        let query = r#"
            PREFIX pc: <http://provchain.org/>
            SELECT ?block ?timestamp WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block pc:hasTimestamp ?timestamp .
                    ?block pc:hasIndex ?index .
                }
            }
            ORDER BY ?index
        "#;

        if let QueryResults::Solutions(solutions) = self.rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(block), Some(timestamp)) = (sol.get("block"), sol.get("timestamp")) {
                    // Parse timestamp
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&timestamp.to_string()) {
                        let utc_dt = dt.with_timezone(&Utc);

                        // Extract block index from URI
                        let block_uri = block.to_string();
                        if let Some(index_str) = block_uri.split('/').next_back() {
                            if let Ok(block_index) = index_str.parse::<usize>() {
                                // Update cumulative knowledge graph
                                self.update_with_block(&mut cumulative_kg, block_index)?;

                                // Clone the current state
                                let snapshot = KnowledgeGraph {
                                    entities: cumulative_kg.entities.clone(),
                                    relationships: cumulative_kg.relationships.clone(),
                                    graph: cumulative_kg.graph.clone(),
                                    entity_index: cumulative_kg.entity_index.clone(),
                                };

                                evolution.push((utc_dt, snapshot));
                            }
                        }
                    }
                }
            }
        }

        Ok(evolution)
    }
}

/// Trait for extracting entities from RDF data
pub trait EntityExtractor: Send + Sync {
    fn extract_entities(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeEntity>>;

    fn extract_entities_from_graph(
        &self,
        rdf_store: &RDFStore,
        _graph: &NamedNode,
    ) -> Result<Vec<KnowledgeEntity>> {
        // Default implementation extracts from all graphs
        self.extract_entities(rdf_store)
    }
}

/// Trait for extracting relationships from RDF data
pub trait RelationshipExtractor: Send + Sync {
    fn extract_relationships(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeRelationship>>;

    fn extract_relationships_from_graph(
        &self,
        rdf_store: &RDFStore,
        _graph: &NamedNode,
    ) -> Result<Vec<KnowledgeRelationship>> {
        // Default implementation extracts from all graphs
        self.extract_relationships(rdf_store)
    }
}

/// Extractor for ProductBatch entities
pub struct ProductBatchExtractor;

impl EntityExtractor for ProductBatchExtractor {
    fn extract_entities(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeEntity>> {
        let query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            
            SELECT ?batch ?batchId ?label ?producedAt WHERE {
                GRAPH ?g {
                    ?batch a trace:ProductBatch .
                    OPTIONAL { ?batch trace:hasBatchID ?batchId }
                    OPTIONAL { ?batch rdfs:label ?label }
                    OPTIONAL { ?batch trace:producedAt ?producedAt }
                }
            }
        "#;

        let mut entities = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let Some(batch) = sol.get("batch") {
                    let mut properties = HashMap::new();

                    if let Some(batch_id) = sol.get("batchId") {
                        properties.insert("batchId".to_string(), batch_id.to_string());
                    }

                    if let Some(produced_at) = sol.get("producedAt") {
                        properties.insert("producedAt".to_string(), produced_at.to_string());
                    }

                    let entity = KnowledgeEntity {
                        uri: batch
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        entity_type: "ProductBatch".to_string(),
                        label: sol.get("label").map(|l| l.to_string()),
                        properties,
                        confidence_score: 0.9,
                    };

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}

/// Extractor for Agent entities (Farmers, Manufacturers, etc.)
pub struct AgentExtractor;

impl EntityExtractor for AgentExtractor {
    fn extract_entities(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeEntity>> {
        let query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            
            SELECT ?agent ?type ?label WHERE {
                GRAPH ?g {
                    ?agent a ?type .
                    OPTIONAL { ?agent rdfs:label ?label }
                    FILTER(?type = trace:Farmer || ?type = trace:Manufacturer || 
                           ?type = trace:LogisticsProvider || ?type = trace:Retailer || 
                           ?type = trace:Customer)
                }
            }
        "#;

        let mut entities = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(agent), Some(agent_type)) = (sol.get("agent"), sol.get("type")) {
                    let type_name = agent_type
                        .to_string()
                        .trim_matches(|c| c == '<' || c == '>')
                        .split('#')
                        .next_back()
                        .unwrap_or("Agent")
                        .to_string();

                    let entity = KnowledgeEntity {
                        uri: agent
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        entity_type: type_name,
                        label: sol.get("label").map(|l| l.to_string()),
                        properties: HashMap::new(),
                        confidence_score: 0.95,
                    };

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}

/// Extractor for Activity entities
pub struct ActivityExtractor;

impl EntityExtractor for ActivityExtractor {
    fn extract_entities(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeEntity>> {
        let query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            
            SELECT ?activity ?type ?recordedAt WHERE {
                GRAPH ?g {
                    ?activity a ?type .
                    ?activity a trace:TraceActivity .
                    OPTIONAL { ?activity trace:recordedAt ?recordedAt }
                    FILTER(?type = trace:ProcessingActivity || ?type = trace:TransportActivity || 
                           ?type = trace:QualityCheck)
                }
            }
        "#;

        let mut entities = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(activity), Some(activity_type)) =
                    (sol.get("activity"), sol.get("type"))
                {
                    let mut properties = HashMap::new();

                    if let Some(recorded_at) = sol.get("recordedAt") {
                        properties.insert("recordedAt".to_string(), recorded_at.to_string());
                    }

                    let type_name = activity_type
                        .to_string()
                        .split('#')
                        .next_back()
                        .unwrap_or("Activity")
                        .to_string();

                    let entity = KnowledgeEntity {
                        uri: activity
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        entity_type: type_name,
                        label: None,
                        properties,
                        confidence_score: 0.9,
                    };

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}

/// Extractor for PROV-O relationships
pub struct ProvenanceRelationshipExtractor;

impl RelationshipExtractor for ProvenanceRelationshipExtractor {
    fn extract_relationships(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeRelationship>> {
        let query = r#"
            PREFIX prov: <http://www.w3.org/ns/prov#>
            
            SELECT ?subject ?predicate ?object WHERE {
                GRAPH ?g {
                    ?subject ?predicate ?object .
                    FILTER(?predicate = prov:wasGeneratedBy || ?predicate = prov:used || 
                           ?predicate = prov:wasAssociatedWith || ?predicate = prov:wasDerivedFrom ||
                           ?predicate = prov:wasAttributedTo)
                }
            }
        "#;

        let mut relationships = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(subject), Some(predicate), Some(object)) =
                    (sol.get("subject"), sol.get("predicate"), sol.get("object"))
                {
                    let relationship = KnowledgeRelationship {
                        subject: subject
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        predicate: predicate
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        object: object
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        confidence_score: 0.95,
                        temporal_info: None,
                    };

                    relationships.push(relationship);
                }
            }
        }

        Ok(relationships)
    }
}

/// Extractor for temporal relationships
pub struct TemporalRelationshipExtractor;

impl RelationshipExtractor for TemporalRelationshipExtractor {
    fn extract_relationships(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeRelationship>> {
        let query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX prov: <http://www.w3.org/ns/prov#>
            
            SELECT ?activity1 ?activity2 ?time1 ?time2 WHERE {
                GRAPH ?g {
                    ?activity1 trace:recordedAt ?time1 .
                    ?activity2 trace:recordedAt ?time2 .
                    FILTER(?activity1 != ?activity2 && ?time1 < ?time2)
                }
            }
        "#;

        let mut relationships = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(activity1), Some(activity2), Some(time2)) =
                    (sol.get("activity1"), sol.get("activity2"), sol.get("time2"))
                {
                    let temporal_info = DateTime::parse_from_rfc3339(&time2.to_string())
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok();

                    let relationship = KnowledgeRelationship {
                        subject: activity1
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        predicate: "http://provchain.org/trace#precedes".to_string(),
                        object: activity2
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        confidence_score: 0.8,
                        temporal_info,
                    };

                    relationships.push(relationship);
                }
            }
        }

        Ok(relationships)
    }
}

/// Extractor for supply chain specific relationships
pub struct SupplyChainRelationshipExtractor;

impl RelationshipExtractor for SupplyChainRelationshipExtractor {
    fn extract_relationships(&self, rdf_store: &RDFStore) -> Result<Vec<KnowledgeRelationship>> {
        let query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            
            SELECT ?subject ?predicate ?object WHERE {
                GRAPH ?g {
                    ?subject ?predicate ?object .
                    FILTER(?predicate = trace:lotDerivedFrom || ?predicate = trace:hasCondition || 
                           ?predicate = trace:hasCertificate)
                }
            }
        "#;

        let mut relationships = Vec::new();
        if let QueryResults::Solutions(solutions) = rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(subject), Some(predicate), Some(object)) =
                    (sol.get("subject"), sol.get("predicate"), sol.get("object"))
                {
                    let relationship = KnowledgeRelationship {
                        subject: subject
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        predicate: predicate
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        object: object
                            .to_string()
                            .trim_matches(|c| c == '<' || c == '>')
                            .to_string(),
                        confidence_score: 0.9,
                        temporal_info: None,
                    };

                    relationships.push(relationship);
                }
            }
        }

        Ok(relationships)
    }
}
