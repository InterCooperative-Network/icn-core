//! Comprehensive tests for ICN Identity Protocol implementation
//!
//! This module tests the complete identity protocol including:
//! - DID document lifecycle
//! - Verifiable credential operations  
//! - Identity lifecycle management
//! - Sybil resistance mechanisms
//! - Privacy-preserving features

use crate::did_document::*;
use crate::verifiable_credential::*;
use crate::identity_lifecycle::*;
use icn_common::{Did, CommonError};
use std::collections::HashMap;
use std::str::FromStr;

// Mock mana ledger for testing
pub struct TestManaLedger {
    balances: HashMap<Did, u64>,
}

impl TestManaLedger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    pub fn set_balance(&mut self, did: Did, balance: u64) {
        self.balances.insert(did, balance);
    }
}

impl ManaLedger for TestManaLedger {
    fn get_balance(&self, did: &Did) -> Result<u64, CommonError> {
        Ok(self.balances.get(did).copied().unwrap_or(0))
    }

    fn spend_mana(&mut self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let current = self.get_balance(did)?;
        if current < amount {
            return Err(CommonError::InsufficientFunds("Not enough mana".to_string()));
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

fn test_person_did(id: &str) -> Did {
    Did::from_str(&format!("did:icn:person:{}", id)).unwrap()
}

fn test_org_did(id: &str) -> Did {
    Did::from_str(&format!("did:icn:organization:{}", id)).unwrap()
}

fn create_test_verification_method(did: &Did, key_id: &str) -> VerificationMethod {
    VerificationMethod {
        id: format!("{}#{}", did, key_id),
        method_type: VerificationMethodType::Ed25519VerificationKey2020,
        controller: did.clone(),
        public_key: PublicKeyMaterial::Ed25519 {
            public_key_bytes: vec![1, 2, 3, 4],
        },
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        expires: None,
        revoked: None,
    }
}

#[cfg(test)]
mod did_document_tests {
    use super::*;

    #[test]
    fn test_did_document_creation() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let doc = DidDocument::new(did.clone(), controller.clone());

        assert_eq!(doc.id, did);
        assert_eq!(doc.controller, vec![controller]);
        assert_eq!(doc.version, 1);
        assert!(doc.verification_method.is_empty());
    }

    #[test]
    fn test_add_verification_method() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let mut doc = DidDocument::new(did.clone(), controller);

        let method = create_test_verification_method(&did, "key1");
        doc.add_verification_method(method.clone());

        assert_eq!(doc.verification_method.len(), 1);
        assert_eq!(doc.verification_method[0].id, method.id);
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_rate_limiting() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let mut doc = DidDocument::new(did, controller);

        // Should be OK initially
        assert!(doc.check_rate_limit().is_ok());

        // Max out the rate limit
        doc.icn_metadata.sybil_resistance.rate_limit_status.operations_this_epoch = 100;

        // Should now fail
        assert!(doc.check_rate_limit().is_err());
    }

    #[test]
    fn test_increment_operation_count() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let mut doc = DidDocument::new(did, controller);

        let initial_count = doc.icn_metadata.sybil_resistance.rate_limit_status.operations_this_epoch;
        doc.increment_operation_count();
        
        assert_eq!(
            doc.icn_metadata.sybil_resistance.rate_limit_status.operations_this_epoch,
            initial_count + 1
        );
    }

    #[test]
    fn test_document_validation_requires_verification_method() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let doc = DidDocument::new(did, controller);

        // Should fail validation - no verification methods
        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_document_validation_with_verification_method() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let mut doc = DidDocument::new(did.clone(), controller);

        let method = create_test_verification_method(&did, "key1");
        doc.add_verification_method(method);

        // Should pass validation now
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_authentication_reference_validation() {
        let did = test_person_did("alice");
        let controller = test_person_did("alice");
        let mut doc = DidDocument::new(did.clone(), controller);

        let method = create_test_verification_method(&did, "key1");
        doc.add_verification_method(method.clone());
        doc.add_authentication(method.id.clone());

        // Should pass validation
        assert!(doc.validate().is_ok());

        // Add invalid authentication reference
        doc.add_authentication("invalid_ref".to_string());

        // Should fail validation
        assert!(doc.validate().is_err());
    }
}

#[cfg(test)]
mod verifiable_credential_tests {
    use super::*;

    #[test]
    fn test_membership_credential_creation() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer.clone(),
            subject.clone(),
            org_id.clone(),
            "member".to_string(),
        ).unwrap();

        assert_eq!(credential.issuer.id, issuer);
        assert_eq!(credential.credential_subject.id, subject);
        assert!(!credential.icn_metadata.transferable);
        assert!(credential.icn_metadata.grants_voting_rights);

        if let CredentialCategory::Membership { organization_id, role, .. } = 
            &credential.icn_metadata.category {
            assert_eq!(*organization_id, org_id);
            assert_eq!(*role, "member");
        } else {
            panic!("Wrong credential category");
        }
    }

    #[test]
    fn test_resource_credential_creation() {
        let issuer = test_person_did("system");
        let subject = test_person_did("node1");
        let resource_types = vec!["cpu".to_string(), "memory".to_string()];
        let mut capacity = HashMap::new();
        capacity.insert("cpu_cores".to_string(), 8);
        capacity.insert("memory_gb".to_string(), 32);

        let credential = VerifiableCredential::new_resource_credential(
            issuer.clone(),
            subject.clone(),
            resource_types.clone(),
            capacity.clone(),
        ).unwrap();

        assert_eq!(credential.issuer.id, issuer);
        assert_eq!(credential.credential_subject.id, subject);
        assert!(!credential.icn_metadata.transferable);
        assert!(!credential.icn_metadata.grants_voting_rights);

        if let CredentialCategory::ResourceProvider { resource_types: rt, capacity: cap } = 
            &credential.icn_metadata.category {
            assert_eq!(*rt, resource_types);
            assert_eq!(*cap, capacity);
        } else {
            panic!("Wrong credential category");
        }
    }

    #[test]
    fn test_credential_validation() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        ).unwrap();

        assert!(credential.validate().is_ok());
    }

    #[test]
    fn test_membership_credential_non_transferable() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let mut credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        ).unwrap();

        // Try to make it transferable (should fail validation)
        credential.icn_metadata.transferable = true;
        assert!(credential.validate().is_err());
    }

    #[test]
    fn test_credential_voting_rights() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        ).unwrap();

        assert!(credential.grants_voting_rights());
        assert!(!credential.is_transferable());
        assert_eq!(credential.subject(), &test_person_did("alice"));
        assert_eq!(credential.issuer(), &test_org_did("coop1"));
    }

    #[test]
    fn test_credential_expiration() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        ).unwrap();

        // Should not be expired without expiration date
        assert!(!credential.is_expired());
    }

    #[test]
    fn test_selective_disclosure() {
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");
        let org_id = test_org_did("coop1");

        let credential = VerifiableCredential::new_membership_credential(
            issuer,
            subject,
            org_id,
            "member".to_string(),
        ).unwrap();

        let disclosed_fields = vec!["role".to_string()];
        let disclosure = credential.create_selective_disclosure(disclosed_fields.clone()).unwrap();

        assert_eq!(disclosure.disclosed_fields, disclosed_fields);
        assert!(!disclosure.salts.is_empty());
    }
}

#[cfg(test)]
mod identity_lifecycle_tests {
    use super::*;
    use tokio;

    async fn setup_test_manager() -> IdentityLifecycleManager {
        let mana_ledger = TestManaLedger::new();
        IdentityLifecycleManager::new(
            Box::new(mana_ledger),
            IdentityConfig::default(),
        )
    }

    #[tokio::test]
    async fn test_did_creation_full_lifecycle() {
        let mut manager = setup_test_manager().await;
        let did = test_person_did("alice");

        // Set up mana balance
        let mana_ledger = manager.mana_ledger_mut();
        if let Some(test_ledger) = mana_ledger.as_any().downcast_mut::<TestManaLedger>() {
            test_ledger.set_balance(did.clone(), 100);
        }

        let verification_method = create_test_verification_method(&did, "key1");

        let request = DidCreationRequest {
            did: did.clone(),
            initial_verification_method: verification_method,
            identity_type: IdentityType::Person,
            proof_of_personhood: None,
            mana_payment_proof: "test_proof".to_string(),
        };

        // Request creation
        let operation_id = manager.request_did_creation(request).await.unwrap();
        
        // Execute creation
        let created_did = manager.execute_did_creation(&operation_id).await.unwrap();

        assert_eq!(created_did, did);
        assert!(manager.get_did_document(&did).is_some());

        let did_doc = manager.get_did_document(&did).unwrap();
        assert_eq!(did_doc.id, did);
        assert_eq!(did_doc.verification_method.len(), 1);
        assert_eq!(did_doc.icn_metadata.identity_type, IdentityType::Person);
    }

    #[tokio::test]
    async fn test_did_creation_insufficient_mana() {
        let mut manager = setup_test_manager().await;
        let did = test_person_did("alice");

        // Don't set up mana balance (defaults to 0)
        let verification_method = create_test_verification_method(&did, "key1");

        let request = DidCreationRequest {
            did: did.clone(),
            initial_verification_method: verification_method,
            identity_type: IdentityType::Person,
            proof_of_personhood: None,
            mana_payment_proof: "test_proof".to_string(),
        };

        // Should fail due to insufficient mana
        let result = manager.request_did_creation(request).await;
        assert!(result.is_err());
        if let Err(CommonError::InsufficientFunds(_)) = result {
            // Expected error
        } else {
            panic!("Expected InsufficientFunds error");
        }
    }

    #[tokio::test]
    async fn test_membership_credential_issuance() {
        let mut manager = setup_test_manager().await;
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");

        // Set up mana balance for issuer
        let mana_ledger = manager.mana_ledger_mut();
        if let Some(test_ledger) = mana_ledger.as_any().downcast_mut::<TestManaLedger>() {
            test_ledger.set_balance(issuer.clone(), 100);
        }

        // Create issuer DID (organization)
        let mut issuer_doc = DidDocument::new(issuer.clone(), issuer.clone());
        issuer_doc.icn_metadata.identity_type = IdentityType::Organization;
        manager.insert_did_document(issuer.clone(), issuer_doc);

        // Create subject DID
        let subject_doc = DidDocument::new(subject.clone(), subject.clone());
        manager.insert_did_document(subject.clone(), subject_doc);

        // Create credential template
        let template = CredentialTemplate {
            credential_type: "MembershipCredential".to_string(),
            required_claims: vec!["role".to_string()],
            optional_claims: Vec::new(),
            transferable: false,
            grants_voting_rights: true,
            expiration_period: None,
        };

        let mut additional_claims = HashMap::new();
        additional_claims.insert("role".to_string(), serde_json::Value::String("member".to_string()));

        let request = CredentialIssuanceRequest {
            issuer: issuer.clone(),
            subject: subject.clone(),
            credential_template: template,
            additional_claims,
            issuer_signature: "test_signature".to_string(),
        };

        // Request issuance
        let operation_id = manager.request_credential_issuance(request).await.unwrap();
        
        // Execute issuance
        let credential_id = manager.execute_credential_issuance(&operation_id).await.unwrap();

        let credential = manager.get_credential(&credential_id).unwrap();
        assert_eq!(credential.subject(), &subject);
        assert_eq!(credential.issuer(), &issuer);
        assert!(!credential.is_transferable());
        assert!(credential.grants_voting_rights());

        // Test credential listing
        let subject_credentials = manager.list_credentials_for_did(&subject);
        assert_eq!(subject_credentials.len(), 1);
        assert_eq!(subject_credentials[0].id, credential_id);
    }

    #[tokio::test]
    async fn test_credential_verification() {
        let mut manager = setup_test_manager().await;
        let issuer = test_org_did("coop1");
        let subject = test_person_did("alice");

        // Create issuer DID (organization)
        let mut issuer_doc = DidDocument::new(issuer.clone(), issuer.clone());
        issuer_doc.icn_metadata.identity_type = IdentityType::Organization;
        manager.insert_did_document(issuer.clone(), issuer_doc);

        // Create a credential
        let credential = VerifiableCredential::new_membership_credential(
            issuer.clone(),
            subject.clone(),
            issuer.clone(),
            "member".to_string(),
        ).unwrap();

        // Verify credential
        let verification_result = manager.verify_credential(&credential);
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[tokio::test]
    async fn test_key_rotation_lifecycle() {
        let mut manager = setup_test_manager().await;
        let did = test_person_did("alice");

        // Set up mana balance
        let mana_ledger = manager.mana_ledger_mut();
        if let Some(test_ledger) = mana_ledger.as_any().downcast_mut::<TestManaLedger>() {
            test_ledger.set_balance(did.clone(), 100);
        }

        // Create DID with initial key
        let mut did_doc = DidDocument::new(did.clone(), did.clone());
        let old_method = create_test_verification_method(&did, "key1");
        did_doc.add_verification_method(old_method.clone());
        manager.insert_did_document(did.clone(), did_doc);

        // Request key rotation
        let new_method = create_test_verification_method(&did, "key2");
        let request = KeyRotationRequest {
            did: did.clone(),
            old_verification_method: old_method.id.clone(),
            new_verification_method: new_method.clone(),
            rotation_reason: KeyRotationReason::Scheduled,
            old_key_signature: "test_signature".to_string(),
        };

        let operation_id = manager.request_key_rotation(request).await.unwrap();
        let result = manager.execute_key_rotation(&operation_id).await;
        assert!(result.is_ok());

        // Verify key was rotated
        let updated_doc = manager.get_did_document(&did).unwrap();
        assert!(!updated_doc.verification_method.iter().any(|vm| vm.id == old_method.id));
        assert!(updated_doc.verification_method.iter().any(|vm| vm.id == new_method.id));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_identity_workflow() {
        let mut manager = setup_test_manager().await;
        
        // Step 1: Create organization DID
        let org_did = test_org_did("techcoop");
        let mana_ledger = manager.mana_ledger_mut();
        if let Some(test_ledger) = mana_ledger.as_any().downcast_mut::<TestManaLedger>() {
            test_ledger.set_balance(org_did.clone(), 1000);
        }

        let org_verification_method = create_test_verification_method(&org_did, "orgkey1");
        let org_request = DidCreationRequest {
            did: org_did.clone(),
            initial_verification_method: org_verification_method,
            identity_type: IdentityType::Organization,
            proof_of_personhood: None,
            mana_payment_proof: "org_proof".to_string(),
        };

        let org_op_id = manager.request_did_creation(org_request).await.unwrap();
        manager.execute_did_creation(&org_op_id).await.unwrap();

        // Step 2: Create person DID
        let person_did = test_person_did("bob");
        let mana_ledger = manager.mana_ledger_mut();
        if let Some(test_ledger) = mana_ledger.as_any().downcast_mut::<TestManaLedger>() {
            test_ledger.set_balance(person_did.clone(), 100);
        }

        let person_verification_method = create_test_verification_method(&person_did, "bobkey1");
        let person_request = DidCreationRequest {
            did: person_did.clone(),
            initial_verification_method: person_verification_method,
            identity_type: IdentityType::Person,
            proof_of_personhood: Some(ProofOfPersonhood::SocialVouching {
                vouchers: vec![test_person_did("alice")],
                threshold: 1,
            }),
            mana_payment_proof: "person_proof".to_string(),
        };

        let person_op_id = manager.request_did_creation(person_request).await.unwrap();
        manager.execute_did_creation(&person_op_id).await.unwrap();

        // Step 3: Issue membership credential from org to person
        let template = CredentialTemplate {
            credential_type: "MembershipCredential".to_string(),
            required_claims: vec!["role".to_string()],
            optional_claims: Vec::new(),
            transferable: false,
            grants_voting_rights: true,
            expiration_period: None,
        };

        let mut additional_claims = HashMap::new();
        additional_claims.insert("role".to_string(), serde_json::Value::String("developer".to_string()));

        let cred_request = CredentialIssuanceRequest {
            issuer: org_did.clone(),
            subject: person_did.clone(),
            credential_template: template,
            additional_claims,
            issuer_signature: "cred_signature".to_string(),
        };

        let cred_op_id = manager.request_credential_issuance(cred_request).await.unwrap();
        let credential_id = manager.execute_credential_issuance(&cred_op_id).await.unwrap();

        // Step 4: Verify the entire workflow
        let org_doc = manager.get_did_document(&org_did).unwrap();
        assert_eq!(org_doc.icn_metadata.identity_type, IdentityType::Organization);

        let person_doc = manager.get_did_document(&person_did).unwrap();
        assert_eq!(person_doc.icn_metadata.identity_type, IdentityType::Person);
        assert!(person_doc.icn_metadata.sybil_resistance.proof_of_personhood.is_some());

        let credential = manager.get_credential(&credential_id).unwrap();
        assert_eq!(credential.subject(), &person_did);
        assert_eq!(credential.issuer(), &org_did);
        assert!(credential.grants_voting_rights());
        assert!(!credential.is_transferable());

        // Step 5: Verify credential
        let verification_result = manager.verify_credential(credential).unwrap();
        assert!(verification_result);

        // Step 6: List credentials for person
        let person_credentials = manager.list_credentials_for_did(&person_did);
        assert_eq!(person_credentials.len(), 1);
        assert!(person_credentials[0].grants_voting_rights());
    }

    async fn setup_test_manager() -> IdentityLifecycleManager {
        let mana_ledger = TestManaLedger::new();
        IdentityLifecycleManager::new(
            Box::new(mana_ledger),
            IdentityConfig::default(),
        )
    }
}