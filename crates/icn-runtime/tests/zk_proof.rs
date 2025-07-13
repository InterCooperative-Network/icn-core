use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_runtime::{
    context::{mesh_network::ZK_VERIFY_COST_MANA, HostAbiError, RuntimeContext},
    host_generate_zk_proof, host_verify_zk_proof,
};
use std::str::FromStr;

#[tokio::test]
async fn generate_and_verify_dummy_proof() {
    let ctx =
        RuntimeContext::new_with_stubs_and_mana("did:key:zProof", ZK_VERIFY_COST_MANA * 2).unwrap();
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
async fn verify_updates_reputation() {
    let ctx =
        RuntimeContext::new_with_stubs_and_mana("did:key:zRep", ZK_VERIFY_COST_MANA * 2).unwrap();
    let issuer = Did::from_str("did:key:zIssuerRep").unwrap();
    let holder = Did::from_str("did:key:zHolderRep").unwrap();
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
    assert_eq!(ctx.reputation_store.get_reputation(&issuer), 0);
    host_verify_zk_proof(&ctx, &proof_json).await.unwrap();
    assert_eq!(ctx.reputation_store.get_reputation(&issuer), 1);
}

#[tokio::test]
async fn verify_invalid_proof_penalizes() {
    let ctx =
        RuntimeContext::new_with_stubs_and_mana("did:key:zBad", ZK_VERIFY_COST_MANA * 2).unwrap();
    let issuer = Did::from_str("did:key:zBadIss").unwrap();
    // Give issuer some reputation
    ctx.reputation_store.record_execution(&issuer, true, 0);
    ctx.reputation_store.record_execution(&issuer, true, 0);
    assert_eq!(ctx.reputation_store.get_reputation(&issuer), 2);
    let proof = ZkCredentialProof {
        issuer: issuer.clone(),
        holder: Did::from_str("did:key:zBadHolder").unwrap(),
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
    assert_eq!(ctx.reputation_store.get_reputation(&issuer), 1);
}

#[tokio::test]
async fn verify_invalid_proof_fails() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zProof2", ZK_VERIFY_COST_MANA * 2)
        .unwrap();
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
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zProof3", ZK_VERIFY_COST_MANA * 2)
        .unwrap();
    let err = host_generate_zk_proof(&ctx, "not-json")
        .await
        .err()
        .unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
}

#[tokio::test]
async fn verify_invalid_proof_refunds_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zRefund1", ZK_VERIFY_COST_MANA * 2)
        .unwrap();
    let initial = ctx.get_mana(&ctx.current_identity).await.unwrap();
    let proof = ZkCredentialProof {
        issuer: Did::from_str("did:key:zIss2").unwrap(),
        holder: Did::from_str("did:key:zHold2").unwrap(),
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
    let final_balance = ctx.get_mana(&ctx.current_identity).await.unwrap();
    assert_eq!(initial, final_balance);
}

#[tokio::test]
async fn malformed_proof_refunds_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:key:zRefund2", ZK_VERIFY_COST_MANA * 2)
        .unwrap();
    let initial = ctx.get_mana(&ctx.current_identity).await.unwrap();
    let err = host_verify_zk_proof(&ctx, "{not json").await.err().unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
    let final_balance = ctx.get_mana(&ctx.current_identity).await.unwrap();
    assert_eq!(initial, final_balance);
}
