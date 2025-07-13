use async_trait::async_trait;
use icn_common::ZkCredentialProof;
use icn_common::{Cid, CommonError, Did};
use icn_identity::{Credential as VerifiableCredential, DisclosedCredential};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Request to issue a verifiable credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCredentialRequest {
    pub issuer: Did,
    pub holder: Did,
    pub attributes: BTreeMap<String, String>,
    pub schema: Cid,
    pub expiration: u64,
}

/// Response containing the issued credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialResponse {
    pub cid: Cid,
    pub credential: VerifiableCredential,
}

/// Result of verifying a credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeCredentialRequest {
    pub cid: Cid,
}

/// Request selective disclosure of certain fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureRequest {
    pub credential: VerifiableCredential,
    pub fields: Vec<String>,
}

/// Response containing disclosed fields and proof of the rest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureResponse {
    pub credential: DisclosedCredential,
    pub proof: ZkCredentialProof,
}

/// Request to verify multiple zero-knowledge proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofsRequest {
    pub proofs: Vec<ZkCredentialProof>,
}

/// Batch verification results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchVerificationResponse {
    pub results: Vec<bool>,
}

#[async_trait]
pub trait IdentityApi {
    async fn issue_credential(
        &self,
        request: IssueCredentialRequest,
    ) -> Result<CredentialResponse, CommonError>;

    async fn verify_credential(
        &self,
        credential: VerifiableCredential,
    ) -> Result<VerificationResponse, CommonError>;

    async fn get_credential(&self, cid: Cid) -> Result<CredentialResponse, CommonError>;

    async fn revoke_credential(&self, cid: Cid) -> Result<(), CommonError>;

    async fn list_schemas(&self) -> Result<Vec<Cid>, CommonError>;

    async fn request_disclosure(
        &self,
        request: DisclosureRequest,
    ) -> Result<DisclosureResponse, CommonError>;

    /// Verify multiple credential proofs in one call.
    async fn verify_proofs(
        &self,
        req: VerifyProofsRequest,
    ) -> Result<BatchVerificationResponse, CommonError>;
}
