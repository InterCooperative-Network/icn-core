//! CRDT Synchronization and Gossip Protocol implementation.
//!
//! This module provides efficient synchronization of CRDT state across
//! distributed nodes using gossip-based protocols. It handles efficient
//! delta synchronization, conflict resolution, and epidemic-style propagation.

use crate::{CRDTError, CRDTResult, NodeId, OperationMetadata, VectorClock, CRDT};
use icn_common::TimeProvider;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

use log::{debug, error, info, warn};

/// Configuration for the gossip protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// How often to perform gossip rounds (in milliseconds).
    pub gossip_interval_ms: u64,
    /// Number of peers to gossip with in each round.
    pub fanout: usize,
    /// Maximum number of operations to include in a single gossip message.
    pub max_operations_per_message: usize,
    /// How long to keep operations in the buffer for anti-entropy (in seconds).
    pub operation_buffer_ttl_seconds: u64,
    /// Maximum size of the operation buffer.
    pub max_operation_buffer_size: usize,
    /// Whether to use delta compression for efficiency.
    pub enable_delta_compression: bool,
    /// Probability of gossiping with a random peer vs. a known lagging peer.
    pub random_gossip_probability: f64,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            gossip_interval_ms: 1000, // 1 second
            fanout: 3,
            max_operations_per_message: 100,
            operation_buffer_ttl_seconds: 3600, // 1 hour
            max_operation_buffer_size: 10000,
            enable_delta_compression: true,
            random_gossip_probability: 0.3,
        }
    }
}

/// Represents a peer in the gossip network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique identifier for the peer.
    pub node_id: NodeId,
    /// Network address for communication.
    pub address: String,
    /// Last known vector clock of the peer.
    pub last_known_clock: VectorClock,
    /// Timestamp of last successful communication.
    pub last_seen: u64,
    /// Whether the peer is currently reachable.
    pub is_reachable: bool,
}

/// A gossip message containing CRDT operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Sender's node ID.
    pub sender: NodeId,
    /// Sender's current vector clock.
    pub sender_clock: VectorClock,
    /// Operations to synchronize.
    pub operations: Vec<GossipOperation>,
    /// Message sequence number for deduplication.
    pub sequence: u64,
    /// Timestamp when message was created.
    pub timestamp: u64,
}

/// A CRDT operation with its associated metadata for gossiping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipOperation {
    /// Unique identifier for the CRDT instance.
    pub crdt_id: String,
    /// The actual operation data (serialized).
    pub operation_data: Vec<u8>,
    /// Metadata about the operation.
    pub metadata: OperationMetadata,
    /// Type identifier for the CRDT (for deserialization).
    pub crdt_type: String,
}

/// Response to a gossip message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipResponse {
    /// Responder's node ID.
    pub responder: NodeId,
    /// Responder's current vector clock.
    pub responder_clock: VectorClock,
    /// Operations the responder has that the sender might not.
    pub operations: Vec<GossipOperation>,
    /// Acknowledgment of received operations.
    pub received_count: usize,
}

/// Statistics about gossip synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipStats {
    /// Total number of gossip rounds performed.
    pub total_rounds: u64,
    /// Number of successful peer communications.
    pub successful_communications: u64,
    /// Number of failed peer communications.
    pub failed_communications: u64,
    /// Total operations sent.
    pub operations_sent: u64,
    /// Total operations received.
    pub operations_received: u64,
    /// Number of known peers.
    pub known_peers: u64,
    /// Number of reachable peers.
    pub reachable_peers: u64,
    /// Average synchronization latency (in milliseconds).
    pub avg_sync_latency_ms: f64,
}

/// Trait for CRDT synchronization transport layer.
///
/// This allows the gossip protocol to work with different transport
/// mechanisms (TCP, UDP, WebSocket, etc.).
#[async_trait::async_trait]
pub trait GossipTransport: Send + Sync {
    /// Send a gossip message to a peer.
    async fn send_message(
        &self,
        peer: &PeerInfo,
        message: GossipMessage,
    ) -> CRDTResult<GossipResponse>;

    /// Broadcast a message to multiple peers.
    async fn broadcast_message(
        &self,
        peers: &[PeerInfo],
        message: GossipMessage,
    ) -> Vec<CRDTResult<GossipResponse>>;

    /// Check if a peer is reachable.
    async fn ping_peer(&self, peer: &PeerInfo) -> bool;

    /// Discover new peers in the network.
    async fn discover_peers(&self) -> Vec<PeerInfo>;
}

/// Main CRDT synchronization engine.
pub struct CRDTSynchronizer {
    /// Local node identifier.
    node_id: NodeId,
    /// Configuration for gossip protocol.
    config: GossipConfig,
    /// Known peers in the network.
    peers: Arc<RwLock<HashMap<NodeId, PeerInfo>>>,
    /// Buffer of operations for anti-entropy.
    operation_buffer: Arc<Mutex<VecDeque<GossipOperation>>>,
    /// Current vector clock state.
    vector_clock: Arc<RwLock<VectorClock>>,
    /// Transport layer for communication.
    transport: Arc<dyn GossipTransport>,
    /// Statistics tracking.
    stats: Arc<Mutex<GossipStats>>,
    /// Channel for incoming operations.
    operation_sender: mpsc::UnboundedSender<GossipOperation>,
    /// Sequence counter for messages.
    message_sequence: Arc<Mutex<u64>>,
    /// Time provider for deterministic timestamps.
    time_provider: Arc<dyn TimeProvider>,
}

impl CRDTSynchronizer {
    /// Create a new CRDT synchronizer.
    pub fn new(
        node_id: NodeId,
        config: GossipConfig,
        transport: Arc<dyn GossipTransport>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> (Self, mpsc::UnboundedReceiver<GossipOperation>) {
        let (operation_sender, operation_receiver) = mpsc::unbounded_channel();

        let synchronizer = Self {
            node_id,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            operation_buffer: Arc::new(Mutex::new(VecDeque::new())),
            vector_clock: Arc::new(RwLock::new(VectorClock::new())),
            transport,
            stats: Arc::new(Mutex::new(GossipStats::default())),
            operation_sender,
            message_sequence: Arc::new(Mutex::new(0)),
            time_provider,
        };

        (synchronizer, operation_receiver)
    }

    /// Add a peer to the known peers list.
    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.peers.write().await;
        peers.insert(peer.node_id.clone(), peer);
    }

    /// Remove a peer from the known peers list.
    pub async fn remove_peer(&self, node_id: &NodeId) {
        let mut peers = self.peers.write().await;
        peers.remove(node_id);
    }

    /// Get the current list of known peers.
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Add an operation to be gossiped.
    pub async fn add_operation(&self, operation: GossipOperation) -> CRDTResult<()> {
        // Update local vector clock
        {
            let mut clock = self.vector_clock.write().await;
            clock.increment(&self.node_id);
        }

        // Add to operation buffer
        {
            let mut buffer = self.operation_buffer.lock().await;
            buffer.push_back(operation.clone());

            // Limit buffer size
            while buffer.len() > self.config.max_operation_buffer_size {
                buffer.pop_front();
            }
        }

        // Send to local handler
        if self.operation_sender.send(operation).is_err() {
            warn!("Failed to send operation to local handler");
        }

        Ok(())
    }

    /// Start the gossip protocol (runs indefinitely).
    pub async fn start_gossip(&self) -> CRDTResult<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            self.config.gossip_interval_ms,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.perform_gossip_round().await {
                error!("Gossip round failed: {e}");
            }

            // Cleanup old operations
            self.cleanup_operation_buffer().await;

            // Update peer reachability
            self.update_peer_reachability().await;
        }
    }

    /// Perform a single gossip round.
    async fn perform_gossip_round(&self) -> CRDTResult<()> {
        debug!("Starting gossip round");

        let peers = self.select_gossip_peers().await;
        if peers.is_empty() {
            debug!("No peers available for gossip");
            return Ok(());
        }

        let message = self.create_gossip_message().await?;
        let mut stats = self.stats.lock().await;
        stats.total_rounds += 1;
        drop(stats);

        // Send gossip messages
        let responses = self.transport.broadcast_message(&peers, message).await;

        // Process responses
        for (peer, response_result) in peers.iter().zip(responses.iter()) {
            match response_result {
                Ok(response) => {
                    self.handle_gossip_response(peer, response).await?;
                    let mut stats = self.stats.lock().await;
                    stats.successful_communications += 1;
                }
                Err(e) => {
                    warn!("Failed to gossip with peer {}: {e}", peer.node_id);
                    let mut stats = self.stats.lock().await;
                    stats.failed_communications += 1;
                }
            }
        }

        debug!("Completed gossip round with {} peers", peers.len());
        Ok(())
    }

    /// Select peers for this gossip round.
    async fn select_gossip_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        let all_peers: Vec<_> = peers
            .values()
            .filter(|p| p.is_reachable && p.node_id != self.node_id)
            .cloned()
            .collect();

        if all_peers.len() <= self.config.fanout {
            return all_peers;
        }

        let mut selected = Vec::new();
        let our_clock = self.vector_clock.read().await;

        // Select some lagging peers (peers with older vector clocks)
        let lagging_peers: Vec<_> = all_peers
            .iter()
            .filter(|p| our_clock.dominates(&p.last_known_clock))
            .cloned()
            .collect();

        let num_lagging =
            ((1.0 - self.config.random_gossip_probability) * self.config.fanout as f64) as usize;
        let num_random = self.config.fanout - num_lagging.min(lagging_peers.len());

        // Add lagging peers
        for (i, peer) in lagging_peers.iter().enumerate() {
            if i < num_lagging {
                selected.push(peer.clone());
            }
        }

        // Add random peers
        use fastrand;
        for peer in all_peers.iter() {
            if selected.len() >= self.config.fanout {
                break;
            }
            if !selected.iter().any(|p| p.node_id == peer.node_id)
                && fastrand::f64() < (num_random as f64 / (all_peers.len() - selected.len()) as f64)
            {
                selected.push(peer.clone());
            }
        }

        selected
    }

    /// Create a gossip message with recent operations.
    async fn create_gossip_message(&self) -> CRDTResult<GossipMessage> {
        let buffer = self.operation_buffer.lock().await;
        let clock = self.vector_clock.read().await;

        // Take recent operations
        let operations: Vec<_> = buffer
            .iter()
            .rev() // Most recent first
            .take(self.config.max_operations_per_message)
            .cloned()
            .collect();

        let mut sequence = self.message_sequence.lock().await;
        *sequence += 1;
        let seq = *sequence;
        drop(sequence);

        Ok(GossipMessage {
            sender: self.node_id.clone(),
            sender_clock: clock.clone(),
            operations,
            sequence: seq,
            timestamp: current_timestamp_from_provider(&self.time_provider),
        })
    }

    /// Handle a response from a gossip peer.
    async fn handle_gossip_response(
        &self,
        peer: &PeerInfo,
        response: &GossipResponse,
    ) -> CRDTResult<()> {
        // Update peer's known vector clock
        {
            let mut peers = self.peers.write().await;
            if let Some(peer_info) = peers.get_mut(&peer.node_id) {
                peer_info.last_known_clock = response.responder_clock.clone();
                peer_info.last_seen = current_timestamp_from_provider(&self.time_provider);
            }
        }

        // Process operations from peer
        for operation in &response.operations {
            self.operation_sender.send(operation.clone()).map_err(|_| {
                CRDTError::NetworkError("Failed to send operation to handler".to_string())
            })?;
        }

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.operations_received += response.operations.len() as u64;
        }

        Ok(())
    }

    /// Clean up old operations from the buffer.
    async fn cleanup_operation_buffer(&self) {
        let mut buffer = self.operation_buffer.lock().await;
        let current_time = current_timestamp_from_provider(&self.time_provider);
        let ttl_ms = self.config.operation_buffer_ttl_seconds * 1000;

        // Remove operations older than TTL
        while let Some(front) = buffer.front() {
            if current_time - front.metadata.timestamp > ttl_ms {
                buffer.pop_front();
            } else {
                break;
            }
        }
    }

    /// Update reachability status of known peers.
    async fn update_peer_reachability(&self) {
        let peers_to_check: Vec<_> = {
            let peers = self.peers.read().await;
            peers.values().cloned().collect()
        };

        for peer in peers_to_check {
            let is_reachable = self.transport.ping_peer(&peer).await;

            let mut peers = self.peers.write().await;
            if let Some(peer_info) = peers.get_mut(&peer.node_id) {
                peer_info.is_reachable = is_reachable;
                if is_reachable {
                    peer_info.last_seen = current_timestamp_from_provider(&self.time_provider);
                }
            }
        }
    }

    /// Get synchronization statistics.
    pub async fn get_stats(&self) -> GossipStats {
        let stats = self.stats.lock().await;
        let peers = self.peers.read().await;

        let mut current_stats = stats.clone();
        current_stats.known_peers = peers.len() as u64;
        current_stats.reachable_peers = peers.values().filter(|p| p.is_reachable).count() as u64;

        current_stats
    }

    /// Force synchronization with a specific peer.
    pub async fn sync_with_peer(&self, peer_node_id: &NodeId) -> CRDTResult<()> {
        let peer = {
            let peers = self.peers.read().await;
            peers
                .get(peer_node_id)
                .cloned()
                .ok_or_else(|| CRDTError::NodeNotFound(peer_node_id.as_str().to_string()))?
        };

        let message = self.create_gossip_message().await?;
        let response = self.transport.send_message(&peer, message).await?;
        self.handle_gossip_response(&peer, &response).await?;

        Ok(())
    }

    /// Get the current vector clock.
    pub async fn get_vector_clock(&self) -> VectorClock {
        let clock = self.vector_clock.read().await;
        clock.clone()
    }

    /// Force discovery of new peers.
    pub async fn discover_peers(&self) -> CRDTResult<usize> {
        let new_peers = self.transport.discover_peers().await;
        let count = new_peers.len();

        {
            let mut peers = self.peers.write().await;
            for peer in new_peers {
                peers.insert(peer.node_id.clone(), peer);
            }
        }

        info!("Discovered {count} new peers");
        Ok(count)
    }
}

impl Default for GossipStats {
    fn default() -> Self {
        Self {
            total_rounds: 0,
            successful_communications: 0,
            failed_communications: 0,
            operations_sent: 0,
            operations_received: 0,
            known_peers: 0,
            reachable_peers: 0,
            avg_sync_latency_ms: 0.0,
        }
    }
}

/// Get current timestamp in seconds since Unix epoch.
fn current_timestamp_from_provider(time_provider: &Arc<dyn TimeProvider>) -> u64 {
    time_provider.unix_seconds()
}

/// Helper trait for serializing CRDT operations for gossip.
pub trait GossipSerializable: CRDT
where
    Self::Operation: Serialize + for<'a> Deserialize<'a>,
{
    /// Get the type identifier for this CRDT.
    fn crdt_type_name() -> &'static str;

    /// Serialize an operation for gossip.
    fn serialize_operation(op: &Self::Operation) -> CRDTResult<Vec<u8>> {
        bincode::serialize(op).map_err(|e| {
            CRDTError::SerializationError(format!("Failed to serialize operation: {e}"))
        })
    }

    /// Deserialize an operation from gossip.
    fn deserialize_operation(data: &[u8]) -> CRDTResult<Self::Operation> {
        bincode::deserialize(data).map_err(|e| {
            CRDTError::SerializationError(format!("Failed to deserialize operation: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GCounter, LWWRegister};
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock transport for testing
    struct MockTransport {
        sent_messages: Arc<Mutex<Vec<GossipMessage>>>,
        ping_count: Arc<AtomicUsize>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(Vec::new())),
                ping_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl GossipTransport for MockTransport {
        async fn send_message(
            &self,
            _peer: &PeerInfo,
            message: GossipMessage,
        ) -> CRDTResult<GossipResponse> {
            {
                let mut messages = self.sent_messages.lock().await;
                messages.push(message.clone());
            }

            Ok(GossipResponse {
                responder: NodeId::new("mock_responder".to_string()),
                responder_clock: VectorClock::new(),
                operations: Vec::new(),
                received_count: message.operations.len(),
            })
        }

        async fn broadcast_message(
            &self,
            peers: &[PeerInfo],
            message: GossipMessage,
        ) -> Vec<CRDTResult<GossipResponse>> {
            let mut results = Vec::new();
            for peer in peers {
                results.push(self.send_message(peer, message.clone()).await);
            }
            results
        }

        async fn ping_peer(&self, _peer: &PeerInfo) -> bool {
            self.ping_count.fetch_add(1, Ordering::SeqCst);
            true
        }

        async fn discover_peers(&self) -> Vec<PeerInfo> {
            vec![PeerInfo {
                node_id: NodeId::new("discovered_peer".to_string()),
                address: "127.0.0.1:8080".to_string(),
                last_known_clock: VectorClock::new(),
                last_seen: 42, // Fixed timestamp for testing
                is_reachable: true,
            }]
        }
    }

    #[tokio::test]
    async fn test_crdt_synchronizer_creation() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, _receiver) =
            CRDTSynchronizer::new(node_id.clone(), config, transport, time_provider);

        assert_eq!(synchronizer.node_id, node_id);
        assert_eq!(synchronizer.get_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_peers() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, _receiver) =
            CRDTSynchronizer::new(node_id, config, transport, time_provider);

        let peer = PeerInfo {
            node_id: NodeId::new("peer1".to_string()),
            address: "127.0.0.1:8080".to_string(),
            last_known_clock: VectorClock::new(),
            last_seen: 42, // Fixed timestamp for testing
            is_reachable: true,
        };

        synchronizer.add_peer(peer.clone()).await;
        assert_eq!(synchronizer.get_peers().await.len(), 1);

        synchronizer.remove_peer(&peer.node_id).await;
        assert_eq!(synchronizer.get_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_operation() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, mut receiver) =
            CRDTSynchronizer::new(node_id.clone(), config, transport, time_provider);

        let operation = GossipOperation {
            crdt_id: "test_crdt".to_string(),
            operation_data: vec![1, 2, 3],
            metadata: OperationMetadata::new(node_id, VectorClock::new(), 42), // Fixed timestamp for testing
            crdt_type: "test".to_string(),
        };

        synchronizer.add_operation(operation.clone()).await.unwrap();

        // Should receive the operation
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.crdt_id, operation.crdt_id);
        assert_eq!(received.operation_data, operation.operation_data);
    }

    #[tokio::test]
    async fn test_gossip_message_creation() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, _receiver) =
            CRDTSynchronizer::new(node_id.clone(), config, transport, time_provider);

        let operation = GossipOperation {
            crdt_id: "test_crdt".to_string(),
            operation_data: vec![1, 2, 3],
            metadata: OperationMetadata::new(node_id.clone(), VectorClock::new(), 42), // Fixed timestamp for testing
            crdt_type: "test".to_string(),
        };

        synchronizer.add_operation(operation).await.unwrap();

        let message = synchronizer.create_gossip_message().await.unwrap();
        assert_eq!(message.sender, node_id);
        assert_eq!(message.operations.len(), 1);
        assert_eq!(message.operations[0].crdt_id, "test_crdt");
    }

    #[tokio::test]
    async fn test_peer_discovery() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, _receiver) =
            CRDTSynchronizer::new(node_id, config, transport, time_provider);

        let discovered_count = synchronizer.discover_peers().await.unwrap();
        assert_eq!(discovered_count, 1);

        let peers = synchronizer.get_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id.as_str(), "discovered_peer");
    }

    #[tokio::test]
    async fn test_stats() {
        let node_id = NodeId::new("test_node".to_string());
        let config = GossipConfig::default();
        let transport = Arc::new(MockTransport::new());
        let time_provider = Arc::new(icn_common::FixedTimeProvider::new(42));

        let (synchronizer, _receiver) =
            CRDTSynchronizer::new(node_id, config, transport, time_provider);

        let stats = synchronizer.get_stats().await;
        assert_eq!(stats.total_rounds, 0);
        assert_eq!(stats.known_peers, 0);
        assert_eq!(stats.reachable_peers, 0);
    }

    // Implement GossipSerializable for our test CRDTs
    impl GossipSerializable for GCounter {
        fn crdt_type_name() -> &'static str {
            "GCounter"
        }
    }

    impl GossipSerializable for LWWRegister<String> {
        fn crdt_type_name() -> &'static str {
            "LWWRegister<String>"
        }
    }
}
