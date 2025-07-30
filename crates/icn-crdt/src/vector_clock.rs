//! Vector Clock implementation for tracking causality in distributed CRDT operations.
//!
//! Vector clocks enable CRDTs to determine the causal ordering of operations
//! across distributed nodes, which is essential for conflict-free merging.

use crate::{CRDTError, CRDTResult, NodeId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

/// A vector clock for tracking causality between distributed events.
///
/// Each node maintains a vector clock that maps node IDs to logical timestamps.
/// When a node performs an operation, it increments its own timestamp.
/// When receiving operations from other nodes, it updates its vector with
/// the maximum timestamp for each node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map from node ID to logical timestamp for that node.
    clocks: HashMap<NodeId, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock.
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Create a new vector clock with an initial timestamp for the given node.
    pub fn with_node(node_id: NodeId, timestamp: u64) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(node_id, timestamp);
        Self { clocks }
    }

    /// Get the current timestamp for a specific node.
    pub fn get(&self, node_id: &NodeId) -> u64 {
        self.clocks.get(node_id).copied().unwrap_or(0)
    }

    /// Increment the timestamp for the given node.
    pub fn increment(&mut self, node_id: &NodeId) {
        let current = self.get(node_id);
        self.clocks.insert(node_id.clone(), current + 1);
    }

    /// Set the timestamp for a specific node.
    pub fn set(&mut self, node_id: NodeId, timestamp: u64) {
        self.clocks.insert(node_id, timestamp);
    }

    /// Update this vector clock with another vector clock by taking the maximum
    /// timestamp for each node (merge operation).
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, &timestamp) in &other.clocks {
            let current = self.get(node_id);
            self.clocks.insert(node_id.clone(), current.max(timestamp));
        }
    }

    /// Compare this vector clock with another to determine causal ordering.
    ///
    /// Returns:
    /// - `Some(Ordering::Less)` if this clock happened-before other  
    /// - `Some(Ordering::Greater)` if other happened-before this
    /// - `Some(Ordering::Equal)` if they are equal
    /// - `None` if they are concurrent (no causal relationship)
    pub fn compare(&self, other: &VectorClock) -> Option<Ordering> {
        let all_nodes: std::collections::HashSet<_> =
            self.clocks.keys().chain(other.clocks.keys()).collect();

        let mut less_or_equal = true;
        let mut greater_or_equal = true;
        let mut has_strict_less = false;
        let mut has_strict_greater = false;

        for node_id in &all_nodes {
            let self_time = self.get(node_id);
            let other_time = other.get(node_id);

            match self_time.cmp(&other_time) {
                Ordering::Less => {
                    greater_or_equal = false;
                    has_strict_less = true;
                }
                Ordering::Greater => {
                    less_or_equal = false;
                    has_strict_greater = true;
                }
                Ordering::Equal => {
                    // Continue checking other nodes
                }
            }
        }

        if less_or_equal && greater_or_equal {
            Some(Ordering::Equal)
        } else if less_or_equal && has_strict_less {
            Some(Ordering::Less)
        } else if greater_or_equal && has_strict_greater {
            Some(Ordering::Greater)
        } else {
            None // Concurrent
        }
    }

    /// Check if this vector clock happened-before another vector clock.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), Some(Ordering::Less))
    }

    /// Check if this vector clock is concurrent with another (no causal relationship).
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        self.compare(other).is_none()
    }

    /// Get all node IDs tracked by this vector clock.
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.clocks.keys().cloned().collect()
    }

    /// Get the total number of events across all nodes.
    pub fn total_events(&self) -> u64 {
        self.clocks.values().sum()
    }

    /// Create a compact representation for network transmission.
    pub fn to_compact_bytes(&self) -> CRDTResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| {
            CRDTError::SerializationError(format!("Vector clock serialization failed: {e}"))
        })
    }

    /// Restore from compact representation.
    pub fn from_compact_bytes(bytes: &[u8]) -> CRDTResult<Self> {
        bincode::deserialize(bytes).map_err(|e| {
            CRDTError::SerializationError(format!("Vector clock deserialization failed: {e}"))
        })
    }

    /// Remove timestamps for nodes that haven't been seen in a while (garbage collection).
    pub fn gc_old_nodes(&mut self, keep_nodes: &[NodeId]) {
        let keep_set: std::collections::HashSet<_> = keep_nodes.iter().collect();
        self.clocks.retain(|node_id, _| keep_set.contains(node_id));
    }

    /// Get a deterministic hash of this vector clock for debugging.
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};

        // Sort entries for deterministic hashing
        let mut entries: Vec<_> = self.clocks.iter().collect();
        entries.sort_by_key(|(node_id, _)| node_id.as_str());

        let mut hasher = Sha256::new();
        for (node_id, timestamp) in entries {
            hasher.update(node_id.as_str().as_bytes());
            hasher.update(timestamp.to_le_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Check if this vector clock dominates another (is ahead in all dimensions).
    pub fn dominates(&self, other: &VectorClock) -> bool {
        let all_nodes: std::collections::HashSet<_> =
            self.clocks.keys().chain(other.clocks.keys()).collect();

        for node_id in &all_nodes {
            if self.get(node_id) < other.get(node_id) {
                return false;
            }
        }

        // Also check that we're strictly ahead in at least one dimension
        for node_id in &all_nodes {
            if self.get(node_id) > other.get(node_id) {
                return true;
            }
        }

        false // Equal vectors don't dominate
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VectorClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VectorClock{{")?;
        let mut first = true;
        for (node_id, timestamp) in &self.clocks {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{node_id}: {timestamp}")?;
            first = false;
        }
        write!(f, "}}")
    }
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
    fn test_vector_clock_creation() {
        let vc = VectorClock::new();
        assert_eq!(vc.get(&node_a()), 0);

        let vc2 = VectorClock::with_node(node_a(), 5);
        assert_eq!(vc2.get(&node_a()), 5);
        assert_eq!(vc2.get(&node_b()), 0);
    }

    #[test]
    fn test_vector_clock_increment() {
        let mut vc = VectorClock::new();
        assert_eq!(vc.get(&node_a()), 0);

        vc.increment(&node_a());
        assert_eq!(vc.get(&node_a()), 1);

        vc.increment(&node_a());
        assert_eq!(vc.get(&node_a()), 2);

        vc.increment(&node_b());
        assert_eq!(vc.get(&node_a()), 2);
        assert_eq!(vc.get(&node_b()), 1);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut vc1 = VectorClock::new();
        vc1.increment(&node_a());
        vc1.increment(&node_a());
        vc1.increment(&node_b());

        let mut vc2 = VectorClock::new();
        vc2.increment(&node_a());
        vc2.increment(&node_b());
        vc2.increment(&node_b());
        vc2.increment(&node_c());

        vc1.merge(&vc2);

        assert_eq!(vc1.get(&node_a()), 2); // max(2, 1)
        assert_eq!(vc1.get(&node_b()), 2); // max(1, 2)
        assert_eq!(vc1.get(&node_c()), 1); // max(0, 1)
    }

    #[test]
    fn test_vector_clock_compare_equal() {
        let mut vc1 = VectorClock::new();
        vc1.increment(&node_a());
        vc1.increment(&node_b());

        let mut vc2 = VectorClock::new();
        vc2.increment(&node_a());
        vc2.increment(&node_b());

        assert_eq!(vc1.compare(&vc2), Some(Ordering::Equal));
    }

    #[test]
    fn test_vector_clock_compare_happened_before() {
        let mut vc1 = VectorClock::new();
        vc1.increment(&node_a());

        let mut vc2 = VectorClock::new();
        vc2.increment(&node_a());
        vc2.increment(&node_a());
        vc2.increment(&node_b());

        assert_eq!(vc1.compare(&vc2), Some(Ordering::Less));
        assert_eq!(vc2.compare(&vc1), Some(Ordering::Greater));
        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_compare_concurrent() {
        let mut vc1 = VectorClock::new();
        vc1.increment(&node_a());
        vc1.increment(&node_a());

        let mut vc2 = VectorClock::new();
        vc2.increment(&node_b());
        vc2.increment(&node_b());

        assert_eq!(vc1.compare(&vc2), None);
        assert!(vc1.is_concurrent(&vc2));
        assert!(vc2.is_concurrent(&vc1));
    }

    #[test]
    fn test_vector_clock_dominates() {
        let mut vc1 = VectorClock::new();
        vc1.set(node_a(), 3);
        vc1.set(node_b(), 2);

        let mut vc2 = VectorClock::new();
        vc2.set(node_a(), 2);
        vc2.set(node_b(), 1);

        assert!(vc1.dominates(&vc2));
        assert!(!vc2.dominates(&vc1));

        // Equal clocks don't dominate
        let vc3 = vc1.clone();
        assert!(!vc1.dominates(&vc3));
        assert!(!vc3.dominates(&vc1));
    }

    #[test]
    fn test_vector_clock_total_events() {
        let mut vc = VectorClock::new();
        vc.set(node_a(), 5);
        vc.set(node_b(), 3);
        vc.set(node_c(), 2);

        assert_eq!(vc.total_events(), 10);
    }

    #[test]
    fn test_vector_clock_serialization() {
        let mut vc = VectorClock::new();
        vc.increment(&node_a());
        vc.increment(&node_b());

        let bytes = vc.to_compact_bytes().unwrap();
        let deserialized = VectorClock::from_compact_bytes(&bytes).unwrap();

        assert_eq!(vc, deserialized);
    }

    #[test]
    fn test_vector_clock_gc() {
        let mut vc = VectorClock::new();
        vc.set(node_a(), 1);
        vc.set(node_b(), 2);
        vc.set(node_c(), 3);

        vc.gc_old_nodes(&[node_a(), node_c()]);

        assert_eq!(vc.get(&node_a()), 1);
        assert_eq!(vc.get(&node_b()), 0); // Removed
        assert_eq!(vc.get(&node_c()), 3);
    }

    #[test]
    fn test_vector_clock_hash() {
        let mut vc1 = VectorClock::new();
        vc1.set(node_a(), 1);
        vc1.set(node_b(), 2);

        let mut vc2 = VectorClock::new();
        vc2.set(node_b(), 2);
        vc2.set(node_a(), 1);

        // Same content should produce same hash regardless of insertion order
        assert_eq!(vc1.hash(), vc2.hash());

        vc2.set(node_c(), 3);
        assert_ne!(vc1.hash(), vc2.hash());
    }
}
