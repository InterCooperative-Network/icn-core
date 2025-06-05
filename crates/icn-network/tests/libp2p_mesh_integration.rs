#[cfg(all(test, feature = "experimental-libp2p"))]
mod libp2p_mesh_integration {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use libp2p::{PeerId as Libp2pPeerId};
    use anyhow::Result;
    use icn_network::{NetworkService, NetworkMessage};
    use icn_common::{Cid, Did};
    use icn_mesh::{ActualMeshJob as Job, MeshJobBid as Bid, JobId, JobSpec, Resources};
    use icn_identity::SignatureBytes;
    use std::str::FromStr;
    use tokio::time::{sleep, Duration, timeout};

    fn generate_dummy_job(id_str: &str) -> Job {
        let job_id_cid = Cid::new_v1_dummy(0x55, 0x13, id_str.as_bytes());
        let job_id = JobId::from(job_id_cid);
        let creator_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8").unwrap();
        let manifest_cid = Cid::new_v1_dummy(0x71, 0x12, b"dummy_manifest_data");
        let job_spec = JobSpec::Echo { payload: "hello world".to_string() };
        Job {
            id: job_id,
            creator_did,
            manifest_cid,
            spec: job_spec,
            cost_mana: 100,
            signature: SignatureBytes(vec![]),
        }
    }

    fn generate_dummy_bid(job_id: &JobId, executor_did_str: &str) -> Bid {
        let executor_did = Did::from_str(executor_did_str).unwrap();
        Bid {
            job_id: job_id.clone(),
            executor_did,
            price_mana: 50,
            resources: Resources::default(),
        }
    }

    #[tokio::test]
    #[ignore = "Testing libp2p event loop debugging with comprehensive logging"]
    async fn test_minimal_gossipsub_connectivity() -> Result<(), anyhow::Error> {
        // Initialize logging
        env_logger::init();
        
        println!("🔧 [DEBUG] Starting minimal gossipsub connectivity test");
        
        // 1. Create Node A with default config
        println!("🔧 [DEBUG] Creating Node A with default NetworkConfig...");
        let config_a = NetworkConfig::default();
        println!("🔧 [DEBUG] Node A config: {:?}", config_a);
        
        let node_a_service = Libp2pNetworkService::new(config_a).await?;
        let node_a_peer_id_str = node_a_service.local_peer_id().to_string();
        println!("✅ [DEBUG] Node A created - Peer ID: {}", node_a_peer_id_str);
        
        // Give Node A time to establish listeners
        println!("🔧 [DEBUG] Waiting 2s for Node A to establish listeners...");
        sleep(Duration::from_secs(2)).await;
        
        let node_a_addrs = node_a_service.listening_addresses();
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        println!("✅ [DEBUG] Node A listening addresses: {:?}", node_a_addrs);

        // 2. Create Node B with explicit bootstrap to Node A
        println!("🔧 [DEBUG] Creating Node B with bootstrap to Node A...");
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
        
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())];
        println!("🔧 [DEBUG] Node B config bootstrap peers: {:?}", config_b.bootstrap_peers);
        
        let node_b_service = Libp2pNetworkService::new(config_b).await?;
        let node_b_peer_id_str = node_b_service.local_peer_id().to_string();
        println!("✅ [DEBUG] Node B created - Peer ID: {}", node_b_peer_id_str);

        // 3. Wait for peer discovery with explicit timeout
        println!("🔧 [DEBUG] Allowing 8s for peer discovery and connection...");
        sleep(Duration::from_secs(8)).await;

        // 4. Subscribe to messages with timeout protection
        println!("🔧 [DEBUG] Node A subscribing to messages...");
        let node_a_subscribe_result = timeout(Duration::from_secs(5), node_a_service.subscribe()).await;
        match node_a_subscribe_result {
            Ok(Ok(mut node_a_receiver)) => {
                println!("✅ [DEBUG] Node A subscription successful");
                
                println!("🔧 [DEBUG] Node B subscribing to messages...");
                let node_b_subscribe_result = timeout(Duration::from_secs(5), node_b_service.subscribe()).await;
                match node_b_subscribe_result {
                    Ok(Ok(mut node_b_receiver)) => {
                        println!("✅ [DEBUG] Node B subscription successful");
                        
                        // 5. Test simple gossipsub message
                        let test_message = NetworkMessage::GossipSub("test_topic".to_string(), b"hello_test".to_vec());
                        println!("🔧 [DEBUG] Node A broadcasting test message: {:?}", test_message);
                        
                        let broadcast_result = timeout(Duration::from_secs(3), node_a_service.broadcast_message(test_message.clone())).await;
                        match broadcast_result {
                            Ok(Ok(())) => {
                                println!("✅ [DEBUG] Node A broadcast successful");
                                
                                // 6. Try to receive message on Node B
                                println!("🔧 [DEBUG] Node B waiting for message (timeout 10s)...");
                                let receive_result = timeout(Duration::from_secs(10), node_b_receiver.recv()).await;
                                match receive_result {
                                    Ok(Some(received_msg)) => {
                                        println!("✅ [DEBUG] Node B received message: {:?}", received_msg);
                                        assert!(matches!(received_msg, NetworkMessage::GossipSub(_, _)), "Expected GossipSub message");
                                    }
                                    Ok(None) => {
                                        println!("❌ [DEBUG] Node B receiver channel closed unexpectedly");
                                        return Err(anyhow::anyhow!("Node B receiver channel closed"));
                                    }
                                    Err(_) => {
                                        println!("❌ [DEBUG] Node B timed out waiting for message");
                                        return Err(anyhow::anyhow!("Node B timeout waiting for message"));
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                println!("❌ [DEBUG] Node A broadcast failed: {:?}", e);
                                return Err(anyhow::anyhow!("Node A broadcast failed: {}", e));
                            }
                            Err(_) => {
                                println!("❌ [DEBUG] Node A broadcast timed out");
                                return Err(anyhow::anyhow!("Node A broadcast timeout"));
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ [DEBUG] Node B subscription failed: {:?}", e);
                        return Err(anyhow::anyhow!("Node B subscription failed: {}", e));
                    }
                    Err(_) => {
                        println!("❌ [DEBUG] Node B subscription timed out");
                        return Err(anyhow::anyhow!("Node B subscription timeout"));
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ [DEBUG] Node A subscription failed: {:?}", e);
                return Err(anyhow::anyhow!("Node A subscription failed: {}", e));
            }
            Err(_) => {
                println!("❌ [DEBUG] Node A subscription timed out");
                return Err(anyhow::anyhow!("Node A subscription timeout"));
            }
        }

        println!("✅ [DEBUG] Minimal gossipsub connectivity test completed successfully!");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "Single-threaded runtime test for debugging event loop issues"]
    async fn test_single_threaded_gossipsub() -> Result<(), anyhow::Error> {
        println!("🔧 [DEBUG] Single-threaded runtime gossipsub test starting...");
        
        // Same test as above but on single-threaded runtime
        let config_a = NetworkConfig::default();
        let node_a_service = Libp2pNetworkService::new(config_a).await?;
        println!("✅ [DEBUG] Node A created in single-threaded runtime");
        
        sleep(Duration::from_secs(1)).await;
        let node_a_addrs = node_a_service.listening_addresses();
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(
            node_a_service.local_peer_id().clone(),
            node_a_addrs[0].clone()
        )];
        
        let node_b_service = Libp2pNetworkService::new(config_b).await?;
        println!("✅ [DEBUG] Node B created in single-threaded runtime");
        
        sleep(Duration::from_secs(3)).await;
        
        let mut node_a_receiver = node_a_service.subscribe().await?;
        let mut node_b_receiver = node_b_service.subscribe().await?;
        println!("✅ [DEBUG] Both nodes subscribed in single-threaded runtime");
        
        println!("✅ [DEBUG] Single-threaded runtime test completed without hanging!");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Original job announcement test with fixed API calls"]
    async fn test_job_announcement_and_bid_submission() -> Result<(), anyhow::Error> {
        println!("[test-mesh-network] Setting up Node A (Job Originator).");
        
        // 1. Create Node A (Job Originator) with proper NetworkConfig
        let config_a = NetworkConfig::default();
        let node_a_service = Libp2pNetworkService::new(config_a).await?;
        let node_a_peer_id_str = node_a_service.local_peer_id().to_string();
        
        // Wait for Node A to establish listeners
        sleep(Duration::from_secs(2)).await;
        let node_a_addrs = node_a_service.listening_addresses();
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        println!("[test-mesh-network] Node A Peer ID: {}, Listening Addresses: {:?}", node_a_peer_id_str, node_a_addrs);

        println!("[test-mesh-network] Setting up Node B (Executor), bootstrapping with Node A.");
        
        // 2. Create Node B (Executor) with proper NetworkConfig
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())];
        
        let node_b_service = Libp2pNetworkService::new(config_b).await?;
        let node_b_peer_id_str = node_b_service.local_peer_id().to_string();
        println!("[test-mesh-network] Node B Peer ID: {}", node_b_peer_id_str);

        println!("[test-mesh-network] Allowing 5s for peer discovery and connection.");
        sleep(Duration::from_secs(5)).await;

        println!("[test-mesh-network] Node A subscribing to messages.");
        let mut node_a_receiver = node_a_service.subscribe().await?;
        println!("[test-mesh-network] Node B subscribing to messages.");
        let mut node_b_receiver = node_b_service.subscribe().await?;

        let job_to_announce = generate_dummy_job("test_job_01");
        let job_announcement_msg = NetworkMessage::MeshJobAnnouncement(job_to_announce.clone());
        println!("[test-mesh-network] Node A broadcasting job announcement for job ID: {}", job_to_announce.id);
        node_a_service.broadcast_message(job_announcement_msg).await?;

        println!("[test-mesh-network] Node B awaiting job announcement (timeout 10s).");
        let received_on_b_res = timeout(Duration::from_secs(10), node_b_receiver.recv()).await;
        match received_on_b_res {
            Ok(Some(network_message_b)) => {
                if let NetworkMessage::MeshJobAnnouncement(received_job) = network_message_b {
                    assert_eq!(received_job.id, job_to_announce.id, "Node B received incorrect job ID");
                    println!("[test-mesh-network] Node B received job announcement for job ID: {}. Submitting bid.", received_job.id);

                    let bid_to_submit = generate_dummy_bid(&received_job.id, "did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH");
                    let bid_submission_msg = NetworkMessage::BidSubmission(bid_to_submit.clone());
                    node_b_service.broadcast_message(bid_submission_msg).await?;
                    println!("[test-mesh-network] Node B broadcasted bid for job ID: {}", received_job.id);

                    println!("[test-mesh-network] Node A awaiting bid submission (timeout 10s).");
                    let received_on_a_res = timeout(Duration::from_secs(10), node_a_receiver.recv()).await;
                    match received_on_a_res {
                        Ok(Some(network_message_a)) => {
                            if let NetworkMessage::BidSubmission(received_bid) = network_message_a {
                                assert_eq!(received_bid.job_id, job_to_announce.id, "Node A received bid for incorrect job ID");
                                assert_eq!(received_bid.executor_did, bid_to_submit.executor_did, "Node A received bid from incorrect executor");
                                println!("[test-mesh-network] Node A received bid for job ID: {} from executor: {}. Test successful.", received_bid.job_id, received_bid.executor_did.to_string());
                            } else {
                                panic!("[test-mesh-network] Node A did not receive a BidSubmission, but: {:?}", network_message_a);
                            }
                        }
                        Ok(None) => {
                            panic!("[test-mesh-network] Node A receiver channel closed unexpectedly.");
                        }
                        Err(_) => {
                            panic!("[test-mesh-network] Node A timed out waiting for bid submission.");
                        }
                    }
                } else {
                    panic!("[test-mesh-network] Node B did not receive a MeshJobAnnouncement, but: {:?}", network_message_b);
                }
            }
            Ok(None) => {
                panic!("[test-mesh-network] Node B receiver channel closed unexpectedly.");
            }
            Err(_) => {
                panic!("[test-mesh-network] Node B timed out waiting for job announcement.");
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Minimal event loop test to isolate hang issue"]
    async fn test_single_node_event_loop_startup() -> Result<(), anyhow::Error> {
        env_logger::init();
        
        println!("🔧 [DEBUG] Testing single node event loop startup...");
        
        // Create a single node with minimal config
        let config = NetworkConfig::default();
        println!("🔧 [DEBUG] Creating single node with config: {:?}", config);
        
        let node_service = Libp2pNetworkService::new(config).await?;
        println!("✅ [DEBUG] Node created successfully - Peer ID: {}", node_service.local_peer_id());
        
        // Give the event loop time to start
        println!("🔧 [DEBUG] Waiting 3s for event loop to initialize...");
        sleep(Duration::from_secs(3)).await;
        
        // Check if we can get listening addresses (this requires the event loop to be running)
        let addrs = node_service.listening_addresses();
        println!("✅ [DEBUG] Node listening addresses: {:?}", addrs);
        assert!(!addrs.is_empty(), "Node should have listening addresses");
        
        // Try to get network stats (this sends a command to the event loop)
        println!("🔧 [DEBUG] Getting network stats...");
        let stats = node_service.get_network_stats().await?;
        println!("✅ [DEBUG] Network stats: {:?}", stats);
        
        println!("✅ [DEBUG] Single node event loop test completed successfully!");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Test without Kademlia to isolate bootstrap hang"]
    async fn test_without_kademlia_bootstrap() -> Result<(), anyhow::Error> {
        env_logger::init();
        
        println!("🔧 [DEBUG] Testing service creation without Kademlia bootstrap...");
        
        // Create a single node with no bootstrap peers (should skip Kademlia bootstrap)
        let config = NetworkConfig::default();
        assert!(config.bootstrap_peers.is_empty(), "Config should have no bootstrap peers");
        
        println!("🔧 [DEBUG] Creating service with no bootstrap peers...");
        let node_service = Libp2pNetworkService::new(config).await?;
        println!("✅ [DEBUG] Node created successfully - Peer ID: {}", node_service.local_peer_id());
        
        // Give the event loop time to start (without bootstrap)
        println!("🔧 [DEBUG] Waiting 5s for event loop to initialize without bootstrap...");
        sleep(Duration::from_secs(5)).await;
        
        // Check if we can get listening addresses
        let addrs = node_service.listening_addresses();
        println!("✅ [DEBUG] Node listening addresses: {:?}", addrs);
        
        // Try to get network stats
        println!("🔧 [DEBUG] Getting network stats...");
        let stats_result = tokio::time::timeout(Duration::from_secs(10), node_service.get_network_stats()).await;
        match stats_result {
            Ok(Ok(stats)) => {
                println!("✅ [DEBUG] Network stats: {:?}", stats);
            }
            Ok(Err(e)) => {
                println!("❌ [DEBUG] Network stats error: {:?}", e);
                return Err(anyhow::anyhow!("Network stats error: {}", e));
            }
            Err(_) => {
                println!("❌ [DEBUG] Network stats timed out");
                return Err(anyhow::anyhow!("Network stats timeout"));
            }
        }
        
        println!("✅ [DEBUG] Test without Kademlia bootstrap completed successfully!");
        Ok(())
    }
} 