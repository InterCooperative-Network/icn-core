use assert_cmd::prelude::*;
use icn_common::{Cid, ZkCredentialProof, ZkProofType};
use std::process::Command;

#[test]
fn identity_generate_produces_valid_json() {
    let cid = Cid::new_v1_sha256(0x71, b"schema");
    let cid_str = cid.to_string();
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let output = Command::new(bin)
        .args([
            "identity",
            "generate-proof",
            "--issuer",
            "did:key:issuer",
            "--holder",
            "did:key:holder",
            "--claim-type",
            "test",
            "--schema",
            &cid_str,
            "--backend",
            "groth16",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let proof: ZkCredentialProof = serde_json::from_str(&stdout).unwrap();
    assert_eq!(proof.issuer.method, "key");
    assert_eq!(proof.holder.method, "key");
    assert_eq!(proof.claim_type, "test");
    assert_eq!(proof.schema, cid);
    assert_eq!(proof.backend, ZkProofType::Groth16);
    assert!(!proof.proof.is_empty());
}
