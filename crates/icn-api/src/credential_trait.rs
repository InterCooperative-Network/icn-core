// crates/icn-api/src/credential_trait.rs
//! HTTP API endpoints for credential lifecycle management

// use crate::types::ResponseEnvelope; // Temporarily removed until types module is available
use icn_common::{Cid, Did, ZkCredentialProof};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to issue a new credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCredentialRequest {
    pub credential_type: String,
    pub holder: Did,
    pub issuer: Did,
    pub claims: HashMap<String, serde_json::Value>,
    pub evidence: Option<Vec<String>>,
    pub validity_period: Option<u64>,
}

/// Response from credential issuance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCredentialResponse {
    pub credential_cid: Cid,
    pub credential_proof: ZkCredentialProof,
    pub issued_at: u64,
    pub valid_until: Option<u64>,
}

/// Request to present a credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentCredentialRequest {
    pub credential_proof: ZkCredentialProof,
    pub context: String,
    pub disclosed_fields: Vec<String>,
    pub challenge: Option<String>,
}

/// Response from credential presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentCredentialResponse {
    pub presentation_id: String,
    pub verification_result: CredentialVerificationResult,
    pub timestamp: u64,
}

/// Request to verify a credential presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCredentialRequest {
    pub presentation_id: String,
    pub verification_level: String,
    pub required_claims: Option<Vec<String>>,
}

/// Result of credential verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialVerificationResult {
    pub valid: bool,
    pub verification_level: String,
    pub verified_claims: HashMap<String, serde_json::Value>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub trust_score: Option<f64>,
}

/// Request to anchor a credential disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorDisclosureRequest {
    pub credential_cid: Cid,
    pub disclosed_fields: Vec<String>,
    pub presentation_context: String,
    pub verifier: Did,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Response from anchoring a disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorDisclosureResponse {
    pub disclosure_cid: Cid,
    pub anchored_at: u64,
    pub dag_block_cid: Cid,
}

/// Request to revoke a credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeCredentialRequest {
    pub credential_cid: Cid,
    pub reason: String,
    pub revoked_by: Did,
}

/// Response from credential revocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeCredentialResponse {
    pub revoked: bool,
    pub revocation_cid: Cid,
    pub revoked_at: u64,
}

/// Request to list credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCredentialsRequest {
    pub holder: Option<Did>,
    pub issuer: Option<Did>,
    pub credential_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Credential metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub cid: Cid,
    pub issuer: Did,
    pub holder: Did,
    pub credential_type: String,
    pub issued_at: u64,
    pub valid_until: Option<u64>,
    pub status: String,
    pub revoked: bool,
    pub presentation_count: u32,
}

/// Response from listing credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCredentialsResponse {
    pub credentials: Vec<CredentialMetadata>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Credential status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub cid: Cid,
    pub issuer: Did,
    pub holder: Did,
    pub credential_type: String,
    pub issued_at: u64,
    pub valid_until: Option<u64>,
    pub revoked: bool,
    pub revoked_at: Option<u64>,
    pub revocation_reason: Option<String>,
    pub presentations: Vec<PresentationInfo>,
    pub anchored_disclosures: Vec<Cid>,
    pub trust_attestations: Vec<TrustAttestationInfo>,
}

/// Information about a credential presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationInfo {
    pub presentation_id: String,
    pub context: String,
    pub presented_at: u64,
    pub verifier: Option<Did>,
    pub verification_result: Option<CredentialVerificationResult>,
}

/// Information about trust attestations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAttestationInfo {
    pub attestor: Did,
    pub trust_level: String,
    pub attested_at: u64,
    pub context: String,
}

/// Response envelope for API responses (simplified for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope<T> {
    pub data: T,
    pub success: bool,
    pub message: Option<String>,
}
pub trait CredentialTrait {
    /// Issue a new credential
    async fn issue_credential(
        &self,
        request: IssueCredentialRequest,
    ) -> Result<ResponseEnvelope<IssueCredentialResponse>, Box<dyn std::error::Error + Send + Sync>>;

    /// Present a credential for verification
    async fn present_credential(
        &self,
        request: PresentCredentialRequest,
    ) -> Result<ResponseEnvelope<PresentCredentialResponse>, Box<dyn std::error::Error + Send + Sync>>;

    /// Verify a credential presentation
    async fn verify_credential(
        &self,
        request: VerifyCredentialRequest,
    ) -> Result<
        ResponseEnvelope<CredentialVerificationResult>,
        Box<dyn std::error::Error + Send + Sync>,
    >;

    /// Anchor a credential disclosure to the DAG
    async fn anchor_disclosure(
        &self,
        request: AnchorDisclosureRequest,
    ) -> Result<ResponseEnvelope<AnchorDisclosureResponse>, Box<dyn std::error::Error + Send + Sync>>;

    /// Revoke a credential
    async fn revoke_credential(
        &self,
        request: RevokeCredentialRequest,
    ) -> Result<ResponseEnvelope<RevokeCredentialResponse>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get credential status
    async fn get_credential_status(
        &self,
        cid: &Cid,
    ) -> Result<ResponseEnvelope<CredentialStatus>, Box<dyn std::error::Error + Send + Sync>>;

    /// List credentials
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
    ) -> Result<ResponseEnvelope<ListCredentialsResponse>, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did};

    #[test]
    fn test_issue_credential_request_serialization() {
        let mut claims = HashMap::new();
        claims.insert(
            "skill_name".to_string(),
            serde_json::Value::String("Rust Programming".to_string()),
        );
        claims.insert(
            "level".to_string(),
            serde_json::Value::Number(serde_json::Number::from(8)),
        );

        let request = IssueCredentialRequest {
            credential_type: "skill".to_string(),
            holder: Did::new("key", "holder123"),
            issuer: Did::new("key", "issuer456"),
            claims,
            evidence: Some(vec!["https://github.com/user/project".to_string()]),
            validity_period: Some(365 * 24 * 3600), // 1 year
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("skill"));
        assert!(serialized.contains("Rust Programming"));

        let deserialized: IssueCredentialRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.credential_type, "skill");
    }

    #[test]
    fn test_credential_verification_result() {
        let mut verified_claims = HashMap::new();
        verified_claims.insert(
            "skill_name".to_string(),
            serde_json::Value::String("Rust Programming".to_string()),
        );

        let result = CredentialVerificationResult {
            valid: true,
            verification_level: "enhanced".to_string(),
            verified_claims,
            warnings: vec!["Credential nearing expiration".to_string()],
            errors: vec![],
            trust_score: Some(0.95),
        };

        assert!(result.valid);
        assert_eq!(result.verification_level, "enhanced");
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 0);
    }
}
