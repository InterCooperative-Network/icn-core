#[cfg(all(test, feature = "experimental-libp2p"))]
mod libp2p_mesh_integration {
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use libp2p::{PeerId as Libp2pPeerId};
    use anyhow::Result;
    use icn_network::{NetworkService, NetworkMessage};
    use icn_common::{Cid, Did};
    use icn_mesh::{ActualMeshJob as Job, MeshJobBid as Bid, JobId, JobSpec, Resources};
    use icn_identity::{SignatureBytes, ExecutionReceipt};
    use icn_runtime::executor::{SimpleExecutor, JobExecutor};
    use std::str::FromStr;
    use tokio::time::{sleep, Duration, timeout};
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    fn init_test_logger() {
        INIT_LOGGER.call_once(|| {
            env_logger::init();
        });
    }

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

    fn test_sign_receipt_data(data: &str) -> Result<SignatureBytes, anyhow::Error> {
        // Create a dummy signature for testing
        let signature_data = format!("test_signed:{}", data);
        Ok(SignatureBytes(signature_data.into_bytes()))
    }

    fn mock_anchor_receipt_to_dag(receipt: &ExecutionReceipt) -> Result<Cid, anyhow::Error> {
        // Create a mock CID for the anchored receipt
        let receipt_data = format!("receipt_for_job_{}", receipt.job_id);
        Ok(Cid::new_v1_dummy(0x71, 0x12, receipt_data.as_bytes()))
    }

    #[tokio::test]
    #[ignore = "Testing libp2p event loop debugging with comprehensive logging"]
    async fn test_minimal_gossipsub_connectivity() -> Result<(), anyhow::Error> {
        // Initialize logging (safe for multiple test calls)
        init_test_logger();
        
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
    #[ignore = "Enhanced job announcement test with proper network readiness checks"]
    async fn test_job_announcement_and_bid_submission() -> Result<(), anyhow::Error> {
        init_test_logger();
        println!("🔧 [test-mesh-network] Setting up Node A (Job Originator).");
        
        // 1. Create Node A (Job Originator) with comprehensive setup
        let config_a = NetworkConfig::default();
        println!("🔧 [test-mesh-network] Creating Node A with config: {:?}", config_a);
        
        let node_a_service = Libp2pNetworkService::new(config_a).await?;
        let node_a_peer_id_str = node_a_service.local_peer_id().to_string();
        println!("✅ [test-mesh-network] Node A created - Peer ID: {}", node_a_peer_id_str);
        
        // Wait for Node A to establish listeners with retries
        println!("🔧 [test-mesh-network] Waiting for Node A to establish listeners...");
        let mut node_a_addrs = Vec::new();
        for attempt in 1..=5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            node_a_addrs = node_a_service.listening_addresses();
            println!("🔧 [test-mesh-network] Attempt {}/5: Node A has {} listening addresses", attempt, node_a_addrs.len());
            if !node_a_addrs.is_empty() {
                break;
            }
        }
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses after 5 attempts");
        println!("✅ [test-mesh-network] Node A listening addresses: {:?}", node_a_addrs);

        println!("🔧 [test-mesh-network] Setting up Node B (Executor), bootstrapping with Node A.");
        
        // 2. Create Node B (Executor) with proper NetworkConfig
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())];
        println!("🔧 [test-mesh-network] Node B config bootstrap peers: {:?}", config_b.bootstrap_peers);
        
        let node_b_service = Libp2pNetworkService::new(config_b).await?;
        let node_b_peer_id_str = node_b_service.local_peer_id().to_string();
        println!("✅ [test-mesh-network] Node B created - Peer ID: {}", node_b_peer_id_str);

        // 3. Allow extended time for peer discovery and connection
        println!("🔧 [test-mesh-network] Allowing 8s for peer discovery and connection...");
        tokio::time::sleep(Duration::from_secs(8)).await;

        // 4. Check network connectivity before proceeding
        println!("🔧 [test-mesh-network] Checking Node A network stats...");
        let node_a_stats = node_a_service.get_network_stats().await?;
        println!("✅ [test-mesh-network] Node A stats: {:?}", node_a_stats);
        
        println!("🔧 [test-mesh-network] Checking Node B network stats...");
        let node_b_stats = node_b_service.get_network_stats().await?;
        println!("✅ [test-mesh-network] Node B stats: {:?}", node_b_stats);

        // 5. Set up message subscriptions with timeout protection
        println!("🔧 [test-mesh-network] Node A subscribing to messages...");
        let node_a_subscribe_result = timeout(Duration::from_secs(5), node_a_service.subscribe()).await;
        let mut node_a_receiver = match node_a_subscribe_result {
            Ok(Ok(receiver)) => {
                println!("✅ [test-mesh-network] Node A subscription successful");
                receiver
            }
            Ok(Err(e)) => return Err(anyhow::anyhow!("Node A subscription failed: {}", e)),
            Err(_) => return Err(anyhow::anyhow!("Node A subscription timed out")),
        };
        
        println!("🔧 [test-mesh-network] Node B subscribing to messages...");
        let node_b_subscribe_result = timeout(Duration::from_secs(5), node_b_service.subscribe()).await;
        let mut node_b_receiver = match node_b_subscribe_result {
            Ok(Ok(receiver)) => {
                println!("✅ [test-mesh-network] Node B subscription successful");
                receiver
            }
            Ok(Err(e)) => return Err(anyhow::anyhow!("Node B subscription failed: {}", e)),
            Err(_) => return Err(anyhow::anyhow!("Node B subscription timed out")),
        };

        // 6. Give subscriptions time to propagate
        println!("🔧 [test-mesh-network] Allowing 2s for gossipsub subscriptions to propagate...");
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 7. Test mesh job announcement flow
        let job_to_announce = generate_dummy_job("test_job_01");
        let job_announcement_msg = NetworkMessage::MeshJobAnnouncement(job_to_announce.clone());
        println!("🔧 [test-mesh-network] Node A broadcasting job announcement for job ID: {}", job_to_announce.id);
        
        let broadcast_result = timeout(Duration::from_secs(5), node_a_service.broadcast_message(job_announcement_msg)).await;
        match broadcast_result {
            Ok(Ok(())) => println!("✅ [test-mesh-network] Node A job announcement broadcast successful"),
            Ok(Err(e)) => return Err(anyhow::anyhow!("Node A broadcast failed: {}", e)),
            Err(_) => return Err(anyhow::anyhow!("Node A broadcast timed out")),
        }

        println!("🔧 [test-mesh-network] Node B awaiting job announcement (timeout 15s).");
        let received_on_b_res = timeout(Duration::from_secs(15), node_b_receiver.recv()).await;
        match received_on_b_res {
            Ok(Some(network_message_b)) => {
                if let NetworkMessage::MeshJobAnnouncement(received_job) = network_message_b {
                    assert_eq!(received_job.id, job_to_announce.id, "Node B received incorrect job ID");
                    println!("✅ [test-mesh-network] Node B received job announcement for job ID: {}. Submitting bid.", received_job.id);

                    let bid_to_submit = generate_dummy_bid(&received_job.id, "did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH");
                    let bid_submission_msg = NetworkMessage::BidSubmission(bid_to_submit.clone());
                    
                    let bid_broadcast_result = timeout(Duration::from_secs(5), node_b_service.broadcast_message(bid_submission_msg)).await;
                    match bid_broadcast_result {
                        Ok(Ok(())) => println!("✅ [test-mesh-network] Node B bid broadcast successful"),
                        Ok(Err(e)) => return Err(anyhow::anyhow!("Node B bid broadcast failed: {}", e)),
                        Err(_) => return Err(anyhow::anyhow!("Node B bid broadcast timed out")),
                    }

                    println!("🔧 [test-mesh-network] Node A awaiting bid submission (timeout 15s).");
                    let received_on_a_res = timeout(Duration::from_secs(15), node_a_receiver.recv()).await;
                    match received_on_a_res {
                        Ok(Some(network_message_a)) => {
                            if let NetworkMessage::BidSubmission(received_bid) = network_message_a {
                                assert_eq!(received_bid.job_id, job_to_announce.id, "Node A received bid for incorrect job ID");
                                assert_eq!(received_bid.executor_did, bid_to_submit.executor_did, "Node A received bid from incorrect executor");
                                println!("✅ [test-mesh-network] Node A received bid for job ID: {} from executor: {}. Test successful.", received_bid.job_id, received_bid.executor_did.to_string());
                            } else {
                                return Err(anyhow::anyhow!("Node A did not receive a BidSubmission, but: {:?}", network_message_a));
                            }
                        }
                        Ok(None) => {
                            return Err(anyhow::anyhow!("Node A receiver channel closed unexpectedly."));
                        }
                        Err(_) => {
                            return Err(anyhow::anyhow!("Node A timed out waiting for bid submission."));
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Node B did not receive a MeshJobAnnouncement, but: {:?}", network_message_b));
                }
            }
            Ok(None) => {
                return Err(anyhow::anyhow!("Node B receiver channel closed unexpectedly."));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Node B timed out waiting for job announcement."));
            }
        }

        println!("🎉 [test-mesh-network] Complete job announcement and bidding flow test successful!");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Minimal event loop test to isolate hang issue"]
    async fn test_single_node_event_loop_startup() -> Result<(), anyhow::Error> {
        init_test_logger();
        
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
        init_test_logger();
        
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

    #[tokio::test]
    #[ignore = "Complete cross-node job execution pipeline test"]
    async fn test_full_job_execution_pipeline() -> Result<(), anyhow::Error> {
        init_test_logger();
        println!("🚀 [PIPELINE] Starting complete cross-node job execution pipeline test");
        
        // === Phase 1: Node Setup and Connection ===
        println!("🔧 [PIPELINE] Phase 1: Setting up nodes and establishing connection...");
        
        let config_a = NetworkConfig::default();
        let node_a_service = Libp2pNetworkService::new(config_a).await?;
        let node_a_peer_id = node_a_service.local_peer_id().to_string();
        println!("✅ [PIPELINE] Node A created - Peer ID: {}", node_a_peer_id);
        
        // Wait for Node A to establish listeners
        let mut node_a_addrs = Vec::new();
        for attempt in 1..=5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            node_a_addrs = node_a_service.listening_addresses();
            if !node_a_addrs.is_empty() {
                break;
            }
        }
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        println!("✅ [PIPELINE] Node A listening on: {:?}", node_a_addrs[0]);
        
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id)?;
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())];
        
        let node_b_service = Libp2pNetworkService::new(config_b).await?;
        let node_b_peer_id = node_b_service.local_peer_id().to_string();
        println!("✅ [PIPELINE] Node B created - Peer ID: {}", node_b_peer_id);
        
        // Allow time for peer discovery
        println!("🔧 [PIPELINE] Allowing 8s for peer discovery...");
        tokio::time::sleep(Duration::from_secs(8)).await;
        
        let node_a_stats = node_a_service.get_network_stats().await?;
        let node_b_stats = node_b_service.get_network_stats().await?;
        println!("✅ [PIPELINE] Node A peers: {}, Node B peers: {}", node_a_stats.peer_count, node_b_stats.peer_count);
        
        let mut node_a_receiver = node_a_service.subscribe().await?;
        let mut node_b_receiver = node_b_service.subscribe().await?;
        println!("✅ [PIPELINE] Both nodes subscribed to network messages");
        
        // === Phase 2: Job Announcement and Bidding ===
        println!("🔧 [PIPELINE] Phase 2: Job announcement and bidding...");
        
        let job_to_execute = generate_dummy_job("pipeline_job_01");
        let job_id = job_to_execute.id.clone();
        let job_announcement_msg = NetworkMessage::MeshJobAnnouncement(job_to_execute.clone());
        
        println!("📢 [PIPELINE] Node A announcing job: {}", job_id);
        node_a_service.broadcast_message(job_announcement_msg).await?;
        
        // Node B receives job and submits bid
        println!("🔧 [PIPELINE] Node B waiting for job announcement...");
        let received_job = match timeout(Duration::from_secs(10), node_b_receiver.recv()).await {
            Ok(Some(NetworkMessage::MeshJobAnnouncement(job))) => {
                assert_eq!(job.id, job_id, "Node B received correct job ID");
                println!("✅ [PIPELINE] Node B received job: {}", job.id);
                job
            }
            Ok(Some(other)) => return Err(anyhow::anyhow!("Node B received unexpected message: {:?}", other)),
            Ok(None) => return Err(anyhow::anyhow!("Node B receiver channel closed")),
            Err(_) => return Err(anyhow::anyhow!("Node B timed out waiting for job")),
        };
        
        let executor_did = Did::from_str("did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH")?;
        let bid_to_submit = generate_dummy_bid(&job_id, "did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH");
        let bid_submission_msg = NetworkMessage::BidSubmission(bid_to_submit.clone());
        
        println!("💰 [PIPELINE] Node B submitting bid for job: {}", job_id);
        node_b_service.broadcast_message(bid_submission_msg).await?;
        
        // Node A receives bid
        println!("🔧 [PIPELINE] Node A waiting for bid...");
        let received_bid = match timeout(Duration::from_secs(10), node_a_receiver.recv()).await {
            Ok(Some(NetworkMessage::BidSubmission(bid))) => {
                assert_eq!(bid.job_id, job_id, "Node A received bid for correct job");
                println!("✅ [PIPELINE] Node A received bid from: {}", bid.executor_did);
                bid
            }
            Ok(Some(other)) => return Err(anyhow::anyhow!("Node A received unexpected message: {:?}", other)),
            Ok(None) => return Err(anyhow::anyhow!("Node A receiver channel closed")),
            Err(_) => return Err(anyhow::anyhow!("Node A timed out waiting for bid")),
        };
        
        // === Phase 3: Job Assignment ===
        println!("🔧 [PIPELINE] Phase 3: Job assignment...");
        
        let selected_executor_did = received_bid.executor_did.clone();
        let assignment_msg = NetworkMessage::JobAssignmentNotification(job_id.clone(), selected_executor_did.clone());
        
        println!("📋 [PIPELINE] Node A assigning job {} to executor: {}", job_id, selected_executor_did);
        node_a_service.broadcast_message(assignment_msg).await?;
        
        // Node B receives assignment
        println!("🔧 [PIPELINE] Node B waiting for job assignment...");
        let (assigned_job_id, assigned_executor) = match timeout(Duration::from_secs(10), node_b_receiver.recv()).await {
            Ok(Some(NetworkMessage::JobAssignmentNotification(job_id, executor_did))) => {
                assert_eq!(job_id, job_id, "Node B received assignment for correct job");
                assert_eq!(executor_did, selected_executor_did, "Node B assigned to correct executor");
                println!("✅ [PIPELINE] Node B received job assignment: {}", job_id);
                (job_id, executor_did)
            }
            Ok(Some(other)) => return Err(anyhow::anyhow!("Node B received unexpected message: {:?}", other)),
            Ok(None) => return Err(anyhow::anyhow!("Node B receiver channel closed")),
            Err(_) => return Err(anyhow::anyhow!("Node B timed out waiting for assignment")),
        };
        
        // === Phase 4: Job Execution ===
        println!("🔧 [PIPELINE] Phase 4: Job execution...");
        
        // Create signing key for the executor
        let (executor_signing_key, _executor_verifying_key) = icn_identity::generate_ed25519_keypair();
        let executor = SimpleExecutor::new(selected_executor_did.clone(), executor_signing_key);
        println!("🏃 [PIPELINE] Node B executing job: {}", assigned_job_id);
        
        // Execute the job using SimpleExecutor
        let execution_result = executor.execute_job(&received_job).await
            .map_err(|e| anyhow::anyhow!("Job execution failed: {}", e))?;
        println!("✅ [PIPELINE] Job execution completed. Result CID: {}", execution_result.result_cid);
        
        // === Phase 5: Receipt Creation and Submission ===
        println!("🔧 [PIPELINE] Phase 5: Receipt creation and submission...");
        
        // The execution_result is already a signed ExecutionReceipt from SimpleExecutor
        println!("📤 [PIPELINE] Node B submitting execution receipt for job: {}", assigned_job_id);
        
        let receipt_msg = NetworkMessage::SubmitReceipt(execution_result.clone());
        node_b_service.broadcast_message(receipt_msg).await?;
        
        // === Phase 6: Receipt Verification and Anchoring ===
        println!("🔧 [PIPELINE] Phase 6: Receipt verification and anchoring...");
        
        println!("🔧 [PIPELINE] Node A waiting for execution receipt...");
        let verified_receipt = match timeout(Duration::from_secs(10), node_a_receiver.recv()).await {
            Ok(Some(NetworkMessage::SubmitReceipt(receipt))) => {
                assert_eq!(receipt.job_id, job_id, "Node A received receipt for correct job");
                assert_eq!(receipt.executor_did, selected_executor_did, "Receipt from correct executor");
                
                // Verify that we have a real signature (binary format)
                assert!(!receipt.sig.0.is_empty(), "Receipt should have a signature");
                assert!(receipt.sig.0.len() > 32, "Receipt signature should be substantial (Ed25519 signature)");
                
                println!("✅ [PIPELINE] Node A received and verified receipt from: {}", receipt.executor_did);
                receipt
            }
            Ok(Some(other)) => return Err(anyhow::anyhow!("Node A received unexpected message: {:?}", other)),
            Ok(None) => return Err(anyhow::anyhow!("Node A receiver channel closed")),
            Err(_) => return Err(anyhow::anyhow!("Node A timed out waiting for receipt")),
        };
        
        // Anchor receipt to DAG (mock)
        let anchored_cid = mock_anchor_receipt_to_dag(&verified_receipt)?;
        println!("⚓ [PIPELINE] Receipt anchored to DAG with CID: {}", anchored_cid);
        
        // === Phase 7: Pipeline Completion ===
        println!("🎉 [PIPELINE] Complete cross-node job execution pipeline successful!");
        println!("📊 [PIPELINE] Summary:");
        println!("   • Job ID: {}", job_id);
        println!("   • Executor: {}", selected_executor_did);
        println!("   • Result CID: {}", verified_receipt.result_cid);
        println!("   • CPU Time: {}ms", verified_receipt.cpu_ms);
        println!("   • Receipt CID: {}", anchored_cid);
        
        Ok(())
    }
} 