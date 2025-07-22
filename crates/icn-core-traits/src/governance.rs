//! Governance service traits and types

use crate::CoreTraitsError;
use async_trait::async_trait;
use icn_common::{Cid, Did};
use serde::{Deserialize, Serialize};

/// Governance provider trait
#[async_trait]
pub trait GovernanceProvider: Send + Sync {
    /// Submit a new proposal
    async fn submit_proposal(
        &self,
        submitter: &Did,
        title: String,
        description: String,
        policy_cid: Option<Cid>,
    ) -> Result<String, CoreTraitsError>; // Returns proposal ID

    /// Get proposal details
    async fn get_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<Option<ProposalInfo>, CoreTraitsError>;

    /// Get all active proposals
    async fn get_active_proposals(&self) -> Result<Vec<ProposalInfo>, CoreTraitsError>;

    /// Check governance permissions
    async fn check_permissions(&self, did: &Did, action: &str) -> Result<bool, CoreTraitsError>;
}

/// Proposal provider trait
#[async_trait]
pub trait ProposalProvider: Send + Sync {
    /// Create a new proposal
    async fn create_proposal(
        &self,
        submitter: &Did,
        proposal_data: ProposalData,
    ) -> Result<String, CoreTraitsError>;

    /// Update proposal status
    async fn update_proposal_status(
        &self,
        proposal_id: &str,
        status: ProposalStatus,
    ) -> Result<(), CoreTraitsError>;

    /// Get proposal by ID
    async fn get_proposal_by_id(
        &self,
        proposal_id: &str,
    ) -> Result<Option<ProposalInfo>, CoreTraitsError>;

    /// List proposals by status
    async fn list_proposals_by_status(
        &self,
        status: ProposalStatus,
    ) -> Result<Vec<ProposalInfo>, CoreTraitsError>;
}

/// Voting provider trait
#[async_trait]
pub trait VotingProvider: Send + Sync {
    /// Cast a vote on a proposal
    async fn cast_vote(
        &self,
        voter: &Did,
        proposal_id: &str,
        vote: VoteOption,
    ) -> Result<(), CoreTraitsError>;

    /// Get vote results for a proposal
    async fn get_vote_results(&self, proposal_id: &str) -> Result<VoteResults, CoreTraitsError>;

    /// Check if a DID has voted on a proposal
    async fn has_voted(&self, voter: &Did, proposal_id: &str) -> Result<bool, CoreTraitsError>;

    /// Calculate voting weight for a DID
    async fn calculate_voting_weight(&self, voter: &Did) -> Result<f64, CoreTraitsError>;
}

/// Proposal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub submitter: Did,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub voting_deadline: Option<u64>,
    pub policy_cid: Option<Cid>,
}

/// Proposal data for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalData {
    pub title: String,
    pub description: String,
    pub policy_cid: Option<Cid>,
    pub voting_duration_days: Option<u32>,
}

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

/// Vote option
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

/// Vote results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResults {
    pub proposal_id: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub total_weight: f64,
    pub participation_rate: f64,
    pub quorum_met: bool,
}
