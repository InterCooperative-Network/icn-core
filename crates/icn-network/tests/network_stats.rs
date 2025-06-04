#[cfg(feature = "experimental-libp2p")]
mod network_stats {
    use icn_network::libp2p_service::Libp2pNetworkService;
    use icn_network::{NetworkMessage, NetworkService};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_network_stats_basic() {
        let node1 = Libp2pNetworkService::new(None)
            .await
            .expect("node1 start");
        let node1_peer = node1.local_peer_id().clone();
        sleep(Duration::from_secs(1)).await;
        let addr = node1
            .listening_addresses()
            .into_iter()
            .next()
            .expect("node1 addr");

        let node2 = Libp2pNetworkService::new(Some(vec![(node1_peer, addr)]))
            .await
            .expect("node2 start");

        sleep(Duration::from_secs(2)).await;

        node2
            .broadcast_message(NetworkMessage::GossipSub(
                "test".to_string(),
                b"hello".to_vec(),
            ))
            .await
            .expect("broadcast");

        sleep(Duration::from_secs(2)).await;

        let stats1 = node1.get_network_stats().await.expect("stats1");
        let stats2 = node2.get_network_stats().await.expect("stats2");

        assert!(stats1.peer_count >= 1, "node1 peers");
        assert!(stats2.peer_count >= 1, "node2 peers");
        assert!(stats2.bytes_sent > 0, "node2 bytes sent");
        assert!(stats1.bytes_received > 0, "node1 bytes received");
    }
}
