use async_trait::async_trait;
use icn_common::{Cid, CommonError, Did, ZkCredentialProof};
use icn_identity::Credential as VerifiableCredential;
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

/// Request containing multiple zero-knowledge credential proofs to verify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofsRequest {
    pub proofs: Vec<ZkCredentialProof>,
}

/// Response for batch proof verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchVerificationResponse {
    /// Verification result for each proof in the request.
    pub verified: Vec<bool>,
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

    /// Verify multiple zero-knowledge credential proofs in a single request.
    async fn verify_proofs(
        &self,
        req: VerifyProofsRequest,
    ) -> Result<BatchVerificationResponse, CommonError>;
}
