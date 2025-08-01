use icn_common::Did;
use icn_node::app_router_with_options;
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]

async fn ledger_persists_between_restarts() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");

    let (_router, ctx) = app_router_with_options(
        icn_node::RuntimeMode::Development,
        None,
        None,
        None,
        Some(icn_runtime::context::LedgerBackend::File),
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    let did = Did::from_str("did:example:alice").unwrap();
    ctx.mana_ledger.set_balance(&did, 42).expect("set balance");

    drop(_router);

    let (_router2, ctx2) = app_router_with_options(
        icn_node::RuntimeMode::Development,
        None,
        None,
        None,
        Some(icn_runtime::context::LedgerBackend::File),
        Some(ledger_path.clone()),
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    assert_eq!(ctx2.mana_ledger.get_balance(&did), 42);
}
