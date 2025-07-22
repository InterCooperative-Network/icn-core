//! Federation management system for ICN
//!
//! This module provides comprehensive federation management capabilities including
//! membership tracking, capability management, and federation discovery.

use crate::DidResolver;
use icn_common::{CommonError, Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Information about a federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationInfo {
    /// Unique federation identifier
    pub federation_id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the federation's purpose
    pub description: String,
    /// Federation creator/administrator
    pub admin_did: Did,
    /// When the federation was created
    pub created_at: u64,
    /// Federation type/category
    pub federation_type: FederationType,
    /// Geographic scope if applicable
    pub geographic_scope: Option<GeographicScope>,
    /// Membership policy
    pub membership_policy: MembershipPolicy,
    /// Required capabilities for members
    pub required_capabilities: Vec<String>,
    /// Federation status
    pub status: FederationStatus,
    /// Public metadata
    pub metadata: HashMap<String, String>,
}

/// Types of federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationType {
    /// Computing resource sharing
    Compute,
    /// Data sharing and storage
    Data,
    /// Governance and decision-making
    Governance,
    /// Economic/financial cooperation
    Economic,
    /// Research and development
    Research,
    /// General purpose cooperation
    General,
    /// Custom federation type
    Custom(String),
}

/// Geographic scope for federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeographicScope {
    Global,
    Continental(String),
    Country(String),
    Region(String),
    City(String),
    Local,
}

/// Membership policy for federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MembershipPolicy {
    /// Open to all nodes
    Open,
    /// Requires invitation from existing members
    InviteOnly,
    /// Requires approval from federation admin
    AdminApproval,
    /// Requires consensus from existing members
    Consensus,
    /// Custom policy with specific requirements
    Custom {
        requirements: Vec<String>,
        approval_threshold: f64,
    },
}

/// Federation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationStatus {
    Active,
    Inactive,
    Suspended,
    Disbanded,
}

/// Member status within a federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MembershipStatus {
    /// Active member with full rights
    Active,
    /// Pending approval/activation
    Pending,
    /// Suspended member
    Suspended,
    /// Member has left the federation
    Left,
    /// Member was removed/banned
    Removed,
}

/// Capabilities that a federation member can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationCapabilities {
    /// Computational capabilities
    pub compute: ComputeCapabilities,
    /// Storage capabilities  
    pub storage: StorageCapabilities,
    /// Network capabilities
    pub network: NetworkCapabilities,
    /// Specialized services
    pub services: Vec<String>,
    /// Geographic presence
    pub geographic_presence: Vec<GeographicScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeCapabilities {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_units: u32,
    pub specialized_hardware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageCapabilities {
    pub total_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub storage_types: Vec<String>,
    pub replication_factor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkCapabilities {
    pub bandwidth_mbps: u32,
    pub latency_ms: u32,
    pub protocols: Vec<String>,
    pub regions: Vec<String>,
}

/// Member information within a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMember {
    pub did: Did,
    pub joined_at: u64,
    pub status: MembershipStatus,
    pub capabilities: FederationCapabilities,
    pub reputation_score: u64,
    pub last_seen: u64,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Registry for managing federations
pub trait FederationRegistry: Send + Sync {
    /// Create a new federation
    fn create_federation(&self, info: FederationInfo) -> Result<(), CommonError>;

    /// Get federation information
    fn get_federation(&self, federation_id: &str) -> Option<FederationInfo>;

    /// Update federation information
    fn update_federation(&self, info: FederationInfo) -> Result<(), CommonError>;

    /// List all federations
    fn list_federations(&self) -> Vec<FederationInfo>;

    /// Search federations by criteria
    fn search_federations(&self, query: &FederationSearchQuery) -> Vec<FederationInfo>;

    /// Add member to federation
    fn add_member(&self, federation_id: &str, member: FederationMember) -> Result<(), CommonError>;

    /// Remove member from federation
    fn remove_member(&self, federation_id: &str, did: &Did) -> Result<(), CommonError>;

    /// Get federation members
    fn get_members(&self, federation_id: &str) -> Vec<FederationMember>;

    /// Get federations for a specific DID
    fn get_federations_for_did(&self, did: &Did) -> Vec<String>;

    /// Update member capabilities
    fn update_member_capabilities(
        &self,
        federation_id: &str,
        did: &Did,
        capabilities: FederationCapabilities,
    ) -> Result<(), CommonError>;
}

/// Query for searching federations
#[derive(Debug, Clone, Default)]
pub struct FederationSearchQuery {
    pub federation_type: Option<FederationType>,
    pub geographic_scope: Option<GeographicScope>,
    pub required_capabilities: Vec<String>,
    pub status: Option<FederationStatus>,
    pub keyword: Option<String>,
}

/// In-memory implementation of FederationRegistry
pub struct InMemoryFederationRegistry {
    federations: Arc<RwLock<HashMap<String, FederationInfo>>>,
    members: Arc<RwLock<HashMap<String, Vec<FederationMember>>>>,
    did_to_federations: Arc<RwLock<HashMap<Did, HashSet<String>>>>,
}

impl InMemoryFederationRegistry {
    pub fn new() -> Self {
        Self {
            federations: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            did_to_federations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryFederationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationRegistry for InMemoryFederationRegistry {
    fn create_federation(&self, info: FederationInfo) -> Result<(), CommonError> {
        let mut federations = self
            .federations
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        if federations.contains_key(&info.federation_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Federation {} already exists",
                info.federation_id
            )));
        }

        federations.insert(info.federation_id.clone(), info);
        Ok(())
    }

    fn get_federation(&self, federation_id: &str) -> Option<FederationInfo> {
        self.federations.read().ok()?.get(federation_id).cloned()
    }

    fn update_federation(&self, info: FederationInfo) -> Result<(), CommonError> {
        let mut federations = self
            .federations
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        federations.insert(info.federation_id.clone(), info);
        Ok(())
    }

    fn list_federations(&self) -> Vec<FederationInfo> {
        self.federations
            .read()
            .map(|f| f.values().cloned().collect())
            .unwrap_or_default()
    }

    fn search_federations(&self, query: &FederationSearchQuery) -> Vec<FederationInfo> {
        let federations = match self.federations.read() {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        federations
            .values()
            .filter(|fed| {
                // Filter by type
                if let Some(ref fed_type) = query.federation_type {
                    if &fed.federation_type != fed_type {
                        return false;
                    }
                }

                // Filter by geographic scope
                if let Some(ref scope) = query.geographic_scope {
                    if fed.geographic_scope.as_ref() != Some(scope) {
                        return false;
                    }
                }

                // Filter by status
                if let Some(ref status) = query.status {
                    if &fed.status != status {
                        return false;
                    }
                }

                // Filter by keyword
                if let Some(ref keyword) = query.keyword {
                    let keyword_lower = keyword.to_lowercase();
                    if !fed.name.to_lowercase().contains(&keyword_lower)
                        && !fed.description.to_lowercase().contains(&keyword_lower)
                    {
                        return false;
                    }
                }

                // Filter by required capabilities
                if !query.required_capabilities.is_empty() {
                    for req_cap in &query.required_capabilities {
                        if !fed.required_capabilities.contains(req_cap) {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    fn add_member(&self, federation_id: &str, member: FederationMember) -> Result<(), CommonError> {
        // Check if federation exists
        if self.get_federation(federation_id).is_none() {
            return Err(CommonError::InvalidInputError(format!(
                "Federation {} does not exist",
                federation_id
            )));
        }

        let mut members = self
            .members
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        let mut did_to_federations = self
            .did_to_federations
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        // Add to members list
        members
            .entry(federation_id.to_string())
            .or_insert_with(Vec::new)
            .push(member.clone());

        // Add to DID->federations mapping
        did_to_federations
            .entry(member.did.clone())
            .or_insert_with(HashSet::new)
            .insert(federation_id.to_string());

        Ok(())
    }

    fn remove_member(&self, federation_id: &str, did: &Did) -> Result<(), CommonError> {
        let mut members = self
            .members
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        let mut did_to_federations = self
            .did_to_federations
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        // Remove from members list
        if let Some(member_list) = members.get_mut(federation_id) {
            member_list.retain(|m| &m.did != did);
        }

        // Remove from DID->federations mapping
        if let Some(fed_set) = did_to_federations.get_mut(did) {
            fed_set.remove(federation_id);
            if fed_set.is_empty() {
                did_to_federations.remove(did);
            }
        }

        Ok(())
    }

    fn get_members(&self, federation_id: &str) -> Vec<FederationMember> {
        self.members
            .read()
            .ok()
            .and_then(|m| m.get(federation_id).cloned())
            .unwrap_or_default()
    }

    fn get_federations_for_did(&self, did: &Did) -> Vec<String> {
        self.did_to_federations
            .read()
            .ok()
            .and_then(|d| d.get(did).cloned())
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }

    fn update_member_capabilities(
        &self,
        federation_id: &str,
        did: &Did,
        capabilities: FederationCapabilities,
    ) -> Result<(), CommonError> {
        let mut members = self
            .members
            .write()
            .map_err(|_| CommonError::InternalError("Failed to acquire write lock".to_string()))?;

        if let Some(member_list) = members.get_mut(federation_id) {
            for member in member_list {
                if &member.did == did {
                    member.capabilities = capabilities;
                    return Ok(());
                }
            }
        }

        Err(CommonError::InvalidInputError(format!(
            "Member {} not found in federation {}",
            did, federation_id
        )))
    }
}

/// Service for managing federation membership
pub trait FederationMembershipService: Send + Sync {
    /// Get federations for a DID
    fn get_federations(&self, did: &Did) -> Vec<String>;

    /// Get capabilities for a DID in a specific federation
    fn get_capabilities(&self, did: &Did, federation_id: &str) -> Option<FederationCapabilities>;

    /// Check if a DID is a member of a federation
    fn is_member(&self, did: &Did, federation_id: &str) -> bool;

    /// Get trust scope for a DID (federation-based)
    fn get_trust_scope(&self, did: &Did) -> Option<String>;

    /// Update member presence
    fn update_presence(&self, did: &Did, timestamp: u64) -> Result<(), CommonError>;

    /// Get member reputation in context
    fn get_contextual_reputation(&self, did: &Did, federation_id: &str) -> Option<u64>;
}

/// Implementation of FederationMembershipService
pub struct DefaultFederationMembershipService {
    registry: Arc<dyn FederationRegistry>,
    time_provider: Arc<dyn TimeProvider>,
}

impl DefaultFederationMembershipService {
    pub fn new(
        registry: Arc<dyn FederationRegistry>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            registry,
            time_provider,
        }
    }
}

impl FederationMembershipService for DefaultFederationMembershipService {
    fn get_federations(&self, did: &Did) -> Vec<String> {
        self.registry.get_federations_for_did(did)
    }

    fn get_capabilities(&self, did: &Did, federation_id: &str) -> Option<FederationCapabilities> {
        let members = self.registry.get_members(federation_id);
        members
            .iter()
            .find(|m| &m.did == did)
            .map(|m| m.capabilities.clone())
    }

    fn is_member(&self, did: &Did, federation_id: &str) -> bool {
        let members = self.registry.get_members(federation_id);
        members
            .iter()
            .any(|m| &m.did == did && m.status == MembershipStatus::Active)
    }

    fn get_trust_scope(&self, did: &Did) -> Option<String> {
        let federations = self.get_federations(did);

        // Return the primary federation as trust scope
        // In a more sophisticated implementation, this could be context-dependent
        federations.first().cloned()
    }

    fn update_presence(&self, did: &Did, timestamp: u64) -> Result<(), CommonError> {
        let federations = self.get_federations(did);

        for federation_id in federations {
            let members = self.registry.get_members(&federation_id);
            for mut member in members {
                if &member.did == did {
                    member.last_seen = timestamp;
                    // Note: This is simplified - in a real implementation,
                    // we'd need a way to update individual member records
                    break;
                }
            }
        }

        Ok(())
    }

    fn get_contextual_reputation(&self, did: &Did, federation_id: &str) -> Option<u64> {
        let members = self.registry.get_members(federation_id);
        members
            .iter()
            .find(|m| &m.did == did)
            .map(|m| m.reputation_score)
    }
}

/// Complete federation management system
pub struct FederationManager {
    registry: Arc<dyn FederationRegistry>,
    membership_service: Arc<dyn FederationMembershipService>,
    did_resolver: Arc<dyn DidResolver>,
    time_provider: Arc<dyn TimeProvider>,
}

impl FederationManager {
    pub fn new(
        registry: Arc<dyn FederationRegistry>,
        did_resolver: Arc<dyn DidResolver>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let membership_service = Arc::new(DefaultFederationMembershipService::new(
            registry.clone(),
            time_provider.clone(),
        ));

        Self {
            registry,
            membership_service,
            did_resolver,
            time_provider,
        }
    }

    /// Create a new federation
    pub fn create_federation(&self, info: FederationInfo) -> Result<(), CommonError> {
        // Validate admin DID
        self.did_resolver.resolve(&info.admin_did)?;

        self.registry.create_federation(info)
    }

    /// Join a federation
    pub fn join_federation(
        &self,
        federation_id: &str,
        did: &Did,
        capabilities: FederationCapabilities,
    ) -> Result<(), CommonError> {
        // Validate DID
        self.did_resolver.resolve(did)?;

        // Check if federation exists
        let federation = self.registry.get_federation(federation_id).ok_or_else(|| {
            CommonError::InvalidInputError(format!("Federation {} not found", federation_id))
        })?;

        // Check membership policy
        match federation.membership_policy {
            MembershipPolicy::Open => {
                // Allow immediate joining
            }
            MembershipPolicy::InviteOnly
            | MembershipPolicy::AdminApproval
            | MembershipPolicy::Consensus => {
                // These would require additional approval flows
                return Err(CommonError::InvalidInputError(
                    "Federation requires approval for membership".to_string(),
                ));
            }
            MembershipPolicy::Custom { .. } => {
                // Custom policy validation would go here
                return Err(CommonError::InvalidInputError(
                    "Custom membership policy not implemented".to_string(),
                ));
            }
        }

        let member = FederationMember {
            did: did.clone(),
            joined_at: self.time_provider.unix_seconds(),
            status: MembershipStatus::Active,
            capabilities,
            reputation_score: 100, // Default reputation
            last_seen: self.time_provider.unix_seconds(),
            roles: vec!["member".to_string()],
            metadata: HashMap::new(),
        };

        self.registry.add_member(federation_id, member)
    }

    /// Leave a federation
    pub fn leave_federation(&self, federation_id: &str, did: &Did) -> Result<(), CommonError> {
        self.registry.remove_member(federation_id, did)
    }

    /// Get federation information
    pub fn get_federation(&self, federation_id: &str) -> Option<FederationInfo> {
        self.registry.get_federation(federation_id)
    }

    /// List federations
    pub fn list_federations(&self) -> Vec<FederationInfo> {
        self.registry.list_federations()
    }

    /// Search federations
    pub fn search_federations(&self, query: &FederationSearchQuery) -> Vec<FederationInfo> {
        self.registry.search_federations(query)
    }

    /// Get membership service
    pub fn membership_service(&self) -> &Arc<dyn FederationMembershipService> {
        &self.membership_service
    }

    /// Get registry
    pub fn registry(&self) -> &Arc<dyn FederationRegistry> {
        &self.registry
    }
}

/// Default capabilities for testing
impl Default for FederationCapabilities {
    fn default() -> Self {
        Self {
            compute: ComputeCapabilities {
                cpu_cores: 4,
                memory_gb: 8,
                gpu_units: 0,
                specialized_hardware: Vec::new(),
            },
            storage: StorageCapabilities {
                total_capacity_gb: 100,
                available_capacity_gb: 80,
                storage_types: vec!["SSD".to_string()],
                replication_factor: 1,
            },
            network: NetworkCapabilities {
                bandwidth_mbps: 100,
                latency_ms: 50,
                protocols: vec!["TCP".to_string(), "UDP".to_string()],
                regions: vec!["local".to_string()],
            },
            services: vec!["compute".to_string()],
            geographic_presence: vec![GeographicScope::Local],
        }
    }
}

impl Default for ComputeCapabilities {
    fn default() -> Self {
        Self {
            cpu_cores: 4,
            memory_gb: 8,
            gpu_units: 0,
            specialized_hardware: Vec::new(),
        }
    }
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            total_capacity_gb: 100,
            available_capacity_gb: 80,
            storage_types: vec!["SSD".to_string()],
            replication_factor: 1,
        }
    }
}

impl Default for NetworkCapabilities {
    fn default() -> Self {
        Self {
            bandwidth_mbps: 100,
            latency_ms: 50,
            protocols: vec!["TCP".to_string(), "UDP".to_string()],
            regions: vec!["local".to_string()],
        }
    }
}
