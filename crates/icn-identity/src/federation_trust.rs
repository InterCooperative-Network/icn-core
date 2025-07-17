//! Federation Trust Framework
//!
//! This module implements the Scoped Federation Trust Framework as specified in requirement 2.2.
//! It provides different trust levels for different activities, trust inheritance models,
//! cross-federation trust bridges, and a configurable trust policy engine.

use crate::cooperative_schemas::{TrustLevel, TrustRelationship};
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

impl TrustPolicyEngine {
    /// Create a new trust policy engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a policy rule
    pub fn add_rule(&mut self, rule: TrustPolicyRule) {
        for context in &rule.applicable_contexts {
            self.rules.entry(context.clone()).or_default().push(rule.clone());
        }
    }

    /// Add federation membership for a DID
    pub fn add_federation_membership(&mut self, did: Did, federation: FederationId) {
        self.memberships.entry(did).or_default().insert(federation);
    }

    /// Add a scoped trust relationship to a federation
    pub fn add_federation_trust(&mut self, federation: FederationId, trust: ScopedTrustRelationship) {
        self.federation_trusts.entry(federation).or_default().push(trust);
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
                    reason: format!("No policy rules defined for context {:?}", context),
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
        for (_, trusts) in &self.federation_trusts {
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
                                    trust_path: vec![
                                        format!("federation_inheritance:{}", trustor_fed.as_str())
                                    ],
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
                if let Some(bridge) = self.bridges.get(&(trustor_fed.clone(), trustee_fed.clone())) {
                    if bridge.bridge_config.allowed_contexts.contains(context) {
                        // Calculate bridged trust level
                        let bridged_level = self.calculate_bridged_trust_level(
                            &bridge.trust.base.trust_level,
                            &bridge.bridge_config,
                        );
                        
                        return Some(TrustValidationResult::Allowed {
                            effective_trust: bridged_level,
                            trust_path: vec![
                                format!("bridge:{}→{}", trustor_fed.as_str(), trustee_fed.as_str())
                            ],
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
        let result = engine.validate_trust(
            &trustor,
            &trustee,
            &TrustContext::Governance,
            "vote"
        );
        
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