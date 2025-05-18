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
    use super::{NetworkMessage, CommonError};
    // use icn_common::NodeInfo;

    use libp2p::{
        core::upgrade,
        futures::StreamExt,
        gossipsub, identify, kad,
        noise,
        swarm::{NetworkBehaviour, SwarmEvent},
        tcp, yamux,
        // Multiaddr, // Removed, used directly via libp2p::Multiaddr
        PeerId as Libp2pPeerId, // Alias to avoid confusion with our PeerId
        Transport,
        SwarmBuilder, // Corrected import location
    };
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Duration;
    use tokio::sync::mpsc; // For passing messages from swarm to service
    // use tokio::time::sleep; // Removed, not used

    // Define the custom NetworkBehaviour
    #[derive(NetworkBehaviour)]
    #[behaviour(to_swarm = "IcnBehaviourEvent")]
    pub struct IcnNetworkBehaviour {
        pub gossipsub: gossipsub::Behaviour,
        pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
        pub identify: identify::Behaviour,
        pub ping: libp2p::ping::Behaviour,
    }

    // Events emitted by our behaviour to be processed by the Swarm owner
    #[allow(clippy::large_enum_variant)] // Kademlia event can be large
    #[derive(Debug)] // Added Debug
    pub enum IcnBehaviourEvent {
        Gossipsub(gossipsub::Event),
        Kademlia(kad::Event),
        Identify(identify::Event),
        Ping(libp2p::ping::Event),
    }

    impl From<gossipsub::Event> for IcnBehaviourEvent {
        fn from(event: gossipsub::Event) -> Self {
            IcnBehaviourEvent::Gossipsub(event)
        }
    }

    impl From<kad::Event> for IcnBehaviourEvent {
        fn from(event: kad::Event) -> Self {
            IcnBehaviourEvent::Kademlia(event)
        }
    }

    impl From<identify::Event> for IcnBehaviourEvent {
        fn from(event: identify::Event) -> Self {
            IcnBehaviourEvent::Identify(event)
        }
    }

    impl From<libp2p::ping::Event> for IcnBehaviourEvent {
        fn from(event: libp2p::ping::Event) -> Self {
            IcnBehaviourEvent::Ping(event)
        }
    }

    pub struct Libp2pNetworkService {
        // swarm: Swarm<IcnNetworkBehaviour>, // Swarm will be run in a separate task
        command_sender: mpsc::Sender<Libp2pCommand>,
        local_peer_id: Libp2pPeerId,
    }

    // Commands to interact with the libp2p swarm task
    enum Libp2pCommand {
        SendMessage(Libp2pPeerId, NetworkMessage),
        BroadcastMessage(NetworkMessage),
        DiscoverPeers,
        GetDiscoveredPeers(tokio::sync::oneshot::Sender<Result<Vec<super::PeerId>, CommonError>>),
        // Add more commands as needed, e.g., for subscribing/publishing to topics
    }

    impl Libp2pNetworkService {
        pub async fn new() -> Result<Self, CommonError> {
            // Create a new identity (keypair) for this node.
            let local_key = libp2p::identity::Keypair::generate_ed25519();
            let local_peer_id = Libp2pPeerId::from(local_key.public());
            println!("[Libp2pNetworkService] Local Peer ID: {}", local_peer_id);

            // Build the transport
            // TODO: Make transport configurable (e.g. QUIC, WebRTC)
            let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
                .upgrade(upgrade::Version::V1Lazy)
                .authenticate(noise::Config::new(&local_key).expect("Failed to create noise config"))
                .multiplex(yamux::Config::default())
                .timeout(std::time::Duration::from_secs(20))
                .boxed();

            // Create a Kademlia behaviour.
            let store = kad::store::MemoryStore::new(local_peer_id);
            let kademlia = kad::Behaviour::new(local_peer_id, store);

            // Create a Gossipsub behaviour with a message authenticator
            let gossipsub_message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // TODO: Make configurable
                .validation_mode(gossipsub::ValidationMode::Strict) // Enforce message signing if applicable
                .message_id_fn(gossipsub_message_id_fn) // Use custom message ID function
                .build()
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to build gossipsub config: {}", e)))?;
            
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()), // Or Anonymous if not signing
                gossipsub_config,
            ).map_err(|e| CommonError::NetworkSetupError(format!("Failed to create gossipsub behaviour: {}", e)))?;

            let identify_config = identify::Config::new(
                "/icn/0.1.0".to_string(), // Protocol version
                local_key.public(),
            );

            // Create the Swarm
            let behaviour = IcnNetworkBehaviour {
                gossipsub,
                kademlia,
                identify: identify::Behaviour::new(identify_config),
                ping: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()), // Default ping config
            };

            let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
                .with_tokio()
                .with_other_transport(transport)
                .expect("Failed to configure transport")
                .with_behaviour(behaviour)
                .expect("Failed to configure behaviour")
                .build();

            // Example: Listen on a specific address (TODO: make configurable)
            let listen_addr_str = "/ip4/0.0.0.0/tcp/0"; // Listen on all interfaces, random port
            let listen_addr: libp2p::Multiaddr = listen_addr_str.parse().expect("Failed to parse listen address");
            swarm.listen_on(listen_addr.clone())
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to listen on {}: {}", listen_addr_str, e)))?;

            let (command_sender, mut command_receiver) = mpsc::channel(32); // Channel for commands

            // Spawn the swarm task
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        event = swarm.select_next_some() => {
                            match event {
                                SwarmEvent::Behaviour(IcnBehaviourEvent::Identify(event)) => {
                                    println!("[SwarmTask] Identify event: {:?}", event);
                                    if let identify::Event::Received { peer_id, info } = event {
                                        println!("[SwarmTask] Received Identify from {}: agent version: {}, protocol version: {}", 
                                                 peer_id, info.agent_version, info.protocol_version);
                                        // Add known addresses to Kademlia
                                        for addr in info.listen_addrs {
                                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                        }
                                    }
                                }
                                SwarmEvent::Behaviour(IcnBehaviourEvent::Gossipsub(event)) => {
                                    if let gossipsub::Event::Message { propagation_source, message_id, message } = event {
                                        println!(
                                            "[SwarmTask] Got GCP message: \"{}\" with id: {} from peer: {:?}",
                                            String::from_utf8_lossy(&message.data),
                                            message_id,
                                            propagation_source
                                        );
                                        // TODO: Process gossipsub message (e.g., deserialize NetworkMessage, pass to handler)
                                    }
                                }
                                SwarmEvent::Behaviour(IcnBehaviourEvent::Kademlia(event)) => {
                                    // println!("[SwarmTask] Kademlia event: {:?}", event);
                                    // TODO: Handle Kademlia events (e.g., routing table updates, query results)
                                }
                                SwarmEvent::Behaviour(IcnBehaviourEvent::Ping(event)) => {
                                     println!("[SwarmTask] Ping event: {:?}", event);
                                }
                                SwarmEvent::NewListenAddr { listener_id, address } => {
                                    println!("[SwarmTask] Listener {:?} started, local node listening on: {}", listener_id, address);
                                }
                                SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                                    println!("[SwarmTask] Incoming connection: local_addr={}, send_back_addr={}", local_addr, send_back_addr);
                                }
                                SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                                    eprintln!(
                                        "[SwarmTask] Incoming connection error: local_addr={}, send_back_addr={}, error={}",
                                        local_addr, send_back_addr, error
                                    );
                                }
                                // Add other SwarmEvent arms as needed
                                _ => { println!("[SwarmTask] Unhandled SwarmEvent: {:?}", event); }
                            }
                        }
                        Some(command) = command_receiver.recv() => {
                            match command {
                                Libp2pCommand::SendMessage(peer_id, msg) => {
                                    // TODO: Serialize NetworkMessage to bytes for gossipsub or direct send
                                    // For now, assuming direct send is not the primary path for NetworkMessage
                                    // and Gossipsub is used via topics.
                                    println!("[SwarmTask] TODO: SendMessage command for peer {} (msg: {:?}) - not fully implemented for direct send", peer_id, msg);
                                    // Example: if it were raw bytes for a specific protocol
                                    // swarm.behaviour_mut().request_response.send_request(&peer_id, YourRequestType::new(bytes));
                                }
                                Libp2pCommand::BroadcastMessage(msg) => {
                                    // TODO: Determine topic, serialize NetworkMessage to bytes
                                    if let NetworkMessage::GossipSub(topic, data) = msg {
                                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(gossipsub::IdentTopic::new(topic.clone()), data) {
                                            eprintln!("[SwarmTask] Failed to publish to topic {}: {:?}", topic, e);
                                        }
                                    } else {
                                        eprintln!("[SwarmTask] BroadcastMessage command received non-GossipSub message: {:?}. Cannot broadcast.", msg);
                                    }
                                }
                                Libp2pCommand::DiscoverPeers => {
                                    // Example: Start a Kademlia random walk
                                    swarm.behaviour_mut().kademlia.get_closest_peers(Libp2pPeerId::random());
                                    println!("[SwarmTask] DiscoverPeers command: Kademlia random walk initiated.");
                                }
                                Libp2pCommand::GetDiscoveredPeers(sender) => {
                                    let mut peers = Vec::new();
                                    for b in swarm.behaviour_mut().kademlia.kbuckets() {
                                        for e in b.iter() {
                                            peers.push(super::PeerId(e.node.key.preimage().to_string())); 
                                        }
                                    }
                                    let _ = sender.send(Ok(peers));
                                }
                            }
                        }
                    }
                }
            });

            Ok(Libp2pNetworkService { command_sender, local_peer_id })
        }
    }

    // TODO: Implement NetworkService trait for Libp2pNetworkService
    // impl super::NetworkService for Libp2pNetworkService {
    //     fn discover_peers(&self, bootstrap_nodes: Vec<String>) -> Result<Vec<super::PeerId>, CommonError> {
    //         // Send command to swarm task, wait for response via oneshot channel
    //         // let (tx, rx) = tokio::sync::oneshot::channel();
    //         // self.command_sender.try_send(Libp2pCommand::GetDiscoveredPeers(tx)).map_err(|e| CommonError::MessageSendError(format!("Failed to send GetDiscoveredPeers command: {}", e)))?;
    //         // tokio::runtime::Handle::current().block_on(rx).map_err(|e| CommonError::MessageReceiveError(format!("Failed to receive peer list: {}",e)))?
    //         unimplemented!("discover_peers not yet implemented for Libp2pNetworkService")
    //     }
    //     fn send_message(&self, peer: &super::PeerId, message: NetworkMessage) -> Result<(), CommonError> {
    //         // Convert super::PeerId to Libp2pPeerId if necessary
    //         // let libp2p_peer_id = Libp2pPeerId::from_bytes(&bs58::decode(&peer.0).into_vec().map_err(|e| CommonError::InvalidInputError(format!("Invalid PeerId string: {}",e)))?[..]).map_err(|_| CommonError::InvalidInputError("Invalid PeerId bytes".to_string()))?;
    //         // self.command_sender.try_send(Libp2pCommand::SendMessage(libp2p_peer_id, message)).map_err(|e| CommonError::MessageSendError(format!("Failed to send SendMessage command: {}", e)))?;
    //         // Ok(())
    //         unimplemented!("send_message not yet implemented for Libp2pNetworkService")
    //     }
    //     fn broadcast_message(&self, message: NetworkMessage) -> Result<(), CommonError> {
    //         // self.command_sender.try_send(Libp2pCommand::BroadcastMessage(message)).map_err(|e| CommonError::MessageSendError(format!("Failed to send BroadcastMessage command: {}", e)))?;
    //         // Ok(())
    //         unimplemented!("broadcast_message not yet implemented for Libp2pNetworkService")
    //     }
    // }

} // Closes: mod libp2p_service
