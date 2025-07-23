//! Integration tests for federation DAG sync and conflict resolution
//!
//! Tests the complete federation sync protocol including conflict detection,
//! resolution, and network synchronization scenarios.

use icn_dag::{
    InMemoryDagStore, 
    conflict_resolution::{ConflictResolver, ConflictResolutionConfig, ResolutionStrategy},
    federation_sync::{FederationSync, FederationSyncConfig, SyncMessage},
};
use icn_common::{DagBlock, DagLink, Did, Cid, compute_merkle_cid};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_block(id: &str, links: Vec<DagLink>) -> DagBlock {
    let data = format!("data for {}", id).into_bytes();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let author = Did::new("key", "tester");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &links, timestamp, &author, &sig, &None);
    
    DagBlock {
        cid,
        data,
        links,
        timestamp,
        author_did: author,
        signature: sig,
        scope: None,
    }
}

#[test]
fn test_federation_sync_complete_workflow() {
    // Setup two nodes with different DAG states
    let node_a_id = Did::new("key", "node_a");
    let node_b_id = Did::new("key", "node_b");
    
    let store_a = InMemoryDagStore::new();
    let store_b = InMemoryDagStore::new();
    
    let config = FederationSyncConfig::default();
    
    let mut sync_a = FederationSync::new(store_a, node_a_id.clone(), config.clone());
    let mut sync_b = FederationSync::new(store_b, node_b_id.clone(), config);
    
    // Node A initiates sync with Node B
    let sync_request = sync_a.initiate_sync_with_peer(node_b_id.clone()).unwrap();
    
    // Verify the sync request message
    if let SyncMessage::SyncStatusRequest { from_node, .. } = sync_request {
        assert_eq!(from_node, node_a_id);
    } else {
        panic!("Expected SyncStatusRequest");
    }
    
    // Node B handles the sync status request
    let response = sync_b.handle_sync_message(sync_request).unwrap();
    assert!(response.is_some());
    
    // Verify the response
    if let Some(SyncMessage::SyncStatusResponse { from_node, block_count, .. }) = response {
        assert_eq!(from_node, node_b_id);
        assert_eq!(block_count, 0); // Empty store initially
    } else {
        panic!("Expected SyncStatusResponse");
    }
}

#[test]
fn test_conflict_detection_and_resolution() {
    let mut store = InMemoryDagStore::new();
    let config = ConflictResolutionConfig {
        resolution_strategy: ResolutionStrategy::FirstWins,
        ..Default::default()
    };
    let node_id = Did::new("key", "test_node");
    
    // Create a scenario with conflicting blocks (two roots)
    let mut block1 = create_test_block("root1", vec![]);
    let mut block2 = create_test_block("root2", vec![]);
    
    // Make sure they have different timestamps for resolution testing
    block1.timestamp = 1000;
    block2.timestamp = 2000;
    
    // Update CIDs to reflect the different timestamps
    let sig = None;
    block1.cid = compute_merkle_cid(0x71, &block1.data, &[], block1.timestamp, &block1.author_did, &sig, &None);
    block2.cid = compute_merkle_cid(0x71, &block2.data, &[], block2.timestamp, &block2.author_did, &sig, &None);
    
    store.put(&block1).unwrap();
    store.put(&block2).unwrap();
    
    let mut resolver = ConflictResolver::new(store, config, node_id);
    
    // Detect conflicts
    let conflicts = resolver.detect_conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    
    let conflict = &conflicts[0];
    assert_eq!(conflict.conflicting_blocks.len(), 2);
    
    // Resolve the conflict
    let resolution_status = resolver.resolve_conflict(&conflict.conflict_id).unwrap();
    
    // Verify that block1 (earlier timestamp) wins with FirstWins strategy
    if let icn_dag::conflict_resolution::ResolutionStatus::Resolved { winner, .. } = resolution_status {
        assert_eq!(winner, block1.cid);
    } else {
        panic!("Expected resolved status");
    }
}

#[test]
fn test_chain_fork_detection() {
    let mut store = InMemoryDagStore::new();
    let config = ConflictResolutionConfig::default();
    let node_id = Did::new("key", "test_node");
    
    // Create a chain fork scenario
    let ancestor = create_test_block("ancestor", vec![]);
    store.put(&ancestor).unwrap();
    
    // Create two different blocks that both reference the ancestor
    let link = DagLink {
        cid: ancestor.cid.clone(),
        name: "parent".to_string(),
        size: 0,
    };
    
    let fork1 = create_test_block("fork1", vec![link.clone()]);
    let fork2 = create_test_block("fork2", vec![link]);
    
    store.put(&fork1).unwrap();
    store.put(&fork2).unwrap();
    
    let mut resolver = ConflictResolver::new(store, config, node_id);
    
    // Detect the fork conflict
    let conflicts = resolver.detect_conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    
    let conflict = &conflicts[0];
    assert_eq!(conflict.conflict_type, icn_dag::conflict_resolution::ConflictType::ChainFork);
    assert_eq!(conflict.fork_point, Some(ancestor.cid));
    assert_eq!(conflict.conflicting_blocks.len(), 2);
}

#[test]
fn test_block_request_response_cycle() {
    let node_a_id = Did::new("key", "node_a");
    let node_b_id = Did::new("key", "node_b");
    
    let mut store_a = InMemoryDagStore::new();
    let mut store_b = InMemoryDagStore::new();
    
    // Add a block to store B
    let test_block = create_test_block("test_block", vec![]);
    store_b.put(&test_block).unwrap();
    
    let config = FederationSyncConfig::default();
    
    let mut sync_a = FederationSync::new(store_a, node_a_id.clone(), config.clone());
    let mut sync_b = FederationSync::new(store_b, node_b_id.clone(), config);
    
    // Node A requests the block from Node B
    let block_request = SyncMessage::BlockRequest {
        from_node: node_a_id.clone(),
        requested_blocks: vec![test_block.cid.clone()],
        priority: icn_dag::federation_sync::RequestPriority::Normal,
    };
    
    // Node B handles the request
    let response = sync_b.handle_sync_message(block_request).unwrap();
    assert!(response.is_some());
    
    // Verify the response contains the requested block
    if let Some(SyncMessage::BlockResponse { blocks, missing_blocks, .. }) = response {
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].cid, test_block.cid);
        assert_eq!(missing_blocks.len(), 0);
    } else {
        panic!("Expected BlockResponse");
    }
}

#[test]
fn test_delta_sync_request() {
    let node_a_id = Did::new("key", "node_a");
    let node_b_id = Did::new("key", "node_b");
    
    let mut store_a = InMemoryDagStore::new();
    let mut store_b = InMemoryDagStore::new();
    
    // Create a chain of blocks in store B
    let block1 = create_test_block("block1", vec![]);
    let link1 = DagLink {
        cid: block1.cid.clone(),
        name: "prev".to_string(),
        size: 0,
    };
    let block2 = create_test_block("block2", vec![link1]);
    
    store_b.put(&block1).unwrap();
    store_b.put(&block2).unwrap();
    
    let config = FederationSyncConfig::default();
    
    let mut sync_a = FederationSync::new(store_a, node_a_id.clone(), config.clone());
    let mut sync_b = FederationSync::new(store_b, node_b_id.clone(), config);
    
    // Node A requests delta sync from a specific point
    let delta_request = sync_a.request_delta_sync(node_b_id.clone(), Some(block1.cid.clone())).unwrap();
    
    // Verify the request
    if let SyncMessage::DeltaSyncRequest { from_node, since_root, max_blocks, .. } = delta_request {
        assert_eq!(from_node, node_a_id);
        assert_eq!(since_root, Some(block1.cid.clone()));
        assert_eq!(max_blocks, 100); // Default config value
    } else {
        panic!("Expected DeltaSyncRequest");
    }
    
    // Node B handles the delta sync request
    let response = sync_b.handle_sync_message(delta_request).unwrap();
    assert!(response.is_some());
    
    // Verify the response
    if let Some(SyncMessage::DeltaSyncResponse { blocks, .. }) = response {
        // Should return blocks that are not in the chain from block1
        assert!(!blocks.is_empty());
    } else {
        panic!("Expected DeltaSyncResponse");
    }
}

#[test]
fn test_multi_criteria_conflict_resolution() {
    let mut store = InMemoryDagStore::new();
    let config = ConflictResolutionConfig {
        resolution_strategy: ResolutionStrategy::MultiCriteria,
        ..Default::default()
    };
    let node_id = Did::new("key", "test_node");
    
    // Create blocks with different characteristics
    let mut early_block = create_test_block("early", vec![]);
    let mut late_block = create_test_block("late", vec![]);
    
    early_block.timestamp = 1000; // Earlier
    late_block.timestamp = 2000;  // Later
    
    // Update CIDs
    let sig = None;
    early_block.cid = compute_merkle_cid(0x71, &early_block.data, &[], early_block.timestamp, &early_block.author_did, &sig, &None);
    late_block.cid = compute_merkle_cid(0x71, &late_block.data, &[], late_block.timestamp, &late_block.author_did, &sig, &None);
    
    store.put(&early_block).unwrap();
    store.put(&late_block).unwrap();
    
    let mut resolver = ConflictResolver::new(store, config, node_id);
    
    // Detect and resolve conflict
    let conflicts = resolver.detect_conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    
    let resolution_status = resolver.resolve_conflict(&conflicts[0].conflict_id).unwrap();
    
    // Verify resolution occurred (multi-criteria should pick one)
    if let icn_dag::conflict_resolution::ResolutionStatus::Resolved { winner, .. } = resolution_status {
        assert!(winner == early_block.cid || winner == late_block.cid);
    } else {
        panic!("Expected resolved status");
    }
}

#[test]
fn test_sync_stats_tracking() {
    let node_id = Did::new("key", "test_node");
    let store = InMemoryDagStore::new();
    let config = FederationSyncConfig::default();
    
    let sync = FederationSync::new(store, node_id, config);
    
    // Get initial stats
    let stats = sync.get_sync_stats();
    assert_eq!(stats.total_peers, 0);
    assert_eq!(stats.healthy_peers, 0);
    assert_eq!(stats.active_syncs, 0);
}

#[test]
fn test_conflict_resolution_history() {
    let mut store = InMemoryDagStore::new();
    let config = ConflictResolutionConfig::default();
    let node_id = Did::new("key", "test_node");
    
    // Create and resolve a conflict
    let block1 = create_test_block("block1", vec![]);
    let block2 = create_test_block("block2", vec![]);
    
    store.put(&block1).unwrap();
    store.put(&block2).unwrap();
    
    let mut resolver = ConflictResolver::new(store, config, node_id);
    
    // Initially no history
    assert_eq!(resolver.get_resolution_history().len(), 0);
    
    // Detect and resolve conflicts
    let conflicts = resolver.detect_conflicts().unwrap();
    if !conflicts.is_empty() {
        let _ = resolver.resolve_conflict(&conflicts[0].conflict_id);
        
        // Check that history was updated
        assert_eq!(resolver.get_resolution_history().len(), 1);
        assert_eq!(resolver.get_active_conflicts().len(), 0);
    }
}