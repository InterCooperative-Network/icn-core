//! Verifiable Credentials Implementation for ICN Identity Protocol
//!
//! This module implements W3C Verifiable Credentials with ICN-specific extensions
//! for membership credentials, resource tokens, and privacy-preserving verification.

use base64::Engine;
use icn_common::{CommonError, Did, Signable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// W3C Verifiable Credential with ICN extensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiableCredential {
    /// W3C context
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Credential identifier
    pub id: String,

    /// Credential types
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,

    /// Credential issuer
    pub issuer: CredentialIssuer,

    /// Issuance date
    pub issuance_date: String,

    /// Optional expiration date
    pub expiration_date: Option<String>,

    /// Credential subject
    pub credential_subject: CredentialSubject,

    /// ICN-specific metadata
    pub icn_metadata: CredentialMetadata,

    /// Cryptographic proof
    pub proof: CredentialProof,
}

/// Credential issuer information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialIssuer {
    /// Issuer DID
    pub id: Did,

    /// Issuer name
    pub name: Option<String>,

    /// Issuer type
    pub issuer_type: IssuerType,
}

/// Types of credential issuers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssuerType {
    Organization,
    Federation,
    System, // For system-issued credentials
    Individual,
}

/// Credential subject containing claims
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// Subject DID
    pub id: Did,

    /// Claims about the subject
    pub claims: HashMap<String, serde_json::Value>,
}

/// ICN-specific credential metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// Credential category
    pub category: CredentialCategory,

    /// Whether this credential is transferable
    pub transferable: bool,

    /// Whether this credential grants voting rights
    pub grants_voting_rights: bool,

    /// Revocation registry identifier
    pub revocation_registry: Option<String>,

    /// Privacy level
    pub privacy_level: PrivacyLevel,

    /// Economic value if applicable
    pub economic_value: Option<EconomicValue>,

    /// Dependency requirements
    pub dependencies: Vec<CredentialDependency>,
}

/// Categories of credentials in ICN
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialCategory {
    /// Membership in organization (non-transferable)
    Membership {
        organization_id: Did,
        role: String,
        permissions: Vec<String>,
    },

    /// Resource provision capability
    ResourceProvider {
        resource_types: Vec<String>,
        capacity: HashMap<String, u64>,
    },

    /// Trust attestation
    TrustAttestation {
        trust_level: i32, // Changed from f64 to i32 for Eq
        attestation_type: String,
        valid_until: u64,
    },

    /// Skill or capability certification
    Capability {
        skill_type: String,
        certification_level: String,
        certifying_body: Did,
    },

    /// Identity verification
    IdentityVerification {
        verification_method: String,
        verification_level: String,
        verified_attributes: Vec<String>,
    },

    /// Economic token representation
    TokenCredential {
        token_class: String,
        amount: u64,
        token_type: TokenType,
    },
}

/// Types of tokens in credential form
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Resource,
    Service,
    Labour,
    MutualCredit,
    Membership, // Always non-transferable
}

/// Privacy levels for credentials
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Fully public credential
    Public,

    /// Publicly verifiable but subject identity can be hidden
    Pseudonymous,

    /// Requires zero-knowledge proof for verification
    ZeroKnowledge,

    /// Completely private, only holder can present
    Private,
}

/// Economic value associated with credentials
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicValue {
    /// Value in mana
    pub mana_value: u64,

    /// Alternative token values
    pub token_values: HashMap<String, u64>,

    /// Whether value decays over time (demurrage)
    pub has_demurrage: bool,

    /// Demurrage rate if applicable (as basis points, e.g., 100 = 1%)
    pub demurrage_rate: Option<u32>, // Changed from f64 to u32 for Eq
}

/// Dependencies between credentials
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialDependency {
    /// Required credential type
    pub required_credential: String,

    /// Required issuer
    pub required_issuer: Option<Did>,

    /// Whether dependency must be current
    pub must_be_current: bool,
}

/// Cryptographic proof for credentials
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProof {
    /// Proof type
    #[serde(rename = "type")]
    pub proof_type: String,

    /// Proof purpose
    pub proof_purpose: String,

    /// Verification method used for proof
    pub verification_method: String,

    /// Proof creation timestamp
    pub created: String,

    /// Signature or proof value
    pub proof_value: String,

    /// Additional proof metadata
    pub proof_metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Presentation of verifiable credentials
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    /// W3C context
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Presentation ID
    pub id: String,

    /// Presentation type
    #[serde(rename = "type")]
    pub presentation_type: Vec<String>,

    /// Credentials included in presentation
    pub verifiable_credential: Vec<VerifiableCredential>,

    /// Presenter DID
    pub holder: Did,

    /// Presentation proof
    pub proof: CredentialProof,
}

/// Selective disclosure proof for privacy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectiveDisclosureProof {
    /// Fields being disclosed
    pub disclosed_fields: Vec<String>,

    /// Zero-knowledge proof of undisclosed fields
    pub zk_proof: String,

    /// Salt values for disclosed fields
    pub salts: HashMap<String, String>,
}

/// Credential revocation information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRevocation {
    /// Credential ID being revoked
    pub credential_id: String,

    /// Revocation reason
    pub reason: String,

    /// Who revoked it
    pub revoked_by: Did,

    /// When it was revoked
    pub revoked_at: u64,

    /// Revocation proof
    pub revocation_proof: CredentialProof,
}

impl VerifiableCredential {
    /// Create a new membership credential (non-transferable)
    pub fn new_membership_credential(
        issuer: Did,
        subject: Did,
        organization_id: Did,
        role: String,
    ) -> Result<Self, CommonError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let credential_id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
        let issuance_date = format_timestamp(now);

        let mut claims = HashMap::new();
        claims.insert("role".to_string(), serde_json::Value::String(role.clone()));
        claims.insert(
            "organizationId".to_string(),
            serde_json::Value::String(organization_id.to_string()),
        );
        claims.insert(
            "grantedAt".to_string(),
            serde_json::Value::Number(serde_json::Number::from(now)),
        );

        Ok(Self {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://icn.network/credentials/v1".to_string(),
            ],
            id: credential_id,
            credential_type: vec![
                "VerifiableCredential".to_string(),
                "MembershipCredential".to_string(),
            ],
            issuer: CredentialIssuer {
                id: issuer,
                name: None,
                issuer_type: IssuerType::Organization,
            },
            issuance_date: issuance_date.clone(),
            expiration_date: None,
            credential_subject: CredentialSubject {
                id: subject,
                claims,
            },
            icn_metadata: CredentialMetadata {
                category: CredentialCategory::Membership {
                    organization_id,
                    role,
                    permissions: vec!["vote".to_string()],
                },
                transferable: false, // Membership is soul-bound
                grants_voting_rights: true,
                revocation_registry: None,
                privacy_level: PrivacyLevel::Public,
                economic_value: None,
                dependencies: Vec::new(),
            },
            proof: CredentialProof {
                proof_type: "Ed25519Signature2020".to_string(),
                proof_purpose: "assertionMethod".to_string(),
                verification_method: "".to_string(), // To be filled during signing
                created: issuance_date.clone(),
                proof_value: "".to_string(), // To be filled during signing
                proof_metadata: None,
            },
        })
    }

    /// Create a new resource provider credential
    pub fn new_resource_credential(
        issuer: Did,
        subject: Did,
        resource_types: Vec<String>,
        capacity: HashMap<String, u64>,
    ) -> Result<Self, CommonError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let credential_id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
        let issuance_date = format_timestamp(now);

        let mut claims = HashMap::new();
        claims.insert(
            "resourceTypes".to_string(),
            serde_json::Value::Array(
                resource_types
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
        claims.insert(
            "capacity".to_string(),
            serde_json::to_value(&capacity).map_err(|e| CommonError::SerError(e.to_string()))?,
        );

        Ok(Self {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://icn.network/credentials/v1".to_string(),
            ],
            id: credential_id,
            credential_type: vec![
                "VerifiableCredential".to_string(),
                "ResourceProviderCredential".to_string(),
            ],
            issuer: CredentialIssuer {
                id: issuer,
                name: None,
                issuer_type: IssuerType::System,
            },
            issuance_date: issuance_date.clone(),
            expiration_date: None,
            credential_subject: CredentialSubject {
                id: subject,
                claims,
            },
            icn_metadata: CredentialMetadata {
                category: CredentialCategory::ResourceProvider {
                    resource_types,
                    capacity,
                },
                transferable: false, // Resource credentials are tied to hardware
                grants_voting_rights: false,
                revocation_registry: None,
                privacy_level: PrivacyLevel::Public,
                economic_value: None,
                dependencies: Vec::new(),
            },
            proof: CredentialProof {
                proof_type: "Ed25519Signature2020".to_string(),
                proof_purpose: "assertionMethod".to_string(),
                verification_method: "".to_string(),
                created: issuance_date.clone(),
                proof_value: "".to_string(),
                proof_metadata: None,
            },
        })
    }

    /// Validate credential structure and constraints
    pub fn validate(&self) -> Result<(), CommonError> {
        // Check required fields
        if self.id.is_empty() {
            return Err(CommonError::ValidationError(
                "Credential ID cannot be empty".to_string(),
            ));
        }

        // Check credential types
        if !self
            .credential_type
            .contains(&"VerifiableCredential".to_string())
        {
            return Err(CommonError::ValidationError(
                "Must include VerifiableCredential type".to_string(),
            ));
        }

        // Validate ICN-specific constraints
        match &self.icn_metadata.category {
            CredentialCategory::Membership { .. } => {
                if self.icn_metadata.transferable {
                    return Err(CommonError::ValidationError(
                        "Membership credentials cannot be transferable".to_string(),
                    ));
                }
            }
            CredentialCategory::TokenCredential {
                token_type: TokenType::Membership,
                ..
            } => {
                if self.icn_metadata.transferable {
                    return Err(CommonError::ValidationError(
                        "Membership tokens cannot be transferable".to_string(),
                    ));
                }
            }
            CredentialCategory::TokenCredential { .. } => {}
            _ => {}
        }

        // Check dependencies
        for dependency in &self.icn_metadata.dependencies {
            if dependency.required_credential.is_empty() {
                return Err(CommonError::ValidationError(
                    "Dependency credential type cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if credential grants voting rights
    pub fn grants_voting_rights(&self) -> bool {
        self.icn_metadata.grants_voting_rights
    }

    /// Check if credential is transferable
    pub fn is_transferable(&self) -> bool {
        self.icn_metadata.transferable
    }

    /// Get credential subject DID
    pub fn subject(&self) -> &Did {
        &self.credential_subject.id
    }

    /// Get credential issuer DID
    pub fn issuer(&self) -> &Did {
        &self.issuer.id
    }

    /// Check if credential is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = &self.expiration_date {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if let Ok(exp_time) = parse_timestamp(expiration) {
                return now > exp_time;
            }
        }
        false
    }

    /// Create a selective disclosure presentation
    pub fn create_selective_disclosure(
        &self,
        disclosed_fields: Vec<String>,
    ) -> Result<SelectiveDisclosureProof, CommonError> {
        // Generate salts for disclosed fields
        let mut salts = HashMap::new();
        for field in &disclosed_fields {
            salts.insert(field.clone(), generate_salt());
        }

        // TODO: Implement actual ZK proof generation
        let zk_proof = "placeholder_zk_proof".to_string();

        Ok(SelectiveDisclosureProof {
            disclosed_fields,
            zk_proof,
            salts,
        })
    }
}

impl Signable for VerifiableCredential {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        // Create canonical representation for signing
        let mut bytes = Vec::new();

        bytes.extend_from_slice(self.id.as_bytes());
        bytes.extend_from_slice(self.issuance_date.as_bytes());

        if let Some(exp) = &self.expiration_date {
            bytes.extend_from_slice(exp.as_bytes());
        }

        bytes.extend_from_slice(self.issuer.id.to_string().as_bytes());
        bytes.extend_from_slice(self.credential_subject.id.to_string().as_bytes());

        // Add claims in sorted order for determinism
        let mut sorted_claims: Vec<_> = self.credential_subject.claims.iter().collect();
        sorted_claims.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_claims {
            bytes.extend_from_slice(key.as_bytes());
            let value_str =
                serde_json::to_string(value).map_err(|e| CommonError::SerError(e.to_string()))?;
            bytes.extend_from_slice(value_str.as_bytes());
        }

        Ok(bytes)
    }
}

// Helper functions

fn format_timestamp(timestamp: u64) -> String {
    // Convert to ISO 8601 format
    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn parse_timestamp(timestamp_str: &str) -> Result<u64, CommonError> {
    chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.timestamp() as u64)
        .map_err(|e| CommonError::ValidationError(format!("Invalid timestamp: {}", e)))
}

fn generate_salt() -> String {
    use rand::Rng;
    let salt: [u8; 32] = rand::thread_rng().gen();
    // Use base64 encoding
    base64::engine::general_purpose::STANDARD.encode(salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    use std::str::FromStr;

    fn test_did(id: &str) -> Did {
        Did::from_str(&format!("did:icn:person:{}", id)).unwrap()
    }

    fn test_org_did(id: &str) -> Did {
        Did::from_str(&format!("did:icn:organization:{}", id)).unwrap()
    }

    #[test]
    fn test_membership_credential_creation() {
        let issuer = test_org_did("coop1");
        let subject = test_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer.clone(),
            subject.clone(),
            org_id.clone(),
            "member".to_string(),
        )
        .unwrap();

        assert_eq!(credential.issuer.id, issuer);
        assert_eq!(credential.credential_subject.id, subject);
        assert!(!credential.icn_metadata.transferable);
        assert!(credential.icn_metadata.grants_voting_rights);
    }

    #[test]
    fn test_resource_credential_creation() {
        let issuer = test_did("system");
        let subject = test_did("node1");
        let resource_types = vec!["cpu".to_string(), "memory".to_string()];
        let mut capacity = HashMap::new();
        capacity.insert("cpu_cores".to_string(), 8);
        capacity.insert("memory_gb".to_string(), 32);

        let credential = VerifiableCredential::new_resource_credential(
            issuer.clone(),
            subject.clone(),
            resource_types.clone(),
            capacity.clone(),
        )
        .unwrap();

        assert_eq!(credential.issuer.id, issuer);
        assert_eq!(credential.credential_subject.id, subject);
        assert!(!credential.icn_metadata.transferable);
        assert!(!credential.icn_metadata.grants_voting_rights);

        if let CredentialCategory::ResourceProvider {
            resource_types: rt,
            capacity: cap,
        } = &credential.icn_metadata.category
        {
            assert_eq!(*rt, resource_types);
            assert_eq!(*cap, capacity);
        } else {
            panic!("Wrong credential category");
        }
    }

    #[test]
    fn test_credential_validation() {
        let issuer = test_org_did("coop1");
        let subject = test_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        )
        .unwrap();

        assert!(credential.validate().is_ok());
    }

    #[test]
    fn test_credential_voting_rights() {
        let issuer = test_org_did("coop1");
        let subject = test_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        )
        .unwrap();

        assert!(credential.grants_voting_rights());
        assert!(!credential.is_transferable());
    }

    #[test]
    fn test_credential_expiration() {
        let issuer = test_org_did("coop1");
        let subject = test_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        )
        .unwrap();

        // Should not be expired without expiration date
        assert!(!credential.is_expired());
    }

    #[test]
    fn test_signable_implementation() {
        let issuer = test_org_did("coop1");
        let subject = test_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        )
        .unwrap();

        let bytes = credential.to_signable_bytes().unwrap();
        assert!(!bytes.is_empty());
    }
}
