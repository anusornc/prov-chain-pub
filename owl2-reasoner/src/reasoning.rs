//! OWL2 Reasoning Engine
//!
//! Provides reasoning capabilities for OWL2 ontologies including
//! tableaux-based reasoning, rule-based inference, and query answering.

pub mod classification;
pub mod consistency;
pub mod profile_optimized;
pub mod query;
pub mod rules;
pub mod simple;
pub mod tableaux;

pub use classification::*;
pub use consistency::*;
pub use profile_optimized::*;
pub use query::{
    FilterExpression, PatternTerm, QueryBinding, QueryConfig, QueryEngine, QueryEngineStats,
    QueryPattern, QueryResult, QueryValue, TriplePattern,
};
pub use rules::*;
pub use simple::*;
pub use tableaux::*;

use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use std::sync::Arc;

/// Main OWL2 reasoning engine
pub struct OwlReasoner {
    simple: SimpleReasoner,
    tableaux: Option<TableauxReasoner>,
    use_advanced_reasoning: bool,
}

/// Reasoning configuration
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Enable basic reasoning
    pub enable_reasoning: bool,
    /// Use advanced tableaux reasoning
    pub use_advanced_reasoning: bool,
    /// Tableaux reasoning configuration
    pub tableaux_config: tableaux::ReasoningConfig,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        ReasoningConfig {
            enable_reasoning: true,
            use_advanced_reasoning: true,
            tableaux_config: tableaux::ReasoningConfig::default(),
        }
    }
}

/// Reasoning capabilities
pub trait Reasoner {
    /// Check if the ontology is consistent
    fn is_consistent(&mut self) -> OwlResult<bool>;

    /// Check if one class is a subclass of another
    fn is_subclass_of(&mut self, sub: &IRI, sup: &IRI) -> OwlResult<bool>;

    /// Check if two classes are equivalent
    fn are_equivalent_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool>;

    /// Check if two classes are disjoint
    fn are_disjoint_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool>;

    /// Get all instances of a class
    fn get_instances(&mut self, class: &IRI) -> OwlResult<Vec<Arc<IRI>>>;

    /// Check if an individual is an instance of a class
    fn is_instance_of(&mut self, individual: &IRI, class: &IRI) -> OwlResult<bool>;
}

impl OwlReasoner {
    /// Create a new OWL2 reasoner
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, ReasoningConfig::default())
    }

    /// Create a new OWL2 reasoner with custom configuration
    pub fn with_config(ontology: Ontology, config: ReasoningConfig) -> Self {
        let simple = SimpleReasoner::new(ontology.clone());
        let tableaux = if config.use_advanced_reasoning {
            Some(TableauxReasoner::with_config(
                ontology,
                config.tableaux_config,
            ))
        } else {
            None
        };

        OwlReasoner {
            simple,
            tableaux,
            use_advanced_reasoning: config.use_advanced_reasoning,
        }
    }

    /// Get the underlying ontology
    pub fn ontology(&self) -> &Ontology {
        &self.simple.ontology
    }

    /// Check if a class is satisfiable
    pub fn is_class_satisfiable(&mut self, class_iri: &IRI) -> OwlResult<bool> {
        self.simple.is_class_satisfiable(class_iri)
    }

    /// Create a query engine for this reasoner
    pub fn query_engine(&self) -> QueryEngine {
        QueryEngine::new(self.simple.ontology.clone())
    }

    /// Execute a SPARQL-like query
    pub fn query(&mut self, query: &str) -> OwlResult<QueryResult> {
        let engine = self.query_engine();
        // Parse the query string into a query pattern
        let pattern = self.parse_sparql_query(query)?;
        engine.execute(&pattern)
    }

    /// Parse a simple SPARQL-like query string
    fn parse_sparql_query(&self, query: &str) -> OwlResult<QueryPattern> {
        let query = query.trim();

        // Basic validation
        if !query.to_uppercase().starts_with("SELECT") {
            return Err(OwlError::QueryError(
                "Only SELECT queries are supported".to_string(),
            ));
        }

        // Find WHERE clause
        let where_pos = query
            .to_uppercase()
            .find("WHERE")
            .ok_or_else(|| OwlError::QueryError("WHERE clause not found in query".to_string()))?;

        // Extract WHERE clause content
        let where_clause = &query[where_pos + 5..]; // Skip "WHERE"
        let where_clause = where_clause.trim();

        // Basic WHERE clause validation
        if !where_clause.starts_with('{') || !where_clause.ends_with('}') {
            return Err(OwlError::QueryError(
                "WHERE clause must be enclosed in curly braces".to_string(),
            ));
        }

        // Extract triple patterns from WHERE clause
        let triples_content = &where_clause[1..where_clause.len() - 1]; // Remove braces
        let triple_patterns = self.parse_triple_patterns(triples_content)?;

        Ok(QueryPattern::BasicGraphPattern(triple_patterns))
    }

    /// Parse triple patterns from WHERE clause content
    fn parse_triple_patterns(&self, content: &str) -> OwlResult<Vec<TriplePattern>> {
        let mut patterns = Vec::new();
        let mut remaining = content.trim();

        while !remaining.is_empty() {
            // Skip whitespace
            remaining = remaining.trim_start();

            if remaining.is_empty() {
                break;
            }

            // Find the end of this triple (next dot or end)
            let dot_pos = remaining.find('.');
            let triple_str = if let Some(pos) = dot_pos {
                let triple = remaining[..pos].trim();
                remaining = &remaining[pos + 1..];
                triple
            } else {
                // Last triple without trailing dot
                let triple = remaining.trim();
                remaining = "";
                triple
            };

            if !triple_str.is_empty() {
                let pattern = self.parse_single_triple(triple_str)?;
                patterns.push(pattern);
            }
        }

        Ok(patterns)
    }

    /// Parse a single triple pattern
    fn parse_single_triple(&self, triple: &str) -> OwlResult<TriplePattern> {
        let triple = triple.trim();

        // Parse subject (variable or IRI)
        let (subject, rest) = self.parse_next_term(triple)?;
        let rest = rest.trim_start();

        // Parse predicate (variable or IRI)
        let (predicate, rest) = self.parse_next_term(rest)?;
        let rest = rest.trim_start();

        // Parse object (variable, IRI, or literal)
        let (object, _) = self.parse_next_term(rest)?;

        Ok(TriplePattern {
            subject,
            predicate,
            object,
        })
    }

    /// Parse the next term from the string, handling IRIs with spaces
    fn parse_next_term<'a>(&self, input: &'a str) -> OwlResult<(PatternTerm, &'a str)> {
        let input = input.trim_start();

        if let Some(stripped) = input.strip_prefix('?') {
            // Variable - find next whitespace
            if let Some(space_pos) = stripped.find(char::is_whitespace) {
                let var_name = &stripped[..space_pos];
                Ok((
                    PatternTerm::Variable(var_name.to_string()),
                    &input[space_pos + 1..],
                ))
            } else {
                // Variable at end of string
                Ok((PatternTerm::Variable(stripped.to_string()), ""))
            }
        } else if input.starts_with('<') {
            // IRI - find closing >
            if let Some(close_pos) = input.find('>') {
                let iri_str = &input[1..close_pos];
                let iri = IRI::new(iri_str).map_err(|e| {
                    OwlError::QueryError(format!("Invalid IRI '{}': {}", iri_str, e))
                })?;
                Ok((PatternTerm::IRI(iri), &input[close_pos + 1..]))
            } else {
                Err(OwlError::QueryError("Unclosed IRI".to_string()))
            }
        } else if let Some(stripped) = input.strip_prefix('"') {
            // Literal - find closing "
            if let Some(close_pos) = stripped.find('"') {
                Ok((
                    PatternTerm::Literal(stripped[..close_pos].to_string()),
                    &input[close_pos + 2..],
                ))
            } else {
                Err(OwlError::QueryError("Unclosed literal".to_string()))
            }
        } else {
            // Try to parse as IRI without angle brackets
            if let Some(space_pos) = input.find(char::is_whitespace) {
                let term = &input[..space_pos];
                let iri = IRI::new(term)
                    .map_err(|e| OwlError::QueryError(format!("Invalid term '{}': {}", term, e)))?;
                Ok((PatternTerm::IRI(iri), &input[space_pos..]))
            } else {
                // Term at end of string
                let iri = IRI::new(input).map_err(|e| {
                    OwlError::QueryError(format!("Invalid term '{}': {}", input, e))
                })?;
                Ok((PatternTerm::IRI(iri), ""))
            }
        }
    }
}

impl Reasoner for OwlReasoner {
    fn is_consistent(&mut self) -> OwlResult<bool> {
        if self.use_advanced_reasoning {
            if let Some(tableaux) = &mut self.tableaux {
                // Use tableaux reasoning for proper consistency checking
                // Check if owl:Thing is satisfiable - if not, ontology is inconsistent
                let thing_iri = IRI::new("http://www.w3.org/2002/07/owl#Thing").map_err(|e| {
                    OwlError::ReasoningError(format!("Failed to create owl:Thing IRI: {}", e))
                })?;
                return tableaux.is_class_satisfiable(&thing_iri);
            }
        }
        self.simple.is_consistent()
    }

    fn is_subclass_of(&mut self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
        self.simple.is_subclass_of(sub, sup)
    }

    fn are_equivalent_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool> {
        // For now, check if a ⊑ b and b ⊑ a
        Ok(self.is_subclass_of(a, b)? && self.is_subclass_of(b, a)?)
    }

    fn are_disjoint_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool> {
        if self.use_advanced_reasoning {
            if let Some(tableaux) = &mut self.tableaux {
                // Use tableaux reasoning for disjointness checking
                return tableaux.are_disjoint_classes(a, b);
            }
        }
        // Fallback to simple reasoning
        self.simple.are_disjoint_classes(a, b)
    }

    fn get_instances(&mut self, class: &IRI) -> OwlResult<Vec<Arc<IRI>>> {
        self.simple.get_instances(class)
    }

    fn is_instance_of(&mut self, individual: &IRI, class: &IRI) -> OwlResult<bool> {
        // For now, check if individual is in instances of class
        let instances = self.get_instances(class)?;
        Ok(instances.contains(&Arc::new((*individual).clone())))
    }
}
