use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn info_command_displays_node_info() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    // Give the server a moment to start listening
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "info"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Node Information"));
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn status_command_reports_node_status() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
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
            .args(["--api-url", &base, "status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Node Status"));
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn metrics_command_outputs_metrics() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
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
            .args(["--api-url", &base, "metrics"])
            .assert()
            .success()
            .stdout(predicates::str::contains("host_submit_mesh_job_calls"));
    })
    .await
    .unwrap();

    server.abort();
}
