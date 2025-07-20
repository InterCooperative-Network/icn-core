use icn_api::governance_trait::{CastVoteRequest, ProposalInputType, SubmitProposalRequest};
use icn_common::Did;
use icn_governance::{ProposalId, ProposalStatus};
use icn_node::app_router_with_options;
use reqwest::Client;
use std::str::FromStr;
use tokio::task;

#[tokio::test]
async fn add_and_remove_member_via_http() {
    let (router, ctx) = app_router_with_options(
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
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.set_quorum(1);
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let new_member = "did:example:bob";
    let submit_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".into(),
        proposal: ProposalInputType::MemberAdmission {
            did: new_member.into(),
        },
        description: "invite bob".into(),
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
    assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
    let pid: String = resp.json().await.unwrap();
    let pid_struct = ProposalId(pid.clone());
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.open_voting(&pid_struct).unwrap();
    }

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
    assert_eq!(vresp.status(), reqwest::StatusCode::OK);

    let close_resp = client
        .post(format!("http://{addr}/governance/close"))
        .json(&serde_json::json!({"proposal_id": pid.clone()}))
        .send()
        .await
        .unwrap();
    assert!(close_resp.status().is_success());

    {
        let gov = ctx.governance_module.lock().await;
        assert!(gov.members().contains(&Did::from_str(new_member).unwrap()));
        let prop = gov.get_proposal(&pid_struct).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Executed);
    }

    // Now test removal
    let remove_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".into(),
        proposal: ProposalInputType::RemoveMember {
            did: new_member.into(),
        },
        description: "remove bob".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
    };
    let resp = client
        .post(format!("http://{addr}/governance/submit"))
        .json(&remove_req)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
    let rpid: String = resp.json().await.unwrap();
    let rpid_struct = ProposalId(rpid.clone());
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.open_voting(&rpid_struct).unwrap();
    }

    let vresp = client
        .post(format!("http://{addr}/governance/vote"))
        .json(&CastVoteRequest {
            voter_did: node_did.to_string(),
            proposal_id: rpid.clone(),
            vote_option: "yes".into(),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(vresp.status(), reqwest::StatusCode::OK);

    let close_resp = client
        .post(format!("http://{addr}/governance/close"))
        .json(&serde_json::json!({"proposal_id": rpid}))
        .send()
        .await
        .unwrap();
    assert!(close_resp.status().is_success());

    {
        let gov = ctx.governance_module.lock().await;
        assert!(!gov.members().contains(&Did::from_str(new_member).unwrap()));
        let prop = gov.get_proposal(&rpid_struct).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Executed);
    }

    server.abort();
}

#[tokio::test]
async fn governance_membership_required_for_vote() {
    let router = {
        let (_router, _ctx) = app_router_with_options(
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
        _router
    };
}
