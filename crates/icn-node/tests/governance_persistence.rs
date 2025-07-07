#[cfg(feature = "persist-sled")]
use icn_common::Did;
#[cfg(feature = "persist-sled")]
use icn_governance::{ProposalId, ProposalType, VoteOption};
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
        None, // api_key
        None, // auth_token
        None, // rate_limit
        None, // mana ledger backend
        Some(ledger_path.clone()),
        None, // storage backend
        None, // storage path
        Some(gov_path.clone()),
        None,
    )
    .await;

    let pid = {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        let pid = gov
            .submit_proposal(
                Did::from_str("did:example:alice").unwrap(),
                ProposalType::GenericText("hello".into()),
                "desc".into(),
                60,
                None,
                None,
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
        )
        .unwrap();
    }

    drop(_router);
    drop(ctx);

    let (_router2, ctx2) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        None,
        None,
        Some(gov_path.clone()),
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
