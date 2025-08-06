//! Cooperation Enhancement Module
//!
//! This module implements trust-weighted resource allocation, mutual aid coordination,
//! collective bonus mechanisms, and democratic resource allocation protocols.

use crate::ManaLedger;
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Trust-weighted resource allocation system
#[derive(Debug, Clone)]
pub struct TrustWeightedAllocator {
    trust_scores: HashMap<(Did, Did), f64>, // (evaluator, target) -> trust score
    trust_decay_rate: f64,
    trust_update_threshold: f64,
    minimum_trust_for_allocation: f64,
    trust_network_effects: TrustNetworkEffects,
}

/// Network effects in trust calculations
#[derive(Debug, Clone)]
pub struct TrustNetworkEffects {
    transitive_trust_weight: f64,    // Weight for transitive trust (A trusts B, B trusts C)
    reputation_amplification: f64,   // Amplification based on reputation
    community_consensus_weight: f64, // Weight for community-wide consensus
    reciprocity_bonus: f64,         // Bonus for mutual trust relationships
}

/// Trust relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRelationship {
    pub trustor: Did,
    pub trustee: Did,
    pub direct_trust: f64,      // Direct trust score (0.0 to 1.0)
    pub transitive_trust: f64,  // Trust derived from network effects
    pub experience_count: u32,  // Number of interactions
    pub last_updated: u64,
    pub trust_decay: f64,       // Current decay factor
    pub relationship_type: TrustRelationshipType,
}

/// Types of trust relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustRelationshipType {
    Personal,        // Direct personal interaction
    Professional,    // Work/economic relationship  
    Community,       // Community-based trust
    Institutional,   // Organizational trust
    Delegated,       // Trust delegated by others
}

/// Mutual aid coordination system
#[derive(Debug, Clone)]
pub struct MutualAidCoordinator {
    aid_requests: HashMap<String, AidRequest>,
    aid_offers: HashMap<String, AidOffer>,
    aid_networks: HashMap<String, AidNetwork>,
    matching_algorithm: MatchingAlgorithm,
    emergency_protocols: EmergencyAidProtocols,
}

/// Request for mutual aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidRequest {
    pub request_id: String,
    pub requester: Did,
    pub aid_type: AidType,
    pub urgency_level: UrgencyLevel,
    pub resource_amount: u64,
    pub description: String,
    pub geographical_scope: GeographicalScope,
    pub time_sensitivity: u64, // Time by which aid is needed
    pub reciprocity_commitment: ReciprocityCommitment,
    pub verification_status: VerificationStatus,
    pub created_at: u64,
}

/// Offer to provide mutual aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidOffer {
    pub offer_id: String,
    pub provider: Did,
    pub aid_type: AidType,
    pub available_amount: u64,
    pub conditions: Vec<AidCondition>,
    pub geographical_reach: GeographicalScope,
    pub availability_window: (u64, u64), // (start_time, end_time)
    pub preferred_recipients: Vec<Did>,
    pub reciprocity_expectations: ReciprocityExpectation,
    pub created_at: u64,
}

/// Types of aid that can be requested/offered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AidType {
    Emergency { emergency_type: String },
    BasicNeeds { need_type: String },
    Skills { skill_categories: Vec<String> },
    Resources { resource_types: Vec<String> },
    Care { care_type: String },
    Knowledge { knowledge_areas: Vec<String> },
    Infrastructure { infrastructure_type: String },
    Emotional { support_type: String },
}

/// Urgency levels for aid requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum UrgencyLevel {
    Low,      // Can wait weeks/months
    Medium,   // Needed within days/weeks  
    High,     // Needed within hours/days
    Critical, // Immediate need
}

/// Geographical scope for aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeographicalScope {
    Local { radius_km: f64 },
    Regional { region_name: String },
    National { country: String },
    Global,
    Virtual, // No geographical constraints
}

/// Reciprocity commitments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReciprocityCommitment {
    Direct { commitment_description: String },
    Indirect { time_window_days: u32 },
    NetworkBased { network_id: String },
    PayItForward,
    NoReciprocityExpected,
}

/// Reciprocity expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReciprocityExpectation {
    Immediate { expected_return: String },
    Eventual { time_window_days: u32 },
    NetworkContribution,
    None,
}

/// Conditions for aid provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AidCondition {
    TrustScore { minimum_score: f64 },
    CommunityMembership { community_id: String },
    PreviousContribution { minimum_contribution: u64 },
    SkillLevel { required_skills: Vec<String> },
    GeographicalProximity { max_distance_km: f64 },
    Verification { verification_type: String },
}

/// Status of aid request/offer verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    CommunityVerified { verifiers: Vec<Did> },
    OrganizationVerified { organization: Did },
    SelfReported,
    Disputed { dispute_details: String },
}

/// Aid network for coordinated mutual aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidNetwork {
    pub network_id: String,
    pub network_name: String,
    pub members: HashSet<Did>,
    pub network_type: AidNetworkType,
    pub coordination_mechanisms: Vec<CoordinationMechanism>,
    pub resource_pools: HashMap<String, u64>,
    pub solidarity_principles: Vec<String>,
    pub decision_making_process: DecisionMakingProcess,
}

/// Types of aid networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AidNetworkType {
    Community { focus_area: String },
    Professional { profession: String },
    Geographical { region: String },
    Thematic { theme: String },
    Emergency { response_type: String },
    Affinity { shared_values: Vec<String> },
}

/// Mechanisms for coordinating aid within networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMechanism {
    CentralCoordinator { coordinator: Did },
    RotatingCoordination { rotation_period_days: u32 },
    DecentralizedMatching,
    AlgorithmicMatching { algorithm_type: String },
    ConsensusCoordination,
}

/// Decision making processes for aid networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionMakingProcess {
    Consensus,
    Majority,
    Delegated { delegates: Vec<Did> },
    Autonomous,
    Hybrid { process_description: String },
}

/// Matching algorithms for aid requests and offers
#[derive(Debug, Clone)]
pub enum MatchingAlgorithm {
    TrustWeighted,      // Match based on trust scores
    NeedBased,          // Prioritize urgent needs
    ProximityBased,     // Prioritize geographical proximity  
    CapacityBased,      // Match based on provider capacity
    NetworkBased,       // Prioritize network members
    Hybrid { weights: HashMap<String, f64> },
}

/// Emergency aid protocols
#[derive(Debug, Clone)]
pub struct EmergencyAidProtocols {
    crisis_detection_criteria: Vec<String>,
    automatic_aid_triggers: Vec<AutomaticTrigger>,
    emergency_resource_reserves: HashMap<String, u64>,
    priority_escalation_rules: Vec<EscalationRule>,
}

/// Automatic triggers for emergency aid
#[derive(Debug, Clone)]
pub struct AutomaticTrigger {
    trigger_type: String,
    threshold_conditions: Vec<String>,
    automatic_actions: Vec<String>,
    notification_targets: Vec<Did>,
}

/// Rules for escalating aid priority during emergencies
#[derive(Debug, Clone)]
pub struct EscalationRule {
    condition: String,
    priority_boost: f64,
    resource_allocation_override: bool,
    notification_requirement: bool,
}

/// Collective bonus system
#[derive(Debug, Clone)]
pub struct CollectiveBonusSystem {
    active_bonus_programs: HashMap<String, BonusProgram>,
    participation_tracking: HashMap<Did, ParticipationRecord>,
    bonus_calculation_rules: BonusCalculationRules,
    distribution_mechanisms: Vec<DistributionMechanism>,
}

/// Bonus program definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusProgram {
    pub program_id: String,
    pub program_name: String,
    pub program_type: BonusProgramType,
    pub eligibility_criteria: Vec<EligibilityCriterion>,
    pub bonus_calculation: BonusCalculation,
    pub duration: ProgramDuration,
    pub max_participants: Option<u32>,
    pub total_bonus_pool: u64,
    pub status: ProgramStatus,
}

/// Types of bonus programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusProgramType {
    CollectiveGoal { goal_description: String, target_metric: String },
    CooperationIncentive { cooperation_type: String },
    MutualAidBonus { aid_category: String },
    InnovationReward { innovation_areas: Vec<String> },
    SustainabilityBonus { sustainability_metrics: Vec<String> },
    CommunityBuilding { community_activities: Vec<String> },
    CrisisResponse { response_type: String },
}

/// Eligibility criteria for bonus programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EligibilityCriterion {
    MinimumParticipation { participation_type: String, minimum_level: f64 },
    TrustThreshold { minimum_trust_score: f64 },
    ContributionHistory { minimum_contributions: u64, time_period_days: u32 },
    CommunityMembership { required_communities: Vec<String> },
    SkillRequirement { required_skills: Vec<String> },
    GeographicalRequirement { geographical_scope: GeographicalScope },
}

/// Bonus calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusCalculation {
    EqualShare,         // Equal bonus for all participants
    ProportionalContribution { contribution_metric: String },
    TierBased { tiers: Vec<BonusTier> },
    PerformanceBased { performance_metrics: Vec<String> },
    Hybrid { calculation_components: Vec<CalculationComponent> },
}

/// Bonus tiers for tier-based calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusTier {
    pub tier_name: String,
    pub threshold: f64,
    pub bonus_multiplier: f64,
}

/// Components for hybrid bonus calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationComponent {
    pub component_name: String,
    pub weight: f64,
    pub calculation_method: String,
}

/// Program duration specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgramDuration {
    Fixed { duration_days: u32 },
    Ongoing,
    GoalBased { completion_criteria: Vec<String> },
    SeasonalRecurring { season_pattern: String },
}

/// Status of bonus programs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgramStatus {
    Planning,
    Active,
    Paused,
    Completed,
    Cancelled,
}

/// Individual participation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationRecord {
    pub participant: Did,
    pub program_participations: HashMap<String, ProgramParticipation>,
    pub overall_cooperation_score: f64,
    pub bonus_earnings_history: Vec<BonusEarning>,
    pub reputation_bonuses: f64,
}

/// Participation in a specific program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramParticipation {
    pub program_id: String,
    pub joined_at: u64,
    pub contribution_metrics: HashMap<String, f64>,
    pub milestone_achievements: Vec<String>,
    pub collaboration_partners: HashSet<Did>,
    pub impact_measurement: f64,
}

/// Record of bonus earnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusEarning {
    pub program_id: String,
    pub earning_period: (u64, u64),
    pub bonus_amount: u64,
    pub earning_reason: String,
    pub awarded_at: u64,
}

/// Rules for calculating collective bonuses
#[derive(Debug, Clone)]
pub struct BonusCalculationRules {
    cooperation_multipliers: HashMap<String, f64>,
    achievement_bonuses: HashMap<String, f64>,
    network_effect_bonuses: HashMap<String, f64>,
    solidarity_multipliers: HashMap<String, f64>,
}

/// Mechanisms for distributing bonuses
#[derive(Debug, Clone)]
pub enum DistributionMechanism {
    DirectDistribution,                    // Direct mana/token transfer
    StakedRewards { staking_period: u64 }, // Bonuses staked for future release
    InvestmentOptions { investment_types: Vec<String> }, // Bonus invested in community projects
    ChoiceBasedDistribution { options: Vec<String> },   // Participants choose distribution method
}

impl TrustWeightedAllocator {
    /// Create a new trust-weighted allocator
    pub fn new() -> Self {
        Self {
            trust_scores: HashMap::new(),
            trust_decay_rate: 0.01, // 1% decay per time unit
            trust_update_threshold: 0.1,
            minimum_trust_for_allocation: 0.3,
            trust_network_effects: TrustNetworkEffects {
                transitive_trust_weight: 0.3,
                reputation_amplification: 0.2,
                community_consensus_weight: 0.4,
                reciprocity_bonus: 0.1,
            },
        }
    }

    /// Update trust score between two entities
    pub fn update_trust_score(
        &mut self,
        trustor: &Did,
        trustee: &Did,
        new_score: f64,
    ) -> Result<(), CommonError> {
        if new_score < 0.0 || new_score > 1.0 {
            return Err(CommonError::InvalidInputError(
                "Trust score must be between 0.0 and 1.0".into()
            ));
        }

        self.trust_scores.insert((trustor.clone(), trustee.clone()), new_score);
        Ok(())
    }

    /// Get trust score between two entities
    pub fn get_trust_score(&self, trustor: &Did, trustee: &Did) -> f64 {
        self.trust_scores.get(&(trustor.clone(), trustee.clone()))
            .copied()
            .unwrap_or(0.0)
    }

    /// Calculate transitive trust (A trusts B, B trusts C, what's A's trust in C?)
    pub fn calculate_transitive_trust(
        &self,
        trustor: &Did,
        trustee: &Did,
        max_hops: u32,
    ) -> f64 {
        if max_hops == 0 {
            return self.get_trust_score(trustor, trustee);
        }

        let mut max_transitive_trust: f64 = 0.0;
        
        // Find intermediate nodes
        for ((intermediate_trustor, intermediate_trustee), _) in &self.trust_scores {
            if intermediate_trustor == trustor {
                let trust_to_intermediate = self.trust_scores[&(trustor.clone(), intermediate_trustee.clone())];
                let transitive_from_intermediate = self.calculate_transitive_trust(
                    intermediate_trustee,
                    trustee,
                    max_hops - 1
                );
                
                let combined_trust = trust_to_intermediate * transitive_from_intermediate * 
                    self.trust_network_effects.transitive_trust_weight;
                
                max_transitive_trust = max_transitive_trust.max(combined_trust);
            }
        }

        max_transitive_trust
    }

    /// Allocate resources based on trust scores
    pub fn trust_weighted_allocation<L: ManaLedger>(
        &self,
        allocator: &Did,
        total_amount: u64,
        candidates: &[Did],
        ledger: &mut L,
    ) -> Result<HashMap<Did, u64>, CommonError> {
        let mut allocations = HashMap::new();
        let mut total_trust_weight = 0.0;

        // Calculate trust weights for each candidate
        let mut trust_weights = HashMap::new();
        for candidate in candidates {
            let trust_score = self.get_trust_score(allocator, candidate);
            
            if trust_score >= self.minimum_trust_for_allocation {
                trust_weights.insert(candidate.clone(), trust_score);
                total_trust_weight += trust_score;
            }
        }

        // Distribute based on trust weights
        if total_trust_weight > 0.0 {
            for (candidate, trust_weight) in trust_weights {
                let allocation = ((trust_weight / total_trust_weight) * total_amount as f64) as u64;
                if allocation > 0 {
                    ledger.credit(&candidate, allocation)?;
                    allocations.insert(candidate, allocation);
                }
            }
        }

        Ok(allocations)
    }

    /// Apply trust decay over time
    pub fn apply_trust_decay(&mut self, time_elapsed: u64) {
        let decay_factor = 1.0 - (self.trust_decay_rate * time_elapsed as f64);
        
        for trust_score in self.trust_scores.values_mut() {
            *trust_score *= decay_factor;
            *trust_score = trust_score.max(0.0); // Ensure non-negative
        }
    }

    /// Get most trusted entities for a given trustor
    pub fn get_most_trusted(&self, trustor: &Did, limit: usize) -> Vec<(Did, f64)> {
        let mut trusted_entities: Vec<(Did, f64)> = self.trust_scores
            .iter()
            .filter_map(|((t_or, t_ee), score)| {
                if t_or == trustor {
                    Some((t_ee.clone(), *score))
                } else {
                    None
                }
            })
            .collect();

        trusted_entities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        trusted_entities.truncate(limit);
        trusted_entities
    }
}

impl MutualAidCoordinator {
    /// Create a new mutual aid coordinator
    pub fn new() -> Self {
        Self {
            aid_requests: HashMap::new(),
            aid_offers: HashMap::new(),
            aid_networks: HashMap::new(),
            matching_algorithm: MatchingAlgorithm::TrustWeighted,
            emergency_protocols: EmergencyAidProtocols {
                crisis_detection_criteria: Vec::new(),
                automatic_aid_triggers: Vec::new(),
                emergency_resource_reserves: HashMap::new(),
                priority_escalation_rules: Vec::new(),
            },
        }
    }

    /// Submit a request for mutual aid
    pub fn submit_aid_request(&mut self, request: AidRequest) -> Result<(), CommonError> {
        // Validate request
        if request.resource_amount == 0 {
            return Err(CommonError::InvalidInputError(
                "Resource amount must be greater than zero".into()
            ));
        }

        self.aid_requests.insert(request.request_id.clone(), request);
        Ok(())
    }

    /// Submit an offer for mutual aid
    pub fn submit_aid_offer(&mut self, offer: AidOffer) -> Result<(), CommonError> {
        // Validate offer
        if offer.available_amount == 0 {
            return Err(CommonError::InvalidInputError(
                "Available amount must be greater than zero".into()
            ));
        }

        self.aid_offers.insert(offer.offer_id.clone(), offer);
        Ok(())
    }

    /// Match aid requests with offers
    pub fn match_aid_requests_and_offers(&self) -> Vec<AidMatch> {
        let mut matches = Vec::new();
        
        for (request_id, request) in &self.aid_requests {
            for (offer_id, offer) in &self.aid_offers {
                if self.is_compatible_match(request, offer) {
                    let match_quality = self.calculate_match_quality(request, offer);
                    
                    matches.push(AidMatch {
                        request_id: request_id.clone(),
                        offer_id: offer_id.clone(),
                        match_quality,
                        recommended_amount: request.resource_amount.min(offer.available_amount),
                        match_type: self.determine_match_type(request, offer),
                    });
                }
            }
        }

        // Sort by match quality
        matches.sort_by(|a, b| b.match_quality.partial_cmp(&a.match_quality).unwrap());
        matches
    }

    /// Check if a request and offer are compatible
    fn is_compatible_match(&self, request: &AidRequest, offer: &AidOffer) -> bool {
        // Check aid type compatibility
        if !self.aid_types_compatible(&request.aid_type, &offer.aid_type) {
            return false;
        }

        // Check geographical compatibility
        if !self.geographical_scopes_compatible(&request.geographical_scope, &offer.geographical_reach) {
            return false;
        }

        // Check time compatibility
        let current_time = SystemTimeProvider.unix_seconds();
        if current_time < offer.availability_window.0 || current_time > offer.availability_window.1 {
            return false;
        }

        // Check conditions
        for condition in &offer.conditions {
            if !self.condition_satisfied(condition, &request.requester) {
                return false;
            }
        }

        true
    }

    /// Check if aid types are compatible
    fn aid_types_compatible(&self, request_type: &AidType, offer_type: &AidType) -> bool {
        match (request_type, offer_type) {
            (AidType::Emergency { emergency_type: req }, AidType::Emergency { emergency_type: off }) => req == off,
            (AidType::BasicNeeds { need_type: req }, AidType::BasicNeeds { need_type: off }) => req == off,
            (AidType::Resources { resource_types: req }, AidType::Resources { resource_types: off }) => {
                req.iter().any(|r| off.contains(r))
            }
            _ => false, // Simplified compatibility check
        }
    }

    /// Check if geographical scopes are compatible
    fn geographical_scopes_compatible(&self, request_scope: &GeographicalScope, offer_reach: &GeographicalScope) -> bool {
        match (request_scope, offer_reach) {
            (GeographicalScope::Global, _) => true,
            (_, GeographicalScope::Global) => true,
            (GeographicalScope::Virtual, GeographicalScope::Virtual) => true,
            (GeographicalScope::Local { radius_km: req_radius }, GeographicalScope::Local { radius_km: off_radius }) => {
                // Simplified distance check - in practice would need actual coordinates
                req_radius <= off_radius
            }
            _ => false, // Simplified check
        }
    }

    /// Check if aid condition is satisfied
    fn condition_satisfied(&self, _condition: &AidCondition, _requester: &Did) -> bool {
        // Placeholder implementation - would check actual conditions
        true
    }

    /// Calculate match quality between request and offer
    fn calculate_match_quality(&self, _request: &AidRequest, _offer: &AidOffer) -> f64 {
        // Simplified quality calculation
        // In practice, would consider multiple factors
        0.8
    }

    /// Determine type of match
    fn determine_match_type(&self, _request: &AidRequest, _offer: &AidOffer) -> AidMatchType {
        AidMatchType::DirectMatch
    }

    /// Execute an aid match
    pub fn execute_aid_match<L: ManaLedger>(
        &mut self,
        aid_match: &AidMatch,
        ledger: &mut L,
    ) -> Result<AidTransaction, CommonError> {
        let request = self.aid_requests.get(&aid_match.request_id)
            .ok_or_else(|| CommonError::InvalidInputError("Request not found".into()))?;
        
        let offer = self.aid_offers.get(&aid_match.offer_id)
            .ok_or_else(|| CommonError::InvalidInputError("Offer not found".into()))?;

        // Transfer resources
        ledger.spend(&offer.provider, aid_match.recommended_amount)?;
        ledger.credit(&request.requester, aid_match.recommended_amount)?;

        // Create transaction record
        let transaction = AidTransaction {
            transaction_id: format!("aid_{}_{}", aid_match.request_id, aid_match.offer_id),
            request_id: aid_match.request_id.clone(),
            offer_id: aid_match.offer_id.clone(),
            provider: offer.provider.clone(),
            recipient: request.requester.clone(),
            amount: aid_match.recommended_amount,
            aid_type: request.aid_type.clone(),
            executed_at: SystemTimeProvider.unix_seconds(),
            status: AidTransactionStatus::Completed,
        };

        Ok(transaction)
    }
}

/// Match between aid request and offer
#[derive(Debug, Clone)]
pub struct AidMatch {
    pub request_id: String,
    pub offer_id: String,
    pub match_quality: f64,
    pub recommended_amount: u64,
    pub match_type: AidMatchType,
}

/// Types of aid matches
#[derive(Debug, Clone)]
pub enum AidMatchType {
    DirectMatch,     // Direct 1:1 match
    PartialMatch,    // Partially fulfills need
    NetworkMatch,    // Coordinated through network
    EmergencyMatch,  // Emergency priority match
}

/// Transaction record for executed aid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidTransaction {
    pub transaction_id: String,
    pub request_id: String,
    pub offer_id: String,
    pub provider: Did,
    pub recipient: Did,
    pub amount: u64,
    pub aid_type: AidType,
    pub executed_at: u64,
    pub status: AidTransactionStatus,
}

/// Status of aid transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AidTransactionStatus {
    Pending,
    Completed,
    Failed,
    Disputed,
}

impl CollectiveBonusSystem {
    /// Create a new collective bonus system
    pub fn new() -> Self {
        Self {
            active_bonus_programs: HashMap::new(),
            participation_tracking: HashMap::new(),
            bonus_calculation_rules: BonusCalculationRules {
                cooperation_multipliers: HashMap::new(),
                achievement_bonuses: HashMap::new(),
                network_effect_bonuses: HashMap::new(),
                solidarity_multipliers: HashMap::new(),
            },
            distribution_mechanisms: vec![DistributionMechanism::DirectDistribution],
        }
    }

    /// Create a new bonus program
    pub fn create_bonus_program(&mut self, program: BonusProgram) -> Result<(), CommonError> {
        if self.active_bonus_programs.contains_key(&program.program_id) {
            return Err(CommonError::InvalidInputError(
                "Program ID already exists".into()
            ));
        }

        self.active_bonus_programs.insert(program.program_id.clone(), program);
        Ok(())
    }

    /// Register participant in a bonus program
    pub fn register_participant(
        &mut self,
        program_id: &str,
        participant: &Did,
    ) -> Result<(), CommonError> {
        let program = self.active_bonus_programs.get_mut(program_id)
            .ok_or_else(|| CommonError::InvalidInputError("Program not found".into()))?;

        if program.status != ProgramStatus::Active {
            return Err(CommonError::PolicyDenied("Program is not active".into()));
        }

        // Check eligibility (extract criteria to avoid borrowing conflict)
        let eligibility_criteria = program.eligibility_criteria.clone();
        if !self.check_eligibility(&eligibility_criteria, participant) {
            return Err(CommonError::PolicyDenied("Participant not eligible".into()));
        }

        // Update participation tracking
        let participation_record = self.participation_tracking
            .entry(participant.clone())
            .or_insert_with(|| ParticipationRecord {
                participant: participant.clone(),
                program_participations: HashMap::new(),
                overall_cooperation_score: 0.0,
                bonus_earnings_history: Vec::new(),
                reputation_bonuses: 0.0,
            });

        participation_record.program_participations.insert(
            program_id.to_string(),
            ProgramParticipation {
                program_id: program_id.to_string(),
                joined_at: SystemTimeProvider.unix_seconds(),
                contribution_metrics: HashMap::new(),
                milestone_achievements: Vec::new(),
                collaboration_partners: HashSet::new(),
                impact_measurement: 0.0,
            }
        );

        Ok(())
    }

    /// Calculate and distribute bonuses for a program
    pub fn distribute_program_bonuses<L: ManaLedger>(
        &mut self,
        program_id: &str,
        ledger: &mut L,
    ) -> Result<Vec<BonusDistribution>, CommonError> {
        let program = self.active_bonus_programs.get(program_id)
            .ok_or_else(|| CommonError::InvalidInputError("Program not found".into()))?
            .clone();

        let participants: Vec<Did> = self.participation_tracking
            .iter()
            .filter(|(_, record)| record.program_participations.contains_key(program_id))
            .map(|(did, _)| did.clone())
            .collect();

        let bonus_distributions = self.calculate_bonus_distributions(&program, &participants)?;
        
        // Execute distributions
        for distribution in &bonus_distributions {
            ledger.credit(&distribution.recipient, distribution.bonus_amount)?;
            
            // Update earning history
            if let Some(record) = self.participation_tracking.get_mut(&distribution.recipient) {
                record.bonus_earnings_history.push(BonusEarning {
                    program_id: program_id.to_string(),
                    earning_period: (distribution.period_start, distribution.period_end),
                    bonus_amount: distribution.bonus_amount,
                    earning_reason: distribution.earning_reason.clone(),
                    awarded_at: SystemTimeProvider.unix_seconds(),
                });
            }
        }

        Ok(bonus_distributions)
    }

    /// Check if participant meets eligibility criteria
    fn check_eligibility(&self, _criteria: &[EligibilityCriterion], _participant: &Did) -> bool {
        // Placeholder implementation
        true
    }

    /// Calculate bonus distributions for participants
    fn calculate_bonus_distributions(
        &self,
        program: &BonusProgram,
        participants: &[Did],
    ) -> Result<Vec<BonusDistribution>, CommonError> {
        let mut distributions = Vec::new();
        let current_time = SystemTimeProvider.unix_seconds();

        match &program.bonus_calculation {
            BonusCalculation::EqualShare => {
                let bonus_per_participant = program.total_bonus_pool / participants.len() as u64;
                
                for participant in participants {
                    distributions.push(BonusDistribution {
                        recipient: participant.clone(),
                        bonus_amount: bonus_per_participant,
                        earning_reason: "Equal share participation".to_string(),
                        period_start: current_time - 3600, // Last hour
                        period_end: current_time,
                    });
                }
            }
            BonusCalculation::ProportionalContribution { contribution_metric } => {
                let total_contributions = self.calculate_total_contributions(participants, contribution_metric);
                
                for participant in participants {
                    let contribution = self.get_participant_contribution(participant, contribution_metric);
                    let bonus_share = if total_contributions > 0.0 {
                        (contribution / total_contributions) * program.total_bonus_pool as f64
                    } else {
                        0.0
                    };
                    
                    distributions.push(BonusDistribution {
                        recipient: participant.clone(),
                        bonus_amount: bonus_share as u64,
                        earning_reason: format!("Proportional contribution: {}", contribution_metric),
                        period_start: current_time - 3600,
                        period_end: current_time,
                    });
                }
            }
            _ => {
                // Other calculation methods would be implemented here
                return Err(CommonError::NotImplementedError(
                    "Bonus calculation method not implemented".into()
                ));
            }
        }

        Ok(distributions)
    }

    /// Calculate total contributions across participants
    fn calculate_total_contributions(&self, _participants: &[Did], _metric: &str) -> f64 {
        // Placeholder implementation
        100.0
    }

    /// Get individual participant contribution
    fn get_participant_contribution(&self, _participant: &Did, _metric: &str) -> f64 {
        // Placeholder implementation
        10.0
    }
}

/// Record of bonus distribution
#[derive(Debug, Clone)]
pub struct BonusDistribution {
    pub recipient: Did,
    pub bonus_amount: u64,
    pub earning_reason: String,
    pub period_start: u64,
    pub period_end: u64,
}

impl Default for TrustWeightedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MutualAidCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CollectiveBonusSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_trust_weighted_allocator() {
        let mut allocator = TrustWeightedAllocator::new();
        
        let trustor = Did::from_str("did:test:trustor").unwrap();
        let trustee1 = Did::from_str("did:test:trustee1").unwrap();
        let trustee2 = Did::from_str("did:test:trustee2").unwrap();
        
        // Set trust scores
        allocator.update_trust_score(&trustor, &trustee1, 0.8).unwrap();
        allocator.update_trust_score(&trustor, &trustee2, 0.4).unwrap();
        
        // Check trust scores
        assert_eq!(allocator.get_trust_score(&trustor, &trustee1), 0.8);
        assert_eq!(allocator.get_trust_score(&trustor, &trustee2), 0.4);
        
        // Test most trusted
        let most_trusted = allocator.get_most_trusted(&trustor, 2);
        assert_eq!(most_trusted.len(), 2);
        assert_eq!(most_trusted[0].1, 0.8); // First should be highest trust
    }

    #[test]
    fn test_mutual_aid_coordinator() {
        let mut coordinator = MutualAidCoordinator::new();
        
        let request = AidRequest {
            request_id: "req_1".to_string(),
            requester: Did::from_str("did:test:requester").unwrap(),
            aid_type: AidType::Emergency { emergency_type: "food".to_string() },
            urgency_level: UrgencyLevel::High,
            resource_amount: 100,
            description: "Need food assistance".to_string(),
            geographical_scope: GeographicalScope::Local { radius_km: 10.0 },
            time_sensitivity: SystemTimeProvider.unix_seconds() + 3600,
            reciprocity_commitment: ReciprocityCommitment::PayItForward,
            verification_status: VerificationStatus::CommunityVerified { verifiers: vec![] },
            created_at: SystemTimeProvider.unix_seconds(),
        };
        
        let offer = AidOffer {
            offer_id: "off_1".to_string(),
            provider: Did::from_str("did:test:provider").unwrap(),
            aid_type: AidType::Emergency { emergency_type: "food".to_string() },
            available_amount: 150,
            conditions: vec![],
            geographical_reach: GeographicalScope::Local { radius_km: 20.0 },
            availability_window: (SystemTimeProvider.unix_seconds(), SystemTimeProvider.unix_seconds() + 7200),
            preferred_recipients: vec![],
            reciprocity_expectations: ReciprocityExpectation::None,
            created_at: SystemTimeProvider.unix_seconds(),
        };
        
        // Submit request and offer
        coordinator.submit_aid_request(request).unwrap();
        coordinator.submit_aid_offer(offer).unwrap();
        
        // Find matches
        let matches = coordinator.match_aid_requests_and_offers();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].recommended_amount, 100); // Should match request amount
    }

    #[test]
    fn test_collective_bonus_system() {
        let mut bonus_system = CollectiveBonusSystem::new();
        
        let program = BonusProgram {
            program_id: "test_program".to_string(),
            program_name: "Test Bonus Program".to_string(),
            program_type: BonusProgramType::CooperationIncentive { cooperation_type: "mutual_aid".to_string() },
            eligibility_criteria: vec![],
            bonus_calculation: BonusCalculation::EqualShare,
            duration: ProgramDuration::Fixed { duration_days: 30 },
            max_participants: Some(10),
            total_bonus_pool: 1000,
            status: ProgramStatus::Active,
        };
        
        // Create program
        bonus_system.create_bonus_program(program).unwrap();
        
        // Register participants
        let participant1 = Did::from_str("did:test:participant1").unwrap();
        let participant2 = Did::from_str("did:test:participant2").unwrap();
        
        bonus_system.register_participant("test_program", &participant1).unwrap();
        bonus_system.register_participant("test_program", &participant2).unwrap();
        
        // Check participation tracking
        assert!(bonus_system.participation_tracking.contains_key(&participant1));
        assert!(bonus_system.participation_tracking.contains_key(&participant2));
    }
}