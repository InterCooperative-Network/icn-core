#![doc = include_str!("../README.md")]

//! # ICN Governance Crate
//! This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).
//! It handles proposal systems, voting procedures, quorum logic, and decision execution,
//! focusing on transparency, fairness, and flexibility.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION, Cid, Did, PeerId};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

// --- Proposal System ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProposalId(pub String); // Could be a hash of the proposal content

impl std::str::FromStr for ProposalId {
    type Err = icn_common::CommonError; // Or a more specific error type if desired
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            // Or based on whatever validation rules ProposalId might have.
            // For now, just ensuring it's not empty.
            Err(icn_common::CommonError::InvalidInputError("Proposal ID cannot be empty".to_string()))
        } else {
            Ok(ProposalId(s.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProposalType {
    SystemParameterChange(String, String), // param_name, new_value
    NewMemberInvitation(Did), // DID of the member to invite
    SoftwareUpgrade(String), // Version or identifier for the upgrade
    GenericText(String), // For general purpose proposals
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProposalStatus {
    Pending,     // Newly created, awaiting votes
    VotingOpen,  // Actively collecting votes
    Accepted,    // Voting period ended, quorum and threshold met
    Rejected,    // Voting period ended, quorum or threshold not met
    Executed,    // For proposals that have an on-chain/system effect
    Failed,      // Execution failed
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Did, // DID of the entity that proposed this
    pub proposal_type: ProposalType,
    pub description: String,
    pub created_at: u64, // Timestamp (e.g., Unix epoch seconds)
    pub voting_deadline: u64, // Timestamp for when voting closes
    pub status: ProposalStatus,
    pub votes: HashMap<Did, Vote>, // Voter DID to their vote
    // Potentially, threshold and quorum requirements could be part of the proposal type or global config
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vote {
    pub voter: Did,
    pub proposal_id: ProposalId,
    pub option: VoteOption,
    pub voted_at: u64, // Timestamp
}

/// Manages governance proposals and voting.
#[derive(Debug, Default)]
pub struct GovernanceModule {
    proposals: HashMap<ProposalId, Proposal>,
    // TODO: Add member list, voting rules (quorum, threshold), etc.
}

impl GovernanceModule {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn submit_proposal(&mut self, proposer: Did, proposal_type: ProposalType, description: String, duration_secs: u64) -> Result<ProposalId, CommonError> {
        // Simulate getting current time
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        // Simple ID generation: ProposerDIDString:ShortDescriptionHash:Timestamp
        let desc_hash_part = description.chars().take(10).collect::<String>(); // First 10 chars as a stand-in for a hash
        // Use proposer.to_string() which correctly formats the DID as a string
        let proposal_id_str = format!("{}:{}:{}", proposer.to_string(), desc_hash_part, now);
        let proposal_id = ProposalId(proposal_id_str);

        if self.proposals.contains_key(&proposal_id) {
            // Use .0 to get the inner String for the error message, consistent with ProposalId structure
            return Err(CommonError::ProposalExists(proposal_id.0.clone()));
        }

        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer,
            proposal_type,
            description,
            created_at: now,
            voting_deadline: now + duration_secs,
            status: ProposalStatus::VotingOpen, // Automatically open for voting
            votes: HashMap::new(),
        };
        self.proposals.insert(proposal_id.clone(), proposal);
        Ok(proposal_id)
    }

    pub fn cast_vote(&mut self, voter: Did, proposal_id: &ProposalId, option: VoteOption) -> Result<(), CommonError> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| CommonError::ProposalNotFound(proposal_id.0.clone()))?;

        // TODO: Validate voter eligibility (e.g., is a member)
        // TODO: Check if voting period is still open
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        if now > proposal.voting_deadline {
            return Err(CommonError::VotingClosed(proposal_id.0.clone()));
        }
        if proposal.status != ProposalStatus::VotingOpen {
             return Err(CommonError::VotingClosed(format!("Proposal {} not open for voting, status: {:?}", proposal_id.0, proposal.status)));
        }

        let vote = Vote {
            voter: voter.clone(),
            proposal_id: proposal_id.clone(),
            option,
            voted_at: now,
        };
        proposal.votes.insert(voter, vote);
        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn list_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    // TODO: Implement tally_votes, close_voting_period, execute_proposal methods
    // TODO: Add logic for quorum and threshold checks.
}

// --- Stubbed Federation Sync API ---

/// Simulates a request to sync governance state (proposals, votes) with another federation member.
pub fn request_federation_sync(target_peer: &PeerId, since_timestamp: Option<u64>) -> Result<String, CommonError> {
    // In a real scenario, this would involve network communication.
    // For now, it's a placeholder.
    Ok(format!("Federation sync request sent to {} (since {:?}). Awaiting response with updated governance data.", 
                target_peer.0, since_timestamp))
}

/// Placeholder function demonstrating use of common types for governance.
pub fn submit_governance_proposal(info: &NodeInfo, proposal_id: u32) -> Result<String, CommonError> {
    Ok(format!("Submitted governance proposal {} from node: {} (v{})", proposal_id, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_governance_proposal() {
        let node_info = NodeInfo {
            name: "GovNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Governance active".to_string(),
        };
        let result = submit_governance_proposal(&node_info, 101);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("101"));
    }
}
