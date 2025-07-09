#[cfg(feature = "libp2p")]
mod job_announcement_bid {
    use icn_common::{Cid, Did};
    use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};
    use icn_network::NetworkService;
    use icn_protocol::{
        MeshBidSubmissionMessage, MeshJobAnnouncementMessage, MessagePayload, ProtocolMessage,
    };
    use std::str::FromStr;
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test]
    async fn announce_job_and_receive_bid() {
        // Node A with default config
        let node_a = Libp2pNetworkService::new(NetworkConfig::default())
            .await
            .expect("node a");
        sleep(Duration::from_secs(1)).await;
        let addr = node_a.listening_addresses()[0].clone();
        let peer_a = *node_a.local_peer_id();

        // Node B bootstraps to node A
        let mut config_b = NetworkConfig::default();
        config_b.bootstrap_peers = vec![(peer_a, addr)];
        let node_b = Libp2pNetworkService::new(config_b).await.expect("node b");

        // allow discovery
        sleep(Duration::from_secs(3)).await;

        // subscribe both nodes
        let mut sub_a = node_a.subscribe().await.expect("sub a");
        let mut sub_b = node_b.subscribe().await.expect("sub b");

        // build job announcement from node A
        let job_id = Cid::new_v1_sha256(0x55, b"job1");
        let job_msg = ProtocolMessage::new(
            MessagePayload::MeshJobAnnouncement(MeshJobAnnouncementMessage {
                job_id: job_id.clone(),
                manifest_cid: Cid::new_v1_sha256(0x71, b"manifest"),
                creator_did: Did::new("key", "node_a"),
                max_cost_mana: 10,
                job_spec: icn_protocol::JobSpec {
                    kind: icn_protocol::JobKind::Echo {
                        payload: "hello".into(),
                    },
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    required_resources: icn_protocol::ResourceRequirements::default(),
                },
                bid_deadline: 0,
            }),
            Did::new("key", "node_a"),
            None,
        );
        node_a
            .broadcast_message(job_msg)
            .await
            .expect("broadcast job");

        // Node B should receive announcement
        let announcement = timeout(Duration::from_secs(10), sub_b.recv())
            .await
            .expect("announcement timeout")
            .expect("announcement recv");
        let received_job = match announcement.payload {
            MessagePayload::MeshJobAnnouncement(j) => j,
            other => panic!("unexpected payload: {:?}", other),
        };
        assert_eq!(received_job.job_id, job_id);

        // Node B submits bid
        let bid_msg = ProtocolMessage::new(
            MessagePayload::MeshBidSubmission(MeshBidSubmissionMessage {
                job_id: job_id.clone(),
                executor_did: Did::from_str("did:key:executor_b").unwrap(),
                cost_mana: 5,
                estimated_duration_secs: 0,
                offered_resources: icn_protocol::ResourceRequirements::default(),
                reputation_score: 0,
            }),
            Did::new("key", "node_b"),
            None,
        );
        node_b
            .broadcast_message(bid_msg)
            .await
            .expect("broadcast bid");

        // Node A should receive bid
        let bid_recv = timeout(Duration::from_secs(10), sub_a.recv())
            .await
            .expect("bid timeout")
            .expect("bid recv");
        match bid_recv.payload {
            MessagePayload::MeshBidSubmission(b) => {
                assert_eq!(b.job_id, job_id);
            }
            other => panic!("unexpected payload: {:?}", other),
        }
    }
}
