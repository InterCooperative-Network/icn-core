//! Social Contract Registry
//!
//! This module provides a canonical, append-only, public log of all social contracts,
//! amendments, ratifications, and forks from local to global scope.

use crate::social_contract::{
    ContractAmendment, ContractVersion, GovernanceScope, MemberConsent, SocialContract,
    SocialContractId, SocialContractStatus,
};
use icn_common::{Cid, DagBlock, DagLink, CommonError, Did, NodeScope};
use icn_dag::StorageService;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Canonical registry for all social contracts in the network
pub struct SocialContractRegistry<S>
where
    S: StorageService<DagBlock>,
{
    /// DAG storage for permanent contract storage
    storage: S,
    /// In-memory index of contracts by ID
    contract_index: HashMap<SocialContractId, ContractRegistryEntry>,
    /// Index by scope
    scope_index: HashMap<GovernanceScope, HashSet<SocialContractId>>,
    /// Index by status
    status_index: HashMap<SocialContractStatus, HashSet<SocialContractId>>,
    /// Amendment history
    amendment_history: HashMap<SocialContractId, Vec<AmendmentRecord>>,
    /// Member consent records
    consent_records: HashMap<(Did, SocialContractId), MemberConsent>,
    /// Registry metadata
    metadata: RegistryMetadata,
}

/// Registry entry for a social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRegistryEntry {
    /// Contract identifier
    pub contract_id: SocialContractId,
    /// Current version
    pub current_version: ContractVersion,
    /// CID of the contract in DAG storage
    pub contract_cid: Cid,
    /// Registration timestamp
    pub registered_at: SystemTime,
    /// Last update timestamp
    pub updated_at: SystemTime,
    /// Current status
    pub status: SocialContractStatus,
    /// Scope
    pub scope: GovernanceScope,
    /// Creator
    pub creator: Did,
    /// Version history CIDs
    pub version_history: Vec<VersionRecord>,
    /// Parent contract (if derived)
    pub parent_contract: Option<SocialContractId>,
    /// Child contracts (if forked)
    pub child_contracts: HashSet<SocialContractId>,
    /// Ratification status
    pub ratification: RatificationStatus,
}

/// Record of a contract version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    /// Version number
    pub version: ContractVersion,
    /// CID of this version in DAG
    pub cid: Cid,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Amendment that created this version (if any)
    pub amendment: Option<AmendmentRecord>,
}

/// Record of an amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentRecord {
    /// Amendment identifier
    pub id: String,
    /// Amendment details CID
    pub amendment_cid: Cid,
    /// Proposer
    pub proposer: Did,
    /// Proposal timestamp
    pub proposed_at: SystemTime,
    /// Ratification status
    pub ratification_status: AmendmentRatificationStatus,
    /// Ratification votes
    pub votes: HashMap<Did, AmendmentVote>,
    /// Ratification timestamp (if ratified)
    pub ratified_at: Option<SystemTime>,
}

/// Ratification status of the contract
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RatificationStatus {
    /// Draft - not yet proposed for ratification
    Draft,
    /// Under deliberation
    Deliberation,
    /// Active voting
    Voting,
    /// Ratified and active
    Ratified,
    /// Failed ratification
    Rejected,
    /// Withdrawn by creator
    Withdrawn,
}

/// Ratification status of an amendment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendmentRatificationStatus {
    /// Proposed
    Proposed,
    /// Under review
    Review,
    /// Active voting
    Voting,
    /// Ratified
    Ratified,
    /// Rejected
    Rejected,
    /// Withdrawn
    Withdrawn,
}

/// Vote on an amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentVote {
    /// Voter
    pub voter: Did,
    /// Vote (true = support, false = oppose)
    pub vote: bool,
    /// Timestamp
    pub voted_at: SystemTime,
    /// Optional reasoning
    pub reasoning: Option<String>,
}

/// Registry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    /// Registry creation timestamp
    pub created_at: SystemTime,
    /// Total contracts registered
    pub total_contracts: usize,
    /// Active contracts
    pub active_contracts: usize,
    /// Registry version
    pub version: String,
    /// Last updated timestamp
    pub updated_at: SystemTime,
}

/// Events emitted by the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryEvent {
    /// Contract registered
    ContractRegistered {
        contract_id: SocialContractId,
        creator: Did,
        scope: GovernanceScope,
        timestamp: SystemTime,
    },
    /// Contract updated
    ContractUpdated {
        contract_id: SocialContractId,
        old_version: ContractVersion,
        new_version: ContractVersion,
        timestamp: SystemTime,
    },
    /// Amendment proposed
    AmendmentProposed {
        contract_id: SocialContractId,
        amendment_id: String,
        proposer: Did,
        timestamp: SystemTime,
    },
    /// Amendment ratified
    AmendmentRatified {
        contract_id: SocialContractId,
        amendment_id: String,
        new_version: ContractVersion,
        timestamp: SystemTime,
    },
    /// Contract forked
    ContractForked {
        parent_contract: SocialContractId,
        child_contract: SocialContractId,
        creator: Did,
        timestamp: SystemTime,
    },
    /// Consent given/withdrawn
    ConsentChanged {
        member: Did,
        contract_id: SocialContractId,
        consent_given: bool,
        timestamp: SystemTime,
    },
}

/// Error types for registry operations
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Contract not found: {0}")]
    ContractNotFound(SocialContractId),

    #[error("Contract already exists: {0}")]
    ContractAlreadyExists(SocialContractId),

    #[error("Invalid contract version: {0}")]
    InvalidVersion(String),

    #[error("Amendment not found: {0}")]
    AmendmentNotFound(String),

    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),

    #[error("Storage error: {0}")]
    Storage(#[from] CommonError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl<S> SocialContractRegistry<S>
where
    S: StorageService<DagBlock>,
{
    /// Create a new social contract registry
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            contract_index: HashMap::new(),
            scope_index: HashMap::new(),
            status_index: HashMap::new(),
            amendment_history: HashMap::new(),
            consent_records: HashMap::new(),
            metadata: RegistryMetadata {
                created_at: SystemTime::now(),
                total_contracts: 0,
                active_contracts: 0,
                version: "1.0.0".to_string(),
                updated_at: SystemTime::now(),
            },
        }
    }

    /// Register a new social contract
    pub fn register_contract(
        &mut self,
        contract: SocialContract,
    ) -> Result<Cid, RegistryError> {
        // Check if contract already exists
        if self.contract_index.contains_key(&contract.id) {
            return Err(RegistryError::ContractAlreadyExists(contract.id.clone()));
        }

        // Store contract in DAG
        let contract_cid = self.store_contract_in_dag(&contract)?;

        // Create registry entry
        let entry = ContractRegistryEntry {
            contract_id: contract.id.clone(),
            current_version: contract.version.clone(),
            contract_cid: contract_cid.clone(),
            registered_at: contract.created_at,
            updated_at: contract.modified_at,
            status: contract.status.clone(),
            scope: contract.scope.clone(),
            creator: contract.creator.clone(),
            version_history: vec![VersionRecord {
                version: contract.version.clone(),
                cid: contract_cid.clone(),
                created_at: contract.created_at,
                amendment: None,
            }],
            parent_contract: contract.parent_contract.clone(),
            child_contracts: HashSet::new(),
            ratification: RatificationStatus::Draft,
        };

        // Update indices
        self.contract_index.insert(contract.id.clone(), entry);
        self.scope_index
            .entry(contract.scope.clone())
            .or_default()
            .insert(contract.id.clone());
        self.status_index
            .entry(contract.status.clone())
            .or_default()
            .insert(contract.id.clone());

        // If this is a child contract, update parent
        if let Some(parent_id) = &contract.parent_contract {
            if let Some(parent_entry) = self.contract_index.get_mut(parent_id) {
                parent_entry.child_contracts.insert(contract.id.clone());
            }
        }

        // Update metadata
        self.metadata.total_contracts += 1;
        if matches!(contract.status, SocialContractStatus::Active) {
            self.metadata.active_contracts += 1;
        }
        self.metadata.updated_at = SystemTime::now();

        Ok(contract_cid)
    }

    /// Get a contract by ID
    pub fn get_contract(
        &self,
        contract_id: &SocialContractId,
    ) -> Result<Option<SocialContract>, RegistryError> {
        if let Some(entry) = self.contract_index.get(contract_id) {
            self.load_contract_from_dag(&entry.contract_cid)
        } else {
            Ok(None)
        }
    }

    /// Get contract registry entry
    pub fn get_contract_entry(
        &self,
        contract_id: &SocialContractId,
    ) -> Option<&ContractRegistryEntry> {
        self.contract_index.get(contract_id)
    }

    /// Get specific version of a contract
    pub fn get_contract_version(
        &self,
        contract_id: &SocialContractId,
        version: &ContractVersion,
    ) -> Result<Option<SocialContract>, RegistryError> {
        if let Some(entry) = self.contract_index.get(contract_id) {
            // Find the version record
            if let Some(version_record) = entry
                .version_history
                .iter()
                .find(|v| v.version == *version)
            {
                self.load_contract_from_dag(&version_record.cid)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// List contracts by scope
    pub fn list_contracts_by_scope(
        &self,
        scope: &GovernanceScope,
    ) -> Vec<&ContractRegistryEntry> {
        if let Some(contract_ids) = self.scope_index.get(scope) {
            contract_ids
                .iter()
                .filter_map(|id| self.contract_index.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// List contracts by status
    pub fn list_contracts_by_status(
        &self,
        status: &SocialContractStatus,
    ) -> Vec<&ContractRegistryEntry> {
        if let Some(contract_ids) = self.status_index.get(status) {
            contract_ids
                .iter()
                .filter_map(|id| self.contract_index.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Propose an amendment to a contract
    pub fn propose_amendment(
        &mut self,
        contract_id: &SocialContractId,
        amendment: ContractAmendment,
        proposer: Did,
    ) -> Result<String, RegistryError> {
        // Verify contract exists
        if !self.contract_index.contains_key(contract_id) {
            return Err(RegistryError::ContractNotFound(contract_id.clone()));
        }

        // Store amendment in DAG
        let amendment_cid = self.store_amendment_in_dag(&amendment)?;

        // Create amendment record
        let amendment_record = AmendmentRecord {
            id: amendment.id.clone(),
            amendment_cid,
            proposer,
            proposed_at: amendment.proposed_at,
            ratification_status: AmendmentRatificationStatus::Proposed,
            votes: HashMap::new(),
            ratified_at: None,
        };

        // Add to amendment history
        self.amendment_history
            .entry(contract_id.clone())
            .or_default()
            .push(amendment_record);

        Ok(amendment.id)
    }

    /// Vote on an amendment
    pub fn vote_on_amendment(
        &mut self,
        contract_id: &SocialContractId,
        amendment_id: &str,
        voter: Did,
        vote: bool,
        reasoning: Option<String>,
    ) -> Result<(), RegistryError> {
        // Find the amendment
        if let Some(amendments) = self.amendment_history.get_mut(contract_id) {
            if let Some(amendment) = amendments.iter_mut().find(|a| a.id == amendment_id) {
                // Check if amendment is in voting status
                if amendment.ratification_status != AmendmentRatificationStatus::Voting {
                    return Err(RegistryError::InvalidOperation(
                        "Amendment not open for voting".to_string(),
                    ));
                }

                // Record vote
                let amendment_vote = AmendmentVote {
                    voter: voter.clone(),
                    vote,
                    voted_at: SystemTime::now(),
                    reasoning,
                };

                amendment.votes.insert(voter, amendment_vote);
                Ok(())
            } else {
                Err(RegistryError::AmendmentNotFound(amendment_id.to_string()))
            }
        } else {
            Err(RegistryError::ContractNotFound(contract_id.clone()))
        }
    }

    /// Ratify an amendment after successful vote
    pub fn ratify_amendment(
        &mut self,
        contract_id: &SocialContractId,
        amendment_id: &str,
        updated_contract: SocialContract,
    ) -> Result<ContractVersion, RegistryError> {
        // Update amendment status
        if let Some(amendments) = self.amendment_history.get_mut(contract_id) {
            if let Some(amendment) = amendments.iter_mut().find(|a| a.id == amendment_id) {
                amendment.ratification_status = AmendmentRatificationStatus::Ratified;
                amendment.ratified_at = Some(SystemTime::now());
            }
        }

        // Store new version and update contract entry
        self.update_contract_version(contract_id, updated_contract, Some(amendment_id.to_string()))
    }

    /// Update contract to new version
    fn update_contract_version(
        &mut self,
        contract_id: &SocialContractId,
        updated_contract: SocialContract,
        amendment_id: Option<String>,
    ) -> Result<ContractVersion, RegistryError> {
        // Store new version in DAG
        let new_cid = self.store_contract_in_dag(&updated_contract)?;

        // Update registry entry
        if let Some(entry) = self.contract_index.get_mut(contract_id) {
            let old_version = entry.current_version.clone();
            entry.current_version = updated_contract.version.clone();
            entry.contract_cid = new_cid.clone();
            entry.updated_at = updated_contract.modified_at;

            // Add to version history
            let version_record = VersionRecord {
                version: updated_contract.version.clone(),
                cid: new_cid,
                created_at: updated_contract.modified_at,
                amendment: amendment_id.and_then(|id| {
                    self.amendment_history
                        .get(contract_id)
                        .and_then(|amendments| amendments.iter().find(|a| a.id == id).cloned())
                }),
            };
            entry.version_history.push(version_record);

            // Update status indices if status changed
            if entry.status != updated_contract.status {
                // Remove from old status index
                if let Some(old_status_set) = self.status_index.get_mut(&entry.status) {
                    old_status_set.remove(contract_id);
                }

                // Add to new status index
                self.status_index
                    .entry(updated_contract.status.clone())
                    .or_default()
                    .insert(contract_id.clone());

                entry.status = updated_contract.status.clone();
            }

            Ok(updated_contract.version)
        } else {
            Err(RegistryError::ContractNotFound(contract_id.clone()))
        }
    }

    /// Fork a contract to create a new derived contract
    pub fn fork_contract(
        &mut self,
        parent_id: &SocialContractId,
        child_contract: SocialContract,
    ) -> Result<Cid, RegistryError> {
        // Verify parent exists
        if !self.contract_index.contains_key(parent_id) {
            return Err(RegistryError::ContractNotFound(parent_id.clone()));
        }

        // Register child contract
        let child_cid = self.register_contract(child_contract)?;

        Ok(child_cid)
    }

    /// Record member consent to a contract
    pub fn record_consent(
        &mut self,
        member: Did,
        contract_id: SocialContractId,
        consent: MemberConsent,
    ) -> Result<(), RegistryError> {
        // Verify contract exists
        if !self.contract_index.contains_key(&contract_id) {
            return Err(RegistryError::ContractNotFound(contract_id.clone()));
        }

        self.consent_records
            .insert((member, contract_id), consent);
        Ok(())
    }

    /// Get member consent for a contract
    pub fn get_member_consent(
        &self,
        member: &Did,
        contract_id: &SocialContractId,
    ) -> Option<&MemberConsent> {
        self.consent_records.get(&(member.clone(), contract_id.clone()))
    }

    /// Get all members who have consented to a contract
    pub fn get_consenting_members(
        &self,
        contract_id: &SocialContractId,
    ) -> Vec<&MemberConsent> {
        self.consent_records
            .iter()
            .filter(|((_, cid), consent)| cid == contract_id && consent.is_valid())
            .map(|(_, consent)| consent)
            .collect()
    }

    /// Get registry metadata
    pub fn get_metadata(&self) -> &RegistryMetadata {
        &self.metadata
    }

    /// Store contract in DAG
    fn store_contract_in_dag(&mut self, contract: &SocialContract) -> Result<Cid, RegistryError> {
        // Serialize contract
        let contract_data = serde_json::to_vec(contract)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;

        // Create DAG block
        let block = DagBlock {
            cid: Cid::new_v1_sha256(0x71, &contract_data),
            data: contract_data,
            links: vec![], // Contracts can link to parent/predecessor later
            timestamp: contract
                .created_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            author_did: contract.creator.clone(),
            signature: None,
            scope: Some(NodeScope(contract.scope.as_str().to_string())),
        };

        // Store in DAG
        self.storage.put(&block)?;
        Ok(block.cid)
    }

    /// Load contract from DAG
    fn load_contract_from_dag(&self, cid: &Cid) -> Result<Option<SocialContract>, RegistryError> {
        match self.storage.get(cid)? {
            Some(block) => {
                let contract: SocialContract = serde_json::from_slice(&block.data)
                    .map_err(|e| RegistryError::Serialization(e.to_string()))?;
                Ok(Some(contract))
            }
            None => Ok(None),
        }
    }

    /// Store amendment in DAG
    fn store_amendment_in_dag(
        &mut self,
        amendment: &ContractAmendment,
    ) -> Result<Cid, RegistryError> {
        // Serialize amendment
        let amendment_data = serde_json::to_vec(amendment)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;

        // Create DAG block
        let block = DagBlock {
            cid: Cid::new_v1_sha256(0x71, &amendment_data),
            data: amendment_data,
            links: vec![], // Could link to target contract
            timestamp: amendment
                .proposed_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            author_did: amendment.proposer.clone(),
            signature: None,
            scope: None,
        };

        // Store in DAG
        self.storage.put(&block)?;
        Ok(block.cid)
    }

    /// Create a new contract ID
    pub fn generate_contract_id(title: &str, creator: &Did) -> SocialContractId {
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = format!(
            "contract:{}:{}:{}",
            title.to_lowercase().replace(' ', "-"),
            creator.to_string().chars().take(8).collect::<String>(),
            timestamp
        );

        SocialContractId(id)
    }

    /// Create a new amendment ID
    pub fn generate_amendment_id(
        contract_id: &SocialContractId,
        proposer: &Did,
    ) -> String {
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!(
            "amendment:{}:{}:{}",
            contract_id.0,
            proposer.to_string().chars().take(8).collect::<String>(),
            timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social_contract::{ConsentStatus, GovernanceScope};
    use icn_dag::InMemoryDagStore;

    fn create_test_contract() -> SocialContract {
        let id = SocialContractId("test-contract".to_string());
        let creator = Did::new("test", "creator");
        let cid = Cid::new_v1_sha256(0x55, b"test-contract-ccl");

        SocialContract::new(
            id,
            "Test Contract".to_string(),
            "A test social contract".to_string(),
            GovernanceScope::Local,
            cid,
            creator,
        )
    }

    #[test]
    fn test_registry_creation() {
        let storage = InMemoryDagStore::new();
        let registry = SocialContractRegistry::new(storage);

        assert_eq!(registry.metadata.total_contracts, 0);
        assert_eq!(registry.metadata.active_contracts, 0);
    }

    #[test]
    fn test_contract_registration() {
        let storage = InMemoryDagStore::new();
        let mut registry = SocialContractRegistry::new(storage);

        let contract = create_test_contract();
        let contract_id = contract.id.clone();

        let result = registry.register_contract(contract);
        assert!(result.is_ok());

        // Verify contract is in registry
        let entry = registry.get_contract_entry(&contract_id);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.contract_id, contract_id);
        assert_eq!(entry.version_history.len(), 1);
    }

    #[test]
    fn test_contract_retrieval() {
        let storage = InMemoryDagStore::new();
        let mut registry = SocialContractRegistry::new(storage);

        let contract = create_test_contract();
        let contract_id = contract.id.clone();

        registry.register_contract(contract.clone()).unwrap();

        let retrieved = registry.get_contract(&contract_id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, contract.id);
        assert_eq!(retrieved.title, contract.title);
    }

    #[test]
    fn test_consent_recording() {
        let storage = InMemoryDagStore::new();
        let mut registry = SocialContractRegistry::new(storage);

        let contract = create_test_contract();
        let contract_id = contract.id.clone();
        registry.register_contract(contract).unwrap();

        let member = Did::new("test", "member");
        let consent = MemberConsent::new(
            member.clone(),
            contract_id.clone(),
            ContractVersion::initial(),
            ConsentStatus::Consented,
            "en".to_string(),
        );

        let result = registry.record_consent(member.clone(), contract_id.clone(), consent);
        assert!(result.is_ok());

        let retrieved_consent = registry.get_member_consent(&member, &contract_id);
        assert!(retrieved_consent.is_some());
        assert!(retrieved_consent.unwrap().is_valid());
    }

    #[test]
    fn test_id_generation() {
        let creator = Did::new("test", "creator");

        let contract_id = SocialContractRegistry::<InMemoryDagStore>::generate_contract_id(
            "Test Contract",
            &creator,
        );

        assert!(contract_id.0.starts_with("contract:test-contract"));

        let amendment_id = SocialContractRegistry::<InMemoryDagStore>::generate_amendment_id(
            &contract_id,
            &creator,
        );

        assert!(amendment_id.starts_with("amendment:"));
    }
}