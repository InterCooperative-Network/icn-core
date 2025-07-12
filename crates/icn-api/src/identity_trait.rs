use async_trait::async_trait;
use icn_common::{Cid, CommonError, Did, VerifiableCredential};
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
    pub credential: VerifiableCredential,
}

/// Result of verifying a credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub valid: bool,
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
}
