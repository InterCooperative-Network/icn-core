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
use std::sync::Arc;

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
// This service should be conditionally compiled when the `with-libp2p` feature is enabled.
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
    use serde_json;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use std::str::FromStr;

    use libp2p::kad::RecordKey as KademliaKey;
    use libp2p::kad::{Record as KademliaRecord, QueryId, Quorum, GetRecordOk, PutRecordOk, store::MemoryStore, Behaviour as KademliaBehaviour, Config as KademliaConfig, Event as KademliaEvent, QueryResult as KademliaQueryResult};

    /* ---------- Public façade ------------------------------------------------ */

    #[derive(Clone, Debug)]
    pub struct Libp2pNetworkService {
        cmd_tx: mpsc::Sender<Command>,
        local_peer_id: Libp2pPeerId,
        listening_addresses: Arc<Mutex<Vec<Multiaddr>>>,
        // We don't store the message_rx here directly.
        // Instead, the Swarm task will manage senders to subscribers.
    }

    impl Libp2pNetworkService {
        pub async fn new(bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Self, CommonError> {
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

            let gossipsub = {
                let cfg = gossipsub::Config::default();
                gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(local_key.clone()), cfg)
                    .map_err(|s| CommonError::NetworkSetupError(format!("Gossipsub setup error: {}",s)))?
            };

            let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(15)));

            let store = MemoryStore::new(local_peer_id);
            let mut kademlia_config = KademliaConfig::default();
            kademlia_config.disjoint_query_paths(true);
            let kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);

            #[derive(NetworkBehaviour)]
            #[behaviour(out_event = "Event")]
            struct Behaviour {
                gossipsub: gossipsub::Behaviour,
                ping: ping::Behaviour,
                kademlia: KademliaBehaviour<MemoryStore>,
            }

            #[allow(clippy::large_enum_variant)]
            #[derive(Debug)]
            enum Event {
                Gossipsub(gossipsub::Event),
                Ping(ping::Event),
                Kademlia(KademliaEvent),
            }

            impl From<gossipsub::Event> for Event {
                fn from(e: gossipsub::Event) -> Self { Event::Gossipsub(e) }
            }
            impl From<ping::Event> for Event {
                fn from(e: ping::Event) -> Self { Event::Ping(e) }
            }
            impl From<KademliaEvent> for Event {
                fn from(e: KademliaEvent) -> Self { Event::Kademlia(e) }
            }

            let behaviour = Behaviour { gossipsub, ping, kademlia };
            
            let swarm_config = SwarmConfig::with_tokio_executor();
            let mut swarm = Swarm::new(transport, behaviour, local_peer_id, swarm_config);

            if let Some(peers) = bootstrap_peers {
                for (peer_id, addr) in peers {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                eprintln!("[libp2p] Kademlia bootstrap failed: {:?}", e);
            }

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

            let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(16);
            let listening_addresses = Arc::new(Mutex::new(Vec::new()));
            let listening_addresses_clone = listening_addresses.clone();

            // Kademlia query tracking
            let mut pending_kad_queries = HashMap::<QueryId, oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>>::new();
            let mut pending_put_kad_queries = HashMap::<QueryId, oneshot::Sender<Result<(), CommonError>>>::new();
            let mut pending_get_kad_records = HashMap::<QueryId, oneshot::Sender<Result<Option<KademliaRecord>, CommonError>>>::new();
            
            // List of senders to subscribers
            let mut subscriber_senders = Vec::<mpsc::Sender<super::NetworkMessage>>::new();

            task::spawn(async move {
                let topic = gossipsub::IdentTopic::new("icn-default");
                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    log::error!("[libp2p_service][mesh-job] Failed to subscribe to topic {}: {:?}", topic, e);
                    return; 
                }
                log::info!("[libp2p_service][mesh-job] Subscribed to Gossipsub topic: {}", topic);

                loop {
                    tokio::select! {
                        swarm_event = swarm.next() => {
                            match swarm_event {
                                Some(SwarmEvent::Behaviour(Event::Gossipsub(e))) => {
                                    if let gossipsub::Event::Message{message, ..} = e {
                                        log::debug!("[libp2p_service][mesh-job] Received Gossipsub message from {:?}, topic: {:?}, data_len: {}", message.source, message.topic, message.data.len());
                                        match serde_json::from_slice::<super::NetworkMessage>(&message.data) {
                                            Ok(network_msg) => {
                                                log::debug!("[libp2p_service][mesh-job] Deserialized NetworkMessage: {:?}", network_msg);
                                                // Send to all subscribers
                                                let mut retain_senders = Vec::new();
                                                for sender in subscriber_senders.iter() {
                                                    if sender.send(network_msg.clone()).await.is_ok() {
                                                        retain_senders.push(sender.clone());
                                                    } else {
                                                        eprintln!("[libp2p] Failed to send message to a subscriber. Removing subscriber.");
                                                    }
                                                }
                                                subscriber_senders = retain_senders;
                                            }
                                            Err(err) => {
                                                eprintln!("[libp2p] Failed to deserialize gossip message: {}", err);
                                            }
                                        }
                                    }
                                }
                                Some(SwarmEvent::Behaviour(Event::Ping(event))) => {
                                    match event {
                                        ping::Event { peer, result: Ok(rtt), .. } => {
                                            println!("[libp2p] Ping to {}: RTT = {:?}", peer, rtt);
                                        }
                                        ping::Event { peer, result: Err(err), .. } => {
                                            eprintln!("[libp2p] Ping to {} failed: {:?}", peer, err);
                                        }
                                    }
                                }
                                Some(SwarmEvent::Behaviour(Event::Kademlia(event))) => {
                                    match event {
                                        KademliaEvent::OutboundQueryProgressed { id, result, .. } => {
                                            println!("[libp2p] Kademlia OutboundQueryProgressed: query_id={:?}, result={:?}", id, result);
                                            // TODO [libp2p_kad]: Handle all Kademlia QueryResult variants robustly,
                                            // especially errors and scenarios affecting job/peer discovery.
                                            match result {
                                                KademliaQueryResult::GetClosestPeers(Ok(data)) => {
                                                    if let Some(sender) = pending_kad_queries.remove(&id) {
                                                        println!("[libp2p] Found sender for query_id={:?}. Sending Ok(peers_count={}).", id, data.peers.len());
                                                        let peers = data.peers.into_iter().map(|p| super::PeerId(p.to_string())).collect();
                                                        if let Err(e) = sender.send(Ok(peers)) {
                                                            eprintln!("[libp2p] Failed to send Kademlia GetClosestPeers (Ok) result for query_id={:?}: {:?}", id, e);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia GetClosestPeers query_id={:?} (Ok data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::GetClosestPeers(Err(err)) => {
                                                    if let Some(sender) = pending_kad_queries.remove(&id) {
                                                        println!("[libp2p] Found sender for query_id={:?}. Sending Err({:?}).", id, err);
                                                        let common_err = CommonError::NetworkSetupError(format!("Kademlia GetClosestPeers query_id={:?} failed: {:?}", id, err));
                                                        if let Err(e) = sender.send(Err(common_err)) {
                                                            eprintln!("[libp2p] Failed to send Kademlia GetClosestPeers (Err) result for query_id={:?}: {:?}", id, e);
                                                        }
                                                    } else {
                                                         println!("[libp2p] Kademlia GetClosestPeers query_id={:?} (Err data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::PutRecord(Ok(PutRecordOk { key: _key })) => {
                                                    if let Some(sender) = pending_put_kad_queries.remove(&id) {
                                                        println!("[libp2p] Kademlia PutRecord successful for query_id={:?}", id);
                                                        if sender.send(Ok(())).is_err() {
                                                            eprintln!("[libp2p] Failed to send Kademlia PutRecord (Ok) result for query_id={:?}", id);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia PutRecord query_id={:?} (Ok data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::PutRecord(Err(err)) => {
                                                    if let Some(sender) = pending_put_kad_queries.remove(&id) {
                                                        println!("[libp2p] Kademlia PutRecord failed for query_id={:?}: {:?}", id, err);
                                                        let common_err = CommonError::NetworkSetupError(format!("Kademlia PutRecord query_id={:?} failed: {:?}", id, err));
                                                        if sender.send(Err(common_err)).is_err() {
                                                            eprintln!("[libp2p] Failed to send Kademlia PutRecord (Err) result for query_id={:?}", id);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia PutRecord query_id={:?} (Err data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                                                    if let Some(sender) = pending_get_kad_records.remove(&id) {
                                                        println!("[libp2p] Kademlia GetRecord FoundRecord for query_id={:?}", id);
                                                        if sender.send(Ok(Some(peer_record.record))).is_err() {
                                                            eprintln!("[libp2p] Failed to send Kademlia GetRecord (FoundRecord) result for query_id={:?}", id);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia GetRecord query_id={:?} (FoundRecord data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::GetRecord(Ok(GetRecordOk::FinishedWithNoAdditionalRecord{..})) => {
                                                    if let Some(sender) = pending_get_kad_records.remove(&id) {
                                                        println!("[libp2p] Kademlia GetRecord FinishedWithNoAdditionalRecord for query_id={:?}", id);
                                                        if sender.send(Ok(None)).is_err() {
                                                             eprintln!("[libp2p] Failed to send Kademlia GetRecord (FinishedWithNoAdditionalRecord) result for query_id={:?}", id);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia GetRecord query_id={:?} (FinishedWithNoAdditionalRecord data) had no sender or already handled.", id);
                                                    }
                                                }
                                                KademliaQueryResult::GetRecord(Err(err)) => {
                                                    if let Some(sender) = pending_get_kad_records.remove(&id) {
                                                        println!("[libp2p] Kademlia GetRecord failed for query_id={:?}: {:?}", id, err);
                                                        let common_err = CommonError::NetworkSetupError(format!("Kademlia GetRecord query_id={:?} failed: {:?}", id, err));
                                                        if sender.send(Err(common_err)).is_err() {
                                                            eprintln!("[libp2p] Failed to send Kademlia GetRecord (Err) result for query_id={:?}", id);
                                                        }
                                                    } else {
                                                        println!("[libp2p] Kademlia GetRecord query_id={:?} (Err data) had no sender or already handled.", id);
                                                    }
                                                }
                                                _ => {
                                                    // This case now also covers other GetClosestPeers results if any, or other query types.
                                                    println!("[libp2p] Kademlia OutboundQueryProgressed for query_id={:?}: Unhandled or already handled QueryResult type: {:?}", id, result);
                                                    // Check all pending maps if this query ID was expected for a different type.
                                                    if pending_kad_queries.contains_key(&id) {
                                                        println!("[libp2p] NOTE: Query_id={:?} for unhandled result is still in pending_kad_queries.", id);
                                                    }
                                                    if pending_put_kad_queries.contains_key(&id) {
                                                        println!("[libp2p] NOTE: Query_id={:?} for unhandled result is still in pending_put_kad_queries.", id);
                                                    }
                                                    if pending_get_kad_records.contains_key(&id) {
                                                        println!("[libp2p] NOTE: Query_id={:?} for unhandled result is still in pending_get_kad_records.", id);
                                                    }
                                                }
                                            }
                                        }
                                        KademliaEvent::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer } => {
                                            println!(
                                                "[libp2p] Kademlia RoutingUpdated: peer={}, is_new_peer={}, addresses={:?}, bucket_range={:?}, old_peer={:?}",
                                                peer, is_new_peer, addresses, bucket_range, old_peer
                                            );
                                            // TODO [libp2p_kad]: Potentially update local peer lists or trigger discovery based on routing updates.
                                        }
                                        KademliaEvent::UnroutablePeer { peer } => {
                                            println!("[libp2p] Kademlia UnroutablePeer: peer={}", peer);
                                            // TODO [libp2p_kad]: Handle unroutable peers, e.g., remove from active peer lists or attempt re-discovery.
                                        }
                                        KademliaEvent::RoutablePeer { peer, address } => {
                                            println!("[libp2p] Kademlia RoutablePeer: peer={}, address={}", peer, address);
                                            // TODO [libp2p_kad]: Add routable peers to a list of known good peers for direct interaction if needed.
                                        }
                                        KademliaEvent::PendingRoutablePeer { peer, address } => {
                                            println!("[libp2p] Kademlia PendingRoutablePeer: peer={}, address={}", peer, address);
                                            // TODO [libp2p_kad]: Monitor pending routable peers.
                                        }
                                        other_kad_event => {
                                            println!("[libp2p] Kademlia other event: {:?}", other_kad_event);
                                            // TODO [libp2p_kad]: Investigate and handle other Kademlia events as necessary.
                                        }
                                    }
                                }
                                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                                    println!("[libp2p] Listening on {address}");
                                    listening_addresses_clone.lock().unwrap().push(address);
                                }
                                Some(SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. }) => {
                                    println!("[libp2p] Incoming connection: local_addr={}, send_back_addr={}", local_addr, send_back_addr);
                                }
                                Some(SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. }) => {
                                    eprintln!(
                                        "[libp2p] Incoming connection error: local_addr={}, send_back_addr={}, error={}",
                                        local_addr, send_back_addr, error
                                    );
                                }
                                Some(SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in }) => {
                                    println!("[libp2p] Connection established with {} ({:?}) at {:?} ({:?}). Total: {}, Concurrent dials err: {:?}", peer_id, connection_id, endpoint, established_in, num_established, concurrent_dial_errors.map(|v| v.len()));
                                }
                                Some(SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause }) => {
                                     println!("[libp2p] Connection closed with {} ({:?}) at {:?}). Total: {}. Cause: {:?}", peer_id, connection_id, endpoint, num_established, cause);
                                }
                                Some(SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error }) => {
                                    eprintln!("[libp2p] Outgoing connection error to {:?} ({:?}): {}", peer_id, connection_id, error);
                                }
                                Some(event) => { 
                                    println!("[libp2p] Other SwarmEvent: {:?}", event);
                                }
                                None => break, 
                            }
                        }
                        Some(cmd) = cmd_rx.recv() => {
                            match cmd {
                                Command::DiscoverPeers { target, rsp } => {
                                    let local_id_for_fallback_logging = *swarm.local_peer_id(); // Keep for logging original intent if needed
                                    let query_target_peer_id = target.clone().unwrap_or_else(|| {
                                        let random_id = Libp2pPeerId::random();
                                        println!("[libp2p] DiscoverPeers CMD: No specific target, using random PeerId: {:?} for Kademlia query.", random_id);
                                        random_id
                                    });

                                    println!(
                                        "[libp2p] DiscoverPeers CMD: Effective Kademlia query target: {:?}. Original CMD target was: {:?} (fallback if None was local: {:?})", 
                                        query_target_peer_id, target, if target.is_none() { Some(local_id_for_fallback_logging) } else { None }
                                    );
                                    
                                    let query_id = swarm.behaviour_mut().kademlia.get_closest_peers(query_target_peer_id);
                                    println!("[libp2p] Inserting pending Kademlia query: id={:?} for effective target={:?} (original CMD target={:?})", query_id, query_target_peer_id, target);
                                    if pending_kad_queries.insert(query_id, rsp).is_some() {
                                        eprintln!("[libp2p] Kademlia query ID {:?} collision detected! Overwriting previous sender.", query_id);
                                    }
                                }
                                Command::GetRoutingTablePeers { rsp } => {
                                    let mut peers = Vec::new();
                                    for bucket in swarm.behaviour_mut().kademlia.kbuckets() {
                                        for entry in bucket.iter() {
                                            peers.push(super::PeerId(entry.node.key.preimage().to_string()));
                                        }
                                    }
                                    println!("[libp2p] GetRoutingTablePeers CMD: Found {} peers in Kademlia buckets.", peers.len());
                                    if let Err(e) = rsp.send(Ok(peers)) {
                                        eprintln!("[libp2p] Failed to send Kademlia routing table peers result: {:?}", e);
                                    }
                                }
                                Command::AddKadAddress { peer_id, addr } => {
                                    println!("[libp2p] AddKadAddress CMD: Adding address {:?} for peer {:?}", addr, peer_id);
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                    // This is fire and forget for the behaviour, no rsp channel needed here.
                                }
                                Command::TriggerKadBootstrap { rsp } => {
                                    println!("[libp2p] TriggerKadBootstrap CMD: Initiating Kademlia bootstrap...");
                                    match swarm.behaviour_mut().kademlia.bootstrap() {
                                        Ok(query_id) => {
                                            println!("[libp2p] Kademlia bootstrap initiated with query_id: {:?}", query_id);
                                            // Note: Bootstrap process is async. Actual success/failure comes via Kademlia events.
                                            // For simplicity, we acknowledge the command was accepted.
                                            // A more robust implementation might track this query_id for completion.
                                            let _ = rsp.send(Ok(())); 
                                        }
                                        Err(e) => {
                                            eprintln!("[libp2p] Kademlia bootstrap command failed immediately: {:?}", e);
                                            let _ = rsp.send(Err(CommonError::NetworkSetupError(format!("Kademlia bootstrap command failed: {:?}", e))));
                                        }
                                    }
                                }
                                Command::PutKadRecord { key, value, rsp } => {
                                    let record = KademliaRecord {
                                        key,
                                        value,
                                        publisher: None, // Libp2p fills this
                                        expires: None, // Kademlia default expiration
                                    };
                                    match swarm.behaviour_mut().kademlia.put_record(record, Quorum::One) {
                                        Ok(query_id) => {
                                            println!("[libp2p] Kademlia put_record initiated with query_id: {:?}", query_id);
                                            if pending_put_kad_queries.insert(query_id, rsp).is_some() {
                                                eprintln!("[libp2p] Kademlia put_record query ID {:?} collision! Overwriting previous sender.", query_id);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[libp2p] Kademlia put_record command failed immediately: {:?}", e);
                                            let _ = rsp.send(Err(CommonError::NetworkSetupError(format!("Kademlia put_record command failed: {:?}", e))));
                                        }
                                    }
                                }
                                Command::GetKadRecord { key, rsp } => {
                                    let query_id = swarm.behaviour_mut().kademlia.get_record(key); // Pass key by value
                                    println!("[libp2p] Kademlia get_record initiated with query_id: {:?}", query_id);
                                     if pending_get_kad_records.insert(query_id, rsp).is_some() {
                                        eprintln!("[libp2p] Kademlia get_record query ID {:?} collision! Overwriting previous sender.", query_id);
                                    }
                                    // Note: The original `get_record` on behavior takes `key: KademliaKeyRef<'_>` (a reference)
                                    // but the `Key` type itself is clonable and often used by value.
                                    // If type errors occur here, we might need to pass `&key`.
                                    // For now, assuming `key` (which is `libp2p::kad::record::Key`) can be moved or cloned as needed
                                    // by the `get_record` method, or the method signature in the `kad::Behaviour` is flexible.
                                    // Let's assume the `kad::Behaviour::get_record` takes `Key` by value or `&Key` and `key.clone()` is cheap.
                                    // The `libp2p::kad::Behaviour::get_record` method takes `key: Key` (a new Key, not a reference for some reason)
                                    // No, looking at the libp2p source, `get_record` takes `key: kad::record::Key` by value.
                                    // So, `let query_id = swarm.behaviour_mut().kademlia.get_record(key);` is correct.
                                }
                                Command::Broadcast { data } => {
                                    log::debug!("[libp2p_service][mesh-job] Broadcasting message (data_len: {}) to topic: {}", data.len(), topic);
                                    if let Err(e) = swarm.behaviour_mut()
                                        .gossipsub
                                        .publish(topic.clone(), data) {
                                        log::error!("[libp2p_service][mesh-job] Failed to publish to topic {}: {:?}", topic, e);
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

            Ok(Self { cmd_tx, local_peer_id, listening_addresses })
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
                .map_err(|e| CommonError::NetworkSetupError(format!("get_routing_table_peers cmd send error: {}", e.to_string())))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("get_routing_table_peers response dropped: {}", e.to_string())))?
        }

        pub async fn add_kad_address(&self, peer_id: Libp2pPeerId, addr: Multiaddr) -> Result<(), CommonError> {
            self.cmd_tx
                .send(Command::AddKadAddress { peer_id, addr }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("add_kad_address cmd send error: {}", e.to_string())))
        }

        pub async fn trigger_kad_bootstrap(&self) -> Result<(), CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::TriggerKadBootstrap { rsp: tx }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("trigger_kad_bootstrap cmd send error: {}", e.to_string())))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("trigger_kad_bootstrap response dropped: {}", e.to_string())))?
        }

        pub async fn put_kad_record(&self, key: KademliaKey, value: Vec<u8>) -> Result<(), CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::PutKadRecord { key, value, rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkSetupError(format!("put_kad_record cmd send error: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("put_kad_record response dropped: {}", e)))?
        }

        pub async fn get_kad_record(&self, key: KademliaKey) -> Result<Option<KademliaRecord>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::GetKadRecord { key, rsp: tx })
                .await
                .map_err(|e| CommonError::NetworkSetupError(format!("get_kad_record cmd send error: {}", e)))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("get_kad_record response dropped: {}", e)))?
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
                        Err(e) => return Err(CommonError::InvalidInputError(format!("Invalid target PeerId string '{}': {}", id_str, e))),
                    }
                }
                None => None,
            };

            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .send(Command::DiscoverPeers { target: target_libp2p_id, rsp: tx }).await
                .map_err(|e| CommonError::NetworkSetupError(format!("discover_peers cmd send error: {}", e.to_string())))?;
            rx.await.map_err(|e| CommonError::NetworkSetupError(format!("discover_peers response dropped: {}", e.to_string())))?
        }

        async fn send_message(
            &self,
            _peer: &super::PeerId, 
            msg: super::NetworkMessage, 
        ) -> Result<(), CommonError> {
            log::debug!("[libp2p_service][mesh-job] send_message (currently broadcasts) called with: {:?}", msg);
            let data = serde_json::to_vec(&msg)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("send_message cmd send error: {}", e.to_string())))
        }
        
        async fn broadcast_message(&self, message: super::NetworkMessage) -> Result<(), CommonError> {
            log::debug!("[libp2p_service][mesh-job] broadcast_message called with: {:?}", message);
            let data = serde_json::to_vec(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("broadcast_message cmd send error: {}", e.to_string())))
        }

        fn subscribe(&self) -> Result<mpsc::Receiver<super::NetworkMessage>, CommonError> {
            let (msg_tx, msg_rx) = mpsc::channel(128); // Channel for this subscriber
            
            // Send the sender half to the Swarm task
            // We use try_send here because we are in a sync method.
            // A better approach would be to make `subscribe` async or use a blocking send.
            // For simplicity now, if the command channel is full, this will fail.
            self.cmd_tx.try_send(Command::AddSubscriber { rsp_tx: msg_tx })
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to send AddSubscriber command: {}", e)))?;
            
            Ok(msg_rx)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
} // end mod
