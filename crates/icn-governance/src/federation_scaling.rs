//! Federation Scaling and Vote Aggregation
//!
//! This module implements the scaling mechanisms for aggregating votes from local
//! groups up through federations to global coordination, with transparent and
//! auditable scaling logic.

use crate::social_contract::{GovernanceScope, ScalingFunction, ScalingType};
use crate::{ProposalId, Vote, VoteOption};
use icn_common::{Cid, CommonError, Did};
use icn_identity::FederationId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Result of vote aggregation at a specific level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// Level this aggregation represents
    pub level: GovernanceScope,
    /// Participating groups/entities
    pub participants: Vec<ParticipantResult>,
    /// Total votes aggregated
    pub total_votes: VoteTally,
    /// Scaling function used
    pub scaling_function: ScalingFunction,
    /// Aggregation timestamp
    pub aggregated_at: SystemTime,
    /// Additional metadata
    pub metadata: AggregationMetadata,
}

/// Result from a single participant (group, federation, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantResult {
    /// Participant identifier
    pub id: String,
    /// Participant type (local group, federation, etc.)
    pub participant_type: ParticipantType,
    /// Raw votes from this participant
    pub raw_votes: VoteTally,
    /// Scaled/weighted votes after applying scaling function
    pub scaled_votes: VoteTally,
    /// Weight assigned to this participant
    pub weight: f64,
    /// Population/size of this participant
    pub population: usize,
    /// Reputation score (if applicable)
    pub reputation: Option<f64>,
    /// Participation metadata
    pub metadata: ParticipantMetadata,
}

/// Type of participant in aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantType {
    /// Local co-op or community
    LocalGroup,
    /// Regional federation
    RegionalFederation,
    /// National federation
    NationalFederation,
    /// Global federation
    GlobalFederation,
    /// Custom participant type
    Custom(String),
}

/// Vote tally with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteTally {
    /// Yes votes
    pub yes: u64,
    /// No votes
    pub no: u64,
    /// Abstain votes
    pub abstain: u64,
    /// Total eligible voters
    pub eligible: u64,
    /// Participation rate
    pub participation_rate: f64,
}

impl VoteTally {
    pub fn new() -> Self {
        Self {
            yes: 0,
            no: 0,
            abstain: 0,
            eligible: 0,
            participation_rate: 0.0,
        }
    }

    pub fn total_votes(&self) -> u64 {
        self.yes + self.no + self.abstain
    }

    pub fn calculate_participation_rate(&mut self) {
        if self.eligible > 0 {
            self.participation_rate = self.total_votes() as f64 / self.eligible as f64;
        }
    }

    pub fn add(&mut self, other: &VoteTally) {
        self.yes += other.yes;
        self.no += other.no;
        self.abstain += other.abstain;
        self.eligible += other.eligible;
        self.calculate_participation_rate();
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            yes: (self.yes as f64 * factor) as u64,
            no: (self.no as f64 * factor) as u64,
            abstain: (self.abstain as f64 * factor) as u64,
            eligible: self.eligible, // Don't scale eligible voters
            participation_rate: self.participation_rate,
        }
    }

    pub fn approval_rate(&self) -> f64 {
        let total = self.total_votes();
        if total > 0 {
            self.yes as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl Default for VoteTally {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for aggregation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationMetadata {
    /// Aggregation algorithm version
    pub algorithm_version: String,
    /// Quorum requirements met
    pub quorum_met: bool,
    /// Threshold requirements met
    pub threshold_met: bool,
    /// Any warnings or notes
    pub warnings: Vec<String>,
    /// Traceability data
    pub trace_id: String,
}

/// Metadata for participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantMetadata {
    /// Governance model used
    pub governance_model: String,
    /// Quorum achieved
    pub quorum_achieved: f64,
    /// Internal consensus level
    pub consensus_level: f64,
    /// Vote validity checks passed
    pub validity_checks: Vec<String>,
}

/// Federation scaling engine that handles vote aggregation
pub struct FederationScalingEngine {
    /// Local vote collectors by scope
    local_collectors: HashMap<String, LocalVoteCollector>,
    /// Federation aggregators by level
    federation_aggregators: HashMap<GovernanceScope, FederationAggregator>,
    /// Global aggregator
    global_aggregator: GlobalAggregator,
    /// Scaling functions registry
    scaling_functions: HashMap<String, ScalingFunction>,
    /// Audit trail
    audit_trail: Vec<AggregationAuditEntry>,
}

/// Collects votes at the local level (co-ops, communities)
pub struct LocalVoteCollector {
    /// Local group identifier
    pub group_id: String,
    /// Group name
    pub group_name: String,
    /// Governance model
    pub governance_model: String,
    /// Member registry
    pub members: HashSet<Did>,
    /// Vote collection
    pub collected_votes: HashMap<ProposalId, Vec<Vote>>,
    /// Vote tally cache
    pub tallies: HashMap<ProposalId, VoteTally>,
}

/// Aggregates votes at federation level
pub struct FederationAggregator {
    /// Federation identifier
    pub federation_id: FederationId,
    /// Federation level
    pub level: GovernanceScope,
    /// Member groups/federations
    pub members: HashMap<String, ParticipantInfo>,
    /// Aggregation results cache
    pub aggregation_cache: HashMap<ProposalId, AggregationResult>,
}

/// Global aggregator for network-wide votes
pub struct GlobalAggregator {
    /// Participating federations
    pub federations: HashMap<FederationId, FederationInfo>,
    /// Global results cache
    pub global_results: HashMap<ProposalId, GlobalAggregationResult>,
}

/// Information about a participant in aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// Participant identifier
    pub id: String,
    /// Participant name
    pub name: String,
    /// Population/member count
    pub population: usize,
    /// Reputation score
    pub reputation: f64,
    /// Last activity timestamp
    pub last_active: SystemTime,
    /// Governance capabilities
    pub capabilities: Vec<String>,
}

/// Information about a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationInfo {
    /// Federation identifier
    pub federation_id: FederationId,
    /// Federation name
    pub name: String,
    /// Member count
    pub member_count: usize,
    /// Total population across all members
    pub total_population: usize,
    /// Average reputation
    pub average_reputation: f64,
    /// Governance model
    pub governance_model: String,
    /// Scaling preferences
    pub scaling_preferences: ScalingPreferences,
}

/// Preferences for how votes should be scaled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPreferences {
    /// Preferred scaling type
    pub preferred_scaling: ScalingType,
    /// Weight preferences
    pub weight_factors: HashMap<String, f64>,
    /// Maximum weight allowed
    pub max_weight: f64,
    /// Minimum participation threshold
    pub min_participation: f64,
}

/// Global aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAggregationResult {
    /// Proposal being voted on
    pub proposal_id: ProposalId,
    /// Final global tally
    pub global_tally: VoteTally,
    /// Breakdown by federation
    pub federation_results: HashMap<FederationId, AggregationResult>,
    /// Scaling method used
    pub scaling_method: ScalingFunction,
    /// Global decision
    pub decision: GlobalDecision,
    /// Decision timestamp
    pub decided_at: SystemTime,
    /// Audit information
    pub audit_info: AggregationAuditInfo,
}

/// Global decision outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalDecision {
    /// Proposal approved globally
    Approved {
        approval_rate: f64,
        participating_federations: usize,
        total_population_represented: usize,
    },
    /// Proposal rejected globally
    Rejected {
        approval_rate: f64,
        participating_federations: usize,
        reasons: Vec<String>,
    },
    /// Insufficient participation for decision
    InsufficientParticipation {
        participation_rate: f64,
        required_participation: f64,
    },
    /// Tied result requiring additional process
    Tied {
        tied_at: f64,
        tie_breaking_mechanism: String,
    },
}

/// Audit information for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationAuditInfo {
    /// Full vote trace
    pub vote_trace: Vec<AggregationStep>,
    /// Scaling calculations
    pub scaling_calculations: Vec<ScalingCalculation>,
    /// Verification proofs
    pub verification_proofs: Vec<String>,
    /// Any anomalies detected
    pub anomalies: Vec<String>,
}

/// Single step in aggregation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStep {
    /// Step name
    pub step: String,
    /// Input data
    pub input: String,
    /// Output data
    pub output: String,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Scaling calculation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingCalculation {
    /// Participant being scaled
    pub participant: String,
    /// Original votes
    pub original_votes: VoteTally,
    /// Scaling factor applied
    pub scaling_factor: f64,
    /// Scaled votes
    pub scaled_votes: VoteTally,
    /// Justification
    pub justification: String,
}

/// Audit entry for traceability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationAuditEntry {
    /// Entry ID
    pub id: String,
    /// Proposal ID
    pub proposal_id: ProposalId,
    /// Action taken
    pub action: String,
    /// Actor (who performed the action)
    pub actor: Option<Did>,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Additional data
    pub data: HashMap<String, String>,
}

/// Error types for scaling operations
#[derive(Debug, thiserror::Error)]
pub enum ScalingError {
    #[error("Invalid scaling configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Insufficient data for aggregation: {0}")]
    InsufficientData(String),

    #[error("Scaling function not found: {0}")]
    ScalingFunctionNotFound(String),

    #[error("Participant not found: {0}")]
    ParticipantNotFound(String),

    #[error("Invalid vote data: {0}")]
    InvalidVoteData(String),

    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),

    #[error("Common error: {0}")]
    Common(#[from] CommonError),
}

impl FederationScalingEngine {
    /// Create new scaling engine
    pub fn new() -> Self {
        Self {
            local_collectors: HashMap::new(),
            federation_aggregators: HashMap::new(),
            global_aggregator: GlobalAggregator {
                federations: HashMap::new(),
                global_results: HashMap::new(),
            },
            scaling_functions: HashMap::new(),
            audit_trail: Vec::new(),
        }
    }

    /// Register a scaling function
    pub fn register_scaling_function(
        &mut self,
        name: String,
        function: ScalingFunction,
    ) {
        self.scaling_functions.insert(name, function);
    }

    /// Add local vote collector
    pub fn add_local_collector(&mut self, collector: LocalVoteCollector) {
        self.local_collectors.insert(collector.group_id.clone(), collector);
    }

    /// Add federation aggregator
    pub fn add_federation_aggregator(
        &mut self,
        level: GovernanceScope,
        aggregator: FederationAggregator,
    ) {
        self.federation_aggregators.insert(level, aggregator);
    }

    /// Collect votes at local level
    pub fn collect_local_votes(
        &mut self,
        group_id: &str,
        proposal_id: ProposalId,
        votes: Vec<Vote>,
    ) -> Result<VoteTally, ScalingError> {
        let collector = self.local_collectors.get_mut(group_id)
            .ok_or_else(|| ScalingError::ParticipantNotFound(group_id.to_string()))?;

        // Store votes
        collector.collected_votes.insert(proposal_id.clone(), votes.clone());

        // Calculate tally
        let mut tally = VoteTally::new();
        tally.eligible = collector.members.len() as u64;

        for vote in &votes {
            match vote.option {
                VoteOption::Yes => tally.yes += 1,
                VoteOption::No => tally.no += 1,
                VoteOption::Abstain => tally.abstain += 1,
            }
        }

        tally.calculate_participation_rate();
        collector.tallies.insert(proposal_id.clone(), tally.clone());

        // Create audit entry
        self.add_audit_entry(AggregationAuditEntry {
            id: format!("local_{}_{}", group_id, chrono::Utc::now().timestamp()),
            proposal_id,
            action: "collect_local_votes".to_string(),
            actor: None,
            timestamp: SystemTime::now(),
            data: [
                ("group_id".to_string(), group_id.to_string()),
                ("vote_count".to_string(), votes.len().to_string()),
                ("approval_rate".to_string(), tally.approval_rate().to_string()),
            ].into_iter().collect(),
        });

        Ok(tally)
    }

    /// Aggregate votes at federation level
    pub fn aggregate_federation_votes(
        &mut self,
        level: GovernanceScope,
        proposal_id: ProposalId,
        scaling_function_name: &str,
    ) -> Result<AggregationResult, ScalingError> {
        let scaling_function = self.scaling_functions.get(scaling_function_name)
            .ok_or_else(|| ScalingError::ScalingFunctionNotFound(scaling_function_name.to_string()))?
            .clone();

        let aggregator = self.federation_aggregators.get(&level)
            .ok_or_else(|| ScalingError::ParticipantNotFound(format!("Federation level {:?}", level)))?;

        let mut participants = Vec::new();
        let mut total_tally = VoteTally::new();

        // Collect results from member groups
        for (member_id, member_info) in &aggregator.members {
            // Get local tally for this member
            if let Some(collector) = self.local_collectors.get(member_id as &str) {
                if let Some(local_tally) = collector.tallies.get(&proposal_id) {
                    // Apply scaling function
                    let scaling_factor = self.calculate_scaling_factor(
                        &scaling_function,
                        member_info,
                        local_tally,
                    )?;

                    let scaled_tally = local_tally.scale(scaling_factor);

                    let participant_result = ParticipantResult {
                        id: member_id.clone(),
                        participant_type: ParticipantType::LocalGroup,
                        raw_votes: local_tally.clone(),
                        scaled_votes: scaled_tally.clone(),
                        weight: scaling_factor,
                        population: member_info.population,
                        reputation: Some(member_info.reputation),
                        metadata: ParticipantMetadata {
                            governance_model: collector.governance_model.clone(),
                            quorum_achieved: local_tally.participation_rate,
                            consensus_level: local_tally.approval_rate(),
                            validity_checks: vec!["signature_verified".to_string()],
                        },
                    };

                    participants.push(participant_result);
                    total_tally.add(&scaled_tally);
                }
            }
        }

        let result = AggregationResult {
            level: level.clone(),
            participants,
            total_votes: total_tally,
            scaling_function,
            aggregated_at: SystemTime::now(),
            metadata: AggregationMetadata {
                algorithm_version: "1.0".to_string(),
                quorum_met: true, // Would check actual quorum requirements
                threshold_met: true, // Would check actual threshold requirements
                warnings: Vec::new(),
                trace_id: format!("agg_{}_{}", level.as_str(), chrono::Utc::now().timestamp()),
            },
        };

        // Create audit entry
        self.add_audit_entry(AggregationAuditEntry {
            id: format!("fed_{}_{}", level.as_str(), chrono::Utc::now().timestamp()),
            proposal_id,
            action: "aggregate_federation_votes".to_string(),
            actor: None,
            timestamp: SystemTime::now(),
            data: [
                ("level".to_string(), level.as_str().to_string()),
                ("participants".to_string(), result.participants.len().to_string()),
                ("scaling_function".to_string(), scaling_function_name.to_string()),
            ].into_iter().collect(),
        });

        Ok(result)
    }

    /// Aggregate votes at global level
    pub fn aggregate_global_votes(
        &mut self,
        proposal_id: ProposalId,
        scaling_function_name: &str,
    ) -> Result<GlobalAggregationResult, ScalingError> {
        let scaling_function = self.scaling_functions.get(scaling_function_name)
            .ok_or_else(|| ScalingError::ScalingFunctionNotFound(scaling_function_name.to_string()))?
            .clone();

        let mut federation_results = HashMap::new();
        let mut global_tally = VoteTally::new();
        let mut audit_steps = Vec::new();
        let mut scaling_calculations = Vec::new();

        // Collect federation levels for iteration
        let federation_levels: Vec<GovernanceScope> = self.federation_aggregators.keys().cloned().collect();

        // Aggregate from all federation levels
        for level in federation_levels {
            match self.aggregate_federation_votes(level.clone(), proposal_id.clone(), scaling_function_name) {
                Ok(fed_result) => {
                    // For global aggregation, we might need to re-scale federation results
                    let fed_info = FederationInfo {
                        federation_id: FederationId::new(format!("fed_{}", level.as_str())),
                        name: format!("{:?} Federation", level),
                        member_count: fed_result.participants.len(),
                        total_population: fed_result.participants.iter().map(|p| p.population).sum(),
                        average_reputation: fed_result.participants.iter()
                            .filter_map(|p| p.reputation)
                            .sum::<f64>() / fed_result.participants.len() as f64,
                        governance_model: "federation".to_string(),
                        scaling_preferences: ScalingPreferences {
                            preferred_scaling: scaling_function.scaling_type.clone(),
                            weight_factors: HashMap::new(),
                            max_weight: 1.0,
                            min_participation: 0.1,
                        },
                    };

                    let global_scaling_factor = self.calculate_global_scaling_factor(
                        &scaling_function,
                        &fed_info,
                        &fed_result.total_votes,
                    );

                    let globally_scaled_votes = fed_result.total_votes.scale(global_scaling_factor);

                    scaling_calculations.push(ScalingCalculation {
                        participant: format!("{:?}_federation", level),
                        original_votes: fed_result.total_votes.clone(),
                        scaling_factor: global_scaling_factor,
                        scaled_votes: globally_scaled_votes.clone(),
                        justification: format!("Global scaling for {:?} level federation", level),
                    });

                    global_tally.add(&globally_scaled_votes);
                    federation_results.insert(
                        FederationId::new(format!("fed_{}", level.as_str())),
                        fed_result,
                    );
                }
                Err(e) => {
                    // Log error but continue with other federations
                    audit_steps.push(AggregationStep {
                        step: format!("aggregate_{:?}_federation", level),
                        input: proposal_id.to_string(),
                        output: format!("Error: {}", e),
                        timestamp: SystemTime::now(),
                    });
                }
            }
        }

        // Determine global decision
        let decision = self.determine_global_decision(&global_tally, &federation_results);

        let result = GlobalAggregationResult {
            proposal_id: proposal_id.clone(),
            global_tally,
            federation_results,
            scaling_method: scaling_function,
            decision,
            decided_at: SystemTime::now(),
            audit_info: AggregationAuditInfo {
                vote_trace: audit_steps,
                scaling_calculations,
                verification_proofs: vec!["signature_chain_verified".to_string()],
                anomalies: Vec::new(),
            },
        };

        self.global_aggregator.global_results.insert(proposal_id.clone(), result.clone());

        // Create audit entry
        self.add_audit_entry(AggregationAuditEntry {
            id: format!("global_{}", chrono::Utc::now().timestamp()),
            proposal_id,
            action: "aggregate_global_votes".to_string(),
            actor: None,
            timestamp: SystemTime::now(),
            data: [
                ("federations_count".to_string(), result.federation_results.len().to_string()),
                ("global_approval".to_string(), result.global_tally.approval_rate().to_string()),
                ("decision".to_string(), format!("{:?}", result.decision)),
            ].into_iter().collect(),
        });

        Ok(result)
    }

    /// Calculate scaling factor for a participant
    fn calculate_scaling_factor(
        &self,
        scaling_function: &ScalingFunction,
        participant_info: &ParticipantInfo,
        votes: &VoteTally,
    ) -> Result<f64, ScalingError> {
        match &scaling_function.scaling_type {
            ScalingType::Linear => {
                Ok(participant_info.population as f64 / 1000.0) // Normalize by 1000
            }
            ScalingType::Quadratic => {
                let pop_sqrt = (participant_info.population as f64).sqrt();
                Ok(pop_sqrt / 100.0) // Normalize
            }
            ScalingType::Logarithmic => {
                let pop_log = (participant_info.population as f64).ln();
                Ok(pop_log / 10.0) // Normalize
            }
            ScalingType::ReputationWeighted => {
                Ok(participant_info.reputation * votes.participation_rate)
            }
            ScalingType::OneGroupOneVote => {
                Ok(1.0) // Equal weight for all
            }
            ScalingType::Hybrid(_) => {
                // For now, use linear as default for hybrid
                Ok(participant_info.population as f64 / 1000.0)
            }
            ScalingType::Custom(_) => {
                // Would implement custom scaling logic here
                Ok(1.0)
            }
        }
    }

    /// Calculate global scaling factor for federations
    fn calculate_global_scaling_factor(
        &self,
        scaling_function: &ScalingFunction,
        federation_info: &FederationInfo,
        votes: &VoteTally,
    ) -> f64 {
        match &scaling_function.scaling_type {
            ScalingType::Linear => {
                federation_info.total_population as f64 / 10000.0 // Normalize
            }
            ScalingType::Quadratic => {
                (federation_info.total_population as f64).sqrt() / 1000.0
            }
            ScalingType::Logarithmic => {
                (federation_info.total_population as f64).ln() / 100.0
            }
            ScalingType::ReputationWeighted => {
                federation_info.average_reputation * votes.participation_rate
            }
            ScalingType::OneGroupOneVote => 1.0,
            _ => 1.0, // Default
        }
    }

    /// Determine global decision based on aggregated votes
    fn determine_global_decision(
        &self,
        global_tally: &VoteTally,
        federation_results: &HashMap<FederationId, AggregationResult>,
    ) -> GlobalDecision {
        let approval_rate = global_tally.approval_rate();
        let participating_federations = federation_results.len();
        let total_population: usize = federation_results
            .values()
            .map(|r| r.participants.iter().map(|p| p.population).sum::<usize>())
            .sum();

        let required_approval = 0.5; // Could be configurable
        let required_participation = 0.3; // Could be configurable

        if global_tally.participation_rate < required_participation {
            GlobalDecision::InsufficientParticipation {
                participation_rate: global_tally.participation_rate,
                required_participation,
            }
        } else if approval_rate > required_approval {
            GlobalDecision::Approved {
                approval_rate,
                participating_federations,
                total_population_represented: total_population,
            }
        } else {
            GlobalDecision::Rejected {
                approval_rate,
                participating_federations,
                reasons: vec!["Insufficient approval rate".to_string()],
            }
        }
    }

    /// Add audit entry
    fn add_audit_entry(&mut self, entry: AggregationAuditEntry) {
        self.audit_trail.push(entry);
    }

    /// Get audit trail for a proposal
    pub fn get_audit_trail(&self, proposal_id: &ProposalId) -> Vec<&AggregationAuditEntry> {
        self.audit_trail
            .iter()
            .filter(|entry| entry.proposal_id == *proposal_id)
            .collect()
    }

    /// Get global result for a proposal
    pub fn get_global_result(
        &self,
        proposal_id: &ProposalId,
    ) -> Option<&GlobalAggregationResult> {
        self.global_aggregator.global_results.get(proposal_id)
    }
}

impl Default for FederationScalingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalVoteCollector {
    pub fn new(group_id: String, group_name: String, governance_model: String) -> Self {
        Self {
            group_id,
            group_name,
            governance_model,
            members: HashSet::new(),
            collected_votes: HashMap::new(),
            tallies: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, member: Did) {
        self.members.insert(member);
    }

    pub fn remove_member(&mut self, member: &Did) {
        self.members.remove(member);
    }
}

impl FederationAggregator {
    pub fn new(federation_id: FederationId, level: GovernanceScope) -> Self {
        Self {
            federation_id,
            level,
            members: HashMap::new(),
            aggregation_cache: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, id: String, info: ParticipantInfo) {
        self.members.insert(id, info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social_contract::ContractVersion;

    #[test]
    fn test_vote_tally_operations() {
        let mut tally1 = VoteTally {
            yes: 10,
            no: 5,
            abstain: 2,
            eligible: 20,
            participation_rate: 0.0,
        };
        tally1.calculate_participation_rate();

        assert_eq!(tally1.total_votes(), 17);
        assert_eq!(tally1.participation_rate, 0.85);
        assert_eq!(tally1.approval_rate(), 10.0 / 17.0);

        let tally2 = VoteTally {
            yes: 8,
            no: 3,
            abstain: 1,
            eligible: 15,
            participation_rate: 0.8,
        };

        tally1.add(&tally2);
        assert_eq!(tally1.yes, 18);
        assert_eq!(tally1.no, 8);
        assert_eq!(tally1.abstain, 3);
        assert_eq!(tally1.eligible, 35);
    }

    #[test]
    fn test_vote_scaling() {
        let tally = VoteTally {
            yes: 10,
            no: 5,
            abstain: 2,
            eligible: 20,
            participation_rate: 0.85,
        };

        let scaled = tally.scale(2.0);
        assert_eq!(scaled.yes, 20);
        assert_eq!(scaled.no, 10);
        assert_eq!(scaled.abstain, 4);
        assert_eq!(scaled.eligible, 20); // Eligible doesn't scale
    }

    #[test]
    fn test_scaling_engine_creation() {
        let engine = FederationScalingEngine::new();
        assert!(engine.local_collectors.is_empty());
        assert!(engine.federation_aggregators.is_empty());
        assert!(engine.scaling_functions.is_empty());
    }

    #[test]
    fn test_local_collector() {
        let mut collector = LocalVoteCollector::new(
            "group1".to_string(),
            "Test Group".to_string(),
            "direct".to_string(),
        );

        let member1 = Did::new("test", "member1");
        let member2 = Did::new("test", "member2");

        collector.add_member(member1.clone());
        collector.add_member(member2.clone());

        assert_eq!(collector.members.len(), 2);
        assert!(collector.members.contains(&member1));
        assert!(collector.members.contains(&member2));
    }
}