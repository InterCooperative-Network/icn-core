use icn_common::Did;
use icn_node::app_router_with_options;
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]

async fn reputation_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let rep_path = dir.path().join("rep.sled");

    let (_router, ctx) = app_router_with_options(
        icn_node::RuntimeMode::Test,
        None,
        None,
        None,
        None,
        Some(icn_node::config::LedgerBackend::Sled(ledger_path.clone())),
        None,
        None,
        Some(rep_path.clone()),
        None,
        None,
    )
    .await;
    let did = Did::from_str("did:example:alice").unwrap();
    ctx.reputation_store.record_execution(&did, true, 100);

    drop(_router);

    let (_router2, ctx2) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        None,
        None,
        Some(rep_path.clone()),
        None,
        None,
    )
    .await;
    assert!(ctx2.reputation_store.get_reputation(&did) > 0);
}
