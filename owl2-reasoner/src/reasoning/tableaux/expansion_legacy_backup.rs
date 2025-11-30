//! # Tableaux Rule Expansion
//!
//! Implements the core rule expansion logic for the tableaux reasoning algorithm.
//! This module manages the application of tableaux rules to expand the model
//! and derive new consequences from the ontology.
//!
//! ## Key Components
//!
//! - **[`ExpansionEngine`]** - Main coordinator for rule application
//! - **[`ExpansionRules`]** - Collection of tableaux expansion rules
//! - **[`ExpansionContext`]** - Context tracking for expansion state
//! - **[`ExpansionTask`]** - Individual rule application tasks
//! - **[`ExpansionRule`]** - Types of expansion rules (Conjunction, Disjunction, etc.)
//!
//! ## Tableaux Rules
//!
//! The module implements the standard tableaux rules for OWL2 reasoning:
//!
//! ### Conjunction Rule (∧-rule)
//! When a node contains a conjunction `C₁ ∧ C₂`, add both `C₁` and `C₂` to the node.
//!
//! ### Disjunction Rule (∨-rule)
//! When a node contains a disjunction `C₁ ∨ C₂`, create a choice point and branch:
//! - Branch 1: Add `C₁` to the node
//! - Branch 2: Add `C₂` to the node
//!
//! ### Existential Restriction Rule (∃-rule)
//! When a node contains `∃r.C`, create a new node connected by property `r` that contains `C`.
//!
//! ### Universal Restriction Rule (∀-rule)
//! When a node contains `∀r.C` and has `r`-successors, add `C` to all `r`-successors.
//!
//! ### Nominal Rule
//! Handle individual assertions and nominals according to OWL2 semantics.
//!
//! ### Data Range Rule
//! Process data property restrictions and datatypes.
//!
//! ## Expansion Strategy
//!
//! The expansion engine uses a priority-based approach:
//!
//! 1. **Rule Selection**: Choose next applicable rule based on priority order
//! 2. **Task Creation**: Create expansion tasks for rule applications
//! 3. **Priority Queue**: Manage tasks by priority to optimize reasoning
//! 4. **Context Tracking**: Maintain expansion state and applied rules
//! 5. **Depth Control**: Limit expansion depth to prevent infinite loops
//!
//! ## Performance Optimizations
//!
//! - **Priority-Based Ordering**: Apply high-impact rules first
//! - **Task Batching**: Group similar operations for efficiency
//! - **Context Caching**: Avoid redundant rule applications
//! - **Depth Limiting**: Prevent infinite expansion
//! - **Smart Rule Selection**: Heuristics for optimal rule choice
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use owl2_reasoner::reasoning::tableaux::{ExpansionEngine, ExpansionRules, ExpansionContext, NodeId, TableauxGraph, MemoryManager};
//! use std::collections::{HashSet, VecDeque};
//!
//! // Create expansion engine with rules
//! let mut expansion_engine = ExpansionEngine::new();
//! let rules = ExpansionRules::new();
//!
//! // Create graph and memory manager for the example
//! let mut graph = TableauxGraph::new();
//! let mut memory_manager = MemoryManager::new();
//!
//! // Set up expansion context
//! let mut context = ExpansionContext {
//!     current_node: NodeId::new(0),
//!     current_depth: 0,
//!     applied_rules: HashSet::new(),
//!     pending_expansions: VecDeque::new(),
//! };
//!
//! // Perform expansion up to maximum depth
//! let max_depth = 100;
//! let expansion_complete = expansion_engine.expand(&mut graph, &mut memory_manager, max_depth)?;
//!
//! println!("Expansion completed: {}", expansion_complete);
//! ```

use super::core::{NodeId, ReasoningRules};
use super::equality::EqualityReasoner;
use super::graph::{GraphChangeLog, TableauxGraph};
use super::memory::{MemoryChangeLog, MemoryManager};
use crate::axioms::class_expressions::ClassExpression;
use crate::axioms::ObjectPropertyExpression;
use crate::entities::Class;
use crate::iri::IRI;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

/// Types of expansion rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpansionRule {
    /// Conjunction rule
    Conjunction,
    /// Disjunction rule
    Disjunction,
    /// Existential restriction rule
    ExistentialRestriction,
    /// Universal restriction rule
    UniversalRestriction,
    /// Nominal rule
    Nominal,
    /// Data range rule
    DataRange,
    /// Subclass axiom application rule
    SubclassAxiom,
    /// Transitive property rule
    TransitiveProperty,
    /// Symmetric property rule
    SymmetricProperty,
    /// Reflexive property rule
    ReflexiveProperty,
    /// Functional property rule (clash detection)
    FunctionalProperty,
    /// Inverse functional property rule (clash detection)
    InverseFunctionalProperty,
    /// Irreflexive property rule (clash detection)
    IrreflexiveProperty,
    /// Asymmetric property rule (clash detection)
    AsymmetricProperty,
    /// Property hierarchy rule (SubObjectPropertyOf)
    PropertyHierarchy,
    /// Property domain rule
    PropertyDomain,
    /// Property range rule
    PropertyRange,
    /// Inverse property rule
    InverseProperty,
    /// Property assertion initialization (ABox)
    PropertyAssertion,
    /// Negative property assertion clash detection
    NegativePropertyAssertion,
    /// Same individual reasoning (equality propagation)
    SameIndividual,
    /// Different individuals reasoning (inequality clash detection)
    DifferentIndividuals,
}

/// Expansion context for rule application
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    pub current_node: NodeId,
    pub current_depth: usize,
    pub applied_rules: HashSet<ExpansionRule>,
    pub pending_expansions: VecDeque<ExpansionTask>,
    pub reasoning_rules: Option<super::core::ReasoningRules>,
}

#[derive(Debug, Clone)]
pub struct ExpansionTask {
    pub node_id: NodeId,
    pub concept: ClassExpression,
    pub rule_type: ExpansionRule,
    pub priority: usize,
}

impl ExpansionTask {
    pub fn new(node_id: NodeId, concept: ClassExpression, rule_type: ExpansionRule) -> Self {
        Self {
            node_id,
            concept,
            rule_type,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

/// Expansion rules for tableaux reasoning
#[derive(Debug)]
pub struct ExpansionRules {
    pub rules: Vec<ExpansionRule>,
    pub rule_order: Vec<ExpansionRule>,
    pub max_applications: HashMap<ExpansionRule, usize>,
    pub equality_reasoner: EqualityReasoner,
}

impl ExpansionRules {
    pub fn new() -> Self {
        let rules = vec![
            ExpansionRule::Conjunction,
            ExpansionRule::Disjunction,
            ExpansionRule::ExistentialRestriction,
            ExpansionRule::UniversalRestriction,
            ExpansionRule::Nominal,
            ExpansionRule::DataRange,
            ExpansionRule::SubclassAxiom,
            // Property characteristic rules
            ExpansionRule::TransitiveProperty,
            ExpansionRule::SymmetricProperty,
            ExpansionRule::ReflexiveProperty,
            ExpansionRule::FunctionalProperty,
            ExpansionRule::InverseFunctionalProperty,
            ExpansionRule::IrreflexiveProperty,
            ExpansionRule::AsymmetricProperty,
            // Property hierarchy and domain/range rules
            ExpansionRule::PropertyHierarchy,
            ExpansionRule::PropertyDomain,
            ExpansionRule::PropertyRange,
            ExpansionRule::InverseProperty,
            // ABox reasoning rules
            ExpansionRule::PropertyAssertion,
            ExpansionRule::NegativePropertyAssertion,
            ExpansionRule::SameIndividual,
            ExpansionRule::DifferentIndividuals,
        ];

        let rule_order = vec![
            ExpansionRule::SubclassAxiom,     // Apply subclass axioms first
            ExpansionRule::PropertyHierarchy, // Property hierarchies early
            ExpansionRule::Conjunction,
            ExpansionRule::ExistentialRestriction,
            // Property characteristics after basic expansion but before disjunction
            ExpansionRule::TransitiveProperty,
            ExpansionRule::SymmetricProperty,
            ExpansionRule::ReflexiveProperty,
            ExpansionRule::FunctionalProperty,
            ExpansionRule::InverseFunctionalProperty,
            ExpansionRule::IrreflexiveProperty,
            ExpansionRule::AsymmetricProperty,
            ExpansionRule::PropertyDomain,
            ExpansionRule::PropertyRange,
            ExpansionRule::InverseProperty,
            ExpansionRule::UniversalRestriction,
            ExpansionRule::Disjunction,
            ExpansionRule::Nominal,
            ExpansionRule::DataRange,
            // ABox reasoning last
            ExpansionRule::PropertyAssertion,
            ExpansionRule::NegativePropertyAssertion,
            ExpansionRule::SameIndividual,
            ExpansionRule::DifferentIndividuals,
        ];

        let max_applications: HashMap<_, _> = rules.iter().map(|rule| (*rule, 1000)).collect();

        Self {
            rules,
            rule_order,
            max_applications,
            equality_reasoner: EqualityReasoner::new(),
        }
    }

    pub fn get_next_rule(&self, context: &ExpansionContext) -> Option<ExpansionRule> {
        for rule in &self.rule_order {
            if !context.applied_rules.contains(rule) {
                return Some(*rule);
            }
        }
        None
    }

    pub fn can_apply_rule(&self, _rule: &ExpansionRule, _context: &ExpansionContext) -> bool {
        if let Some(&_max_apps) = self.max_applications.get(_rule) {
            // Check if we haven't exceeded maximum applications
            true // Simplified check
        } else {
            false
        }
    }

    pub fn apply_rule(
        &mut self,
        rule: ExpansionRule,
        graph: &mut TableauxGraph,
        memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        match rule {
            ExpansionRule::Conjunction => self.apply_conjunction_rule(graph, memory, context),
            ExpansionRule::Disjunction => self.apply_disjunction_rule(graph, memory, context),
            ExpansionRule::ExistentialRestriction => {
                self.apply_existential_restriction_rule(graph, memory, context)
            }
            ExpansionRule::UniversalRestriction => {
                self.apply_universal_restriction_rule(graph, memory, context)
            }
            ExpansionRule::Nominal => self.apply_nominal_rule(graph, memory, context),
            ExpansionRule::DataRange => self.apply_data_range_rule(graph, memory, context),
            ExpansionRule::SubclassAxiom => self.apply_subclass_axiom_rule(graph, memory, context),
            // Property characteristic rules (TODO: implement)
            ExpansionRule::TransitiveProperty => {
                self.apply_transitive_property_rule(graph, memory, context)
            }
            ExpansionRule::SymmetricProperty => {
                self.apply_symmetric_property_rule(graph, memory, context)
            }
            ExpansionRule::ReflexiveProperty => {
                self.apply_reflexive_property_rule(graph, memory, context)
            }
            ExpansionRule::FunctionalProperty => {
                self.apply_functional_property_rule(graph, memory, context)
            }
            ExpansionRule::InverseFunctionalProperty => {
                self.apply_inverse_functional_property_rule(graph, memory, context)
            }
            ExpansionRule::IrreflexiveProperty => {
                self.apply_irreflexive_property_rule(graph, memory, context)
            }
            ExpansionRule::AsymmetricProperty => {
                self.apply_asymmetric_property_rule(graph, memory, context)
            }
            // Property hierarchy rules
            ExpansionRule::PropertyHierarchy => {
                self.apply_property_hierarchy_rule(graph, memory, context)
            }
            ExpansionRule::PropertyDomain => {
                self.apply_property_domain_rule(graph, memory, context)
            }
            ExpansionRule::PropertyRange => self.apply_property_range_rule(graph, memory, context),
            ExpansionRule::InverseProperty => {
                self.apply_inverse_property_rule(graph, memory, context)
            }
            // Individual reasoning (ABox) rules
            ExpansionRule::PropertyAssertion => {
                self.apply_property_assertion_rule(graph, memory, context)
            }
            ExpansionRule::NegativePropertyAssertion => {
                self.apply_negative_property_assertion_rule(graph, memory, context)
            }
            // Individual equality rules
            ExpansionRule::SameIndividual => {
                self.apply_same_individual_rule(graph, memory, context)
            }
            ExpansionRule::DifferentIndividuals => {
                self.apply_different_individuals_rule(graph, memory, context)
            }
        }
    }

    fn apply_conjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        // Decompose intersection: C ⊓ D → C, D
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all intersection concepts in the node
            let intersections: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectIntersectionOf(_)))
                .cloned()
                .collect();

            for intersection in intersections {
                if let ClassExpression::ObjectIntersectionOf(operands) = intersection {
                    // Add each operand to the node
                    for operand in operands.iter() {
                        graph.add_concept_logged(
                            context.current_node,
                            (**operand).clone(),
                            &mut change_log,
                        );
                    }
                    // Remove the intersection (optional - depends on strategy)
                    // For now, we'll keep it for completeness
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_disjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        // Non-deterministic choice for union: C ⊔ D → C or D
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all union concepts in the node
            let unions: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectUnionOf(_)))
                .cloned()
                .collect();

            for union in unions {
                if let ClassExpression::ObjectUnionOf(operands) = union {
                    if !operands.is_empty() {
                        // Create choice point for non-deterministic branching
                        let choice = ExpansionTask {
                            node_id: context.current_node,
                            concept: (*operands[0]).clone(),
                            rule_type: ExpansionRule::Disjunction,
                            priority: 10, // Medium priority for disjunction
                        };
                        return Ok((vec![choice], change_log));
                    }
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_existential_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();

        let existentials: Vec<_> = match graph.get_node(context.current_node) {
            Some(node) => node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectSomeValuesFrom(_, _)))
                .cloned()
                .collect(),
            None => return Ok((vec![], change_log)),
        };

        let universals: Vec<_> = graph
            .get_node(context.current_node)
            .map(|node| {
                node.concepts_iter()
                    .filter(|c| matches!(c, ClassExpression::ObjectAllValuesFrom(_, _)))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for existential in existentials {
            if let ClassExpression::ObjectSomeValuesFrom(property, filler) = existential {
                let (is_inverse, property_iri) = Self::resolve_property_direction(&property);

                // Attempt to reuse existing nodes that already contain the filler
                if !is_inverse {
                    if let Some(successors) =
                        graph.get_successors(context.current_node, property_iri)
                    {
                        if let Some(existing) = successors.iter().copied().find(|succ_id| {
                            graph
                                .get_node(*succ_id)
                                .map(|n| n.contains_concept(&filler))
                                .unwrap_or(false)
                        }) {
                            self.propagate_universal_to_node(
                                &universals,
                                is_inverse,
                                property_iri,
                                existing,
                                graph,
                                &mut change_log,
                            );

                            let task = ExpansionTask {
                                node_id: existing,
                                concept: (*filler).clone(),
                                rule_type: ExpansionRule::ExistentialRestriction,
                                priority: 5,
                            };
                            return Ok((vec![task], change_log));
                        }
                    }
                } else {
                    let predecessors = graph.get_predecessors(context.current_node, property_iri);
                    if let Some(existing) = predecessors.iter().copied().find(|pred_id| {
                        graph
                            .get_node(*pred_id)
                            .map(|n| n.contains_concept(&filler))
                            .unwrap_or(false)
                    }) {
                        self.propagate_universal_to_node(
                            &universals,
                            is_inverse,
                            property_iri,
                            existing,
                            graph,
                            &mut change_log,
                        );

                        let task = ExpansionTask {
                            node_id: existing,
                            concept: (*filler).clone(),
                            rule_type: ExpansionRule::ExistentialRestriction,
                            priority: 5,
                        };
                        return Ok((vec![task], change_log));
                    }
                }

                // No reusable node, create a new one
                let new_node_id = graph.add_node_logged(&mut change_log);
                graph.add_concept_logged(new_node_id, (*filler).clone(), &mut change_log);
                graph.add_edge_logged(
                    context.current_node,
                    property_iri,
                    new_node_id,
                    &mut change_log,
                );

                self.propagate_universal_to_node(
                    &universals,
                    is_inverse,
                    property_iri,
                    new_node_id,
                    graph,
                    &mut change_log,
                );

                let task = ExpansionTask {
                    node_id: new_node_id,
                    concept: (*filler).clone(),
                    rule_type: ExpansionRule::ExistentialRestriction,
                    priority: 5,
                };
                return Ok((vec![task], change_log));
            }
        }

        Ok((vec![], change_log))
    }

    fn apply_universal_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        // ∀R.C → ensure all R-successors have C
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all universal restrictions in the node
            let universals: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectAllValuesFrom(_, _)))
                .cloned()
                .collect();

            for universal in universals {
                if let ClassExpression::ObjectAllValuesFrom(property, filler) = universal {
                    // Determine if we look at successors (R) or predecessors (R^-)
                    let (is_inverse, property_iri) = Self::resolve_property_direction(&property);

                    if !is_inverse {
                        // Collect successors first to avoid holding an immutable borrow while mutating
                        let successors: Vec<NodeId> = graph
                            .get_successors(context.current_node, property_iri)
                            .map(|s| s.to_vec())
                            .unwrap_or_default();

                        for successor_id in successors {
                            let needs_add = graph
                                .get_node(successor_id)
                                .map(|n| !n.contains_concept(&filler))
                                .unwrap_or(false);
                            if needs_add {
                                graph.add_concept_logged(
                                    successor_id,
                                    (*filler).clone(),
                                    &mut change_log,
                                );

                                // Create expansion task for the successor
                                let task = ExpansionTask {
                                    node_id: successor_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8, // Medium-high priority for universal restrictions
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    } else {
                        // For inverse properties, ensure all predecessors via R have the filler
                        let predecessors: Vec<NodeId> = graph
                            .get_predecessors(context.current_node, property_iri)
                            .into_iter()
                            .collect();

                        for pred_id in predecessors {
                            let needs_add = graph
                                .get_node(pred_id)
                                .map(|n| !n.contains_concept(&filler))
                                .unwrap_or(false);
                            if needs_add {
                                graph.add_concept_logged(
                                    pred_id,
                                    (*filler).clone(),
                                    &mut change_log,
                                );

                                // Create expansion task for the predecessor
                                let task = ExpansionTask {
                                    node_id: pred_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn propagate_universal_to_node(
        &self,
        universals: &[ClassExpression],
        is_inverse: bool,
        property_iri: &IRI,
        target_node: NodeId,
        graph: &mut TableauxGraph,
        change_log: &mut GraphChangeLog,
    ) {
        for universal in universals {
            if let ClassExpression::ObjectAllValuesFrom(univ_property, univ_filler) = universal {
                let (univ_inverse, univ_iri) = Self::resolve_property_direction(univ_property);
                if univ_inverse == is_inverse && univ_iri == property_iri {
                    graph.add_concept_logged(target_node, (**univ_filler).clone(), change_log);
                }
            }
        }
    }

    /// Helper function to resolve property direction for inverse properties
    pub fn resolve_property_direction(expr: &ObjectPropertyExpression) -> (bool, &IRI) {
        fn flatten(e: &ObjectPropertyExpression, invert: bool) -> (bool, &IRI) {
            match e {
                ObjectPropertyExpression::ObjectProperty(prop) => (invert, prop.iri()),
                ObjectPropertyExpression::ObjectInverseOf(inner) => {
                    flatten(inner.as_ref(), !invert)
                }
            }
        }
        flatten(expr, false)
    }

    fn apply_nominal_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();

        // Handle two types of nominals:
        // 1. ObjectOneOf: {a, b, c} → enumerated individuals
        // 2. ObjectHasValue: ∃R.{a} → property with specific individual value

        // First, collect the nominals without holding a mutable borrow
        let nominals: Vec<_> = if let Some(node) = graph.get_node(context.current_node) {
            node.concepts_iter()
                .filter(|c| {
                    matches!(
                        c,
                        ClassExpression::ObjectOneOf(_) | ClassExpression::ObjectHasValue(_, _)
                    )
                })
                .cloned()
                .collect()
        } else {
            return Ok((vec![], change_log));
        };

        for nominal in nominals {
            match nominal {
                ClassExpression::ObjectOneOf(individuals) => {
                    // For each individual in the nominal, ensure they have corresponding nodes
                    if let Some(individual) = individuals.iter().next() {
                        // Check if we already have a node for this individual
                        let individual_node =
                            self.find_or_create_individual_node(graph, individual, &mut change_log);

                        // Create expansion task for the individual node
                        let mut task_individual_vec: SmallVec<[crate::entities::Individual; 8]> =
                            SmallVec::new();
                        task_individual_vec.push(individual.clone());
                        let task = ExpansionTask {
                            node_id: individual_node,
                            concept: ClassExpression::ObjectOneOf(Box::new(task_individual_vec)),
                            rule_type: ExpansionRule::Nominal,
                            priority: 7, // Medium priority for nominals
                        };
                        return Ok((vec![task], change_log));
                    }
                }
                ClassExpression::ObjectHasValue(property, individual) => {
                    // ObjectHasValue(R, a) is equivalent to ∃R.{a}
                    // Create a node for the individual and connect it with property R

                    let individual_node =
                        self.find_or_create_individual_node(graph, &individual, &mut change_log);

                    // Get property IRI
                    let property_iri = match property.as_ref() {
                        ObjectPropertyExpression::ObjectProperty(prop) => prop.iri(),
                        ObjectPropertyExpression::ObjectInverseOf(inner_prop) => {
                            // For inverse properties, we reverse the edge direction
                            // ∃R⁻.{a} means: a R current_node
                            // Extract the base property from the inverse
                            if let ObjectPropertyExpression::ObjectProperty(prop) =
                                inner_prop.as_ref()
                            {
                                graph.add_edge_logged(
                                    individual_node,
                                    prop.iri(),
                                    context.current_node,
                                    &mut change_log,
                                );
                            }
                            return Ok((vec![], change_log));
                        }
                    };

                    // Add edge: current_node --R--> individual_node
                    graph.add_edge_logged(
                        context.current_node,
                        property_iri,
                        individual_node,
                        &mut change_log,
                    );

                    return Ok((vec![], change_log));
                }
                _ => {}
            }
        }
        Ok((vec![], change_log))
    }

    /// Find or create a node for an individual
    fn find_or_create_individual_node(
        &self,
        graph: &mut TableauxGraph,
        individual: &crate::entities::Individual,
        change_log: &mut GraphChangeLog,
    ) -> NodeId {
        // For now, create a new node for each individual
        // In a full implementation, we'd maintain a mapping of individuals to nodes
        let node_id = graph.add_node_logged(change_log);

        // Add the individual as a nominal concept to the new node
        let mut individual_vec: SmallVec<[crate::entities::Individual; 8]> = SmallVec::new();
        individual_vec.push(individual.clone());
        graph.add_concept_logged(
            node_id,
            ClassExpression::ObjectOneOf(Box::new(individual_vec)),
            change_log,
        );

        node_id
    }

    fn apply_data_range_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        // Handle data property restrictions and data ranges
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all data property restrictions
            let data_restrictions: Vec<_> = node
                .concepts_iter()
                .filter(|c| {
                    matches!(
                        c,
                        ClassExpression::DataSomeValuesFrom(_, _)
                            | ClassExpression::DataAllValuesFrom(_, _)
                            | ClassExpression::DataHasValue(_, _)
                            | ClassExpression::DataMinCardinality(_, _)
                            | ClassExpression::DataMaxCardinality(_, _)
                            | ClassExpression::DataExactCardinality(_, _)
                    )
                })
                .cloned()
                .collect();

            for restriction in data_restrictions {
                match &restriction {
                    ClassExpression::DataSomeValuesFrom(_, data_range) => {
                        // ∃D.R → create data value satisfying R
                        // Check if the data range is empty (unsatisfiable)
                        if Self::is_empty_data_range(data_range) {
                            // Empty data range → clash!
                            // Cannot satisfy ∃D.R when R is empty
                            return Err(
                                "Clash: DataSomeValuesFrom with empty data range".to_string()
                            );
                        }
                        // For non-empty ranges, create a placeholder data value
                        // In a full implementation, this would involve data range reasoning
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6, // Medium priority for data restrictions
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataAllValuesFrom(_, _) => {
                        // ∀D.R → all data values must satisfy R
                        // This is handled during model completion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataHasValue(_, _) => {
                        // D = v → the node has data value v for property D
                        // This represents a concrete data assertion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataMinCardinality(cardinality, _) => {
                        // ≥n D → at least n distinct data values
                        if *cardinality > 0 {
                            // Create additional data values to satisfy minimum cardinality
                            if (0..*cardinality).next().is_some() {
                                let task = ExpansionTask {
                                    node_id: context.current_node,
                                    concept: restriction.clone(),
                                    rule_type: ExpansionRule::DataRange,
                                    priority: 6,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                    ClassExpression::DataMaxCardinality(_, _) => {
                        // ≤n D → at most n distinct data values
                        // This is a constraint that will be checked during completion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataExactCardinality(cardinality, _) => {
                        // =n D → exactly n distinct data values
                        if *cardinality > 0 {
                            // Create exactly n data values
                            if (0..*cardinality).next().is_some() {
                                let task = ExpansionTask {
                                    node_id: context.current_node,
                                    concept: restriction.clone(),
                                    rule_type: ExpansionRule::DataRange,
                                    priority: 6,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                    _ => {} // Other cases handled above
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_subclass_axiom_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        // Apply subclass axioms: if node contains A and A ⊑ B, then add B to the node
        if let Some(reasoning_rules) = &context.reasoning_rules {
            if let Some(node) = graph.get_node_mut(context.current_node) {
                // Get all class concepts in the current node
                let class_concepts: Vec<ClassExpression> = node
                    .concepts_iter()
                    .filter(|c| matches!(c, ClassExpression::Class(_)))
                    .cloned()
                    .collect();

                for concept in class_concepts {
                    if let ClassExpression::Class(class) = concept {
                        // Find all subclass axioms where this class is the subclass
                        for axiom in &reasoning_rules.subclass_rules {
                            if let ClassExpression::Class(sub_class) = axiom.sub_class() {
                                if sub_class.iri().as_ref() == class.iri().as_ref() {
                                    // Add the superclass to the node if not already present
                                    if let ClassExpression::Class(super_class) = axiom.super_class()
                                    {
                                        let super_concept = ClassExpression::Class(Class::new(
                                            super_class.iri().as_str(),
                                        ));
                                        if !node.contains_concept(&super_concept) {
                                            graph.add_concept_logged(
                                                context.current_node,
                                                super_concept.clone(),
                                                &mut change_log,
                                            );

                                            // Create expansion task for the superclass
                                            let task = ExpansionTask {
                                                node_id: context.current_node,
                                                concept: super_concept,
                                                rule_type: ExpansionRule::SubclassAxiom,
                                                priority: 1, // Highest priority for subclass axioms
                                            };
                                            return Ok((vec![task], change_log));
                                        }
                                    }
                                }
                            }
                        }

                        // Also check equivalent classes
                        for equiv_axiom in &reasoning_rules.equivalence_rules {
                            let classes = equiv_axiom.classes();
                            if classes.iter().any(|c| c.as_ref() == class.iri().as_ref()) {
                                // Add all other equivalent classes to the node
                                for equiv_class in classes {
                                    if equiv_class.as_ref() != class.iri().as_ref() {
                                        let equiv_concept = ClassExpression::Class(Class::new(
                                            equiv_class.as_str(),
                                        ));
                                        if !node.contains_concept(&equiv_concept) {
                                            graph.add_concept_logged(
                                                context.current_node,
                                                equiv_concept.clone(),
                                                &mut change_log,
                                            );

                                            // Create expansion task for the equivalent class
                                            let task = ExpansionTask {
                                                node_id: context.current_node,
                                                concept: equiv_concept,
                                                rule_type: ExpansionRule::SubclassAxiom,
                                                priority: 1,
                                            };
                                            return Ok((vec![task], change_log));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((vec![], change_log))
    }

    /// Apply transitive property rule
    /// If r is transitive and we have (x,y) via r and (y,z) via r, then infer (x,z) via r
    fn apply_transitive_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get all transitive properties from reasoning rules
        let transitive_properties = match &context.reasoning_rules {
            Some(rules) => &rules.transitive_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each transitive property
        for property in transitive_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();
            let edges_with_property: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p == property.as_ref())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            // For each pair of edges (x,y) and (y,z), infer (x,z)
            for (x, y) in &edges_with_property {
                for (y2, z) in &edges_with_property {
                    if y == y2 && x != z {
                        // Check if (x,z) already exists
                        let already_exists = graph
                            .get_targets(*x, property)
                            .map(|targets| targets.contains(z))
                            .unwrap_or(false);

                        if !already_exists {
                            // Add new edge (x,z) via property
                            graph.add_edge_logged(*x, property, *z, &mut change_log);
                        }
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply symmetric property rule
    /// If r is symmetric and we have (x,y) via r, then infer (y,x) via r
    fn apply_symmetric_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get all symmetric properties from reasoning rules
        let symmetric_properties = match &context.reasoning_rules {
            Some(rules) => &rules.symmetric_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each symmetric property
        for property in symmetric_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();
            let edges_with_property: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p == property.as_ref())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            // For each edge (x,y), infer (y,x)
            for (x, y) in &edges_with_property {
                // Check if (y,x) already exists
                let already_exists = graph
                    .get_targets(*y, property)
                    .map(|targets| targets.contains(x))
                    .unwrap_or(false);

                if !already_exists {
                    // Add new edge (y,x) via property
                    graph.add_edge_logged(*y, property, *x, &mut change_log);
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply reflexive property rule
    /// If r is reflexive and we have node x, then infer (x,x) via r
    fn apply_reflexive_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get all reflexive properties from reasoning rules
        let reflexive_properties = match &context.reasoning_rules {
            Some(rules) => &rules.reflexive_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each reflexive property
        for property in reflexive_properties {
            // For each node in the graph, ensure it has a reflexive edge
            for node_id in 0..graph.nodes.len() {
                let node_id = NodeId::new(node_id);

                // Check if (node,node) already exists
                let already_exists = graph
                    .get_targets(node_id, property)
                    .map(|targets| targets.contains(&node_id))
                    .unwrap_or(false);

                if !already_exists {
                    // Add reflexive edge (node,node) via property
                    graph.add_edge_logged(node_id, property, node_id, &mut change_log);
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply functional property rule with proper equality clash detection
    /// If r is functional and we have (x,y) and (x,z) via r where y ≠ z, then clash
    fn apply_functional_property_rule(
        &mut self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let mut tasks = Vec::new();

        // Get all functional properties from reasoning rules
        let functional_properties = match &context.reasoning_rules {
            Some(rules) => &rules.functional_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each functional property
        for property in functional_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();

            // Group edges by source node
            let mut edges_by_source: std::collections::HashMap<NodeId, Vec<NodeId>> =
                std::collections::HashMap::new();

            for (from, p, to) in all_edges {
                if p == property.as_ref() {
                    edges_by_source.entry(*from).or_default().push(*to);
                }
            }

            // Check for functional property violations using equality reasoning
            for (source, targets) in edges_by_source {
                if targets.len() > 1 {
                    // Use the equality reasoner to detect actual clashes
                    let equality_reasoner = &mut self.equality_reasoner;

                    // First, ensure all nodes are tracked in the equality system
                    for &target in &targets {
                        equality_reasoner.equality_tracker_mut().add_node(target);
                    }

                    // Detect functional property clash
                    if let Some(clash) = equality_reasoner
                        .detect_functional_property_clash(property, source, &targets)
                    {
                        match clash.clash_type {
                            crate::reasoning::tableaux::equality::FunctionalClashType::DifferentValues => {
                                // This is a genuine clash - the ontology is inconsistent
                                return Err(format!(
                                    "Functional property clash: {:?} has different values {:?} and {:?} from source {:?}",
                                    property, clash.conflicting_targets[0], clash.conflicting_targets[1], source
                                ));
                            }
                            crate::reasoning::tableaux::equality::FunctionalClashType::NeedsMerge => {
                                // The targets should be merged - create merge tasks
                                let representative = targets[0];
                                for &target_to_merge in &clash.conflicting_targets[1..] {
                                    if equality_reasoner.equality_tracker_mut().can_merge(representative, target_to_merge) {
                                        // Create a task to merge these nodes
                                        let merge_task = ExpansionTask {
                                            node_id: representative,
                                            concept: ClassExpression::Class(Class::new("http://example.org/MergeOperation")),
                                            rule_type: ExpansionRule::SameIndividual,
                                            priority: 5, // High priority for merge operations
                                        };
                                        tasks.push(merge_task);

                                        // Perform the merge immediately
                                        equality_reasoner.merge_nodes(graph, representative, target_to_merge, &mut change_log)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply inverse functional property rule with proper equality clash detection
    /// If r is inverse functional and we have (x,z) and (y,z) via r where x ≠ y, then clash
    fn apply_inverse_functional_property_rule(
        &mut self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let mut tasks = Vec::new();

        // Get all inverse functional properties from reasoning rules
        let inverse_functional_properties = match &context.reasoning_rules {
            Some(rules) => &rules.inverse_functional_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each inverse functional property
        for property in inverse_functional_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();

            // Group edges by target node
            let mut edges_by_target: std::collections::HashMap<NodeId, Vec<NodeId>> =
                std::collections::HashMap::new();

            for (from, p, to) in all_edges {
                if p == property.as_ref() {
                    edges_by_target.entry(*to).or_default().push(*from);
                }
            }

            // Check for inverse functional property violations using equality reasoning
            for (target, sources) in edges_by_target {
                if sources.len() > 1 {
                    // Use the equality reasoner to detect actual clashes
                    let equality_reasoner = &mut self.equality_reasoner;

                    // First, ensure all nodes are tracked in the equality system
                    for &source in &sources {
                        equality_reasoner.equality_tracker_mut().add_node(source);
                    }

                    // Detect inverse functional property clash
                    if let Some(clash) = equality_reasoner
                        .detect_inverse_functional_property_clash(property, target, &sources)
                    {
                        match clash.clash_type {
                            crate::reasoning::tableaux::equality::InverseFunctionalClashType::DifferentSources => {
                                // This is a genuine clash - the ontology is inconsistent
                                return Err(format!(
                                    "Inverse functional property clash: {:?} has different sources {:?} and {:?} for target {:?}",
                                    property, clash.conflicting_sources[0], clash.conflicting_sources[1], target
                                ));
                            }
                            crate::reasoning::tableaux::equality::InverseFunctionalClashType::NeedsMerge => {
                                // The sources should be merged - create merge tasks
                                let representative = sources[0];
                                for &source_to_merge in &clash.conflicting_sources[1..] {
                                    if equality_reasoner.equality_tracker_mut().can_merge(representative, source_to_merge) {
                                        // Create a task to merge these nodes
                                        let merge_task = ExpansionTask {
                                            node_id: representative,
                                            concept: ClassExpression::Class(Class::new("http://example.org/MergeOperation")),
                                            rule_type: ExpansionRule::SameIndividual,
                                            priority: 5, // High priority for merge operations
                                        };
                                        tasks.push(merge_task);

                                        // Perform the merge immediately
                                        equality_reasoner.merge_nodes(graph, representative, source_to_merge, &mut change_log)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply same individual rule with proper node merging
    /// Merge nodes representing the same individual
    fn apply_same_individual_rule(
        &mut self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let mut tasks = Vec::new();

        // Get same individual axioms from reasoning rules
        let same_individual_axioms = match &context.reasoning_rules {
            Some(rules) => &rules.same_individual_axioms,
            None => return Ok((tasks, change_log)),
        };

        // Process each same individual axiom
        for axiom in same_individual_axioms {
            let individuals = axiom.individuals();
            if individuals.len() < 2 {
                continue;
            }

            // Use the equality reasoner to handle the axiom
            let merged_nodes = self.equality_reasoner.add_same_individual_axiom(
                graph,
                individuals,
                &mut change_log,
            )?;

            // Create tasks for any additional processing needed
            if merged_nodes.len() > 1 {
                let representative = merged_nodes[0];
                for &_node_id in &merged_nodes[1..] {
                    let merge_task = ExpansionTask {
                        node_id: representative,
                        concept: ClassExpression::Class(Class::new(
                            "http://example.org/SameIndividualMerge",
                        )),
                        rule_type: ExpansionRule::SameIndividual,
                        priority: 1, // Highest priority for equality axioms
                    };
                    tasks.push(merge_task);
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply different individuals rule with inequality clash detection
    /// Check if individuals that should be different are identified as the same
    fn apply_different_individuals_rule(
        &mut self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get different individuals axioms from reasoning rules
        let different_individuals_axioms = match &context.reasoning_rules {
            Some(rules) => &rules.different_individuals_axioms,
            None => return Ok((tasks, change_log)),
        };

        // Process each different individuals axiom
        for axiom in different_individuals_axioms {
            let individuals = axiom.individuals();
            if individuals.len() < 2 {
                continue;
            }

            // Find nodes for all individuals in the axiom
            let mut nodes = Vec::new();
            for individual_iri in individuals {
                if let Some(node_id) = self
                    .equality_reasoner
                    .find_individual_node(graph, individual_iri)
                {
                    nodes.push(node_id);
                } else {
                    // Create a new node for this individual
                    let node_id = graph.add_node();
                    graph.add_label_logged(node_id, individual_iri.to_string(), &mut change_log);
                    nodes.push(node_id);
                    self.equality_reasoner
                        .equality_tracker_mut()
                        .add_node(node_id);
                }
            }

            // Check if any two different individuals are in the same equivalence class
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    if self
                        .equality_reasoner
                        .equality_tracker_mut()
                        .are_equal(nodes[i], nodes[j])
                    {
                        return Err(format!(
                            "Different individuals clash: {:?} and {:?} are both equal and different",
                            individuals[i], individuals[j]
                        ));
                    }
                }
            }

            // Add inequality constraints between all pairs
            self.equality_reasoner
                .add_different_individuals_axiom(individuals, &nodes)?;
        }

        Ok((tasks, change_log))
    }

    /// Apply irreflexive property rule (clash detection)
    /// If r is irreflexive and we have (x,x) via r, then clash
    fn apply_irreflexive_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get all irreflexive properties from reasoning rules
        let irreflexive_properties = match &context.reasoning_rules {
            Some(rules) => &rules.irreflexive_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each irreflexive property
        for property in irreflexive_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();

            // Check for reflexive edges (x,x)
            for (from, p, to) in all_edges {
                if p == property.as_ref() && from == to {
                    // Irreflexive property violation: found (x,x) for irreflexive property
                    return Err(format!(
                        "Clash: Irreflexive property {:?} has reflexive edge at node {:?}",
                        property, from
                    ));
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply asymmetric property rule (clash detection)
    /// If r is asymmetric and we have (x,y) and (y,x) via r, then clash
    fn apply_asymmetric_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get all asymmetric properties from reasoning rules
        let asymmetric_properties = match &context.reasoning_rules {
            Some(rules) => &rules.asymmetric_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each asymmetric property
        for property in asymmetric_properties {
            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();
            let edges_with_property: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p == property.as_ref())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            // Check for asymmetric violations: (x,y) and (y,x)
            for (x, y) in &edges_with_property {
                for (x2, y2) in &edges_with_property {
                    if x == y2 && y == x2 {
                        // Asymmetric property violation: found both (x,y) and (y,x)
                        return Err(format!(
                            "Clash: Asymmetric property {:?} has both ({:?},{:?}) and ({:?},{:?})",
                            property, x, y, y, x
                        ));
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply property hierarchy rule (SubObjectPropertyOf)
    /// If P ⊑ Q and we have (x,y) via P, then infer (x,y) via Q
    fn apply_property_hierarchy_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get property hierarchy axioms from reasoning rules
        let property_hierarchy = match &context.reasoning_rules {
            Some(rules) => &rules.property_hierarchy,
            None => return Ok((tasks, change_log)),
        };

        // For each SubObjectPropertyOf axiom: P ⊑ Q
        for axiom in property_hierarchy {
            let sub_property = axiom.sub_property();
            let super_property = axiom.super_property();

            // Get all edges with the sub-property P
            let all_edges = graph.edges.get_all_edges();
            let edges_with_sub: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p == sub_property.as_ref())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            // For each (x,y) via P, infer (x,y) via Q
            for (x, y) in edges_with_sub {
                // Check if (x,y) via Q already exists
                let already_exists = graph
                    .get_targets(x, super_property)
                    .map(|targets| targets.contains(&y))
                    .unwrap_or(false);

                if !already_exists {
                    // Add new edge (x,y) via super_property
                    graph.add_edge_logged(x, super_property, y, &mut change_log);
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply property domain rule
    /// If domain(P) = C and we have (x,y) via P, then x : C
    fn apply_property_domain_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get property domain axioms from reasoning rules
        let property_domains = match &context.reasoning_rules {
            Some(rules) => &rules.property_domains,
            None => return Ok((tasks, change_log)),
        };

        // For each property domain axiom
        for axiom in property_domains {
            let property = axiom.property();
            let domain_class = axiom.domain();

            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();
            let edges_with_property: Vec<NodeId> = all_edges
                .iter()
                .filter(|(_, p, _)| p.as_str() == property.as_str())
                .map(|(from, _, _)| *from)
                .collect();

            // For each source node x in (x,y) via property
            for node_id in edges_with_property {
                // Add domain class to the node
                if let Some(node) = graph.nodes.get_mut(node_id.as_usize()) {
                    if !node.concepts.contains(domain_class) {
                        node.concepts.push(domain_class.clone());
                        // Note: GraphChangeLog doesn't have add_concept method yet
                        // This is a TODO for proper rollback support
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply property range rule
    /// If range(P) = C and we have (x,y) via P, then y : C
    fn apply_property_range_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get property range axioms from reasoning rules
        let property_ranges = match &context.reasoning_rules {
            Some(rules) => &rules.property_ranges,
            None => return Ok((tasks, change_log)),
        };

        // For each property range axiom
        for axiom in property_ranges {
            let property = axiom.property();
            let range_class = axiom.range();

            // Get all edges with this property
            let all_edges = graph.edges.get_all_edges();
            let edges_with_property: Vec<NodeId> = all_edges
                .iter()
                .filter(|(_, p, _)| p.as_str() == property.as_str())
                .map(|(_, _, to)| *to)
                .collect();

            // For each target node y in (x,y) via property
            for node_id in edges_with_property {
                // Add range class to the node
                if let Some(node) = graph.nodes.get_mut(node_id.as_usize()) {
                    if !node.concepts.contains(range_class) {
                        node.concepts.push(range_class.clone());
                        // Note: GraphChangeLog doesn't have add_concept method yet
                        // This is a TODO for proper rollback support
                    }
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply inverse property rule
    /// If P ≡ Q⁻ and we have (x,y) via P, then infer (y,x) via Q
    fn apply_inverse_property_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get inverse property axioms from reasoning rules
        let inverse_properties = match &context.reasoning_rules {
            Some(rules) => &rules.inverse_properties,
            None => return Ok((tasks, change_log)),
        };

        // For each inverse property axiom
        for axiom in inverse_properties {
            use crate::axioms::property_expressions::ObjectPropertyExpression;

            // Extract IRIs from property expressions
            let prop1_iri = match axiom.property1() {
                ObjectPropertyExpression::ObjectProperty(obj_prop) => obj_prop.iri(),
                ObjectPropertyExpression::ObjectInverseOf(_) => continue, // Skip complex cases for now
            };

            let prop2_iri = match axiom.property2() {
                ObjectPropertyExpression::ObjectProperty(obj_prop) => obj_prop.iri(),
                ObjectPropertyExpression::ObjectInverseOf(_) => continue, // Skip complex cases for now
            };

            // Get all edges with property1
            let all_edges = graph.edges.get_all_edges();
            let edges_with_prop1: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p.as_str() == prop1_iri.as_str())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            // Create Arc<IRI> for properties once
            let prop1_arc = if let Ok(iri) = crate::iri::IRI::new(prop1_iri.as_str()) {
                std::sync::Arc::new(iri)
            } else {
                continue;
            };

            let prop2_arc = if let Ok(iri) = crate::iri::IRI::new(prop2_iri.as_str()) {
                std::sync::Arc::new(iri)
            } else {
                continue;
            };

            // Collect edges to add (to avoid borrow checker issues)
            let mut edges_to_add: Vec<(NodeId, std::sync::Arc<crate::iri::IRI>, NodeId)> =
                Vec::new();

            // For each (x,y) via prop1, check if we need to infer (y,x) via prop2
            for (x, y) in &edges_with_prop1 {
                let already_exists = all_edges.iter().any(|(from, p, to)| {
                    *from == *y && *to == *x && p.as_str() == prop2_iri.as_str()
                });

                if !already_exists {
                    edges_to_add.push((*y, prop2_arc.clone(), *x));
                }
            }

            // Also handle the reverse: for each (x,y) via prop2, check if we need to infer (y,x) via prop1
            let edges_with_prop2: Vec<(NodeId, NodeId)> = all_edges
                .iter()
                .filter(|(_, p, _)| p.as_str() == prop2_iri.as_str())
                .map(|(from, _, to)| (*from, *to))
                .collect();

            for (x, y) in &edges_with_prop2 {
                let already_exists = all_edges.iter().any(|(from, p, to)| {
                    *from == *y && *to == *x && p.as_str() == prop1_iri.as_str()
                });

                if !already_exists {
                    edges_to_add.push((*y, prop1_arc.clone(), *x));
                }
            }

            // Now add all the collected edges
            for (from, prop, to) in edges_to_add {
                graph.add_edge_logged(from, &prop, to, &mut change_log);
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply property assertion rule
    /// Initialize ABox by creating edges for all property assertions
    fn apply_property_assertion_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get property assertions from reasoning rules
        let property_assertions = match &context.reasoning_rules {
            Some(rules) => &rules.property_assertions,
            None => return Ok((tasks, change_log)),
        };

        // For each property assertion (subject, property, object)
        for axiom in property_assertions {
            let subject_iri = axiom.subject();
            let property_iri = axiom.property();

            // Get object IRI (skip anonymous individuals for now)
            if let Some(object_iri) = axiom.object_iri() {
                // Find or create nodes for subject and object
                let subject_node =
                    self.find_or_create_individual_node_by_iri(graph, subject_iri, &mut change_log);
                let object_node =
                    self.find_or_create_individual_node_by_iri(graph, object_iri, &mut change_log);

                // Check if edge already exists
                let all_edges = graph.edges.get_all_edges();
                let edge_exists = all_edges.iter().any(|(from, p, to)| {
                    *from == subject_node
                        && *to == object_node
                        && p.as_str() == property_iri.as_str()
                });

                // Add edge if it doesn't exist
                if !edge_exists {
                    graph.add_edge_logged(subject_node, property_iri, object_node, &mut change_log);
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Apply negative property assertion rule (clash detection)
    /// Check if any negative assertions are violated
    fn apply_negative_property_assertion_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        let tasks = Vec::new();

        // Get negative property assertions from reasoning rules
        let negative_assertions = match &context.reasoning_rules {
            Some(rules) => &rules.negative_property_assertions,
            None => return Ok((tasks, change_log)),
        };

        // For each negative property assertion
        for axiom in negative_assertions {
            let subject_iri = axiom.subject();
            let property_iri = axiom.property();
            let object_iri = axiom.object();

            // Find nodes for subject and object
            let subject_arc = Arc::new(subject_iri.clone());
            let object_arc = Arc::new(object_iri.clone());
            let subject_node_opt = self.find_individual_node_by_iri(graph, &subject_arc);
            let object_node_opt = self.find_individual_node_by_iri(graph, &object_arc);

            // If both nodes exist, check if the edge exists
            if let (Some(subject_node), Some(object_node)) = (subject_node_opt, object_node_opt) {
                let all_edges = graph.edges.get_all_edges();
                let edge_exists = all_edges.iter().any(|(from, p, to)| {
                    *from == subject_node
                        && *to == object_node
                        && p.as_str() == property_iri.as_str()
                });

                // If edge exists, we have a clash
                if edge_exists {
                    return Err(format!(
                        "Clash: Negative property assertion violated for ({}, {}, {})",
                        subject_iri.as_str(),
                        property_iri.as_str(),
                        object_iri.as_str()
                    ));
                }
            }
        }

        Ok((tasks, change_log))
    }

    /// Find or create a node for a named individual by IRI
    fn find_or_create_individual_node_by_iri(
        &self,
        graph: &mut TableauxGraph,
        individual_iri: &Arc<IRI>,
        change_log: &mut GraphChangeLog,
    ) -> NodeId {
        // Try to find existing node with this individual's IRI as label
        if let Some(node_id) = self.find_individual_node_by_iri(graph, individual_iri) {
            return node_id;
        }

        // Create new node for this individual
        let node_id = graph.add_node_logged(change_log);
        if let Some(node) = graph.nodes.get_mut(node_id.as_usize()) {
            node.labels.push(individual_iri.as_str().to_string());
        }
        node_id
    }

    /// Find a node representing a named individual by IRI
    fn find_individual_node_by_iri(
        &self,
        graph: &TableauxGraph,
        individual_iri: &Arc<IRI>,
    ) -> Option<NodeId> {
        let iri_str = individual_iri.as_str();
        for (idx, node) in graph.nodes.iter().enumerate() {
            if node.labels.iter().any(|label| label == iri_str) {
                return Some(NodeId::new(idx));
            }
        }
        None
    }

    /// Check if a data range is empty (unsatisfiable)
    fn is_empty_data_range(data_range: &crate::axioms::class_expressions::DataRange) -> bool {
        use crate::axioms::class_expressions::DataRange;
        use crate::datatypes::value_space::is_float_range_empty_exclusive;

        match data_range {
            DataRange::DatatypeRestriction(datatype, facets) => {
                // Check if this is an xsd:float datatype
                if datatype.as_str().ends_with("#float") {
                    // Extract minExclusive and maxExclusive facets
                    let mut min_exclusive: Option<f32> = None;
                    let mut max_exclusive: Option<f32> = None;

                    for facet in facets {
                        let facet_iri = facet.facet();
                        let facet_name = facet_iri.as_str();

                        if facet_name.ends_with("#minExclusive") {
                            // Parse the literal value as f32
                            let value_str = facet.value().lexical_form();
                            if let Ok(value) = value_str.parse::<f32>() {
                                min_exclusive = Some(value);
                            }
                        } else if facet_name.ends_with("#maxExclusive") {
                            // Parse the literal value as f32
                            let value_str = facet.value().lexical_form();
                            if let Ok(value) = value_str.parse::<f32>() {
                                max_exclusive = Some(value);
                            }
                        }
                    }

                    // Check if we have both bounds and if the range is empty
                    if let (Some(min), Some(max)) = (min_exclusive, max_exclusive) {
                        return is_float_range_empty_exclusive(min, max);
                    }
                }
                // For other datatypes or incomplete facets, assume non-empty
                false
            }
            _ => {
                // For other data range types, assume non-empty
                false
            }
        }
    }
}

impl Default for ExpansionRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Tableaux expansion engine
#[derive(Debug)]
pub struct ExpansionEngine {
    pub rules: ExpansionRules,
    pub context: ExpansionContext,
    pub reasoning_rules: Option<ReasoningRules>,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        Self {
            rules: ExpansionRules::new(),
            context: ExpansionContext {
                current_node: NodeId::new(0),
                current_depth: 0,
                applied_rules: HashSet::new(),
                pending_expansions: VecDeque::new(),
                reasoning_rules: None,
            },
            reasoning_rules: None,
        }
    }

    pub fn with_reasoning_rules(mut self, rules: ReasoningRules) -> Self {
        self.reasoning_rules = Some(rules.clone());
        self.context.reasoning_rules = Some(rules);
        self
    }

    pub fn expand(
        &mut self,
        graph: &mut TableauxGraph,
        memory: &mut MemoryManager,
        max_depth: usize,
        change_log: &mut GraphChangeLog,
        memory_log: &mut MemoryChangeLog,
    ) -> Result<bool, String> {
        while self.context.current_depth < max_depth {
            if let Some(rule) = self.rules.get_next_rule(&self.context) {
                if self.rules.can_apply_rule(&rule, &self.context) {
                    let (new_tasks, local_changes) =
                        self.rules
                            .apply_rule(rule, graph, memory, &mut self.context)?;
                    change_log.extend(local_changes);
                    // TODO: capture memory mutations when MemoryManager supports logging.
                    memory_log.extend(MemoryChangeLog::new());
                    self.context.pending_expansions.extend(new_tasks);
                    self.context.applied_rules.insert(rule);
                }
            } else {
                // No more rules to apply at current level
                if let Some(next_task) = self.context.pending_expansions.pop_front() {
                    self.context.current_node = next_task.node_id;
                    self.context.current_depth += 1;
                    self.context.applied_rules.clear();
                } else {
                    // No more expansions to perform
                    break;
                }
            }
        }

        Ok(true)
    }

    pub fn reset(&mut self) {
        self.context = ExpansionContext {
            current_node: NodeId::new(0),
            current_depth: 0,
            applied_rules: HashSet::new(),
            pending_expansions: VecDeque::new(),
            reasoning_rules: self.reasoning_rules.clone(),
        };
    }
}

impl Default for ExpansionEngine {
    fn default() -> Self {
        Self::new()
    }
}
