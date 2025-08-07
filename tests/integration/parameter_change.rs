use icn_common::Did;
use icn_governance::{ProposalSubmission, ProposalType, VoteOption};
use icn_node::{app_router_with_options, RuntimeMode};
use reqwest::Client;
use std::str::FromStr;
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn system_parameter_change_updates_rate_limit() {
    let (router, ctx) = app_router_with_options(
        RuntimeMode::Testing,
        None,
        None,
        Some(2),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });

    let pid = {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: Did::from_str("did:example:alice").unwrap(),
                    proposal_type: ProposalType::SystemParameterChange(
                        "open_rate_limit".into(),
                        "5".into(),
                    ),
                    description: "increase limit".into(),
                    duration_secs: 60,
                    timelock_delay: None,
                    quorum: None,
                    threshold: None,
                    content_cid: None,
                },
                &*ctx.time_provider,
            )
            .unwrap();
        gov.open_voting(&pid).unwrap();
        pid
    };
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.cast_vote(
            Did::from_str("did:example:bob").unwrap(),
            &pid,
            VoteOption::Yes,
            &*ctx.time_provider,
        )
        .unwrap();
        gov.close_voting_period(&pid, &*ctx.time_provider).unwrap();
        gov.execute_proposal(&pid).unwrap();
    }

    sleep(Duration::from_millis(100)).await;

    let client = Client::new();
    let url = format!("http://{addr}/info");

    for _ in 0..3 {
        let resp = client.get(&url).send().await.unwrap();
        assert!(resp.status().is_success());
    }

    server.abort();
}
