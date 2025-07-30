//! Federation Trust Framework
//!
//! This module implements the Scoped Federation Trust Framework as specified in requirement 2.2.
//! It provides different trust levels for different activities, trust inheritance models,
//! cross-federation trust bridges, and a configurable trust policy engine.

use crate::cooperative_schemas::{CooperativeProfile, TrustLevel, TrustRelationship};
use icn_common::Did;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Different trust contexts for federation activities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustContext {
    /// Trust for governance activities (voting, proposals, decision-making)
    Governance,
    /// Trust for resource sharing (compute, storage, network resources)
    ResourceSharing,
    /// Trust for mutual credit and economic transactions
    MutualCredit,
    /// Trust for identity verification and credential validation
    Identity,
    /// Trust for network infrastructure and routing
    Infrastructure,
    /// Trust for data sharing and privacy-sensitive operations
    DataSharing,
    /// General purpose trust for basic cooperation
    General,
    /// Custom trust context with specified name
    Custom(String),
}

impl TrustContext {
    /// Get string representation of the trust context
    pub fn as_str(&self) -> &str {
        match self {
            TrustContext::Governance => "governance",
            TrustContext::ResourceSharing => "resource_sharing",
            TrustContext::MutualCredit => "mutual_credit",
            TrustContext::Identity => "identity",
            TrustContext::Infrastructure => "infrastructure",
            TrustContext::DataSharing => "data_sharing",
            TrustContext::General => "general",
            TrustContext::Custom(name) => name,
        }
    }

    /// Parse trust context from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "governance" => TrustContext::Governance,
            "resource_sharing" => TrustContext::ResourceSharing,
            "mutual_credit" => TrustContext::MutualCredit,
            "identity" => TrustContext::Identity,
            "infrastructure" => TrustContext::Infrastructure,
            "data_sharing" => TrustContext::DataSharing,
            "general" => TrustContext::General,
            custom => TrustContext::Custom(custom.to_string()),
        }
    }
}

/// Federation identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FederationId(pub String);

impl FederationId {
    pub fn new(id: String) -> Self {
        FederationId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Scoped trust relationship that includes context and federation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedTrustRelationship {
    /// Base trust relationship
    pub base: TrustRelationship,
    /// Specific trust context
    pub context: TrustContext,
    /// Federation this trust relationship belongs to (if any)
    pub federation: Option<FederationId>,
    /// Trust inheritance rules
    pub inheritance: TrustInheritance,
    /// Additional metadata for this scoped trust
    pub metadata: HashMap<String, String>,
}

/// Trust inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustInheritance {
    /// Whether this trust can be inherited by child cooperatives
    pub inheritable: bool,
    /// Maximum inheritance depth (how many levels down trust can propagate)
    pub max_depth: Option<u32>,
    /// Trust degradation factor per inheritance level (0.0-1.0)
    pub degradation_factor: f64,
    /// Minimum trust level that can be inherited
    pub min_inherited_level: TrustLevel,
}

impl Default for TrustInheritance {
    fn default() -> Self {
        Self {
            inheritable: true,
            max_depth: Some(3),
            degradation_factor: 0.8,
            min_inherited_level: TrustLevel::Basic,
        }
    }
}

/// Cross-federation trust bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTrustBridge {
    /// Source federation
    pub from_federation: FederationId,
    /// Target federation  
    pub to_federation: FederationId,
    /// Trust relationship between federations
    pub trust: ScopedTrustRelationship,
    /// Bridge configuration
    pub bridge_config: BridgeConfig,
    /// Timestamp when bridge was established
    pub established_at: u64,
    /// Optional expiration timestamp
    pub expires_at: Option<u64>,
}

/// Configuration for cross-federation trust bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Whether trust can flow bidirectionally across this bridge
    pub bidirectional: bool,
    /// Trust contexts that can flow across this bridge
    pub allowed_contexts: HashSet<TrustContext>,
    /// Maximum trust level that can flow across the bridge
    pub max_bridge_trust: TrustLevel,
    /// Trust degradation when crossing the bridge
    pub bridge_degradation: f64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            bidirectional: false,
            allowed_contexts: [TrustContext::General].into_iter().collect(),
            max_bridge_trust: TrustLevel::Basic,
            bridge_degradation: 0.5,
        }
    }
}

/// Trust policy rule for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPolicyRule {
    /// Name of the policy rule
    pub name: String,
    /// Trust contexts this rule applies to
    pub applicable_contexts: HashSet<TrustContext>,
    /// Minimum trust level required
    pub min_trust_level: TrustLevel,
    /// Whether federation membership is required
    pub require_federation_membership: bool,
    /// Maximum inheritance depth allowed
    pub max_inheritance_depth: Option<u32>,
    /// Whether cross-federation trust is allowed
    pub allow_cross_federation: bool,
    /// Custom validation logic identifier
    pub custom_validator: Option<String>,
}

/// Trust policy engine for validating trust relationships and permissions
#[derive(Debug, Default)]
pub struct TrustPolicyEngine {
    /// Policy rules indexed by context
    rules: HashMap<TrustContext, Vec<TrustPolicyRule>>,
    /// Federation trust relationships
    federation_trusts: HashMap<FederationId, Vec<ScopedTrustRelationship>>,
    /// Cross-federation bridges
    bridges: HashMap<(FederationId, FederationId), FederationTrustBridge>,
    /// Federation memberships
    memberships: HashMap<Did, HashSet<FederationId>>,
}

/// Trust validation result
#[derive(Debug, Clone)]
pub enum TrustValidationResult {
    /// Trust validation passed
    Allowed {
        /// Effective trust level after all calculations
        effective_trust: TrustLevel,
        /// Trust path taken (for inherited or bridged trust)
        trust_path: Vec<String>,
    },
    /// Trust validation failed
    Denied {
        /// Reason for denial
        reason: String,
    },
}

/// Federation metadata containing configuration and member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMetadata {
    /// Federation identifier
    pub id: FederationId,
    /// Human-readable name
    pub name: String,
    /// Description of the federation's purpose
    pub description: String,
    /// Federation scope configuration
    pub scope: FederationScope,
    /// Quorum policies for different types of decisions
    pub quorum_policies: HashMap<String, QuorumPolicy>,
    /// List of member cooperatives
    pub members: HashMap<Did, FederationMember>,
    /// Trust policy configuration
    pub trust_policies: HashMap<TrustContext, FederationTrustPolicy>,
    /// Cross-federation bridges
    pub bridges: HashMap<FederationId, FederationTrustBridge>,
    /// Federation DID document
    pub did_document: Option<FederationDidDocument>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// Federation status
    pub status: FederationStatus,
}

/// Scope configuration for a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationScope {
    /// Geographic scope (if applicable)
    pub geographic: Option<GeographicScope>,
    /// Sectoral scope (industry/domain focus)
    pub sectoral: Vec<String>,
    /// Size limits
    pub size_limits: SizeLimits,
    /// Membership criteria
    pub membership_criteria: MembershipCriteria,
}

/// Geographic scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicScope {
    /// Countries included
    pub countries: Vec<String>,
    /// Regions/states included
    pub regions: Vec<String>,
    /// Local areas included
    pub localities: Vec<String>,
    /// Whether the scope is global
    pub global: bool,
}

/// Size limits for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeLimits {
    /// Maximum number of member cooperatives
    pub max_members: Option<u32>,
    /// Minimum number of members to remain active
    pub min_members: u32,
    /// Maximum total membership across all cooperatives
    pub max_total_membership: Option<u32>,
}

/// Membership criteria for joining the federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipCriteria {
    /// Required cooperative types
    pub required_coop_types: Vec<String>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Minimum trust level required
    pub min_trust_level: TrustLevel,
    /// Required attestations
    pub required_attestations: u32,
    /// Probationary period (seconds)
    pub probationary_period: Option<u64>,
    /// Custom criteria
    pub custom_criteria: HashMap<String, String>,
}

/// Quorum policy for federation decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumPolicy {
    /// Decision type this policy applies to
    pub decision_type: String,
    /// Minimum percentage of members required for quorum (0.0-1.0)
    pub quorum_threshold: f64,
    /// Minimum percentage required for approval (0.0-1.0)
    pub approval_threshold: f64,
    /// Whether unanimous consent is required
    pub require_unanimous: bool,
    /// Voting weight assignment
    pub weight_assignment: WeightAssignment,
    /// Voting deadline in seconds
    pub voting_deadline: u64,
}

/// How voting weights are assigned to members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightAssignment {
    /// Equal weight for all members
    Equal,
    /// Weight based on member size
    MembershipBased,
    /// Weight based on trust level
    TrustBased,
    /// Weight based on contribution history
    ContributionBased,
    /// Custom weight assignment
    Custom(HashMap<Did, f64>),
}

/// Federation member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMember {
    /// Member DID
    pub did: Did,
    /// Cooperative profile
    pub profile: CooperativeProfile,
    /// Trust relationships with other members
    pub trust_relationships: HashMap<Did, ScopedTrustRelationship>,
    /// Member status
    pub status: MemberStatus,
    /// Roles within the federation
    pub roles: Vec<FederationRole>,
    /// Join timestamp
    pub joined_at: u64,
    /// Last activity timestamp
    pub last_active_at: u64,
    /// Voting weight
    pub voting_weight: f64,
}

/// Status of a federation member
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberStatus {
    /// Active member with full rights
    Active,
    /// Probationary member with limited rights
    Probationary,
    /// Suspended member with no rights
    Suspended,
    /// Member leaving the federation
    Leaving,
    /// Former member
    Expelled,
}

/// Roles a member can have within a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationRole {
    /// Standard member
    Member,
    /// Coordinator/facilitator role
    Coordinator,
    /// Trust committee member
    TrustCommittee,
    /// Governance committee member
    GovernanceCommittee,
    /// Technical committee member
    TechnicalCommittee,
    /// Founding member
    Founder,
    /// Custom role
    Custom(String),
}

/// Status of a federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationStatus {
    /// Active and operational
    Active,
    /// Forming - not yet fully operational
    Forming,
    /// Suspended - temporarily inactive
    Suspended,
    /// Dissolving - in process of shutdown
    Dissolving,
    /// Dissolved - no longer active
    Dissolved,
}

/// Trust policy specific to a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationTrustPolicy {
    /// Trust context this policy applies to
    pub context: TrustContext,
    /// Minimum trust level required
    pub min_trust_level: TrustLevel,
    /// Trust inheritance configuration
    pub inheritance: TrustInheritance,
    /// Whether cross-federation trust is allowed
    pub allow_cross_federation: bool,
    /// Policy rules
    pub rules: Vec<TrustPolicyRule>,
    /// Trust verification requirements
    pub verification_requirements: TrustVerificationRequirements,
}

/// Requirements for trust verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustVerificationRequirements {
    /// Minimum number of attestations required
    pub min_attestations: u32,
    /// Required attestor trust levels
    pub attestor_min_trust: TrustLevel,
    /// Whether periodic re-verification is required
    pub periodic_reverification: bool,
    /// Re-verification interval (seconds)
    pub reverification_interval: Option<u64>,
    /// Challenge mechanisms enabled
    pub challenge_mechanisms: Vec<String>,
}

/// DID document for federation-level identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDidDocument {
    /// Federation DID
    pub id: Did,
    /// Public keys for federation operations
    pub public_keys: Vec<FederationPublicKey>,
    /// Service endpoints
    pub services: Vec<FederationService>,
    /// Verification methods
    pub verification_methods: Vec<VerificationMethod>,
    /// Authentication methods
    pub authentication: Vec<String>,
    /// Assertion methods
    pub assertion_method: Vec<String>,
    /// Key agreement methods
    pub key_agreement: Vec<String>,
    /// Capability invocation methods
    pub capability_invocation: Vec<String>,
    /// Capability delegation methods
    pub capability_delegation: Vec<String>,
    /// Document metadata
    pub metadata: FederationDidMetadata,
}

/// Public key in a federation DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPublicKey {
    /// Key ID
    pub id: String,
    /// Key type (e.g., "Ed25519VerificationKey2020")
    pub key_type: String,
    /// Key controller
    pub controller: Did,
    /// Public key bytes
    #[serde(with = "serde_bytes")]
    pub public_key_bytes: Vec<u8>,
    /// Key purpose
    pub purpose: Vec<KeyPurpose>,
}

/// Purpose for which a key can be used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyPurpose {
    /// Authentication
    Authentication,
    /// Assertion/signing
    Assertion,
    /// Key agreement/encryption
    KeyAgreement,
    /// Capability invocation
    CapabilityInvocation,
    /// Capability delegation
    CapabilityDelegation,
}

/// Service endpoint in a federation DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationService {
    /// Service ID
    pub id: String,
    /// Service type
    pub service_type: String,
    /// Service endpoint URL
    pub service_endpoint: String,
    /// Service description
    pub description: Option<String>,
    /// Service properties
    pub properties: HashMap<String, String>,
}

/// Verification method for DID operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Method ID
    pub id: String,
    /// Method type
    pub method_type: String,
    /// Controller DID
    pub controller: Did,
    /// Verification material
    pub verification_material: VerificationMaterial,
}

/// Verification material for a method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMaterial {
    /// Public key bytes
    PublicKeyBytes(#[serde(with = "serde_bytes")] Vec<u8>),
    /// Multibase-encoded public key
    PublicKeyMultibase(String),
    /// JSON Web Key
    JsonWebKey(serde_json::Value),
}

/// Metadata for federation DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDidMetadata {
    /// Creation timestamp
    pub created: u64,
    /// Last updated timestamp
    pub updated: u64,
    /// Version of the document
    pub version: u32,
    /// Next update deadline
    pub next_update: Option<u64>,
    /// Deactivated status
    pub deactivated: bool,
}

impl FederationMetadata {
    /// Create new federation metadata
    pub fn new(
        id: FederationId,
        name: String,
        description: String,
        scope: FederationScope,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            name,
            description,
            scope,
            quorum_policies: HashMap::new(),
            members: HashMap::new(),
            trust_policies: HashMap::new(),
            bridges: HashMap::new(),
            did_document: None,
            created_at: now,
            updated_at: now,
            status: FederationStatus::Forming,
        }
    }

    /// Add a member to the federation
    pub fn add_member(&mut self, member: FederationMember) -> Result<(), String> {
        if self.members.contains_key(&member.did) {
            return Err("Member already exists".to_string());
        }

        // Check size limits
        if let Some(max_members) = self.scope.size_limits.max_members {
            if self.members.len() >= max_members as usize {
                return Err("Federation has reached maximum member limit".to_string());
            }
        }

        self.members.insert(member.did.clone(), member);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(())
    }

    /// Remove a member from the federation
    pub fn remove_member(&mut self, did: &Did) -> Result<FederationMember, String> {
        let member = self
            .members
            .remove(did)
            .ok_or_else(|| "Member not found".to_string())?;

        // Check minimum member requirement
        if self.members.len() < self.scope.size_limits.min_members as usize {
            // Re-add the member to maintain minimum
            self.members.insert(did.clone(), member.clone());
            return Err("Removing member would violate minimum member requirement".to_string());
        }

        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(member)
    }

    /// Get active members
    pub fn get_active_members(&self) -> Vec<&FederationMember> {
        self.members
            .values()
            .filter(|m| m.status == MemberStatus::Active)
            .collect()
    }

    /// Check if federation meets quorum for a decision type
    pub fn check_quorum(&self, decision_type: &str, participating_members: &[&Did]) -> bool {
        if let Some(policy) = self.quorum_policies.get(decision_type) {
            let active_members = self.get_active_members();
            let quorum_required =
                (active_members.len() as f64 * policy.quorum_threshold).ceil() as usize;
            participating_members.len() >= quorum_required
        } else {
            // Default: simple majority
            let active_members = self.get_active_members();
            participating_members.len() >= active_members.len().div_ceil(2)
        }
    }

    /// Set federation status
    pub fn set_status(&mut self, status: FederationStatus) {
        self.status = status;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

impl TrustPolicyEngine {
    /// Create a new trust policy engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a policy rule
    pub fn add_rule(&mut self, rule: TrustPolicyRule) {
        for context in &rule.applicable_contexts {
            self.rules
                .entry(context.clone())
                .or_default()
                .push(rule.clone());
        }
    }

    /// Add federation membership for a DID
    pub fn add_federation_membership(&mut self, did: Did, federation: FederationId) {
        self.memberships.entry(did).or_default().insert(federation);
    }

    /// Add a scoped trust relationship to a federation
    pub fn add_federation_trust(
        &mut self,
        federation: FederationId,
        trust: ScopedTrustRelationship,
    ) {
        self.federation_trusts
            .entry(federation)
            .or_default()
            .push(trust);
    }

    /// Add a cross-federation trust bridge
    pub fn add_bridge(&mut self, bridge: FederationTrustBridge) {
        let key = (bridge.from_federation.clone(), bridge.to_federation.clone());
        self.bridges.insert(key, bridge);
    }

    /// Validate trust for a specific context and operation
    pub fn validate_trust(
        &self,
        trustor: &Did,
        trustee: &Did,
        context: &TrustContext,
        _operation: &str,
    ) -> TrustValidationResult {
        // Get applicable rules for this context
        let rules = match self.rules.get(context) {
            Some(rules) if !rules.is_empty() => rules,
            _ => {
                return TrustValidationResult::Denied {
                    reason: format!("No policy rules defined for context {context:?}"),
                };
            }
        };

        // Check direct trust relationships first
        if let Some(direct_trust) = self.find_direct_trust(trustor, trustee, context) {
            return self.validate_against_rules(&direct_trust, rules, vec!["direct".to_string()]);
        }

        // Check inherited trust
        if let Some(inherited_trust) = self.find_inherited_trust(trustor, trustee, context) {
            return inherited_trust;
        }

        // Check cross-federation trust
        if let Some(bridge_trust) = self.find_bridged_trust(trustor, trustee, context) {
            return bridge_trust;
        }

        TrustValidationResult::Denied {
            reason: "No valid trust relationship found".to_string(),
        }
    }

    /// Find direct trust relationship between two DIDs in a specific context
    fn find_direct_trust(
        &self,
        trustor: &Did,
        trustee: &Did,
        context: &TrustContext,
    ) -> Option<ScopedTrustRelationship> {
        for trusts in self.federation_trusts.values() {
            for trust in trusts {
                if trust.base.attestor == *trustor
                    && trust.base.subject == *trustee
                    && trust.context == *context
                    && trust.base.is_valid()
                {
                    return Some(trust.clone());
                }
            }
        }
        None
    }

    /// Find inherited trust through federation membership
    fn find_inherited_trust(
        &self,
        trustor: &Did,
        trustee: &Did,
        context: &TrustContext,
    ) -> Option<TrustValidationResult> {
        // Get federations that trustor is a member of
        let trustor_federations = self.memberships.get(trustor)?;

        // Get federations that trustee is a member of
        let trustee_federations = self.memberships.get(trustee)?;

        for trustor_fed in trustor_federations {
            for trustee_fed in trustee_federations {
                if trustor_fed == trustee_fed {
                    // Same federation - check for federation-level trust
                    if let Some(fed_trusts) = self.federation_trusts.get(trustor_fed) {
                        for trust in fed_trusts {
                            if trust.context == *context
                                && trust.inheritance.inheritable
                                && trust.base.is_valid()
                            {
                                // Calculate inherited trust level
                                let inherited_level = self.calculate_inherited_trust_level(
                                    &trust.base.trust_level,
                                    &trust.inheritance,
                                    1, // inheritance depth
                                );

                                return Some(TrustValidationResult::Allowed {
                                    effective_trust: inherited_level,
                                    trust_path: vec![format!(
                                        "federation_inheritance:{}",
                                        trustor_fed.as_str()
                                    )],
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Find trust through cross-federation bridges
    fn find_bridged_trust(
        &self,
        trustor: &Did,
        trustee: &Did,
        context: &TrustContext,
    ) -> Option<TrustValidationResult> {
        let trustor_federations = self.memberships.get(trustor)?;
        let trustee_federations = self.memberships.get(trustee)?;

        for trustor_fed in trustor_federations {
            for trustee_fed in trustee_federations {
                if let Some(bridge) = self
                    .bridges
                    .get(&(trustor_fed.clone(), trustee_fed.clone()))
                {
                    // Skip expired bridges
                    if let Some(expiry) = bridge.expires_at {
                        let now = chrono::Utc::now().timestamp() as u64;
                        if now >= expiry {
                            continue;
                        }
                    }

                    if bridge.bridge_config.allowed_contexts.contains(context) {
                        // Calculate bridged trust level
                        let bridged_level = self.calculate_bridged_trust_level(
                            &bridge.trust.base.trust_level,
                            &bridge.bridge_config,
                        );

                        return Some(TrustValidationResult::Allowed {
                            effective_trust: bridged_level,
                            trust_path: vec![format!(
                                "bridge:{}→{}",
                                trustor_fed.as_str(),
                                trustee_fed.as_str()
                            )],
                        });
                    }
                }
            }
        }
        None
    }

    /// Validate a trust relationship against policy rules
    fn validate_against_rules(
        &self,
        trust: &ScopedTrustRelationship,
        rules: &[TrustPolicyRule],
        trust_path: Vec<String>,
    ) -> TrustValidationResult {
        for rule in rules {
            if !self.meets_minimum_trust(&trust.base.trust_level, &rule.min_trust_level) {
                return TrustValidationResult::Denied {
                    reason: format!(
                        "Trust level {:?} does not meet minimum required {:?} for rule '{}'",
                        trust.base.trust_level, rule.min_trust_level, rule.name
                    ),
                };
            }

            if rule.require_federation_membership && trust.federation.is_none() {
                return TrustValidationResult::Denied {
                    reason: format!("Rule '{}' requires federation membership", rule.name),
                };
            }
        }

        TrustValidationResult::Allowed {
            effective_trust: trust.base.trust_level.clone(),
            trust_path,
        }
    }

    /// Calculate inherited trust level with degradation
    fn calculate_inherited_trust_level(
        &self,
        base_trust: &TrustLevel,
        inheritance: &TrustInheritance,
        depth: u32,
    ) -> TrustLevel {
        if let Some(max_depth) = inheritance.max_depth {
            if depth > max_depth {
                return TrustLevel::None;
            }
        }

        // Apply degradation factor
        let degraded = match base_trust {
            TrustLevel::Full => {
                if inheritance.degradation_factor >= 0.8 {
                    TrustLevel::Partial
                } else {
                    TrustLevel::Basic
                }
            }
            TrustLevel::Partial => {
                if inheritance.degradation_factor >= 0.6 {
                    TrustLevel::Basic
                } else {
                    TrustLevel::None
                }
            }
            TrustLevel::Basic => {
                if inheritance.degradation_factor >= 0.4 {
                    TrustLevel::Basic
                } else {
                    TrustLevel::None
                }
            }
            TrustLevel::None => TrustLevel::None,
        };

        // Ensure doesn't go below minimum
        if self.meets_minimum_trust(&degraded, &inheritance.min_inherited_level) {
            degraded
        } else {
            inheritance.min_inherited_level.clone()
        }
    }

    /// Calculate trust level after crossing a federation bridge
    fn calculate_bridged_trust_level(
        &self,
        base_trust: &TrustLevel,
        bridge_config: &BridgeConfig,
    ) -> TrustLevel {
        // Apply bridge degradation
        let degraded = match base_trust {
            TrustLevel::Full => {
                if bridge_config.bridge_degradation >= 0.8 {
                    TrustLevel::Partial
                } else {
                    TrustLevel::Basic
                }
            }
            TrustLevel::Partial => {
                if bridge_config.bridge_degradation >= 0.6 {
                    TrustLevel::Basic
                } else {
                    TrustLevel::None
                }
            }
            TrustLevel::Basic => {
                if bridge_config.bridge_degradation >= 0.4 {
                    TrustLevel::Basic
                } else {
                    TrustLevel::None
                }
            }
            TrustLevel::None => TrustLevel::None,
        };

        // Ensure doesn't exceed bridge maximum
        if self.meets_minimum_trust(&bridge_config.max_bridge_trust, &degraded) {
            degraded
        } else {
            bridge_config.max_bridge_trust.clone()
        }
    }

    /// Check if one trust level meets the minimum requirement of another
    fn meets_minimum_trust(&self, actual: &TrustLevel, required: &TrustLevel) -> bool {
        let actual_value = self.trust_level_value(actual);
        let required_value = self.trust_level_value(required);
        actual_value >= required_value
    }

    /// Convert trust level to numeric value for comparison
    fn trust_level_value(&self, level: &TrustLevel) -> u8 {
        match level {
            TrustLevel::None => 0,
            TrustLevel::Basic => 1,
            TrustLevel::Partial => 2,
            TrustLevel::Full => 3,
        }
    }

    /// Get federation memberships for a DID
    pub fn get_federation_memberships(&self, did: &Did) -> Option<&HashSet<FederationId>> {
        self.memberships.get(did)
    }

    /// Check if a DID is a member of a specific federation
    pub fn is_federation_member(&self, did: &Did, federation: &FederationId) -> bool {
        self.memberships
            .get(did)
            .map(|federations| federations.contains(federation))
            .unwrap_or(false)
    }
}

/// DID document verification engine for federations
#[derive(Debug)]
pub struct FederationDidVerifier {
    /// Known federation DID documents
    known_documents: HashMap<Did, FederationDidDocument>,
    /// Trust anchors for verification
    trust_anchors: HashMap<Did, ed25519_dalek::VerifyingKey>,
    /// Verification policies
    policies: HashMap<String, DidVerificationPolicy>,
}

/// Policy for DID document verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidVerificationPolicy {
    /// Policy name
    pub name: String,
    /// Required verification methods
    pub required_methods: Vec<String>,
    /// Required key purposes
    pub required_purposes: Vec<KeyPurpose>,
    /// Maximum document age (seconds)
    pub max_document_age: Option<u64>,
    /// Required trust anchors
    pub required_trust_anchors: Vec<Did>,
    /// Whether cross-federation verification is allowed
    pub allow_cross_federation: bool,
}

/// Result of DID document verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidVerificationResult {
    /// Verification passed
    Valid {
        /// DID that was verified
        did: Did,
        /// Trust level established
        trust_level: TrustLevel,
        /// Verification path
        verification_path: Vec<String>,
    },
    /// Verification failed
    Invalid {
        /// Reason for failure
        reason: String,
        /// DID that failed verification
        did: Did,
    },
    /// Verification requires additional information
    Pending {
        /// Required additional information
        required_info: Vec<String>,
        /// DID under verification
        did: Did,
    },
}

/// Trust bootstrapping protocol for federations
#[derive(Debug)]
pub struct FederationTrustBootstrap {
    /// Local federation metadata
    local_federation: FederationMetadata,
    /// Trust policy engine
    #[allow(dead_code)]
    trust_engine: TrustPolicyEngine,
    /// DID verifier
    #[allow(dead_code)]
    did_verifier: FederationDidVerifier,
    /// Bootstrap sessions in progress
    active_sessions: HashMap<String, BootstrapSession>,
}

/// Bootstrap session between two federations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSession {
    /// Session ID
    pub session_id: String,
    /// Local federation ID
    pub local_federation: FederationId,
    /// Remote federation ID
    pub remote_federation: FederationId,
    /// Session status
    pub status: BootstrapStatus,
    /// Proposed trust contexts
    pub proposed_contexts: Vec<TrustContext>,
    /// Proposed trust level
    pub proposed_trust_level: TrustLevel,
    /// Verification challenges
    pub challenges: Vec<VerificationChallenge>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
}

/// Status of a bootstrap session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootstrapStatus {
    /// Session initiated
    Initiated,
    /// Waiting for response
    PendingResponse,
    /// Verifying credentials
    Verifying,
    /// Establishing trust
    EstablishingTrust,
    /// Bootstrap completed successfully
    Completed,
    /// Bootstrap failed
    Failed(String),
    /// Session expired
    Expired,
}

/// Verification challenge for bootstrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationChallenge {
    /// Challenge ID
    pub challenge_id: String,
    /// Challenge type
    pub challenge_type: ChallengeType,
    /// Challenge data
    pub challenge_data: Vec<u8>,
    /// Expected response
    pub expected_response: Option<Vec<u8>>,
    /// Challenge deadline
    pub deadline: u64,
    /// Whether challenge is completed
    pub completed: bool,
}

/// Type of verification challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Signature challenge
    Signature,
    /// DID ownership proof
    DidOwnership,
    /// Federation membership proof
    MembershipProof,
    /// Trust attestation
    TrustAttestation,
    /// Custom challenge
    Custom(String),
}

impl FederationDidVerifier {
    /// Create a new DID verifier
    pub fn new() -> Self {
        Self {
            known_documents: HashMap::new(),
            trust_anchors: HashMap::new(),
            policies: HashMap::new(),
        }
    }

    /// Add a trusted DID document
    pub fn add_trusted_document(&mut self, document: FederationDidDocument) {
        self.known_documents.insert(document.id.clone(), document);
    }

    /// Add a trust anchor
    pub fn add_trust_anchor(&mut self, did: Did, key: ed25519_dalek::VerifyingKey) {
        self.trust_anchors.insert(did, key);
    }

    /// Add a verification policy
    pub fn add_policy(&mut self, policy: DidVerificationPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Verify a DID document
    pub fn verify_document(
        &self,
        document: &FederationDidDocument,
        policy_name: &str,
    ) -> DidVerificationResult {
        let policy = match self.policies.get(policy_name) {
            Some(p) => p,
            None => {
                return DidVerificationResult::Invalid {
                    reason: format!("Unknown verification policy: {policy_name}"),
                    did: document.id.clone(),
                };
            }
        };

        // Check document age
        if let Some(max_age) = policy.max_document_age {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if now - document.metadata.created > max_age {
                return DidVerificationResult::Invalid {
                    reason: "Document too old".to_string(),
                    did: document.id.clone(),
                };
            }
        }

        // Check required verification methods
        for required_method in &policy.required_methods {
            if !document
                .verification_methods
                .iter()
                .any(|vm| vm.method_type == *required_method)
            {
                return DidVerificationResult::Invalid {
                    reason: format!("Missing required verification method: {required_method}"),
                    did: document.id.clone(),
                };
            }
        }

        // Check required key purposes
        for required_purpose in &policy.required_purposes {
            if !document
                .public_keys
                .iter()
                .any(|pk| pk.purpose.contains(required_purpose))
            {
                return DidVerificationResult::Invalid {
                    reason: format!("Missing required key purpose: {required_purpose:?}"),
                    did: document.id.clone(),
                };
            }
        }

        // Check trust anchor requirements
        if !policy.required_trust_anchors.is_empty() {
            let has_required_anchor = policy
                .required_trust_anchors
                .iter()
                .any(|anchor| self.trust_anchors.contains_key(anchor));

            if !has_required_anchor {
                return DidVerificationResult::Pending {
                    required_info: vec!["Trust anchor verification".to_string()],
                    did: document.id.clone(),
                };
            }
        }

        // Verification passed
        DidVerificationResult::Valid {
            did: document.id.clone(),
            trust_level: TrustLevel::Partial, // Default, could be policy-based
            verification_path: vec!["direct_verification".to_string()],
        }
    }

    /// Verify trust chain between DIDs
    pub fn verify_trust_chain(
        &self,
        from_did: &Did,
        to_did: &Did,
        max_chain_length: usize,
    ) -> Result<Vec<Did>, String> {
        // Simple breadth-first search for trust chain
        let mut queue = vec![(from_did.clone(), vec![from_did.clone()])];
        let mut visited = HashSet::new();

        while let Some((current_did, path)) = queue.pop() {
            if current_did == *to_did {
                return Ok(path);
            }

            if path.len() >= max_chain_length {
                continue;
            }

            if visited.contains(&current_did) {
                continue;
            }
            visited.insert(current_did.clone());

            // Find DIDs that current_did trusts (simplified)
            if let Some(document) = self.known_documents.get(&current_did) {
                for service in &document.services {
                    if service.service_type == "TrustRelationship" {
                        // Parse trusted DIDs from service endpoint (simplified)
                        // In practice, this would query the actual trust relationships
                    }
                }
            }
        }

        Err("No trust chain found".to_string())
    }

    /// Resolve DID document from federation network
    pub async fn resolve_did(
        &self,
        did: &Did,
        _federation: Option<&FederationId>,
    ) -> Result<FederationDidDocument, String> {
        // Check local cache first
        if let Some(document) = self.known_documents.get(did) {
            return Ok(document.clone());
        }

        // For now, return an error as we don't have network resolution implemented
        Err(format!("DID document not found for {did}"))
    }
}

impl Default for FederationDidVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationTrustBootstrap {
    /// Create a new bootstrap engine
    pub fn new(
        local_federation: FederationMetadata,
        trust_engine: TrustPolicyEngine,
        did_verifier: FederationDidVerifier,
    ) -> Self {
        Self {
            local_federation,
            trust_engine,
            did_verifier,
            active_sessions: HashMap::new(),
        }
    }

    /// Initiate trust bootstrap with another federation
    pub fn initiate_bootstrap(
        &mut self,
        remote_federation: FederationId,
        contexts: Vec<TrustContext>,
        trust_level: TrustLevel,
    ) -> Result<String, String> {
        let session_id = format!(
            "bootstrap_{}_{}",
            self.local_federation.id.as_str(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session = BootstrapSession {
            session_id: session_id.clone(),
            local_federation: self.local_federation.id.clone(),
            remote_federation,
            status: BootstrapStatus::Initiated,
            proposed_contexts: contexts,
            proposed_trust_level: trust_level,
            challenges: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            expires_at: now + 3600, // 1 hour default
        };

        self.active_sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Process bootstrap response
    pub fn process_response(
        &mut self,
        session_id: &str,
        _response_data: &[u8],
    ) -> Result<BootstrapStatus, String> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if session.status != BootstrapStatus::PendingResponse {
            return Err("Session not waiting for response".to_string());
        }

        // Process the response (simplified)
        session.status = BootstrapStatus::Verifying;

        // Add verification challenges
        let challenge = VerificationChallenge {
            challenge_id: format!("challenge_{}", fastrand::u64(..)),
            challenge_type: ChallengeType::Signature,
            challenge_data: b"federation_trust_bootstrap".to_vec(),
            expected_response: None,
            deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 300, // 5 minutes
            completed: false,
        };

        session.challenges.push(challenge);
        Ok(session.status.clone())
    }

    /// Complete trust establishment
    pub fn complete_trust_establishment(
        &mut self,
        session_id: &str,
    ) -> Result<ScopedTrustRelationship, String> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if session.status != BootstrapStatus::EstablishingTrust {
            return Err("Session not ready for trust establishment".to_string());
        }

        // Create trust relationship (simplified)
        let trust = ScopedTrustRelationship {
            base: TrustRelationship {
                attestor: Did::new("federation", self.local_federation.id.as_str()),
                subject: Did::new("federation", session.remote_federation.as_str()),
                trust_level: session.proposed_trust_level.clone(),
                trust_scope: session
                    .proposed_contexts
                    .iter()
                    .map(|c| c.as_str().to_string())
                    .collect(),
                justification: Some("Automated federation trust bootstrap".to_string()),
                established_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                expires_at: None,
                reciprocal: true,
            },
            context: session
                .proposed_contexts
                .first()
                .unwrap_or(&TrustContext::General)
                .clone(),
            federation: Some(self.local_federation.id.clone()),
            inheritance: TrustInheritance::default(),
            metadata: HashMap::new(),
        };

        session.status = BootstrapStatus::Completed;
        Ok(trust)
    }

    /// Get active bootstrap sessions
    pub fn get_active_sessions(&self) -> Vec<&BootstrapSession> {
        self.active_sessions
            .values()
            .filter(|s| {
                !matches!(
                    s.status,
                    BootstrapStatus::Completed
                        | BootstrapStatus::Failed(_)
                        | BootstrapStatus::Expired
                )
            })
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.active_sessions
            .retain(|_, session| session.expires_at >= now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_engine() -> TrustPolicyEngine {
        let mut engine = TrustPolicyEngine::new();

        // Add a basic governance rule
        let governance_rule = TrustPolicyRule {
            name: "governance_basic".to_string(),
            applicable_contexts: [TrustContext::Governance].into_iter().collect(),
            min_trust_level: TrustLevel::Partial,
            require_federation_membership: true,
            max_inheritance_depth: Some(2),
            allow_cross_federation: false,
            custom_validator: None,
        };
        engine.add_rule(governance_rule);

        // Add a resource sharing rule
        let resource_rule = TrustPolicyRule {
            name: "resource_sharing_basic".to_string(),
            applicable_contexts: [TrustContext::ResourceSharing].into_iter().collect(),
            min_trust_level: TrustLevel::Basic,
            require_federation_membership: false,
            max_inheritance_depth: Some(3),
            allow_cross_federation: true,
            custom_validator: None,
        };
        engine.add_rule(resource_rule);

        engine
    }

    #[test]
    fn test_trust_context_conversion() {
        let context = TrustContext::Governance;
        assert_eq!(context.as_str(), "governance");

        let parsed = TrustContext::from_str("governance");
        assert_eq!(parsed, TrustContext::Governance);

        let custom = TrustContext::from_str("custom_context");
        assert_eq!(custom, TrustContext::Custom("custom_context".to_string()));
    }

    #[test]
    fn test_trust_policy_engine_basic() {
        let engine = setup_test_engine();

        let trustor = Did::new("key", "alice");
        let trustee = Did::new("key", "bob");

        // Should be denied due to no trust relationship
        let result = engine.validate_trust(&trustor, &trustee, &TrustContext::Governance, "vote");

        match result {
            TrustValidationResult::Denied { reason } => {
                assert!(reason.contains("No valid trust relationship found"));
            }
            _ => panic!("Expected denial"),
        }
    }

    #[test]
    fn test_federation_membership() {
        let mut engine = setup_test_engine();

        let alice = Did::new("key", "alice");
        let federation = FederationId::new("test_federation".to_string());

        engine.add_federation_membership(alice.clone(), federation.clone());

        assert!(engine.is_federation_member(&alice, &federation));

        let bob = Did::new("key", "bob");
        assert!(!engine.is_federation_member(&bob, &federation));
    }

    #[test]
    fn test_trust_inheritance() {
        let inheritance = TrustInheritance::default();
        assert!(inheritance.inheritable);
        assert_eq!(inheritance.max_depth, Some(3));
        assert_eq!(inheritance.degradation_factor, 0.8);
        assert_eq!(inheritance.min_inherited_level, TrustLevel::Basic);
    }

    #[test]
    fn test_bridge_config() {
        let config = BridgeConfig::default();
        assert!(!config.bidirectional);
        assert!(config.allowed_contexts.contains(&TrustContext::General));
        assert_eq!(config.max_bridge_trust, TrustLevel::Basic);
        assert_eq!(config.bridge_degradation, 0.5);
    }
}
