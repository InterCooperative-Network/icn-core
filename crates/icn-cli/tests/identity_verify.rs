use assert_cmd::prelude::*;
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn identity_verify_command() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Test with invalid proof data - should fail
    let invalid_proof = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".to_string(),
        proof: vec![1, 2, 3], // Invalid proof data
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };
    let proof_json = serde_json::to_string(&invalid_proof).unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let output = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "identity", "verify-proof", &proof_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    
    // Invalid proof should cause command to fail
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should contain error information about the failed verification
    assert!(stderr.contains("400") || stderr.contains("error") || stderr.contains("failed"));

    server.abort();
}
