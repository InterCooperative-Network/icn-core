use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[ignore]
async fn info_command_displays_node_info() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    Command::new(bin)
        .args(["--api-url", &base, "info"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Node Information"));

    server.abort();
}

#[tokio::test]
#[ignore]
async fn status_command_reports_node_status() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    Command::new(bin)
        .args(["--api-url", &base, "status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Node Status"));

    server.abort();
}
