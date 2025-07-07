#[cfg(feature = "enable-libp2p")]
mod icn_node_end_to_end {
    use axum::Router;
    use icn_common::{Cid, Did};
    use icn_identity::{generate_ed25519_keypair, SignatureBytes};
    use icn_mesh::{ActualMeshJob, JobSpec, MeshJobBid, Resources};
    use icn_network::NetworkService;
    use icn_node::app_router_from_context;
    use icn_protocol::{MeshJobAssignmentMessage, MessagePayload, ProtocolMessage};
    use icn_runtime::context::{DefaultMeshNetworkService, MeshNetworkService, RuntimeContext};
    use icn_runtime::executor::{JobExecutor, SimpleExecutor};
    use icn_runtime::host_submit_mesh_job;
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use reqwest::Client;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::task::JoinHandle;
    use tokio::time::{sleep, timeout, Duration};

    async fn start_http(router: Router) -> (String, JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        (url, handle)
    }

    async fn create_node(
        suffix: &str,
        bootstrap: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
    ) -> (
        String,
        Arc<RuntimeContext>,
        Arc<dyn NetworkService>,
        JoinHandle<()>,
    ) {
        let did = format!("did:key:z6Mkv{}", suffix);
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let ctx = RuntimeContext::new_with_real_libp2p(
            &did,
            listen,
            bootstrap,
            std::path::PathBuf::from(format!("./dag_{suffix}")),
            std::path::PathBuf::from(format!("./mana_{suffix}.sled")),
            std::path::PathBuf::from(format!("./rep_{suffix}.sled")),
        )
        .await
        .unwrap();
        let did_struct = Did::from_str(&did).unwrap();
        ctx.mana_ledger.set_balance(&did_struct, 1000).unwrap();
        ctx.clone().spawn_mesh_job_manager().await;
        let service = ctx.get_libp2p_service().unwrap();
        sleep(Duration::from_millis(500)).await;
        let router = app_router_from_context(ctx.clone(), None, None, None).await;
        let (url, handle) = start_http(router).await;
        (url, ctx, service as Arc<dyn NetworkService>, handle)
    }

    fn make_job(id_suffix: &str, creator: &Did) -> ActualMeshJob {
        let job_id = Cid::new_v1_sha256(0x55, format!("job_{id_suffix}").as_bytes());
        let manifest_cid = Cid::new_v1_sha256(0x55, format!("manifest_{id_suffix}").as_bytes());
        ActualMeshJob {
            id: job_id,
            manifest_cid,
            spec: JobSpec::Echo {
                payload: "libp2p integration".into(),
            },
            creator_did: creator.clone(),
            cost_mana: 50,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]),
        }
    }

    #[tokio::test]
    async fn job_flow_with_reputation() -> Result<(), anyhow::Error> {
        env_logger::try_init().ok();

        // Node A bootstrap
        let (url_a, ctx_a, net_a, handle_a) = create_node("IntA", None).await;
        let peer_a = net_a
            .as_any()
            .downcast_ref::<icn_network::Libp2pNetworkService>()
            .unwrap()
            .local_peer_id()
            .clone();
        let addr_a = net_a
            .as_any()
            .downcast_ref::<icn_network::Libp2pNetworkService>()
            .unwrap()
            .listening_addresses()[0]
            .clone();

        // Node B bootstrapped to A
        let bootstrap = vec![(peer_a, addr_a.clone())];
        let (url_b, ctx_b, net_b, handle_b) = create_node("IntB", Some(bootstrap)).await;

        // Reputation before
        let rep_before = ctx_a
            .reputation_store
            .get_reputation(&ctx_b.current_identity);

        // Prepare network receivers
        let mut recv_a = net_a.subscribe().await.unwrap();
        let mut recv_b = net_b.subscribe().await.unwrap();
        let net_a_mesh = DefaultMeshNetworkService::new(net_a.clone());

        // Submit job via HTTP to Node A
        let client = Client::new();
        let job = make_job("e2e", &ctx_a.current_identity);
        let job_json = serde_json::to_string(&job)?;
        let submit: serde_json::Value = client
            .post(format!("{}/mesh/submit", url_a))
            .header("Content-Type", "application/json")
            .body(job_json)
            .send()
            .await?
            .json()
            .await?;
        let job_id = submit["job_id"].as_str().unwrap().to_string();

        // Announce job manually so Node B sees it
        net_a_mesh.announce_job(&job).await?;

        // Wait for announcement on B
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAnnouncement(j) = &message.payload {
                        if j.id.to_string() == job_id {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Node B sends bid
        let unsigned = MeshJobBid {
            job_id: job.id.clone(),
            executor_did: ctx_b.current_identity.clone(),
            price_mana: 30,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };
        let bytes = unsigned.to_signable_bytes().unwrap();
        let sig = ctx_b.signer.sign(&bytes).unwrap();
        let bid = MeshJobBid {
            signature: SignatureBytes(sig),
            ..unsigned
        };
        let bid_msg = ProtocolMessage::new(
            MessagePayload::MeshBidSubmission(bid),
            ctx_b.current_identity.clone(),
            None,
        );
        net_b.broadcast_message(bid_msg).await?;

        // Wait for bid on A
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_a.recv().await {
                    if let MessagePayload::MeshBidSubmission(_) = &message.payload {
                        break;
                    }
                }
            }
        })
        .await?;

        // Assign job to B
        let assign_msg = ProtocolMessage::new(
            MessagePayload::MeshJobAssignment(MeshJobAssignmentMessage {
                job_id: job.id.clone(),
                executor_did: ctx_b.current_identity.clone(),
            }),
            ctx_a.current_identity.clone(),
            None,
        );
        net_a.broadcast_message(assign_msg).await?;

        // Wait for assignment on B
        timeout(Duration::from_secs(5), async {
            loop {
                if let Some(message) = recv_b.recv().await {
                    if let MessagePayload::MeshJobAssignment(assign) = &message.payload {
                        if assign.job_id == job.id && assign.executor_did == ctx_b.current_identity
                        {
                            break;
                        }
                    }
                }
            }
        })
        .await?;

        // Execute job and broadcast receipt
        let (sk, pk) = generate_ed25519_keypair();
        let exec = SimpleExecutor::new(ctx_b.current_identity.clone(), sk);
        let receipt = exec.execute_job(&job).await?;
        assert!(receipt.verify_against_key(&pk).is_ok());
        let receipt_msg = ProtocolMessage::new(
            MessagePayload::MeshReceiptSubmission(receipt.clone()),
            ctx_b.current_identity.clone(),
            None,
        );
        net_b.broadcast_message(receipt_msg).await?;

        // Wait for completion via HTTP on A
        let mut done = false;
        for _ in 0..20 {
            let resp = client
                .get(format!("{}/mesh/jobs/{}", url_a, job_id))
                .send()
                .await?;
            if resp.status().is_success() {
                let v: serde_json::Value = resp.json().await?;
                if v["status"]["status"] == "completed" {
                    done = true;
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }

        assert!(done, "job did not complete");

        let rep_after = ctx_a
            .reputation_store
            .get_reputation(&ctx_b.current_identity);
        assert!(rep_after > rep_before, "reputation should increase");

        handle_a.abort();
        handle_b.abort();
        Ok(())
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn libp2p_disabled_stub() {
    println!("libp2p disabled; skipping icn-node end-to-end test");
}
