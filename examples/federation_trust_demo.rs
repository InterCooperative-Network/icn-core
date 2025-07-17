//! Example demonstrating the Scoped Federation Trust Framework
//!
//! This example shows how to:
//! 1. Set up federation trust contexts
//! 2. Configure trust inheritance models
//! 3. Create cross-federation trust bridges
//! 4. Use the trust policy engine for validation

use icn_identity::{
    TrustContext, FederationId, ScopedTrustRelationship, TrustInheritance,
    FederationTrustBridge, BridgeConfig, TrustPolicyRule, TrustPolicyEngine,
    TrustValidationResult, TrustLevel, TrustRelationship,
};
use icn_governance::{
    FederationGovernanceEngine, TrustAwareGovernancePolicy, GovernanceAction,
    ProposalId,
};
use icn_common::Did;
use std::collections::{HashMap, HashSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 ICN Scoped Federation Trust Framework Demo");
    println!("==============================================\n");

    // === 1. Set up Federation Trust Contexts ===
    println!("📋 1. Setting up Federation Trust Contexts");
    
    let mut trust_engine = TrustPolicyEngine::new();
    
    // Create federations
    let housing_federation = FederationId::new("cooperative_housing_network".to_string());
    let tech_federation = FederationId::new("tech_worker_collective".to_string());
    let energy_federation = FederationId::new("renewable_energy_coop".to_string());
    
    println!("   ✓ Created federations:");
    println!("     - {}", housing_federation.as_str());
    println!("     - {}", tech_federation.as_str());
    println!("     - {}", energy_federation.as_str());
    
    // Create cooperative DIDs
    let alice_housing = Did::new("key", "alice_housing_coop");
    let bob_tech = Did::new("key", "bob_tech_coop");
    let charlie_energy = Did::new("key", "charlie_energy_coop");
    
    // Add federation memberships
    trust_engine.add_federation_membership(alice_housing.clone(), housing_federation.clone());
    trust_engine.add_federation_membership(bob_tech.clone(), tech_federation.clone());
    trust_engine.add_federation_membership(charlie_energy.clone(), energy_federation.clone());
    
    println!("   ✓ Added federation memberships\n");

    // === 2. Configure Trust Inheritance Models ===
    println!("🧬 2. Configuring Trust Inheritance Models");
    
    // Create inheritance configuration for governance
    let governance_inheritance = TrustInheritance {
        inheritable: true,
        max_depth: Some(3),
        degradation_factor: 0.8, // 20% degradation per level
        min_inherited_level: TrustLevel::Basic,
    };
    
    // Create inheritance configuration for resource sharing
    let resource_inheritance = TrustInheritance {
        inheritable: true,
        max_depth: Some(2),
        degradation_factor: 0.7, // 30% degradation per level
        min_inherited_level: TrustLevel::Partial,
    };
    
    println!("   ✓ Governance inheritance: degradation={}, max_depth={:?}", 
             governance_inheritance.degradation_factor, governance_inheritance.max_depth);
    println!("   ✓ Resource inheritance: degradation={}, max_depth={:?}\n", 
             resource_inheritance.degradation_factor, resource_inheritance.max_depth);

    // === 3. Create Scoped Trust Relationships ===
    println!("🤝 3. Creating Scoped Trust Relationships");
    
    // Governance trust within housing federation
    let governance_trust = ScopedTrustRelationship {
        base: TrustRelationship::new(
            alice_housing.clone(),
            bob_tech.clone(),
            TrustLevel::Partial,
            vec!["governance".to_string()],
        ),
        context: TrustContext::Governance,
        federation: Some(housing_federation.clone()),
        inheritance: governance_inheritance,
        metadata: [("established_reason".to_string(), "successful_collaboration".to_string())]
            .into_iter().collect(),
    };
    
    // Resource sharing trust
    let resource_trust = ScopedTrustRelationship {
        base: TrustRelationship::new(
            alice_housing.clone(),
            charlie_energy.clone(),
            TrustLevel::Full,
            vec!["resource_sharing".to_string()],
        ),
        context: TrustContext::ResourceSharing,
        federation: Some(housing_federation.clone()),
        inheritance: resource_inheritance,
        metadata: [("established_reason".to_string(), "energy_efficiency_project".to_string())]
            .into_iter().collect(),
    };
    
    // Mutual credit trust
    let credit_trust = ScopedTrustRelationship {
        base: TrustRelationship::new(
            bob_tech.clone(),
            charlie_energy.clone(),
            TrustLevel::Basic,
            vec!["mutual_credit".to_string()],
        ),
        context: TrustContext::MutualCredit,
        federation: Some(tech_federation.clone()),
        inheritance: TrustInheritance::default(),
        metadata: HashMap::new(),
    };
    
    trust_engine.add_federation_trust(housing_federation.clone(), governance_trust);
    trust_engine.add_federation_trust(housing_federation.clone(), resource_trust);
    trust_engine.add_federation_trust(tech_federation.clone(), credit_trust);
    
    println!("   ✓ Created governance trust: {} → {} (Partial)",
             alice_housing.to_string(), bob_tech.to_string());
    println!("   ✓ Created resource trust: {} → {} (Full)",
             alice_housing.to_string(), charlie_energy.to_string());
    println!("   ✓ Created mutual credit trust: {} → {} (Basic)\n",
             bob_tech.to_string(), charlie_energy.to_string());

    // === 4. Create Cross-Federation Trust Bridges ===
    println!("🌉 4. Creating Cross-Federation Trust Bridges");
    
    // Configure bridge between housing and tech federations
    let housing_tech_bridge_config = BridgeConfig {
        bidirectional: true,
        allowed_contexts: [TrustContext::ResourceSharing, TrustContext::MutualCredit]
            .into_iter().collect(),
        max_bridge_trust: TrustLevel::Partial,
        bridge_degradation: 0.6, // 40% degradation when crossing bridge
    };
    
    // Configure bridge between tech and energy federations
    let tech_energy_bridge_config = BridgeConfig {
        bidirectional: false, // One-way trust
        allowed_contexts: [TrustContext::ResourceSharing].into_iter().collect(),
        max_bridge_trust: TrustLevel::Basic,
        bridge_degradation: 0.5, // 50% degradation
    };
    
    // Create bridge trust relationships
    let housing_tech_bridge = FederationTrustBridge {
        from_federation: housing_federation.clone(),
        to_federation: tech_federation.clone(),
        trust: ScopedTrustRelationship {
            base: TrustRelationship::new(
                Did::new("key", "housing_federation_admin"),
                Did::new("key", "tech_federation_admin"),
                TrustLevel::Partial,
                vec!["resource_sharing".to_string(), "mutual_credit".to_string()],
            ),
            context: TrustContext::General,
            federation: Some(housing_federation.clone()),
            inheritance: TrustInheritance::default(),
            metadata: HashMap::new(),
        },
        bridge_config: housing_tech_bridge_config,
        established_at: chrono::Utc::now().timestamp() as u64,
        expires_at: None,
    };
    
    let tech_energy_bridge = FederationTrustBridge {
        from_federation: tech_federation.clone(),
        to_federation: energy_federation.clone(),
        trust: ScopedTrustRelationship {
            base: TrustRelationship::new(
                Did::new("key", "tech_federation_admin"),
                Did::new("key", "energy_federation_admin"),
                TrustLevel::Basic,
                vec!["resource_sharing".to_string()],
            ),
            context: TrustContext::ResourceSharing,
            federation: Some(tech_federation.clone()),
            inheritance: TrustInheritance::default(),
            metadata: HashMap::new(),
        },
        bridge_config: tech_energy_bridge_config,
        established_at: chrono::Utc::now().timestamp() as u64,
        expires_at: None,
    };
    
    trust_engine.add_bridge(housing_tech_bridge);
    trust_engine.add_bridge(tech_energy_bridge);
    
    println!("   ✓ Created bidirectional bridge: Housing ↔ Tech");
    println!("     - Contexts: ResourceSharing, MutualCredit");
    println!("     - Max trust: Partial, Degradation: 40%");
    println!("   ✓ Created unidirectional bridge: Tech → Energy");
    println!("     - Contexts: ResourceSharing");
    println!("     - Max trust: Basic, Degradation: 50%\n");

    // === 5. Configure Trust Policy Engine ===
    println!("⚙️  5. Configuring Trust Policy Rules");
    
    let policies = vec![
        TrustPolicyRule {
            name: "governance_strict".to_string(),
            applicable_contexts: [TrustContext::Governance].into_iter().collect(),
            min_trust_level: TrustLevel::Partial,
            require_federation_membership: true,
            max_inheritance_depth: Some(2),
            allow_cross_federation: false,
            custom_validator: None,
        },
        TrustPolicyRule {
            name: "resource_sharing_moderate".to_string(),
            applicable_contexts: [TrustContext::ResourceSharing].into_iter().collect(),
            min_trust_level: TrustLevel::Basic,
            require_federation_membership: false,
            max_inheritance_depth: Some(3),
            allow_cross_federation: true,
            custom_validator: None,
        },
        TrustPolicyRule {
            name: "mutual_credit_flexible".to_string(),
            applicable_contexts: [TrustContext::MutualCredit].into_iter().collect(),
            min_trust_level: TrustLevel::Basic,
            require_federation_membership: false,
            max_inheritance_depth: Some(2),
            allow_cross_federation: true,
            custom_validator: None,
        },
        TrustPolicyRule {
            name: "data_sharing_strict".to_string(),
            applicable_contexts: [TrustContext::DataSharing].into_iter().collect(),
            min_trust_level: TrustLevel::Full,
            require_federation_membership: true,
            max_inheritance_depth: None,
            allow_cross_federation: false,
            custom_validator: None,
        },
    ];
    
    for policy in policies {
        println!("   ✓ Added policy: {} (min_trust: {:?})", 
                 policy.name, policy.min_trust_level);
        trust_engine.add_rule(policy);
    }
    
    println!();

    // === 6. Test Trust Validation Scenarios ===
    println!("🧪 6. Testing Trust Validation Scenarios");
    
    let test_scenarios = vec![
        (
            &alice_housing,
            &bob_tech,
            &TrustContext::Governance,
            "vote_on_proposal",
            "Governance between housing and tech cooperatives",
        ),
        (
            &alice_housing,
            &charlie_energy,
            &TrustContext::ResourceSharing,
            "share_compute_resources",
            "Resource sharing between housing and energy cooperatives",
        ),
        (
            &bob_tech,
            &charlie_energy,
            &TrustContext::MutualCredit,
            "credit_transaction",
            "Mutual credit between tech and energy cooperatives",
        ),
        (
            &alice_housing,
            &bob_tech,
            &TrustContext::DataSharing,
            "share_sensitive_data",
            "Data sharing (should be denied due to strict policy)",
        ),
    ];
    
    for (i, (trustor, trustee, context, operation, description)) in test_scenarios.iter().enumerate() {
        println!("   Test {}: {}", i + 1, description);
        
        let result = trust_engine.validate_trust(trustor, trustee, context, operation);
        
        match result {
            TrustValidationResult::Allowed { effective_trust, trust_path } => {
                println!("     ✅ ALLOWED - Effective trust: {:?}", effective_trust);
                if !trust_path.is_empty() {
                    println!("     📍 Trust path: {}", trust_path.join(" → "));
                }
            }
            TrustValidationResult::Denied { reason } => {
                println!("     ❌ DENIED - Reason: {}", reason);
            }
        }
        println!();
    }

    // === 7. Demonstrate Federation Governance ===
    println!("🏛️  7. Demonstrating Federation Governance");
    
    let governance_engine = FederationGovernanceEngine::new(
        trust_engine,
        Some(housing_federation.clone())
    );
    
    println!("   ✓ Created federation governance engine for: {}", 
             housing_federation.as_str());
    
    // Show federation memberships
    println!("   📋 Federation memberships:");
    if let Some(memberships) = governance_engine.trust_engine.get_federation_memberships(&alice_housing) {
        for federation in memberships {
            println!("     - {} is member of: {}", alice_housing.to_string(), federation.as_str());
        }
    }
    
    println!("\n🎉 Federation Trust Framework Demo Complete!");
    println!("===============================================");
    println!("The framework successfully demonstrates:");
    println!("✓ Multiple trust contexts for different activities");
    println!("✓ Trust inheritance with configurable degradation");
    println!("✓ Cross-federation trust bridges");
    println!("✓ Configurable trust policy validation");
    println!("✓ Integration with federation governance");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demonstration_example() {
        // This test runs the main demonstration function
        assert!(main().is_ok());
    }
}