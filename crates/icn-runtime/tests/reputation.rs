use icn_common::Cid;
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
};
use icn_reputation::ReputationStore;
use icn_runtime::{
    context::{RuntimeContext, StubSigner},
    host_anchor_receipt, ReputationUpdater,
};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn anchor_receipt_updates_reputation() {
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = icn_common::Did::from_str(&did_str).unwrap();
    let signer = Arc::new(StubSigner::new_with_keys(sk.clone(), vk));

    let ctx = RuntimeContext::new_with_ledger_path(
        &did_str,
        std::path::PathBuf::from("./mana_ledger.sled"),
        signer.clone(),
    ).unwrap();
    
    let job_id = Cid::new_v1_sha256(0x55, b"rep_job");
    let result_cid = Cid::new_v1_sha256(0x55, b"res");

    let receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: ctx.current_identity.clone(),
        result_cid,
        cpu_ms: 2000,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };

    let mut msg = Vec::new();
    msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
    msg.extend_from_slice(did.to_string().as_bytes());
    msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
    msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
    msg.push(receipt.success as u8);
    let sig_bytes = ctx.signer.sign(&msg).expect("sign");
    let mut signed_receipt = receipt.clone();
    signed_receipt.sig = SignatureBytes(sig_bytes);

    let json = serde_json::to_string(&signed_receipt).unwrap();
    let updater = ReputationUpdater::new();

    host_anchor_receipt(&ctx, &json, &updater)
        .await
        .expect("anchor");

    assert_eq!(ctx.reputation_store.get_reputation(&did), 3);
}

#[test]
fn reputation_updater_increments_store() {
    let store = icn_reputation::InMemoryReputationStore::new();
    let updater = ReputationUpdater::new();
    let did = icn_common::Did::new("key", "tester");
    let receipt = ExecutionReceipt {
        job_id: Cid::new_v1_sha256(0x55, b"rep"),
        executor_did: did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 1000,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    updater.submit(&store, &receipt);
    assert_eq!(store.get_reputation(&did), 2);
    updater.submit(&store, &receipt);
    assert_eq!(store.get_reputation(&did), 4);
}
