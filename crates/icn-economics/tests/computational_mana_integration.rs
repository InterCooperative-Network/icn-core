//! Integration tests for computational mana system demonstrating
//! how mana generation is tied to computational resource contribution

use icn_common::{Did, SysinfoSystemInfoProvider, SystemTimeProvider};
use icn_economics::{ComputationalCapacity, ComputationalManaConfig, ComputationalManaService};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_computational_mana_full_integration() {
    // Create nodes with different computational contributions
    let high_contrib_node = Did::from_str("did:key:high-contributor").unwrap();
    let medium_contrib_node = Did::from_str("did:key:medium-contributor").unwrap();
    let low_contrib_node = Did::from_str("did:key:low-contributor").unwrap();

    let system_info = Arc::new(SysinfoSystemInfoProvider);
    let time_provider = Arc::new(SystemTimeProvider);

    let service = ComputationalManaService::new(
        ComputationalManaConfig::default(),
        system_info,
        time_provider,
    );

    // Register high-contribution node (powerful server)
    let high_capacity = ComputationalCapacity {
        cpu_cores: 32,
        memory_mb: 128 * 1024,        // 128GB
        storage_mb: 10 * 1024 * 1024, // 10TB
        network_mbps: 10000,          // 10Gbps
        gpu_compute_units: Some(8),   // High-end GPUs
    };

    service
        .update_node_contribution(
            high_contrib_node.clone(),
            high_capacity,
            0.99,   // 99% uptime
            1000,   // Many successful jobs
            10,     // Few failed jobs
            5000.0, // 5000 compute hours contributed
        )
        .await
        .unwrap();

    // Register medium-contribution node (typical workstation)
    let medium_capacity = ComputationalCapacity {
        cpu_cores: 8,
        memory_mb: 32 * 1024,        // 32GB
        storage_mb: 2 * 1024 * 1024, // 2TB
        network_mbps: 1000,          // 1Gbps
        gpu_compute_units: Some(2),  // Modest GPU
    };

    service
        .update_node_contribution(
            medium_contrib_node.clone(),
            medium_capacity,
            0.95,   // 95% uptime
            200,    // Moderate job completion
            20,     // Some failures
            1000.0, // 1000 compute hours
        )
        .await
        .unwrap();

    // Register low-contribution node (raspberry pi)
    let low_capacity = ComputationalCapacity {
        cpu_cores: 4,
        memory_mb: 4 * 1024,     // 4GB
        storage_mb: 128 * 1024,  // 128GB
        network_mbps: 100,       // 100Mbps
        gpu_compute_units: None, // No GPU
    };

    service
        .update_node_contribution(
            low_contrib_node.clone(),
            low_capacity,
            0.85,  // 85% uptime (less reliable)
            50,    // Few jobs completed
            15,    // Several failures
            100.0, // Only 100 compute hours
        )
        .await
        .unwrap();

    // Calculate mana regeneration rates
    let high_rate = service
        .calculate_mana_regeneration_rate(&high_contrib_node)
        .await
        .unwrap();
    let medium_rate = service
        .calculate_mana_regeneration_rate(&medium_contrib_node)
        .await
        .unwrap();
    let low_rate = service
        .calculate_mana_regeneration_rate(&low_contrib_node)
        .await
        .unwrap();

    println!("Mana regeneration rates:");
    println!("High contributor: {} mana/hour", high_rate);
    println!("Medium contributor: {} mana/hour", medium_rate);
    println!("Low contributor: {} mana/hour", low_rate);

    // Verify that rates reflect contribution levels
    assert!(
        high_rate > medium_rate,
        "High contributor should get more mana than medium"
    );
    assert!(
        medium_rate > low_rate,
        "Medium contributor should get more mana than low"
    );
    assert!(
        high_rate > low_rate * 3,
        "High contributor should get significantly more mana"
    );

    // Calculate maximum mana capacities
    let high_capacity_mana = service
        .calculate_max_mana_capacity(&high_contrib_node)
        .await
        .unwrap();
    let medium_capacity_mana = service
        .calculate_max_mana_capacity(&medium_contrib_node)
        .await
        .unwrap();
    let low_capacity_mana = service
        .calculate_max_mana_capacity(&low_contrib_node)
        .await
        .unwrap();

    println!("Maximum mana capacities:");
    println!("High contributor: {} mana max", high_capacity_mana);
    println!("Medium contributor: {} mana max", medium_capacity_mana);
    println!("Low contributor: {} mana max", low_capacity_mana);

    // Verify capacity scales with contribution
    assert!(high_capacity_mana > medium_capacity_mana);
    assert!(medium_capacity_mana > low_capacity_mana);

    // Check federation statistics
    let federation_stats = service.get_federation_stats().await.unwrap();
    assert_eq!(federation_stats.active_nodes, 3);
    assert!(federation_stats.total_capacity.cpu_cores >= 44); // Sum of all cores
    assert!(federation_stats.average_contribution_score > 0.0);

    println!(
        "Federation stats: {} active nodes, {} total CPU cores",
        federation_stats.active_nodes, federation_stats.total_capacity.cpu_cores
    );
}

#[tokio::test]
async fn test_demand_supply_affects_mana_generation() {
    let node_did = Did::from_str("did:key:test-node").unwrap();

    let system_info = Arc::new(SysinfoSystemInfoProvider);
    let time_provider = Arc::new(SystemTimeProvider);

    let service = ComputationalManaService::new(
        ComputationalManaConfig::default(),
        system_info,
        time_provider,
    );

    // Register a node
    let capacity = ComputationalCapacity {
        cpu_cores: 8,
        memory_mb: 16 * 1024,
        storage_mb: 1024 * 1024,
        network_mbps: 1000,
        gpu_compute_units: Some(2),
    };

    service
        .update_node_contribution(node_did.clone(), capacity, 0.95, 100, 5, 500.0)
        .await
        .unwrap();

    // Test with low demand (supply exceeds demand)
    service.update_demand_supply_ratio(0.5).await.unwrap();
    let rate_low_demand = service
        .calculate_mana_regeneration_rate(&node_did)
        .await
        .unwrap();

    // Test with high demand (demand exceeds supply)
    service.update_demand_supply_ratio(2.0).await.unwrap();
    let rate_high_demand = service
        .calculate_mana_regeneration_rate(&node_did)
        .await
        .unwrap();

    println!(
        "Mana rates - Low demand: {}, High demand: {}",
        rate_low_demand, rate_high_demand
    );

    // Low demand should result in higher mana generation (incentivize contribution)
    assert!(
        rate_low_demand > rate_high_demand,
        "Low demand should result in higher mana generation to incentivize contribution"
    );
}

#[tokio::test]
async fn test_insufficient_contribution_gets_no_mana() {
    let node_did = Did::from_str("did:key:insufficient-node").unwrap();

    let system_info = Arc::new(SysinfoSystemInfoProvider);
    let time_provider = Arc::new(SystemTimeProvider);

    let mut config = ComputationalManaConfig::default();
    config.minimum_contribution_threshold = 1000.0; // High threshold

    let service = ComputationalManaService::new(config, system_info, time_provider);

    // Register a node with very low contribution
    let weak_capacity = ComputationalCapacity {
        cpu_cores: 1,
        memory_mb: 512,   // 512MB
        storage_mb: 1024, // 1GB
        network_mbps: 10, // 10Mbps
        gpu_compute_units: None,
    };

    service
        .update_node_contribution(
            node_did.clone(),
            weak_capacity,
            0.5, // Poor uptime
            1,   // Very few jobs
            5,   // Many failures
            1.0, // Minimal compute hours
        )
        .await
        .unwrap();

    // Should get no mana regeneration due to insufficient contribution
    let rate = service
        .calculate_mana_regeneration_rate(&node_did)
        .await
        .unwrap();
    assert_eq!(
        rate, 0,
        "Insufficient contributors should get no mana regeneration"
    );

    // Should get minimal capacity
    let capacity = service
        .calculate_max_mana_capacity(&node_did)
        .await
        .unwrap();
    assert_eq!(
        capacity, 100,
        "Insufficient contributors should get minimal capacity"
    );
}

#[tokio::test]
async fn test_new_nodes_get_benefit_of_doubt() {
    let new_node_did = Did::from_str("did:key:new-node").unwrap();

    let system_info = Arc::new(SysinfoSystemInfoProvider);
    let time_provider = Arc::new(SystemTimeProvider);

    let service = ComputationalManaService::new(
        ComputationalManaConfig::default(),
        system_info,
        time_provider,
    );

    // Register a new node with good capacity but no job history
    let good_capacity = ComputationalCapacity {
        cpu_cores: 16,
        memory_mb: 64 * 1024,        // 64GB
        storage_mb: 4 * 1024 * 1024, // 4TB
        network_mbps: 1000,
        gpu_compute_units: Some(4),
    };

    service
        .update_node_contribution(
            new_node_did.clone(),
            good_capacity,
            1.0, // Perfect uptime so far
            0,   // No jobs completed yet
            0,   // No jobs failed yet
            0.0, // No compute hours yet
        )
        .await
        .unwrap();

    // Should get mana based on capacity despite no history
    let rate = service
        .calculate_mana_regeneration_rate(&new_node_did)
        .await
        .unwrap();
    assert!(
        rate > 0,
        "New nodes with good capacity should get mana regeneration"
    );

    let capacity = service
        .calculate_max_mana_capacity(&new_node_did)
        .await
        .unwrap();
    assert!(
        capacity > 1000,
        "New nodes with good capacity should get decent mana capacity"
    );

    println!(
        "New node with good capacity gets {} mana/hour, {} max capacity",
        rate, capacity
    );
}

#[tokio::test]
async fn test_local_computational_capacity_detection() {
    let system_info = Arc::new(SysinfoSystemInfoProvider);
    let time_provider = Arc::new(SystemTimeProvider);

    let service = ComputationalManaService::new(
        ComputationalManaConfig::default(),
        system_info,
        time_provider,
    );

    // Get local system capacity
    let local_capacity = service.get_local_computational_capacity().await.unwrap();

    // Should detect real system resources
    assert!(local_capacity.cpu_cores > 0, "Should detect CPU cores");
    assert!(local_capacity.memory_mb > 0, "Should detect memory");
    assert!(local_capacity.storage_mb > 0, "Should detect storage");
    assert!(
        local_capacity.network_mbps > 0,
        "Should have default network capacity"
    );

    println!(
        "Local system capacity: {} cores, {} MB memory, {} MB storage",
        local_capacity.cpu_cores, local_capacity.memory_mb, local_capacity.storage_mb
    );

    // Capacity should have reasonable values for a test system
    assert!(
        local_capacity.cpu_cores <= 256,
        "CPU cores should be reasonable"
    );
    assert!(
        local_capacity.memory_mb <= 1024 * 1024,
        "Memory should be reasonable"
    );
}
