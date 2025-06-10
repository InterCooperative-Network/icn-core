use icn_common::{Cid, Did};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, InMemoryDidResolver,
    SignatureBytes,
};
use icn_runtime::context::{RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn anchor_receipt_succeeds_with_correct_key() {
    let (sk_exec, vk_exec) = generate_ed25519_keypair();
    let exec_did_str = did_key_from_verifying_key(&vk_exec);
    let exec_did = Did::from_str(&exec_did_str).unwrap();

    let resolver = InMemoryDidResolver::new();
    resolver.register(exec_did.clone(), vk_exec);

    let ctx = RuntimeContext::new(
        Did::from_str("did:icn:test:manager").unwrap(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new()),
        Arc::new(StubDagStore::new()),
        Arc::new(resolver),
    );

    let job_id = Cid::new_v1_dummy(0x55, 0x13, b"job1");
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"res1");
    let mut receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: exec_did.clone(),
        result_cid,
        cpu_ms: 1,
        sig: SignatureBytes(Vec::new()),
    };
    receipt = receipt.sign_with_key(&sk_exec).unwrap();

    let cid = ctx.anchor_receipt(&receipt).await.unwrap();
    assert!(!cid.hash_bytes.is_empty());
}

#[tokio::test]
async fn anchor_receipt_fails_with_wrong_key() {
    let (_sk_exec, vk_exec) = generate_ed25519_keypair();
    let exec_did_str = did_key_from_verifying_key(&vk_exec);
    let exec_did = Did::from_str(&exec_did_str).unwrap();

    let resolver = InMemoryDidResolver::new();
    resolver.register(exec_did.clone(), vk_exec);

    let ctx = RuntimeContext::new(
        Did::from_str("did:icn:test:manager").unwrap(),
        Arc::new(StubMeshNetworkService::new()),
        Arc::new(StubSigner::new()),
        Arc::new(StubDagStore::new()),
        Arc::new(resolver),
    );

    let (sk_wrong, _vk_wrong) = generate_ed25519_keypair();

    let job_id = Cid::new_v1_dummy(0x55, 0x13, b"job2");
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"res2");
    let mut receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: exec_did.clone(),
        result_cid,
        cpu_ms: 1,
        sig: SignatureBytes(Vec::new()),
    };
    receipt = receipt.sign_with_key(&sk_wrong).unwrap();

    let err = ctx.anchor_receipt(&receipt).await.unwrap_err();
    match err {
        icn_runtime::context::HostAbiError::SignatureError(_) => (),
        other => panic!("Unexpected error: {other:?}"),
    }
}
