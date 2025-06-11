#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default
)]

#[cfg(feature = "experimental-libp2p")]
mod kademlia_peer_discovery_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService; // Import the trait
    use icn_network::PeerId as IcnPeerId; // Renamed to avoid confusion
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    #[ignore]
    async fn test_kademlia_two_node_peer_discovery() {
        println!("Starting Kademlia two_node_peer_discovery test...");

        // Node 1 Setup
        let config1 = NetworkConfig::default();
        let node1_service = Libp2pNetworkService::new(config1)
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
            .find(|addr| {
                addr.to_string().contains("127.0.0.1") || addr.to_string().contains("/::1/")
            }) // Prefer loopback
            .unwrap_or_else(|| {
                node1_addrs
                    .first()
                    .expect("Node 1 has no listen addresses at all")
            })
            .clone();
        println!(
            "Node 1 chosen listen address for Kademlia bootstrap: {}",
            node1_listen_addr_for_kad
        );

        // Node 2 Setup
        let mut config2 = NetworkConfig::default();
        config2.bootstrap_peers = vec![(node1_libp2p_peer_id.clone(), node1_listen_addr_for_kad)];
        let node2_service = Libp2pNetworkService::new(config2)
            .await
            .expect("Node 2 failed to start");
        let node2_libp2p_peer_id = node2_service.local_peer_id().clone();
        println!("Node 2 Libp2p Peer ID: {}", node2_libp2p_peer_id);

        // Allow time for Node 2 to establish listeners and connect to Node 1
        println!("Allowing Node 2 to initialize and connect (5 seconds)...");
        sleep(Duration::from_secs(5)).await;

        // Allow time for connections to establish
        println!("Allowing time for connection (5 seconds)...");
        sleep(Duration::from_secs(5)).await;

        // Verify that peer discovery via Kademlia works
        let discovered = node2_service
            .discover_peers(Some(node1_libp2p_peer_id.to_string()))
            .await
            .expect("discover");
        assert!(discovered
            .iter()
            .any(|p| p.0 == node1_libp2p_peer_id.to_string()));

        println!("Two node connectivity test finished successfully.");
    }

    #[tokio::test]
    #[ignore]
    async fn test_kademlia_record_exchange() {
        let config1 = NetworkConfig::default();
        let node1_service = Libp2pNetworkService::new(config1).await.expect("n1");
        let peer1 = node1_service.local_peer_id().clone();
        sleep(Duration::from_secs(2)).await;
        let addr1 = node1_service.listening_addresses()[0].clone();

        let mut config2 = NetworkConfig::default();
        config2.bootstrap_peers = vec![(peer1.clone(), addr1)];
        let node2_service = Libp2pNetworkService::new(config2).await.expect("n2");

        sleep(Duration::from_secs(5)).await;

        node2_service
            .put_kademlia_record("test-key", b"hello".to_vec())
            .await
            .expect("put");

        sleep(Duration::from_secs(5)).await;

        let rec = node1_service
            .get_kademlia_record("test-key")
            .await
            .expect("get");
        assert_eq!(rec.unwrap().value, b"hello".to_vec());
    }
}

#[cfg(feature = "experimental-libp2p")]
mod kademlia_three_node_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use std::time::Duration;
    use tokio::time::sleep;
    #[tokio::test]
    #[ignore]
    async fn test_three_node_basic_connectivity() {
        println!("Starting three node connectivity test...");
        let config1 = NetworkConfig::default();
        let node1_service = Libp2pNetworkService::new(config1)
            .await
            .expect("Node 1 failed to start");
        let node1_peer_id = node1_service.local_peer_id().clone();
        sleep(Duration::from_secs(2)).await;
        let node1_addr = node1_service.listening_addresses()[0].clone();

        let mut config2 = NetworkConfig::default();
        config2.bootstrap_peers = vec![(node1_peer_id.clone(), node1_addr.clone())];
        let node2_service = Libp2pNetworkService::new(config2)
            .await
            .expect("Node 2 failed to start");

        let mut config3 = NetworkConfig::default();
        config3.bootstrap_peers = vec![(node1_peer_id.clone(), node1_addr.clone())];
        let node3_service = Libp2pNetworkService::new(config3)
            .await
            .expect("Node 3 failed to start");

        sleep(Duration::from_secs(6)).await;

        let stats1 = node1_service.get_network_stats().await.expect("stats1");
        let stats2 = node2_service.get_network_stats().await.expect("stats2");
        let stats3 = node3_service.get_network_stats().await.expect("stats3");

        assert!(stats1.peer_count >= 2, "Node1 should see two peers");
        assert!(stats2.peer_count >= 1, "Node2 should see at least one peer");
        assert!(stats3.peer_count >= 1, "Node3 should see at least one peer");

        println!("Three node connectivity test finished successfully.");
    }
}
