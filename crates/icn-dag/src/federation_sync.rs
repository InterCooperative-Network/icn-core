//! Federation Sync Protocol Implementation
//!
//! This module implements the network protocol for synchronizing DAG state
//! across federation nodes, including efficient delta sync and conflict resolution.

use crate::{StorageService, conflict_resolution::ConflictResolver};
use icn_common::{Cid, CommonError, DagBlock, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Network message types for federation sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request sync status from a peer
    SyncStatusRequest {
        from_node: Did,
        last_known_root: Option<Cid>,
    },
    /// Response with current sync status
    SyncStatusResponse {
        from_node: Did,
        current_root: Option<Cid>,
        block_count: usize,
        last_update: u64,
        available_blocks: Vec<Cid>,
    },
    /// Request specific blocks from a peer
    BlockRequest {
        from_node: Did,
        requested_blocks: Vec<Cid>,
        priority: RequestPriority,
    },
    /// Response with requested blocks
    BlockResponse {
        from_node: Did,
        blocks: Vec<DagBlock>,
        missing_blocks: Vec<Cid>,
    },
    /// Announce new blocks to peers
    BlockAnnouncement {
        from_node: Did,
        new_blocks: Vec<Cid>,
        priority: AnnouncementPriority,
    },
    /// Request delta sync from a specific point
    DeltaSyncRequest {
        from_node: Did,
        since_root: Option<Cid>,
        since_timestamp: Option<u64>,
        max_blocks: usize,
    },
    /// Response with delta changes
    DeltaSyncResponse {
        from_node: Did,
        blocks: Vec<DagBlock>,
        new_root: Option<Cid>,
        more_available: bool,
    },
    /// Report a detected conflict to peers
    ConflictReport {
        from_node: Did,
        conflict_id: String,
        conflicting_blocks: Vec<Cid>,
        evidence: Vec<u8>, // Serialized conflict evidence
    },
    /// Resolution decision for a conflict
    ConflictResolution {
        from_node: Did,
        conflict_id: String,
        winner: Cid,
        supporting_nodes: Vec<Did>,
    },
}

/// Priority levels for block requests
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Priority levels for block announcements
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnouncementPriority {
    Background,
    Normal,
    Important,
    Urgent,
}

/// Configuration for federation sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSyncConfig {
    /// Maximum blocks to request in a single message
    pub max_blocks_per_request: usize,
    /// Timeout for sync operations (seconds)
    pub sync_timeout: u64,
    /// Interval between periodic sync checks (seconds)
    pub sync_interval: u64,
    /// Maximum number of concurrent sync operations
    pub max_concurrent_syncs: usize,
    /// Enable delta sync optimization
    pub enable_delta_sync: bool,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
}

impl Default for FederationSyncConfig {
    fn default() -> Self {
        Self {
            max_blocks_per_request: 100,
            sync_timeout: 60,
            sync_interval: 30,
            max_concurrent_syncs: 5,
            enable_delta_sync: true,
            enable_conflict_resolution: true,
        }
    }
}

/// State tracking for sync operations with peers
#[derive(Debug, Clone)]
pub struct PeerSyncState {
    /// Peer node identity
    pub peer_id: Did,
    /// Last known root hash from this peer
    pub last_known_root: Option<Cid>,
    /// Last successful sync timestamp
    pub last_sync: u64,
    /// Current sync operation status
    pub sync_status: SyncOperationStatus,
    /// Blocks we know this peer has
    pub known_blocks: HashSet<Cid>,
    /// Blocks we've requested from this peer
    pub requested_blocks: HashSet<Cid>,
    /// Failed sync attempts counter
    pub failed_attempts: u32,
}

/// Status of ongoing sync operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOperationStatus {
    /// No sync operation in progress
    Idle,
    /// Requesting sync status from peer
    RequestingStatus,
    /// Requesting specific blocks
    RequestingBlocks { count: usize },
    /// Performing delta sync
    DeltaSync { since: Option<Cid> },
    /// Resolving conflicts
    ResolvingConflicts { conflict_ids: Vec<String> },
    /// Sync operation failed
    Failed { reason: String },
}

/// Main federation sync protocol handler
pub struct FederationSync<S: StorageService<DagBlock>> {
    /// Local DAG storage
    store: S,
    /// Node identity
    node_id: Did,
    /// Sync configuration
    config: FederationSyncConfig,
    /// State tracking for each peer
    peer_states: HashMap<Did, PeerSyncState>,
    /// Active sync operations
    active_syncs: HashMap<String, SyncOperation>,
    /// Conflict resolver
    conflict_resolver: Option<ConflictResolver<S>>,
}

/// Represents an active sync operation
#[derive(Debug, Clone)]
pub struct SyncOperation {
    /// Unique operation ID
    pub operation_id: String,
    /// Target peer
    pub peer_id: Did,
    /// Operation type
    pub operation_type: SyncOperationType,
    /// Start timestamp
    pub started_at: u64,
    /// Blocks involved in this operation
    pub blocks: HashSet<Cid>,
    /// Current status
    pub status: SyncOperationStatus,
}

/// Types of sync operations
#[derive(Debug, Clone)]
pub enum SyncOperationType {
    /// Full sync with peer
    FullSync,
    /// Delta sync from specific point
    DeltaSync { since: Option<Cid> },
    /// Request specific blocks
    BlockRequest { blocks: Vec<Cid> },
    /// Conflict resolution
    ConflictResolution { conflict_id: String },
}

impl<S: StorageService<DagBlock>> FederationSync<S> {
    /// Create a new federation sync handler
    pub fn new(store: S, node_id: Did, config: FederationSyncConfig) -> Self {
        Self {
            store,
            node_id,
            config,
            peer_states: HashMap::new(),
            active_syncs: HashMap::new(),
            conflict_resolver: None,
        }
    }

    /// Enable conflict resolution functionality
    pub fn enable_conflict_resolution(&mut self, resolver: ConflictResolver<S>) {
        self.conflict_resolver = Some(resolver);
    }

    /// Handle incoming sync message from a peer
    pub fn handle_sync_message(&mut self, message: SyncMessage) -> Result<Option<SyncMessage>, CommonError> {
        match message {
            SyncMessage::SyncStatusRequest { from_node, last_known_root } => {
                self.handle_status_request(from_node, last_known_root)
            }
            SyncMessage::BlockRequest { from_node, requested_blocks, priority } => {
                self.handle_block_request(from_node, requested_blocks, priority)
            }
            SyncMessage::BlockResponse { from_node, blocks, missing_blocks } => {
                self.handle_block_response(from_node, blocks, missing_blocks)
            }
            SyncMessage::DeltaSyncRequest { from_node, since_root, since_timestamp, max_blocks } => {
                self.handle_delta_sync_request(from_node, since_root, since_timestamp, max_blocks)
            }
            SyncMessage::BlockAnnouncement { from_node, new_blocks, priority } => {
                self.handle_block_announcement(from_node, new_blocks, priority)
            }
            SyncMessage::ConflictReport { from_node, conflict_id, conflicting_blocks, evidence } => {
                self.handle_conflict_report(from_node, conflict_id, conflicting_blocks, evidence)
            }
            SyncMessage::ConflictResolution { from_node, conflict_id, winner, supporting_nodes } => {
                self.handle_conflict_resolution(from_node, conflict_id, winner, supporting_nodes)
            }
            _ => Ok(None), // Other message types handled elsewhere
        }
    }

    /// Initialize sync with a new peer
    pub fn initiate_sync_with_peer(&mut self, peer_id: Did) -> Result<SyncMessage, CommonError> {
        let current_root = self.get_current_root()?;
        
        // Initialize peer state if not exists
        if !self.peer_states.contains_key(&peer_id) {
            self.peer_states.insert(peer_id.clone(), PeerSyncState {
                peer_id: peer_id.clone(),
                last_known_root: None,
                last_sync: 0,
                sync_status: SyncOperationStatus::Idle,
                known_blocks: HashSet::new(),
                requested_blocks: HashSet::new(),
                failed_attempts: 0,
            });
        }

        // Update peer state
        if let Some(peer_state) = self.peer_states.get_mut(&peer_id) {
            peer_state.sync_status = SyncOperationStatus::RequestingStatus;
        }

        Ok(SyncMessage::SyncStatusRequest {
            from_node: self.node_id.clone(),
            last_known_root: current_root,
        })
    }

    /// Perform delta sync with peer from a specific point
    pub fn request_delta_sync(&mut self, peer_id: Did, since_root: Option<Cid>) -> Result<SyncMessage, CommonError> {
        let operation_id = format!("delta_{}_{}", peer_id, 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        
        self.active_syncs.insert(operation_id.clone(), SyncOperation {
            operation_id: operation_id.clone(),
            peer_id: peer_id.clone(),
            operation_type: SyncOperationType::DeltaSync { since: since_root.clone() },
            started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            blocks: HashSet::new(),
            status: SyncOperationStatus::DeltaSync { since: since_root.clone() },
        });

        Ok(SyncMessage::DeltaSyncRequest {
            from_node: self.node_id.clone(),
            since_root,
            since_timestamp: None,
            max_blocks: self.config.max_blocks_per_request,
        })
    }

    /// Handle sync status request from peer
    fn handle_status_request(&mut self, _from_node: Did, _last_known_root: Option<Cid>) -> Result<Option<SyncMessage>, CommonError> {
        let current_root = self.get_current_root()?;
        let blocks = self.store.list_blocks()?;
        let block_count = blocks.len();
        let available_blocks: Vec<Cid> = blocks.iter().map(|b| b.cid.clone()).collect();
        
        let last_update = blocks.iter()
            .map(|b| b.timestamp)
            .max()
            .unwrap_or(0);

        Ok(Some(SyncMessage::SyncStatusResponse {
            from_node: self.node_id.clone(),
            current_root,
            block_count,  
            last_update,
            available_blocks,
        }))
    }

    /// Handle block request from peer
    fn handle_block_request(&mut self, _from_node: Did, requested_blocks: Vec<Cid>, _priority: RequestPriority) -> Result<Option<SyncMessage>, CommonError> {
        let mut blocks = Vec::new();
        let mut missing_blocks = Vec::new();

        for cid in requested_blocks {
            match self.store.get(&cid)? {
                Some(block) => blocks.push(block),
                None => missing_blocks.push(cid),
            }
        }

        Ok(Some(SyncMessage::BlockResponse {
            from_node: self.node_id.clone(),
            blocks,
            missing_blocks,
        }))
    }

    /// Handle block response from peer
    fn handle_block_response(&mut self, from_node: Did, blocks: Vec<DagBlock>, _missing_blocks: Vec<Cid>) -> Result<Option<SyncMessage>, CommonError> {
        // Store received blocks
        for block in blocks {
            // Verify block integrity before storing
            icn_common::verify_block_integrity(&block)?;
            self.store.put(&block)?;
            
            // Update peer state
            if let Some(peer_state) = self.peer_states.get_mut(&from_node) {
                peer_state.known_blocks.insert(block.cid.clone());
                peer_state.requested_blocks.remove(&block.cid);
            }
        }

        // Update sync operation status
        self.update_sync_operation_status(&from_node);

        // Check if we need to detect conflicts
        if self.config.enable_conflict_resolution {
            self.detect_and_report_conflicts()?;
        }

        Ok(None)
    }

    /// Handle delta sync request from peer
    fn handle_delta_sync_request(&mut self, _from_node: Did, since_root: Option<Cid>, _since_timestamp: Option<u64>, max_blocks: usize) -> Result<Option<SyncMessage>, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut delta_blocks = Vec::new();
        
        // If since_root is provided, find blocks newer than that point
        if let Some(since_cid) = since_root {
            // Simple implementation: return blocks not in the chain from since_cid
            let chain_from_since = self.get_chain_from_block(&since_cid)?;
            let chain_cids: HashSet<_> = chain_from_since.iter().collect();
            
            for block in blocks {
                if !chain_cids.contains(&block.cid) && delta_blocks.len() < max_blocks {
                    delta_blocks.push(block);
                }
            }
        } else {
            // Return recent blocks
            let mut sorted_blocks = blocks;
            sorted_blocks.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
            delta_blocks = sorted_blocks.into_iter().take(max_blocks).collect();
        }

        let current_root = self.get_current_root()?;
        let more_available = false; // Simplified for now

        Ok(Some(SyncMessage::DeltaSyncResponse {
            from_node: self.node_id.clone(),
            blocks: delta_blocks,
            new_root: current_root,
            more_available,
        }))
    }

    /// Handle block announcement from peer
    fn handle_block_announcement(&mut self, from_node: Did, new_blocks: Vec<Cid>, priority: AnnouncementPriority) -> Result<Option<SyncMessage>, CommonError> {
        // Update peer state with known blocks
        if let Some(peer_state) = self.peer_states.get_mut(&from_node) {
            for cid in &new_blocks {
                peer_state.known_blocks.insert(cid.clone());
            }
        }

        // Check if we need these blocks
        let mut needed_blocks = Vec::new();
        for cid in new_blocks {
            if !self.store.contains(&cid)? {
                needed_blocks.push(cid);
            }
        }

        if !needed_blocks.is_empty() {
            let request_priority = match priority {
                AnnouncementPriority::Urgent => RequestPriority::Critical,
                AnnouncementPriority::Important => RequestPriority::High,
                AnnouncementPriority::Normal => RequestPriority::Normal,
                AnnouncementPriority::Background => RequestPriority::Low,
            };

            return Ok(Some(SyncMessage::BlockRequest {
                from_node: self.node_id.clone(),
                requested_blocks: needed_blocks,
                priority: request_priority,
            }));
        }

        Ok(None)
    }

    /// Handle conflict report from peer
    fn handle_conflict_report(&mut self, _from_node: Did, conflict_id: String, conflicting_blocks: Vec<Cid>, _evidence: Vec<u8>) -> Result<Option<SyncMessage>, CommonError> {
        if !self.config.enable_conflict_resolution {
            return Ok(None);
        }

        // TODO: Implement conflict analysis and resolution
        // For now, just log the conflict
                println!("Conflict reported: {} with blocks {:?}", conflict_id, conflicting_blocks);

        Ok(None)
    }

    /// Handle conflict resolution from peer
    fn handle_conflict_resolution(&mut self, _from_node: Did, conflict_id: String, winner: Cid, supporting_nodes: Vec<Did>) -> Result<Option<SyncMessage>, CommonError> {
        if !self.config.enable_conflict_resolution {
            return Ok(None);
        }

        println!("Conflict {} resolved: winner {} supported by {} nodes", 
            conflict_id, winner, supporting_nodes.len());

        Ok(None)
    }

    /// Get current DAG root
    fn get_current_root(&self) -> Result<Option<Cid>, CommonError> {
        let blocks = self.store.list_blocks()?;
        if blocks.is_empty() {
            return Ok(None);
        }

        // Find blocks that are not referenced by others (roots)
        let mut referenced = HashSet::new();
        for block in &blocks {
            for link in &block.links {
                referenced.insert(link.cid.clone());
            }
        }

        let roots: Vec<_> = blocks.iter()
            .filter(|b| !referenced.contains(&b.cid))
            .map(|b| (b.cid.clone(), b.timestamp)) // Use timestamp as height proxy
            .collect();

        Ok(crate::choose_canonical_root(roots))
    }

    /// Get chain of blocks starting from a specific block
    fn get_chain_from_block(&self, start_cid: &Cid) -> Result<Vec<Cid>, CommonError> {
        let mut chain = Vec::new();
        let mut current = Some(start_cid.clone());
        let mut visited = HashSet::new();

        while let Some(cid) = current {
            if visited.contains(&cid) {
                break; // Cycle detected
            }
            visited.insert(cid.clone());
            chain.push(cid.clone());

            // Follow first link (main chain)
            current = if let Ok(Some(block)) = self.store.get(&cid) {
                block.links.first().map(|link| link.cid.clone())
            } else {
                None
            };
        }

        Ok(chain)
    }

    /// Update sync operation status based on completed requests
    fn update_sync_operation_status(&mut self, peer_id: &Did) {
        if let Some(peer_state) = self.peer_states.get_mut(peer_id) {
            if peer_state.requested_blocks.is_empty() {
                peer_state.sync_status = SyncOperationStatus::Idle;
                peer_state.last_sync = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                peer_state.failed_attempts = 0;
            }
        }
    }

    /// Detect conflicts and report them to the network
    fn detect_and_report_conflicts(&mut self) -> Result<(), CommonError> {
        if let Some(ref mut resolver) = self.conflict_resolver {
            let conflicts = resolver.detect_conflicts()?;
            
            for conflict in conflicts {
                println!("Detected conflict: {} of type {:?}", 
                    conflict.conflict_id, conflict.conflict_type);
                
                // In a full implementation, we would broadcast conflict reports to peers
                // For now, just try to resolve locally
                if let Ok(status) = resolver.resolve_conflict(&conflict.conflict_id) {
                    println!("Conflict {} resolved with status: {:?}", 
                        conflict.conflict_id, status);
                }
            }
        }
        
        Ok(())
    }

    /// Get sync statistics
    pub fn get_sync_stats(&self) -> SyncStats {
        let total_peers = self.peer_states.len();
        let active_syncs = self.active_syncs.len();
        let healthy_peers = self.peer_states.values()
            .filter(|p| p.failed_attempts < 3)
            .count();

        SyncStats {
            total_peers,
            healthy_peers,
            active_syncs,
            last_sync: self.peer_states.values()
                .map(|p| p.last_sync)
                .max()
                .unwrap_or(0),
        }
    }

    /// Get peer states for monitoring
    pub fn get_peer_states(&self) -> &HashMap<Did, PeerSyncState> {
        &self.peer_states
    }
}

/// Statistics for federation sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// Total number of peers
    pub total_peers: usize,
    /// Number of healthy peers (low failure count)
    pub healthy_peers: usize,
    /// Number of active sync operations
    pub active_syncs: usize,
    /// Timestamp of last successful sync
    pub last_sync: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryDagStore;
    use icn_common::{DagLink, compute_merkle_cid};

    fn create_test_block(id: &str) -> DagBlock {
        let data = format!("data for {}", id).into_bytes();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(0x71, &data, &[], timestamp, &author, &sig, &None);
        
        DagBlock {
            cid,
            data,
            links: vec![],
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
        }
    }

    #[test]
    fn test_federation_sync_creation() {
        let store = InMemoryDagStore::new();
        let node_id = Did::new("key", "test_node");
        let config = FederationSyncConfig::default();
        
        let sync = FederationSync::new(store, node_id, config);
        assert_eq!(sync.peer_states.len(), 0);
        assert_eq!(sync.active_syncs.len(), 0);
    }

    #[test]
    fn test_sync_status_request_handling() {
        let mut store = InMemoryDagStore::new();
        let node_id = Did::new("key", "test_node");
        let config = FederationSyncConfig::default();
        
        // Add a test block
        let block = create_test_block("test");
        store.put(&block).unwrap();
        
        let mut sync = FederationSync::new(store, node_id, config);
        let from_node = Did::new("key", "peer_node");
        
        let response = sync.handle_status_request(from_node, None).unwrap();
        assert!(response.is_some());
        
        if let Some(SyncMessage::SyncStatusResponse { block_count, .. }) = response {
            assert_eq!(block_count, 1);
        } else {
            panic!("Expected SyncStatusResponse");
        }
    }

    #[test]
    fn test_block_request_handling() {
        let mut store = InMemoryDagStore::new();
        let node_id = Did::new("key", "test_node");
        let config = FederationSyncConfig::default();
        
        // Add a test block
        let block = create_test_block("test");
        store.put(&block).unwrap();
        
        let mut sync = FederationSync::new(store, node_id, config);
        let from_node = Did::new("key", "peer_node");
        
        let response = sync.handle_block_request(
            from_node, 
            vec![block.cid.clone()], 
            RequestPriority::Normal
        ).unwrap();
        
        assert!(response.is_some());
        if let Some(SyncMessage::BlockResponse { blocks, missing_blocks, .. }) = response {
            assert_eq!(blocks.len(), 1);
            assert_eq!(missing_blocks.len(), 0);
            assert_eq!(blocks[0].cid, block.cid);
        } else {
            panic!("Expected BlockResponse");
        }
    }

    #[test]
    fn test_initiate_sync_with_peer() {
        let store = InMemoryDagStore::new();
        let node_id = Did::new("key", "test_node");
        let config = FederationSyncConfig::default();
        
        let mut sync = FederationSync::new(store, node_id.clone(), config);
        let peer_id = Did::new("key", "peer_node");
        
        let message = sync.initiate_sync_with_peer(peer_id.clone()).unwrap();
        
        if let SyncMessage::SyncStatusRequest { from_node, .. } = message {
            assert_eq!(from_node, node_id);
        } else {
            panic!("Expected SyncStatusRequest");
        }
        
        // Check that peer state was created
        assert!(sync.peer_states.contains_key(&peer_id));
    }
}