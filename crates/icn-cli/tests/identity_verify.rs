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

    let proof = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".to_string(),
        proof: vec![1, 2, 3],
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };
    let proof_json = serde_json::to_string(&proof).unwrap();

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
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("verified"));

    server.abort();
}
