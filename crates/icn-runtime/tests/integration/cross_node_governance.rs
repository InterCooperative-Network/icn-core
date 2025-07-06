#[cfg(feature = "enable-libp2p")]
mod cross_node_governance {
    use icn_governance::Proposal;
    use icn_network::{NetworkMessage, NetworkService};
    use icn_runtime::context::RuntimeContext;
    use icn_runtime::{host_cast_governance_vote, host_create_governance_proposal};
    use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration};

    async fn create_ctx(
        id_suffix: &str,
        bootstrap: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
    ) -> anyhow::Result<Arc<RuntimeContext>> {
        let id = format!("did:key:z6Mkgov{id_suffix}");
        let listen: Vec<Multiaddr> = vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()];
        let ctx = RuntimeContext::new_with_real_libp2p(
            &id,
            listen,
            bootstrap,
            std::path::PathBuf::from("./mana_ledger.sled"),
            std::path::PathBuf::from("./reputation.sled"),
        )
        .await?;
        Ok(ctx)
    }

    #[tokio::test]
    async fn proposal_and_vote_propagate() -> anyhow::Result<()> {
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

        let node_b = create_ctx("B", Some(vec![(a_peer, addrs[0].clone())])).await?;
        let b_net = node_b.get_libp2p_service()?;
        sleep(Duration::from_secs(2)).await;

        let mut b_rx = b_net.subscribe().await?;
        let mut a_rx = a_net.subscribe().await?;

        let payload = serde_json::json!({
            "proposal_type_str": "GenericText",
            "type_specific_payload": b"hello".to_vec(),
            "description": "test",
            "duration_secs": 60
        });
        let pid_str = host_create_governance_proposal(&node_a, &payload.to_string()).await?;

        let proposal_bytes = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = b_rx.recv().await {
                    if let MessagePayload::GovernanceProposalAnnouncement(bytes) = &message.payload
                    {
                        break bytes.clone();
                    }
                }
            }
        })
        .await
        .expect("timeout waiting for proposal");
        node_b.ingest_external_proposal(&proposal_bytes).await?;
        let proposal: Proposal = bincode::deserialize(&proposal_bytes)?;
        {
            let gov = node_b.governance_module.lock().await;
            assert!(gov.get_proposal(&proposal.id)?.is_some());
        }

        let vote_payload = serde_json::json!({
            "proposal_id_str": proposal.id.0,
            "vote_option_str": "yes"
        });
        host_cast_governance_vote(&node_b, &vote_payload.to_string()).await?;

        let vote_bytes = timeout(Duration::from_secs(10), async {
            loop {
                if let Some(message) = a_rx.recv().await {
                    if let MessagePayload::GovernanceVoteAnnouncement(bytes) = &message.payload {
                        break bytes.clone();
                    }
                }
            }
        })
        .await
        .expect("timeout waiting for vote");
        node_a.ingest_external_vote(&vote_bytes).await?;
        {
            let gov = node_a.governance_module.lock().await;
            let pid = icn_governance::ProposalId(pid_str.clone());
            let prop = gov.get_proposal(&pid)?.unwrap();
            assert_eq!(prop.votes.len(), 1);
        }

        Ok(())
    }
}
