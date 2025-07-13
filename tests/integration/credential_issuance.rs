use icn_api::identity_trait::IssueCredentialRequest;
use icn_node::app_router_with_options;
use icn_common::{Cid, Did, VerifiableCredential};
use reqwest::Client;
use tokio::task;
use tokio::time::{sleep, Duration};
use std::collections::BTreeMap;

#[tokio::test]
async fn credential_issue_route() {
    std::fs::write("fixtures/mana_ledger.tmp", "{\"balances\":{}}").unwrap();
    let (router, ctx) = app_router_with_options(
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
    let node_did = ctx.current_identity.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(100)).await;
    let client = Client::new();
    let url = format!("http://{}/identity/credentials/issue", addr);

    let mut attrs = BTreeMap::new();
    attrs.insert("role".to_string(), "tester".to_string());
    let req = IssueCredentialRequest {
        issuer: node_did.clone(),
        holder: Did::new("key", "holder"),
        attributes: attrs,
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        expiration: 1,
    };

    let resp = client.post(url).json(&req).send().await.unwrap();
    assert!(resp.status().is_success());
    let cred: VerifiableCredential = resp.json().await.unwrap();
    assert!(cred.verify_against_key(ctx.signer.verifying_key_ref()).is_ok());

    server.abort();
}
