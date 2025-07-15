use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_node::app_router_with_options;
use reqwest::Client;
use std::str::FromStr;
use tempfile::tempdir;
use tokio::task;

#[tokio::test]
async fn verification_failure_refunds_mana() {
    let tmp = tempdir().unwrap();
    let ledger_path = tmp.path().join("ledger.json");
    let (router, ctx) = app_router_with_options(
        None,
        None,
        None,
        None,
        Some(ledger_path),
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
        axum::serve(listener, router).await.unwrap();
    });

    let initial = ctx.get_mana(&ctx.current_identity).await.unwrap();

    let bad_proof = ZkCredentialProof {
        issuer: Did::from_str("did:key:bad").unwrap(),
        holder: Did::from_str("did:key:badholder").unwrap(),
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
    let client = Client::new();
    let resp = client
        .post(&format!("http://{}/identity/verify", addr))
        .json(&bad_proof)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);

    let final_balance = ctx.get_mana(&ctx.current_identity).await.unwrap();
    assert_eq!(initial, final_balance);
    server.abort();
}
