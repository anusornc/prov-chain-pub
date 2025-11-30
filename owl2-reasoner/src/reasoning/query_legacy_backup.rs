//! Query answering for OWL2 ontologies
//!
//! Provides SPARQL-like query capabilities for OWL2 ontologies with reasoning support.
//! Features advanced query optimizations including caching, indexing, and pattern compilation.

use crate::axioms::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::reasoning::Reasoner;

use dashmap::DashMap;
use hashbrown::HashMap;
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Helper function to avoid unnecessary (**arc_iri).clone() operations
#[inline(always)]
fn arc_iri_to_owned(arc_iri: &Arc<IRI>) -> IRI {
    // This is still needed for PatternTerm::IRI which takes owned IRI
    // but we optimize by avoiding the double dereference
    (**arc_iri).clone()
}

/// Helper function to create PatternTerm::IRI from Arc<IRI> with minimal cloning
#[inline(always)]
fn pattern_term_from_arc(arc_iri: &Arc<IRI>) -> PatternTerm {
    PatternTerm::IRI(arc_iri_to_owned(arc_iri))
}

/// Query engine for OWL2 ontologies with advanced optimizations
pub struct QueryEngine {
    ontology: Arc<Ontology>,
    #[allow(dead_code)]
    reasoner: Option<Box<dyn Reasoner>>,
    config: QueryConfig,
    /// Query result cache with LRU eviction
    query_cache: Arc<RwLock<LruCache<QueryCacheKey, QueryResult>>>,
    /// Compiled pattern cache for fast execution
    pattern_cache: Arc<RwLock<HashMap<u64, CompiledPattern>>>,
    /// Memory pool for reusing allocations
    result_pool: Arc<ResultPool>,
    /// Index-based access structures for fast pattern matching
    type_index: Arc<DashMap<Arc<IRI>, Vec<Arc<crate::axioms::ClassAssertionAxiom>>>>,
    property_index: Arc<DashMap<Arc<IRI>, Vec<Arc<crate::axioms::PropertyAssertionAxiom>>>>,
    /// Statistics for performance monitoring
    stats: Arc<RwLock<QueryEngineStats>>,
}

/// Query engine performance statistics
#[derive(Debug, Default, Clone)]
pub struct QueryEngineStats {
    cache_hits: u64,
    cache_misses: u64,
    queries_executed: u64,
    total_execution_time_ms: u64,
    parallel_queries_executed: u64,
    parallel_execution_time_ms: u64,
}

/// Query configuration with optimization options
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Enable reasoning during query answering
    pub enable_reasoning: bool,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
    /// Enable query result caching
    pub enable_caching: bool,
    /// Cache size (number of cached results)
    pub cache_size: Option<usize>,
    /// Enable parallel query execution
    pub enable_parallel: bool,
    /// Maximum number of parallel threads (None = use system default)
    pub max_parallel_threads: Option<usize>,
    /// Minimum number of union patterns to trigger parallel execution
    pub parallel_threshold: usize,
    /// Use memory pool for result allocation
    pub use_memory_pool: bool,
}

impl Default for QueryConfig {
    fn default() -> Self {
        QueryConfig {
            enable_reasoning: true,
            max_results: None,
            timeout: Some(10000), // 10 seconds default
            enable_caching: true,
            cache_size: Some(1000),
            enable_parallel: true,
            max_parallel_threads: None, // Use system default
            parallel_threshold: 2,      // Parallel for 2+ union patterns
            use_memory_pool: true,
        }
    }
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub bindings: Vec<QueryBinding>,
    pub variables: Vec<String>,
    pub stats: QueryStats,
}

/// Query binding (variable to value mapping)
#[derive(Debug, Clone)]
pub struct QueryBinding {
    pub variables: HashMap<String, QueryValue>,
}

/// Query value
#[derive(Debug, Clone, PartialEq)]
pub enum QueryValue {
    IRI(IRI),
    Literal(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Eq for QueryValue {}

impl std::hash::Hash for QueryValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            QueryValue::IRI(iri) => {
                state.write_u8(0);
                iri.hash(state);
            }
            QueryValue::Literal(lit) => {
                state.write_u8(1);
                lit.hash(state);
            }
            QueryValue::Boolean(b) => {
                state.write_u8(2);
                b.hash(state);
            }
            QueryValue::Integer(i) => {
                state.write_u8(3);
                i.hash(state);
            }
            QueryValue::Float(f) => {
                state.write_u8(4);
                // Convert to bits for hashing since f64 doesn't implement Hash
                f.to_bits().hash(state);
            }
        }
    }
}

/// Query statistics
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub results_count: usize,
    pub time_ms: u64,
    pub reasoning_used: bool,
}

/// Query pattern
#[derive(Debug, Clone, Hash)]
pub enum QueryPattern {
    /// Basic graph pattern
    BasicGraphPattern(Vec<TriplePattern>),
    /// Optional pattern
    OptionalPattern(Box<QueryPattern>),
    /// Union pattern
    UnionPattern(Vec<QueryPattern>),
    /// Filter pattern
    FilterPattern {
        pattern: Box<QueryPattern>,
        expression: FilterExpression,
    },
}

// Safety: All variants in QueryPattern contain Send + Sync types
unsafe impl Send for QueryPattern {}
unsafe impl Sync for QueryPattern {}

/// Triple pattern for SPARQL-like queries
#[derive(Debug, Clone, Hash)]
pub struct TriplePattern {
    pub subject: PatternTerm,
    pub predicate: PatternTerm,
    pub object: PatternTerm,
}

/// Pattern term (can be variable or constant)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternTerm {
    Variable(String),
    IRI(IRI),
    Literal(String),
}

/// Filter expression
#[derive(Debug, Clone, Hash)]
pub enum FilterExpression {
    /// Equality comparison
    Equals {
        left: PatternTerm,
        right: PatternTerm,
    },
    /// Type check
    Type { term: PatternTerm, type_iri: IRI },
    /// Logical AND
    And(Vec<FilterExpression>),
    /// Logical OR
    Or(Vec<FilterExpression>),
    /// Logical NOT
    Not(Box<FilterExpression>),
}

// Safety: All variants in FilterExpression contain Send + Sync types
unsafe impl Send for FilterExpression {}
unsafe impl Sync for FilterExpression {}

/// RDF vocabulary constants
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Cached RDF type IRI to avoid repeated creation
static RDF_TYPE_IRI: Lazy<IRI> =
    Lazy::new(|| IRI::new(RDF_TYPE).expect("Failed to create cached rdf:type IRI"));

static RDF_TYPE_TERM: Lazy<PatternTerm> = Lazy::new(|| PatternTerm::IRI(RDF_TYPE_IRI.clone()));

/// Query cache key for result caching
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct QueryCacheKey {
    pattern_hash: u64,
    config_hash: u64,
}

/// Compiled query pattern for fast execution
#[derive(Debug, Clone)]
struct CompiledPattern {
    /// Original pattern
    #[allow(dead_code)]
    pattern: QueryPattern,
    /// Pre-computed hash for caching
    #[allow(dead_code)]
    hash: u64,
    /// Optimized execution plan
    execution_plan: ExecutionPlan,
    /// Variable positions for fast binding
    variable_positions: Vec<String>,
}

// Safety: All fields in CompiledPattern are Send + Sync
unsafe impl Send for CompiledPattern {}
unsafe impl Sync for CompiledPattern {}

/// Query execution plan for optimized evaluation
#[derive(Debug, Clone)]
enum ExecutionPlan {
    /// Single triple pattern with optimized access path
    SingleTriple {
        query_type: QueryType,
        pattern: TriplePattern,
    },
    /// Multi-triple pattern with join ordering
    MultiTriple {
        patterns: Vec<TriplePattern>,
        join_order: Vec<usize>,
        access_paths: Vec<QueryType>,
    },
    /// Optional pattern with left outer join
    Optional {
        base: Box<ExecutionPlan>,
        optional: Box<ExecutionPlan>,
    },
    /// Union pattern with parallel execution
    Union { plans: Vec<ExecutionPlan> },
    /// Filter pattern with early filtering
    Filter {
        base: Box<ExecutionPlan>,
        filter_expr: FilterExpression,
    },
}

// Safety: All variants in ExecutionPlan contain Send + Sync types
unsafe impl Send for ExecutionPlan {}
unsafe impl Sync for ExecutionPlan {}

/// Memory pool for reusing query result allocations
struct ResultPool {
    binding_pool: RwLock<Vec<QueryBinding>>,
    #[allow(dead_code)]
    result_pool: RwLock<Vec<QueryResult>>,
}

impl ResultPool {
    fn new() -> Self {
        Self {
            binding_pool: RwLock::new(Vec::with_capacity(1000)),
            result_pool: RwLock::new(Vec::with_capacity(100)),
        }
    }

    fn get_binding(&self) -> QueryBinding {
        let mut pool = self.binding_pool.write();
        pool.pop().unwrap_or_else(|| QueryBinding {
            variables: HashMap::new(),
        })
    }

    fn return_binding(&self, mut binding: QueryBinding) {
        binding.variables.clear();
        let mut pool = self.binding_pool.write();
        if pool.len() < 1000 {
            pool.push(binding);
        }
    }

    #[allow(dead_code)]
    fn get_result(&self) -> QueryResult {
        let mut pool = self.result_pool.write();
        pool.pop().unwrap_or_else(|| QueryResult {
            bindings: Vec::new(),
            variables: Vec::new(),
            stats: QueryStats {
                results_count: 0,
                time_ms: 0,
                reasoning_used: false,
            },
        })
    }

    #[allow(dead_code)]
    fn return_result(&self, mut result: QueryResult) {
        result.bindings.clear();
        result.variables.clear();
        let mut pool = self.result_pool.write();
        if pool.len() < 100 {
            pool.push(result);
        }
    }
}

/// Types of triple pattern queries
#[derive(Debug, Clone, PartialEq)]
enum QueryType {
    TypeQuery,
    PropertyQuery,
    VariablePredicate,
}

/// Thread-safe query context for parallel execution
struct ThreadQueryContext<'a> {
    ontology: &'a Arc<Ontology>,
    type_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<crate::axioms::ClassAssertionAxiom>>>>,
    property_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<crate::axioms::PropertyAssertionAxiom>>>>,
    result_pool: &'a Arc<ResultPool>,
}

impl<'a> ThreadQueryContext<'a> {
    /// Execute a single execution plan in this thread context
    fn execute_plan(&self, plan: &ExecutionPlan) -> OwlResult<Vec<QueryBinding>> {
        let compiled_pattern = CompiledPattern {
            pattern: QueryPattern::BasicGraphPattern(Vec::new()),
            hash: 0,
            execution_plan: plan.clone(),
            variable_positions: Vec::new(),
        };

        self.execute_compiled_pattern(&compiled_pattern)
    }

    /// Execute compiled pattern (thread-safe version)
    fn execute_compiled_pattern(&self, compiled: &CompiledPattern) -> OwlResult<Vec<QueryBinding>> {
        match &compiled.execution_plan {
            ExecutionPlan::SingleTriple {
                query_type,
                pattern,
            } => self.match_triple_pattern_optimized_type(pattern, query_type),
            ExecutionPlan::MultiTriple {
                patterns,
                join_order,
                access_paths,
            } => self.evaluate_multi_triple_optimized(patterns, join_order, access_paths),
            ExecutionPlan::Optional { base, optional } => {
                self.evaluate_optional_optimized(base, optional)
            }
            ExecutionPlan::Union { plans } => {
                // For nested unions, use sequential execution to avoid thread explosion
                let mut all_bindings = Vec::new();
                for plan in plans {
                    let plan_bindings = self.execute_plan(plan)?;
                    all_bindings.extend(plan_bindings);
                }
                Ok(all_bindings)
            }
            ExecutionPlan::Filter { base, filter_expr } => {
                let bindings = self.execute_compiled_filter(base)?;
                self.apply_filter(&bindings, filter_expr)
            }
        }
    }

    /// Execute single triple pattern with type-specific optimization
    fn match_triple_pattern_optimized_type(
        &self,
        triple: &TriplePattern,
        query_type: &QueryType,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        match query_type {
            QueryType::TypeQuery => {
                self.collect_type_query_bindings_optimized(triple, &mut bindings);
            }
            QueryType::PropertyQuery => {
                self.collect_property_query_bindings_optimized(triple, &mut bindings);
            }
            QueryType::VariablePredicate => {
                self.collect_variable_predicate_bindings_optimized(triple, &mut bindings);
            }
        }

        Ok(bindings)
    }

    /// Optimized type query collection using index
    fn collect_type_query_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use index when object is a specific type
        if let PatternTerm::IRI(type_iri) = &triple.object {
            if let Some(axioms) = self.type_index.get(type_iri) {
                for axiom in axioms.iter() {
                    if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                        bindings.push(binding);
                    }
                }
            }
        } else {
            // Fall back to linear scan for variable types
            for axiom in self.ontology.class_assertions() {
                if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                    bindings.push(binding);
                }
            }
        }
    }

    /// Optimized property query collection using index
    fn collect_property_query_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use index when predicate is a specific property
        if let PatternTerm::IRI(prop_iri) = &triple.predicate {
            if let Some(axioms) = self.property_index.get(prop_iri) {
                for axiom in axioms.iter() {
                    if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                        bindings.push(binding);
                    }
                }
            }
        } else {
            // Fall back to linear scan for variable predicates
            for axiom in self.ontology.property_assertions() {
                if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                    bindings.push(binding);
                }
            }
        }
    }

    /// Optimized variable predicate query collection
    fn collect_variable_predicate_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use indexes for both type and property assertions
        self.collect_type_query_bindings_optimized(triple, bindings);
        self.collect_property_query_bindings_optimized(triple, bindings);
    }

    /// Execute multi-triple pattern with optimized joins
    fn evaluate_multi_triple_optimized(
        &self,
        patterns: &[TriplePattern],
        join_order: &[usize],
        access_paths: &[QueryType],
    ) -> OwlResult<Vec<QueryBinding>> {
        if join_order.is_empty() {
            return Ok(Vec::new());
        }

        // Start with the most selective pattern
        let first_idx = join_order[0];
        let mut bindings =
            self.match_triple_pattern_optimized_type(&patterns[first_idx], &access_paths[0])?;

        // Join with remaining patterns
        for &idx in join_order.iter().skip(1) {
            let pattern_bindings =
                self.match_triple_pattern_optimized_type(&patterns[idx], &access_paths[idx])?;
            bindings = self.hash_join_bindings_optimized(&bindings, &pattern_bindings)?;

            if bindings.is_empty() {
                break; // Early termination
            }
        }

        Ok(bindings)
    }

    /// Optimized hash join with memory reuse
    fn hash_join_bindings_optimized(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &[QueryBinding],
    ) -> OwlResult<Vec<QueryBinding>> {
        if left_bindings.is_empty() || right_bindings.is_empty() {
            return Ok(Vec::new());
        }

        // Find common variables
        let left_vars: HashSet<String> = left_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();
        let right_vars: HashSet<String> = right_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();

        let common_vars: Vec<String> = left_vars.intersection(&right_vars).cloned().collect();

        if common_vars.is_empty() {
            // Cartesian product
            let mut result = Vec::with_capacity(left_bindings.len() * right_bindings.len());
            for left in left_bindings {
                for right in right_bindings {
                    let mut combined = self.result_pool.get_binding();
                    combined.variables.extend(left.variables.clone());
                    combined.variables.extend(right.variables.clone());
                    result.push(combined);
                }
            }
            return Ok(result);
        }

        // Hash join optimization
        let mut hash_table: HashMap<Vec<QueryValue>, Vec<&QueryBinding>> = HashMap::new();

        // Build phase
        for right_binding in right_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    right_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in right binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();
            hash_table.entry(key).or_default().push(right_binding);
        }

        // Probe phase
        let mut result = Vec::new();
        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    left_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in left binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            if let Some(matching_rights) = hash_table.get(&key) {
                for right_binding in matching_rights {
                    let mut combined = self.result_pool.get_binding();
                    combined.variables.extend(left_binding.variables.clone());
                    combined.variables.extend(right_binding.variables.clone());
                    result.push(combined);
                }
            }
        }

        Ok(result)
    }

    /// Evaluate optional pattern with optimization
    fn evaluate_optional_optimized(
        &self,
        base: &ExecutionPlan,
        optional: &ExecutionPlan,
    ) -> OwlResult<Vec<QueryBinding>> {
        let base_bindings = self.execute_compiled_pattern(&CompiledPattern {
            pattern: QueryPattern::BasicGraphPattern(Vec::new()),
            hash: 0,
            execution_plan: base.clone(),
            variable_positions: Vec::new(),
        })?;

        if base_bindings.is_empty() {
            return Ok(base_bindings);
        }

        let optional_bindings = self.execute_compiled_pattern(&CompiledPattern {
            pattern: QueryPattern::BasicGraphPattern(Vec::new()),
            hash: 0,
            execution_plan: optional.clone(),
            variable_positions: Vec::new(),
        })?;

        // Left outer join logic
        let mut result = Vec::new();
        for base_binding in base_bindings {
            let mut found_match = false;
            for opt_binding in &optional_bindings {
                if let Some(joined) = self.join_bindings_optimized(&base_binding, opt_binding) {
                    result.push(joined);
                    found_match = true;
                }
            }
            if !found_match {
                result.push(base_binding.clone());
            }
        }

        Ok(result)
    }

    /// Execute compiled filter pattern
    fn execute_compiled_filter(&self, plan: &ExecutionPlan) -> OwlResult<Vec<QueryBinding>> {
        match plan {
            ExecutionPlan::SingleTriple {
                query_type,
                pattern,
            } => self.match_triple_pattern_optimized_type(pattern, query_type),
            _ => {
                // For complex patterns, fall back to the original method
                self.evaluate_basic_graph_pattern(&[])
            }
        }
    }

    /// Evaluate a basic graph pattern using hash joins for optimization
    fn evaluate_basic_graph_pattern(
        &self,
        triples: &[TriplePattern],
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        if triples.is_empty() {
            return Ok(bindings);
        }

        // Start with the first triple pattern
        let first_bindings = self.match_triple_pattern_optimized(&triples[0])?;
        bindings = first_bindings;

        // Join with remaining triple patterns using hash joins
        for triple in triples.iter().skip(1) {
            let triple_bindings = self.match_triple_pattern_optimized(triple)?;
            bindings = self.hash_join_bindings(&bindings, &triple_bindings)?;

            if bindings.is_empty() {
                break; // No more matches possible
            }
        }

        Ok(bindings)
    }

    /// Match a single triple pattern against the ontology using indexed storage
    fn match_triple_pattern_optimized(
        &self,
        triple: &TriplePattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        let query_type = match &triple.predicate {
            PatternTerm::IRI(pred_iri) => {
                if pred_iri.as_str() == RDF_TYPE {
                    QueryType::TypeQuery
                } else {
                    QueryType::PropertyQuery
                }
            }
            _ => QueryType::VariablePredicate,
        };

        self.match_triple_pattern_optimized_type(triple, &query_type)
    }

    /// Match triple pattern against class assertion (optimized)
    fn match_class_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::ClassAssertionAxiom,
    ) -> Option<QueryBinding> {
        let individual_iri = axiom.individual();
        let individual_term = pattern_term_from_arc(individual_iri);

        if self.is_class_assertion_match(triple, &individual_term, axiom.class_expr()) {
            Some(self.create_class_assertion_binding(triple, &individual_term, axiom.class_expr()))
        } else {
            None
        }
    }

    /// Check if a class assertion matches the triple pattern
    fn is_class_assertion_match(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        class_expr: &crate::axioms::ClassExpression,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, individual_term);
        // Use cached rdf:type term instead of creating new IRI (optimization)
        let predicate_match = self.match_term(&triple.predicate, &RDF_TYPE_TERM);
        let object_match = self.match_class_expr_term(&triple.object, class_expr);

        subject_match && predicate_match && object_match
    }

    /// Create a binding for a class assertion match
    fn create_class_assertion_binding(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        class_expr: &crate::axioms::ClassExpression,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, individual_term);
        self.add_class_expr_binding(&mut binding, &triple.object, class_expr);

        binding
    }

    /// Match triple pattern against property assertion (optimized)
    fn match_property_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> Option<QueryBinding> {
        let subject_iri = axiom.subject();
        let property_iri = axiom.property();

        let subject_term = pattern_term_from_arc(subject_iri);
        let property_term = pattern_term_from_arc(property_iri);

        if self.is_property_assertion_match(triple, &subject_term, &property_term, axiom) {
            Some(self.create_property_assertion_binding(
                triple,
                &subject_term,
                &property_term,
                axiom,
            ))
        } else {
            None
        }
    }

    /// Check if a property assertion matches the triple pattern
    fn is_property_assertion_match(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, subject_term);
        let predicate_match = self.match_term(&triple.predicate, property_term);
        let object_match = self.match_property_object(&triple.object, axiom);

        subject_match && predicate_match && object_match
    }

    /// Match property object term
    fn match_property_object(
        &self,
        object_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        if let Some(object_iri) = axiom.object_iri() {
            self.match_term(object_term, &pattern_term_from_arc(object_iri))
        } else {
            // Skip anonymous individuals in query matching for now
            false
        }
    }

    /// Create a binding for a property assertion match
    fn create_property_assertion_binding(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, subject_term);
        self.add_binding(&mut binding, &triple.predicate, property_term);

        if let Some(object_iri) = axiom.object_iri() {
            self.add_binding(
                &mut binding,
                &triple.object,
                &pattern_term_from_arc(object_iri),
            );
        }

        binding
    }

    /// Match two pattern terms
    fn match_term(&self, pattern: &PatternTerm, value: &PatternTerm) -> bool {
        match (pattern, value) {
            (PatternTerm::Variable(_), _) => true,
            (PatternTerm::IRI(pattern_iri), PatternTerm::IRI(value_iri)) => {
                pattern_iri == value_iri
            }
            (PatternTerm::Literal(pattern_lit), PatternTerm::Literal(value_lit)) => {
                pattern_lit == value_lit
            }
            _ => false,
        }
    }

    /// Match pattern term against class expression
    fn match_class_expr_term(&self, pattern: &PatternTerm, class_expr: &ClassExpression) -> bool {
        match pattern {
            PatternTerm::Variable(_) => true,
            PatternTerm::IRI(iri) => class_expr.contains_class(iri),
            _ => false,
        }
    }

    /// Add binding from pattern term to value
    fn add_binding(&self, binding: &mut QueryBinding, pattern: &PatternTerm, value: &PatternTerm) {
        if let PatternTerm::Variable(var_name) = pattern {
            let query_value = match value {
                PatternTerm::IRI(iri) => QueryValue::IRI(iri.clone()),
                PatternTerm::Literal(lit) => QueryValue::Literal(lit.clone()),
                PatternTerm::Variable(_) => return, // Can't bind variable to variable
            };

            binding.variables.insert(var_name.clone(), query_value);
        }
    }

    /// Add binding from pattern term to class expression
    fn add_class_expr_binding(
        &self,
        binding: &mut QueryBinding,
        pattern: &PatternTerm,
        class_expr: &ClassExpression,
    ) {
        if let PatternTerm::Variable(var_name) = pattern {
            if let ClassExpression::Class(class) = class_expr {
                binding
                    .variables
                    .insert(var_name.clone(), QueryValue::IRI((**class.iri()).clone()));
            }
        }
    }

    /// Optimized join operation with memory reuse
    fn join_bindings_optimized(
        &self,
        binding1: &QueryBinding,
        binding2: &QueryBinding,
    ) -> Option<QueryBinding> {
        let mut joined = self.result_pool.get_binding();

        for (var, value) in &binding1.variables {
            joined.variables.insert(var.clone(), value.clone());
        }

        for (var, value) in &binding2.variables {
            if let Some(existing_value) = joined.variables.get(var) {
                if existing_value != value {
                    self.result_pool.return_binding(joined);
                    return None; // Variable conflict
                }
            } else {
                joined.variables.insert(var.clone(), value.clone());
            }
        }

        Some(joined)
    }

    /// Perform hash join between two sets of bindings
    fn hash_join_bindings(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &[QueryBinding],
    ) -> OwlResult<Vec<QueryBinding>> {
        if left_bindings.is_empty() || right_bindings.is_empty() {
            return Ok(Vec::new());
        }

        // Find common variables between left and right bindings
        let left_vars: HashSet<String> = left_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();
        let right_vars: HashSet<String> = right_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();

        let common_vars: Vec<String> = left_vars.intersection(&right_vars).cloned().collect();

        if common_vars.is_empty() {
            // No common variables - return cartesian product
            let mut result = Vec::new();
            for left in left_bindings {
                for right in right_bindings {
                    let mut combined = left.clone();
                    combined.variables.extend(right.variables.clone());
                    result.push(combined);
                }
            }
            return Ok(result);
        }

        // Use hash join for common variables
        let mut hash_table: HashMap<Vec<QueryValue>, Vec<&QueryBinding>> = HashMap::new();

        // Build hash table from right bindings
        for right_binding in right_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    right_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in right binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            hash_table.entry(key).or_default().push(right_binding);
        }

        // Probe with left bindings
        let mut result = Vec::new();
        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    left_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in left binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            if let Some(matching_rights) = hash_table.get(&key) {
                for right_binding in matching_rights {
                    let mut combined = left_binding.clone();
                    combined.variables.extend(right_binding.variables.clone());
                    result.push(combined);
                }
            }
        }

        Ok(result)
    }

    /// Apply filter expression to bindings
    fn apply_filter(
        &self,
        bindings: &[QueryBinding],
        expression: &FilterExpression,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut filtered_bindings = Vec::new();

        for binding in bindings {
            if self.evaluate_filter_expression(binding, expression) {
                filtered_bindings.push(binding.clone());
            }
        }

        Ok(filtered_bindings)
    }

    /// Evaluate filter expression for a binding
    fn evaluate_filter_expression(
        &self,
        binding: &QueryBinding,
        expression: &FilterExpression,
    ) -> bool {
        match expression {
            FilterExpression::Equals { left, right } => {
                let left_value = self.evaluate_term(binding, left);
                let right_value = self.evaluate_term(binding, right);
                left_value == right_value
            }
            FilterExpression::Type { term, type_iri: _ } => {
                if let Some(QueryValue::IRI(_iri)) = self.evaluate_term_opt(binding, term) {
                    // Check if the IRI has the specified type
                    // This is simplified - in practice, we'd need to reason about types
                    false // Placeholder implementation
                } else {
                    false
                }
            }
            FilterExpression::And(expressions) => expressions
                .iter()
                .all(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Or(expressions) => expressions
                .iter()
                .any(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Not(expr) => !self.evaluate_filter_expression(binding, expr),
        }
    }

    /// Evaluate pattern term to query value
    fn evaluate_term(&self, binding: &QueryBinding, term: &PatternTerm) -> QueryValue {
        self.evaluate_term_opt(binding, term)
            .unwrap_or(QueryValue::Literal("".to_string()))
    }

    /// Evaluate pattern term to query value (optional)
    fn evaluate_term_opt(&self, binding: &QueryBinding, term: &PatternTerm) -> Option<QueryValue> {
        match term {
            PatternTerm::Variable(var_name) => binding.variables.get(var_name).cloned(),
            PatternTerm::IRI(iri) => Some(QueryValue::IRI(iri.clone())),
            PatternTerm::Literal(lit) => Some(QueryValue::Literal(lit.clone())),
        }
    }
}

impl QueryEngine {
    /// Determine the type of query based on the triple pattern
    fn determine_query_type(&self, triple: &TriplePattern) -> QueryType {
        match &triple.predicate {
            PatternTerm::IRI(pred_iri) => {
                if pred_iri.as_str() == RDF_TYPE {
                    QueryType::TypeQuery
                } else {
                    QueryType::PropertyQuery
                }
            }
            _ => QueryType::VariablePredicate,
        }
    }

    /// Collect bindings for type queries (rdf:type)
    fn collect_type_query_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.class_assertions() {
            if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }

    /// Collect bindings for property queries
    fn collect_property_query_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.property_assertions() {
            if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }

    /// Collect bindings for variable predicate queries
    fn collect_variable_predicate_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.class_assertions() {
            if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }

        for axiom in self.ontology.property_assertions() {
            if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }
    /// Create a new query engine
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, QueryConfig::default())
    }

    /// Create a new query engine with custom configuration
    pub fn with_config(ontology: Ontology, config: QueryConfig) -> Self {
        let ontology = Arc::new(ontology);
        let reasoner = if config.enable_reasoning {
            // This would be initialized with a proper reasoner
            None
        } else {
            None
        };

        // Initialize query cache
        let cache_size = config.cache_size.unwrap_or(1000);
        let cache_size = NonZeroUsize::new(cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(1000).expect("Failed to create cache size"));
        let query_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        // Initialize pattern cache
        let pattern_cache = Arc::new(RwLock::new(HashMap::new()));

        // Initialize memory pool
        let result_pool = Arc::new(ResultPool::new());

        // Initialize indexes
        let type_index = Arc::new(DashMap::new());
        let property_index = Arc::new(DashMap::new());

        // Build indexes from ontology
        let engine = QueryEngine {
            ontology: ontology.clone(),
            reasoner,
            config: config.clone(),
            query_cache,
            pattern_cache,
            result_pool,
            type_index,
            property_index,
            stats: Arc::new(RwLock::new(QueryEngineStats::default())),
        };

        // Build indexes for fast access
        engine.build_indexes();

        engine
    }

    /// Build indexes for fast pattern matching
    fn build_indexes(&self) {
        // Index class assertions by type
        for axiom in self.ontology.class_assertions() {
            let class_expr = axiom.class_expr();
            if let ClassExpression::Class(class) = class_expr {
                let class_iri = class.iri().clone();
                self.type_index
                    .entry(class_iri)
                    .or_default()
                    .push(Arc::new(axiom.clone()));
            }
        }

        // Index property assertions by property
        for axiom in self.ontology.property_assertions() {
            let prop_iri = axiom.property().clone();
            self.property_index
                .entry(prop_iri)
                .or_default()
                .push(Arc::new(axiom.clone()));
        }
    }

    /// Get query engine statistics
    pub fn get_stats(&self) -> QueryEngineStats {
        self.stats.read().clone()
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.query_cache.write().clear();
        self.pattern_cache.write().clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize, f64) {
        let cache = self.query_cache.read();
        let stats = self.stats.read();
        let hits = stats.cache_hits as f64;
        let misses = stats.cache_misses as f64;
        let hit_rate = if hits + misses > 0.0 {
            hits / (hits + misses) * 100.0
        } else {
            0.0
        };

        (cache.len(), stats.cache_hits as usize, hit_rate)
    }

    /// Get parallel execution statistics
    pub fn get_parallel_stats(&self) -> (u64, u64) {
        let stats = self.stats.read();
        (
            stats.parallel_queries_executed,
            stats.parallel_execution_time_ms,
        )
    }

    /// Configure parallel execution settings
    pub fn configure_parallel(
        &mut self,
        enable_parallel: bool,
        max_threads: Option<usize>,
        threshold: usize,
    ) {
        self.config.enable_parallel = enable_parallel;
        self.config.max_parallel_threads = max_threads;
        self.config.parallel_threshold = threshold;
    }

    /// Force parallel execution for a specific query
    pub fn execute_query_parallel(&mut self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        // Store original config
        let original_parallel = self.config.enable_parallel;
        let original_threshold = self.config.parallel_threshold;

        // Force parallel execution
        self.config.enable_parallel = true;
        self.config.parallel_threshold = 1; // Always use parallel

        let result = self.execute_query(pattern);

        // Restore original config
        self.config.enable_parallel = original_parallel;
        self.config.parallel_threshold = original_threshold;

        result
    }

    /// Force sequential execution for a specific query
    pub fn execute_query_sequential(&mut self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        // Store original config
        let original_parallel = self.config.enable_parallel;

        // Force sequential execution
        self.config.enable_parallel = false;

        let result = self.execute_query(pattern);

        // Restore original config
        self.config.enable_parallel = original_parallel;

        result
    }

    /// Execute a query with advanced optimizations
    pub fn execute_query(&mut self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let start_time = std::time::Instant::now();

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.queries_executed += 1;
        }

        // Check cache if enabled
        if self.config.enable_caching {
            if let Some(cached_result) = self.check_query_cache(pattern) {
                {
                    let mut stats = self.stats.write();
                    stats.cache_hits += 1;
                }
                return Ok(cached_result);
            }
            {
                let mut stats = self.stats.write();
                stats.cache_misses += 1;
            }
        }

        // Get or compile execution plan
        let compiled_pattern = self.get_or_compile_pattern(pattern)?;
        let bindings = self.execute_compiled_pattern(&compiled_pattern)?;

        // Apply result limit
        let bindings = if let Some(max_results) = self.config.max_results {
            if bindings.len() > max_results {
                bindings.into_iter().take(max_results).collect()
            } else {
                bindings
            }
        } else {
            bindings
        };

        let variables = compiled_pattern.variable_positions.clone();
        let time_ms = start_time.elapsed().as_millis() as u64;

        // Update total execution time
        {
            let mut stats = self.stats.write();
            stats.total_execution_time_ms += time_ms;
        }

        let results_count = bindings.len();
        let result = QueryResult {
            bindings,
            variables,
            stats: QueryStats {
                results_count,
                time_ms,
                reasoning_used: self.config.enable_reasoning,
            },
        };

        // Cache result if enabled
        if self.config.enable_caching {
            self.cache_query_result(pattern, result.clone());
        }

        Ok(result)
    }

    /// Check query cache for existing results
    fn check_query_cache(&self, pattern: &QueryPattern) -> Option<QueryResult> {
        let cache_key = self.create_cache_key(pattern);
        let mut cache = self.query_cache.write();
        cache.get(&cache_key).cloned()
    }

    /// Cache query result for future use
    fn cache_query_result(&self, pattern: &QueryPattern, result: QueryResult) {
        let cache_key = self.create_cache_key(pattern);
        let mut cache = self.query_cache.write();
        cache.put(cache_key, result);
    }

    /// Create cache key for query pattern
    fn create_cache_key(&self, pattern: &QueryPattern) -> QueryCacheKey {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        let pattern_hash = hasher.finish();

        let mut config_hasher = DefaultHasher::new();
        self.config.enable_reasoning.hash(&mut config_hasher);
        self.config.max_results.hash(&mut config_hasher);
        let config_hash = config_hasher.finish();

        QueryCacheKey {
            pattern_hash,
            config_hash,
        }
    }

    /// Get compiled pattern or compile if not exists
    fn get_or_compile_pattern(&self, pattern: &QueryPattern) -> OwlResult<CompiledPattern> {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        let pattern_hash = hasher.finish();

        // Check pattern cache
        {
            let cache = self.pattern_cache.write();
            if let Some(compiled) = cache.get(&pattern_hash) {
                return Ok(compiled.clone());
            }
        }

        // Compile pattern
        let compiled = self.compile_pattern(pattern)?;

        // Cache compiled pattern
        {
            let mut cache = self.pattern_cache.write();
            cache.insert(pattern_hash, compiled.clone());
        }

        Ok(compiled)
    }

    /// Compile query pattern into optimized execution plan
    fn compile_pattern(&self, pattern: &QueryPattern) -> OwlResult<CompiledPattern> {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        pattern.hash(&mut hasher);
        let hash = hasher.finish();

        let execution_plan = match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                if triples.len() == 1 {
                    ExecutionPlan::SingleTriple {
                        query_type: self.determine_query_type(&triples[0]),
                        pattern: triples[0].clone(),
                    }
                } else {
                    // Optimize join order based on selectivity
                    let (join_order, access_paths) = self.optimize_join_order(triples);
                    ExecutionPlan::MultiTriple {
                        patterns: triples.clone(),
                        join_order,
                        access_paths,
                    }
                }
            }
            QueryPattern::OptionalPattern(pattern) => {
                let base_plan = self.compile_pattern(pattern)?;
                ExecutionPlan::Optional {
                    base: Box::new(base_plan.execution_plan),
                    optional: Box::new(ExecutionPlan::SingleTriple {
                        query_type: QueryType::TypeQuery,
                        pattern: TriplePattern {
                            subject: PatternTerm::Variable("?opt".to_string()),
                            predicate: RDF_TYPE_TERM.clone(),
                            object: PatternTerm::Variable("?type".to_string()),
                        },
                    }),
                }
            }
            QueryPattern::UnionPattern(patterns) => {
                let plans = patterns
                    .iter()
                    .map(|p| self.compile_pattern(p).map(|c| c.execution_plan))
                    .collect::<OwlResult<Vec<_>>>()?;
                ExecutionPlan::Union { plans }
            }
            QueryPattern::FilterPattern {
                pattern,
                expression,
            } => {
                let base_plan = self.compile_pattern(pattern)?;
                ExecutionPlan::Filter {
                    base: Box::new(base_plan.execution_plan),
                    filter_expr: expression.clone(),
                }
            }
        };

        let variables = self.extract_variables(pattern);

        Ok(CompiledPattern {
            pattern: pattern.clone(),
            hash,
            execution_plan,
            variable_positions: variables,
        })
    }

    /// Optimize join order for multi-triple patterns
    fn optimize_join_order(&self, triples: &[TriplePattern]) -> (Vec<usize>, Vec<QueryType>) {
        let mut patterns: Vec<(usize, TriplePattern, QueryType)> = triples
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.clone(), self.determine_query_type(t)))
            .collect();

        // Sort by estimated selectivity (type queries first, then property queries)
        patterns.sort_by(|a, b| match (&a.2, &b.2) {
            (QueryType::TypeQuery, QueryType::PropertyQuery) => std::cmp::Ordering::Less,
            (QueryType::PropertyQuery, QueryType::TypeQuery) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });

        let join_order: Vec<usize> = patterns.iter().map(|(i, _, _)| *i).collect();
        let access_paths: Vec<QueryType> = patterns.iter().map(|(_, _, t)| t.clone()).collect();

        (join_order, access_paths)
    }

    /// Execute compiled pattern with optimized plan
    fn execute_compiled_pattern(&self, compiled: &CompiledPattern) -> OwlResult<Vec<QueryBinding>> {
        match &compiled.execution_plan {
            ExecutionPlan::SingleTriple {
                query_type,
                pattern,
            } => self.match_triple_pattern_optimized_type(pattern, query_type),
            ExecutionPlan::MultiTriple {
                patterns,
                join_order,
                access_paths,
            } => self.evaluate_multi_triple_optimized(patterns, join_order, access_paths),
            ExecutionPlan::Optional { base, optional } => {
                self.evaluate_optional_optimized(base, optional)
            }
            ExecutionPlan::Union { plans } => self.evaluate_union_optimized(plans),
            ExecutionPlan::Filter { base, filter_expr } => {
                let bindings = self.execute_compiled_filter(base)?;
                self.apply_filter(&bindings, filter_expr)
            }
        }
    }

    /// Execute single triple pattern with type-specific optimization
    fn match_triple_pattern_optimized_type(
        &self,
        triple: &TriplePattern,
        query_type: &QueryType,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        match query_type {
            QueryType::TypeQuery => {
                self.collect_type_query_bindings_optimized(triple, &mut bindings);
            }
            QueryType::PropertyQuery => {
                self.collect_property_query_bindings_optimized(triple, &mut bindings);
            }
            QueryType::VariablePredicate => {
                self.collect_variable_predicate_bindings_optimized(triple, &mut bindings);
            }
        }

        Ok(bindings)
    }

    /// Optimized type query collection using index
    fn collect_type_query_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use index when object is a specific type
        if let PatternTerm::IRI(type_iri) = &triple.object {
            if let Some(axioms) = self.type_index.get(type_iri) {
                for axiom in axioms.iter() {
                    if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                        bindings.push(binding);
                    }
                }
            }
        } else {
            // Fall back to linear scan for variable types
            for axiom in self.ontology.class_assertions() {
                if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                    bindings.push(binding);
                }
            }
        }
    }

    /// Optimized property query collection using index
    fn collect_property_query_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use index when predicate is a specific property
        if let PatternTerm::IRI(prop_iri) = &triple.predicate {
            if let Some(axioms) = self.property_index.get(prop_iri) {
                for axiom in axioms.iter() {
                    if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                        bindings.push(binding);
                    }
                }
            }
        } else {
            // Fall back to linear scan for variable predicates
            for axiom in self.ontology.property_assertions() {
                if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                    bindings.push(binding);
                }
            }
        }
    }

    /// Optimized variable predicate query collection
    fn collect_variable_predicate_bindings_optimized(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        // Use indexes for both type and property assertions
        self.collect_type_query_bindings_optimized(triple, bindings);
        self.collect_property_query_bindings_optimized(triple, bindings);
    }

    /// Execute multi-triple pattern with optimized joins
    fn evaluate_multi_triple_optimized(
        &self,
        patterns: &[TriplePattern],
        join_order: &[usize],
        access_paths: &[QueryType],
    ) -> OwlResult<Vec<QueryBinding>> {
        if join_order.is_empty() {
            return Ok(Vec::new());
        }

        // Start with the most selective pattern
        let first_idx = join_order[0];
        let mut bindings =
            self.match_triple_pattern_optimized_type(&patterns[first_idx], &access_paths[0])?;

        // Join with remaining patterns
        for &idx in join_order.iter().skip(1) {
            let pattern_bindings =
                self.match_triple_pattern_optimized_type(&patterns[idx], &access_paths[idx])?;
            bindings = self.hash_join_bindings_optimized(&bindings, &pattern_bindings)?;

            if bindings.is_empty() {
                break; // Early termination
            }
        }

        Ok(bindings)
    }

    /// Optimized hash join with memory reuse
    fn hash_join_bindings_optimized(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &[QueryBinding],
    ) -> OwlResult<Vec<QueryBinding>> {
        if left_bindings.is_empty() || right_bindings.is_empty() {
            return Ok(Vec::new());
        }

        // Find common variables
        let left_vars: HashSet<String> = left_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();
        let right_vars: HashSet<String> = right_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();

        let common_vars: Vec<String> = left_vars.intersection(&right_vars).cloned().collect();

        if common_vars.is_empty() {
            // Cartesian product
            let mut result = Vec::with_capacity(left_bindings.len() * right_bindings.len());
            for left in left_bindings {
                for right in right_bindings {
                    let mut combined = self.result_pool.get_binding();
                    combined.variables.extend(left.variables.clone());
                    combined.variables.extend(right.variables.clone());
                    result.push(combined);
                }
            }
            return Ok(result);
        }

        // Hash join optimization
        let mut hash_table: HashMap<Vec<QueryValue>, Vec<&QueryBinding>> = HashMap::new();

        // Build phase
        for right_binding in right_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    right_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in right binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();
            hash_table.entry(key).or_default().push(right_binding);
        }

        // Probe phase
        let mut result = Vec::new();
        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    left_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in left binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            if let Some(matching_rights) = hash_table.get(&key) {
                for right_binding in matching_rights {
                    let mut combined = self.result_pool.get_binding();
                    combined.variables.extend(left_binding.variables.clone());
                    combined.variables.extend(right_binding.variables.clone());
                    result.push(combined);
                }
            }
        }

        Ok(result)
    }

    /// Execute compiled filter pattern
    fn execute_compiled_filter(&self, plan: &ExecutionPlan) -> OwlResult<Vec<QueryBinding>> {
        match plan {
            ExecutionPlan::SingleTriple {
                query_type,
                pattern,
            } => self.match_triple_pattern_optimized_type(pattern, query_type),
            _ => {
                // For complex patterns, fall back to the original method
                self.evaluate_basic_graph_pattern(&[])
            }
        }
    }

    /// Evaluate optional pattern with optimization
    fn evaluate_optional_optimized(
        &self,
        base: &ExecutionPlan,
        optional: &ExecutionPlan,
    ) -> OwlResult<Vec<QueryBinding>> {
        let base_bindings = self.execute_compiled_pattern(&CompiledPattern {
            pattern: QueryPattern::BasicGraphPattern(Vec::new()),
            hash: 0,
            execution_plan: base.clone(),
            variable_positions: Vec::new(),
        })?;

        if base_bindings.is_empty() {
            return Ok(base_bindings);
        }

        let optional_bindings = self.execute_compiled_pattern(&CompiledPattern {
            pattern: QueryPattern::BasicGraphPattern(Vec::new()),
            hash: 0,
            execution_plan: optional.clone(),
            variable_positions: Vec::new(),
        })?;

        // Left outer join logic
        let mut result = Vec::new();
        for base_binding in base_bindings {
            let mut found_match = false;
            for opt_binding in &optional_bindings {
                if let Some(joined) = self.join_bindings_optimized(&base_binding, opt_binding) {
                    result.push(joined);
                    found_match = true;
                }
            }
            if !found_match {
                result.push(base_binding.clone());
            }
        }

        Ok(result)
    }

    /// Evaluate union pattern with parallel execution using Rayon
    fn evaluate_union_optimized(&self, plans: &[ExecutionPlan]) -> OwlResult<Vec<QueryBinding>> {
        if plans.is_empty() {
            return Ok(Vec::new());
        }

        // Determine if we should use parallel execution
        let use_parallel =
            self.config.enable_parallel && plans.len() >= self.config.parallel_threshold;

        if use_parallel {
            self.evaluate_union_parallel(plans)
        } else {
            self.evaluate_union_sequential(plans)
        }
    }

    /// Sequential union evaluation for small queries or when parallel is disabled
    fn evaluate_union_sequential(&self, plans: &[ExecutionPlan]) -> OwlResult<Vec<QueryBinding>> {
        let mut all_bindings = Vec::new();
        for plan in plans {
            let plan_bindings = self.execute_compiled_pattern(&CompiledPattern {
                pattern: QueryPattern::BasicGraphPattern(Vec::new()),
                hash: 0,
                execution_plan: plan.clone(),
                variable_positions: Vec::new(),
            })?;
            all_bindings.extend(plan_bindings);
        }
        Ok(all_bindings)
    }

    /// Parallel union evaluation using Rayon for improved performance
    fn evaluate_union_parallel(&self, plans: &[ExecutionPlan]) -> OwlResult<Vec<QueryBinding>> {
        let start_time = std::time::Instant::now();

        // Configure Rayon thread pool if max threads is specified
        let thread_pool = if let Some(max_threads) = self.config.max_parallel_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .map_err(|e| OwlError::Other(format!("Failed to create thread pool: {}", e)))?
        } else {
            // Use default Rayon global thread pool
            rayon::ThreadPoolBuilder::new()
                .build()
                .map_err(|e| OwlError::Other(format!("Failed to create thread pool: {}", e)))?
        };

        // Create shared references for thread safety
        let ontology_ref = self.ontology.clone();
        let type_index_ref = self.type_index.clone();
        let property_index_ref = self.property_index.clone();
        let result_pool_ref = self.result_pool.clone();

        // Execute plans in parallel
        let results: Result<Vec<_>, _> = thread_pool.install(|| {
            plans
                .par_iter()
                .map(|plan| {
                    // Create a temporary query engine context for this thread
                    let thread_context = ThreadQueryContext {
                        ontology: &ontology_ref,
                        type_index: &type_index_ref,
                        property_index: &property_index_ref,
                        result_pool: &result_pool_ref,
                    };

                    thread_context.execute_plan(plan)
                })
                .collect()
        });

        let all_bindings: Vec<QueryBinding> = results?.into_iter().flatten().collect();

        // Update parallel execution statistics
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        {
            let mut stats = self.stats.write();
            stats.parallel_queries_executed += 1;
            stats.parallel_execution_time_ms += execution_time_ms;
        }

        Ok(all_bindings)
    }

    /// Optimized join operation with memory reuse
    fn join_bindings_optimized(
        &self,
        binding1: &QueryBinding,
        binding2: &QueryBinding,
    ) -> Option<QueryBinding> {
        let mut joined = self.result_pool.get_binding();

        for (var, value) in &binding1.variables {
            joined.variables.insert(var.clone(), value.clone());
        }

        for (var, value) in &binding2.variables {
            if let Some(existing_value) = joined.variables.get(var) {
                if existing_value != value {
                    self.result_pool.return_binding(joined);
                    return None; // Variable conflict
                }
            } else {
                joined.variables.insert(var.clone(), value.clone());
            }
        }

        Some(joined)
    }

    /// Evaluate a basic graph pattern using hash joins for optimization
    fn evaluate_basic_graph_pattern(
        &self,
        triples: &[TriplePattern],
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        if triples.is_empty() {
            return Ok(bindings);
        }

        // Start with the first triple pattern
        let first_bindings = self.match_triple_pattern_optimized(&triples[0])?;
        bindings = first_bindings;

        // Join with remaining triple patterns using hash joins
        for triple in triples.iter().skip(1) {
            let triple_bindings = self.match_triple_pattern_optimized(triple)?;
            bindings = self.hash_join_bindings(&bindings, &triple_bindings)?;

            if bindings.is_empty() {
                break; // No more matches possible
            }
        }

        Ok(bindings)
    }

    /// Match a single triple pattern against the ontology using indexed storage
    fn match_triple_pattern_optimized(
        &self,
        triple: &TriplePattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        match self.determine_query_type(triple) {
            QueryType::TypeQuery => {
                self.collect_type_query_bindings(triple, &mut bindings);
            }
            QueryType::PropertyQuery => {
                self.collect_property_query_bindings(triple, &mut bindings);
            }
            QueryType::VariablePredicate => {
                self.collect_variable_predicate_bindings(triple, &mut bindings);
            }
        }

        Ok(bindings)
    }

    /// Match triple pattern against class assertion (optimized)
    fn match_class_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::ClassAssertionAxiom,
    ) -> Option<QueryBinding> {
        let type_iri = IRI::new(RDF_TYPE)
            .map_err(|e| OwlError::IriParseError {
                iri: RDF_TYPE.to_string(),
                context: format!("Failed to create rdf:type IRI: {}", e),
            })
            .expect("Failed to create rdf:type IRI");

        let individual_iri = axiom.individual();
        let individual_term = pattern_term_from_arc(individual_iri);

        if self.is_class_assertion_match(triple, &individual_term, &type_iri, axiom.class_expr()) {
            Some(self.create_class_assertion_binding(triple, &individual_term, axiom.class_expr()))
        } else {
            None
        }
    }

    /// Check if a class assertion matches the triple pattern
    fn is_class_assertion_match(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        _type_iri: &IRI, // Parameter kept for compatibility but no longer used
        class_expr: &crate::axioms::ClassExpression,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, individual_term);
        // Use cached rdf:type term instead of creating new IRI (optimization)
        let predicate_match = self.match_term(&triple.predicate, &RDF_TYPE_TERM);
        let object_match = self.match_class_expr_term(&triple.object, class_expr);

        subject_match && predicate_match && object_match
    }

    /// Create a binding for a class assertion match
    fn create_class_assertion_binding(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        class_expr: &crate::axioms::ClassExpression,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, individual_term);
        self.add_class_expr_binding(&mut binding, &triple.object, class_expr);

        binding
    }

    /// Match triple pattern against property assertion (optimized)
    fn match_property_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> Option<QueryBinding> {
        let subject_iri = axiom.subject();
        let property_iri = axiom.property();

        let subject_term = pattern_term_from_arc(subject_iri);
        let property_term = pattern_term_from_arc(property_iri);

        if self.is_property_assertion_match(triple, &subject_term, &property_term, axiom) {
            Some(self.create_property_assertion_binding(
                triple,
                &subject_term,
                &property_term,
                axiom,
            ))
        } else {
            None
        }
    }

    /// Check if a property assertion matches the triple pattern
    fn is_property_assertion_match(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, subject_term);
        let predicate_match = self.match_term(&triple.predicate, property_term);
        let object_match = self.match_property_object(&triple.object, axiom);

        subject_match && predicate_match && object_match
    }

    /// Match property object term
    fn match_property_object(
        &self,
        object_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        if let Some(object_iri) = axiom.object_iri() {
            self.match_term(object_term, &pattern_term_from_arc(object_iri))
        } else {
            // Skip anonymous individuals in query matching for now
            false
        }
    }

    /// Create a binding for a property assertion match
    fn create_property_assertion_binding(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, subject_term);
        self.add_binding(&mut binding, &triple.predicate, property_term);

        if let Some(object_iri) = axiom.object_iri() {
            self.add_binding(
                &mut binding,
                &triple.object,
                &pattern_term_from_arc(object_iri),
            );
        }

        binding
    }

    /// Perform hash join between two sets of bindings
    fn hash_join_bindings(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &[QueryBinding],
    ) -> OwlResult<Vec<QueryBinding>> {
        if left_bindings.is_empty() || right_bindings.is_empty() {
            return Ok(Vec::new());
        }

        // Find common variables between left and right bindings
        let left_vars: HashSet<String> = left_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();
        let right_vars: HashSet<String> = right_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();

        let common_vars: Vec<String> = left_vars.intersection(&right_vars).cloned().collect();

        if common_vars.is_empty() {
            // No common variables - return cartesian product
            let mut result = Vec::new();
            for left in left_bindings {
                for right in right_bindings {
                    let mut combined = left.clone();
                    combined.variables.extend(right.variables.clone());
                    result.push(combined);
                }
            }
            return Ok(result);
        }

        // Use hash join for common variables
        let mut hash_table: HashMap<Vec<QueryValue>, Vec<&QueryBinding>> = HashMap::new();

        // Build hash table from right bindings
        for right_binding in right_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    right_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in right binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            hash_table.entry(key).or_default().push(right_binding);
        }

        // Probe with left bindings
        let mut result = Vec::new();
        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| {
                    left_binding.variables.get(var).cloned().unwrap_or_else(|| {
                        panic!("Variable '{}' not found in left binding during hash join - this indicates a bug in join logic", var)
                    })
                })
                .collect();

            if let Some(matching_rights) = hash_table.get(&key) {
                for right_binding in matching_rights {
                    let mut combined = left_binding.clone();
                    combined.variables.extend(right_binding.variables.clone());
                    result.push(combined);
                }
            }
        }

        Ok(result)
    }

    /// Match triple pattern against class assertion
    #[allow(dead_code)]
    fn match_class_assertion(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::ClassAssertionAxiom,
    ) -> Option<QueryBinding> {
        // Try to match: individual rdf:type class
        let type_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| OwlError::IriParseError {
                iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                context: format!("Failed to create rdf:type IRI: {}", e),
            })
            .expect("Failed to create rdf:type IRI");

        let subject_match =
            self.match_term(&triple.subject, &pattern_term_from_arc(axiom.individual()));
        let predicate_match = self.match_term(&triple.predicate, &PatternTerm::IRI(type_iri));
        let object_match = self.match_class_expr_term(&triple.object, axiom.class_expr());

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(
                &mut binding,
                &triple.subject,
                &pattern_term_from_arc(axiom.individual()),
            );
            self.add_class_expr_binding(&mut binding, &triple.object, axiom.class_expr());

            Some(binding)
        } else {
            None
        }
    }

    /// Match triple pattern against property assertion
    #[allow(dead_code)]
    fn match_property_assertion(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> Option<QueryBinding> {
        let subject_match =
            self.match_term(&triple.subject, &pattern_term_from_arc(axiom.subject()));
        let predicate_match =
            self.match_term(&triple.predicate, &pattern_term_from_arc(axiom.property()));
        let object_match = if let Some(object_iri) = axiom.object_iri() {
            self.match_term(&triple.object, &PatternTerm::IRI((**object_iri).clone()))
        } else {
            // Skip anonymous individuals in query matching for now
            false
        };

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(
                &mut binding,
                &triple.subject,
                &pattern_term_from_arc(axiom.subject()),
            );
            self.add_binding(
                &mut binding,
                &triple.predicate,
                &pattern_term_from_arc(axiom.property()),
            );
            if let Some(object_iri) = axiom.object_iri() {
                self.add_binding(
                    &mut binding,
                    &triple.object,
                    &pattern_term_from_arc(object_iri),
                );
            }

            Some(binding)
        } else {
            None
        }
    }

    /// Match triple pattern against subclass axiom
    #[allow(dead_code)]
    fn match_subclass_axiom(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::SubClassOfAxiom,
    ) -> Option<QueryBinding> {
        let sub_iri = if let ClassExpression::Class(class) = axiom.sub_class() {
            class.iri()
        } else {
            return None;
        };

        let super_iri = if let ClassExpression::Class(class) = axiom.super_class() {
            class.iri()
        } else {
            return None;
        };

        let rdfs_subclassof = IRI::new("http://www.w3.org/2000/01/rdf-schema#subClassOf")
            .map_err(|e| OwlError::IriParseError {
                iri: "http://www.w3.org/2000/01/rdf-schema#subClassOf".to_string(),
                context: format!("Failed to create rdfs:subClassOf IRI: {}", e),
            })
            .ok()?;

        // Clone once and reuse terms
        let sub_term = pattern_term_from_arc(sub_iri);
        let super_term = pattern_term_from_arc(super_iri);
        let subclass_term = PatternTerm::IRI(rdfs_subclassof.clone());

        let subject_match = self.match_term(&triple.subject, &sub_term);
        let predicate_match = self.match_term(&triple.predicate, &subclass_term);
        let object_match = self.match_term(&triple.object, &super_term);

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(&mut binding, &triple.subject, &sub_term);
            self.add_binding(&mut binding, &triple.predicate, &subclass_term);
            self.add_binding(&mut binding, &triple.object, &super_term);

            Some(binding)
        } else {
            None
        }
    }

    /// Match two pattern terms
    fn match_term(&self, pattern: &PatternTerm, value: &PatternTerm) -> bool {
        match (pattern, value) {
            (PatternTerm::Variable(_), _) => true,
            (PatternTerm::IRI(pattern_iri), PatternTerm::IRI(value_iri)) => {
                pattern_iri == value_iri
            }
            (PatternTerm::Literal(pattern_lit), PatternTerm::Literal(value_lit)) => {
                pattern_lit == value_lit
            }
            _ => false,
        }
    }

    /// Match pattern term against class expression
    fn match_class_expr_term(&self, pattern: &PatternTerm, class_expr: &ClassExpression) -> bool {
        match pattern {
            PatternTerm::Variable(_) => true,
            PatternTerm::IRI(iri) => class_expr.contains_class(iri),
            _ => false,
        }
    }

    /// Add binding from pattern term to value
    fn add_binding(&self, binding: &mut QueryBinding, pattern: &PatternTerm, value: &PatternTerm) {
        if let PatternTerm::Variable(var_name) = pattern {
            let query_value = match value {
                PatternTerm::IRI(iri) => QueryValue::IRI(iri.clone()),
                PatternTerm::Literal(lit) => QueryValue::Literal(lit.clone()),
                PatternTerm::Variable(_) => return, // Can't bind variable to variable
            };

            binding.variables.insert(var_name.clone(), query_value);
        }
    }

    /// Add binding from pattern term to class expression
    fn add_class_expr_binding(
        &self,
        binding: &mut QueryBinding,
        pattern: &PatternTerm,
        class_expr: &ClassExpression,
    ) {
        if let PatternTerm::Variable(var_name) = pattern {
            if let ClassExpression::Class(class) = class_expr {
                binding
                    .variables
                    .insert(var_name.clone(), QueryValue::IRI((**class.iri()).clone()));
            }
        }
    }

    /// Join two bindings
    #[allow(dead_code)]
    fn join_bindings(
        &self,
        binding1: &QueryBinding,
        binding2: &QueryBinding,
    ) -> Option<QueryBinding> {
        let mut joined = binding1.clone();

        for (var, value) in &binding2.variables {
            if let Some(existing_value) = joined.variables.get(var) {
                if existing_value != value {
                    return None; // Variable conflict
                }
            } else {
                joined.variables.insert(var.clone(), value.clone());
            }
        }

        Some(joined)
    }

    /// Evaluate optional pattern
    #[allow(dead_code)]
    fn evaluate_optional_pattern(
        &mut self,
        pattern: &QueryPattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        // For optional patterns, we need to handle cases where the pattern may not match
        // This is a simplified implementation
        self.execute_query(pattern).map(|result| result.bindings)
    }

    /// Evaluate union pattern
    #[allow(dead_code)]
    fn evaluate_union_pattern(
        &mut self,
        patterns: &[QueryPattern],
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut all_bindings = Vec::new();

        for pattern in patterns {
            let pattern_bindings = self.execute_query(pattern)?;
            all_bindings.extend(pattern_bindings.bindings);
        }

        Ok(all_bindings)
    }

    /// Apply filter expression to bindings
    fn apply_filter(
        &self,
        bindings: &[QueryBinding],
        expression: &FilterExpression,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut filtered_bindings = Vec::new();

        for binding in bindings {
            if self.evaluate_filter_expression(binding, expression) {
                filtered_bindings.push(binding.clone());
            }
        }

        Ok(filtered_bindings)
    }

    /// Evaluate filter expression for a binding
    fn evaluate_filter_expression(
        &self,
        binding: &QueryBinding,
        expression: &FilterExpression,
    ) -> bool {
        match expression {
            FilterExpression::Equals { left, right } => {
                let left_value = self.evaluate_term(binding, left);
                let right_value = self.evaluate_term(binding, right);
                left_value == right_value
            }
            FilterExpression::Type { term, type_iri: _ } => {
                if let Some(QueryValue::IRI(_iri)) = self.evaluate_term_opt(binding, term) {
                    // Check if the IRI has the specified type
                    // This is simplified - in practice, we'd need to reason about types
                    false // Placeholder implementation
                } else {
                    false
                }
            }
            FilterExpression::And(expressions) => expressions
                .iter()
                .all(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Or(expressions) => expressions
                .iter()
                .any(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Not(expr) => !self.evaluate_filter_expression(binding, expr),
        }
    }

    /// Evaluate pattern term to query value
    fn evaluate_term(&self, binding: &QueryBinding, term: &PatternTerm) -> QueryValue {
        self.evaluate_term_opt(binding, term)
            .unwrap_or(QueryValue::Literal("".to_string()))
    }

    /// Evaluate pattern term to query value (optional)
    fn evaluate_term_opt(&self, binding: &QueryBinding, term: &PatternTerm) -> Option<QueryValue> {
        match term {
            PatternTerm::Variable(var_name) => binding.variables.get(var_name).cloned(),
            PatternTerm::IRI(iri) => Some(QueryValue::IRI(iri.clone())),
            PatternTerm::Literal(lit) => Some(QueryValue::Literal(lit.clone())),
        }
    }

    /// Extract variables from query pattern
    fn extract_variables(&self, pattern: &QueryPattern) -> Vec<String> {
        let mut variables = HashSet::new();

        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                for triple in triples {
                    self.extract_variables_from_term(&triple.subject, &mut variables);
                    self.extract_variables_from_term(&triple.predicate, &mut variables);
                    self.extract_variables_from_term(&triple.object, &mut variables);
                }
            }
            QueryPattern::OptionalPattern(pattern) => {
                variables.extend(self.extract_variables(pattern.as_ref()));
            }
            QueryPattern::UnionPattern(patterns) => {
                for pattern in patterns {
                    variables.extend(self.extract_variables(pattern));
                }
            }
            QueryPattern::FilterPattern {
                pattern,
                expression,
            } => {
                variables.extend(self.extract_variables(pattern.as_ref()));
                self.extract_variables_from_expression(expression, &mut variables);
            }
        }

        let mut sorted_vars: Vec<_> = variables.into_iter().collect();
        sorted_vars.sort();
        sorted_vars
    }

    /// Extract variables from pattern term
    fn extract_variables_from_term(&self, term: &PatternTerm, variables: &mut HashSet<String>) {
        if let PatternTerm::Variable(var_name) = term {
            variables.insert(var_name.clone());
        }
    }

    /// Extract variables from filter expression
    fn extract_variables_from_expression(
        &self,
        expression: &FilterExpression,
        variables: &mut HashSet<String>,
    ) {
        match expression {
            FilterExpression::Equals { left, right } => {
                self.extract_variables_from_term(left, variables);
                self.extract_variables_from_term(right, variables);
            }
            FilterExpression::Type { term, .. } => {
                self.extract_variables_from_term(term, variables);
            }
            FilterExpression::And(expressions) | FilterExpression::Or(expressions) => {
                for expr in expressions {
                    self.extract_variables_from_expression(expr, variables);
                }
            }
            FilterExpression::Not(expr) => {
                self.extract_variables_from_expression(expr, variables);
            }
        }
    }

    /// Get all classes in the ontology
    pub fn get_all_classes(&self) -> Vec<IRI> {
        self.ontology
            .classes()
            .iter()
            .map(|c| (**c.iri()).clone())
            .collect()
    }

    /// Get all properties in the ontology
    pub fn get_all_properties(&self) -> Vec<IRI> {
        let mut properties = Vec::new();

        for prop in self.ontology.object_properties() {
            properties.push((**prop.iri()).clone());
        }

        for prop in self.ontology.data_properties() {
            properties.push((**prop.iri()).clone());
        }

        properties
    }

    /// Get all individuals in the ontology
    pub fn get_all_individuals(&self) -> Vec<IRI> {
        self.ontology
            .named_individuals()
            .iter()
            .map(|i| (**i.iri()).clone())
            .collect()
    }
}
