use icn_node::app_router_with_options;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::test]
async fn asset_class_lifecycle_unimplemented() {
    let (router, _ctx) = app_router_with_options(
        icn_node::RuntimeMode::Testing,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();

    let resp = client
        .post(format!("http://{}/assets/classes", addr))
        .json(&json!({"name": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let resp = client
        .post(format!("http://{}/assets/mint", addr))
        .json(&json!({"class_id": "test", "to": "did:example:alice", "amount": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let resp = client
        .post(format!("http://{}/assets/transfer", addr))
        .json(&json!({
            "class_id": "test",
            "from": "did:example:alice",
            "to": "did:example:bob",
            "amount": 1
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let resp = client
        .post(format!("http://{}/assets/burn", addr))
        .json(&json!({"class_id": "test", "from": "did:example:bob", "amount": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    let resp = client
        .get(format!("http://{}/dag/events", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    server.abort();
}
