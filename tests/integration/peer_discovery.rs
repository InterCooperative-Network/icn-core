#[cfg(feature = "enable-libp2p")]
mod peer_discovery {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    // ...existing code...
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn nodes_discover_each_other() {
        // ...existing code...
        let mut cfg_a = NetworkConfig::default();
        cfg_a.peer_discovery_interval = Duration::from_secs(2);
        let node_a = Libp2pNetworkService::new(cfg_a).await.expect("a start");
        sleep(Duration::from_secs(1)).await;
        let addr_a = node_a.listening_addresses()[0].clone();
        let peer_a = node_a.local_peer_id().clone();

        let mut cfg_b = NetworkConfig::default();
        cfg_b.bootstrap_peers = vec![(peer_a, addr_a.clone())];
        cfg_b.peer_discovery_interval = Duration::from_secs(2);
        cfg_b.bootstrap_interval = Duration::from_secs(2);
        let node_b = Libp2pNetworkService::new(cfg_b).await.expect("b start");

        sleep(Duration::from_secs(6)).await;

        let peers = node_b
            .discover_peers(Some(peer_a.to_string()))
            .await
            .expect("discover");
        assert!(peers.iter().any(|p| p.0 == peer_a.to_string()));

        node_a.shutdown().await.unwrap();
        node_b.shutdown().await.unwrap();
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn libp2p_disabled_discovery_stub() {
    println!("libp2p feature disabled; skipping discovery test");
}
