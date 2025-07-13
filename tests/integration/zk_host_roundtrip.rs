use icn_common::{Cid, Did};
use icn_runtime::{context::RuntimeContext, host_generate_zk_proof, host_verify_zk_proof};
use std::str::FromStr;

#[tokio::test]
async fn host_generate_then_verify() {
    let ctx = RuntimeContext::new_with_stubs("did:key:test").unwrap();
    let issuer = Did::from_str("did:key:issuer").unwrap();
    let holder = Did::from_str("did:key:holder").unwrap();
    let schema = Cid::new_v1_sha256(0x55, b"schema");
    let req = serde_json::json!({
        "issuer": issuer.to_string(),
        "holder": holder.to_string(),
        "claim_type": "test",
        "schema": schema.to_string(),
        "backend": "dummy"
    });
    let proof_json = host_generate_zk_proof(&ctx, &req.to_string()).await.unwrap();
    let verified = host_verify_zk_proof(&ctx, &proof_json).await.unwrap();
    assert!(verified);
}
