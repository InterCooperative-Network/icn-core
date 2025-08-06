//! Organizational Structure Differentiation Module
//!
//! This module defines and implements different organizational structures within ICN:
//! - Co-ops: Economic hubs focused on production and resource coordination
//! - Communities: Cultural/governance centers focused on social coordination
//! - Federations: Bridge organizations for interoperability between other organizations

use icn_common::{CommonError, Did, NodeScope, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of organizations in the ICN ecosystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrganizationType {
    /// Economic-focused cooperatives
    Coop {
        economic_focus: EconomicFocus,
        production_capacity: ProductionCapacity,
    },
    /// Cultural and governance-focused communities
    Community {
        governance_model: GovernanceModel,
        cultural_values: CulturalValues,
    },
    /// Bridge organizations for inter-organizational coordination
    Federation {
        member_organizations: Vec<Did>,
        interop_protocols: Vec<InteropProtocol>,
    },
}

/// Economic focus areas for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EconomicFocus {
    /// Production cooperatives
    Production { sectors: Vec<String> },
    /// Service provision cooperatives
    Services { service_types: Vec<String> },
    /// Resource sharing cooperatives
    ResourceSharing { resource_types: Vec<String> },
    /// Financial services cooperatives
    Financial { instruments: Vec<String> },
    /// Multi-stakeholder cooperatives
    MultiStakeholder { stakeholder_types: Vec<String> },
}

/// Production capacity metrics for cooperatives
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductionCapacity {
    pub total_capacity: u64,
    pub current_utilization: f64, // 0.0 to 1.0
    pub capacity_by_resource: HashMap<String, u64>,
    pub seasonal_variations: Option<SeasonalCapacity>,
}

/// Seasonal capacity variations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeasonalCapacity {
    pub spring: f64,
    pub summer: f64,
    pub autumn: f64,
    pub winter: f64,
}

/// Governance models for communities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GovernanceModel {
    /// Direct democracy with all decisions by membership vote
    DirectDemocracy,
    /// Representative democracy with elected councils
    RepresentativeDemocracy { council_size: u32 },
    /// Consensus-based decision making
    Consensus { quorum_threshold: f64 },
    /// Sociocracy with circles and double-linking
    Sociocracy { circles: Vec<String> },
    /// Delegated liquid democracy
    LiquidDemocracy { delegation_depth: u32 },
    /// Custom governance hybrid
    Custom { description: String },
}

/// Cultural values framework for communities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CulturalValues {
    pub core_principles: Vec<String>,
    pub conflict_resolution: ConflictResolutionModel,
    pub inclusion_practices: InclusionPractices,
    pub decision_making_culture: DecisionMakingCulture,
}

/// Models for resolving conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolutionModel {
    Mediation,
    Arbitration,
    RestorativeJustice,
    CommunityCouncil,
    PeerSupport,
    Hybrid { models: Vec<String> },
}

/// Practices for inclusion and accessibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InclusionPractices {
    pub accessibility_measures: Vec<String>,
    pub diversity_commitments: Vec<String>,
    pub language_support: Vec<String>,
    pub economic_inclusion: Vec<String>,
}

/// Culture around decision-making processes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionMakingCulture {
    Collaborative,
    Consultative,
    Autonomous,
    Hierarchical,
    NetworkBased,
}

/// Interoperability protocols for federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InteropProtocol {
    /// Economic resource sharing protocols
    ResourceSharing {
        resource_types: Vec<String>,
        sharing_rules: ResourceSharingRules,
    },
    /// Governance coordination protocols
    GovernanceCoordination {
        decision_types: Vec<String>,
        coordination_mechanisms: Vec<String>,
    },
    /// Cultural exchange protocols
    CulturalExchange {
        exchange_types: Vec<String>,
        cultural_safeguards: Vec<String>,
    },
    /// Technical interoperability protocols
    Technical {
        protocol_standards: Vec<String>,
        compatibility_requirements: Vec<String>,
    },
}

/// Rules for resource sharing between organizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSharingRules {
    pub reciprocity_requirements: ReciprocityRequirements,
    pub priority_systems: Vec<PriorityRule>,
    pub capacity_limits: HashMap<String, u64>,
    pub emergency_protocols: Vec<String>,
}

/// Requirements for reciprocal resource sharing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReciprocityRequirements {
    Direct, // 1:1 exchange
    Indirect { time_window: u64 }, // Exchange within time window
    NetworkBased, // Contribute to network, benefit from network
    ValueBased { valuation_method: String }, // Exchange based on agreed valuation
}

/// Priority rules for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorityRule {
    pub rule_type: PriorityType,
    pub weight: f64,
    pub conditions: Vec<String>,
}

/// Types of priority in resource allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityType {
    Emergency,
    BasicNeeds,
    Community,
    Production,
    Development,
    Cultural,
}

/// Complete organization definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Did,
    pub name: String,
    pub organization_type: OrganizationType,
    pub scope: NodeScope,
    pub created_at: u64,
    pub member_count: u32,
    pub economic_policies: EconomicPolicies,
    pub relationships: Vec<OrganizationalRelationship>,
    pub reputation_metrics: ReputationMetrics,
}

/// Economic policies specific to organization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicPolicies {
    pub mana_regeneration_policy: ManaRegenerationPolicy,
    pub resource_allocation_policy: ResourceAllocationPolicy,
    pub surplus_distribution_policy: SurplusDistributionPolicy,
    pub contribution_recognition_policy: ContributionRecognitionPolicy,
}

/// Mana regeneration policies tailored to organization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaRegenerationPolicy {
    pub base_rate: f64,
    pub contribution_multiplier: f64,
    pub capacity_weight: f64,
    pub organization_bonus: f64, // Additional mana based on organization type
    pub solidarity_bonus: f64, // Bonus for inter-organizational cooperation
}

/// Resource allocation policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAllocationPolicy {
    /// Allocate based on need assessment
    NeedsBased {
        assessment_criteria: Vec<String>,
        priority_weights: HashMap<String, f64>,
    },
    /// Allocate based on contribution
    ContributionBased {
        contribution_metrics: Vec<String>,
        time_weighting: f64,
    },
    /// Equal allocation among members
    Equal {
        minimum_guarantee: Option<u64>,
    },
    /// Mixed allocation combining multiple approaches
    Mixed {
        need_weight: f64,
        contribution_weight: f64,
        equality_weight: f64,
    },
    /// Democratic allocation through member voting
    Democratic {
        voting_mechanism: String,
        quorum_requirement: f64,
    },
}

/// Policies for distributing organizational surplus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurplusDistributionPolicy {
    /// Reinvest all surplus back into the organization
    Reinvestment {
        investment_priorities: Vec<String>,
    },
    /// Distribute surplus equally among members
    EqualDistribution,
    /// Distribute based on contribution levels
    ContributionBasedDistribution {
        contribution_metrics: Vec<String>,
    },
    /// Mixed approach
    Mixed {
        reinvestment_percentage: f64,
        distribution_method: Box<SurplusDistributionPolicy>,
    },
    /// Community decision on surplus use
    CommunityDecided {
        decision_mechanism: String,
    },
}

/// Policies for recognizing and rewarding contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionRecognitionPolicy {
    pub recognition_types: Vec<RecognitionType>,
    pub measurement_methods: Vec<MeasurementMethod>,
    pub reward_mechanisms: Vec<RewardMechanism>,
    pub peer_validation_required: bool,
}

/// Types of contribution recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecognitionType {
    Labor { skill_categories: Vec<String> },
    Knowledge { expertise_areas: Vec<String> },
    Care { care_categories: Vec<String> },
    Innovation { innovation_types: Vec<String> },
    Leadership { leadership_roles: Vec<String> },
    Community { community_activities: Vec<String> },
}

/// Methods for measuring contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementMethod {
    TimeTracking,
    PeerAssessment,
    OutputMeasurement,
    ImpactAssessment,
    QualitativeEvaluation,
    CombinedMetrics,
}

/// Mechanisms for rewarding contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardMechanism {
    ManaBonus { multiplier: f64 },
    ResourceTokens { token_types: Vec<String> },
    AccessPrivileges { privileges: Vec<String> },
    Recognition { recognition_forms: Vec<String> },
    DevelopmentOpportunities { opportunities: Vec<String> },
    Hybrid { mechanisms: Vec<String> },
}

/// Relationships between organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalRelationship {
    pub partner_org: Did,
    pub relationship_type: RelationshipType,
    pub established_at: u64,
    pub agreements: Vec<Agreement>,
    pub trust_level: f64, // 0.0 to 1.0
    pub interaction_history: InteractionHistory,
}

/// Types of relationships between organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Economic partnership for trade and resource sharing
    Economic {
        trade_volume: u64,
        primary_exchanges: Vec<String>,
    },
    /// Governance coordination relationship
    Governance {
        coordination_areas: Vec<String>,
        decision_sharing_level: DecisionSharingLevel,
    },
    /// Cultural exchange and learning relationship
    Cultural {
        exchange_programs: Vec<String>,
        shared_activities: Vec<String>,
    },
    /// Technical collaboration relationship
    Technical {
        collaboration_areas: Vec<String>,
        shared_infrastructure: Vec<String>,
    },
    /// Federation membership
    Federation {
        federation_id: Did,
        membership_level: FederationMembershipLevel,
    },
}

/// Levels of decision sharing in governance relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionSharingLevel {
    Advisory, // Advisory input only
    Consultative, // Must be consulted before decisions
    Collaborative, // Joint decision making
    Autonomous, // Independent with information sharing
}

/// Levels of federation membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMembershipLevel {
    Observer, // Can observe but not vote
    Participant, // Can participate and vote on some issues
    Full, // Full participation and voting rights
    Leadership, // Leadership role within federation
}

/// History of interactions between organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionHistory {
    pub total_interactions: u32,
    pub successful_collaborations: u32,
    pub resolved_conflicts: u32,
    pub resource_exchanges: u32,
    pub last_interaction: u64,
    pub satisfaction_scores: Vec<f64>, // Historical satisfaction ratings
}

/// Reputation metrics for organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationMetrics {
    pub reliability_score: f64,
    pub cooperation_score: f64,
    pub innovation_score: f64,
    pub sustainability_score: f64,
    pub transparency_score: f64,
    pub member_satisfaction_score: f64,
    pub external_reputation_score: f64,
}

/// Agreements between organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agreement {
    pub agreement_id: String,
    pub agreement_type: AgreementType,
    pub terms: Vec<String>,
    pub duration: Option<u64>, // Duration in seconds, None for indefinite
    pub signed_at: u64,
    pub status: AgreementStatus,
    pub performance_metrics: Vec<PerformanceMetric>,
}

/// Types of agreements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgreementType {
    ResourceSharing,
    ServiceProvision,
    GovernanceCoordination,
    CulturalExchange,
    TechnicalCollaboration,
    MutualAid,
    ConflictResolution,
}

/// Status of agreements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgreementStatus {
    Proposed,
    Negotiating,
    Signed,
    Active,
    Completed,
    Suspended,
    Terminated,
}

/// Performance metrics for agreements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub metric_name: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub measurement_date: u64,
}

/// Organization-specific economic behavior patterns
pub trait OrganizationalEconomics {
    /// Calculate mana regeneration rate for organization type
    fn calculate_mana_regeneration(&self, base_rate: f64, contribution_score: f64, capacity_score: f64) -> f64;
    
    /// Determine resource allocation priorities
    fn get_resource_allocation_priorities(&self) -> Vec<(String, f64)>;
    
    /// Calculate surplus distribution
    fn calculate_surplus_distribution(&self, surplus: u64, members: &[Did]) -> HashMap<Did, u64>;
    
    /// Evaluate contribution value
    fn evaluate_contribution_value(&self, contribution_type: &str, metrics: &HashMap<String, f64>) -> f64;
}

impl OrganizationalEconomics for OrganizationType {
    fn calculate_mana_regeneration(&self, base_rate: f64, contribution_score: f64, capacity_score: f64) -> f64 {
        match self {
            OrganizationType::Coop { economic_focus, production_capacity } => {
                // Coops get bonuses for production capacity and economic contribution
                let production_multiplier = production_capacity.current_utilization * 0.5 + 0.5;
                let economic_multiplier = match economic_focus {
                    EconomicFocus::Production { .. } => 1.3,
                    EconomicFocus::Services { .. } => 1.2,
                    EconomicFocus::ResourceSharing { .. } => 1.4,
                    EconomicFocus::Financial { .. } => 1.1,
                    EconomicFocus::MultiStakeholder { .. } => 1.25,
                };
                base_rate * contribution_score * capacity_score * production_multiplier * economic_multiplier
            }
            OrganizationType::Community { governance_model, .. } => {
                // Communities get bonuses for governance participation
                let governance_multiplier = match governance_model {
                    GovernanceModel::DirectDemocracy => 1.4,
                    GovernanceModel::Consensus { .. } => 1.5,
                    GovernanceModel::Sociocracy { .. } => 1.3,
                    GovernanceModel::LiquidDemocracy { .. } => 1.25,
                    _ => 1.2,
                };
                base_rate * contribution_score * capacity_score * governance_multiplier
            }
            OrganizationType::Federation { member_organizations, .. } => {
                // Federations get bonuses for coordination and bridge-building
                let federation_multiplier = 1.0 + (member_organizations.len() as f64 * 0.1).min(0.5);
                base_rate * contribution_score * capacity_score * federation_multiplier * 1.2
            }
        }
    }

    fn get_resource_allocation_priorities(&self) -> Vec<(String, f64)> {
        match self {
            OrganizationType::Coop { economic_focus, .. } => {
                match economic_focus {
                    EconomicFocus::Production { sectors } => {
                        let mut priorities = vec![("production_infrastructure".to_string(), 0.4)];
                        for sector in sectors {
                            priorities.push((format!("sector_{}", sector), 0.3 / sectors.len() as f64));
                        }
                        priorities.push(("member_welfare".to_string(), 0.3));
                        priorities
                    }
                    EconomicFocus::Services { .. } => {
                        vec![
                            ("service_capacity".to_string(), 0.35),
                            ("quality_improvement".to_string(), 0.25),
                            ("member_development".to_string(), 0.25),
                            ("infrastructure".to_string(), 0.15),
                        ]
                    }
                    _ => vec![("general_operations".to_string(), 1.0)],
                }
            }
            OrganizationType::Community { .. } => {
                vec![
                    ("basic_needs".to_string(), 0.4),
                    ("cultural_activities".to_string(), 0.25),
                    ("governance_infrastructure".to_string(), 0.2),
                    ("community_development".to_string(), 0.15),
                ]
            }
            OrganizationType::Federation { .. } => {
                vec![
                    ("coordination_infrastructure".to_string(), 0.3),
                    ("interoperability_development".to_string(), 0.25),
                    ("member_support".to_string(), 0.25),
                    ("external_relations".to_string(), 0.2),
                ]
            }
        }
    }

    fn calculate_surplus_distribution(&self, surplus: u64, members: &[Did]) -> HashMap<Did, u64> {
        let mut distribution = HashMap::new();
        
        match self {
            OrganizationType::Coop { .. } => {
                // Equal distribution with small reinvestment
                let reinvestment = surplus / 10; // 10% reinvestment
                let distributable = surplus - reinvestment;
                let per_member = distributable / members.len() as u64;
                
                for member in members {
                    distribution.insert(member.clone(), per_member);
                }
            }
            OrganizationType::Community { .. } => {
                // Needs-based distribution (simplified as equal for now)
                let per_member = surplus / members.len() as u64;
                for member in members {
                    distribution.insert(member.clone(), per_member);
                }
            }
            OrganizationType::Federation { .. } => {
                // Larger reinvestment for coordination activities
                let reinvestment = surplus / 4; // 25% reinvestment
                let distributable = surplus - reinvestment;
                let per_member = distributable / members.len() as u64;
                
                for member in members {
                    distribution.insert(member.clone(), per_member);
                }
            }
        }
        
        distribution
    }

    fn evaluate_contribution_value(&self, contribution_type: &str, metrics: &HashMap<String, f64>) -> f64 {
        let base_value = metrics.get("base_value").copied().unwrap_or(1.0);
        
        let multiplier = match self {
            OrganizationType::Coop { economic_focus, .. } => {
                match contribution_type {
                    "production" => match economic_focus {
                        EconomicFocus::Production { .. } => 1.5,
                        _ => 1.0,
                    },
                    "service_delivery" => match economic_focus {
                        EconomicFocus::Services { .. } => 1.4,
                        _ => 1.0,
                    },
                    "resource_coordination" => match economic_focus {
                        EconomicFocus::ResourceSharing { .. } => 1.6,
                        _ => 1.0,
                    },
                    _ => 1.0,
                }
            }
            OrganizationType::Community { .. } => {
                match contribution_type {
                    "governance_participation" => 1.5,
                    "cultural_activities" => 1.3,
                    "care_work" => 1.4,
                    "conflict_resolution" => 1.3,
                    _ => 1.0,
                }
            }
            OrganizationType::Federation { .. } => {
                match contribution_type {
                    "coordination" => 1.6,
                    "bridge_building" => 1.5,
                    "interoperability" => 1.4,
                    "external_relations" => 1.3,
                    _ => 1.0,
                }
            }
        };
        
        base_value * multiplier
    }
}

/// Organization registry for managing organizations
#[derive(Debug)]
pub struct OrganizationRegistry {
    organizations: HashMap<Did, Organization>,
    type_index: HashMap<String, Vec<Did>>, // Index by organization type
    scope_index: HashMap<NodeScope, Vec<Did>>, // Index by scope
}

impl OrganizationRegistry {
    /// Create a new organization registry
    pub fn new() -> Self {
        Self {
            organizations: HashMap::new(),
            type_index: HashMap::new(),
            scope_index: HashMap::new(),
        }
    }

    /// Register a new organization
    pub fn register_organization(&mut self, organization: Organization) -> Result<(), CommonError> {
        let org_id = organization.id.clone();
        let org_type = self.get_type_key(&organization.organization_type);
        let scope = organization.scope.clone();

        // Add to main registry
        self.organizations.insert(org_id.clone(), organization);

        // Update type index
        self.type_index.entry(org_type).or_default().push(org_id.clone());

        // Update scope index
        self.scope_index.entry(scope).or_default().push(org_id);

        Ok(())
    }

    /// Get organization by ID
    pub fn get_organization(&self, id: &Did) -> Option<&Organization> {
        self.organizations.get(id)
    }

    /// Get organizations by type
    pub fn get_organizations_by_type(&self, org_type: &str) -> Vec<&Organization> {
        self.type_index
            .get(org_type)
            .map(|ids| ids.iter().filter_map(|id| self.organizations.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get organizations in a scope
    pub fn get_organizations_by_scope(&self, scope: &NodeScope) -> Vec<&Organization> {
        self.scope_index
            .get(scope)
            .map(|ids| ids.iter().filter_map(|id| self.organizations.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find potential collaboration partners
    pub fn find_collaboration_partners(&self, org_id: &Did, collaboration_type: &str) -> Vec<&Organization> {
        let mut partners = Vec::new();
        
        if let Some(org) = self.get_organization(org_id) {
            for other_org in self.organizations.values() {
                if other_org.id != org.id {
                    // Simple matching logic - could be enhanced
                    if self.is_compatible_for_collaboration(org, other_org, collaboration_type) {
                        partners.push(other_org);
                    }
                }
            }
        }
        
        partners
    }

    /// Check if two organizations are compatible for collaboration
    fn is_compatible_for_collaboration(
        &self,
        org1: &Organization,
        org2: &Organization,
        collaboration_type: &str,
    ) -> bool {
        match collaboration_type {
            "economic" => {
                // Economic collaboration compatibility
                matches!(
                    (&org1.organization_type, &org2.organization_type),
                    (OrganizationType::Coop { .. }, OrganizationType::Coop { .. })
                )
            }
            "governance" => {
                // Governance collaboration compatibility
                matches!(
                    (&org1.organization_type, &org2.organization_type),
                    (OrganizationType::Community { .. }, OrganizationType::Community { .. })
                    | (OrganizationType::Community { .. }, OrganizationType::Federation { .. })
                    | (OrganizationType::Federation { .. }, OrganizationType::Community { .. })
                )
            }
            "cultural" => {
                // Cultural exchange compatibility
                matches!(
                    (&org1.organization_type, &org2.organization_type),
                    (OrganizationType::Community { .. }, _)
                    | (_, OrganizationType::Community { .. })
                )
            }
            _ => false,
        }
    }

    /// Get type key for indexing
    fn get_type_key(&self, org_type: &OrganizationType) -> String {
        match org_type {
            OrganizationType::Coop { economic_focus, .. } => {
                format!("coop_{}", match economic_focus {
                    EconomicFocus::Production { .. } => "production",
                    EconomicFocus::Services { .. } => "services",
                    EconomicFocus::ResourceSharing { .. } => "resource_sharing",
                    EconomicFocus::Financial { .. } => "financial",
                    EconomicFocus::MultiStakeholder { .. } => "multi_stakeholder",
                })
            }
            OrganizationType::Community { governance_model, .. } => {
                format!("community_{}", match governance_model {
                    GovernanceModel::DirectDemocracy => "direct_democracy",
                    GovernanceModel::RepresentativeDemocracy { .. } => "representative",
                    GovernanceModel::Consensus { .. } => "consensus",
                    GovernanceModel::Sociocracy { .. } => "sociocracy",
                    GovernanceModel::LiquidDemocracy { .. } => "liquid_democracy",
                    GovernanceModel::Custom { .. } => "custom",
                })
            }
            OrganizationType::Federation { .. } => "federation".to_string(),
        }
    }
}

impl Default for OrganizationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_organization_creation() {
        let coop = OrganizationType::Coop {
            economic_focus: EconomicFocus::Production { sectors: vec!["agriculture".to_string()] },
            production_capacity: ProductionCapacity {
                total_capacity: 1000,
                current_utilization: 0.8,
                capacity_by_resource: HashMap::new(),
                seasonal_variations: None,
            },
        };

        let org = Organization {
            id: Did::from_str("did:icn:coop:farm1").unwrap(),
            name: "Sunshine Farm Cooperative".to_string(),
            organization_type: coop,
            scope: NodeScope("test_scope".to_string()),
            created_at: SystemTimeProvider.unix_seconds(),
            member_count: 25,
            economic_policies: EconomicPolicies {
                mana_regeneration_policy: ManaRegenerationPolicy {
                    base_rate: 10.0,
                    contribution_multiplier: 1.5,
                    capacity_weight: 0.6,
                    organization_bonus: 0.2,
                    solidarity_bonus: 0.1,
                },
                resource_allocation_policy: ResourceAllocationPolicy::Mixed {
                    need_weight: 0.3,
                    contribution_weight: 0.4,
                    equality_weight: 0.3,
                },
                surplus_distribution_policy: SurplusDistributionPolicy::ContributionBasedDistribution {
                    contribution_metrics: vec!["labor_hours".to_string(), "expertise_sharing".to_string()],
                },
                contribution_recognition_policy: ContributionRecognitionPolicy {
                    recognition_types: vec![RecognitionType::Labor { skill_categories: vec!["farming".to_string()] }],
                    measurement_methods: vec![MeasurementMethod::TimeTracking],
                    reward_mechanisms: vec![RewardMechanism::ManaBonus { multiplier: 1.2 }],
                    peer_validation_required: true,
                },
            },
            relationships: Vec::new(),
            reputation_metrics: ReputationMetrics {
                reliability_score: 0.85,
                cooperation_score: 0.9,
                innovation_score: 0.75,
                sustainability_score: 0.95,
                transparency_score: 0.9,
                member_satisfaction_score: 0.88,
                external_reputation_score: 0.82,
            },
        };

        assert_eq!(org.name, "Sunshine Farm Cooperative");
        assert_eq!(org.member_count, 25);
    }

    #[test]
    fn test_mana_regeneration_calculation() {
        let coop = OrganizationType::Coop {
            economic_focus: EconomicFocus::Production { sectors: vec!["manufacturing".to_string()] },
            production_capacity: ProductionCapacity {
                total_capacity: 500,
                current_utilization: 0.9,
                capacity_by_resource: HashMap::new(),
                seasonal_variations: None,
            },
        };

        let regeneration = coop.calculate_mana_regeneration(10.0, 1.5, 1.2);
        assert!(regeneration > 10.0); // Should be higher than base rate
        assert!(regeneration < 30.0); // But not unreasonably high
    }

    #[test]
    fn test_organization_registry() {
        let mut registry = OrganizationRegistry::new();

        let coop = Organization {
            id: Did::from_str("did:icn:coop:test1").unwrap(),
            name: "Test Coop".to_string(),
            organization_type: OrganizationType::Coop {
                economic_focus: EconomicFocus::Services { service_types: vec!["consulting".to_string()] },
                production_capacity: ProductionCapacity {
                    total_capacity: 100,
                    current_utilization: 0.6,
                    capacity_by_resource: HashMap::new(),
                    seasonal_variations: None,
                },
            },
            scope: NodeScope("test_coop_scope".to_string()),
            created_at: SystemTimeProvider.unix_seconds(),
            member_count: 10,
            economic_policies: EconomicPolicies {
                mana_regeneration_policy: ManaRegenerationPolicy {
                    base_rate: 8.0,
                    contribution_multiplier: 1.3,
                    capacity_weight: 0.5,
                    organization_bonus: 0.15,
                    solidarity_bonus: 0.05,
                },
                resource_allocation_policy: ResourceAllocationPolicy::Equal { minimum_guarantee: Some(50) },
                surplus_distribution_policy: SurplusDistributionPolicy::EqualDistribution,
                contribution_recognition_policy: ContributionRecognitionPolicy {
                    recognition_types: vec![RecognitionType::Knowledge { expertise_areas: vec!["consulting".to_string()] }],
                    measurement_methods: vec![MeasurementMethod::PeerAssessment],
                    reward_mechanisms: vec![RewardMechanism::Recognition { recognition_forms: vec!["peer_thanks".to_string()] }],
                    peer_validation_required: false,
                },
            },
            relationships: Vec::new(),
            reputation_metrics: ReputationMetrics {
                reliability_score: 0.8,
                cooperation_score: 0.85,
                innovation_score: 0.9,
                sustainability_score: 0.7,
                transparency_score: 0.95,
                member_satisfaction_score: 0.9,
                external_reputation_score: 0.75,
            },
        };

        let org_id = coop.id.clone();
        registry.register_organization(coop).unwrap();

        assert!(registry.get_organization(&org_id).is_some());
        assert_eq!(registry.get_organizations_by_type("coop_services").len(), 1);
    }

    #[test]
    fn test_surplus_distribution() {
        let community = OrganizationType::Community {
            governance_model: GovernanceModel::DirectDemocracy,
            cultural_values: CulturalValues {
                core_principles: vec!["equality".to_string(), "sustainability".to_string()],
                conflict_resolution: ConflictResolutionModel::Mediation,
                inclusion_practices: InclusionPractices {
                    accessibility_measures: vec!["wheelchair_access".to_string()],
                    diversity_commitments: vec!["cultural_diversity".to_string()],
                    language_support: vec!["spanish".to_string(), "english".to_string()],
                    economic_inclusion: vec!["sliding_scale_fees".to_string()],
                },
                decision_making_culture: DecisionMakingCulture::Collaborative,
            },
        };

        let members = vec![
            Did::from_str("did:icn:member1").unwrap(),
            Did::from_str("did:icn:member2").unwrap(),
            Did::from_str("did:icn:member3").unwrap(),
        ];

        let distribution = community.calculate_surplus_distribution(300, &members);
        assert_eq!(distribution.len(), 3);
        assert_eq!(distribution[&members[0]], 100); // Equal distribution
    }
}