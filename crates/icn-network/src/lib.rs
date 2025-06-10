#![doc = include_str!("../README.md")]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_constructed_unit_structs)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::clone_on_copy)]

//! # ICN Network Crate - Production-Ready P2P Networking
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! using libp2p for distributed communication between ICN nodes.

pub mod error;
pub use error::MeshNetworkError;

use async_trait::async_trait;
use downcast_rs::{impl_downcast, DowncastSync};
use icn_common::{Cid, CommonError, DagBlock, Did, NodeInfo};
use icn_identity::ExecutionReceipt;
use icn_mesh::{ActualMeshJob as Job, JobId, MeshJobBid as Bid};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
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

// --- Core Types ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn from_string(s: String) -> Self {
        PeerId(s)
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    AnnounceBlock(DagBlock),
    RequestBlock(Cid),
    GossipSub(String, Vec<u8>),
    FederationSyncRequest(Did),
    MeshJobAnnouncement(Job),
    BidSubmission(Bid),
    JobAssignmentNotification(JobId, Did),
    SubmitReceipt(ExecutionReceipt),
}

impl NetworkMessage {
    pub fn message_type(&self) -> &'static str {
        match self {
            NetworkMessage::AnnounceBlock(_) => "AnnounceBlock",
            NetworkMessage::RequestBlock(_) => "RequestBlock",
            NetworkMessage::GossipSub(_, _) => "GossipSub",
            NetworkMessage::FederationSyncRequest(_) => "FederationSyncRequest",
            NetworkMessage::MeshJobAnnouncement(_) => "MeshJobAnnouncement",
            NetworkMessage::BidSubmission(_) => "BidSubmission",
            NetworkMessage::JobAssignmentNotification(_, _) => "JobAssignmentNotification",
            NetworkMessage::SubmitReceipt(_) => "SubmitReceipt",
        }
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
    ) -> Result<Vec<PeerId>, CommonError>;
    async fn send_message(&self, peer: &PeerId, message: NetworkMessage)
        -> Result<(), CommonError>;
    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError>;
    async fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError>;
    async fn get_network_stats(&self) -> Result<NetworkStats, CommonError>;
    fn as_any(&self) -> &dyn Any;
}
impl_downcast!(sync NetworkService);

/// Stub implementation for testing.
#[derive(Default, Debug)]
pub struct StubNetworkService;

#[async_trait]
impl NetworkService for StubNetworkService {
    async fn discover_peers(
        &self,
        target_peer_id_str: Option<String>,
    ) -> Result<Vec<PeerId>, CommonError> {
        println!(
            "[StubNetworkService] Discovering peers (target: {:?})... returning mock peers.",
            target_peer_id_str
        );
        Ok(vec![
            PeerId("mock_peer_1".to_string()),
            PeerId("mock_peer_2".to_string()),
        ])
    }

    async fn send_message(
        &self,
        peer: &PeerId,
        message: NetworkMessage,
    ) -> Result<(), CommonError> {
        println!(
            "[StubNetworkService] Sending message to peer {:?}: {:?}",
            peer, message
        );
        if peer.0 == "error_peer" {
            return Err(CommonError::MessageSendError(format!(
                "Failed to send message to peer: {}",
                peer.0
            )));
        }
        if peer.0 == "unknown_peer_id" {
            return Err(CommonError::PeerNotFound(format!(
                "Peer with ID {} not found.",
                peer.0
            )));
        }
        Ok(())
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Broadcasting message: {:?}", message);
        if let NetworkMessage::GossipSub(topic, _) = &message {
            if topic == "system_critical_error_topic" {
                return Err(CommonError::NetworkUnhealthy(
                    "Broadcast failed: system critical topic is currently down.".to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError> {
        println!("[StubNetworkService] Subscribing to messages... returning an empty channel.");
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }

    async fn get_network_stats(&self) -> Result<NetworkStats, CommonError> {
        Ok(NetworkStats::default())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Placeholder function for testing network operations.
pub async fn send_network_ping(info: &NodeInfo, target_peer: &str) -> Result<String, CommonError> {
    let service = StubNetworkService::default();
    let _ = service
        .send_message(
            &PeerId(target_peer.to_string()),
            NetworkMessage::GossipSub("ping_topic".to_string(), vec![1, 2, 3]),
        )
        .await?;
    Ok(format!(
        "Sent (stubbed) ping to {} from node: {} (v{})",
        target_peer, info.name, info.version
    ))
}

#[cfg(all(test, feature = "experimental-libp2p"))]
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
#[cfg(feature = "experimental-libp2p")]
pub mod libp2p_service {
    use super::*;
    use libp2p::futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
    use libp2p::{
        core::upgrade,
        dns, gossipsub, identity,
        kad::{Record as KademliaRecord, RecordKey as KademliaKey},
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

    #[derive(Debug, Clone)]
    pub struct NetworkConfig {
        pub max_peers: usize,
        pub max_peers_per_ip: usize,
        pub connection_timeout: Duration,
        pub request_timeout: Duration,
        pub heartbeat_interval: Duration,
        pub bootstrap_interval: Duration,
        pub enable_mdns: bool,
        pub kademlia_replication_factor: usize,
        pub bootstrap_peers: Vec<(Libp2pPeerId, Multiaddr)>,
    }

    impl Default for NetworkConfig {
        fn default() -> Self {
            Self {
                max_peers: 1000,
                max_peers_per_ip: 5,
                connection_timeout: Duration::from_secs(30),
                request_timeout: Duration::from_secs(10),
                heartbeat_interval: Duration::from_secs(15),
                bootstrap_interval: Duration::from_secs(300),
                enable_mdns: false,
                kademlia_replication_factor: 20,
                bootstrap_peers: Vec::new(),
            }
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
    }

    impl EnhancedNetworkStats {
        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        fn update_kademlia_peers(&mut self, count: usize) {
            self.kademlia_peers = count;
        }
    }

    #[derive(Debug, Default, Clone)]
    struct MessageTypeStats {
        sent: u64,
        received: u64,
        bytes_sent: u64,
        bytes_received: u64,
    }

    #[derive(Debug)]
    pub struct Libp2pNetworkService {
        local_peer_id: Libp2pPeerId,
        cmd_tx: mpsc::Sender<Command>,
        config: NetworkConfig,
        listening_addresses: Arc<Mutex<Vec<Multiaddr>>>,
        _event_loop_handle: task::JoinHandle<()>, // Hold the handle to prevent task cancellation
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
                _event_loop_handle: task::spawn(async {}), // Dummy handle for clones
            }
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    enum Command {
        DiscoverPeers {
            target: Option<Libp2pPeerId>,
            rsp: oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>,
        },
        SendMessage {
            peer: Libp2pPeerId,
            message: super::NetworkMessage,
            rsp: oneshot::Sender<Result<(), CommonError>>,
        },
        Broadcast {
            data: Vec<u8>,
        },
        Subscribe {
            rsp: oneshot::Sender<mpsc::Receiver<super::NetworkMessage>>,
        },
        GetStats {
            rsp: oneshot::Sender<super::NetworkStats>,
        },
        GetKademliaRecord {
            key: KademliaKey,
            rsp: oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>,
        },
        PutKademliaRecord {
            key: KademliaKey,
            value: Vec<u8>,
            rsp: oneshot::Sender<Result<(), CommonError>>,
        },
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
        type Request = super::NetworkMessage;
        type Response = super::NetworkMessage;

        async fn read_request<T>(
            &mut self,
            _: &MessageProtocol,
            io: &mut T,
        ) -> std::io::Result<super::NetworkMessage>
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
        ) -> std::io::Result<super::NetworkMessage>
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
            req: super::NetworkMessage,
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
            res: super::NetworkMessage,
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

    #[derive(NetworkBehaviour)]
    pub struct CombinedBehaviour {
        gossipsub: gossipsub::Behaviour,
        ping: ping::Behaviour,
        // Temporarily disable Kademlia to test if it's causing the hang
        // kademlia: KademliaBehaviour<MemoryStore>,
        request_response: RequestResponseBehaviour<MessageCodec>,
    }

    // CombinedEvent removed - using auto-generated CombinedBehaviourEvent instead

    impl Libp2pNetworkService {
        pub async fn new(config: NetworkConfig) -> Result<Self, CommonError> {
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = Libp2pPeerId::from(local_key.public());

            let transport =
                dns::tokio::Transport::system(tcp::tokio::Transport::new(
                    tcp::Config::default().nodelay(true),
                ))
                .map_err(|e| CommonError::NetworkSetupError(format!("DNS config error: {}", e)))?
                .upgrade(upgrade::Version::V1Lazy)
                .authenticate(noise::Config::new(&local_key).map_err(|e| {
                    CommonError::NetworkSetupError(format!("Noise auth error: {}", e))
                })?)
                .multiplex(yamux::Config::default())
                .timeout(config.connection_timeout)
                .boxed();

            let gossipsub_config = gossipsub::Config::default();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            )
            .map_err(|s| CommonError::NetworkSetupError(format!("Gossipsub setup error: {}", s)))?;

            let ping =
                ping::Behaviour::new(ping::Config::new().with_interval(config.heartbeat_interval));

            // Temporarily disable Kademlia to test if it's causing the hang
            // let store = MemoryStore::new(local_peer_id);
            // let mut kademlia_config = KademliaConfig::default();
            // kademlia_config.disjoint_query_paths(true);
            // if let Some(replication_factor) = NonZero::new(config.kademlia_replication_factor) {
            //     kademlia_config.set_replication_factor(replication_factor);
            // }
            // let kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);

            let request_response = RequestResponseBehaviour::with_codec(
                MessageCodec,
                std::iter::once((MessageProtocol(), ProtocolSupport::Full)),
                libp2p::request_response::Config::default(),
            );

            let behaviour = CombinedBehaviour {
                gossipsub,
                ping,
                request_response,
            };
            let mut swarm = Swarm::new(
                transport,
                behaviour,
                local_peer_id,
                SwarmConfig::with_tokio_executor(),
            );

            swarm
                .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                .map_err(|e| CommonError::NetworkSetupError(format!("Listen error: {}", e)))?;

            // Connect to bootstrap peers with improved error handling (Kademlia disabled for testing)
            for (peer_id, addr) in &config.bootstrap_peers {
                info!("Attempting to dial bootstrap peer: {} at {}", peer_id, addr);
                match swarm.dial(addr.clone()) {
                    Ok(_) => {
                        info!("Successfully initiated dial to bootstrap peer: {}", peer_id);
                        // Kademlia disabled: swarm.behaviour_mut().kademlia.add_address(peer_id, addr.clone());
                    }
                    Err(e) => {
                        warn!("Failed to dial bootstrap peer {}: {}", peer_id, e);
                        // Continue trying other bootstrap peers instead of stopping
                    }
                }
            }

            let (cmd_tx, mut cmd_rx) = mpsc::channel(256);
            let stats = Arc::new(Mutex::new(EnhancedNetworkStats::new()));
            let stats_clone = stats.clone();

            // Clone bootstrap_peers for use in the async task
            let has_bootstrap_peers = !config.bootstrap_peers.is_empty();

            // Store the listening addresses for the service
            let listening_addresses = Arc::new(Mutex::new(Vec::new()));
            let listening_addresses_clone = listening_addresses.clone();

            // Give the swarm a moment to initialize properly
            log::debug!("🔧 [LIBP2P] Allowing swarm to initialize...");
            tokio::time::sleep(Duration::from_millis(100)).await;

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

                // Kademlia bootstrap disabled for testing
                if has_bootstrap_peers {
                    log::debug!(
                        "🔧 [LIBP2P] Has bootstrap peers, but Kademlia disabled for testing"
                    );
                } else {
                    log::debug!("🔧 [LIBP2P] No bootstrap peers configured");
                }

                let mut subscribers: Vec<mpsc::Sender<super::NetworkMessage>> = Vec::new();
                // Kademlia queries disabled for testing
                // let mut pending_kad_queries: HashMap<QueryId, oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>> = HashMap::new();

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
                            Self::handle_swarm_event(event, &stats_clone, &mut subscribers).await;
                            log::debug!("🔧 [LIBP2P] Finished handling swarm event");
                        }
                        Some(command) = cmd_rx.recv() => {
                            log::debug!("🔧 [LIBP2P] Received command: {:?}", std::mem::discriminant(&command));
                            match command {
                                Command::DiscoverPeers { target: _target, rsp } => {
                                    log::debug!("Peer discovery disabled (Kademlia disabled for testing)");
                                    // Return empty peer list since Kademlia is disabled
                                    let peers: Vec<super::PeerId> = Vec::new();
                                    let _ = rsp.send(Ok(peers));
                                }
                                Command::SendMessage { peer, message, rsp } => {
                                    let request_id = swarm.behaviour_mut().request_response.send_request(&peer, message.clone());
                                    stats_clone.lock().unwrap().messages_sent += 1;
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
                                        stats_guard.messages_sent += 1;

                                        // Update message type statistics for broadcasts
                                        if let Ok(network_msg) = bincode::deserialize::<super::NetworkMessage>(&data) {
                                            let msg_type = network_msg.message_type().to_string();
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
                                    let network_info = swarm.network_info();
                                    let mut stats_guard = stats_clone.lock().unwrap();

                                    // Kademlia disabled for testing, so no Kademlia peers
                                    let kademlia_peer_count = 0;
                                    stats_guard.update_kademlia_peers(kademlia_peer_count);

                                    let network_stats = super::NetworkStats {
                                        peer_count: network_info.num_peers(),
                                        bytes_sent: stats_guard.bytes_sent,
                                        bytes_received: stats_guard.bytes_received,
                                        messages_sent: stats_guard.messages_sent,
                                        messages_received: stats_guard.messages_received,
                                        failed_connections: stats_guard.failed_connections,
                                        avg_latency_ms: None, // TODO: Implement latency tracking
                                        kademlia_peers: stats_guard.kademlia_peers,
                                    };
                                    let _ = rsp.send(network_stats);
                                }
                                Command::GetKademliaRecord { key: _, rsp } => {
                                    log::debug!("Kademlia get record disabled for testing");
                                    let _ = rsp.send(Err(CommonError::NetworkError("Kademlia disabled for testing".to_string())));
                                }
                                Command::PutKademliaRecord { key: _, value: _, rsp } => {
                                    log::debug!("Kademlia put record disabled for testing");
                                    let _ = rsp.send(Err(CommonError::NetworkError("Kademlia disabled for testing".to_string())));
                                }
                            }
                            log::debug!("🔧 [LIBP2P] Finished handling command");
                        }
                        _ = tokio::time::sleep(timeout_duration) => {
                            log::debug!("🔧 [LIBP2P] Event loop timeout - continuing to drive swarm");
                            // This timeout ensures the event loop continues running even if no events are available
                            // This is important for proper swarm operation and prevents hanging
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
                _event_loop_handle: event_loop_handle,
            })
        }

        async fn handle_swarm_event(
            event: SwarmEvent<CombinedBehaviourEvent>,
            stats: &Arc<Mutex<EnhancedNetworkStats>>,
            subscribers: &mut Vec<mpsc::Sender<super::NetworkMessage>>,
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
                        stats_guard.messages_received += 1;
                    }

                    if let Ok(network_msg) =
                        bincode::deserialize::<super::NetworkMessage>(&message.data)
                    {
                        log::debug!(
                            "Received gossipsub message: {:?}",
                            network_msg.message_type()
                        );

                        // Update message type statistics
                        let msg_type = network_msg.message_type().to_string();
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
                // Kademlia events disabled for testing
                // SwarmEvent::Behaviour(CombinedEvent::Kademlia(...)) => { ... }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.peer_count += 1;
                    }
                    log::info!("Connected to peer: {}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.peer_count = stats_guard.peer_count.saturating_sub(1);
                    }
                    log::info!("Disconnected from peer: {}", peer_id);
                }
                SwarmEvent::OutgoingConnectionError { .. } => {
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.failed_connections += 1;
                }
                _ => {}
            }
        }

        pub fn local_peer_id(&self) -> &Libp2pPeerId {
            &self.local_peer_id
        }

        /// Get the current listening addresses for this node
        pub fn listening_addresses(&self) -> Vec<Multiaddr> {
            self.listening_addresses.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl super::NetworkService for Libp2pNetworkService {
        async fn discover_peers(
            &self,
            target_peer_id_str: Option<String>,
        ) -> Result<Vec<super::PeerId>, CommonError> {
            let target = match target_peer_id_str {
                Some(id_str) => Some(Libp2pPeerId::from_str(&id_str).map_err(|e| {
                    CommonError::InvalidInputError(format!("Invalid peer ID: {}", e))
                })?),
                None => None,
            };

            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::DiscoverPeers { target, rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkError(format!("Command send failed: {}", e)))?;
            rx.await
                .map_err(|e| CommonError::NetworkError(format!("Response dropped: {}", e)))?
        }

        async fn send_message(
            &self,
            peer: &super::PeerId,
            message: super::NetworkMessage,
        ) -> Result<(), CommonError> {
            let libp2p_peer = Libp2pPeerId::from_str(&peer.0)
                .map_err(|e| CommonError::InvalidInputError(format!("Invalid peer ID: {}", e)))?;

            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::SendMessage {
                    peer: libp2p_peer,
                    message,
                    rsp: tx,
                })
                .await
                .map_err(|e| {
                    CommonError::MessageSendError(format!("Command send failed: {}", e))
                })?;
            rx.await
                .map_err(|e| CommonError::MessageSendError(format!("Response dropped: {}", e)))?
        }

        async fn broadcast_message(
            &self,
            message: super::NetworkMessage,
        ) -> Result<(), CommonError> {
            let data = bincode::serialize(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;

            self.cmd_tx
                .send(Command::Broadcast { data })
                .await
                .map_err(|e| CommonError::MessageSendError(format!("Broadcast failed: {}", e)))
        }

        async fn subscribe(&self) -> Result<Receiver<super::NetworkMessage>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::Subscribe { rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkError(format!("Subscribe failed: {}", e)))?;

            rx.await
                .map_err(|e| CommonError::NetworkError(format!("Subscribe response failed: {}", e)))
        }

        async fn get_network_stats(&self) -> Result<super::NetworkStats, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::GetStats { rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkError(format!("Get stats failed: {}", e)))?;
            rx.await
                .map_err(|e| CommonError::NetworkError(format!("Stats response failed: {}", e)))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}
