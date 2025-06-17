#[cfg(feature = "enable-libp2p")]
#[tokio::test]
async fn federation_sync_after_restart() -> Result<(), anyhow::Error> {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use libp2p::Multiaddr;
    use std::time::Duration;
    use tokio::time::sleep;

    let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/37001".parse().unwrap();
    let config_a = NetworkConfig {
        listen_addresses: vec![listen_addr.clone()],
        ..NetworkConfig::default()
    };
    let node_a = Libp2pNetworkService::new(config_a).await?;
    let peer_a = *node_a.local_peer_id();
    sleep(Duration::from_secs(1)).await;
    let addr_a = node_a
        .listening_addresses()
        .get(0)
        .cloned()
        .expect("addr");
    let config_b = NetworkConfig {
        bootstrap_peers: vec![(peer_a, addr_a.clone())],
        bootstrap_interval: Duration::from_secs(2),
        ..NetworkConfig::default()
    };
    let node_b = Libp2pNetworkService::new(config_b).await?;
    sleep(Duration::from_secs(3)).await;
    assert!(node_b.get_network_stats().await?.peer_count > 0);

    node_b.shutdown().await?;

    let config_b_restart = NetworkConfig {
        bootstrap_peers: vec![(peer_a, addr_a)],
        bootstrap_interval: Duration::from_secs(2),
        ..NetworkConfig::default()
    };
    let node_b2 = Libp2pNetworkService::new(config_b_restart).await?;
    sleep(Duration::from_secs(5)).await;
    assert!(node_b2.get_network_stats().await?.peer_count > 0);

    node_a.shutdown().await?;
    node_b2.shutdown().await?;
    Ok(())
}
