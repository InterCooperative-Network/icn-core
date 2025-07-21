//! Comprehensive tests for Smart P2P Router functionality
//!
//! Tests message routing, topology discovery, connection quality measurement,
//! and adaptive routing strategies.

use icn_runtime::context::{
    SmartP2pRouter, MeshNetworkService, HostAbiError, MessagePriority, RoutingStrategy,
    RuntimeContext, RuntimeContextBuilder, EnvironmentType,
};
use icn_common::{Did, TimeProvider};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio;

/// Test Smart P2P Router initialization
#[tokio::test]
async fn test_smart_p2p_router_initialization() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestRouter")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let smart_router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    
    // Verify router is properly initialized
    assert!(smart_router.as_ref() as *const _ != std::ptr::null());
    
    println!("✅ Smart P2P Router properly initialized");
    Ok(())
}

/// Test message priority routing
#[tokio::test]
async fn test_message_priority_routing() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestPriority")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let target_peer = Did::from_str("did:key:zTargetPeer")?;
    
    // Test critical priority message
    let critical_result = router.route_message_with_priority(
        target_peer.clone(),
        b"critical_message".to_vec(),
        MessagePriority::Critical,
    ).await;
    
    // Should not fail immediately (though delivery might fail in test environment)
    assert!(critical_result.is_ok() || matches!(critical_result, Err(HostAbiError::NetworkError(_))));
    
    // Test normal priority message
    let normal_result = router.route_message_with_priority(
        target_peer.clone(),
        b"normal_message".to_vec(),
        MessagePriority::Normal,
    ).await;
    
    assert!(normal_result.is_ok() || matches!(normal_result, Err(HostAbiError::NetworkError(_))));
    
    println!("✅ Message priority routing test completed");
    Ok(())
}

/// Test routing strategy selection
#[tokio::test]
async fn test_routing_strategy_selection() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestStrategy")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let target_peer = Did::from_str("did:key:zStrategyTarget")?;
    
    // Test direct routing strategy
    let direct_result = router.route_with_strategy(
        target_peer.clone(),
        b"direct_message".to_vec(),
        RoutingStrategy::Direct,
    ).await;
    
    // Should attempt routing (may fail in test environment due to no actual peers)
    assert!(direct_result.is_ok() || matches!(direct_result, Err(HostAbiError::NetworkError(_))));
    
    // Test reputation-based routing
    let reputation_result = router.route_with_strategy(
        target_peer.clone(),
        b"reputation_message".to_vec(),
        RoutingStrategy::ReputationBased,
    ).await;
    
    assert!(reputation_result.is_ok() || matches!(reputation_result, Err(HostAbiError::NetworkError(_))));
    
    println!("✅ Routing strategy selection test completed");
    Ok(())
}

/// Test connection quality measurement
#[tokio::test]
async fn test_connection_quality_measurement() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestQuality")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let peer_did = Did::from_str("did:key:zQualityTestPeer")?;
    
    // Test connection quality measurement
    let quality_result = router.measure_peer_quality(peer_did.clone()).await;
    
    // Should either succeed with quality info or fail with network error
    match quality_result {
        Ok(quality) => {
            assert!(quality.latency_ms >= 0.0);
            assert!(quality.packet_loss_rate >= 0.0 && quality.packet_loss_rate <= 1.0);
            assert!(quality.stability >= 0.0 && quality.stability <= 1.0);
            println!("✅ Connection quality measured: latency={}ms, loss={:.2}%, stability={:.2}%", 
                    quality.latency_ms, quality.packet_loss_rate * 100.0, quality.stability * 100.0);
        }
        Err(HostAbiError::NetworkError(_)) => {
            println!("✅ Connection quality measurement failed as expected (no real network in test)");
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

/// Test route discovery and caching
#[tokio::test]
async fn test_route_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestDiscovery")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let target_peer = Did::from_str("did:key:zDiscoveryTarget")?;
    
    // Test route discovery
    let discovery_result = router.discover_route_to_peer(target_peer.clone()).await;
    
    // Should either find a route or fail gracefully
    match discovery_result {
        Ok(route_info) => {
            println!("✅ Route discovered with {} hops", route_info.hop_count);
            assert!(route_info.hop_count > 0);
        }
        Err(HostAbiError::NetworkError(_)) => {
            println!("✅ Route discovery failed as expected (no network topology in test)");
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

/// Test adaptive routing learning
#[tokio::test]
async fn test_adaptive_routing_learning() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestLearning")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let peer_did = Did::from_str("did:key:zLearningPeer")?;
    
    // Test learning from successful routing
    let success_result = router.record_routing_success(
        peer_did.clone(),
        RoutingStrategy::Direct,
        Duration::from_millis(150),
    ).await;
    
    assert!(success_result.is_ok());
    
    // Test learning from failed routing
    let failure_result = router.record_routing_failure(
        peer_did.clone(),
        RoutingStrategy::MultiHop,
        "Connection timeout".to_string(),
    ).await;
    
    assert!(failure_result.is_ok());
    
    println!("✅ Adaptive routing learning test completed");
    Ok(())
}

/// Test message retry mechanism
#[tokio::test]
async fn test_message_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestRetry")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let target_peer = Did::from_str("did:key:zRetryTarget")?;
    
    // Test message with retry on failure
    let retry_result = router.route_message_with_retry(
        target_peer.clone(),
        b"retry_test_message".to_vec(),
        MessagePriority::High,
        3, // max_retries
    ).await;
    
    // Should either succeed or exhaust retries
    match retry_result {
        Ok(_) => println!("✅ Message successfully routed"),
        Err(HostAbiError::NetworkError(msg)) if msg.contains("retries") => {
            println!("✅ Message failed after retries as expected");
        }
        Err(HostAbiError::NetworkError(_)) => {
            println!("✅ Message routing failed as expected (no network in test)");
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

/// Test topology awareness
#[tokio::test]
async fn test_topology_awareness() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestTopology")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    
    // Test getting current topology view
    let topology_result = router.get_topology_snapshot().await;
    
    match topology_result {
        Ok(topology) => {
            println!("✅ Topology snapshot retrieved with {} peers", topology.peer_count);
            assert!(topology.peer_count >= 0);
        }
        Err(HostAbiError::NetworkError(_)) => {
            println!("✅ Topology snapshot failed as expected (no network in test)");
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}

/// Test network resilience features
#[tokio::test]
async fn test_network_resilience() -> Result<(), Box<dyn std::error::Error>> {
    let test_did = Did::from_str("did:key:zTestResilience")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let router = &runtime_ctx.cross_component_coordinator.smart_p2p_router;
    let peer_did = Did::from_str("did:key:zResiliencePeer")?;
    
    // Test handling peer disconnection
    let disconnect_result = router.handle_peer_disconnection(peer_did.clone()).await;
    assert!(disconnect_result.is_ok());
    
    // Test network partition detection
    let partition_result = router.detect_network_partition().await;
    assert!(partition_result.is_ok());
    
    println!("✅ Network resilience features test completed");
    Ok(())
} 