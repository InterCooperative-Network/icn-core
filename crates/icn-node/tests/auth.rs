use icn_node::app_router_with_options;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn api_key_required_for_requests() {
    let (router, _ctx) = app_router_with_options(Some("secret".into()), None, None, None, None).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let url = format!("http://{addr}/info");

    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(&url)
        .header("x-api-key", "secret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    server.abort();
}
