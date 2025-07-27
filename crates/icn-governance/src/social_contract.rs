//! Social Contract System
//!
//! This module implements the literal, enforceable social contracts for ICN governance.
//! Social contracts are versioned, codified CCL contracts that define rights, 
//! responsibilities, resource flows, and governance at all levels.

use crate::{ProposalId, ProposalType, ProposalSubmission, VoteOption};
use icn_common::{Cid, CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Unique identifier for a social contract
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SocialContractId(pub String);

impl std::fmt::Display for SocialContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Version identifier for social contracts
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ContractVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn initial() -> Self {
        Self::new(1, 0, 0)
    }

    pub fn increment_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    pub fn increment_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn increment_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl std::fmt::Display for ContractVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Governance scope for social contracts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceScope {
    /// Local co-op/community level
    Local,
    /// Regional federation level
    Regional,
    /// Global network level
    Global,
    /// Custom scope
    Custom(String),
}

impl GovernanceScope {
    pub fn as_str(&self) -> &str {
        match self {
            GovernanceScope::Local => "local",
            GovernanceScope::Regional => "regional", 
            GovernanceScope::Global => "global",
            GovernanceScope::Custom(name) => name,
        }
    }
}

/// Status of a social contract
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialContractStatus {
    /// Draft - being developed
    Draft,
    /// Under deliberation
    Deliberation,
    /// Active voting
    Voting,
    /// Ratified and active
    Active,
    /// Deprecated but still valid for existing commitments
    Deprecated,
    /// Superseded by newer version
    Superseded { successor: SocialContractId },
    /// Revoked/invalid
    Revoked,
}

/// Type of amendment to a social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmendmentType {
    /// Minor clarification or bug fix
    Clarification,
    /// Functional change that doesn't affect core structure
    Functional,
    /// Major structural change
    Structural,
    /// Complete replacement
    Replacement,
}

/// A complete social contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialContract {
    /// Unique contract identifier
    pub id: SocialContractId,
    /// Contract version
    pub version: ContractVersion,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Governance scope this contract applies to
    pub scope: GovernanceScope,
    /// Current status
    pub status: SocialContractStatus,
    /// CCL contract code stored in DAG
    pub ccl_contract_cid: Cid,
    /// Rights defined by this contract
    pub rights: Vec<ContractRight>,
    /// Responsibilities defined by this contract
    pub responsibilities: Vec<ContractResponsibility>,
    /// Resource flows defined by this contract
    pub resource_flows: Vec<ResourceFlow>,
    /// Governance mechanisms defined by this contract
    pub governance_mechanisms: Vec<GovernanceMechanism>,
    /// Required consent from members
    pub consent_requirements: ConsentRequirements,
    /// Multi-lingual explanations
    pub translations: HashMap<String, ContractTranslation>,
    /// Contract creator
    pub creator: Did,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last modified timestamp
    pub modified_at: SystemTime,
    /// Parent contract this derives from (if any)
    pub parent_contract: Option<SocialContractId>,
    /// Predecessor version (if this is an amendment)
    pub predecessor: Option<SocialContractId>,
    /// Digital signature of the contract
    pub signature: Option<ContractSignature>,
}

/// Rights granted by a social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRight {
    /// Unique identifier for the right
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Who this right applies to
    pub applies_to: RightSubject,
    /// Conditions for this right
    pub conditions: Vec<String>,
    /// Whether this right can be delegated
    pub delegable: bool,
}

/// Subject that a right applies to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RightSubject {
    /// All members
    AllMembers,
    /// Members with specific roles
    Role(String),
    /// Members meeting specific criteria
    Criteria(Vec<String>),
    /// Specific individuals
    Individuals(Vec<Did>),
}

/// Responsibilities imposed by a social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResponsibility {
    /// Unique identifier for the responsibility
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Who this responsibility applies to
    pub applies_to: RightSubject,
    /// Required actions
    pub required_actions: Vec<String>,
    /// Enforcement mechanism
    pub enforcement: EnforcementMechanism,
    /// Penalties for non-compliance
    pub penalties: Vec<Penalty>,
}

/// Enforcement mechanism for responsibilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementMechanism {
    /// Automatic enforcement by smart contract
    Automatic,
    /// Community-based enforcement
    Community,
    /// Committee-based enforcement
    Committee(String),
    /// Hybrid approach
    Hybrid(Vec<EnforcementMechanism>),
}

/// Penalty for non-compliance with responsibilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    /// Type of penalty
    pub penalty_type: PenaltyType,
    /// Severity level
    pub severity: PenaltySeverity,
    /// Description
    pub description: String,
    /// Duration (if applicable)
    pub duration: Option<u64>,
}

/// Type of penalty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltyType {
    /// Warning
    Warning,
    /// Reputation reduction
    ReputationReduction(f64),
    /// Mana fine
    ManaFine(u64),
    /// Temporary suspension
    Suspension,
    /// Removal from role
    RoleRemoval(String),
    /// Complete expulsion
    Expulsion,
}

/// Severity of penalty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltySeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

/// Resource flow defined by contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFlow {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Source of resources
    pub source: ResourceSource,
    /// Destination of resources  
    pub destination: ResourceDestination,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Amount or calculation method
    pub amount: ResourceAmount,
    /// Trigger conditions
    pub triggers: Vec<String>,
    /// Frequency
    pub frequency: ResourceFrequency,
}

/// Source of resources in a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceSource {
    /// Community pool
    CommunityPool,
    /// Individual member
    Member(Did),
    /// External source
    External(String),
    /// Generated by activity
    Generated(String),
}

/// Destination of resources in a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceDestination {
    /// Community pool
    CommunityPool,
    /// Individual member
    Member(Did),
    /// Role-based distribution
    Role(String),
    /// Criteria-based distribution
    Criteria(Vec<String>),
}

/// Type of resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Mana tokens
    Mana,
    /// Reputation points
    Reputation,
    /// Custom token
    Token(String),
    /// Physical resources
    Physical(String),
    /// Services
    Service(String),
}

/// Amount of resource in a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAmount {
    /// Fixed amount
    Fixed(u64),
    /// Percentage of total
    Percentage(f64),
    /// Based on formula stored in DAG
    Formula(Cid),
    /// Dynamic based on conditions
    Dynamic(Vec<String>),
}

/// Frequency of resource flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceFrequency {
    /// One-time
    Once,
    /// Daily
    Daily,
    /// Weekly
    Weekly,
    /// Monthly
    Monthly,
    /// On specific events
    EventBased(Vec<String>),
}

/// Governance mechanism defined by contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceMechanism {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Type of governance mechanism
    pub mechanism_type: GovernanceMechanismType,
    /// Eligibility rules for participation
    pub eligibility: Vec<String>,
    /// Voting rules
    pub voting_rules: VotingRules,
    /// Execution rules
    pub execution_rules: ExecutionRules,
}

/// Type of governance mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceMechanismType {
    /// Direct democracy - all eligible members vote
    Direct,
    /// Representative democracy - elected delegates vote
    Representative,
    /// Delegated democracy - liquid democracy
    Delegated,
    /// Quadratic voting
    Quadratic,
    /// Reputation-weighted voting
    ReputationWeighted,
    /// Consensus-based decision making
    Consensus,
    /// Custom mechanism
    Custom(String),
}

/// Voting rules for governance mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRules {
    /// Quorum requirement (percentage)
    pub quorum: f64,
    /// Threshold for passage (percentage)
    pub threshold: f64,
    /// Voting period duration (seconds)
    pub voting_period: u64,
    /// Whether delegation is allowed
    pub allow_delegation: bool,
    /// Scaling function for federation aggregation
    pub scaling_function: Option<ScalingFunction>,
}

/// Scaling function for federation vote aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingFunction {
    /// Name of the scaling function
    pub name: String,
    /// Type of scaling
    pub scaling_type: ScalingType,
    /// Parameters for the function
    pub parameters: HashMap<String, f64>,
    /// Description
    pub description: String,
}

/// Type of scaling for vote aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingType {
    /// Linear scaling by population
    Linear,
    /// Quadratic scaling to prevent plutocracy
    Quadratic,
    /// Logarithmic scaling
    Logarithmic,
    /// Weighted by reputation
    ReputationWeighted,
    /// One group, one vote
    OneGroupOneVote,
    /// Hybrid approach
    Hybrid(Vec<ScalingType>),
    /// Custom formula stored in DAG
    Custom(Cid),
}

/// Execution rules for governance decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRules {
    /// Delay before execution (seconds)
    pub execution_delay: u64,
    /// Whether execution is automatic
    pub automatic_execution: bool,
    /// Who can trigger execution
    pub execution_authority: ExecutionAuthority,
    /// Conditions for execution
    pub execution_conditions: Vec<String>,
}

/// Authority for executing governance decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionAuthority {
    /// Automatic by smart contract
    Automatic,
    /// Proposer can execute
    Proposer,
    /// Any member can execute
    AnyMember,
    /// Specific role can execute
    Role(String),
    /// Committee can execute
    Committee(String),
}

/// Consent requirements for contract adoption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequirements {
    /// Type of consent required
    pub consent_type: ConsentType,
    /// Threshold for consent (percentage)
    pub threshold: f64,
    /// Grace period for providing consent (seconds)
    pub grace_period: u64,
    /// What happens if consent is not provided
    pub non_consent_action: NonConsentAction,
}

/// Type of consent required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentType {
    /// Explicit opt-in consent
    OptIn,
    /// Implicit consent (assumed unless opt-out)
    OptOut,
    /// Active confirmation required
    ActiveConfirmation,
    /// Witnessed consent with attestation
    Witnessed,
}

/// Action taken when consent is not provided
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NonConsentAction {
    /// Member is excluded from contract benefits
    Exclude,
    /// Member is removed from community
    Remove,
    /// Contract cannot be adopted
    BlockAdoption,
    /// Member gets limited participation
    LimitedParticipation,
}

/// Translation of contract to other languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTranslation {
    /// Language code (ISO 639-1)
    pub language: String,
    /// Translated title
    pub title: String,
    /// Translated description
    pub description: String,
    /// Translated explanations stored in DAG
    pub explanation_cid: Option<Cid>,
    /// Translator's DID
    pub translator: Did,
    /// Translation timestamp
    pub translated_at: SystemTime,
}

/// Digital signature of a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSignature {
    /// Signature algorithm
    pub algorithm: String,
    /// Signature value
    pub signature: Vec<u8>,
    /// Signer's DID
    pub signer: Did,
    /// Signature timestamp
    pub signed_at: SystemTime,
}

/// Amendment proposal to modify an existing social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAmendment {
    /// Amendment identifier
    pub id: String,
    /// Contract being amended
    pub target_contract: SocialContractId,
    /// Type of amendment
    pub amendment_type: AmendmentType,
    /// Title of the amendment
    pub title: String,
    /// Description of changes
    pub description: String,
    /// Detailed changes in structured format
    pub changes: Vec<ContractChange>,
    /// Rationale for the amendment
    pub rationale: String,
    /// Impact assessment
    pub impact_assessment: ImpactAssessment,
    /// Proposer
    pub proposer: Did,
    /// Proposal timestamp
    pub proposed_at: SystemTime,
}

/// Specific change within an amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Path to the element being changed
    pub path: String,
    /// Old value (for modifications)
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Reason for this specific change
    pub reason: String,
}

/// Type of change in an amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Add new element
    Add,
    /// Modify existing element
    Modify,
    /// Remove existing element
    Remove,
    /// Replace entire section
    Replace,
}

/// Impact assessment of a proposed amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Members affected by the change
    pub affected_members: Vec<Did>,
    /// Estimated impact on resources
    pub resource_impact: ResourceImpact,
    /// Estimated impact on governance
    pub governance_impact: GovernanceImpact,
    /// Risk assessment
    pub risks: Vec<Risk>,
    /// Mitigation strategies
    pub mitigations: Vec<String>,
    /// Implementation timeline
    pub timeline: ImplementationTimeline,
}

/// Impact on resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    /// Estimated change in mana flows
    pub mana_impact: i64,
    /// Estimated change in reputation flows
    pub reputation_impact: f64,
    /// Other resource impacts
    pub other_impacts: HashMap<String, String>,
}

/// Impact on governance processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceImpact {
    /// Changes to voting procedures
    pub voting_changes: Vec<String>,
    /// Changes to decision-making authority
    pub authority_changes: Vec<String>,
    /// Changes to participation rules
    pub participation_changes: Vec<String>,
}

/// Risk identified in impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// Risk category
    pub category: RiskCategory,
    /// Risk description
    pub description: String,
    /// Probability (0.0 to 1.0)
    pub probability: f64,
    /// Severity (0.0 to 1.0)
    pub severity: f64,
}

/// Category of risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Economic risk
    Economic,
    /// Governance risk
    Governance,
    /// Social risk
    Social,
    /// Technical risk
    Technical,
    /// Legal risk
    Legal,
    /// Security risk
    Security,
}

/// Implementation timeline for amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationTimeline {
    /// Phases of implementation
    pub phases: Vec<ImplementationPhase>,
    /// Total estimated duration (seconds)
    pub total_duration: u64,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Phase of amendment implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    /// Phase name
    pub name: String,
    /// Phase description
    pub description: String,
    /// Estimated duration (seconds)
    pub duration: u64,
    /// Prerequisites
    pub prerequisites: Vec<String>,
    /// Deliverables
    pub deliverables: Vec<String>,
}

/// Member's consent record for a social contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberConsent {
    /// Member's DID
    pub member: Did,
    /// Contract consented to
    pub contract: SocialContractId,
    /// Contract version consented to
    pub version: ContractVersion,
    /// Consent status
    pub status: ConsentStatus,
    /// Consent timestamp
    pub consented_at: SystemTime,
    /// Explanation provided to member (language-specific)
    pub explanation_language: String,
    /// Member's signature
    pub signature: Option<ContractSignature>,
    /// Witness attestations (if required)
    pub witnesses: Vec<Did>,
}

/// Status of member's consent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    /// Explicitly consented
    Consented,
    /// Explicitly declined
    Declined,
    /// Pending decision
    Pending,
    /// Consent withdrawn
    Withdrawn,
    /// Implicitly consented (for opt-out systems)
    ImplicitConsent,
}

/// Integration with existing proposal system
impl From<ContractAmendment> for ProposalType {
    fn from(amendment: ContractAmendment) -> Self {
        ProposalType::GenericText(format!(
            "Amendment to Social Contract {}: {}",
            amendment.target_contract, amendment.title
        ))
    }
}

/// Helper functions for social contract management
impl SocialContract {
    /// Create a new social contract
    pub fn new(
        id: SocialContractId,
        title: String,
        description: String,
        scope: GovernanceScope,
        ccl_contract_cid: Cid,
        creator: Did,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            version: ContractVersion::initial(),
            title,
            description,
            scope,
            status: SocialContractStatus::Draft,
            ccl_contract_cid,
            rights: Vec::new(),
            responsibilities: Vec::new(),
            resource_flows: Vec::new(),
            governance_mechanisms: Vec::new(),
            consent_requirements: ConsentRequirements {
                consent_type: ConsentType::OptIn,
                threshold: 0.5,
                grace_period: 2592000, // 30 days
                non_consent_action: NonConsentAction::Exclude,
            },
            translations: HashMap::new(),
            creator,
            created_at: now,
            modified_at: now,
            parent_contract: None,
            predecessor: None,
            signature: None,
        }
    }

    /// Check if contract is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SocialContractStatus::Active)
    }

    /// Get all members who need to provide consent
    pub fn members_requiring_consent(&self) -> Vec<Did> {
        // This would typically query the membership system
        // For now, return empty vec
        Vec::new()
    }

    /// Create amendment proposal
    pub fn create_amendment_proposal(
        &self,
        amendment: ContractAmendment,
        proposer: Did,
    ) -> ProposalSubmission {
        let amendment_description = amendment.description.clone();
        ProposalSubmission {
            proposer,
            proposal_type: amendment.into(),
            description: format!(
                "Amendment to {}: {}",
                self.title, amendment_description
            ),
            duration_secs: 1209600, // 14 days default
            quorum: None,
            threshold: Some(0.67), // Supermajority for amendments
            content_cid: None,
        }
    }
}

impl MemberConsent {
    /// Create new consent record
    pub fn new(
        member: Did,
        contract: SocialContractId,
        version: ContractVersion,
        status: ConsentStatus,
        explanation_language: String,
    ) -> Self {
        Self {
            member,
            contract,
            version,
            status,
            consented_at: SystemTime::now(),
            explanation_language,
            signature: None,
            witnesses: Vec::new(),
        }
    }

    /// Check if consent is valid
    pub fn is_valid(&self) -> bool {
        matches!(
            self.status,
            ConsentStatus::Consented | ConsentStatus::ImplicitConsent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_version_ordering() {
        let v1 = ContractVersion::new(1, 0, 0);
        let v2 = ContractVersion::new(1, 1, 0);
        let v3 = ContractVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_contract_version_increment() {
        let v1 = ContractVersion::new(1, 2, 3);
        
        assert_eq!(v1.increment_major(), ContractVersion::new(2, 0, 0));
        assert_eq!(v1.increment_minor(), ContractVersion::new(1, 3, 0));
        assert_eq!(v1.increment_patch(), ContractVersion::new(1, 2, 4));
    }

    #[test]
    fn test_social_contract_creation() {
        let id = SocialContractId("test-contract".to_string());
        let creator = Did::new("test", "creator");
        let cid = Cid::new_v1_sha256(0x55, b"test-contract");

        let contract = SocialContract::new(
            id.clone(),
            "Test Contract".to_string(),
            "A test social contract".to_string(),
            GovernanceScope::Local,
            cid,
            creator.clone(),
        );

        assert_eq!(contract.id, id);
        assert_eq!(contract.creator, creator);
        assert_eq!(contract.version, ContractVersion::initial());
        assert!(matches!(contract.status, SocialContractStatus::Draft));
    }

    #[test]
    fn test_member_consent() {
        let member = Did::new("test", "member");
        let contract_id = SocialContractId("test-contract".to_string());
        let version = ContractVersion::new(1, 0, 0);

        let consent = MemberConsent::new(
            member.clone(),
            contract_id.clone(),
            version.clone(),
            ConsentStatus::Consented,
            "en".to_string(),
        );

        assert_eq!(consent.member, member);
        assert_eq!(consent.contract, contract_id);
        assert_eq!(consent.version, version);
        assert!(consent.is_valid());
    }

    #[test]
    fn test_consent_status_validity() {
        let consented_consent = MemberConsent::new(
            Did::new("test", "member"),
            SocialContractId("test".to_string()),
            ContractVersion::initial(),
            ConsentStatus::Consented,
            "en".to_string(),
        );
        assert!(consented_consent.is_valid());

        let declined_consent = MemberConsent::new(
            Did::new("test", "member"),
            SocialContractId("test".to_string()),
            ContractVersion::initial(),
            ConsentStatus::Declined,
            "en".to_string(),
        );
        assert!(!declined_consent.is_valid());
    }
}