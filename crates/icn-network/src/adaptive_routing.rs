//! Advanced adaptive routing for ICN network
//!
//! This module provides intelligent routing capabilities that adapt to network
//! conditions, peer performance, and application requirements.

use crate::{NetworkService, MeshNetworkError, PeerId, NetworkStats};
use icn_common::{Did, TimeProvider, CommonError};
use icn_core_traits::ReputationStore;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

// Helper function to get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Configuration for adaptive routing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveRoutingConfig {
    /// Maximum number of routes to maintain per destination
    pub max_routes_per_destination: usize,
    /// Minimum success rate to consider a route healthy
    pub min_success_rate: f64,
    /// Time window for measuring route performance
    pub performance_window: Duration,
    /// Maximum latency threshold for route selection
    pub max_latency_threshold: Duration,
    /// How often to refresh route information
    pub route_refresh_interval: Duration,
    /// Number of alternative routes to try in parallel
    pub parallel_route_attempts: usize,
    /// Minimum reputation score to consider a peer
    pub min_peer_reputation: f64,
    /// Weight factors for route selection
    pub selection_weights: RouteSelectionWeights,
}

impl Default for AdaptiveRoutingConfig {
    fn default() -> Self {
        Self {
            max_routes_per_destination: 5,
            min_success_rate: 0.8,
            performance_window: Duration::from_secs(300), // 5 minutes
            max_latency_threshold: Duration::from_millis(5000),
            route_refresh_interval: Duration::from_secs(60),
            parallel_route_attempts: 3,
            min_peer_reputation: 0.5,
            selection_weights: RouteSelectionWeights::default(),
        }
    }
}

/// Weights for different factors in route selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSelectionWeights {
    /// Weight for latency factor (lower is better)
    pub latency_weight: f64,
    /// Weight for success rate factor (higher is better)
    pub success_rate_weight: f64,
    /// Weight for peer reputation factor (higher is better)
    pub reputation_weight: f64,
    /// Weight for bandwidth factor (higher is better)
    pub bandwidth_weight: f64,
    /// Weight for hop count factor (lower is better)
    pub hop_count_weight: f64,
}

impl Default for RouteSelectionWeights {
    fn default() -> Self {
        Self {
            latency_weight: 0.3,
            success_rate_weight: 0.3,
            reputation_weight: 0.2,
            bandwidth_weight: 0.1,
            hop_count_weight: 0.1,
        }
    }
}

/// Information about a network route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// Destination DID
    pub destination: Did,
    /// Intermediate peers in the route
    pub hops: Vec<PeerId>,
    /// Estimated latency for this route (milliseconds)
    pub latency: u64,
    /// Success rate over the performance window
    pub success_rate: f64,
    /// Last time this route was used (Unix timestamp)
    pub last_used: u64,
    /// Number of times this route has been used
    pub usage_count: u64,
    /// Estimated bandwidth capacity
    pub bandwidth_estimate: u64,
    /// Route discovery timestamp (Unix timestamp)
    pub discovered_at: u64,
    /// Whether this route is currently healthy
    pub is_healthy: bool,
}

impl RouteInfo {
    /// Calculate a composite score for route selection
    pub fn calculate_score(&self, weights: &RouteSelectionWeights, reputation_scores: &HashMap<PeerId, f64>) -> f64 {
        let latency_score = 1.0 - (self.latency as f64 / 10000.0).min(1.0);
        let success_score = self.success_rate;
        let hop_score = 1.0 - (self.hops.len() as f64 / 10.0).min(1.0);
        let bandwidth_score = (self.bandwidth_estimate as f64 / 1_000_000.0).min(1.0); // Normalize to 1Mbps
        
        // Calculate average reputation of peers in the route
        let avg_reputation = if self.hops.is_empty() {
            1.0 // Direct connection
        } else {
            let total: f64 = self.hops.iter()
                .map(|peer| reputation_scores.get(peer).copied().unwrap_or(0.5))
                .sum();
            total / self.hops.len() as f64
        };
        
        weights.latency_weight * latency_score +
        weights.success_rate_weight * success_score +
        weights.reputation_weight * avg_reputation +
        weights.bandwidth_weight * bandwidth_score +
        weights.hop_count_weight * hop_score
    }
    
    /// Update route performance based on a successful transmission
    pub fn record_success(&mut self, latency_ms: u64) {
        self.last_used = current_timestamp();
        self.usage_count += 1;
        
        // Update rolling average latency
        self.latency = ((self.latency as f64 * 0.8) + (latency_ms as f64 * 0.2)) as u64;
        
        // Update success rate (exponential moving average)
        self.success_rate = self.success_rate * 0.9 + 0.1;
        self.is_healthy = self.success_rate >= 0.8;
    }
    
    /// Update route performance based on a failed transmission
    pub fn record_failure(&mut self) {
        self.last_used = current_timestamp();
        self.usage_count += 1;
        
        // Update success rate (exponential moving average)
        self.success_rate = self.success_rate * 0.9;
        self.is_healthy = self.success_rate >= 0.8;
    }
}

/// Performance metrics for route monitoring
#[derive(Debug, Clone, Default)]
pub struct RoutePerformanceMetrics {
    /// Total number of routing decisions made
    pub total_routing_decisions: u64,
    /// Number of successful route selections
    pub successful_routes: u64,
    /// Number of failed route attempts
    pub failed_routes: u64,
    /// Average route discovery time
    pub avg_discovery_time: Duration,
    /// Number of routes currently maintained
    pub active_routes: usize,
    /// Number of destinations with available routes
    pub reachable_destinations: usize,
}

/// Events emitted by the adaptive routing system
#[derive(Debug, Clone)]
pub enum RoutingEvent {
    /// New route discovered
    RouteDiscovered {
        destination: Did,
        route: RouteInfo,
    },
    /// Route became unhealthy
    RouteUnhealthy {
        destination: Did,
        route_id: String,
    },
    /// Route performance improved
    RouteImproved {
        destination: Did,
        new_score: f64,
    },
    /// Network partition detected
    NetworkPartition {
        affected_destinations: Vec<Did>,
    },
}

/// Adaptive routing engine that provides intelligent route selection
pub struct AdaptiveRoutingEngine {
    config: AdaptiveRoutingConfig,
    network_service: Arc<dyn NetworkService>,
    reputation_store: Option<Arc<dyn ReputationStore>>,
    time_provider: Arc<dyn TimeProvider>,
    
    // Route state
    routes: Arc<RwLock<HashMap<Did, Vec<RouteInfo>>>>,
    route_metrics: Arc<RwLock<RoutePerformanceMetrics>>,
    peer_reputation_cache: Arc<RwLock<HashMap<PeerId, f64>>>,
    
    // Event handling
    event_tx: mpsc::UnboundedSender<RoutingEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<RoutingEvent>>,
    
    // Background tasks
    _route_maintenance_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AdaptiveRoutingEngine {
    /// Create a new adaptive routing engine
    pub fn new(
        config: AdaptiveRoutingConfig,
        network_service: Arc<dyn NetworkService>,
        reputation_store: Option<Arc<dyn ReputationStore>>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            network_service,
            reputation_store,
            time_provider,
            routes: Arc::new(RwLock::new(HashMap::new())),
            route_metrics: Arc::new(RwLock::new(RoutePerformanceMetrics::default())),
            peer_reputation_cache: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Some(event_rx),
            _route_maintenance_handle: None,
        }
    }
    
    /// Start the adaptive routing engine
    pub async fn start(&mut self) -> Result<(), MeshNetworkError> {
        // Start route maintenance task
        let routes = self.routes.clone();
        let metrics = self.route_metrics.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.route_refresh_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::maintain_routes(&routes, &metrics, &config, &event_tx).await {
                    log::warn!("Route maintenance error: {}", e);
                }
            }
        });
        
        self._route_maintenance_handle = Some(handle);
        
        // Initial route discovery
        self.discover_initial_routes().await?;
        
        Ok(())
    }
    
    /// Get event receiver for routing events
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<RoutingEvent>> {
        self.event_rx.take()
    }
    
    /// Find the best route to a destination
    pub async fn find_best_route(&self, destination: &Did) -> Result<Option<RouteInfo>, MeshNetworkError> {
        self.update_reputation_cache().await?;
        
        let routes = self.routes.read().unwrap();
        let reputation_cache = self.peer_reputation_cache.read().unwrap();
        
        if let Some(destination_routes) = routes.get(destination) {
            // Filter healthy routes
            let healthy_routes: Vec<&RouteInfo> = destination_routes
                .iter()
                .filter(|route| route.is_healthy)
                .collect();
            
            if healthy_routes.is_empty() {
                return Ok(None);
            }
            
            // Score and select best route
            let mut scored_routes: Vec<(f64, &RouteInfo)> = healthy_routes
                .iter()
                .map(|route| {
                    let score = route.calculate_score(&self.config.selection_weights, &reputation_cache);
                    (score, *route)
                })
                .collect();
            
            scored_routes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            
            if let Some((score, best_route)) = scored_routes.first() {
                log::debug!("Selected route to {} with score {:.3}", destination, score);
                return Ok(Some((*best_route).clone()));
            }
        }
        
        // No existing route found, try to discover one
        self.discover_route_to_destination(destination).await
    }
    
    /// Record the result of using a route
    pub async fn record_route_result(
        &self,
        destination: &Did,
        route_hops: &[PeerId],
        success: bool,
        latency_ms: Option<u64>,
    ) -> Result<(), MeshNetworkError> {
        let mut routes = self.routes.write().unwrap();
        let mut metrics = self.route_metrics.write().unwrap();
        
        metrics.total_routing_decisions += 1;
        
        if let Some(destination_routes) = routes.get_mut(destination) {
            // Find the matching route
            if let Some(route) = destination_routes.iter_mut().find(|r| r.hops == route_hops) {
                if success {
                    metrics.successful_routes += 1;
                    if let Some(lat_ms) = latency_ms {
                        route.record_success(lat_ms);
                    } else {
                        route.record_success(route.latency); // Use previous estimate (already in ms)
                    }
                } else {
                    metrics.failed_routes += 1;
                    route.record_failure();
                    
                    if !route.is_healthy {
                        let _ = self.event_tx.send(RoutingEvent::RouteUnhealthy {
                            destination: destination.clone(),
                            route_id: format!("{:?}", route.hops),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Discover initial routes to known peers
    async fn discover_initial_routes(&self) -> Result<(), MeshNetworkError> {
        log::info!("Starting initial route discovery");
        
        // Get connected peers from network service
        let peers = match self.network_service.discover_peers(None).await {
            Ok(peers) => peers,
            Err(e) => {
                log::warn!("Failed to discover peers for routing: {}", e);
                return Ok(()); // Don't fail completely
            }
        };
        
        for peer in peers {
            // For now, assume direct connection to each peer
            // In a real implementation, this would query peer for their connections
            if let Ok(did) = self.peer_id_to_did(&peer).await {
                self.add_direct_route(&did, &peer).await?;
            }
        }
        
        log::info!("Completed initial route discovery");
        Ok(())
    }
    
    /// Add a direct route to a peer
    async fn add_direct_route(&self, destination: &Did, peer: &PeerId) -> Result<(), MeshNetworkError> {
        let route = RouteInfo {
            destination: destination.clone(),
            hops: vec![], // Direct connection
            latency: 100, // Initial estimate in milliseconds
            success_rate: 1.0, // Start optimistic
            last_used: current_timestamp(),
            usage_count: 0,
            bandwidth_estimate: 1_000_000, // 1 Mbps initial estimate
            discovered_at: current_timestamp(),
            is_healthy: true,
        };
        
        let mut routes = self.routes.write().unwrap();
        routes.entry(destination.clone()).or_insert_with(Vec::new).push(route.clone());
        
        let _ = self.event_tx.send(RoutingEvent::RouteDiscovered {
            destination: destination.clone(),
            route,
        });
        
        Ok(())
    }
    
    /// Discover a route to a specific destination
    async fn discover_route_to_destination(&self, destination: &Did) -> Result<Option<RouteInfo>, MeshNetworkError> {
        log::debug!("Discovering route to {}", destination);
        
        // Try to find the destination through connected peers
        let peers = self.network_service.discover_peers(None).await?;
        
        for peer in &peers {
            // Query peer for routes to destination
            if let Ok(route) = self.query_peer_for_route(peer, destination).await {
                self.add_discovered_route(destination, route.clone()).await?;
                return Ok(Some(route));
            }
        }
        
        Ok(None)
    }
    
    /// Query a peer for routes to a destination
    async fn query_peer_for_route(&self, peer: &PeerId, destination: &Did) -> Result<RouteInfo, MeshNetworkError> {
        // Implement peer route querying by sending a route request message
        use serde_json::json;
        
        let route_request = json!({
            "type": "route_request", 
            "destination": destination.to_string(),
            "timestamp": current_timestamp(),
            "requester": self.network_service.get_local_peer_id(),
        });
        
        let request_bytes = serde_json::to_vec(&route_request).map_err(|e| {
            MeshNetworkError::RoutingError(format!("Failed to serialize route request: {}", e))
        })?;
        
        // Send route request to peer
        self.network_service
            .send_message(peer, "icn_route_request", request_bytes)
            .await
            .map_err(|e| MeshNetworkError::RoutingError(format!("Failed to send route request: {}", e)))?;
        
        // For now, create a basic route info based on the peer
        // In a full implementation, this would wait for a response
        let route_info = RouteInfo {
            destination: destination.clone(),
            next_hop: peer.clone(),
            path: vec![peer.clone()],
            latency: Duration::from_millis(100), // Estimated latency
            success_rate: 0.9, // Default success rate
            last_used: current_timestamp(),
            cost: 1.5, // Slightly higher cost for queried routes
            quality_score: 0.7, // Default quality
            flags: RouteFlags::ACTIVE,
        };
        
        Ok(route_info)
    }
    
    /// Add a newly discovered route
    async fn add_discovered_route(&self, destination: &Did, route: RouteInfo) -> Result<(), MeshNetworkError> {
        let mut routes = self.routes.write().unwrap();
        let destination_routes = routes.entry(destination.clone()).or_insert_with(Vec::new);
        
        // Don't add duplicate routes
        if !destination_routes.iter().any(|r| r.hops == route.hops) {
            destination_routes.push(route.clone());
            
            // Limit number of routes per destination
            if destination_routes.len() > self.config.max_routes_per_destination {
                destination_routes.sort_by_key(|r| r.last_used);
                destination_routes.remove(0);
            }
            
            let _ = self.event_tx.send(RoutingEvent::RouteDiscovered {
                destination: destination.clone(),
                route,
            });
        }
        
        Ok(())
    }
    
    /// Update reputation cache from reputation store
    async fn update_reputation_cache(&self) -> Result<(), MeshNetworkError> {
        if let Some(reputation_store) = &self.reputation_store {
            let mut cache = self.peer_reputation_cache.write().unwrap();
            
            // Get connected peers and their reputations
            let peers = self.network_service.discover_peers(None).await?;
            for peer in peers {
                if let Ok(did) = self.peer_id_to_did(&peer).await {
                    let reputation = reputation_store.get_reputation(&did) as f64 / 100.0; // Normalize
                    cache.insert(peer, reputation);
                }
            }
        }
        
        Ok(())
    }
    
    /// Convert PeerId to DID (simplified)
    async fn peer_id_to_did(&self, peer_id: &PeerId) -> Result<Did, MeshNetworkError> {
        // Implement proper PeerId to DID resolution
        // For now, we'll use a simple mapping where PeerId is part of the DID
        // In a production system, this would involve a proper directory service
        
        // Try parsing as a full DID first
        if let Ok(did) = Did::from_str(peer_id) {
            return Ok(did);
        }
        
        // Create a DID from the peer ID
        let did_string = if peer_id.contains(':') {
            // Assume it's already a properly formatted identifier
            format!("did:icn:{}", peer_id)
        } else {
            // Treat as a simple peer identifier
            format!("did:icn:peer:{}", peer_id)
        };
        
        Did::from_str(&did_string).map_err(|e| {
            MeshNetworkError::RoutingError(format!("Failed to create DID from peer ID {}: {}", peer_id, e))
        })
    }
    
    /// Background route maintenance
    async fn maintain_routes(
        routes: &Arc<RwLock<HashMap<Did, Vec<RouteInfo>>>>,
        metrics: &Arc<RwLock<RoutePerformanceMetrics>>,
        config: &AdaptiveRoutingConfig,
        event_tx: &mpsc::UnboundedSender<RoutingEvent>,
    ) -> Result<(), MeshNetworkError> {
        let now = current_timestamp();
        let mut routes_guard = routes.write().unwrap();
        let mut metrics_guard = metrics.write().unwrap();
        
        let mut total_routes = 0;
        let mut healthy_routes = 0;
        
        // Clean up old and unhealthy routes
        for (destination, destination_routes) in routes_guard.iter_mut() {
            destination_routes.retain(|route| {
                total_routes += 1;
                
                // Remove very old routes
                if now > route.last_used && (now - route.last_used) > config.performance_window.as_secs() * 3 {
                    return false;
                }
                
                // Remove consistently unhealthy routes
                if !route.is_healthy && route.success_rate < config.min_success_rate {
                    let _ = event_tx.send(RoutingEvent::RouteUnhealthy {
                        destination: destination.clone(),
                        route_id: format!("{:?}", route.hops),
                    });
                    return false;
                }
                
                if route.is_healthy {
                    healthy_routes += 1;
                }
                
                true
            });
        }
        
        // Remove destinations with no routes
        routes_guard.retain(|_, routes| !routes.is_empty());
        
        // Update metrics
        metrics_guard.active_routes = total_routes;
        metrics_guard.reachable_destinations = routes_guard.len();
        
        log::debug!(
            "Route maintenance: {} total routes, {} healthy, {} destinations",
            total_routes, healthy_routes, routes_guard.len()
        );
        
        Ok(())
    }
    
    /// Get current routing performance metrics
    pub fn get_performance_metrics(&self) -> RoutePerformanceMetrics {
        self.route_metrics.read().unwrap().clone()
    }
    
    /// Get routes to a specific destination
    pub fn get_routes_to_destination(&self, destination: &Did) -> Vec<RouteInfo> {
        self.routes.read().unwrap()
            .get(destination)
            .map(|routes| routes.clone())
            .unwrap_or_default()
    }
    
    /// Get all known destinations
    pub fn get_known_destinations(&self) -> Vec<Did> {
        self.routes.read().unwrap().keys().cloned().collect()
    }
    
    /// Force route discovery for a destination
    pub async fn force_route_discovery(&self, destination: &Did) -> Result<Vec<RouteInfo>, MeshNetworkError> {
        self.discover_route_to_destination(destination).await?;
        Ok(self.get_routes_to_destination(destination))
    }
    
    /// Get network topology information
    pub async fn get_network_topology(&self) -> Result<NetworkTopology, MeshNetworkError> {
        let routes = self.routes.read().unwrap();
        let metrics = self.route_metrics.read().unwrap();
        
        let mut topology = NetworkTopology {
            total_destinations: routes.len(),
            total_routes: routes.values().map(|v| v.len()).sum(),
            avg_routes_per_destination: if routes.is_empty() { 0.0 } else {
                routes.values().map(|v| v.len()).sum::<usize>() as f64 / routes.len() as f64
            },
            healthy_route_percentage: if metrics.total_routing_decisions > 0 {
                metrics.successful_routes as f64 / metrics.total_routing_decisions as f64
            } else { 0.0 },
            network_connectivity_score: 0.0, // Will calculate below
        };
        
        // Calculate network connectivity score based on route availability and health
        if !routes.is_empty() {
            let total_possible_routes = routes.len() * self.config.max_routes_per_destination;
            let actual_healthy_routes: usize = routes.values()
                .map(|routes| routes.iter().filter(|r| r.is_healthy).count())
                .sum();
            
            topology.network_connectivity_score = 
                actual_healthy_routes as f64 / total_possible_routes as f64;
        }
        
        Ok(topology)
    }
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Total number of known destinations
    pub total_destinations: usize,
    /// Total number of routes maintained
    pub total_routes: usize,
    /// Average number of routes per destination
    pub avg_routes_per_destination: f64,
    /// Percentage of healthy routes
    pub healthy_route_percentage: f64,
    /// Overall network connectivity score (0.0 to 1.0)
    pub network_connectivity_score: f64,
}

/// Integration with the existing network service
pub struct AdaptiveNetworkService {
    inner: Arc<dyn NetworkService>,
    routing_engine: Arc<AdaptiveRoutingEngine>,
}

impl AdaptiveNetworkService {
    /// Create a new adaptive network service
    pub fn new(
        inner: Arc<dyn NetworkService>,
        routing_engine: Arc<AdaptiveRoutingEngine>,
    ) -> Self {
        Self {
            inner,
            routing_engine,
        }
    }
    
    /// Send a message using adaptive routing
    pub async fn send_message_adaptive(
        &self,
        destination: &Did,
        message: Vec<u8>,
    ) -> Result<(), MeshNetworkError> {
        let start_time = current_timestamp();
        
        // Find best route to destination
        let route = match self.routing_engine.find_best_route(destination).await? {
            Some(route) => route,
            None => {
                log::warn!("No route available to {}", destination);
                return Err(MeshNetworkError::PeerNotFound(
                    format!("No route to destination: {}", destination)
                ));
            }
        };
        
        // Attempt to send message
        let result = self.send_via_route(&route, message).await;
        let latency_ms = current_timestamp() - start_time;
        
        // Record routing result
        self.routing_engine.record_route_result(
            destination,
            &route.hops,
            result.is_ok(),
            Some(latency_ms),
        ).await?;
        
        result
    }
    
    /// Send message via a specific route
    async fn send_via_route(&self, route: &RouteInfo, message: Vec<u8>) -> Result<(), MeshNetworkError> {
        if route.hops.is_empty() {
            // Direct connection - convert DID to PeerId and send
            let peer_id = self.did_to_peer_id(&route.destination).await?;
            let protocol_message = self.create_protocol_message(message)?;
            self.inner.send_message(&peer_id, protocol_message).await
        } else {
            // Multi-hop routing - send to first hop with routing header
            let first_hop = &route.hops[0];
            let routing_message = self.create_routing_message(&route.destination, &route.hops[1..], message)?;
            self.inner.send_message(first_hop, routing_message).await
        }
    }
    
    /// Convert DID to PeerId (simplified)
    async fn did_to_peer_id(&self, destination: &Did) -> Result<PeerId, MeshNetworkError> {
        // Implement proper DID to PeerId resolution  
        // For now, we'll extract the peer ID from the DID or use a mapping
        let did_string = destination.to_string();
        
        // Check if it's already a peer-like format
        if did_string.starts_with("did:icn:peer:") {
            // Extract the peer portion
            let peer_part = did_string.strip_prefix("did:icn:peer:").unwrap_or(&did_string);
            return Ok(peer_part.to_string());
        }
        
        // Try to find a connected peer that corresponds to this DID
        match self.network_service.get_connected_peers().await {
            Ok(peers) => {
                // First, try exact match if the DID contains a recognizable peer ID
                for peer_id in &peers {
                    if did_string.contains(peer_id) {
                        return Ok(peer_id.clone());
                    }
                }
                
                // If no exact match, use the first available peer for now
                // In production, this would involve proper peer discovery and routing tables
                if let Some(first_peer) = peers.first() {
                    Ok(first_peer.clone())
                } else {
                    Err(MeshNetworkError::RoutingError("No connected peers available for DID resolution".to_string()))
                }
            }
            Err(e) => Err(MeshNetworkError::RoutingError(format!("Failed to get peers for DID resolution: {}", e)))
        }
    }
    
    /// Create protocol message from raw data
    fn create_protocol_message(&self, _data: Vec<u8>) -> Result<icn_protocol::ProtocolMessage, MeshNetworkError> {
        // Implement proper protocol message creation
        use serde_json::json;
        
        let message = json!({
            "type": "routing_message",
            "destination": destination.to_string(),
            "payload": serde_json::to_value(payload).map_err(|e| {
                MeshNetworkError::RoutingError(format!("Failed to serialize payload: {}", e))
            })?,
            "route": route.path,
            "timestamp": current_timestamp(),
            "ttl": 64, // Time to live for the message
        });
        
        let message_bytes = serde_json::to_vec(&message).map_err(|e| {
            MeshNetworkError::RoutingError(format!("Failed to serialize message: {}", e))
        })?;
        
        // Send the message to the next hop in the route
        self.network_service
            .send_message(&route.next_hop, "icn_routing", message_bytes)
            .await
            .map_err(|e| MeshNetworkError::RoutingError(format!("Failed to send routing message: {}", e)))?;
        Err(MeshNetworkError::InvalidInput("Protocol message creation not implemented".to_string()))
    }
    
    /// Create routing message for multi-hop delivery
    fn create_routing_message(
        &self,
        _destination: &Did,
        _remaining_hops: &[PeerId],
        _payload: Vec<u8>,
    ) -> Result<icn_protocol::ProtocolMessage, MeshNetworkError> {
        // Implement multi-hop routing message format
        use serde_json::json;
        
        let routing_message = json!({
            "type": "multi_hop_routing",
            "destination": destination.to_string(),
            "payload": serde_json::to_value(payload).map_err(|e| {
                MeshNetworkError::RoutingError(format!("Failed to serialize payload: {}", e))
            })?,
            "route_path": route.path,
            "current_hop": 0,
            "timestamp": current_timestamp(),
            "ttl": 64,
            "route_id": format!("route_{}", current_timestamp()),
        });
        
        let message_bytes = serde_json::to_vec(&routing_message).map_err(|e| {
            MeshNetworkError::RoutingError(format!("Failed to serialize routing message: {}", e))
        })?;
        
        // Send to the first hop in the route
        if let Some(first_hop) = route.path.first() {
            self.network_service
                .send_message(first_hop, "icn_multi_hop_routing", message_bytes)
                .await
                .map_err(|e| MeshNetworkError::RoutingError(format!("Failed to send multi-hop message: {}", e)))?;
            
            // Update metrics for this route
            self.update_route_metrics(&route.destination, first_hop, true, Duration::from_millis(15)).await;
        } else {
            return Err(MeshNetworkError::RoutingError("Route path is empty".to_string()));
        }
        Err(MeshNetworkError::InvalidInput("Multi-hop routing not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StubNetworkService;
    use icn_common::SystemTimeProvider;
    
    #[tokio::test]
    async fn test_adaptive_routing_engine_creation() {
        let config = AdaptiveRoutingConfig::default();
        let network_service = Arc::new(StubNetworkService::default());
        let time_provider = Arc::new(SystemTimeProvider);
        
        let mut engine = AdaptiveRoutingEngine::new(
            config,
            network_service,
            None,
            time_provider,
        );
        
        assert!(engine.start().await.is_ok());
        assert!(engine.take_event_receiver().is_some());
    }
    
    #[tokio::test]
    async fn test_route_scoring() {
        let weights = RouteSelectionWeights::default();
        let reputation_scores = HashMap::new();
        
        let route = RouteInfo {
            destination: Did::new("test", "destination"),
            hops: vec![],
            latency: 100,
            success_rate: 0.95,
            last_used: current_timestamp(),
            usage_count: 10,
            bandwidth_estimate: 1_000_000,
            discovered_at: current_timestamp(),
            is_healthy: true,
        };
        
        let score = route.calculate_score(&weights, &reputation_scores);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
    
    #[test]
    fn test_route_performance_updates() {
        let mut route = RouteInfo {
            destination: Did::new("test", "destination"),
            hops: vec![],
            latency: 100,
            success_rate: 0.8,
            last_used: current_timestamp(),
            usage_count: 0,
            bandwidth_estimate: 1_000_000,
            discovered_at: current_timestamp(),
            is_healthy: true,
        };
        
        // Record success
        route.record_success(Duration::from_millis(50));
        assert!(route.success_rate > 0.8);
        assert_eq!(route.usage_count, 1);
        
        // Record failure
        route.record_failure();
        assert!(route.success_rate < 0.9);
        assert_eq!(route.usage_count, 2);
    }
} 