//! Query data types and patterns for OWL2 ontologies
//!
//! Contains the core data structures for representing queries, patterns, and results.

use crate::iri::IRI;
use hashbrown::HashMap;
use std::hash::{Hash, Hasher};

/// Query result containing bindings and metadata
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Query results (variable bindings)
    pub bindings: Vec<QueryBinding>,
    /// Variable names in order
    pub variables: Vec<String>,
    /// Query execution statistics
    pub stats: QueryStats,
}

/// Query binding (variable to value mapping)
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBinding {
    /// Variable name to value mapping
    pub variables: HashMap<String, QueryValue>,
}

/// Query value
#[derive(Debug, Clone, PartialEq)]
pub enum QueryValue {
    IRI(IRI),
    Literal(String),
    BlankNode(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Eq for QueryValue {}

impl std::hash::Hash for QueryValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            QueryValue::IRI(iri) => iri.as_str().hash(state),
            QueryValue::Literal(lit) => lit.hash(state),
            QueryValue::BlankNode(bn) => bn.hash(state),
            QueryValue::Boolean(b) => b.hash(state),
            QueryValue::Integer(i) => i.hash(state),
            QueryValue::Float(f) => f.to_bits().hash(state),
        }
    }
}

/// Query statistics
#[derive(Debug, Clone, PartialEq)]
pub struct QueryStats {
    /// Number of results returned
    pub results_count: usize,
    /// Query execution time in milliseconds
    pub time_ms: u64,
    /// Whether reasoning was used
    pub reasoning_used: bool,
}

/// Query pattern
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum QueryPattern {
    BasicGraphPattern(Vec<TriplePattern>),
    Optional {
        left: Box<QueryPattern>,
        right: Box<QueryPattern>,
    },
    Union {
        left: Box<QueryPattern>,
        right: Box<QueryPattern>,
    },
    Filter {
        pattern: Box<QueryPattern>,
        expression: FilterExpression,
    },
    Reduced(Box<QueryPattern>),
    Distinct(Box<QueryPattern>),
}

// Safety: All variants in QueryPattern contain Send + Sync types
unsafe impl Send for QueryPattern {}
unsafe impl Sync for QueryPattern {}

/// Triple pattern for SPARQL-like queries
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct TriplePattern {
    pub subject: PatternTerm,
    pub predicate: PatternTerm,
    pub object: PatternTerm,
}

/// Pattern term (can be variable or constant)
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum PatternTerm {
    Variable(String),
    IRI(IRI),
    Literal(String),
    BlankNode(String),
}

/// Filter expression
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FilterExpression {
    Equals(Box<FilterExpression>, Box<FilterExpression>),
    NotEquals(Box<FilterExpression>, Box<FilterExpression>),
    LessThan(Box<FilterExpression>, Box<FilterExpression>),
    GreaterThan(Box<FilterExpression>, Box<FilterExpression>),
    LessThanOrEqual(Box<FilterExpression>, Box<FilterExpression>),
    GreaterThanOrEqual(Box<FilterExpression>, Box<FilterExpression>),
    And(Box<FilterExpression>, Box<FilterExpression>),
    Or(Box<FilterExpression>, Box<FilterExpression>),
    Not(Box<FilterExpression>),
    IsVariable(String),
    IsIRI(String),
    IsLiteral(String),
    IsBlankNode(String),
    Bound(String),
}

// Safety: All variants in FilterExpression contain Send + Sync types
unsafe impl Send for FilterExpression {}
unsafe impl Sync for FilterExpression {}

/// RDF vocabulary constants
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

impl QueryBinding {
    /// Create a new empty query binding
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Add a variable binding
    pub fn add_binding(&mut self, variable: String, value: QueryValue) {
        self.variables.insert(variable, value);
    }

    /// Get the value for a variable
    pub fn get_value(&self, variable: &str) -> Option<&QueryValue> {
        self.variables.get(variable)
    }

    /// Check if a variable is bound
    pub fn is_bound(&self, variable: &str) -> bool {
        self.variables.contains_key(variable)
    }

    /// Get all variable names
    pub fn variables(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }

    /// Get all bindings
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &QueryValue)> {
        self.variables.iter()
    }

    /// Merge with another binding (this takes precedence)
    pub fn merge(&mut self, other: &QueryBinding) {
        for (var, value) in &other.variables {
            if !self.variables.contains_key(var) {
                self.variables.insert(var.clone(), value.clone());
            }
        }
    }

    /// Check if this binding is compatible with another
    pub fn is_compatible(&self, other: &QueryBinding) -> bool {
        for (var, value) in &self.variables {
            if let Some(other_value) = other.variables.get(var) {
                if value != other_value {
                    return false;
                }
            }
        }
        true
    }

    /// Create a merged binding if compatible
    pub fn join(&self, other: &QueryBinding) -> Option<QueryBinding> {
        if !self.is_compatible(other) {
            return None;
        }

        let mut merged = self.clone();
        merged.merge(other);
        Some(merged)
    }
}

impl Default for QueryBinding {
    fn default() -> Self {
        Self::new()
    }
}

impl TriplePattern {
    /// Create a new triple pattern
    pub fn new(subject: PatternTerm, predicate: PatternTerm, object: PatternTerm) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }

    /// Get all variables in this pattern
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();

        if let PatternTerm::Variable(var) = &self.subject {
            vars.push(var.clone());
        }
        if let PatternTerm::Variable(var) = &self.predicate {
            vars.push(var.clone());
        }
        if let PatternTerm::Variable(var) = &self.object {
            vars.push(var.clone());
        }

        vars
    }

    /// Check if this pattern is ground (no variables)
    pub fn is_ground(&self) -> bool {
        !matches!(&self.subject, PatternTerm::Variable(_))
            && !matches!(&self.predicate, PatternTerm::Variable(_))
            && !matches!(&self.object, PatternTerm::Variable(_))
    }

    /// Count the number of variables
    pub fn variable_count(&self) -> usize {
        let mut count = 0;
        if matches!(&self.subject, PatternTerm::Variable(_)) {
            count += 1;
        }
        if matches!(&self.predicate, PatternTerm::Variable(_)) {
            count += 1;
        }
        if matches!(&self.object, PatternTerm::Variable(_)) {
            count += 1;
        }
        count
    }
}

impl QueryResult {
    /// Create a new empty query result
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            variables: Vec::new(),
            stats: QueryStats {
                results_count: 0,
                time_ms: 0,
                reasoning_used: false,
            },
        }
    }

    /// Add a binding to the result
    pub fn add_binding(&mut self, binding: QueryBinding) {
        self.bindings.push(binding);
    }

    /// Get the number of results
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get an iterator over the bindings
    pub fn iter(&self) -> impl Iterator<Item = &QueryBinding> {
        self.bindings.iter()
    }

    /// Rename a variable in the result
    pub fn rename_variable(&mut self, old_name: &str, new_name: &str) {
        // Update variables list
        if let Some(pos) = self.variables.iter().position(|v| v == old_name) {
            self.variables[pos] = new_name.to_string();
        }

        // Update bindings
        for binding in &mut self.bindings {
            if let Some(value) = binding.variables.remove(old_name) {
                binding.variables.insert(new_name.to_string(), value);
            }
        }
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::new()
    }
}
