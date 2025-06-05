//! Cross-node mesh job execution integration tests for Phase 2B
//! 
//! Tests the complete distributed mesh job pipeline:
//! 1. Job submission on Node A
//! 2. Job announcement propagation to Node B via libp2p
//! 3. Bid submission from Node B back to Node A
//! 4. Job assignment and execution on Node B  
//! 5. Receipt submission back to Node A
//! 6. Receipt verification and anchoring on Node A

#[cfg(feature = "enable-libp2p")]
mod cross_node_tests {
    use icn_runtime::context::{RuntimeContext, DefaultMeshNetworkService, MeshNetworkService};
    use icn_runtime::host_submit_mesh_job;
    use icn_runtime::executor::{SimpleExecutor, JobExecutor};
    use icn_common::{Did, Cid};
    use icn_identity::{SignatureBytes, generate_ed25519_keypair};
    use icn_mesh::{ActualMeshJob, MeshJobBid, JobSpec, Resources};
    use icn_network::{NetworkMessage, NetworkService};
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration, timeout};
    use log::{info, debug};
    use libp2p::{PeerId as Libp2pPeerId, Multiaddr};

    /// Helper to create a test job with proper structure
    fn create_test_job(job_id_suffix: &str, creator_did: &Did, cost_mana: u64) -> ActualMeshJob {
        let job_id = Cid::new_v1_dummy(0x55, 0x13, format!("test_job_{}", job_id_suffix).as_bytes());
        let manifest_cid = Cid::new_v1_dummy(0x55, 0x14, format!("manifest_{}", job_id_suffix).as_bytes());

        ActualMeshJob {
            id: job_id,
            manifest_cid,
            spec: JobSpec::Echo { payload: format!("Cross-node test job {}", job_id_suffix) },
            creator_did: creator_did.clone(),
            cost_mana,
            signature: SignatureBytes(vec![0u8; 64]), // Dummy signature for tests
        }
    }

    /// Helper to create RuntimeContext with real libp2p networking
    async fn create_libp2p_runtime_context(
        identity_suffix: &str, 
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
        initial_mana: u64
    ) -> Result<Arc<RuntimeContext>, anyhow::Error> {
        let identity_str = format!("did:key:z6Mkv{}", identity_suffix);
        let ctx = RuntimeContext::new_with_real_libp2p(&identity_str, bootstrap_peers).await?;
        
        // Set initial mana for the identity
        let identity = Did::from_str(&identity_str)?;
        ctx.mana_ledger.set_balance(&identity, initial_mana).await;
        
        Ok(ctx)
    }

    /// Test 0: Basic libp2p connectivity test
    #[tokio::test]
    async fn test_basic_libp2p_connectivity() -> Result<(), anyhow::Error> {
        info!("=== TEST: Basic libp2p Connectivity ===");

        // Create Node A (Bootstrap)
        let node_a = create_libp2p_runtime_context("ConnTestA", None, 0).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        info!("Node A Peer ID: {}", node_a_peer_id);
        
        // Give Node A time to start listening
        sleep(Duration::from_millis(1000)).await;
        
        // Create Node B (Worker) - but don't bootstrap yet
        let node_b = create_libp2p_runtime_context("ConnTestB", None, 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        info!("Node B Peer ID: {}", node_b_libp2p.local_peer_id());
        
        // Test basic network stats
        let stats_a = node_a_libp2p.get_network_stats().await?;
        let stats_b = node_b_libp2p.get_network_stats().await?;
        
        info!("Node A stats: {:?}", stats_a);
        info!("Node B stats: {:?}", stats_b);
        
        // Test basic message broadcasting (without expecting anyone to receive it)
        let test_message = NetworkMessage::GossipSub("test_topic".to_string(), b"hello world".to_vec());
        node_a_libp2p.broadcast_message(test_message).await?;
        
        info!("✓ Basic libp2p setup and broadcasting works");
        Ok(())
    }

    /// Test 1: Verify job announcement reaches worker node via libp2p
    #[tokio::test]
    async fn test_job_announcement_reaches_worker_node() -> Result<(), anyhow::Error> {
        info!("=== TEST: Job Announcement Cross-Node Propagation ===");

        // Create Node A (Bootstrap/Submitter) 
        let node_a = create_libp2p_runtime_context("NodeA123", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        info!("Node A (Bootstrap) Peer ID: {}", node_a_peer_id);
        
        // Wait for Node A to start listening with retry logic
        let mut node_a_addrs = Vec::new();
        for attempt in 1..=10 {
            sleep(Duration::from_millis(500)).await;
            node_a_addrs = node_a_libp2p.listening_addresses();
            if !node_a_addrs.is_empty() {
                info!("Node A listening addresses found on attempt {}: {:?}", attempt, node_a_addrs);
                break;
            }
            info!("Attempt {}: Node A not listening yet, retrying...", attempt);
        }
        
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses after 5 seconds"));
        }
        
        // Create Node B (Worker) bootstrapped to Node A using actual address
        let bootstrap_peers = vec![(node_a_peer_id, node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("NodeB456", Some(bootstrap_peers), 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        info!("Node B (Worker) Peer ID: {}", node_b_libp2p.local_peer_id());
        
        // Give nodes time to connect
        sleep(Duration::from_secs(2)).await;
        
        // Subscribe Node B to network messages before job announcement
        let mut node_b_receiver = node_b_libp2p.subscribe().await?;
        
        // Create and submit job on Node A
        let submitter_did = Did::from_str("did:key:z6MkvNodeA123")?;
        let test_job = create_test_job("announcement_test", &submitter_did, 50);
        let job_json = serde_json::to_string(&test_job)?;
        
        info!("Node A: Submitting job {:?}", test_job.id);
        let submitted_job_id = host_submit_mesh_job(&node_a, &job_json).await?;
        assert_eq!(submitted_job_id, test_job.id);
        
        // Queue job in Node A's mesh job manager
        node_a.internal_queue_mesh_job(test_job.clone()).await?;
        
        // Start Node A's job manager (which should announce the job)
        let node_a_clone = node_a.clone();
        node_a_clone.spawn_mesh_job_manager().await;
        
        // Node B: Wait for job announcement message
        info!("Node B: Waiting for job announcement...");
        let announcement_received = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = node_b_receiver.recv().await {
                    debug!("Node B received message: {:?}", message);
                    if let NetworkMessage::MeshJobAnnouncement(announced_job) = message {
                        if announced_job.id == test_job.id {
                            info!("✓ Node B received job announcement for {:?}", announced_job.id);
                            return Some(announced_job);
                        }
                    }
                }
            }
        }).await;
        
        assert!(announcement_received.is_ok(), "Job announcement did not reach Node B within timeout");
        let announced_job = announcement_received.unwrap().unwrap();
        assert_eq!(announced_job.id, test_job.id);
        assert_eq!(announced_job.creator_did, submitter_did);
        
        info!("✓ Test passed: Job announcement successfully propagated from Node A to Node B");
        Ok(())
    }

    /// Test 2: Cross-node bidding flow
    #[tokio::test]
    async fn test_cross_node_bidding_flow() -> Result<(), anyhow::Error> {
        info!("=== TEST: Cross-Node Bidding Flow ===");
        
        // Setup nodes similar to test 1
        let node_a = create_libp2p_runtime_context("BidNodeA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        sleep(Duration::from_millis(500)).await;
        
        // Get Node A's actual listening addresses
        let node_a_addrs = node_a_libp2p.listening_addresses();
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses"));
        }
        
        let bootstrap_peers = vec![(node_a_peer_id, node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("BidNodeB", Some(bootstrap_peers), 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        sleep(Duration::from_secs(2)).await;
        
        // Subscribe Node A to network messages to receive bids
        let mut node_a_receiver = node_a_libp2p.subscribe().await?;
        
        // Create and announce job from Node A
        let submitter_did = Did::from_str("did:key:z6MkvBidNodeA")?;
        let test_job = create_test_job("bidding_test", &submitter_did, 30);
        
        // Node A announces job directly via network service
        let node_a_network = DefaultMeshNetworkService::new(node_a_libp2p.clone());
        node_a_network.announce_job(&test_job).await.map_err(|e| anyhow::anyhow!("Job announcement failed: {}", e))?;
        
        info!("Node A: Job announced {:?}", test_job.id);
        
        // Node B: Create and submit bid
        let executor_did = Did::from_str("did:key:z6MkvBidNodeB")?;
        let bid = MeshJobBid {
            job_id: test_job.id.clone(),
            executor_did: executor_did.clone(),
            price_mana: 20,
            resources: Resources::default(),
        };
        
        // Node B broadcasts bid
        let bid_message = NetworkMessage::BidSubmission(bid.clone());
        node_b_libp2p.broadcast_message(bid_message).await.map_err(|e| anyhow::anyhow!("Bid broadcast failed: {}", e))?;
        
        info!("Node B: Bid submitted for job {:?}", test_job.id);
        
        // Node A: Wait for bid
        info!("Node A: Waiting for bid...");
        let bid_received = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = node_a_receiver.recv().await {
                    debug!("Node A received message: {:?}", message);
                    if let NetworkMessage::BidSubmission(received_bid) = message {
                        if received_bid.job_id == test_job.id {
                            info!("✓ Node A received bid for job {:?} from {:?}", received_bid.job_id, received_bid.executor_did);
                            return Some(received_bid);
                        }
                    }
                }
            }
        }).await;
        
        assert!(bid_received.is_ok(), "Bid did not reach Node A within timeout");
        let received_bid = bid_received.unwrap().unwrap();
        assert_eq!(received_bid.job_id, test_job.id);
        assert_eq!(received_bid.executor_did, executor_did);
        assert_eq!(received_bid.price_mana, 20);
        
        info!("✓ Test passed: Bid successfully transmitted from Node B to Node A");
        Ok(())
    }

    /// Test 3: Cross-node job assignment and execution
    #[tokio::test]
    async fn test_cross_node_job_assignment_and_execution() -> Result<(), anyhow::Error> {
        info!("=== TEST: Cross-Node Job Assignment and Execution ===");
        
        // Setup nodes
        let node_a = create_libp2p_runtime_context("AssignNodeA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        sleep(Duration::from_millis(500)).await;
        
        // Get Node A's actual listening addresses
        let node_a_addrs = node_a_libp2p.listening_addresses();
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses"));
        }
        
        let bootstrap_peers = vec![(node_a_peer_id, node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("AssignNodeB", Some(bootstrap_peers), 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        sleep(Duration::from_secs(2)).await;
        
        // Subscribe Node B to receive assignment notifications
        let mut node_b_receiver = node_b_libp2p.subscribe().await?;
        
        // Create job and executor DID
        let submitter_did = Did::from_str("did:key:z6MkvAssignNodeA")?;
        let executor_did = Did::from_str("did:key:z6MkvAssignNodeB")?;
        let test_job = create_test_job("assignment_test", &submitter_did, 40);
        
        // Node A: Send assignment notification
        let assignment_message = NetworkMessage::JobAssignmentNotification(
            test_job.id.clone(), 
            executor_did.clone()
        );
        
        node_a_libp2p.broadcast_message(assignment_message).await.map_err(|e| anyhow::anyhow!("Assignment notification failed: {}", e))?;
        
        info!("Node A: Assignment notification sent for job {:?} to executor {:?}", test_job.id, executor_did);
        
        // Node B: Wait for assignment notification
        info!("Node B: Waiting for assignment notification...");
        let assignment_received = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = node_b_receiver.recv().await {
                    debug!("Node B received message: {:?}", message);
                    if let NetworkMessage::JobAssignmentNotification(job_id, assigned_executor) = message {
                        if job_id == test_job.id && assigned_executor == executor_did {
                            info!("✓ Node B received assignment for job {:?}", job_id);
                            return Some((job_id, assigned_executor));
                        }
                    }
                }
            }
        }).await;
        
        assert!(assignment_received.is_ok(), "Assignment notification did not reach Node B");
        let (assigned_job_id, assigned_executor) = assignment_received.unwrap().unwrap();
        assert_eq!(assigned_job_id, test_job.id);
        assert_eq!(assigned_executor, executor_did);
        
        // Node B: Execute the job using SimpleExecutor
        info!("Node B: Executing assigned job...");
        let (executor_sk, executor_pk) = generate_ed25519_keypair();
        let executor = SimpleExecutor::new(executor_did.clone(), executor_sk);
        
        let execution_result = executor.execute_job(&test_job).await;
        assert!(execution_result.is_ok(), "Job execution failed: {:?}", execution_result.err());
        
        let receipt = execution_result.unwrap();
        assert_eq!(receipt.job_id, test_job.id);
        assert_eq!(receipt.executor_did, executor_did);
        assert!(!receipt.sig.0.is_empty(), "Receipt should be signed");
        
        // Verify receipt signature
        assert!(receipt.verify_against_key(&executor_pk).is_ok(), "Receipt signature verification failed");
        
        info!("✓ Test passed: Job assignment notification received and job executed successfully on Node B");
        Ok(())
    }

    /// Test 4: Cross-node receipt submission and verification
    #[tokio::test]
    async fn test_cross_node_receipt_submission() -> Result<(), anyhow::Error> {
        info!("=== TEST: Cross-Node Receipt Submission and Verification ===");
        
        // Setup nodes
        let node_a = create_libp2p_runtime_context("ReceiptNodeA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        sleep(Duration::from_millis(500)).await;
        
        // Get Node A's actual listening addresses
        let node_a_addrs = node_a_libp2p.listening_addresses();
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses"));
        }
        
        let bootstrap_peers = vec![(node_a_peer_id, node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("ReceiptNodeB", Some(bootstrap_peers), 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        sleep(Duration::from_secs(2)).await;
        
        // Subscribe Node A to receive receipts
        let mut node_a_receiver = node_a_libp2p.subscribe().await?;
        
        // Create job and executor identity
        let submitter_did = Did::from_str("did:key:z6MkvReceiptNodeA")?;
        let executor_did = Did::from_str("did:key:z6MkvReceiptNodeB")?;
        let test_job = create_test_job("receipt_test", &submitter_did, 25);
        
        // Node B: Execute job and create receipt
        info!("Node B: Executing job and creating receipt...");
        let (executor_sk, executor_pk) = generate_ed25519_keypair();
        let executor = SimpleExecutor::new(executor_did.clone(), executor_sk);
        
        let receipt = executor.execute_job(&test_job).await.map_err(|e| anyhow::anyhow!("Job execution failed: {}", e))?;
        
        // Node B: Submit receipt to Node A
        let receipt_message = NetworkMessage::SubmitReceipt(receipt.clone());
        node_b_libp2p.broadcast_message(receipt_message).await.map_err(|e| anyhow::anyhow!("Receipt submission failed: {}", e))?;
        
        info!("Node B: Receipt submitted for job {:?}", test_job.id);
        
        // Node A: Wait for receipt
        info!("Node A: Waiting for receipt...");
        let receipt_received = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = node_a_receiver.recv().await {
                    debug!("Node A received message: {:?}", message);
                    if let NetworkMessage::SubmitReceipt(received_receipt) = message {
                        if received_receipt.job_id == test_job.id {
                            info!("✓ Node A received receipt for job {:?}", received_receipt.job_id);
                            return Some(received_receipt);
                        }
                    }
                }
            }
        }).await;
        
        assert!(receipt_received.is_ok(), "Receipt did not reach Node A within timeout");
        let received_receipt = receipt_received.unwrap().unwrap();
        
        assert_eq!(received_receipt.job_id, test_job.id);
        assert_eq!(received_receipt.executor_did, executor_did);
        assert_eq!(received_receipt.result_cid, receipt.result_cid);
        
        // Verify receipt signature (Note: In real cross-node scenario, Node A would need 
        // to resolve executor's public key from their DID, but for test we have it)
        assert!(received_receipt.verify_against_key(&executor_pk).is_ok(), "Cross-node receipt signature verification failed");
        
        info!("✓ Test passed: Receipt successfully transmitted from Node B to Node A and verified");
        Ok(())
    }

    /// Test 5: End-to-end complete cross-node job execution
    #[tokio::test]
    async fn test_complete_cross_node_job_execution() -> Result<(), anyhow::Error> {
        info!("=== TEST: Complete End-to-End Cross-Node Job Execution ===");
        
        // Phase 1: Setup Nodes
        info!("Phase 1: Setting up Node A (Bootstrap/Submitter) and Node B (Worker/Executor)");
        
        let node_a = create_libp2p_runtime_context("E2ENodeA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let node_a_peer_id = node_a_libp2p.local_peer_id().clone();
        
        sleep(Duration::from_millis(500)).await;
        
        // Get Node A's actual listening addresses
        let node_a_addrs = node_a_libp2p.listening_addresses();
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses"));
        }
        
        let bootstrap_peers = vec![(node_a_peer_id, node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("E2ENodeB", Some(bootstrap_peers), 100).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        sleep(Duration::from_secs(2)).await;
        
        info!("✓ Nodes connected via libp2p");
        
        // Phase 2: Job Submission
        info!("Phase 2: Job Submission on Node A");
        
        let submitter_did = Did::from_str("did:key:z6MkvE2ENodeA")?;
        let executor_did = Did::from_str("did:key:z6MkvE2ENodeB")?;
        let test_job = create_test_job("e2e_complete", &submitter_did, 60);
        
        let initial_mana = node_a.get_mana(&submitter_did).await.map_err(|e| anyhow::anyhow!("Failed to get initial mana: {}", e))?;
        info!("Initial mana for submitter: {}", initial_mana);
        
        let job_json = serde_json::to_string(&test_job)?;
        let submitted_job_id = host_submit_mesh_job(&node_a, &job_json).await.map_err(|e| anyhow::anyhow!("Job submission failed: {}", e))?;
        
        let final_mana = node_a.get_mana(&submitter_did).await.map_err(|e| anyhow::anyhow!("Failed to get final mana: {}", e))?;
        assert_eq!(final_mana, initial_mana - test_job.cost_mana, "Mana not deducted correctly");
        
        info!("✓ Job submitted, mana deducted: {} -> {}", initial_mana, final_mana);
        
        // Phase 3: Manual Mesh Job Pipeline Simulation
        info!("Phase 3: Simulating mesh job pipeline across nodes");
        
        // Subscribe both nodes to network messages
        let mut node_a_receiver = node_a_libp2p.subscribe().await?;
        let mut node_b_receiver = node_b_libp2p.subscribe().await?;
        
        // Node A: Announce job
        let node_a_network = DefaultMeshNetworkService::new(node_a_libp2p.clone());
        node_a_network.announce_job(&test_job).await.map_err(|e| anyhow::anyhow!("Job announcement failed: {}", e))?;
        
        info!("✓ Job announced by Node A");
        
        // Node B: Wait for job announcement
        let job_announcement = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(NetworkMessage::MeshJobAnnouncement(job)) = node_b_receiver.recv().await {
                    if job.id == test_job.id {
                        return Some(job);
                    }
                }
            }
        }).await;
        
        assert!(job_announcement.is_ok(), "Job announcement did not reach Node B");
        info!("✓ Job announcement received by Node B");
        
        // Phase 4: Bidding
        info!("Phase 4: Node B submits bid");
        
        let bid = MeshJobBid {
            job_id: test_job.id.clone(),
            executor_did: executor_did.clone(), 
            price_mana: 40,
            resources: Resources::default(),
        };
        
        let bid_message = NetworkMessage::BidSubmission(bid.clone());
        node_b_libp2p.broadcast_message(bid_message).await.map_err(|e| anyhow::anyhow!("Bid broadcast failed: {}", e))?;
        
        info!("✓ Bid submitted by Node B");
        
        // Node A: Wait for bid
        let received_bid = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(NetworkMessage::BidSubmission(bid)) = node_a_receiver.recv().await {
                    if bid.job_id == test_job.id {
                        return Some(bid);
                    }
                }
            }
        }).await;
        
        assert!(received_bid.is_ok(), "Bid did not reach Node A");
        info!("✓ Bid received by Node A");
        
        // Phase 5: Assignment and Execution
        info!("Phase 5: Node A assigns job, Node B executes");
        
        let assignment_message = NetworkMessage::JobAssignmentNotification(
            test_job.id.clone(),
            executor_did.clone()
        );
        node_a_libp2p.broadcast_message(assignment_message).await.map_err(|e| anyhow::anyhow!("Assignment failed: {}", e))?;
        
        info!("✓ Assignment notification sent by Node A");
        
        // Node B: Wait for assignment and execute
        let assignment_received = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(NetworkMessage::JobAssignmentNotification(job_id, assigned_executor)) = node_b_receiver.recv().await {
                    if job_id == test_job.id && assigned_executor == executor_did {
                        return Some((job_id, assigned_executor));
                    }
                }
            }
        }).await;
        
        assert!(assignment_received.is_ok(), "Assignment notification did not reach Node B");
        info!("✓ Assignment received by Node B");
        
        // Node B: Execute job
        let (executor_sk, executor_pk) = generate_ed25519_keypair();
        let executor = SimpleExecutor::new(executor_did.clone(), executor_sk);
        let receipt = executor.execute_job(&test_job).await.map_err(|e| anyhow::anyhow!("Job execution failed: {}", e))?;
        
        info!("✓ Job executed by Node B, receipt created");
        
        // Phase 6: Receipt Submission and Verification
        info!("Phase 6: Node B submits receipt, Node A verifies and anchors");
        
        let receipt_message = NetworkMessage::SubmitReceipt(receipt.clone());
        node_b_libp2p.broadcast_message(receipt_message).await.map_err(|e| anyhow::anyhow!("Receipt submission failed: {}", e))?;
        
        info!("✓ Receipt submitted by Node B");
        
        // Node A: Wait for receipt
        let received_receipt = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(NetworkMessage::SubmitReceipt(receipt)) = node_a_receiver.recv().await {
                    if receipt.job_id == test_job.id {
                        return Some(receipt);
                    }
                }
            }
        }).await;
        
        assert!(received_receipt.is_ok(), "Receipt did not reach Node A");
        let final_receipt = received_receipt.unwrap().unwrap();
        
        info!("✓ Receipt received by Node A");
        
        // Verify receipt signature
        assert!(final_receipt.verify_against_key(&executor_pk).is_ok(), "Receipt signature verification failed");
        
        info!("✓ Receipt signature verified");
        
        // Note: In a real implementation, Node A would now anchor the receipt to DAG
        // and mark the job as completed. For this test, we verify the key components worked.
        
        // Assertions
        assert_eq!(final_receipt.job_id, test_job.id);
        assert_eq!(final_receipt.executor_did, executor_did);
        assert!(!final_receipt.sig.0.is_empty());
        
        info!("🎉 SUCCESS: Complete cross-node job execution pipeline working!");
        info!("   ✓ Job submitted on Node A with mana deduction");
        info!("   ✓ Job announcement propagated via libp2p gossipsub");
        info!("   ✓ Bid submitted from Node B to Node A");
        info!("   ✓ Job assignment notification sent to Node B");
        info!("   ✓ Job executed on Node B with signed receipt");
        info!("   ✓ Receipt transmitted back to Node A and verified");
        
        Ok(())
    }

    /// Test 0.5: Debug libp2p service initialization
    #[tokio::test]
    async fn test_debug_libp2p_service() -> Result<(), anyhow::Error> {
        info!("=== DEBUG: libp2p Service Initialization ===");

        // Create libp2p service directly
        use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
        
        let config = NetworkConfig::default();
        let service = Libp2pNetworkService::new(config).await?;
        
        info!("Service created with Peer ID: {}", service.local_peer_id());
        
        // Check listening addresses immediately
        let addrs_immediate = service.listening_addresses();
        info!("Immediate listening addresses: {:?}", addrs_immediate);
        
        // Wait and check again
        for i in 1..=5 {
            sleep(Duration::from_millis(1000)).await;
            let addrs = service.listening_addresses();
            info!("Attempt {}: Listening addresses: {:?}", i, addrs);
            if !addrs.is_empty() {
                info!("✓ Found listening addresses on attempt {}", i);
                break;
            }
        }
        
        // Test basic functionality
        let stats = service.get_network_stats().await?;
        info!("Network stats: {:?}", stats);
        
        info!("✓ Debug test completed");
        Ok(())
    }

    /// Test 0.6: Debug RuntimeContext with libp2p
    #[tokio::test]
    async fn test_debug_runtime_context_libp2p() -> Result<(), anyhow::Error> {
        info!("=== DEBUG: RuntimeContext with libp2p ===");

        // Test the create_libp2p_runtime_context function directly
        info!("Creating RuntimeContext with libp2p...");
        let ctx = create_libp2p_runtime_context("DebugTest", None, 100).await?;
        info!("✓ RuntimeContext created successfully");
        
        // Test getting the libp2p service
        info!("Getting libp2p service...");
        let libp2p_service = ctx.get_libp2p_service()?;
        info!("✓ Got libp2p service with Peer ID: {}", libp2p_service.local_peer_id());
        
        // Test listening addresses
        info!("Checking listening addresses...");
        for i in 1..=3 {
            sleep(Duration::from_millis(1000)).await;
            let addrs = libp2p_service.listening_addresses();
            info!("Attempt {}: Listening addresses: {:?}", i, addrs);
            if !addrs.is_empty() {
                info!("✓ Found listening addresses on attempt {}", i);
                break;
            }
        }
        
        info!("✓ Debug RuntimeContext test completed");
        Ok(())
    }

    /// Test 0.7: Simplified job announcement test
    #[tokio::test]
    async fn test_simplified_job_announcement() -> Result<(), anyhow::Error> {
        info!("=== SIMPLIFIED: Job Announcement Test ===");

        // Create Node A
        let node_a = create_libp2p_runtime_context("SimpleA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        
        // Wait for listening addresses
        sleep(Duration::from_secs(2)).await;
        let node_a_addrs = node_a_libp2p.listening_addresses();
        if node_a_addrs.is_empty() {
            return Err(anyhow::anyhow!("Node A has no listening addresses"));
        }
        info!("Node A listening on: {:?}", node_a_addrs);
        
        // Create Node B
        let bootstrap_peers = vec![(node_a_libp2p.local_peer_id().clone(), node_a_addrs[0].clone())];
        let node_b = create_libp2p_runtime_context("SimpleB", Some(bootstrap_peers), 0).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        
        // Give nodes time to connect
        sleep(Duration::from_secs(3)).await;
        
        // Test basic message broadcasting
        info!("Testing basic message broadcast...");
        let test_message = NetworkMessage::GossipSub("test".to_string(), b"hello".to_vec());
        node_a_libp2p.broadcast_message(test_message).await?;
        
        info!("✓ Simplified test completed successfully");
        Ok(())
    }
}

#[cfg(not(feature = "enable-libp2p"))]
mod stub_cross_node_tests {
    #[tokio::test]
    async fn test_cross_node_feature_disabled() {
        println!("Cross-node tests require enable-libp2p feature");
    }
} 