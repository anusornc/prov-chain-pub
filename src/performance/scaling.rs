//! Horizontal Scaling Module
//!
//! This module provides horizontal scaling capabilities for ProvChain,
//! including load balancing, sharding strategies, and auto-scaling features.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Node configuration for horizontal scaling
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_id: String,
    pub address: String,
    pub port: u16,
    pub capacity: u32,
    pub current_load: u32,
    pub is_active: bool,
}

impl NodeConfig {
    pub fn new(node_id: String, address: String, port: u16, capacity: u32) -> Self {
        Self {
            node_id,
            address,
            port,
            capacity,
            current_load: 0,
            is_active: true,
        }
    }

    pub fn load_percentage(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.current_load as f64 / self.capacity as f64) * 100.0
        }
    }

    pub fn available_capacity(&self) -> u32 {
        self.capacity.saturating_sub(self.current_load)
    }
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections/load
    LeastLoad,
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    /// Hash-based distribution for consistent routing
    ConsistentHash,
}

/// Sharding strategy for data distribution
#[derive(Debug, Clone)]
pub enum ShardingStrategy {
    /// Hash-based sharding
    HashBased,
    /// Range-based sharding
    RangeBased,
    /// Directory-based sharding
    DirectoryBased,
    /// Composite sharding (combination of strategies)
    Composite,
}

/// Auto-scaling configuration
#[derive(Debug, Clone)]
pub struct AutoScalingConfig {
    /// Minimum number of nodes
    pub min_nodes: usize,
    /// Maximum number of nodes
    pub max_nodes: usize,
    /// CPU threshold for scaling up (percentage)
    pub scale_up_cpu_threshold: f64,
    /// CPU threshold for scaling down (percentage)
    pub scale_down_cpu_threshold: f64,
    /// Memory threshold for scaling up (percentage)
    pub scale_up_memory_threshold: f64,
    /// Memory threshold for scaling down (percentage)
    pub scale_down_memory_threshold: f64,
    /// Cooldown period between scaling operations
    pub cooldown_period: Duration,
    /// Number of consecutive threshold breaches required
    pub threshold_breach_count: u32,
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            min_nodes: 2,
            max_nodes: 10,
            scale_up_cpu_threshold: 80.0,
            scale_down_cpu_threshold: 30.0,
            scale_up_memory_threshold: 85.0,
            scale_down_memory_threshold: 40.0,
            cooldown_period: Duration::from_secs(300), // 5 minutes
            threshold_breach_count: 3,
        }
    }
}

/// Horizontal scaling manager
pub struct HorizontalScaler {
    nodes: HashMap<String, NodeConfig>,
    load_balancing_strategy: LoadBalancingStrategy,
    sharding_strategy: ShardingStrategy,
    auto_scaling_config: AutoScalingConfig,
    round_robin_index: usize,
    last_scaling_action: Option<Instant>,
    threshold_breach_counts: HashMap<String, u32>,
}

impl HorizontalScaler {
    /// Create a new horizontal scaler
    pub fn new(
        load_balancing_strategy: LoadBalancingStrategy,
        sharding_strategy: ShardingStrategy,
        auto_scaling_config: AutoScalingConfig,
    ) -> Self {
        Self {
            nodes: HashMap::new(),
            load_balancing_strategy,
            sharding_strategy,
            auto_scaling_config,
            round_robin_index: 0,
            last_scaling_action: None,
            threshold_breach_counts: HashMap::new(),
        }
    }

    /// Add a node to the cluster
    pub fn add_node(&mut self, node: NodeConfig) {
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// Remove a node from the cluster
    pub fn remove_node(&mut self, node_id: &str) -> Option<NodeConfig> {
        self.nodes.remove(node_id)
    }

    /// Get the best node for a new request based on load balancing strategy
    pub fn select_node(&mut self, request_key: Option<&str>) -> Option<&NodeConfig> {
        let active_nodes: Vec<_> = self
            .nodes
            .values()
            .filter(|node| node.is_active && node.available_capacity() > 0)
            .collect();

        if active_nodes.is_empty() {
            return None;
        }

        match self.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                let node = active_nodes[self.round_robin_index % active_nodes.len()];
                self.round_robin_index += 1;
                Some(node)
            }
            LoadBalancingStrategy::LeastLoad => active_nodes
                .iter()
                .min_by_key(|node| node.current_load)
                .copied(),
            LoadBalancingStrategy::WeightedRoundRobin => {
                // Select based on available capacity
                active_nodes
                    .iter()
                    .max_by_key(|node| node.available_capacity())
                    .copied()
            }
            LoadBalancingStrategy::ConsistentHash => {
                if let Some(key) = request_key {
                    let hash = self.hash_key(key);
                    let index = hash % active_nodes.len();
                    Some(active_nodes[index])
                } else {
                    // Fallback to round-robin if no key provided
                    let node = active_nodes[self.round_robin_index % active_nodes.len()];
                    self.round_robin_index += 1;
                    Some(node)
                }
            }
        }
    }

    /// Determine which shard a piece of data belongs to
    pub fn determine_shard(&self, data_key: &str, num_shards: usize) -> usize {
        match self.sharding_strategy {
            ShardingStrategy::HashBased => {
                let hash = self.hash_key(data_key);
                hash % num_shards
            }
            ShardingStrategy::RangeBased => {
                // Simple range-based sharding (alphabetical)
                let first_char = data_key.chars().next().unwrap_or('a') as u32;
                let range_size = (u32::from('z') - u32::from('a') + 1) / num_shards as u32;
                let shard = ((first_char - u32::from('a')) / range_size) as usize;
                shard.min(num_shards - 1)
            }
            ShardingStrategy::DirectoryBased => {
                // Use a simple modulo for directory-based sharding
                self.hash_key(data_key) % num_shards
            }
            ShardingStrategy::Composite => {
                // Combine hash and range-based strategies
                let hash_shard = self.hash_key(data_key) % num_shards;
                let range_shard = {
                    let first_char = data_key.chars().next().unwrap_or('a') as u32;
                    let range_size = (u32::from('z') - u32::from('a') + 1) / num_shards as u32;
                    ((first_char - u32::from('a')) / range_size) as usize
                };
                (hash_shard + range_shard) % num_shards
            }
        }
    }

    /// Update node load
    pub fn update_node_load(&mut self, node_id: &str, new_load: u32) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.current_load = new_load;
        }
    }

    /// Check if auto-scaling is needed
    pub fn check_auto_scaling(&mut self, metrics: &ClusterMetrics) -> Option<ScalingAction> {
        // Check cooldown period
        if let Some(last_action) = self.last_scaling_action {
            if last_action.elapsed() < self.auto_scaling_config.cooldown_period {
                return None;
            }
        }

        let active_node_count = self.nodes.values().filter(|n| n.is_active).count();

        // Check scale-up conditions
        if active_node_count < self.auto_scaling_config.max_nodes {
            if metrics.avg_cpu_usage > self.auto_scaling_config.scale_up_cpu_threshold
                || metrics.avg_memory_usage > self.auto_scaling_config.scale_up_memory_threshold
            {
                let breach_count = self
                    .threshold_breach_counts
                    .entry("scale_up".to_string())
                    .or_insert(0);
                *breach_count += 1;

                if *breach_count >= self.auto_scaling_config.threshold_breach_count {
                    self.threshold_breach_counts.clear();
                    self.last_scaling_action = Some(Instant::now());
                    return Some(ScalingAction::ScaleUp);
                }
            } else {
                self.threshold_breach_counts.remove("scale_up");
            }
        }

        // Check scale-down conditions
        if active_node_count > self.auto_scaling_config.min_nodes {
            if metrics.avg_cpu_usage < self.auto_scaling_config.scale_down_cpu_threshold
                && metrics.avg_memory_usage < self.auto_scaling_config.scale_down_memory_threshold
            {
                let breach_count = self
                    .threshold_breach_counts
                    .entry("scale_down".to_string())
                    .or_insert(0);
                *breach_count += 1;

                if *breach_count >= self.auto_scaling_config.threshold_breach_count {
                    self.threshold_breach_counts.clear();
                    self.last_scaling_action = Some(Instant::now());
                    return Some(ScalingAction::ScaleDown);
                }
            } else {
                self.threshold_breach_counts.remove("scale_down");
            }
        }

        None
    }

    /// Get cluster statistics
    pub fn get_cluster_stats(&self) -> ClusterStats {
        let active_nodes: Vec<_> = self.nodes.values().filter(|n| n.is_active).collect();
        let total_capacity: u32 = active_nodes.iter().map(|n| n.capacity).sum();
        let total_load: u32 = active_nodes.iter().map(|n| n.current_load).sum();
        let avg_load = if !active_nodes.is_empty() {
            total_load as f64 / active_nodes.len() as f64
        } else {
            0.0
        };

        ClusterStats {
            total_nodes: self.nodes.len(),
            active_nodes: active_nodes.len(),
            total_capacity,
            total_load,
            avg_load_percentage: if total_capacity > 0 {
                (total_load as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
            avg_load_per_node: avg_load,
            load_balancing_strategy: self.load_balancing_strategy.clone(),
            sharding_strategy: self.sharding_strategy.clone(),
        }
    }

    /// Rebalance load across nodes
    pub fn rebalance_load(&mut self) -> Vec<RebalanceAction> {
        let mut actions = Vec::new();
        let active_nodes: Vec<_> = self.nodes.values().collect();

        if active_nodes.len() < 2 {
            return actions;
        }

        let total_load: u32 = active_nodes.iter().map(|n| n.current_load).sum();
        let avg_load = total_load / active_nodes.len() as u32;

        // Find overloaded and underloaded nodes
        let overloaded: Vec<_> = active_nodes
            .iter()
            .filter(|n| n.current_load > avg_load + (avg_load / 4)) // 25% above average
            .collect();

        let underloaded: Vec<_> = active_nodes
            .iter()
            .filter(|n| n.current_load < avg_load - (avg_load / 4)) // 25% below average
            .collect();

        // Create rebalance actions
        for (overloaded_node, underloaded_node) in overloaded.iter().zip(underloaded.iter()) {
            let excess_load = overloaded_node.current_load - avg_load;
            let available_capacity = underloaded_node.available_capacity();
            let transfer_amount = excess_load.min(available_capacity).min(excess_load / 2);

            if transfer_amount > 0 {
                actions.push(RebalanceAction {
                    from_node: overloaded_node.node_id.clone(),
                    to_node: underloaded_node.node_id.clone(),
                    load_amount: transfer_amount,
                });
            }
        }

        actions
    }

    /// Hash function for consistent hashing and sharding
    fn hash_key(&self, key: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&NodeConfig> {
        self.nodes.get(node_id)
    }

    /// Get all active nodes
    pub fn get_active_nodes(&self) -> Vec<&NodeConfig> {
        self.nodes.values().filter(|n| n.is_active).collect()
    }

    /// Set node active status
    pub fn set_node_active(&mut self, node_id: &str, active: bool) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.is_active = active;
        }
    }
}

/// Cluster performance metrics
#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub avg_cpu_usage: f64,
    pub avg_memory_usage: f64,
    pub avg_response_time: Duration,
    pub total_requests: u64,
    pub error_rate: f64,
}

/// Scaling actions
#[derive(Debug, Clone)]
pub enum ScalingAction {
    ScaleUp,
    ScaleDown,
    NoAction,
}

/// Cluster statistics
#[derive(Debug, Clone)]
pub struct ClusterStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_capacity: u32,
    pub total_load: u32,
    pub avg_load_percentage: f64,
    pub avg_load_per_node: f64,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub sharding_strategy: ShardingStrategy,
}

impl ClusterStats {
    pub fn print_summary(&self) {
        println!("\n=== Cluster Statistics ===");
        println!("Total nodes: {}", self.total_nodes);
        println!("Active nodes: {}", self.active_nodes);
        println!("Total capacity: {}", self.total_capacity);
        println!("Total load: {}", self.total_load);
        println!("Average load: {:.2}%", self.avg_load_percentage);
        println!("Average load per node: {:.2}", self.avg_load_per_node);
        println!("Load balancing: {:?}", self.load_balancing_strategy);
        println!("Sharding strategy: {:?}", self.sharding_strategy);
        println!("==========================\n");
    }
}

/// Load rebalancing action
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub from_node: String,
    pub to_node: String,
    pub load_amount: u32,
}

impl RebalanceAction {
    pub fn print_summary(&self) {
        println!(
            "Rebalance: {} units from {} to {}",
            self.load_amount, self.from_node, self.to_node
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_config() {
        let node = NodeConfig::new("node1".to_string(), "127.0.0.1".to_string(), 8080, 100);

        assert_eq!(node.node_id, "node1");
        assert_eq!(node.capacity, 100);
        assert_eq!(node.current_load, 0);
        assert_eq!(node.load_percentage(), 0.0);
        assert_eq!(node.available_capacity(), 100);
        assert!(node.is_active);
    }

    #[test]
    fn test_node_load_calculation() {
        let mut node = NodeConfig::new("node1".to_string(), "127.0.0.1".to_string(), 8080, 100);
        node.current_load = 75;

        assert_eq!(node.load_percentage(), 75.0);
        assert_eq!(node.available_capacity(), 25);
    }

    #[test]
    fn test_horizontal_scaler_creation() {
        let scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        let stats = scaler.get_cluster_stats();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.active_nodes, 0);
    }

    #[test]
    fn test_add_remove_nodes() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        let node1 = NodeConfig::new("node1".to_string(), "127.0.0.1".to_string(), 8080, 100);
        let node2 = NodeConfig::new("node2".to_string(), "127.0.0.1".to_string(), 8081, 100);

        scaler.add_node(node1);
        scaler.add_node(node2);

        let stats = scaler.get_cluster_stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.active_nodes, 2);

        let removed = scaler.remove_node("node1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().node_id, "node1");

        let stats = scaler.get_cluster_stats();
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_round_robin_load_balancing() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        scaler.add_node(NodeConfig::new(
            "node1".to_string(),
            "127.0.0.1".to_string(),
            8080,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node2".to_string(),
            "127.0.0.1".to_string(),
            8081,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node3".to_string(),
            "127.0.0.1".to_string(),
            8082,
            100,
        ));

        let node1_id = scaler.select_node(None).unwrap().node_id.clone();
        let node2_id = scaler.select_node(None).unwrap().node_id.clone();
        let node3_id = scaler.select_node(None).unwrap().node_id.clone();
        let node4_id = scaler.select_node(None).unwrap().node_id.clone(); // Should wrap around

        // Should cycle through nodes
        assert_ne!(node1_id, node2_id);
        assert_ne!(node2_id, node3_id);
        assert_eq!(node1_id, node4_id); // Wrapped around
    }

    #[test]
    fn test_least_load_balancing() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::LeastLoad,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        scaler.add_node(NodeConfig::new(
            "node1".to_string(),
            "127.0.0.1".to_string(),
            8080,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node2".to_string(),
            "127.0.0.1".to_string(),
            8081,
            100,
        ));

        // Update loads
        scaler.update_node_load("node1", 50);
        scaler.update_node_load("node2", 25);

        let selected = scaler.select_node(None).unwrap();
        assert_eq!(selected.node_id, "node2"); // Should select node with least load
    }

    #[test]
    fn test_hash_based_sharding() {
        let scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        let shard1 = scaler.determine_shard("key1", 4);
        let shard2 = scaler.determine_shard("key2", 4);
        let shard3 = scaler.determine_shard("key1", 4); // Same key should go to same shard

        assert!(shard1 < 4);
        assert!(shard2 < 4);
        assert_eq!(shard1, shard3); // Consistent hashing
    }

    #[test]
    fn test_range_based_sharding() {
        let scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::RangeBased,
            AutoScalingConfig::default(),
        );

        let shard_a = scaler.determine_shard("apple", 4);
        let shard_m = scaler.determine_shard("mango", 4);
        let shard_z = scaler.determine_shard("zebra", 4);

        assert!(shard_a < 4);
        assert!(shard_m < 4);
        assert!(shard_z < 4);

        // Should generally distribute alphabetically
        assert!(shard_a <= shard_m);
        assert!(shard_m <= shard_z);
    }

    #[test]
    fn test_auto_scaling_config() {
        let config = AutoScalingConfig::default();

        assert_eq!(config.min_nodes, 2);
        assert_eq!(config.max_nodes, 10);
        assert_eq!(config.scale_up_cpu_threshold, 80.0);
        assert_eq!(config.scale_down_cpu_threshold, 30.0);
    }

    #[test]
    fn test_cluster_stats() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        scaler.add_node(NodeConfig::new(
            "node1".to_string(),
            "127.0.0.1".to_string(),
            8080,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node2".to_string(),
            "127.0.0.1".to_string(),
            8081,
            100,
        ));

        scaler.update_node_load("node1", 60);
        scaler.update_node_load("node2", 40);

        let stats = scaler.get_cluster_stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.active_nodes, 2);
        assert_eq!(stats.total_capacity, 200);
        assert_eq!(stats.total_load, 100);
        assert_eq!(stats.avg_load_percentage, 50.0);
    }

    #[test]
    fn test_rebalance_load() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::RoundRobin,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        scaler.add_node(NodeConfig::new(
            "node1".to_string(),
            "127.0.0.1".to_string(),
            8080,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node2".to_string(),
            "127.0.0.1".to_string(),
            8081,
            100,
        ));

        // Create imbalanced load
        scaler.update_node_load("node1", 80);
        scaler.update_node_load("node2", 20);

        let actions = scaler.rebalance_load();
        assert!(!actions.is_empty());

        let action = &actions[0];
        assert_eq!(action.from_node, "node1");
        assert_eq!(action.to_node, "node2");
        assert!(action.load_amount > 0);
    }

    #[test]
    fn test_consistent_hash_load_balancing() {
        let mut scaler = HorizontalScaler::new(
            LoadBalancingStrategy::ConsistentHash,
            ShardingStrategy::HashBased,
            AutoScalingConfig::default(),
        );

        scaler.add_node(NodeConfig::new(
            "node1".to_string(),
            "127.0.0.1".to_string(),
            8080,
            100,
        ));
        scaler.add_node(NodeConfig::new(
            "node2".to_string(),
            "127.0.0.1".to_string(),
            8081,
            100,
        ));

        let node1_id = scaler.select_node(Some("key1")).unwrap().node_id.clone();
        let node2_id = scaler.select_node(Some("key1")).unwrap().node_id.clone(); // Same key
        let _node3_id = scaler.select_node(Some("key2")).unwrap().node_id.clone(); // Different key

        assert_eq!(node1_id, node2_id); // Same key should go to same node
                                        // Different keys may or may not go to different nodes (depends on hash)
    }
}
