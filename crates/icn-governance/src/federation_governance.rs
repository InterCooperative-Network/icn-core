//! Federation Trust Governance
//!
//! This module extends the governance system to work with the scoped federation trust framework.
//! It provides governance mechanisms that respect federation trust contexts and inheritance.

use crate::ProposalId;
use icn_common::{CommonError, Did};
use icn_identity::{
    FederationId, TrustContext, TrustLevel, TrustPolicyEngine, TrustValidationResult,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    ModifyMembership {
        target: Did,
        action: MembershipAction,
    },
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
    /// Trust committees
    trust_committees: HashMap<String, TrustCommittee>,
    /// Trust threshold policies
    threshold_policies: HashMap<String, TrustThresholdPolicy>,
    /// Trust violations
    violations: HashMap<String, TrustViolation>,
    /// Membership trust gates
    membership_gates: HashMap<FederationId, FederationMembershipTrustGate>,
    /// Active sanctions
    active_sanctions: HashMap<Did, Vec<TrustSanction>>,
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
            trust_committees: HashMap::new(),
            threshold_policies: HashMap::new(),
            violations: HashMap::new(),
            membership_gates: HashMap::new(),
            active_sanctions: HashMap::new(),
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
                return Err(GovernanceError::AdditionalValidationRequired(
                    additional_checks,
                ));
            }
        }

        // Create proposal
        let proposal_id = ProposalId(format!(
            "prop_{}_{}",
            federation.as_str(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));
        let proposal = FederationProposal {
            id: proposal_id.clone(),
            proposer: proposer.clone(),
            federation,
            trust_context,
            content,
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
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
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.clone()))?;

        if proposal.status != ProposalStatus::Open {
            return Err(GovernanceError::ProposalNotOpen(proposal_id.clone()));
        }

        if std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            > proposal.voting_deadline
        {
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
            TrustValidationResult::Allowed { .. } => {}
            TrustValidationResult::Denied { reason } => {
                return Err(GovernanceError::Unauthorized(format!(
                    "Vote denied: {}",
                    reason
                )));
            }
        }

        // Check if voter is federation member (if required)
        if let Some(policy) = self.get_voting_policy(&proposal.trust_context) {
            if policy.require_federation_membership
                && !self
                    .trust_engine
                    .is_federation_member(voter, &proposal.federation)
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
    pub fn finalize_proposal(
        &mut self,
        proposal_id: &ProposalId,
    ) -> Result<VotingResult, GovernanceError> {
        // First, extract needed data while we have the mutable borrow
        let (total_votes, yes_votes, trust_context, federation, _voting_deadline, _status) = {
            let proposal = self
                .proposals
                .get_mut(proposal_id)
                .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.clone()))?;

            if proposal.status != ProposalStatus::Open {
                return Err(GovernanceError::ProposalNotOpen(proposal_id.clone()));
            }

            // Check if voting deadline has passed
            if std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                <= proposal.voting_deadline
            {
                return Err(GovernanceError::VotingStillOpen(proposal_id.clone()));
            }

            // Calculate results
            let total_votes = proposal.votes.len();
            let yes_votes = proposal.votes.values().filter(|&&v| v).count();

            (
                total_votes,
                yes_votes,
                proposal.trust_context.clone(),
                proposal.federation.clone(),
                proposal.voting_deadline,
                proposal.status.clone(),
            )
        }; // Mutable borrow ends here

        let no_votes = total_votes - yes_votes;

        // Now we can safely call immutable methods
        let policy = self.get_voting_policy(&trust_context);
        let required_threshold = policy.map(|p| p.voting_threshold).unwrap_or(0.5);
        let required_quorum = policy.map(|p| p.quorum_requirement).unwrap_or(0.3);

        // Check quorum
        let federation_size = self.get_federation_size(&federation);
        let quorum_met = if federation_size > 0 {
            (total_votes as f64) / (federation_size as f64) >= required_quorum
        } else {
            total_votes > 0
        };

        if !quorum_met {
            // Update proposal status
            if let Some(proposal) = self.proposals.get_mut(proposal_id) {
                proposal.status = ProposalStatus::Failed;
            }
            return Ok(VotingResult::Failed(format!(
                "Quorum not met: {}/{} votes required",
                (federation_size as f64 * required_quorum) as usize,
                federation_size
            )));
        }

        // Check threshold
        let yes_ratio = if total_votes > 0 {
            (yes_votes as f64) / (total_votes as f64)
        } else {
            0.0
        };

        let result = if yes_ratio >= required_threshold {
            VotingResult::Passed {
                yes_votes,
                no_votes,
                total_eligible: federation_size,
            }
        } else {
            VotingResult::Failed(format!(
                "Threshold not met: {:.2}% yes votes, {:.2}% required",
                yes_ratio * 100.0,
                required_threshold * 100.0
            ))
        };

        // Update proposal status based on result
        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
            proposal.status = match &result {
                VotingResult::Passed { .. } => ProposalStatus::Passed,
                VotingResult::Failed(_) => ProposalStatus::Failed,
            };
        }

        Ok(result)
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
    fn get_federation_size(&self, _federation: &FederationId) -> usize {
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

    // === Trust Committee Management ===

    /// Create a new trust committee
    pub fn create_trust_committee(
        &mut self,
        committee_id: String,
        federation: FederationId,
        chair: Did,
        managed_contexts: HashSet<TrustContext>,
    ) -> Result<(), FederationGovernanceError> {
        if self.trust_committees.contains_key(&committee_id) {
            return Err(FederationGovernanceError::PolicyValidationFailed(format!(
                "Committee {} already exists",
                committee_id
            )));
        }

        let chair_member = TrustCommitteeMember {
            did: chair,
            role: TrustCommitteeRole::Chair,
            contexts: managed_contexts.clone(),
            voting_weight: 1.0,
            joined_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active: true,
        };

        let committee = TrustCommittee {
            id: committee_id.clone(),
            federation,
            members: [(chair_member.did.clone(), chair_member)]
                .into_iter()
                .collect(),
            managed_contexts,
            policies: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: CommitteeStatus::Active,
        };

        self.trust_committees.insert(committee_id, committee);
        Ok(())
    }

    /// Add member to trust committee
    pub fn add_committee_member(
        &mut self,
        committee_id: &str,
        member: TrustCommitteeMember,
    ) -> Result<(), FederationGovernanceError> {
        let committee = self.trust_committees.get_mut(committee_id).ok_or_else(|| {
            FederationGovernanceError::CommitteeNotFound(committee_id.to_string())
        })?;

        if committee.status != CommitteeStatus::Active {
            return Err(FederationGovernanceError::PolicyValidationFailed(
                "Cannot add members to inactive committee".to_string(),
            ));
        }

        committee.members.insert(member.did.clone(), member);
        Ok(())
    }

    /// Remove member from trust committee
    pub fn remove_committee_member(
        &mut self,
        committee_id: &str,
        member_did: &Did,
    ) -> Result<(), FederationGovernanceError> {
        let committee = self.trust_committees.get_mut(committee_id).ok_or_else(|| {
            FederationGovernanceError::CommitteeNotFound(committee_id.to_string())
        })?;

        // Cannot remove the last chair
        if let Some(member) = committee.members.get(member_did) {
            if member.role == TrustCommitteeRole::Chair {
                let chair_count = committee
                    .members
                    .values()
                    .filter(|m| m.role == TrustCommitteeRole::Chair)
                    .count();
                if chair_count <= 1 {
                    return Err(FederationGovernanceError::PolicyValidationFailed(
                        "Cannot remove the last committee chair".to_string(),
                    ));
                }
            }
        }

        committee.members.remove(member_did);
        Ok(())
    }

    /// Set trust threshold policy
    pub fn set_threshold_policy(&mut self, activity: String, policy: TrustThresholdPolicy) {
        self.threshold_policies.insert(activity, policy);
    }

    /// Validate trust threshold for an activity
    pub fn validate_trust_threshold(
        &self,
        actor: &Did,
        activity: &str,
        context: &TrustContext,
    ) -> Result<(), FederationGovernanceError> {
        if let Some(policy) = self.threshold_policies.get(activity) {
            // Check minimum trust level
            if let Some(_fed_id) = &self.federation_id {
                let validation = self.trust_engine.validate_trust(
                    actor, actor, // Self-validation for activity permission
                    context, activity,
                );

                match validation {
                    TrustValidationResult::Allowed {
                        effective_trust, ..
                    } => {
                        if !self.meets_minimum_trust(&effective_trust, &policy.min_trust_level) {
                            return Err(FederationGovernanceError::TrustThresholdNotMet(format!(
                                "Trust level {:?} does not meet minimum {:?} for activity '{}'",
                                effective_trust, policy.min_trust_level, activity
                            )));
                        }
                    }
                    TrustValidationResult::Denied { reason } => {
                        return Err(FederationGovernanceError::TrustValidationFailed(reason));
                    }
                }
            }
        }
        Ok(())
    }

    // === Trust Violations and Sanctions ===

    /// Report a trust violation
    pub fn report_violation(
        &mut self,
        violator: Did,
        violation_type: ViolationType,
        context: TrustContext,
        description: String,
        evidence: Vec<String>,
        reported_by: Did,
    ) -> Result<String, FederationGovernanceError> {
        let violation_id = format!(
            "violation_{}_{}",
            violator.to_string().chars().take(8).collect::<String>(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        let violation = TrustViolation {
            id: violation_id.clone(),
            violator,
            violation_type,
            federation: self.federation_id.clone().ok_or_else(|| {
                FederationGovernanceError::ViolationProcessingFailed(
                    "No federation context".to_string(),
                )
            })?,
            context,
            description,
            evidence,
            reported_by,
            reported_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: ViolationStatus::Reported,
            committee_decisions: Vec::new(),
            sanctions: Vec::new(),
        };

        self.violations.insert(violation_id.clone(), violation);
        Ok(violation_id)
    }

    /// Committee member votes on a violation
    pub fn committee_vote_on_violation(
        &mut self,
        violation_id: &str,
        committee_member: &Did,
        vote: DecisionVote,
        reasoning: String,
        recommended_sanctions: Vec<TrustSanction>,
    ) -> Result<(), FederationGovernanceError> {
        // First get the context while we have access
        let context = {
            let violation = self.violations.get(violation_id).ok_or_else(|| {
                FederationGovernanceError::ViolationProcessingFailed(format!(
                    "Violation {} not found",
                    violation_id
                ))
            })?;
            violation.context.clone()
        };

        // Verify member is on the appropriate committee
        let committee = self.get_committee_for_context(&context).ok_or_else(|| {
            FederationGovernanceError::CommitteeNotFound(format!(
                "No committee found for context {:?}",
                context
            ))
        })?;

        if !committee.members.contains_key(committee_member) {
            return Err(FederationGovernanceError::Unauthorized(
                "Not a committee member".to_string(),
            ));
        }

        let decision = CommitteeDecision {
            member: committee_member.clone(),
            vote,
            reasoning,
            decided_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            recommended_sanctions,
        };

        // Now update the violation with the decision
        let violation = self.violations.get_mut(violation_id).ok_or_else(|| {
            FederationGovernanceError::ViolationProcessingFailed(format!(
                "Violation {} not found",
                violation_id
            ))
        })?;

        violation.committee_decisions.push(decision);
        violation.status = ViolationStatus::UnderReview;

        Ok(())
    }

    /// Apply sanctions based on committee decision
    pub fn apply_sanctions(&mut self, violation_id: &str) -> Result<(), FederationGovernanceError> {
        // Extract needed data first
        let (context, violator, committee_decisions, total_votes, confirm_votes) = {
            let violation = self.violations.get(violation_id).ok_or_else(|| {
                FederationGovernanceError::ViolationProcessingFailed(format!(
                    "Violation {} not found",
                    violation_id
                ))
            })?;

            if violation.status != ViolationStatus::UnderReview {
                return Err(FederationGovernanceError::ViolationProcessingFailed(
                    "Violation not ready for sanction application".to_string(),
                ));
            }

            // Calculate committee consensus
            let total_votes = violation.committee_decisions.len();
            let confirm_votes = violation
                .committee_decisions
                .iter()
                .filter(|d| matches!(d.vote, DecisionVote::Confirm))
                .count();

            (
                violation.context.clone(),
                violation.violator.clone(),
                violation.committee_decisions.clone(),
                total_votes,
                confirm_votes,
            )
        };

        let _committee = self.get_committee_for_context(&context).ok_or_else(|| {
            FederationGovernanceError::CommitteeNotFound(format!(
                "No committee found for context {:?}",
                context
            ))
        })?;

        // Check if threshold is met (majority vote)
        let threshold = 0.5; // Could be configurable
        let (new_status, all_sanctions) =
            if (confirm_votes as f64) / (total_votes as f64) >= threshold {
                // Apply recommended sanctions
                let mut all_sanctions = Vec::new();
                for decision in &committee_decisions {
                    if matches!(decision.vote, DecisionVote::Confirm) {
                        all_sanctions.extend(decision.recommended_sanctions.clone());
                    }
                }

                // Add sanctions to active sanctions for the violator
                self.active_sanctions
                    .entry(violator.clone())
                    .or_default()
                    .extend(all_sanctions.clone());

                (ViolationStatus::Confirmed, all_sanctions)
            } else {
                (ViolationStatus::Dismissed, Vec::new())
            };

        // Update the violation with the results
        if let Some(violation) = self.violations.get_mut(violation_id) {
            violation.sanctions = all_sanctions;
            violation.status = new_status;
        }

        Ok(())
    }

    // === Federation Membership Trust Gates ===

    /// Set membership trust gate for federation
    pub fn set_membership_gate(
        &mut self,
        federation: FederationId,
        gate: FederationMembershipTrustGate,
    ) {
        self.membership_gates.insert(federation, gate);
    }

    /// Evaluate membership application
    pub fn evaluate_membership_application(
        &self,
        _applicant: &Did,
        federation: &FederationId,
        attestations: Vec<(&Did, TrustLevel, TrustContext)>,
    ) -> Result<MembershipApplicationResult, FederationGovernanceError> {
        let gate = self.membership_gates.get(federation).ok_or_else(|| {
            FederationGovernanceError::MembershipGateValidationFailed(format!(
                "No membership gate for federation {}",
                federation.as_str()
            ))
        })?;

        // Check minimum attestations
        if attestations.len() < gate.min_attestations {
            return Ok(MembershipApplicationResult::Rejected {
                reason: format!(
                    "Need {} attestations, got {}",
                    gate.min_attestations,
                    attestations.len()
                ),
                can_reapply_after: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + 2592000,
                ), // 30 days
            });
        }

        // Check trust levels and contexts
        let mut valid_contexts = HashSet::new();
        for (_attestor, trust_level, context) in &attestations {
            if gate.required_contexts.contains(context) {
                if self.meets_minimum_trust(trust_level, &gate.min_trust_level) {
                    valid_contexts.insert(context.clone());
                }
            }
        }

        if valid_contexts.len() < gate.required_contexts.len() {
            return Ok(MembershipApplicationResult::Rejected {
                reason: "Insufficient trust attestations for required contexts".to_string(),
                can_reapply_after: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + 2592000,
                ), // 30 days
            });
        }

        // If we have a committee, check approval
        if let Some(_committee) = self.get_committee_for_federation(federation) {
            // For now, automatically approve if basic requirements are met
            // In practice, this would trigger committee review process
            return Ok(MembershipApplicationResult::Approved {
                probationary_until: gate.probationary_period.map(|period| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + period
                }),
                granted_contexts: valid_contexts,
            });
        }

        Ok(MembershipApplicationResult::Approved {
            probationary_until: gate.probationary_period.map(|period| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    + period
            }),
            granted_contexts: valid_contexts,
        })
    }

    // === Helper Methods ===

    /// Check if trust level meets minimum requirement
    fn meets_minimum_trust(&self, actual: &TrustLevel, required: &TrustLevel) -> bool {
        use TrustLevel::*;
        match (actual, required) {
            (Full, _) => true,
            (Partial, Partial) | (Partial, Basic) | (Partial, None) => true,
            (Basic, Basic) | (Basic, None) => true,
            (None, None) => true,
            _ => false,
        }
    }

    /// Get committee responsible for a trust context
    fn get_committee_for_context(&self, context: &TrustContext) -> Option<&TrustCommittee> {
        self.trust_committees
            .values()
            .find(|c| c.managed_contexts.contains(context) && c.status == CommitteeStatus::Active)
    }

    /// Get committee for a federation
    fn get_committee_for_federation(&self, federation: &FederationId) -> Option<&TrustCommittee> {
        self.trust_committees
            .values()
            .find(|c| c.federation == *federation && c.status == CommitteeStatus::Active)
    }

    /// Check if member has active sanctions
    pub fn has_active_sanctions(&self, member: &Did) -> bool {
        self.active_sanctions
            .get(member)
            .map_or(false, |sanctions| !sanctions.is_empty())
    }

    /// Get violations for a member
    pub fn get_violations_for_member(&self, member: &Did) -> Vec<&TrustViolation> {
        self.violations
            .values()
            .filter(|v| v.violator == *member)
            .collect()
    }

    /// Get committee by ID
    pub fn get_committee(&self, committee_id: &str) -> Option<&TrustCommittee> {
        self.trust_committees.get(committee_id)
    }
}

/// Trust threshold policy for federation activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustThresholdPolicy {
    /// Activity name this policy applies to
    pub activity: String,
    /// Minimum trust level required
    pub min_trust_level: TrustLevel,
    /// Required trust context
    pub required_context: TrustContext,
    /// Minimum committee approval percentage (0.0-1.0)
    pub committee_threshold: f64,
    /// Minimum member votes required
    pub min_votes: usize,
    /// Whether unanimous committee approval is required
    pub require_unanimous: bool,
    /// Additional custom requirements
    pub custom_requirements: HashMap<String, String>,
}

/// Trust committee member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustCommitteeMember {
    /// Member DID
    pub did: Did,
    /// Role in the committee
    pub role: TrustCommitteeRole,
    /// Trust contexts this member can evaluate
    pub contexts: HashSet<TrustContext>,
    /// Voting weight (default 1.0)
    pub voting_weight: f64,
    /// When member was added to committee
    pub joined_at: u64,
    /// Whether member is currently active
    pub active: bool,
}

/// Roles within a trust committee
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustCommitteeRole {
    /// Committee chair with enhanced privileges
    Chair,
    /// Standard voting member
    Member,
    /// Advisory member with limited voting rights
    Advisor,
    /// Observer without voting rights
    Observer,
}

/// Trust committee governance structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustCommittee {
    /// Committee ID
    pub id: String,
    /// Federation this committee serves
    pub federation: FederationId,
    /// Committee members
    pub members: HashMap<Did, TrustCommitteeMember>,
    /// Trust contexts this committee handles
    pub managed_contexts: HashSet<TrustContext>,
    /// Committee policies
    pub policies: HashMap<String, TrustThresholdPolicy>,
    /// When committee was formed
    pub created_at: u64,
    /// Committee status
    pub status: CommitteeStatus,
}

/// Status of a trust committee
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommitteeStatus {
    /// Active and functioning
    Active,
    /// Temporarily suspended
    Suspended,
    /// Permanently dissolved
    Dissolved,
    /// Under review/restructuring
    UnderReview,
}

/// Trust sanction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustSanction {
    /// Warning issued to member
    Warning {
        reason: String,
        issued_at: u64,
        expires_at: Option<u64>,
    },
    /// Temporary trust level reduction
    TrustReduction {
        original_level: TrustLevel,
        reduced_level: TrustLevel,
        duration_seconds: u64,
        reason: String,
    },
    /// Temporary suspension from certain contexts
    ContextSuspension {
        suspended_contexts: HashSet<TrustContext>,
        duration_seconds: u64,
        reason: String,
    },
    /// Full federation suspension
    FederationSuspension {
        duration_seconds: u64,
        reason: String,
    },
    /// Permanent expulsion from federation
    Expulsion { reason: String, decided_at: u64 },
}

/// Trust violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustViolation {
    /// Unique violation ID
    pub id: String,
    /// DID of the violating member
    pub violator: Did,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Federation where violation occurred
    pub federation: FederationId,
    /// Trust context of the violation
    pub context: TrustContext,
    /// Detailed description
    pub description: String,
    /// Evidence supporting the violation
    pub evidence: Vec<String>,
    /// Reporter DID
    pub reported_by: Did,
    /// When violation was reported
    pub reported_at: u64,
    /// Current status
    pub status: ViolationStatus,
    /// Committee decisions on this violation
    pub committee_decisions: Vec<CommitteeDecision>,
    /// Applied sanctions
    pub sanctions: Vec<TrustSanction>,
}

/// Types of trust violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Malicious behavior
    Malicious,
    /// Resource abuse
    ResourceAbuse,
    /// Governance manipulation
    GovernanceManipulation,
    /// Identity misrepresentation
    IdentityMisrepresentation,
    /// Economic misconduct
    EconomicMisconduct,
    /// Data privacy violation
    DataPrivacyViolation,
    /// Infrastructure abuse
    InfrastructureAbuse,
    /// Custom violation type
    Custom(String),
}

/// Status of a trust violation case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationStatus {
    /// Just reported, under initial review
    Reported,
    /// Under investigation by committee
    UnderInvestigation,
    /// Investigation complete, pending decision
    UnderReview,
    /// Violation confirmed, sanctions applied
    Confirmed,
    /// Violation dismissed as unfounded
    Dismissed,
    /// Case closed with remediation
    Remediated,
}

/// Committee decision on a violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeDecision {
    /// Committee member making decision
    pub member: Did,
    /// Decision vote
    pub vote: DecisionVote,
    /// Reasoning for the decision
    pub reasoning: String,
    /// When decision was made
    pub decided_at: u64,
    /// Recommended sanctions
    pub recommended_sanctions: Vec<TrustSanction>,
}

/// Vote options for committee decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionVote {
    /// Confirm violation and apply sanctions
    Confirm,
    /// Dismiss as unfounded
    Dismiss,
    /// Recommend remediation without sanctions
    Remediate,
    /// Abstain from voting
    Abstain,
}

/// Federation membership trust gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMembershipTrustGate {
    /// Federation this gate applies to
    pub federation: FederationId,
    /// Minimum trust level from existing members
    pub min_trust_level: TrustLevel,
    /// Required trust contexts to be evaluated
    pub required_contexts: HashSet<TrustContext>,
    /// Minimum number of existing member attestations
    pub min_attestations: usize,
    /// Required committee approval percentage (0.0-1.0)
    pub committee_approval_threshold: f64,
    /// Probationary period for new members (seconds)
    pub probationary_period: Option<u64>,
    /// Additional requirements
    pub additional_requirements: HashMap<String, String>,
}

/// Result of membership application evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MembershipApplicationResult {
    /// Application approved
    Approved {
        probationary_until: Option<u64>,
        granted_contexts: HashSet<TrustContext>,
    },
    /// Application rejected
    Rejected {
        reason: String,
        can_reapply_after: Option<u64>,
    },
    /// Application pending additional review
    Pending {
        required_actions: Vec<String>,
        review_deadline: u64,
    },
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

    #[error("Committee not found: {0}")]
    CommitteeNotFound(String),

    #[error("Trust threshold not met: {0}")]
    TrustThresholdNotMet(String),

    #[error("Insufficient committee approval: {0}")]
    InsufficientCommitteeApproval(String),

    #[error("Trust violation processing failed: {0}")]
    ViolationProcessingFailed(String),

    #[error("Membership gate validation failed: {0}")]
    MembershipGateValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::{TrustLevel, TrustPolicyRule};
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
