//! Trust Verification & Attestation System
//!
//! This module implements a multi-party trust attestation system where multiple
//! cooperatives can vouch for trust relationships. The system includes:
//! - Multi-party trust attestations with cryptographic signatures
//! - Reputation-based trust weighting 
//! - Trust challenge and dispute resolution mechanisms
//! - Immutable audit trails anchored in the DAG

use crate::{
    cooperative_schemas::{TrustLevel, TrustRelationship},
    federation_trust::TrustContext,
    sign_message, verify_signature, DidResolver, EdSignature, SigningKey, VerifyingKey,
};
use icn_common::{Cid, CommonError, Did, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A single attestation in a multi-party trust relationship
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustAttestation {
    /// DID of the cooperative making this attestation
    pub attester: Did,
    /// DID of the entity being attested to (target of trust)
    pub subject: Did,
    /// Trust context for this attestation
    pub context: TrustContext,
    /// Trust level being attested
    pub trust_level: TrustLevel,
    /// Timestamp when attestation was created
    pub timestamp: u64,
    /// Optional evidence or justification for the attestation
    pub evidence: Option<String>,
    /// Cryptographic signature by the attester
    pub signature: Vec<u8>,
}

impl TrustAttestation {
    /// Create a new trust attestation (unsigned)
    pub fn new(
        attester: Did,
        subject: Did,
        context: TrustContext,
        trust_level: TrustLevel,
        timestamp: u64,
        evidence: Option<String>,
    ) -> Self {
        Self {
            attester,
            subject,
            context,
            trust_level,
            timestamp,
            evidence,
            signature: Vec::new(),
        }
    }

    /// Create the canonical message bytes for signing
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.attester.to_string().as_bytes());
        bytes.extend_from_slice(self.subject.to_string().as_bytes());
        bytes.extend_from_slice(self.context.as_str().as_bytes());
        bytes.extend_from_slice(self.trust_level.as_str().as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        if let Some(evidence) = &self.evidence {
            bytes.extend_from_slice(evidence.as_bytes());
        }
        Ok(bytes)
    }

    /// Sign this attestation with the provided signing key
    pub fn sign_with_key(mut self, signing_key: &SigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let signature = sign_message(signing_key, &message);
        self.signature = signature.to_bytes().to_vec();
        Ok(self)
    }

    /// Verify the signature of this attestation
    pub fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let signature = EdSignature::from_bytes(
            self.signature.as_slice().try_into().map_err(|_| {
                CommonError::IdentityError("Invalid signature length".into())
            })?,
        );
        
        if verify_signature(verifying_key, &message, &signature) {
            Ok(())
        } else {
            Err(CommonError::IdentityError(
                "Trust attestation signature verification failed".into(),
            ))
        }
    }

    /// Verify the attestation using a DID resolver
    pub fn verify_with_resolver(&self, resolver: &dyn DidResolver) -> Result<(), CommonError> {
        let verifying_key = resolver.resolve(&self.attester)?;
        self.verify_with_key(&verifying_key)
    }
}

/// Multi-party trust attestation record containing multiple attestations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPartyTrustRecord {
    /// DID of the entity being attested to
    pub subject: Did,
    /// Trust context for all attestations in this record
    pub context: TrustContext,
    /// All attestations for this subject in this context
    pub attestations: Vec<TrustAttestation>,
    /// Aggregated trust score based on reputation weighting
    pub aggregated_score: f64,
    /// Timestamp when this record was last updated
    pub last_updated: u64,
    /// CID of this record when anchored in DAG
    pub dag_cid: Option<Cid>,
}

impl MultiPartyTrustRecord {
    /// Create a new multi-party trust record
    pub fn new(subject: Did, context: TrustContext) -> Self {
        Self {
            subject,
            context,
            attestations: Vec::new(),
            aggregated_score: 0.0,
            last_updated: 0,
            dag_cid: None,
        }
    }

    /// Add an attestation to this record
    pub fn add_attestation(&mut self, attestation: TrustAttestation, timestamp: u64) -> Result<(), CommonError> {
        // Verify the attestation is for the correct subject and context
        if attestation.subject != self.subject {
            return Err(CommonError::InvalidInputError(
                "Attestation subject does not match record subject".into(),
            ));
        }
        if attestation.context != self.context {
            return Err(CommonError::InvalidInputError(
                "Attestation context does not match record context".into(),
            ));
        }

        // Check if attester already has an attestation and replace it
        if let Some(existing_pos) = self.attestations.iter().position(|a| a.attester == attestation.attester) {
            self.attestations[existing_pos] = attestation;
        } else {
            self.attestations.push(attestation);
        }
        
        self.last_updated = timestamp;
        Ok(())
    }

    /// Remove an attestation by attester DID
    pub fn remove_attestation(&mut self, attester: &Did, timestamp: u64) -> bool {
        if let Some(pos) = self.attestations.iter().position(|a| &a.attester == attester) {
            self.attestations.remove(pos);
            self.last_updated = timestamp;
            true
        } else {
            false
        }
    }

    /// Calculate aggregated trust score using simple averaging (reputation integration removed for now)
    pub fn calculate_aggregated_score_simple(&mut self) -> f64 {
        if self.attestations.is_empty() {
            self.aggregated_score = 0.0;
            return self.aggregated_score;
        }

        let mut total_score = 0.0;
        for attestation in &self.attestations {
            let trust_value = match attestation.trust_level {
                TrustLevel::Full => 1.0,
                TrustLevel::Partial => 0.6,
                TrustLevel::Basic => 0.3,
                TrustLevel::None => 0.0,
            };
            total_score += trust_value;
        }

        self.aggregated_score = (total_score / self.attestations.len() as f64).clamp(0.0, 1.0);
        self.aggregated_score
    }

    /// Get attesters who have vouched for this subject
    pub fn get_attesters(&self) -> HashSet<Did> {
        self.attestations.iter().map(|a| a.attester.clone()).collect()
    }

    /// Verify all attestations in this record
    pub fn verify_all_attestations(&self, resolver: &dyn DidResolver) -> Result<Vec<Did>, CommonError> {
        let mut verified_attesters = Vec::new();
        
        for attestation in &self.attestations {
            match attestation.verify_with_resolver(resolver) {
                Ok(_) => verified_attesters.push(attestation.attester.clone()),
                Err(e) => {
                    eprintln!("Failed to verify attestation from {}: {}", attestation.attester, e);
                }
            }
        }
        
        Ok(verified_attesters)
    }
}

/// Trust challenge record for disputing trust relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChallenge {
    /// Unique identifier for this challenge
    pub challenge_id: String,
    /// DID of the entity challenging the trust
    pub challenger: Did,
    /// DID of the entity whose trust is being challenged
    pub challenged_subject: Did,
    /// Trust context being challenged
    pub context: TrustContext,
    /// Specific attestation being challenged (optional)
    pub challenged_attestation: Option<TrustAttestation>,
    /// Reason for the challenge
    pub reason: String,
    /// Evidence supporting the challenge
    pub evidence: Option<String>,
    /// Timestamp when challenge was created
    pub timestamp: u64,
    /// Current status of the challenge
    pub status: ChallengeStatus,
    /// CID when anchored in DAG
    pub dag_cid: Option<Cid>,
}

/// Status of a trust challenge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeStatus {
    /// Challenge is pending review
    Pending,
    /// Challenge is under investigation
    UnderReview,
    /// Challenge was accepted and trust was revoked/reduced
    Accepted,
    /// Challenge was rejected and trust remains
    Rejected,
    /// Challenge was withdrawn by challenger
    Withdrawn,
}

impl TrustChallenge {
    /// Create a new trust challenge
    pub fn new(
        challenge_id: String,
        challenger: Did,
        challenged_subject: Did,
        context: TrustContext,
        reason: String,
        timestamp: u64,
    ) -> Self {
        Self {
            challenge_id,
            challenger,
            challenged_subject,
            context,
            challenged_attestation: None,
            reason,
            evidence: None,
            timestamp,
            status: ChallengeStatus::Pending,
            dag_cid: None,
        }
    }

    /// Set evidence for this challenge
    pub fn set_evidence(&mut self, evidence: String) {
        self.evidence = Some(evidence);
    }

    /// Update challenge status
    pub fn update_status(&mut self, status: ChallengeStatus, timestamp: u64) {
        self.status = status;
        // Note: In a full implementation, we might want to track status change history
    }
}

/// Trust audit event for maintaining immutable history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAuditEvent {
    /// Unique identifier for this audit event
    pub event_id: String,
    /// Type of trust event
    pub event_type: TrustEventType,
    /// DID of the entity performing the action
    pub actor: Did,
    /// DID of the entity affected by the action
    pub subject: Did,
    /// Trust context for the action
    pub context: TrustContext,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Additional event data
    pub data: serde_json::Value,
    /// CID when anchored in DAG
    pub dag_cid: Option<Cid>,
}

/// Types of trust audit events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustEventType {
    /// New attestation created
    AttestationCreated,
    /// Existing attestation updated
    AttestationUpdated,
    /// Attestation revoked
    AttestationRevoked,
    /// Trust challenge created
    ChallengeCreated,
    /// Trust challenge resolved
    ChallengeResolved,
    /// Trust score recalculated
    ScoreRecalculated,
}

/// Store for managing trust attestations and challenges
pub trait TrustAttestationStore: Send + Sync {
    /// Store a multi-party trust record
    fn store_trust_record(&mut self, record: MultiPartyTrustRecord) -> Result<(), CommonError>;
    
    /// Retrieve a trust record by subject and context
    fn get_trust_record(&self, subject: &Did, context: &TrustContext) -> Option<MultiPartyTrustRecord>;
    
    /// Store a trust challenge
    fn store_challenge(&mut self, challenge: TrustChallenge) -> Result<(), CommonError>;
    
    /// Retrieve a challenge by ID
    fn get_challenge(&self, challenge_id: &str) -> Option<TrustChallenge>;
    
    /// List challenges by status
    fn list_challenges_by_status(&self, status: ChallengeStatus) -> Vec<TrustChallenge>;
    
    /// Store an audit event
    fn store_audit_event(&mut self, event: TrustAuditEvent) -> Result<(), CommonError>;
    
    /// Retrieve audit events for a subject
    fn get_audit_events(&self, subject: &Did, context: &TrustContext) -> Vec<TrustAuditEvent>;
}

/// In-memory implementation of TrustAttestationStore for testing
#[derive(Default)]
pub struct InMemoryTrustAttestationStore {
    trust_records: HashMap<(Did, TrustContext), MultiPartyTrustRecord>,
    challenges: HashMap<String, TrustChallenge>,
    audit_events: HashMap<(Did, TrustContext), Vec<TrustAuditEvent>>,
}

impl InMemoryTrustAttestationStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }
}

impl TrustAttestationStore for InMemoryTrustAttestationStore {
    fn store_trust_record(&mut self, record: MultiPartyTrustRecord) -> Result<(), CommonError> {
        let key = (record.subject.clone(), record.context.clone());
        self.trust_records.insert(key, record);
        Ok(())
    }
    
    fn get_trust_record(&self, subject: &Did, context: &TrustContext) -> Option<MultiPartyTrustRecord> {
        let key = (subject.clone(), context.clone());
        self.trust_records.get(&key).cloned()
    }
    
    fn store_challenge(&mut self, challenge: TrustChallenge) -> Result<(), CommonError> {
        self.challenges.insert(challenge.challenge_id.clone(), challenge);
        Ok(())
    }
    
    fn get_challenge(&self, challenge_id: &str) -> Option<TrustChallenge> {
        self.challenges.get(challenge_id).cloned()
    }
    
    fn list_challenges_by_status(&self, status: ChallengeStatus) -> Vec<TrustChallenge> {
        self.challenges
            .values()
            .filter(|c| c.status == status)
            .cloned()
            .collect()
    }
    
    fn store_audit_event(&mut self, event: TrustAuditEvent) -> Result<(), CommonError> {
        let key = (event.subject.clone(), event.context.clone());
        self.audit_events.entry(key).or_default().push(event);
        Ok(())
    }
    
    fn get_audit_events(&self, subject: &Did, context: &TrustContext) -> Vec<TrustAuditEvent> {
        let key = (subject.clone(), context.clone());
        self.audit_events.get(&key).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{generate_ed25519_keypair, did_key_from_verifying_key, KeyDidResolver};
    use icn_reputation::InMemoryReputationStore;
    use std::str::FromStr;

    #[test]
    fn test_trust_attestation_signing_and_verification() {
        let (sk, pk) = generate_ed25519_keypair();
        let attester = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();
        let subject = Did::new("key", "subject123");
        
        let attestation = TrustAttestation::new(
            attester.clone(),
            subject,
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            Some("Test evidence".to_string()),
        );
        
        let signed_attestation = attestation.sign_with_key(&sk).unwrap();
        assert!(!signed_attestation.signature.is_empty());
        
        // Verification should succeed
        assert!(signed_attestation.verify_with_key(&pk).is_ok());
        
        // Verification with resolver should also succeed
        let resolver = KeyDidResolver;
        assert!(signed_attestation.verify_with_resolver(&resolver).is_ok());
    }

    #[test]
    fn test_multi_party_trust_record() {
        let subject = Did::new("key", "subject123");
        let mut record = MultiPartyTrustRecord::new(subject.clone(), TrustContext::General);
        
        // Create attestations from different attesters
        let (sk1, pk1) = generate_ed25519_keypair();
        let attester1 = Did::from_str(&did_key_from_verifying_key(&pk1)).unwrap();
        let attestation1 = TrustAttestation::new(
            attester1.clone(),
            subject.clone(),
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            None,
        ).sign_with_key(&sk1).unwrap();
        
        let (sk2, pk2) = generate_ed25519_keypair();
        let attester2 = Did::from_str(&did_key_from_verifying_key(&pk2)).unwrap();
        let attestation2 = TrustAttestation::new(
            attester2.clone(),
            subject.clone(),
            TrustContext::General,
            TrustLevel::Partial,
            1234567891,
            None,
        ).sign_with_key(&sk2).unwrap();
        
        // Add attestations
        assert!(record.add_attestation(attestation1, 1234567890).is_ok());
        assert!(record.add_attestation(attestation2, 1234567891).is_ok());
        
        assert_eq!(record.attestations.len(), 2);
        assert_eq!(record.get_attesters().len(), 2);
        assert!(record.get_attesters().contains(&attester1));
        assert!(record.get_attesters().contains(&attester2));
    }

    #[test]
    fn test_reputation_based_trust_weighting() {
        let subject = Did::new("key", "subject123");
        let mut record = MultiPartyTrustRecord::new(subject.clone(), TrustContext::General);
        
        // Create reputation store with different scores
        let reputation_store = InMemoryReputationStore::new();
        
        // Create attesters with different reputation
        let (sk1, pk1) = generate_ed25519_keypair();
        let attester1 = Did::from_str(&did_key_from_verifying_key(&pk1)).unwrap();
        reputation_store.set_score(attester1.clone(), 100); // High reputation
        
        let (sk2, pk2) = generate_ed25519_keypair();
        let attester2 = Did::from_str(&did_key_from_verifying_key(&pk2)).unwrap();
        reputation_store.set_score(attester2.clone(), 10); // Low reputation
        
        // Both give high trust, but attester1 should have more weight
        let attestation1 = TrustAttestation::new(
            attester1,
            subject.clone(),
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            None,
        ).sign_with_key(&sk1).unwrap();
        
        let attestation2 = TrustAttestation::new(
            attester2,
            subject.clone(),
            TrustContext::General,
            TrustLevel::Full,
            1234567891,
            None,
        ).sign_with_key(&sk2).unwrap();
        
        record.add_attestation(attestation1, 1234567890).unwrap();
        record.add_attestation(attestation2, 1234567891).unwrap();
        
        let score = record.calculate_aggregated_score(&reputation_store);
        assert!(score > 0.0 && score <= 1.0);
        // The score should be closer to 1.0 due to high reputation attester
        assert!(score > 0.8);
    }

    #[test]
    fn test_trust_challenge_creation() {
        let challenger = Did::new("key", "challenger123");
        let subject = Did::new("key", "subject123");
        
        let challenge = TrustChallenge::new(
            "challenge-001".to_string(),
            challenger.clone(),
            subject.clone(),
            TrustContext::General,
            "Suspicious behavior observed".to_string(),
            1234567890,
        );
        
        assert_eq!(challenge.challenge_id, "challenge-001");
        assert_eq!(challenge.challenger, challenger);
        assert_eq!(challenge.challenged_subject, subject);
        assert_eq!(challenge.status, ChallengeStatus::Pending);
    }

    #[test]
    fn test_trust_attestation_store() {
        let mut store = InMemoryTrustAttestationStore::new();
        let subject = Did::new("key", "subject123");
        let context = TrustContext::General;
        
        // Create and store a trust record
        let mut record = MultiPartyTrustRecord::new(subject.clone(), context.clone());
        record.aggregated_score = 0.8;
        
        store.store_trust_record(record.clone()).unwrap();
        
        // Retrieve the record
        let retrieved = store.get_trust_record(&subject, &context).unwrap();
        assert_eq!(retrieved.subject, subject);
        assert_eq!(retrieved.aggregated_score, 0.8);
        
        // Create and store a challenge
        let challenge = TrustChallenge::new(
            "challenge-001".to_string(),
            Did::new("key", "challenger"),
            subject.clone(),
            context.clone(),
            "Test challenge".to_string(),
            1234567890,
        );
        
        store.store_challenge(challenge.clone()).unwrap();
        
        // Retrieve the challenge
        let retrieved_challenge = store.get_challenge("challenge-001").unwrap();
        assert_eq!(retrieved_challenge.challenge_id, "challenge-001");
        
        // Test listing challenges by status
        let pending_challenges = store.list_challenges_by_status(ChallengeStatus::Pending);
        assert_eq!(pending_challenges.len(), 1);
    }

    #[test]
    fn test_audit_event_storage() {
        let mut store = InMemoryTrustAttestationStore::new();
        let subject = Did::new("key", "subject123");
        let actor = Did::new("key", "actor123");
        let context = TrustContext::General;
        
        let audit_event = TrustAuditEvent {
            event_id: "event-001".to_string(),
            event_type: TrustEventType::AttestationCreated,
            actor: actor.clone(),
            subject: subject.clone(),
            context: context.clone(),
            timestamp: 1234567890,
            data: serde_json::json!({"test": "data"}),
            dag_cid: None,
        };
        
        store.store_audit_event(audit_event.clone()).unwrap();
        
        let events = store.get_audit_events(&subject, &context);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, "event-001");
        assert_eq!(events[0].event_type, TrustEventType::AttestationCreated);
    }
}