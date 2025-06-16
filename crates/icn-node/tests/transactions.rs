use icn_api::transaction::SubmitTransactionRequest;
use icn_common::{Did, Transaction};
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn submit_transaction_endpoint() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let client = Client::new();
    let tx_req = SubmitTransactionRequest {
        transaction: Transaction {
            id: "tx1".into(),
            timestamp: 0,
            sender_did: Did::new("key", "alice"),
            recipient_did: None,
            payload_type: "test".into(),
            payload: vec![1, 2, 3],
            signature: None,
        },
    };
    let resp = client
        .post(format!("http://{addr}/transactions/submit"))
        .json(&tx_req)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::ACCEPTED);

    server.abort();
}
