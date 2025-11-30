//! Tableaux expansion engine
//!
//! Main coordinator for rule application and expansion management.

use super::class_rules;
use super::context::{ExpansionContext, ExpansionStats};
use super::types::{ExpansionRule, ExpansionTask};
use crate::reasoning::tableaux::{
    core::NodeId,
    graph::{GraphChangeLog, TableauxGraph},
    memory::MemoryManager,
};

/// Tableaux expansion rules collection
#[derive(Debug)]
pub struct ExpansionRules {
    enabled_rules: std::collections::HashSet<ExpansionRule>,
}

impl ExpansionRules {
    /// Create a new rules collection with all rules enabled
    pub fn new() -> Self {
        let mut enabled_rules = std::collections::HashSet::new();
        enabled_rules.insert(ExpansionRule::Conjunction);
        enabled_rules.insert(ExpansionRule::Disjunction);
        enabled_rules.insert(ExpansionRule::ExistentialRestriction);
        enabled_rules.insert(ExpansionRule::UniversalRestriction);
        enabled_rules.insert(ExpansionRule::Nominal);
        enabled_rules.insert(ExpansionRule::DataRange);
        enabled_rules.insert(ExpansionRule::SubclassAxiom);
        enabled_rules.insert(ExpansionRule::TransitiveProperty);
        enabled_rules.insert(ExpansionRule::SymmetricProperty);
        enabled_rules.insert(ExpansionRule::ReflexiveProperty);
        enabled_rules.insert(ExpansionRule::FunctionalProperty);
        enabled_rules.insert(ExpansionRule::InverseFunctionalProperty);
        enabled_rules.insert(ExpansionRule::IrreflexiveProperty);
        enabled_rules.insert(ExpansionRule::AsymmetricProperty);
        enabled_rules.insert(ExpansionRule::PropertyHierarchy);
        enabled_rules.insert(ExpansionRule::PropertyDomain);
        enabled_rules.insert(ExpansionRule::PropertyRange);
        enabled_rules.insert(ExpansionRule::InverseProperty);
        enabled_rules.insert(ExpansionRule::PropertyAssertion);
        enabled_rules.insert(ExpansionRule::NegativePropertyAssertion);
        enabled_rules.insert(ExpansionRule::SameIndividual);
        enabled_rules.insert(ExpansionRule::DifferentIndividuals);

        Self { enabled_rules }
    }

    /// Check if a rule is enabled
    pub fn is_enabled(&self, rule: ExpansionRule) -> bool {
        self.enabled_rules.contains(&rule)
    }

    /// Enable a specific rule
    pub fn enable_rule(&mut self, rule: ExpansionRule) {
        self.enabled_rules.insert(rule);
    }

    /// Disable a specific rule
    pub fn disable_rule(&mut self, rule: ExpansionRule) {
        self.enabled_rules.remove(&rule);
    }

    /// Get all enabled rules
    pub fn enabled_rules(&self) -> &std::collections::HashSet<ExpansionRule> {
        &self.enabled_rules
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
    /// Maximum expansion depth
    max_depth: u32,
    /// Maximum number of expansions
    max_expansions: u32,
    /// Statistics
    stats: ExpansionStats,
    /// Reasoning rules to apply during expansion
    reasoning_rules: Option<crate::reasoning::tableaux::ReasoningRules>,
}

impl ExpansionEngine {
    /// Create a new expansion engine
    pub fn new() -> Self {
        Self::with_limits(1000, 10000) // Default limits
    }

    /// Create expansion engine with custom limits
    pub fn with_limits(max_depth: u32, max_expansions: u32) -> Self {
        Self {
            max_depth,
            max_expansions,
            stats: ExpansionStats::default(),
            reasoning_rules: None,
        }
    }

    /// Create expansion engine with reasoning rules
    pub fn with_reasoning_rules(
        mut self,
        rules: crate::reasoning::tableaux::ReasoningRules,
    ) -> Self {
        self.reasoning_rules = Some(rules);
        self
    }

    /// Perform expansion on the tableau graph
    pub fn expand(
        &mut self,
        graph: &mut TableauxGraph,
        memory_manager: &mut MemoryManager,
        max_depth: u32,
        _graph_log: &mut GraphChangeLog,
        _memory_log: &mut crate::reasoning::memory::MemoryChangeLog,
    ) -> crate::error::OwlResult<bool> {
        // Initialize expansion context - find root node from graph
        let root_node = graph.get_root_node().unwrap_or_else(|| NodeId::new(0));
        let mut context = ExpansionContext::new(root_node, max_depth);

        // Initialize change log
        let mut change_log = GraphChangeLog::new();

        // Add initial class expressions from root node
        self.add_initial_tasks(&mut context, graph, root_node)?;

        // Main expansion loop
        while let Some(task) = context.next_task() {
            if context.expansion_count >= self.max_expansions {
                break;
            }

            if task.depth > self.max_depth {
                continue;
            }

            // Apply the task
            let new_tasks =
                self.apply_task(graph, memory_manager, &mut context, &mut change_log, task)?;

            // Add new tasks to context
            for new_task in new_tasks {
                context.add_task(new_task);
            }

            context.increment_expansion_count();
        }

        // Update statistics
        self.stats = context.stats();

        // Check if expansion is complete
        Ok(!context.has_pending_tasks())
    }

    /// Get expansion statistics
    pub fn stats(&self) -> &ExpansionStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExpansionStats::default();
    }

    // Private methods

    /// Add initial tasks for the root node
    fn add_initial_tasks(
        &self,
        context: &mut ExpansionContext,
        graph: &TableauxGraph,
        root_node: NodeId,
    ) -> crate::error::OwlResult<()> {
        // Get all class expressions for the root node
        let class_expressions = graph.get_node_class_expressions(root_node);

        for class_expression in class_expressions {
            // Determine applicable rules
            for rule in self.get_applicable_rules(&class_expression) {
                let task = ExpansionTask::new(rule, root_node)
                    .with_class_expression(class_expression.clone())
                    .with_depth(1);
                context.add_task(task);
            }
        }

        Ok(())
    }

    /// Apply a single expansion task
    fn apply_task(
        &mut self,
        graph: &mut TableauxGraph,
        memory_manager: &mut MemoryManager,
        context: &mut ExpansionContext,
        change_log: &mut GraphChangeLog,
        task: ExpansionTask,
    ) -> crate::error::OwlResult<Vec<ExpansionTask>> {
        // Check if rule already applied to this node
        if context.has_rule_applied(task.node_id, task.rule) {
            return Ok(Vec::new());
        }

        // Set current context state
        context.set_current_node(task.node_id);

        let result = if let Some(ref class_expression) = task.class_expression {
            // Apply class expression rules
            class_rules::apply_class_rules(
                graph,
                memory_manager,
                context,
                change_log,
                task.rule,
                task.node_id,
                class_expression,
            )?
        } else {
            // Apply other rules
            Vec::new()
        };

        // Mark rule as applied
        context.mark_rule_applied(task.node_id, task.rule);

        Ok(result)
    }

    /// Get applicable rules for a class expression
    fn get_applicable_rules(
        &self,
        class_expression: &crate::axioms::class_expressions::ClassExpression,
    ) -> Vec<ExpansionRule> {
        let mut rules = Vec::new();

        // Check each rule type
        for rule in [
            ExpansionRule::Conjunction,
            ExpansionRule::Disjunction,
            ExpansionRule::ExistentialRestriction,
            ExpansionRule::UniversalRestriction,
            ExpansionRule::Nominal,
            ExpansionRule::DataRange,
        ] {
            if class_rules::can_apply_rule(rule, class_expression) {
                rules.push(rule);
            }
        }

        // Sort by priority
        rules.sort_by_key(|rule| rule.priority());

        rules
    }
}

impl Default for ExpansionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if expansion should continue
pub fn should_continue_expansion(context: &ExpansionContext, max_expansions: u32) -> bool {
    context.has_pending_tasks()
        && context.expansion_count < max_expansions
        && !context.is_max_depth_reached()
}

/// Create a basic expansion task for a node
pub fn create_basic_task(rule: ExpansionRule, node_id: NodeId, depth: u32) -> ExpansionTask {
    ExpansionTask::new(rule, node_id).with_depth(depth)
}

/// Calculate expansion progress percentage
pub fn calculate_progress(context: &ExpansionContext, max_expansions: u32) -> f64 {
    if max_expansions == 0 {
        0.0
    } else {
        (context.expansion_count as f64 / max_expansions as f64) * 100.0
    }
}
