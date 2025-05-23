#![doc = include_str!("../README.md")]

//! # ICN Governance Crate
//! This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).
//! It handles proposal systems, voting procedures, quorum logic, and decision execution,
//! focusing on transparency, fairness, and flexibility.

use icn_common::{NodeInfo, CommonError, Did};
use icn_network::PeerId;
use std::collections::HashMap;
#[cfg(feature = "persist-sled")]
use std::path::PathBuf;

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

#[derive(Debug, Clone, PartialEq)]
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

// Define the Backend enum
#[derive(Debug)]
enum Backend {
    InMemory {
        proposals: HashMap<ProposalId, Proposal>,
    },
    #[cfg(feature = "persist-sled")]
    Sled {
        db: sled::Db,
        // Define names for trees (sled's equivalent of tables/collections)
        // We'll store proposals in a tree named "proposals"
        // Key: ProposalId.0 (String), Value: bincode-serialized Proposal
        proposals_tree_name: String,
    },
}

/// Manages governance proposals and voting.
#[derive(Debug)] // Removed Default, as `new` is now more explicit
pub struct GovernanceModule {
    backend: Backend,
    // TODO: Add member list, voting rules (quorum, threshold), etc.
}

impl GovernanceModule {
    /// Creates a new GovernanceModule with an in-memory backend.
    pub fn new() -> Self {
        GovernanceModule {
            backend: Backend::InMemory {
                proposals: HashMap::new(),
            },
        }
    }

    #[cfg(feature = "persist-sled")]
    /// Creates a new GovernanceModule with a sled persistent backend.
    pub fn new_sled(db_path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(db_path)
            .map_err(|e| CommonError::StorageError(format!("Failed to open sled database: {}", e)))?;
        
        let proposals_tree_name = "proposals_v1".to_string(); // versioned tree name
        // sled automatically creates trees when first accessed, so no explicit creation needed here.

        Ok(GovernanceModule {
            backend: Backend::Sled {
                db,
                proposals_tree_name,
            },
        })
    }

    pub fn submit_proposal(&mut self, proposer: Did, proposal_type: ProposalType, description: String, duration_secs: u64) -> Result<ProposalId, CommonError> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let desc_hash_part = description.chars().take(10).collect::<String>();
        let proposal_id_str = format!("prop:{}:{}:{}", proposer.to_string(), desc_hash_part, now);
        let proposal_id = ProposalId(proposal_id_str);

        let proposal = Proposal {
            id: proposal_id.clone(),
            proposer,
            proposal_type,
            description,
            created_at: now,
            voting_deadline: now + duration_secs,
            status: ProposalStatus::VotingOpen,
            votes: HashMap::new(),
        };

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                if proposals.contains_key(&proposal_id) {
                    return Err(CommonError::ProposalExists(proposal_id.0.clone()));
                }
                proposals.insert(proposal_id.clone(), proposal);
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { db, proposals_tree_name } => {
                let tree = db.open_tree(proposals_tree_name)
                    .map_err(|e| CommonError::StorageError(format!("Failed to open proposals tree: {}", e)))?;
                
                let key = proposal_id.0.as_bytes();
                if tree.contains_key(key)
                    .map_err(|e| CommonError::StorageError(format!("Failed to check key existence in proposals tree: {}", e)))? {
                    return Err(CommonError::ProposalExists(proposal_id.0.clone()));
                }

                // Serialize using bincode for sled
                let encoded_proposal = bincode::serialize(&proposal)
                    .map_err(|e| CommonError::SerializationError(format!("Failed to serialize proposal: {}", e)))?;
                
                tree.insert(key, encoded_proposal)
                    .map_err(|e| CommonError::StorageError(format!("Failed to insert proposal into sled: {}", e)))?;
            }
        }
        Ok(proposal_id)
    }

    pub fn cast_vote(&mut self, voter: Did, proposal_id: &ProposalId, option: VoteOption) -> Result<(), CommonError> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        match &mut self.backend {
            Backend::InMemory { proposals } => {
                let proposal = proposals.get_mut(proposal_id)
                    .ok_or_else(|| CommonError::ProposalNotFound(proposal_id.0.clone()))?;

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
            #[cfg(feature = "persist-sled")]
            Backend::Sled { db, proposals_tree_name } => {
                let tree = db.open_tree(proposals_tree_name)
                    .map_err(|e| CommonError::StorageError(format!("Failed to open proposals tree: {}", e)))?;
                
                let key = proposal_id.0.as_bytes();
                let proposal_bytes = tree.get(key)
                    .map_err(|e| CommonError::StorageError(format!("Failed to get proposal from sled: {}", e)))?
                    .ok_or_else(|| CommonError::ProposalNotFound(proposal_id.0.clone()))?;
                
                let mut proposal: Proposal = bincode::deserialize(&proposal_bytes)
                    .map_err(|e| CommonError::SerializationError(format!("Failed to deserialize proposal: {}", e)))?;

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

                let encoded_proposal = bincode::serialize(&proposal)
                    .map_err(|e| CommonError::SerializationError(format!("Failed to serialize updated proposal: {}", e)))?;
                
                tree.insert(key, encoded_proposal)
                    .map_err(|e| CommonError::StorageError(format!("Failed to insert updated proposal into sled: {}", e)))?;
                Ok(())
            }
        }
    }

    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Result<Option<Proposal>, CommonError> {
        match &self.backend {
            Backend::InMemory { proposals } => {
                Ok(proposals.get(proposal_id).cloned())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { db, proposals_tree_name } => {
                let tree = db.open_tree(proposals_tree_name)
                    .map_err(|e| CommonError::StorageError(format!("Failed to open proposals tree: {}", e)))?;
                let key = proposal_id.0.as_bytes();
                match tree.get(key) {
                    Ok(Some(proposal_bytes)) => {
                        let proposal: Proposal = bincode::deserialize(&proposal_bytes)
                            .map_err(|e| CommonError::SerializationError(format!("Failed to deserialize proposal: {}", e)))?;
                        Ok(Some(proposal))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(CommonError::StorageError(format!("Failed to get proposal from sled: {}", e))),
                }
            }
        }
    }

    pub fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError> {
        match &self.backend {
            Backend::InMemory { proposals } => {
                Ok(proposals.values().cloned().collect())
            }
            #[cfg(feature = "persist-sled")]
            Backend::Sled { db, proposals_tree_name } => {
                let tree = db.open_tree(proposals_tree_name)
                    .map_err(|e| CommonError::StorageError(format!("Failed to open proposals tree: {}", e)))?;
                
                let mut result_proposals = Vec::new();
                for item in tree.iter() {
                    let (_key, val_bytes) = item
                        .map_err(|e| CommonError::StorageError(format!("Failed to iterate over proposals in sled: {}", e)))?;
                    let proposal: Proposal = bincode::deserialize(&val_bytes)
                        .map_err(|e| CommonError::SerializationError(format!("Failed to deserialize proposal from sled: {}", e)))?;
                    result_proposals.push(proposal);
                }
                Ok(result_proposals)
            }
        }
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
            version: icn_common::ICN_CORE_VERSION.to_string(),
            status_message: "Governance active".to_string(),
        };
        let result = submit_governance_proposal(&node_info, 101);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("101"));
    }
}
