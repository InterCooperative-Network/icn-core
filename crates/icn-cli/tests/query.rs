use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn query_command_returns_value() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "query", "hello"])
            .assert()
            .success()
            .stdout(predicates::str::contains("hello"));
    })
    .await
    .unwrap();

    server.abort();
}
