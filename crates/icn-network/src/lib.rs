#![doc = include_str!("../README.md")]

//! # ICN Network Crate - Production-Ready P2P Networking
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! using libp2p for distributed communication between ICN nodes.

pub mod error;
pub use error::MeshNetworkError;

use icn_common::{NodeInfo, CommonError, DagBlock, Cid, Did};
use icn_mesh::{ActualMeshJob as Job, MeshJobBid as Bid, JobId};
use icn_identity::ExecutionReceipt;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Receiver;
use std::fmt::Debug;
use async_trait::async_trait;
use downcast_rs::{impl_downcast, DowncastSync};
use std::any::Any;
use std::time::{Duration, Instant};
use std::collections::HashMap;

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
    async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<PeerId>, CommonError>;
    async fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError>;
    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError>;
    fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError>;
    async fn get_network_stats(&self) -> Result<NetworkStats, CommonError>;
    fn as_any(&self) -> &dyn Any;
}
impl_downcast!(sync NetworkService);

/// Stub implementation for testing.
#[derive(Default, Debug)]
pub struct StubNetworkService;

#[async_trait]
impl NetworkService for StubNetworkService {
    async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<PeerId>, CommonError> {
        println!("[StubNetworkService] Discovering peers (target: {:?})... returning mock peers.", target_peer_id_str);
        Ok(vec![PeerId("mock_peer_1".to_string()), PeerId("mock_peer_2".to_string())])
    }

    async fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Sending message to peer {:?}: {:?}", peer, message);
        if peer.0 == "error_peer" {
            return Err(CommonError::MessageSendError(format!("Failed to send message to peer: {}", peer.0)));
        }
        if peer.0 == "unknown_peer_id" {
            return Err(CommonError::PeerNotFound(format!("Peer with ID {} not found.", peer.0)));
        }
        Ok(())
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Broadcasting message: {:?}", message);
        if let NetworkMessage::GossipSub(topic, _) = &message {
            if topic == "system_critical_error_topic" {
                return Err(CommonError::NetworkUnhealthy("Broadcast failed: system critical topic is currently down.".to_string()));
            }
        }
        Ok(())
    }

    fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError> {
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
    let _ = service.send_message(&PeerId(target_peer.to_string()), NetworkMessage::GossipSub("ping_topic".to_string(), vec![1,2,3])).await?;
    Ok(format!("Sent (stubbed) ping to {} from node: {} (v{})", target_peer, info.name, info.version))
}

#[cfg(all(test, feature = "experimental-libp2p"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_network_service_basic() {
        let service = StubNetworkService::default();
        let peers = service.discover_peers(Some("/ip4/127.0.0.1/tcp/12345".to_string())).await.unwrap();
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
    use libp2p::futures::{StreamExt, AsyncReadExt, AsyncWriteExt};
    use libp2p::{
        core::upgrade, gossipsub, identity, noise, ping,
        swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
        tcp, yamux, PeerId as Libp2pPeerId, Transport, dns,
        kad::{Record as KademliaRecord, RecordKey as KademliaKey, GetRecordOk, 
              store::MemoryStore, Behaviour as KademliaBehaviour, Config as KademliaConfig, 
              Event as KademliaEvent, QueryResult as KademliaQueryResult, Quorum, QueryId},
        request_response::{Behaviour as RequestResponseBehaviour, Codec as RequestResponseCodec, 
                          Event as RequestResponseEvent, ProtocolSupport},
    };
    use tokio::{sync::{mpsc, oneshot}, task};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::num::NonZero;
    use bincode;
    use log;

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
        last_bootstrap: Option<Instant>,
        kademlia_peers: usize,
    }

    impl EnhancedNetworkStats {
        fn new() -> Self {
            Self {
                last_bootstrap: Some(Instant::now()),
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

    #[derive(Clone, Debug)]
    pub struct Libp2pNetworkService {
        local_peer_id: Libp2pPeerId,
        cmd_tx: mpsc::Sender<Command>,
        config: NetworkConfig,
    }

    #[derive(Debug)]
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
        Broadcast { data: Vec<u8> },
        Subscribe { rsp: oneshot::Sender<mpsc::Receiver<super::NetworkMessage>> },
        GetStats { rsp: oneshot::Sender<super::NetworkStats> },
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
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid protocol")),
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

        async fn read_request<T>(&mut self, _: &MessageProtocol, io: &mut T) -> std::io::Result<super::NetworkMessage>
        where T: libp2p::futures::AsyncRead + Unpin + Send
        {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            bincode::deserialize(&buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }

        async fn read_response<T>(&mut self, _: &MessageProtocol, io: &mut T) -> std::io::Result<super::NetworkMessage>
        where T: libp2p::futures::AsyncRead + Unpin + Send
        {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            bincode::deserialize(&buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }

        async fn write_request<T>(&mut self, _: &MessageProtocol, io: &mut T, req: super::NetworkMessage) -> std::io::Result<()>
        where T: libp2p::futures::AsyncWrite + Unpin + Send
        {
            let data = bincode::serialize(&req).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await
        }

        async fn write_response<T>(&mut self, _: &MessageProtocol, io: &mut T, res: super::NetworkMessage) -> std::io::Result<()>
        where T: libp2p::futures::AsyncWrite + Unpin + Send
        {
            let data = bincode::serialize(&res).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await
        }
    }

    // --- Network Behaviour Definition ---
    
    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "CombinedEvent")]
    pub struct CombinedBehaviour {
        gossipsub: gossipsub::Behaviour,
        ping: ping::Behaviour,
        kademlia: KademliaBehaviour<MemoryStore>,
        request_response: RequestResponseBehaviour<MessageCodec>,
    }

    #[derive(Debug)]
    pub enum CombinedEvent {
        Gossipsub(gossipsub::Event),
        Ping(ping::Event),
        Kademlia(KademliaEvent),
        RequestResponse(RequestResponseEvent<super::NetworkMessage, super::NetworkMessage>),
    }

    impl From<gossipsub::Event> for CombinedEvent {
        fn from(e: gossipsub::Event) -> Self { CombinedEvent::Gossipsub(e) }
    }
    impl From<ping::Event> for CombinedEvent {
        fn from(e: ping::Event) -> Self { CombinedEvent::Ping(e) }
    }
    impl From<KademliaEvent> for CombinedEvent {
        fn from(e: KademliaEvent) -> Self { CombinedEvent::Kademlia(e) }
    }
    impl From<RequestResponseEvent<super::NetworkMessage, super::NetworkMessage>> for CombinedEvent {
        fn from(e: RequestResponseEvent<super::NetworkMessage, super::NetworkMessage>) -> Self { CombinedEvent::RequestResponse(e) }
    }

    impl Libp2pNetworkService {
        pub async fn new(config: NetworkConfig) -> Result<Self, CommonError> {
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = Libp2pPeerId::from(local_key.public());

            let transport = dns::tokio::Transport::system(
                tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            )
            .map_err(|e| CommonError::NetworkSetupError(format!("DNS config error: {}", e)))?
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key).map_err(|e| CommonError::NetworkSetupError(format!("Noise auth error: {}", e)))?)
            .multiplex(yamux::Config::default())
            .timeout(config.connection_timeout)
            .boxed();

            let gossipsub_config = gossipsub::Config::default();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config
            ).map_err(|s| CommonError::NetworkSetupError(format!("Gossipsub setup error: {}", s)))?;

            let ping = ping::Behaviour::new(ping::Config::new().with_interval(config.heartbeat_interval));

            let store = MemoryStore::new(local_peer_id);
            let mut kademlia_config = KademliaConfig::default();
            kademlia_config.disjoint_query_paths(true);
            if let Some(replication_factor) = NonZero::new(config.kademlia_replication_factor) {
                kademlia_config.set_replication_factor(replication_factor);
            }
            let kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);

            let request_response = RequestResponseBehaviour::with_codec(
                MessageCodec,
                std::iter::once((MessageProtocol(), ProtocolSupport::Full)),
                libp2p::request_response::Config::default(),
            );

            let behaviour = CombinedBehaviour { gossipsub, ping, kademlia, request_response };
            let mut swarm = Swarm::new(transport, behaviour, local_peer_id, SwarmConfig::with_tokio_executor());

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                .map_err(|e| CommonError::NetworkSetupError(format!("Listen error: {}", e)))?;

            let (cmd_tx, mut cmd_rx) = mpsc::channel(256);
            let stats = Arc::new(Mutex::new(EnhancedNetworkStats::new()));
            let stats_clone = stats.clone();

            // Spawn the network event loop
            task::spawn(async move {
                let topic = gossipsub::IdentTopic::new("icn-global");
                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    log::error!("Failed to subscribe to global topic: {:?}", e);
                } else {
                    log::info!("Subscribed to global topic: {}", topic.hash());
                }

                let mut subscribers: Vec<mpsc::Sender<super::NetworkMessage>> = Vec::new();
                let mut pending_kad_queries: HashMap<QueryId, oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>> = HashMap::new();

                loop {
                    tokio::select! {
                        event = swarm.select_next_some() => {
                            Self::handle_swarm_event(event, &stats_clone, &mut subscribers, &mut pending_kad_queries).await;
                        }
                        Some(command) = cmd_rx.recv() => {
                            match command {
                                Command::DiscoverPeers { target, rsp } => {
                                    let query_id = match target {
                                        Some(peer) => swarm.behaviour_mut().kademlia.get_closest_peers(peer),
                                        None => swarm.behaviour_mut().kademlia.get_closest_peers(Libp2pPeerId::random()),
                                    };
                                    log::debug!("Started peer discovery query: {:?}", query_id);
                                    
                                    // Extract current peers from Kademlia routing table
                                    let peers: Vec<super::PeerId> = Vec::new(); // Simplified for now
                                    let _ = rsp.send(Ok(peers));
                                }
                                Command::SendMessage { peer, message, rsp } => {
                                    let request_id = swarm.behaviour_mut().request_response.send_request(&peer, message.clone());
                                    stats_clone.lock().unwrap().messages_sent += 1;
                                    log::debug!("Sent message request: {:?}", request_id);
                                    let _ = rsp.send(Ok(()));
                                }
                                Command::Broadcast { data } => {
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data.clone()) {
                                        log::error!("Failed to broadcast: {:?}", e);
                                    } else {
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
                                    let (tx, rx) = mpsc::channel(128);
                                    subscribers.push(tx);
                                    let _ = rsp.send(rx);
                                }
                                Command::GetStats { rsp } => {
                                    let network_info = swarm.network_info();
                                    let mut stats_guard = stats_clone.lock().unwrap();
                                    
                                    // Count Kademlia peers by inspecting the routing table
                                    // Use the actual method available on Kademlia behaviour
                                    let kademlia_peer_count = {
                                        let mut count = 0;
                                        for bucket in swarm.behaviour_mut().kademlia.kbuckets() {
                                            count += bucket.num_entries();
                                        }
                                        count
                                    };
                                    
                                    // Update the kademlia peers count in our enhanced stats
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
                                Command::GetKademliaRecord { key, rsp } => {
                                    let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                                    pending_kad_queries.insert(query_id, rsp);
                                }
                                Command::PutKademliaRecord { key, value, rsp } => {
                                    let record = KademliaRecord { key, value, publisher: None, expires: None };
                                    match swarm.behaviour_mut().kademlia.put_record(record, Quorum::One) {
                                        Ok(_) => { let _ = rsp.send(Ok(())); }
                                        Err(e) => { let _ = rsp.send(Err(CommonError::NetworkError(format!("Put record failed: {:?}", e)))); }
                                    }
                                }
                            }
                        }
                        else => break,
                    }
                }
                log::info!("Network event loop ended");
            });

            Ok(Self {
                local_peer_id,
                cmd_tx,
                config,
            })
        }

        async fn handle_swarm_event(
            event: SwarmEvent<CombinedEvent>,
            stats: &Arc<Mutex<EnhancedNetworkStats>>,
            subscribers: &mut Vec<mpsc::Sender<super::NetworkMessage>>,
            pending_kad_queries: &mut HashMap<QueryId, oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>>,
        ) {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(CombinedEvent::Gossipsub(gossipsub::Event::Message {
                    message, ..
                })) => {
                    let message_size = message.data.len() as u64;
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.bytes_received += message_size;
                        stats_guard.messages_received += 1;
                    }
                    
                    if let Ok(network_msg) = bincode::deserialize::<super::NetworkMessage>(&message.data) {
                        log::debug!("Received gossipsub message: {:?}", network_msg.message_type());
                        
                        // Update message type statistics
                        let msg_type = network_msg.message_type().to_string();
                        {
                            let mut stats_guard = stats.lock().unwrap();
                            let type_stats = stats_guard.message_counts.entry(msg_type).or_default();
                            type_stats.received += 1;
                            type_stats.bytes_received += message_size;
                        }
                        
                        // Distribute to subscribers
                        subscribers.retain_mut(|subscriber| {
                            subscriber.try_send(network_msg.clone()).is_ok()
                        });
                    }
                }
                SwarmEvent::Behaviour(CombinedEvent::Kademlia(KademliaEvent::OutboundQueryProgressed {
                    id, result, ..
                })) => {
                    if let Some(sender) = pending_kad_queries.remove(&id) {
                        match result {
                            KademliaQueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(record))) => {
                                let _ = sender.send(Ok(Some(record.record)));
                            }
                            KademliaQueryResult::GetRecord(Ok(GetRecordOk::FinishedWithNoAdditionalRecord { .. })) => {
                                let _ = sender.send(Ok(None));
                            }
                            KademliaQueryResult::GetRecord(Err(e)) => {
                                let _ = sender.send(Err(CommonError::NetworkError(format!("Kademlia error: {:?}", e))));
                            }
                            _ => {}
                        }
                    }
                }
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
    }

    #[async_trait]
    impl super::NetworkService for Libp2pNetworkService {
        async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<super::PeerId>, CommonError> {
            let target = match target_peer_id_str {
                Some(id_str) => Some(Libp2pPeerId::from_str(&id_str)
                    .map_err(|e| CommonError::InvalidInputError(format!("Invalid peer ID: {}", e)))?),
                None => None,
            };

            let (tx, rx) = oneshot::channel();
            self.cmd_tx.send(Command::DiscoverPeers { target, rsp: tx }).await
                .map_err(|e| CommonError::NetworkError(format!("Command send failed: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkError(format!("Response dropped: {}", e)))?
        }

        async fn send_message(&self, peer: &super::PeerId, message: super::NetworkMessage) -> Result<(), CommonError> {
            let libp2p_peer = Libp2pPeerId::from_str(&peer.0)
                .map_err(|e| CommonError::InvalidInputError(format!("Invalid peer ID: {}", e)))?;

            let (tx, rx) = oneshot::channel();
            self.cmd_tx.send(Command::SendMessage { peer: libp2p_peer, message, rsp: tx }).await
                .map_err(|e| CommonError::MessageSendError(format!("Command send failed: {}", e)))?;
            rx.await.map_err(|e| CommonError::MessageSendError(format!("Response dropped: {}", e)))?
        }

        async fn broadcast_message(&self, message: super::NetworkMessage) -> Result<(), CommonError> {
            let data = bincode::serialize(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            
            self.cmd_tx.send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("Broadcast failed: {}", e)))
        }

        fn subscribe(&self) -> Result<Receiver<super::NetworkMessage>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx.try_send(Command::Subscribe { rsp: tx })
                .map_err(|e| CommonError::NetworkError(format!("Subscribe failed: {}", e)))?;
            
            // This is a blocking operation but should be quick
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    rx.await.map_err(|e| CommonError::NetworkError(format!("Subscribe response failed: {}", e)))
                })
            })
        }

        async fn get_network_stats(&self) -> Result<super::NetworkStats, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx.send(Command::GetStats { rsp: tx }).await
                .map_err(|e| CommonError::NetworkError(format!("Get stats failed: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkError(format!("Stats response failed: {}", e)))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
} 