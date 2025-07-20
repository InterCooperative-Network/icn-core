use icn_api::governance_trait::{
    CastVoteRequest, DelegateRequest, ProposalInputType, RevokeDelegationRequest,
    SubmitProposalRequest,
};
use icn_common::Did;
use icn_governance::{ProposalId, ProposalStatus, VoteOption};
use icn_node::{app_router_with_options, RuntimeMode};
use reqwest::Client;
use std::str::FromStr;
use tokio::task;

#[tokio::test]
async fn delegate_and_revoke_flow() {
    let (router, ctx) = app_router_with_options(
        RuntimeMode::Testing, // runtime_mode
        None, // api_key
        None, // auth_token
        None, // rate_limit
        None, // mana_ledger_backend
        None, // mana_ledger_path
        None, // storage_backend
        None, // storage_path
        None, // governance_db_path
        None, // reputation_db_path
        None, // parameter_store_path
    )
    .await;
    let node_did = ctx.current_identity.clone();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(node_did.clone());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        gov.add_member(Did::from_str("did:example:carol").unwrap());
        gov.set_quorum(2);
        gov.set_threshold(0.6);
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let submit_req = SubmitProposalRequest {
        proposer_did: node_did.to_string(),
        proposal: ProposalInputType::GenericText { text: "hi".into() },
        description: "test".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
    };
    let resp = client
        .post(format!("http://{addr}/governance/submit"))
        .json(&submit_req)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let pid: String = resp.json().await.unwrap();
    let pid_struct = ProposalId(pid.clone());
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.open_voting(&pid_struct).unwrap();
    }

    let dreq = DelegateRequest {
        from_did: "did:example:bob".into(),
        to_did: node_did.to_string(),
    };
    let dresp = client
        .post(format!("http://{addr}/governance/delegate"))
        .json(&dreq)
        .send()
        .await
        .unwrap();
    assert!(dresp.status().is_success());

    let vote_req = CastVoteRequest {
        voter_did: node_did.to_string(),
        proposal_id: pid.clone(),
        vote_option: "yes".into(),
    };
    let vresp = client
        .post(format!("http://{addr}/governance/vote"))
        .json(&vote_req)
        .send()
        .await
        .unwrap();
    assert!(vresp.status().is_success());

    {
        let mut gov = ctx.governance_module.lock().await;
        gov.cast_vote(
            Did::from_str("did:example:carol").unwrap(),
            &pid_struct,
            VoteOption::No,
        )
        .unwrap();
    }

    let close = client
        .post(format!("http://{addr}/governance/close"))
        .json(&serde_json::json!({"proposal_id": pid.clone()}))
        .send()
        .await
        .unwrap();
    assert!(close.status().is_success());
    {
        let gov = ctx.governance_module.lock().await;
        let prop = gov.get_proposal(&pid_struct).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Executed);
    }

    // revoke delegation for second proposal
    let submit_req2 = SubmitProposalRequest {
        proposer_did: node_did.to_string(),
        proposal: ProposalInputType::GenericText { text: "bye".into() },
        description: "test2".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
    };
    let resp2 = client
        .post(format!("http://{addr}/governance/submit"))
        .json(&submit_req2)
        .send()
        .await
        .unwrap();
    let pid2: String = resp2.json().await.unwrap();
    let pid2_struct = ProposalId(pid2.clone());
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.open_voting(&pid2_struct).unwrap();
    }
    let rreq = RevokeDelegationRequest {
        from_did: "did:example:bob".into(),
    };
    client
        .post(format!("http://{addr}/governance/revoke"))
        .json(&rreq)
        .send()
        .await
        .unwrap();
    let vote_req2 = CastVoteRequest {
        voter_did: node_did.to_string(),
        proposal_id: pid2.clone(),
        vote_option: "yes".into(),
    };
    client
        .post(format!("http://{addr}/governance/vote"))
        .json(&vote_req2)
        .send()
        .await
        .unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.cast_vote(
            Did::from_str("did:example:carol").unwrap(),
            &pid2_struct,
            VoteOption::No,
        )
        .unwrap();
    }
    let close2 = client
        .post(format!("http://{addr}/governance/close"))
        .json(&serde_json::json!({"proposal_id": pid2.clone()}))
        .send()
        .await
        .unwrap();
    assert!(close2.status().is_success());
    {
        let gov = ctx.governance_module.lock().await;
        let prop = gov.get_proposal(&pid2_struct).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Rejected);
    }

    server.abort();
}
