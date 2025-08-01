use icn_common::{Cid, Did};
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, ExecutionReceipt, SignatureBytes,
};
use icn_runtime::context::{HostAbiError, RuntimeContext, StubSigner};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn anchor_receipt_valid_signature() {
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = Did::from_str(&did_str).unwrap();
    let _signer = Arc::new(StubSigner::new_with_keys(sk.clone(), vk));
    let ctx = RuntimeContext::new_for_testing(did.clone(), Some(100)).unwrap();

    let job_id = Cid::new_v1_sha256(0x55, b"sig_job");
    ctx.job_states.insert(
        icn_mesh::JobId(job_id.clone()),
        icn_mesh::JobState::Assigned {
            executor: did.clone(),
        },
    );

    let receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res"),
        cpu_ms: 10,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    let signed = receipt.sign_with_key(&sk).unwrap();

    let cid = ctx.anchor_receipt(&signed).await.unwrap();
    assert_eq!(cid, signed.result_cid);
}

#[tokio::test]
async fn anchor_receipt_invalid_signature() {
    let (sk, vk) = generate_ed25519_keypair();
    let did_str = did_key_from_verifying_key(&vk);
    let did = Did::from_str(&did_str).unwrap();
    let _signer = Arc::new(StubSigner::new_with_keys(sk.clone(), vk));
    let ctx = RuntimeContext::new_for_testing(did.clone(), Some(100)).unwrap();

    let job_id = Cid::new_v1_sha256(0x55, b"sig_job_bad");
    ctx.job_states.insert(
        icn_mesh::JobId(job_id.clone()),
        icn_mesh::JobState::Assigned {
            executor: did.clone(),
        },
    );

    let receipt = ExecutionReceipt {
        job_id: job_id.clone(),
        executor_did: did.clone(),
        result_cid: Cid::new_v1_sha256(0x55, b"res_bad"),
        cpu_ms: 10,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };
    let mut signed = receipt.sign_with_key(&sk).unwrap();
    signed.sig.0[0] ^= 0xFF;

    let err = ctx.anchor_receipt(&signed).await.err().unwrap();
    assert!(matches!(err, HostAbiError::SignatureError(_)));
}
