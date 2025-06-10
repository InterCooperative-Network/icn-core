use icn_node::app_router;
use tokio::task;
use reqwest::Client;
use icn_api::governance_trait::{SubmitProposalRequest, ProposalInputType, CastVoteRequest};
use serde_json::Value;

#[tokio::test]
async fn submit_and_vote_proposal() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let client = Client::new();
    let submit_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".to_string(),
        proposal: ProposalInputType::GenericText { text: "hi".into() },
        description: "test".into(),
        duration_secs: 60,
    };
    let resp: Value = client
        .post(&format!("http://{}/governance/submit", addr))
        .json(&submit_req)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let pid = resp["0"].as_str().unwrap_or_else(|| resp["id"].as_str().unwrap()).to_string();

    let vote_req = CastVoteRequest {
        voter_did: "did:example:bob".to_string(),
        proposal_id: pid.clone(),
        vote_option: "yes".to_string(),
    };
    let vote_resp = client
        .post(&format!("http://{}/governance/vote", addr))
        .json(&vote_req)
        .send()
        .await
        .unwrap();
    assert_eq!(vote_resp.status(), 200);

    let proposal: Value = client
        .get(&format!("http://{}/governance/proposal/{}", addr, pid))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(proposal["votes"].as_object().unwrap().len(), 1);

    server.abort();
}
