#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default,
    dead_code
)]

#[cfg(feature = "libp2p")]
mod federation_sync {
    use icn_common::Did;
    use icn_governance::request_federation_sync;
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::{NetworkMessage, NetworkService, PeerId};
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn federation_sync_message_delivered() {
        // Node A
        let config_a = NetworkConfig::default();
        let node_a = Libp2pNetworkService::new(config_a)
            .await
            .expect("node A start");
        let peer_a = node_a.local_peer_id().clone();
        sleep(Duration::from_secs(1)).await;
        let addr_a = node_a
            .listening_addresses()
            .into_iter()
            .next()
            .expect("node A addr");

        // Node B bootstraps to A
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(peer_a, addr_a.clone())];
        let node_b = Libp2pNetworkService::new(config_b)
            .await
            .expect("node B start");

        // Allow peers to connect
        sleep(Duration::from_secs(2)).await;

        let mut sub_b = node_b.subscribe().await.expect("subscribe");
        let peer_b = PeerId(node_b.local_peer_id().to_string());

        let ts = 42u64;
        request_federation_sync(&node_a, &peer_b, Some(ts))
            .await
            .expect("send sync");

        let msg = timeout(Duration::from_secs(5), sub_b.recv())
            .await
            .expect("recv timeout")
            .expect("recv message");

        match msg {
            NetworkMessage::FederationSyncRequest(did) => {
                assert_eq!(did, Did::new("sync", &ts.to_string()));
            }
            other => panic!("unexpected message: {:?}", other),
        }

        node_a.shutdown().await.unwrap();
        node_b.shutdown().await.unwrap();
    }
}
