use icn_common::{Cid, Did};
use icn_runtime::{context::RuntimeContext, host_generate_zk_proof, host_verify_zk_proof};

#[tokio::test]
async fn host_generate_then_verify_roundtrip() {
    let ctx = RuntimeContext::new_for_testing(&Did::new("key", "testZk"), Some(10)).unwrap();
    let issuer = Did::new("key", "issuer");
    let holder = Did::new("key", "holder");
    let schema = Cid::new_v1_sha256(0x55, b"schema");

    let request = serde_json::json!({
        "issuer": issuer.to_string(),
        "holder": holder.to_string(),
        "claim_type": "test",
        "schema": schema.to_string(),
        "backend": "dummy"
    });

    let proof_json = host_generate_zk_proof(&ctx, &request.to_string())
        .await
        .expect("generate proof");

    let verified = host_verify_zk_proof(&ctx, &proof_json)
        .await
        .expect("verify proof");

    assert!(verified, "generated proof should verify");
}
