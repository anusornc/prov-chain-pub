//! Query execution utilities and helpers
//!
//! Provides low-level query execution utilities and thread-safe contexts.

use crate::axioms::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;

use super::{
    PatternTerm, QueryBinding, QueryType, QueryValue,
    TriplePattern, RDF_TYPE,
};

use dashmap::DashMap;
use rayon::prelude::*;
use std::sync::Arc;

/// Thread-safe query context for parallel execution
pub struct ThreadQueryContext<'a> {
    pub ontology: &'a Arc<Ontology>,
    pub type_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<ClassAssertionAxiom>>>>,
    pub property_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<PropertyAssertionAxiom>>>>,
}

impl<'a> ThreadQueryContext<'a> {
    /// Create a new thread query context
    pub fn new(
        ontology: &'a Arc<Ontology>,
        type_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<ClassAssertionAxiom>>>>,
        property_index: &'a Arc<DashMap<Arc<IRI>, Vec<Arc<PropertyAssertionAxiom>>>>,
    ) -> Self {
        Self {
            ontology,
            type_index,
            property_index,
        }
    }

    /// Find all instances of a specific type
    pub fn find_instances_of_type(&self, type_iri: &IRI) -> Vec<IRI> {

        self.ontology
            .class_assertions()
            .par_iter()
            .filter(|axiom| axiom.class_expr().contains_class(type_iri))
            .filter_map(|axiom| Some((**axiom.individual()).clone()))
            .collect()
    }

    /// Find all property values for a subject and property
    pub fn find_property_values(&self, subject_iri: &IRI, property_iri: &IRI) -> Vec<QueryValue> {

        self.ontology
            .property_assertions()
            .par_iter()
            .filter(|axiom| {
                (**axiom.subject()) == *subject_iri && (**axiom.property()) == *property_iri
            })
            .filter_map(|axiom| match axiom.object() {
                PropertyAssertionObject::Named(individual) => {
                    Some(QueryValue::IRI((**individual).clone()))
                }
                PropertyAssertionObject::Anonymous(_) => None,
            })
            .collect()
    }

    /// Execute a basic triple pattern matching
    pub fn execute_triple_pattern(&self, pattern: &TriplePattern) -> OwlResult<Vec<QueryBinding>> {
        let query_type = self.determine_query_type(pattern);

        match query_type {
            QueryType::TypeQuery => self.execute_type_pattern(pattern),
            QueryType::PropertyQuery => self.execute_property_pattern(pattern),
            QueryType::VariablePredicate => self.execute_variable_predicate_pattern(pattern),
        }
    }

    /// Determine query type for a triple pattern
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

    /// Execute type pattern (rdf:type)
    fn execute_type_pattern(&self, triple: &TriplePattern) -> OwlResult<Vec<QueryBinding>> {
        if let PatternTerm::IRI(class_iri) = &triple.object {
            let instances = self.find_instances_of_type(class_iri);

            let mut bindings = Vec::new();

            for instance in instances {
                let mut binding = QueryBinding::new();

                // Bind subject variable
                match &triple.subject {
                    PatternTerm::Variable(var_name) => {
                        binding.add_binding(var_name.clone(), QueryValue::IRI(instance));
                    }
                    PatternTerm::IRI(subject_iri) => {
                        if subject_iri.as_str() != instance.as_str() {
                            continue; // Skip if IRIs don't match
                        }
                    }
                    _ => {}
                }

                // Bind object variable (though it's usually a constant in type queries)
                if let PatternTerm::Variable(var_name) = &triple.object {
                    binding.add_binding(var_name.clone(), QueryValue::IRI(class_iri.clone()));
                }

                bindings.push(binding);
            }

            Ok(bindings)
        } else {
            Ok(Vec::new())
        }
    }

    /// Execute property pattern
    fn execute_property_pattern(&self, triple: &TriplePattern) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        match (&triple.subject, &triple.predicate, &triple.object) {
            (
                PatternTerm::IRI(subject_iri),
                PatternTerm::IRI(property_iri),
                PatternTerm::Variable(object_var),
            ) => {
                let values = self.find_property_values(subject_iri, property_iri);

                for value in values {
                    let mut binding = QueryBinding::new();
                    binding.add_binding(object_var.clone(), value);
                    bindings.push(binding);
                }
            }

            (
                PatternTerm::Variable(_subject_var),
                PatternTerm::IRI(_property_iri),
                PatternTerm::IRI(_object_iri),
            ) => {
                // TODO: Implement complex property patterns
                // For now, return empty bindings
            }

            (
                PatternTerm::IRI(_subject_iri),
                PatternTerm::Variable(_property_var),
                PatternTerm::IRI(_object_iri),
            ) => {
                // TODO: Implement complex property patterns
                // For now, return empty bindings
            }

            _ => {
                // For more complex patterns, implement general matching
                // TODO: Implement full pattern matching
            }
        }

        Ok(bindings)
    }

    /// Execute variable predicate pattern
    fn execute_variable_predicate_pattern(
        &self,
        _triple: &TriplePattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        // TODO: Implement variable predicate queries
        Ok(Vec::new())
    }

    /// Join two sets of bindings
    pub fn join_bindings(
        &self,
        left: &[QueryBinding],
        right: &[QueryBinding],
    ) -> Vec<QueryBinding> {
        left.par_iter()
            .flat_map(|left_binding| {
                right
                    .iter()
                    .filter_map(|right_binding| left_binding.join(right_binding))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Apply filter expression to bindings
    pub fn apply_filter(
        &self,
        bindings: Vec<QueryBinding>,
        _filter_expr: &super::FilterExpression,
    ) -> Vec<QueryBinding> {
        // TODO: Implement filter evaluation
        bindings
    }
}

/// Utility functions for query execution
pub mod utils {
    use super::*;

    /// Convert PatternTerm to QueryValue
    pub fn pattern_term_to_value(term: &PatternTerm) -> Option<QueryValue> {
        match term {
            PatternTerm::IRI(iri) => Some(QueryValue::IRI(iri.clone())),
            PatternTerm::Literal(lit) => Some(QueryValue::Literal(lit.clone())),
            PatternTerm::BlankNode(bn) => Some(QueryValue::BlankNode(bn.clone())),
            PatternTerm::Variable(_) => None,
        }
    }

    /// Create variable binding from triple pattern
    pub fn create_binding_from_triple(
        triple: &TriplePattern,
        subject: &IRI,
        predicate: &IRI,
        object: &QueryValue,
    ) -> QueryBinding {
        let mut binding = QueryBinding::new();

        // Bind subject
        if let PatternTerm::Variable(var_name) = &triple.subject {
            binding.add_binding(var_name.clone(), QueryValue::IRI(subject.clone()));
        }

        // Bind predicate
        if let PatternTerm::Variable(var_name) = &triple.predicate {
            binding.add_binding(var_name.clone(), QueryValue::IRI(predicate.clone()));
        }

        // Bind object
        if let PatternTerm::Variable(var_name) = &triple.object {
            binding.add_binding(var_name.clone(), object.clone());
        }

        binding
    }

    /// Check if two patterns can be joined
    pub fn can_join_patterns(left: &TriplePattern, right: &TriplePattern) -> bool {
        // Check if patterns share any variables
        let left_vars: std::collections::HashSet<_> = left.variables().into_iter().collect();
        let right_vars: std::collections::HashSet<_> = right.variables().into_iter().collect();

        !left_vars.is_disjoint(&right_vars)
    }

    /// Find join variables between two patterns
    pub fn find_join_variables(left: &TriplePattern, right: &TriplePattern) -> Vec<String> {
        let left_vars: std::collections::HashSet<_> = left.variables().into_iter().collect();
        let right_vars: std::collections::HashSet<_> = right.variables().into_iter().collect();

        left_vars.intersection(&right_vars).cloned().collect()
    }
}
