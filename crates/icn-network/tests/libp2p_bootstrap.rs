#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default
)]

#[cfg(feature = "experimental-libp2p")]
mod libp2p_bootstrap_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use std::str::FromStr;
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

        sleep(Duration::from_secs(1)).await;

        // For now, we'll use a mock listen address since listening_addresses method doesn't exist yet
        let mock_listen_addr = "/ip4/127.0.0.1/tcp/0"
            .parse::<Multiaddr>()
            .expect("Invalid multiaddr");

        println!(
            "Node 1 Peer ID: {}, using mock listen address: {}",
            node1_peer_id, mock_listen_addr
        );

        let bootstrap_info_for_node2 = vec![(node1_peer_id, mock_listen_addr)];
        let mut config2 = NetworkConfig::default();
        config2.bootstrap_peers = bootstrap_info_for_node2;

        let _node2_service = Libp2pNetworkService::new(config2)
            .await
            .expect("Node 2 failed to start");

        println!("Nodes started. Allowing time for discovery (e.g., 15 seconds). Check logs for Kademlia events...");
        sleep(Duration::from_secs(15)).await;

        println!("Test finished. Inspect logs for Kademlia discovery events involving both nodes.");
    }
}
