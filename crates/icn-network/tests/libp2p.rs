#[cfg(feature = "libp2p")]
mod libp2p_tests {
    use icn_common::Did;
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::{NetworkService, PeerId};
    use icn_protocol::{GossipMessage, MessagePayload, ProtocolMessage};
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn test_gossipsub_and_request_response() {
        let node_a = Libp2pNetworkService::new(NetworkConfig::default())
            .await
            .expect("node a start");
        sleep(Duration::from_secs(1)).await;
        let addr = node_a
            .listening_addresses()
            .into_iter()
            .next()
            .expect("node a addr");
        let peer_a = node_a.local_peer_id();

        let config_b = NetworkConfig {
            bootstrap_peers: vec![(*peer_a, addr)],
            ..NetworkConfig::default()
        };
        let node_b = Libp2pNetworkService::new(config_b)
            .await
            .expect("node b start");

        sleep(Duration::from_secs(3)).await;

        let peers = node_b
            .discover_peers(Some(node_a.local_peer_id().to_string()))
            .await
            .expect("discover");
        assert!(!peers.is_empty(), "peer discovery");

        let mut sub_a = node_a.subscribe().await.expect("sub a");
        let mut sub_b = node_b.subscribe().await.expect("sub b");

        let gossip = ProtocolMessage::new(
            MessagePayload::GossipMessage(GossipMessage {
                topic: "test".to_string(),
                payload: b"hello".to_vec(),
                ttl: 1,
            }),
            Did::new("key", "libp2p_a"),
            None,
        );
        node_a.broadcast_message(gossip).await.expect("broadcast");

        let msg = timeout(Duration::from_secs(10), sub_b.recv())
            .await
            .expect("recv timeout")
            .expect("recv");
        match msg.payload {
            MessagePayload::GossipMessage(_) => {}
            _ => panic!("unexpected message"),
        }

        node_b
            .send_message(
                &PeerId(node_a.local_peer_id().to_string()),
                ProtocolMessage::new(
                    MessagePayload::FederationSyncRequest(
                        icn_protocol::FederationSyncRequestMessage {
                            federation_id: "test".to_string(),
                            since_timestamp: None,
                            sync_types: vec![],
                        },
                    ),
                    Did::default(),
                    None,
                ),
            )
            .await
            .expect("send");

        let req = timeout(Duration::from_secs(10), sub_a.recv())
            .await
            .expect("req timeout")
            .expect("recv");
        match req.payload {
            MessagePayload::FederationSyncRequest(_) => {}
            _ => panic!("unexpected request"),
        }

        let resp = timeout(Duration::from_secs(10), sub_b.recv())
            .await
            .expect("resp timeout")
            .expect("recv");
        match resp.payload {
            MessagePayload::FederationSyncRequest(_) => {}
            _ => panic!("unexpected response"),
        }
    }
}
