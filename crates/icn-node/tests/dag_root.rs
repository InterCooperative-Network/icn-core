use icn_common::parse_cid_from_string;
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn dag_root_matches_across_nodes() {
    let listener1 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr1 = listener1.local_addr().unwrap();
    let server1 = task::spawn(async move {
        axum::serve(listener1, app_router().await).await.unwrap();
    });

    let listener2 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener2.local_addr().unwrap();
    let server2 = task::spawn(async move {
        axum::serve(listener2, app_router().await).await.unwrap();
    });

    let client = Client::new();
    for addr in &[addr1, addr2] {
        let resp = client
            .post(format!("http://{addr}/dag/put"))
            .json(&serde_json::json!({ "data": [1, 2, 3] }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
    }

    let root1: String = client
        .get(format!("http://{addr1}/dag/root"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let root2: String = client
        .get(format!("http://{addr2}/dag/root"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        parse_cid_from_string(&root1).unwrap(),
        parse_cid_from_string(&root2).unwrap()
    );

    server1.abort();
    server2.abort();
}
