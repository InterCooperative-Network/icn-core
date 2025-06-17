use icn_common::Did;
use icn_governance::{ProposalType, VoteOption};
use icn_node::app_router_with_options;
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]
async fn governance_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let gov_path = dir.path().join("gov.sled");

    let (_router, ctx) = app_router_with_options(
        None,
        None,
        Some(ledger_path.clone()),
        Some(gov_path.clone()),
    )
    .await;

    let pid = {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        gov.submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("hello".into()),
            "desc".into(),
            60,
        )
        .unwrap()
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

    let (_router2, ctx2) = app_router_with_options(
        None,
        None,
        Some(ledger_path.clone()),
        Some(gov_path.clone()),
    )
    .await;
    let gov2 = ctx2.governance_module.lock().await;
    let prop = gov2.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.votes.len(), 1);
}
