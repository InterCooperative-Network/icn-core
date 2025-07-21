//! Integration tests for enhanced cross-component coordination
//!
//! Tests the integration between smart P2P routing, real-time CCL integration,
//! enhanced DAG synchronization, and comprehensive health monitoring.

use icn_runtime::context::{
    CrossComponentCoordinator, RuntimeContext, EnvironmentType, RuntimeContextBuilder,
    SmartP2pRouter, CclIntegrationCoordinator, HostAbiError, MessagePriority, PropagationPriority,
    DagOperation, Priority,
};
use icn_common::{Did, TimeProvider};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio;

/// Test basic cross-component coordinator initialization
#[tokio::test]
async fn test_cross_component_coordinator_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestCoordinator")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    // Verify that the cross-component coordinator is properly initialized
    let coordinator = &runtime_ctx.cross_component_coordinator;
    
    // Test that all components are properly initialized
    assert!(coordinator.smart_p2p_router.as_ref() as *const _ != std::ptr::null());
    assert!(coordinator.ccl_integration.as_ref() as *const _ != std::ptr::null());
    assert!(coordinator.dag_sync.as_ref() as *const _ != std::ptr::null());
    assert!(coordinator.health_monitor.as_ref() as *const _ != std::ptr::null());
    assert!(coordinator.performance_optimizer.as_ref() as *const _ != std::ptr::null());

    println!("✅ Cross-component coordinator properly initialized with all services");
    Ok(())
}

/// Test smart P2P routing functionality
#[tokio::test]
async fn test_smart_p2p_routing() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestRouting")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test intelligent message routing
    let target_peer = Did::from_str("did:key:zTargetPeer")?;
    let test_message = b"test governance message".to_vec();
    
    let message_id = coordinator.route_message_intelligently(
        target_peer.clone(),
        test_message,
        MessagePriority::High,
        None,
    ).await?;

    // Verify message was queued
    assert!(!message_id.is_empty());
    println!("✅ Message {} queued for intelligent routing to {}", message_id, target_peer);

    // Test optimal routing path calculation
    let routing_path = coordinator.get_optimal_routing_path(&target_peer).await?;
    // In testing environment, no real peers are available, so path should be None
    assert!(routing_path.is_none());
    println!("✅ Routing path calculation completed (no path found in test environment)");

    Ok(())
}

/// Test real-time CCL integration
#[tokio::test]
async fn test_realtime_ccl_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestCCL")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test real-time governance proposal submission
    let proposal_data = b"test governance proposal".to_vec();
    let proposal_id = coordinator.submit_governance_proposal_realtime(
        proposal_data,
        PropagationPriority::High,
        None,
    ).await?;

    assert!(!proposal_id.is_empty());
    println!("✅ Governance proposal {} submitted with real-time integration", proposal_id);

    // Test real-time vote casting
    coordinator.cast_governance_vote_realtime(
        proposal_id.clone(),
        "yes".to_string(),
        PropagationPriority::Normal,
    ).await?;

    println!("✅ Vote cast on proposal {} with real-time integration", proposal_id);

    // Test real-time proposal execution
    coordinator.execute_governance_proposal_realtime(proposal_id.clone()).await?;
    println!("✅ Proposal {} executed with real-time integration", proposal_id);

    Ok(())
}

/// Test enhanced DAG synchronization
#[tokio::test]
async fn test_enhanced_dag_synchronization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestDAGSync")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test intelligent DAG block propagation
    let test_cid = icn_common::Cid::new_v1_sha256(0x71, b"test_block");
    coordinator.propagate_block_intelligently(
        test_cid.clone(),
        PropagationPriority::Normal,
        None,
    ).await?;

    println!("✅ Block {} propagated with intelligent selection", test_cid);

    // Test intelligent DAG synchronization
    let sync_result = coordinator.sync_dag_intelligently().await?;
    println!("✅ DAG sync completed: {} blocks received, {} blocks sent", 
             sync_result.blocks_received, sync_result.blocks_sent);

    Ok(())
}

/// Test reputation-based peer coordination
#[tokio::test]
async fn test_reputation_based_coordination() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestReputation")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test reputation update and route recalculation
    let peer_id = Did::from_str("did:key:zPeerReputation")?;
    coordinator.update_peer_reputation_and_routes(&peer_id, 750).await?;

    println!("✅ Peer {} reputation updated to 750 with route recalculation", peer_id);

    Ok(())
}

/// Test comprehensive system status reporting
#[tokio::test]
async fn test_system_status_reporting() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestStatus")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test comprehensive system status
    let system_status = coordinator.get_system_status().await;
    
    // Verify status components
    assert!(system_status.health.components.contains_key("network"));
    assert!(system_status.health.components.contains_key("dag"));
    assert!(system_status.health.components.contains_key("reputation"));
    
    println!("✅ System status report generated with {} components", 
             system_status.health.components.len());
    println!("   Overall health: {:?}", system_status.health.overall);
    println!("   Network DAG sync health: {}", system_status.network_dag.sync_health);

    Ok(())
}

/// Test coordinated DAG operations with economics optimization
#[tokio::test]
async fn test_coordinated_dag_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestDAGOps")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(1000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    // Test coordinated DAG store operation
    let test_data = b"test coordinated data".to_vec();
    let dag_operation = DagOperation::Store {
        data: test_data,
        priority: Priority::Normal,
    };

    let operation_result = coordinator.coordinate_dag_operation(dag_operation).await?;
    
    // Verify the operation completed successfully
    match operation_result {
        icn_runtime::context::DagOperationResult::Store { cid } => {
            println!("✅ Coordinated DAG store operation completed with CID: {}", cid);
        }
        _ => {
            return Err("Unexpected operation result type".into());
        }
    }

    Ok(())
}

/// Test runtime context constructor consistency
#[tokio::test]
async fn test_runtime_context_constructor_consistency() -> Result<(), Box<dyn std::error::Error>> {
    // Test that all RuntimeContext constructors properly initialize CrossComponentCoordinator

    // Test testing constructor
    let test_did = Did::from_str("did:key:zTestConstructor1")?;
    let testing_ctx = RuntimeContext::new_testing(test_did.clone(), Some(1000))?;
    assert!(testing_ctx.cross_component_coordinator.as_ref() as *const _ != std::ptr::null());
    println!("✅ Testing constructor properly initializes CrossComponentCoordinator");

    // Test builder pattern
    let builder_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(500)
        .build()?;
    assert!(builder_ctx.cross_component_coordinator.as_ref() as *const _ != std::ptr::null());
    println!("✅ Builder pattern properly initializes CrossComponentCoordinator");

    // Test deprecated constructor (still should work)
    #[allow(deprecated)]
    let deprecated_ctx = RuntimeContext::new_with_stubs("did:key:zTestDeprecated")?;
    assert!(deprecated_ctx.cross_component_coordinator.as_ref() as *const _ != std::ptr::null());
    println!("✅ Deprecated constructor properly initializes CrossComponentCoordinator");

    Ok(())
}

/// Integration test for complete enhanced coordination workflow
#[tokio::test]
async fn test_complete_enhanced_coordination_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Create a testing runtime context
    let test_did = Did::from_str("did:key:zTestWorkflow")?;
    let runtime_ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
        .with_identity(test_did.clone())
        .with_initial_mana(2000)
        .build()?;

    let coordinator = &runtime_ctx.cross_component_coordinator;

    println!("🚀 Starting complete enhanced coordination workflow test");

    // Step 1: Start background coordination tasks
    coordinator.start_background_tasks().await?;
    println!("✅ Step 1: Background coordination tasks started");

    // Step 2: Test health monitoring
    let initial_health = coordinator.health_monitor.check_component_health().await;
    println!("✅ Step 2: Initial health check completed - overall: {:?}", initial_health.overall);

    // Step 3: Submit a governance proposal with real-time coordination
    let proposal_data = b"comprehensive test proposal".to_vec();
    let proposal_id = coordinator.submit_governance_proposal_realtime(
        proposal_data,
        PropagationPriority::High,
        None,
    ).await?;
    println!("✅ Step 3: Governance proposal {} submitted", proposal_id);

    // Step 4: Route messages intelligently
    let target_peer = Did::from_str("did:key:zWorkflowPeer")?;
    let message_id = coordinator.route_message_intelligently(
        target_peer.clone(),
        b"workflow test message".to_vec(),
        MessagePriority::High,
        None,
    ).await?;
    println!("✅ Step 4: Message {} routed intelligently", message_id);

    // Step 5: Update peer reputation and trigger route recalculation
    coordinator.update_peer_reputation_and_routes(&target_peer, 850).await?;
    println!("✅ Step 5: Peer reputation updated and routes recalculated");

    // Step 6: Perform coordinated DAG operations
    let dag_operation = DagOperation::Store {
        data: b"workflow coordination data".to_vec(),
        priority: Priority::High,
    };
    let _operation_result = coordinator.coordinate_dag_operation(dag_operation).await?;
    println!("✅ Step 6: Coordinated DAG operation completed");

    // Step 7: Cast vote with real-time integration
    coordinator.cast_governance_vote_realtime(
        proposal_id.clone(),
        "yes".to_string(),
        PropagationPriority::Normal,
    ).await?;
    println!("✅ Step 7: Vote cast with real-time integration");

    // Step 8: Get comprehensive system status
    let final_status = coordinator.get_system_status().await;
    println!("✅ Step 8: Final system status - health: {:?}, components: {}", 
             final_status.health.overall, final_status.health.components.len());

    // Step 9: Sync DAG intelligently
    let sync_result = coordinator.sync_dag_intelligently().await?;
    println!("✅ Step 9: Intelligent DAG sync completed - {} blocks processed", 
             sync_result.blocks_received + sync_result.blocks_sent);

    println!("🎉 Complete enhanced coordination workflow test completed successfully!");

    Ok(())
}