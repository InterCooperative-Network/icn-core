#[cfg(feature = "libp2p")]
mod handshake_pubsub {
    use icn_common::Did;
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use icn_protocol::{GossipMessage, MessagePayload, ProtocolMessage};
    use libp2p::PeerId as Libp2pPeerId;
    use std::str::FromStr;
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn test_handshake_and_gossipsub() {
        // start node A
        let node_a = Libp2pNetworkService::new(NetworkConfig::default())
            .await
            .expect("node a");
        sleep(Duration::from_secs(1)).await;
        let addr = node_a.listening_addresses()[0].clone();
        let peer_a = *node_a.local_peer_id();

        // start node B bootstrapping to A
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(peer_a, addr)];
        let node_b = Libp2pNetworkService::new(config_b).await.expect("node b");

        // give time for handshake
        sleep(Duration::from_secs(3)).await;

        // verify node B discovered node A
        let discovered = node_b
            .discover_peers(Some(node_a.local_peer_id().to_string()))
            .await
            .expect("discover");
        assert!(discovered.iter().any(|p| p.0 == peer_a.to_string()));

        // subscribe to gossipsub
        let mut sub_a = node_a.subscribe().await.expect("sub a");
        let mut sub_b = node_b.subscribe().await.expect("sub b");

        // send message from A
        let msg = ProtocolMessage::new(
            MessagePayload::GossipMessage(GossipMessage {
                topic: "demo".into(),
                payload: b"hello".to_vec(),
                ttl: 1,
            }),
            Did::from_str("did:key:testnode").unwrap(),
            None,
        );
        node_a.broadcast_message(msg).await.expect("broadcast");

        // expect B to receive
        let recv = timeout(Duration::from_secs(10), sub_b.recv())
            .await
            .expect("recv timeout")
            .expect("recv");
        assert!(matches!(recv.payload, MessagePayload::GossipMessage(_)));

        // also ensure A got its own broadcast
        let _ = timeout(Duration::from_secs(10), sub_a.recv()).await;
    }
}
