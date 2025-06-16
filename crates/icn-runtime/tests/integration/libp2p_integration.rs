//! Integration tests for libp2p networking with RuntimeContext

#[cfg(feature = "enable-libp2p")]
mod libp2p_integration_tests {
    use icn_runtime::context::RuntimeContext;
    use std::str::FromStr;
    use tokio::time::{sleep, Duration};
    use icn_common::Did;

    #[tokio::test]
    async fn test_runtime_context_with_real_libp2p() {
        println!("Testing RuntimeContext creation with real libp2p networking...");
        
        // Test creating a RuntimeContext with real libp2p
        let node_identity = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let result = RuntimeContext::new_with_real_libp2p(
            node_identity,
            listen,
            None,  // No bootstrap peers for this simple test
            std::path::PathBuf::from("./mana_ledger.sled"),
        )
        .await;
        
        assert!(result.is_ok(), "Failed to create RuntimeContext with libp2p: {:?}", result.err());
        
        let ctx = result.unwrap();
        println!("✓ RuntimeContext created successfully with real libp2p networking");
        
        // Test that we can access the libp2p service
        let libp2p_service_result = ctx.get_libp2p_service();
        assert!(libp2p_service_result.is_ok(), "Failed to get libp2p service: {:?}", libp2p_service_result.err());
        
        let libp2p_service = libp2p_service_result.unwrap();
        println!("✓ Successfully accessed libp2p service");
        println!("✓ Libp2p Peer ID: {}", libp2p_service.local_peer_id());
        
        // Test basic mana operations still work
        let identity = Did::from_str(node_identity).unwrap();
        ctx
            .mana_ledger
            .set_balance(&identity, 1000)
            .expect("init mana");
        
        let balance = ctx.get_mana(&identity).await;
        assert!(balance.is_ok(), "Failed to get mana balance: {:?}", balance.err());
        assert_eq!(balance.unwrap(), 1000, "Mana balance should be 1000");
        println!("✓ Mana operations work correctly with libp2p RuntimeContext");
        
        // Give a bit of time for libp2p to settle
        sleep(Duration::from_secs(1)).await;
        
        println!("✓ All libp2p integration tests passed!");
    }

    #[tokio::test]
    async fn test_runtime_context_with_bootstrap_peers() {
        println!("Testing RuntimeContext with bootstrap peers...");
        
        use libp2p::{PeerId as Libp2pPeerId, Multiaddr};
        use std::str::FromStr;
        
        // Create mock bootstrap peer info
        let bootstrap_peer_id = Libp2pPeerId::random();
        let bootstrap_addr = "/ip4/127.0.0.1/tcp/12345".parse::<Multiaddr>().unwrap();
        let bootstrap_peers = vec![(bootstrap_peer_id, bootstrap_addr)];
        
        let node_identity = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let result = RuntimeContext::new_with_real_libp2p(
            node_identity,
            listen,
            Some(bootstrap_peers),
            std::path::PathBuf::from("./mana_ledger.sled"),
        )
        .await;
        
        assert!(result.is_ok(), "Failed to create RuntimeContext with bootstrap peers: {:?}", result.err());
        
        let ctx = result.unwrap();
        println!("✓ RuntimeContext created with bootstrap peers");
        
        // Verify libp2p service is accessible
        let libp2p_service = ctx.get_libp2p_service().unwrap();
        println!("✓ Peer ID: {}", libp2p_service.local_peer_id());
        
        println!("✓ Bootstrap peers test passed!");
    }
}

// If libp2p feature is not enabled, provide a simple stub test
#[cfg(not(feature = "enable-libp2p"))]
mod stub_tests {
    #[tokio::test]
    async fn test_libp2p_feature_disabled() {
        println!("libp2p feature is disabled - using stubs only");
        // This test always passes to indicate that the build works without libp2p
    }
} 