//! Last-Write-Wins Register (LWW-Register) CRDT implementation.
//!
//! An LWW-Register stores a single value with timestamps, where the value
//! with the latest timestamp wins in case of conflicts. Perfect for identity
//! attributes, node status, configuration settings, and other single-value
//! state that needs conflict resolution through timestamps.

use crate::{NodeId, VectorClock, CRDT, CRDTError, CRDTResult, CausalCRDT, CRDTValue};
use serde::{Deserialize, Serialize};


/// A register that stores a single value with last-write-wins semantics.
/// 
/// The register maintains the current value along with metadata about when
/// and by whom it was last written. In case of conflicts, the value with
/// the latest timestamp wins. If timestamps are equal, a deterministic
/// tie-breaking mechanism is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue")]
pub struct LWWRegister<T> 
where 
    T: CRDTValue
{
    /// Unique identifier for this CRDT instance.
    id: String,
    /// Current value stored in the register.
    value: Option<T>,
    /// Timestamp when the current value was written.
    timestamp: u64,
    /// Node ID that wrote the current value.
    writer_node: Option<NodeId>,
    /// Vector clock for causality tracking.
    vector_clock: VectorClock,
    /// Sequence number for deterministic tie-breaking within same timestamp.
    sequence: u64,
}

/// Operations that can be applied to an LWW-Register.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue")]
pub enum LWWRegisterOperation<T> 
where 
    T: CRDTValue
{
    /// Write a new value to the register.
    Write { 
        value: T, 
        timestamp: u64, 
        node_id: NodeId, 
        sequence: u64 
    },
    /// Clear the register (set to None).
    Clear { 
        timestamp: u64, 
        node_id: NodeId, 
        sequence: u64 
    },
}

/// Metadata about a write operation for debugging and auditing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteMetadata {
    /// When the write occurred.
    pub timestamp: u64,
    /// Node that performed the write.
    pub node_id: NodeId,
    /// Sequence number for tie-breaking.
    pub sequence: u64,
    /// Vector clock at time of write.
    pub vector_clock: VectorClock,
}

impl<T> LWWRegister<T> 
where 
    T: CRDTValue
{
    /// Create a new LWW-Register with the given ID.
    pub fn new(id: String) -> Self {
        Self {
            id,
            value: None,
            timestamp: 0,
            writer_node: None,
            vector_clock: VectorClock::new(),
            sequence: 0,
        }
    }
    
    /// Create a new LWW-Register with an initial value.
    pub fn with_initial_value(id: String, value: T, node_id: NodeId) -> Self {
        let mut register = Self::new(id);
        register.write(value, node_id).unwrap();
        register
    }
    
    /// Get the current value stored in the register.
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
    
    /// Check if the register has a value.
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }
    
    /// Write a new value to the register.
    /// 
    /// This automatically uses the current vector clock timestamp and increments
    /// the local sequence number for tie-breaking.
    pub fn write(&mut self, value: T, node_id: NodeId) -> CRDTResult<()> {
        self.vector_clock.increment(&node_id);
        let timestamp = self.vector_clock.get(&node_id);
        
        // Generate sequence number for deterministic ordering
        self.sequence += 1;
        
        self.apply_write(value, timestamp, node_id, self.sequence)
    }
    
    /// Clear the register (set to None).
    pub fn clear(&mut self, node_id: NodeId) -> CRDTResult<()> {
        self.vector_clock.increment(&node_id);
        let timestamp = self.vector_clock.get(&node_id);
        
        self.sequence += 1;
        
        self.apply_clear(timestamp, node_id, self.sequence)
    }
    
    /// Apply a write operation with explicit timestamp and sequence.
    /// 
    /// This is used internally and for applying operations from other nodes.
    fn apply_write(&mut self, value: T, timestamp: u64, node_id: NodeId, sequence: u64) -> CRDTResult<()> {
        if self.should_apply_operation(timestamp, &node_id, sequence) {
            self.value = Some(value);
            self.timestamp = timestamp;
            self.writer_node = Some(node_id.clone());
            self.sequence = sequence;
            
            // Update vector clock to reflect we've seen this timestamp
            let current_time = self.vector_clock.get(&node_id);
            if timestamp > current_time {
                self.vector_clock.set(node_id, timestamp);
            }
        }
        
        Ok(())
    }
    
    /// Apply a clear operation with explicit timestamp and sequence.
    fn apply_clear(&mut self, timestamp: u64, node_id: NodeId, sequence: u64) -> CRDTResult<()> {
        if self.should_apply_operation(timestamp, &node_id, sequence) {
            self.value = None;
            self.timestamp = timestamp;
            self.writer_node = Some(node_id.clone());
            self.sequence = sequence;
            
            // Update vector clock to reflect we've seen this timestamp
            let current_time = self.vector_clock.get(&node_id);
            if timestamp > current_time {
                self.vector_clock.set(node_id, timestamp);
            }
        }
        
        Ok(())
    }
    
    /// Determine if a new operation should be applied based on LWW semantics.
    /// 
    /// Returns true if:
    /// 1. New timestamp is greater than current, OR
    /// 2. Timestamps are equal but new sequence is greater, OR
    /// 3. Timestamps and sequences are equal but node ID comparison favors new operation
    fn should_apply_operation(&self, new_timestamp: u64, new_node: &NodeId, new_sequence: u64) -> bool {
        // If we have no current value, always apply
        if self.writer_node.is_none() {
            return true;
        }
        
        match new_timestamp.cmp(&self.timestamp) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                // Timestamps are equal, check sequence numbers
                match new_sequence.cmp(&self.sequence) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => {
                        // Sequence numbers are also equal, use deterministic node comparison
                        if let Some(current_node) = &self.writer_node {
                            new_node.as_str() > current_node.as_str()
                        } else {
                            true
                        }
                    }
                }
            }
        }
    }
    
    /// Get metadata about the current value.
    pub fn get_metadata(&self) -> Option<WriteMetadata> {
        self.writer_node.as_ref().map(|node| WriteMetadata {
            timestamp: self.timestamp,
            node_id: node.clone(),
            sequence: self.sequence,
            vector_clock: self.vector_clock.clone(),
        })
    }
    
    /// Get the timestamp of the current value.
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Get the node that wrote the current value.
    pub fn get_writer_node(&self) -> Option<&NodeId> {
        self.writer_node.as_ref()
    }
    
    /// Check if this register has been written by the given node.
    pub fn was_written_by(&self, node_id: &NodeId) -> bool {
        self.writer_node.as_ref() == Some(node_id)
    }
    
    /// Get the age of the current value (current vector clock time - write timestamp).
    /// 
    /// Returns None if the register is empty or if we can't determine age.
    pub fn get_age(&self, current_node: &NodeId) -> Option<u64> {
        if self.writer_node.is_some() {
            let current_time = self.vector_clock.get(current_node);
            if current_time >= self.timestamp {
                Some(current_time - self.timestamp)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Create a snapshot of the current state for synchronization.
    pub fn snapshot(&self) -> LWWRegisterSnapshot<T> {
        LWWRegisterSnapshot {
            value: self.value.clone(),
            timestamp: self.timestamp,
            writer_node: self.writer_node.clone(),
            sequence: self.sequence,
            vector_clock: self.vector_clock.clone(),
        }
    }
    
    /// Apply a snapshot from another register.
    pub fn apply_snapshot(&mut self, snapshot: LWWRegisterSnapshot<T>) -> CRDTResult<()> {
        if let Some(writer_node) = &snapshot.writer_node {
            if let Some(value) = snapshot.value {
                self.apply_write(value, snapshot.timestamp, writer_node.clone(), snapshot.sequence)?;
            } else {
                self.apply_clear(snapshot.timestamp, writer_node.clone(), snapshot.sequence)?;
            }
        }
        
        // Merge vector clocks
        self.vector_clock.merge(&snapshot.vector_clock);
        
        Ok(())
    }
}

impl<T> CRDT for LWWRegister<T> 
where 
    T: CRDTValue
{
    type Operation = LWWRegisterOperation<T>;
    
    fn merge(&mut self, other: &Self) {
        // Apply the other register's current state as an operation
        if let Some(other_node) = &other.writer_node {
            if let Some(other_value) = &other.value {
                let _ = self.apply_write(
                    other_value.clone(), 
                    other.timestamp, 
                    other_node.clone(), 
                    other.sequence
                );
            } else {
                let _ = self.apply_clear(
                    other.timestamp, 
                    other_node.clone(), 
                    other.sequence
                );
            }
        }
        
        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);
        
        // Take maximum sequence counter to avoid conflicts
        self.sequence = self.sequence.max(other.sequence);
    }
    
    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError> {
        match op {
            LWWRegisterOperation::Write { value, timestamp, node_id, sequence } => {
                self.apply_write(value, timestamp, node_id, sequence)
            },
            LWWRegisterOperation::Clear { timestamp, node_id, sequence } => {
                self.apply_clear(timestamp, node_id, sequence)
            }
        }
    }
    
    fn value(&self) -> serde_json::Value {
        serde_json::json!({
            "value": self.value,
            "timestamp": self.timestamp,
            "writer_node": self.writer_node.as_ref().map(|n| n.as_str()),
            "sequence": self.sequence,
            "is_empty": self.is_empty()
        })
    }
    
    fn crdt_id(&self) -> String {
        self.id.clone()
    }
}

impl<T> CausalCRDT for LWWRegister<T> 
where 
    T: CRDTValue
{
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

impl<T> PartialEq for LWWRegister<T> 
where 
    T: CRDTValue + PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && 
        self.value == other.value &&
        self.timestamp == other.timestamp &&
        self.writer_node == other.writer_node &&
        self.sequence == other.sequence
    }
}

impl<T> Eq for LWWRegister<T> 
where 
    T: CRDTValue + Eq + PartialEq
{}

/// Snapshot of an LWW-Register for synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue")]
pub struct LWWRegisterSnapshot<T> 
where 
    T: CRDTValue
{
    /// Current value.
    pub value: Option<T>,
    /// Timestamp of current value.
    pub timestamp: u64,
    /// Node that wrote current value.
    pub writer_node: Option<NodeId>,
    /// Sequence number.
    pub sequence: u64,
    /// Vector clock.
    pub vector_clock: VectorClock,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    
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
    fn test_lww_register_creation() {
        let register: LWWRegister<String> = LWWRegister::new("test_register".to_string());
        assert!(register.is_empty());
        assert_eq!(register.get(), None);
        assert_eq!(register.crdt_id(), "test_register");
        assert_eq!(register.get_timestamp(), 0);
    }
    
    #[test]
    fn test_lww_register_with_initial_value() {
        let register = LWWRegister::with_initial_value(
            "test".to_string(), 
            "initial_value".to_string(), 
            node_a()
        );
        
        assert!(!register.is_empty());
        assert_eq!(register.get(), Some(&"initial_value".to_string()));
        assert_eq!(register.get_writer_node(), Some(&node_a()));
        assert!(register.was_written_by(&node_a()));
    }
    
    #[test]
    fn test_lww_register_write() {
        let mut register: LWWRegister<String> = LWWRegister::new("test".to_string());
        
        register.write("first_value".to_string(), node_a()).unwrap();
        assert_eq!(register.get(), Some(&"first_value".to_string()));
        assert_eq!(register.get_timestamp(), 1);
        assert!(register.was_written_by(&node_a()));
        
        register.write("second_value".to_string(), node_a()).unwrap();
        assert_eq!(register.get(), Some(&"second_value".to_string()));
        assert_eq!(register.get_timestamp(), 2);
        
        register.write("third_value".to_string(), node_b()).unwrap();
        assert_eq!(register.get(), Some(&"third_value".to_string()));
        assert!(register.was_written_by(&node_b()));
    }
    
    #[test]
    fn test_lww_register_clear() {
        let mut register = LWWRegister::with_initial_value(
            "test".to_string(), 
            42u64, 
            node_a()
        );
        
        assert!(!register.is_empty());
        assert_eq!(register.get(), Some(&42u64));
        
        register.clear(node_a()).unwrap();
        assert!(register.is_empty());
        assert_eq!(register.get(), None);
        assert!(register.was_written_by(&node_a()));
    }
    
    #[test]
    fn test_lww_register_last_write_wins() {
        let mut register1: LWWRegister<String> = LWWRegister::new("test".to_string());
        let mut register2: LWWRegister<String> = LWWRegister::new("test".to_string());
        
        // Writes at different times
        register1.write("value_1".to_string(), node_a()).unwrap();
        register2.write("value_2".to_string(), node_b()).unwrap();
        register2.write("value_3".to_string(), node_b()).unwrap();
        
        register1.merge(&register2);
        
        // register2 had the later write, so its value should win
        assert_eq!(register1.get(), Some(&"value_3".to_string()));
        assert!(register1.was_written_by(&node_b()));
    }
    
    #[test]
    fn test_lww_register_concurrent_writes_tie_breaking() {
        let mut register1: LWWRegister<String> = LWWRegister::new("test".to_string());
        let mut register2: LWWRegister<String> = LWWRegister::new("test".to_string());
        
        // Simulate concurrent writes with same timestamp
        register1.apply_write("value_a".to_string(), 5, node_a(), 1).unwrap();
        register2.apply_write("value_b".to_string(), 5, node_b(), 1).unwrap();
        
        register1.merge(&register2);
        
        // node_b comes after node_a lexicographically, so value_b should win
        assert_eq!(register1.get(), Some(&"value_b".to_string()));
        assert!(register1.was_written_by(&node_b()));
    }
    
    #[test]
    fn test_lww_register_merge_idempotent() {
        let mut register1 = LWWRegister::with_initial_value(
            "test".to_string(), 
            "test_value".to_string(), 
            node_a()
        );
        
        let register2 = register1.clone();
        let original_value = register1.get().cloned();
        
        register1.merge(&register2);
        assert_eq!(register1.get(), original_value.as_ref());
    }
    
    #[test]
    fn test_lww_register_merge_commutative() {
        let mut register1: LWWRegister<String> = LWWRegister::new("test".to_string());
        let mut register2: LWWRegister<String> = LWWRegister::new("test".to_string());
        
        register1.write("value_1".to_string(), node_a()).unwrap();
        register2.write("value_2".to_string(), node_b()).unwrap();
        
        let mut register1_copy = register1.clone();
        let register2_copy = register2.clone();
        
        // register1.merge(register2)
        register1.merge(&register2);
        
        // register2.merge(register1_copy)
        register2.merge(&register1_copy);
        
        assert_eq!(register1.get(), register2.get());
        assert_eq!(register1.get_writer_node(), register2.get_writer_node());
    }
    
    #[test]
    fn test_lww_register_apply_operations() {
        let mut register: LWWRegister<String> = LWWRegister::new("test".to_string());
        
        let write_op = LWWRegisterOperation::Write {
            value: "applied_value".to_string(),
            timestamp: 10,
            node_id: node_c(),
            sequence: 1,
        };
        
        register.apply_operation(write_op).unwrap();
        assert_eq!(register.get(), Some(&"applied_value".to_string()));
        assert_eq!(register.get_timestamp(), 10);
        assert!(register.was_written_by(&node_c()));
        
        let clear_op = LWWRegisterOperation::Clear {
            timestamp: 15,
            node_id: node_c(),
            sequence: 2,
        };
        
        register.apply_operation(clear_op).unwrap();
        assert!(register.is_empty());
        assert_eq!(register.get_timestamp(), 15);
    }
    
    #[test]
    fn test_lww_register_with_dids() {
        let mut register: LWWRegister<Did> = LWWRegister::new("identity_register".to_string());
        
        let alice_did = Did::new("key", "alice");
        let bob_did = Did::new("key", "bob");
        
        register.write(alice_did.clone(), node_a()).unwrap();
        assert_eq!(register.get(), Some(&alice_did));
        
        register.write(bob_did.clone(), node_b()).unwrap();
        assert_eq!(register.get(), Some(&bob_did));
        assert!(register.was_written_by(&node_b()));
    }
    
    #[test]
    fn test_lww_register_metadata() {
        let mut register: LWWRegister<u64> = LWWRegister::new("test".to_string());
        
        assert_eq!(register.get_metadata(), None);
        
        register.write(100, node_a()).unwrap();
        
        let metadata = register.get_metadata().unwrap();
        assert_eq!(metadata.node_id, node_a());
        assert_eq!(metadata.timestamp, 1);
        assert_eq!(metadata.sequence, 1);
    }
    
    #[test]
    fn test_lww_register_age() {
        let mut register: LWWRegister<u64> = LWWRegister::new("test".to_string());
        
        register.write(100, node_a()).unwrap();
        assert_eq!(register.get_age(&node_a()), Some(0)); // Just written
        
        register.advance_clock(&node_a());
        register.advance_clock(&node_a());
        assert_eq!(register.get_age(&node_a()), Some(2)); // 2 ticks later
    }
    
    #[test]
    fn test_lww_register_snapshot() {
        let mut register = LWWRegister::with_initial_value(
            "test".to_string(), 
            "snapshot_value".to_string(), 
            node_a()
        );
        
        let snapshot = register.snapshot();
        assert_eq!(snapshot.value, Some("snapshot_value".to_string()));
        assert_eq!(snapshot.writer_node, Some(node_a()));
        
        let mut register2: LWWRegister<String> = LWWRegister::new("test2".to_string());
        register2.apply_snapshot(snapshot).unwrap();
        
        assert_eq!(register2.get(), Some(&"snapshot_value".to_string()));
        assert!(register2.was_written_by(&node_a()));
    }
    
    #[test]
    fn test_lww_register_value_json() {
        let mut register = LWWRegister::with_initial_value(
            "test".to_string(), 
            "json_value".to_string(), 
            node_a()
        );
        
        let json_value = register.value();
        assert_eq!(json_value["value"], "json_value");
        assert_eq!(json_value["writer_node"], node_a().as_str());
        assert_eq!(json_value["is_empty"], false);
        
        register.clear(node_a()).unwrap();
        let json_cleared = register.value();
        assert_eq!(json_cleared["value"], serde_json::Value::Null);
        assert_eq!(json_cleared["is_empty"], true);
    }
}