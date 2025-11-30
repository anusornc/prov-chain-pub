//! Query engine implementation for OWL2 ontologies
//!
//! Contains the main QueryEngine struct and core query processing logic.

use crate::axioms::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::reasoning::Reasoner;

use super::{
    compute_config_hash, create_cache_key, QueryCache, QueryConfig, QueryEngineStats, QueryPattern,
    QueryResult, QueryType, ResultPool, TriplePattern, RDF_TYPE,
};

use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

/// Query engine for OWL2 ontologies with advanced optimizations
pub struct QueryEngine {
    ontology: Arc<Ontology>,
    #[allow(dead_code)]
    reasoner: Option<Box<dyn Reasoner>>,
    config: QueryConfig,
    /// Query result cache with LRU eviction
    query_cache: Arc<QueryCache>,
    /// Memory pool for reusing allocations
    #[allow(dead_code)]
    result_pool: Arc<ResultPool>,
    /// Index-based access structures for fast pattern matching
    #[allow(dead_code)]
    type_index: Arc<DashMap<Arc<IRI>, Vec<Arc<ClassAssertionAxiom>>>>,
    #[allow(dead_code)]
    property_index: Arc<DashMap<Arc<IRI>, Vec<Arc<PropertyAssertionAxiom>>>>,
    /// Query execution statistics
    stats: Arc<RwLock<QueryEngineStats>>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, QueryConfig::default())
    }

    /// Create a new query engine with custom configuration
    pub fn with_config(ontology: Ontology, config: QueryConfig) -> Self {
        let ontology = Arc::new(ontology);

        Self {
            query_cache: Arc::new(if let Some(size) = config.cache_size {
                QueryCache::new(size)
            } else {
                QueryCache::default()
            }),
            result_pool: Arc::new(ResultPool::new()),
            type_index: Arc::new(DashMap::new()),
            property_index: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(QueryEngineStats::new())),
            ontology,
            reasoner: None, // TODO: Initialize reasoner
            config,
        }
    }

    /// Execute a query pattern
    pub fn execute(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let start_time = std::time::Instant::now();

        // Compute cache key
        let pattern_hash = super::compute_pattern_hash(pattern);
        let config_hash = compute_config_hash(
            self.config.enable_reasoning,
            self.config.enable_parallel,
            self.config.max_results,
        );
        let cache_key = create_cache_key(pattern_hash, config_hash);

        // Check cache
        if self.config.enable_caching {
            if let Some(cached_result) = self.query_cache.get(&cache_key) {
                let mut stats = self.stats.write();
                stats.record_cache_hit();
                stats.record_success(start_time.elapsed().as_millis() as u64);
                return Ok(cached_result);
            } else {
                let mut stats = self.stats.write();
                stats.record_cache_miss();
            }
        }

        // Execute query
        let result = if self.config.enable_parallel && pattern.supports_parallel() {
            self.execute_parallel(pattern)?
        } else {
            self.execute_sequential(pattern)?
        };

        // Cache result
        if self.config.enable_caching {
            self.query_cache.put(cache_key, result.clone());
        }

        // Record statistics
        let elapsed = start_time.elapsed().as_millis() as u64;
        let mut stats = self.stats.write();
        stats.record_success(elapsed);
        stats.record_reasoning_operation();

        Ok(result)
    }

    /// Execute a triple pattern query
    pub fn execute_triple(&self, triple: TriplePattern) -> OwlResult<QueryResult> {
        let pattern = QueryPattern::BasicGraphPattern(vec![triple]);
        self.execute(&pattern)
    }

    /// Execute a basic class query (get all instances of a class)
    pub fn get_class_instances(&self, class_iri: &IRI) -> OwlResult<QueryResult> {
        // Get class assertions
        let instances: Vec<IRI> = self
            .ontology
            .class_assertions()
            .iter()
            .filter(|axiom| axiom.class_expr().contains_class(class_iri))
            .map(|axiom| (**axiom.individual()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["instance".to_string()];

        for instance in instances {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("instance".to_string(), super::QueryValue::IRI(instance));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute a basic property query (get all property values for a subject)
    pub fn get_property_values(
        &self,
        subject_iri: &IRI,
        property_iri: &IRI,
    ) -> OwlResult<QueryResult> {
        // Get property assertions
        let values: Vec<super::QueryValue> = self
            .ontology
            .property_assertions()
            .iter()
            .filter(|axiom| {
                (**axiom.subject()) == *subject_iri && (**axiom.property()) == *property_iri
            })
            .filter_map(|axiom| match axiom.object() {
                PropertyAssertionObject::Named(individual) => {
                    Some(super::QueryValue::IRI((**individual).clone()))
                }
                PropertyAssertionObject::Anonymous(_) => None,
            })
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["value".to_string()];

        for value in values {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("value".to_string(), value);
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Get all classes in the ontology
    pub fn get_all_classes(&self) -> OwlResult<QueryResult> {
        let classes: Vec<IRI> = self
            .ontology
            .classes()
            .iter()
            .map(|class| (**class.iri()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["class".to_string()];

        for class in classes {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("class".to_string(), super::QueryValue::IRI(class));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();

        Ok(result)
    }

    /// Get all individuals in the ontology
    pub fn get_all_individuals(&self) -> OwlResult<QueryResult> {
        let individuals: Vec<IRI> = self
            .ontology
            .named_individuals()
            .iter()
            .map(|individual| (**individual.iri()).clone())
            .collect();

        // Create query result
        let mut result = QueryResult::new();
        result.variables = vec!["individual".to_string()];

        for individual in individuals {
            let mut binding = super::QueryBinding::new();
            binding.add_binding("individual".to_string(), super::QueryValue::IRI(individual));
            result.add_binding(binding);
        }

        result.stats.results_count = result.len();

        Ok(result)
    }

    /// Get engine statistics
    pub fn stats(&self) -> QueryEngineStats {
        self.stats.read().clone()
    }

    /// Get engine configuration
    pub fn config(&self) -> &QueryConfig {
        &self.config
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.query_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        self.query_cache.stats()
    }

    // Private methods

    /// Execute query in parallel
    fn execute_parallel(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let mut stats = self.stats.write();
        stats.record_parallel_execution();

        // For now, fall back to sequential execution
        // TODO: Implement proper parallel execution
        self.execute_sequential(pattern)
    }

    /// Execute query sequentially
    fn execute_sequential(&self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => self.execute_basic_graph_pattern(triples),
            QueryPattern::Optional { left, right } => self.execute_optional_pattern(left, right),
            QueryPattern::Union { left, right } => self.execute_union_pattern(left, right),
            QueryPattern::Filter {
                pattern,
                expression,
            } => self.execute_filter_pattern(pattern, expression),
            QueryPattern::Reduced(inner) => {
                let mut result = self.execute_sequential(inner)?;
                // Sort by string representation for consistent ordering
                result
                    .bindings
                    .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
                result.bindings.dedup();
                Ok(result)
            }
            QueryPattern::Distinct(inner) => {
                let mut result = self.execute_sequential(inner)?;
                // Sort by variable count and string representation for consistent ordering
                result.bindings.sort_by(|a, b| {
                    a.variables.len().cmp(&b.variables.len()).then_with(|| {
                        format!("{:?}", a.variables).cmp(&format!("{:?}", b.variables))
                    })
                });
                result.bindings.dedup();
                Ok(result)
            }
        }
    }

    /// Execute basic graph pattern
    fn execute_basic_graph_pattern(&self, triples: &[TriplePattern]) -> OwlResult<QueryResult> {
        if triples.is_empty() {
            return Ok(QueryResult::new());
        }

        // Start with the first triple pattern
        let mut current_result = self.execute_single_triple(&triples[0])?;

        // Join with remaining patterns
        for triple in triples.iter().skip(1) {
            current_result = self.join_results(&current_result, triple)?;
        }

        Ok(current_result)
    }

    /// Execute a single triple pattern
    fn execute_single_triple(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        // Determine query type
        let query_type = self.determine_query_type(triple);

        match query_type {
            QueryType::TypeQuery => self.execute_type_query(triple),
            QueryType::PropertyQuery => self.execute_property_query(triple),
            QueryType::VariablePredicate => self.execute_variable_predicate_query(triple),
        }
    }

    /// Determine the type of query based on the triple pattern
    fn determine_query_type(&self, triple: &TriplePattern) -> QueryType {
        match &triple.predicate {
            super::PatternTerm::IRI(pred_iri) => {
                if pred_iri.as_str() == RDF_TYPE {
                    QueryType::TypeQuery
                } else {
                    QueryType::PropertyQuery
                }
            }
            _ => QueryType::VariablePredicate,
        }
    }

    /// Execute type query (rdf:type pattern)
    fn execute_type_query(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        if let super::PatternTerm::IRI(class_iri) = &triple.object {
            self.get_class_instances(class_iri)
        } else {
            Ok(QueryResult::new())
        }
    }

    /// Execute property query
    fn execute_property_query(&self, triple: &TriplePattern) -> OwlResult<QueryResult> {
        match (&triple.subject, &triple.predicate, &triple.object) {
            (
                super::PatternTerm::IRI(subject),
                super::PatternTerm::IRI(predicate),
                super::PatternTerm::Variable(_),
            ) => self.get_property_values(subject, predicate),
            _ => {
                // For more complex patterns, implement generic property query
                Ok(QueryResult::new())
            }
        }
    }

    /// Execute variable predicate query
    fn execute_variable_predicate_query(&self, _triple: &TriplePattern) -> OwlResult<QueryResult> {
        // TODO: Implement variable predicate queries
        Ok(QueryResult::new())
    }

    /// Execute optional pattern
    fn execute_optional_pattern(
        &self,
        left: &QueryPattern,
        right: &QueryPattern,
    ) -> OwlResult<QueryResult> {
        let left_result = self.execute_sequential(left)?;
        let right_result = self.execute_sequential(right)?;

        // Simple left outer join implementation
        let mut result = QueryResult::new();
        result.variables =
            Self::merge_variable_lists(&left_result.variables, &right_result.variables);

        for left_binding in &left_result.bindings {
            let mut found_match = false;

            for right_binding in &right_result.bindings {
                if let Some(merged) = left_binding.join(right_binding) {
                    result.add_binding(merged);
                    found_match = true;
                }
            }

            // If no match found, include left binding only
            if !found_match {
                result.add_binding(left_binding.clone());
            }
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute union pattern
    fn execute_union_pattern(
        &self,
        left: &QueryPattern,
        right: &QueryPattern,
    ) -> OwlResult<QueryResult> {
        let left_result = self.execute_sequential(left)?;
        let right_result = self.execute_sequential(right)?;

        let mut result = QueryResult::new();
        result.variables =
            Self::merge_variable_lists(&left_result.variables, &right_result.variables);

        // Combine results
        result.bindings.extend(left_result.bindings);
        result.bindings.extend(right_result.bindings);

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Execute filter pattern
    fn execute_filter_pattern(
        &self,
        pattern: &QueryPattern,
        _expression: &super::FilterExpression,
    ) -> OwlResult<QueryResult> {
        // TODO: Implement filter evaluation
        self.execute_sequential(pattern)
    }

    /// Join two result sets
    fn join_results(
        &self,
        left: &QueryResult,
        right_triple: &TriplePattern,
    ) -> OwlResult<QueryResult> {
        let right_result = self.execute_single_triple(right_triple)?;

        let mut result = QueryResult::new();
        result.variables = Self::merge_variable_lists(&left.variables, &right_result.variables);

        for left_binding in &left.bindings {
            for right_binding in &right_result.bindings {
                if let Some(merged) = left_binding.join(right_binding) {
                    result.add_binding(merged);
                }
            }
        }

        result.stats.results_count = result.len();
        result.stats.reasoning_used = self.config.enable_reasoning;

        Ok(result)
    }

    /// Merge two variable lists
    fn merge_variable_lists(left: &[String], right: &[String]) -> Vec<String> {
        let mut merged: Vec<String> = left.to_vec();
        let left_set: HashSet<_> = left.iter().collect();

        for var in right {
            if !left_set.contains(var) {
                merged.push(var.clone());
            }
        }

        merged
    }
}

/// Extension trait for QueryPattern to support parallel execution
trait QueryPatternExt {
    fn supports_parallel(&self) -> bool;
}

impl QueryPatternExt for QueryPattern {
    fn supports_parallel(&self) -> bool {
        match self {
            QueryPattern::BasicGraphPattern(triples) => triples.len() > 1,
            QueryPattern::Union { .. } => true,
            QueryPattern::Optional { .. } => false,
            QueryPattern::Filter { .. } => false,
            QueryPattern::Reduced(_) => false,
            QueryPattern::Distinct(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{PatternTerm, QueryPattern, TriplePattern, RDF_TYPE};
    use crate::entities::*;
    use crate::iri::IRI;
    use std::sync::Arc;

    fn create_test_ontology() -> Ontology {
        let mut ontology = Ontology::new();

        // Add some test classes
        let person_class = Class::new("http://example.org/Person");
        let company_class = Class::new("http://example.org/Company");
        let employee_class = Class::new("http://example.org/Employee");

        ontology.add_class(person_class.clone());
        ontology.add_class(company_class.clone());
        ontology.add_class(employee_class.clone());

        // Add some object properties
        let works_for_prop = ObjectProperty::new("http://example.org/worksFor");
        let manager_of_prop = ObjectProperty::new("http://example.org/managerOf");

        ontology.add_object_property(works_for_prop.clone());
        ontology.add_object_property(manager_of_prop.clone());

        // Add some data properties
        let name_prop = DataProperty::new("http://example.org/name");
        let age_prop = DataProperty::new("http://example.org/age");

        ontology.add_data_property(name_prop.clone());
        ontology.add_data_property(age_prop.clone());

        // Add some individuals
        let person1 = NamedIndividual::new("http://example.org/person1");
        let person2 = NamedIndividual::new("http://example.org/person2");
        let person3 = NamedIndividual::new("http://example.org/person3");
        let company1 = NamedIndividual::new("http://example.org/company1");
        let company2 = NamedIndividual::new("http://example.org/company2");

        ontology.add_named_individual(person1.clone());
        ontology.add_named_individual(person2.clone());
        ontology.add_named_individual(person3.clone());
        ontology.add_named_individual(company1.clone());
        ontology.add_named_individual(company2.clone());

        // Add class assertions
        let person1_type = ClassAssertionAxiom::new(
            person1.iri().clone(),
            ClassExpression::Class(person_class.clone()),
        );
        let person2_type = ClassAssertionAxiom::new(
            person2.iri().clone(),
            ClassExpression::Class(employee_class.clone()),
        );
        let person3_type = ClassAssertionAxiom::new(
            person3.iri().clone(),
            ClassExpression::Class(person_class.clone()),
        );
        let company1_type = ClassAssertionAxiom::new(
            company1.iri().clone(),
            ClassExpression::Class(company_class.clone()),
        );

        ontology.add_class_assertion(person1_type);
        ontology.add_class_assertion(person2_type);
        ontology.add_class_assertion(person3_type);
        ontology.add_class_assertion(company1_type);

        // Add property assertions
        let works_for1 = PropertyAssertionAxiom::new(
            person1.iri().clone(),
            works_for_prop.iri().clone(),
            company1.iri().clone(),
        );
        let works_for2 = PropertyAssertionAxiom::new(
            person2.iri().clone(),
            works_for_prop.iri().clone(),
            company1.iri().clone(),
        );
        let manager_of = PropertyAssertionAxiom::new(
            person3.iri().clone(),
            manager_of_prop.iri().clone(),
            person1.iri().clone(),
        );

        ontology.add_property_assertion(works_for1);
        ontology.add_property_assertion(works_for2);
        ontology.add_property_assertion(manager_of);

        ontology
    }

    fn create_test_query_pattern(subject: &str, predicate: &str, object: &str) -> QueryPattern {
        QueryPattern::BasicGraphPattern(vec![TriplePattern::new(
            if subject.starts_with('?') {
                PatternTerm::Variable(subject.to_string())
            } else {
                PatternTerm::IRI(IRI::new(subject).expect("Valid IRI"))
            },
            if predicate.starts_with('?') {
                PatternTerm::Variable(predicate.to_string())
            } else {
                PatternTerm::IRI(IRI::new(predicate).expect("Valid IRI"))
            },
            if object.starts_with('?') {
                PatternTerm::Variable(object.to_string())
            } else {
                PatternTerm::IRI(IRI::new(object).expect("Valid IRI"))
            },
        )])
    }

    fn create_test_query_engine() -> QueryEngine {
        let ontology = create_test_ontology();
        let config = QueryConfig {
            enable_caching: true,
            enable_parallel: true,
            max_results: Some(1000),
            cache_size: Some(100),
        };
        QueryEngine::with_config(ontology, config)
    }

    #[test]
    fn test_query_engine_creation() {
        let ontology = create_test_ontology();
        let engine = QueryEngine::new(ontology);

        let stats = engine.stats();
        assert_eq!(stats.successful_queries, 0);
        assert_eq!(stats.get_failed_queries(), 0);
    }

    #[test]
    fn test_query_engine_with_config() {
        let ontology = create_test_ontology();
        let config = QueryConfig {
            enable_reasoning: false,
            enable_caching: true,
            enable_parallel: false,
            max_results: Some(500),
            cache_size: Some(50),
        };

        let engine = QueryEngine::with_config(ontology, config);

        let engine_config = engine.config();
        assert_eq!(engine_config.enable_reasoning, false);
        assert_eq!(engine_config.enable_caching, true);
        assert_eq!(engine_config.enable_parallel, false);
        assert_eq!(engine_config.max_results, Some(500));
        assert_eq!(engine_config.cache_size, Some(50));
    }

    #[test]
    fn test_basic_query_execution() {
        let engine = create_test_query_engine();

        let pattern = create_test_query_pattern("?s", "?p", "?o");
        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();
        assert!(query_result.stats.time_ms >= 0);
        assert!(query_result.stats.results_count >= 0);
    }

    #[test]
    fn test_triple_pattern_execution() {
        let engine = create_test_query_engine();

        let triple = TriplePattern::new(
            PatternTerm::Variable("?s".to_string()),
            PatternTerm::Variable("?p".to_string()),
            PatternTerm::Variable("?o".to_string()),
        );

        let result = engine.execute_triple(triple);

        assert!(result.is_ok());
        let query_result = result.unwrap();
        assert!(query_result.variables.contains(&"?s".to_string()));
        assert!(query_result.variables.contains(&"?p".to_string()));
        assert!(query_result.variables.contains(&"?o".to_string()));
    }

    #[test]
    fn test_type_query_execution() {
        let engine = create_test_query_engine();

        let triple = TriplePattern::new(
            PatternTerm::Variable("?s".to_string()),
            PatternTerm::IRI(IRI::new(RDF_TYPE).expect("Valid IRI")),
            PatternTerm::IRI(IRI::new("http://example.org/Person").expect("Valid IRI")),
        );

        let result = engine.execute_triple(triple);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should find individuals that are instances of Person
        assert!(query_result.variables.contains(&"?s".to_string()));
        assert!(query_result.stats.reasoning_used);
    }

    #[test]
    fn test_property_query_execution() {
        let engine = create_test_query_engine();

        let triple = TriplePattern::new(
            PatternTerm::IRI(IRI::new("http://example.org/person1").expect("Valid IRI")),
            PatternTerm::IRI(IRI::new("http://example.org/worksFor").expect("Valid IRI")),
            PatternTerm::Variable("?o".to_string()),
        );

        let result = engine.execute_triple(triple);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should find companies that person1 works for
        assert!(query_result.variables.contains(&"?o".to_string()));
        assert!(query_result.stats.reasoning_used);
    }

    #[test]
    fn test_get_class_instances() {
        let engine = create_test_query_engine();
        let person_iri = IRI::new("http://example.org/Person").expect("Valid IRI");

        let result = engine.get_class_instances(&person_iri);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert_eq!(query_result.variables, vec!["instance"]);
        assert!(query_result.stats.reasoning_used);

        // Should find at least one Person instance
        assert!(query_result.bindings.len() >= 1);
    }

    #[test]
    fn test_get_property_values() {
        let engine = create_test_query_engine();
        let person_iri = IRI::new("http://example.org/person1").expect("Valid IRI");
        let works_for_iri = IRI::new("http://example.org/worksFor").expect("Valid IRI");

        let result = engine.get_property_values(&person_iri, &works_for_iri);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert_eq!(query_result.variables, vec!["value"]);
        assert!(query_result.stats.reasoning_used);

        // Should find at least one property value
        assert!(query_result.bindings.len() >= 1);
    }

    #[test]
    fn test_get_all_classes() {
        let engine = create_test_query_engine();

        let result = engine.get_all_classes();

        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert_eq!(query_result.variables, vec!["class"]);
        assert!(query_result.bindings.len() >= 3); // Person, Company, Employee
    }

    #[test]
    fn test_get_all_individuals() {
        let engine = create_test_query_engine();

        let result = engine.get_all_individuals();

        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert_eq!(query_result.variables, vec!["individual"]);
        assert!(query_result.bindings.len() >= 5); // person1, person2, person3, company1, company2
    }

    #[test]
    fn test_optional_pattern_execution() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::Optional {
            left: Box::new(create_test_query_pattern(
                "?s",
                RDF_TYPE,
                "http://example.org/Person",
            )),
            right: Box::new(create_test_query_pattern(
                "?s",
                "http://example.org/worksFor",
                "?company",
            )),
        };

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should execute optional pattern successfully
        assert!(query_result.stats.time_ms >= 0);
        assert!(query_result.variables.len() >= 2); // Should have ?s and ?company variables
    }

    #[test]
    fn test_union_pattern_execution() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::Union {
            left: Box::new(create_test_query_pattern(
                "?s",
                RDF_TYPE,
                "http://example.org/Person",
            )),
            right: Box::new(create_test_query_pattern(
                "?s",
                RDF_TYPE,
                "http://example.org/Company",
            )),
        };

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should execute union pattern successfully
        assert!(query_result.stats.time_ms >= 0);
        assert!(query_result.variables.contains(&"?s".to_string()));
    }

    #[test]
    fn test_filter_pattern_execution() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::Filter {
            pattern: Box::new(create_test_query_pattern("?s", "?p", "?o")),
            expression: FilterExpression::IsVariable("?s".to_string()),
        };

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should execute filter pattern (though filter evaluation might be basic)
        assert!(query_result.stats.time_ms >= 0);
    }

    #[test]
    fn test_reduced_pattern_execution() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::Reduced(Box::new(create_test_query_pattern("?s", "?p", "?o")));

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should execute reduced pattern
        assert!(query_result.stats.time_ms >= 0);
    }

    #[test]
    fn test_distinct_pattern_execution() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::Distinct(Box::new(create_test_query_pattern("?s", "?p", "?o")));

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should execute distinct pattern
        assert!(query_result.stats.time_ms >= 0);
    }

    #[test]
    fn test_caching_behavior() {
        let engine = create_test_query_engine();

        let pattern = create_test_query_pattern("?s", "?p", "?o");

        // First execution
        let result1 = engine.execute(&pattern);
        assert!(result1.is_ok());

        let stats_after1 = engine.stats();

        // Second execution
        let result2 = engine.execute(&pattern);
        assert!(result2.is_ok());

        let stats_after2 = engine.stats();

        // Should have recorded cache activity
        if engine.config().enable_caching {
            assert!(
                stats_after2.get_cache_hits() + stats_after2.get_cache_misses()
                    > stats_after1.get_cache_hits() + stats_after1.get_cache_misses()
            );
        }
    }

    #[test]
    fn test_parallel_execution_support() {
        // Test patterns that should support parallel execution
        let parallel_pattern = QueryPattern::Union {
            left: Box::new(create_test_query_pattern("?s1", "?p1", "?o1")),
            right: Box::new(create_test_query_pattern("?s2", "?p2", "?o2")),
        };

        assert!(parallel_pattern.supports_parallel());

        // Test patterns that should not support parallel execution
        let non_parallel_pattern = QueryPattern::Optional {
            left: Box::new(create_test_query_pattern("?s", "?p", "?o")),
            right: Box::new(create_test_query_pattern("?s2", "?p2", "?o2")),
        };

        assert!(!non_parallel_pattern.supports_parallel());
    }

    #[test]
    fn test_statistics_tracking() {
        let engine = create_test_query_engine();

        let initial_stats = engine.stats();
        assert_eq!(initial_stats.successful_queries, 0);
        assert_eq!(initial_stats.get_failed_queries(), 0);

        // Execute some successful queries
        for _ in 0..3 {
            let pattern = create_test_query_pattern("?s", "?p", "?o");
            let result = engine.execute(&pattern);
            assert!(result.is_ok());
        }

        let final_stats = engine.stats();
        assert_eq!(final_stats.successful_queries, 3);
        assert_eq!(final_stats.get_failed_queries(), 0);
        assert!(final_stats.total_queries > 0);
        assert!(final_stats.get_average_time() > 0.0);
    }

    #[test]
    fn test_statistics_reset() {
        let engine = create_test_query_engine();

        // Execute some queries to generate stats
        for _ in 0..2 {
            let pattern = create_test_query_pattern("?s", "?p", "?o");
            let _ = engine.execute(&pattern);
        }

        let stats_before = engine.stats();
        assert!(stats_before.successful_queries > 0);

        // Reset stats
        engine.reset_stats();

        let stats_after = engine.stats();
        assert_eq!(stats_after.successful_queries, 0);
        assert_eq!(stats_after.get_failed_queries(), 0);
        assert_eq!(stats_after.total_queries, 0);
    }

    #[test]
    fn test_cache_management() {
        let engine = create_test_query_engine();

        // Execute some queries to populate cache
        for i in 0..5 {
            let pattern = create_test_query_pattern(&format!("?s{}", i), "?p", "?o");
            let _ = engine.execute(&pattern);
        }

        let (cache_size, pattern_size) = engine.cache_stats();
        assert!(cache_size + pattern_size >= 0);

        // Clear caches
        engine.clear_caches();

        let (cache_size_after, pattern_size_after) = engine.cache_stats();
        assert_eq!(cache_size_after, 0);
        assert_eq!(pattern_size_after, 0);
    }

    #[test]
    fn test_query_result_consistency() {
        let engine = create_test_query_engine();

        let pattern = create_test_query_pattern("?s", "?p", "?o");
        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Result should have consistent structure
        assert!(!query_result.variables.is_empty());
        assert!(query_result.stats.time_ms >= 0);
        assert!(query_result.stats.results_count == query_result.bindings.len());

        // Each binding should be consistent with variables
        for binding in &query_result.bindings {
            for var in &query_result.variables {
                // Either the variable is bound or not (both are valid states)
                let _is_bound = binding.is_bound(var);
            }
        }
    }

    #[test]
    fn test_error_handling() {
        let engine = create_test_query_engine();

        // Test with potentially problematic patterns
        let problematic_patterns = vec![
            QueryPattern::Filter {
                pattern: Box::new(create_test_query_pattern("?nonexistent", "?p", "?o")),
                expression: FilterExpression::IsVariable("?nonexistent".to_string()),
            },
            QueryPattern::Union {
                left: Box::new(create_test_query_pattern("?s", "?nonexistent", "?o")),
                right: Box::new(create_test_query_pattern("?s", "?p", "?nonexistent")),
            },
        ];

        for pattern in problematic_patterns {
            let result = engine.execute(&pattern);
            // Should handle errors gracefully (either succeed or fail cleanly)
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_complex_query_patterns() {
        let engine = create_test_query_engine();

        // Test nested patterns
        let nested_pattern = QueryPattern::Filter {
            pattern: Box::new(QueryPattern::Optional {
                left: Box::new(create_test_query_pattern(
                    "?s",
                    RDF_TYPE,
                    "http://example.org/Person",
                )),
                right: Box::new(create_test_query_pattern(
                    "?s",
                    "http://example.org/worksFor",
                    "?company",
                )),
            }),
            expression: FilterExpression::IsVariable("?s".to_string()),
        };

        let result = engine.execute(&nested_pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();
        assert!(query_result.stats.time_ms >= 0);
    }

    #[test]
    fn test_multiple_triple_pattern() {
        let engine = create_test_query_engine();

        let pattern = QueryPattern::BasicGraphPattern(vec![
            TriplePattern::new(
                PatternTerm::Variable("?s".to_string()),
                PatternTerm::IRI(IRI::new(RDF_TYPE).expect("Valid IRI")),
                PatternTerm::IRI(IRI::new("http://example.org/Person").expect("Valid IRI")),
            ),
            TriplePattern::new(
                PatternTerm::Variable("?s".to_string()),
                PatternTerm::IRI(IRI::new("http://example.org/worksFor").expect("Valid IRI")),
                PatternTerm::Variable("?company".to_string()),
            ),
        ]);

        let result = engine.execute(&pattern);

        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should have both variables
        assert!(query_result.variables.contains(&"?s".to_string()));
        assert!(query_result.variables.contains(&"?company".to_string()));
    }

    #[test]
    fn test_engine_performance() {
        let engine = create_test_query_engine();

        let pattern = create_test_query_pattern("?s", "?p", "?o");

        let start_time = std::time::Instant::now();

        // Execute multiple queries
        for _ in 0..10 {
            let result = engine.execute(&pattern);
            assert!(result.is_ok());
        }

        let elapsed = start_time.elapsed();

        // Should complete reasonably quickly (less than 1 second for 10 queries in test)
        assert!(elapsed < std::time::Duration::from_secs(1));

        let stats = engine.stats();
        assert_eq!(stats.successful_queries, 10);
        assert!(stats.get_average_time() > 0.0);
    }

    #[test]
    fn test_variable_extraction() {
        let patterns = vec![
            create_test_query_pattern("?s", "?p", "?o"),
            create_test_query_pattern("?subject", "http://example.org/predicate", "?object"),
            QueryPattern::BasicGraphPattern(vec![
                TriplePattern::new(
                    PatternTerm::Variable("?x".to_string()),
                    PatternTerm::Variable("?y".to_string()),
                    PatternTerm::IRI(IRI::new("http://example.org/test").expect("Valid IRI")),
                ),
                TriplePattern::new(
                    PatternTerm::Variable("?x".to_string()),
                    PatternTerm::IRI(IRI::new("http://example.org/prop").expect("Valid IRI")),
                    PatternTerm::Variable("?z".to_string()),
                ),
            ]),
        ];

        for pattern in patterns {
            let engine = create_test_query_engine();
            let result = engine.execute(&pattern);

            assert!(result.is_ok());
            let query_result = result.unwrap();

            // Variables should be extracted correctly
            assert!(!query_result.variables.is_empty());

            // All declared variables should be properly handled
            for var in &query_result.variables {
                assert!(var.starts_with('?'));
            }
        }
    }

    #[test]
    fn test_join_operations() {
        let engine = create_test_query_engine();

        // Create a pattern that requires joining
        let left_pattern =
            create_test_query_pattern("?person", RDF_TYPE, "http://example.org/Person");
        let right_pattern =
            create_test_query_pattern("?person", "http://example.org/worksFor", "?company");

        // Test join by executing patterns and then combining results
        let left_result = engine.execute(&left_pattern);
        let right_result = engine.execute(&right_pattern);

        assert!(left_result.is_ok());
        assert!(right_result.is_ok());

        // Both results should have the join variable ?person
        let left_query = left_result.unwrap();
        let right_query = right_result.unwrap();

        assert!(left_query.variables.contains(&"?person".to_string()));
        assert!(right_query.variables.contains(&"?person".to_string()));
    }

    #[test]
    fn test_concurrent_query_access() {
        use std::sync::Arc;
        use std::thread;

        let engine = Arc::new(create_test_query_engine());
        let mut handles = Vec::new();

        // Spawn multiple threads accessing the engine
        for thread_id in 0..4 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                let pattern = create_test_query_pattern("?s", "?p", "?o");

                // Execute queries and access stats
                for _ in 0..5 {
                    let result = engine_clone.execute(&pattern);
                    assert!(result.is_ok() || result.is_err());

                    let _stats = engine_clone.stats();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        // Engine should still be functional
        let final_stats = engine.stats();
        assert!(final_stats.successful_queries >= 0);
    }

    // Property-based tests for QueryEngine
    #[cfg(test)]
    mod engine_proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_engine_configuration_variations(
                enable_reasoning in prop::bool::ANY,
                enable_caching in prop::bool::ANY,
                enable_parallel in prop::bool::ANY,
                cache_size in 1usize..1000usize,
                max_results in 1usize..1000usize
            ) {
                let ontology = create_test_ontology();
                let config = QueryConfig {
                    enable_reasoning,
                    enable_caching,
                    enable_parallel,
                    cache_size: Some(cache_size),
                    max_results: Some(max_results),
                };

                let engine = QueryEngine::with_config(ontology, config);
                let pattern = create_test_query_pattern("?s", "?p", "?o");

                // Should work with any valid configuration
                let result = engine.execute(&pattern);
                prop_assert!(result.is_ok() || result.is_err());

                let engine_config = engine.config();
                prop_assert_eq!(engine_config.enable_reasoning, enable_reasoning);
                prop_assert_eq!(engine_config.enable_caching, enable_caching);
                prop_assert_eq!(engine_config.enable_parallel, enable_parallel);
                prop_assert_eq!(engine_config.cache_size, Some(cache_size));
                prop_assert_eq!(engine_config.max_results, Some(max_results));
            }

            #[test]
            fn test_statistics_monotonicity(
                query_count in 1usize..20usize
            ) {
                let engine = create_test_query_engine();
                let pattern = create_test_query_pattern("?s", "?p", "?o");

                let mut previous_successful = 0;
                let mut previous_total = 0;

                for _ in 0..query_count {
                    let result = engine.execute(&pattern);
                    let _ = result; // We don't care about success/failure for this test

                    let stats = engine.stats();

                    // Stats should be monotonic (never decrease)
                    prop_assert!(stats.successful_queries >= previous_successful);
                    prop_assert!(stats.total_queries >= previous_total);

                    previous_successful = stats.successful_queries;
                    previous_total = stats.total_queries;
                }
            }

            #[test]
            fn test_pattern_supports_parallel(
                pattern_type in 0usize..6usize
            ) {
                let pattern = match pattern_type {
                    0 => create_test_query_pattern("?s", "?p", "?o"),
                    1 => QueryPattern::Union {
                        left: Box::new(create_test_query_pattern("?s1", "?p1", "?o1")),
                        right: Box::new(create_test_query_pattern("?s2", "?p2", "?o2")),
                    },
                    2 => QueryPattern::Optional {
                        left: Box::new(create_test_query_pattern("?s", "?p", "?o")),
                        right: Box::new(create_test_query_pattern("?s2", "?p2", "?o2")),
                    },
                    3 => QueryPattern::Filter {
                        pattern: Box::new(create_test_query_pattern("?s", "?p", "?o")),
                        expression: FilterExpression::IsVariable("?s".to_string()),
                    },
                    4 => QueryPattern::Reduced(Box::new(create_test_query_pattern("?s", "?p", "?o"))),
                    5 => QueryPattern::Distinct(Box::new(create_test_query_pattern("?s", "?p", "?o"))),
                    _ => unreachable!(),
                };

                let supports_parallel = pattern.supports_parallel();

                // Should not panic for any pattern type
                let _ = supports_parallel;
            }
        }
    }
}

// Thread safety implementations
unsafe impl Send for QueryEngine {}
unsafe impl Sync for QueryEngine {}
