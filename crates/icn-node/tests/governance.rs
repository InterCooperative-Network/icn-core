use icn_api::governance_trait::{CastVoteRequest, ProposalInputType, SubmitProposalRequest};
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
#[ignore]
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
    let submit_resp = client
        .post(format!("http://{addr}/governance/submit"))
        .json(&submit_req)
        .send()
        .await
        .unwrap();
    assert_eq!(submit_resp.status(), reqwest::StatusCode::CREATED);
    let pid: String = submit_resp.json().await.unwrap();

    let vote_req = CastVoteRequest {
        voter_did: "did:example:bob".to_string(),
        proposal_id: pid,
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
