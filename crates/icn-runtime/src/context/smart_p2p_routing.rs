//! Smart P2P Message Routing with Reputation-Based Selection
//!
//! This module implements intelligent message routing that leverages peer reputation,
//! network topology, and adaptive routing strategies for optimal message delivery.

use super::{DagStorageService, DagStoreMutexType, HostAbiError, MeshNetworkServiceType};
use icn_common::{Did, TimeProvider};
use icn_reputation::ReputationStore;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, error, info, warn};

/// Smart P2P routing coordinator with reputation-based peer selection
pub struct SmartP2pRouter {
    /// Network service for P2P communication
    network_service: Arc<MeshNetworkServiceType>,
    /// Reputation store for peer scoring
    reputation_store: Arc<dyn ReputationStore>,
    /// Current node identity
    node_identity: Did,
    /// Time provider for timestamps
    time_provider: Arc<dyn TimeProvider>,
    /// Routing table with peer information
    routing_table: Arc<RwLock<RoutingTable>>,
    /// Message queue for intelligent buffering
    message_queue: Arc<Mutex<MessageQueue>>,
    /// Routing strategies and performance tracking
    strategy_manager: Arc<Mutex<RoutingStrategyManager>>,
    /// Performance metrics collector
    metrics: Arc<Mutex<RoutingMetrics>>,
}

/// Routing table containing peer information and network topology
#[derive(Debug, Clone)]
pub struct RoutingTable {
    /// Direct peer connections with quality metrics
    pub direct_peers: HashMap<Did, PeerRouteInfo>,
    /// Multi-hop routes through the network
    pub multi_hop_routes: HashMap<Did, Vec<RoutePath>>,
    /// Network topology clusters for efficient routing
    pub topology_clusters: Vec<TopologyCluster>,
    /// Last topology update timestamp
    pub last_topology_update: Instant,
}

/// Information about routing to a specific peer
#[derive(Debug, Clone)]
pub struct PeerRouteInfo {
    /// Target peer identity
    pub peer_id: Did,
    /// Direct connection quality
    pub direct_quality: Option<ConnectionQuality>,
    /// Available routing paths
    pub routing_paths: Vec<RoutePath>,
    /// Peer reputation score
    pub reputation_score: u64,
    /// Last successful communication
    pub last_success: Instant,
    /// Communication failure count
    pub failure_count: u32,
    /// Estimated message delivery time
    pub estimated_delivery_ms: u64,
}

/// A routing path through the network to reach a target peer
#[derive(Debug, Clone)]
pub struct RoutePath {
    /// Sequence of peer IDs forming the path
    pub path_peers: Vec<Did>,
    /// Total path quality score
    pub path_quality: f64,
    /// Estimated latency for this path
    pub estimated_latency_ms: u64,
    /// Path reliability score (0.0 - 1.0)
    pub reliability: f64,
    /// Last time this path was used
    pub last_used: Instant,
    /// Success rate for this path
    pub success_rate: f64,
}

/// Network topology cluster for efficient routing
#[derive(Debug, Clone)]
pub struct TopologyCluster {
    /// Cluster identifier
    pub cluster_id: String,
    /// Peers in this cluster
    pub peers: Vec<Did>,
    /// Representative/gateway peer for the cluster
    pub gateway_peer: Did,
    /// Cluster quality metrics
    pub cluster_metrics: ClusterMetrics,
}

/// Metrics for a network topology cluster
#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    /// Average intra-cluster latency
    pub avg_internal_latency_ms: f64,
    /// Cluster reliability score
    pub reliability: f64,
    /// Bandwidth availability within cluster
    pub bandwidth_availability: f64,
    /// Cluster reputation average
    pub avg_reputation: f64,
}

/// Connection quality metrics between peers
#[derive(Debug, Clone)]
pub struct ConnectionQuality {
    /// Round-trip latency in milliseconds
    pub latency_ms: f64,
    /// Packet loss rate (0.0 - 1.0)
    pub packet_loss_rate: f64,
    /// Connection stability score (0.0 - 1.0)
    pub stability: f64,
    /// Available bandwidth (bytes/second)
    pub bandwidth_bps: Option<u64>,
    /// Last quality measurement timestamp
    pub last_measured: Instant,
}

/// Intelligent message queue for buffering and prioritization
pub struct MessageQueue {
    /// High priority messages (governance, security)
    high_priority: VecDeque<QueuedMessage>,
    /// Normal priority messages (regular operations)
    normal_priority: VecDeque<QueuedMessage>,
    /// Low priority messages (background sync)
    low_priority: VecDeque<QueuedMessage>,
    /// Failed messages awaiting retry
    retry_queue: VecDeque<QueuedMessage>,
    /// Maximum queue sizes for each priority level
    max_sizes: QueueSizeLimits,
}

/// A message in the routing queue
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    /// Unique message identifier
    pub message_id: String,
    /// Target peer for delivery
    pub target_peer: Did,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message priority level
    pub priority: MessagePriority,
    /// Creation timestamp
    pub created_at: Instant,
    /// Number of delivery attempts
    pub attempts: u32,
    /// Maximum attempts before giving up
    pub max_attempts: u32,
    /// Routing strategy to use for this message
    pub routing_strategy: Option<RoutingStrategy>,
    /// Delivery deadline (if any)
    pub deadline: Option<Instant>,
}

/// Message priority levels for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Background/eventual delivery
    Low,
    /// Regular operational messages
    Normal,
    /// Important system messages
    High,
    /// Critical/emergency messages
    Critical,
}

/// Maximum queue sizes for different priority levels
#[derive(Debug, Clone)]
pub struct QueueSizeLimits {
    pub high_priority_max: usize,
    pub normal_priority_max: usize,
    pub low_priority_max: usize,
    pub retry_queue_max: usize,
}

impl Default for QueueSizeLimits {
    fn default() -> Self {
        Self {
            high_priority_max: 1000,
            normal_priority_max: 5000,
            low_priority_max: 2000,
            retry_queue_max: 1000,
        }
    }
}

/// Routing strategy selection and management
pub struct RoutingStrategyManager {
    /// Available routing strategies
    available_strategies: Vec<RoutingStrategy>,
    /// Strategy performance tracking
    strategy_performance: HashMap<String, StrategyPerformance>,
    /// Current network conditions affecting strategy choice
    network_conditions: NetworkConditions,
    /// Adaptive learning parameters
    learning_params: AdaptiveLearningParams,
}

/// Different routing strategies for message delivery
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Direct connection to target peer
    Direct,
    /// Route through highest reputation intermediate peers
    ReputationBased { min_reputation: u64 },
    /// Route through lowest latency path
    LowestLatency,
    /// Route through most reliable path
    MostReliable { min_reliability: f64 },
    /// Redundant routing through multiple paths
    Redundant { path_count: usize },
    /// Adaptive strategy that learns optimal routes
    Adaptive,
    /// Geographic-aware routing for distributed networks
    Geographic,
    /// Load-balanced routing to distribute network usage
    LoadBalanced,
}

/// Performance tracking for routing strategies
#[derive(Debug, Clone)]
pub struct StrategyPerformance {
    /// Total messages routed with this strategy
    pub messages_routed: u64,
    /// Successful deliveries
    pub successful_deliveries: u64,
    /// Average delivery time
    pub avg_delivery_time_ms: f64,
    /// Resource efficiency (lower is better)
    pub resource_efficiency: f64,
    /// Last performance update
    pub last_updated: Instant,
}

/// Current network conditions for strategy selection
#[derive(Debug, Clone)]
pub struct NetworkConditions {
    /// Number of reachable peers
    pub reachable_peers: usize,
    /// Average network latency
    pub avg_latency_ms: f64,
    /// Network congestion level (0.0 - 1.0)
    pub congestion_level: f64,
    /// Partition detection status
    pub partition_detected: bool,
    /// Overall network stability
    pub stability_score: f64,
}

/// Parameters for adaptive learning in routing
#[derive(Debug, Clone)]
pub struct AdaptiveLearningParams {
    /// Learning rate for strategy optimization
    pub learning_rate: f64,
    /// Weight for recent performance vs historical
    pub recency_weight: f64,
    /// Exploration vs exploitation balance
    pub exploration_factor: f64,
    /// Minimum samples before strategy adaptation
    pub min_samples: u32,
}

impl Default for AdaptiveLearningParams {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            recency_weight: 0.3,
            exploration_factor: 0.05,
            min_samples: 10,
        }
    }
}

/// Performance metrics for the routing system
pub struct RoutingMetrics {
    /// Total messages routed
    pub total_messages: u64,
    /// Successful deliveries
    pub successful_deliveries: u64,
    /// Failed deliveries
    pub failed_deliveries: u64,
    /// Average delivery time per priority
    pub avg_delivery_time_by_priority: HashMap<MessagePriority, f64>,
    /// Network utilization metrics
    pub network_utilization: f64,
    /// Per-peer routing statistics
    pub peer_routing_stats: HashMap<Did, PeerRoutingStats>,
}

/// Routing statistics for individual peers
#[derive(Debug, Clone)]
pub struct PeerRoutingStats {
    /// Messages sent to this peer
    pub messages_sent: u64,
    /// Successful deliveries to this peer
    pub successful_deliveries: u64,
    /// Average delivery time to this peer
    pub avg_delivery_time_ms: f64,
    /// Preferred routing strategy for this peer
    pub preferred_strategy: Option<RoutingStrategy>,
}

impl SmartP2pRouter {
    /// Create a new smart P2P router
    pub fn new(
        network_service: Arc<MeshNetworkServiceType>,
        reputation_store: Arc<dyn ReputationStore>,
        node_identity: Did,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            network_service,
            reputation_store,
            node_identity,
            time_provider,
            routing_table: Arc::new(RwLock::new(RoutingTable::new())),
            message_queue: Arc::new(Mutex::new(MessageQueue::new())),
            strategy_manager: Arc::new(Mutex::new(RoutingStrategyManager::new())),
            metrics: Arc::new(Mutex::new(RoutingMetrics::new())),
        }
    }

    /// Start the smart P2P routing service
    pub async fn start(&self) -> Result<(), HostAbiError> {
        info!("Starting smart P2P routing service with reputation-based selection");

        // Start background tasks for routing management
        self.start_message_processing_task().await?;
        self.start_topology_discovery_task().await?;
        self.start_performance_monitoring_task().await?;
        self.start_adaptive_learning_task().await?;

        info!("Smart P2P routing service started successfully");
        Ok(())
    }

    /// Route a message to a target peer using intelligent selection
    pub async fn route_message(
        &self,
        target_peer: Did,
        payload: Vec<u8>,
        priority: MessagePriority,
        deadline: Option<Instant>,
    ) -> Result<String, HostAbiError> {
        let message_id = self.generate_message_id().await;
        
        let message = QueuedMessage {
            message_id: message_id.clone(),
            target_peer: target_peer.clone(),
            payload,
            priority,
            created_at: Instant::now(),
            attempts: 0,
            max_attempts: match priority {
                MessagePriority::Critical => 10,
                MessagePriority::High => 5,
                MessagePriority::Normal => 3,
                MessagePriority::Low => 1,
            },
            routing_strategy: None, // Will be determined dynamically
            deadline,
        };

        // Add to appropriate queue based on priority
        {
            let mut queue = self.message_queue.lock().await;
            queue.enqueue_message(message)?;
        }

        debug!("Queued message {} for peer {} with priority {:?}", 
               message_id, target_peer, priority);
        
        Ok(message_id)
    }

    /// Get the best routing path to a target peer
    pub async fn get_best_route(&self, target_peer: &Did) -> Result<Option<RoutePath>, HostAbiError> {
        let routing_table = self.routing_table.read().await;
        
        if let Some(peer_info) = routing_table.direct_peers.get(target_peer) {
            // Check if direct connection is available and good quality
            if let Some(direct_quality) = &peer_info.direct_quality {
                if direct_quality.latency_ms < 1000.0 && direct_quality.packet_loss_rate < 0.05 {
                    return Ok(Some(RoutePath {
                        path_peers: vec![target_peer.clone()],
                        path_quality: self.calculate_direct_quality_score(direct_quality).await?,
                        estimated_latency_ms: direct_quality.latency_ms as u64,
                        reliability: direct_quality.stability,
                        last_used: Instant::now(),
                        success_rate: 1.0 - direct_quality.packet_loss_rate,
                    }));
                }
            }
            
            // Find best multi-hop route
            let best_route = peer_info.routing_paths.iter()
                .max_by(|a, b| a.path_quality.partial_cmp(&b.path_quality).unwrap_or(std::cmp::Ordering::Equal))
                .cloned();
                
            return Ok(best_route);
        }
        
        // Peer not found in routing table
        Ok(None)
    }

    /// Update peer reputation and adjust routing preferences
    pub async fn update_peer_reputation(&self, peer_id: &Did, new_reputation: u64) -> Result<(), HostAbiError> {
        let mut routing_table = self.routing_table.write().await;
        
        if let Some(peer_info) = routing_table.direct_peers.get_mut(peer_id) {
            let old_reputation = peer_info.reputation_score;
            peer_info.reputation_score = new_reputation;
            
            // Recalculate routing paths if reputation changed significantly
            if (new_reputation as i64 - old_reputation as i64).abs() > 50 {
                self.recalculate_routing_paths_for_peer(peer_id).await?;
            }
            
            debug!("Updated reputation for peer {} from {} to {}", 
                   peer_id, old_reputation, new_reputation);
        }
        
        Ok(())
    }

    /// Discover and update network topology
    pub async fn discover_network_topology(&self) -> Result<(), HostAbiError> {
        info!("Starting network topology discovery");
        
        // Get current peer list from network service
        let connected_peers = self.get_connected_peers().await?;
        
        // Update routing table with discovered peers
        let mut routing_table = self.routing_table.write().await;
        
        for peer_id in connected_peers {
            if peer_id != self.node_identity {
                let reputation = self.reputation_store.get_reputation(&peer_id);
                let quality = self.measure_connection_quality(&peer_id).await?;
                
                routing_table.direct_peers.insert(peer_id.clone(), PeerRouteInfo {
                    peer_id: peer_id.clone(),
                    direct_quality: Some(quality),
                    routing_paths: vec![], // Will be calculated separately
                    reputation_score: reputation,
                    last_success: Instant::now(),
                    failure_count: 0,
                    estimated_delivery_ms: 100, // Default estimate
                });
            }
        }
        
        routing_table.last_topology_update = Instant::now();
        
        // Discover multi-hop routes and topology clusters
        self.discover_multi_hop_routes(&mut routing_table).await?;
        self.analyze_topology_clusters(&mut routing_table).await?;
        
        info!("Network topology discovery completed: {} direct peers, {} clusters",
              routing_table.direct_peers.len(), routing_table.topology_clusters.len());
        
        Ok(())
    }

    // Background task implementations

    async fn start_message_processing_task(&self) -> Result<(), HostAbiError> {
        let message_queue = self.message_queue.clone();
        let routing_table = self.routing_table.clone();
        let strategy_manager = self.strategy_manager.clone();
        let metrics = self.metrics.clone();
        let network_service = self.network_service.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::process_message_queue(
                    &message_queue,
                    &routing_table,
                    &strategy_manager,
                    &metrics,
                    &network_service,
                ).await {
                    error!("Error processing message queue: {}", e);
                }
            }
        });
        
        Ok(())
    }

    async fn start_topology_discovery_task(&self) -> Result<(), HostAbiError> {
        let router = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = router.discover_network_topology().await {
                    error!("Error in topology discovery: {}", e);
                }
            }
        });
        
        Ok(())
    }

    async fn start_performance_monitoring_task(&self) -> Result<(), HostAbiError> {
        let metrics = self.metrics.clone();
        let routing_table = self.routing_table.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::monitor_routing_performance(&metrics, &routing_table).await {
                    error!("Error monitoring routing performance: {}", e);
                }
            }
        });
        
        Ok(())
    }

    async fn start_adaptive_learning_task(&self) -> Result<(), HostAbiError> {
        let strategy_manager = self.strategy_manager.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::adaptive_strategy_learning(&strategy_manager, &metrics).await {
                    error!("Error in adaptive learning: {}", e);
                }
            }
        });
        
        Ok(())
    }

    // Helper methods for message processing
    
    async fn select_optimal_routing_strategy(
        target_peer: &Did,
        routing_table: &Arc<RwLock<RoutingTable>>,
        strategy_manager: &Arc<Mutex<RoutingStrategyManager>>,
    ) -> Result<RoutingStrategy, HostAbiError> {
        let table = routing_table.read().await;
        let manager = strategy_manager.lock().await;
        
        // Check if we have direct connection with good quality
        if let Some(peer_info) = table.direct_peers.get(target_peer) {
            if let Some(quality) = &peer_info.direct_quality {
                if quality.latency_ms < 500.0 && quality.packet_loss_rate < 0.05 {
                    return Ok(RoutingStrategy::Direct);
                }
            }
            
            // Use reputation-based routing for high-reputation peers
            if peer_info.reputation_score > 500 {
                return Ok(RoutingStrategy::ReputationBased { min_reputation: 300 });
            }
        }
        
        // Use adaptive strategy based on network conditions
        if manager.network_conditions.reachable_peers < 5 {
            Ok(RoutingStrategy::MostReliable { min_reliability: 0.7 })
        } else if manager.network_conditions.congestion_level > 0.8 {
            Ok(RoutingStrategy::LoadBalanced)
        } else {
            Ok(RoutingStrategy::LowestLatency)
        }
    }

    async fn route_message_with_strategy(
        message: &QueuedMessage,
        strategy: &RoutingStrategy,
        routing_table: &Arc<RwLock<RoutingTable>>,
        network_service: &Arc<MeshNetworkServiceType>,
    ) -> Result<(), HostAbiError> {
        match strategy {
            RoutingStrategy::Direct => {
                // Send directly to target peer
                network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                    .map_err(|e| HostAbiError::NetworkError(format!("Direct routing failed: {}", e)))
            }
            RoutingStrategy::ReputationBased { min_reputation } => {
                let table = routing_table.read().await;
                
                // Find best path through high-reputation peers
                if let Some(peer_info) = table.direct_peers.get(&message.target_peer) {
                    let best_path = peer_info.routing_paths.iter()
                        .filter(|path| {
                            // Check if all peers in path meet reputation requirement
                            path.path_peers.iter().all(|peer| {
                                table.direct_peers.get(peer)
                                    .map(|info| info.reputation_score >= *min_reputation)
                                    .unwrap_or(false)
                            })
                        })
                        .max_by(|a, b| a.path_quality.partial_cmp(&b.path_quality).unwrap_or(std::cmp::Ordering::Equal));
                        
                    if let Some(path) = best_path {
                        Self::send_via_path(message, path, network_service).await
                    } else {
                        // Fallback to direct if no reputation-based path available
                        network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                            .map_err(|e| HostAbiError::NetworkError(format!("Reputation-based routing failed: {}", e)))
                    }
                } else {
                    Err(HostAbiError::NetworkError("Target peer not found in routing table".to_string()))
                }
            }
            RoutingStrategy::LowestLatency => {
                let table = routing_table.read().await;
                
                if let Some(peer_info) = table.direct_peers.get(&message.target_peer) {
                    let best_path = peer_info.routing_paths.iter()
                        .min_by(|a, b| a.estimated_latency_ms.cmp(&b.estimated_latency_ms));
                        
                    if let Some(path) = best_path {
                        Self::send_via_path(message, path, network_service).await
                    } else {
                        network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                            .map_err(|e| HostAbiError::NetworkError(format!("Low latency routing failed: {}", e)))
                    }
                } else {
                    Err(HostAbiError::NetworkError("Target peer not found in routing table".to_string()))
                }
            }
            RoutingStrategy::MostReliable { min_reliability } => {
                let table = routing_table.read().await;
                
                if let Some(peer_info) = table.direct_peers.get(&message.target_peer) {
                    let best_path = peer_info.routing_paths.iter()
                        .filter(|path| path.reliability >= *min_reliability)
                        .max_by(|a, b| a.reliability.partial_cmp(&b.reliability).unwrap_or(std::cmp::Ordering::Equal));
                        
                    if let Some(path) = best_path {
                        Self::send_via_path(message, path, network_service).await
                    } else {
                        network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                            .map_err(|e| HostAbiError::NetworkError(format!("Reliable routing failed: {}", e)))
                    }
                } else {
                    Err(HostAbiError::NetworkError("Target peer not found in routing table".to_string()))
                }
            }
            RoutingStrategy::Redundant { path_count } => {
                let table = routing_table.read().await;
                
                if let Some(peer_info) = table.direct_peers.get(&message.target_peer) {
                    let paths: Vec<_> = peer_info.routing_paths.iter()
                        .take(*path_count)
                        .collect();
                    
                    if paths.is_empty() {
                        // Fallback to direct
                        network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                            .map_err(|e| HostAbiError::NetworkError(format!("Redundant routing failed: {}", e)))
                    } else {
                        // Send via multiple paths (fire and forget for redundancy)
                        let mut success = false;
                        for path in paths {
                            if Self::send_via_path(message, path, network_service).await.is_ok() {
                                success = true;
                            }
                        }
                        
                        if success {
                            Ok(())
                        } else {
                            Err(HostAbiError::NetworkError("All redundant paths failed".to_string()))
                        }
                    }
                } else {
                    Err(HostAbiError::NetworkError("Target peer not found in routing table".to_string()))
                }
            }
            _ => {
                // For other strategies, fallback to direct routing for now
                network_service.send_direct_message(message.target_peer.clone(), message.payload.clone()).await
                    .map_err(|e| HostAbiError::NetworkError(format!("Strategy {:?} routing failed: {}", strategy, e)))
            }
        }
    }

    async fn send_via_path(
        message: &QueuedMessage,
        path: &RoutePath,
        network_service: &Arc<MeshNetworkServiceType>,
    ) -> Result<(), HostAbiError> {
        if path.path_peers.is_empty() {
            return Err(HostAbiError::NetworkError("Empty routing path".to_string()));
        }
        
        if path.path_peers.len() == 1 {
            // Direct path
            network_service.send_direct_message(path.path_peers[0].clone(), message.payload.clone()).await
                .map_err(|e| HostAbiError::NetworkError(format!("Path routing failed: {}", e)))
        } else {
            // Multi-hop path
            network_service.send_multi_hop_message(path.path_peers.clone(), message.payload.clone()).await
                .map_err(|e| HostAbiError::NetworkError(format!("Multi-hop routing failed: {}", e)))
        }
    }

    async fn update_metrics_for_successful_message(
        metrics: &Arc<Mutex<RoutingMetrics>>,
        message: &QueuedMessage,
        processing_time: Duration,
    ) -> Result<(), HostAbiError> {
        let mut metrics_guard = metrics.lock().await;
        
        metrics_guard.total_messages += 1;
        metrics_guard.successful_deliveries += 1;
        
        // Update priority-specific metrics
        let avg_time = metrics_guard.avg_delivery_time_by_priority
            .entry(message.priority)
            .or_insert(0.0);
        *avg_time = (*avg_time + processing_time.as_millis() as f64) / 2.0;
        
        // Update per-peer statistics
        let peer_stats = metrics_guard.peer_routing_stats
            .entry(message.target_peer.clone())
            .or_insert(PeerRoutingStats {
                messages_sent: 0,
                successful_deliveries: 0,
                avg_delivery_time_ms: 0.0,
                preferred_strategy: None,
            });
            
        peer_stats.messages_sent += 1;
        peer_stats.successful_deliveries += 1;
        peer_stats.avg_delivery_time_ms = 
            (peer_stats.avg_delivery_time_ms + processing_time.as_millis() as f64) / 2.0;
        
        Ok(())
    }

    async fn update_metrics_for_failed_message(
        metrics: &Arc<Mutex<RoutingMetrics>>,
        message: &QueuedMessage,
        failure_reason: &str,
    ) -> Result<(), HostAbiError> {
        let mut metrics_guard = metrics.lock().await;
        
        metrics_guard.total_messages += 1;
        metrics_guard.failed_deliveries += 1;
        
        // Update per-peer statistics
        let peer_stats = metrics_guard.peer_routing_stats
            .entry(message.target_peer.clone())
            .or_insert(PeerRoutingStats {
                messages_sent: 0,
                successful_deliveries: 0,
                avg_delivery_time_ms: 0.0,
                preferred_strategy: None,
            });
            
        peer_stats.messages_sent += 1;
        
        debug!("Message {} failed for peer {}: {}", message.message_id, message.target_peer, failure_reason);
        Ok(())
    }

    async fn update_strategy_performance(
        strategy_manager: &Arc<Mutex<RoutingStrategyManager>>,
        strategy: &RoutingStrategy,
        success: bool,
        processing_time: Duration,
    ) -> Result<(), HostAbiError> {
        let mut manager = strategy_manager.lock().await;
        let strategy_name = format!("{:?}", strategy);
        
        let performance = manager.strategy_performance
            .entry(strategy_name)
            .or_insert(StrategyPerformance {
                messages_routed: 0,
                successful_deliveries: 0,
                avg_delivery_time_ms: 0.0,
                resource_efficiency: 1.0,
                last_updated: Instant::now(),
            });
            
        performance.messages_routed += 1;
        if success {
            performance.successful_deliveries += 1;
        }
        
        let delivery_time_ms = processing_time.as_millis() as f64;
        performance.avg_delivery_time_ms = 
            (performance.avg_delivery_time_ms * (performance.messages_routed - 1) as f64 + delivery_time_ms) 
            / performance.messages_routed as f64;
            
        performance.last_updated = Instant::now();
        
        Ok(())
    }

    // Helper methods (implementation stubs for now)

    async fn generate_message_id(&self) -> String {
        format!("msg_{}_{}", 
                self.time_provider.unix_seconds(),
                fastrand::u32(..))
    }

    async fn get_connected_peers(&self) -> Result<Vec<Did>, HostAbiError> {
        // Query the network service for currently connected peers
        match self.network_service.get_connected_peers().await {
            Ok(peer_ids) => {
                debug!("Retrieved {} connected peers from network service", peer_ids.len());
                Ok(peer_ids)
            }
            Err(e) => {
                error!("Failed to get connected peers: {}", e);
                Err(HostAbiError::NetworkError(format!("Failed to get connected peers: {}", e)))
            }
        }
    }

    async fn measure_connection_quality(&self, peer_id: &Did) -> Result<ConnectionQuality, HostAbiError> {
        // Measure actual connection quality through ping and statistics
        let start_time = Instant::now();
        
        // Perform connection quality test
        match self.network_service.ping_peer(peer_id.clone()).await {
            Ok(ping_result) => {
                let latency_ms = ping_result.round_trip_time.as_millis() as f64;
                
                // Get historical performance data
                let historical_stats = self.network_service.get_peer_statistics(peer_id.clone()).await
                    .unwrap_or_default();
                
                // Calculate stability based on recent connection history
                let stability = if historical_stats.total_messages > 0 {
                    historical_stats.successful_messages as f64 / historical_stats.total_messages as f64
                } else {
                    1.0 // Assume good until proven otherwise
                };
                
                // Calculate packet loss rate from historical data
                let packet_loss_rate = if historical_stats.total_messages > 0 {
                    1.0 - stability
                } else {
                    0.0
                };
                
                let quality = ConnectionQuality {
                    latency_ms,
                    packet_loss_rate,
                    stability,
                    bandwidth_bps: historical_stats.estimated_bandwidth,
                    last_measured: Instant::now(),
                };
                
                debug!("Measured connection quality for peer {}: {:.2}ms latency, {:.2}% loss, {:.2} stability",
                       peer_id, latency_ms, packet_loss_rate * 100.0, stability);
                
                Ok(quality)
            }
            Err(e) => {
                warn!("Failed to measure connection quality for peer {}: {}", peer_id, e);
                
                // Return degraded quality for unreachable peers
                Ok(ConnectionQuality {
                    latency_ms: 5000.0, // High latency indicates poor connection
                    packet_loss_rate: 1.0, // 100% loss for unreachable peers
                    stability: 0.0,
                    bandwidth_bps: None,
                    last_measured: Instant::now(),
                })
            }
        }
    }

    async fn calculate_direct_quality_score(&self, quality: &ConnectionQuality) -> Result<f64, HostAbiError> {
        // Calculate a composite quality score
        let latency_score = 1.0 - (quality.latency_ms / 5000.0).min(1.0);
        let loss_score = 1.0 - quality.packet_loss_rate;
        let stability_score = quality.stability;
        
        Ok((latency_score + loss_score + stability_score) / 3.0)
    }

    async fn recalculate_routing_paths_for_peer(&self, _peer_id: &Did) -> Result<(), HostAbiError> {
        // Implementation would recalculate optimal routing paths
        Ok(())
    }

    async fn discover_multi_hop_routes(&self, routing_table: &mut RoutingTable) -> Result<(), HostAbiError> {
        // Discover multi-hop routing paths using network topology
        info!("Discovering multi-hop routes through network topology");
        
        let direct_peers: Vec<Did> = routing_table.direct_peers.keys().cloned().collect();
        
        // For each direct peer, discover their connections (2-hop routes)
        for peer_id in &direct_peers {
            match self.network_service.query_peer_connections(peer_id.clone()).await {
                Ok(peer_connections) => {
                    // Create 2-hop routes through this peer
                    for target_peer in peer_connections {
                        if target_peer != self.node_identity && !direct_peers.contains(&target_peer) {
                            let route = RoutePath {
                                path_peers: vec![peer_id.clone(), target_peer.clone()],
                                path_quality: self.calculate_multi_hop_quality(peer_id, &target_peer).await?,
                                estimated_latency_ms: self.estimate_multi_hop_latency(peer_id, &target_peer).await?,
                                reliability: self.calculate_multi_hop_reliability(peer_id, &target_peer).await?,
                                last_used: Instant::now(),
                                success_rate: 1.0, // Initial optimistic assumption
                            };
                            
                            // Add to routing table
                            routing_table.multi_hop_routes
                                .entry(target_peer.clone())
                                .or_insert_with(Vec::new)
                                .push(route);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to query connections for peer {}: {}", peer_id, e);
                }
            }
        }
        
        // Discover 3-hop routes for better coverage
        let two_hop_peers: Vec<Did> = routing_table.multi_hop_routes.keys().cloned().collect();
        
        for intermediate_peer in &direct_peers {
            for target_peer in &two_hop_peers {
                // Try to find routes through intermediate peer to 2-hop targets
                if let Some(routes_to_target) = routing_table.multi_hop_routes.get(target_peer) {
                    for existing_route in routes_to_target {
                        if existing_route.path_peers.len() == 2 {
                            // Create 3-hop route
                            let mut new_path = vec![intermediate_peer.clone()];
                            new_path.extend_from_slice(&existing_route.path_peers);
                            
                            let route = RoutePath {
                                path_peers: new_path,
                                path_quality: existing_route.path_quality * 0.8, // Degrade quality for longer paths
                                estimated_latency_ms: existing_route.estimated_latency_ms + 100, // Add overhead
                                reliability: existing_route.reliability * 0.9, // Reduce reliability
                                last_used: Instant::now(),
                                success_rate: existing_route.success_rate * 0.9,
                            };
                            
                            routing_table.multi_hop_routes
                                .entry(target_peer.clone())
                                .or_insert_with(Vec::new)
                                .push(route);
                        }
                    }
                }
            }
        }
        
        // Sort routes by quality for each target
        for routes in routing_table.multi_hop_routes.values_mut() {
            routes.sort_by(|a, b| b.path_quality.partial_cmp(&a.path_quality).unwrap_or(std::cmp::Ordering::Equal));
            
            // Keep only top 5 routes per target to avoid excessive memory usage
            routes.truncate(5);
        }
        
        info!("Discovered multi-hop routes to {} targets", routing_table.multi_hop_routes.len());
        Ok(())
    }

    async fn calculate_multi_hop_quality(&self, intermediate_peer: &Did, target_peer: &Did) -> Result<f64, HostAbiError> {
        let intermediate_quality = if let Ok(quality) = self.measure_connection_quality(intermediate_peer).await {
            self.calculate_direct_quality_score(&quality).await?
        } else {
            0.5 // Default moderate quality
        };
        
        // Estimate target quality based on reputation
        let target_reputation = self.reputation_store.get_reputation(target_peer);
        let target_quality = (target_reputation as f64 / 1000.0).min(1.0);
        
        // Multi-hop quality is the product of link qualities with degradation
        Ok(intermediate_quality * target_quality * 0.85) // 15% degradation for multi-hop
    }

    async fn estimate_multi_hop_latency(&self, intermediate_peer: &Did, _target_peer: &Did) -> Result<u64, HostAbiError> {
        let intermediate_latency = if let Ok(quality) = self.measure_connection_quality(intermediate_peer).await {
            quality.latency_ms as u64
        } else {
            500 // Default moderate latency
        };
        
        // Estimate additional latency for the second hop
        let estimated_second_hop_latency = 150; // ms
        
        Ok(intermediate_latency + estimated_second_hop_latency)
    }

    async fn calculate_multi_hop_reliability(&self, intermediate_peer: &Did, target_peer: &Did) -> Result<f64, HostAbiError> {
        let intermediate_reliability = if let Ok(quality) = self.measure_connection_quality(intermediate_peer).await {
            quality.stability
        } else {
            0.8 // Default moderate reliability
        };
        
        // Estimate target reliability based on reputation
        let target_reputation = self.reputation_store.get_reputation(target_peer);
        let target_reliability = ((target_reputation as f64 / 1000.0) * 0.5 + 0.5).min(1.0);
        
        // Multi-hop reliability is the product of individual reliabilities
        Ok(intermediate_reliability * target_reliability)
    }

    async fn analyze_topology_clusters(&self, _routing_table: &mut RoutingTable) -> Result<(), HostAbiError> {
        // Implementation would analyze network topology and identify clusters
        Ok(())
    }

    // Static methods for background tasks

    async fn process_message_queue(
        message_queue: &Arc<Mutex<MessageQueue>>,
        routing_table: &Arc<RwLock<RoutingTable>>,
        strategy_manager: &Arc<Mutex<RoutingStrategyManager>>,
        metrics: &Arc<Mutex<RoutingMetrics>>,
        network_service: &Arc<MeshNetworkServiceType>,
    ) -> Result<(), HostAbiError> {
        // Process messages from highest to lowest priority
        let message_to_process = {
            let mut queue = message_queue.lock().await;
            
            // Try high priority first
            if let Some(message) = queue.high_priority.pop_front() {
                Some(message)
            }
            // Then normal priority
            else if let Some(message) = queue.normal_priority.pop_front() {
                Some(message)
            }
            // Then low priority
            else if let Some(message) = queue.low_priority.pop_front() {
                Some(message)
            }
            // Finally retry queue
            else if let Some(message) = queue.retry_queue.pop_front() {
                Some(message)
            } else {
                None
            }
        };

        if let Some(mut message) = message_to_process {
            let process_start = Instant::now();
            
            // Check if message has expired
            if let Some(deadline) = message.deadline {
                if Instant::now() > deadline {
                    warn!("Message {} expired before processing", message.message_id);
                    Self::update_metrics_for_failed_message(&metrics, &message, "expired").await?;
                    return Ok(());
                }
            }

            // Increment attempt counter
            message.attempts += 1;
            
            // Determine routing strategy
            let routing_strategy = if let Some(strategy) = &message.routing_strategy {
                strategy.clone()
            } else {
                Self::select_optimal_routing_strategy(
                    &message.target_peer,
                    routing_table,
                    strategy_manager,
                ).await?
            };

            // Attempt to route the message
            match Self::route_message_with_strategy(
                &message,
                &routing_strategy,
                routing_table,
                network_service,
            ).await {
                Ok(_) => {
                    // Message routed successfully
                    let processing_time = process_start.elapsed();
                    debug!("Successfully routed message {} to {} in {:.2}ms using strategy {:?}",
                           message.message_id, message.target_peer, processing_time.as_millis(), routing_strategy);
                    
                    Self::update_metrics_for_successful_message(&metrics, &message, processing_time).await?;
                    Self::update_strategy_performance(&strategy_manager, &routing_strategy, true, processing_time).await?;
                }
                Err(e) => {
                    // Message routing failed
                    warn!("Failed to route message {} to {} (attempt {}/{}): {}",
                          message.message_id, message.target_peer, message.attempts, message.max_attempts, e);

                    Self::update_strategy_performance(&strategy_manager, &routing_strategy, false, process_start.elapsed()).await?;

                    // Retry if attempts remaining
                    if message.attempts < message.max_attempts {
                        // Add back to retry queue with exponential backoff
                        message.routing_strategy = Some(routing_strategy);
                        
                        // Schedule retry with backoff
                        let backoff_delay = Duration::from_millis(100 * (1 << (message.attempts - 1)).min(32));
                        
                        tokio::spawn(async move {
                            tokio::time::sleep(backoff_delay).await;
                            if let Ok(mut queue) = message_queue.try_lock() {
                                if queue.retry_queue.len() < queue.max_sizes.retry_queue_max {
                                    queue.retry_queue.push_back(message);
                                }
                            }
                        });
                    } else {
                        // Max attempts reached, message failed permanently
                        error!("Message {} permanently failed after {} attempts", message.message_id, message.attempts);
                        Self::update_metrics_for_failed_message(&metrics, &message, "max_attempts").await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn monitor_routing_performance(
        _metrics: &Arc<Mutex<RoutingMetrics>>,
        _routing_table: &Arc<RwLock<RoutingTable>>,
    ) -> Result<(), HostAbiError> {
        // Implementation would monitor and update performance metrics
        Ok(())
    }

    async fn adaptive_strategy_learning(
        _strategy_manager: &Arc<Mutex<RoutingStrategyManager>>,
        _metrics: &Arc<Mutex<RoutingMetrics>>,
    ) -> Result<(), HostAbiError> {
        // Implementation would adapt routing strategies based on performance
        Ok(())
    }
}

// For the clone method required by Arc usage
impl Clone for SmartP2pRouter {
    fn clone(&self) -> Self {
        Self {
            network_service: self.network_service.clone(),
            reputation_store: self.reputation_store.clone(),
            node_identity: self.node_identity.clone(),
            time_provider: self.time_provider.clone(),
            routing_table: self.routing_table.clone(),
            message_queue: self.message_queue.clone(),
            strategy_manager: self.strategy_manager.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

// Implementation of supporting structures

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            direct_peers: HashMap::new(),
            multi_hop_routes: HashMap::new(),
            topology_clusters: Vec::new(),
            last_topology_update: Instant::now(),
        }
    }
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            retry_queue: VecDeque::new(),
            max_sizes: QueueSizeLimits::default(),
        }
    }

    pub fn enqueue_message(&mut self, message: QueuedMessage) -> Result<(), HostAbiError> {
        match message.priority {
            MessagePriority::Critical | MessagePriority::High => {
                if self.high_priority.len() >= self.max_sizes.high_priority_max {
                    return Err(HostAbiError::InternalError("High priority queue full".to_string()));
                }
                self.high_priority.push_back(message);
            }
            MessagePriority::Normal => {
                if self.normal_priority.len() >= self.max_sizes.normal_priority_max {
                    return Err(HostAbiError::InternalError("Normal priority queue full".to_string()));
                }
                self.normal_priority.push_back(message);
            }
            MessagePriority::Low => {
                if self.low_priority.len() >= self.max_sizes.low_priority_max {
                    return Err(HostAbiError::InternalError("Low priority queue full".to_string()));
                }
                self.low_priority.push_back(message);
            }
        }
        Ok(())
    }
}

impl RoutingStrategyManager {
    pub fn new() -> Self {
        Self {
            available_strategies: vec![
                RoutingStrategy::Direct,
                RoutingStrategy::ReputationBased { min_reputation: 100 },
                RoutingStrategy::LowestLatency,
                RoutingStrategy::MostReliable { min_reliability: 0.8 },
                RoutingStrategy::Adaptive,
            ],
            strategy_performance: HashMap::new(),
            network_conditions: NetworkConditions {
                reachable_peers: 0,
                avg_latency_ms: 0.0,
                congestion_level: 0.0,
                partition_detected: false,
                stability_score: 1.0,
            },
            learning_params: AdaptiveLearningParams::default(),
        }
    }
}

impl RoutingMetrics {
    pub fn new() -> Self {
        Self {
            total_messages: 0,
            successful_deliveries: 0,
            failed_deliveries: 0,
            avg_delivery_time_by_priority: HashMap::new(),
            network_utilization: 0.0,
            peer_routing_stats: HashMap::new(),
        }
    }
}