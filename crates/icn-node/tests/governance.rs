use icn_api::governance_trait::{CastVoteRequest, ProposalInputType, SubmitProposalRequest};
use icn_common::Did;
use icn_governance::ProposalId;
use icn_node::{app_router_with_options, RuntimeMode};
use reqwest::Client;
use std::str::FromStr;
use tokio::task;

#[tokio::test]
async fn submit_and_vote_proposal() {
    let (router, ctx) = app_router_with_options(
        RuntimeMode::Testing, // runtime_mode
        None,                 // api_key
        None,                 // auth_token
        None,                 // rate_limit
        None,                 // mana_ledger_backend
        None,                 // mana_ledger_path
        None,                 // storage_backend
        None,                 // storage_path
        None,                 // governance_db_path
        None,                 // reputation_db_path
        None,                 // parameter_store_path
    )
    .await;
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let submit_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".to_string(),
        proposal: ProposalInputType::GenericText { text: "hi".into() },
        description: "test".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
    };
    let submit_resp = client
        .post(format!("http://{addr}/governance/submit"))
        .json(&submit_req)
        .send()
        .await
        .unwrap();
    assert_eq!(submit_resp.status(), reqwest::StatusCode::CREATED);
    let pid_str: String = submit_resp.json().await.unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        let pid = ProposalId(pid_str.clone());
        gov.open_voting(&pid).unwrap();
    }

    let vote_req = CastVoteRequest {
        voter_did: "did:example:bob".to_string(),
        proposal_id: pid_str,
        vote_option: "yes".to_string(),
    };
    let vote_resp = client
        .post(format!("http://{addr}/governance/vote"))
        .json(&vote_req)
        .send()
        .await
        .unwrap();
    assert_eq!(vote_resp.status(), reqwest::StatusCode::OK);

    server.abort();
}
