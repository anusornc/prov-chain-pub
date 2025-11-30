//! Expansion context for rule application
//!
//! Provides tracking and management of expansion state during tableaux reasoning.

use super::types::{ExpansionRule, ExpansionTask};
use crate::reasoning::tableaux::core::NodeId;
use std::collections::{HashSet, VecDeque};

/// Expansion context for rule application
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    /// Current node being expanded
    pub current_node: NodeId,
    /// Current expansion depth
    pub current_depth: u32,
    /// Rules already applied to avoid repetition
    pub applied_rules: HashSet<(NodeId, ExpansionRule)>,
    /// Pending expansion tasks
    pub pending_expansions: VecDeque<ExpansionTask>,
    /// Branch points created during expansion
    pub branch_points: Vec<BranchPoint>,
    /// Maximum allowed depth
    pub max_depth: u32,
    /// Number of expansions performed
    pub expansion_count: u32,
    /// Nodes that have been processed
    pub processed_nodes: HashSet<NodeId>,
}

/// Branch point for non-deterministic choices
#[derive(Debug, Clone)]
pub struct BranchPoint {
    /// Node where branching occurred
    pub node_id: NodeId,
    /// Task that caused the branching
    pub branching_task: ExpansionTask,
    /// Available branches (choices)
    pub branches: Vec<Branch>,
    /// Currently selected branch index
    pub selected_branch: usize,
}

/// Individual branch in a branching point
#[derive(Debug, Clone)]
pub struct Branch {
    /// Branch identifier
    pub id: usize,
    /// Tasks specific to this branch
    pub tasks: Vec<ExpansionTask>,
    /// Description for debugging
    pub description: String,
}

impl ExpansionContext {
    /// Create a new expansion context
    pub fn new(start_node: NodeId, max_depth: u32) -> Self {
        Self {
            current_node: start_node,
            current_depth: 0,
            applied_rules: HashSet::new(),
            pending_expansions: VecDeque::new(),
            branch_points: Vec::new(),
            max_depth,
            expansion_count: 0,
            processed_nodes: HashSet::new(),
        }
    }

    /// Check if a rule has already been applied to a node
    pub fn has_rule_applied(&self, node_id: NodeId, rule: ExpansionRule) -> bool {
        self.applied_rules.contains(&(node_id, rule))
    }

    /// Mark a rule as applied to a node
    pub fn mark_rule_applied(&mut self, node_id: NodeId, rule: ExpansionRule) {
        self.applied_rules.insert((node_id, rule));
    }

    /// Add an expansion task to the pending queue
    pub fn add_task(&mut self, task: ExpansionTask) {
        if task.depth <= self.max_depth {
            self.pending_expansions.push_back(task);
        }
    }

    /// Get the next pending task
    pub fn next_task(&mut self) -> Option<ExpansionTask> {
        self.pending_expansions.pop_front()
    }

    /// Check if there are pending tasks
    pub fn has_pending_tasks(&self) -> bool {
        !self.pending_expansions.is_empty()
    }

    /// Get the number of pending tasks
    pub fn pending_task_count(&self) -> usize {
        self.pending_expansions.len()
    }

    /// Create a new branch point
    pub fn create_branch_point(
        &mut self,
        node_id: NodeId,
        branching_task: ExpansionTask,
        branches: Vec<Branch>,
    ) {
        let branch_point = BranchPoint {
            node_id,
            branching_task,
            branches,
            selected_branch: 0,
        };
        self.branch_points.push(branch_point);
    }

    /// Get the current branch point (if any)
    pub fn current_branch_point(&self) -> Option<&BranchPoint> {
        self.branch_points.last()
    }

    /// Get the current branch (if any)
    pub fn current_branch(&self) -> Option<&Branch> {
        self.current_branch_point()
            .and_then(|bp| bp.branches.get(bp.selected_branch))
    }

    /// Check if we can backtrack to explore alternative branches
    pub fn can_backtrack(&self) -> bool {
        self.branch_points
            .iter()
            .any(|bp| bp.selected_branch + 1 < bp.branches.len())
    }

    /// Backtrack to the next available branch
    pub fn backtrack(&mut self) -> bool {
        // Find the branch point that can be advanced
        for i in (0..self.branch_points.len()).rev() {
            if self.branch_points[i].selected_branch + 1 < self.branch_points[i].branches.len() {
                // Reset all branch points after this one
                self.branch_points.truncate(i + 1);

                // Move to the next branch
                self.branch_points[i].selected_branch += 1;

                // Add tasks from the new branch
                let tasks_to_add: Vec<_> = self
                    .current_branch()
                    .map(|branch| branch.tasks.clone())
                    .unwrap_or_default();

                for task in tasks_to_add {
                    self.add_task(task);
                }

                return true;
            }
        }
        false
    }

    /// Mark a node as processed
    pub fn mark_node_processed(&mut self, node_id: NodeId) {
        self.processed_nodes.insert(node_id);
    }

    /// Check if a node has been processed
    pub fn is_node_processed(&self, node_id: NodeId) -> bool {
        self.processed_nodes.contains(&node_id)
    }

    /// Set the current node and update depth
    pub fn set_current_node(&mut self, node_id: NodeId) {
        self.current_node = node_id;
        self.current_depth = self.current_depth.saturating_add(1);
    }

    /// Check if we've reached the maximum depth
    pub fn is_max_depth_reached(&self) -> bool {
        self.current_depth >= self.max_depth
    }

    /// Increment the expansion count
    pub fn increment_expansion_count(&mut self) {
        self.expansion_count += 1;
    }

    /// Get expansion statistics
    pub fn stats(&self) -> ExpansionStats {
        ExpansionStats {
            applied_rules_count: self.applied_rules.len(),
            pending_tasks_count: self.pending_expansions.len(),
            branch_points_count: self.branch_points.len(),
            processed_nodes_count: self.processed_nodes.len(),
            current_depth: self.current_depth,
            expansion_count: self.expansion_count,
        }
    }

    /// Reset the context for a new expansion phase
    pub fn reset(&mut self) {
        self.current_depth = 0;
        self.applied_rules.clear();
        self.pending_expansions.clear();
        self.branch_points.clear();
        self.expansion_count = 0;
        // Keep processed_nodes and max_depth
    }

    /// Get a summary of the current state
    pub fn summary(&self) -> String {
        format!(
            "Context: node={:?}, depth={}, pending={}, branches={}, expansions={}",
            self.current_node,
            self.current_depth,
            self.pending_expansions.len(),
            self.branch_points.len(),
            self.expansion_count
        )
    }
}

/// Expansion statistics
#[derive(Debug, Clone, Default)]
pub struct ExpansionStats {
    /// Number of rules applied
    pub applied_rules_count: usize,
    /// Number of pending tasks
    pub pending_tasks_count: usize,
    /// Number of branch points
    pub branch_points_count: usize,
    /// Number of processed nodes
    pub processed_nodes_count: usize,
    /// Current expansion depth
    pub current_depth: u32,
    /// Total number of expansions performed
    pub expansion_count: u32,
}

impl Branch {
    /// Create a new branch
    pub fn new(id: usize, tasks: Vec<ExpansionTask>, description: String) -> Self {
        Self {
            id,
            tasks,
            description,
        }
    }

    /// Create a simple branch with a single task
    pub fn simple(id: usize, task: ExpansionTask, description: String) -> Self {
        Self {
            id,
            tasks: vec![task],
            description,
        }
    }
}

impl BranchPoint {
    /// Create a new branch point
    pub fn new(node_id: NodeId, branching_task: ExpansionTask, branches: Vec<Branch>) -> Self {
        Self {
            node_id,
            branching_task,
            branches,
            selected_branch: 0,
        }
    }

    /// Get the currently selected branch
    pub fn current_branch(&self) -> Option<&Branch> {
        self.branches.get(self.selected_branch)
    }

    /// Check if there are unexplored branches
    pub fn has_unexplored_branches(&self) -> bool {
        self.selected_branch + 1 < self.branches.len()
    }

    /// Get the total number of branches
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Get a summary for debugging
    pub fn summary(&self) -> String {
        format!(
            "Branch at node {:?}: {}/{} branches explored",
            self.node_id,
            self.selected_branch + 1,
            self.branches.len()
        )
    }
}
