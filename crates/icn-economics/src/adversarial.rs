//! Adversarial-resilient economic components
//!
//! This module implements Byzantine fault tolerance, anti-gaming mechanisms,
//! and emergency protocols for the ICN economic system.

use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Byzantine fault tolerance for economic operations
pub trait ByzantineEconomics {
    /// Verify economic operation with Byzantine consensus
    fn verify_with_consensus(
        &self,
        operation: &EconomicOperation,
        validator_signatures: &[ValidatorSignature],
    ) -> Result<ConsensusResult, CommonError>;

    /// Check if sufficient consensus exists for operation
    fn check_consensus_threshold(
        &self,
        validator_signatures: &[ValidatorSignature],
    ) -> Result<bool, CommonError>;

    /// Get current validator set
    fn get_validator_set(&self) -> Result<Vec<Did>, CommonError>;
}

/// Game-theoretic security mechanisms
pub trait GameTheoreticSecurity {
    /// Detect potential gaming or exploitation attempts
    fn detect_gaming_attempt(
        &self,
        account: &Did,
        behavior_history: &BehaviorHistory,
    ) -> Result<GamingDetectionResult, CommonError>;

    /// Apply anti-gaming adjustments
    fn apply_anti_gaming_measures(
        &self,
        account: &Did,
        gaming_indicators: &GamingIndicators,
    ) -> Result<AntiGamingResult, CommonError>;

    /// Check for Sybil attack patterns
    fn detect_sybil_attack(
        &self,
        accounts: &[Did],
        network_analysis: &NetworkAnalysis,
    ) -> Result<SybilDetectionResult, CommonError>;
}

/// Emergency protocols for coordinated attacks
pub trait EmergencyProtocols {
    /// Detect coordinated economic attacks
    fn detect_coordinated_attack(
        &self,
        attack_indicators: &AttackIndicators,
    ) -> Result<AttackDetectionResult, CommonError>;

    /// Activate emergency response protocols
    fn activate_emergency_response(
        &self,
        attack_type: AttackType,
        severity: AttackSeverity,
    ) -> Result<EmergencyResponse, CommonError>;

    /// Check current threat level
    fn get_threat_level(&self) -> Result<ThreatLevel, CommonError>;
}

/// Economic operation for Byzantine consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicOperation {
    pub operation_type: EconomicOperationType,
    pub initiator: Did,
    pub parameters: HashMap<String, String>,
    pub timestamp: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicOperationType {
    ManaRegeneration {
        account: Did,
        amount: u64,
        capacity_proof: String,
    },
    ResourceAllocation {
        requester: Did,
        resource_type: String,
        amount: u64,
    },
    CreditIssuance {
        creditor: Did,
        debtor: Did,
        amount: u64,
    },
    GovernanceAction {
        proposal_id: String,
        action_type: String,
    },
}

/// Validator signature for Byzantine consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator: Did,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

/// Consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub consensus_reached: bool,
    pub supporting_validators: Vec<Did>,
    pub opposing_validators: Vec<Did>,
    pub abstaining_validators: Vec<Did>,
    pub consensus_strength: f64,
}

/// Behavior history for gaming detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorHistory {
    pub account: Did,
    pub transaction_patterns: TransactionPatterns,
    pub capacity_claims: Vec<CapacityClaimRecord>,
    pub reputation_changes: Vec<ReputationChangeRecord>,
    pub social_connections: Vec<Did>,
    pub temporal_patterns: TemporalPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPatterns {
    pub transaction_frequency: f64,
    pub amount_distribution: AmountDistribution,
    pub counterparty_diversity: f64,
    pub temporal_clustering: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistribution {
    pub mean: f64,
    pub variance: f64,
    pub skewness: f64,
    pub outlier_frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityClaimRecord {
    pub timestamp: u64,
    pub claimed_capacity: CapacityMetrics,
    pub verification_status: VerificationStatus,
    pub proof_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityMetrics {
    pub compute_capacity: f64,
    pub storage_capacity: f64,
    pub bandwidth_capacity: f64,
    pub uptime_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Disputed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangeRecord {
    pub timestamp: u64,
    pub change_amount: i64,
    pub change_reason: String,
    pub evidence_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    pub activity_periodicity: f64,
    pub burst_patterns: Vec<BurstPattern>,
    pub dormancy_periods: Vec<DormancyPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstPattern {
    pub start_time: u64,
    pub end_time: u64,
    pub intensity: f64,
    pub pattern_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DormancyPeriod {
    pub start_time: u64,
    pub end_time: u64,
    pub activity_level: f64,
}

/// Gaming detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingDetectionResult {
    pub gaming_detected: bool,
    pub confidence_score: f64,
    pub gaming_indicators: GamingIndicators,
    pub recommended_actions: Vec<AntiGamingAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingIndicators {
    pub capacity_inflation_score: f64,
    pub reputation_farming_score: f64,
    pub transaction_manipulation_score: f64,
    pub collusion_score: f64,
    pub sybil_attack_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntiGamingAction {
    ReduceCapacityWeight(f64),
    FreezeAccount { duration_hours: u64 },
    RequireAdditionalVerification,
    ApplyPenalty { penalty_amount: u64 },
    EscalateToGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiGamingResult {
    pub actions_applied: Vec<AntiGamingAction>,
    pub adjusted_parameters: HashMap<String, f64>,
    pub follow_up_required: bool,
}

/// Network analysis for Sybil detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysis {
    pub social_graph: SocialGraph,
    pub connectivity_metrics: ConnectivityMetrics,
    pub clustering_analysis: ClusteringAnalysis,
    pub identity_verification_data: IdentityVerificationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialGraph {
    pub nodes: Vec<Did>,
    pub edges: Vec<SocialConnection>,
    pub trust_scores: HashMap<Did, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConnection {
    pub from: Did,
    pub to: Did,
    pub connection_strength: f64,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    DirectInteraction,
    MutualConnection,
    TransactionHistory,
    ReputationEndorsement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityMetrics {
    pub average_path_length: f64,
    pub clustering_coefficient: f64,
    pub betweenness_centrality: HashMap<Did, f64>,
    pub eigenvector_centrality: HashMap<Did, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringAnalysis {
    pub detected_clusters: Vec<IdentityCluster>,
    pub cluster_suspicion_scores: HashMap<String, f64>,
    pub anomalous_patterns: Vec<AnomalousPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCluster {
    pub cluster_id: String,
    pub members: Vec<Did>,
    pub cluster_type: ClusterType,
    pub formation_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    NaturalCommunity,
    SuspiciousSybil,
    CoordinatedBehavior,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalousPattern {
    pub pattern_id: String,
    pub affected_accounts: Vec<Did>,
    pub pattern_description: String,
    pub anomaly_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityVerificationData {
    pub verification_levels: HashMap<Did, VerificationLevel>,
    pub proof_documents: HashMap<Did, Vec<ProofDocument>>,
    pub cross_references: HashMap<Did, Vec<CrossReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationLevel {
    Unverified,
    BasicVerification,
    ExtendedVerification,
    CommunityVerification,
    InstitutionalVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDocument {
    pub document_type: String,
    pub verification_method: String,
    pub verification_timestamp: u64,
    pub verifier: Option<Did>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub reference_type: String,
    pub reference_data: String,
    pub verification_status: VerificationStatus,
}

/// Sybil detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SybilDetectionResult {
    pub sybil_attack_detected: bool,
    pub confidence_score: f64,
    pub suspected_sybil_clusters: Vec<IdentityCluster>,
    pub recommended_countermeasures: Vec<SybilCountermeasure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SybilCountermeasure {
    IncreaseVerificationRequirements,
    FreezeNewAccountCreation { duration_hours: u64 },
    RequireIdentityProofs,
    ImplementWaitingPeriods,
    EnableManualReview,
}

/// Attack indicators for emergency detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackIndicators {
    pub attack_type: AttackType,
    pub severity_indicators: SeverityIndicators,
    pub affected_accounts: Vec<Did>,
    pub attack_vectors: Vec<AttackVector>,
    pub temporal_signature: TemporalSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    MassResourceDrain,
    SybilSwarm,
    EconomicManipulation,
    NetworkPartition,
    GovernanceCapture,
    ReputationAttack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityIndicators {
    pub resource_impact: f64,
    pub network_disruption: f64,
    pub economic_damage: f64,
    pub participation_affected: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackVector {
    pub vector_type: String,
    pub exploitation_method: String,
    pub target_vulnerabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalSignature {
    pub attack_start_time: u64,
    pub attack_duration: u64,
    pub escalation_pattern: EscalationPattern,
    pub coordination_indicators: Vec<CoordinationIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationPattern {
    Gradual,
    Sudden,
    Periodic,
    Coordinated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationIndicator {
    pub indicator_type: String,
    pub evidence: String,
    pub confidence: f64,
}

/// Attack detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDetectionResult {
    pub attack_confirmed: bool,
    pub confidence_level: f64,
    pub attack_classification: AttackClassification,
    pub recommended_response_level: ResponseLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackClassification {
    pub primary_attack_type: AttackType,
    pub secondary_attack_types: Vec<AttackType>,
    pub attack_sophistication: AttackSophistication,
    pub estimated_resources: ResourceEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackSophistication {
    Basic,
    Intermediate,
    Advanced,
    StateLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub computational_resources: f64,
    pub financial_resources: f64,
    pub human_resources: f64,
    pub time_investment: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseLevel {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

/// Attack severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Emergency response actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyResponse {
    pub response_id: String,
    pub activated_protocols: Vec<EmergencyProtocol>,
    pub response_timeline: ResponseTimeline,
    pub coordination_requirements: CoordinationRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyProtocol {
    RateLimitingActivated {
        rate_limit_factor: f64,
        duration_hours: u64,
    },
    AccountFreeze {
        affected_accounts: Vec<Did>,
        freeze_duration_hours: u64,
    },
    VerificationRequirementsIncreased {
        new_verification_level: VerificationLevel,
        affected_operations: Vec<String>,
    },
    NetworkIsolation {
        isolated_nodes: Vec<Did>,
        isolation_reason: String,
    },
    EmergencyGovernance {
        governance_override: bool,
        emergency_council: Vec<Did>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeline {
    pub immediate_actions: Vec<ImmediateAction>,
    pub short_term_actions: Vec<ShortTermAction>,
    pub long_term_actions: Vec<LongTermAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmediateAction {
    pub action: String,
    pub execution_time_seconds: u64,
    pub responsible_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermAction {
    pub action: String,
    pub execution_time_hours: u64,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermAction {
    pub action: String,
    pub execution_time_days: u64,
    pub governance_approval_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequirements {
    pub requires_human_approval: bool,
    pub requires_governance_vote: bool,
    pub requires_external_coordination: bool,
    pub notification_targets: Vec<NotificationTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTarget {
    pub target_type: String,
    pub target_id: String,
    pub notification_method: String,
}

/// Current threat level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Green,   // Normal operations
    Yellow,  // Elevated monitoring
    Orange,  // High alert
    Red,     // Critical threat
    Black,   // Maximum threat level
}