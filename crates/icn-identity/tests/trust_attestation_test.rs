//! Simple test for trust attestation without reputation dependency

#[cfg(test)]
mod tests {
    use icn_common::Did;
    use icn_identity::{
        cooperative_schemas::TrustLevel, did_key_from_verifying_key,
        federation_trust::TrustContext, generate_ed25519_keypair, ChallengeStatus,
        InMemoryTrustAttestationStore, KeyDidResolver, MultiPartyTrustRecord, TrustAttestation,
        TrustAttestationStore, TrustAuditEvent, TrustChallenge, TrustEventType,
    };
    use std::str::FromStr;

    #[test]
    fn test_trust_attestation_creation_and_signing() {
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
    fn test_multi_party_trust_record_management() {
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
        )
        .sign_with_key(&sk1)
        .unwrap();

        let (sk2, pk2) = generate_ed25519_keypair();
        let attester2 = Did::from_str(&did_key_from_verifying_key(&pk2)).unwrap();
        let attestation2 = TrustAttestation::new(
            attester2.clone(),
            subject.clone(),
            TrustContext::General,
            TrustLevel::Partial,
            1234567891,
            None,
        )
        .sign_with_key(&sk2)
        .unwrap();

        // Add attestations
        assert!(record.add_attestation(attestation1, 1234567890).is_ok());
        assert!(record.add_attestation(attestation2, 1234567891).is_ok());

        assert_eq!(record.attestations.len(), 2);
        assert_eq!(record.get_attesters().len(), 2);
        assert!(record.get_attesters().contains(&attester1));
        assert!(record.get_attesters().contains(&attester2));

        // Test simple aggregated scoring
        let score = record.calculate_aggregated_score_simple();
        assert!(score > 0.0 && score <= 1.0);
        // Should be average of 1.0 (Full) and 0.6 (Partial) = 0.8
        assert!((score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_trust_challenge_workflow() {
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

    #[test]
    fn test_attestation_verification_failures() {
        let (sk1, pk1) = generate_ed25519_keypair();
        let (sk2, pk2) = generate_ed25519_keypair();
        let attester = Did::from_str(&did_key_from_verifying_key(&pk1)).unwrap();
        let subject = Did::new("key", "subject123");

        let attestation = TrustAttestation::new(
            attester,
            subject,
            TrustContext::General,
            TrustLevel::Full,
            1234567890,
            None,
        )
        .sign_with_key(&sk1)
        .unwrap();

        // Verification should fail with wrong key
        assert!(attestation.verify_with_key(&pk2).is_err());
    }

    #[test]
    fn test_multi_party_record_context_validation() {
        let subject = Did::new("key", "subject123");
        let mut record = MultiPartyTrustRecord::new(subject.clone(), TrustContext::General);

        let (sk, pk) = generate_ed25519_keypair();
        let attester = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();

        // Try to add attestation with wrong context
        let wrong_context_attestation = TrustAttestation::new(
            attester,
            subject.clone(),
            TrustContext::Governance, // Wrong context
            TrustLevel::Full,
            1234567890,
            None,
        )
        .sign_with_key(&sk)
        .unwrap();

        let result = record.add_attestation(wrong_context_attestation, 1234567890);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("context does not match"));
    }
}
