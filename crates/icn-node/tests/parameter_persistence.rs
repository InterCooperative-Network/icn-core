use icn_common::Did;
use icn_governance::{ProposalType, VoteOption};
use icn_node::app_router_with_options;
use icn_node::config::NodeConfig;
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]
async fn parameter_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let param_path = dir.path().join("params.toml");

    let (_router, ctx) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        Some(param_path.clone()),
    )
    .await;

    let pid = {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        let pid = gov
            .submit_proposal(
                Did::from_str("did:example:alice").unwrap(),
                ProposalType::SystemParameterChange("open_rate_limit".into(), "5".into()),
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
        gov.close_voting_period(&pid).unwrap();
        gov.execute_proposal(&pid).unwrap();
    }

    drop(_router);
    drop(ctx);

    let cfg = NodeConfig::from_file(&param_path).unwrap();
    assert_eq!(cfg.open_rate_limit, 5);

    let (_r2, _ctx2) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        Some(param_path.clone()),
    )
    .await;
    let cfg2 = NodeConfig::from_file(&param_path).unwrap();
    assert_eq!(cfg2.open_rate_limit, 5);
}
