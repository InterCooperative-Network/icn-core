//! Multi-tiered Dispute Resolution System
//!
//! This module implements the built-in, multi-tiered system for interpreting and 
//! enforcing contract terms at every level, including restorative and transformative 
//! justice options.

use crate::social_contract::{GovernanceScope, SocialContractId};
use icn_common::{Cid, CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Unique identifier for a dispute
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisputeId(pub String);

impl std::fmt::Display for DisputeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Multi-tiered dispute resolution system
pub struct DisputeResolutionSystem {
    /// Active disputes
    disputes: HashMap<DisputeId, Dispute>,
    /// Resolution tiers configuration
    tiers: Vec<ResolutionTier>,
    /// Mediators by tier
    mediators: HashMap<usize, Vec<MediatorInfo>>,
    /// Resolution outcomes
    outcomes: HashMap<DisputeId, DisputeOutcome>,
    /// Resolution templates
    templates: HashMap<String, ResolutionTemplate>,
    /// Appeals registry
    appeals: HashMap<DisputeId, Vec<Appeal>>,
}

/// A dispute case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    /// Unique dispute identifier
    pub id: DisputeId,
    /// Contract this dispute relates to
    pub contract_id: SocialContractId,
    /// Parties involved in the dispute
    pub parties: Vec<DisputeParty>,
    /// Type of dispute
    pub dispute_type: DisputeType,
    /// Current status
    pub status: DisputeStatus,
    /// Current resolution tier
    pub current_tier: usize,
    /// Issue description
    pub description: String,
    /// Evidence provided
    pub evidence: Vec<Evidence>,
    /// Timeline of events
    pub timeline: Vec<DisputeEvent>,
    /// Filed timestamp
    pub filed_at: SystemTime,
    /// Assigned mediators
    pub assigned_mediators: Vec<Did>,
    /// Resolution deadline
    pub resolution_deadline: Option<SystemTime>,
    /// Governance scope
    pub scope: GovernanceScope,
}

/// Party involved in a dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeParty {
    /// Party's DID
    pub did: Did,
    /// Role in the dispute
    pub role: PartyRole,
    /// Contact preferences
    pub contact_preferences: ContactPreferences,
    /// Representation (if any)
    pub representation: Option<Did>,
}

/// Role of a party in dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartyRole {
    /// Primary complainant
    Complainant,
    /// Responding party
    Respondent,
    /// Affected third party
    AffectedParty,
    /// Expert witness
    Witness,
    /// Community representative
    CommunityRep,
}

/// Contact preferences for dispute communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPreferences {
    /// Preferred communication method
    pub method: CommunicationMethod,
    /// Language preference
    pub language: String,
    /// Accessibility requirements
    pub accessibility: Vec<String>,
    /// Time zone
    pub timezone: String,
}

/// Communication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationMethod {
    /// Encrypted messaging
    SecureMessage,
    /// Video conference
    VideoCall,
    /// Voice call
    VoiceCall,
    /// In-person meeting
    InPerson,
    /// Written correspondence
    Written,
}

/// Type of dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeType {
    /// Contract interpretation dispute
    ContractInterpretation,
    /// Breach of contract
    ContractBreach,
    /// Resource allocation dispute
    ResourceDispute,
    /// Governance process dispute
    GovernanceDispute,
    /// Rights violation
    RightsViolation,
    /// Responsibility failure
    ResponsibilityFailure,
    /// Economic misconduct
    EconomicMisconduct,
    /// Technical disagreement
    TechnicalDispute,
    /// Cultural/social conflict
    SocialConflict,
    /// Custom dispute type
    Custom(String),
}

/// Current status of a dispute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisputeStatus {
    /// Newly filed
    Filed,
    /// Under initial review
    UnderReview,
    /// Assigned to mediator
    Assigned,
    /// In mediation process
    InMediation,
    /// Escalated to higher tier
    Escalated,
    /// Resolved
    Resolved,
    /// Closed without resolution
    Closed,
    /// Appealed
    Appealed,
    /// Withdrawn by complainant
    Withdrawn,
}

/// Evidence in a dispute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Evidence identifier
    pub id: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Description
    pub description: String,
    /// Evidence data stored in DAG
    pub content_cid: Cid,
    /// Submitted by
    pub submitted_by: Did,
    /// Submission timestamp
    pub submitted_at: SystemTime,
    /// Verification status
    pub verification_status: VerificationStatus,
}

/// Type of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Documentation
    Document,
    /// Audio/video recording
    Recording,
    /// Transaction records
    TransactionRecord,
    /// Witness statement
    WitnessStatement,
    /// Technical logs
    TechnicalLog,
    /// Communication records
    Communication,
    /// Expert analysis
    ExpertAnalysis,
    /// Other evidence type
    Other(String),
}

/// Verification status of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Pending verification
    Pending,
    /// Verified authentic
    Verified,
    /// Disputed authenticity
    Disputed,
    /// Rejected as invalid
    Rejected,
}

/// Event in dispute timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeEvent {
    /// Event type
    pub event_type: EventType,
    /// Description
    pub description: String,
    /// Actor who caused the event
    pub actor: Option<Did>,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Associated data
    pub metadata: HashMap<String, String>,
}

/// Type of dispute event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Dispute filed
    Filed,
    /// Mediator assigned
    MediatorAssigned,
    /// Evidence submitted
    EvidenceSubmitted,
    /// Mediation session held
    MediationSession,
    /// Status changed
    StatusChanged,
    /// Resolution proposed
    ResolutionProposed,
    /// Resolution accepted
    ResolutionAccepted,
    /// Resolution rejected
    ResolutionRejected,
    /// Appeal filed
    AppealFiled,
    /// Escalated to higher tier
    Escalated,
}

/// Resolution tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionTier {
    /// Tier level (0 = first tier)
    pub level: usize,
    /// Tier name
    pub name: String,
    /// Description
    pub description: String,
    /// Types of disputes this tier handles
    pub handles_types: Vec<DisputeType>,
    /// Resolution methods available
    pub resolution_methods: Vec<ResolutionMethod>,
    /// Maximum resolution timeframe
    pub max_duration: u64,
    /// Escalation criteria
    pub escalation_criteria: EscalationCriteria,
    /// Required mediator qualifications
    pub mediator_requirements: MediatorRequirements,
}

/// Resolution method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionMethod {
    /// Direct negotiation between parties
    DirectNegotiation,
    /// Facilitated mediation
    Mediation,
    /// Binding arbitration
    Arbitration,
    /// Community circle process
    CommunityCircle,
    /// Restorative justice process
    RestorativeJustice,
    /// Transformative justice process
    TransformativeJustice,
    /// Technical evaluation
    TechnicalEvaluation,
    /// Peer review
    PeerReview,
    /// Expert panel
    ExpertPanel,
}

/// Criteria for escalation to next tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationCriteria {
    /// Time limit exceeded
    pub time_limit_exceeded: bool,
    /// Parties cannot agree
    pub impasse_reached: bool,
    /// Complex legal/technical issues
    pub complexity_threshold: f64,
    /// Significant impact scope
    pub impact_threshold: ImpactThreshold,
    /// Appeal requested
    pub appeal_requested: bool,
}

/// Impact threshold for escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactThreshold {
    /// Number of people affected
    pub affected_people: usize,
    /// Economic impact
    pub economic_impact: f64,
    /// Governance impact
    pub governance_impact: GovernanceImpactLevel,
}

/// Level of governance impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceImpactLevel {
    /// Local impact only
    Local,
    /// Regional impact
    Regional,
    /// Network-wide impact
    NetworkWide,
    /// Precedent-setting
    PrecedentSetting,
}

/// Mediator qualification requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediatorRequirements {
    /// Minimum experience level
    pub min_experience: ExperienceLevel,
    /// Required certifications
    pub required_certifications: Vec<String>,
    /// Required skills
    pub required_skills: Vec<String>,
    /// Language requirements
    pub language_requirements: Vec<String>,
    /// Cultural competency
    pub cultural_requirements: Vec<String>,
    /// Conflict of interest restrictions
    pub conflict_restrictions: Vec<String>,
}

/// Experience level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    /// New mediator
    Novice,
    /// Some experience
    Intermediate,
    /// Experienced mediator
    Experienced,
    /// Expert mediator
    Expert,
    /// Master mediator
    Master,
}

/// Information about a mediator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediatorInfo {
    /// Mediator's DID
    pub did: Did,
    /// Display name
    pub name: String,
    /// Experience level
    pub experience: ExperienceLevel,
    /// Certifications
    pub certifications: Vec<String>,
    /// Skills and specializations
    pub skills: Vec<String>,
    /// Languages spoken
    pub languages: Vec<String>,
    /// Cultural backgrounds
    pub cultural_backgrounds: Vec<String>,
    /// Success rate
    pub success_rate: f64,
    /// Availability
    pub availability: AvailabilityInfo,
    /// Current case load
    pub case_load: usize,
    /// Maximum cases
    pub max_cases: usize,
}

/// Mediator availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityInfo {
    /// Currently available
    pub available: bool,
    /// Next available date
    pub next_available: Option<SystemTime>,
    /// Timezone
    pub timezone: String,
    /// Preferred working hours
    pub working_hours: WorkingHours,
}

/// Working hours specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    /// Start time (24-hour format)
    pub start_hour: u8,
    /// End time (24-hour format)
    pub end_hour: u8,
    /// Days of week (0 = Sunday)
    pub days: Vec<u8>,
}

/// Outcome of a dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeOutcome {
    /// Dispute this outcome applies to
    pub dispute_id: DisputeId,
    /// Resolution method used
    pub resolution_method: ResolutionMethod,
    /// Final decision
    pub decision: Decision,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Actions required
    pub required_actions: Vec<RequiredAction>,
    /// Compensation/restitution
    pub restitution: Vec<Restitution>,
    /// Preventive measures
    pub preventive_measures: Vec<PreventiveMeasure>,
    /// Implementation timeline
    pub implementation_timeline: ImplementationTimeline,
    /// Mediator(s) who reached this outcome
    pub mediators: Vec<Did>,
    /// Outcome timestamp
    pub decided_at: SystemTime,
    /// Appeal rights
    pub appeal_rights: AppealRights,
}

/// Resolution decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Complainant's position upheld
    UpheldComplainant,
    /// Respondent's position upheld
    UpheldRespondent,
    /// Partial resolution favoring complainant
    PartialComplainant,
    /// Partial resolution favoring respondent
    PartialRespondent,
    /// Compromise solution
    Compromise,
    /// No fault found
    NoFault,
    /// Insufficient evidence
    InsufficientEvidence,
    /// Referred to higher authority
    Referred,
}

/// Required action from dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredAction {
    /// Action description
    pub description: String,
    /// Responsible party
    pub responsible_party: Did,
    /// Deadline for completion
    pub deadline: SystemTime,
    /// Verification method
    pub verification_method: String,
    /// Consequences of non-compliance
    pub non_compliance_consequences: Vec<String>,
}

/// Restitution or compensation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restitution {
    /// Type of restitution
    pub restitution_type: RestitutionType,
    /// Amount (if applicable)
    pub amount: Option<f64>,
    /// Description
    pub description: String,
    /// From party
    pub from_party: Did,
    /// To party
    pub to_party: Did,
    /// Due date
    pub due_date: SystemTime,
}

/// Type of restitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestitutionType {
    /// Monetary compensation
    Monetary,
    /// Service provision
    Service,
    /// Resource allocation
    Resource,
    /// Public acknowledgment
    PublicAcknowledgment,
    /// Private apology
    PrivateApology,
    /// Community service
    CommunityService,
    /// Education/training
    Education,
    /// Other form
    Other(String),
}

/// Preventive measure to avoid future disputes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreventiveMeasure {
    /// Measure description
    pub description: String,
    /// Responsible parties
    pub responsible_parties: Vec<Did>,
    /// Implementation deadline
    pub deadline: SystemTime,
    /// Monitoring method
    pub monitoring_method: String,
    /// Success metrics
    pub success_metrics: Vec<String>,
}

/// Implementation timeline for resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationTimeline {
    /// Phases of implementation
    pub phases: Vec<ImplementationPhase>,
    /// Overall deadline
    pub overall_deadline: SystemTime,
    /// Monitoring checkpoints
    pub checkpoints: Vec<Checkpoint>,
}

/// Phase of implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    /// Phase name
    pub name: String,
    /// Description
    pub description: String,
    /// Start date
    pub start_date: SystemTime,
    /// End date
    pub end_date: SystemTime,
    /// Responsible parties
    pub responsible_parties: Vec<Did>,
    /// Deliverables
    pub deliverables: Vec<String>,
}

/// Monitoring checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint name
    pub name: String,
    /// Date
    pub date: SystemTime,
    /// What to verify
    pub verification_items: Vec<String>,
    /// Responsible for verification
    pub verifier: Did,
}

/// Appeal rights information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppealRights {
    /// Whether appeal is allowed
    pub appeal_allowed: bool,
    /// Appeal deadline
    pub appeal_deadline: Option<SystemTime>,
    /// Next tier for appeal
    pub next_tier: Option<usize>,
    /// Appeal requirements
    pub appeal_requirements: Vec<String>,
    /// Appeal fee (if any)
    pub appeal_fee: Option<f64>,
}

/// Appeal of a dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appeal {
    /// Appeal identifier
    pub id: String,
    /// Original dispute
    pub dispute_id: DisputeId,
    /// Appealing party
    pub appellant: Did,
    /// Grounds for appeal
    pub grounds: AppealGrounds,
    /// Supporting evidence
    pub evidence: Vec<Evidence>,
    /// Appeal status
    pub status: AppealStatus,
    /// Filed timestamp
    pub filed_at: SystemTime,
    /// New evidence allowed
    pub new_evidence_allowed: bool,
}

/// Grounds for appeal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppealGrounds {
    /// Procedural error in original resolution
    ProceduralError,
    /// New evidence discovered
    NewEvidence,
    /// Bias or conflict of interest
    Bias,
    /// Incorrect application of rules
    IncorrectRuleApplication,
    /// Disproportionate outcome
    DisproportionateOutcome,
    /// Failure to consider relevant factors
    FailureToConsider,
    /// Other grounds
    Other(String),
}

/// Status of an appeal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppealStatus {
    /// Filed and pending review
    Pending,
    /// Under review
    UnderReview,
    /// Accepted for full hearing
    Accepted,
    /// Rejected
    Rejected,
    /// Hearing scheduled
    Scheduled,
    /// Decision pending
    DecisionPending,
    /// Appeal upheld
    Upheld,
    /// Appeal denied
    Denied,
}

/// Resolution template for common dispute types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionTemplate {
    /// Template name
    pub name: String,
    /// Applicable dispute types
    pub applies_to: Vec<DisputeType>,
    /// Standard process steps
    pub process_steps: Vec<ProcessStep>,
    /// Common resolutions
    pub common_resolutions: Vec<String>,
    /// Typical timeline
    pub typical_timeline: u64,
    /// Required evidence types
    pub required_evidence: Vec<EvidenceType>,
    /// Success rate with this template
    pub success_rate: f64,
}

/// Process step in resolution template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    /// Step name
    pub name: String,
    /// Description
    pub description: String,
    /// Estimated duration
    pub duration: u64,
    /// Required participants
    pub participants: Vec<ParticipantRole>,
    /// Outputs expected
    pub outputs: Vec<String>,
}

/// Role of participant in process step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    /// All parties
    AllParties,
    /// Complainant only
    Complainant,
    /// Respondent only
    Respondent,
    /// Mediator
    Mediator,
    /// Expert witness
    Expert,
    /// Community representative
    Community,
}

/// Error types for dispute resolution
#[derive(Debug, thiserror::Error)]
pub enum DisputeResolutionError {
    #[error("Dispute not found: {0}")]
    DisputeNotFound(DisputeId),

    #[error("Invalid dispute status: {0:?}")]
    InvalidStatus(DisputeStatus),

    #[error("Mediator not available: {0}")]
    MediatorNotAvailable(Did),

    #[error("Insufficient evidence: {0}")]
    InsufficientEvidence(String),

    #[error("Escalation not allowed: {0}")]
    EscalationNotAllowed(String),

    #[error("Appeal deadline passed")]
    AppealDeadlinePassed,

    #[error("Resolution method not supported: {0:?}")]
    UnsupportedMethod(ResolutionMethod),

    #[error("Common error: {0}")]
    Common(#[from] CommonError),
}

impl DisputeResolutionSystem {
    /// Create new dispute resolution system
    pub fn new() -> Self {
        Self {
            disputes: HashMap::new(),
            tiers: Vec::new(),
            mediators: HashMap::new(),
            outcomes: HashMap::new(),
            templates: HashMap::new(),
            appeals: HashMap::new(),
        }
    }

    /// Initialize with standard resolution tiers
    pub fn with_standard_tiers() -> Self {
        let mut system = Self::new();
        
        // Tier 0: Direct negotiation and mediation
        system.add_tier(ResolutionTier {
            level: 0,
            name: "Local Mediation".to_string(),
            description: "Direct negotiation and local mediation".to_string(),
            handles_types: vec![
                DisputeType::ContractInterpretation,
                DisputeType::ResourceDispute,
                DisputeType::SocialConflict,
            ],
            resolution_methods: vec![
                ResolutionMethod::DirectNegotiation,
                ResolutionMethod::Mediation,
                ResolutionMethod::CommunityCircle,
            ],
            max_duration: 2592000, // 30 days
            escalation_criteria: EscalationCriteria {
                time_limit_exceeded: true,
                impasse_reached: true,
                complexity_threshold: 0.3,
                impact_threshold: ImpactThreshold {
                    affected_people: 10,
                    economic_impact: 1000.0,
                    governance_impact: GovernanceImpactLevel::Local,
                },
                appeal_requested: true,
            },
            mediator_requirements: MediatorRequirements {
                min_experience: ExperienceLevel::Intermediate,
                required_certifications: vec!["basic_mediation".to_string()],
                required_skills: vec!["communication".to_string(), "conflict_resolution".to_string()],
                language_requirements: vec!["local_language".to_string()],
                cultural_requirements: vec!["local_culture".to_string()],
                conflict_restrictions: vec!["no_direct_involvement".to_string()],
            },
        });

        // Tier 1: Expert mediation and arbitration
        system.add_tier(ResolutionTier {
            level: 1,
            name: "Expert Mediation".to_string(),
            description: "Expert mediation and binding arbitration".to_string(),
            handles_types: vec![
                DisputeType::ContractBreach,
                DisputeType::GovernanceDispute,
                DisputeType::EconomicMisconduct,
                DisputeType::TechnicalDispute,
            ],
            resolution_methods: vec![
                ResolutionMethod::Arbitration,
                ResolutionMethod::ExpertPanel,
                ResolutionMethod::TechnicalEvaluation,
                ResolutionMethod::RestorativeJustice,
            ],
            max_duration: 5184000, // 60 days
            escalation_criteria: EscalationCriteria {
                time_limit_exceeded: true,
                impasse_reached: true,
                complexity_threshold: 0.7,
                impact_threshold: ImpactThreshold {
                    affected_people: 100,
                    economic_impact: 10000.0,
                    governance_impact: GovernanceImpactLevel::Regional,
                },
                appeal_requested: true,
            },
            mediator_requirements: MediatorRequirements {
                min_experience: ExperienceLevel::Experienced,
                required_certifications: vec!["advanced_mediation".to_string(), "arbitration".to_string()],
                required_skills: vec!["legal_knowledge".to_string(), "technical_expertise".to_string()],
                language_requirements: vec!["multi_lingual".to_string()],
                cultural_requirements: vec!["cross_cultural".to_string()],
                conflict_restrictions: vec!["no_financial_interest".to_string()],
            },
        });

        // Tier 2: Network-wide resolution
        system.add_tier(ResolutionTier {
            level: 2,
            name: "Network Tribunal".to_string(),
            description: "Network-wide resolution for complex cases".to_string(),
            handles_types: vec![
                DisputeType::RightsViolation,
                DisputeType::ResponsibilityFailure,
                DisputeType::GovernanceDispute,
            ],
            resolution_methods: vec![
                ResolutionMethod::ExpertPanel,
                ResolutionMethod::TransformativeJustice,
                ResolutionMethod::PeerReview,
            ],
            max_duration: 7776000, // 90 days
            escalation_criteria: EscalationCriteria {
                time_limit_exceeded: false, // Final tier
                impasse_reached: false,
                complexity_threshold: 1.0,
                impact_threshold: ImpactThreshold {
                    affected_people: 1000,
                    economic_impact: 100000.0,
                    governance_impact: GovernanceImpactLevel::NetworkWide,
                },
                appeal_requested: false, // No further appeals
            },
            mediator_requirements: MediatorRequirements {
                min_experience: ExperienceLevel::Master,
                required_certifications: vec!["master_mediation".to_string(), "network_governance".to_string()],
                required_skills: vec!["constitutional_law".to_string(), "systems_thinking".to_string()],
                language_requirements: vec!["international".to_string()],
                cultural_requirements: vec!["global_perspective".to_string()],
                conflict_restrictions: vec!["complete_independence".to_string()],
            },
        });

        system
    }

    /// Add a resolution tier
    pub fn add_tier(&mut self, tier: ResolutionTier) {
        self.tiers.push(tier);
        self.tiers.sort_by_key(|t| t.level);
    }

    /// File a new dispute
    pub fn file_dispute(
        &mut self,
        contract_id: SocialContractId,
        complainant: Did,
        respondent: Did,
        dispute_type: DisputeType,
        description: String,
        scope: GovernanceScope,
    ) -> Result<DisputeId, DisputeResolutionError> {
        let dispute_id = DisputeId(format!(
            "dispute_{}_{}_{}",
            contract_id.0,
            complainant.to_string().chars().take(8).collect::<String>(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));

        let dispute = Dispute {
            id: dispute_id.clone(),
            contract_id,
            parties: vec![
                DisputeParty {
                    did: complainant.clone(),
                    role: PartyRole::Complainant,
                    contact_preferences: ContactPreferences {
                        method: CommunicationMethod::SecureMessage,
                        language: "en".to_string(),
                        accessibility: Vec::new(),
                        timezone: "UTC".to_string(),
                    },
                    representation: None,
                },
                DisputeParty {
                    did: respondent,
                    role: PartyRole::Respondent,
                    contact_preferences: ContactPreferences {
                        method: CommunicationMethod::SecureMessage,
                        language: "en".to_string(),
                        accessibility: Vec::new(),
                        timezone: "UTC".to_string(),
                    },
                    representation: None,
                },
            ],
            dispute_type,
            status: DisputeStatus::Filed,
            current_tier: 0,
            description,
            evidence: Vec::new(),
            timeline: vec![DisputeEvent {
                event_type: EventType::Filed,
                description: "Dispute filed".to_string(),
                actor: Some(complainant),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            }],
            filed_at: SystemTime::now(),
            assigned_mediators: Vec::new(),
            resolution_deadline: None,
            scope,
        };

        self.disputes.insert(dispute_id.clone(), dispute);
        Ok(dispute_id)
    }

    /// Assign mediator to dispute
    pub fn assign_mediator(
        &mut self,
        dispute_id: &DisputeId,
        mediator: Did,
    ) -> Result<(), DisputeResolutionError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or_else(|| DisputeResolutionError::DisputeNotFound(dispute_id.clone()))?;

        // Check mediator availability
        let empty_vec = Vec::new();
        let tier_mediators = self.mediators.get(&dispute.current_tier).unwrap_or(&empty_vec);
        let mediator_info = tier_mediators.iter()
            .find(|m| m.did == mediator)
            .ok_or_else(|| DisputeResolutionError::MediatorNotAvailable(mediator.clone()))?;

        if !mediator_info.availability.available || mediator_info.case_load >= mediator_info.max_cases {
            return Err(DisputeResolutionError::MediatorNotAvailable(mediator));
        }

        dispute.assigned_mediators.push(mediator.clone());
        dispute.status = DisputeStatus::Assigned;
        
        // Set resolution deadline based on tier
        if let Some(tier) = self.tiers.iter().find(|t| t.level == dispute.current_tier) {
            dispute.resolution_deadline = Some(
                SystemTime::now() + std::time::Duration::from_secs(tier.max_duration)
            );
        }

        // Add timeline event
        dispute.timeline.push(DisputeEvent {
            event_type: EventType::MediatorAssigned,
            description: format!("Mediator {} assigned", mediator),
            actor: None,
            timestamp: SystemTime::now(),
            metadata: [("mediator".to_string(), mediator.to_string())].into_iter().collect(),
        });

        Ok(())
    }

    /// Submit evidence for a dispute
    pub fn submit_evidence(
        &mut self,
        dispute_id: &DisputeId,
        evidence: Evidence,
    ) -> Result<(), DisputeResolutionError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or_else(|| DisputeResolutionError::DisputeNotFound(dispute_id.clone()))?;

        dispute.evidence.push(evidence.clone());
        dispute.timeline.push(DisputeEvent {
            event_type: EventType::EvidenceSubmitted,
            description: format!("Evidence submitted: {}", evidence.description),
            actor: Some(evidence.submitted_by),
            timestamp: SystemTime::now(),
            metadata: [("evidence_id".to_string(), evidence.id)].into_iter().collect(),
        });

        Ok(())
    }

    /// Escalate dispute to next tier
    pub fn escalate_dispute(
        &mut self,
        dispute_id: &DisputeId,
        reason: String,
    ) -> Result<(), DisputeResolutionError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or_else(|| DisputeResolutionError::DisputeNotFound(dispute_id.clone()))?;

        // Check if escalation is allowed
        if dispute.current_tier + 1 >= self.tiers.len() {
            return Err(DisputeResolutionError::EscalationNotAllowed(
                "Already at highest tier".to_string()
            ));
        }

        let current_tier = &self.tiers[dispute.current_tier];
        
        // Check escalation criteria (simplified check)
        if !current_tier.escalation_criteria.appeal_requested {
            return Err(DisputeResolutionError::EscalationNotAllowed(
                "Escalation not allowed for this tier".to_string()
            ));
        }

        dispute.current_tier += 1;
        dispute.status = DisputeStatus::Escalated;
        dispute.assigned_mediators.clear(); // Will need new mediators

        dispute.timeline.push(DisputeEvent {
            event_type: EventType::Escalated,
            description: format!("Escalated to tier {}: {}", dispute.current_tier, reason),
            actor: None,
            timestamp: SystemTime::now(),
            metadata: [
                ("reason".to_string(), reason),
                ("new_tier".to_string(), dispute.current_tier.to_string()),
            ].into_iter().collect(),
        });

        Ok(())
    }

    /// Resolve dispute with outcome
    pub fn resolve_dispute(
        &mut self,
        dispute_id: &DisputeId,
        outcome: DisputeOutcome,
    ) -> Result<(), DisputeResolutionError> {
        let dispute = self.disputes.get_mut(dispute_id)
            .ok_or_else(|| DisputeResolutionError::DisputeNotFound(dispute_id.clone()))?;

        dispute.status = DisputeStatus::Resolved;
        dispute.timeline.push(DisputeEvent {
            event_type: EventType::ResolutionAccepted,
            description: "Dispute resolved".to_string(),
            actor: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        });

        self.outcomes.insert(dispute_id.clone(), outcome);
        Ok(())
    }

    /// File an appeal
    pub fn file_appeal(
        &mut self,
        dispute_id: &DisputeId,
        appellant: Did,
        grounds: AppealGrounds,
        evidence: Vec<Evidence>,
    ) -> Result<String, DisputeResolutionError> {
        // Check if dispute exists and has outcome
        if !self.disputes.contains_key(dispute_id) {
            return Err(DisputeResolutionError::DisputeNotFound(dispute_id.clone()));
        }

        let outcome = self.outcomes.get(dispute_id)
            .ok_or_else(|| DisputeResolutionError::Common(
                CommonError::InvalidInputError("No resolution outcome found".to_string())
            ))?;

        // Check appeal rights
        if !outcome.appeal_rights.appeal_allowed {
            return Err(DisputeResolutionError::Common(
                CommonError::InvalidInputError("Appeals not allowed for this resolution".to_string())
            ));
        }

        if let Some(deadline) = outcome.appeal_rights.appeal_deadline {
            if SystemTime::now() > deadline {
                return Err(DisputeResolutionError::AppealDeadlinePassed);
            }
        }

        let appeal_id = format!("appeal_{}_{}", dispute_id.0, chrono::Utc::now().timestamp());
        let appeal = Appeal {
            id: appeal_id.clone(),
            dispute_id: dispute_id.clone(),
            appellant: appellant.clone(),
            grounds,
            evidence,
            status: AppealStatus::Pending,
            filed_at: SystemTime::now(),
            new_evidence_allowed: true,
        };

        self.appeals.entry(dispute_id.clone()).or_default().push(appeal);

        // Update dispute status
        if let Some(dispute) = self.disputes.get_mut(dispute_id) {
            dispute.status = DisputeStatus::Appealed;
            dispute.timeline.push(DisputeEvent {
                event_type: EventType::AppealFiled,
                description: "Appeal filed".to_string(),
                actor: Some(appellant),
                timestamp: SystemTime::now(),
                metadata: [("appeal_id".to_string(), appeal_id.clone())].into_iter().collect(),
            });
        }

        Ok(appeal_id)
    }

    /// Get dispute by ID
    pub fn get_dispute(&self, dispute_id: &DisputeId) -> Option<&Dispute> {
        self.disputes.get(dispute_id)
    }

    /// Get disputes by status
    pub fn get_disputes_by_status(&self, status: DisputeStatus) -> Vec<&Dispute> {
        self.disputes.values()
            .filter(|d| d.status == status)
            .collect()
    }

    /// Get available mediators for a tier
    pub fn get_available_mediators(&self, tier: usize) -> Vec<&MediatorInfo> {
        self.mediators.get(&tier)
            .map(|mediators| {
                mediators.iter()
                    .filter(|m| m.availability.available && m.case_load < m.max_cases)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add mediator to system
    pub fn add_mediator(&mut self, tier: usize, mediator: MediatorInfo) {
        self.mediators.entry(tier).or_default().push(mediator);
    }

    /// Generate dispute ID
    pub fn generate_dispute_id(contract_id: &SocialContractId, complainant: &Did) -> DisputeId {
        DisputeId(format!(
            "dispute_{}_{}_{}",
            contract_id.0,
            complainant.to_string().chars().take(8).collect::<String>(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ))
    }
}

impl Default for DisputeResolutionSystem {
    fn default() -> Self {
        Self::with_standard_tiers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispute_resolution_system_creation() {
        let system = DisputeResolutionSystem::with_standard_tiers();
        assert_eq!(system.tiers.len(), 3);
        assert_eq!(system.tiers[0].level, 0);
        assert_eq!(system.tiers[1].level, 1);
        assert_eq!(system.tiers[2].level, 2);
    }

    #[test]
    fn test_dispute_filing() {
        let mut system = DisputeResolutionSystem::with_standard_tiers();
        
        let contract_id = SocialContractId("test_contract".to_string());
        let complainant = Did::new("test", "complainant");
        let respondent = Did::new("test", "respondent");

        let dispute_id = system.file_dispute(
            contract_id,
            complainant,
            respondent,
            DisputeType::ContractInterpretation,
            "Disagreement over contract terms".to_string(),
            GovernanceScope::Local,
        );

        assert!(dispute_id.is_ok());
        let dispute_id = dispute_id.unwrap();
        
        let dispute = system.get_dispute(&dispute_id);
        assert!(dispute.is_some());
        
        let dispute = dispute.unwrap();
        assert_eq!(dispute.status, DisputeStatus::Filed);
        assert_eq!(dispute.parties.len(), 2);
        assert_eq!(dispute.current_tier, 0);
    }

    #[test]
    fn test_mediator_assignment() {
        let mut system = DisputeResolutionSystem::with_standard_tiers();
        
        // Add a mediator
        let mediator = MediatorInfo {
            did: Did::new("test", "mediator"),
            name: "Test Mediator".to_string(),
            experience: ExperienceLevel::Experienced,
            certifications: vec!["basic_mediation".to_string()],
            skills: vec!["communication".to_string()],
            languages: vec!["en".to_string()],
            cultural_backgrounds: vec!["western".to_string()],
            success_rate: 0.85,
            availability: AvailabilityInfo {
                available: true,
                next_available: None,
                timezone: "UTC".to_string(),
                working_hours: WorkingHours {
                    start_hour: 9,
                    end_hour: 17,
                    days: vec![1, 2, 3, 4, 5],
                },
            },
            case_load: 0,
            max_cases: 10,
        };
        
        system.add_mediator(0, mediator.clone());
        
        // File a dispute
        let dispute_id = system.file_dispute(
            SocialContractId("test_contract".to_string()),
            Did::new("test", "complainant"),
            Did::new("test", "respondent"),
            DisputeType::ContractInterpretation,
            "Test dispute".to_string(),
            GovernanceScope::Local,
        ).unwrap();

        // Assign mediator
        let result = system.assign_mediator(&dispute_id, mediator.did.clone());
        assert!(result.is_ok());
        
        let dispute = system.get_dispute(&dispute_id).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Assigned);
        assert_eq!(dispute.assigned_mediators.len(), 1);
        assert_eq!(dispute.assigned_mediators[0], mediator.did);
    }

    #[test]
    fn test_evidence_submission() {
        let mut system = DisputeResolutionSystem::with_standard_tiers();
        
        let dispute_id = system.file_dispute(
            SocialContractId("test_contract".to_string()),
            Did::new("test", "complainant"),
            Did::new("test", "respondent"),
            DisputeType::ContractInterpretation,
            "Test dispute".to_string(),
            GovernanceScope::Local,
        ).unwrap();

        let evidence = Evidence {
            id: "evidence_001".to_string(),
            evidence_type: EvidenceType::Document,
            description: "Contract document".to_string(),
            content_cid: Cid::new_v1_sha256(0x55, b"evidence_data"),
            submitted_by: Did::new("test", "complainant"),
            submitted_at: SystemTime::now(),
            verification_status: VerificationStatus::Pending,
        };

        let result = system.submit_evidence(&dispute_id, evidence);
        assert!(result.is_ok());
        
        let dispute = system.get_dispute(&dispute_id).unwrap();
        assert_eq!(dispute.evidence.len(), 1);
        assert_eq!(dispute.evidence[0].id, "evidence_001");
    }

    #[test]
    fn test_dispute_escalation() {
        let mut system = DisputeResolutionSystem::with_standard_tiers();
        
        let dispute_id = system.file_dispute(
            SocialContractId("test_contract".to_string()),
            Did::new("test", "complainant"),
            Did::new("test", "respondent"),
            DisputeType::ContractInterpretation,
            "Test dispute".to_string(),
            GovernanceScope::Local,
        ).unwrap();

        let result = system.escalate_dispute(&dispute_id, "Complex legal issues".to_string());
        assert!(result.is_ok());
        
        let dispute = system.get_dispute(&dispute_id).unwrap();
        assert_eq!(dispute.current_tier, 1);
        assert_eq!(dispute.status, DisputeStatus::Escalated);
    }

    #[test]
    fn test_dispute_id_generation() {
        let contract_id = SocialContractId("test_contract".to_string());
        let complainant = Did::new("test", "complainant");
        
        let dispute_id = DisputeResolutionSystem::generate_dispute_id(&contract_id, &complainant);
        assert!(dispute_id.0.starts_with("dispute_test_contract"));
    }
}