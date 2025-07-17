//! Trust Verification Engine
//!
//! This module provides the main engine for trust verification and attestation
//! that integrates with DAG system for audit trails. Governance integration
//! is handled at a higher level to avoid circular dependencies.

use crate::{
    trust_attestation::*,
    federation_trust::TrustContext,

    DidResolver,
};
use icn_common::{Cid, CommonError, Did, TimeProvider};
use icn_reputation::ReputationStore;
use icn_dag::StorageService;
use icn_common::DagBlock;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Configuration for the trust verification engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustVerificationConfig {
    /// Minimum number of attestations required for trust establishment
    pub min_attestations: usize,
    /// Minimum aggregate reputation score required for attesters
    pub min_attester_reputation: u64,
    /// Time-to-live for trust attestations in seconds
    pub attestation_ttl: u64,
    /// Minimum reputation required to challenge trust
    pub min_challenger_reputation: u64,
}

impl Default for TrustVerificationConfig {
    fn default() -> Self {
        Self {
            min_attestations: 2,
            min_attester_reputation: 50,
            attestation_ttl: 86400 * 30, // 30 days
            min_challenger_reputation: 25,
        }
    }
}

/// Trust verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustVerificationResult {
    /// Whether trust is verified
    pub verified: bool,
    /// Aggregated trust score
    pub trust_score: f64,
    /// Number of valid attestations
    pub attestation_count: usize,
    /// Total reputation of attesters
    pub attester_reputation: u64,
    /// Issues found during verification
    pub issues: Vec<String>,
    /// CID of verification record in DAG
    pub verification_cid: Option<Cid>,
}

/// Main trust verification engine
pub struct TrustVerificationEngine<D, R, T>
where
    D: StorageService<DagBlock>,
    R: ReputationStore,
    T: TrustAttestationStore,
{
    config: TrustVerificationConfig,
    dag_store: Arc<Mutex<D>>,
    reputation_store: Arc<R>,
    attestation_store: Arc<Mutex<T>>,
    time_provider: Arc<dyn TimeProvider>,
}

impl<D, R, T> TrustVerificationEngine<D, R, T>
where
    D: StorageService<DagBlock>,
    R: ReputationStore,
    T: TrustAttestationStore,
{
    /// Create a new trust verification engine
    pub fn new(
        config: TrustVerificationConfig,
        dag_store: Arc<Mutex<D>>,
        reputation_store: Arc<R>,
        attestation_store: Arc<Mutex<T>>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            config,
            dag_store,
            reputation_store,
            attestation_store,
            time_provider,
        }
    }

    /// Submit a new trust attestation
    pub fn submit_attestation(
        &self,
        attestation: TrustAttestation,
        resolver: &dyn DidResolver,
    ) -> Result<Cid, CommonError> {
        let current_time = self.time_provider.unix_seconds();
        
        // Verify attestation signature
        attestation.verify_with_resolver(resolver)?;
        
        // Check attester reputation
        let attester_reputation = self.reputation_store.get_reputation(&attestation.attester);
        if attester_reputation < self.config.min_attester_reputation {
            return Err(CommonError::PermissionDenied(
                "Insufficient attester reputation".into(),
            ));
        }

        // Get or create trust record
        let mut store = self.attestation_store.lock().unwrap();
        let mut record = store
            .get_trust_record(&attestation.subject, &attestation.context)
            .unwrap_or_else(|| {
                MultiPartyTrustRecord::new(attestation.subject.clone(), attestation.context.clone())
            });

        // Add attestation to record
        record.add_attestation(attestation.clone(), current_time)?;
        
        // Recalculate aggregated score
        record.calculate_aggregated_score_simple();
        
        // Store updated record
        store.store_trust_record(record.clone())?;

        // Create audit event
        let audit_event = TrustAuditEvent {
            event_id: format!("attestation-{}-{}", attestation.attester, current_time),
            event_type: TrustEventType::AttestationCreated,
            actor: attestation.attester.clone(),
            subject: attestation.subject.clone(),
            context: attestation.context.clone(),
            timestamp: current_time,
            data: serde_json::to_value(&attestation).unwrap(),
            dag_cid: None,
        };
        
        store.store_audit_event(audit_event.clone())?;
        
        // Anchor audit event in DAG
        let dag_cid = self.anchor_in_dag(&audit_event)?;
        
        Ok(dag_cid)
    }

    /// Verify trust for a subject in a given context
    pub fn verify_trust(
        &self,
        subject: &Did,
        context: &TrustContext,
        resolver: &dyn DidResolver,
    ) -> Result<TrustVerificationResult, CommonError> {
        let current_time = self.time_provider.unix_seconds();
        
        // Get trust record
        let store = self.attestation_store.lock().unwrap();
        let record = match store.get_trust_record(subject, context) {
            Some(record) => record,
            None => {
                return Ok(TrustVerificationResult {
                    verified: false,
                    trust_score: 0.0,
                    attestation_count: 0,
                    attester_reputation: 0,
                    issues: vec!["No trust record found".to_string()],
                    verification_cid: None,
                });
            }
        };

        // Verify all attestations
        let verified_attesters = record.verify_all_attestations(resolver)?;
        let mut issues = Vec::new();
        
        // Check for expired attestations
        let valid_attestations: Vec<_> = record
            .attestations
            .iter()
            .filter(|a| {
                if current_time.saturating_sub(a.timestamp) > self.config.attestation_ttl {
                    issues.push(format!("Expired attestation from {}", a.attester));
                    false
                } else {
                    verified_attesters.contains(&a.attester)
                }
            })
            .collect();

        let attestation_count = valid_attestations.len();
        
        // Calculate total attester reputation
        let attester_reputation: u64 = valid_attestations
            .iter()
            .map(|a| self.reputation_store.get_reputation(&a.attester))
            .sum();

        // Check verification criteria
        let verified = attestation_count >= self.config.min_attestations
            && attester_reputation >= self.config.min_attester_reputation;

        if attestation_count < self.config.min_attestations {
            issues.push(format!(
                "Insufficient attestations: {} < {}",
                attestation_count, self.config.min_attestations
            ));
        }

        if attester_reputation < self.config.min_attester_reputation {
            issues.push(format!(
                "Insufficient attester reputation: {} < {}",
                attester_reputation, self.config.min_attester_reputation
            ));
        }

        // Create verification record and anchor in DAG
        let verification_record = TrustVerificationRecord {
            subject: subject.clone(),
            context: context.clone(),
            verified,
            trust_score: record.aggregated_score,
            attestation_count,
            attester_reputation,
            timestamp: current_time,
            issues: issues.clone(),
        };

        let verification_cid = self.anchor_in_dag(&verification_record)?;

        Ok(TrustVerificationResult {
            verified,
            trust_score: record.aggregated_score,
            attestation_count,
            attester_reputation,
            issues,
            verification_cid: Some(verification_cid),
        })
    }

    /// Submit a trust challenge
    pub fn submit_challenge(
        &self,
        challenger: Did,
        challenged_subject: Did,
        context: TrustContext,
        reason: String,
        evidence: Option<String>,
    ) -> Result<String, CommonError> {
        let current_time = self.time_provider.unix_seconds();
        
        // Check challenger reputation
        let challenger_reputation = self.reputation_store.get_reputation(&challenger);
        if challenger_reputation < self.config.min_challenger_reputation {
            return Err(CommonError::PermissionDenied(
                "Insufficient reputation to challenge trust".into(),
            ));
        }

        // Create challenge
        let challenge_id = format!("challenge-{}-{}-{}", challenger, challenged_subject, current_time);
        let mut challenge = TrustChallenge::new(
            challenge_id.clone(),
            challenger.clone(),
            challenged_subject.clone(),
            context.clone(),
            reason,
            current_time,
        );

        if let Some(evidence) = evidence {
            challenge.set_evidence(evidence);
        }

        // If governance approval is required, governance proposal would be created by external system
        // Store challenge for potential governance review
        let mut store = self.attestation_store.lock().unwrap();
        store.store_challenge(challenge.clone())?;

        // Create audit event
        let audit_event = TrustAuditEvent {
            event_id: format!("challenge-{}", challenge_id),
            event_type: TrustEventType::ChallengeCreated,
            actor: challenger,
            subject: challenged_subject,
            context,
            timestamp: current_time,
            data: serde_json::to_value(&challenge).unwrap(),
            dag_cid: None,
        };
        
        store.store_audit_event(audit_event.clone())?;
        
        // Anchor in DAG
        self.anchor_in_dag(&audit_event)?;

        Ok(challenge_id)
    }

    /// Resolve a trust challenge
    pub fn resolve_challenge(
        &self,
        challenge_id: &str,
        resolution: ChallengeResolution,
        resolver_did: Did,
    ) -> Result<(), CommonError> {
        let current_time = self.time_provider.unix_seconds();
        
        // Get and update challenge
        let mut store = self.attestation_store.lock().unwrap();
        let mut challenge = store
            .get_challenge(challenge_id)
            .ok_or_else(|| CommonError::IdentityError("Challenge not found".into()))?;

        let new_status = match resolution {
            ChallengeResolution::Accept => ChallengeStatus::Accepted,
            ChallengeResolution::Reject => ChallengeStatus::Rejected,
        };

        challenge.update_status(new_status.clone(), current_time);
        store.store_challenge(challenge.clone())?;

        // If challenge was accepted, update trust record
        if resolution == ChallengeResolution::Accept {
            if let Some(mut record) = store.get_trust_record(&challenge.challenged_subject, &challenge.context) {
                // For simplicity, remove all attestations - in practice, might be more nuanced
                record.attestations.clear();
                record.aggregated_score = 0.0;
                record.last_updated = current_time;
                store.store_trust_record(record)?;
            }
        }

        // Create audit event
        let audit_event = TrustAuditEvent {
            event_id: format!("challenge-resolved-{}", challenge_id),
            event_type: TrustEventType::ChallengeResolved,
            actor: resolver_did,
            subject: challenge.challenged_subject,
            context: challenge.context,
            timestamp: current_time,
            data: serde_json::json!({
                "challenge_id": challenge_id,
                "resolution": resolution,
                "status": new_status
            }),
            dag_cid: None,
        };
        
        store.store_audit_event(audit_event.clone())?;
        
        // Anchor in DAG
        self.anchor_in_dag(&audit_event)?;

        Ok(())
    }

    /// Get trust audit trail for a subject
    pub fn get_audit_trail(
        &self,
        subject: &Did,
        context: &TrustContext,
    ) -> Vec<TrustAuditEvent> {
        let store = self.attestation_store.lock().unwrap();
        store.get_audit_events(subject, context)
    }

    /// Anchor data in DAG store
    fn anchor_in_dag<S: Serialize>(&self, data: &S) -> Result<Cid, CommonError> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| CommonError::InternalError(format!("Serialization failed: {}", e)))?;
        
        // Create a DAG block from the serialized data
        let cid = icn_common::Cid::new_v1_sha256(0x55, &serialized);
        let block = DagBlock {
            cid: cid.clone(),
            data: serialized,
            links: Vec::new(),
            timestamp: self.time_provider.unix_seconds(),
            author_did: Did::new("system", "trust_verifier"), // System DID for trust verification
            signature: None,
            scope: None,
        };
        
        let mut dag_store = self.dag_store.lock().unwrap();
        dag_store.put(&block)?;
        Ok(block.cid)
    }
}

/// Trust verification record for DAG anchoring
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrustVerificationRecord {
    subject: Did,
    context: TrustContext,
    verified: bool,
    trust_score: f64,
    attestation_count: usize,
    attester_reputation: u64,
    timestamp: u64,
    issues: Vec<String>,
}

/// Challenge resolution options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeResolution {
    Accept,
    Reject,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        generate_ed25519_keypair, did_key_from_verifying_key, KeyDidResolver,
        InMemoryTrustAttestationStore, TrustLevel,
    };
    use icn_common::FixedTimeProvider;
    use icn_reputation::InMemoryReputationStore;
    use icn_dag::InMemoryDagStore;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    fn setup_test_engine() -> (
        TrustVerificationEngine<
            InMemoryDagStore,
            InMemoryReputationStore,
            InMemoryTrustAttestationStore,
        >,
        Arc<FixedTimeProvider>,
    ) {
        let config = TrustVerificationConfig::default();
        let dag_store = Arc::new(Mutex::new(InMemoryDagStore::new()));
        let reputation_store = Arc::new(InMemoryReputationStore::new());
        let attestation_store = Arc::new(Mutex::new(InMemoryTrustAttestationStore::new()));
        let time_provider = Arc::new(FixedTimeProvider::new(1234567890));

        let engine = TrustVerificationEngine::new(
            config,
            dag_store,
            reputation_store.clone(),
            attestation_store,
            time_provider.clone(),
        );

        (engine, time_provider)
    }

    #[test]
    fn test_submit_attestation() {
        let (engine, _time_provider) = setup_test_engine();
        let resolver = KeyDidResolver;

        // Create attester with sufficient reputation
        let (sk, pk) = generate_ed25519_keypair();
        let attester = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();
        engine.reputation_store.set_score(attester.clone(), 100);

        let subject = Did::new("key", "subject123");
        let attestation = TrustAttestation::new(
            attester,
            subject,
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            None,
        ).sign_with_key(&sk).unwrap();

        let result = engine.submit_attestation(attestation, &resolver);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_trust() {
        let (engine, _time_provider) = setup_test_engine();
        let resolver = KeyDidResolver;

        let subject = Did::new("key", "subject123");
        
        // Create multiple attesters with good reputation
        for i in 0..3 {
            let (sk, pk) = generate_ed25519_keypair();
            let attester = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();
            engine.reputation_store.set_score(attester.clone(), 80);

            let attestation = TrustAttestation::new(
                attester,
                subject.clone(),
                TrustContext::General,
                TrustLevel::Full,
                1234567890 + i,
                None,
            ).sign_with_key(&sk).unwrap();

            engine.submit_attestation(attestation, &resolver).unwrap();
        }

        let result = engine.verify_trust(&subject, &TrustContext::General, &resolver).unwrap();
        assert!(result.verified);
        assert!(result.trust_score > 0.0);
        assert_eq!(result.attestation_count, 3);
    }

    #[test]
    fn test_submit_challenge() {
        let (engine, _time_provider) = setup_test_engine();

        let challenger = Did::new("key", "challenger");
        let subject = Did::new("key", "subject");
        
        // Set sufficient reputation for challenger
        engine.reputation_store.set_score(challenger.clone(), 50);

        let result = engine.submit_challenge(
            challenger,
            subject,
            TrustContext::General,
            "Suspicious behavior".to_string(),
            Some("Evidence here".to_string()),
        );

        assert!(result.is_ok());
        let challenge_id = result.unwrap();
        assert!(challenge_id.starts_with("challenge-"));
    }

    #[test]
    fn test_insufficient_reputation_for_attestation() {
        let (engine, _time_provider) = setup_test_engine();
        let resolver = KeyDidResolver;

        // Create attester with insufficient reputation
        let (sk, pk) = generate_ed25519_keypair();
        let attester = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();
        engine.reputation_store.set_score(attester.clone(), 10); // Below minimum

        let subject = Did::new("key", "subject123");
        let attestation = TrustAttestation::new(
            attester,
            subject,
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            None,
        ).sign_with_key(&sk).unwrap();

        let result = engine.submit_attestation(attestation, &resolver);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient attester reputation"));
    }
}