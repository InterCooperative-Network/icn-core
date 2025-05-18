#[cfg(feature = "experimental-libp2p")]
mod kademlia_peer_discovery_tests {
    use icn_network::libp2p_service::Libp2pNetworkService;
    use icn_network::PeerId as IcnPeerId; // Renamed to avoid confusion
    use icn_network::NetworkService; // Import the trait
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_kademlia_two_node_peer_discovery() {
        println!("Starting Kademlia two_node_peer_discovery test...");

        // Node 1 Setup
        let node1_service = Libp2pNetworkService::new(None)
            .await
            .expect("Node 1 failed to start");
        let node1_libp2p_peer_id = node1_service.local_peer_id().clone();
        println!("Node 1 Libp2p Peer ID: {}", node1_libp2p_peer_id);

        // Allow Node 1 to establish listeners
        sleep(Duration::from_secs(2)).await; // Increased slightly for stability
        let node1_addrs = node1_service.listening_addresses();
        assert!(!node1_addrs.is_empty(), "Node 1 has no listening addresses");
        
        let node1_listen_addr_for_kad = node1_addrs
            .iter()
            .find(|addr| addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/")) // Prefer loopback
            .unwrap_or_else(|| node1_addrs.first().expect("Node 1 has no listen addresses at all"))
            .clone();
        println!("Node 1 chosen listen address for Kademlia bootstrap: {}", node1_listen_addr_for_kad);

        // Node 2 Setup
        let bootstrap_info_for_node2 = Some(vec![(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad)]);
        let node2_service = Libp2pNetworkService::new(bootstrap_info_for_node2)
            .await
            .expect("Node 2 failed to start");
        let node2_libp2p_peer_id = node2_service.local_peer_id().clone();
        println!("Node 2 Libp2p Peer ID: {}", node2_libp2p_peer_id);
        
        // Allow time for Node 2 to connect to Node 1 and for Kademlia to exchange information.
        // Kademlia discovery can take some time.
        println!("Nodes started. Allowing time for Kademlia discovery (20 seconds)...");
        sleep(Duration::from_secs(20)).await;

        // Node 2 discovers peers
        println!("Node 2 attempting to discover peers...");
        let discovered_peers_on_node2 = node2_service
            .discover_peers(Vec::new()) // Pass Vec::new() as per current trait signature
            .await
            .expect("Node 2 discover_peers failed");
        
        println!("Node 2 discovered peers: {:?}", discovered_peers_on_node2.iter().map(|p| &p.0).collect::<Vec<_>>());

        let node1_icn_peer_id = IcnPeerId(node1_libp2p_peer_id.to_string());
        assert!(
            discovered_peers_on_node2.contains(&node1_icn_peer_id),
            "Node 2 should have discovered Node 1. Expected: {}, Found: {:?}", node1_icn_peer_id.0, discovered_peers_on_node2
        );
        println!("Node 2 successfully discovered Node 1 ({}) via Kademlia.", node1_icn_peer_id.0);

        // Optionally, Node 1 discovers peers (Node 2)
        println!("Node 1 attempting to discover peers...");
        let discovered_peers_on_node1 = node1_service
            .discover_peers(Vec::new()) // Pass Vec::new() as per current trait signature
            .await
            .expect("Node 1 discover_peers failed");

        println!("Node 1 discovered peers: {:?}", discovered_peers_on_node1.iter().map(|p| &p.0).collect::<Vec<_>>());
        
        let node2_icn_peer_id = IcnPeerId(node2_libp2p_peer_id.to_string());
        assert!(
            discovered_peers_on_node1.contains(&node2_icn_peer_id),
            "Node 1 should have discovered Node 2. Expected: {}, Found: {:?}", node2_icn_peer_id.0, discovered_peers_on_node1
        );
        println!("Node 1 successfully discovered Node 2 ({}) via Kademlia.", node2_icn_peer_id.0);

        println!("Kademlia two_node_peer_discovery test finished successfully.");
    }
} 