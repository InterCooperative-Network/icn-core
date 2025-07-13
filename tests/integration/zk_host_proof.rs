use icn_runtime::{host_generate_zk_proof, host_verify_zk_proof, RuntimeContext};
use serde_json::json;

#[tokio::test]
async fn zk_proof_roundtrip_via_host() {
    let ctx = RuntimeContext::new_with_stubs("did:example:test").unwrap();
    let req = json!({
        "issuer": "did:example:issuer",
        "holder": "did:example:holder"
    });
    let proof_json = host_generate_zk_proof(&ctx, &req.to_string())
        .await
        .expect("generate proof");
    let verified = host_verify_zk_proof(&ctx, &proof_json)
        .await
        .expect("verify proof");
    assert!(verified);
}
