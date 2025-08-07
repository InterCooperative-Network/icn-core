//! W3C-Compliant DID Document Implementation for ICN Identity Protocol
//!
//! This module implements the full DID Document specification according to the
//! ICN Identity Protocol and W3C DID Core 1.0 specification.

use icn_common::{CommonError, Did, Signable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// W3C-compliant DID Document structure with ICN-specific extensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidDocument {
    /// The identifier this document describes
    pub id: Did,

    /// Controllers who can update this DID document
    pub controller: Vec<Did>,

    /// Cryptographic material for verification
    pub verification_method: Vec<VerificationMethod>,

    /// Authentication capabilities
    pub authentication: Vec<String>,

    /// Assertion method references
    pub assertion_method: Vec<String>,

    /// Key agreement references for encryption
    pub key_agreement: Vec<String>,

    /// Capability invocation references
    pub capability_invocation: Vec<String>,

    /// Capability delegation references  
    pub capability_delegation: Vec<String>,

    /// Service endpoints
    pub service: Vec<ServiceEndpoint>,

    /// ICN-specific metadata
    pub icn_metadata: IcnMetadata,

    /// Document versioning
    pub version: u64,

    /// Creation timestamp
    pub created: u64,

    /// Last updated timestamp
    pub updated: u64,

    /// Cryptographic proof
    pub proof: Option<DocumentProof>,
}

/// Verification method for cryptographic operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Unique identifier for this verification method
    pub id: String,

    /// Type of verification method
    pub method_type: VerificationMethodType,

    /// Controller of this verification method
    pub controller: Did,

    /// Public key material
    pub public_key: PublicKeyMaterial,

    /// Creation timestamp
    pub created: u64,

    /// Optional expiration timestamp
    pub expires: Option<u64>,

    /// Revocation information if revoked
    pub revoked: Option<RevocationInfo>,
}

/// Types of verification methods supported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethodType {
    Ed25519VerificationKey2020,
    X25519KeyAgreementKey2020,
    Secp256k1VerificationKey2018,
    RsaVerificationKey2018,
    JsonWebKey2020,
}

/// Public key material for different key types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKeyMaterial {
    Ed25519 {
        #[serde(with = "serde_bytes")]
        public_key_bytes: Vec<u8>,
    },
    X25519 {
        #[serde(with = "serde_bytes")]
        public_key_bytes: Vec<u8>,
    },
    Secp256k1 {
        #[serde(with = "serde_bytes")]
        public_key_bytes: Vec<u8>,
    },
    Rsa {
        modulus: String,
        exponent: String,
    },
    Jwk {
        jwk: serde_json::Value,
    },
}

/// Service endpoint for external services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Unique identifier for the service
    pub id: String,

    /// Type of service
    pub service_type: String,

    /// Service endpoint URL
    pub service_endpoint: String,

    /// Additional service properties
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// ICN-specific metadata extensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IcnMetadata {
    /// Type of entity (person, organization, device, etc.)
    pub identity_type: IdentityType,

    /// Federation memberships
    pub federation_memberships: Vec<FederationMembership>,

    /// Organization memberships
    pub organization_memberships: Vec<OrganizationMembership>,

    /// Trust score if applicable  
    pub trust_score: Option<i32>, // Changed from f64 to i32 for Eq

    /// Sybil resistance data
    pub sybil_resistance: SybildResistanceData,

    /// Recovery configuration
    pub recovery_config: Option<RecoveryConfig>,
}

/// Types of identities in ICN
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityType {
    Person,
    Organization,
    Device,
    Service,
    Ephemeral,
}

/// Federation membership information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationMembership {
    /// Federation DID
    pub federation_id: Did,

    /// Role within the federation
    pub role: FederationRole,

    /// Membership status
    pub status: MembershipStatus,

    /// When membership was granted
    pub granted_at: u64,

    /// Optional expiration
    pub expires_at: Option<u64>,
}

/// Organization membership information  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationMembership {
    /// Organization DID
    pub organization_id: Did,

    /// Role within organization
    pub role: String,

    /// Voting rights granted by this membership
    pub voting_rights: bool,

    /// When membership was granted
    pub granted_at: u64,

    /// Optional expiration
    pub expires_at: Option<u64>,
}

/// Roles within a federation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FederationRole {
    Member,
    Validator,
    Coordinator,
    Observer,
}

/// Status of membership
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipStatus {
    Active,
    Pending,
    Suspended,
    Revoked,
}

/// Sybil resistance mechanisms data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SybildResistanceData {
    /// Mana spent to create this DID
    pub creation_mana_cost: u64,

    /// Proof of personhood type if applicable
    pub proof_of_personhood: Option<ProofOfPersonhood>,

    /// Rate limiting status
    pub rate_limit_status: RateLimitStatus,

    /// Creation timestamp for rate limiting
    pub created_at: u64,
}

/// Types of proof of personhood
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofOfPersonhood {
    /// Social vouching by existing members
    SocialVouching { vouchers: Vec<Did>, threshold: u32 },
    /// Biometric verification (privacy-preserving)
    BiometricHash {
        hash: String,
        verification_method: String,
    },
    /// External identity verification
    ExternalVerification {
        verifier: String,
        verification_id: String,
    },
}

/// Rate limiting status for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Operations performed in current epoch
    pub operations_this_epoch: u32,

    /// Maximum operations allowed per epoch
    pub max_operations_per_epoch: u32,

    /// Current epoch start time
    pub epoch_start: u64,

    /// Epoch duration in seconds
    pub epoch_duration: u64,
}

/// Recovery configuration for key rotation and account recovery
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Social recovery guardians
    pub social_guardians: Vec<Did>,

    /// Threshold of guardians needed for recovery
    pub guardian_threshold: u32,

    /// Backup key for recovery
    pub backup_key: Option<String>,

    /// Recovery delay period in seconds
    pub recovery_delay: u64,
}

/// Revocation information for verification methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationInfo {
    /// When the key was revoked
    pub revoked_at: u64,

    /// Reason for revocation
    pub reason: String,

    /// Who revoked it
    pub revoked_by: Did,
}

/// Cryptographic proof for DID document integrity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentProof {
    /// Proof type
    pub proof_type: String,

    /// Creation method/algorithm
    pub proof_purpose: String,

    /// Verification method used
    pub verification_method: String,

    /// Timestamp of proof creation
    pub created: u64,

    /// Signature bytes
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

impl DidDocument {
    /// Create a new DID document with minimal required fields
    pub fn new(id: Did, controller: Did) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            controller: vec![controller],
            verification_method: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: Vec::new(),
            icn_metadata: IcnMetadata {
                identity_type: IdentityType::Person,
                federation_memberships: Vec::new(),
                organization_memberships: Vec::new(),
                trust_score: None,
                sybil_resistance: SybildResistanceData {
                    creation_mana_cost: 10, // Default mana cost
                    proof_of_personhood: None,
                    rate_limit_status: RateLimitStatus {
                        operations_this_epoch: 0,
                        max_operations_per_epoch: 100,
                        epoch_start: now,
                        epoch_duration: 3600, // 1 hour
                    },
                    created_at: now,
                },
                recovery_config: None,
            },
            version: 1,
            created: now,
            updated: now,
            proof: None,
        }
    }

    /// Add a verification method to the DID document
    pub fn add_verification_method(&mut self, method: VerificationMethod) {
        self.verification_method.push(method);
        self.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.version += 1;
    }

    /// Add authentication capability
    pub fn add_authentication(&mut self, method_id: String) {
        self.authentication.push(method_id);
        self.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.version += 1;
    }

    /// Add service endpoint
    pub fn add_service(&mut self, service: ServiceEndpoint) {
        self.service.push(service);
        self.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.version += 1;
    }

    /// Check if DID has sufficient mana for rate limiting
    pub fn check_rate_limit(&self) -> Result<(), CommonError> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let rate_limit = &self.icn_metadata.sybil_resistance.rate_limit_status;

        // Check if we're in a new epoch
        if current_time >= rate_limit.epoch_start + rate_limit.epoch_duration {
            return Ok(()); // New epoch, rate limit reset
        }

        // Check if under rate limit
        if rate_limit.operations_this_epoch >= rate_limit.max_operations_per_epoch {
            return Err(CommonError::RateLimitError(
                "DID operation rate limit exceeded".to_string(),
            ));
        }

        Ok(())
    }

    /// Increment operation count for rate limiting
    pub fn increment_operation_count(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let rate_limit = &mut self.icn_metadata.sybil_resistance.rate_limit_status;

        // Reset if in new epoch
        if current_time >= rate_limit.epoch_start + rate_limit.epoch_duration {
            rate_limit.operations_this_epoch = 0;
            rate_limit.epoch_start = current_time;
        }

        rate_limit.operations_this_epoch += 1;
    }

    /// Validate DID document structure and constraints
    pub fn validate(&self) -> Result<(), CommonError> {
        // Basic validation
        if self.verification_method.is_empty() {
            return Err(CommonError::ValidationError(
                "DID document must have at least one verification method".to_string(),
            ));
        }

        // Validate verification method references
        let method_ids: std::collections::HashSet<String> = self
            .verification_method
            .iter()
            .map(|m| m.id.clone())
            .collect();

        for auth_ref in &self.authentication {
            if !method_ids.contains(auth_ref) {
                return Err(CommonError::ValidationError(format!(
                    "Authentication reference '{}' not found in verification methods",
                    auth_ref
                )));
            }
        }

        // Check rate limiting constraints
        self.check_rate_limit()?;

        Ok(())
    }
}

impl Signable for DidDocument {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        // Create canonical representation for signing
        let mut bytes = Vec::new();

        // Core fields
        bytes.extend_from_slice(self.id.to_string().as_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.created.to_le_bytes());
        bytes.extend_from_slice(&self.updated.to_le_bytes());

        // Controllers
        let mut controllers: Vec<String> = self.controller.iter().map(|c| c.to_string()).collect();
        controllers.sort();
        for controller in controllers {
            bytes.extend_from_slice(controller.as_bytes());
        }

        // Verification methods (sorted by ID for determinism)
        let mut methods = self.verification_method.clone();
        methods.sort_by(|a, b| a.id.cmp(&b.id));

        for method in methods {
            bytes.extend_from_slice(method.id.as_bytes());
            match method.public_key {
                PublicKeyMaterial::Ed25519 { public_key_bytes } => {
                    bytes.extend_from_slice(&public_key_bytes);
                }
                PublicKeyMaterial::X25519 { public_key_bytes } => {
                    bytes.extend_from_slice(&public_key_bytes);
                }
                PublicKeyMaterial::Secp256k1 { public_key_bytes } => {
                    bytes.extend_from_slice(&public_key_bytes);
                }
                PublicKeyMaterial::Rsa { modulus, exponent } => {
                    bytes.extend_from_slice(modulus.as_bytes());
                    bytes.extend_from_slice(exponent.as_bytes());
                }
                PublicKeyMaterial::Jwk { jwk } => {
                    let jwk_str = serde_json::to_string(&jwk)
                        .map_err(|e| CommonError::SerError(e.to_string()))?;
                    bytes.extend_from_slice(jwk_str.as_bytes());
                }
            }
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    use std::str::FromStr;

    fn test_did() -> Did {
        Did::from_str("did:icn:person:alice").unwrap()
    }

    #[test]
    fn test_did_document_creation() {
        let did = test_did();
        let controller = test_did();
        let doc = DidDocument::new(did.clone(), controller);

        assert_eq!(doc.id, did);
        assert_eq!(doc.version, 1);
        assert!(doc.verification_method.is_empty());
    }

    #[test]
    fn test_add_verification_method() {
        let did = test_did();
        let controller = test_did();
        let mut doc = DidDocument::new(did.clone(), controller);

        let method = VerificationMethod {
            id: format!("{}#key1", did),
            method_type: VerificationMethodType::Ed25519VerificationKey2020,
            controller: did.clone(),
            public_key: PublicKeyMaterial::Ed25519 {
                public_key_bytes: vec![1, 2, 3, 4],
            },
            created: doc.created,
            expires: None,
            revoked: None,
        };

        doc.add_verification_method(method);
        assert_eq!(doc.verification_method.len(), 1);
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_rate_limiting() {
        let did = test_did();
        let controller = test_did();
        let mut doc = DidDocument::new(did, controller);

        // Should be OK initially
        assert!(doc.check_rate_limit().is_ok());

        // Max out the rate limit
        doc.icn_metadata
            .sybil_resistance
            .rate_limit_status
            .operations_this_epoch = 100;

        // Should now fail
        assert!(doc.check_rate_limit().is_err());
    }

    #[test]
    fn test_document_validation() {
        let did = test_did();
        let controller = test_did();
        let doc = DidDocument::new(did, controller);

        // Should fail validation - no verification methods
        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_signable_implementation() {
        let did = test_did();
        let controller = test_did();
        let doc = DidDocument::new(did, controller);

        let bytes = doc.to_signable_bytes().unwrap();
        assert!(!bytes.is_empty());
    }
}
