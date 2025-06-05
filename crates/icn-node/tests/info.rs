use icn_node::app_router;             // expose a fn that builds the Router<State>
use tokio::task;
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn info_endpoint_returns_expected_json() {
    // spin up the app on a random OS port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let url = format!("http://{}/info", addr);
    let json: Value = Client::new().get(&url).send().await.unwrap().json().await.unwrap();

    assert!(json["name"].as_str().unwrap().contains("ICN"));
    assert!(json["version"].as_str().unwrap().contains("0.1.0"));

    server.abort(); // shut the axum task down
} 