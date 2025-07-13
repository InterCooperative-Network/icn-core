use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_runtime::{
    context::{
        HostAbiError, RuntimeContext, ZK_GENERATE_COST_MANA, ZK_VERIFY_COST_MANA,
    },
    host_generate_zk_proof, host_verify_zk_proof,
};
use std::str::FromStr;

#[tokio::test]
async fn generate_and_verify_dummy_proof() {
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:key:zProof",
        ZK_GENERATE_COST_MANA + ZK_VERIFY_COST_MANA + 1,
    )
    .unwrap();
    let initial = ctx.get_mana(&ctx.current_identity).await.unwrap();
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
    assert_eq!(
        ctx.get_mana(&ctx.current_identity).await.unwrap(),
        initial - ZK_GENERATE_COST_MANA
    );
    let proof: ZkCredentialProof = serde_json::from_str(&proof_json).unwrap();
    assert_eq!(proof.backend, ZkProofType::Other("dummy".into()));
    let verified = host_verify_zk_proof(&ctx, &proof_json).await.unwrap();
    assert!(verified);
    assert_eq!(
        ctx.get_mana(&ctx.current_identity).await.unwrap(),
        initial - ZK_GENERATE_COST_MANA - ZK_VERIFY_COST_MANA
    );
}

#[tokio::test]
async fn verify_invalid_proof_refunds_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:key:zProof2",
        ZK_VERIFY_COST_MANA,
    )
    .unwrap();
    let start = ctx.get_mana(&ctx.current_identity).await.unwrap();
    let proof = ZkCredentialProof {
        issuer: Did::from_str("did:key:zIss").unwrap(),
        holder: Did::from_str("did:key:zHold").unwrap(),
        claim_type: "test".into(),
        proof: vec![1,2,3],
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
    let end = ctx.get_mana(&ctx.current_identity).await.unwrap();
    assert_eq!(start, end);
}

#[tokio::test]
async fn generate_invalid_json() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zProof3").unwrap();
    let err = host_generate_zk_proof(&ctx, "not-json").await.err().unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
}

#[tokio::test]
async fn generate_proof_spends_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:key:zProof4",
        ZK_GENERATE_COST_MANA + 1,
    )
    .unwrap();

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

    let start = ctx.get_mana(&ctx.current_identity).await.unwrap();
    let _proof = host_generate_zk_proof(&ctx, &req.to_string())
        .await
        .unwrap();
    let end = ctx.get_mana(&ctx.current_identity).await.unwrap();
    assert_eq!(start - ZK_GENERATE_COST_MANA, end);
}
