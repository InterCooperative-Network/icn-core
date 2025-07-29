//! Governance-Level Conflict Resolution
//!
//! This module implements conflict resolution for governance-level disputes such as:
//! - Proposal clashes (conflicting proposals for the same resource/action)
//! - Voting disputes (irregular voting patterns, disputed vote counts)
//! - Policy contradictions (new proposals that conflict with existing policies)
//! - Escalation mechanisms for unresolved governance issues

use crate::{GovernanceModule, Proposal, ProposalId, ProposalStatus, ProposalType, Vote};
use icn_common::{CommonError, Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of governance conflicts that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceConflictType {
    /// Multiple proposals targeting the same resource or action
    ProposalClash,
    /// Disputed vote counts or irregular voting patterns
    VotingDispute,
    /// New proposal conflicts with existing active policy
    PolicyContradiction,
    /// Quorum manipulation or voting irregularities
    QuorumDispute,
    /// Procedural violations in proposal submission or voting
    ProceduralViolation,
}

/// A specific governance conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConflict {
    /// Unique identifier for this conflict
    pub conflict_id: String,
    /// Type of governance conflict
    pub conflict_type: GovernanceConflictType,
    /// Proposals involved in the conflict
    pub involved_proposals: Vec<ProposalId>,
    /// Affected members/voters
    pub affected_members: HashSet<Did>,
    /// Timestamp when conflict was detected
    pub detected_at: u64,
    /// Current resolution status
    pub resolution_status: GovernanceResolutionStatus,
    /// Evidence supporting the conflict claim
    pub evidence: Vec<ConflictEvidence>,
    /// Description of the conflict
    pub description: String,
    /// Severity level of the conflict
    pub severity: ConflictSeverity,
}

/// Current status of governance conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceResolutionStatus {
    /// Conflict detected but not yet addressed
    Detected,
    /// Under investigation by governance authorities
    UnderInvestigation,
    /// Escalated to higher governance level
    Escalated,
    /// Mediation in progress
    Mediation,
    /// Voting on resolution in progress
    ResolutionVoting {
        resolution_proposal_id: ProposalId,
        deadline: u64,
    },
    /// Conflict resolved
    Resolved {
        resolution: GovernanceResolution,
        applied_at: u64,
    },
    /// Resolution failed or rejected
    Failed { reason: String },
}

/// Evidence supporting a conflict claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictEvidence {
    /// Multiple proposals with conflicting actions
    ConflictingProposals { proposals: Vec<ProposalId> },
    /// Irregular voting pattern detected
    IrregularVotingPattern {
        voter: Did,
        pattern_description: String,
    },
    /// Policy contradiction detected
    PolicyContradiction {
        new_proposal: ProposalId,
        conflicting_policy: String,
    },
    /// Quorum manipulation evidence
    QuorumManipulation {
        suspicious_votes: Vec<Vote>,
        manipulation_type: String,
    },
    /// Procedural violation evidence
    ProceduralViolation {
        violation_type: String,
        details: String,
    },
}

/// Severity levels for governance conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConflictSeverity {
    /// Low severity - minor procedural issues
    Low,
    /// Medium severity - significant governance disruption
    Medium,
    /// High severity - major governance failure
    High,
    /// Critical severity - governance system integrity at risk
    Critical,
}

/// Resolution actions for governance conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceResolution {
    /// Suspend conflicting proposals
    SuspendProposals { proposals: Vec<ProposalId> },
    /// Invalidate disputed votes
    InvalidateVotes { votes: Vec<Vote> },
    /// Implement policy override
    PolicyOverride {
        suspended_policies: Vec<String>,
        new_policy: String,
    },
    /// Implement procedural correction
    ProceduralCorrection {
        correction_type: String,
        affected_proposals: Vec<ProposalId>,
    },
    /// Escalate to manual intervention
    EscalateToManual { reason: String },
    /// No action required (false positive)
    NoActionRequired,
}

/// Configuration for governance conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConflictConfig {
    /// Enable automatic conflict detection
    pub auto_detection: bool,
    /// Minimum severity level for automatic resolution
    pub auto_resolution_threshold: ConflictSeverity,
    /// Maximum time for conflict investigation (seconds)
    pub investigation_timeout: u64,
    /// Escalation threshold (number of unresolved conflicts)
    pub escalation_threshold: usize,
    /// Enable policy contradiction checking
    pub policy_contradiction_checking: bool,
    /// Voting pattern anomaly detection sensitivity (0.0-1.0)
    pub anomaly_detection_sensitivity: f64,
}

impl Default for GovernanceConflictConfig {
    fn default() -> Self {
        Self {
            auto_detection: true,
            auto_resolution_threshold: ConflictSeverity::Medium,
            investigation_timeout: 86400, // 24 hours
            escalation_threshold: 5,
            policy_contradiction_checking: true,
            anomaly_detection_sensitivity: 0.8,
        }
    }
}

/// Manages governance-level conflict detection and resolution
pub struct GovernanceConflictResolver {
    config: GovernanceConflictConfig,
    active_conflicts: HashMap<String, GovernanceConflict>,
    resolution_history: Vec<GovernanceConflict>,
    policy_registry: HashMap<String, String>, // policy_id -> policy_content
    governance_authorities: HashSet<Did>,
}

impl GovernanceConflictResolver {
    /// Create a new governance conflict resolver
    pub fn new(config: GovernanceConflictConfig) -> Self {
        Self {
            config,
            active_conflicts: HashMap::new(),
            resolution_history: Vec::new(),
            policy_registry: HashMap::new(),
            governance_authorities: HashSet::new(),
        }
    }

    /// Add a governance authority who can resolve conflicts
    pub fn add_governance_authority(&mut self, authority: Did) {
        self.governance_authorities.insert(authority);
    }

    /// Remove a governance authority
    pub fn remove_governance_authority(&mut self, authority: &Did) {
        self.governance_authorities.remove(authority);
    }

    /// Register a policy in the registry for conflict checking
    pub fn register_policy(&mut self, policy_id: String, policy_content: String) {
        self.policy_registry.insert(policy_id, policy_content);
    }

    /// Detect governance conflicts in the provided governance module
    pub fn detect_conflicts(
        &mut self,
        governance: &GovernanceModule,
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<GovernanceConflict>, CommonError> {
        if !self.config.auto_detection {
            return Ok(Vec::new());
        }

        let mut new_conflicts = Vec::new();

        // Get all proposals from governance module
        let proposals = governance.list_proposals()?;

        // Check for proposal clashes
        new_conflicts.extend(self.detect_proposal_clashes(&proposals, time_provider)?);

        // Check for voting disputes
        new_conflicts.extend(self.detect_voting_disputes(&proposals, time_provider)?);

        // Check for policy contradictions
        if self.config.policy_contradiction_checking {
            new_conflicts.extend(self.detect_policy_contradictions(&proposals, time_provider)?);
        }

        // Check for procedural violations
        new_conflicts.extend(self.detect_procedural_violations(&proposals, time_provider)?);

        // Add new conflicts to active tracking
        for conflict in &new_conflicts {
            self.active_conflicts
                .insert(conflict.conflict_id.clone(), conflict.clone());
        }

        Ok(new_conflicts)
    }

    /// Detect proposal clashes (multiple proposals targeting same resource)
    fn detect_proposal_clashes(
        &self,
        proposals: &[Proposal],
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<GovernanceConflict>, CommonError> {
        let mut conflicts = Vec::new();
        let active_proposals: Vec<&Proposal> = proposals
            .iter()
            .filter(|p| {
                matches!(
                    p.status,
                    ProposalStatus::VotingOpen | ProposalStatus::Deliberation
                )
            })
            .collect();

        // Group proposals by their target (simplified - in real implementation would be more sophisticated)
        let mut target_groups: HashMap<String, Vec<&Proposal>> = HashMap::new();

        for proposal in &active_proposals {
            let target = self.extract_proposal_target(proposal);
            target_groups.entry(target).or_default().push(proposal);
        }

        // Check for conflicts (multiple proposals targeting same resource)
        for (target, proposals_for_target) in target_groups {
            if proposals_for_target.len() > 1 {
                let conflict_id = format!(
                    "clash_{}_{}",
                    target,
                    time_provider.unix_seconds()
                );

                let involved_proposals: Vec<ProposalId> =
                    proposals_for_target.iter().map(|p| p.id.clone()).collect();

                let affected_members: HashSet<Did> = proposals_for_target
                    .iter()
                    .flat_map(|p| p.votes.keys().cloned())
                    .collect();

                conflicts.push(GovernanceConflict {
                    conflict_id,
                    conflict_type: GovernanceConflictType::ProposalClash,
                    involved_proposals,
                    affected_members,
                    detected_at: time_provider.unix_seconds(),
                    resolution_status: GovernanceResolutionStatus::Detected,
                    evidence: vec![ConflictEvidence::ConflictingProposals {
                        proposals: proposals_for_target.iter().map(|p| p.id.clone()).collect(),
                    }],
                    description: format!("Multiple active proposals targeting: {}", target),
                    severity: ConflictSeverity::Medium,
                });
            }
        }

        Ok(conflicts)
    }

    /// Extract the target/subject of a proposal for conflict detection
    fn extract_proposal_target(&self, proposal: &Proposal) -> String {
        match &proposal.proposal_type {
            ProposalType::SystemParameterChange(param, _) => format!("system_param:{}", param),
            ProposalType::NewMemberInvitation(did) => format!("member_invite:{}", did),
            ProposalType::RemoveMember(did) => format!("member_remove:{}", did),
            ProposalType::SoftwareUpgrade(_) => "software_upgrade".to_string(),
            ProposalType::BudgetAllocation(recipient, _, _) => format!("budget:{}", recipient),
            ProposalType::Resolution(_) => "resolution".to_string(),
            ProposalType::GenericText(_) => "generic".to_string(),
        }
    }

    /// Detect voting disputes and irregularities
    fn detect_voting_disputes(
        &self,
        proposals: &[Proposal],
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<GovernanceConflict>, CommonError> {
        let mut conflicts = Vec::new();

        for proposal in proposals {
            // Check for unusual voting patterns
            if let Some(anomaly) = self.detect_voting_anomaly(proposal) {
                let conflict_id = format!(
                    "vote_dispute_{}_{}",
                    proposal.id.0,
                    time_provider.unix_seconds()
                );

                conflicts.push(GovernanceConflict {
                    conflict_id,
                    conflict_type: GovernanceConflictType::VotingDispute,
                    involved_proposals: vec![proposal.id.clone()],
                    affected_members: proposal.votes.keys().cloned().collect(),
                    detected_at: time_provider.unix_seconds(),
                    resolution_status: GovernanceResolutionStatus::Detected,
                    evidence: vec![anomaly],
                    description: format!("Voting anomaly detected in proposal {}", proposal.id.0),
                    severity: ConflictSeverity::High,
                });
            }
        }

        Ok(conflicts)
    }

    /// Detect voting anomalies in a proposal
    fn detect_voting_anomaly(&self, proposal: &Proposal) -> Option<ConflictEvidence> {
        // Check for suspicious voting patterns
        let votes: Vec<&Vote> = proposal.votes.values().collect();

        if votes.len() < 3 {
            return None; // Need minimum votes to detect patterns
        }

        // Check for rapid sequential voting (possible coordination)
        let mut sorted_votes = votes.clone();
        sorted_votes.sort_by_key(|v| v.voted_at);

        let mut rapid_sequence_count = 0;
        for window in sorted_votes.windows(2) {
            if window[1].voted_at - window[0].voted_at < 60 {
                // Less than 60 seconds apart
                rapid_sequence_count += 1;
            }
        }

        // If more than 50% of votes are in rapid sequence, flag as suspicious
        if rapid_sequence_count as f64 / votes.len() as f64 > 0.5 {
            let suspicious_voters: HashSet<Did> = sorted_votes
                .windows(2)
                .filter(|window| window[1].voted_at - window[0].voted_at < 60)
                .map(|window| window[0].voter.clone())
                .collect();

            return Some(ConflictEvidence::IrregularVotingPattern {
                voter: suspicious_voters.into_iter().next().unwrap_or_default(),
                pattern_description: "Rapid sequential voting detected - possible coordination"
                    .to_string(),
            });
        }

        None
    }

    /// Detect policy contradictions
    fn detect_policy_contradictions(
        &self,
        proposals: &[Proposal],
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<GovernanceConflict>, CommonError> {
        let mut conflicts = Vec::new();

        for proposal in proposals {
            if proposal.status == ProposalStatus::VotingOpen {
                // Check if this proposal contradicts existing policies
                if let Some(contradiction) = self.check_policy_contradiction(proposal) {
                    let conflict_id = format!(
                        "policy_contradiction_{}_{}",
                        proposal.id.0,
                        time_provider.unix_seconds()
                    );

                    conflicts.push(GovernanceConflict {
                        conflict_id,
                        conflict_type: GovernanceConflictType::PolicyContradiction,
                        involved_proposals: vec![proposal.id.clone()],
                        affected_members: HashSet::new(),
                        detected_at: time_provider.unix_seconds(),
                        resolution_status: GovernanceResolutionStatus::Detected,
                        evidence: vec![contradiction],
                        description: format!("Policy contradiction in proposal {}", proposal.id.0),
                        severity: ConflictSeverity::Medium,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Check if a proposal contradicts existing policies
    fn check_policy_contradiction(&self, proposal: &Proposal) -> Option<ConflictEvidence> {
        // Simplified policy contradiction checking
        // In real implementation, this would involve sophisticated policy analysis

        match &proposal.proposal_type {
            ProposalType::SystemParameterChange(param, _new_value) => {
                // Check if this parameter change contradicts existing policies
                for (policy_id, policy_content) in &self.policy_registry {
                    if policy_content.contains(param)
                        && policy_content.contains("shall not be changed")
                    {
                        return Some(ConflictEvidence::PolicyContradiction {
                            new_proposal: proposal.id.clone(),
                            conflicting_policy: policy_id.clone(),
                        });
                    }
                }
            }
            ProposalType::RemoveMember(did) => {
                // Check if this member has special protection
                for (policy_id, policy_content) in &self.policy_registry {
                    if policy_content.contains(&did.to_string())
                        && policy_content.contains("protected")
                    {
                        return Some(ConflictEvidence::PolicyContradiction {
                            new_proposal: proposal.id.clone(),
                            conflicting_policy: policy_id.clone(),
                        });
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Detect procedural violations
    fn detect_procedural_violations(
        &self,
        proposals: &[Proposal],
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<GovernanceConflict>, CommonError> {
        let mut conflicts = Vec::new();

        for proposal in proposals {
            // Check for procedural violations (simplified)
            if let Some(violation) = self.check_procedural_violation(proposal) {
                let conflict_id = format!(
                    "procedural_{}_{}",
                    proposal.id.0,
                    time_provider.unix_seconds()
                );

                conflicts.push(GovernanceConflict {
                    conflict_id,
                    conflict_type: GovernanceConflictType::ProceduralViolation,
                    involved_proposals: vec![proposal.id.clone()],
                    affected_members: HashSet::new(),
                    detected_at: time_provider.unix_seconds(),
                    resolution_status: GovernanceResolutionStatus::Detected,
                    evidence: vec![violation],
                    description: format!("Procedural violation in proposal {}", proposal.id.0),
                    severity: ConflictSeverity::Low,
                });
            }
        }

        Ok(conflicts)
    }

    /// Check for procedural violations in a proposal
    fn check_procedural_violation(&self, proposal: &Proposal) -> Option<ConflictEvidence> {
        // Check for basic procedural requirements

        // Check minimum description length
        if proposal.description.len() < 10 {
            return Some(ConflictEvidence::ProceduralViolation {
                violation_type: "insufficient_description".to_string(),
                details: "Proposal description too short".to_string(),
            });
        }

        // Check voting period duration
        let voting_duration = proposal.voting_deadline - proposal.created_at;
        if voting_duration < 3600 {
            // Less than 1 hour
            return Some(ConflictEvidence::ProceduralViolation {
                violation_type: "insufficient_voting_period".to_string(),
                details: "Voting period too short".to_string(),
            });
        }

        None
    }

    /// Resolve a governance conflict
    pub fn resolve_conflict(
        &mut self,
        conflict_id: &str,
        resolver: &Did,
        time_provider: &dyn TimeProvider,
    ) -> Result<GovernanceResolutionStatus, CommonError> {
        // Verify resolver is authorized
        if !self.governance_authorities.contains(resolver) {
            return Err(CommonError::PolicyDenied(
                "Not authorized to resolve governance conflicts".to_string(),
            ));
        }

        let conflict = self.active_conflicts.get_mut(conflict_id).ok_or_else(|| {
            CommonError::ResourceNotFound(format!("Conflict {} not found", conflict_id))
        })?;

        // Update status based on conflict type and severity
        let new_status = match (&conflict.conflict_type, &conflict.severity) {
            (_, ConflictSeverity::Critical) => GovernanceResolutionStatus::Escalated,
            (GovernanceConflictType::ProposalClash, _) => {
                // Suspend conflicting proposals pending resolution
                GovernanceResolutionStatus::UnderInvestigation
            }
            (GovernanceConflictType::VotingDispute, _) => {
                // Require investigation and potential re-vote
                GovernanceResolutionStatus::UnderInvestigation
            }
            (GovernanceConflictType::PolicyContradiction, _) => {
                // Extract actual policy information from conflict evidence
                let (suspended_policy, new_policy) = match conflict.evidence.first() {
                    Some(ConflictEvidence::PolicyContradiction {
                        conflicting_policy, ..
                    }) => (
                        conflicting_policy.clone(),
                        format!("Override for {}", conflicting_policy),
                    ),
                    _ => ("unknown_policy".to_string(), "default_override".to_string()),
                };

                GovernanceResolutionStatus::Resolved {
                    resolution: GovernanceResolution::PolicyOverride {
                        suspended_policies: vec![suspended_policy],
                        new_policy,
                    },
                    applied_at: time_provider.unix_seconds(),
                }
            }
            (GovernanceConflictType::ProceduralViolation, ConflictSeverity::Low) => {
                // Auto-resolve minor procedural issues
                GovernanceResolutionStatus::Resolved {
                    resolution: GovernanceResolution::ProceduralCorrection {
                        correction_type: "minor_procedural_fix".to_string(),
                        affected_proposals: conflict.involved_proposals.clone(),
                    },
                    applied_at: time_provider.unix_seconds(),
                }
            }
            _ => GovernanceResolutionStatus::UnderInvestigation,
        };

        conflict.resolution_status = new_status.clone();

        // If resolved, move to history
        if matches!(new_status, GovernanceResolutionStatus::Resolved { .. }) {
            let resolved_conflict = conflict.clone();
            self.resolution_history.push(resolved_conflict);
            self.active_conflicts.remove(conflict_id);
        }

        Ok(new_status)
    }

    /// Get all active conflicts
    pub fn get_active_conflicts(&self) -> &HashMap<String, GovernanceConflict> {
        &self.active_conflicts
    }

    /// Get conflict resolution history
    pub fn get_resolution_history(&self) -> &Vec<GovernanceConflict> {
        &self.resolution_history
    }

    /// Check if escalation is needed based on unresolved conflicts
    pub fn check_escalation_needed(&self) -> bool {
        let unresolved_count = self
            .active_conflicts
            .values()
            .filter(|c| {
                matches!(
                    c.resolution_status,
                    GovernanceResolutionStatus::Detected
                        | GovernanceResolutionStatus::UnderInvestigation
                )
            })
            .count();

        unresolved_count >= self.config.escalation_threshold
    }

    /// Process periodic maintenance tasks
    pub fn process_periodic_tasks(
        &mut self,
        time_provider: &dyn TimeProvider,
    ) -> Result<Vec<String>, CommonError> {
        let mut timed_out_conflicts = Vec::new();
        let current_time = time_provider.unix_seconds();

        // Check for investigation timeouts
        let conflict_ids: Vec<String> = self.active_conflicts.keys().cloned().collect();

        for conflict_id in conflict_ids {
            if let Some(conflict) = self.active_conflicts.get_mut(&conflict_id) {
                if matches!(
                    conflict.resolution_status,
                    GovernanceResolutionStatus::UnderInvestigation
                ) {
                    let investigation_duration = current_time - conflict.detected_at;
                    if investigation_duration > self.config.investigation_timeout {
                        // Escalate timed-out investigations
                        conflict.resolution_status = GovernanceResolutionStatus::Escalated;
                        timed_out_conflicts.push(conflict_id);
                    }
                }
            }
        }

        Ok(timed_out_conflicts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GovernanceModule, ProposalSubmission, ProposalType};
    use std::str::FromStr;

    #[test]
    fn test_governance_conflict_resolver_creation() {
        let config = GovernanceConflictConfig::default();
        let resolver = GovernanceConflictResolver::new(config);

        assert_eq!(resolver.active_conflicts.len(), 0);
        assert_eq!(resolver.resolution_history.len(), 0);
    }

    #[test]
    fn test_proposal_clash_detection() {
        let mut resolver = GovernanceConflictResolver::new(GovernanceConflictConfig::default());
        let mut governance = GovernanceModule::new();

        // Add some test members
        let alice = Did::from_str("did:example:alice").unwrap();
        let bob = Did::from_str("did:example:bob").unwrap();
        governance.add_member(alice.clone());
        governance.add_member(bob.clone());

        // Create conflicting proposals
        let proposal1 = ProposalSubmission {
            proposer: alice.clone(),
            proposal_type: ProposalType::SystemParameterChange(
                "max_users".to_string(),
                "100".to_string(),
            ),
            description: "Increase max users to 100".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal2 = ProposalSubmission {
            proposer: bob.clone(),
            proposal_type: ProposalType::SystemParameterChange(
                "max_users".to_string(),
                "50".to_string(),
            ),
            description: "Decrease max users to 50".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let id1 = governance.submit_proposal(proposal1).unwrap();
        let id2 = governance.submit_proposal(proposal2).unwrap();

        governance.open_voting(&id1).unwrap();
        governance.open_voting(&id2).unwrap();

        // Detect conflicts
        let conflicts = resolver.detect_conflicts(&governance).unwrap();

        assert_eq!(conflicts.len(), 1);
        assert_eq!(
            conflicts[0].conflict_type,
            GovernanceConflictType::ProposalClash
        );
        assert_eq!(conflicts[0].involved_proposals.len(), 2);
    }

    #[test]
    fn test_policy_contradiction_detection() {
        let mut resolver = GovernanceConflictResolver::new(GovernanceConflictConfig::default());

        // Register a policy that protects certain parameters
        resolver.register_policy(
            "param_protection".to_string(),
            "The max_users parameter shall not be changed without board approval".to_string(),
        );

        let mut governance = GovernanceModule::new();
        let alice = Did::from_str("did:example:alice").unwrap();
        governance.add_member(alice.clone());

        // Create a proposal that contradicts the policy
        let proposal = ProposalSubmission {
            proposer: alice.clone(),
            proposal_type: ProposalType::SystemParameterChange(
                "max_users".to_string(),
                "200".to_string(),
            ),
            description: "Change max users".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let id = governance.submit_proposal(proposal).unwrap();
        governance.open_voting(&id).unwrap();

        // Detect conflicts
        let conflicts = resolver.detect_conflicts(&governance).unwrap();

        assert_eq!(conflicts.len(), 1);
        assert_eq!(
            conflicts[0].conflict_type,
            GovernanceConflictType::PolicyContradiction
        );
    }

    #[test]
    fn test_procedural_violation_detection() {
        let mut resolver = GovernanceConflictResolver::new(GovernanceConflictConfig::default());
        let mut governance = GovernanceModule::new();

        let alice = Did::from_str("did:example:alice").unwrap();
        governance.add_member(alice.clone());

        // Create a proposal with procedural violations (short description, short voting period)
        let proposal = ProposalSubmission {
            proposer: alice.clone(),
            proposal_type: ProposalType::GenericText("test".to_string()),
            description: "short".to_string(), // Too short
            duration_secs: 60,                // Too short (less than 1 hour)
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let id = governance.submit_proposal(proposal).unwrap();
        governance.open_voting(&id).unwrap();

        // Detect conflicts
        let conflicts = resolver.detect_conflicts(&governance).unwrap();

        assert!(!conflicts.is_empty());
        assert!(conflicts
            .iter()
            .any(|c| c.conflict_type == GovernanceConflictType::ProceduralViolation));
    }

    #[test]
    fn test_conflict_resolution() {
        let mut resolver = GovernanceConflictResolver::new(GovernanceConflictConfig::default());
        let authority = Did::from_str("did:example:authority").unwrap();
        resolver.add_governance_authority(authority.clone());

        // Create a mock conflict
        let conflict = GovernanceConflict {
            conflict_id: "test_conflict".to_string(),
            conflict_type: GovernanceConflictType::ProceduralViolation,
            involved_proposals: vec![],
            affected_members: HashSet::new(),
            detected_at: 1000,
            resolution_status: GovernanceResolutionStatus::Detected,
            evidence: vec![],
            description: "Test conflict".to_string(),
            severity: ConflictSeverity::Low,
        };

        resolver
            .active_conflicts
            .insert("test_conflict".to_string(), conflict);

        // Resolve the conflict
        let result = resolver
            .resolve_conflict("test_conflict", &authority)
            .unwrap();

        assert!(matches!(
            result,
            GovernanceResolutionStatus::Resolved { .. }
        ));
        assert_eq!(resolver.active_conflicts.len(), 0);
        assert_eq!(resolver.resolution_history.len(), 1);
    }
}
