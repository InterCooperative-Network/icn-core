//! PN-Counter (Increment/Decrement Counter) CRDT implementation.
//!
//! A PN-Counter combines two G-Counters (one for increments, one for decrements)
//! to provide a counter that can both increase and decrease while remaining
//! conflict-free. Perfect for mana balances, reputation changes, or any value
//! that can go up or down.

use crate::{NodeId, VectorClock, CRDT, CRDTError, CRDTResult, CausalCRDT, GCounter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A counter that supports both increment and decrement operations.
/// 
/// Internally maintains two G-Counters: one tracking total increments
/// and another tracking total decrements. The final value is the difference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounter {
    /// Unique identifier for this CRDT instance.
    id: String,
    /// G-Counter tracking increments (positive changes).
    increment_counter: GCounter,
    /// G-Counter tracking decrements (negative changes).
    decrement_counter: GCounter,
    /// Vector clock for causality tracking.
    vector_clock: VectorClock,
}

/// Operations that can be applied to a PN-Counter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PNCounterOperation {
    /// Increment the counter by the specified amount on the given node.
    Increment { node_id: NodeId, amount: u64 },
    /// Decrement the counter by the specified amount on the given node.
    Decrement { node_id: NodeId, amount: u64 },
}

impl PNCounter {
    /// Create a new PN-Counter with the given ID.
    pub fn new(id: String) -> Self {
        let increment_id = format!("{}_inc", id);
        let decrement_id = format!("{}_dec", id);
        
        Self {
            id: id.clone(),
            increment_counter: GCounter::new(increment_id),
            decrement_counter: GCounter::new(decrement_id),
            vector_clock: VectorClock::new(),
        }
    }
    
    /// Create a new PN-Counter for a specific node with an initial value.
    pub fn with_initial_value(id: String, node_id: NodeId, initial_value: i64) -> Self {
        let mut counter = Self::new(id);
        
        if initial_value > 0 {
            counter.increment(&node_id, initial_value as u64).unwrap();
        } else if initial_value < 0 {
            counter.decrement(&node_id, (-initial_value) as u64).unwrap();
        }
        
        counter
    }
    
    /// Get the current value of the counter (increments - decrements).
    pub fn get_total(&self) -> i64 {
        let total_increments = self.increment_counter.get_total() as i64;
        let total_decrements = self.decrement_counter.get_total() as i64;
        total_increments - total_decrements
    }
    
    /// Get the net contribution from a specific node.
    pub fn get_node_value(&self, node_id: &NodeId) -> i64 {
        let increments = self.increment_counter.get_node_value(node_id) as i64;
        let decrements = self.decrement_counter.get_node_value(node_id) as i64;
        increments - decrements
    }
    
    /// Get the total increments from a specific node.
    pub fn get_node_increments(&self, node_id: &NodeId) -> u64 {
        self.increment_counter.get_node_value(node_id)
    }
    
    /// Get the total decrements from a specific node.
    pub fn get_node_decrements(&self, node_id: &NodeId) -> u64 {
        self.decrement_counter.get_node_value(node_id)
    }
    
    /// Increment the counter by the specified amount for the given node.
    pub fn increment(&mut self, node_id: &NodeId, amount: u64) -> CRDTResult<()> {
        if amount == 0 {
            return Err(CRDTError::InvalidOperation("Increment amount must be greater than 0".to_string()));
        }
        
        self.increment_counter.increment(node_id, amount)?;
        self.vector_clock.increment(node_id);
        
        Ok(())
    }
    
    /// Decrement the counter by the specified amount for the given node.
    pub fn decrement(&mut self, node_id: &NodeId, amount: u64) -> CRDTResult<()> {
        if amount == 0 {
            return Err(CRDTError::InvalidOperation("Decrement amount must be greater than 0".to_string()));
        }
        
        self.decrement_counter.increment(node_id, amount)?;
        self.vector_clock.increment(node_id);
        
        Ok(())
    }
    
    /// Add a delta value (can be positive or negative).
    pub fn add(&mut self, node_id: &NodeId, delta: i64) -> CRDTResult<()> {
        if delta > 0 {
            self.increment(node_id, delta as u64)
        } else if delta < 0 {
            self.decrement(node_id, (-delta) as u64)
        } else {
            Err(CRDTError::InvalidOperation("Delta cannot be zero".to_string()))
        }
    }
    
    /// Get all nodes that have contributed to this counter.
    pub fn contributing_nodes(&self) -> Vec<NodeId> {
        let mut nodes = std::collections::HashSet::new();
        nodes.extend(self.increment_counter.contributing_nodes());
        nodes.extend(self.decrement_counter.contributing_nodes());
        nodes.into_iter().collect()
    }
    
    /// Get a breakdown of net contributions per node.
    pub fn node_contributions(&self) -> HashMap<NodeId, i64> {
        let mut contributions = HashMap::new();
        
        for node_id in self.contributing_nodes() {
            let net_value = self.get_node_value(&node_id);
            if net_value != 0 {
                contributions.insert(node_id, net_value);
            }
        }
        
        contributions
    }
    
    /// Get detailed breakdown showing increments and decrements separately.
    pub fn detailed_contributions(&self) -> HashMap<NodeId, (u64, u64, i64)> {
        let mut contributions = HashMap::new();
        
        for node_id in self.contributing_nodes() {
            let increments = self.get_node_increments(&node_id);
            let decrements = self.get_node_decrements(&node_id);
            let net = increments as i64 - decrements as i64;
            
            if increments > 0 || decrements > 0 {
                contributions.insert(node_id, (increments, decrements, net));
            }
        }
        
        contributions
    }
    
    /// Check if this counter has any contributions from the given node.
    pub fn has_node_contribution(&self, node_id: &NodeId) -> bool {
        self.increment_counter.has_node_contribution(node_id) ||
        self.decrement_counter.has_node_contribution(node_id)
    }
    
    /// Get the total magnitude of all operations (useful for activity metrics).
    pub fn total_activity(&self) -> u64 {
        self.increment_counter.get_total() + self.decrement_counter.get_total()
    }
    
    /// Get statistics about this counter.
    pub fn stats(&self) -> PNCounterStats {
        let contributions = self.detailed_contributions();
        
        let (positive_nodes, negative_nodes, zero_nodes) = contributions.values()
            .fold((0, 0, 0), |(pos, neg, zero), &(_, _, net)| {
                if net > 0 {
                    (pos + 1, neg, zero)
                } else if net < 0 {
                    (pos, neg + 1, zero)
                } else {
                    (pos, neg, zero + 1)
                }
            });
            
        let max_contribution = contributions.values()
            .map(|(_, _, net)| *net)
            .max()
            .unwrap_or(0);
            
        let min_contribution = contributions.values()
            .map(|(_, _, net)| *net)
            .min()
            .unwrap_or(0);
        
        PNCounterStats {
            current_value: self.value(),
            total_increments: self.increment_counter.get_total(),
            total_decrements: self.decrement_counter.get_total(),
            total_activity: self.total_activity(),
            contributing_nodes: contributions.len() as u64,
            positive_contributors: positive_nodes,
            negative_contributors: negative_nodes,
            zero_contributors: zero_nodes,
            max_node_contribution: max_contribution,
            min_node_contribution: min_contribution,
        }
    }
}

impl CRDT for PNCounter {
    type Operation = PNCounterOperation;
    
    fn merge(&mut self, other: &Self) {
        self.increment_counter.merge(&other.increment_counter);
        self.decrement_counter.merge(&other.decrement_counter);
        self.vector_clock.merge(&other.vector_clock);
    }
    
    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError> {
        match op {
            PNCounterOperation::Increment { node_id, amount } => {
                self.increment(&node_id, amount)
            },
            PNCounterOperation::Decrement { node_id, amount } => {
                self.decrement(&node_id, amount)
            }
        }
    }
    
    fn value(&self) -> serde_json::Value {
        serde_json::json!({
            "current_value": self.get_total(),
            "total_increments": self.increment_counter.get_total(),
            "total_decrements": self.decrement_counter.get_total(),
            "contributions": self.node_contributions(),
            "detailed_contributions": self.detailed_contributions()
        })
    }
    
    fn crdt_id(&self) -> String {
        self.id.clone()
    }
}

impl CausalCRDT for PNCounter {
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

impl PartialEq for PNCounter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && 
        self.increment_counter == other.increment_counter &&
        self.decrement_counter == other.decrement_counter
    }
}

impl Eq for PNCounter {}

/// Statistics about a PN-Counter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PNCounterStats {
    /// The current net value (increments - decrements).
    pub current_value: i64,
    /// Total of all increment operations.
    pub total_increments: u64,
    /// Total of all decrement operations.
    pub total_decrements: u64,
    /// Total activity (increments + decrements).
    pub total_activity: u64,
    /// Number of nodes that have contributed.
    pub contributing_nodes: u64,
    /// Number of nodes with positive net contributions.
    pub positive_contributors: u64,
    /// Number of nodes with negative net contributions.
    pub negative_contributors: u64,
    /// Number of nodes with zero net contributions.
    pub zero_contributors: u64,
    /// Highest net contribution from any node.
    pub max_node_contribution: i64,
    /// Lowest net contribution from any node.
    pub min_node_contribution: i64,
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
    fn test_pncounter_creation() {
        let counter = PNCounter::new("test_counter".to_string());
        assert_eq!(counter.get_total(), 0);
        assert_eq!(counter.crdt_id(), "test_counter");
        assert_eq!(counter.contributing_nodes().len(), 0);
        assert_eq!(counter.total_activity(), 0);
    }
    
    #[test]
    fn test_pncounter_with_initial_value() {
        let counter_pos = PNCounter::with_initial_value("test".to_string(), node_a(), 42);
        assert_eq!(counter_pos.value(), 42);
        assert_eq!(counter_pos.get_node_value(&node_a()), 42);
        
        let counter_neg = PNCounter::with_initial_value("test".to_string(), node_b(), -30);
        assert_eq!(counter_neg.value(), -30);
        assert_eq!(counter_neg.get_node_value(&node_b()), -30);
        
        let counter_zero = PNCounter::with_initial_value("test".to_string(), node_c(), 0);
        assert_eq!(counter_zero.value(), 0);
    }
    
    #[test]
    fn test_pncounter_increment() {
        let mut counter = PNCounter::new("test".to_string());
        
        counter.increment(&node_a(), 10).unwrap();
        assert_eq!(counter.get_total(), 10);
        assert_eq!(counter.get_node_value(&node_a()), 10);
        assert_eq!(counter.get_node_increments(&node_a()), 10);
        assert_eq!(counter.get_node_decrements(&node_a()), 0);
        
        counter.increment(&node_a(), 5).unwrap();
        assert_eq!(counter.get_total(), 15);
        assert_eq!(counter.get_node_value(&node_a()), 15);
        
        counter.increment(&node_b(), 20).unwrap();
        assert_eq!(counter.get_total(), 35);
    }
    
    #[test]
    fn test_pncounter_decrement() {
        let mut counter = PNCounter::new("test".to_string());
        
        counter.decrement(&node_a(), 10).unwrap();
        assert_eq!(counter.get_total(), -10);
        assert_eq!(counter.get_node_value(&node_a()), -10);
        assert_eq!(counter.get_node_increments(&node_a()), 0);
        assert_eq!(counter.get_node_decrements(&node_a()), 10);
        
        counter.decrement(&node_a(), 5).unwrap();
        assert_eq!(counter.get_total(), -15);
        assert_eq!(counter.get_node_value(&node_a()), -15);
    }
    
    #[test]
    fn test_pncounter_mixed_operations() {
        let mut counter = PNCounter::new("test".to_string());
        
        counter.increment(&node_a(), 20).unwrap();
        counter.decrement(&node_a(), 8).unwrap();
        assert_eq!(counter.get_total(), 12);
        assert_eq!(counter.get_node_value(&node_a()), 12);
        
        counter.increment(&node_b(), 15).unwrap();
        counter.decrement(&node_b(), 25).unwrap();
        assert_eq!(counter.get_node_value(&node_b()), -10);
        assert_eq!(counter.get_total(), 2); // 12 + (-10)
    }
    
    #[test]
    fn test_pncounter_add_delta() {
        let mut counter = PNCounter::new("test".to_string());
        
        counter.add(&node_a(), 15).unwrap();
        assert_eq!(counter.get_total(), 15);
        
        counter.add(&node_a(), -8).unwrap();
        assert_eq!(counter.get_total(), 7);
        
        counter.add(&node_b(), -12).unwrap();
        assert_eq!(counter.get_total(), -5);
        
        // Zero delta should fail
        assert!(counter.add(&node_a(), 0).is_err());
    }
    
    #[test]
    fn test_pncounter_zero_operations_fail() {
        let mut counter = PNCounter::new("test".to_string());
        
        assert!(counter.increment(&node_a(), 0).is_err());
        assert!(counter.decrement(&node_a(), 0).is_err());
        assert_eq!(counter.get_total(), 0);
    }
    
    #[test]
    fn test_pncounter_merge() {
        let mut counter1 = PNCounter::new("test".to_string());
        counter1.increment(&node_a(), 20).unwrap();
        counter1.decrement(&node_a(), 5).unwrap();
        counter1.increment(&node_b(), 10).unwrap();
        
        let mut counter2 = PNCounter::new("test".to_string());
        counter2.increment(&node_a(), 15).unwrap(); // Less than counter1's increments
        counter2.decrement(&node_a(), 8).unwrap(); // More than counter1's decrements
        counter2.increment(&node_c(), 25).unwrap(); // New node
        
        counter1.merge(&counter2);
        
        // node_a: max(20, 15) - max(5, 8) = 20 - 8 = 12
        // node_b: 10 - 0 = 10  
        // node_c: 25 - 0 = 25
        // Total: 12 + 10 + 25 = 47
        assert_eq!(counter1.value(), 47);
        assert_eq!(counter1.get_node_value(&node_a()), 12);
        assert_eq!(counter1.get_node_value(&node_b()), 10);
        assert_eq!(counter1.get_node_value(&node_c()), 25);
    }
    
    #[test]
    fn test_pncounter_merge_idempotent() {
        let mut counter1 = PNCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        counter1.decrement(&node_b(), 5).unwrap();
        
        let counter2 = counter1.clone();
        let original_value = counter1.value();
        
        counter1.merge(&counter2);
        assert_eq!(counter1.value(), original_value);
    }
    
    #[test]
    fn test_pncounter_merge_commutative() {
        let mut counter1 = PNCounter::new("test".to_string());
        counter1.increment(&node_a(), 10).unwrap();
        counter1.decrement(&node_b(), 5).unwrap();
        
        let mut counter2 = PNCounter::new("test".to_string());
        counter2.increment(&node_b(), 15).unwrap();
        counter2.decrement(&node_c(), 8).unwrap();
        
        let mut counter1_copy = counter1.clone();
        let counter2_copy = counter2.clone();
        
        // counter1.merge(counter2)
        counter1.merge(&counter2);
        
        // counter2.merge(counter1_copy)
        counter2.merge(&counter1_copy);
        
        assert_eq!(counter1.value(), counter2.value());
        assert_eq!(counter1.node_contributions(), counter2.node_contributions());
    }
    
    #[test]
    fn test_pncounter_apply_operations() {
        let mut counter = PNCounter::new("test".to_string());
        
        let inc_op = PNCounterOperation::Increment {
            node_id: node_a(),
            amount: 25,
        };
        
        let dec_op = PNCounterOperation::Decrement {
            node_id: node_a(),
            amount: 8,
        };
        
        counter.apply_operation(inc_op).unwrap();
        assert_eq!(counter.get_total(), 25);
        
        counter.apply_operation(dec_op).unwrap();
        assert_eq!(counter.get_total(), 17);
    }
    
    #[test]
    fn test_pncounter_contributions() {
        let mut counter = PNCounter::new("test".to_string());
        counter.increment(&node_a(), 30).unwrap();
        counter.decrement(&node_a(), 10).unwrap();
        counter.increment(&node_b(), 5).unwrap();
        counter.decrement(&node_b(), 15).unwrap();
        counter.increment(&node_c(), 20).unwrap();
        
        let contributions = counter.node_contributions();
        assert_eq!(contributions.get(&node_a()).copied().unwrap(), 20);
        assert_eq!(contributions.get(&node_b()).copied().unwrap(), -10);
        assert_eq!(contributions.get(&node_c()).copied().unwrap(), 20);
        
        let detailed = counter.detailed_contributions();
        assert_eq!(detailed.get(&node_a()).copied().unwrap(), (30, 10, 20));
        assert_eq!(detailed.get(&node_b()).copied().unwrap(), (5, 15, -10));
        assert_eq!(detailed.get(&node_c()).copied().unwrap(), (20, 0, 20));
        
        assert!(counter.has_node_contribution(&node_a()));
        assert!(counter.has_node_contribution(&node_b()));
        assert!(counter.has_node_contribution(&node_c()));
    }
    
    #[test]
    fn test_pncounter_stats() {
        let mut counter = PNCounter::new("test".to_string());
        counter.increment(&node_a(), 40).unwrap();
        counter.decrement(&node_a(), 15).unwrap();
        counter.increment(&node_b(), 10).unwrap();
        counter.decrement(&node_b(), 20).unwrap();
        counter.increment(&node_c(), 30).unwrap();
        
        let stats = counter.stats();
        assert_eq!(stats.current_value, 45); // (40-15) + (10-20) + 30 = 25 + (-10) + 30
        assert_eq!(stats.total_increments, 80); // 40 + 10 + 30
        assert_eq!(stats.total_decrements, 35); // 15 + 20
        assert_eq!(stats.total_activity, 115); // 80 + 35
        assert_eq!(stats.contributing_nodes, 3);
        assert_eq!(stats.positive_contributors, 2); // node_a: +25, node_c: +30
        assert_eq!(stats.negative_contributors, 1); // node_b: -10
        assert_eq!(stats.zero_contributors, 0);
        assert_eq!(stats.max_node_contribution, 30);
        assert_eq!(stats.min_node_contribution, -10);
    }
    
    #[test]
    fn test_pncounter_total_activity() {
        let mut counter = PNCounter::new("test".to_string());
        counter.increment(&node_a(), 100).unwrap();
        counter.decrement(&node_a(), 30).unwrap();
        counter.increment(&node_b(), 50).unwrap();
        
        assert_eq!(counter.total_activity(), 180); // 100 + 30 + 50
        assert_eq!(counter.get_total(), 120); // (100 - 30) + 50
    }
    
    #[test]
    fn test_pncounter_value_json() {
        let mut counter = PNCounter::new("test".to_string());
        counter.increment(&node_a(), 25).unwrap();
        counter.decrement(&node_b(), 10).unwrap();
        
        let json_value = counter.get_total();
        assert_eq!(json_value["current_value"], 15);
        assert_eq!(json_value["total_increments"], 25);
        assert_eq!(json_value["total_decrements"], 10);
        
        let contributions = &json_value["contributions"];
        assert_eq!(contributions[node_a().as_str()], 25);
        assert_eq!(contributions[node_b().as_str()], -10);
    }
}