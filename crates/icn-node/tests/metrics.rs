use icn_node::app_router;
use prometheus_parse::Scrape;
use reqwest::Client;
use tokio::task;

#[tokio::test]
#[ignore]
async fn metrics_endpoint_returns_prometheus_text() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let body = Client::new()
        .get(format!("http://{addr}/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let scrape = Scrape::parse(body.lines().map(|l| Ok(l.to_string()))).unwrap();
    assert!(scrape
        .samples
        .iter()
        .any(|s| s.metric == "host_submit_mesh_job_calls"));

    server.abort();
}
