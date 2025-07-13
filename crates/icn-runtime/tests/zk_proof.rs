use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_runtime::{
    context::{HostAbiError, RuntimeContext},
    host_generate_zk_proof, host_verify_zk_proof,
};
use std::str::FromStr;

#[tokio::test]
async fn generate_and_verify_dummy_proof() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zProof").unwrap();
    let issuer = Did::from_str("did:key:zIssuer").unwrap();
    let holder = Did::from_str("did:key:zHolder").unwrap();
    let schema = Cid::new_v1_sha256(0x55, b"schema");
    let req = serde_json::json!({
        "issuer": issuer.to_string(),
        "holder": holder.to_string(),
        "claim_type": "test",
        "schema": schema.to_string(),
        "backend": "dummy",
    });
    let proof_json = host_generate_zk_proof(&ctx, &req.to_string())
        .await
        .unwrap();
    let proof: ZkCredentialProof = serde_json::from_str(&proof_json).unwrap();
    assert_eq!(proof.backend, ZkProofType::Other("dummy".into()));
    let verified = host_verify_zk_proof(&ctx, &proof_json).await.unwrap();
    assert!(verified);
}

#[tokio::test]
async fn verify_invalid_proof_fails() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zProof2").unwrap();
    let proof = ZkCredentialProof {
        issuer: Did::from_str("did:key:zIss").unwrap(),
        holder: Did::from_str("did:key:zHold").unwrap(),
        claim_type: "test".into(),
        proof: vec![1, 2, 3],
        schema: Cid::new_v1_sha256(0x55, b"s"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };
    let json = serde_json::to_string(&proof).unwrap();
    assert!(host_verify_zk_proof(&ctx, &json).await.is_err());
}

#[tokio::test]
async fn generate_invalid_json() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zProof3").unwrap();
    let err = host_generate_zk_proof(&ctx, "not-json")
        .await
        .err()
        .unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
}

#[tokio::test]
async fn different_circuit_costs_charge_different_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zMana", 10).unwrap();

    let low_req = serde_json::json!({
        "issuer": "did:key:zIss",
        "holder": "did:key:zHold",
        "claim_type": "membership",
        "schema": "cid:low",
        "backend": "dummy"
    });
    host_generate_zk_proof(&ctx, &low_req.to_string())
        .await
        .unwrap();
    let after_low = ctx.get_mana(&ctx.current_identity).await.unwrap();

    let high_req = serde_json::json!({
        "issuer": "did:key:zIss",
        "holder": "did:key:zHold",
        "claim_type": "age_rep_membership",
        "schema": "cid:high",
        "backend": "dummy"
    });
    host_generate_zk_proof(&ctx, &high_req.to_string())
        .await
        .unwrap();
    let after_high = ctx.get_mana(&ctx.current_identity).await.unwrap();

    assert!(after_low > after_high);
}
