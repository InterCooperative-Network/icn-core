#![doc = include_str!("../README.md")]

//! # ICN CRDT Crate
//!
//! This crate provides Conflict-free Replicated Data Types (CRDTs) for the
//! InterCooperative Network (ICN) to enable real-time, conflict-free state
//! synchronization across distributed nodes, clusters, and federations.

use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

pub mod crdt_map;
pub mod g_counter;
pub mod gossip;
pub mod lww_register;
pub mod or_set;
pub mod pn_counter;
pub mod vector_clock;

pub use crdt_map::CRDTMap;
pub use g_counter::GCounter;
pub use gossip::{CRDTSynchronizer, GossipConfig, GossipSerializable, GossipTransport};
pub use lww_register::LWWRegister;
pub use or_set::ORSet;
pub use pn_counter::PNCounter;
pub use vector_clock::VectorClock;

/// Unique identifier for a node in the CRDT network.
/// This should be stable across restarts and unique across all nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a new NodeId from a string.
    pub fn new(id: String) -> Self {
        NodeId(id)
    }

    /// Create a NodeId from a DID.
    pub fn from_did(did: &Did) -> Self {
        NodeId(did.to_string())
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        NodeId(s.to_string())
    }
}

/// Core trait that all CRDT types must implement.
///
/// This trait ensures that all CRDTs can be merged in a conflict-free manner
/// and provide the mathematical guarantees required for distributed consensus.
pub trait CRDT: Clone + Serialize {
    /// The type of operations that can be applied to this CRDT.
    type Operation: Clone + Serialize;

    /// Merge this CRDT with another instance of the same type.
    ///
    /// This operation must be:
    /// - Commutative: merge(a, b) = merge(b, a)  
    /// - Associative: merge(merge(a, b), c) = merge(a, merge(b, c))
    /// - Idempotent: merge(a, a) = a
    fn merge(&mut self, other: &Self);

    /// Apply an operation to this CRDT.
    /// Returns Ok(()) if the operation was applied, or an error if invalid.
    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError>;

    /// Get the current state value of this CRDT.
    fn value(&self) -> serde_json::Value;

    /// Get a unique identifier for this CRDT instance.
    fn crdt_id(&self) -> String;

    /// Check if this CRDT is causally ready to receive operations from the given vector clock.
    fn can_apply_operation(&self, _op: &Self::Operation, _vector_clock: &VectorClock) -> bool {
        true // Default implementation allows all operations
    }
}

/// Trait for CRDTs that can be observed for changes.
/// Useful for triggering synchronization when local state changes.
pub trait ObservableCRDT: CRDT {
    /// Register a callback to be invoked when the CRDT state changes.
    fn on_change(&mut self, callback: Box<dyn Fn(&Self) + Send + Sync>);

    /// Get the current version/timestamp of this CRDT for change detection.
    fn version(&self) -> u64;
}

/// Trait for CRDTs that support causal ordering via vector clocks.
pub trait CausalCRDT: CRDT {
    /// Get the current vector clock for this CRDT.
    fn vector_clock(&self) -> &VectorClock;

    /// Update the vector clock when applying an operation.
    fn advance_clock(&mut self, node_id: &NodeId);

    /// Check if this CRDT has observed all changes up to the given vector clock.
    fn has_seen(&self, vector_clock: &VectorClock) -> bool;
}

/// Errors that can occur during CRDT operations.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CRDTError {
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Vector clock error: {0}")]
    VectorClockError(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Common error: {0}")]
    CommonError(#[from] CommonError),
}

/// Result type for CRDT operations.
pub type CRDTResult<T> = Result<T, CRDTError>;

/// Metadata associated with CRDT operations for causality and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// The node that generated this operation.
    pub node_id: NodeId,
    /// Vector clock at the time of operation.
    pub vector_clock: VectorClock,
    /// Timestamp when the operation was created (for debugging).
    pub timestamp: u64,
    /// Optional operation identifier for deduplication.
    pub operation_id: Option<String>,
}

impl OperationMetadata {
    /// Create new operation metadata.
    pub fn new(node_id: NodeId, vector_clock: VectorClock, timestamp: u64) -> Self {
        Self {
            node_id,
            vector_clock,
            timestamp,
            operation_id: None,
        }
    }

    /// Create new operation metadata with an operation ID.
    pub fn with_id(
        node_id: NodeId,
        vector_clock: VectorClock,
        timestamp: u64,
        op_id: String,
    ) -> Self {
        Self {
            node_id,
            vector_clock,
            timestamp,
            operation_id: Some(op_id),
        }
    }
}

/// A CRDT operation with its associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTOperation<T> {
    /// The actual operation data.
    pub operation: T,
    /// Metadata for causality and tracking.
    pub metadata: OperationMetadata,
}

impl<T> CRDTOperation<T> {
    /// Create a new CRDT operation.
    pub fn new(operation: T, metadata: OperationMetadata) -> Self {
        Self {
            operation,
            metadata,
        }
    }
}

/// Helper trait for converting values to/from CRDT-compatible types.
pub trait CRDTValue: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync {}

// Implement CRDTValue for common types
impl CRDTValue for String {}
impl CRDTValue for u64 {}
impl CRDTValue for i64 {}
impl CRDTValue for f64 {}
impl CRDTValue for bool {}
impl CRDTValue for Did {}
impl<T: CRDTValue> CRDTValue for Vec<T> {}
impl<K: CRDTValue + Eq + Hash, V: CRDTValue> CRDTValue for HashMap<K, V> {}

/// Configuration for CRDT behavior and optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTConfig {
    /// Maximum number of operations to keep in memory for causality checking.
    pub max_operation_history: usize,
    /// How often to perform garbage collection of old operations (in seconds).
    pub gc_interval_seconds: u64,
    /// Whether to enable operation compression for network efficiency.
    pub enable_compression: bool,
    /// Maximum age of operations to keep (in seconds).
    pub max_operation_age_seconds: u64,
}

impl Default for CRDTConfig {
    fn default() -> Self {
        Self {
            max_operation_history: 10000,
            gc_interval_seconds: 300, // 5 minutes
            enable_compression: true,
            max_operation_age_seconds: 86400, // 24 hours
        }
    }
}

/// Statistics about CRDT operations and performance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CRDTStats {
    /// Total number of operations applied.
    pub operations_applied: u64,
    /// Number of merge operations performed.
    pub merges_performed: u64,
    /// Number of conflicts resolved.
    pub conflicts_resolved: u64,
    /// Current size of the CRDT state in bytes.
    pub state_size_bytes: u64,
    /// Number of operations currently in memory.
    pub operations_in_memory: u64,
    /// Last synchronization timestamp.
    pub last_sync_timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let node_id = NodeId::new("test-node-1".to_string());
        assert_eq!(node_id.as_str(), "test-node-1");
        assert_eq!(node_id.to_string(), "test-node-1");
    }

    #[test]
    fn test_node_id_from_did() {
        let did = Did::new("key", "test123");
        let node_id = NodeId::from_did(&did);
        assert_eq!(node_id.as_str(), "did:key:test123");
    }

    #[test]
    fn test_crdt_config_default() {
        let config = CRDTConfig::default();
        assert_eq!(config.max_operation_history, 10000);
        assert_eq!(config.gc_interval_seconds, 300);
        assert!(config.enable_compression);
        assert_eq!(config.max_operation_age_seconds, 86400);
    }

    #[test]
    fn test_operation_metadata() {
        let node_id = NodeId::new("test".to_string());
        let vector_clock = VectorClock::new();
        let metadata = OperationMetadata::new(node_id.clone(), vector_clock.clone(), 1000);

        assert_eq!(metadata.node_id, node_id);
        assert_eq!(metadata.vector_clock, vector_clock);
        assert_eq!(metadata.timestamp, 1000);
        assert_eq!(metadata.operation_id, None);
    }
}
