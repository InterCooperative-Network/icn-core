//! Organizational types for ICN economic framework
//!
//! This module defines the three organizational types in ICN:
//! - Cooperatives: Economic production hubs
//! - Communities: Civil and cultural centers
//! - Federations: Bridging organizations

use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Organizational type in the ICN network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationType {
    Cooperative(CooperativeProfile),
    Community(CommunityProfile),
    Federation(FederationProfile),
}

/// Cooperative organizational profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeProfile {
    pub cooperative_id: Did,
    pub cooperative_name: String,
    pub production_capacity: ProductionCapacity,
    pub worker_ownership: WorkerOwnership,
    pub economic_focus: EconomicFocus,
    pub trade_relationships: Vec<TradeRelationship>,
    pub resource_specialization: Vec<ResourceSpecialization>,
}

/// Community organizational profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityProfile {
    pub community_id: Did,
    pub community_name: String,
    pub governance_model: GovernanceModel,
    pub mutual_aid_networks: Vec<MutualAidNetwork>,
    pub care_economy_support: CareEconomySupport,
    pub cultural_activities: Vec<CulturalActivity>,
    pub democratic_processes: DemocraticProcesses,
}

/// Federation organizational profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationProfile {
    pub federation_id: Did,
    pub federation_name: String,
    pub member_organizations: Vec<Did>,
    pub coordination_protocols: CoordinationProtocols,
    pub resource_sharing_agreements: Vec<ResourceSharingAgreement>,
    pub dispute_resolution_framework: DisputeResolutionFramework,
    pub collective_action_capabilities: CollectiveActionCapabilities,
}

/// Production capacity metrics for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionCapacity {
    pub output_categories: Vec<OutputCategory>,
    pub production_efficiency: f64,
    pub scalability_factor: f64,
    pub quality_metrics: QualityMetrics,
    pub innovation_capacity: InnovationCapacity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputCategory {
    pub category_name: String,
    pub output_volume: u64,
    pub quality_score: f64,
    pub market_demand: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub consistency_score: f64,
    pub reliability_score: f64,
    pub customer_satisfaction: f64,
    pub defect_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnovationCapacity {
    pub research_investment: f64,
    pub development_projects: u64,
    pub innovation_adoption_rate: f64,
    pub knowledge_sharing_score: f64,
}

/// Worker ownership structure for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerOwnership {
    pub ownership_model: OwnershipModel,
    pub decision_making_structure: DecisionMakingStructure,
    pub profit_distribution: ProfitDistribution,
    pub member_benefits: MemberBenefits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipModel {
    FullWorkerOwnership,
    MajorityWorkerOwnership { worker_percentage: f64 },
    MultiStakeholder { stakeholder_percentages: HashMap<String, f64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMakingStructure {
    pub governance_type: CooperativeGovernanceType,
    pub voting_mechanisms: Vec<VotingMechanism>,
    pub leadership_structure: LeadershipStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CooperativeGovernanceType {
    Democratic,
    Consensus,
    Delegated,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingMechanism {
    pub mechanism_type: String,
    pub threshold_required: f64,
    pub applicable_decisions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadershipStructure {
    pub leadership_model: LeadershipModel,
    pub term_lengths: HashMap<String, u64>,
    pub rotation_requirements: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeadershipModel {
    Elected,
    Rotational,
    Consensus,
    Flat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitDistribution {
    pub distribution_model: DistributionModel,
    pub reinvestment_percentage: f64,
    pub member_dividends: f64,
    pub community_contributions: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionModel {
    EqualShares,
    ContributionBased,
    NeedsBased,
    Hybrid { weights: HashMap<String, f64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberBenefits {
    pub healthcare_coverage: bool,
    pub education_support: f64,
    pub retirement_benefits: f64,
    pub flexible_work_arrangements: bool,
}

/// Economic focus areas for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicFocus {
    pub primary_sectors: Vec<EconomicSector>,
    pub value_creation_activities: Vec<ValueCreationActivity>,
    pub sustainability_practices: SustainabilityPractices,
    pub market_positioning: MarketPositioning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicSector {
    Agriculture,
    Manufacturing,
    Technology,
    Services,
    CreativeArts,
    Healthcare,
    Education,
    Energy,
    Transportation,
    Construction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCreationActivity {
    pub activity_name: String,
    pub value_add_percentage: f64,
    pub resource_requirements: HashMap<String, u64>,
    pub output_products: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityPractices {
    pub environmental_score: f64,
    pub social_impact_score: f64,
    pub economic_sustainability_score: f64,
    pub certification_standards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositioning {
    pub target_markets: Vec<String>,
    pub competitive_advantages: Vec<String>,
    pub brand_values: Vec<String>,
    pub market_share_estimates: HashMap<String, f64>,
}

/// Trade relationship for inter-cooperative commerce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRelationship {
    pub partner_cooperative: Did,
    pub trade_volume: u64,
    pub traded_resources: Vec<TradedResource>,
    pub relationship_strength: f64,
    pub trade_agreements: Vec<TradeAgreement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradedResource {
    pub resource_type: String,
    pub volume: u64,
    pub quality_requirements: QualityRequirements,
    pub pricing_model: ResourcePricingModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub minimum_quality_score: f64,
    pub certification_required: bool,
    pub inspection_frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePricingModel {
    Fixed { price: u64 },
    Variable { base_price: u64, adjustment_factors: Vec<String> },
    Negotiated,
    CostPlus { cost_multiplier: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreement {
    pub agreement_id: String,
    pub agreement_type: String,
    pub terms: HashMap<String, String>,
    pub duration: u64,
    pub renewal_conditions: Vec<String>,
}

/// Resource specialization for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpecialization {
    pub specialization_area: String,
    pub expertise_level: ExpertiseLevel,
    pub capacity_metrics: SpecializationCapacity,
    pub knowledge_base: KnowledgeBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    WorldClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationCapacity {
    pub production_capacity: u64,
    pub service_capacity: u64,
    pub teaching_capacity: u64,
    pub research_capacity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub documented_processes: u64,
    pub training_programs: u64,
    pub research_publications: u64,
    pub knowledge_sharing_frequency: f64,
}

/// Governance model for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceModel {
    pub governance_type: CommunityGovernanceType,
    pub decision_making_processes: Vec<DecisionMakingProcess>,
    pub representation_structure: RepresentationStructure,
    pub accountability_mechanisms: AccountabilityMechanisms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunityGovernanceType {
    DirectDemocracy,
    RepresentativeDemocracy,
    ConsensusDecisionMaking,
    ParticipantBudgeting,
    SortitionBasedGovernance,
    Hybrid { components: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMakingProcess {
    pub process_name: String,
    pub applicable_decisions: Vec<String>,
    pub participation_requirements: ParticipationRequirements,
    pub decision_criteria: DecisionCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationRequirements {
    pub minimum_engagement_level: f64,
    pub required_qualifications: Vec<String>,
    pub time_commitment: u64,
    pub preparation_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCriteria {
    pub consensus_threshold: f64,
    pub expertise_weighting: bool,
    pub impact_consideration: bool,
    pub time_constraints: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentationStructure {
    pub representation_type: RepresentationType,
    pub selection_mechanisms: Vec<SelectionMechanism>,
    pub term_limits: HashMap<String, u64>,
    pub recall_mechanisms: Vec<RecallMechanism>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepresentationType {
    GeographicRepresentation,
    InterestBasedRepresentation,
    DemographicRepresentation,
    ExpertiseBasedRepresentation,
    RandomSelection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionMechanism {
    pub mechanism_type: String,
    pub selection_criteria: Vec<String>,
    pub fairness_measures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallMechanism {
    pub trigger_conditions: Vec<String>,
    pub process_requirements: Vec<String>,
    pub threshold_required: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityMechanisms {
    pub transparency_requirements: TransparencyRequirements,
    pub performance_monitoring: PerformanceMonitoring,
    pub feedback_systems: Vec<FeedbackSystem>,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyRequirements {
    pub public_reporting_frequency: u64,
    pub decision_documentation: bool,
    pub financial_transparency: bool,
    pub process_transparency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoring {
    pub performance_indicators: Vec<PerformanceIndicator>,
    pub monitoring_frequency: u64,
    pub evaluation_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIndicator {
    pub indicator_name: String,
    pub measurement_method: String,
    pub target_value: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSystem {
    pub system_type: String,
    pub feedback_channels: Vec<String>,
    pub response_time_requirements: u64,
    pub action_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub resolution_processes: Vec<ResolutionProcess>,
    pub mediation_services: bool,
    pub escalation_procedures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionProcess {
    pub process_name: String,
    pub applicable_conflicts: Vec<String>,
    pub resolution_steps: Vec<String>,
    pub success_metrics: Vec<String>,
}

/// Mutual aid network for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualAidNetwork {
    pub network_id: String,
    pub network_type: MutualAidType,
    pub participants: Vec<Did>,
    pub resource_pools: Vec<ResourcePool>,
    pub coordination_mechanisms: CoordinationMechanisms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutualAidType {
    EmergencySupport,
    ChildcareSupport,
    ElderCare,
    HealthcareSupport,
    EducationalSupport,
    SkillSharing,
    ToolSharing,
    GeneralSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub resource_type: String,
    pub available_amount: u64,
    pub access_criteria: Vec<String>,
    pub contribution_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMechanisms {
    pub coordination_tools: Vec<String>,
    pub communication_channels: Vec<String>,
    pub scheduling_systems: Vec<String>,
    pub tracking_systems: Vec<String>,
}

/// Care economy support for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareEconomySupport {
    pub care_work_recognition: CareWorkRecognition,
    pub care_infrastructure: CareInfrastructure,
    pub caregiver_support: CaregiverSupport,
    pub care_quality_assurance: CareQualityAssurance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareWorkRecognition {
    pub recognition_mechanisms: Vec<String>,
    pub compensation_models: Vec<CompensationModel>,
    pub career_development: bool,
    pub social_status_enhancement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationModel {
    pub model_type: String,
    pub compensation_rate: f64,
    pub eligibility_criteria: Vec<String>,
    pub performance_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareInfrastructure {
    pub childcare_facilities: u64,
    pub eldercare_facilities: u64,
    pub healthcare_facilities: u64,
    pub community_centers: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaregiverSupport {
    pub training_programs: Vec<TrainingProgram>,
    pub support_groups: bool,
    pub respite_care: bool,
    pub financial_assistance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgram {
    pub program_name: String,
    pub duration_hours: u64,
    pub certification_provided: bool,
    pub ongoing_education: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareQualityAssurance {
    pub quality_standards: Vec<String>,
    pub monitoring_mechanisms: Vec<String>,
    pub improvement_processes: Vec<String>,
    pub outcome_measurements: Vec<String>,
}

/// Cultural activity for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalActivity {
    pub activity_type: CulturalActivityType,
    pub frequency: ActivityFrequency,
    pub participation_level: f64,
    pub community_impact: f64,
    pub resource_requirements: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CulturalActivityType {
    Arts,
    Music,
    Literature,
    Sports,
    Festivals,
    Education,
    Spirituality,
    Crafts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityFrequency {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    Annual,
    Irregular,
}

/// Democratic processes for communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemocraticProcesses {
    pub voting_systems: Vec<VotingSystem>,
    pub participation_rates: ParticipationRates,
    pub civic_education: CivicEducation,
    pub democratic_innovations: Vec<DemocraticInnovation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingSystem {
    pub system_type: String,
    pub applicable_decisions: Vec<String>,
    pub participation_requirements: Vec<String>,
    pub fairness_measures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationRates {
    pub average_participation: f64,
    pub participation_by_demographic: HashMap<String, f64>,
    pub participation_trends: Vec<ParticipationTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationTrend {
    pub time_period: String,
    pub participation_change: f64,
    pub influencing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicEducation {
    pub education_programs: Vec<EducationProgram>,
    pub media_literacy: bool,
    pub critical_thinking: bool,
    pub civic_engagement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationProgram {
    pub program_name: String,
    pub target_audience: String,
    pub duration: u64,
    pub learning_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemocraticInnovation {
    pub innovation_name: String,
    pub innovation_description: String,
    pub effectiveness_score: f64,
    pub adoption_rate: f64,
}

/// Coordination protocols for federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationProtocols {
    pub communication_protocols: Vec<CommunicationProtocol>,
    pub decision_coordination: DecisionCoordination,
    pub resource_coordination: ResourceCoordination,
    pub information_sharing: InformationSharing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationProtocol {
    pub protocol_name: String,
    pub communication_frequency: u64,
    pub participants: Vec<String>,
    pub information_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCoordination {
    pub coordination_mechanisms: Vec<String>,
    pub consensus_building: bool,
    pub conflict_mediation: bool,
    pub implementation_coordination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCoordination {
    pub resource_sharing_protocols: Vec<String>,
    pub allocation_mechanisms: Vec<String>,
    pub efficiency_optimization: bool,
    pub redundancy_management: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationSharing {
    pub sharing_protocols: Vec<String>,
    pub data_standards: Vec<String>,
    pub privacy_protection: bool,
    pub knowledge_management: bool,
}

/// Resource sharing agreement for federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingAgreement {
    pub agreement_id: String,
    pub participating_organizations: Vec<Did>,
    pub shared_resources: Vec<SharedResource>,
    pub sharing_conditions: SharingConditions,
    pub governance_structure: SharingGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedResource {
    pub resource_type: String,
    pub total_capacity: u64,
    pub allocation_method: AllocationMethod,
    pub access_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationMethod {
    EqualShare,
    NeedsBased,
    ContributionBased,
    Auction,
    FirstComeFirstServed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConditions {
    pub usage_limitations: Vec<String>,
    pub contribution_requirements: Vec<String>,
    pub maintenance_responsibilities: Vec<String>,
    pub termination_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingGovernance {
    pub governance_model: String,
    pub decision_making_authority: Vec<String>,
    pub dispute_resolution: String,
    pub modification_procedures: Vec<String>,
}

/// Dispute resolution framework for federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolutionFramework {
    pub resolution_stages: Vec<ResolutionStage>,
    pub mediation_services: MediationServices,
    pub arbitration_services: ArbitrationServices,
    pub enforcement_mechanisms: EnforcementMechanisms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionStage {
    pub stage_name: String,
    pub stage_requirements: Vec<String>,
    pub time_limits: Option<u64>,
    pub escalation_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationServices {
    pub available_mediators: Vec<Did>,
    pub mediation_processes: Vec<String>,
    pub success_rates: f64,
    pub cost_structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationServices {
    pub arbitration_panels: Vec<ArbitrationPanel>,
    pub arbitration_rules: Vec<String>,
    pub binding_authority: bool,
    pub appeal_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationPanel {
    pub panel_id: String,
    pub arbitrators: Vec<Did>,
    pub expertise_areas: Vec<String>,
    pub case_load: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementMechanisms {
    pub enforcement_tools: Vec<String>,
    pub compliance_monitoring: bool,
    pub sanctions: Vec<Sanction>,
    pub incentive_structures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sanction {
    pub sanction_type: String,
    pub severity: SanctionSeverity,
    pub applicable_violations: Vec<String>,
    pub appeal_rights: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanctionSeverity {
    Warning,
    Suspension,
    FinancialPenalty,
    ResourceRestriction,
    Expulsion,
}

/// Collective action capabilities for federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveActionCapabilities {
    pub action_types: Vec<CollectiveActionType>,
    pub coordination_capacity: CoordinationCapacity,
    pub resource_mobilization: ResourceMobilization,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveActionType {
    pub action_name: String,
    pub action_description: String,
    pub required_resources: HashMap<String, u64>,
    pub expected_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationCapacity {
    pub max_participating_organizations: u64,
    pub coordination_complexity: CoordinationComplexity,
    pub communication_infrastructure: bool,
    pub leadership_capacity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationComplexity {
    Simple,
    Moderate,
    Complex,
    HighlyComplex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMobilization {
    pub financial_resources: f64,
    pub human_resources: u64,
    pub technical_resources: HashMap<String, u64>,
    pub time_resources: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub assessment_methods: Vec<String>,
    pub success_metrics: Vec<String>,
    pub impact_monitoring: bool,
    pub feedback_integration: bool,
}

/// Trait for managing organizational economics
pub trait OrganizationalEconomics {
    /// Get the organization type and profile
    fn get_organization_profile(&self, org_id: &Did) -> Result<OrganizationType, CommonError>;

    /// Update organization profile
    fn update_organization_profile(
        &self,
        org_id: &Did,
        profile: OrganizationType,
    ) -> Result<(), CommonError>;

    /// Calculate organization-specific economic metrics
    fn calculate_org_economic_metrics(
        &self,
        org_id: &Did,
    ) -> Result<OrganizationalEconomicMetrics, CommonError>;

    /// Facilitate inter-organizational collaboration
    fn facilitate_collaboration(
        &self,
        initiator_org: &Did,
        target_org: &Did,
        collaboration_type: CollaborationType,
    ) -> Result<CollaborationResult, CommonError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalEconomicMetrics {
    pub organization_id: Did,
    pub organization_type: String,
    pub economic_health_score: f64,
    pub productivity_metrics: ProductivityMetrics,
    pub collaboration_metrics: CollaborationMetrics,
    pub sustainability_metrics: SustainabilityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub output_efficiency: f64,
    pub resource_utilization: f64,
    pub innovation_rate: f64,
    pub quality_metrics: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub partnership_count: u64,
    pub collaboration_success_rate: f64,
    pub network_centrality: f64,
    pub knowledge_sharing_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityMetrics {
    pub environmental_impact: f64,
    pub social_impact: f64,
    pub economic_sustainability: f64,
    pub long_term_viability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationType {
    ResourceSharing,
    KnowledgeExchange,
    JointVenture,
    MutualAid,
    TradePartnership,
    ResearchCollaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationResult {
    pub collaboration_id: String,
    pub collaboration_established: bool,
    pub collaboration_terms: HashMap<String, String>,
    pub expected_benefits: Vec<String>,
    pub success_metrics: Vec<String>,
}