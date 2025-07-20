use icn_common::{Cid, Did, FixedTimeProvider, TimeProvider};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_runtime::{
    context::{RuntimeContext, StubSigner},
    host_anchor_receipt, ReputationUpdater,
};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn anchor_receipt_uses_time_provider() {
    let provider = Arc::new(FixedTimeProvider::new(42));
    let did_str = "did:icn:test:time";
    let did = Did::from_str(did_str).unwrap();
    let signer = Arc::new(StubSigner::new());

    let ctx = RuntimeContext::new_with_ledger_path_and_time(
        did_str,
        std::path::PathBuf::from("./mana_ledger.sled"),
        provider.clone(),
        signer,
    )
    .unwrap();

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
    let block = store.get(&cid).await.unwrap().unwrap();
    assert_eq!(block.timestamp, provider.unix_seconds());
}
