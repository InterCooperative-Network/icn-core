#[path = "federation.rs"]
mod federation;

#[cfg(feature = "enable-libp2p")]
mod libp2p_job_pipeline {
    use super::federation::{ensure_devnet, NODE_A_URL, NODE_B_URL, NODE_C_URL};

    use base64::Engine;
    use icn_common::{Cid, Did};
    use icn_identity::{generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobId, JobKind, JobSpec, JobState, MeshJobBid, Resources};
    use icn_network::NetworkService;
    use icn_protocol::{MeshJobAssignmentMessage, MessagePayload, ProtocolMessage};
    use icn_runtime::context::{DefaultMeshNetworkService, MeshNetworkService, RuntimeContext};
    use icn_runtime::executor::{JobExecutor, SimpleExecutor};
    use icn_runtime::{host_anchor_receipt, host_submit_mesh_job, ReputationUpdater};
    use libp2p::Multiaddr;
    use std::sync::Arc;
    use reqwest::Client;
    use serde_json::Value;

    use tokio::time::{sleep, timeout, Duration};

    const RETRY_DELAY: Duration = Duration::from_secs(3);
    const MAX_RETRIES: u32 = 20;

    async fn extract_did(client: &Client, url: &str) -> String {
        let info: Value = client
            .get(&format!("{}/info", url))
            .send()
            .await
            .expect("info request")
            .json()
            .await
            .expect("info json");
        let status = info["status_message"].as_str().unwrap_or("");
        status
            .trim_start_matches("Node DID: ")
            .split(',')
            .next()
            .unwrap_or("")
            .to_string()
    }

    #[tokio::test]
    async fn job_exec_across_nodes() {
        let _devnet = ensure_devnet().await;
        let client = Client::new();

        let node_a_did = extract_did(&client, NODE_A_URL).await;
        let node_b_did = extract_did(&client, NODE_B_URL).await;
        let node_c_did = extract_did(&client, NODE_C_URL).await;

        let spec = icn_mesh::JobSpec {
            kind: icn_mesh::JobKind::Echo {
                payload: "libp2p pipeline test".into(),
            },
            ..Default::default()
        };
        let job_request = serde_json::json!({
            "manifest_cid": "cidv1-libp2p-test-manifest",
            "spec_bytes": base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&spec).unwrap()),
            "spec_json": null,
            "cost_mana": 50
        });

        let submit_res: Value = client
            .post(&format!("{}/mesh/submit", NODE_A_URL))
            .header("Content-Type", "application/json")
            .json(&job_request)
            .send()
            .await
            .expect("submit job")
            .json()
            .await
            .expect("submit json");

        let job_id = submit_res["job_id"].as_str().expect("job_id").to_string();

        let mut final_status: Value = Value::Null;
        for _ in 0..MAX_RETRIES {
            let resp = client
                .get(&format!("{}/mesh/jobs/{}", NODE_A_URL, job_id))
                .send()
                .await
                .expect("status");
            if resp.status().is_success() {
                final_status = resp.json().await.expect("status json");
                if final_status["status"]["status"] == "completed" {
                    break;
                }
            }
            sleep(RETRY_DELAY).await;
        }

        assert_eq!(final_status["status"]["status"], "completed");
        let executor = final_status["status"]["executor"]
            .as_str()
            .expect("executor");
        assert_ne!(executor, node_a_did);

        let executor_url = if executor == node_b_did {
            NODE_B_URL
        } else if executor == node_c_did {
            NODE_C_URL
        } else {
            panic!("executor DID not recognized: {}", executor);
        };

        let exec_status: Value = client
            .get(&format!("{}/mesh/jobs/{}", executor_url, job_id))
            .send()
            .await
            .expect("executor status")
            .json()
            .await
            .expect("executor status json");
        assert_eq!(exec_status["status"]["status"], "completed");

        let result_cid = exec_status["status"]["result_cid"]
            .as_str()
            .expect("result_cid");

        for url in [NODE_A_URL, NODE_B_URL, NODE_C_URL] {
            let dag_res = client
                .post(&format!("{}/dag/get", url))
                .json(&serde_json::json!({ "cid": result_cid }))
                .send()
                .await
                .expect("dag get");
            assert!(dag_res.status().is_success(), "dag get failed on {}", url);
        }
    }

    #[tokio::test]
    async fn ccl_contract_job_execs() {
        let _devnet = ensure_devnet().await;
        let client = Client::new();

        let contract_source = "fn run() -> Integer { return 7; }";
        let contract_res: Value = client
            .post(&format!("{}/contracts", NODE_A_URL))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "source": contract_source }))
            .send()
            .await
            .expect("compile contract")
            .json()
            .await
            .expect("contract json");

        let manifest_cid = contract_res["manifest_cid"].as_str().expect("cid");
        let job_spec = icn_mesh::JobSpec {
            kind: icn_mesh::JobKind::CclWasm,
            ..Default::default()
        };
        let job_request = serde_json::json!({
            "manifest_cid": manifest_cid,
            "spec_bytes": base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&job_spec).unwrap()),
            "spec_json": null,
            "cost_mana": 50
        });

        let submit_res: Value = client
            .post(&format!("{}/mesh/submit", NODE_A_URL))
            .header("Content-Type", "application/json")
            .json(&job_request)
            .send()
            .await
            .expect("submit job")
            .json()
            .await
            .expect("submit json");

        let job_id = submit_res["job_id"].as_str().expect("job_id").to_string();

        let mut final_status: Value = Value::Null;
        for _ in 0..MAX_RETRIES {
            let resp = client
                .get(&format!("{}/mesh/jobs/{}", NODE_A_URL, job_id))
                .send()
                .await
                .expect("status");
            if resp.status().is_success() {
                final_status = resp.json().await.expect("status json");
                if final_status["status"]["status"] == "completed" {
                    break;
                }
            }
            sleep(RETRY_DELAY).await;
        }

        assert_eq!(final_status["status"]["status"], "completed");
        let result_cid = final_status["status"]["result_cid"].as_str().unwrap();
        let expected = icn_common::Cid::new_v1_sha256(0x55, &7i64.to_le_bytes());
        assert_eq!(result_cid, expected.to_string());

        for url in [NODE_A_URL, NODE_B_URL, NODE_C_URL] {
            let dag_res = client
                .post(&format!("{}/dag/get", url))
                .json(&serde_json::json!({ "cid": result_cid }))
                .send()
                .await
                .expect("dag get");
            assert!(dag_res.status().is_success(), "dag get failed on {}", url);
        }
    }

    #[tokio::test]
    async fn runtime_context_two_node_pipeline() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        // --- Node A setup ---
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let node_a = RuntimeContext::new_with_real_libp2p(
            "did:key:z6MktestA",
            listen.clone(),
            None,
            Arc::new(icn_runtime::context::StubSigner::new()),
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
            .first()
            .cloned()
            .expect("addr_a");

        // --- Node B setup bootstrapped to A ---
        let node_b = RuntimeContext::new_with_real_libp2p(
            "did:key:z6MktestB",
            listen,
            Some(vec![(peer_a, addr_a.clone())]),
            Arc::new(icn_runtime::context::StubSigner::new()),
        )
        .await?;
        let service_b = node_b.get_libp2p_service()?;
        sleep(Duration::from_secs(2)).await;

        let rep_before = node_a
            .reputation_store
            .get_reputation(&node_b.current_identity);

        let mut recv_a = service_a.subscribe().await?;
        let mut recv_b = service_b.subscribe().await?;
        let mesh_a = DefaultMeshNetworkService::new(service_a.clone(), Arc::new(icn_runtime::context::StubSigner::new()));

        // --- Submit job on Node A ---
        let job = create_job("pipeline", &node_a.current_identity);
        let job_json = serde_json::to_string(&job)?;
        let job_id = host_submit_mesh_job(&node_a, &job_json).await?;

        {
            assert!(matches!(
                node_a.job_states.get(&job_id).map(|s| s.value().clone()),
                Some(JobState::Pending)
            ));
        }

        mesh_a.announce_job(&job).await?;

        // Wait for announcement
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAnnouncement(j) = &message.payload {
                        if j.job_id == job_id.clone().into() {
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
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };
        let sig = node_b
            .signer
            .sign(&unsigned.to_signable_bytes().unwrap())
            .unwrap();
        let _bid = MeshJobBid {
            signature: SignatureBytes(sig),
            ..unsigned
        };
        let bid_message = icn_protocol::MeshBidSubmissionMessage {
            job_id: job_id.clone().into(),
            executor_did: node_b.current_identity.clone(),
            cost_mana: 30,
            estimated_duration_secs: 300,
            offered_resources: icn_protocol::ResourceRequirements::default(),
            reputation_score: 100,
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
        };
        let msg = ProtocolMessage::new(
            MessagePayload::MeshBidSubmission(bid_message),
            node_b.current_identity.clone(),
            None,
        );
        service_b.broadcast_message(msg).await?;

        // Wait for bid on A
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_a.recv().await {
                    if let MessagePayload::MeshBidSubmission(b) = &message.payload {
                        if b.job_id == job_id.clone().into() {
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
                job_id: job_id.clone().into(),
                executor_did: node_b.current_identity.clone(),
                agreed_cost_mana: 30,
                completion_deadline: chrono::Utc::now().timestamp() as u64 + 3600,
                manifest_cid: None,
            }),
            node_a.current_identity.clone(),
            None,
        );
        service_a.broadcast_message(msg).await?;

        {
            node_a.job_states.insert(
                job_id.clone(),
                JobState::Assigned {
                    executor: node_b.current_identity.clone(),
                },
            );
        }

        // Wait for assignment on B
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAssignment(assign) = &message.payload {
                        if assign.job_id == job_id.clone().into() && assign.executor_did == node_b.current_identity
                        {
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

        let receipt_message = icn_protocol::MeshReceiptSubmissionMessage {
            receipt: receipt.clone(),
            execution_metadata: icn_protocol::ExecutionMetadata {
                wall_time_ms: 1000,
                peak_memory_mb: 64,
                exit_code: 0,
                execution_logs: Some("Job executed successfully".to_string()),
            },
        };
        let msg = ProtocolMessage::new(
            MessagePayload::MeshReceiptSubmission(receipt_message),
            node_b.current_identity.clone(),
            None,
        );
        service_b.broadcast_message(msg).await?;

        // Node A waits for receipt
        let final_receipt = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_a.recv().await {
                    if let MessagePayload::MeshReceiptSubmission(r) = &message.payload {
                        if r.receipt.job_id == job_id.clone().into() {
                            break r.clone();
                        }
                    }
                }
            }
        })
        .await?;

        let receipt_json = serde_json::to_string(&final_receipt)?;
        let cid = host_anchor_receipt(&node_a, &receipt_json, &ReputationUpdater::new()).await?;

        {
            match node_a.job_states.get(&job_id).map(|s| s.value().clone()) {
                Some(JobState::Completed { .. }) => {}
                other => panic!("Job not completed: {:?}", other),
            }
        }

        let rep_after = node_a
            .reputation_store
            .get_reputation(&node_b.current_identity);
        assert!(rep_after > rep_before, "reputation should increase");

        let stored = node_a
            .dag_store
            .store
            .lock()
            .await
            .get(&cid)
            .await?
            .expect("receipt stored");
        assert_eq!(stored.cid, cid);

        Ok(())
    }

    fn create_job(suffix: &str, creator: &Did) -> ActualMeshJob {
        let job_id = Cid::new_v1_sha256(0x55, format!("job_{suffix}").as_bytes());
        let manifest_cid = Cid::new_v1_sha256(0x55, format!("manifest_{suffix}").as_bytes());
        ActualMeshJob {
            id: JobId(job_id),
            manifest_cid,
            spec: JobSpec {
                kind: JobKind::GenericPlaceholder,
                ..Default::default()
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
    println!("libp2p feature disabled; skipping libp2p job pipeline test");
}
