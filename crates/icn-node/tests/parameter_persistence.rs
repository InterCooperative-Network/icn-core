use icn_common::{Did, FixedTimeProvider};
use icn_governance::{ProposalSubmission, ProposalType, VoteOption};
use icn_node::app_router_with_options;
use icn_node::parameter_store::ParameterStore;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn parameter_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let param_path = dir.path().join("params.toml");

    let (_router, ctx) = app_router_with_options(
        icn_node::RuntimeMode::Testing,
        None,
        None,
        None,
        Some(icn_runtime::context::LedgerBackend::File),
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        Some(param_path.clone()),
    )
    .await;

    let time_provider = Arc::new(FixedTimeProvider::new(1640995200)); // Jan 1, 2022

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
                    description: "desc".into(),
                    duration_secs: 60,
                    quorum: None,
                    threshold: None,
                    content_cid: None,
                },
                &*time_provider,
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
            &*time_provider,
        )
        .unwrap();
        gov.close_voting_period(&pid, &*time_provider).unwrap();
        gov.execute_proposal(&pid).unwrap();
    }

    drop(_router);
    drop(ctx);

    // Remove the file so event replay is required
    std::fs::remove_file(&param_path).unwrap();

    let store = ParameterStore::load(param_path.clone()).unwrap();
    assert_eq!(store.open_rate_limit(), 5);

    let (_r2, _ctx2) = app_router_with_options(
        icn_node::RuntimeMode::Testing,
        None,
        None,
        None,
        Some(icn_runtime::context::LedgerBackend::File),
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        Some(param_path.clone()),
    )
    .await;
    let ps = ParameterStore::load(param_path.clone()).unwrap();
    assert_eq!(ps.open_rate_limit(), 5);
}
