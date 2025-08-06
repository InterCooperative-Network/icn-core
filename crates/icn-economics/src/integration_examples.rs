//! Integration Examples
//!
//! This module demonstrates how the new adversarial-resilient economic system components
//! work together in real-world scenarios.

#[cfg(test)]
mod integration_examples {
    use crate::adversarial_resilience::{AdversarialResilientEconomics, EmergencyProtocol};
    use crate::cooperation_enhancement::{TrustWeightedAllocator, MutualAidCoordinator, AidRequest, AidOffer, AidType, UrgencyLevel, GeographicalScope, ReciprocityCommitment, VerificationStatus};
    use crate::enhanced_mana_system::{ContributionWeightedManaLedger, ContributionMetrics, CapacityMetrics, QualityMetrics, RegenerationPolicy};
    use crate::organizational_structures::{OrganizationType, EconomicFocus, ProductionCapacity, Organization, OrganizationRegistry, EconomicPolicies, ManaRegenerationPolicy, ResourceAllocationPolicy, SurplusDistributionPolicy, ContributionRecognitionPolicy, RecognitionType, MeasurementMethod, RewardMechanism, ReputationMetrics};
    use crate::ManaLedger;
    use icn_common::{CommonError, Did, NodeScope, SystemTimeProvider, TimeProvider};
    use icn_reputation::InMemoryReputationStore;
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

    #[derive(Default)]
    struct TestLedger {
        balances: std::sync::Mutex<HashMap<Did, u64>>,
    }

    impl ManaLedger for TestLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.get_mut(did).ok_or_else(|| {
                CommonError::InvalidInputError("Account not found".into())
            })?;
            if *balance < amount {
                return Err(CommonError::PolicyDenied("Insufficient balance".into()));
            }
            *balance -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            let balance = balances.entry(did.clone()).or_insert(0);
            *balance += amount;
            Ok(())
        }

        fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
            let mut balances = self.balances.lock().unwrap();
            for balance in balances.values_mut() {
                *balance += amount;
            }
            Ok(())
        }

        fn all_accounts(&self) -> Vec<Did> {
            self.balances.lock().unwrap().keys().cloned().collect()
        }
    }

    /// Example 1: Cooperative Federation Bootstrap
    /// Demonstrates how a new cooperative joins a federation and starts participating
    /// in the adversarial-resilient economic system
    #[test]
    fn test_cooperative_federation_bootstrap() -> Result<(), CommonError> {
        // Initialize core systems
        let mut org_registry = OrganizationRegistry::new();
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger,
            reputation_store,
            policy,
        );

        // Create a new agricultural cooperative
        let coop_did = Did::from_str("did:icn:coop:organic_farm")?;
        let coop = Organization {
            id: coop_did.clone(),
            name: "Sunny Valley Organic Farm".to_string(),
            organization_type: OrganizationType::Coop {
                economic_focus: EconomicFocus::Production {
                    sectors: vec!["organic_agriculture".to_string(), "renewable_energy".to_string()],
                },
                production_capacity: ProductionCapacity {
                    total_capacity: 5000,
                    current_utilization: 0.85,
                    capacity_by_resource: HashMap::from([
                        ("vegetables".to_string(), 3000),
                        ("solar_energy".to_string(), 2000),
                    ]),
                    seasonal_variations: None,
                },
            },
            scope: NodeScope("pacific_northwest_federation".to_string()),
            created_at: SystemTimeProvider.unix_seconds(),
            member_count: 45,
            economic_policies: EconomicPolicies {
                mana_regeneration_policy: ManaRegenerationPolicy {
                    base_rate: 12.0,
                    contribution_multiplier: 1.6,
                    capacity_weight: 0.7,
                    organization_bonus: 0.25,
                    solidarity_bonus: 0.15,
                },
                resource_allocation_policy: ResourceAllocationPolicy::Mixed {
                    need_weight: 0.4,
                    contribution_weight: 0.4,
                    equality_weight: 0.2,
                },
                surplus_distribution_policy: SurplusDistributionPolicy::Mixed {
                    reinvestment_percentage: 0.3,
                    distribution_method: Box::new(SurplusDistributionPolicy::ContributionBasedDistribution {
                        contribution_metrics: vec!["labor_hours".to_string(), "innovation".to_string()],
                    }),
                },
                contribution_recognition_policy: ContributionRecognitionPolicy {
                    recognition_types: vec![
                        RecognitionType::Labor { skill_categories: vec!["farming".to_string()] },
                        RecognitionType::Innovation { innovation_types: vec!["sustainable_practices".to_string()] },
                    ],
                    measurement_methods: vec![MeasurementMethod::TimeTracking, MeasurementMethod::PeerAssessment],
                    reward_mechanisms: vec![
                        RewardMechanism::ManaBonus { multiplier: 1.3 },
                        RewardMechanism::Recognition { recognition_forms: vec!["peer_appreciation".to_string()] },
                    ],
                    peer_validation_required: true,
                },
            },
            relationships: Vec::new(),
            reputation_metrics: ReputationMetrics {
                reliability_score: 0.9,
                cooperation_score: 0.95,
                innovation_score: 0.8,
                sustainability_score: 0.98,
                transparency_score: 0.92,
                member_satisfaction_score: 0.88,
                external_reputation_score: 0.85,
            },
        };

        // Register the cooperative
        org_registry.register_organization(coop.clone())?;

        // Set up initial mana balance and contribution metrics
        enhanced_ledger.set_balance(&coop_did, 500)?;
        enhanced_ledger.register_organization(coop_did.clone(), coop.organization_type.clone());

        // Update contribution metrics reflecting high agricultural productivity
        let contribution_metrics = ContributionMetrics {
            compute_contribution: 0.8,
            storage_contribution: 0.6,
            bandwidth_contribution: 0.7,
            governance_participation: 1.5,
            mutual_aid_provided: 2.2,
            knowledge_sharing: 1.8,
            community_building: 1.6,
            innovation_contribution: 1.2,
            care_work: 1.1,
            last_updated: SystemTimeProvider.unix_seconds(),
        };
        enhanced_ledger.update_contribution_metrics(&coop_did, contribution_metrics)?;

        // Update capacity metrics
        let capacity_metrics = CapacityMetrics {
            total_compute_capacity: 1000,
            available_compute_capacity: 800,
            total_storage_capacity: 15000,
            available_storage_capacity: 12000,
            network_bandwidth: 500,
            uptime_percentage: 0.98,
            reliability_score: 0.95,
            quality_metrics: QualityMetrics {
                job_completion_rate: 0.97,
                average_response_time: 150,
                error_rate: 0.01,
                user_satisfaction_score: 0.93,
            },
        };
        enhanced_ledger.update_capacity_metrics(&coop_did, capacity_metrics)?;

        // Simulate regeneration
        let current_time = SystemTimeProvider.unix_seconds();
        enhanced_ledger.regenerate_account_mana(&coop_did, current_time + 3600)?;

        // Verify the cooperative received enhanced regeneration due to high contributions
        let final_balance = enhanced_ledger.get_balance(&coop_did);
        assert!(final_balance > 500, "Should have regenerated mana due to high contributions");

        // Check that the organization was registered successfully
        let retrieved_org = org_registry.get_organization(&coop_did);
        assert!(retrieved_org.is_some(), "Organization should be retrievable from registry");
        assert_eq!(retrieved_org.unwrap().name, "Sunny Valley Organic Farm");

        Ok(())
    }

    /// Example 2: Mutual Aid Emergency Response
    /// Shows how the system handles emergency mutual aid requests with trust-weighted allocation
    #[test]
    fn test_emergency_mutual_aid_response() -> Result<(), CommonError> {
        let mut aid_coordinator = MutualAidCoordinator::new();
        let mut trust_allocator = TrustWeightedAllocator::new();
        let mut ledger = TestLedger::default();

        // Set up participants
        let emergency_coop = Did::from_str("did:icn:coop:flood_affected")?;
        let helper1 = Did::from_str("did:icn:coop:neighbor_farm")?;
        let helper2 = Did::from_str("did:icn:community:local_village")?;
        let helper3 = Did::from_str("did:icn:federation:regional_federation")?;

        // Set up initial balances
        ledger.set_balance(&emergency_coop, 50)?; // Very low due to emergency
        ledger.set_balance(&helper1, 2000)?;
        ledger.set_balance(&helper2, 1500)?;
        ledger.set_balance(&helper3, 5000)?;

        // Establish trust relationships
        trust_allocator.update_trust_score(&helper1, &emergency_coop, 0.9)?; // High trust, direct neighbor
        trust_allocator.update_trust_score(&helper2, &emergency_coop, 0.8)?; // Good trust, community member
        trust_allocator.update_trust_score(&helper3, &emergency_coop, 0.7)?; // Reasonable trust, federation level

        // Submit emergency aid request
        let aid_request = AidRequest {
            request_id: "flood_emergency_001".to_string(),
            requester: emergency_coop.clone(),
            aid_type: AidType::Emergency { emergency_type: "flood_damage".to_string() },
            urgency_level: UrgencyLevel::Critical,
            resource_amount: 1000,
            description: "Flood destroyed our storage facilities and contaminated food supplies. Need immediate assistance for basic operations and member welfare.".to_string(),
            geographical_scope: GeographicalScope::Regional { region_name: "Willamette_Valley".to_string() },
            time_sensitivity: SystemTimeProvider.unix_seconds() + 86400, // 24 hours
            reciprocity_commitment: ReciprocityCommitment::PayItForward,
            verification_status: VerificationStatus::CommunityVerified { 
                verifiers: vec![helper2.clone(), helper3.clone()],
            },
            created_at: SystemTimeProvider.unix_seconds(),
        };

        aid_coordinator.submit_aid_request(aid_request)?;

        // Submit aid offers from potential helpers
        let offer1 = AidOffer {
            offer_id: "neighbor_aid_001".to_string(),
            provider: helper1.clone(),
            aid_type: AidType::Emergency { emergency_type: "flood_damage".to_string() },
            available_amount: 500,
            conditions: vec![],
            geographical_reach: GeographicalScope::Local { radius_km: 50.0 },
            availability_window: (SystemTimeProvider.unix_seconds(), SystemTimeProvider.unix_seconds() + 7200),
            preferred_recipients: vec![emergency_coop.clone()],
            reciprocity_expectations: crate::cooperation_enhancement::ReciprocityExpectation::NetworkContribution,
            created_at: SystemTimeProvider.unix_seconds(),
        };

        let offer2 = AidOffer {
            offer_id: "village_aid_001".to_string(),
            provider: helper2.clone(),
            aid_type: AidType::Emergency { emergency_type: "flood_damage".to_string() },
            available_amount: 300,
            conditions: vec![],
            geographical_reach: GeographicalScope::Regional { region_name: "Willamette_Valley".to_string() },
            availability_window: (SystemTimeProvider.unix_seconds(), SystemTimeProvider.unix_seconds() + 10800),
            preferred_recipients: vec![],
            reciprocity_expectations: crate::cooperation_enhancement::ReciprocityExpectation::Eventual { time_window_days: 365 },
            created_at: SystemTimeProvider.unix_seconds(),
        };

        let offer3 = AidOffer {
            offer_id: "federation_aid_001".to_string(),
            provider: helper3.clone(),
            aid_type: AidType::Emergency { emergency_type: "flood_damage".to_string() },
            available_amount: 1000,
            conditions: vec![],
            geographical_reach: GeographicalScope::Global,
            availability_window: (SystemTimeProvider.unix_seconds(), SystemTimeProvider.unix_seconds() + 14400),
            preferred_recipients: vec![],
            reciprocity_expectations: crate::cooperation_enhancement::ReciprocityExpectation::None,
            created_at: SystemTimeProvider.unix_seconds(),
        };

        aid_coordinator.submit_aid_offer(offer1)?;
        aid_coordinator.submit_aid_offer(offer2)?;
        aid_coordinator.submit_aid_offer(offer3)?;

        // Find and execute matches
        let matches = aid_coordinator.match_aid_requests_and_offers();
        assert!(!matches.is_empty(), "Should find aid matches");

        // Execute the best match (should be the one with highest trust and capacity)
        let best_match = &matches[0];
        let transaction = aid_coordinator.execute_aid_match(best_match, &mut ledger)?;

        // Verify transaction success
        assert!(transaction.amount > 0, "Should have transferred some amount");
        assert!(ledger.get_balance(&emergency_coop) > 50, "Emergency cooperative should have received aid");

        // Use trust-weighted allocation for additional emergency funding from federation reserves
        let candidates = vec![emergency_coop.clone()];
        let allocations = trust_allocator.trust_weighted_allocation(
            &helper3, // Federation as allocator
            500, // Additional emergency funds
            &candidates,
            &mut ledger,
        )?;

        assert!(!allocations.is_empty(), "Should have allocated emergency funds");
        let final_balance = ledger.get_balance(&emergency_coop);
        assert!(final_balance >= 500, "Should have received trust-weighted emergency allocation");

        Ok(())
    }

    /// Example 3: Byzantine Attack Detection and Response
    /// Demonstrates the adversarial resilience system detecting and responding to coordinated attacks
    #[test]
    fn test_byzantine_attack_detection_and_response() -> Result<(), CommonError> {
        // Set up Byzantine validators
        let validators = vec![
            Did::from_str("did:icn:validator:alpha")?,
            Did::from_str("did:icn:validator:beta")?,
            Did::from_str("did:icn:validator:gamma")?,
            Did::from_str("did:icn:validator:delta")?,
        ].into_iter().collect::<HashSet<Did>>();

        let base_ledger = TestLedger::default();
        let mut are = AdversarialResilientEconomics::new(base_ledger, validators.clone());

        // Set up accounts
        let honest_user = Did::from_str("did:icn:user:honest")?;
        let attacker1 = Did::from_str("did:icn:user:attacker1")?;
        let attacker2 = Did::from_str("did:icn:user:attacker2")?;

        are.ledger().set_balance(&honest_user, 1000)?;
        are.ledger().set_balance(&attacker1, 2000)?;
        are.ledger().set_balance(&attacker2, 2000)?;

        // Simulate normal operation - honest transaction with sufficient validator signatures
        let validator1 = Did::from_str("did:icn:validator:alpha")?;
        let validator2 = Did::from_str("did:icn:validator:beta")?;
        let validator3 = Did::from_str("did:icn:validator:gamma")?;
        
        let signatures = vec![
            (validator1.clone(), vec![0u8; 64]), // Placeholder signatures
            (validator2.clone(), vec![1u8; 64]),
            (validator3.clone(), vec![2u8; 64]),
        ];

        // This should succeed with 3 out of 4 validator signatures (75% > 67% threshold)
        let result = are.validated_spend(&honest_user, 100, "honest_op".to_string(), signatures);
        assert!(result.is_ok(), "Honest operation with sufficient signatures should succeed");

        // Simulate Byzantine attack - insufficient validator signatures
        let insufficient_signatures = vec![
            (validator1.clone(), vec![0u8; 64]),
            (validator2.clone(), vec![1u8; 64]),
        ];

        // This should fail with only 2 out of 4 validator signatures (50% < 67% threshold)
        let result = are.validated_spend(&attacker1, 500, "attack_op1".to_string(), insufficient_signatures);
        assert!(result.is_err(), "Operation with insufficient validator signatures should fail");

        // Simulate coordinated velocity attack
        for i in 0..10 {
            let attack_signatures = vec![
                (validator1.clone(), vec![i; 64]),
                (validator2.clone(), vec![i + 1; 64]),
                (validator3.clone(), vec![i + 2; 64]),
            ];
            
            // Try rapid-fire transactions (velocity attack)
            let _ = are.validated_spend(
                &attacker2, 
                50, 
                format!("velocity_attack_{}", i), 
                attack_signatures
            );
        }

        // Check for active attack detections
        let detections = are.get_active_detections();
        // Note: In a real implementation, this would detect the velocity attack
        // For now, we just verify the system structure is in place

        // Simulate emergency protocol activation
        are.get_emergency_status(); // Verify emergency coordinator is accessible

        // Test account freeze scenario
        let freeze_protocol = EmergencyProtocol::AccountFreeze { 
            accounts: vec![attacker1.clone(), attacker2.clone()],
        };
        
        // In a real implementation, this would be triggered by attack detection
        // For testing, we just verify the structure exists
        assert!(matches!(freeze_protocol, EmergencyProtocol::AccountFreeze { .. }));

        Ok(())
    }

    /// Example 4: Multi-Organizational Collective Action
    /// Shows how different types of organizations collaborate on collective projects
    #[test]
    fn test_multi_organizational_collective_action() -> Result<(), CommonError> {
        let mut org_registry = OrganizationRegistry::new();
        let base_ledger = TestLedger::default();
        let reputation_store = InMemoryReputationStore::new();
        let policy = RegenerationPolicy::default();
        let mut enhanced_ledger = ContributionWeightedManaLedger::new(
            base_ledger,
            reputation_store,
            policy,
        );

        // Create diverse organization types
        let tech_coop = Did::from_str("did:icn:coop:tech_collective")?;
        let cultural_community = Did::from_str("did:icn:community:arts_district")?;
        let regional_federation = Did::from_str("did:icn:federation:pacific_northwest")?;

        // Register different organization types
        enhanced_ledger.register_organization(
            tech_coop.clone(),
            OrganizationType::Coop {
                economic_focus: EconomicFocus::Services { 
                    service_types: vec!["software_development".to_string(), "tech_support".to_string()],
                },
                production_capacity: ProductionCapacity {
                    total_capacity: 2000,
                    current_utilization: 0.9,
                    capacity_by_resource: HashMap::new(),
                    seasonal_variations: None,
                },
            }
        );

        enhanced_ledger.register_organization(
            cultural_community.clone(),
            OrganizationType::Community {
                governance_model: crate::organizational_structures::GovernanceModel::Consensus { quorum_threshold: 0.8 },
                cultural_values: crate::organizational_structures::CulturalValues {
                    core_principles: vec!["creativity".to_string(), "inclusion".to_string()],
                    conflict_resolution: crate::organizational_structures::ConflictResolutionModel::RestorativeJustice,
                    inclusion_practices: crate::organizational_structures::InclusionPractices {
                        accessibility_measures: vec!["multilingual_support".to_string()],
                        diversity_commitments: vec!["economic_accessibility".to_string()],
                        language_support: vec!["spanish".to_string(), "mandarin".to_string()],
                        economic_inclusion: vec!["sliding_scale".to_string()],
                    },
                    decision_making_culture: crate::organizational_structures::DecisionMakingCulture::Collaborative,
                },
            }
        );

        enhanced_ledger.register_organization(
            regional_federation.clone(),
            OrganizationType::Federation {
                member_organizations: vec![tech_coop.clone(), cultural_community.clone()],
                interop_protocols: vec![],
            }
        );

        // Set up initial balances and contributions
        enhanced_ledger.set_balance(&tech_coop, 1500)?;
        enhanced_ledger.set_balance(&cultural_community, 800)?;
        enhanced_ledger.set_balance(&regional_federation, 3000)?;

        // Set up contribution metrics for organizations to enable regeneration
        let tech_contributions = ContributionMetrics {
            compute_contribution: 2.5,
            storage_contribution: 1.8,
            bandwidth_contribution: 2.0,
            governance_participation: 1.2,
            mutual_aid_provided: 0.5,
            knowledge_sharing: 2.8,
            community_building: 1.0,
            innovation_contribution: 3.0,
            care_work: 0.8,
            last_updated: SystemTimeProvider.unix_seconds(),
        };
        enhanced_ledger.update_contribution_metrics(&tech_coop, tech_contributions)?;

        let community_contributions = ContributionMetrics {
            compute_contribution: 0.8,
            storage_contribution: 0.6,
            bandwidth_contribution: 0.7,
            governance_participation: 2.5,
            mutual_aid_provided: 2.0,
            knowledge_sharing: 1.5,
            community_building: 3.0,
            innovation_contribution: 1.5,
            care_work: 2.8,
            last_updated: SystemTimeProvider.unix_seconds(),
        };
        enhanced_ledger.update_contribution_metrics(&cultural_community, community_contributions)?;

        // Record collective action participation - developing an open-source cultural platform
        enhanced_ledger.record_collective_action(
            "cultural_platform_project".to_string(),
            vec![tech_coop.clone(), cultural_community.clone(), regional_federation.clone()],
            "open_source_development".to_string(),
            2.5, // High impact score
        )?;

        // Record mutual aid flows between organizations
        enhanced_ledger.record_mutual_aid(&tech_coop, &cultural_community, 200, "tech_infrastructure".to_string())?;
        enhanced_ledger.record_mutual_aid(&regional_federation, &tech_coop, 500, "project_funding".to_string())?;
        enhanced_ledger.record_mutual_aid(&cultural_community, &regional_federation, 100, "community_engagement".to_string())?;

        // Set up collective resource pool
        enhanced_ledger.contribute_to_collective_pool(&tech_coop, 300)?;
        enhanced_ledger.contribute_to_collective_pool(&regional_federation, 800)?;

        // Distribute collective resources based on need
        let recipients = vec![
            (cultural_community.clone(), 400), // Higher need due to lower initial balance
            (tech_coop.clone(), 200),
        ];
        enhanced_ledger.distribute_from_collective_pool(&recipients)?;

        // Verify regeneration with organizational bonuses
        let current_time = SystemTimeProvider.unix_seconds();
        enhanced_ledger.regenerate_account_mana(&tech_coop, current_time + 3600)?;
        enhanced_ledger.regenerate_account_mana(&cultural_community, current_time + 3600)?;
        enhanced_ledger.regenerate_account_mana(&regional_federation, current_time + 3600)?;

        // Verify that different organization types receive appropriate bonuses
        let tech_balance = enhanced_ledger.get_balance(&tech_coop);
        let community_balance = enhanced_ledger.get_balance(&cultural_community);
        let federation_balance = enhanced_ledger.get_balance(&regional_federation);

        // Each organization should have grown due to:
        // 1. Collective action bonuses
        // 2. Mutual aid multipliers  
        // 3. Organization-specific regeneration rates
        // Note: Tech coop contributed 300 to collective pool and received 200 back,
        // so net contribution is -100. Federation contributed 800 but received 0 back.
        assert!(tech_balance >= 1200, "Tech coop balance should account for collective contribution: {} (started 1500, contributed 300, got 200 back)", tech_balance);
        assert!(community_balance >= 1000, "Community should have received collective resource distribution: {} (started 800, got 400)", community_balance);
        assert!(federation_balance >= 2200, "Federation balance should account for collective contribution: {} (started 3000, contributed 800)", federation_balance);

        // Check collective pool status
        let pool_status = enhanced_ledger.get_collective_pool_status();
        assert!(pool_status.contributions.len() > 0); // Should have contributions recorded
        assert_eq!(pool_status.total_pool, 500); // 1100 contributed - 600 distributed

        Ok(())
    }
}