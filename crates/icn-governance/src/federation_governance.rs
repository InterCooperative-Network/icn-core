//! Federation Trust Governance
//!
//! This module extends the governance system to work with the scoped federation trust framework.
//! It provides governance mechanisms that respect federation trust contexts and inheritance.

use crate::{ProposalId};
use icn_common::{Did, CommonError};
use icn_identity::{
    TrustContext, FederationId, TrustPolicyEngine, TrustValidationResult, TrustLevel,
};
use serde::{Deserialize, Serialize};
/// Result of voting on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingResult {
    /// Proposal passed
    Passed {
        yes_votes: usize,
        no_votes: usize,
        total_eligible: usize,
    },
    /// Proposal failed
    Failed(String),
}

/// Governance error types
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Proposal not found: {0:?}")]
    ProposalNotFound(ProposalId),
    
    #[error("Proposal not open: {0:?}")]
    ProposalNotOpen(ProposalId),
    
    #[error("Voting deadline passed: {0:?}")]
    VotingDeadlinePassed(ProposalId),
    
    #[error("Voting still open: {0:?}")]
    VotingStillOpen(ProposalId),
    
    #[error("Additional validation required: {0:?}")]
    AdditionalValidationRequired(Vec<String>),
    
    #[error("Common error: {0}")]
    Common(#[from] CommonError),
}

/// Governance action that requires trust validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    /// Submit a proposal
    SubmitProposal { proposal_id: ProposalId },
    /// Vote on a proposal
    Vote { proposal_id: ProposalId, vote: bool },
    /// Execute a proposal
    ExecuteProposal { proposal_id: ProposalId },
    /// Modify federation membership
    ModifyMembership { target: Did, action: MembershipAction },
    /// Update trust relationships
    UpdateTrust { target: Did, new_level: TrustLevel },
    /// Bridge federations
    CreateBridge { target_federation: FederationId },
}

/// Membership action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MembershipAction {
    /// Add member to federation
    Add,
    /// Remove member from federation
    Remove,
    /// Suspend member
    Suspend,
    /// Restore suspended member
    Restore,
}

/// Governance policy with trust requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAwareGovernancePolicy {
    /// Action this policy applies to
    pub action: GovernanceAction,
    /// Required trust context
    pub required_context: TrustContext,
    /// Minimum trust level required
    pub min_trust_level: TrustLevel,
    /// Whether federation membership is required
    pub require_federation_membership: bool,
    /// Minimum voting threshold (0.0-1.0)
    pub voting_threshold: f64,
    /// Quorum requirement (0.0-1.0)
    pub quorum_requirement: f64,
    /// Whether cross-federation participation is allowed
    pub allow_cross_federation: bool,
}

/// Federation governance engine that enforces trust-aware policies
#[derive(Debug)]
pub struct FederationGovernanceEngine {
    /// Trust policy engine
    trust_engine: TrustPolicyEngine,
    /// Governance policies indexed by action type
    policies: HashMap<String, TrustAwareGovernancePolicy>,
    /// Active proposals
    proposals: HashMap<ProposalId, FederationProposal>,
    /// Federation this engine serves
    federation_id: Option<FederationId>,
}

/// Proposal within a federation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationProposal {
    /// Proposal ID
    pub id: ProposalId,
    /// Proposer DID
    pub proposer: Did,
    /// Federation this proposal belongs to
    pub federation: FederationId,
    /// Trust context required for participation
    pub trust_context: TrustContext,
    /// Proposal content
    pub content: String,
    /// Votes cast
    pub votes: HashMap<Did, bool>,
    /// Proposal status
    pub status: ProposalStatus,
    /// Creation timestamp
    pub created_at: u64,
    /// Voting deadline
    pub voting_deadline: u64,
}

/// Status of a federation proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Open for voting
    Open,
    /// Passed and ready for execution
    Passed,
    /// Failed to meet requirements
    Failed,
    /// Executed successfully
    Executed,
    /// Cancelled
    Cancelled,
}

/// Result of governance action validation
#[derive(Debug)]
pub enum GovernanceValidationResult {
    /// Action is allowed
    Allowed,
    /// Action is denied
    Denied { reason: String },
    /// Action requires additional validation
    RequiresValidation { additional_checks: Vec<String> },
}

impl FederationGovernanceEngine {
    /// Create a new federation governance engine
    pub fn new(trust_engine: TrustPolicyEngine, federation_id: Option<FederationId>) -> Self {
        Self {
            trust_engine,
            policies: HashMap::new(),
            proposals: HashMap::new(),
            federation_id,
        }
    }

    /// Add a governance policy
    pub fn add_policy(&mut self, action_key: String, policy: TrustAwareGovernancePolicy) {
        self.policies.insert(action_key, policy);
    }

    /// Submit a proposal with trust validation
    pub fn submit_proposal(
        &mut self,
        proposer: &Did,
        federation: FederationId,
        trust_context: TrustContext,
        content: String,
        voting_deadline: u64,
    ) -> Result<ProposalId, GovernanceError> {
        // Validate proposer has permission to submit proposals
        let validation = self.validate_action(
            proposer,
            &GovernanceAction::SubmitProposal {
                proposal_id: ProposalId("dummy".to_string()),
            },
        );

        match validation {
            GovernanceValidationResult::Allowed => {}
            GovernanceValidationResult::Denied { reason } => {
                return Err(GovernanceError::Unauthorized(reason));
            }
            GovernanceValidationResult::RequiresValidation { additional_checks } => {
                return Err(GovernanceError::AdditionalValidationRequired(additional_checks));
            }
        }

        // Create proposal
        let proposal_id = ProposalId(format!("prop_{}_{}", federation.as_str(), chrono::Utc::now().timestamp()));
        let proposal = FederationProposal {
            id: proposal_id.clone(),
            proposer: proposer.clone(),
            federation,
            trust_context,
            content,
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            created_at: chrono::Utc::now().timestamp() as u64,
            voting_deadline,
        };

        self.proposals.insert(proposal_id.clone(), proposal);
        Ok(proposal_id)
    }

    /// Cast a vote on a proposal with trust validation
    pub fn vote(
        &mut self,
        voter: &Did,
        proposal_id: &ProposalId,
        vote: bool,
    ) -> Result<(), GovernanceError> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.clone()))?;

        if proposal.status != ProposalStatus::Open {
            return Err(GovernanceError::ProposalNotOpen(proposal_id.clone()));
        }

        if chrono::Utc::now().timestamp() as u64 > proposal.voting_deadline {
            return Err(GovernanceError::VotingDeadlinePassed(proposal_id.clone()));
        }

        // Validate voter has permission in the trust context
        let validation_result = self.trust_engine.validate_trust(
            voter,
            &proposal.proposer,
            &proposal.trust_context,
            "vote",
        );

        match validation_result {
            TrustValidationResult::Allowed { .. } => {},
            TrustValidationResult::Denied { reason } => {
                return Err(GovernanceError::Unauthorized(format!("Vote denied: {}", reason)));
            }
        }

        // Check if voter is federation member (if required)
        if let Some(policy) = self.get_voting_policy(&proposal.trust_context) {
            if policy.require_federation_membership
                && !self.trust_engine.is_federation_member(voter, &proposal.federation)
            {
                return Err(GovernanceError::Unauthorized(
                    "Federation membership required for voting".to_string(),
                ));
            }
        }

        // Cast vote
        let proposal = self.proposals.get_mut(proposal_id).unwrap();
        proposal.votes.insert(voter.clone(), vote);

        Ok(())
    }

    /// Finalize a proposal and determine the result
    pub fn finalize_proposal(&mut self, proposal_id: &ProposalId) -> Result<VotingResult, GovernanceError> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.clone()))?;

        if proposal.status != ProposalStatus::Open {
            return Err(GovernanceError::ProposalNotOpen(proposal_id.clone()));
        }

        // Check if voting deadline has passed
        if chrono::Utc::now().timestamp() as u64 <= proposal.voting_deadline {
            return Err(GovernanceError::VotingStillOpen(proposal_id.clone()));
        }

        // Calculate results
        let total_votes = proposal.votes.len();
        let yes_votes = proposal.votes.values().filter(|&&v| v).count();
        let no_votes = total_votes - yes_votes;

        // Get policy requirements
        let policy = self.get_voting_policy(&proposal.trust_context);
        let required_threshold = policy.map(|p| p.voting_threshold).unwrap_or(0.5);
        let required_quorum = policy.map(|p| p.quorum_requirement).unwrap_or(0.3);

        // Check quorum
        let federation_size = self.get_federation_size(&proposal.federation);
        let quorum_met = if federation_size > 0 {
            (total_votes as f64) / (federation_size as f64) >= required_quorum
        } else {
            total_votes > 0
        };

        if !quorum_met {
            proposal.status = ProposalStatus::Failed;
            return Ok(VotingResult::Failed(format!("Quorum not met: {}/{} votes required", 
                (federation_size as f64 * required_quorum) as usize, federation_size)));
        }

        // Check threshold
        let yes_ratio = if total_votes > 0 {
            (yes_votes as f64) / (total_votes as f64)
        } else {
            0.0
        };

        if yes_ratio >= required_threshold {
            proposal.status = ProposalStatus::Passed;
            Ok(VotingResult::Passed {
                yes_votes,
                no_votes,
                total_eligible: federation_size,
            })
        } else {
            proposal.status = ProposalStatus::Failed;
            Ok(VotingResult::Failed(format!("Threshold not met: {:.2}% yes votes, {:.2}% required", 
                yes_ratio * 100.0, required_threshold * 100.0)))
        }
    }

    /// Validate if an action is allowed for a given actor
    pub fn validate_action(
        &self,
        actor: &Did,
        action: &GovernanceAction,
    ) -> GovernanceValidationResult {
        let action_key = self.action_key(action);
        
        if let Some(policy) = self.policies.get(&action_key) {
            // Check federation membership if required
            if policy.require_federation_membership {
                if let Some(fed_id) = &self.federation_id {
                    if !self.trust_engine.is_federation_member(actor, fed_id) {
                        return GovernanceValidationResult::Denied {
                            reason: "Federation membership required".to_string(),
                        };
                    }
                }
            }

            // For actions that involve other actors, validate trust relationship
            match action {
                GovernanceAction::Vote { proposal_id, .. } => {
                    if let Some(proposal) = self.proposals.get(proposal_id) {
                        let validation = self.trust_engine.validate_trust(
                            actor,
                            &proposal.proposer,
                            &proposal.trust_context,
                            "vote",
                        );
                        
                        match validation {
                            TrustValidationResult::Allowed { .. } => {
                                GovernanceValidationResult::Allowed
                            }
                            TrustValidationResult::Denied { reason } => {
                                GovernanceValidationResult::Denied { reason }
                            }
                        }
                    } else {
                        GovernanceValidationResult::Denied {
                            reason: "Proposal not found".to_string(),
                        }
                    }
                }
                _ => GovernanceValidationResult::Allowed,
            }
        } else {
            GovernanceValidationResult::Denied {
                reason: format!("No policy defined for action: {}", action_key),
            }
        }
    }

    /// Get voting policy for a trust context
    fn get_voting_policy(&self, context: &TrustContext) -> Option<&TrustAwareGovernancePolicy> {
        let vote_key = format!("vote_{}", context.as_str());
        self.policies.get(&vote_key)
    }

    /// Get federation size for quorum calculations
    fn get_federation_size(&self, federation: &FederationId) -> usize {
        // This would typically query the federation membership
        // For now, return a placeholder
        10 // TODO: Implement actual federation size lookup
    }

    /// Generate action key for policy lookup
    fn action_key(&self, action: &GovernanceAction) -> String {
        match action {
            GovernanceAction::SubmitProposal { .. } => "submit_proposal".to_string(),
            GovernanceAction::Vote { .. } => "vote".to_string(),
            GovernanceAction::ExecuteProposal { .. } => "execute_proposal".to_string(),
            GovernanceAction::ModifyMembership { action, .. } => {
                format!("modify_membership_{:?}", action).to_lowercase()
            }
            GovernanceAction::UpdateTrust { .. } => "update_trust".to_string(),
            GovernanceAction::CreateBridge { .. } => "create_bridge".to_string(),
        }
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, id: &ProposalId) -> Option<&FederationProposal> {
        self.proposals.get(id)
    }

    /// List all proposals in a federation
    pub fn list_proposals(&self, federation: &FederationId) -> Vec<&FederationProposal> {
        self.proposals
            .values()
            .filter(|p| p.federation == *federation)
            .collect()
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: ProposalStatus) -> Vec<&FederationProposal> {
        self.proposals
            .values()
            .filter(|p| p.status == status)
            .collect()
    }
}

/// Error types for federation governance
#[derive(Debug, thiserror::Error)]
pub enum FederationGovernanceError {
    #[error("Trust validation failed: {0}")]
    TrustValidationFailed(String),
    
    #[error("Federation membership required")]
    FederationMembershipRequired,
    
    #[error("Proposal not found: {0:?}")]
    ProposalNotFound(ProposalId),
    
    #[error("Proposal not open for voting: {0:?}")]
    ProposalNotOpen(ProposalId),
    
    #[error("Voting deadline passed: {0:?}")]
    VotingDeadlinePassed(ProposalId),
    
    #[error("Voting still open: {0:?}")]
    VotingStillOpen(ProposalId),
    
    #[error("Policy validation failed: {0}")]
    PolicyValidationFailed(String),
    
    #[error("Unauthorized action: {0}")]
    Unauthorized(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::{TrustPolicyRule, TrustLevel};
    use std::str::FromStr;

    fn setup_test_governance() -> FederationGovernanceEngine {
        let mut trust_engine = TrustPolicyEngine::new();
        
        // Add governance rule
        let rule = TrustPolicyRule {
            name: "governance_basic".to_string(),
            applicable_contexts: [TrustContext::Governance].into_iter().collect(),
            min_trust_level: TrustLevel::Basic,
            require_federation_membership: true,
            max_inheritance_depth: Some(2),
            allow_cross_federation: false,
            custom_validator: None,
        };
        trust_engine.add_rule(rule);

        let federation_id = Some(FederationId::new("test_federation".to_string()));
        let mut engine = FederationGovernanceEngine::new(trust_engine, federation_id);

        // Add voting policy
        let voting_policy = TrustAwareGovernancePolicy {
            action: GovernanceAction::Vote {
                proposal_id: ProposalId("dummy".to_string()),
                vote: true,
            },
            required_context: TrustContext::Governance,
            min_trust_level: TrustLevel::Basic,
            require_federation_membership: true,
            voting_threshold: 0.6,
            quorum_requirement: 0.3,
            allow_cross_federation: false,
        };
        engine.add_policy("vote".to_string(), voting_policy);

        engine
    }

    #[test]
    fn test_governance_action_key_generation() {
        let engine = setup_test_governance();
        
        let action = GovernanceAction::SubmitProposal {
            proposal_id: ProposalId("test".to_string()),
        };
        assert_eq!(engine.action_key(&action), "submit_proposal");
        
        let vote_action = GovernanceAction::Vote {
            proposal_id: ProposalId("test".to_string()),
            vote: true,
        };
        assert_eq!(engine.action_key(&vote_action), "vote");
    }

    #[test]
    fn test_proposal_status_lifecycle() {
        let engine = setup_test_governance();
        
        let proposal = FederationProposal {
            id: ProposalId("test_proposal".to_string()),
            proposer: Did::new("key", "alice"),
            federation: FederationId::new("test_federation".to_string()),
            trust_context: TrustContext::Governance,
            content: "Test proposal".to_string(),
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            created_at: chrono::Utc::now().timestamp() as u64,
            voting_deadline: (chrono::Utc::now().timestamp() + 3600) as u64,
        };
        
        assert_eq!(proposal.status, ProposalStatus::Open);
    }

    #[test]
    fn test_trust_aware_governance_policy() {
        let policy = TrustAwareGovernancePolicy {
            action: GovernanceAction::Vote {
                proposal_id: ProposalId("test".to_string()),
                vote: true,
            },
            required_context: TrustContext::Governance,
            min_trust_level: TrustLevel::Partial,
            require_federation_membership: true,
            voting_threshold: 0.67,
            quorum_requirement: 0.5,
            allow_cross_federation: false,
        };
        
        assert_eq!(policy.min_trust_level, TrustLevel::Partial);
        assert_eq!(policy.voting_threshold, 0.67);
        assert!(policy.require_federation_membership);
    }
}