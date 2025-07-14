use icn_api::identity_trait::{
    DisclosureRequest, DisclosureResponse, IssueCredentialRequest, CredentialReceipt,
};
use icn_common::{Cid, Did};
use icn_node::app_router_with_options;
use reqwest::{Client, StatusCode};
use std::collections::BTreeMap;
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn credential_disclose_route() {
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
    let issue_url = format!("http://{}/identity/credentials/issue", addr);

    let mut attrs = BTreeMap::new();
    attrs.insert("role".to_string(), "tester".to_string());
    attrs.insert("age".to_string(), "30".to_string());
    let req = IssueCredentialRequest {
        issuer: node_did.clone(),
        holder: Did::new("key", "holder"),
        attributes: attrs,
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        expiration: 1,
    };

    let resp = client.post(&issue_url).json(&req).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let cred_resp: CredentialReceipt = resp.json().await.unwrap();

    let disclose_url = format!("http://{}/identity/credentials/disclose", addr);
    let disc_req = DisclosureRequest {
        credential: cred_resp.credential.clone(),
        fields: vec!["role".to_string()],
    };
    let resp = client.post(&disclose_url).json(&disc_req).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let disc_resp: DisclosureResponse = resp.json().await.unwrap();
    assert!(disc_resp.credential.claims.contains_key("role"));
    assert!(!disc_resp.credential.claims.contains_key("age"));
    assert_eq!(disc_resp.proof.disclosed_fields, vec!["age".to_string()]);

    server.abort();
}
