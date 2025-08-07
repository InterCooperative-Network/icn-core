//! Example demonstrating the ICN Mana System
//!
//! This example shows how to set up and use the regenerative mana system
//! with different organization types, hardware configurations, and trust levels.

use icn_common::{FixedSystemInfoProvider, Did};
use icn_economics::{
    InMemoryLedger, DefaultHardwareMetricsProvider, InMemoryOrganizationProvider,
    InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider,
    RegenerativeManaLedger, OrganizationType, HardwareMetrics, ManaSystemConfig,
    BASE_MANA_CAP, REGEN_EPOCH_SECONDS,
};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🔋 ICN Mana System Example");
    println!("==========================\n");

    // Create system configuration
    let config = ManaSystemConfig::default();
    println!("📋 System Configuration:");
    println!("   Base Mana Cap: {}", config.base_mana_cap);
    println!("   Min Balance: {}", config.min_mana_balance);
    println!("   Regen Epoch: {} seconds\n", config.regen_epoch_seconds);

    // Set up the mana system components
    let system_info = FixedSystemInfoProvider::new(8, 16384);
    let hardware_provider = DefaultHardwareMetricsProvider::new(system_info);
    let org_provider = InMemoryOrganizationProvider::new();
    let trust_provider = InMemoryTrustProvider::new();
    let emergency_detector = SimpleEmergencyDetector::new();
    let network_health = SimpleNetworkHealthProvider::new(1.0);

    // Create the regenerative mana ledger
    let mana_ledger = RegenerativeManaLedger::new(
        InMemoryLedger::new(),
        hardware_provider,
        org_provider,
        trust_provider,
        emergency_detector,
        network_health,
    );

    // Create example participants
    let participants = setup_participants(&mana_ledger);
    
    println!("👥 Participants Setup:");
    for (name, did) in &participants {
        let account = mana_ledger.get_account_info(did).unwrap();
        println!("   {}: {:?} (Trust: {:.1}, Participation: {:.1})", 
                name, 
                account.organization_type, 
                account.trust_multiplier,
                account.participation_factor);
    }
    println!();

    // Demonstrate mana operations
    demonstrate_mana_operations(&mana_ledger, &participants);

    // Demonstrate regeneration calculation
    demonstrate_regeneration(&mana_ledger, &participants);

    // Demonstrate emergency modulation
    demonstrate_emergency_modulation(&mana_ledger, &participants);

    // Demonstrate compute score impact
    demonstrate_compute_scores(&mana_ledger, &participants);
}

fn setup_participants(mana_ledger: &RegenerativeManaLedger<InMemoryLedger, DefaultHardwareMetricsProvider<FixedSystemInfoProvider>, InMemoryOrganizationProvider, InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider>) -> HashMap<String, Did> {
    let mut participants = HashMap::new();

    // Alice - High-performance cooperative member
    let alice = Did::new("icn", "alice");
    mana_ledger.organization_provider.set_organization_type(alice.clone(), OrganizationType::Cooperative);
    mana_ledger.trust_provider.set_trust_multiplier(alice.clone(), 1.8);
    mana_ledger.trust_provider.set_participation_factor(alice.clone(), 1.4);
    mana_ledger.trust_provider.set_governance_engagement(alice.clone(), 1.3);
    
    let alice_hardware = HardwareMetrics {
        cpu_cores: 16,
        memory_mb: 32768,
        storage_gb: 2000,
        bandwidth_mbps: 1000,
        gpu_units: 4,
        uptime_percentage: 0.99,
        job_success_rate: 0.98,
    };
    mana_ledger.hardware_provider.update_metrics(&alice, alice_hardware);
    participants.insert("Alice".to_string(), alice);

    // Bob - Federation coordinator
    let bob = Did::new("icn", "bob");
    mana_ledger.organization_provider.set_organization_type(bob.clone(), OrganizationType::Federation);
    mana_ledger.organization_provider.set_federation_bonus(bob.clone(), 0.25);
    mana_ledger.trust_provider.set_trust_multiplier(bob.clone(), 1.6);
    mana_ledger.trust_provider.set_participation_factor(bob.clone(), 1.2);
    mana_ledger.trust_provider.set_governance_engagement(bob.clone(), 1.5);
    
    let bob_hardware = HardwareMetrics {
        cpu_cores: 8,
        memory_mb: 16384,
        storage_gb: 1000,
        bandwidth_mbps: 500,
        gpu_units: 2,
        uptime_percentage: 0.97,
        job_success_rate: 0.95,
    };
    mana_ledger.hardware_provider.update_metrics(&bob, bob_hardware);
    participants.insert("Bob".to_string(), bob);

    // Carol - Community organizer
    let carol = Did::new("icn", "carol");
    mana_ledger.organization_provider.set_organization_type(carol.clone(), OrganizationType::Community);
    mana_ledger.trust_provider.set_trust_multiplier(carol.clone(), 1.2);
    mana_ledger.trust_provider.set_participation_factor(carol.clone(), 1.1);
    mana_ledger.trust_provider.set_governance_engagement(carol.clone(), 1.4); // High governance engagement
    
    let carol_hardware = HardwareMetrics {
        cpu_cores: 4,
        memory_mb: 8192,
        storage_gb: 500,
        bandwidth_mbps: 100,
        gpu_units: 0,
        uptime_percentage: 0.95,
        job_success_rate: 0.92,
    };
    mana_ledger.hardware_provider.update_metrics(&carol, carol_hardware);
    participants.insert("Carol".to_string(), carol);

    // Dave - Unaffiliated newcomer
    let dave = Did::new("icn", "dave");
    mana_ledger.organization_provider.set_organization_type(dave.clone(), OrganizationType::Unaffiliated);
    mana_ledger.trust_provider.set_trust_multiplier(dave.clone(), 0.8);
    mana_ledger.trust_provider.set_participation_factor(dave.clone(), 0.6);
    mana_ledger.trust_provider.set_governance_engagement(dave.clone(), 0.7);
    participants.insert("Dave".to_string(), dave);

    participants
}

fn demonstrate_mana_operations(mana_ledger: &RegenerativeManaLedger<InMemoryLedger, DefaultHardwareMetricsProvider<FixedSystemInfoProvider>, InMemoryOrganizationProvider, InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider>, participants: &HashMap<String, Did>) {
    println!("💰 Initial Mana Balances:");
    for (name, did) in participants {
        let balance = mana_ledger.get_balance(did);
        let account = mana_ledger.get_account_info(did).unwrap();
        println!("   {}: {} / {} mana ({}% capacity)", 
                name, balance, account.max_capacity, 
                (balance as f64 / account.max_capacity as f64 * 100.0) as u32);
    }
    println!();

    // Demonstrate spending
    println!("💸 Spending Operations:");
    let alice = participants.get("Alice").unwrap();
    let bob = participants.get("Bob").unwrap();
    
    println!("   Alice spends 500 mana for DAG write...");
    if let Err(e) = mana_ledger.spend(alice, 500) {
        println!("   ❌ Failed: {}", e);
    } else {
        println!("   ✅ Success! New balance: {}", mana_ledger.get_balance(alice));
    }

    println!("   Bob attempts to spend 50,000 mana...");
    if let Err(e) = mana_ledger.spend(bob, 50_000) {
        println!("   ❌ Failed: {}", e);
    } else {
        println!("   ✅ Success! New balance: {}", mana_ledger.get_balance(bob));
    }
    println!();

    // Demonstrate crediting
    println!("💵 Crediting Operations:");
    let carol = participants.get("Carol").unwrap();
    println!("   Carol receives 1000 mana reward...");
    if let Err(e) = mana_ledger.credit(carol, 1000) {
        println!("   ❌ Failed: {}", e);
    } else {
        println!("   ✅ Success! New balance: {}", mana_ledger.get_balance(carol));
    }
    println!();
}

fn demonstrate_regeneration(mana_ledger: &RegenerativeManaLedger<InMemoryLedger, DefaultHardwareMetricsProvider<FixedSystemInfoProvider>, InMemoryOrganizationProvider, InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider>, participants: &HashMap<String, Did>) {
    println!("🔄 Regeneration Analysis:");
    println!("   Formula: R(t) = κ_org × σ × β × η × network_health_factor");
    println!();

    for (name, did) in participants {
        let account = mana_ledger.get_account_info(did).unwrap();
        let regen_rate = mana_ledger.calculate_regeneration_rate(&account);
        let max_capacity = mana_ledger.calculate_max_capacity(&account);
        
        // Calculate components
        let network_avg = HardwareMetrics::default(); // Simplified for demo
        let sigma = mana_ledger.calculate_compute_score(&account.hardware_metrics, &network_avg);
        let kappa_org = account.organization_type.weight();
        
        println!("   {}:", name);
        println!("     κ_org (org weight): {:.2}", kappa_org);
        println!("     σ (compute score): {:.2}", sigma);
        println!("     β (trust mult.): {:.2}", account.trust_multiplier);
        println!("     η (participation): {:.2}", account.participation_factor);
        println!("     → Regen rate: {:.2} mana/hour", regen_rate * 100.0);
        println!("     → Max capacity: {} mana", max_capacity);
        println!();
    }
}

fn demonstrate_emergency_modulation(mana_ledger: &RegenerativeManaLedger<InMemoryLedger, DefaultHardwareMetricsProvider<FixedSystemInfoProvider>, InMemoryOrganizationProvider, InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider>, participants: &HashMap<String, Did>) {
    println!("🚨 Emergency Modulation:");
    
    let alice = participants.get("Alice").unwrap();
    let account = mana_ledger.get_account_info(alice).unwrap();
    
    let normal_rate = mana_ledger.calculate_regeneration_rate(&account);
    
    println!("   Normal regeneration rate: {:.2} mana/hour", normal_rate * 100.0);
    
    // Activate emergency
    mana_ledger.emergency_detector.set_emergency(true);
    let emergency_rate = mana_ledger.calculate_regeneration_rate(&account);
    
    println!("   Emergency regeneration rate: {:.2} mana/hour", emergency_rate * 100.0);
    println!("   Reduction factor: {:.1}%", (1.0 - emergency_rate / normal_rate) * 100.0);
    
    // Deactivate emergency
    mana_ledger.emergency_detector.set_emergency(false);
    println!("   Emergency deactivated ✅");
    println!();
}

fn demonstrate_compute_scores(mana_ledger: &RegenerativeManaLedger<InMemoryLedger, DefaultHardwareMetricsProvider<FixedSystemInfoProvider>, InMemoryOrganizationProvider, InMemoryTrustProvider, SimpleEmergencyDetector, SimpleNetworkHealthProvider>, participants: &HashMap<String, Did>) {
    println!("🖥️  Compute Score Analysis:");
    println!("   Weighted sum: CPU×0.25 + RAM×0.20 + Storage×0.15 + Bandwidth×0.15");
    println!("                 + GPU×0.10 + Uptime×0.10 + Success×0.05");
    println!();

    let network_avg = HardwareMetrics {
        cpu_cores: 4,
        memory_mb: 8192,
        storage_gb: 500,
        bandwidth_mbps: 100,
        gpu_units: 0,
        uptime_percentage: 0.95,
        job_success_rate: 0.90,
    };
    
    // Update network average
    mana_ledger.update_network_average(network_avg.clone());

    for (name, did) in participants {
        let account = mana_ledger.get_account_info(did).unwrap();
        let hardware = &account.hardware_metrics;
        let score = mana_ledger.calculate_compute_score(hardware, &network_avg);
        
        println!("   {} Hardware:", name);
        println!("     CPU: {} cores", hardware.cpu_cores);
        println!("     RAM: {} MB", hardware.memory_mb);
        println!("     Storage: {} GB", hardware.storage_gb);
        println!("     Bandwidth: {} Mbps", hardware.bandwidth_mbps);
        println!("     GPU: {} units", hardware.gpu_units);
        println!("     Uptime: {:.1}%", hardware.uptime_percentage * 100.0);
        println!("     Success Rate: {:.1}%", hardware.job_success_rate * 100.0);
        println!("     → Compute Score: {:.2}x", score);
        println!();
    }
}
