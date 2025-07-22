//! Network service traits and types

use crate::CoreTraitsError;
use async_trait::async_trait;
use icn_common::CommonError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Peer identifier for networking
pub type PeerId = String;

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Number of connected peers
    pub connected_peers: usize,
    /// Number of sent messages
    pub messages_sent: u64,
    /// Number of received messages
    pub messages_received: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Connection uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            connected_peers: 0,
            messages_sent: 0,
            messages_received: 0,
            avg_latency_ms: 0.0,
            uptime_seconds: 0,
        }
    }
}

/// Network events that can be observed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// A peer connected
    PeerConnected { peer_id: PeerId },
    /// A peer disconnected
    PeerDisconnected { peer_id: PeerId },
    /// A message was received
    MessageReceived { from: PeerId, topic: String },
    /// A message was sent
    MessageSent { to: PeerId, topic: String },
    /// Network error occurred
    NetworkError { error: String },
}

/// Core network service trait
#[async_trait]
pub trait NetworkService: Send + Sync {
    /// Start the network service
    async fn start(&self) -> Result<(), CoreTraitsError>;
    
    /// Stop the network service
    async fn stop(&self) -> Result<(), CoreTraitsError>;
    
    /// Check if the service is running
    fn is_running(&self) -> bool;
    
    /// Get connected peers
    async fn get_connected_peers(&self) -> Result<Vec<PeerId>, CoreTraitsError>;
    
    /// Send a message to a specific peer
    async fn send_message(
        &self,
        peer_id: &str,
        topic: &str,
        data: Vec<u8>,
    ) -> Result<(), CoreTraitsError>;
    
    /// Broadcast a message to all peers
    async fn broadcast_message(&self, topic: &str, data: Vec<u8>) -> Result<(), CoreTraitsError>;
    
    /// Get network statistics
    async fn get_network_stats(&self) -> Result<NetworkStats, CoreTraitsError>;
    
    /// Subscribe to network events
    async fn subscribe_to_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>, CoreTraitsError>;
    
    /// Get the local peer ID
    fn get_local_peer_id(&self) -> PeerId;
}

/// Provider trait for network services
#[async_trait]
pub trait NetworkServiceProvider: Send + Sync {
    /// Create a new network service instance
    async fn create_network_service(
        &self,
        config: HashMap<String, String>,
    ) -> Result<Arc<dyn NetworkService>, CoreTraitsError>;
    
    /// Get the service type name
    fn get_service_type(&self) -> &'static str;
    
    /// Check if the provider is available
    fn is_available(&self) -> bool;
}

/// Simplified network trait for basic operations
#[async_trait]
pub trait BasicNetworkService: Send + Sync {
    /// Send data to a peer
    async fn send_to_peer(&self, peer_id: &str, data: &[u8]) -> Result<(), CommonError>;
    
    /// Get list of connected peers
    async fn get_peers(&self) -> Result<Vec<String>, CommonError>;
    
    /// Check if connected to a specific peer
    async fn is_connected(&self, peer_id: &str) -> Result<bool, CommonError>;
}

// Blanket implementation for basic operations
#[async_trait]
impl<T: NetworkService> BasicNetworkService for T {
    async fn send_to_peer(&self, peer_id: &str, data: &[u8]) -> Result<(), CommonError> {
        self.send_message(peer_id, "default", data.to_vec())
            .await
            .map_err(|e| CommonError::InvalidInputError(e.to_string()))
    }
    
    async fn get_peers(&self) -> Result<Vec<String>, CommonError> {
        self.get_connected_peers()
            .await
            .map_err(|e| CommonError::InvalidInputError(e.to_string()))
    }
    
    async fn is_connected(&self, peer_id: &str) -> Result<bool, CommonError> {
        let peers = self.get_connected_peers().await
            .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
        Ok(peers.contains(&peer_id.to_string()))
    }
}