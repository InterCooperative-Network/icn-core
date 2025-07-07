#[cfg(feature = "enable-libp2p")]
mod multi_node_libp2p {
    use icn_common::{Cid, Did};
    use icn_identity::{generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobSpec, MeshJobBid, Resources};
    use icn_network::NetworkService;
    use icn_protocol::{MeshJobAssignmentMessage, MessagePayload, ProtocolMessage};
    use icn_runtime::context::{DefaultMeshNetworkService, MeshNetworkService, RuntimeContext};
    use icn_runtime::executor::{JobExecutor, SimpleExecutor};
    use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use log::info;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration};

    fn create_test_job(job_id_suffix: &str, creator_did: &Did, cost_mana: u64) -> ActualMeshJob {
        let job_id = Cid::new_v1_sha256(0x55, format!("test_job_{}", job_id_suffix).as_bytes());
        let manifest_cid =
            Cid::new_v1_sha256(0x55, format!("manifest_{}", job_id_suffix).as_bytes());
        ActualMeshJob {
            id: job_id,
            manifest_cid,
            spec: JobSpec::Echo {
                payload: format!("Cross-node test job {}", job_id_suffix),
            },
            creator_did: creator_did.clone(),
            cost_mana,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]),
        }
    }

    async fn create_libp2p_runtime_context(
        identity_suffix: &str,
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
        initial_mana: u64,
    ) -> Result<Arc<RuntimeContext>, anyhow::Error> {
        let identity_str = format!("did:key:z6Mkv{}", identity_suffix);
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let ctx = RuntimeContext::new_with_real_libp2p(
            &identity_str,
            listen,
            bootstrap_peers,
            std::path::PathBuf::from("./mana_ledger.sled"),
            std::path::PathBuf::from("./reputation.sled"),
        )
        .await?;
        let identity = Did::from_str(&identity_str)?;
        ctx.mana_ledger
            .set_balance(&identity, initial_mana)
            .expect("init mana");
        Ok(ctx)
    }

    #[tokio::test]
    async fn job_execution_receipt_and_reputation() -> Result<(), anyhow::Error> {
        env_logger::try_init().ok();

        // --- Setup nodes ---
        let node_a = create_libp2p_runtime_context("IntA", None, 1000).await?;
        let node_a_libp2p = node_a.get_libp2p_service()?;
        let peer_a = node_a_libp2p.local_peer_id().clone();
        sleep(Duration::from_millis(500)).await;
        let addr_a = node_a_libp2p
            .listening_addresses()
            .get(0)
            .cloned()
            .expect("node A address");
        let bootstrap = vec![(peer_a, addr_a)];
        let node_b = create_libp2p_runtime_context("IntB", Some(bootstrap), 100).await?;
        let node_b_libp2p = node_b.get_libp2p_service()?;
        sleep(Duration::from_secs(2)).await;

        // --- Submit job on Node A ---
        let submitter_did = node_a.current_identity.clone();
        let executor_did = node_b.current_identity.clone();
        let test_job = create_test_job("integration", &submitter_did, 50);
        let job_json = serde_json::to_string(&test_job)?;
        let job_id = host_submit_mesh_job(&node_a, &job_json).await?;

        // --- Manual pipeline via network messages ---
        let mut recv_a = node_a_libp2p.subscribe().await?;
        let mut recv_b = node_b_libp2p.subscribe().await?;
        let network_a = DefaultMeshNetworkService::new(node_a_libp2p.clone());
        network_a.announce_job(&test_job).await?;

        // Wait for announcement
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAnnouncement(job) = &message.payload {
                        if job.id == job_id {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Send bid from Node B
        let unsigned_bid = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_did.clone(),
            price_mana: 30,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };
        let bid_bytes = unsigned_bid.to_signable_bytes().unwrap();
        let sig = node_b.signer.sign(&bid_bytes).unwrap();
        let bid = MeshJobBid {
            signature: SignatureBytes(sig),
            ..unsigned_bid
        };
        let msg = ProtocolMessage::new(
            MessagePayload::MeshBidSubmission(bid),
            executor_did.clone(),
            None,
        );
        node_b_libp2p.broadcast_message(msg).await?;

        // Wait for bid on Node A
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_a.recv().await {
                    if let MessagePayload::MeshBidSubmission(b) = &message.payload {
                        if b.job_id == job_id {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Assign job to Node B
        let assign_msg = ProtocolMessage::new(
            MessagePayload::MeshJobAssignment(MeshJobAssignmentMessage {
                job_id: job_id.clone(),
                executor_did: executor_did.clone(),
            }),
            node_a.current_identity.clone(),
            None,
        );
        node_a_libp2p.broadcast_message(assign_msg).await?;

        // Wait for assignment on Node B
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAssignment(assign) = &message.payload {
                        if assign.job_id == job_id && assign.executor_did == executor_did {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Node B executes job
        let (sk, pk) = generate_ed25519_keypair();
        let executor = SimpleExecutor::new(executor_did.clone(), sk);
        let receipt = executor.execute_job(&test_job).await?;
        assert!(receipt.verify_against_key(&pk).is_ok());

        // Submit receipt
        let receipt_msg = ProtocolMessage::new(
            MessagePayload::MeshReceiptSubmission(receipt.clone()),
            executor_did.clone(),
            None,
        );
        node_b_libp2p.broadcast_message(receipt_msg).await?;

        // Node A waits for receipt
        let final_receipt = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_a.recv().await {
                    if let MessagePayload::MeshReceiptSubmission(r) = &message.payload {
                        if r.job_id == job_id {
                            break r.clone();
                        }
                    }
                }
            }
        })
        .await?;

        // Anchor receipt and update reputation
        let rep_before = node_a.reputation_store.get_reputation(&executor_did);
        let receipt_json = serde_json::to_string(&final_receipt)?;
        let cid = host_anchor_receipt(&node_a, &receipt_json, &ReputationUpdater::new()).await?;
        let rep_after = node_a.reputation_store.get_reputation(&executor_did);
        assert!(rep_after > rep_before, "reputation should increase");
        let stored = node_a
            .dag_store
            .lock()
            .await
            .get(&cid)?
            .expect("receipt stored");
        assert_eq!(stored.cid, cid);
        info!("Test completed with anchored receipt {:?}", cid);
        Ok(())
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn libp2p_feature_disabled_stub() {
    println!("libp2p feature disabled; skipping multi-node test");
}
