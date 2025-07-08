use icn_runtime::context::{RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner};
use icn_common::Did;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[tokio::test]
async fn mana_persists_across_contexts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ledger_path = temp_dir.path().join("mana.sled");
    let did = Did::parse("did:example:alice").unwrap();

    let ctx1 = RuntimeContext::new_with_ledger_path(
        did.clone(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(TokioMutex::new(StubDagStore::new())),
        ledger_path.clone(),
        temp_dir.path().join("rep.sled"),
    );
    ctx1.mana_ledger.set_balance(&did, 42).unwrap();
    drop(ctx1);

    let ctx2 = RuntimeContext::new_with_ledger_path(
        did.clone(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(TokioMutex::new(StubDagStore::new())),
        ledger_path,
        temp_dir.path().join("rep.sled"),
    );
    assert_eq!(ctx2.mana_ledger.get_balance(&did), 42);
}
