use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_node::app_router_with_options;
use reqwest::Client;
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn zk_proof_verification_route() {
    std::fs::write("fixtures/mana_ledger.tmp", "{\"balances\":{}}").unwrap();
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

    server.abort();
}
