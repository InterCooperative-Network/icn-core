use icn_common::{Cid, Did, FixedTimeProvider, TimeProvider};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_runtime::{
    context::{RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner},
    host_anchor_receipt, ReputationUpdater,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[tokio::test]
async fn anchor_receipt_uses_time_provider() {
    let provider = Arc::new(FixedTimeProvider::new(42));
    let did = Did::from_str("did:icn:test:time").unwrap();
    let ctx = RuntimeContext::new_with_ledger_path_and_time(
        did.clone(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new().unwrap()),
        Arc::new(icn_identity::KeyDidResolver),
        Arc::new(TokioMutex::new(StubDagStore::new())),
        std::path::PathBuf::from("./mana_ledger.sled"),
        std::path::PathBuf::from("./reputation.sled"),
        None,
        provider.clone(),
    );

    let receipt = ExecutionReceipt {
        job_id: Cid::new_v1_sha256(0x55, b"job"),
        executor_did: did,
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 1,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let cid = host_anchor_receipt(&ctx, &json, &ReputationUpdater::new())
        .await
        .unwrap();
    let store = ctx.dag_store.lock().await;
    let block = store.get(&cid).unwrap().unwrap();
    assert_eq!(block.timestamp, provider.unix_seconds());
}
