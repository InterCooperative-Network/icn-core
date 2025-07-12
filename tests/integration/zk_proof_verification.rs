use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_node::app_router_with_options;
use merlin::Transcript;
use reqwest::Client;
use tokio::task;
use tokio::time::{sleep, Duration};

fn make_bulletproof(value: u64) -> Vec<u8> {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, 1);
    let mut transcript = Transcript::new(b"icn-bulletproof");
    let (proof, _) = RangeProof::prove_single(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        value,
        &Scalar::ZERO,
        64,
    )
    .unwrap();
    proof.to_bytes()
}

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

    let proof = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".to_string(),
        proof: make_bulletproof(42),
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Bulletproofs,
    };

    let resp = client.post(url).json(&proof).send().await.unwrap();
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["verified"], true);

    server.abort();
}

#[tokio::test]
async fn zk_proof_verification_invalid() {
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

    let proof = ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".to_string(),
        proof: make_bulletproof(7),
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Bulletproofs,
    };

    let resp = client.post(url).json(&proof).send().await.unwrap();
    assert_eq!(resp.status(), 400);

    server.abort();
}
