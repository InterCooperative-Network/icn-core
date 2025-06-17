use reqwest::StatusCode;
use icn_common::{Cid, DagBlock, Did, Transaction};
use icn_node::app_router;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn submit_transaction_and_query_data() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Submit transaction via HTTP
    let tx = Transaction {
        id: "tx-test".to_string(),
        timestamp: 1,
        sender_did: Did::new("key", "alice"),
        recipient_did: None,
        payload_type: "test".to_string(),
        payload: b"hello".to_vec(),
        signature: None,
    };
    let tx_json = serde_json::to_string(&tx).unwrap();
    let client = reqwest::Client::new();
    let submit_url = format!("http://{addr}/transaction/submit");
    let res = client
        .post(&submit_url)
        .body(tx_json)
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["tx_id"], "tx-test");

    // Put a DAG block then query it
    let block = DagBlock {
        cid: Cid::new_v1_dummy(0x71, 0x12, b"data"),
        data: b"data".to_vec(),
        links: vec![],
    };
    let put_url = format!("http://{addr}/dag/put");
    let res = client.post(&put_url).json(&block).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let cid: Cid = res.json().await.unwrap();

    let query_url = format!("http://{addr}/data/query");
    let res = client
        .post(&query_url)
        .json(&serde_json::json!({"cid": cid.to_string()}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let returned: DagBlock = res.json().await.unwrap();
    assert_eq!(returned.cid, cid);

    server.abort();
}
