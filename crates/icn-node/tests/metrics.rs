use icn_node::app_router;
use reqwest::Client;
use tokio::task;

#[tokio::test]
async fn metrics_endpoint_returns_metrics_text() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let url = format!("http://{addr}/metrics");
    let body = Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(body.contains("host_submit_mesh_job_calls"));

    server.abort();
}
