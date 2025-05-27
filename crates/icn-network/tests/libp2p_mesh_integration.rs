#[cfg(all(test, feature = "experimental-libp2p"))]
mod libp2p_mesh_integration {
    use icn_network::libp2p_service::{Libp2pNetworkService};
    use libp2p::{PeerId as Libp2pPeerId};
    use anyhow::Result;
    use icn_network::{NetworkService, NetworkMessage};
    use icn_common::{Cid, Did};
    use icn_mesh::{ActualMeshJob as Job, MeshJobBid as Bid, JobId, JobSpec, Resources};
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
    #[ignore = "Blocked on environment/macro/import issues related to libp2p Kademlia and tokio macros."]
    async fn test_job_announcement_and_bid_submission() -> Result<(), anyhow::Error> {
        println!("[test-mesh-network] Setting up Node A (Job Originator).");
        // 1. Create Node A (Job Originator)
        let node_a_service = Libp2pNetworkService::new(None).await?;
        let node_a_peer_id_str = node_a_service.local_peer_id().to_string();
        let node_a_addrs = node_a_service.listening_addresses();
        assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
        println!("[test-mesh-network] Node A Peer ID: {}, Listening Addresses: {:?}", node_a_peer_id_str, node_a_addrs);

        println!("[test-mesh-network] Setting up Node B (Executor), bootstrapping with Node A.");
        // 2. Create Node B (Executor)
        let node_a_libp2p_peer_id = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
        let bootstrap_peers_for_b = Some(vec![(node_a_libp2p_peer_id, node_a_addrs[0].clone())]);
        let node_b_service = Libp2pNetworkService::new(bootstrap_peers_for_b).await?;
        let node_b_peer_id_str = node_b_service.local_peer_id().to_string();
        println!("[test-mesh-network] Node B Peer ID: {}", node_b_peer_id_str);

        println!("[test-mesh-network] Allowing 5s for peer discovery and connection.");
        sleep(Duration::from_secs(5)).await;

        println!("[test-mesh-network] Node A subscribing to messages.");
        let mut node_a_receiver = node_a_service.subscribe()?;
        println!("[test-mesh-network] Node B subscribing to messages.");
        let mut node_b_receiver = node_b_service.subscribe()?;

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
} 