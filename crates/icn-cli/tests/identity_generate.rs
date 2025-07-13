use assert_cmd::prelude::*;
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use serde::Deserialize;
use std::process::Command;

#[tokio::test]
#[serial_test::serial]
async fn identity_generate_command() {
    let issuer = Did::new("key", "issuer");
    let holder = Did::new("key", "holder");
    let schema_cid = Cid::new_v1_sha256(0x55, b"schema");
    let issuer_str = issuer.to_string();
    let holder_str = holder.to_string();
    let schema_str = schema_cid.to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "identity",
                "generate-proof",
                "--issuer",
                &issuer_str,
                "--holder",
                &holder_str,
                "--claim-type",
                "test",
                "--schema",
                &schema_str,
                "--backend",
                "groth16",
                "--age-over-18",
                "2024",
            ])
            .output()
            .unwrap()
    })
    .await
    .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    #[derive(Deserialize)]
    struct Output {
        proof: ZkCredentialProof,
        verifying_key: String,
    }
    let out: Output = serde_json::from_str(&stdout).unwrap();
    assert_eq!(out.proof.issuer, issuer);
    assert_eq!(out.proof.holder, holder);
    assert_eq!(out.proof.claim_type, "test");
    assert_eq!(out.proof.schema, schema_cid);
    assert_eq!(out.proof.backend, ZkProofType::Groth16);
    assert!(!out.proof.proof.is_empty());
    assert!(!out.verifying_key.is_empty());
}
