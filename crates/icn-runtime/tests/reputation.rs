use icn_common::Cid;
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_runtime::{context::RuntimeContext, host_anchor_receipt, ReputationUpdater};

#[tokio::test]
async fn anchor_receipt_updates_reputation() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:rep", 0);
    let job_id = Cid::new_v1_dummy(0x55, 0x13, b"rep_job");
    let result_cid = Cid::new_v1_dummy(0x55, 0x14, b"res");

    let receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: ctx.current_identity.clone(),
        result_cid,
        cpu_ms: 1,
        sig: SignatureBytes(Vec::new()),
    };

    let mut msg = Vec::new();
    msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
    msg.extend_from_slice(ctx.current_identity.to_string().as_bytes());
    msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
    msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
    let sig_bytes = ctx.signer.sign(&msg).expect("sign");
    let mut signed_receipt = receipt.clone();
    signed_receipt.sig = SignatureBytes(sig_bytes);

    let json = serde_json::to_string(&signed_receipt).unwrap();
    let updater = ReputationUpdater::new();

    host_anchor_receipt(&ctx, &json, &updater)
        .await
        .expect("anchor");

    assert_eq!(
        ctx.reputation_store.get_reputation(&ctx.current_identity),
        1
    );
}
#[test]
fn reputation_updater_increments_store() {
    let store = icn_reputation::InMemoryReputationStore::new();
    let updater = ReputationUpdater::new();
    let did = icn_common::Did::new("key", "tester");
    let receipt = ExecutionReceipt {
        job_id: Cid::new_v1_dummy(0x55, 0x15, b"rep"),
        executor_did: did.clone(),
        result_cid: Cid::new_v1_dummy(0x55, 0x15, b"res"),
        cpu_ms: 1,
        sig: SignatureBytes(Vec::new()),
    };
    updater.submit(&store, &receipt);
    assert_eq!(store.get_reputation(&did), 1);
    updater.submit(&store, &receipt);
    assert_eq!(store.get_reputation(&did), 2);
}
