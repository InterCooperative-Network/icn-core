//! CRDT Map implementation for complex nested state management.
//!
//! A CRDT Map stores key-value pairs where each value is itself a CRDT,
//! enabling complex nested state that can be merged conflict-free.
//! Perfect for managing hierarchical configuration, user profiles,
//! organization structures, and other complex state.

use crate::{CRDTError, CRDTResult, CRDTValue, CausalCRDT, LWWRegister, NodeId, VectorClock, CRDT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// A CRDT Map that stores key-value pairs where values are CRDTs.
///
/// This enables hierarchical and complex state management where each
/// key maps to a CRDT that can be independently updated and merged.
/// The map itself uses LWW semantics for key insertion/removal while
/// the values use their own CRDT semantics for updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    bound = "K: CRDTValue + Eq + Hash + std::fmt::Debug + Serialize + for<'a> Deserialize<'a>, V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>"
)]
pub struct CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
{
    /// Unique identifier for this CRDT instance.
    id: String,
    /// Map from keys to their associated CRDT values.
    entries: HashMap<K, V>,
    /// Tombstones for tracking removed keys (using LWW-Register).
    /// A key exists if it's in entries and not in tombstones,
    /// or if it's in entries and the tombstone is older.
    tombstones: HashMap<K, LWWRegister<bool>>,
    /// Vector clock for causality tracking.
    vector_clock: VectorClock,
    /// Sequence counter for deterministic ordering.
    sequence: u64,
}

/// Operations that can be applied to a CRDT Map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    bound = "K: CRDTValue + Eq + Hash + std::fmt::Debug + Serialize + for<'a> Deserialize<'a>, V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>, V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>"
)]
pub enum CRDTMapOperation<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
    V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
{
    /// Insert or update a value for a key.
    Put {
        key: K,
        value: V,
        timestamp: u64,
        node_id: NodeId,
    },
    /// Remove a key from the map.
    Remove {
        key: K,
        timestamp: u64,
        node_id: NodeId,
    },
    /// Update an existing CRDT value at a key.
    UpdateValue { key: K, operation: V::Operation },
}

/// Statistics about the CRDT Map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTMapStats {
    /// Number of active keys in the map.
    pub active_keys: u64,
    /// Number of tombstoned (removed) keys.
    pub tombstoned_keys: u64,
    /// Total number of keys ever seen.
    pub total_keys_seen: u64,
    /// Number of nodes that have contributed operations.
    pub contributing_nodes: u64,
    /// Size of the largest CRDT value in bytes (approximate).
    pub largest_value_size: u64,
    /// Total size of all values in bytes (approximate).
    pub total_size_bytes: u64,
}

impl<K, V> CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
{
    /// Create a new CRDT Map with the given ID.
    pub fn new(id: String) -> Self {
        Self {
            id,
            entries: HashMap::new(),
            tombstones: HashMap::new(),
            vector_clock: VectorClock::new(),
            sequence: 0,
        }
    }

    /// Check if a key exists in the map (not tombstoned).
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key) && !self.is_tombstoned(key)
    }

    /// Get the CRDT value for a key.
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.is_tombstoned(key) {
            None
        } else {
            self.entries.get(key)
        }
    }

    /// Get a mutable reference to the CRDT value for a key.
    ///
    /// This allows direct mutation of the CRDT which will be tracked
    /// by the CRDT's own vector clock.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.is_tombstoned(key) {
            None
        } else {
            self.entries.get_mut(key)
        }
    }

    /// Insert or update a value for a key.
    pub fn put(&mut self, key: K, value: V, node_id: NodeId) -> CRDTResult<()> {
        self.vector_clock.increment(&node_id);
        let timestamp = self.vector_clock.get(&node_id);

        self.apply_put(key, value, timestamp, node_id)
    }

    /// Remove a key from the map.
    pub fn remove(&mut self, key: &K, node_id: NodeId) -> CRDTResult<bool> {
        self.vector_clock.increment(&node_id);
        let timestamp = self.vector_clock.get(&node_id);

        Ok(self.apply_remove(key.clone(), timestamp, node_id))
    }

    /// Update an existing CRDT value using its operation.
    pub fn update_value(&mut self, key: &K, operation: V::Operation) -> CRDTResult<()> {
        if let Some(value) = self.get_mut(key) {
            value.apply_operation(operation)?;
            Ok(())
        } else {
            Err(CRDTError::InvalidOperation(format!(
                "Key not found: {key:?}"
            )))
        }
    }

    /// Get all active keys in the map.
    pub fn keys(&self) -> Vec<K> {
        self.entries
            .keys()
            .filter(|key| !self.is_tombstoned(key))
            .cloned()
            .collect()
    }

    /// Get all active key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries
            .iter()
            .filter(|(key, _)| !self.is_tombstoned(key))
    }

    /// Get the number of active entries.
    pub fn len(&self) -> usize {
        self.keys().len()
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Apply a put operation with explicit timestamp.
    fn apply_put(&mut self, key: K, value: V, timestamp: u64, node_id: NodeId) -> CRDTResult<()> {
        // Check if this put should override an existing tombstone
        if let Some(tombstone) = self.tombstones.get(&key) {
            if tombstone.get_timestamp() >= timestamp {
                // Tombstone is newer or equal, put operation is ignored
                return Ok(());
            }
        }

        // Insert or update the value
        self.entries.insert(key.clone(), value);

        // Remove any existing tombstone that's older
        self.tombstones.remove(&key);

        // Update vector clock
        let current_time = self.vector_clock.get(&node_id);
        if timestamp > current_time {
            self.vector_clock.set(node_id, timestamp);
        }

        Ok(())
    }

    /// Apply a remove operation with explicit timestamp.
    fn apply_remove(&mut self, key: K, timestamp: u64, node_id: NodeId) -> bool {
        // Create or update tombstone
        let tombstone_id = format!("{}:tombstone:{key:?}", self.id);
        let mut tombstone = self
            .tombstones
            .get(&key)
            .cloned()
            .unwrap_or_else(|| LWWRegister::new(tombstone_id));

        let _ = tombstone.write(true, node_id.clone());
        self.tombstones.insert(key.clone(), tombstone);

        // Update vector clock
        let current_time = self.vector_clock.get(&node_id);
        if timestamp > current_time {
            self.vector_clock.set(node_id, timestamp);
        }

        // Return whether the key was actually present
        self.entries.contains_key(&key)
    }

    /// Check if a key is tombstoned (marked as removed).
    fn is_tombstoned(&self, key: &K) -> bool {
        if let Some(tombstone) = self.tombstones.get(key) {
            tombstone.get().unwrap_or(&false) == &true
        } else {
            false
        }
    }

    /// Get all keys that have ever been in the map (including tombstoned).
    pub fn all_keys(&self) -> Vec<K> {
        let mut keys: Vec<K> = self.entries.keys().cloned().collect();
        keys.extend(self.tombstones.keys().cloned());
        keys.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}"))); // Deterministic ordering
        keys.dedup();
        keys
    }

    /// Get statistics about this map.
    pub fn stats(&self) -> CRDTMapStats {
        let active_keys = self.len() as u64;
        let tombstoned_keys = self.tombstones.len() as u64;
        let total_keys_seen = self.all_keys().len() as u64;

        // Estimate value sizes (rough approximation)
        let mut total_size = 0u64;
        let mut largest_size = 0u64;

        for (_, value) in self.iter() {
            if let Ok(serialized) = bincode::serialize(value) {
                let size = serialized.len() as u64;
                total_size += size;
                largest_size = largest_size.max(size);
            }
        }

        // Count contributing nodes from vector clock
        let contributing_nodes = self.vector_clock.node_ids().len() as u64;

        CRDTMapStats {
            active_keys,
            tombstoned_keys,
            total_keys_seen,
            contributing_nodes,
            largest_value_size: largest_size,
            total_size_bytes: total_size,
        }
    }

    /// Create a delta containing only operations newer than the given vector clock.
    pub fn delta_since(&self, other_clock: &VectorClock) -> CRDTMapDelta<K, V>
    where
        V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
    {
        let mut operations = Vec::new();

        // Find entries that are newer
        for (key, value) in &self.entries {
            if !self.is_tombstoned(key) {
                // Check if this entry is newer than other_clock
                // This is a simplified version - in practice you'd need timestamps per entry
                if let Ok(_serialized) = bincode::serialize(value) {
                    // For now, include all entries as we don't have per-entry timestamps
                    // In a full implementation, each entry would have its own timestamp
                    operations.push(CRDTMapOperation::Put {
                        key: key.clone(),
                        value: value.clone(),
                        timestamp: self.vector_clock.total_events(), // Approximation
                        node_id: NodeId::new("unknown".to_string()), // Would need to track
                    });
                }
            }
        }

        // Find tombstones that are newer
        for (key, tombstone) in &self.tombstones {
            if tombstone.get().unwrap_or(&false) == &true {
                // Check if tombstone is newer than other_clock
                let tombstone_time = tombstone.get_timestamp();
                if other_clock.total_events() < tombstone_time {
                    operations.push(CRDTMapOperation::Remove {
                        key: key.clone(),
                        timestamp: tombstone_time,
                        node_id: tombstone
                            .get_writer_node()
                            .cloned()
                            .unwrap_or_else(|| NodeId::new("unknown".to_string())),
                    });
                }
            }
        }

        CRDTMapDelta { operations }
    }

    /// Garbage collect old tombstones that are no longer needed.
    pub fn gc_tombstones(&mut self, min_age: u64) {
        let current_time = self.vector_clock.total_events();

        self.tombstones.retain(|_key, tombstone| {
            let age = current_time.saturating_sub(tombstone.get_timestamp());
            age < min_age
        });
    }

    /// Merge a delta into this map.
    pub fn apply_delta(&mut self, delta: CRDTMapDelta<K, V>) -> CRDTResult<()>
    where
        V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
    {
        for operation in delta.operations {
            self.apply_operation(operation)?;
        }
        Ok(())
    }
}

impl<K, V> CRDT for CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
    V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
{
    type Operation = CRDTMapOperation<K, V>;

    fn merge(&mut self, other: &Self) {
        // Merge all entries
        for (key, other_value) in &other.entries {
            if let Some(existing_value) = self.entries.get_mut(key) {
                // Merge the CRDT values
                existing_value.merge(other_value);
            } else {
                // Add new entry if not tombstoned by us
                if !self.is_tombstoned(key) {
                    self.entries.insert(key.clone(), other_value.clone());
                }
            }
        }

        // Merge tombstones
        for (key, other_tombstone) in &other.tombstones {
            if let Some(existing_tombstone) = self.tombstones.get_mut(key) {
                existing_tombstone.merge(other_tombstone);
            } else {
                self.tombstones.insert(key.clone(), other_tombstone.clone());
            }
        }

        // Check for keys that should be removed due to merged tombstones
        let keys_to_check: Vec<K> = self.entries.keys().cloned().collect();
        for key in keys_to_check {
            if self.is_tombstoned(&key) {
                self.entries.remove(&key);
            }
        }

        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);

        // Take maximum sequence to avoid conflicts
        self.sequence = self.sequence.max(other.sequence);
    }

    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError> {
        match op {
            CRDTMapOperation::Put {
                key,
                value,
                timestamp,
                node_id,
            } => self.apply_put(key, value, timestamp, node_id),
            CRDTMapOperation::Remove {
                key,
                timestamp,
                node_id,
            } => {
                self.apply_remove(key, timestamp, node_id);
                Ok(())
            }
            CRDTMapOperation::UpdateValue { key, operation } => self.update_value(&key, operation),
        }
    }

    fn value(&self) -> serde_json::Value {
        let mut map_value = serde_json::Map::new();

        for (key, value) in self.iter() {
            let key_str = format!("{key:?}");
            map_value.insert(key_str, value.value());
        }

        serde_json::json!({
            "entries": map_value,
            "active_keys": self.len(),
            "tombstoned_keys": self.tombstones.len(),
            "stats": self.stats()
        })
    }

    fn crdt_id(&self) -> String {
        self.id.clone()
    }
}

impl<K, V> CausalCRDT for CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
    V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
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

impl<K, V> PartialEq for CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.entries == other.entries && self.tombstones == other.tombstones
    }
}

impl<K, V> Eq for CRDTMap<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a> + Eq + PartialEq,
{
}

/// Delta containing operations for efficient synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    bound = "K: CRDTValue + Eq + Hash + std::fmt::Debug + Serialize + for<'a> Deserialize<'a>, V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>, V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>"
)]
pub struct CRDTMapDelta<K, V>
where
    K: CRDTValue + Eq + Hash + std::fmt::Debug,
    V: CRDT + Clone + Serialize + for<'a> Deserialize<'a>,
    V::Operation: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>,
{
    /// Operations to apply.
    pub operations: Vec<CRDTMapOperation<K, V>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GCounter, LWWRegister};

    fn node_a() -> NodeId {
        NodeId::new("node_a".to_string())
    }

    fn node_b() -> NodeId {
        NodeId::new("node_b".to_string())
    }

    fn node_c() -> NodeId {
        NodeId::new("node_c".to_string())
    }

    type TestMap = CRDTMap<String, LWWRegister<u64>>;
    type CounterMap = CRDTMap<String, GCounter>;

    #[test]
    fn test_crdt_map_creation() {
        let map: TestMap = CRDTMap::new("test_map".to_string());
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.crdt_id(), "test_map");
    }

    #[test]
    fn test_crdt_map_put_get() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let register = LWWRegister::with_initial_value("value1".to_string(), 42u64, node_a());

        map.put("key1".to_string(), register, node_a()).unwrap();
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&"key1".to_string()));

        let value = map.get(&"key1".to_string()).unwrap();
        assert_eq!(value.get(), Some(&42u64));
    }

    #[test]
    fn test_crdt_map_update_value() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let register = LWWRegister::new("value1".to_string());
        map.put("key1".to_string(), register, node_a()).unwrap();

        // Update the LWW register using its own operation
        use crate::lww_register::LWWRegisterOperation;
        let update_op = LWWRegisterOperation::Write {
            value: 100u64,
            timestamp: 5,
            node_id: node_b(),
            sequence: 1,
        };

        map.update_value(&"key1".to_string(), update_op).unwrap();

        let value = map.get(&"key1".to_string()).unwrap();
        assert_eq!(value.get(), Some(&100u64));
    }

    #[test]
    fn test_crdt_map_remove() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let register = LWWRegister::with_initial_value("value1".to_string(), 42u64, node_a());

        map.put("key1".to_string(), register, node_a()).unwrap();
        assert!(map.contains_key(&"key1".to_string()));

        let was_present = map.remove(&"key1".to_string(), node_a()).unwrap();
        assert!(was_present);
        assert!(!map.contains_key(&"key1".to_string()));
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_crdt_map_keys_iteration() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let reg1 = LWWRegister::with_initial_value("v1".to_string(), 1u64, node_a());
        let reg2 = LWWRegister::with_initial_value("v2".to_string(), 2u64, node_a());
        let reg3 = LWWRegister::with_initial_value("v3".to_string(), 3u64, node_a());

        map.put("key1".to_string(), reg1, node_a()).unwrap();
        map.put("key2".to_string(), reg2, node_a()).unwrap();
        map.put("key3".to_string(), reg3, node_a()).unwrap();

        let keys = map.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        let pairs: Vec<_> = map.iter().collect();
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn test_crdt_map_merge() {
        let mut map1: TestMap = CRDTMap::new("test".to_string());
        let mut map2: TestMap = CRDTMap::new("test".to_string());

        // Add different keys to each map
        let reg1 = LWWRegister::with_initial_value("v1".to_string(), 1u64, node_a());
        let reg2 = LWWRegister::with_initial_value("v2".to_string(), 2u64, node_b());

        map1.put("key1".to_string(), reg1, node_a()).unwrap();
        map2.put("key2".to_string(), reg2, node_b()).unwrap();

        map1.merge(&map2);

        assert_eq!(map1.len(), 2);
        assert!(map1.contains_key(&"key1".to_string()));
        assert!(map1.contains_key(&"key2".to_string()));
    }

    #[test]
    fn test_crdt_map_merge_same_key() {
        let mut map1: TestMap = CRDTMap::new("test".to_string());
        let mut map2: TestMap = CRDTMap::new("test".to_string());

        // Add same key with different values to each map
        let mut reg1 = LWWRegister::new("v1".to_string());
        reg1.write(10u64, node_a()).unwrap();

        let mut reg2 = LWWRegister::new("v2".to_string());
        reg2.write(20u64, node_b()).unwrap();
        reg2.write(30u64, node_b()).unwrap(); // Later write

        map1.put("same_key".to_string(), reg1, node_a()).unwrap();
        map2.put("same_key".to_string(), reg2, node_b()).unwrap();

        map1.merge(&map2);

        assert_eq!(map1.len(), 1);
        let merged_value = map1.get(&"same_key".to_string()).unwrap();
        assert_eq!(merged_value.get(), Some(&30u64)); // Latest write wins
    }

    #[test]
    fn test_crdt_map_remove_and_merge() {
        let mut map1: TestMap = CRDTMap::new("test".to_string());
        let mut map2: TestMap = CRDTMap::new("test".to_string());

        let reg = LWWRegister::with_initial_value("v1".to_string(), 42u64, node_a());

        // Both maps have the same key
        map1.put("key1".to_string(), reg.clone(), node_a()).unwrap();
        map2.put("key1".to_string(), reg, node_a()).unwrap();

        // map1 removes the key
        map1.remove(&"key1".to_string(), node_a()).unwrap();
        assert!(!map1.contains_key(&"key1".to_string()));

        // Merge should propagate the removal
        map2.merge(&map1);
        assert!(!map2.contains_key(&"key1".to_string()));
    }

    #[test]
    fn test_crdt_map_with_counters() {
        let mut map: CounterMap = CRDTMap::new("counter_map".to_string());

        let mut counter1 = GCounter::new("c1".to_string());
        counter1.increment(&node_a(), 5).unwrap();

        let mut counter2 = GCounter::new("c2".to_string());
        counter2.increment(&node_b(), 10).unwrap();

        map.put("counter_a".to_string(), counter1, node_a())
            .unwrap();
        map.put("counter_b".to_string(), counter2, node_b())
            .unwrap();

        assert_eq!(map.len(), 2);

        // Get mutable reference and update counter
        if let Some(counter) = map.get_mut(&"counter_a".to_string()) {
            counter.increment(&node_c(), 3).unwrap();
        }

        let updated_counter = map.get(&"counter_a".to_string()).unwrap();
        assert_eq!(updated_counter.get_total(), 8); // 5 + 3
    }

    #[test]
    fn test_crdt_map_apply_operations() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let register = LWWRegister::with_initial_value("v1".to_string(), 100u64, node_a());

        let put_op = CRDTMapOperation::Put {
            key: "op_key".to_string(),
            value: register,
            timestamp: 5,
            node_id: node_a(),
        };

        map.apply_operation(put_op).unwrap();
        assert!(map.contains_key(&"op_key".to_string()));

        let remove_op = CRDTMapOperation::Remove {
            key: "op_key".to_string(),
            timestamp: 10,
            node_id: node_a(),
        };

        map.apply_operation(remove_op).unwrap();
        assert!(!map.contains_key(&"op_key".to_string()));
    }

    #[test]
    fn test_crdt_map_stats() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let reg1 = LWWRegister::with_initial_value("v1".to_string(), 1u64, node_a());
        let reg2 = LWWRegister::with_initial_value("v2".to_string(), 2u64, node_b());

        map.put("key1".to_string(), reg1, node_a()).unwrap();
        map.put("key2".to_string(), reg2, node_b()).unwrap();
        map.remove(&"key1".to_string(), node_a()).unwrap();

        let stats = map.stats();
        assert_eq!(stats.active_keys, 1);
        assert_eq!(stats.tombstoned_keys, 1);
        assert_eq!(stats.total_keys_seen, 2);
        assert!(stats.contributing_nodes >= 2);
    }

    #[test]
    fn test_crdt_map_gc_tombstones() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let reg = LWWRegister::with_initial_value("v1".to_string(), 42u64, node_a());
        map.put("key1".to_string(), reg, node_a()).unwrap();
        map.remove(&"key1".to_string(), node_a()).unwrap();

        assert_eq!(map.tombstones.len(), 1);

        // GC with high min_age (should keep tombstone)
        map.gc_tombstones(1000);
        assert_eq!(map.tombstones.len(), 1);

        // GC with low min_age (should remove tombstone)
        map.gc_tombstones(0);
        assert_eq!(map.tombstones.len(), 0);
    }

    #[test]
    fn test_crdt_map_value_json() {
        let mut map: TestMap = CRDTMap::new("test".to_string());

        let reg = LWWRegister::with_initial_value("v1".to_string(), 42u64, node_a());
        map.put("key1".to_string(), reg, node_a()).unwrap();

        let json_value = map.value();
        assert_eq!(json_value["active_keys"], 1);

        let entries = &json_value["entries"];
        assert!(entries.is_object());
        assert!(entries.get("\"key1\"").is_some());
    }
}
