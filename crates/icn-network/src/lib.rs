#![doc = include_str!("../README.md")]

//! # ICN Network Crate
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! likely using libp2p. It covers P2P communication, transport protocols, peer discovery,
//! message routing, and federation synchronization.

use icn_common::{NodeInfo, CommonError, DagBlock, Cid, Did};
use serde::{Serialize, Deserialize};

// --- Peer and Message Scaffolding ---

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn discover_peers(&self, bootstrap_nodes: Vec<String>) -> Result<Vec<PeerId>, CommonError>;
    fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError>;
    fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError>;
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
    fn discover_peers(&self, bootstrap_nodes: Vec<String>) -> Result<Vec<PeerId>, CommonError> {
        println!("[StubNetworkService] Discovering peers (bootstrap: {:?})... returning mock peers.", bootstrap_nodes);
        if bootstrap_nodes.is_empty() && true { // Simulate an error condition for an empty bootstrap list for demonstration
            // For a real implementation, this might be a configuration error or a different specific error.
            // Here, using a generic NetworkConnectionError to illustrate.
            // return Err(CommonError::InvalidInputError("Bootstrap node list cannot be empty.".to_string()));
        }
        Ok(vec![PeerId("mock_peer_1".to_string()), PeerId("mock_peer_2".to_string())])
    }

    fn send_message(&self, peer: &PeerId, message: NetworkMessage) -> Result<(), CommonError> {
        println!("[StubNetworkService] Sending message to peer {:?}: {:?}", peer, message);
        if peer.0 == "error_peer" { // Simulate a peer that causes an error
            return Err(CommonError::MessageSendError(format!("Failed to send message to peer: {}", peer.0)));
        }
        if peer.0 == "unknown_peer_id" { // Simulate peer not found
            return Err(CommonError::PeerNotFound(format!("Peer with ID {} not found.", peer.0)));
        }
        Ok(())
    }

    fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError> {
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
pub fn send_network_ping(info: &NodeInfo, target_peer: &str) -> Result<String, CommonError> {
    let service = StubNetworkService::default();
    let _ = service.send_message(&PeerId(target_peer.to_string()), NetworkMessage::GossipSub("ping_topic".to_string(), vec![1,2,3]));
    Ok(format!("Sent (stubbed) ping to {} from node: {} (v{})", target_peer, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_network_service_discover() {
        let service = StubNetworkService::default();
        let peers = service.discover_peers(vec!["/ip4/127.0.0.1/tcp/12345".to_string()]).unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].0, "mock_peer_1");

        // Test a case that might be an error (though stub currently doesn't error on empty list here)
        // let result_empty_bootstrap = service.discover_peers(Vec::new());
        // assert!(matches!(result_empty_bootstrap, Err(CommonError::InvalidInputError(_))));
    }

    #[test]
    fn test_stub_network_service_send_broadcast() {
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

        assert!(service.send_message(&peer_ok, message_announce.clone()).is_ok());
        
        let send_error_result = service.send_message(&peer_error, message_announce.clone());
        assert!(matches!(send_error_result, Err(CommonError::MessageSendError(_))));

        let send_unknown_result = service.send_message(&peer_unknown, message_request.clone());
        assert!(matches!(send_unknown_result, Err(CommonError::PeerNotFound(_))));

        assert!(service.broadcast_message(message_announce).is_ok());

        let broadcast_error_message = NetworkMessage::GossipSub("system_critical_error_topic".to_string(), vec![]);
        let broadcast_error_result = service.broadcast_message(broadcast_error_message);
        assert!(matches!(broadcast_error_result, Err(CommonError::NetworkUnhealthy(_))));
    }

    #[test]
    fn test_send_network_ping_uses_stub() {
        let node_info = NodeInfo {
            name: "NetNodePing".to_string(),
            version: "0.1.0".to_string(),
            status_message: "Network ping test".to_string(),
        };
        let result = send_network_ping(&node_info, "peer-xyz-ping");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("peer-xyz-ping"));
        // Output will show [StubNetworkService] logs if test is run with --nocapture
    }
}

// --- Libp2p Network Service Implementation ---

#[cfg(feature = "experimental-libp2p")]
mod libp2p_service {
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
        yamux, PeerId as Libp2pPeerId, Transport,
        dns,
    };
    use std::time::Duration;
    use tokio::{sync::{mpsc, oneshot}, task};
    use serde_json;

    /* ---------- Public façade ------------------------------------------------ */

    #[derive(Clone)]
    pub struct Libp2pNetworkService {
        cmd_tx: mpsc::Sender<Command>,
        local_peer: Libp2pPeerId,
    }

    impl Libp2pNetworkService {
        pub async fn new() -> Result<Self, CommonError> {
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer = Libp2pPeerId::from(local_key.public());

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

            #[derive(NetworkBehaviour)]
            #[behaviour(out_event = "Event")]
            struct Behaviour {
                gossipsub: gossipsub::Behaviour,
                ping: ping::Behaviour,
            }

            #[allow(clippy::large_enum_variant)]
            #[derive(Debug)]
            enum Event {
                Gossipsub(gossipsub::Event),
                Ping(ping::Event),
            }

            impl From<gossipsub::Event> for Event {
                fn from(e: gossipsub::Event) -> Self { Event::Gossipsub(e) }
            }
            impl From<ping::Event> for Event {
                fn from(e: ping::Event) -> Self { Event::Ping(e) }
            }

            let behaviour = Behaviour { gossipsub, ping };
            
            let swarm_config = SwarmConfig::with_tokio_executor();
            let mut swarm = Swarm::new(transport, behaviour, local_peer, swarm_config);

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

            let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(16);

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
                                        println!("[libp2p] got gossip: {:?}", message.data);
                                    }
                                }
                                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                                    println!("[libp2p] listening on {address}");
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
                                Command::DiscoverPeers { rsp } => {
                                    let _ = rsp.send(vec![super::PeerId(local_peer.to_string())]);
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

            Ok(Self { cmd_tx, local_peer })
        }
    }

    /* ---------- internal command enum -------------------------------------- */

    #[derive(Debug)]
    enum Command {
        DiscoverPeers { rsp: oneshot::Sender<Vec<super::PeerId>> },
        Broadcast { data: Vec<u8> },
    }

    /* ---------- hook into crate-level trait -------------------------------- */

    impl super::NetworkService for Libp2pNetworkService {
        fn discover_peers(
            &self,
            _bootstrap: Vec<String>,
        ) -> Result<Vec<super::PeerId>, CommonError> {
            let (tx, rx) = oneshot::channel();
            self.cmd_tx
                .blocking_send(Command::DiscoverPeers { rsp: tx })
                .map_err(|e| CommonError::NetworkSetupError(format!("discover_peers cmd send error: {}", e.to_string())))?;
            rx.blocking_recv().map_err(|e| CommonError::NetworkSetupError(format!("discover_peers response dropped: {}", e.to_string())))
        }

        fn send_message(
            &self,
            _peer: &super::PeerId, 
            msg: super::NetworkMessage, 
        ) -> Result<(), CommonError> {
            let data = serde_json::to_vec(&msg)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .blocking_send(Command::Broadcast { data })
                .map_err(|e| CommonError::MessageSendError(format!("send_message cmd send error: {}", e.to_string())))
        }
        
        fn broadcast_message(&self, message: super::NetworkMessage) -> Result<(), CommonError> {
            let data = serde_json::to_vec(&message)
                .map_err(|e| CommonError::SerializationError(e.to_string()))?;
            self.cmd_tx
                .blocking_send(Command::Broadcast { data })
                .map_err(|e| CommonError::MessageSendError(format!("broadcast_message cmd send error: {}", e.to_string())))
        }
    }
} // end mod
