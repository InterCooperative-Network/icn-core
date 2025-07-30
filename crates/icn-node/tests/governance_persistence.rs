use icn_runtime::context::LedgerBackend;
#[cfg(feature = "persist-sled")]
use icn_common::Did;
#[cfg(feature = "persist-sled")]
use icn_governance::{ProposalSubmission, ProposalType, VoteOption};
#[cfg(feature = "persist-sled")]
use icn_node::app_router_with_options;
#[cfg(feature = "persist-sled")]
use std::str::FromStr;
#[cfg(feature = "persist-sled")]
use tempfile::tempdir;

#[cfg(feature = "persist-sled")]
#[tokio::test]
async fn governance_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let gov_path = dir.path().join("gov.sled");

    let (_router, ctx) = app_router_with_options(
        icn_node::RuntimeMode::Testing,
        None, // api_key
        None, // auth_token
        None, // rate_limit
        None, // mana ledger backend
        Some(LedgerBackend::Sled(ledger_path.clone())),
        None, // storage backend
        None, // storage path
        Some(gov_path.clone()),
        None,
        None,
    )
    .await;

    let pid = {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: Did::from_str("did:example:alice").unwrap(),
                    proposal_type: ProposalType::GenericText("hello".into()),
                    description: "desc".into(),
                    duration_secs: 60,
                    quorum: None,
                    threshold: None,
                    content_cid: None,
                },
                ctx.time_provider,
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
            ctx.time_provider,
        )
        .unwrap();
    }

    drop(_router);
    drop(ctx);

    let (_router2, ctx2) = app_router_with_options(
        icn_node::RuntimeMode::Testing,
        None,
        None,
        None,
        None,
        Some(LedgerBackend::Sled(ledger_path.clone())),
        None,
        None,
        Some(gov_path.clone()),
        None,
        None,
    )
    .await;
    let gov2 = ctx2.governance_module.lock().await;
    let prop = gov2.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.votes.len(), 1);
}

#[cfg(not(feature = "persist-sled"))]
#[tokio::test]
async fn governance_persists_between_restarts() {
    // Feature not enabled; nothing to test
}
