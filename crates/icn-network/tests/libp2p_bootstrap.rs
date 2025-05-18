#[cfg(feature = "experimental-libp2p")]
mod libp2p_bootstrap_tests {
    use icn_network::libp2p_service::Libp2pNetworkService;
    // libp2p::{Multiaddr, PeerId as Libp2pPeerId} are not strictly needed if types are inferred
    // but can be kept for clarity if preferred.
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_two_nodes_discover_each_other() {
        println!("Starting test_two_nodes_discover_each_other...");

        let node1_service = Libp2pNetworkService::new(None).await.expect("Node 1 failed to start");
        let node1_peer_id = node1_service.local_peer_id().clone();
        println!("Node 1 Peer ID: {}", node1_peer_id);

        sleep(Duration::from_secs(1)).await;
        let node1_addrs = node1_service.listening_addresses();
        assert!(!node1_addrs.is_empty(), "Node 1 has no listening addresses");
        
        // For Kademlia add_address, we need the PeerId and one of its listen Multiaddrs (without the /p2p suffix).
        let node1_listen_addr_for_kad = node1_addrs.iter()
            .find(|addr| addr.to_string().contains("127.0.0.1")) // Prefer a loopback for local tests
            .unwrap_or_else(|| node1_addrs.first().expect("Node 1 has no listen addresses at all"))
            .clone();

        println!("Node 1 Peer ID: {}, chosen listen address for Kademlia bootstrap: {}", node1_peer_id, node1_listen_addr_for_kad);

        let bootstrap_info_for_node2 = Some(vec![(node1_peer_id, node1_listen_addr_for_kad)]);
        
        let _node2_service = Libp2pNetworkService::new(bootstrap_info_for_node2).await.expect("Node 2 failed to start");
        // let node2_peer_id = node2_service.local_peer_id().clone(); // Mark as unused if not used further
        // println!("Node 2 Peer ID: {}", node2_peer_id);

        println!("Nodes started. Allowing time for discovery (e.g., 15 seconds). Check logs for Kademlia events...");
        sleep(Duration::from_secs(15)).await;

        println!("Test finished. Inspect logs for Kademlia discovery events involving both nodes.");
    }
} 