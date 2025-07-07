#[cfg(feature = "enable-libp2p")]
mod job_pipeline {
    use icn_common::{Cid, Did};
    use icn_identity::{generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobSpec, MeshJobBid, Resources};
    use icn_protocol::{MeshJobAssignmentMessage, MessagePayload, ProtocolMessage};
    use icn_runtime::context::{DefaultMeshNetworkService, RuntimeContext};
    use icn_runtime::executor::SimpleExecutor;
    use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};
    use libp2p::Multiaddr;
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn runtime_context_two_node_pipeline() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        // --- Node A setup ---
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let node_a = RuntimeContext::new_with_real_libp2p(
            "did:key:z6MktestA",
            listen.clone(),
            None,
            std::path::PathBuf::from("./mana_a.sled"),
            std::path::PathBuf::from("./rep_a.sled"),
        )
        .await?;
        node_a
            .mana_ledger
            .set_balance(&node_a.current_identity, 1000)
            .unwrap();
        let service_a = node_a.get_libp2p_service()?;
        sleep(Duration::from_millis(500)).await;
        let peer_a = *service_a.local_peer_id();
        let addr_a = service_a
            .listening_addresses()
            .get(0)
            .cloned()
            .expect("addr_a");

        // --- Node B setup bootstrapped to A ---
        let node_b = RuntimeContext::new_with_real_libp2p(
            "did:key:z6MktestB",
            listen,
            Some(vec![(peer_a, addr_a.clone())]),
            std::path::PathBuf::from("./mana_b.sled"),
            std::path::PathBuf::from("./rep_b.sled"),
        )
        .await?;
        let service_b = node_b.get_libp2p_service()?;
        sleep(Duration::from_secs(2)).await;

        let rep_before = node_a
            .reputation_store
            .get_reputation(&node_b.current_identity);

        let mut recv_a = service_a.subscribe().await?;
        let mut recv_b = service_b.subscribe().await?;
        let mesh_a = DefaultMeshNetworkService::new(service_a.clone());

        // --- Submit job on Node A ---
        let job = create_job("pipeline", &node_a.current_identity);
        let job_json = serde_json::to_string(&job)?;
        let job_id = host_submit_mesh_job(&node_a, &job_json).await?;

        {
            let states = node_a.job_states.lock().await;
            assert!(matches!(states.get(&job_id), Some(icn_mesh::JobState::Pending)));
        }

        mesh_a.announce_job(&job).await?;

        // Wait for announcement
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAnnouncement(j) = &message.payload {
                        if j.id == job_id {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Node B sends bid
        let unsigned = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: node_b.current_identity.clone(),
            price_mana: 30,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };
        let sig = node_b
            .signer
            .sign(&unsigned.to_signable_bytes().unwrap())
            .unwrap();
        let bid = MeshJobBid {
            signature: SignatureBytes(sig),
            ..unsigned
        };
        let msg = ProtocolMessage::new(
            MessagePayload::MeshBidSubmission(bid),
            node_b.current_identity.clone(),
            None,
        );
        service_b.broadcast_message(msg).await?;

        // Wait for bid on A
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

        // Assign job to B
        let msg = ProtocolMessage::new(
            MessagePayload::MeshJobAssignment(MeshJobAssignmentMessage {
                job_id: job_id.clone(),
                executor_did: node_b.current_identity.clone(),
            }),
            node_a.current_identity.clone(),
            None,
        );
        service_a.broadcast_message(msg).await?;

        {
            let mut states = node_a.job_states.lock().await;
            states.insert(
                job_id.clone(),
                icn_mesh::JobState::Assigned {
                    executor: node_b.current_identity.clone(),
                },
            );
        }

        // Wait for assignment on B
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAssignment(assign) = &message.payload {
                        if assign.job_id == job_id && assign.executor_did == node_b.current_identity {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Execute job on B
        let (sk, pk) = generate_ed25519_keypair();
        let executor = SimpleExecutor::new(node_b.current_identity.clone(), sk);
        let receipt = executor.execute_job(&job).await?;
        assert!(receipt.verify_against_key(&pk).is_ok());

        let msg = ProtocolMessage::new(
            MessagePayload::MeshReceiptSubmission(receipt.clone()),
            node_b.current_identity.clone(),
            None,
        );
        service_b.broadcast_message(msg).await?;

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
        .await?
        .unwrap();

        let receipt_json = serde_json::to_string(&final_receipt)?;
        let cid = host_anchor_receipt(&node_a, &receipt_json, &ReputationUpdater::new()).await?;

        {
            let states = node_a.job_states.lock().await;
            match states.get(&job_id) {
                Some(icn_mesh::JobState::Completed { .. }) => {}
                other => panic!("Job not completed: {:?}", other),
            }
        }

        let rep_after = node_a
            .reputation_store
            .get_reputation(&node_b.current_identity);
        assert!(rep_after > rep_before, "reputation should increase");

        let stored = node_a
            .dag_store
            .lock()
            .await
            .get(&cid)?
            .expect("receipt stored");
        assert_eq!(stored.cid, cid);

        Ok(())
    }

    fn create_job(suffix: &str, creator: &Did) -> ActualMeshJob {
        let job_id = Cid::new_v1_sha256(0x55, format!("job_{suffix}").as_bytes());
        let manifest_cid = Cid::new_v1_sha256(0x55, format!("manifest_{suffix}").as_bytes());
        ActualMeshJob {
            id: job_id,
            manifest_cid,
            spec: JobSpec::Echo {
                payload: "two node".into(),
            },
            creator_did: creator.clone(),
            cost_mana: 50,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]),
        }
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn libp2p_feature_disabled_stub() {
    println!("libp2p feature disabled; skipping job pipeline test");
}
