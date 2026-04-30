//! Neo4j Client for ProvChain Benchmark Suite
//!
//! This module provides a complete Neo4j client implementation for benchmarking
//! against ProvChain. It handles:
//! - Async connection to Neo4j using neo4rs
//! - RDF/Turtle to Cypher conversion
//! - Query execution with timing
//! - Health checks

use crate::dataset::normalize_turtle;
use anyhow::{Context, Result};
use neo4rs::{query as neo_query, ConfigBuilder, Graph};
use rio_api::parser::TriplesParser;
use rio_turtle::TurtleParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Configuration for Neo4j connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jClientConfig {
    /// Neo4j Bolt URI (e.g., bolt://localhost:7687)
    pub uri: String,
    /// Neo4j username
    pub user: String,
    /// Neo4j password
    pub password: String,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Max connections in pool
    pub max_connections: usize,
}

impl Default for Neo4jClientConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            user: "neo4j".to_string(),
            password: "password".to_string(),
            timeout_secs: 30,
            max_connections: 10,
        }
    }
}

impl Neo4jClientConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            uri: std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string()),
            user: std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string()),
            password: std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            timeout_secs: std::env::var("NEO4J_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_connections: std::env::var("NEO4J_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        }
    }
}

/// Alias for backward compatibility
pub type Neo4jConfig = Neo4jClientConfig;

/// Result of a Neo4j query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jBenchmarkResult {
    /// Query execution time in milliseconds
    pub duration_ms: f64,
    /// Number of records returned
    pub record_count: usize,
    /// Whether the query succeeded
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Query metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Alias for backward compatibility
pub type QueryResult = Neo4jBenchmarkResult;

/// RDF Triple representation for internal processing
#[derive(Debug, Clone)]
struct RdfTriple {
    subject: String,
    predicate: String,
    object: String,
    object_is_literal: bool,
    object_type: Option<String>,
}

/// Neo4j Client for benchmark operations
pub struct Neo4jClient {
    graph: Option<Graph>,
    config: Neo4jClientConfig,
}

impl Neo4jClient {
    /// Create a new Neo4j client with the given configuration
    pub fn new(config: Neo4jClientConfig) -> Self {
        Self {
            graph: None,
            config,
        }
    }

    /// Create a new Neo4j client from environment variables
    pub fn from_env() -> Self {
        Self::new(Neo4jClientConfig::from_env())
    }

    /// Connect to Neo4j database
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Neo4j at {}...", self.config.uri);

        let config = ConfigBuilder::default()
            .uri(&self.config.uri)
            .user(&self.config.user)
            .password(&self.config.password)
            .max_connections(self.config.max_connections)
            .build()?;

        let graph = Graph::connect(config)
            .await
            .context("Failed to connect to Neo4j")?;

        // Test connection
        let query = neo_query("RETURN 1 as test");
        let mut result = graph.execute(query).await?;

        if result.next().await?.is_some() {
            info!("✓ Neo4j connection successful");
        } else {
            anyhow::bail!("Neo4j connection test failed");
        }

        self.graph = Some(graph);
        Ok(())
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        self.graph.is_some()
    }

    /// Perform health check on Neo4j connection
    pub async fn health_check(&self) -> Result<bool> {
        if let Some(graph) = &self.graph {
            let query =
                neo_query("CALL dbms.components() YIELD name, edition RETURN name, edition");
            let mut result = graph.execute(query).await?;

            if let Some(row) = result.next().await? {
                let name: String = row.get("name").unwrap_or_default();
                let edition: String = row.get("edition").unwrap_or_default();
                info!("Neo4j Health Check: {} ({})", name, edition);
                Ok(true)
            } else {
                warn!("Neo4j health check returned no results");
                Ok(false)
            }
        } else {
            Err(anyhow::anyhow!("Not connected to Neo4j"))
        }
    }

    /// Clear all data from Neo4j (for benchmark cleanup)
    pub async fn clear_all_data(&self) -> Result<()> {
        if let Some(graph) = &self.graph {
            info!("Clearing all data from Neo4j...");
            let query = neo_query("MATCH (n) DETACH DELETE n");
            let _ = graph.execute(query).await?;
            info!("✓ All data cleared");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected to Neo4j"))
        }
    }

    /// Load RDF/Turtle data into Neo4j by converting to Cypher
    pub async fn load_turtle_data(&self, ttl_path: &Path) -> Result<Duration> {
        info!("Loading Turtle data from {:?}...", ttl_path);

        let start = Instant::now();

        // Read the Turtle file
        let mut file = std::fs::File::open(ttl_path)
            .with_context(|| format!("Failed to open Turtle file: {:?}", ttl_path))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let content = normalize_turtle(&content);

        // Parse RDF triples
        let triples = self.parse_turtle(&content)?;
        info!("Parsed {} triples from Turtle file", triples.len());

        // Convert to Cypher and load
        self.load_triples_as_cypher(triples).await?;

        let duration = start.elapsed();
        info!("✓ Loaded Turtle data in {:?}", duration);

        Ok(duration)
    }

    /// Parse Turtle content into RDF triples
    fn parse_turtle(&self, content: &str) -> Result<Vec<RdfTriple>> {
        let mut triples = Vec::new();

        let mut parser = TurtleParser::new(content.as_bytes(), None);

        parser
            .parse_all(&mut |t| {
                let subject = Self::format_subject(&t.subject);
                let predicate = t.predicate.iri.to_string();
                let (object, is_literal, obj_type) = Self::format_object(&t.object);

                triples.push(RdfTriple {
                    subject,
                    predicate,
                    object,
                    object_is_literal: is_literal,
                    object_type: obj_type,
                });

                Ok(()) as Result<(), rio_turtle::TurtleError>
            })
            .map_err(|e| anyhow::anyhow!("RDF parsing error: {:?}", e))?;

        Ok(triples)
    }

    /// Format RDF subject to string
    fn format_subject(subject: &rio_api::model::Subject) -> String {
        match subject {
            rio_api::model::Subject::NamedNode(n) => n.iri.to_string(),
            rio_api::model::Subject::BlankNode(b) => format!("_:{}", b.id),
            _ => "unknown".to_string(),
        }
    }

    /// Format RDF object to string, returning (value, is_literal, datatype)
    fn format_object(object: &rio_api::model::Term) -> (String, bool, Option<String>) {
        match object {
            rio_api::model::Term::NamedNode(n) => (n.iri.to_string(), false, None),
            rio_api::model::Term::BlankNode(b) => (format!("_:{}", b.id), false, None),
            rio_api::model::Term::Literal(l) => {
                // Handle different literal types in rio_api
                let (value, dtype) = match l {
                    rio_api::model::Literal::Simple { value } => (value.to_string(), None),
                    rio_api::model::Literal::LanguageTaggedString { value, language: _ } => {
                        (value.to_string(), None)
                    }
                    rio_api::model::Literal::Typed { value, datatype } => {
                        (value.to_string(), Some(datatype.iri.to_string()))
                    }
                };
                (value, true, dtype)
            }
            _ => ("unknown".to_string(), true, None),
        }
    }

    /// Load RDF triples into Neo4j using Cypher
    async fn load_triples_as_cypher(&self, triples: Vec<RdfTriple>) -> Result<()> {
        if let Some(graph) = &self.graph {
            // Group triples by subject for efficient loading
            let mut subject_groups: HashMap<String, Vec<RdfTriple>> = HashMap::new();

            for triple in triples {
                subject_groups
                    .entry(triple.subject.clone())
                    .or_default()
                    .push(triple);
            }

            // Process in batches. Larger scale slices can exceed the default
            // Neo4j heap when too many subject MERGEs are concatenated into one
            // Cypher request, so keep this tunable from the campaign wrapper.
            let batch_size = std::env::var("NEO4J_LOAD_BATCH_SIZE")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(100);
            info!("Loading Neo4j triples with subject batch size {}", batch_size);
            let subjects: Vec<_> = subject_groups.keys().cloned().collect();

            for chunk in subjects.chunks(batch_size) {
                let mut query_builder = String::new();

                for (i, subject) in chunk.iter().enumerate() {
                    if let Some(triples) = subject_groups.get(subject) {
                        // Build Cypher for this subject
                        let cypher = self.build_merge_cypher(subject, triples, i);
                        query_builder.push_str(&cypher);
                    }
                }

                if !query_builder.is_empty() {
                    let query = neo_query(&query_builder);
                    let _ = graph.execute(query).await?;
                }
            }

            // Create indexes for common query patterns
            self.create_indexes().await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected to Neo4j"))
        }
    }

    /// Build a MERGE Cypher statement for a subject and its triples
    fn build_merge_cypher(&self, subject: &str, triples: &[RdfTriple], idx: usize) -> String {
        let node_var = format!("n{}", idx);
        let mut cypher = String::new();

        // Extract type and properties
        let mut types = Vec::new();
        let mut properties = Vec::new();
        let mut relationships = Vec::new();

        for triple in triples {
            let pred_short = Self::shorten_iri(&triple.predicate);

            if pred_short == "rdf:type" || pred_short == "a" {
                types.push(Self::shorten_iri(&triple.object));
            } else if triple.object_is_literal {
                // Property
                let value = Self::escape_cypher_string(&triple.object);
                let property_key = Self::property_key(&pred_short);
                properties.push(format!("`{}`: '{}'", property_key, value));
            } else {
                // Relationship
                relationships.push((pred_short, triple.object.clone()));
            }
        }

        // Build MERGE for node
        let label = if types.is_empty() {
            "Resource".to_string()
        } else {
            types.join(":")
        };

        let safe_subject = Self::escape_cypher_string(subject);
        cypher.push_str(&format!(
            "MERGE ({}:`{}` {{uri: '{}'}})\n",
            node_var, label, safe_subject
        ));

        // Set properties
        if !properties.is_empty() {
            cypher.push_str(&format!(
                "SET {} += {{{}}}\n",
                node_var,
                properties.join(", ")
            ));
        }

        // Build relationships
        for (rel_type, target) in relationships {
            let safe_target = Self::escape_cypher_string(&target);
            let rel_var = format!("r{}_{}", idx, rel_type.replace(':', "_"));
            cypher.push_str(&format!(
                "MERGE ({}_{}:`Resource` {{uri: '{}'}})\n",
                node_var, rel_var, safe_target
            ));
            cypher.push_str(&format!(
                "MERGE ({})-[:`{}`]->({}_{})\n",
                node_var, rel_type, node_var, rel_var
            ));
        }

        cypher
    }

    /// Shorten IRI to prefix form if possible
    fn shorten_iri(iri: &str) -> String {
        // Common namespace prefixes
        let prefixes = [
            ("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:"),
            ("http://www.w3.org/2000/01/rdf-schema#", "rdfs:"),
            ("http://www.w3.org/2001/XMLSchema#", "xsd:"),
            ("http://www.w3.org/ns/prov#", "prov:"),
            ("http://example.org/supplychain/", "ex:"),
            ("http://example.org/traceability#", "trace:"),
            ("http://example.org/food/", "food:"),
            ("http://example.org/permissions#", "perm:"),
        ];

        for (full, prefix) in &prefixes {
            if iri.starts_with(full) {
                return format!("{}{}", prefix, &iri[full.len()..]);
            }
        }

        iri.to_string()
    }

    fn property_key(pred_short: &str) -> String {
        pred_short
            .rsplit_once(':')
            .map(|(_, local)| local)
            .unwrap_or(pred_short)
            .to_string()
    }

    /// Escape string for Cypher
    fn escape_cypher_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Create indexes for common query patterns
    async fn create_indexes(&self) -> Result<()> {
        if let Some(graph) = &self.graph {
            let indexes = vec![
                "CREATE INDEX batch_id_idx IF NOT EXISTS FOR (p:`ex:Product`) ON (p.batchId)",
                "CREATE INDEX uri_idx IF NOT EXISTS FOR (n:Resource) ON (n.uri)",
                "CREATE INDEX producer_idx IF NOT EXISTS FOR (p:`ex:Producer`) ON (p.name)",
            ];

            for idx_cypher in indexes {
                let query = neo_query(idx_cypher);
                if let Err(e) = graph.execute(query).await {
                    warn!("Failed to create index '{}': {}", idx_cypher, e);
                }
            }
        }
        Ok(())
    }

    /// Execute a Cypher query and return timing results
    pub async fn execute_query(&self, cypher: &str) -> Result<Neo4jBenchmarkResult> {
        if let Some(graph) = &self.graph {
            let start = Instant::now();

            match graph.execute(neo_query(cypher)).await {
                Ok(mut result) => {
                    let mut record_count = 0;

                    // Count records
                    while result.next().await?.is_some() {
                        record_count += 1;
                    }

                    let duration = start.elapsed();

                    Ok(Neo4jBenchmarkResult {
                        duration_ms: duration.as_millis() as f64,
                        record_count,
                        success: true,
                        error_message: None,
                        metadata: {
                            let mut m = HashMap::new();
                            m.insert("query".to_string(), cypher.to_string().into());
                            m
                        },
                    })
                }
                Err(e) => {
                    let duration = start.elapsed();
                    Err(anyhow::anyhow!("Query failed after {:?}: {}", duration, e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Not connected to Neo4j"))
        }
    }

    /// Execute a Cypher query with parameters
    pub async fn execute_query_with_params(
        &self,
        cypher: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<Neo4jBenchmarkResult> {
        if let Some(graph) = &self.graph {
            let start = Instant::now();

            let mut query = neo_query(cypher);

            // Add parameters
            for (key, value) in &params {
                match value {
                    serde_json::Value::String(s) => {
                        query = query.param(key.as_str(), s.as_str());
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            query = query.param(key.as_str(), i);
                        } else if let Some(f) = n.as_f64() {
                            query = query.param(key.as_str(), f);
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        query = query.param(key.as_str(), *b);
                    }
                    _ => {}
                }
            }

            match graph.execute(query).await {
                Ok(mut result) => {
                    let mut record_count = 0;

                    while result.next().await?.is_some() {
                        record_count += 1;
                    }

                    let duration = start.elapsed();

                    Ok(Neo4jBenchmarkResult {
                        duration_ms: duration.as_millis() as f64,
                        record_count,
                        success: true,
                        error_message: None,
                        metadata: {
                            let mut m = HashMap::new();
                            m.insert("query".to_string(), cypher.to_string().into());
                            m.insert(
                                "params".to_string(),
                                serde_json::to_value(&params).unwrap_or_default(),
                            );
                            m
                        },
                    })
                }
                Err(e) => {
                    let duration = start.elapsed();
                    Err(anyhow::anyhow!("Query failed after {:?}: {}", duration, e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Not connected to Neo4j"))
        }
    }

    // ==================== Benchmark Query Methods ====================

    /// Query 1: Simple product lookup by batch ID
    /// Equivalent to SPARQL: SELECT ?product WHERE { ?product a ex:Product . ?product ex:batchId "BATCH001" }
    pub async fn query_product_by_batch_id(&self, batch_id: &str) -> Result<Neo4jBenchmarkResult> {
        let cypher = format!(
            "MATCH (p:`ex:Product` {{batchId: '{}'}}) RETURN p.uri as product, p.batchId as batchId",
            Self::escape_cypher_string(batch_id)
        );
        self.execute_query(&cypher).await
    }

    /// Query 2: Multi-hop traceability query
    /// Follows the chain of transactions/processing for a product
    pub async fn query_multi_hop_traceability(
        &self,
        batch_id: &str,
        max_hops: usize,
    ) -> Result<Neo4jBenchmarkResult> {
        let cypher = format!(
            "MATCH path = (start:`ex:Product` {{batchId: '{}'}})-[:`trace:hasTransaction`|`trace:processedBy`|`trace:distributedBy`|`trace:soldBy`*1..{}]->(end) \
             RETURN start.uri as start_product, \
                    [node in nodes(path) | node.uri] as path_nodes, \
                    [rel in relationships(path) | type(rel)] as path_relationships, \
                    length(path) as hops",
            Self::escape_cypher_string(batch_id),
            max_hops
        );
        self.execute_query(&cypher).await
    }

    /// Query 3: Aggregation by producer
    /// Equivalent to SPARQL: SELECT ?producer (SUM(?quantity) AS ?total) WHERE { ... } GROUP BY ?producer
    pub async fn query_aggregation_by_producer(&self) -> Result<Neo4jBenchmarkResult> {
        let cypher = r#"
            MATCH (producer:`ex:Producer`)<-[:`trace:hasProducer`]-(product:`ex:Product`)-[:`trace:hasTransaction`]->(tx)
            WHERE tx.quantity IS NOT NULL
            RETURN producer.name as producer, 
                   producer.uri as producer_uri,
                   sum(toFloat(tx.quantity)) as total_quantity,
                   count(DISTINCT product) as product_count
            ORDER BY total_quantity DESC
        "#;
        self.execute_query(cypher).await
    }

    /// Query 4: Get product with full provenance chain
    pub async fn query_product_provenance(&self, batch_id: &str) -> Result<Neo4jBenchmarkResult> {
        let cypher = format!(
            "MATCH (p:`ex:Product` {{batchId: '{}'}}) \
             OPTIONAL MATCH (p)-[:`trace:hasProducer`]->(producer) \
             OPTIONAL MATCH (p)-[:`trace:hasTransaction`]->(tx) \
             OPTIONAL MATCH (p)-[:`trace:processedBy`]->(processor) \
             OPTIONAL MATCH (p)-[:`trace:distributedBy`]->(distributor) \
             RETURN p.uri as product, \
                    p.batchId as batch_id, \
                    producer.name as producer_name, \
                    collect(DISTINCT tx.uri) as transactions, \
                    collect(DISTINCT processor.name) as processors",
            Self::escape_cypher_string(batch_id)
        );
        self.execute_query(&cypher).await
    }

    /// Query 5: Count all products and transactions (for verification)
    pub async fn query_statistics(&self) -> Result<Neo4jBenchmarkResult> {
        let cypher = r#"
            MATCH (p:`ex:Product`)
            OPTIONAL MATCH (p)-[:`trace:hasTransaction`]->(tx)
            RETURN count(DISTINCT p) as product_count,
                   count(DISTINCT tx) as transaction_count
        "#;
        self.execute_query(cypher).await
    }

    /// Close the connection
    pub async fn close(&self) -> Result<()> {
        // neo4rs handles connection pooling automatically
        info!("Neo4j connection closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_iri() {
        assert_eq!(
            Neo4jClient::shorten_iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            "rdf:type"
        );
        assert_eq!(
            Neo4jClient::shorten_iri("http://example.org/supplychain/Product"),
            "ex:Product"
        );
        assert_eq!(
            Neo4jClient::shorten_iri("http://example.org/traceability#hasTransaction"),
            "trace:hasTransaction"
        );
    }

    #[test]
    fn test_escape_cypher_string() {
        assert_eq!(
            Neo4jClient::escape_cypher_string("It's a test"),
            "It\\'s a test"
        );
        assert_eq!(
            Neo4jClient::escape_cypher_string("Line1\nLine2"),
            "Line1\\nLine2"
        );
    }

    #[test]
    fn test_build_merge_cypher() {
        let client = Neo4jClient::new(Neo4jClientConfig::default());

        let triples = vec![
            RdfTriple {
                subject: "http://example.org/product/1".to_string(),
                predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                object: "http://example.org/supplychain/Product".to_string(),
                object_is_literal: false,
                object_type: None,
            },
            RdfTriple {
                subject: "http://example.org/product/1".to_string(),
                predicate: "http://example.org/supplychain/batchId".to_string(),
                object: "BATCH001".to_string(),
                object_is_literal: true,
                object_type: None,
            },
        ];

        let cypher = client.build_merge_cypher("http://example.org/product/1", &triples, 0);

        assert!(cypher.contains("MERGE"));
        assert!(cypher.contains("BATCH001"));
    }
}
