#![allow(
    unused_imports,
    clippy::clone_on_copy,
    clippy::uninlined_format_args,
    clippy::field_reassign_with_default
)]

#[cfg(feature = "experimental-libp2p")]
mod record_store_tests {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_record_store_roundtrip() {
        let config_a = NetworkConfig::default();
        let node_a = Libp2pNetworkService::new(config_a)
            .await
            .expect("node a start");
        let node_a_peer = node_a.local_peer_id().clone();
        sleep(Duration::from_secs(1)).await;
        let addr_a = node_a
            .listening_addresses()
            .into_iter()
            .next()
            .expect("node a addr");

        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(node_a_peer, addr_a)];
        let node_b = Libp2pNetworkService::new(config_b)
            .await
            .expect("node b start");

        let key = b"test-key".to_vec();
        let value = b"test-value".to_vec();

        node_a
            .store_record(key.clone(), value.clone())
            .await
            .expect("store record");

        sleep(Duration::from_secs(1)).await;

        let result = node_b.get_record(key.clone()).await.expect("get record");
        let retrieved = result.expect("record not found");
        assert_eq!(retrieved, value);
    }
}
