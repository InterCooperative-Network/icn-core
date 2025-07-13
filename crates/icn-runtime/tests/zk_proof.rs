use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_reputation::{InMemoryReputationStore, ReputationStore};
use icn_runtime::context::mana::SimpleManaLedger;
use icn_runtime::{
    context::{
        HostAbiError, RuntimeContext as Rc, ServiceConfigBuilder, ServiceEnvironment, StubSigner,
    },
    host_generate_zk_proof, host_verify_zk_proof,
};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn generate_and_verify_dummy_proof() {
    let ctx = Rc::new_with_stubs("did:key:zProof").unwrap();
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
    let ctx = Rc::new_with_stubs("did:key:zProof2").unwrap();
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
    let ctx = Rc::new_with_stubs("did:key:zProof3").unwrap();
    let err = host_generate_zk_proof(&ctx, "not-json")
        .await
        .err()
        .unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
}

#[tokio::test]
async fn reputation_updated_on_proof_result() {
    let store = Arc::new(InMemoryReputationStore::new());
    let temp = tempfile::NamedTempFile::new().unwrap();
    let ledger = SimpleManaLedger::new(temp.path().to_path_buf());
    let cfg = ServiceConfigBuilder::new(ServiceEnvironment::Testing)
        .with_identity(Did::from_str("did:key:verifier").unwrap())
        .with_signer(Arc::new(StubSigner::new()))
        .with_did_resolver(Arc::new(icn_identity::KeyDidResolver))
        .with_mana_ledger(ledger)
        .with_reputation_store(store.clone())
        .build()
        .unwrap();
    let ctx = Rc::from_service_config(cfg).unwrap();

    let issuer = Did::from_str("did:key:issuer").unwrap();
    let holder = Did::from_str("did:key:holder").unwrap();
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
    assert!(host_verify_zk_proof(&ctx, &proof_json).await.unwrap());
    assert_eq!(store.get_reputation(&holder), 1);

    store.set_score(holder.clone(), 1);
    let mut bad: ZkCredentialProof = serde_json::from_str(&proof_json).unwrap();
    bad.backend = ZkProofType::Groth16;
    let j = serde_json::to_string(&bad).unwrap();
    assert!(host_verify_zk_proof(&ctx, &j).await.is_err());
    assert_eq!(store.get_reputation(&holder), 0);
}
