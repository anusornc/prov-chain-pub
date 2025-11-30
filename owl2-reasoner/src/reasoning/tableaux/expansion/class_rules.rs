//! Class expression expansion rules
//!
//! Implements tableaux rules for class expressions including conjunction,
//! disjunction, existential restrictions, universal restrictions, and nominals.

use super::context::ExpansionContext;
use super::types::{ExpansionRule, ExpansionTask};
use crate::axioms::class_expressions::ClassExpression;
use crate::reasoning::tableaux::{
    core::NodeId,
    graph::{GraphChange, GraphChangeLog, TableauxGraph},
    memory::MemoryManager,
};

/// Apply class expression rules to expand the tableau
pub fn apply_class_rules(
    graph: &mut TableauxGraph,
    memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    change_log: &mut GraphChangeLog,
    rule: ExpansionRule,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    let mut tasks = Vec::new();

    match rule {
        ExpansionRule::Conjunction => {
            tasks.extend(apply_conjunction_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        ExpansionRule::Disjunction => {
            tasks.extend(apply_disjunction_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        ExpansionRule::ExistentialRestriction => {
            tasks.extend(apply_existential_restriction_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        ExpansionRule::UniversalRestriction => {
            tasks.extend(apply_universal_restriction_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        ExpansionRule::Nominal => {
            tasks.extend(apply_nominal_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        ExpansionRule::DataRange => {
            tasks.extend(apply_data_range_rule(
                graph,
                memory_manager,
                context,
                change_log,
                node_id,
                class_expression,
            )?);
        }
        _ => {
            // Not a class expression rule
        }
    }

    Ok(tasks)
}

/// Apply conjunction rule: C1 ∧ C2 ⇒ add C1 and C2 to the node
fn apply_conjunction_rule(
    graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    change_log: &mut GraphChangeLog,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    let mut tasks = Vec::new();

    if let ClassExpression::ObjectIntersectionOf(class_expressions) = class_expression {
        // Add each conjunct to the node
        for conjunct in class_expressions {
            if !graph.node_has_class_expression(node_id, conjunct) {
                let change = GraphChange::AddConcept {
                    node_id,
                    concept: conjunct.clone(),
                };
                change_log.record(change);

                graph.add_class_expression_to_node(node_id, (**conjunct).clone())?;

                // Create task for expanding the conjunct
                let task = ExpansionTask::new(ExpansionRule::Conjunction, node_id)
                    .with_class_expression((**conjunct).clone())
                    .with_depth(context.current_depth + 1);
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

/// Apply disjunction rule: C1 ∨ C2 ⇒ create choice point with branches
fn apply_disjunction_rule(
    _graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    _change_log: &mut GraphChangeLog,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    use super::context::Branch;

    let mut tasks = Vec::new();

    if let ClassExpression::ObjectUnionOf(class_expressions) = class_expression {
        let branches: Vec<Branch> = class_expressions
            .iter()
            .enumerate()
            .map(|(i, disjunct)| {
                let task = ExpansionTask::new(ExpansionRule::Disjunction, node_id)
                    .with_class_expression((**disjunct).clone())
                    .with_depth(context.current_depth + 1);

                Branch::simple(i, task, format!("Disjunction branch {}: {:?}", i, disjunct))
            })
            .collect();

        // Create branch point
        let branching_task = ExpansionTask::new(ExpansionRule::Disjunction, node_id)
            .with_class_expression((*class_expression).clone())
            .with_depth(context.current_depth);

        context.create_branch_point(node_id, branching_task, branches);

        // Add tasks from the first branch
        if let Some(first_branch) = context.current_branch() {
            tasks.extend(first_branch.tasks.clone());
        }
    }

    Ok(tasks)
}

/// Apply existential restriction rule: ∃r.C ⇒ create new node with C connected by r
fn apply_existential_restriction_rule(
    graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    change_log: &mut GraphChangeLog,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    let mut tasks = Vec::new();

    if let ClassExpression::ObjectSomeValuesFrom(property, filler) = class_expression {
        // Extract property IRI from property expression
        let property_iri = match &**property {
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                obj_prop,
            ) => (**obj_prop.iri()).clone(),
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectInverseOf(
                _inv_prop,
            ) => {
                // For inverse properties, we need to handle them differently
                // For now, create a placeholder IRI
                crate::iri::IRI::new("http://example.org/inverse").unwrap()
            }
        };

        // Check if we already have a suitable successor
        if let Some(existing_successors) = graph.get_successors(node_id, &property_iri) {
            let mut suitable_successor = None;

            for &successor_id in existing_successors {
                if graph.node_has_class_expression(successor_id, filler) {
                    suitable_successor = Some(successor_id);
                    break;
                }
            }

            if let Some(id) = suitable_successor {
                // Found a suitable existing successor
                let task = ExpansionTask::new(ExpansionRule::ExistentialRestriction, id)
                    .with_class_expression((**filler).clone())
                    .with_depth(context.current_depth + 1);
                tasks.push(task);
                return Ok(tasks);
            }
        }

        // Create new successor node (no suitable successor found)
        let new_node_id = graph.add_node();

        // Add edge from current node to new node
        let edge_change = GraphChange::AddEdge {
            from: node_id,
            property: property_iri.clone(),
            to: new_node_id,
        };
        change_log.record(edge_change);

        graph.add_edge(node_id, &property_iri, new_node_id);

        // Add filler class expression to new node
        let concept_change = GraphChange::AddConcept {
            node_id: new_node_id,
            concept: filler.clone(),
        };
        change_log.record(concept_change);

        graph.add_class_expression_to_node(new_node_id, (**filler).clone())?;

        // Create task for expanding the filler
        let task = ExpansionTask::new(ExpansionRule::ExistentialRestriction, new_node_id)
            .with_class_expression((**filler).clone())
            .with_depth(context.current_depth + 1);
        tasks.push(task);

        // Record that this existential restriction has been processed
        let task_key = (node_id, ExpansionRule::ExistentialRestriction);
        context.applied_rules.insert(task_key);
    }

    Ok(tasks)
}

/// Apply universal restriction rule: ∀r.C ⇒ add C to all r-successors
fn apply_universal_restriction_rule(
    graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    change_log: &mut GraphChangeLog,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    let mut tasks = Vec::new();

    if let ClassExpression::ObjectAllValuesFrom(property, filler) = class_expression {
        // Extract property IRI from property expression
        let property_iri = match &**property {
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                obj_prop,
            ) => (**obj_prop.iri()).clone(),
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectInverseOf(
                _inv_prop,
            ) => {
                // For inverse properties, we need to handle them differently
                // For now, create a placeholder IRI
                crate::iri::IRI::new("http://example.org/inverse").unwrap()
            }
        };

        // Get all r-successors of the current node
        let successors = graph
            .get_successors(node_id, &property_iri)
            .map(|slice| slice.to_vec())
            .unwrap_or_default();

        // Add filler class expression to each successor if not already present
        for successor_node_id in successors {
            if !graph.node_has_class_expression(successor_node_id, filler) {
                // Record the change in the change log
                change_log.record(GraphChange::AddConcept {
                    node_id: successor_node_id,
                    concept: filler.clone(),
                });

                graph.add_class_expression_to_node(successor_node_id, (**filler).clone())?;

                // Create task for expanding the filler in the successor
                let task =
                    ExpansionTask::new(ExpansionRule::UniversalRestriction, successor_node_id)
                        .with_class_expression((**filler).clone())
                        .with_depth(context.current_depth + 1);
                tasks.push(task);
            }
        }

        // Record that this universal restriction has been processed for this node
        let task_key = (node_id, ExpansionRule::UniversalRestriction);
        context.applied_rules.insert(task_key);
    }

    Ok(tasks)
}

/// Apply nominal rule: handle individual assertions
fn apply_nominal_rule(
    graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    context: &mut ExpansionContext,
    _change_log: &mut GraphChangeLog,
    node_id: NodeId,
    class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    let mut tasks = Vec::new();

    if let ClassExpression::ObjectOneOf(individuals) = class_expression {
        // For nominal (oneOf), ensure the node is one of the specified individuals
        for individual in individuals.iter() {
            if let Some(individual_iri) = individual.iri() {
                // Check if there's already a node representing this individual
                if let Some(existing_node_id) = graph.get_node_for_individual(individual_iri) {
                    // For now, just use the existing node id instead of merging
                    // TODO: Implement proper node merging in the graph

                    // Create tasks for the existing node
                    let task = ExpansionTask::new(ExpansionRule::Nominal, existing_node_id)
                        .with_iri(individual_iri.clone().into())
                        .with_depth(context.current_depth + 1);
                    tasks.push(task);
                } else {
                    // Associate this node with the individual
                    graph.associate_node_with_individual(node_id, individual_iri)?;

                    let task = ExpansionTask::new(ExpansionRule::Nominal, node_id)
                        .with_iri(individual_iri.clone().into())
                        .with_depth(context.current_depth + 1);
                    tasks.push(task);
                }
            }
        }
    }

    Ok(tasks)
}

/// Apply data range rule: handle datatype restrictions
fn apply_data_range_rule(
    _graph: &mut TableauxGraph,
    _memory_manager: &mut MemoryManager,
    _context: &mut ExpansionContext,
    _change_log: &mut GraphChangeLog,
    _node_id: NodeId,
    _class_expression: &ClassExpression,
) -> crate::error::OwlResult<Vec<ExpansionTask>> {
    // TODO: Implement data range rules
    // For now, return empty task list
    Ok(Vec::new())
}

/// Check if a class expression can be expanded with the given rule
pub fn can_apply_rule(rule: ExpansionRule, class_expression: &ClassExpression) -> bool {
    match rule {
        ExpansionRule::Conjunction => {
            matches!(class_expression, ClassExpression::ObjectIntersectionOf(_))
        }
        ExpansionRule::Disjunction => {
            matches!(class_expression, ClassExpression::ObjectUnionOf(_))
        }
        ExpansionRule::ExistentialRestriction => {
            matches!(
                class_expression,
                ClassExpression::ObjectSomeValuesFrom(_, _)
            )
        }
        ExpansionRule::UniversalRestriction => {
            matches!(class_expression, ClassExpression::ObjectAllValuesFrom(_, _))
        }
        ExpansionRule::Nominal => {
            matches!(class_expression, ClassExpression::ObjectOneOf(_))
        }
        ExpansionRule::DataRange => {
            matches!(class_expression, ClassExpression::DataSomeValuesFrom(_, _))
        }
        _ => false,
    }
}
