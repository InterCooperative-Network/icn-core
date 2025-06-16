use icn_common::{Did, SubmitTransactionRequest, Transaction};
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn submit_transaction_returns_id() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let tx = Transaction {
        id: "t1".into(),
        timestamp: 0,
        sender_did: Did::new("key", "alice"),
        recipient_did: None,
        payload_type: "test".into(),
        payload: vec![],
        signature: None,
    };
    let req = SubmitTransactionRequest {
        transaction: tx.clone(),
    };

    let client = Client::new();
    let resp: serde_json::Value = client
        .post(format!("http://{addr}/transaction/submit"))
        .json(&req)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["transaction_id"], tx.id);

    server.abort();
}
