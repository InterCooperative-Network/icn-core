use icn_common::{Did, Transaction};
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn submit_transaction_and_query() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = Client::new();
    let base = format!("http://{}", addr);

    // Submit a transaction
    let tx = Transaction {
        id: "tx1".to_string(),
        timestamp: 1,
        sender_did: Did::new("key", "alice"),
        recipient_did: None,
        payload_type: "test".to_string(),
        payload: b"hello".to_vec(),
        signature: None,
    };
    let resp = client
        .post(&format!("{}/transaction/submit", base))
        .json(&tx)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let id_resp: String = resp.json().await.unwrap();
    assert_eq!(id_resp, tx.id);

    // Query data endpoint
    let resp = client
        .post(&format!("{}/data/query", base))
        .json(&serde_json::json!({"query": "example"}))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let result: String = resp.json().await.unwrap();
    assert!(result.contains("example"));

    server.abort();
}
