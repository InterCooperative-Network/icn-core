use icn_node::app_router;
use reqwest::Client;
use serde_json::Value;
use tokio::task;

#[tokio::test]
async fn status_endpoint_returns_runtime_data() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let url = format!("http://{addr}/status");
    let json: Value = Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(json["is_online"].as_bool().unwrap());
    assert_eq!(json["peer_count"].as_u64().unwrap(), 0);
    assert_eq!(json["current_block_height"].as_u64().unwrap(), 0);

    server.abort();
}
