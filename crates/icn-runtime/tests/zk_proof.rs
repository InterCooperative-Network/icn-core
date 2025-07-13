use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_runtime::{context::{RuntimeContext, HostAbiError}, host_generate_zk_proof, host_verify_zk_proof};
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
    let proof_json = host_generate_zk_proof(&ctx, &req.to_string()).await.unwrap();
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
}

#[tokio::test]
async fn generate_invalid_json() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zProof3").unwrap();
    let err = host_generate_zk_proof(&ctx, "not-json").await.err().unwrap();
    assert!(matches!(err, HostAbiError::InvalidParameters(_)));
}

#[tokio::test]
async fn verify_with_circuit_registry() {
    use icn_identity::zk::{register_circuit, CircuitEntry};
    use icn_zk::{prove, setup, AgeOver18Circuit};
    use rand_core::OsRng;
    use ark_serialize::CanonicalSerialize;

    let mut rng = OsRng;
    let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2020 };
    let pk = setup(circuit.clone(), &mut rng).expect("setup");
    let proof_obj = prove(&pk, circuit, &mut rng).expect("prove");
    let mut proof_bytes = Vec::new();
    proof_obj.serialize_compressed(&mut proof_bytes).unwrap();

    let mut vk_bytes = Vec::new();
    pk.vk.serialize_compressed(&mut vk_bytes).unwrap();
    let vk_cid = Cid::new_v1_sha256(0x55, &vk_bytes);

    register_circuit(
        "age_over_18",
        Some(&vk_cid.to_string()),
        CircuitEntry { verifying_key: vk_bytes, public_inputs: vec![2020] },
    );

    let proof = ZkCredentialProof {
        issuer: Did::from_str("did:key:zIssReg").unwrap(),
        holder: Did::from_str("did:key:zHoldReg").unwrap(),
        claim_type: "age_over_18".into(),
        proof: proof_bytes,
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: Some(vk_cid),
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };

    let ctx = RuntimeContext::new_with_stubs("did:key:zProofReg").unwrap();
    let json = serde_json::to_string(&proof).unwrap();
    let verified = host_verify_zk_proof(&ctx, &json).await.unwrap();
    assert!(verified);
}
