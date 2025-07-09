use icn_common::{compute_merkle_cid, Cid, DagBlock, Did, Transaction};
use icn_node::app_router;
use reqwest::StatusCode;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
#[ignore]
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
        nonce: 0,
        mana_limit: 100,
        mana_price: 1,
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
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, b"data", &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid,
        data: b"data".to_vec(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
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
