#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default
)]

#[cfg(feature = "libp2p")]
mod libp2p_bootstrap_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use libp2p::PeerId as Libp2pPeerId;
    use std::time::Duration;
    use tokio::time::sleep;
    #[tokio::test]
    async fn test_two_nodes_discover_each_other() {
        println!("Starting test_two_nodes_discover_each_other...");

        let config1 = NetworkConfig::default();
        let node1_service = Libp2pNetworkService::new(config1)
            .await
            .expect("Node 1 failed to start");
        let node1_peer_id = node1_service.local_peer_id().clone();
        println!("Node 1 Peer ID: {}", node1_peer_id);

        sleep(Duration::from_secs(2)).await;

        let addr = node1_service.listening_addresses()[0].clone();

        let bootstrap_info_for_node2 = vec![(node1_peer_id, addr.clone())];
        let mut config2 = NetworkConfig::default();
        config2.bootstrap_peers = bootstrap_info_for_node2;
        let node2_service = Libp2pNetworkService::new(config2)
            .await
            .expect("Node 2 failed to start");

        sleep(Duration::from_secs(5)).await;

        let discovered = node2_service
            .discover_peers(Some(node1_peer_id.to_string()))
            .await
            .expect("discover");
        assert!(discovered.iter().any(|p| p.0 == node1_peer_id.to_string()));
    }
}
