//! CRDT-backed proposal state for conflict-free distributed governance coordination.
//!
//! This module provides proposal state management using CRDTs to ensure
//! conflict-free replication across multiple nodes. Proposals are managed using
//! a combination of LWW-Registers for proposal metadata and CRDT Maps for votes
//! to enable concurrent proposal management and voting without conflicts.

use icn_common::{CommonError, Did, TimeProvider};
use icn_crdt::{CRDTMap, CRDTValue, LWWRegister, NodeId, CRDT};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

/// CRDT-backed proposal state manager that enables conflict-free distributed governance.
///
/// Uses a CRDT Map where each proposal ID maps to a ProposalCRDT containing
/// proposal metadata and vote state. This allows multiple nodes to concurrently
/// create proposals and cast votes without conflicts.
pub struct CRDTProposalState {
    /// Node identifier for this proposal state instance.
    node_id: NodeId,
    /// CRDT Map storing proposal_id -> ProposalCRDT mappings.
    proposal_map: Arc<RwLock<CRDTMap<String, ProposalCRDT>>>,
    /// Configuration for proposal state management.
    config: CRDTProposalStateConfig,
    /// Time provider for deterministic timestamps.
    time_provider: Arc<dyn TimeProvider>,
}

/// Individual proposal state using CRDTs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalCRDT {
    /// Proposal metadata using LWW-Register for conflict-free updates.
    pub metadata: LWWRegister<ProposalMetadata>,
    /// Vote mappings using CRDT Map (DID -> Vote).
    pub votes: CRDTMap<String, LWWRegister<Vote>>,
}

/// Proposal metadata that can be updated conflict-free.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposalMetadata {
    /// Unique proposal identifier.
    pub proposal_id: String,
    /// Title of the proposal.
    pub title: String,
    /// Description of the proposal.
    pub description: String,
    /// DID of the proposal creator.
    pub proposer: Did,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Voting deadline (Unix timestamp).
    pub voting_deadline: u64,
    /// Required quorum for the proposal.
    pub required_quorum: u64,
    /// Required approval percentage (0-100).
    pub required_approval: u64,
    /// Timestamp when the proposal was created.
    pub created_at: u64,
    /// Timestamp when the proposal was last updated.
    pub updated_at: u64,
}

impl CRDTValue for ProposalMetadata {}

/// Vote on a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vote {
    /// The vote decision.
    pub decision: VoteDecision,
    /// DID of the voter.
    pub voter: Did,
    /// Timestamp when the vote was cast.
    pub timestamp: u64,
    /// Optional comment with the vote.
    pub comment: Option<String>,
}

impl CRDTValue for Vote {}

/// Vote decision options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteDecision {
    /// Vote in favor of the proposal.
    Approve,
    /// Vote against the proposal.
    Reject,
    /// Abstain from voting.
    Abstain,
}

impl CRDTValue for VoteDecision {}

/// Status of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is open for voting.
    Open,
    /// Proposal has been approved.
    Approved,
    /// Proposal has been rejected.
    Rejected,
    /// Proposal has expired without reaching quorum.
    Expired,
    /// Proposal has been withdrawn by the proposer.
    Withdrawn,
}

impl CRDTValue for ProposalStatus {}

/// Configuration for CRDT proposal state management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTProposalStateConfig {
    /// Node identifier for this state manager instance.
    pub node_id: String,
    /// Default voting deadline duration in seconds.
    pub default_voting_duration: u64,
    /// Default required quorum.
    pub default_quorum: u64,
    /// Default required approval percentage.
    pub default_approval: u64,
    /// Whether to enable automatic proposal expiration.
    pub auto_expire_proposals: bool,
    /// Maximum number of active proposals per proposer.
    pub max_proposals_per_proposer: usize,
}

impl Default for CRDTProposalStateConfig {
    fn default() -> Self {
        Self {
            node_id: "default_governance_node".to_string(),
            default_voting_duration: 7 * 24 * 60 * 60, // 7 days
            default_quorum: 10,
            default_approval: 50, // 50%
            auto_expire_proposals: true,
            max_proposals_per_proposer: 5,
        }
    }
}

/// Statistics about the CRDT proposal state manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRDTProposalStateStats {
    /// Total number of proposals.
    pub total_proposals: u64,
    /// Number of open proposals.
    pub open_proposals: u64,
    /// Number of approved proposals.
    pub approved_proposals: u64,
    /// Number of rejected proposals.
    pub rejected_proposals: u64,
    /// Number of expired proposals.
    pub expired_proposals: u64,
    /// Total number of votes cast.
    pub total_votes: u64,
    /// Average votes per proposal.
    pub average_votes_per_proposal: u64,
    /// Node ID of this proposal state manager instance.
    pub node_id: NodeId,
}

/// Information about a specific proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalInfo {
    /// Proposal metadata.
    pub metadata: ProposalMetadata,
    /// Current vote tally.
    pub vote_tally: VoteTally,
    /// Whether the proposal has reached quorum.
    pub has_quorum: bool,
    /// Whether the proposal is approved (if voting has ended).
    pub is_approved: Option<bool>,
}

/// Vote tally for a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteTally {
    /// Number of approve votes.
    pub approve: u64,
    /// Number of reject votes.
    pub reject: u64,
    /// Number of abstain votes.
    pub abstain: u64,
    /// Total number of votes.
    pub total: u64,
    /// Approval percentage.
    pub approval_percentage: u64,
}

impl ProposalCRDT {
    /// Create a new proposal CRDT with the given metadata.
    pub fn new(metadata: ProposalMetadata, node_id: NodeId) -> Self {
        let metadata_register = LWWRegister::with_initial_value(
            format!("proposal_metadata_{}", metadata.proposal_id),
            metadata,
            node_id.clone(),
        );

        let votes_map = CRDTMap::new(format!(
            "proposal_votes_{}",
            metadata_register.get().unwrap().proposal_id
        ));

        Self {
            metadata: metadata_register,
            votes: votes_map,
        }
    }

    /// Get the current proposal metadata.
    pub fn get_metadata(&self) -> &ProposalMetadata {
        self.metadata
            .get()
            .expect("Proposal metadata should always be present")
    }

    /// Update the proposal metadata.
    pub fn update_metadata(
        &mut self,
        new_metadata: ProposalMetadata,
        node_id: NodeId,
    ) -> Result<(), CommonError> {
        self.metadata
            .write(new_metadata, node_id)
            .map_err(|e| CommonError::CRDTError(format!("Failed to update metadata: {}", e)))
    }

    /// Add or update a vote for this proposal.
    pub fn cast_vote(&mut self, vote: Vote, node_id: NodeId) -> Result<(), CommonError> {
        let vote_id = format!("vote_{}", vote.voter.to_string());
        let vote_register =
            LWWRegister::with_initial_value(vote_id.clone(), vote.clone(), node_id.clone());

        self.votes
            .put(vote.voter.to_string(), vote_register, node_id)
            .map_err(|e| CommonError::CRDTError(format!("Failed to cast vote: {}", e)))?;

        Ok(())
    }

    /// Get all votes for this proposal.
    pub fn get_votes(&self) -> HashMap<Did, Vote> {
        let mut votes = HashMap::new();

        for voter_did_str in self.votes.keys() {
            if let Some(vote_register) = self.votes.get(&voter_did_str) {
                if let Ok(voter_did) = Did::from_str(&voter_did_str) {
                    if let Some(vote) = vote_register.get() {
                        votes.insert(voter_did, vote.clone());
                    }
                }
            }
        }

        votes
    }

    /// Calculate the current vote tally.
    pub fn calculate_tally(&self) -> VoteTally {
        let votes = self.get_votes();
        let mut approve = 0u64;
        let mut reject = 0u64;
        let mut abstain = 0u64;

        for vote in votes.values() {
            match vote.decision {
                VoteDecision::Approve => approve += 1,
                VoteDecision::Reject => reject += 1,
                VoteDecision::Abstain => abstain += 1,
            }
        }

        let total = approve + reject + abstain;
        let approval_percentage = if total > 0 {
            (approve * 100) / total
        } else {
            0
        };

        VoteTally {
            approve,
            reject,
            abstain,
            total,
            approval_percentage,
        }
    }

    /// Check if the proposal has reached quorum.
    pub fn has_quorum(&self, required_quorum: u64) -> bool {
        self.calculate_tally().total >= required_quorum
    }

    /// Check if the proposal is approved based on current votes.
    pub fn is_approved(&self, required_approval: u64) -> bool {
        let tally = self.calculate_tally();
        tally.approval_percentage >= required_approval
    }
}

impl CRDT for ProposalCRDT {
    type Operation = (); // We don't use operations for this composite CRDT

    fn merge(&mut self, other: &Self) {
        self.metadata.merge(&other.metadata);
        self.votes.merge(&other.votes);
    }

    fn apply_operation(&mut self, _op: Self::Operation) -> Result<(), icn_crdt::CRDTError> {
        // Not used for this composite CRDT
        Ok(())
    }

    fn value(&self) -> serde_json::Value {
        serde_json::json!({
            "metadata": self.metadata.value(),
            "votes": self.votes.value(),
            "tally": self.calculate_tally()
        })
    }

    fn crdt_id(&self) -> String {
        format!("proposal_{}", self.get_metadata().proposal_id)
    }
}

impl CRDTProposalState {
    /// Create a new CRDT proposal state manager with the given configuration.
    pub fn new(config: CRDTProposalStateConfig, time_provider: Arc<dyn TimeProvider>) -> Self {
        let node_id = NodeId::new(config.node_id.clone());
        let proposal_map = CRDTMap::new("governance_proposals".to_string());

        Self {
            node_id,
            proposal_map: Arc::new(RwLock::new(proposal_map)),
            config,
            time_provider,
        }
    }

    /// Create a new CRDT proposal state manager with a specific node ID.
    pub fn with_node_id(node_id: String, time_provider: Arc<dyn TimeProvider>) -> Self {
        Self::new(
            CRDTProposalStateConfig {
                node_id,
                ..Default::default()
            },
            time_provider,
        )
    }

    /// Get the node ID for this proposal state manager instance.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Merge state from another CRDT proposal state manager.
    ///
    /// This enables synchronization between distributed proposal state manager instances.
    pub fn merge(&self, other: &Self) -> Result<(), CommonError> {
        let mut our_map = self
            .proposal_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        let other_map = other
            .proposal_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        our_map.merge(&*other_map);

        debug!("Merged CRDT proposal state from node {}", other.node_id);
        Ok(())
    }

    /// Create a new proposal.
    #[allow(clippy::too_many_arguments)]
    pub fn create_proposal(
        &self,
        proposal_id: String,
        title: String,
        description: String,
        proposer: Did,
        custom_deadline: Option<u64>,
        custom_quorum: Option<u64>,
        custom_approval: Option<u64>,
    ) -> Result<(), CommonError> {
        debug!("Creating proposal: {}", proposal_id);

        if self.proposal_exists(&proposal_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Proposal {} already exists",
                proposal_id
            )));
        }

        // Check proposer limits
        if self.config.max_proposals_per_proposer > 0 {
            let proposer_count = self.count_active_proposals_by_proposer(&proposer);
            if proposer_count >= self.config.max_proposals_per_proposer {
                return Err(CommonError::InvalidInputError(format!(
                    "Proposer {} has reached maximum active proposals limit ({})",
                    proposer, self.config.max_proposals_per_proposer
                )));
            }
        }

        let current_time = self.time_provider.unix_seconds();

        let voting_deadline =
            custom_deadline.unwrap_or(current_time + self.config.default_voting_duration);
        let required_quorum = custom_quorum.unwrap_or(self.config.default_quorum);
        let required_approval = custom_approval.unwrap_or(self.config.default_approval);

        let metadata = ProposalMetadata {
            proposal_id: proposal_id.clone(),
            title,
            description,
            proposer,
            status: ProposalStatus::Open,
            voting_deadline,
            required_quorum,
            required_approval,
            created_at: current_time,
            updated_at: current_time,
        };

        let proposal_crdt = ProposalCRDT::new(metadata, self.node_id.clone());

        let mut map = self
            .proposal_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        map.put(proposal_id.clone(), proposal_crdt, self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to create proposal: {}", e)))?;

        debug!("Successfully created proposal: {}", proposal_id);
        Ok(())
    }

    /// Cast a vote on a proposal.
    pub fn cast_vote(
        &self,
        proposal_id: &str,
        voter: Did,
        decision: VoteDecision,
        comment: Option<String>,
    ) -> Result<(), CommonError> {
        debug!(
            "Casting vote for proposal {} by {}: {:?}",
            proposal_id, voter, decision
        );

        if !self.proposal_exists(proposal_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Proposal {} does not exist",
                proposal_id
            )));
        }

        let current_time = self.time_provider.unix_seconds();

        // Check if proposal is still open for voting
        let proposal_info = self.get_proposal_info(proposal_id)?;
        match proposal_info.metadata.status {
            ProposalStatus::Open => {
                if current_time > proposal_info.metadata.voting_deadline {
                    return Err(CommonError::InvalidInputError(format!(
                        "Proposal {} voting deadline has passed",
                        proposal_id
                    )));
                }
            }
            _ => {
                return Err(CommonError::InvalidInputError(format!(
                    "Proposal {} is not open for voting (status: {:?})",
                    proposal_id, proposal_info.metadata.status
                )));
            }
        }

        let vote = Vote {
            decision,
            voter: voter.clone(),
            timestamp: current_time,
            comment,
        };

        // Get and update the proposal CRDT
        let mut proposal_crdt = self.get_proposal_crdt(proposal_id)?;
        proposal_crdt.cast_vote(vote, self.node_id.clone())?;

        // Update the proposal in the map
        self.update_proposal_crdt(proposal_id, proposal_crdt)?;

        debug!(
            "Successfully cast vote for proposal {} by {}",
            proposal_id, voter
        );
        Ok(())
    }

    /// Update proposal status.
    pub fn update_proposal_status(
        &self,
        proposal_id: &str,
        new_status: ProposalStatus,
    ) -> Result<(), CommonError> {
        debug!(
            "Updating proposal {} status to {:?}",
            proposal_id, new_status
        );

        if !self.proposal_exists(proposal_id) {
            return Err(CommonError::InvalidInputError(format!(
                "Proposal {} does not exist",
                proposal_id
            )));
        }

        let mut proposal_crdt = self.get_proposal_crdt(proposal_id)?;
        let mut metadata = proposal_crdt.get_metadata().clone();
        metadata.status = new_status;
        metadata.updated_at = self.time_provider.unix_seconds();

        proposal_crdt.update_metadata(metadata, self.node_id.clone())?;
        self.update_proposal_crdt(proposal_id, proposal_crdt)?;

        debug!("Successfully updated proposal {} status", proposal_id);
        Ok(())
    }

    /// Check if a proposal exists.
    pub fn proposal_exists(&self, proposal_id: &str) -> bool {
        match self.proposal_map.read() {
            Ok(map) => map.contains_key(&proposal_id.to_string()),
            Err(_) => {
                error!("Failed to acquire read lock for proposal existence check");
                false
            }
        }
    }

    /// Get all proposal IDs.
    pub fn get_all_proposals(&self) -> Vec<String> {
        match self.proposal_map.read() {
            Ok(map) => map.keys().into_iter().collect(),
            Err(_) => {
                error!("Failed to acquire read lock for proposal listing");
                Vec::new()
            }
        }
    }

    /// Get detailed information about a proposal.
    pub fn get_proposal_info(&self, proposal_id: &str) -> Result<ProposalInfo, CommonError> {
        let proposal_crdt = self.get_proposal_crdt(proposal_id)?;
        let metadata = proposal_crdt.get_metadata().clone();
        let vote_tally = proposal_crdt.calculate_tally();
        let has_quorum = proposal_crdt.has_quorum(metadata.required_quorum);

        let is_approved = match metadata.status {
            ProposalStatus::Open => {
                let current_time = self.time_provider.unix_seconds();

                if current_time > metadata.voting_deadline {
                    Some(has_quorum && proposal_crdt.is_approved(metadata.required_approval))
                } else {
                    None // Still voting
                }
            }
            ProposalStatus::Approved => Some(true),
            ProposalStatus::Rejected => Some(false),
            ProposalStatus::Expired => Some(false),
            ProposalStatus::Withdrawn => Some(false),
        };

        Ok(ProposalInfo {
            metadata,
            vote_tally,
            has_quorum,
            is_approved,
        })
    }

    /// Get proposals by status.
    pub fn get_proposals_by_status(&self, status: ProposalStatus) -> Vec<String> {
        let mut matching_proposals = Vec::new();

        for proposal_id in self.get_all_proposals() {
            if let Ok(info) = self.get_proposal_info(&proposal_id) {
                if info.metadata.status == status {
                    matching_proposals.push(proposal_id);
                }
            }
        }

        matching_proposals
    }

    /// Get proposals by proposer.
    pub fn get_proposals_by_proposer(&self, proposer: &Did) -> Vec<String> {
        let mut proposer_proposals = Vec::new();

        for proposal_id in self.get_all_proposals() {
            if let Ok(info) = self.get_proposal_info(&proposal_id) {
                if info.metadata.proposer == *proposer {
                    proposer_proposals.push(proposal_id);
                }
            }
        }

        proposer_proposals
    }

    /// Count active proposals by a proposer.
    pub fn count_active_proposals_by_proposer(&self, proposer: &Did) -> usize {
        let proposer_proposals = self.get_proposals_by_proposer(proposer);
        proposer_proposals
            .into_iter()
            .filter(|proposal_id| {
                if let Ok(info) = self.get_proposal_info(proposal_id) {
                    matches!(info.metadata.status, ProposalStatus::Open)
                } else {
                    false
                }
            })
            .count()
    }

    /// Process expired proposals (mark them as expired if deadline passed).
    pub fn process_expired_proposals(&self) -> Result<Vec<String>, CommonError> {
        if !self.config.auto_expire_proposals {
            return Ok(Vec::new());
        }

        let current_time = self.time_provider.unix_seconds();

        let mut expired_proposals = Vec::new();

        for proposal_id in self.get_proposals_by_status(ProposalStatus::Open) {
            if let Ok(info) = self.get_proposal_info(&proposal_id) {
                if current_time > info.metadata.voting_deadline {
                    // Determine final status based on votes
                    let final_status = if info.has_quorum
                        && info.vote_tally.approval_percentage >= info.metadata.required_approval
                    {
                        ProposalStatus::Approved
                    } else {
                        ProposalStatus::Expired
                    };

                    self.update_proposal_status(&proposal_id, final_status)?;
                    expired_proposals.push(proposal_id);
                }
            }
        }

        if !expired_proposals.is_empty() {
            debug!("Processed {} expired proposals", expired_proposals.len());
        }

        Ok(expired_proposals)
    }

    /// Get statistics about the proposal state manager.
    pub fn get_stats(&self) -> Result<CRDTProposalStateStats, CommonError> {
        let all_proposals = self.get_all_proposals();
        let total_proposals = all_proposals.len() as u64;

        let mut open_proposals = 0u64;
        let mut approved_proposals = 0u64;
        let mut rejected_proposals = 0u64;
        let mut expired_proposals = 0u64;
        let mut total_votes = 0u64;

        for proposal_id in &all_proposals {
            if let Ok(info) = self.get_proposal_info(proposal_id) {
                match info.metadata.status {
                    ProposalStatus::Open => open_proposals += 1,
                    ProposalStatus::Approved => approved_proposals += 1,
                    ProposalStatus::Rejected => rejected_proposals += 1,
                    ProposalStatus::Expired => expired_proposals += 1,
                    ProposalStatus::Withdrawn => {} // Not counted in main categories
                }
                total_votes += info.vote_tally.total;
            }
        }

        let average_votes_per_proposal = if total_proposals > 0 {
            total_votes / total_proposals
        } else {
            0
        };

        Ok(CRDTProposalStateStats {
            total_proposals,
            open_proposals,
            approved_proposals,
            rejected_proposals,
            expired_proposals,
            total_votes,
            average_votes_per_proposal,
            node_id: self.node_id.clone(),
        })
    }

    /// Get a proposal CRDT by ID.
    fn get_proposal_crdt(&self, proposal_id: &str) -> Result<ProposalCRDT, CommonError> {
        let map = self
            .proposal_map
            .read()
            .map_err(|_| CommonError::LockError("Failed to acquire read lock".to_string()))?;

        if let Some(proposal_crdt) = map.get(&proposal_id.to_string()) {
            Ok(proposal_crdt.clone())
        } else {
            Err(CommonError::InvalidInputError(format!(
                "Proposal {} not found",
                proposal_id
            )))
        }
    }

    /// Update a proposal CRDT in the map.
    fn update_proposal_crdt(
        &self,
        proposal_id: &str,
        proposal_crdt: ProposalCRDT,
    ) -> Result<(), CommonError> {
        let mut map = self
            .proposal_map
            .write()
            .map_err(|_| CommonError::LockError("Failed to acquire write lock".to_string()))?;

        map.put(proposal_id.to_string(), proposal_crdt, self.node_id.clone())
            .map_err(|e| CommonError::CRDTError(format!("Failed to update proposal: {}", e)))?;

        Ok(())
    }
}

impl Clone for CRDTProposalState {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            proposal_map: self.proposal_map.clone(),
            config: self.config.clone(),
            time_provider: self.time_provider.clone(),
        }
    }
}

impl std::fmt::Debug for CRDTProposalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRDTProposalState")
            .field("node_id", &self.node_id)
            .field("proposal_map", &"<CRDTMap>")
            .field("config", &self.config)
            .field("time_provider", &"<TimeProvider>")
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

    fn current_timestamp() -> u64 {
        1640995200 // Fixed timestamp for deterministic testing (2022-01-01)
    }

    fn test_time_provider() -> Arc<dyn TimeProvider> {
        Arc::new(icn_common::FixedTimeProvider::new(current_timestamp()))
    }

    #[test]
    fn test_crdt_proposal_state_creation() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());
        assert_eq!(state.node_id().as_str(), "test_node");
        assert_eq!(state.get_all_proposals().len(), 0);
    }

    #[test]
    fn test_create_proposal() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        let result = state.create_proposal(
            "proposal_1".to_string(),
            "Test Proposal".to_string(),
            "A test proposal for unit testing".to_string(),
            alice_did(),
            None,
            None,
            None,
        );

        assert!(result.is_ok());
        assert!(state.proposal_exists("proposal_1"));

        let info = state.get_proposal_info("proposal_1").unwrap();
        assert_eq!(info.metadata.title, "Test Proposal");
        assert_eq!(info.metadata.proposer, alice_did());
        assert_eq!(info.metadata.status, ProposalStatus::Open);
    }

    #[test]
    fn test_duplicate_proposal_creation() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();

        let result = state.create_proposal(
            "proposal_1".to_string(),
            "Duplicate Proposal".to_string(),
            "This should fail".to_string(),
            bob_did(),
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_cast_vote() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                alice_did(),
                Some(current_timestamp() + 3600), // 1 hour from now
                Some(2),
                Some(50),
            )
            .unwrap();

        let result = state.cast_vote(
            "proposal_1",
            bob_did(),
            VoteDecision::Approve,
            Some("I approve this proposal".to_string()),
        );

        assert!(result.is_ok());

        let info = state.get_proposal_info("proposal_1").unwrap();
        assert_eq!(info.vote_tally.approve, 1);
        assert_eq!(info.vote_tally.total, 1);
        assert!(!info.has_quorum); // Needs 2 votes for quorum
    }

    #[test]
    fn test_proposal_quorum_and_approval() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                alice_did(),
                Some(current_timestamp() + 3600),
                Some(2),  // Quorum of 2
                Some(60), // 60% approval required
            )
            .unwrap();

        // First vote
        state
            .cast_vote("proposal_1", bob_did(), VoteDecision::Approve, None)
            .unwrap();

        let info = state.get_proposal_info("proposal_1").unwrap();
        assert!(!info.has_quorum);
        assert_eq!(info.vote_tally.approval_percentage, 100);

        // Second vote to reach quorum
        state
            .cast_vote("proposal_1", charlie_did(), VoteDecision::Approve, None)
            .unwrap();

        let info = state.get_proposal_info("proposal_1").unwrap();
        assert!(info.has_quorum);
        assert_eq!(info.vote_tally.approval_percentage, 100);
        assert_eq!(info.vote_tally.total, 2);
    }

    #[test]
    fn test_vote_on_expired_proposal() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "Expired Proposal".to_string(),
                "This proposal has expired".to_string(),
                alice_did(),
                Some(current_timestamp() - 3600), // 1 hour ago (expired)
                Some(1),
                Some(50),
            )
            .unwrap();

        let result = state.cast_vote("proposal_1", bob_did(), VoteDecision::Approve, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_proposal_status() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();

        let result = state.update_proposal_status("proposal_1", ProposalStatus::Withdrawn);
        assert!(result.is_ok());

        let info = state.get_proposal_info("proposal_1").unwrap();
        assert_eq!(info.metadata.status, ProposalStatus::Withdrawn);
    }

    #[test]
    fn test_get_proposals_by_status() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "P1".to_string(),
                "D1".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state
            .create_proposal(
                "proposal_2".to_string(),
                "P2".to_string(),
                "D2".to_string(),
                bob_did(),
                None,
                None,
                None,
            )
            .unwrap();

        state
            .update_proposal_status("proposal_2", ProposalStatus::Approved)
            .unwrap();

        let open_proposals = state.get_proposals_by_status(ProposalStatus::Open);
        let approved_proposals = state.get_proposals_by_status(ProposalStatus::Approved);

        assert_eq!(open_proposals.len(), 1);
        assert!(open_proposals.contains(&"proposal_1".to_string()));

        assert_eq!(approved_proposals.len(), 1);
        assert!(approved_proposals.contains(&"proposal_2".to_string()));
    }

    #[test]
    fn test_get_proposals_by_proposer() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "P1".to_string(),
                "D1".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state
            .create_proposal(
                "proposal_2".to_string(),
                "P2".to_string(),
                "D2".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state
            .create_proposal(
                "proposal_3".to_string(),
                "P3".to_string(),
                "D3".to_string(),
                bob_did(),
                None,
                None,
                None,
            )
            .unwrap();

        let alice_proposals = state.get_proposals_by_proposer(&alice_did());
        let bob_proposals = state.get_proposals_by_proposer(&bob_did());

        assert_eq!(alice_proposals.len(), 2);
        assert_eq!(bob_proposals.len(), 1);
    }

    #[test]
    fn test_merge_proposal_states() {
        let state1 = CRDTProposalState::with_node_id("node1".to_string(), test_time_provider());
        let state2 = CRDTProposalState::with_node_id("node2".to_string(), test_time_provider());

        // Create proposals on different nodes
        state1
            .create_proposal(
                "proposal_1".to_string(),
                "P1".to_string(),
                "D1".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state2
            .create_proposal(
                "proposal_2".to_string(),
                "P2".to_string(),
                "D2".to_string(),
                bob_did(),
                None,
                None,
                None,
            )
            .unwrap();

        // Cast votes on different nodes for the same proposal (if it existed on both)
        state1
            .create_proposal(
                "shared_proposal".to_string(),
                "Shared".to_string(),
                "Shared proposal".to_string(),
                charlie_did(),
                Some(current_timestamp() + 3600),
                None,
                None,
            )
            .unwrap();
        state2
            .create_proposal(
                "shared_proposal".to_string(),
                "Shared".to_string(),
                "Shared proposal".to_string(),
                charlie_did(),
                Some(current_timestamp() + 3600),
                None,
                None,
            )
            .unwrap();

        state1
            .cast_vote("shared_proposal", alice_did(), VoteDecision::Approve, None)
            .unwrap();
        state2
            .cast_vote("shared_proposal", bob_did(), VoteDecision::Reject, None)
            .unwrap();

        // Before merge
        assert!(state1.proposal_exists("proposal_1"));
        assert!(!state1.proposal_exists("proposal_2"));
        assert!(state2.proposal_exists("proposal_2"));
        assert!(!state2.proposal_exists("proposal_1"));

        // Merge state2 into state1
        state1.merge(&state2).unwrap();

        // After merge, state1 should have all proposals
        assert!(state1.proposal_exists("proposal_1"));
        assert!(state1.proposal_exists("proposal_2"));
        assert!(state1.proposal_exists("shared_proposal"));

        // Check that votes were merged for shared proposal
        let info = state1.get_proposal_info("shared_proposal").unwrap();
        assert_eq!(info.vote_tally.total, 2);
        assert_eq!(info.vote_tally.approve, 1);
        assert_eq!(info.vote_tally.reject, 1);
    }

    #[test]
    fn test_proposal_stats() {
        let state = CRDTProposalState::with_node_id("test_node".to_string(), test_time_provider());

        state
            .create_proposal(
                "proposal_1".to_string(),
                "P1".to_string(),
                "D1".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state
            .create_proposal(
                "proposal_2".to_string(),
                "P2".to_string(),
                "D2".to_string(),
                bob_did(),
                None,
                None,
                None,
            )
            .unwrap();

        state
            .cast_vote("proposal_1", alice_did(), VoteDecision::Approve, None)
            .unwrap();
        state
            .cast_vote("proposal_1", bob_did(), VoteDecision::Reject, None)
            .unwrap();
        state
            .cast_vote("proposal_2", charlie_did(), VoteDecision::Approve, None)
            .unwrap();

        state
            .update_proposal_status("proposal_2", ProposalStatus::Approved)
            .unwrap();

        let stats = state.get_stats().unwrap();
        assert_eq!(stats.total_proposals, 2);
        assert_eq!(stats.open_proposals, 1);
        assert_eq!(stats.approved_proposals, 1);
        assert_eq!(stats.total_votes, 3);
        assert_eq!(stats.average_votes_per_proposal, 1); // 3 votes / 2 proposals = 1.5, rounded down
        assert_eq!(stats.node_id.as_str(), "test_node");
    }

    #[test]
    fn test_max_proposals_per_proposer() {
        let mut config = CRDTProposalStateConfig::default();
        config.node_id = "test_node".to_string();
        config.max_proposals_per_proposer = 2;

        let state = CRDTProposalState::new(config, test_time_provider());

        // First two proposals should succeed
        state
            .create_proposal(
                "proposal_1".to_string(),
                "P1".to_string(),
                "D1".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();
        state
            .create_proposal(
                "proposal_2".to_string(),
                "P2".to_string(),
                "D2".to_string(),
                alice_did(),
                None,
                None,
                None,
            )
            .unwrap();

        // Third proposal should fail due to limit
        let result = state.create_proposal(
            "proposal_3".to_string(),
            "P3".to_string(),
            "D3".to_string(),
            alice_did(),
            None,
            None,
            None,
        );
        assert!(result.is_err());

        // But Bob should still be able to create proposals
        let result = state.create_proposal(
            "proposal_4".to_string(),
            "P4".to_string(),
            "D4".to_string(),
            bob_did(),
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }
}
