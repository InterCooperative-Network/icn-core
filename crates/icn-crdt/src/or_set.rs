//! OR-Set (Observed-Remove Set) CRDT implementation.
//!
//! An OR-Set allows elements to be added and removed while ensuring that
//! concurrent operations converge to a consistent state. Elements can only
//! be removed if they were previously observed to be added. Perfect for
//! group memberships, federation memberships, and other set-based state.

use crate::{CRDTError, CRDTValue, CausalCRDT, NodeId, VectorClock, CRDT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// An observed-remove set that tracks both additions and removals.
///
/// Elements are uniquely identified by both their value and a unique tag
/// (timestamp + node ID). An element is considered present in the set if
/// there exists at least one add operation that hasn't been removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue + Eq + Hash")]
pub struct ORSet<T>
where
    T: CRDTValue + Eq + Hash,
{
    /// Unique identifier for this CRDT instance.
    id: String,
    /// Map from elements to sets of unique tags representing additions.
    added: HashMap<T, HashSet<ElementTag>>,
    /// Map from remove tags to (element, add_tag) pairs they remove.
    /// This allows remove operations to have their own timestamps while
    /// still tracking which specific add operations they're removing.
    removed: HashMap<ElementTag, (T, ElementTag)>,
    /// Vector clock for causality tracking.
    vector_clock: VectorClock,
    /// Counter for generating unique tags within this node.
    tag_counter: u64,
    /// Node ID for this instance.
    node_id: NodeId,
}

/// Unique tag for tracking individual add/remove operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementTag {
    /// Node that performed the operation.
    pub node_id: NodeId,
    /// Logical timestamp when the operation occurred.
    pub timestamp: u64,
    /// Sequence number for operations within the same timestamp.
    pub sequence: u64,
}

/// Operations that can be applied to an OR-Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue + Eq + Hash")]
pub enum ORSetOperation<T>
where
    T: CRDTValue + Eq + Hash,
{
    /// Add an element to the set with the given tag.
    Add { element: T, tag: ElementTag },
    /// Remove an element from the set with the given remove_tag.
    /// The remove_tag is a new timestamp representing when the removal occurred.
    Remove {
        element: T,
        remove_tag: ElementTag,
        add_tag: ElementTag,
    },
}

impl<T> ORSet<T>
where
    T: CRDTValue + Eq + Hash,
{
    /// Create a new OR-Set with the given ID and node ID.
    pub fn new(id: String, node_id: NodeId) -> Self {
        Self {
            id,
            added: HashMap::new(),
            removed: HashMap::new(),
            vector_clock: VectorClock::new(),
            tag_counter: 0,
            node_id,
        }
    }

    /// Check if an element is currently in the set.
    ///
    /// An element is present if there exists at least one add tag
    /// that doesn't have a corresponding remove tag.
    pub fn contains(&self, element: &T) -> bool {
        if let Some(add_tags) = self.added.get(element) {
            // Find all add tags that have been removed
            let mut removed_add_tags = HashSet::new();
            for (element_removed, add_tag_removed) in self.removed.values() {
                if element_removed == element {
                    removed_add_tags.insert(add_tag_removed);
                }
            }

            // Element is present if there are adds not covered by removes
            add_tags.iter().any(|tag| !removed_add_tags.contains(tag))
        } else {
            // Element has never been added
            false
        }
    }

    /// Get all elements currently in the set.
    pub fn elements(&self) -> HashSet<T> {
        let mut result = HashSet::new();

        for element in self.added.keys() {
            if self.contains(element) {
                result.insert(element.clone());
            }
        }

        result
    }

    /// Get the number of elements currently in the set.
    pub fn size(&self) -> usize {
        self.elements().len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Add an element to the set.
    ///
    /// Returns the tag used for this add operation, which can be used
    /// later for removal.
    pub fn add(&mut self, element: T) -> ElementTag {
        self.tag_counter += 1;
        self.vector_clock.increment(&self.node_id);

        let tag = ElementTag {
            node_id: self.node_id.clone(),
            timestamp: self.vector_clock.get(&self.node_id),
            sequence: self.tag_counter,
        };

        self.added
            .entry(element)
            .or_default()
            .insert(tag.clone());

        tag
    }

    /// Remove an element from the set.
    ///
    /// This will remove all currently present instances of the element
    /// by creating new remove operations with fresh timestamps that reference
    /// all current add tags.
    pub fn remove(&mut self, element: &T) -> Vec<ElementTag> {
        let mut removed_tags = Vec::new();

        if let Some(add_tags) = self.added.get(element) {
            // Find all add tags that haven't been removed yet
            let mut already_removed_add_tags = HashSet::new();
            for (element_removed, add_tag_removed) in self.removed.values() {
                if element_removed == element {
                    already_removed_add_tags.insert(add_tag_removed);
                }
            }

            // Collect add tags to remove (to avoid borrow checker issues)
            let tags_to_remove: Vec<_> = add_tags
                .iter()
                .filter(|add_tag| !already_removed_add_tags.contains(add_tag))
                .cloned()
                .collect();

            // Create new remove operations for each unremoved add tag
            for add_tag in tags_to_remove {
                // Generate a new timestamp for this remove operation
                self.tag_counter += 1;
                self.vector_clock.increment(&self.node_id);

                let remove_tag = ElementTag {
                    node_id: self.node_id.clone(),
                    timestamp: self.vector_clock.get(&self.node_id),
                    sequence: self.tag_counter,
                };

                // Store the remove operation
                self.removed
                    .insert(remove_tag.clone(), (element.clone(), add_tag));
                removed_tags.push(remove_tag);
            }
        }

        removed_tags
    }

    /// Remove a specific tagged instance of an element.
    ///
    /// This removes only the instance corresponding to the given add tag
    /// by creating a new remove operation with a fresh timestamp.
    pub fn remove_tag(&mut self, element: &T, add_tag: &ElementTag) -> Option<ElementTag> {
        // Check if the add tag exists
        if let Some(add_tags) = self.added.get(element) {
            if add_tags.contains(add_tag) {
                // Check if not already removed
                let already_removed = self
                    .removed
                    .values()
                    .any(|(elem, removed_add_tag)| elem == element && removed_add_tag == add_tag);

                if !already_removed {
                    // Generate a new timestamp for this remove operation
                    self.tag_counter += 1;
                    self.vector_clock.increment(&self.node_id);

                    let remove_tag = ElementTag {
                        node_id: self.node_id.clone(),
                        timestamp: self.vector_clock.get(&self.node_id),
                        sequence: self.tag_counter,
                    };

                    // Store the remove operation
                    self.removed
                        .insert(remove_tag.clone(), (element.clone(), add_tag.clone()));
                    return Some(remove_tag);
                }
            }
        }

        None
    }

    /// Get all add tags for an element.
    pub fn get_add_tags(&self, element: &T) -> HashSet<ElementTag> {
        self.added.get(element).cloned().unwrap_or_default()
    }

    /// Get all remove tags for an element.
    pub fn get_remove_tags(&self, element: &T) -> HashSet<ElementTag> {
        self.removed
            .iter()
            .filter_map(|(remove_tag, (elem, _add_tag))| {
                if elem == element {
                    Some(remove_tag.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all elements that have ever been added (including removed ones).
    pub fn all_known_elements(&self) -> HashSet<T> {
        self.added.keys().cloned().collect()
    }

    /// Check if an element has ever been added to the set.
    pub fn has_been_added(&self, element: &T) -> bool {
        self.added.contains_key(element)
    }

    /// Get statistics about this set.
    pub fn stats(&self) -> ORSetStats {
        let total_known = self.all_known_elements().len();
        let currently_present = self.size();
        let total_add_operations = self.added.values().map(|tags| tags.len()).sum::<usize>();
        let total_remove_operations = self.removed.len();

        ORSetStats {
            current_size: currently_present as u64,
            total_known_elements: total_known as u64,
            total_add_operations: total_add_operations as u64,
            total_remove_operations: total_remove_operations as u64,
            nodes_contributed: self.contributing_nodes().len() as u64,
        }
    }

    /// Get all nodes that have contributed operations to this set.
    pub fn contributing_nodes(&self) -> HashSet<NodeId> {
        let mut nodes = HashSet::new();

        for tags in self.added.values() {
            for tag in tags {
                nodes.insert(tag.node_id.clone());
            }
        }

        for remove_tag in self.removed.keys() {
            nodes.insert(remove_tag.node_id.clone());
        }

        nodes
    }

    /// Create a compact delta for synchronization containing only operations
    /// not seen by the given vector clock.
    pub fn delta_since(&self, other_clock: &VectorClock) -> ORSetDelta<T> {
        let mut add_ops = Vec::new();
        let mut remove_ops = Vec::new();

        // Find add operations newer than other_clock
        for (element, tags) in &self.added {
            for tag in tags {
                if other_clock.get(&tag.node_id) < tag.timestamp {
                    add_ops.push(ORSetOperation::Add {
                        element: element.clone(),
                        tag: tag.clone(),
                    });
                }
            }
        }

        // Find remove operations newer than other_clock
        // Now remove operations have their own timestamps, so we can properly
        // filter based on when the remove operation actually occurred
        for (remove_tag, (element, add_tag)) in &self.removed {
            if other_clock.get(&remove_tag.node_id) < remove_tag.timestamp {
                remove_ops.push(ORSetOperation::Remove {
                    element: element.clone(),
                    remove_tag: remove_tag.clone(),
                    add_tag: add_tag.clone(),
                });
            }
        }

        ORSetDelta {
            operations: [add_ops, remove_ops].concat(),
        }
    }
}

impl<T> CRDT for ORSet<T>
where
    T: CRDTValue + Eq + Hash,
{
    type Operation = ORSetOperation<T>;

    fn merge(&mut self, other: &Self) {
        // Merge all add operations
        for (element, other_tags) in &other.added {
            let current_tags = self
                .added
                .entry(element.clone())
                .or_default();
            current_tags.extend(other_tags.clone());
        }

        // Merge all remove operations
        for (remove_tag, (element, add_tag)) in &other.removed {
            self.removed
                .insert(remove_tag.clone(), (element.clone(), add_tag.clone()));
        }

        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);

        // Update tag counter to avoid conflicts
        self.tag_counter = self.tag_counter.max(other.tag_counter);
    }

    fn apply_operation(&mut self, op: Self::Operation) -> Result<(), CRDTError> {
        match op {
            ORSetOperation::Add { element, tag } => {
                self.added
                    .entry(element)
                    .or_default()
                    .insert(tag.clone());

                // Update vector clock based on the tag's timestamp
                let current_time = self.vector_clock.get(&tag.node_id);
                if tag.timestamp > current_time {
                    self.vector_clock.set(tag.node_id.clone(), tag.timestamp);
                }

                Ok(())
            }
            ORSetOperation::Remove {
                element,
                remove_tag,
                add_tag,
            } => {
                self.removed.insert(remove_tag.clone(), (element, add_tag));

                // Update vector clock based on the remove_tag's timestamp
                let current_time = self.vector_clock.get(&remove_tag.node_id);
                if remove_tag.timestamp > current_time {
                    self.vector_clock
                        .set(remove_tag.node_id.clone(), remove_tag.timestamp);
                }

                Ok(())
            }
        }
    }

    fn value(&self) -> serde_json::Value {
        let elements: Vec<_> = self.elements().into_iter().collect();
        serde_json::json!({
            "elements": elements,
            "size": self.size(),
            "stats": self.stats()
        })
    }

    fn crdt_id(&self) -> String {
        self.id.clone()
    }
}

impl<T> CausalCRDT for ORSet<T>
where
    T: CRDTValue + Eq + Hash,
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

impl<T> PartialEq for ORSet<T>
where
    T: CRDTValue + Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.added == other.added && self.removed == other.removed
    }
}

impl<T> Eq for ORSet<T> where T: CRDTValue + Eq + Hash {}

/// Delta containing operations for efficient synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: CRDTValue + Eq + Hash")]
pub struct ORSetDelta<T>
where
    T: CRDTValue + Eq + Hash,
{
    /// Operations to apply.
    pub operations: Vec<ORSetOperation<T>>,
}

/// Statistics about an OR-Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSetStats {
    /// Number of elements currently in the set.
    pub current_size: u64,
    /// Total number of distinct elements ever added.
    pub total_known_elements: u64,
    /// Total number of add operations performed.
    pub total_add_operations: u64,
    /// Total number of remove operations performed.
    pub total_remove_operations: u64,
    /// Number of nodes that have contributed operations.
    pub nodes_contributed: u64,
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

    fn alice_did() -> Did {
        Did::new("key", "alice")
    }

    fn bob_did() -> Did {
        Did::new("key", "bob")
    }

    fn charlie_did() -> Did {
        Did::new("key", "charlie")
    }

    #[test]
    fn test_orset_creation() {
        let set: ORSet<String> = ORSet::new("test_set".to_string(), node_a());
        assert_eq!(set.size(), 0);
        assert!(set.is_empty());
        assert_eq!(set.crdt_id(), "test_set");
    }

    #[test]
    fn test_orset_add_contains() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        assert!(!set.contains(&alice));

        let tag = set.add(alice.clone());
        assert!(set.contains(&alice));
        assert_eq!(set.size(), 1);
        assert!(!set.is_empty());

        // Check tag properties
        assert_eq!(tag.node_id, node_a());
        assert_eq!(tag.timestamp, 1);
        assert_eq!(tag.sequence, 1);

        set.add(bob.clone());
        assert_eq!(set.size(), 2);
        assert!(set.contains(&alice));
        assert!(set.contains(&bob));
    }

    #[test]
    fn test_orset_remove() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());
        assert_eq!(set.size(), 2);

        let remove_tags = set.remove(&alice);
        assert_eq!(remove_tags.len(), 1);
        assert!(!set.contains(&alice));
        assert!(set.contains(&bob));
        assert_eq!(set.size(), 1);

        // Remove non-existent element
        let empty_tags = set.remove(&charlie);
        assert!(empty_tags.is_empty());
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_orset_remove_tag() {
        let alice = "alice".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        let tag1 = set.add(alice.clone());
        let tag2 = set.add(alice.clone()); // Add same element again
        assert_eq!(set.size(), 1); // Still one element

        // Remove specific tag
        let remove_tag = set.remove_tag(&alice, &tag1);
        assert!(remove_tag.is_some());
        assert!(set.contains(&alice)); // Still present due to tag2

        // Remove the other tag
        let remove_tag2 = set.remove_tag(&alice, &tag2);
        assert!(remove_tag2.is_some());
        assert!(!set.contains(&alice)); // Now removed

        // Try to remove already removed tag
        let no_remove = set.remove_tag(&alice, &tag1);
        assert!(no_remove.is_none());
    }

    #[test]
    fn test_orset_elements() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());
        set.add(charlie.clone());
        set.remove(&bob);

        let elements = set.elements();
        assert_eq!(elements.len(), 2);
        assert!(elements.contains(&alice));
        assert!(!elements.contains(&bob));
        assert!(elements.contains(&charlie));
    }

    #[test]
    fn test_orset_merge() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set1: ORSet<String> = ORSet::new("test".to_string(), node_a());
        set1.add(alice.clone());
        set1.add(bob.clone());
        set1.remove(&bob);

        let mut set2: ORSet<String> = ORSet::new("test".to_string(), node_b());
        set2.add(bob.clone()); // Re-add bob
        set2.add(charlie.clone());

        set1.merge(&set2);

        let elements = set1.elements();
        assert_eq!(elements.len(), 3);
        assert!(elements.contains(&alice));
        assert!(elements.contains(&bob)); // Present due to set2's add
        assert!(elements.contains(&charlie));
    }

    #[test]
    fn test_orset_merge_idempotent() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        
        let mut set1: ORSet<String> = ORSet::new("test".to_string(), node_a());
        set1.add(alice.clone());
        set1.add(bob.clone());

        let set2 = set1.clone();
        let original_elements = set1.elements();

        set1.merge(&set2);
        assert_eq!(set1.elements(), original_elements);
    }

    #[test]
    fn test_orset_merge_commutative() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        
        let mut set1: ORSet<String> = ORSet::new("test".to_string(), node_a());
        set1.add(alice.clone());

        let mut set2: ORSet<String> = ORSet::new("test".to_string(), node_b());
        set2.add(bob.clone());

        let set1_copy = set1.clone();
        let _set2_copy = set2.clone();

        // set1.merge(set2)
        set1.merge(&set2);

        // set2.merge(set1_copy)
        set2.merge(&set1_copy);

        assert_eq!(set1.elements(), set2.elements());
    }

    #[test]
    fn test_orset_apply_operations() {
        let alice = "alice".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        let tag = ElementTag {
            node_id: node_b(),
            timestamp: 5,
            sequence: 1,
        };

        let add_op = ORSetOperation::Add {
            element: alice.clone(),
            tag: tag.clone(),
        };

        set.apply_operation(add_op).unwrap();
        assert!(set.contains(&alice));
        assert_eq!(set.vector_clock().get(&node_b()), 5);

        let remove_op = ORSetOperation::Remove {
            element: alice.clone(),
            remove_tag: tag.clone(),
            add_tag: tag.clone(),
        };

        set.apply_operation(remove_op).unwrap();
        assert!(!set.contains(&alice));
    }

    #[test]
    fn test_orset_with_dids() {
        let mut members: ORSet<Did> = ORSet::new("group_members".to_string(), node_a());

        members.add(alice_did());
        members.add(bob_did());
        assert_eq!(members.size(), 2);

        members.remove(&alice_did());
        assert_eq!(members.size(), 1);
        assert!(members.contains(&bob_did()));
        assert!(!members.contains(&alice_did()));

        members.add(charlie_did());
        assert_eq!(members.size(), 2);

        let all_members = members.elements();
        assert!(all_members.contains(&bob_did()));
        assert!(all_members.contains(&charlie_did()));
    }

    #[test]
    fn test_orset_stats() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());
        set.add(charlie.clone());
        set.remove(&bob);

        let stats = set.stats();
        assert_eq!(stats.current_size, 2);
        assert_eq!(stats.total_known_elements, 3);
        assert_eq!(stats.total_add_operations, 3);
        assert_eq!(stats.total_remove_operations, 1);
        assert_eq!(stats.nodes_contributed, 1); // Only node_a contributed
    }

    #[test]
    fn test_orset_known_elements() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());
        set.remove(&bob);

        let known = set.all_known_elements();
        assert_eq!(known.len(), 2);
        assert!(known.contains(&alice));
        assert!(known.contains(&bob));

        assert!(set.has_been_added(&alice));
        assert!(set.has_been_added(&bob));
        assert!(!set.has_been_added(&charlie));
    }

    #[test]
    fn test_orset_contributing_nodes() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        // Add from node_a
        set.add(alice.clone());

        // Simulate operation from node_b
        let tag_b = ElementTag {
            node_id: node_b(),
            timestamp: 1,
            sequence: 1,
        };
        set.apply_operation(ORSetOperation::Add {
            element: bob.clone(),
            tag: tag_b,
        })
        .unwrap();

        let contributing = set.contributing_nodes();
        assert_eq!(contributing.len(), 2);
        assert!(contributing.contains(&node_a()));
        assert!(contributing.contains(&node_b()));
    }

    #[test]
    fn test_orset_delta_since() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let charlie = "charlie".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());

        let old_clock = set.vector_clock().clone();

        set.add(charlie.clone());
        set.remove(&bob);

        let delta = set.delta_since(&old_clock);
        assert_eq!(delta.operations.len(), 2); // Both new add and remove operations

        // Check that operations are for new changes
        let has_charlie_add = delta.operations.iter().any(|op| {
            matches!(op, ORSetOperation::Add { element, .. } if element == &charlie)
        });

        let has_bob_remove = delta.operations.iter().any(|op| {
            matches!(op, ORSetOperation::Remove { element, .. } if element == &bob)
        });

        assert!(has_charlie_add);
        assert!(has_bob_remove);
    }

    #[test]
    fn test_orset_value_json() {
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        
        let mut set: ORSet<String> = ORSet::new("test".to_string(), node_a());

        set.add(alice.clone());
        set.add(bob.clone());
        set.remove(&bob);

        let json_value = set.value();
        assert_eq!(json_value["size"], 1);

        let elements = json_value["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 1);
        assert!(elements.contains(&serde_json::Value::String(alice.clone())));
    }
}
