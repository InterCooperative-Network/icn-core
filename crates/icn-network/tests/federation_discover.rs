#![allow(unused_imports, clippy::clone_on_copy, clippy::uninlined_format_args)]

#[cfg(feature = "libp2p")]
mod federation_discover {
    use icn_common::Did;
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::{NetworkService, FEDERATION_INFO_PREFIX};
    use icn_protocol::FederationInfo;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn federation_discover_two_nodes() {
        let node_a = Libp2pNetworkService::new(NetworkConfig::default())
            .await
            .expect("node a start");
        sleep(Duration::from_secs(1)).await;
        let addr = node_a
            .listening_addresses()
            .into_iter()
            .next()
            .expect("node a addr");
        let peer_a = node_a.local_peer_id().clone();

        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(peer_a, addr.clone())];
        let node_b = Libp2pNetworkService::new(config_b)
            .await
            .expect("node b start");

        sleep(Duration::from_secs(2)).await;

        let info = FederationInfo {
            federation_id: "fed1".to_string(),
            members: vec![Did::new("key", "member1")],
        };
        let key = format!("{}fed1", FEDERATION_INFO_PREFIX);
        node_a
            .store_record(key, bincode::serialize(&info).unwrap())
            .await
            .unwrap();

        sleep(Duration::from_secs(2)).await;

        let discovered = node_b.discover_federations().await.unwrap();
        assert!(discovered.iter().any(|f| f.federation_id == "fed1"));

        node_a.shutdown().await.unwrap();
        node_b.shutdown().await.unwrap();
    }
}
