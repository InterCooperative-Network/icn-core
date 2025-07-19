use icn_identity::Credential;
use icn_identity::generate_ed25519_keypair;
use icn_identity::{did_key_from_verifying_key};
use icn_node::app_router_with_options;
use reqwest::{Client, StatusCode};
use tokio::task;
use tokio::time::{sleep, Duration};
use icn_common::{Did, Cid};
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::test]
async fn verify_trusted_and_untrusted_issuers() {
    std::fs::write("fixtures/mana_ledger.tmp", "{\"balances\":{}}").unwrap();
    let (sk_trusted, pk_trusted) = generate_ed25519_keypair();
    let trusted_did = Did::from_str(&did_key_from_verifying_key(&pk_trusted)).unwrap();
    std::env::set_var("ICN_TRUSTED_ISSUERS", trusted_did.to_string());
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
    std::env::remove_var("ICN_TRUSTED_ISSUERS");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });
    sleep(Duration::from_millis(100)).await;
    let client = Client::new();
    let url = format!("http://{}/identity/credentials/verify", addr);

    let mut claims = HashMap::new();
    claims.insert("role".to_string(), "tester".to_string());
    let mut cred = Credential::new(trusted_did.clone(), Did::new("key", "holder"), claims, Some(Cid::new_v1_sha256(0x55, b"schema")));
    cred.expires_at = Some(chrono::Utc::now().timestamp() as u64 + 60);
    cred.sign_claims(&sk_trusted);
    let resp = client.post(&url).json(&cred).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Untrusted issuer
    let (sk_untrusted, pk_untrusted) = generate_ed25519_keypair();
    let untrusted_did = Did::from_str(&did_key_from_verifying_key(&pk_untrusted)).unwrap();
    let mut claims2 = HashMap::new();
    claims2.insert("role".to_string(), "tester".to_string());
    let mut cred2 = Credential::new(untrusted_did, Did::new("key", "holder"), claims2, Some(Cid::new_v1_sha256(0x55, b"schema")));
    cred2.expires_at = Some(chrono::Utc::now().timestamp() as u64 + 60);
    cred2.sign_claims(&sk_untrusted);
    let resp = client.post(&url).json(&cred2).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    server.abort();
}
