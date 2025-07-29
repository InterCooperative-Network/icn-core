//! CRDT-backed group membership for conflict-free distributed group management.
//!
//! This module provides group membership management using CRDTs to ensure
//! conflict-free replication across multiple nodes. Groups are managed using
//! OR-Sets (Observed-Remove Sets) that allow concurrent additions and removals
//! of members without conflicts.

use icn_common::{CommonError, Did};
use icn_crdt::{CRDTMap, NodeId, ORSet, CRDT};
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// CRDT-backed group membership manager that enables conflict-free distributed group management.
///
/// Uses a CRDT Map where each group ID maps to an OR-Set of member DIDs.
/// This allows multiple nodes to concurrently add/remove members without conflicts.
pub struct CRDTGroupMembership {
    /// Node identifier for this group membership instance.
    node_id: NodeId,
    /// CRDT Map storing group_id -> `OR-Set<DID>` mappings for group memberships.
    membership_map: Arc<RwLock<CRDTMap<String, ORSet<String>>>>,
    /// Configuration for group membership.
    config: CRDTGroupMembershipConfig,
}

/// Configuration for CRDT group membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTGroupMembershipConfig {
    /// Node identifier for this membership instance.
    pub node_id: String,
    /// Initial group memberships (for bootstrapping).
    pub initial_groups: HashMap<String, Vec<String>>,
    /// Maximum number of members per group (0 = unlimited).
    pub max_members_per_group: usize,
    /// Whether to enable automatic group creation.
    pub auto_create_groups: bool,
}

impl Default for CRDTGroupMembershipConfig {
    fn default() -> Self {
        Self {
            node_id: "default_membership_node".to_string(),
            initial_groups: HashMap::new(),
            max_members_per_group: 0, // Unlimited
            auto_create_groups: true,
        }
    }
}

/// Statistics about a CRDT group membership manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTGroupMembershipStats {
    /// Number of groups managed.
    pub group_count: u64,
    /// Total number of memberships across all groups.
    pub total_memberships: u64,
    /// Average members per group.
    pub average_members_per_group: u64,
    /// Largest group size.
    pub max_group_size: u64,
    /// Smallest group size.
    pub min_group_size: u64,
    /// Node ID of this membership manager instance.
    pub node_id: NodeId,
}

/// Information about a specific group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    /// Group identifier.
    pub group_id: String,
    /// List of member DIDs.
    pub members: Vec<Did>,
    /// Number of members.
    pub member_count: usize,
    /// Timestamp of last membership change.
    pub last_updated: u64,
}

impl CRDTGroupMembership {
    /// Create a new CRDT group membership manager with the given configuration.
    pub fn new(config: CRDTGroupMembershipConfig) -> Self {
        let node_id = NodeId::new(config.node_id.clone());
        let membership_map = CRDTMap::new("group_memberships".to_string());

        let manager = Self {
            node_id,
            membership_map: Arc::new(RwLock::new(membership_map)),
            config,
        };

        // Initialize with provided groups
        for (group_id, member_strs) in &manager.config.initial_groups {
            let members: Vec<Did> = member_strs
                .iter()
                .filter_map(|s| Did::from_str(s).ok())
                .collect();

            if let Err(e) = manager.create_group_with_members(group_id, &members) {
                warn!("Failed to create initial group {group_id}: {e}");
            }
        }

        manager
    }

    /// Create a new CRDT group membership manager with a specific node ID.
    pub fn with_node_id(node_id: String) -> Self {
        Self::new(CRDTGroupMembershipConfig {
            node_id,
            ..Default::default()
        })
    }

    /// Get the node ID for this membership manager instance.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Merge state from another CRDT group membership manager.
    ///
    /// This enables synchronization between distributed membership manager instances.
    pub fn merge(&self, other: &Self) -> Result<(), CommonError> {
        let mut our_map = self
            .membership_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let other_map = other
            .membership_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        our_map.merge(&*other_map);

        debug!(
            "Merged CRDT group membership state from node {}",
            other.node_id
        );
        Ok(())
    }

    /// Create a new group.
    pub fn create_group(&self, group_id: &str) -> Result<(), CommonError> {
        debug!("Creating group: {group_id}");

        if self.group_exists(group_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Group {group_id} already exists"
            )));
        }

        let mut map = self
            .membership_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let or_set_id = format!("group_{group_id}");
        let or_set = ORSet::new(or_set_id, self.node_id.clone());

        map.put(group_id.to_string(), or_set, self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to create group: {e}")))?;

        debug!("Successfully created group: {group_id}");
        Ok(())
    }

    /// Create a group with initial members.
    pub fn create_group_with_members(
        &self,
        group_id: &str,
        members: &[Did],
    ) -> Result<(), CommonError> {
        self.create_group(group_id)?;

        for member in members {
            self.add_member(group_id, member)?;
        }

        debug!(
            "Created group {} with {} initial members",
            group_id,
            members.len()
        );
        Ok(())
    }

    /// Add a member to a group.
    pub fn add_member(&self, group_id: &str, member: &Did) -> Result<(), CommonError> {
        debug!("Adding member {member} to group {group_id}");

        // Auto-create group if enabled and group doesn't exist
        if !self.group_exists(group_id) {
            if self.config.auto_create_groups {
                self.create_group(group_id)?;
            } else {
                return Err(CommonError::InvalidInputError(format!(
                    "Group {group_id} does not exist"
                )));
            }
        }

        // Check member limit
        if self.config.max_members_per_group > 0 {
            let current_size = self.get_member_count(group_id);
            if current_size >= self.config.max_members_per_group {
                return Err(CommonError::InvalidInputError(format!(
                    "Group {} is at maximum capacity ({} members)",
                    group_id, self.config.max_members_per_group
                )));
            }
        }

        // Get or create the OR-Set for this group
        let mut or_set = self.get_or_create_or_set(group_id)?;

        // Add the member
        or_set.add(member.to_string());

        // Update the OR-Set in the map
        self.update_or_set(group_id, or_set)?;

        debug!("Successfully added member {member} to group {group_id}");
        Ok(())
    }

    /// Remove a member from a group.
    pub fn remove_member(&self, group_id: &str, member: &Did) -> Result<(), CommonError> {
        debug!("Removing member {member} from group {group_id}");

        if !self.group_exists(group_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Group {group_id} does not exist"
            )));
        }

        // Get the OR-Set for this group
        let mut or_set = self.get_or_create_or_set(group_id)?;

        // Remove the member
        or_set.remove(&member.to_string());

        // Update the OR-Set in the map
        self.update_or_set(group_id, or_set)?;

        debug!(
            "Successfully removed member {member} from group {group_id}"
        );
        Ok(())
    }

    /// Check if a DID is a member of a group.
    pub fn is_member(&self, group_id: &str, member: &Did) -> bool {
        match self.membership_map.read() {
            Ok(map) => {
                if let Some(or_set) = map.get(&group_id.to_string()) {
                    or_set.contains(&member.to_string())
                } else {
                    false
                }
            }
            Err(_) => {
                error!("Failed to acquire read lock for membership check");
                false
            }
        }
    }

    /// Get all members of a group.
    pub fn get_members(&self, group_id: &str) -> Vec<Did> {
        match self.membership_map.read() {
            Ok(map) => {
                if let Some(or_set) = map.get(&group_id.to_string()) {
                    or_set
                        .elements()
                        .iter()
                        .filter_map(|s| Did::from_str(s).ok())
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Err(_) => {
                error!("Failed to acquire read lock for member listing");
                Vec::new()
            }
        }
    }

    /// Get the number of members in a group.
    pub fn get_member_count(&self, group_id: &str) -> usize {
        match self.membership_map.read() {
            Ok(map) => {
                if let Some(or_set) = map.get(&group_id.to_string()) {
                    or_set.size()
                } else {
                    0
                }
            }
            Err(_) => {
                error!("Failed to acquire read lock for member count");
                0
            }
        }
    }

    /// Check if a group exists.
    pub fn group_exists(&self, group_id: &str) -> bool {
        match self.membership_map.read() {
            Ok(map) => map.contains_key(&group_id.to_string()),
            Err(_) => {
                error!("Failed to acquire read lock for group existence check");
                false
            }
        }
    }

    /// Get all group IDs.
    pub fn get_all_groups(&self) -> Vec<String> {
        match self.membership_map.read() {
            Ok(map) => map.keys().into_iter().collect(),
            Err(_) => {
                error!("Failed to acquire read lock for group listing");
                Vec::new()
            }
        }
    }

    /// Get information about a specific group.
    pub fn get_group_info(&self, group_id: &str) -> Option<GroupInfo> {
        if !self.group_exists(group_id) {
            return None;
        }

        let members = self.get_members(group_id);
        let member_count = members.len();

        Some(GroupInfo {
            group_id: group_id.to_string(),
            members,
            member_count,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Get all groups that a DID is a member of.
    pub fn get_member_groups(&self, member: &Did) -> Vec<String> {
        let mut groups = Vec::new();

        for group_id in self.get_all_groups() {
            if self.is_member(&group_id, member) {
                groups.push(group_id);
            }
        }

        groups
    }

    /// Remove a group entirely.
    pub fn remove_group(&self, group_id: &str) -> Result<(), CommonError> {
        debug!("Removing group: {group_id}");

        if !self.group_exists(group_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Group {group_id} does not exist"
            )));
        }

        let mut map = self
            .membership_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        map.remove(&group_id.to_string(), self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to remove group: {e}")))?;

        debug!("Successfully removed group: {group_id}");
        Ok(())
    }

    /// Get statistics about the group membership manager.
    pub fn get_stats(&self) -> Result<CRDTGroupMembershipStats, CommonError> {
        let groups = self.get_all_groups();
        let group_count = groups.len() as u64;

        let mut total_memberships = 0u64;
        let mut group_sizes = Vec::new();

        for group_id in &groups {
            let size = self.get_member_count(group_id) as u64;
            total_memberships += size;
            group_sizes.push(size);
        }

        let average_members_per_group = if group_count > 0 {
            total_memberships / group_count
        } else {
            0
        };

        let max_group_size = group_sizes.iter().max().copied().unwrap_or(0);
        let min_group_size = group_sizes.iter().min().copied().unwrap_or(0);

        Ok(CRDTGroupMembershipStats {
            group_count,
            total_memberships,
            average_members_per_group,
            max_group_size,
            min_group_size,
            node_id: self.node_id.clone(),
        })
    }

    /// Get or create an OR-Set for the given group.
    fn get_or_create_or_set(&self, group_id: &str) -> Result<ORSet<String>, CommonError> {
        let mut map = self
            .membership_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        if let Some(or_set) = map.get(&group_id.to_string()) {
            Ok(or_set.clone())
        } else {
            // Create new OR-Set for this group
            let or_set_id = format!("group_{group_id}");
            let or_set = ORSet::new(or_set_id, self.node_id.clone());

            map.put(group_id.to_string(), or_set.clone(), self.node_id.clone())
                .map_err(|e| {
                    CommonError::CRDTError(format!("Failed to create group OR-Set: {e}"))
                })?;

            debug!("Created new OR-Set for group: {group_id}");
            Ok(or_set)
        }
    }

    /// Update an OR-Set in the map after modification.
    fn update_or_set(&self, group_id: &str, or_set: ORSet<String>) -> Result<(), CommonError> {
        let mut map = self
            .membership_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        map.put(group_id.to_string(), or_set, self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to update group OR-Set: {e}")))?;

        Ok(())
    }
}

impl Clone for CRDTGroupMembership {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            membership_map: self.membership_map.clone(),
            config: self.config.clone(),
        }
    }
}

impl std::fmt::Debug for CRDTGroupMembership {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRDTGroupMembership")
            .field("node_id", &self.node_id)
            .field("membership_map", &"<CRDTMap>")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn alice_did() -> Did {
        Did::from_str("did:key:alice").unwrap()
    }

    fn bob_did() -> Did {
        Did::from_str("did:key:bob").unwrap()
    }

    fn charlie_did() -> Did {
        Did::from_str("did:key:charlie").unwrap()
    }

    #[test]
    fn test_crdt_group_membership_creation() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());
        assert_eq!(manager.node_id().as_str(), "test_node");
        assert_eq!(manager.get_all_groups().len(), 0);
    }

    #[test]
    fn test_crdt_group_membership_initial_groups() {
        let mut initial_groups = HashMap::new();
        initial_groups.insert(
            "admins".to_string(),
            vec!["did:key:alice".to_string(), "did:key:bob".to_string()],
        );
        initial_groups.insert("users".to_string(), vec!["did:key:charlie".to_string()]);

        let config = CRDTGroupMembershipConfig {
            node_id: "test_node".to_string(),
            initial_groups,
            ..Default::default()
        };

        let manager = CRDTGroupMembership::new(config);

        assert!(manager.group_exists("admins"));
        assert!(manager.group_exists("users"));
        assert!(manager.is_member("admins", &alice_did()));
        assert!(manager.is_member("admins", &bob_did()));
        assert!(manager.is_member("users", &charlie_did()));
        assert!(!manager.is_member("users", &alice_did()));
    }

    #[test]
    fn test_crdt_group_membership_create_group() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        // Create a new group
        manager.create_group("developers").unwrap();
        assert!(manager.group_exists("developers"));
        assert_eq!(manager.get_member_count("developers"), 0);

        // Try to create duplicate group
        let result = manager.create_group("developers");
        assert!(result.is_err());
    }

    #[test]
    fn test_crdt_group_membership_add_remove_members() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        // Create group and add members
        manager.create_group("team").unwrap();
        manager.add_member("team", &alice_did()).unwrap();
        manager.add_member("team", &bob_did()).unwrap();

        assert!(manager.is_member("team", &alice_did()));
        assert!(manager.is_member("team", &bob_did()));
        assert!(!manager.is_member("team", &charlie_did()));
        assert_eq!(manager.get_member_count("team"), 2);

        // Remove a member
        manager.remove_member("team", &alice_did()).unwrap();
        assert!(!manager.is_member("team", &alice_did()));
        assert!(manager.is_member("team", &bob_did()));
        assert_eq!(manager.get_member_count("team"), 1);
    }

    #[test]
    fn test_crdt_group_membership_auto_create() {
        let mut config = CRDTGroupMembershipConfig::default();
        config.node_id = "test_node".to_string();
        config.auto_create_groups = true;

        let manager = CRDTGroupMembership::new(config);

        // Add member to non-existent group (should auto-create)
        manager.add_member("auto_group", &alice_did()).unwrap();
        assert!(manager.group_exists("auto_group"));
        assert!(manager.is_member("auto_group", &alice_did()));
    }

    #[test]
    fn test_crdt_group_membership_max_members() {
        let mut config = CRDTGroupMembershipConfig::default();
        config.node_id = "test_node".to_string();
        config.max_members_per_group = 2;

        let manager = CRDTGroupMembership::new(config);

        manager.create_group("limited").unwrap();
        manager.add_member("limited", &alice_did()).unwrap();
        manager.add_member("limited", &bob_did()).unwrap();

        // Third member should fail
        let result = manager.add_member("limited", &charlie_did());
        assert!(result.is_err());
        assert_eq!(manager.get_member_count("limited"), 2);
    }

    #[test]
    fn test_crdt_group_membership_get_members() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        manager.create_group("test_group").unwrap();
        manager.add_member("test_group", &alice_did()).unwrap();
        manager.add_member("test_group", &bob_did()).unwrap();

        let members = manager.get_members("test_group");
        assert_eq!(members.len(), 2);
        assert!(members.contains(&alice_did()));
        assert!(members.contains(&bob_did()));
    }

    #[test]
    fn test_crdt_group_membership_get_member_groups() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        manager.create_group("group1").unwrap();
        manager.create_group("group2").unwrap();
        manager.add_member("group1", &alice_did()).unwrap();
        manager.add_member("group2", &alice_did()).unwrap();
        manager.add_member("group1", &bob_did()).unwrap();

        let alice_groups = manager.get_member_groups(&alice_did());
        assert_eq!(alice_groups.len(), 2);
        assert!(alice_groups.contains(&"group1".to_string()));
        assert!(alice_groups.contains(&"group2".to_string()));

        let bob_groups = manager.get_member_groups(&bob_did());
        assert_eq!(bob_groups.len(), 1);
        assert!(bob_groups.contains(&"group1".to_string()));
    }

    #[test]
    fn test_crdt_group_membership_merge() {
        let manager1 = CRDTGroupMembership::with_node_id("node1".to_string());
        let manager2 = CRDTGroupMembership::with_node_id("node2".to_string());

        // Each manager creates different groups and members
        manager1.create_group("group1").unwrap();
        manager1.add_member("group1", &alice_did()).unwrap();

        manager2.create_group("group1").unwrap();
        manager2.add_member("group1", &bob_did()).unwrap();
        manager2.create_group("group2").unwrap();
        manager2.add_member("group2", &charlie_did()).unwrap();

        // Before merge
        assert!(manager1.is_member("group1", &alice_did()));
        assert!(!manager1.is_member("group1", &bob_did()));
        assert!(!manager1.group_exists("group2"));

        // Merge manager2 into manager1
        manager1.merge(&manager2).unwrap();

        // After merge, manager1 should have all members and groups
        assert!(manager1.is_member("group1", &alice_did()));
        assert!(manager1.is_member("group1", &bob_did()));
        assert!(manager1.group_exists("group2"));
        assert!(manager1.is_member("group2", &charlie_did()));
    }

    #[test]
    fn test_crdt_group_membership_remove_group() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        manager.create_group("temp_group").unwrap();
        manager.add_member("temp_group", &alice_did()).unwrap();

        assert!(manager.group_exists("temp_group"));
        assert!(manager.is_member("temp_group", &alice_did()));

        manager.remove_group("temp_group").unwrap();

        assert!(!manager.group_exists("temp_group"));
        assert!(!manager.is_member("temp_group", &alice_did()));
    }

    #[test]
    fn test_crdt_group_membership_stats() {
        let manager = CRDTGroupMembership::with_node_id("test_node".to_string());

        manager.create_group("group1").unwrap();
        manager.add_member("group1", &alice_did()).unwrap();
        manager.add_member("group1", &bob_did()).unwrap();

        manager.create_group("group2").unwrap();
        manager.add_member("group2", &charlie_did()).unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.group_count, 2);
        assert_eq!(stats.total_memberships, 3);
        assert_eq!(stats.average_members_per_group, 1); // 3/2 = 1.5, rounded down
        assert_eq!(stats.max_group_size, 2);
        assert_eq!(stats.min_group_size, 1);
        assert_eq!(stats.node_id.as_str(), "test_node");
    }

    #[test]
    fn test_crdt_group_membership_concurrent_operations() {
        let manager1 = CRDTGroupMembership::with_node_id("node1".to_string());
        let manager2 = CRDTGroupMembership::with_node_id("node2".to_string());

        // Both managers work on the same group concurrently
        manager1.create_group("shared_group").unwrap();
        manager2.create_group("shared_group").unwrap();

        // Concurrent additions
        manager1.add_member("shared_group", &alice_did()).unwrap();
        manager1.add_member("shared_group", &bob_did()).unwrap();

        manager2.add_member("shared_group", &charlie_did()).unwrap();
        manager2.remove_member("shared_group", &bob_did()).unwrap(); // This won't affect anything since bob wasn't added by manager2

        // Before merge
        assert_eq!(manager1.get_member_count("shared_group"), 2);
        assert_eq!(manager2.get_member_count("shared_group"), 1);

        // Merge the states
        manager1.merge(&manager2).unwrap();

        // After merge, all additions should be preserved
        assert!(manager1.is_member("shared_group", &alice_did()));
        assert!(manager1.is_member("shared_group", &bob_did()));
        assert!(manager1.is_member("shared_group", &charlie_did()));
        assert_eq!(manager1.get_member_count("shared_group"), 3);
    }
}
