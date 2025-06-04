#![doc = include_str!("../README.md")]

//! # ICN Network Crate
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! likely using libp2p. It covers P2P communication, transport protocols, peer discovery,
//! message routing, and federation synchronization.
//!
//! To enable detailed logging for mesh-related events, run tests or binaries with:
//! `RUST_LOG=icn_network=debug,icn_runtime=debug` (or adjust levels as needed).

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
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::SystemTime;

use libp2p::kad::RecordKey as KademliaKey;
use libp2p::kad::{Record as KademliaRecord, QueryId, Quorum, GetRecordOk, PutRecordOk, store::MemoryStore, Behaviour as KademliaBehaviour, Config as KademliaConfig, Event as KademliaEvent, QueryResult as KademliaQueryResult};
use ::libp2p_request_response::{Behaviour as RequestResponseBehaviour, Codec as RequestResponseCodec, Config as RequestResponseConfig, Event as RequestResponseEvent, Message as RequestResponseMessage, RequestId, OutboundRequestId, ProtocolSupport};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bincode;

// --- Peer and Message Scaffolding ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String); // Placeholder, typically a libp2p PeerId

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
    GossipSub(String, Vec<u8>), // topic, data
    FederationSyncRequest(Did), // Request sync from a federation representative DID
    // TODO: Add more message types as protocols develop
    MeshJobAnnouncement(Job),
    BidSubmission(Bid),
    JobAssignmentNotification(JobId, Did),
    SubmitReceipt(ExecutionReceipt),
}

/// Network service trait definition.
#[async_trait]
pub trait NetworkService: Send + Sync + Debug + DowncastSync + 'static {
    async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<PeerId>, CommonError>;
    async fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError>;
    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError>;
    fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError>;
    fn as_any(&self) -> &dyn Any;
}
impl_downcast!(sync NetworkService);

/// Stub implementation for NetworkService.
#[derive(Default, Debug)]
pub struct StubNetworkService;

// TODO (#issue_url_for_libp2p_integration): Implement `Libp2pNetworkService` that uses a real libp2p stack.
// This service should be conditionally compiled when the `experimental-libp2p` feature is enabled.
// It will involve managing Swarm, Behaviours (e.g., Kademlia, Gossipsub), and transport configurations.

#[async_trait]
impl NetworkService for StubNetworkService {
    async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<PeerId>, CommonError> {
        println!("[StubNetworkService] Discovering peers (target: {:?})... returning mock peers.", target_peer_id_str);
        // Keep existing mock logic, target_peer_id_str is ignored by stub for now
        Ok(vec![PeerId("mock_peer_1".to_string()), PeerId("mock_peer_2".to_string())])
    }

    async fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Sending message to peer {:?}: {:?}", peer, message);
        if peer.0 == "error_peer" { // Simulate a peer that causes an error
            return Err(CommonError::MessageSendError(format!("Failed to send message to peer: {}", peer.0)));
        }
        if peer.0 == "unknown_peer_id" { // Simulate peer not found
            return Err(CommonError::PeerNotFound(format!("Peer with ID {} not found.", peer.0)));
        }
        Ok(())
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Broadcasting message: {:?}", message);
        // Simulate a potential broadcast error condition, e.g. network not ready
        if let NetworkMessage::GossipSub(topic, _) = &message {
            if topic == "system_critical_error_topic" {
                return Err(CommonError::NetworkUnhealthy("Broadcast failed: system critical topic is currently down.".to_string()));
            }
        }
        Ok(())
    }

    fn subscribe(&self) -> Result<Receiver<NetworkMessage>, CommonError> {
        println!("[StubNetworkService] Subscribing to messages... returning an empty channel.");
        let (_tx, rx) = tokio::sync::mpsc::channel(1); // Create a dummy channel
        Ok(rx)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Placeholder function demonstrating use of common types for network operations.
// TODO (#issue_url_for_libp2p_integration): Update this function or provide new examples 
// once `Libp2pNetworkService` is available to show real network interaction.
pub async fn send_network_ping(info: &NodeInfo, target_peer: &str) -> Result<String, CommonError> {
    let service = StubNetworkService::default();
    let _ = service.send_message(&PeerId(target_peer.to_string()), NetworkMessage::GossipSub("ping_topic".to_string(), vec![1,2,3])).await?;
    Ok(format!("Sent (stubbed) ping to {} from node: {} (v{})", target_peer, info.name, info.version))
}

#[cfg(all(test, feature = "experimental-libp2p"))]
mod tests {
    use super::*;
    // use tokio; // Keeping this commented out for now, as the outer feature gate might be enough

    #[tokio::test]
    async fn test_stub_network_service_discover() {
        let service = StubNetworkService::default();
        let peers = service.discover_peers(Some("/ip4/127.0.0.1/tcp/12345".to_string())).await.unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].0, "mock_peer_1");

        // Test a case that might be an error (though stub currently doesn't error on empty list here)
        // let result_empty_bootstrap = service.discover_peers(Vec::new());
        // assert!(matches!(result_empty_bootstrap, Err(CommonError::InvalidInputError(_))));
    }

    #[tokio::test]
    async fn test_stub_network_service_send_broadcast() {
        let service = StubNetworkService::default();
        let peer_ok = PeerId("test_target_peer".to_string());
        let peer_error = PeerId("error_peer".to_string());
        let peer_unknown = PeerId("unknown_peer_id".to_string());

        let block_cid = Cid::new_v1_dummy(0x55, 0x12, b"net test");
        // Corrected: NetworkMessage::AnnounceBlock expects a DagBlock, not just a CID.
        let dummy_block = DagBlock { 
            cid: block_cid.clone(), 
            data: b"dummy data".to_vec(), 
            links: vec![] 
        };
        let message_announce = NetworkMessage::AnnounceBlock(dummy_block.clone()); // clone if used again for broadcast
        let message_request = NetworkMessage::RequestBlock(block_cid.clone());

        assert!(service.send_message(&peer_ok, message_announce.clone()).await.is_ok());
        
        let send_error_result = service.send_message(&peer_error, message_announce.clone()).await;
        assert!(matches!(send_error_result, Err(CommonError::MessageSendError(_))));

        let send_unknown_result = service.send_message(&peer_unknown, message_request.clone()).await;
        assert!(matches!(send_unknown_result, Err(CommonError::PeerNotFound(_))));

        assert!(service.broadcast_message(message_announce).await.is_ok());

        let broadcast_error_message = NetworkMessage::GossipSub("system_critical_error_topic".to_string(), vec![]);
        let broadcast_error_result = service.broadcast_message(broadcast_error_message).await;
        assert!(matches!(broadcast_error_result, Err(CommonError::NetworkUnhealthy(_))));
    }

    #[tokio::test]
    async fn test_send_network_ping_uses_stub() {
        let node_info = NodeInfo {
            name: "NetNodePing".to_string(),
            version: "0.1.0".to_string(),
            status_message: "Network ping test".to_string(),
        };
        let result = send_network_ping(&node_info, "peer-xyz-ping").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("peer-xyz-ping"));
        // Output will show [StubNetworkService] logs if test is run with --nocapture
    }
}

// --- Libp2p Network Service Implementation ---

#[cfg(feature = "experimental-libp2p")]
pub mod libp2p_service {
    use super::*;
    use libp2p::futures::StreamExt;
    use libp2p::{
        core::upgrade,
        gossipsub,
        identity,
        noise,
        ping,
        swarm::{NetworkBehaviour, Swarm, SwarmEvent, Config as SwarmConfig},
        tcp,
        yamux, PeerId as Libp2pPeerId, Transport, Multiaddr,
        dns,
    };
    use std::time::Duration;
    use tokio::{sync::{mpsc, oneshot}, task};
    use std::sync::{Arc, Mutex, RwLock};
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;
    use std::time::SystemTime;

    use libp2p::kad::RecordKey as KademliaKey;
    use bincode;

    #[cfg(feature = "experimental-libp2p")] // Technically redundant due to module cfg, but explicit.
    const _FORCE_RESOLVE: Option<libp2p_request_response::RequestId> = None;

    /* ---------- Public façade ------------------------------------------------ */

    #[derive(Clone, Debug)]
    pub struct Libp2pNetworkService {
        local_peer_id: Libp2pPeerId,
        listening_addresses: Arc<Mutex<Vec<Multiaddr>>>,
        peer_manager: Arc<PeerManager>,
        message_router: Arc<MessageRouter>,
        cmd_tx: mpsc::Sender<Command>,
    }

    #[derive(Debug)]
    pub struct PeerManager {
        connected_peers: Arc<RwLock<HashMap<Libp2pPeerId, PeerInfo>>>,
        blacklisted_peers: Arc<RwLock<HashSet<Libp2pPeerId>>>,
    }

    #[derive(Debug)]
    pub struct PeerInfo {
        pub peer_id: Libp2pPeerId,
        pub addresses: Vec<Multiaddr>,
        pub connection_time: SystemTime,
        pub last_seen: SystemTime,
        pub connection_state: ConnectionState,
    }

    #[derive(Debug, Clone)]
    pub enum ConnectionState {
        Connecting,
        Connected,
        Disconnecting,
        Failed { reason: String, retry_after: SystemTime },
    }

    #[derive(Debug)]
    pub struct MessageRouter {
        gossipsub: Arc<Mutex<gossipsub::Behaviour>>,
        request_response: Arc<Mutex<RequestResponseBehaviour<MessageCodec>>>,
        topics: Arc<RwLock<HashMap<String, gossipsub::IdentTopic>>>,
        pending_requests: Arc<Mutex<HashMap<libp2p_request_response::RequestId, oneshot::Sender<super::NetworkMessage>>>>,
    }

    #[derive(Debug, Clone)]
    pub struct MessageCodec;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MessageProtocol();

    impl FromStr for MessageProtocol {
        type Err = std::io::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "/icn/message/1.0.0" {
                Ok(MessageProtocol())
            }
            else {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid protocol string"))
            }
        }
    }

    impl AsRef<str> for MessageProtocol {
        fn as_ref(&self) -> &str {
            "/icn/message/1.0.0"
        }
    }

    #[async_trait]
    impl RequestResponseCodec for MessageCodec {
        type Protocol = MessageProtocol;
        type Request = super::NetworkMessage;
        type Response = super::NetworkMessage;
    
        async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Request>
        where
            T: libp2p::futures::AsyncRead + Unpin + Send,
        {
            let mut buf = Vec::new();
            io.read_to_end(&mut buf).await?;
            bincode::deserialize(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }

        async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Response>
        where
            T: libp2p::futures::AsyncRead + Unpin + Send,
        {
            let mut buf = Vec::new();
            io.read_to_end(&mut buf).await?;
            bincode::deserialize(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }
    
        async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> std::io::Result<()>
        where
            T: libp2p::futures::AsyncWrite + Unpin + Send,
        {
            let data = bincode::serialize(&req)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await?;
            io.close().await
        }

        async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) -> std::io::Result<()>
        where
            T: libp2p::futures::AsyncWrite + Unpin + Send,
        {
            let data = bincode::serialize(&res)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&data).await?;
            io.close().await
        }
    }

    impl Libp2pNetworkService {
        pub async fn new(bootstrap_peers_opt: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Self, CommonError> {
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = Libp2pPeerId::from(local_key.public());

            let transport = dns::tokio::Transport::system(
                tcp::tokio::Transport::new(tcp::Config::default().nodelay(true)),
            )
            .map_err(|e| CommonError::NetworkSetupError(format!("DNS config error: {}", e)))?
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key).map_err(|e| CommonError::NetworkSetupError(format!("Noise auth error: {}",e)))?)
            .multiplex(yamux::Config::default())
            .timeout(std::time::Duration::from_secs(20))
            .boxed();

            let gossipsub_config = gossipsub::Config::default();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config
            )
            .map_err(|s| CommonError::NetworkSetupError(format!("Gossipsub setup error: {}",s)))?;

            let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(15)));

            let store = MemoryStore::new(local_peer_id);
            let mut kademlia_config = KademliaConfig::default();
            kademlia_config.disjoint_query_paths(true);
            let kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);
            
            let request_response_protocols = std::iter::once((MessageProtocol(), ProtocolSupport::Full));
            let request_response_config = libp2p_request_response::Config::default();
            let request_response = RequestResponseBehaviour::with_codec(
                MessageCodec,
                request_response_protocols,
                request_response_config,
            );

            #[derive(NetworkBehaviour)]
            #[behaviour(out_event = "CombinedEvent")]
            struct CombinedBehaviour {
                gossipsub: gossipsub::Behaviour,
                ping: ping::Behaviour,
                kademlia: KademliaBehaviour<MemoryStore>,
                request_response: RequestResponseBehaviour<MessageCodec>,
            }

            #[allow(clippy::large_enum_variant)]
            enum CombinedEvent {
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


            let behaviour = CombinedBehaviour { gossipsub, ping, kademlia, request_response };
            let mut swarm = Swarm::new(transport, behaviour, local_peer_id, SwarmConfig::with_tokio_executor());

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                 .map_err(|e| CommonError::NetworkSetupError(format!("Listen on error: {}",e)))?;

            if let Some(bootstrap_peers) = bootstrap_peers_opt {
                for (peer_id, addr) in bootstrap_peers {
                    log::info!("[libp2p_service][mesh-job] Adding bootstrap peer: {peer_id} at {addr}");
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
                if !swarm.behaviour().kademlia.kbuckets_entries().is_empty() {
                     log::info!("[libp2p_service][mesh-job] Attempting Kademlia bootstrap...");
                     if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                        log::warn!("[libp2p_service][mesh-job] Kademlia bootstrap failed: {:?}",e);
                     } else {
                        log::info!("[libp2p_service][mesh-job] Kademlia bootstrap initiated.");
                     }
                } else {
                    log::warn!("[libp2p_service][mesh-job] No bootstrap peers to connect to for Kademlia bootstrap.");
                }
            }


            let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
            let listening_addresses = Arc::new(Mutex::new(Vec::new()));
            let listening_addresses_clone = listening_addresses.clone();
            
            let peer_manager = Arc::new(PeerManager {
                connected_peers: Arc::new(RwLock::new(HashMap::new())),
                blacklisted_peers: Arc::new(RwLock::new(HashSet::new())),
            });
            let message_router_gossipsub = Arc::new(Mutex::new(swarm.behaviour().gossipsub.clone()));
            let message_router_req_res = Arc::new(Mutex::new(swarm.behaviour().request_response.clone()));

            let message_router = Arc::new(MessageRouter {
                gossipsub: Arc::new(Mutex::new(gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(local_key.clone()), gossipsub::Config::default()).unwrap())),
                request_response: Arc::new(Mutex::new(RequestResponseBehaviour::with_codec(MessageCodec, std::iter::once((MessageProtocol(), ProtocolSupport::Full)), libp2p_request_response::Config::default()))),
                topics: Arc::new(RwLock::new(HashMap::new())),
                pending_requests: Arc::new(Mutex::new(HashMap::new())),
            });


            task::spawn(async move {
                let topic = gossipsub::IdentTopic::new("icn-global");
                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                     log::error!("[libp2p_service][mesh-job] Failed to subscribe to global topic: {:?}", e);
                } else {
                    log::info!("[libp2p_service][mesh-job] Subscribed to global topic: {}", topic.hash());
                }

                let mut subscriber_senders: Vec<mpsc::Sender<super::NetworkMessage>> = Vec::new();

                loop {
                    tokio::select! {
                        event = swarm.select_next_some() => {
                            match event {
                                SwarmEvent::NewListenAddr { address, .. } => {
                                    log::info!("[libp2p_service][mesh-job] Listening on {}", address);
                                    listening_addresses_clone.lock().unwrap().push(address);
                                }
                                SwarmEvent::Behaviour(CombinedEvent::Gossipsub(gossipsub::Event::Message {
                                    propagation_source: _peer_id,
                                    message_id: _id,
                                    message,
                                })) => {
                                    log::debug!("[libp2p_service][mesh-job] Received gossipsub message: {:?}", message.data.len());
                                    match bincode::deserialize::<super::NetworkMessage>(&message.data) {
                                        Ok(network_msg) => {
                                            log::debug!("[libp2p_service][mesh-job] Deserialized gossipsub message: {:?}", network_msg);
                                            for sender in subscriber_senders.iter_mut() {
                                                if let Err(e) = sender.try_send(network_msg.clone()) {
                                                    log::error!("[libp2p_service][mesh-job] Failed to send message to subscriber: {:?}", e);
                                                }
                                            }
                                        }
                                        Err(e) => log::error!("[libp2p_service][mesh-job] Failed to deserialize gossipsub message: {:?}", e),
                                    }
                                }
                                SwarmEvent::Behaviour(CombinedEvent::Ping(_)) => {
                                    // log::debug!("[libp2p_service][mesh-job] Ping event: {:?}", event);
                                }
                                SwarmEvent::Behaviour(CombinedEvent::Kademlia(event)) => {
                                    log::debug!("[libp2p_service][mesh-job] Kademlia event: {:?}", event);
                                    match event {
                                        KademliaEvent::OutboundQueryProgressed { result, .. } => match result {
                                            KademliaQueryResult::GetClosestPeers(Ok(ok)) => {
                                                 log::info!("[libp2p_service][mesh-job] KAD GetClosestPeers OK: {:?} peers found", ok.peers.len());
                                            }
                                            KademliaQueryResult::GetClosestPeers(Err(err)) => {
                                                log::warn!("[libp2p_service][mesh-job] KAD GetClosestPeers ERR: {:?}", err);
                                            }
                                            KademliaQueryResult::GetRecord(Ok(
                                                GetRecordOk::FoundRecord(record)
                                            )) => {
                                                log::info!("[libp2p_service][mesh-job] KAD GetRecord Found: key={:?}, value_len={}", String::from_utf8_lossy(&record.record.key), record.record.value.len());
                                            }
                                            KademliaQueryResult::GetRecord(Ok(
                                                GetRecordOk::FinishedWithNoAdditionalRecord { .. }
                                            )) => {
                                                 log::info!("[libp2p_service][mesh-job] KAD GetRecord Finished with no additional record.");
                                            }
                                            KademliaQueryResult::GetRecord(Err(err)) => {
                                                 log::warn!("[libp2p_service][mesh-job] KAD GetRecord ERR: {:?}", err);
                                            }
                                            KademliaQueryResult::PutRecord(Ok(PutRecordOk{key})) => {
                                                log::info!("[libp2p_service][mesh-job] KAD PutRecord OK: key={:?}", String::from_utf8_lossy(&key));
                                            }
                                            KademliaQueryResult::PutRecord(Err(err)) => {
                                                 log::warn!("[libp2p_service][mesh-job] KAD PutRecord ERR: {:?}", err);
                                            }
                                            _ => {}
                                        }
                                        _ => {}
                                    }
                                }
                                SwarmEvent::Behaviour(CombinedEvent::RequestResponse(event)) => {
                                    log::debug!("[libp2p_service][mesh-job] RequestResponse event: {:?}", event);
                                    match event {
                                        RequestResponseEvent::Message { peer, message } => {
                                            match message {
                                                RequestResponseMessage::Request { request, channel, .. } => {
                                                    log::info!("Received request {:?} from peer {:?}", request, peer);
                                                    if let Err(e) = swarm.behaviour_mut().request_response.send_response(channel, request.clone()) {
                                                        log::error!("Failed to send response: {:?}", e);
                                                    }
                                                },
                                                RequestResponseMessage::Response { request_id, response } => {
                                                    log::info!("Received response {:?} for request {:?}", response, request_id);
                                                }
                                            }
                                        }
                                        RequestResponseEvent::OutboundFailure { peer, request_id, error } => {
                                            log::error!("Outbound request {:?} to peer {:?} failed: {:?}", request_id, peer, error);
                                        }
                                        RequestResponseEvent::InboundFailure { peer, request_id, error } => {
                                            log::error!("Inbound request {:?} from peer {:?} failed: {:?}", request_id, peer, error);
                                        }
                                        RequestResponseEvent::ResponseSent { peer, request_id } => {
                                            log::info!("Response sent for request {:?} to peer {:?}", request_id, peer);
                                        }
                                    }
                                }
                                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                                    log::info!("[libp2p_service][mesh-job] Connection established with {} on {:?}", peer_id, endpoint);
                                    if swarm.behaviour().gossipsub.add_explicit_peer(&peer_id) {
                                        log::debug!("[libp2p_service][mesh-job] Added {} to gossipsub explicit peers", peer_id);
                                    }
                                }
                                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                                     log::info!("[libp2p_service][mesh-job] Connection closed with {}, cause: {:?}", peer_id, cause);
                                }
                                other_event => {
                                    log::trace!("[libp2p_service][mesh-job] Other Swarm event: {:?}", other_event);
                                }
                            }
                        }
                        Some(command) = cmd_rx.recv() => {
                            match command {
                                Command::DiscoverPeers { target, rsp } => {
                                    log::debug!("[libp2p_service][mesh-job] DiscoverPeers command received, target: {:?}", target);
                                    let query_id = match target {
                                        Some(specific_peer) => swarm.behaviour_mut().kademlia.get_closest_peers(specific_peer),
                                        None => swarm.behaviour_mut().kademlia.get_closest_peers(Libp2pPeerId::random()),
                                    };
                                    log::debug!("[libp2p_service][mesh-job] Kademlia get_closest_peers query started: {:?}", query_id);
                                    let peers: Vec<super::PeerId> = swarm.behaviour().kademlia.kbuckets_entries()
                                        .map(|entry| super::PeerId(entry.node.key.preimage().to_string()))
                                        .collect();
                                    if rsp.send(Ok(peers)).is_err() {
                                        log::warn!("[libp2p_service] DiscoverPeers: Receiver for Kademlia query result was dropped before sending.");
                                    }
                                }
                                Command::GetRoutingTablePeers { rsp } => {
                                    let peers = swarm.behaviour().kademlia.kbuckets_entries()
                                        .map(|entry| super::PeerId(entry.node.key.preimage().to_string()))
                                        .collect();
                                    if rsp.send(Ok(peers)).is_err() {
                                        log::warn!("[libp2p_service] GetRoutingTablePeers: Receiver for Kademlia query result was dropped before sending.");
                                    }
                                }
                                Command::AddKadAddress { peer_id, addr } => {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                                Command::TriggerKadBootstrap { rsp } => {
                                    match swarm.behaviour_mut().kademlia.bootstrap() {
                                        Ok(id) => {
                                            log::info!("[libp2p_service][mesh-job] KAD bootstrap initiated with query id: {:?}", id);
                                            if rsp.send(Ok(())).is_err() {
                                                log::warn!("[libp2p_service] TriggerKadBootstrap: Receiver for bootstrap OK was dropped.");
                                            }
                                        }
                                        Err(e) => {
                                            log::warn!("[libp2p_service][mesh-job] KAD bootstrap failed to start: {:?}", e);
                                            if rsp.send(Err(CommonError::NetworkOperationError(format!("Kademlia bootstrap error: {:?}", e)))).is_err() {
                                                log::warn!("[libp2p_service] TriggerKadBootstrap: Receiver for bootstrap error was dropped.");
                                            }
                                        }
                                    }
                                }
                                Command::PutKadRecord { key, value, rsp } => {
                                    let record = KademliaRecord { key, value, publisher: None, expires: None };
                                    match swarm.behaviour_mut().kademlia.put_record(record, Quorum::One) {
                                        Ok(query_id) => {
                                            log::info!("[libp2p_service][mesh-job] KAD PutRecord initiated with query id: {:?}", query_id);
                                            if rsp.send(Ok(())).is_err() {
                                                log::warn!("[libp2p_service] PutKadRecord: Receiver for Kademlia put record result was dropped before sending.");
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("[libp2p_service][mesh-job] KAD PutRecord failed: {:?}", e);
                                            if rsp.send(Err(CommonError::NetworkOperationError(format!("Kademlia put_record error: {:?}", e)))).is_err() {
                                                log::warn!("[libp2p_service] PutKadRecord: Receiver for Kademlia put record error was dropped before sending.");
                                            }
                                        }
                                    }
                                }
                                Command::GetKadRecord { key, rsp } => {
                                     let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                                     log::info!("[libp2p_service][mesh-job] KAD GetRecord initiated with query id: {:?}", query_id);
                                     if rsp.send(Ok(None)).is_err() {
                                        log::warn!("[libp2p_service] GetKadRecord: Receiver for Kademlia get record result was dropped before sending.");
                                     }
                                }
                                Command::Broadcast { data } => {
                                    log::debug!("[libp2p_service][mesh-job] Broadcasting message (data_len: {}) to topic: {}", data.len(), topic);
                                    if let Err(e) = swarm.behaviour_mut()
                                        .gossipsub
                                        .publish(topic.clone(), data) {
                                        log::error!("[libp2p_service][mesh-job] Failed to publish to topic {topic}: {:?}", e);
                                    }
                                }
                                 Command::SendMessage { peer, message, rsp } => {
                                    log::debug!("[libp2p_service] SendMessage command to peer: {:?}, message: {:?}", peer, message);
                                    let request_id = swarm.behaviour_mut().request_response.send_request(&peer, message);
                                    log::info!("Sent request with id: {:?}", request_id);
                                    if rsp.send(Ok(())).is_err() {
                                        log::warn!("[libp2p_service] SendMessage: Receiver for send message result was dropped before sending.");
                                    }
                                }
                                Command::AddSubscriber { rsp_tx } => {
                                    subscriber_senders.push(rsp_tx);
                                    log::info!("[libp2p_service][mesh-job] Added new NetworkMessage subscriber. Total subscribers: {}", subscriber_senders.len());
                                }
                            }
                        }
                        else => break,
                    }
                }
                log::info!("[libp2p_service][mesh-job] Swarm task ended.");
            });

            Ok(Self { 
                cmd_tx, 
                local_peer_id, 
                listening_addresses,
                peer_manager,
                message_router,
            })
        }

        pub fn local_peer_id(&self) -> &Libp2pPeerId {
            &self.local_peer_id
        }

        pub fn listening_addresses(&self) -> Vec<Multiaddr> {
            self.listening_addresses.lock().unwrap().clone()
        }

        pub async fn get_routing_table_peers(&self) -> Result<Vec<super::PeerId>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::GetRoutingTablePeers { rsp: tx }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("get_routing_table_peers cmd send error: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("get_routing_table_peers response dropped: {}", e)))?
        }

        pub async fn add_kad_address(&self, peer_id: Libp2pPeerId, addr: Multiaddr) -> Result<(), CommonError> {
            self.cmd_tx
                .send(Command::AddKadAddress { peer_id, addr }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("add_kad_address cmd send error: {}", e)))
        }

        pub async fn trigger_kad_bootstrap(&self) -> Result<(), CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::TriggerKadBootstrap { rsp: tx }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("trigger_kad_bootstrap cmd send error: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("trigger_kad_bootstrap response dropped: {}", e)))?
        }

        pub async fn put_kad_record(&self, key: KademliaKey, value: Vec<u8>) -> Result<(), CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::PutKadRecord { key, value, rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkSetupError(format!("put_kad_record cmd send error: {e}")))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("put_kad_record response dropped: {e}")))?
        }

        pub async fn get_kad_record(&self, key: KademliaKey) -> Result<Option<KademliaRecord>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::GetKadRecord { key: key.clone(), rsp: tx }) 
                .await
                .map_err(|e| CommonError::NetworkSetupError(format!("get_kad_record cmd send error: {e}")))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("get_kad_record response dropped: {e}")))?
        }
    }

    /* ---------- internal command enum -------------------------------------- */

    #[derive(Debug)]
    enum Command {
        DiscoverPeers {
            target: Option<Libp2pPeerId>,
            rsp: oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>
        },
        GetRoutingTablePeers {
            rsp: oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>
        },
        AddKadAddress {
            peer_id: Libp2pPeerId,
            addr: Multiaddr,
        },
        TriggerKadBootstrap {
            rsp: oneshot::Sender<Result<(), CommonError>>
        },
        PutKadRecord {
            key: KademliaKey,
            value: Vec<u8>,
            rsp: oneshot::Sender<Result<(), CommonError>>,
        },
        GetKadRecord {
            key: KademliaKey,
            rsp: oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>,
        },
        Broadcast { data: Vec<u8> },
        SendMessage {
            peer: Libp2pPeerId,
            message: super::NetworkMessage,
            rsp: oneshot::Sender<Result<(), CommonError>>,
        },
        AddSubscriber { rsp_tx: mpsc::Sender<super::NetworkMessage> },
    }

    /* ---------- hook into crate-level trait -------------------------------- */

    #[async_trait]
    impl super::NetworkService for Libp2pNetworkService {
        async fn discover_peers(
            &self,
            target_peer_id_str: Option<String>,
        ) -> Result<Vec<super::PeerId>, CommonError> {
            log::debug!("[libp2p_service][mesh-job] Discovering peers, target: {:?}", target_peer_id_str);
            let target_libp2p_id = match target_peer_id_str {
                Some(id_str) => {
                    match Libp2pPeerId::from_str(&id_str) {
                        Ok(peer_id) => Some(peer_id),
                        Err(e) => return Err(CommonError::InvalidInputError(format!("Invalid target PeerId string '{}': {e}", id_str))),
                    }
                }
                None => None,
            };

            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::DiscoverPeers { target: target_libp2p_id, rsp: tx }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("discover_peers cmd send error: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("discover_peers response dropped: {}", e)))?
        }

        async fn send_message(
            &self,
            peer: &super::PeerId,
            msg: super::NetworkMessage,
        ) -> Result<(), CommonError> {
            log::debug!("[libp2p_service][mesh-job] send_message to peer {:?} with: {:?}", peer, msg);
            let libp2p_peer_id = Libp2pPeerId::from_str(&peer.0)
                .map_err(|e| CommonError::InvalidInputError(format!("Invalid peer ID string '{}': {}", peer.0, e)))?;
            
            let (tx, rx) = oneshot::channel();
            self.cmd_tx.send(Command::SendMessage {
                peer: libp2p_peer_id,
                message: msg,
                rsp: tx,
            }).await.map_err(|e| CommonError::MessageSendError(format!("send_message cmd send error: {}", e)))?;
            
            rx.await.map_err(|e| CommonError::MessageSendError(format!("send_message response dropped: {}",e)))?
        }
        
        async fn broadcast_message(&self, message: super::NetworkMessage) -> Result<(), CommonError> {
            log::debug!("[libp2p_service][mesh-job] broadcast_message called with: {:?}", message);
            let data = bincode::serialize(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("broadcast_message cmd send error: {}", e)))
        }

        fn subscribe(&self) -> Result<mpsc::Receiver<super::NetworkMessage>, CommonError> {
            let (msg_tx, msg_rx) = mpsc::channel(128); 
            
            self.cmd_tx.try_send(Command::AddSubscriber { rsp_tx: msg_tx })
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to send AddSubscriber command: {e}")))?;
            
            Ok(msg_rx)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}