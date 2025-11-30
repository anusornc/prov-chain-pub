//! Safe RDF Store implementation with proper error handling
//!
//! This module provides a safer version of RDF store operations
//! with comprehensive error handling and no unsafe unwrap() calls.

use oxigraph::io::RdfFormat;
use oxigraph::model::*;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::io::Cursor;
use tracing::warn;

use crate::core::blockchain::Block;
use crate::error::{Result, StorageError};

/// Safe RDF Store operations
impl crate::storage::rdf_store::RDFStore {
    /// Safe version of add_rdf_to_graph with proper error handling
    pub fn add_rdf_to_graph_safe(&mut self, rdf_data: &str, graph_name: &NamedNode) -> Result<()> {
        // Try to parse as RDF using a temporary store
        let temp_store = Store::new().map_err(|e| {
            StorageError::ConnectionFailed(format!("Failed to create temporary store: {}", e))
        })?;
        let reader = Cursor::new(rdf_data.as_bytes());

        match temp_store.load_from_reader(RdfFormat::Turtle, reader) {
            Ok(_) => {
                // Successfully parsed as RDF, now copy all triples to the target graph
                let mut quads_to_insert = Vec::new();
                for quad in temp_store.iter() {
                    match quad {
                        Ok(original_quad) => {
                            // Create a new quad with the specified graph name
                            let new_quad = Quad::new(
                                original_quad.subject.clone(),
                                original_quad.predicate.clone(),
                                original_quad.object.clone(),
                                graph_name.clone(),
                            );
                            quads_to_insert.push(new_quad);
                        }
                        Err(e) => {
                            warn!("Failed to read quad from temporary store: {}", e);
                        }
                    }
                }

                // Insert all quads into the main store
                for quad in &quads_to_insert {
                    self.store.insert(quad).map_err(|e| {
                        StorageError::QueryFailed(format!("Failed to insert quad: {}", e))
                    })?;
                }

                // Update cache if it exists
                if let Some(ref mut cache) = self.memory_cache {
                    let mut cached_quads = quads_to_insert.clone();
                    // Add existing quads from cache if any
                    let existing_quads = cache.get(graph_name.as_str()).cloned();
                    if let Some(quads) = existing_quads {
                        cached_quads.extend(quads.iter().cloned());
                    }
                    cache.insert(graph_name.as_str().to_string(), cached_quads);
                }
            }
            Err(_) => {
                // If parsing fails, create a simple triple with the data as a literal
                let subject_uri = format!(
                    "http://provchain.org/data/{}",
                    graph_name
                        .as_str()
                        .replace("http://provchain.org/block/", "")
                );
                let subject = NamedNode::new(subject_uri).map_err(|e| {
                    StorageError::RdfParsingFailed(format!("Failed to create subject node: {}", e))
                })?;

                let predicate = NamedNode::new("http://provchain.org/hasData").map_err(|e| {
                    StorageError::RdfParsingFailed(format!(
                        "Failed to create predicate node: {}",
                        e
                    ))
                })?;

                let object = Literal::new_simple_literal(rdf_data);
                let quad = Quad::new(subject, predicate, object, graph_name.clone());

                self.store.insert(&quad).map_err(|e| {
                    StorageError::QueryFailed(format!("Failed to insert fallback quad: {}", e))
                })?;

                // Update cache if it exists
                if let Some(ref mut cache) = self.memory_cache {
                    let mut cached_quads = vec![quad.clone()];
                    let existing_quads = cache.get(graph_name.as_str()).cloned();
                    if let Some(quads) = existing_quads {
                        cached_quads.extend(quads.iter().cloned());
                    }
                    cache.insert(graph_name.as_str().to_string(), cached_quads);
                }
            }
        }

        Ok(())
    }

    /// Safe version of load_ontology with proper error handling
    pub fn load_ontology_safe(
        &mut self,
        ontology_data: &str,
        _graph_name: &NamedNode,
    ) -> Result<()> {
        let reader = Cursor::new(ontology_data.as_bytes());
        self.store
            .load_from_reader(RdfFormat::Turtle, reader)
            .map_err(|e| {
                StorageError::RdfParsingFailed(format!("Failed to load ontology: {}", e))
            })?;

        Ok(())
    }

    /// Safe version of add_block_metadata with proper error handling
    pub fn add_block_metadata_safe(&mut self, block: &Block) -> Result<()> {
        let graph_name = NamedNode::new("http://provchain.org/blockchain").map_err(|e| {
            StorageError::RdfParsingFailed(format!("Failed to create blockchain graph name: {}", e))
        })?;

        let block_uri = NamedNode::new(format!("http://provchain.org/block/{}", block.index))
            .map_err(|e| {
                StorageError::RdfParsingFailed(format!("Failed to create block URI: {}", e))
            })?;

        let data_graph_uri = NamedNode::new(format!("http://provchain.org/block/{}", block.index))
            .map_err(|e| {
                StorageError::RdfParsingFailed(format!("Failed to create data graph URI: {}", e))
            })?;

        let prev_block_uri = if block.index > 0 {
            Some(
                NamedNode::new(format!("http://provchain.org/block/{}", block.index - 1)).map_err(
                    |e| {
                        StorageError::RdfParsingFailed(format!(
                            "Failed to create previous block URI: {}",
                            e
                        ))
                    },
                )?,
            )
        } else {
            None
        };

        // Determine block type (Genesis or regular Block)
        let block_type = if block.index == 0 {
            NamedNode::new("http://provchain.org/GenesisBlock").map_err(|e| {
                StorageError::RdfParsingFailed(format!(
                    "Failed to create genesis block type: {}",
                    e
                ))
            })?
        } else {
            NamedNode::new("http://provchain.org/Block").map_err(|e| {
                StorageError::RdfParsingFailed(format!("Failed to create block type: {}", e))
            })?
        };

        let type_predicate = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| {
                StorageError::RdfParsingFailed(format!("Failed to create type predicate: {}", e))
            })?;

        let index_predicate = NamedNode::new("http://provchain.org/hasIndex").map_err(|e| {
            StorageError::RdfParsingFailed(format!("Failed to create index predicate: {}", e))
        })?;

        let timestamp_predicate =
            NamedNode::new("http://provchain.org/hasTimestamp").map_err(|e| {
                StorageError::RdfParsingFailed(format!(
                    "Failed to create timestamp predicate: {}",
                    e
                ))
            })?;

        let hash_predicate = NamedNode::new("http://provchain.org/hasHash").map_err(|e| {
            StorageError::RdfParsingFailed(format!("Failed to create hash predicate: {}", e))
        })?;

        let prev_hash_predicate =
            NamedNode::new("http://provchain.org/hasPreviousHash").map_err(|e| {
                StorageError::RdfParsingFailed(format!(
                    "Failed to create previous hash predicate: {}",
                    e
                ))
            })?;

        let data_graph_predicate =
            NamedNode::new("http://provchain.org/hasDataGraphIRI").map_err(|e| {
                StorageError::RdfParsingFailed(format!(
                    "Failed to create data graph predicate: {}",
                    e
                ))
            })?;

        let mut quads = vec![
            Quad::new(
                block_uri.clone(),
                type_predicate,
                block_type,
                graph_name.clone(),
            ),
            Quad::new(
                block_uri.clone(),
                index_predicate,
                Literal::new_typed_literal(
                    block.index.to_string(),
                    NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer"),
                ),
                graph_name.clone(),
            ),
            Quad::new(
                block_uri.clone(),
                timestamp_predicate,
                Literal::new_typed_literal(
                    block.timestamp.clone(),
                    NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#dateTime"),
                ),
                graph_name.clone(),
            ),
            Quad::new(
                block_uri.clone(),
                hash_predicate,
                Literal::new_simple_literal(block.hash.clone()),
                graph_name.clone(),
            ),
            Quad::new(
                block_uri.clone(),
                prev_hash_predicate,
                Literal::new_simple_literal(block.previous_hash.clone()),
                graph_name.clone(),
            ),
            Quad::new(
                block_uri.clone(),
                data_graph_predicate,
                Literal::new_typed_literal(
                    data_graph_uri.as_str(),
                    NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#anyURI"),
                ),
                graph_name.clone(),
            ),
        ];

        if let Some(prev) = prev_block_uri {
            let preceded_by_predicate = NamedNode::new("http://www.w3.org/ns/prov#wasPrecededBy")
                .map_err(|e| {
                StorageError::RdfParsingFailed(format!(
                    "Failed to create preceded by predicate: {}",
                    e
                ))
            })?;

            quads.push(Quad::new(
                block_uri,
                preceded_by_predicate,
                prev,
                graph_name,
            ));
        }

        for quad in &quads {
            self.store.insert(quad).map_err(|e| {
                StorageError::QueryFailed(format!("Failed to insert block metadata quad: {}", e))
            })?;
        }

        Ok(())
    }

    /// Safe version of query with proper error handling
    pub fn query_safe(&self, sparql: &str) -> Result<QueryResults> {
        self.store
            .query(sparql)
            .map_err(|e| StorageError::SparqlError(format!("SPARQL query failed: {}", e)).into())
    }
}

/// Helper functions for safe RDF operations
pub struct SafeRDFOperations;

impl SafeRDFOperations {
    /// Safely create a NamedNode with error handling
    pub fn create_named_node(uri: &str) -> Result<NamedNode> {
        NamedNode::new(uri).map_err(|e| {
            StorageError::RdfParsingFailed(format!("Invalid URI '{}': {}", uri, e)).into()
        })
    }

    /// Safely create a Store with error handling
    pub fn create_store() -> Result<Store> {
        Store::new().map_err(|e| {
            StorageError::ConnectionFailed(format!("Failed to create RDF store: {}", e)).into()
        })
    }

    /// Safely insert a quad with error handling
    pub fn insert_quad(store: &Store, quad: &Quad) -> Result<()> {
        store
            .insert(quad)
            .map(|_| ()) // Convert bool to ()
            .map_err(|e| StorageError::QueryFailed(format!("Failed to insert quad: {}", e)).into())
    }

    /// Safely execute a SPARQL query with error handling
    pub fn execute_query(store: &Store, sparql: &str) -> Result<QueryResults> {
        store
            .query(sparql)
            .map_err(|e| StorageError::SparqlError(format!("SPARQL query failed: {}", e)).into())
    }

    /// Safely load RDF data from reader with error handling
    pub fn load_from_reader(store: &Store, format: RdfFormat, reader: Cursor<&[u8]>) -> Result<()> {
        store.load_from_reader(format, reader).map_err(|e| {
            StorageError::RdfParsingFailed(format!("Failed to parse RDF data: {}", e)).into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::rdf_store::RDFStore;

    #[test]
    fn test_safe_named_node_creation() {
        let result = SafeRDFOperations::create_named_node("http://example.org/test");
        assert!(result.is_ok());

        let result = SafeRDFOperations::create_named_node("invalid uri");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_store_creation() {
        let result = SafeRDFOperations::create_store();
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_rdf_operations() {
        let mut store = RDFStore::new();
        let graph_name = NamedNode::new("http://example.org/test").unwrap();

        let result = store.add_rdf_to_graph_safe(
            "@prefix ex: <http://example.org/> . ex:test ex:prop \"value\" .",
            &graph_name,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_query_execution() {
        let store = RDFStore::new();
        let result = store.query_safe("SELECT * WHERE { ?s ?p ?o }");
        assert!(result.is_ok());
    }
}
