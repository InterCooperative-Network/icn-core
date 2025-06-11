#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default
)]

#[cfg(feature = "libp2p")]
mod reconnect_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_reconnect_after_restart() {
        let listen_addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/34001".parse().unwrap();
        let config1 = NetworkConfig {
            listen_addresses: vec![listen_addr.clone()],
            ..NetworkConfig::default()
        };
        let node1 = Libp2pNetworkService::new(config1).await.expect("n1");
        let peer1 = *node1.local_peer_id();

        // node2 with small bootstrap interval
        let config2 = NetworkConfig {
            bootstrap_peers: vec![(peer1, listen_addr.clone())],
            bootstrap_interval: Duration::from_secs(2),
            ..NetworkConfig::default()
        };
        let node2 = Libp2pNetworkService::new(config2).await.expect("n2 start");

        sleep(Duration::from_secs(3)).await;
        assert!(node2.get_network_stats().await.unwrap().peer_count > 0);

        node2.shutdown().await.unwrap();

        // restart node2
        let config2b = NetworkConfig {
            bootstrap_peers: vec![(peer1, listen_addr.clone())],
            bootstrap_interval: Duration::from_secs(2),
            ..NetworkConfig::default()
        };
        let node2b = Libp2pNetworkService::new(config2b).await.expect("n2b");

        sleep(Duration::from_secs(5)).await;
        assert!(node2b.get_network_stats().await.unwrap().peer_count > 0);

        node1.shutdown().await.unwrap();
        node2b.shutdown().await.unwrap();
    }
}
