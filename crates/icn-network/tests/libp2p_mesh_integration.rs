#[cfg(all(test, feature = "experimental-libp2p"))]
mod libp2p_mesh_integration {
    use icn_network::libp2p_service::{Libp2pNetworkService, Libp2pPeerId, Multiaddr};
    use icn_network::{NetworkService, NetworkMessage};
    use icn_common::{Cid, Did, CommonError};
    use icn_mesh::{ActualMeshJob as Job, MeshJobBid as Bid, JobId, JobSpec, Resources};
    use std::str::FromStr;
    use tokio::time::{sleep, Duration, timeout};

    fn generate_dummy_job(id_str: &str) -> Job {
        let job_id = Cid::new_v1_dummy(0x55, 0x13, id_str.as_bytes());
        let creator_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8").unwrap();
        let manifest_cid = Cid::new_v1_dummy(0x71, 0x12, b"dummy_manifest_data");
        Job {
            id: job_id,
            manifest_cid,
            spec: JobSpec::default(),
            creator_did,
            cost_mana: 100,
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
    async fn test_job_announcement_and_bid_submission() -> Result<(), anyhow::Error> {
        // 1. Create Node A (Job Originator)
        let node_a_service = Libp2pNetworkService::new(None).await?;
        let node_a_peer_id_str = node_a_service.local_peer_id().to_string();
        let node_a_addrs = node_a_service.listening_addresses();
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        println!("Node A Peer ID: {}", node_a_peer_id_str);
        println!("Node A Listening Addresses: {:?}", node_a_addrs);


        // 2. Create Node B (Executor)
        // Bootstrap Node B with Node A's address
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
        let bootstrap_peers_for_b = Some(vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())]);
        let node_b_service = Libp2pNetworkService::new(bootstrap_peers_for_b).await?;
        let node_b_peer_id_str = node_b_service.local_peer_id().to_string();
        println!("Node B Peer ID: {}", node_b_peer_id_str);

        // Allow some time for discovery and connection
        sleep(Duration::from_secs(5)).await;

        // 3. Node A subscribes to messages (to receive bids)
        let mut node_a_receiver = node_a_service.subscribe()?;

        // 4. Node B subscribes to messages (to receive job announcements)
        let mut node_b_receiver = node_b_service.subscribe()?;

        // 5. Node A creates and broadcasts a Job Announcement
        let job_to_announce = generate_dummy_job("test_job_01");
        let job_announcement_msg = NetworkMessage::MeshJobAnnouncement(job_to_announce.clone());
        node_a_service.broadcast_message(job_announcement_msg).await?;
        println!("Node A broadcasted job announcement for job ID: {}", job_to_announce.id);

        // 6. Node B receives the Job Announcement
        let received_on_b = timeout(Duration::from_secs(10), node_b_receiver.recv()).await??;
        assert!(received_on_b.is_some(), "Node B receiver channel closed unexpectedly or got None");
        
        if let Some(NetworkMessage::MeshJobAnnouncement(received_job)) = received_on_b {
            assert_eq!(received_job.id, job_to_announce.id, "Node B received incorrect job ID");
            println!("Node B received job announcement for job ID: {}", received_job.id);

            // 7. Node B creates and broadcasts a Bid Submission
            let bid_to_submit = generate_dummy_bid(&received_job.id, "did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH");
            let bid_submission_msg = NetworkMessage::BidSubmission(bid_to_submit.clone());
            node_b_service.broadcast_message(bid_submission_msg).await?;
            println!("Node B broadcasted bid for job ID: {}", received_job.id);

            // 8. Node A receives the Bid Submission
            let received_on_a = timeout(Duration::from_secs(10), node_a_receiver.recv()).await??;
            assert!(received_on_a.is_some(), "Node A receiver channel closed unexpectedly or got None");

            if let Some(NetworkMessage::BidSubmission(received_bid)) = received_on_a {
                assert_eq!(received_bid.job_id, job_to_announce.id, "Node A received bid for incorrect job ID");
                assert_eq!(received_bid.executor_did, bid_to_submit.executor_did, "Node A received bid from incorrect executor");
                println!("Node A received bid for job ID: {} from executor: {}", received_bid.job_id, received_bid.executor_did.to_string());
            } else {
                panic!("Node A did not receive a BidSubmission, but: {:?}", received_on_a);
            }
        } else {
            panic!("Node B did not receive a MeshJobAnnouncement, but: {:?}", received_on_b);
        }

        Ok(())
    }
} 