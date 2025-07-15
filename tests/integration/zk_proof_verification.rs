use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_node::app_router_with_options;
use reqwest::Client;
use icn_identity::{
    generate_ed25519_keypair, sign_message, verify_signature, SignatureBytes,
};
use icn_zk::{prepare_vk, setup, prove, AgeOver18Circuit};
use rand_core::OsRng;
use ark_serialize::CanonicalSerialize;
use tokio::task;
use tokio::time::{sleep, Duration};

struct Groth16KeyManager {
    pk: ark_groth16::ProvingKey<ark_bn254::Bn254>,
    vk_bytes: Vec<u8>,
    vk_sig: SignatureBytes,
    signer_pk: icn_identity::VerifyingKey,
}

impl Groth16KeyManager {
    fn new() -> Self {
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = OsRng;
        let pk = setup(circuit, &mut rng).expect("setup");
        let (sk, signer_pk) = generate_ed25519_keypair();
        let mut vk_bytes = Vec::new();
        pk.vk
            .serialize_compressed(&mut vk_bytes)
            .expect("serialize");
        let sig = sign_message(&sk, &vk_bytes);
        let vk_sig = SignatureBytes::from_ed_signature(sig);
        Self {
            pk,
            vk_bytes,
            vk_sig,
            signer_pk,
        }
    }

    fn verify_vk_signature(&self) -> bool {
        let sig = self.vk_sig.to_ed_signature().unwrap();
        verify_signature(&self.signer_pk, &self.vk_bytes, &sig)
    }
}

#[tokio::test]
async fn zk_proof_verification_route() {
    std::fs::write("fixtures/mana_ledger.tmp", "{\"balances\":{}}").unwrap();
    let manager = Groth16KeyManager::new();
    assert!(manager.verify_vk_signature());
    let _prepared_vk = prepare_vk(&manager.pk);
    let (router, _ctx) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(std::path::PathBuf::from("fixtures/mana_ledger.tmp")),
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(100)).await;
    let client = Client::new();
    let url = format!("http://{}/identity/verify", addr);

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

    let resp = client.post(&url).json(&invalid_proof).send().await.unwrap();
    // Invalid proof should result in a 400 Bad Request
    assert_eq!(resp.status().as_u16(), 400);

    // Test with bulletproofs backend (should also fail with invalid data)
    let bulletproof_invalid = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".to_string(),
        proof: vec![1, 2, 3], // Invalid proof data
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Bulletproofs,
        verification_key: None,
        public_inputs: None,
    };

    let resp = client.post(&url).json(&bulletproof_invalid).send().await.unwrap();
    // Invalid proof should result in a 400 Bad Request
    assert_eq!(resp.status().as_u16(), 400);

    // Now test a valid Groth16 proof with explicit public inputs

    let circuit = AgeOver18Circuit { birth_year: 2000, current_year: 2020 };
    let mut rng = OsRng;
    let proof_obj = prove(&manager.pk, circuit, &mut rng).unwrap();
    let mut proof_bytes = Vec::new();
    proof_obj.serialize_compressed(&mut proof_bytes).unwrap();

    let valid_proof = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "age_over_18".to_string(),
        proof: proof_bytes,
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: Some(manager.vk_bytes.clone()),
        public_inputs: Some(serde_json::json!([2020])),
    };

    let resp = client.post(&url).json(&valid_proof).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    server.abort();
}
