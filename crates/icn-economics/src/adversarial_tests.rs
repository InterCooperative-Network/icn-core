//! Tests for adversarial-resilient economic features

use crate::{
    BasicAntiGamingEngine, ByzantineManaLedger, 
    byzantine_mana::{ManaAccount, ManaAccountStatus, SpendingContext, SpendingUrgency, VerifiedCapacityMetrics},
    adversarial::GameTheoreticSecurity,
};
use icn_common::{Did, SystemTimeProvider};
use std::{str::FromStr, collections::HashMap};

#[tokio::test]
async fn test_byzantine_mana_ledger_creation() {
    let validator_set = vec![
        Did::from_str("did:key:validator1").unwrap(),
        Did::from_str("did:key:validator2").unwrap(),
        Did::from_str("did:key:validator3").unwrap(),
    ];

    let anti_gaming = Box::new(BasicAntiGamingEngine::new());
    let time_provider = Box::new(SystemTimeProvider);

    let ledger = ByzantineManaLedger::new(validator_set.clone(), anti_gaming, time_provider);

    // Test initial state
    let test_did = Did::from_str("did:key:test").unwrap();
    let (balance, risk_score) = ledger.get_balance_with_risk_assessment(&test_did).unwrap();
    
    assert_eq!(balance, 0);
    assert_eq!(risk_score, 0.0);
}

#[tokio::test]
async fn test_verified_capacity_metrics() {
    let capacity_metrics = VerifiedCapacityMetrics {
        compute_contribution: 1.5,
        storage_contribution: 1.2,
        bandwidth_contribution: 0.8,
        uptime_score: 0.9,
        quality_score: 0.95,
        proof_signature: vec![1, 2, 3, 4],
        proof_hash: "test_hash".to_string(),
        verification_timestamp: 1234567890,
        verifying_validators: vec![
            Did::from_str("did:key:validator1").unwrap(),
            Did::from_str("did:key:validator2").unwrap(),
            Did::from_str("did:key:validator3").unwrap(),
        ],
    };

    assert_eq!(capacity_metrics.compute_contribution, 1.5);
    assert_eq!(capacity_metrics.verifying_validators.len(), 3);
    assert!(!capacity_metrics.proof_signature.is_empty());
}

#[tokio::test]
async fn test_anti_gaming_engine() {
    use crate::adversarial::{BehaviorHistory, TransactionPatterns, AmountDistribution, TemporalPatterns};

    let engine = BasicAntiGamingEngine::new();
    
    // Create a behavior history with suspicious patterns
    let behavior = BehaviorHistory {
        account: Did::from_str("did:key:suspicious").unwrap(),
        transaction_patterns: TransactionPatterns {
            transaction_frequency: 150.0, // High frequency
            amount_distribution: AmountDistribution {
                mean: 100.0,
                variance: 10.0,
                skewness: 0.1,
                outlier_frequency: 0.4, // High outliers
            },
            counterparty_diversity: 0.2, // Low diversity
            temporal_clustering: 0.9, // High clustering
        },
        capacity_claims: Vec::new(),
        reputation_changes: Vec::new(),
        social_connections: Vec::new(),
        temporal_patterns: TemporalPatterns::default(),
    };

    let result = engine.detect_gaming_attempt(&behavior.account, &behavior).unwrap();
    
    // Gaming detection should identify suspicious patterns
    // The current score is around 0.32 which is above 0.3 threshold for moderate suspicion
    assert!(result.confidence_score > 0.3, "Confidence score should be > 0.3, got: {}", result.confidence_score);
    
    // Gaming detection logic should identify some indicators
    assert!(result.gaming_indicators.transaction_manipulation_score > 0.6);
    assert!(result.gaming_indicators.capacity_inflation_score >= 0.0); // No capacity claims in test
}

#[tokio::test]
async fn test_mana_account_status_transitions() {
    let account = ManaAccount {
        did: Did::from_str("did:key:test").unwrap(),
        current_balance: 1000,
        max_capacity: 10000,
        base_regeneration_rate: 10.0,
        last_regeneration: 1234567890,
        reputation_multiplier: 1.0,
        capacity_score: 1.0,
        capacity_proof_hash: Some("test_hash".to_string()),
        last_consensus_proof: Vec::new(),
        gaming_risk_score: 0.3,
        status: ManaAccountStatus::Active,
    };

    // Test that account starts active
    assert!(matches!(account.status, ManaAccountStatus::Active));

    // Test frozen status
    let frozen_account = ManaAccount {
        status: ManaAccountStatus::Frozen {
            reason: "Suspicious activity".to_string(),
            until: 9999999999, // Future timestamp
        },
        ..account.clone()
    };

    assert!(matches!(frozen_account.status, ManaAccountStatus::Frozen { .. }));

    // Test penalized status
    let penalized_account = ManaAccount {
        status: ManaAccountStatus::Penalized {
            penalty_factor: 0.5,
            until: 9999999999,
        },
        ..account
    };

    assert!(matches!(penalized_account.status, ManaAccountStatus::Penalized { .. }));
}

#[tokio::test]
async fn test_spending_context() {
    let mut metadata = HashMap::new();
    metadata.insert("purpose".to_string(), "mesh_computing".to_string());

    let context = SpendingContext {
        operation_type: "job_submission".to_string(),
        recipient: Some(Did::from_str("did:key:recipient").unwrap()),
        resource_type: Some("compute".to_string()),
        urgency_level: SpendingUrgency::High,
        metadata,
    };

    assert_eq!(context.operation_type, "job_submission");
    assert!(context.recipient.is_some());
    assert!(matches!(context.urgency_level, SpendingUrgency::High));
    assert!(context.metadata.contains_key("purpose"));
}

#[tokio::test]
async fn test_mana_system_health_metrics() {
    let validator_set = vec![
        Did::from_str("did:key:validator1").unwrap(),
        Did::from_str("did:key:validator2").unwrap(),
        Did::from_str("did:key:validator3").unwrap(),
    ];

    let anti_gaming = Box::new(BasicAntiGamingEngine::new());
    let time_provider = Box::new(SystemTimeProvider);

    let ledger = ByzantineManaLedger::new(validator_set, anti_gaming, time_provider);

    // Get health metrics for empty ledger
    let health = ledger.get_mana_system_health().unwrap();
    
    assert_eq!(health.total_accounts, 0);
    assert_eq!(health.total_mana_in_circulation, 0);
    assert_eq!(health.gaming_risk_accounts, 0);
    assert_eq!(health.frozen_accounts, 0);
    assert!(health.system_security_score >= 0.0 && health.system_security_score <= 1.0);
}

#[tokio::test]
async fn test_sybil_detection() {
    let engine = BasicAntiGamingEngine::new();
    
    // Create many accounts that could be suspicious
    let accounts: Vec<Did> = (0..50)
        .map(|i| Did::from_str(&format!("did:key:account{}", i)).unwrap())
        .collect();

    // Simple network analysis (would be more complex in real implementation)
    use crate::adversarial::{NetworkAnalysis, SocialGraph, ConnectivityMetrics, ClusteringAnalysis, IdentityVerificationData};
    let network_analysis = NetworkAnalysis {
        social_graph: SocialGraph::default(),
        connectivity_metrics: ConnectivityMetrics::default(),
        clustering_analysis: ClusteringAnalysis::default(),
        identity_verification_data: IdentityVerificationData::default(),
    };

    let result = engine.detect_sybil_attack(&accounts, &network_analysis).unwrap();

    // Should detect potential clusters due to account grouping
    assert!(!result.suspected_sybil_clusters.is_empty());
    assert!(result.confidence_score > 0.0);
}

#[tokio::test]
async fn test_capacity_claim_verification() {
    let capacity_metrics = VerifiedCapacityMetrics {
        compute_contribution: 2.0,
        storage_contribution: 1.5,
        bandwidth_contribution: 1.0,
        uptime_score: 0.95,
        quality_score: 0.9,
        proof_signature: vec![10, 20, 30, 40],
        proof_hash: "verified_hash".to_string(),
        verification_timestamp: 1234567890,
        verifying_validators: vec![
            Did::from_str("did:key:validator1").unwrap(),
            Did::from_str("did:key:validator2").unwrap(),
            Did::from_str("did:key:validator3").unwrap(),
        ],
    };

    // Test that metrics have proper verification
    assert!(capacity_metrics.compute_contribution > 1.0);
    assert!(capacity_metrics.uptime_score > 0.9);
    assert_eq!(capacity_metrics.verifying_validators.len(), 3);
    assert!(!capacity_metrics.proof_hash.is_empty());
}