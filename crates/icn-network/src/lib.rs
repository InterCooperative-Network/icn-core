#![doc = include_str!("../README.md")]

//! # ICN Network Crate
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! likely using libp2p. It covers P2P communication, transport protocols, peer discovery,
//! message routing, and federation synchronization.

use icn_common::{NodeInfo, CommonError, DagBlock, Cid, Did};
use serde::{Serialize, Deserialize};

// --- Peer and Message Scaffolding ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerId(pub String); // Placeholder, typically a libp2p PeerId

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    AnnounceBlock(DagBlock),
    RequestBlock(Cid),
    GossipSub(String, Vec<u8>), // topic, data
    FederationSyncRequest(Did), // Request sync from a federation representative DID
    // TODO: Add more message types as protocols develop
}

/// Placeholder for a network service trait.
/// TODO: Define methods for sending messages, discovering peers, subscribing to topics.
pub trait NetworkService {
    async fn discover_peers(&self, target_peer_id_str: Option<String>) -> Result<Vec<PeerId>, CommonError>;
    async fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError>;
    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError>;
    // fn subscribe_to_topic(&self, topic: &str) -> Result<(), CommonError>;
    // fn publish_to_topic(&self, topic: &str, data: Vec<u8>) -> Result<(), CommonError>;
}

/// Stub implementation for NetworkService.
#[derive(Default)]
pub struct StubNetworkService;

// TODO (#issue_url_for_libp2p_integration): Implement `Libp2pNetworkService` that uses a real libp2p stack.
// This service should be conditionally compiled when the `with-libp2p` feature is enabled.
// It will involve managing Swarm, Behaviours (e.g., Kademlia, Gossipsub), and transport configurations.

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
        kad,
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

    /* ---------- Public façade ------------------------------------------------ */

    #[derive(Clone)]
    pub struct Libp2pNetworkService {
        cmd_tx: mpsc::Sender<Command>,
        local_peer_id: Libp2pPeerId,
        listening_addresses: Arc<Mutex<Vec<Multiaddr>>>,
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

            let store = kad::store::MemoryStore::new(local_peer_id);
            let kademlia_config = kad::Config::default();
            let kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

            #[derive(NetworkBehaviour)]
            #[behaviour(out_event = "Event")]
            struct Behaviour {
                gossipsub: gossipsub::Behaviour,
                ping: ping::Behaviour,
                kademlia: kad::Behaviour<kad::store::MemoryStore>,
            }

            #[allow(clippy::large_enum_variant)]
            #[derive(Debug)]
            enum Event {
                Gossipsub(gossipsub::Event),
                Ping(ping::Event),
                Kademlia(kad::Event),
            }

            impl From<gossipsub::Event> for Event {
                fn from(e: gossipsub::Event) -> Self { Event::Gossipsub(e) }
            }
            impl From<ping::Event> for Event {
                fn from(e: ping::Event) -> Self { Event::Ping(e) }
            }
            impl From<kad::Event> for Event {
                fn from(e: kad::Event) -> Self { Event::Kademlia(e) }
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
            let mut pending_kad_queries = HashMap::<kad::QueryId, oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>>::new();

            task::spawn(async move {
                let topic = gossipsub::IdentTopic::new("icn-default");
                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    eprintln!("[libp2p] Failed to subscribe to topic {}: {:?}", topic, e);
                    return; 
                }

                loop {
                    tokio::select! {
                        swarm_event = swarm.next() => {
                            match swarm_event {
                                Some(SwarmEvent::Behaviour(Event::Gossipsub(e))) => {
                                    if let gossipsub::Event::Message{message, ..} = e {
                                        println!("[libp2p] Got gossip message: {:?}", message.data);
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
                                        kad::Event::OutboundQueryProgressed { id, result, .. } => {
                                            println!("[libp2p] Kademlia OutboundQueryProgressed: query_id={:?}, result={:?}", id, result);
                                            if !pending_kad_queries.contains_key(&id) {
                                                println!("[libp2p] WARNING: No pending Kademlia query found in map for id={:?}! This event won't be sent to a discover_peers caller.", id);
                                            }

                                            match result {
                                                kad::QueryResult::GetClosestPeers(Ok(data)) => {
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
                                                kad::QueryResult::GetClosestPeers(Err(err)) => {
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
                                                _ => {
                                                    println!("[libp2p] Kademlia OutboundQueryProgressed for query_id={:?}: Unhandled QueryResult type: {:?}", id, result);
                                                    if pending_kad_queries.contains_key(&id) {
                                                         println!("[libp2p] NOTE: Query_id={:?} for unhandled result is still in pending_kad_queries. Potential hang if not processed further by other Kademlia events.", id);
                                                    }
                                                }
                                            }
                                        }
                                        kad::Event::RoutingUpdated { peer, is_new_peer, addresses, bucket_range, old_peer } => {
                                            println!(
                                                "[libp2p] Kademlia RoutingUpdated: peer={}, is_new_peer={}, addresses={:?}, bucket_range={:?}, old_peer={:?}",
                                                peer, is_new_peer, addresses, bucket_range, old_peer
                                            );
                                        }
                                        kad::Event::UnroutablePeer { peer } => {
                                            println!("[libp2p] Kademlia UnroutablePeer: peer={}", peer);
                                        }
                                        kad::Event::RoutablePeer { peer, address } => {
                                            println!("[libp2p] Kademlia RoutablePeer: peer={}, address={}", peer, address);
                                        }
                                        kad::Event::PendingRoutablePeer { peer, address } => {
                                            println!("[libp2p] Kademlia PendingRoutablePeer: peer={}, address={}", peer, address);
                                        }
                                        other_kad_event => {
                                            println!("[libp2p] Kademlia other event: {:?}", other_kad_event);
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
                                    let local_id_for_fallback = *swarm.local_peer_id();
                                    let query_target_peer_id = target.clone().unwrap_or(local_id_for_fallback);
                                    println!("[libp2p] DiscoverPeers CMD: Will query Kademlia for effective target: {:?}. Original CMD target was: {:?}", query_target_peer_id, target);
                                    let query_id = swarm.behaviour_mut().kademlia.get_closest_peers(query_target_peer_id);
                                    println!("[libp2p] Inserting pending Kademlia query: id={:?} for CMD target={:?}", query_id, target);
                                    if pending_kad_queries.insert(query_id, rsp).is_some() {
                                        eprintln!("[libp2p] Kademlia query ID {:?} collision detected! Overwriting previous sender.", query_id);
                                    }
                                }
                                Command::Broadcast { data } => {
                                    if let Err(e) = swarm.behaviour_mut()
                                        .gossipsub
                                        .publish(topic.clone(), data) {
                                        eprintln!("[libp2p] Failed to publish to topic {}: {:?}", topic, e);
                                    }
                                }
                            }
                        }
                        else => break, 
                    }
                }
                println!("[libp2p] Swarm task ended.");
            });

            Ok(Self { cmd_tx, local_peer_id, listening_addresses })
        }

        pub fn local_peer_id(&self) -> &Libp2pPeerId {
            &self.local_peer_id
        }

        pub fn listening_addresses(&self) -> Vec<Multiaddr> {
            self.listening_addresses.lock().unwrap().clone()
        }
    }

    /* ---------- internal command enum -------------------------------------- */

    #[derive(Debug)]
    enum Command {
        DiscoverPeers { 
            target: Option<Libp2pPeerId>, 
            rsp: oneshot::Sender<Result<Vec<super::PeerId>, CommonError>> 
        },
        Broadcast { data: Vec<u8> },
    }

    /* ---------- hook into crate-level trait -------------------------------- */

    impl super::NetworkService for Libp2pNetworkService {
        async fn discover_peers(
            &self,
            target_peer_id_str: Option<String>,
        ) -> Result<Vec<super::PeerId>, CommonError> {
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
            let data = serde_json::to_vec(&msg)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("send_message cmd send error: {}", e.to_string())))
        }
        
        async fn broadcast_message(&self, message: super::NetworkMessage) -> Result<(), CommonError> {
            let data = serde_json::to_vec(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .send(Command::Broadcast { data }).await
                .map_err(|e| CommonError::MessageSendError(format!("broadcast_message cmd send error: {}", e.to_string())))
        }
    }
} // end mod
