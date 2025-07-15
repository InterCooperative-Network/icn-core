use icn_api::identity_trait::{
    CredentialResponse, IssueCredentialRequest, RevokeCredentialRequest, VerificationResponse,
};
use icn_common::{Cid, Did};
use icn_identity::Credential;
use icn_node::app_router_with_options;
use reqwest::{Client, StatusCode};
use std::collections::BTreeMap;
use tokio::task;
use tokio::time::{sleep, Duration};

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

    let resp = client.post(&url).json(&req).send().await.unwrap();
    assert!(resp.status().is_success());
    let resp_body: CredentialResponse = resp.json().await.unwrap();
    let cid = resp_body.cid.clone();
    let cred: Credential = resp_body.credential;
    for (k, _) in &cred.claims {
        assert!(cred.verify_claim(k, ctx.signer.verifying_key_ref()).is_ok());
    }

    // retrieve
    let get_url = format!("http://{}/identity/credentials/{}", addr, cid.to_string());
    let resp = client.get(&get_url).send().await.unwrap();
    assert!(resp.status().is_success());
    let retrieved: CredentialResponse = resp.json().await.unwrap();
    assert_eq!(retrieved.cid, cid);

    // verify via API
    let verify_url = format!("http://{}/identity/credentials/verify", addr);
    let resp = client
        .post(&verify_url)
        .json(&retrieved.credential)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let v: VerificationResponse = resp.json().await.unwrap();
    assert!(v.valid);

    // revoke
    let revoke_url = format!("http://{}/identity/credentials/revoke", addr);
    let resp = client
        .post(&revoke_url)
        .json(&RevokeCredentialRequest { cid: cid.clone() })
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client.get(&get_url).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    server.abort();
}
