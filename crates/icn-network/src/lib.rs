#![doc = include_str!("../README.md")]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_constructed_unit_structs)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::clone_on_copy)]

//! # Networking Crate for ICN - Production-Ready P2P Networking
//! This crate manages peer-to-peer (P2P) networking aspects for ICN,
//! using libp2p for distributed communication between nodes.

pub mod error;
pub use error::MeshNetworkError;
pub mod adaptive_routing;
pub mod metrics;
pub mod service_factory;
pub use adaptive_routing::{
    AdaptiveNetworkService, AdaptiveRoutingConfig, AdaptiveRoutingEngine, NetworkTopology,
    RouteInfo, RouteSelectionWeights, RoutingEvent,
};
pub use service_factory::{
    BootstrapPeer, NetworkEnvironment, NetworkServiceConfig, NetworkServiceCreationResult,
    NetworkServiceFactory, NetworkServiceOptions, NetworkServiceOptionsBuilder,
};

use async_trait::async_trait;
use downcast_rs::{impl_downcast, DowncastSync};
use icn_common::{
    retry_with_backoff, Cid, CircuitBreaker, CircuitBreakerError, Did, NodeInfo, SystemTimeProvider,
};
use icn_protocol::{MessagePayload, ProtocolMessage};
#[cfg(feature = "libp2p")]
use libp2p::PeerId as Libp2pPeerId;
#[cfg(feature = "libp2p")]
use log::{info, warn};
use lru::LruCache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(feature = "libp2p")]
use std::str::FromStr;
use std::sync::Mutex;
#[cfg(feature = "libp2p")]
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
// Removed unused imports for testing Kademlia disabled build

/// Prefix for service advertisement records stored in the DHT.
///
/// Keys are constructed as `format!("{SERVICE_AD_PREFIX}{did}")` where `did`
/// is the DID of the advertising node. For example, a node with the DID
/// `did:web:example.com` should advertise under the key
/// `/icn/service/did:web:example.com`.
pub const SERVICE_AD_PREFIX: &str = "/icn/service/";

/// Prefix for DID document records stored in the DHT.
///
/// Keys are constructed as `format!("{DID_DOC_PREFIX}{did}")`. A DID
/// document for `did:web:example.com` would therefore be stored under
/// `/icn/did/did:web:example.com`.
pub const DID_DOC_PREFIX: &str = "/icn/did/";

/// Prefix for federation info records stored in the DHT.
pub const FEDERATION_INFO_PREFIX: &str = "/icn/fedinfo/";

/// Legacy type aliases for compatibility
pub type Job = icn_protocol::MeshJobAnnouncementMessage;
pub type Bid = icn_protocol::MeshBidSubmissionMessage;
pub type JobId = Cid;

/// Cache of recently verified message hashes to prevent replay.
static MESSAGE_CACHE: Lazy<Mutex<LruCache<Vec<u8>, ()>>> = Lazy::new(|| {
    use std::num::NonZeroUsize;
    Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap()))
});

static NETWORK_BREAKER: Lazy<tokio::sync::Mutex<CircuitBreaker<SystemTimeProvider>>> =
    Lazy::new(|| {
        tokio::sync::Mutex::new(CircuitBreaker::new(
            SystemTimeProvider,
            3,
            std::time::Duration::from_secs(5),
        ))
    });

async fn with_resilience<F, Fut, T>(mut op: F) -> Result<T, MeshNetworkError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MeshNetworkError>>,
{
    use std::time::Duration;
    let breaker = NETWORK_BREAKER.lock().await;
    breaker
        .call(|| async {
            retry_with_backoff(
                &mut op,
                3,
                Duration::from_millis(100),
                Duration::from_secs(2),
            )
            .await
        })
        .await
        .map_err(|e| match e {
            CircuitBreakerError::Open => MeshNetworkError::Timeout("circuit open".to_string()),
            CircuitBreakerError::Inner(err) => err,
        })
}

// --- Core Types ---

/// Wrapper around a peer's stable identifier used within the network layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PeerId {
    /// Construct from a raw string.
    pub fn from_string(s: String) -> Self {
        PeerId(s)
    }
    /// Borrow the underlying string representation.
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(feature = "libp2p")]
impl From<Libp2pPeerId> for PeerId {
    fn from(p: Libp2pPeerId) -> Self {
        PeerId(p.to_string())
    }
}

#[cfg(feature = "libp2p")]
impl std::convert::TryFrom<PeerId> for Libp2pPeerId {
    type Error = libp2p::identity::ParseError;

    fn try_from(value: PeerId) -> Result<Self, Self::Error> {
        Self::from_str(&value.0)
    }
}

/// A network message signed by the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    /// The underlying payload.
    pub message: ProtocolMessage,
    /// DID of the sender.
    pub sender: Did,
    /// Signature over the message and DID.
    pub signature: icn_identity::SignatureBytes,
}

/// Decode a raw byte slice into a [`ProtocolMessage`].
///
/// Returns [`MeshNetworkError::MessageDecodeFailed`] if the bytes cannot be
/// deserialized using `bincode`.
pub fn decode_protocol_message(data: &[u8]) -> Result<ProtocolMessage, MeshNetworkError> {
    bincode::deserialize(data).map_err(|e| MeshNetworkError::MessageDecodeFailed(e.to_string()))
}

/// Create a [`SignedMessage`] by signing `message` with `signing_key` and recording `sender`.
pub fn sign_message(
    message: &ProtocolMessage,
    sender: &Did,
    signing_key: &icn_identity::SigningKey,
) -> Result<SignedMessage, icn_common::CommonError> {
    let mut bytes = sender.to_string().into_bytes();
    let msg_bytes = bincode::serialize(message)
        .map_err(|e| icn_common::CommonError::SerializationError(e.to_string()))?;
    bytes.extend_from_slice(&msg_bytes);
    let sig = icn_identity::sign_message(signing_key, &bytes);
    Ok(SignedMessage {
        message: message.clone(),
        sender: sender.clone(),
        signature: icn_identity::SignatureBytes(sig.to_bytes().to_vec()),
    })
}

/// Verify the signature contained in a [`SignedMessage`].
pub fn verify_message_signature(msg: &SignedMessage) -> Result<(), icn_common::CommonError> {
    let mut bytes = msg.sender.to_string().into_bytes();
    let msg_bytes = bincode::serialize(&msg.message)
        .map_err(|e| icn_common::CommonError::SerializationError(e.to_string()))?;
    bytes.extend_from_slice(&msg_bytes);
    let digest = Sha256::digest(&bytes);
    {
        let cache = &mut *MESSAGE_CACHE.lock().expect("cache mutex poisoned");
        if cache.contains(&digest.to_vec()) {
            return Err(icn_common::CommonError::DuplicateMessage);
        }
    }

    let verifying_key = icn_identity::verifying_key_from_did_key(&msg.sender)?;
    let ed_sig = msg.signature.to_ed_signature()?;
    if icn_identity::verify_signature(&verifying_key, &bytes, &ed_sig) {
        let cache = &mut *MESSAGE_CACHE.lock().expect("cache mutex poisoned");
        cache.put(digest.to_vec(), ());
        Ok(())
    } else {
        Err(icn_common::CommonError::CryptoError(
            "Message signature verification failed".into(),
        ))
    }
}

/// Comprehensive network statistics for monitoring and observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub peer_count: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub failed_connections: u64,
    pub avg_latency_ms: Option<u64>,
    /// Minimum observed round-trip latency in milliseconds.
    pub min_latency_ms: Option<u64>,
    /// Maximum observed round-trip latency in milliseconds.
    pub max_latency_ms: Option<u64>,
    /// Last measured round-trip latency in milliseconds.
    pub last_latency_ms: Option<u64>,
    pub kademlia_peers: usize,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            peer_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            failed_connections: 0,
            avg_latency_ms: None,
            min_latency_ms: None,
            max_latency_ms: None,
            last_latency_ms: None,
            kademlia_peers: 0,
        }
    }
}

/// Network service trait definition.
#[async_trait]
pub trait NetworkService: Send + Sync + Debug + DowncastSync + 'static {
    async fn discover_peers(
        &self,
        target_peer_id_str: Option<String>,
    ) -> Result<Vec<PeerId>, MeshNetworkError>;
    /// Discover federations known to connected peers.
    async fn discover_federations(
        &self,
    ) -> Result<Vec<icn_protocol::FederationInfo>, MeshNetworkError> {
        Err(MeshNetworkError::InvalidInput(
            "Federation discovery not supported".into(),
        ))
    }
    async fn send_message(
        &self,
        peer: &PeerId,
        message: ProtocolMessage,
    ) -> Result<(), MeshNetworkError>;
    async fn broadcast_message(&self, message: ProtocolMessage) -> Result<(), MeshNetworkError>;
    async fn subscribe(&self) -> Result<Receiver<ProtocolMessage>, MeshNetworkError>;
    /// Send a pre-signed message. Default implementation returns an error.
    async fn send_signed_message(
        &self,
        _peer: &PeerId,
        _message: SignedMessage,
    ) -> Result<(), MeshNetworkError> {
        Err(MeshNetworkError::InvalidInput(
            "Signed messaging not supported".to_string(),
        ))
    }

    /// Broadcast a pre-signed message. Default implementation returns an error.
    async fn broadcast_signed_message(
        &self,
        _message: SignedMessage,
    ) -> Result<(), MeshNetworkError> {
        Err(MeshNetworkError::InvalidInput(
            "Signed messaging not supported".to_string(),
        ))
    }

    /// Subscribe to signed messages. Default implementation returns an error.
    async fn subscribe_signed(&self) -> Result<Receiver<SignedMessage>, MeshNetworkError> {
        Err(MeshNetworkError::InvalidInput(
            "Signed messaging not supported".to_string(),
        ))
    }
    async fn get_network_stats(&self) -> Result<NetworkStats, MeshNetworkError>;
    /// Store a record in the network DHT. Keys should take the form
    /// `/icn/service/<id>` to avoid collisions between components.
    async fn store_record(&self, key: String, value: Vec<u8>) -> Result<(), MeshNetworkError>;

    /// Retrieve a record previously stored via [`NetworkService::store_record`].
    /// Keys use the `/icn/service/<id>` format.
    async fn get_record(&self, key: String) -> Result<Option<Vec<u8>>, MeshNetworkError>;
    /// Connect to a peer at the given multiaddress.
    #[cfg(feature = "libp2p")]
    async fn connect_peer(&self, addr: libp2p::Multiaddr) -> Result<(), MeshNetworkError>;
    fn as_any(&self) -> &dyn Any;
}
impl_downcast!(sync NetworkService);

/// Stub implementation for testing.
#[derive(Debug)]
pub struct StubNetworkService {
    records: std::sync::Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>,
    federations: std::sync::Arc<tokio::sync::Mutex<Vec<icn_protocol::FederationInfo>>>,
}

impl Default for StubNetworkService {
    fn default() -> Self {
        Self {
            records: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            federations: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl NetworkService for StubNetworkService {
    async fn discover_peers(
        &self,
        target_peer_id_str: Option<String>,
    ) -> Result<Vec<PeerId>, MeshNetworkError> {
        with_resilience(|| {
            let target = target_peer_id_str.clone();
            async move {
                log::info!(
                    "[StubNetworkService] Discovering peers (target: {:?})... returning mock peers.",
                    target
                );
                Ok(vec![
                    PeerId("mock_peer_1".to_string()),
                    PeerId("mock_peer_2".to_string()),
                ])
            }
        })
        .await
    }

    async fn send_message(
        &self,
        peer: &PeerId,
        message: ProtocolMessage,
    ) -> Result<(), MeshNetworkError> {
        let peer = peer.clone();
        let msg = message.clone();
        with_resilience(|| {
            let peer = peer.clone();
            let msg = msg.clone();
            async move {
                log::debug!(
                    "[StubNetworkService] Sending message to peer {:?}: {:?}",
                    peer,
                    msg
                );
                if peer.0 == "error_peer" {
                    return Err(MeshNetworkError::SendFailure(format!(
                        "Failed to send message to peer: {}",
                        peer.0
                    )));
                }
                if peer.0 == "unknown_peer_id" {
                    return Err(MeshNetworkError::PeerNotFound(format!(
                        "Peer with ID {} not found.",
                        peer.0
                    )));
                }
                Ok(())
            }
        })
        .await
    }

    async fn broadcast_message(&self, message: ProtocolMessage) -> Result<(), MeshNetworkError> {
        let msg = message.clone();
        with_resilience(|| {
            let m = msg.clone();
            async move {
                log::debug!("[StubNetworkService] Broadcasting message: {:?}", m);
                if let MessagePayload::GossipMessage(gossip) = &m.payload {
                    if gossip.topic == "system_critical_error_topic" {
                        return Err(MeshNetworkError::Libp2p(
                            "Broadcast failed: system critical topic is currently down."
                                .to_string(),
                        ));
                    }
                }
                Ok(())
            }
        })
        .await
    }

    async fn subscribe(&self) -> Result<Receiver<ProtocolMessage>, MeshNetworkError> {
        with_resilience(|| async {
            log::info!(
                "[StubNetworkService] Subscribing to messages... returning an empty channel."
            );
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        })
        .await
    }

    async fn discover_federations(
        &self,
    ) -> Result<Vec<icn_protocol::FederationInfo>, MeshNetworkError> {
        Ok(self.federations.lock().await.clone())
    }

    async fn get_network_stats(&self) -> Result<NetworkStats, MeshNetworkError> {
        with_resilience(|| async { Ok(NetworkStats::default()) }).await
    }

    async fn send_signed_message(
        &self,
        peer: &PeerId,
        message: SignedMessage,
    ) -> Result<(), MeshNetworkError> {
        verify_message_signature(&message).map_err(MeshNetworkError::Common)?;
        log::debug!(
            "[StubNetworkService] Sending signed message to peer {:?}: {:?}",
            peer,
            message.message
        );
        self.send_message(peer, message.message).await
    }

    async fn broadcast_signed_message(
        &self,
        message: SignedMessage,
    ) -> Result<(), MeshNetworkError> {
        verify_message_signature(&message).map_err(MeshNetworkError::Common)?;
        log::debug!(
            "[StubNetworkService] Broadcasting signed message: {:?}",
            message.message
        );
        self.broadcast_message(message.message).await
    }

    async fn subscribe_signed(&self) -> Result<Receiver<SignedMessage>, MeshNetworkError> {
        log::info!(
            "[StubNetworkService] Subscribing to signed messages... returning an empty channel."
        );
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }

    async fn store_record(&self, key: String, value: Vec<u8>) -> Result<(), MeshNetworkError> {
        let key_cl = key.clone();
        let val_cl = value.clone();
        with_resilience(|| {
            let k = key_cl.clone();
            let v = val_cl.clone();
            async move {
                let mut map = self.records.lock().await;
                map.insert(k.clone(), v.clone());
                if k.starts_with(FEDERATION_INFO_PREFIX) {
                    if let Ok(info) = bincode::deserialize::<icn_protocol::FederationInfo>(&v) {
                        let mut feds = self.federations.lock().await;
                        if !feds.iter().any(|i| i.federation_id == info.federation_id) {
                            feds.push(info);
                        }
                    }
                }
                Ok(())
            }
        })
        .await
    }

    async fn get_record(&self, key: String) -> Result<Option<Vec<u8>>, MeshNetworkError> {
        let key_cl = key.clone();
        with_resilience(|| {
            let k = key_cl.clone();
            async move {
                let map = self.records.lock().await;
                Ok(map.get(&k).cloned())
            }
        })
        .await
    }

    #[cfg(feature = "libp2p")]
    async fn connect_peer(&self, addr: libp2p::Multiaddr) -> Result<(), MeshNetworkError> {
        let a = addr.clone();
        with_resilience(|| {
            let value = a.clone();
            async move {
                log::info!(
                    "[StubNetworkService] Pretending to connect to peer at {}",
                    value
                );
                Ok(())
            }
        })
        .await
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Placeholder function for testing network operations.
pub async fn send_network_ping(
    info: &NodeInfo,
    target_peer: &str,
) -> Result<String, MeshNetworkError> {
    let service = StubNetworkService::default();
    use icn_protocol::{GossipMessage, MessagePayload, ProtocolMessage};
    use std::str::FromStr;

    let ping_message = ProtocolMessage::new(
        MessagePayload::GossipMessage(GossipMessage {
            topic: "ping_topic".to_string(),
            payload: vec![1, 2, 3],
            ttl: 5,
        }),
        Did::from_str("did:key:stub").unwrap(),
        None,
    );

    use std::time::Duration;
    {
        let breaker = NETWORK_BREAKER.lock().await;
        breaker
            .call(|| async {
                retry_with_backoff(
                    || async {
                        service
                            .send_message(&PeerId(target_peer.to_string()), ping_message.clone())
                            .await
                    },
                    3,
                    Duration::from_millis(100),
                    Duration::from_secs(2),
                )
                .await
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Open => MeshNetworkError::Timeout("circuit open".to_string()),
                CircuitBreakerError::Inner(err) => err,
            })?;
    }
    Ok(format!(
        "Sent (stubbed) ping to {} from node: {} (v{})",
        target_peer, info.name, info.version
    ))
}

/// Broadcast a governance proposal with retry logic.
pub async fn gossip_proposal_with_retry<S: NetworkService>(
    service: &S,
    message: ProtocolMessage,
) -> Result<(), MeshNetworkError> {
    with_resilience(|| {
        let msg = message.clone();
        async move { service.broadcast_message(msg).await }
    })
    .await
}

pub async fn gossip_message_with_retry<S: NetworkService>(
    service: &S,
    message: ProtocolMessage,
) -> Result<(), MeshNetworkError> {
    with_resilience(|| {
        let msg = message.clone();
        async move { service.broadcast_message(msg).await }
    })
    .await
}

/// Confirm quorum status by querying the network with retries.
pub async fn confirm_quorum_with_retry<S: NetworkService>(
    service: &S,
    quorum_key: String,
) -> Result<bool, MeshNetworkError> {
    with_resilience(|| {
        let key = quorum_key.clone();
        async move {
            match service.get_record(key).await? {
                Some(v) => Ok(v == b"confirmed"),
                None => Ok(false),
            }
        }
    })
    .await
}

#[cfg(all(test, feature = "libp2p"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_network_service_basic() {
        let service = StubNetworkService::default();
        let peers = service
            .discover_peers(Some("/ip4/127.0.0.1/tcp/12345".to_string()))
            .await
            .unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].0, "mock_peer_1");

        let stats = service.get_network_stats().await.unwrap();
        assert_eq!(stats.peer_count, 0);
    }
}

// --- Production libp2p Implementation ---
#[cfg(feature = "libp2p")]
pub mod libp2p_service {
    use super::*;
    use libp2p::futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
    use libp2p::{
        core::upgrade,
        dns, gossipsub, identity,
        kad::{
            self as kad, store::MemoryStore, Behaviour as KademliaBehaviour,
            Config as KademliaConfig, Event as KademliaEvent, QueryId, Quorum,
            Record as KademliaRecord, RecordKey as KademliaKey,
        },
        noise, ping,
        request_response::{
            Behaviour as RequestResponseBehaviour, Codec as RequestResponseCodec, ProtocolSupport,
        },
        swarm::{Config as SwarmConfig, NetworkBehaviour, Swarm, SwarmEvent},
        tcp, yamux, Multiaddr, PeerId as Libp2pPeerId, Transport,
    };
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use tokio::{
        sync::{mpsc, oneshot},
        task,
    };

    // --- Enhanced Statistics and Configuration ---

    /// Configuration options for the libp2p networking backend.
    ///
    /// **🏭 PRODUCTION CONFIGURATION**
    ///
    /// This configuration is designed for production use with the following characteristics:
    /// - **max_peers**: 1000 concurrent connections (suitable for most production workloads)
    /// - **connection_timeout**: 30s (balances reliability with responsiveness)
    /// - **heartbeat_interval**: 15s (keeps connections alive without excessive overhead)
    /// - **bootstrap_interval**: 5min (periodic reconnection to bootstrap peers)
    /// - **peer_discovery_interval**: 1min (regular discovery of new peers)
    /// - **kademlia_replication_factor**: 20 (good balance of redundancy and performance)
    ///
    /// **📋 CONFIGURATION CHECKLIST FOR PRODUCTION:**
    /// - [ ] Configure `bootstrap_peers` with stable, well-connected nodes
    /// - [ ] Set `listen_addresses` to appropriate network interfaces
    /// - [ ] Consider enabling `enable_mdns` for local network discovery
    /// - [ ] Monitor peer count and connection health
    /// - [ ] Configure firewall rules for p2p port range
    ///
    /// **🔧 EXAMPLE PRODUCTION CONFIG:**
    /// ```rust
    /// NetworkConfig {
    ///     listen_addresses: vec!["/ip4/0.0.0.0/tcp/7946".parse().unwrap()],
    ///     bootstrap_peers: vec![
    ///         (peer_id_1, "/ip4/1.2.3.4/tcp/7946/p2p/12D3...".parse().unwrap()),
    ///         (peer_id_2, "/ip4/5.6.7.8/tcp/7946/p2p/12D3...".parse().unwrap()),
    ///     ],
    ///     enable_mdns: false, // Disable in production for security
    ///     ..Default::default()
    /// }
    /// ```
    #[derive(Debug, Clone)]
    pub struct NetworkConfig {
        pub listen_addresses: Vec<Multiaddr>,
        pub max_peers: usize,
        pub max_peers_per_ip: usize,
        pub connection_timeout: Duration,
        pub request_timeout: Duration,
        pub heartbeat_interval: Duration,
        pub bootstrap_interval: Duration,
        /// Interval for automatic peer discovery queries
        pub peer_discovery_interval: Duration,
        pub enable_mdns: bool,
        pub kademlia_replication_factor: usize,
        pub bootstrap_peers: Vec<(Libp2pPeerId, Multiaddr)>,
    }

    impl Default for NetworkConfig {
        fn default() -> Self {
            Self {
                listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()],
                max_peers: 1000,
                max_peers_per_ip: 5,
                connection_timeout: Duration::from_secs(30),
                request_timeout: Duration::from_secs(10),
                heartbeat_interval: Duration::from_secs(15),
                bootstrap_interval: Duration::from_secs(300),
                peer_discovery_interval: Duration::from_secs(60),
                enable_mdns: false,
                kademlia_replication_factor: 20,
                bootstrap_peers: Vec::new(),
            }
        }
    }

    impl NetworkConfig {
        /// Create a production-ready network configuration with sensible defaults.
        ///
        /// This configuration is optimized for production deployments with:
        /// - Increased connection limits and timeouts
        /// - Disabled mDNS for security
        /// - Optimized Kademlia settings
        pub fn production() -> Self {
            Self {
                listen_addresses: vec!["/ip4/0.0.0.0/tcp/7946".parse().unwrap()],
                max_peers: 1000,
                max_peers_per_ip: 5,
                connection_timeout: Duration::from_secs(30),
                request_timeout: Duration::from_secs(10),
                heartbeat_interval: Duration::from_secs(15),
                bootstrap_interval: Duration::from_secs(300),
                peer_discovery_interval: Duration::from_secs(60),
                enable_mdns: false, // Disabled for production security
                kademlia_replication_factor: 20,
                bootstrap_peers: Vec::new(),
            }
        }

        /// Create a development-friendly network configuration.
        ///
        /// This configuration enables mDNS for local peer discovery and uses
        /// more aggressive timings for faster development iteration.
        pub fn development() -> Self {
            Self {
                listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse().unwrap()],
                max_peers: 100,
                max_peers_per_ip: 10,
                connection_timeout: Duration::from_secs(10),
                request_timeout: Duration::from_secs(5),
                heartbeat_interval: Duration::from_secs(5),
                bootstrap_interval: Duration::from_secs(60),
                peer_discovery_interval: Duration::from_secs(30),
                enable_mdns: true, // Enabled for local development
                kademlia_replication_factor: 10,
                bootstrap_peers: Vec::new(),
            }
        }

        /// Validate the network configuration for production readiness.
        ///
        /// Returns an error if the configuration has settings that are not
        /// suitable for production use.
        pub fn validate_production(&self) -> Result<(), MeshNetworkError> {
            // Check for reasonable connection limits
            if self.max_peers < 10 {
                return Err(MeshNetworkError::SetupError(
                    "max_peers too low for production (minimum 10)".to_string(),
                ));
            }

            if self.max_peers_per_ip > 50 {
                return Err(MeshNetworkError::SetupError(
                    "max_peers_per_ip too high for production (maximum 50)".to_string(),
                ));
            }

            // Check for reasonable timeouts
            if self.connection_timeout < Duration::from_secs(5) {
                return Err(MeshNetworkError::SetupError(
                    "connection_timeout too low for production (minimum 5s)".to_string(),
                ));
            }

            if self.request_timeout < Duration::from_secs(1) {
                return Err(MeshNetworkError::SetupError(
                    "request_timeout too low for production (minimum 1s)".to_string(),
                ));
            }

            // Check for listen addresses
            if self.listen_addresses.is_empty() {
                return Err(MeshNetworkError::SetupError(
                    "No listen addresses configured".to_string(),
                ));
            }

            // Warn about mDNS in production
            if self.enable_mdns {
                log::warn!(
                    "⚠️  mDNS is enabled in production configuration. Consider disabling for security."
                );
            }

            // Check for bootstrap peers
            if self.bootstrap_peers.is_empty() {
                log::warn!(
                    "⚠️  No bootstrap peers configured. Node may have difficulty joining the network."
                );
            }

            Ok(())
        }

        /// Add a bootstrap peer from a multiaddr string.
        ///
        /// The multiaddr must include a peer ID component (e.g., `/p2p/12D3...`).
        pub fn add_bootstrap_peer(&mut self, multiaddr: &str) -> Result<(), MeshNetworkError> {
            let addr = multiaddr
                .parse::<Multiaddr>()
                .map_err(|e| MeshNetworkError::InvalidInput(format!("Invalid multiaddr: {}", e)))?;

            // Extract peer ID from multiaddr
            let peer_id = addr
                .iter()
                .find_map(|protocol| {
                    #[allow(clippy::useless_conversion)]
                    match protocol {
                        libp2p::core::multiaddr::Protocol::P2p(pid) => Some(pid.try_into().ok()?),
                        _ => None,
                    }
                })
                .ok_or_else(|| {
                    MeshNetworkError::InvalidInput(
                        "Multiaddr must contain a peer ID component (/p2p/...)".to_string(),
                    )
                })?;

            self.bootstrap_peers.push((peer_id, addr));
            Ok(())
        }

        /// Set listen addresses from string representations.
        pub fn set_listen_addresses(
            &mut self,
            addresses: Vec<&str>,
        ) -> Result<(), MeshNetworkError> {
            let mut parsed_addresses = Vec::new();

            for addr_str in addresses {
                let addr = addr_str.parse::<Multiaddr>().map_err(|e| {
                    MeshNetworkError::InvalidInput(format!(
                        "Invalid listen address '{}': {}",
                        addr_str, e
                    ))
                })?;
                parsed_addresses.push(addr);
            }

            self.listen_addresses = parsed_addresses;
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct EnhancedNetworkStats {
        peer_count: usize,
        bytes_sent: u64,
        bytes_received: u64,
        messages_sent: u64,
        messages_received: u64,
        failed_connections: u64,
        message_counts: HashMap<String, MessageTypeStats>,
        kademlia_peers: usize,
        last_latency_ms: Option<u64>,
        min_latency_ms: Option<u64>,
        max_latency_ms: Option<u64>,
        total_latency_ms: u64,
        ping_count: u64,
    }

    impl EnhancedNetworkStats {
        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        fn update_kademlia_peers(&mut self, count: usize) {
            self.kademlia_peers = count;
            crate::metrics::KADEMLIA_PEERS_GAUGE.set(count as i64);
        }

        fn record_latency(&mut self, rtt: std::time::Duration) {
            let ms = rtt.as_millis() as u64;
            self.last_latency_ms = Some(ms);
            self.min_latency_ms = Some(self.min_latency_ms.map_or(ms, |m| m.min(ms)));
            self.max_latency_ms = Some(self.max_latency_ms.map_or(ms, |m| m.max(ms)));
            self.total_latency_ms += ms;
            self.ping_count += 1;

            #[allow(unused_variables)]
            {
                use crate::metrics::{
                    PING_AVG_RTT_MS, PING_LAST_RTT_MS, PING_MAX_RTT_MS, PING_MIN_RTT_MS,
                };
                PING_LAST_RTT_MS.set(ms as f64);
                if let Some(min) = self.min_latency_ms {
                    PING_MIN_RTT_MS.set(min as f64);
                }
                if let Some(max) = self.max_latency_ms {
                    PING_MAX_RTT_MS.set(max as f64);
                }
                if let Some(avg) = self.avg_latency() {
                    PING_AVG_RTT_MS.set(avg as f64);
                }
            }
        }

        fn avg_latency(&self) -> Option<u64> {
            if self.ping_count > 0 {
                Some(self.total_latency_ms / self.ping_count)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    struct MessageTypeStats {
        sent: u64,
        received: u64,
        bytes_sent: u64,
        bytes_received: u64,
    }

    /// Production implementation of [`NetworkService`] backed by libp2p.
    #[derive(Debug)]
    pub struct Libp2pNetworkService {
        local_peer_id: Libp2pPeerId,
        cmd_tx: mpsc::Sender<Command>,
        config: NetworkConfig,
        listening_addresses: Arc<Mutex<Vec<Multiaddr>>>,
        event_loop_handle: task::JoinHandle<()>, // Hold the handle to prevent task cancellation
        federations: Arc<Mutex<Vec<icn_protocol::FederationInfo>>>,
    }

    impl Clone for Libp2pNetworkService {
        fn clone(&self) -> Self {
            // We can't clone the JoinHandle, but we can create a new service that shares
            // the same command channel and peer ID. The event loop is already running.
            Self {
                local_peer_id: self.local_peer_id.clone(),
                cmd_tx: self.cmd_tx.clone(),
                config: self.config.clone(),
                listening_addresses: self.listening_addresses.clone(),
                event_loop_handle: task::spawn(async {}), // Dummy handle for clones
                federations: self.federations.clone(),
            }
        }
    }

    #[derive(Debug)]
    enum Command {
        DiscoverPeers {
            target: Option<Libp2pPeerId>,
            rsp: oneshot::Sender<Result<Vec<super::PeerId>, MeshNetworkError>>,
        },
        SendMessage {
            peer: Libp2pPeerId,
            message: super::ProtocolMessage,
            rsp: oneshot::Sender<Result<(), MeshNetworkError>>,
        },
        Broadcast {
            data: Vec<u8>,
        },
        Subscribe {
            rsp: oneshot::Sender<mpsc::Receiver<super::ProtocolMessage>>,
        },
        GetStats {
            rsp: oneshot::Sender<super::NetworkStats>,
        },
        GetKademliaRecord {
            key: KademliaKey,
            rsp: oneshot::Sender<Result<Option<KademliaRecord>, MeshNetworkError>>,
        },
        PutKademliaRecord {
            key: KademliaKey,
            value: Vec<u8>,
            rsp: oneshot::Sender<Result<(), MeshNetworkError>>,
        },
        ConnectPeer {
            addr: Multiaddr,
            rsp: oneshot::Sender<Result<(), MeshNetworkError>>,
        },
        Shutdown,
    }

    #[derive(Debug)]
    enum PendingQuery {
        GetRecord(oneshot::Sender<Result<Option<KademliaRecord>, MeshNetworkError>>),
        PutRecord(oneshot::Sender<Result<(), MeshNetworkError>>),
        GetPeers(oneshot::Sender<Result<Vec<super::PeerId>, MeshNetworkError>>),
    }

    // --- Protocol Implementation ---

    #[derive(Debug, Clone)]
    pub struct MessageCodec;

    #[derive(Debug, Clone)]
    pub struct MessageProtocol();

    impl FromStr for MessageProtocol {
        type Err = std::io::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "/icn/message/1.0.0" => Ok(MessageProtocol()),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid protocol",
                )),
            }
        }
    }

    impl AsRef<str> for MessageProtocol {
        fn as_ref(&self) -> &str {
            "/icn/message/1.0.0"
        }
    }

    #[async_trait::async_trait]
    impl RequestResponseCodec for MessageCodec {
        type Protocol = MessageProtocol;
        type Request = super::ProtocolMessage;
        type Response = super::ProtocolMessage;

        async fn read_request<T>(
            &mut self,
            _: &MessageProtocol,
            io: &mut T,
        ) -> std::io::Result<super::ProtocolMessage>
        where
            T: libp2p::futures::AsyncRead + Unpin + Send,
        {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            bincode::deserialize(&buffer)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }

        async fn read_response<T>(
            &mut self,
            _: &MessageProtocol,
            io: &mut T,
        ) -> std::io::Result<super::ProtocolMessage>
        where
            T: libp2p::futures::AsyncRead + Unpin + Send,
        {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            bincode::deserialize(&buffer)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }

        async fn write_request<T>(
            &mut self,
            _: &MessageProtocol,
            io: &mut T,
            req: super::ProtocolMessage,
        ) -> std::io::Result<()>
        where
            T: libp2p::futures::AsyncWrite + Unpin + Send,
        {
            let data = bincode::serialize(&req)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await
        }

        async fn write_response<T>(
            &mut self,
            _: &MessageProtocol,
            io: &mut T,
            res: super::ProtocolMessage,
        ) -> std::io::Result<()>
        where
            T: libp2p::futures::AsyncWrite + Unpin + Send,
        {
            let data = bincode::serialize(&res)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await
        }
    }

    // --- Network Behaviour Definition ---

    use libp2p::swarm::behaviour::toggle::Toggle;

    #[derive(NetworkBehaviour)]
    #[behaviour(to_swarm = "CombinedBehaviourEvent")]
    pub struct CombinedBehaviour {
        gossipsub: gossipsub::Behaviour,
        ping: ping::Behaviour,
        kademlia: KademliaBehaviour<MemoryStore>,
        request_response: RequestResponseBehaviour<MessageCodec>,
        mdns: Toggle<libp2p::mdns::tokio::Behaviour>,
    }

    // Define the combined event type manually with proper From implementations
    #[derive(Debug)]
    pub enum CombinedBehaviourEvent {
        Gossipsub(gossipsub::Event),
        Ping(ping::Event),
        Kademlia(kad::Event),
        RequestResponse(
            libp2p::request_response::Event<super::ProtocolMessage, super::ProtocolMessage>,
        ),
        Mdns(libp2p::mdns::Event),
    }

    // Implement From traits for each event type
    impl From<gossipsub::Event> for CombinedBehaviourEvent {
        fn from(event: gossipsub::Event) -> Self {
            CombinedBehaviourEvent::Gossipsub(event)
        }
    }

    impl From<ping::Event> for CombinedBehaviourEvent {
        fn from(event: ping::Event) -> Self {
            CombinedBehaviourEvent::Ping(event)
        }
    }

    impl From<kad::Event> for CombinedBehaviourEvent {
        fn from(event: kad::Event) -> Self {
            CombinedBehaviourEvent::Kademlia(event)
        }
    }

    impl From<libp2p::request_response::Event<super::ProtocolMessage, super::ProtocolMessage>>
        for CombinedBehaviourEvent
    {
        fn from(
            event: libp2p::request_response::Event<super::ProtocolMessage, super::ProtocolMessage>,
        ) -> Self {
            CombinedBehaviourEvent::RequestResponse(event)
        }
    }

    impl From<libp2p::mdns::Event> for CombinedBehaviourEvent {
        fn from(event: libp2p::mdns::Event) -> Self {
            CombinedBehaviourEvent::Mdns(event)
        }
    }

    impl Libp2pNetworkService {
        /// Spawn the networking service using the given configuration.
        pub async fn new(config: NetworkConfig) -> Result<Self, MeshNetworkError> {
            if config.connection_timeout.is_zero() {
                return Err(MeshNetworkError::HandshakeFailed(
                    "connection_timeout must be greater than zero".into(),
                ));
            }
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = Libp2pPeerId::from(local_key.public());

            let transport = dns::tokio::Transport::system(tcp::tokio::Transport::new(
                tcp::Config::default().nodelay(true),
            ))
            .map_err(|e| MeshNetworkError::SetupError(format!("DNS config error: {}", e)))?
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key).map_err(|e| {
                MeshNetworkError::HandshakeFailed(format!("Noise auth error: {}", e))
            })?)
            .multiplex(yamux::Config::default())
            .timeout(config.connection_timeout)
            .boxed();

            let gossipsub_config = gossipsub::Config::default();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            )
            .map_err(|s| MeshNetworkError::SetupError(format!("Gossipsub setup error: {}", s)))?;

            let ping =
                ping::Behaviour::new(ping::Config::new().with_interval(config.heartbeat_interval));

            let store = MemoryStore::new(local_peer_id);
            let mut kademlia_config = KademliaConfig::default();
            kademlia_config.disjoint_query_paths(true);
            kademlia_config.set_query_timeout(config.request_timeout);
            if let Some(replication_factor) =
                std::num::NonZeroUsize::new(config.kademlia_replication_factor)
            {
                kademlia_config.set_replication_factor(replication_factor);
            }
            let kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);

            let request_response = RequestResponseBehaviour::with_codec(
                MessageCodec,
                std::iter::once((MessageProtocol(), ProtocolSupport::Full)),
                libp2p::request_response::Config::default(),
            );

            let mdns_behaviour = if config.enable_mdns {
                Some(
                    libp2p::mdns::tokio::Behaviour::new(
                        libp2p::mdns::Config::default(),
                        local_peer_id,
                    )
                    .map_err(|e| MeshNetworkError::SetupError(format!("mDNS error: {}", e)))?,
                )
            } else {
                None
            };

            let behaviour = CombinedBehaviour {
                gossipsub,
                ping,
                kademlia,
                request_response,
                mdns: mdns_behaviour.into(),
            };
            let mut swarm = Swarm::new(
                transport,
                behaviour,
                local_peer_id,
                SwarmConfig::with_executor(Box::new(|fut| {
                    tokio::spawn(fut);
                })),
            );

            for addr in &config.listen_addresses {
                swarm
                    .listen_on(addr.clone())
                    .map_err(|e| MeshNetworkError::SetupError(format!("Listen error: {}", e)))?;
            }

            // Connect to bootstrap peers with improved error handling
            for (peer_id, addr) in &config.bootstrap_peers {
                info!("Attempting to dial bootstrap peer: {} at {}", peer_id, addr);
                match swarm.dial(addr.clone()) {
                    Ok(_) => {
                        info!("Successfully initiated dial to bootstrap peer: {}", peer_id);
                        swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(peer_id, addr.clone());
                    }
                    Err(e) => {
                        warn!("Failed to dial bootstrap peer {}: {}", peer_id, e);
                        // Continue trying other bootstrap peers instead of stopping
                    }
                }
            }

            let (cmd_tx, mut cmd_rx) = mpsc::channel(256);
            let stats = Arc::new(Mutex::new(EnhancedNetworkStats::new()));
            crate::metrics::PEER_COUNT_GAUGE.set(0);
            crate::metrics::KADEMLIA_PEERS_GAUGE.set(0);
            let stats_clone = stats.clone();

            let federations = Arc::new(Mutex::new(Vec::new()));
            let federations_clone = federations.clone();

            // Clone bootstrap_peers for use in the async task
            let bootstrap_peers_clone = config.bootstrap_peers.clone();
            let has_bootstrap_peers = !bootstrap_peers_clone.is_empty();

            // Store the listening addresses for the service
            let listening_addresses = Arc::new(Mutex::new(Vec::new()));
            let listening_addresses_clone = listening_addresses.clone();

            // Give the swarm a moment to initialize properly
            log::debug!("🔧 [LIBP2P] Allowing swarm to initialize...");
            tokio::time::sleep(Duration::from_millis(100)).await;

            let local_peer_id_inner = local_peer_id.clone();

            // Spawn the network event loop and hold the JoinHandle
            let event_loop_handle = task::spawn(async move {
                log::debug!("🔧 [LIBP2P] Starting libp2p event loop task");

                let topic = gossipsub::IdentTopic::new("icn-global");
                log::debug!("🔧 [LIBP2P] Created gossipsub topic: {}", topic.hash());

                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    log::error!("❌ [LIBP2P] Failed to subscribe to global topic: {:?}", e);
                } else {
                    log::info!("✅ [LIBP2P] Subscribed to global topic: {}", topic.hash());
                }

                if has_bootstrap_peers {
                    if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                        log::error!("❌ [LIBP2P] Initial Kademlia bootstrap failed: {:?}", e);
                    }
                } else {
                    log::debug!("🔧 [LIBP2P] No bootstrap peers configured");
                }

                let mut subscribers: Vec<mpsc::Sender<super::ProtocolMessage>> = Vec::new();
                let mut pending_kad_queries: HashMap<QueryId, PendingQuery> = HashMap::new();
                let mut bootstrap_tick = tokio::time::interval(config.bootstrap_interval);
                let mut discovery_tick = tokio::time::interval(config.peer_discovery_interval);

                log::debug!("🔧 [LIBP2P] Entering main event loop...");
                loop {
                    log::debug!(
                        "🔧 [LIBP2P] Event loop iteration starting - waiting for events..."
                    );

                    // Use timeout to prevent infinite hanging and ensure the swarm is driven
                    let timeout_duration = Duration::from_millis(100);

                    tokio::select! {
                        event = swarm.select_next_some() => {
                            log::debug!("🔧 [LIBP2P] Received swarm event: {:?}", std::mem::discriminant(&event));

                            // Update listening addresses when a new one is discovered
                            if let SwarmEvent::NewListenAddr { address, .. } = &event {
                                log::info!("✅ [LIBP2P] Listening on {}", address);
                                listening_addresses_clone.lock().unwrap().push(address.clone());
                            }
                            Self::handle_swarm_event(event, &stats_clone, &federations_clone, &mut subscribers, &mut swarm, &mut pending_kad_queries).await;
                            log::debug!("🔧 [LIBP2P] Finished handling swarm event");
                        }
                        Some(command) = cmd_rx.recv() => {
                            log::debug!("🔧 [LIBP2P] Received command: {:?}", std::mem::discriminant(&command));
                            match command {
                                Command::DiscoverPeers { target, rsp } => {
                                    let target = target.unwrap_or(local_peer_id_inner);
                                    let query_id = swarm.behaviour_mut().kademlia.get_closest_peers(target);
                                    pending_kad_queries.insert(query_id, PendingQuery::GetPeers(rsp));
                                }
                                Command::SendMessage { peer, message, rsp } => {
                                    let request_id = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_request(&peer, message.clone());

                                    let message_size =
                                        bincode::serialize(&message).map(|d| d.len()).unwrap_or(0) as u64;
                                    let mut stats_guard = stats_clone.lock().unwrap();
                                    stats_guard.bytes_sent += message_size;
                                    crate::metrics::BYTES_SENT_TOTAL.inc_by(message_size);
                                    stats_guard.messages_sent += 1;
                                    crate::metrics::MESSAGES_SENT_TOTAL.inc();
                                    let msg_type = message.payload.message_type().to_string();
                                    let type_stats = stats_guard.message_counts.entry(msg_type).or_default();
                                    type_stats.sent += 1;
                                    type_stats.bytes_sent += message_size;

                                    log::debug!("Sent message request: {:?}", request_id);
                                    let _ = rsp.send(Ok(()));
                                }
                                Command::Broadcast { data } => {
                                    log::debug!("🔧 [LIBP2P] Broadcasting {} bytes", data.len());
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data.clone()) {
                                        log::error!("❌ [LIBP2P] Failed to broadcast: {:?}", e);
                                    } else {
                                        log::debug!("✅ [LIBP2P] Broadcast successful");
                                        let mut stats_guard = stats_clone.lock().unwrap();
                                        stats_guard.bytes_sent += data.len() as u64;
                                        crate::metrics::BYTES_SENT_TOTAL.inc_by(data.len() as u64);
                                        stats_guard.messages_sent += 1;
                                        crate::metrics::MESSAGES_SENT_TOTAL.inc();

                                        // Update message type statistics for broadcasts
                                        if let Ok(network_msg) = bincode::deserialize::<super::ProtocolMessage>(&data) {
                                            let msg_type = network_msg.payload.message_type().to_string();
                                            let type_stats = stats_guard.message_counts.entry(msg_type).or_default();
                                            type_stats.sent += 1;
                                            type_stats.bytes_sent += data.len() as u64;
                                        }
                                    }
                                }
                                Command::Subscribe { rsp } => {
                                    log::debug!("🔧 [LIBP2P] Creating new subscription channel");
                                    let (tx, rx) = mpsc::channel(128);
                                    subscribers.push(tx);
                                    let _ = rsp.send(rx);
                                    log::debug!("✅ [LIBP2P] Subscription channel created, {} total subscribers", subscribers.len());
                                }
                                Command::GetStats { rsp } => {
                                    let kademlia_peer_count: usize = swarm
                                        .behaviour_mut()
                                        .kademlia
                                        .kbuckets()
                                        .map(|b| b.num_entries())
                                        .sum();
                                    let network_info = swarm.network_info();
                                    let mut stats_guard = stats_clone.lock().unwrap();
                                    stats_guard.update_kademlia_peers(kademlia_peer_count);

                                    let network_stats = super::NetworkStats {
                                        peer_count: network_info.num_peers(),
                                        bytes_sent: stats_guard.bytes_sent,
                                        bytes_received: stats_guard.bytes_received,
                                        messages_sent: stats_guard.messages_sent,
                                        messages_received: stats_guard.messages_received,
                                        failed_connections: stats_guard.failed_connections,
                                        avg_latency_ms: stats_guard.avg_latency(),
                                        min_latency_ms: stats_guard.min_latency_ms,
                                        max_latency_ms: stats_guard.max_latency_ms,
                                        last_latency_ms: stats_guard.last_latency_ms,
                                        kademlia_peers: stats_guard.kademlia_peers,
                                    };
                                    let _ = rsp.send(network_stats);
                                }
                                Command::GetKademliaRecord { key, rsp } => {
                                    let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                                    pending_kad_queries.insert(query_id, PendingQuery::GetRecord(rsp));
                                }
                                Command::PutKademliaRecord { key, value, rsp } => {
                                    let record = KademliaRecord { key, value, publisher: None, expires: None };
                                    match swarm.behaviour_mut().kademlia.put_record(record, Quorum::One) {
                                        Ok(query_id) => {
                                            pending_kad_queries.insert(query_id, PendingQuery::PutRecord(rsp));
                                        }
                                        Err(e) => {
                                            let _ = rsp.send(Err(MeshNetworkError::Libp2p(format!("put_record error: {}", e))));
                                        }
                                    }
                                }
                                Command::ConnectPeer { addr, rsp } => {
                                    match swarm.dial(addr.clone()) {
                                        Ok(_) => {
                                            log::info!("Dialing peer at {}", addr);
                                            let _ = rsp.send(Ok(()));
                                        }
                                        Err(e) => {
                                            let _ = rsp.send(Err(MeshNetworkError::Libp2p(format!("dial error: {}", e))));
                                        }
                                    }
                                }
                                Command::Shutdown => {
                                    log::info!("🔧 [LIBP2P] Shutdown command received");
                                    break;
                                }
                            }
                            log::debug!("🔧 [LIBP2P] Finished handling command");
                        }
                        _ = tokio::time::sleep(timeout_duration) => {
                            log::debug!("🔧 [LIBP2P] Event loop timeout - continuing to drive swarm");
                            // This timeout ensures the event loop continues running even if no events are available
                            // This is important for proper swarm operation and prevents hanging
                        }
                        _ = bootstrap_tick.tick(), if has_bootstrap_peers => {
                            for (peer, addr) in &bootstrap_peers_clone {
                                if !swarm.is_connected(peer) {
                                    if let Err(e) = swarm.dial(addr.clone()) {
                                        log::warn!("Failed to redial bootstrap {}: {}", peer, e);
                                    }
                                }
                            }
                            if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                                log::error!("❌ [LIBP2P] Periodic bootstrap error: {:?}", e);
                            }
                        }
                        _ = discovery_tick.tick() => {
                            let _ = swarm.behaviour_mut().kademlia.get_closest_peers(local_peer_id_inner);
                        }
                        else => {
                            log::debug!("🔧 [LIBP2P] Event loop terminating - no more events");
                            break;
                        }
                    }
                }
                log::info!("❌ [LIBP2P] Network event loop ended");
            });

            Ok(Self {
                local_peer_id,
                cmd_tx,
                config,
                listening_addresses,
                event_loop_handle,
                federations,
            })
        }

        async fn handle_swarm_event(
            event: SwarmEvent<CombinedBehaviourEvent>,
            stats: &Arc<Mutex<EnhancedNetworkStats>>,
            federations: &Arc<Mutex<Vec<icn_protocol::FederationInfo>>>,
            subscribers: &mut Vec<mpsc::Sender<super::ProtocolMessage>>,
            swarm: &mut Swarm<CombinedBehaviour>,
            pending_kad_queries: &mut HashMap<QueryId, PendingQuery>,
        ) {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(CombinedBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. },
                )) => {
                    let message_size = message.data.len() as u64;
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.bytes_received += message_size;
                        crate::metrics::BYTES_RECEIVED_TOTAL.inc_by(message_size);
                        stats_guard.messages_received += 1;
                        crate::metrics::MESSAGES_RECEIVED_TOTAL.inc();
                    }

                    if let Ok(network_msg) =
                        bincode::deserialize::<super::ProtocolMessage>(&message.data)
                    {
                        log::debug!(
                            "Received gossipsub message: {:?}",
                            network_msg.payload.message_type()
                        );

                        // Update message type statistics
                        let msg_type = network_msg.payload.message_type().to_string();
                        {
                            let mut stats_guard = stats.lock().unwrap();
                            let type_stats =
                                stats_guard.message_counts.entry(msg_type).or_default();
                            type_stats.received += 1;
                            type_stats.bytes_received += message_size;
                        }

                        // Distribute to subscribers
                        subscribers.retain_mut(|subscriber| {
                            subscriber.try_send(network_msg.clone()).is_ok()
                        });
                    }
                }
                SwarmEvent::Behaviour(CombinedBehaviourEvent::Ping(ping::Event {
                    result: Ok(rtt),
                    ..
                })) => {
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.record_latency(rtt);
                }
                SwarmEvent::Behaviour(CombinedBehaviourEvent::Kademlia(ev)) => match ev {
                    KademliaEvent::OutboundQueryProgressed { id, result, .. } => {
                        if let Some(query) = pending_kad_queries.remove(&id) {
                            match (query, result) {
                                (PendingQuery::GetRecord(tx), kad::QueryResult::GetRecord(res)) => {
                                    let send_res = match res {
                                        Ok(kad::GetRecordOk::FoundRecord(rec)) => {
                                            Ok(Some(rec.record))
                                        }
                                        Ok(_) => Ok(None),
                                        Err(e) => Err(MeshNetworkError::Libp2p(e.to_string())),
                                    };
                                    let _ = tx.send(send_res);
                                }
                                (PendingQuery::PutRecord(tx), kad::QueryResult::PutRecord(res)) => {
                                    let send_res = res
                                        .map(|_| ())
                                        .map_err(|e| MeshNetworkError::Libp2p(e.to_string()));
                                    let _ = tx.send(send_res);
                                }
                                (
                                    PendingQuery::GetPeers(tx),
                                    kad::QueryResult::GetClosestPeers(res),
                                ) => {
                                    let peers = match res {
                                        Ok(ok) => ok
                                            .peers
                                            .into_iter()
                                            .map(|p| super::PeerId(p.peer_id.to_string()))
                                            .collect(),
                                        Err(e) => {
                                            let _ = tx
                                                .send(Err(MeshNetworkError::Libp2p(e.to_string())));
                                            return;
                                        }
                                    };
                                    let _ = tx.send(Ok(peers));
                                }
                                (q, _) => {
                                    log::debug!("Received mismatched query result {:?}", q);
                                }
                            }
                        }
                    }
                    KademliaEvent::RoutingUpdated { .. } => {
                        let peer_count: usize = swarm
                            .behaviour_mut()
                            .kademlia
                            .kbuckets()
                            .map(|b| b.num_entries())
                            .sum();
                        stats.lock().unwrap().update_kademlia_peers(peer_count);
                    }
                    _ => {}
                },
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.peer_count += 1;
                        crate::metrics::PEER_COUNT_GAUGE.set(stats_guard.peer_count as i64);
                    }
                    log::info!("Connected to peer: {}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.peer_count = stats_guard.peer_count.saturating_sub(1);
                        crate::metrics::PEER_COUNT_GAUGE.set(stats_guard.peer_count as i64);
                    }
                    log::info!("Disconnected from peer: {}", peer_id);
                }
                SwarmEvent::OutgoingConnectionError { .. } => {
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.failed_connections += 1;
                }
                SwarmEvent::Behaviour(CombinedBehaviourEvent::RequestResponse(ev)) => {
                    use libp2p::request_response::{Event as ReqEvent, Message};
                    match ev {
                        ReqEvent::Message {
                            peer: _,
                            message,
                            connection_id: _,
                        } => match message {
                            Message::Request {
                                request, channel, ..
                            } => {
                                let message_size =
                                    bincode::serialize(&request).map(|d| d.len()).unwrap_or(0)
                                        as u64;
                                {
                                    let mut stats_guard = stats.lock().unwrap();
                                    stats_guard.bytes_received += message_size;
                                    crate::metrics::BYTES_RECEIVED_TOTAL.inc_by(message_size);
                                    stats_guard.messages_received += 1;
                                    crate::metrics::MESSAGES_RECEIVED_TOTAL.inc();
                                    let msg_type = request.payload.message_type().to_string();
                                    let type_stats =
                                        stats_guard.message_counts.entry(msg_type).or_default();
                                    type_stats.received += 1;
                                    type_stats.bytes_received += message_size;
                                }
                                subscribers.retain_mut(|sub| sub.try_send(request.clone()).is_ok());
                                let mut response = request.clone();
                                if let MessagePayload::FederationDiscoverRequest(_) =
                                    &request.payload
                                {
                                    let feds = federations.lock().unwrap().clone();
                                    response.payload = MessagePayload::FederationDiscoverResponse(
                                        icn_protocol::FederationDiscoverResponseMessage {
                                            federations: feds,
                                        },
                                    );
                                }
                                if let Err(e) = swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(channel, response)
                                {
                                    log::error!("Failed to send response: {:?}", e);
                                }
                            }
                            Message::Response { response, .. } => {
                                let message_size =
                                    bincode::serialize(&response).map(|d| d.len()).unwrap_or(0)
                                        as u64;
                                {
                                    let mut stats_guard = stats.lock().unwrap();
                                    stats_guard.bytes_received += message_size;
                                    crate::metrics::BYTES_RECEIVED_TOTAL.inc_by(message_size);
                                    stats_guard.messages_received += 1;
                                    crate::metrics::MESSAGES_RECEIVED_TOTAL.inc();
                                    let msg_type = response.payload.message_type().to_string();
                                    let type_stats =
                                        stats_guard.message_counts.entry(msg_type).or_default();
                                    type_stats.received += 1;
                                    type_stats.bytes_received += message_size;
                                }
                                subscribers
                                    .retain_mut(|sub| sub.try_send(response.clone()).is_ok());
                            }
                        },
                        ReqEvent::OutboundFailure { peer, error, .. } => {
                            log::warn!("Outbound request to {} failed: {:?}", peer, error);
                            stats.lock().unwrap().failed_connections += 1;
                        }
                        ReqEvent::InboundFailure { peer, error, .. } => {
                            log::warn!("Inbound request from {} failed: {:?}", peer, error);
                            stats.lock().unwrap().failed_connections += 1;
                        }
                        ReqEvent::ResponseSent { .. } => {}
                    }
                }
                SwarmEvent::Behaviour(CombinedBehaviourEvent::Mdns(event)) => match event {
                    libp2p::mdns::Event::Discovered(list) => {
                        for (peer, addr) in list {
                            swarm.behaviour_mut().kademlia.add_address(&peer, addr);
                        }
                    }
                    libp2p::mdns::Event::Expired(list) => {
                        for (peer, addr) in list {
                            swarm.behaviour_mut().kademlia.remove_address(&peer, &addr);
                        }
                    }
                },
                _ => {}
            }
        }

        /// Return the local peer's identifier.
        pub fn local_peer_id(&self) -> &Libp2pPeerId {
            &self.local_peer_id
        }

        /// Get the current listening addresses for this node
        pub fn listening_addresses(&self) -> Vec<Multiaddr> {
            self.listening_addresses.lock().unwrap().clone()
        }

        /// Gracefully shut down the networking task
        pub async fn shutdown(self) -> Result<(), MeshNetworkError> {
            if let Err(e) = self.cmd_tx.send(Command::Shutdown).await {
                return Err(MeshNetworkError::Libp2p(format!(
                    "shutdown send failed: {}",
                    e
                )));
            }
            self.event_loop_handle
                .await
                .map_err(|e| MeshNetworkError::Libp2p(format!("task join error: {}", e)))
        }

        /// Retrieve a record from the DHT
        pub async fn get_kademlia_record(
            &self,
            key: &str,
        ) -> Result<Option<KademliaRecord>, MeshNetworkError> {
            let (tx, rx) = oneshot::channel();
            let key = KademliaKey::new(&key.as_bytes());
            self.cmd_tx
                .send(Command::GetKademliaRecord { key, rsp: tx })
                .await
                .map_err(|e| MeshNetworkError::Libp2p(format!("command send failed: {}", e)))?;
            rx.await
                .map_err(|e| MeshNetworkError::Libp2p(format!("response dropped: {}", e)))?
        }

        /// Put a record into the DHT
        pub async fn put_kademlia_record(
            &self,
            key: &str,
            value: Vec<u8>,
        ) -> Result<(), MeshNetworkError> {
            let (tx, rx) = oneshot::channel();
            let key = KademliaKey::new(&key.as_bytes());
            self.cmd_tx
                .send(Command::PutKademliaRecord {
                    key,
                    value,
                    rsp: tx,
                })
                .await
                .map_err(|e| MeshNetworkError::Libp2p(format!("command send failed: {}", e)))?;
            rx.await
                .map_err(|e| MeshNetworkError::Libp2p(format!("response dropped: {}", e)))?
        }

        /// Attempt to connect to the given peer multiaddress.
        pub async fn connect_peer(&self, addr: Multiaddr) -> Result<(), MeshNetworkError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::ConnectPeer { addr, rsp: tx })
                .await
                .map_err(|e| MeshNetworkError::Libp2p(format!("command send failed: {}", e)))?;
            rx.await
                .map_err(|e| MeshNetworkError::Libp2p(format!("response dropped: {}", e)))?
        }
    }

    #[async_trait]
    impl super::NetworkService for Libp2pNetworkService {
        async fn discover_peers(
            &self,
            target_peer_id_str: Option<String>,
        ) -> Result<Vec<super::PeerId>, MeshNetworkError> {
            let target = match target_peer_id_str {
                Some(id_str) => Some(Libp2pPeerId::from_str(&id_str).map_err(|e| {
                    MeshNetworkError::InvalidInput(format!("Invalid peer ID: {}", e))
                })?),
                None => None,
            };

            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let target = target.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    cmd.send(Command::DiscoverPeers { target, rsp: tx })
                        .await
                        .map_err(|e| {
                            MeshNetworkError::Libp2p(format!("Command send failed: {}", e))
                        })?;
                    rx.await
                        .map_err(|e| MeshNetworkError::Libp2p(format!("Response dropped: {}", e)))
                }
            })
            .await?
        }

        async fn send_message(
            &self,
            peer: &super::PeerId,
            message: super::ProtocolMessage,
        ) -> Result<(), MeshNetworkError> {
            let libp2p_peer = Libp2pPeerId::from_str(&peer.0)
                .map_err(|e| MeshNetworkError::InvalidInput(format!("Invalid peer ID: {}", e)))?;

            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let peer_id = libp2p_peer.clone();
                let msg = message.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    cmd.send(Command::SendMessage {
                        peer: peer_id,
                        message: msg,
                        rsp: tx,
                    })
                    .await
                    .map_err(|e| {
                        MeshNetworkError::SendFailure(format!("Command send failed: {}", e))
                    })?;
                    rx.await.map_err(|e| {
                        MeshNetworkError::SendFailure(format!("Response dropped: {}", e))
                    })
                }
            })
            .await?
        }

        async fn broadcast_message(
            &self,
            message: super::ProtocolMessage,
        ) -> Result<(), MeshNetworkError> {
            let data = bincode::serialize(&message)
                .map_err(|e| MeshNetworkError::MessageDecodeFailed(e.to_string()))?;

            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let bytes = data.clone();
                async move {
                    cmd.send(Command::Broadcast { data: bytes })
                        .await
                        .map_err(|e| {
                            MeshNetworkError::SendFailure(format!("Broadcast failed: {}", e))
                        })
                }
            })
            .await
        }

        async fn subscribe(&self) -> Result<Receiver<super::ProtocolMessage>, MeshNetworkError> {
            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    cmd.send(Command::Subscribe { rsp: tx })
                        .await
                        .map_err(|e| {
                            MeshNetworkError::Libp2p(format!("Subscribe failed: {}", e))
                        })?;
                    rx.await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Subscribe response failed: {}", e))
                    })
                }
            })
            .await
        }

        async fn discover_federations(
            &self,
        ) -> Result<Vec<icn_protocol::FederationInfo>, MeshNetworkError> {
            let peers = self.discover_peers(None).await?;
            let mut sub = self.subscribe().await?;
            for peer in &peers {
                let req = icn_protocol::ProtocolMessage::new(
                    icn_protocol::MessagePayload::FederationDiscoverRequest(
                        icn_protocol::FederationDiscoverRequestMessage,
                    ),
                    Did::default(),
                    None,
                );
                let _ = self.send_message(peer, req).await;
            }
            use tokio::time::{timeout, Duration};
            let mut results = Vec::new();
            for _ in 0..peers.len() {
                if let Ok(Some(msg)) = timeout(Duration::from_secs(5), sub.recv()).await {
                    if let icn_protocol::MessagePayload::FederationDiscoverResponse(resp) =
                        msg.payload
                    {
                        results.extend(resp.federations);
                    }
                }
            }
            Ok(results)
        }

        async fn get_network_stats(&self) -> Result<super::NetworkStats, MeshNetworkError> {
            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    cmd.send(Command::GetStats { rsp: tx }).await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Get stats failed: {}", e))
                    })?;
                    rx.await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Stats response failed: {}", e))
                    })
                }
            })
            .await
        }

        async fn store_record(&self, key: String, value: Vec<u8>) -> Result<(), MeshNetworkError> {
            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let key_bytes = key.clone().into_bytes();
                let val = value.clone();
                let key_clone2 = key.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    let record_key = KademliaKey::new(&key_bytes);
                    cmd.send(Command::PutKademliaRecord {
                        key: record_key,
                        value: val.clone(),
                        rsp: tx,
                    })
                    .await
                    .map_err(|e| MeshNetworkError::Libp2p(format!("Put record failed: {}", e)))?;
                    let _ = rx.await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Put record response failed: {}", e))
                    })?;
                    if key_clone2.starts_with(FEDERATION_INFO_PREFIX) {
                        if let Ok(info) = bincode::deserialize::<icn_protocol::FederationInfo>(&val)
                        {
                            let mut feds = self.federations.lock().unwrap();
                            if !feds.iter().any(|i| i.federation_id == info.federation_id) {
                                feds.push(info);
                            }
                        }
                    }
                    Ok(())
                }
            })
            .await
        }

        async fn get_record(&self, key: String) -> Result<Option<Vec<u8>>, MeshNetworkError> {
            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let key_bytes = key.clone().into_bytes();
                async move {
                    let (tx, rx) = oneshot::channel();
                    let record_key = KademliaKey::new(&key_bytes);
                    cmd.send(Command::GetKademliaRecord {
                        key: record_key,
                        rsp: tx,
                    })
                    .await
                    .map_err(|e| MeshNetworkError::Libp2p(format!("Get record failed: {}", e)))?;
                    let record_opt = rx.await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Get record response failed: {}", e))
                    })??;
                    Ok(record_opt.map(|rec| rec.value))
                }
            })
            .await
        }

        async fn connect_peer(&self, addr: Multiaddr) -> Result<(), MeshNetworkError> {
            with_resilience(|| {
                let cmd = self.cmd_tx.clone();
                let addr = addr.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    cmd.send(Command::ConnectPeer { addr, rsp: tx })
                        .await
                        .map_err(|e| {
                            MeshNetworkError::Libp2p(format!("Connect send failed: {}", e))
                        })?;
                    let _ = rx.await.map_err(|e| {
                        MeshNetworkError::Libp2p(format!("Connect response failed: {}", e))
                    })?;
                    Ok(())
                }
            })
            .await
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}
