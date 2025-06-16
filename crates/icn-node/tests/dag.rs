use icn_common::Cid;
use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn dag_put_and_get_round_trip() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let client = Client::new();
    let put_resp = client
        .post(format!("http://{addr}/dag/put"))
        .json(&serde_json::json!({ "data": [1, 2, 3] }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_resp.status(), reqwest::StatusCode::CREATED);
    let cid: Cid = put_resp.json().await.unwrap();

    let get_resp = client
        .post(format!("http://{addr}/dag/get"))
        .json(&serde_json::json!({ "cid": cid.to_string() }))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status(), reqwest::StatusCode::OK);
    let data: Vec<u8> = get_resp.json().await.unwrap();
    assert_eq!(data, vec![1, 2, 3]);

    server.abort();
}
