#[cfg(feature = "enable-libp2p")]
mod ten_node_propagation {
    use anyhow::Result;
    use icn_common::{Cid, Did};
    use icn_governance::ProposalId;
    use icn_identity::SignatureBytes;
    use icn_mesh::{ActualMeshJob, JobKind, JobSpec, Resources};
    use icn_network::NetworkService;
    use icn_runtime::context::RuntimeContext;
    use icn_runtime::{host_create_governance_proposal, DefaultMeshNetworkService};
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration};

    async fn create_ctx(
        suffix: &str,
        bootstrap: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
    ) -> Result<Arc<RuntimeContext>> {
        let did = format!("did:key:z6MkTenNode{}", suffix);
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let path = format!("./dag_store_{}", suffix);
        let ctx = RuntimeContext::new_with_real_libp2p(
            &did,
            listen,
            bootstrap,
            path.clone().into(),
            format!("./mana_{}.sled", suffix).into(),
            format!("./rep_{}.sled", suffix).into(),
        )
        .await?;
        let did_parsed = Did::from_str(&did)?;
        ctx.mana_ledger
            .set_balance(&did_parsed, 1000)
            .expect("init mana");
        Ok(ctx)
    }

    #[tokio::test]
    async fn job_announcement_reaches_all_nodes() -> Result<()> {
        let node_a = create_ctx("A", None).await?;
        let a_net = node_a.get_libp2p_service()?;
        let a_peer = *a_net.local_peer_id();
        let mut addrs = Vec::new();
        for _ in 0..10 {
            sleep(Duration::from_millis(500)).await;
            addrs = a_net.listening_addresses();
            if !addrs.is_empty() {
                break;
            }
        }
        assert!(!addrs.is_empty());
        let bootstrap = vec![(a_peer, addrs[0].clone())];

        let mut receivers = Vec::new();
        let mut others = Vec::new();
        for s in ["B", "C", "D", "E", "F", "G", "H", "I", "J"] {
            let ctx = create_ctx(s, Some(bootstrap.clone())).await?;
            let net = ctx.get_libp2p_service()?;
            let rx = net.subscribe().await?;
            others.push(ctx);
            receivers.push(rx);
        }
        sleep(Duration::from_secs(2)).await;

        let submitter_did = Did::from_str("did:key:z6MkTenNodeA")?;
        let job = ActualMeshJob {
            id: Cid::new_v1_sha256(0x55, b"ten_node_job"),
            manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
            spec: JobSpec {
                kind: JobKind::Echo {
                    payload: "hello".into(),
                },
                ..Default::default()
            },
            creator_did: submitter_did.clone(),
            cost_mana: 10,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]),
        };
        let network = DefaultMeshNetworkService::new(a_net.clone());
        network.announce_job(&job).await?;

        for mut rx in receivers {
            let received = timeout(Duration::from_secs(10), async {
                loop {
                    if let Some(msg) = rx.recv().await {
                        if let icn_protocol::MessagePayload::MeshJobAnnouncement(ann) = &msg.payload
                        {
                            if ann.id == job.id {
                                break;
                            }
                        }
                    }
                }
            })
            .await;
            assert!(received.is_ok(), "job announcement missing");
        }
        Ok(())
    }

    #[tokio::test]
    async fn governance_propagates_to_all_nodes() -> Result<()> {
        let node_a = create_ctx("A1", None).await?;
        let a_net = node_a.get_libp2p_service()?;
        let a_peer = *a_net.local_peer_id();
        let mut addrs = Vec::new();
        for _ in 0..10 {
            sleep(Duration::from_millis(500)).await;
            addrs = a_net.listening_addresses();
            if !addrs.is_empty() {
                break;
            }
        }
        assert!(!addrs.is_empty());
        let bootstrap = vec![(a_peer, addrs[0].clone())];

        let mut receivers = Vec::new();
        let mut others = Vec::new();
        for s in ["B1", "C1", "D1", "E1", "F1", "G1", "H1", "I1", "J1"] {
            let ctx = create_ctx(s, Some(bootstrap.clone())).await?;
            let net = ctx.get_libp2p_service()?;
            let rx = net.subscribe().await?;
            others.push(ctx);
            receivers.push(rx);
        }
        sleep(Duration::from_secs(2)).await;

        let payload = serde_json::json!({
            "proposal_type_str": "GenericText",
            "type_specific_payload": b"hi".to_vec(),
            "description": "ten node",
            "duration_secs": 60
        });
        let pid = host_create_governance_proposal(&node_a, &payload.to_string()).await?;
        let pid = ProposalId(pid);

        for (ctx, mut rx) in others.into_iter().zip(receivers.into_iter()) {
            let bytes = timeout(Duration::from_secs(10), async {
                loop {
                    if let Some(msg) = rx.recv().await {
                        if let icn_protocol::MessagePayload::GovernanceProposalAnnouncement(b) =
                            &msg.payload
                        {
                            break b.clone();
                        }
                    }
                }
            })
            .await?;
            ctx.ingest_external_proposal(&bytes).await?;
            let gov = ctx.governance_module.lock().await;
            assert!(gov.get_proposal(&pid)?.is_some());
        }
        Ok(())
    }
}
