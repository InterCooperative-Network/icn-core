//! Cooperative Identity Schemas for Federation Discovery
//!
//! This module defines standard schemas and credential types for cooperatives
//! to discover, trust, and collaborate with each other in the ICN federation.

use icn_common::{Cid, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard schema identifiers for cooperative credentials
pub mod schemas {
    use icn_common::Cid;

    /// Schema for cooperative membership credentials
    pub const COOPERATIVE_MEMBERSHIP: &str = "icn:cooperative:membership:v1";
    
    /// Schema for service provider credentials
    pub const SERVICE_PROVIDER: &str = "icn:cooperative:service_provider:v1";
    
    /// Schema for federation membership credentials
    pub const FEDERATION_MEMBERSHIP: &str = "icn:federation:membership:v1";
    
    /// Schema for cooperative profiles stored in DAG
    pub const COOPERATIVE_PROFILE: &str = "icn:cooperative:profile:v1";
    
    /// Schema for trust relationship attestations
    pub const TRUST_RELATIONSHIP: &str = "icn:federation:trust:v1";

    /// Generate CID for a schema
    pub fn schema_cid(schema_name: &str) -> Cid {
        Cid::new_v1_sha256(0x55, schema_name.as_bytes())
    }
}

/// Types of cooperatives in the ICN federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CooperativeType {
    /// Worker cooperatives owned and managed by their workers
    Worker,
    /// Consumer cooperatives owned by the people who use the services
    Consumer,
    /// Multi-stakeholder cooperatives with various types of members
    MultiStakeholder,
    /// Housing cooperatives for shared living
    Housing,
    /// Credit unions and financial cooperatives
    Financial,
    /// Platform cooperatives for digital services
    Platform,
    /// Agricultural cooperatives
    Agricultural,
    /// Research and education cooperatives
    Education,
    /// Energy cooperatives
    Energy,
    /// Healthcare cooperatives
    Healthcare,
    /// Community land trusts and commons management
    Commons,
    /// General purpose cooperative
    General,
}

/// Capability or service offered by a cooperative
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CooperativeCapability {
    /// Type of capability (e.g., "compute", "storage", "consultation", "housing")
    pub capability_type: String,
    /// Human-readable description
    pub description: String,
    /// Resource requirements or specifications
    pub specifications: HashMap<String, String>,
    /// Whether this capability is currently available
    pub available: bool,
    /// Pricing or exchange model (e.g., "time_bank", "mutual_credit", "mana")
    pub exchange_model: String,
}

/// Geographic scope of a cooperative's operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeographicScope {
    /// Country code (ISO 3166-1)
    pub country: Option<String>,
    /// State/province/region
    pub region: Option<String>,
    /// City or locality
    pub locality: Option<String>,
    /// Whether the cooperative operates globally
    pub global: bool,
}

/// Comprehensive profile of a cooperative stored in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativeProfile {
    /// DID of the cooperative
    pub did: Did,
    /// Human-readable name
    pub name: String,
    /// Type of cooperative
    pub cooperative_type: CooperativeType,
    /// Brief description of the cooperative's mission
    pub description: String,
    /// Website URL
    pub website: Option<String>,
    /// Contact information
    pub contact_email: Option<String>,
    /// Geographic scope of operations
    pub geographic_scope: GeographicScope,
    /// List of capabilities/services offered
    pub capabilities: Vec<CooperativeCapability>,
    /// Member count (approximate)
    pub member_count: Option<u32>,
    /// Year founded
    pub founded_year: Option<u32>,
    /// Legal structure information
    pub legal_structure: Option<String>,
    /// Federation memberships
    pub federation_memberships: Vec<String>,
    /// Trust relationships with other cooperatives
    pub trusted_cooperatives: Vec<Did>,
    /// Public keys for federation communication
    pub public_keys: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// Digital signature from the cooperative's DID
    pub signature: Option<String>,
}

impl CooperativeProfile {
    /// Create a new cooperative profile
    pub fn new(
        did: Did,
        name: String,
        cooperative_type: CooperativeType,
        description: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        
        Self {
            did,
            name,
            cooperative_type,
            description,
            website: None,
            contact_email: None,
            geographic_scope: GeographicScope {
                country: None,
                region: None,
                locality: None,
                global: false,
            },
            capabilities: Vec::new(),
            member_count: None,
            founded_year: None,
            legal_structure: None,
            federation_memberships: Vec::new(),
            trusted_cooperatives: Vec::new(),
            public_keys: HashMap::new(),
            created_at: now,
            updated_at: now,
            signature: None,
        }
    }

    /// Add a capability to the cooperative profile
    pub fn add_capability(&mut self, capability: CooperativeCapability) {
        self.capabilities.push(capability);
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    /// Add a trusted cooperative relationship
    pub fn add_trust_relationship(&mut self, cooperative_did: Did) {
        if !self.trusted_cooperatives.contains(&cooperative_did) {
            self.trusted_cooperatives.push(cooperative_did);
            self.updated_at = chrono::Utc::now().timestamp() as u64;
        }
    }

    /// Check if this cooperative offers a specific capability
    pub fn has_capability(&self, capability_type: &str) -> bool {
        self.capabilities
            .iter()
            .any(|cap| cap.capability_type == capability_type && cap.available)
    }

    /// Get available capabilities of a specific type
    pub fn get_capabilities(&self, capability_type: &str) -> Vec<&CooperativeCapability> {
        self.capabilities
            .iter()
            .filter(|cap| cap.capability_type == capability_type && cap.available)
            .collect()
    }
}

/// Helper for creating standard cooperative membership credentials
pub struct CooperativeMembershipBuilder {
    issuer: Did,
    holder: Did,
    cooperative_name: String,
    role: String,
    membership_level: String,
    joined_at: u64,
}

impl CooperativeMembershipBuilder {
    pub fn new(issuer: Did, holder: Did, cooperative_name: String) -> Self {
        Self {
            issuer,
            holder,
            cooperative_name,
            role: "member".to_string(),
            membership_level: "basic".to_string(),
            joined_at: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    pub fn with_membership_level(mut self, level: String) -> Self {
        self.membership_level = level;
        self
    }

    pub fn with_join_date(mut self, timestamp: u64) -> Self {
        self.joined_at = timestamp;
        self
    }

    pub fn build(self) -> HashMap<String, String> {
        let mut claims = HashMap::new();
        claims.insert("cooperative_name".to_string(), self.cooperative_name);
        claims.insert("role".to_string(), self.role);
        claims.insert("membership_level".to_string(), self.membership_level);
        claims.insert("joined_at".to_string(), self.joined_at.to_string());
        claims.insert("credential_type".to_string(), "cooperative_membership".to_string());
        claims
    }
}

/// Helper for creating service provider credentials
pub struct ServiceProviderBuilder {
    issuer: Did,
    holder: Did,
    service_types: Vec<String>,
    verified_at: u64,
}

impl ServiceProviderBuilder {
    pub fn new(issuer: Did, holder: Did, service_types: Vec<String>) -> Self {
        Self {
            issuer,
            holder,
            service_types,
            verified_at: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn build(self) -> HashMap<String, String> {
        let mut claims = HashMap::new();
        claims.insert("service_types".to_string(), self.service_types.join(","));
        claims.insert("verified_at".to_string(), self.verified_at.to_string());
        claims.insert("credential_type".to_string(), "service_provider".to_string());
        claims
    }
}

/// Trust level between cooperatives
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustLevel {
    /// Full trust - can share sensitive information and resources
    Full,
    /// Partial trust - limited cooperation and resource sharing
    Partial,
    /// Basic trust - basic information sharing and public collaboration
    Basic,
    /// No trust - no cooperation
    None,
}

/// Trust relationship between two cooperatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRelationship {
    /// The cooperative making the trust attestation
    pub attestor: Did,
    /// The cooperative being trusted
    pub subject: Did,
    /// Level of trust
    pub trust_level: TrustLevel,
    /// Scope of trust (e.g., "mesh_computing", "governance", "financial")
    pub trust_scope: Vec<String>,
    /// Human-readable justification
    pub justification: Option<String>,
    /// Timestamp when trust was established
    pub established_at: u64,
    /// Expiration timestamp (if any)
    pub expires_at: Option<u64>,
    /// Whether this trust relationship is reciprocal
    pub reciprocal: bool,
}

impl TrustRelationship {
    pub fn new(
        attestor: Did,
        subject: Did,
        trust_level: TrustLevel,
        trust_scope: Vec<String>,
    ) -> Self {
        Self {
            attestor,
            subject,
            trust_level,
            trust_scope,
            justification: None,
            established_at: chrono::Utc::now().timestamp() as u64,
            expires_at: None,
            reciprocal: false,
        }
    }

    pub fn with_justification(mut self, justification: String) -> Self {
        self.justification = Some(justification);
        self
    }

    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_reciprocal(mut self, reciprocal: bool) -> Self {
        self.reciprocal = reciprocal;
        self
    }

    /// Check if this trust relationship is valid (not expired)
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            (chrono::Utc::now().timestamp() as u64) < expires_at
        } else {
            true
        }
    }

    /// Check if this trust covers a specific scope
    pub fn covers_scope(&self, scope: &str) -> bool {
        self.trust_scope.contains(&scope.to_string()) || self.trust_scope.contains(&"*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooperative_profile_creation() {
        let did = Did::new("key", "test_coop");
        let profile = CooperativeProfile::new(
            did.clone(),
            "Test Cooperative".to_string(),
            CooperativeType::Worker,
            "A test cooperative for unit testing".to_string(),
        );

        assert_eq!(profile.did, did);
        assert_eq!(profile.name, "Test Cooperative");
        assert_eq!(profile.cooperative_type, CooperativeType::Worker);
        assert!(profile.capabilities.is_empty());
    }

    #[test]
    fn test_capability_management() {
        let did = Did::new("key", "test_coop");
        let mut profile = CooperativeProfile::new(
            did,
            "Test Cooperative".to_string(),
            CooperativeType::Platform,
            "A test cooperative".to_string(),
        );

        let capability = CooperativeCapability {
            capability_type: "web_development".to_string(),
            description: "Custom web application development".to_string(),
            specifications: HashMap::new(),
            available: true,
            exchange_model: "time_bank".to_string(),
        };

        profile.add_capability(capability);
        assert!(profile.has_capability("web_development"));
        assert!(!profile.has_capability("farming"));
        assert_eq!(profile.get_capabilities("web_development").len(), 1);
    }

    #[test]
    fn test_trust_relationship() {
        let coop_a = Did::new("key", "coop_a");
        let coop_b = Did::new("key", "coop_b");

        let trust = TrustRelationship::new(
            coop_a,
            coop_b,
            TrustLevel::Partial,
            vec!["mesh_computing".to_string()],
        )
        .with_justification("Successful collaboration on previous projects".to_string());

        assert!(trust.is_valid());
        assert!(trust.covers_scope("mesh_computing"));
        assert!(!trust.covers_scope("financial"));
    }

    #[test]
    fn test_membership_credential_builder() {
        let issuer = Did::new("key", "test_coop");
        let holder = Did::new("key", "alice");

        let claims = CooperativeMembershipBuilder::new(
            issuer,
            holder,
            "Test Cooperative".to_string(),
        )
        .with_role("worker_owner".to_string())
        .with_membership_level("verified".to_string())
        .build();

        assert_eq!(claims.get("cooperative_name").unwrap(), "Test Cooperative");
        assert_eq!(claims.get("role").unwrap(), "worker_owner");
        assert_eq!(claims.get("membership_level").unwrap(), "verified");
        assert_eq!(claims.get("credential_type").unwrap(), "cooperative_membership");
    }
} 