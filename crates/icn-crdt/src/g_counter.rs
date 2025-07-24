//! Grow-only Counter (G-Counter) CRDT implementation.
//!
//! A G-Counter is a state-based CRDT that represents a counter that can only increase.
//! It's perfect for tracking monotonic values like mana generation, total reputation earned,
//! or any other value that only grows over time.

use crate::{NodeId, VectorClock, CRDT, CRDTError, CRDTResult, CausalCRDT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A grow-only counter that can only be incremented.
/// 
/// The G-Counter maintains a separate counter for each node, ensuring that
/// increments from different nodes can be merged without conflicts. The total
/// value is the sum of all per-node counters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    /// Unique identifier for this CRDT instance.
    id: String,
    /// Per-node counters mapping node ID to that node's contribution.
    counters: HashMap<NodeId, u64>,
    /// Vector clock for causality tracking.
    vector_clock: VectorClock,
}

/// Operations that can be applied to a G-Counter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GCounterOperation {
    /// Increment the counter by the specified amount on the given node.
    Increment { node_id: NodeId, amount: u64 },
}

impl GCounter {
    /// Create a new G-Counter with the given ID.
    pub fn new(id: String) -> Self {
        Self {
            id,
            counters: HashMap::new(),
            vector_clock: VectorClock::new(),
        }
    }
    
    /// Create a new G-Counter for a specific node with an initial value.
    pub fn with_initial_value(id: String, node_id: NodeId, initial_value: u64) -> Self {
        let mut counters = HashMap::new();
        counters.insert(node_id.clone(), initial_value);
        
        let mut vector_clock = VectorClock::new();
        if initial_value > 0 {
            vector_clock.set(node_id, 1);
        }
        
        Self {
            id,
            counters,
            vector_clock,
        }
    }
    
    /// Get the current total value of the counter.
    pub fn get_total(&self) -> u64 {
        self.counters.values().sum()
    }
    
    /// Get the contribution from a specific node.
    pub fn get_node_value(&self, node_id: &NodeId) -> u64 {
        self.counters.get(node_id).copied().unwrap_or(0)
    }
    
    /// Increment the counter by the specified amount for the local node.
    pub fn increment(&mut self, node_id: &NodeId, amount: u64) -> CRDTResult<()> {
        if amount == 0 {
            return Err(CRDTError::InvalidOperation("Increment amount must be greater than 0".to_string()));
        }
        
        let current = self.get_node_value(node_id);
        self.counters.insert(node_id.clone(), current + amount);
        self.vector_clock.increment(node_id);
        
        Ok(())
    }
    
    /// Get all nodes that have contributed to this counter.
    pub fn contributing_nodes(&self) -> Vec<NodeId> {
        self.counters.keys().cloned().collect()
    }
    
    /// Get a breakdown of contributions per node.
    pub fn node_contributions(&self) -> HashMap<NodeId, u64> {
        self.counters.clone()
    }
    
    /// Check if this counter has any contributions from the given node.
    pub fn has_node_contribution(&self, node_id: &NodeId) -> bool {
        self.counters.contains_key(node_id)
    }
    
    /// Get statistics about this counter.
    pub fn stats(&self) -> GCounterStats {
        GCounterStats {
            total_value: self.get_total(),
            contributing_nodes: self.counters.len() as u64,
            max_node_contribution: self.counters.values().max().copied().unwrap_or(0),
            min_node_contribution: self.counters.values().min().copied().unwrap_or(0),
        }
    }
}

impl CRDT for GCounter {
    type Operation = GCounterOperation;
    
    fn merge(&mut self, other: &Self) {
        // For each node, take the maximum value (grows only)
        for (node_id, &other_value) in &other.counters {
            let current_value = self.get_node_value(node_id);
            if other_value > current_value {
                self.counters.insert(node_id.clone(), other_value);
            }
        }
        
        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);
    }
    
    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError> {
        match op {
            GCounterOperation::Increment { node_id, amount } => {
                self.increment(&node_id, amount)
            }
        }
    }
    
    fn value(&self) -> serde_json::Value {
        serde_json::json!({
            "total": self.get_total(),
            "contributions": self.node_contributions()
        })
    }
    
    fn crdt_id(&self) -> String {
        self.id.clone()
    }
}

impl CausalCRDT for GCounter {
    fn vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }
    
    fn advance_clock(&mut self, node_id: &NodeId) {
        self.vector_clock.increment(node_id);
    }
    
    fn has_seen(&self, vector_clock: &VectorClock) -> bool {
        for node_id in vector_clock.node_ids() {
            if self.vector_clock.get(&node_id) < vector_clock.get(&node_id) {
                return false;
            }
        }
        true
    }
}

impl PartialEq for GCounter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.counters == other.counters
    }
}

impl Eq for GCounter {}

/// Statistics about a G-Counter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounterStats {
    /// The total value across all nodes.
    pub total_value: u64,
    /// Number of nodes that have contributed.
    pub contributing_nodes: u64,
    /// Largest contribution from any single node.
    pub max_node_contribution: u64,
    /// Smallest contribution from any contributing node.
    pub min_node_contribution: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn node_a() -> NodeId {
        NodeId::new("node_a".to_string())
    }
    
    fn node_b() -> NodeId {
        NodeId::new("node_b".to_string())
    }
    
    fn node_c() -> NodeId {
        NodeId::new("node_c".to_string())
    }
    
    #[test]
    fn test_gcounter_creation() {
        let counter = GCounter::new("test_counter".to_string());
        assert_eq!(counter.get_total(), 0);
        assert_eq!(counter.crdt_id(), "test_counter");
        assert_eq!(counter.contributing_nodes().len(), 0);
    }
    
    #[test]
    fn test_gcounter_with_initial_value() {
        let counter = GCounter::with_initial_value("test".to_string(), node_a(), 42);
        assert_eq!(counter.get_total(), 42);
        assert_eq!(counter.get_node_value(&node_a()), 42);
        assert_eq!(counter.get_node_value(&node_b()), 0);
    }
    
    #[test]
    fn test_gcounter_increment() {
        let mut counter = GCounter::new("test".to_string());
        
        counter.increment(&node_a(), 10).unwrap();
        assert_eq!(counter.get_total(), 10);
        assert_eq!(counter.get_node_value(&node_a()), 10);
        
        counter.increment(&node_a(), 5).unwrap();
        assert_eq!(counter.get_total(), 15);
        assert_eq!(counter.get_node_value(&node_a()), 15);
        
        counter.increment(&node_b(), 20).unwrap();
        assert_eq!(counter.get_total(), 35);
        assert_eq!(counter.get_node_value(&node_a()), 15);
        assert_eq!(counter.get_node_value(&node_b()), 20);
    }
    
    #[test]
    fn test_gcounter_increment_zero_fails() {
        let mut counter = GCounter::new("test".to_string());
        let result = counter.increment(&node_a(), 0);
        assert!(result.is_err());
        assert_eq!(counter.get_total(), 0);
    }
    
    #[test]
    fn test_gcounter_merge() {
        let mut counter1 = GCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        counter1.increment(&node_b(), 5).unwrap();
        
        let mut counter2 = GCounter::new("test".to_string());
        counter2.increment(&node_a(), 8).unwrap(); // Lower than counter1
        counter2.increment(&node_b(), 15).unwrap(); // Higher than counter1
        counter2.increment(&node_c(), 12).unwrap(); // New node
        
        counter1.merge(&counter2);
        
        assert_eq!(counter1.get_total(), 37); // 10 + 15 + 12
        assert_eq!(counter1.get_node_value(&node_a()), 10); // Kept higher value
        assert_eq!(counter1.get_node_value(&node_b()), 15); // Took higher value
        assert_eq!(counter1.get_node_value(&node_c()), 12); // Added new node
    }
    
    #[test]
    fn test_gcounter_merge_idempotent() {
        let mut counter1 = GCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        
        let counter2 = counter1.clone();
        let original_value = counter1.get_total();
        
        counter1.merge(&counter2);
        assert_eq!(counter1.get_total(), original_value);
    }
    
    #[test]
    fn test_gcounter_merge_commutative() {
        let mut counter1 = GCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        
        let mut counter2 = GCounter::new("test".to_string());
        counter2.increment(&node_b(), 20).unwrap();
        
        let mut counter1_copy = counter1.clone();
        let counter2_copy = counter2.clone();
        
        // counter1.merge(counter2)
        counter1.merge(&counter2);
        
        // counter2.merge(counter1_copy)  
        counter2.merge(&counter1_copy);
        
        assert_eq!(counter1.get_total(), counter2.get_total());
        assert_eq!(counter1.node_contributions(), counter2.node_contributions());
    }
    
    #[test]
    fn test_gcounter_apply_operation() {
        let mut counter = GCounter::new("test".to_string());
        
        let op = GCounterOperation::Increment {
            node_id: node_a(),
            amount: 25,
        };
        
        counter.apply_operation(op).unwrap();
        assert_eq!(counter.get_total(), 25);
        assert_eq!(counter.get_node_value(&node_a()), 25);
    }
    
    #[test]
    fn test_gcounter_contributing_nodes() {
        let mut counter = GCounter::new("test".to_string());
        
        assert!(counter.contributing_nodes().is_empty());
        assert!(!counter.has_node_contribution(&node_a()));
        
        counter.increment(&node_a(), 10).unwrap();
        counter.increment(&node_c(), 5).unwrap();
        
        let nodes = counter.contributing_nodes();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&node_a()));
        assert!(nodes.contains(&node_c()));
        assert!(!nodes.contains(&node_b()));
        
        assert!(counter.has_node_contribution(&node_a()));
        assert!(!counter.has_node_contribution(&node_b()));
        assert!(counter.has_node_contribution(&node_c()));
    }
    
    #[test]
    fn test_gcounter_stats() {
        let mut counter = GCounter::new("test".to_string());
        counter.increment(&node_a(), 30).unwrap();
        counter.increment(&node_b(), 10).unwrap();
        counter.increment(&node_c(), 20).unwrap();
        
        let stats = counter.stats();
        assert_eq!(stats.total_value, 60);
        assert_eq!(stats.contributing_nodes, 3);
        assert_eq!(stats.max_node_contribution, 30);
        assert_eq!(stats.min_node_contribution, 10);
    }
    
    #[test]
    fn test_gcounter_value_json() {
        let mut counter = GCounter::new("test".to_string());
        counter.increment(&node_a(), 15).unwrap();
        counter.increment(&node_b(), 25).unwrap();
        
        let json_value = counter.value();
        assert_eq!(json_value["total"], 40);
        
        let contributions = &json_value["contributions"];
        assert_eq!(contributions[node_a().as_str()], 15);
        assert_eq!(contributions[node_b().as_str()], 25);
    }
    
    #[test]
    fn test_gcounter_vector_clock() {
        let mut counter = GCounter::new("test".to_string());
        assert_eq!(counter.vector_clock().get(&node_a()), 0);
        
        counter.increment(&node_a(), 10).unwrap();
        assert_eq!(counter.vector_clock().get(&node_a()), 1);
        
        counter.increment(&node_a(), 5).unwrap();
        assert_eq!(counter.vector_clock().get(&node_a()), 2);
        
        counter.increment(&node_b(), 8).unwrap();
        assert_eq!(counter.vector_clock().get(&node_a()), 2);
        assert_eq!(counter.vector_clock().get(&node_b()), 1);
    }
    
    #[test]
    fn test_gcounter_causality() {
        let mut counter1 = GCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        
        let mut counter2 = GCounter::new("test".to_string());
        counter2.increment(&node_b(), 20).unwrap();
        
        // counter1 hasn't seen counter2's changes
        assert!(!counter1.has_seen(counter2.vector_clock()));
        assert!(!counter2.has_seen(counter1.vector_clock()));
        
        // After merging, counter1 has seen all changes
        counter1.merge(&counter2);
        assert!(counter1.has_seen(counter2.vector_clock()));
    }
}