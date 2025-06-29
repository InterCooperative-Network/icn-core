#[cfg(feature = "persist-sled")]
use icn_common::{compute_merkle_cid, DagBlock, Did};
use icn_node::{app_router_with_options, config::StorageBackendType};
use std::str::FromStr;
use tempfile::tempdir;

#[cfg(feature = "persist-sled")]
#[tokio::test]
async fn dag_persists_between_restarts_sled() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("mana.sled");
    let dag_path = dir.path().join("dag_db");

    let (_router, ctx) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        Some(StorageBackendType::Sled),
        Some(dag_path.clone()),
        None,
        None,
    )
    .await;

    let data = b"hello".to_vec();
    let ts = 0u64;
    let author = Did::from_str("did:example:tester").unwrap();
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: data.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }

    drop(_router);

    let (_router2, ctx2) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path.clone()),
        Some(StorageBackendType::Sled),
        Some(dag_path.clone()),
        None,
        None,
    )
    .await;

    let stored = ctx2.dag_store.lock().await.get(&cid).unwrap();
    assert!(stored.is_some());
}
