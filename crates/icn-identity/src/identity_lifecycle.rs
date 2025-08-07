//! Identity Lifecycle Management for ICN Identity Protocol
//!
//! This module implements the complete lifecycle of identities in ICN including:
//! - DID creation with mana-based rate limiting
//! - Credential issuance and verification workflows
//! - Key rotation and recovery mechanisms
//! - Sybil resistance and proof-of-personhood

use crate::did_document::{DidDocument, VerificationMethod, VerificationMethodType, PublicKeyMaterial};
use crate::verifiable_credential::VerifiableCredential;
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identity lifecycle manager
pub struct IdentityLifecycleManager {
    /// Active DID documents
    did_registry: HashMap<Did, DidDocument>,

    /// Issued credentials  
    credential_registry: HashMap<String, VerifiableCredential>,

    /// Pending identity operations
    pending_operations: HashMap<String, PendingOperation>,

    /// Mana ledger for rate limiting
    mana_ledger: Box<dyn ManaLedger>,

    /// Configuration
    config: IdentityConfig,
}

/// Configuration for identity operations
#[derive(Debug, Clone)]
pub struct IdentityConfig {
    /// Mana cost for DID creation
    pub did_creation_cost: u64,

    /// Mana cost for credential issuance
    pub credential_issuance_cost: u64,

    /// Mana cost for key rotation
    pub key_rotation_cost: u64,

    /// Rate limiting: max operations per hour
    pub max_operations_per_hour: u32,

    /// Recovery delay in seconds
    pub recovery_delay_seconds: u64,

    /// Minimum guardians for social recovery
    pub min_recovery_guardians: u32,
}

/// Pending operations awaiting completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    /// Operation ID
    pub id: String,

    /// Type of operation
    pub operation_type: OperationType,

    /// DID affected
    pub target_did: Did,

    /// Who initiated the operation
    pub initiated_by: Did,

    /// When operation was initiated
    pub initiated_at: u64,

    /// When operation can be completed
    pub executable_at: u64,

    /// Operation-specific data
    pub operation_data: serde_json::Value,

    /// Required approvals
    pub required_approvals: Vec<Did>,

    /// Received approvals
    pub received_approvals: Vec<Approval>,
}

/// Types of identity operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    CreateDid,
    IssueCredential,
    RevokeCredential,
    RotateKey,
    RecoverDid,
    UpdateDidDocument,
}

/// Approval for pending operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// Who provided the approval
    pub approver: Did,

    /// When approval was given
    pub approved_at: u64,

    /// Cryptographic signature
    pub signature: String,
}

/// DID creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidCreationRequest {
    /// Requested DID
    pub did: Did,

    /// Initial verification method
    pub initial_verification_method: VerificationMethod,

    /// Identity type
    pub identity_type: crate::did_document::IdentityType,

    /// Proof of personhood if required
    pub proof_of_personhood: Option<crate::did_document::ProofOfPersonhood>,

    /// Mana payment proof
    pub mana_payment_proof: String,
}

/// Credential issuance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialIssuanceRequest {
    /// Issuer DID
    pub issuer: Did,

    /// Subject DID
    pub subject: Did,

    /// Credential template
    pub credential_template: CredentialTemplate,

    /// Additional claims
    pub additional_claims: HashMap<String, serde_json::Value>,

    /// Issuer signature
    pub issuer_signature: String,
}

/// Template for credential creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTemplate {
    /// Credential type
    pub credential_type: String,

    /// Required claims
    pub required_claims: Vec<String>,

    /// Optional claims
    pub optional_claims: Vec<String>,

    /// Transferability
    pub transferable: bool,

    /// Voting rights
    pub grants_voting_rights: bool,

    /// Expiration period in seconds
    pub expiration_period: Option<u64>,
}

/// Key rotation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationRequest {
    /// DID being rotated
    pub did: Did,

    /// Old verification method ID
    pub old_verification_method: String,

    /// New verification method
    pub new_verification_method: VerificationMethod,

    /// Reason for rotation
    pub rotation_reason: KeyRotationReason,

    /// Signature with old key
    pub old_key_signature: String,
}

/// Reasons for key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyRotationReason {
    Scheduled,
    Compromised,
    Lost,
    Upgrade,
}

/// Recovery request for lost DIDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequest {
    /// DID to recover
    pub did: Did,

    /// Recovery method
    pub recovery_method: RecoveryMethod,

    /// Guardian approvals
    pub guardian_approvals: Vec<GuardianApproval>,

    /// New verification method
    pub new_verification_method: VerificationMethod,
}

/// Methods for DID recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryMethod {
    SocialRecovery,
    BackupKey,
    EmergencyRecovery,
}

/// Guardian approval for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianApproval {
    /// Guardian DID
    pub guardian: Did,

    /// Approval timestamp
    pub approved_at: u64,

    /// Cryptographic signature
    pub signature: String,

    /// Optional challenge response
    pub challenge_response: Option<String>,
}

/// Trait for mana ledger operations
pub trait ManaLedger: Send + Sync {
    /// Get mana balance for a DID
    fn get_balance(&self, did: &Did) -> Result<u64, CommonError>;

    /// Spend mana for an operation
    fn spend_mana(&mut self, did: &Did, amount: u64) -> Result<(), CommonError>;

    /// Check if DID can afford operation
    fn can_afford(&self, did: &Did, amount: u64) -> Result<bool, CommonError>;

    /// Downcast helper for testing
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

impl IdentityLifecycleManager {
    /// Create new identity lifecycle manager
    pub fn new(mana_ledger: Box<dyn ManaLedger>, config: IdentityConfig) -> Self {
        Self {
            did_registry: HashMap::new(),
            credential_registry: HashMap::new(),
            pending_operations: HashMap::new(),
            mana_ledger,
            config,
        }
    }

    /// Test helper: get mutable access to mana ledger for testing
    #[cfg(test)]
    pub fn mana_ledger_mut(&mut self) -> &mut dyn ManaLedger {
        self.mana_ledger.as_mut()
    }

    /// Test helper: insert DID document for testing
    #[cfg(test)]
    pub fn insert_did_document(&mut self, did: Did, doc: DidDocument) {
        self.did_registry.insert(did, doc);
    }

    /// Request DID creation (returns operation ID)
    pub async fn request_did_creation(
        &mut self,
        request: DidCreationRequest,
    ) -> Result<String, CommonError> {
        // Validate request
        self.validate_did_creation_request(&request)?;

        // Check mana balance
        if !self
            .mana_ledger
            .can_afford(&request.did, self.config.did_creation_cost)?
        {
            return Err(CommonError::InsufficientFunds(
                "Insufficient mana for DID creation".to_string(),
            ));
        }

        // Check rate limiting
        self.check_rate_limit(&request.did)?;

        // Create pending operation
        let operation_id = format!("did_create_{}", uuid::Uuid::new_v4());
        let now = current_timestamp();

        let operation = PendingOperation {
            id: operation_id.clone(),
            operation_type: OperationType::CreateDid,
            target_did: request.did.clone(),
            initiated_by: request.did.clone(),
            initiated_at: now,
            executable_at: now, // DID creation is immediate if valid
            operation_data: serde_json::to_value(&request)
                .map_err(|e| CommonError::SerError(e.to_string()))?,
            required_approvals: Vec::new(), // No approvals needed for self-sovereign DIDs
            received_approvals: Vec::new(),
        };

        self.pending_operations
            .insert(operation_id.clone(), operation);

        Ok(operation_id)
    }

    /// Execute DID creation
    pub async fn execute_did_creation(&mut self, operation_id: &str) -> Result<Did, CommonError> {
        let operation = self
            .pending_operations
            .get(operation_id)
            .ok_or_else(|| CommonError::NotFound("Operation not found".to_string()))?
            .clone();

        // Parse request data
        let request: DidCreationRequest = serde_json::from_value(operation.operation_data)
            .map_err(|e| CommonError::DeserError(e.to_string()))?;

        // Spend mana
        self.mana_ledger
            .spend_mana(&request.did, self.config.did_creation_cost)?;

        // Create DID document
        let mut did_doc = DidDocument::new(request.did.clone(), request.did.clone());
        did_doc.icn_metadata.identity_type = request.identity_type;
        did_doc.icn_metadata.sybil_resistance.proof_of_personhood = request.proof_of_personhood;
        did_doc.add_verification_method(request.initial_verification_method);

        // Store in registry
        self.did_registry.insert(request.did.clone(), did_doc);

        // Remove pending operation
        self.pending_operations.remove(operation_id);

        Ok(request.did)
    }

    /// Request credential issuance
    pub async fn request_credential_issuance(
        &mut self,
        request: CredentialIssuanceRequest,
    ) -> Result<String, CommonError> {
        // Validate issuer exists
        let _issuer_doc = self
            .did_registry
            .get(&request.issuer)
            .ok_or_else(|| CommonError::NotFound("Issuer DID not found".to_string()))?;

        // Validate subject exists
        let _subject_doc = self
            .did_registry
            .get(&request.subject)
            .ok_or_else(|| CommonError::NotFound("Subject DID not found".to_string()))?;

        // Check issuer has authority to issue this credential type
        self.validate_issuer_authority(&request.issuer, &request.credential_template)?;

        // Check mana balance
        if !self
            .mana_ledger
            .can_afford(&request.issuer, self.config.credential_issuance_cost)?
        {
            return Err(CommonError::InsufficientFunds(
                "Insufficient mana for credential issuance".to_string(),
            ));
        }

        // Create pending operation
        let operation_id = format!("cred_issue_{}", uuid::Uuid::new_v4());
        let now = current_timestamp();

        let operation = PendingOperation {
            id: operation_id.clone(),
            operation_type: OperationType::IssueCredential,
            target_did: request.subject.clone(),
            initiated_by: request.issuer.clone(),
            initiated_at: now,
            executable_at: now, // Credential issuance is immediate if valid
            operation_data: serde_json::to_value(&request)
                .map_err(|e| CommonError::SerError(e.to_string()))?,
            required_approvals: Vec::new(),
            received_approvals: Vec::new(),
        };

        self.pending_operations
            .insert(operation_id.clone(), operation);

        Ok(operation_id)
    }

    /// Execute credential issuance
    pub async fn execute_credential_issuance(
        &mut self,
        operation_id: &str,
    ) -> Result<String, CommonError> {
        let operation = self
            .pending_operations
            .get(operation_id)
            .ok_or_else(|| CommonError::NotFound("Operation not found".to_string()))?
            .clone();

        // Parse request data
        let request: CredentialIssuanceRequest =
            serde_json::from_value(operation.operation_data)
                .map_err(|e| CommonError::DeserError(e.to_string()))?;

        // Spend mana
        self.mana_ledger
            .spend_mana(&request.issuer, self.config.credential_issuance_cost)?;

        // Create credential based on template
        let credential = self.create_credential_from_template(
            &request.issuer,
            &request.subject,
            &request.credential_template,
            &request.additional_claims,
        )?;

        // Store credential
        self.credential_registry
            .insert(credential.id.clone(), credential.clone());

        // Remove pending operation
        self.pending_operations.remove(operation_id);

        Ok(credential.id)
    }

    /// Request key rotation
    pub async fn request_key_rotation(
        &mut self,
        request: KeyRotationRequest,
    ) -> Result<String, CommonError> {
        // Validate DID exists
        let did_doc = self
            .did_registry
            .get(&request.did)
            .ok_or_else(|| CommonError::NotFound("DID not found".to_string()))?;

        // Verify old key signature
        self.verify_key_rotation_signature(&request, did_doc)?;

        // Check mana balance
        if !self
            .mana_ledger
            .can_afford(&request.did, self.config.key_rotation_cost)?
        {
            return Err(CommonError::InsufficientFunds(
                "Insufficient mana for key rotation".to_string(),
            ));
        }

        // Create pending operation
        let operation_id = format!("key_rotate_{}", uuid::Uuid::new_v4());
        let now = current_timestamp();

        // Add delay for security-sensitive rotations
        let delay = match request.rotation_reason {
            KeyRotationReason::Compromised => 0, // Immediate for compromised keys
            KeyRotationReason::Lost => self.config.recovery_delay_seconds,
            _ => 3600, // 1 hour for scheduled rotations
        };

        let operation = PendingOperation {
            id: operation_id.clone(),
            operation_type: OperationType::RotateKey,
            target_did: request.did.clone(),
            initiated_by: request.did.clone(),
            initiated_at: now,
            executable_at: now + delay,
            operation_data: serde_json::to_value(&request)
                .map_err(|e| CommonError::SerError(e.to_string()))?,
            required_approvals: Vec::new(),
            received_approvals: Vec::new(),
        };

        self.pending_operations
            .insert(operation_id.clone(), operation);

        Ok(operation_id)
    }

    /// Execute key rotation
    pub async fn execute_key_rotation(&mut self, operation_id: &str) -> Result<(), CommonError> {
        let operation = self
            .pending_operations
            .get(operation_id)
            .ok_or_else(|| CommonError::NotFound("Operation not found".to_string()))?
            .clone();

        // Check if executable
        let now = current_timestamp();
        if now < operation.executable_at {
            return Err(CommonError::ValidationError(
                "Operation not yet executable".to_string(),
            ));
        }

        // Parse request data
        let request: KeyRotationRequest = serde_json::from_value(operation.operation_data)
            .map_err(|e| CommonError::DeserError(e.to_string()))?;

        // Spend mana
        self.mana_ledger
            .spend_mana(&request.did, self.config.key_rotation_cost)?;

        // Update DID document
        let did_doc = self
            .did_registry
            .get_mut(&request.did)
            .ok_or_else(|| CommonError::NotFound("DID not found".to_string()))?;

        // Remove old verification method and add new one
        did_doc
            .verification_method
            .retain(|vm| vm.id != request.old_verification_method);
        did_doc.add_verification_method(request.new_verification_method);

        // Remove pending operation
        self.pending_operations.remove(operation_id);

        Ok(())
    }

    /// Get DID document
    pub fn get_did_document(&self, did: &Did) -> Option<&DidDocument> {
        self.did_registry.get(did)
    }

    /// Get credential
    pub fn get_credential(&self, credential_id: &str) -> Option<&VerifiableCredential> {
        self.credential_registry.get(credential_id)
    }

    /// List credentials for a DID
    pub fn list_credentials_for_did(&self, did: &Did) -> Vec<&VerifiableCredential> {
        self.credential_registry
            .values()
            .filter(|cred| cred.subject() == did)
            .collect()
    }

    /// Verify a credential
    pub fn verify_credential(
        &self,
        credential: &VerifiableCredential,
    ) -> Result<bool, CommonError> {
        // Basic validation
        credential.validate()?;

        // Check issuer exists
        let _issuer_doc = self
            .did_registry
            .get(credential.issuer())
            .ok_or_else(|| CommonError::NotFound("Issuer DID not found".to_string()))?;

        // Verify issuer has authority to issue this credential
        self.validate_issuer_authority(
            credential.issuer(),
            &self.credential_template_from_credential(credential)?,
        )?;

        // Check not expired
        if credential.is_expired() {
            return Ok(false);
        }

        // TODO: Verify cryptographic signature

        Ok(true)
    }

    // Private helper methods

    fn validate_did_creation_request(
        &self,
        request: &DidCreationRequest,
    ) -> Result<(), CommonError> {
        // Check DID doesn't already exist
        if self.did_registry.contains_key(&request.did) {
            return Err(CommonError::ValidationError(
                "DID already exists".to_string(),
            ));
        }

        // Validate verification method
        if request.initial_verification_method.id.is_empty() {
            return Err(CommonError::ValidationError(
                "Verification method ID cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_issuer_authority(
        &self,
        issuer: &Did,
        template: &CredentialTemplate,
    ) -> Result<(), CommonError> {
        // For membership credentials, only organizations can issue
        if template.credential_type == "MembershipCredential" {
            let issuer_doc = self
                .did_registry
                .get(issuer)
                .ok_or_else(|| CommonError::NotFound("Issuer not found".to_string()))?;

            if issuer_doc.icn_metadata.identity_type
                != crate::did_document::IdentityType::Organization
            {
                return Err(CommonError::ValidationError(
                    "Only organizations can issue membership credentials".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn check_rate_limit(&self, did: &Did) -> Result<(), CommonError> {
        if let Some(did_doc) = self.did_registry.get(did) {
            did_doc.check_rate_limit()?;
        }
        Ok(())
    }

    fn verify_key_rotation_signature(
        &self,
        request: &KeyRotationRequest,
        did_doc: &DidDocument,
    ) -> Result<(), CommonError> {
        // Find the old verification method
        let _old_method = did_doc
            .verification_method
            .iter()
            .find(|vm| vm.id == request.old_verification_method)
            .ok_or_else(|| {
                CommonError::NotFound("Old verification method not found".to_string())
            })?;

        // TODO: Verify signature with old key
        // This would involve verifying request.old_key_signature against request data

        Ok(())
    }

    fn create_credential_from_template(
        &self,
        issuer: &Did,
        subject: &Did,
        template: &CredentialTemplate,
        additional_claims: &HashMap<String, serde_json::Value>,
    ) -> Result<VerifiableCredential, CommonError> {
        match template.credential_type.as_str() {
            "MembershipCredential" => {
                let org_id = issuer.clone(); // Issuer is the organization
                let role = additional_claims
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("member")
                    .to_string();

                VerifiableCredential::new_membership_credential(
                    issuer.clone(),
                    subject.clone(),
                    org_id,
                    role,
                )
            }
            "ResourceProviderCredential" => {
                let resource_types = additional_claims
                    .get("resourceTypes")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        CommonError::ValidationError("Missing resourceTypes".to_string())
                    })?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let capacity = additional_claims
                    .get("capacity")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                VerifiableCredential::new_resource_credential(
                    issuer.clone(),
                    subject.clone(),
                    resource_types,
                    capacity,
                )
            }
            _ => Err(CommonError::NotImplemented(format!(
                "Credential type {} not yet implemented",
                template.credential_type
            ))),
        }
    }

    fn credential_template_from_credential(
        &self,
        credential: &VerifiableCredential,
    ) -> Result<CredentialTemplate, CommonError> {
        let credential_type = credential
            .credential_type
            .iter()
            .find(|t| *t != "VerifiableCredential")
            .ok_or_else(|| {
                CommonError::ValidationError("No specific credential type found".to_string())
            })?
            .clone();

        Ok(CredentialTemplate {
            credential_type,
            required_claims: Vec::new(),
            optional_claims: Vec::new(),
            transferable: credential.icn_metadata.transferable,
            grants_voting_rights: credential.icn_metadata.grants_voting_rights,
            expiration_period: None,
        })
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            did_creation_cost: 10,
            credential_issuance_cost: 1,
            key_rotation_cost: 5,
            max_operations_per_hour: 100,
            recovery_delay_seconds: 24 * 3600, // 24 hours
            min_recovery_guardians: 3,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // Mock mana ledger for testing
    struct MockManaLedger {
        balances: HashMap<Did, u64>,
    }

    impl MockManaLedger {
        fn new() -> Self {
            Self {
                balances: HashMap::new(),
            }
        }

        fn set_balance(&mut self, did: Did, balance: u64) {
            self.balances.insert(did, balance);
        }
    }

    impl ManaLedger for MockManaLedger {
        fn get_balance(&self, did: &Did) -> Result<u64, CommonError> {
            Ok(self.balances.get(did).copied().unwrap_or(0))
        }

        fn spend_mana(&mut self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let current = self.get_balance(did)?;
            if current < amount {
                return Err(CommonError::InsufficientFunds(
                    "Not enough mana".to_string(),
                ));
            }
            self.balances.insert(did.clone(), current - amount);
            Ok(())
        }

        fn can_afford(&self, did: &Did, amount: u64) -> Result<bool, CommonError> {
            Ok(self.get_balance(did)? >= amount)
        }

        fn as_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn test_did(id: &str) -> Did {
        Did::from_str(&format!("did:icn:person:{}", id)).unwrap()
    }

    #[tokio::test]
    async fn test_did_creation_lifecycle() {
        let mut mana_ledger = MockManaLedger::new();
        let did = test_did("alice");
        mana_ledger.set_balance(did.clone(), 100);

        let mut manager =
            IdentityLifecycleManager::new(Box::new(mana_ledger), IdentityConfig::default());

        // Create verification method
        let verification_method = VerificationMethod {
            id: format!("{}#key1", did),
            method_type: VerificationMethodType::Ed25519VerificationKey2020,
            controller: did.clone(),
            public_key: PublicKeyMaterial::Ed25519 {
                public_key_bytes: vec![1, 2, 3, 4],
            },
            created: current_timestamp(),
            expires: None,
            revoked: None,
        };

        // Request DID creation
        let request = DidCreationRequest {
            did: did.clone(),
            initial_verification_method: verification_method,
            identity_type: crate::did_document::IdentityType::Person,
            proof_of_personhood: None,
            mana_payment_proof: "test_proof".to_string(),
        };

        let operation_id = manager.request_did_creation(request).await.unwrap();
        let created_did = manager.execute_did_creation(&operation_id).await.unwrap();

        assert_eq!(created_did, did);
        assert!(manager.get_did_document(&did).is_some());
    }

    #[tokio::test]
    async fn test_credential_issuance_lifecycle() {
        let mut mana_ledger = MockManaLedger::new();
        let issuer = test_did("org1");
        let subject = test_did("alice");
        mana_ledger.set_balance(issuer.clone(), 100);

        let mut manager =
            IdentityLifecycleManager::new(Box::new(mana_ledger), IdentityConfig::default());

        // Create issuer DID (organization)
        let mut issuer_doc = DidDocument::new(issuer.clone(), issuer.clone());
        issuer_doc.icn_metadata.identity_type = crate::did_document::IdentityType::Organization;
        manager.did_registry.insert(issuer.clone(), issuer_doc);

        // Create subject DID
        let subject_doc = DidDocument::new(subject.clone(), subject.clone());
        manager.did_registry.insert(subject.clone(), subject_doc);

        // Request credential issuance
        let template = CredentialTemplate {
            credential_type: "MembershipCredential".to_string(),
            required_claims: vec!["role".to_string()],
            optional_claims: Vec::new(),
            transferable: false,
            grants_voting_rights: true,
            expiration_period: None,
        };

        let mut additional_claims = HashMap::new();
        additional_claims.insert(
            "role".to_string(),
            serde_json::Value::String("member".to_string()),
        );

        let request = CredentialIssuanceRequest {
            issuer: issuer.clone(),
            subject: subject.clone(),
            credential_template: template,
            additional_claims,
            issuer_signature: "test_signature".to_string(),
        };

        let operation_id = manager.request_credential_issuance(request).await.unwrap();
        let credential_id = manager
            .execute_credential_issuance(&operation_id)
            .await
            .unwrap();

        let credential = manager.get_credential(&credential_id).unwrap();
        assert_eq!(credential.subject(), &subject);
        assert_eq!(credential.issuer(), &issuer);
        assert!(!credential.is_transferable());
        assert!(credential.grants_voting_rights());
    }
}
