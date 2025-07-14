use assert_cmd::prelude::*;
use icn_node::app_router;
use predicates::str::contains;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn health_commands_work() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "health"])
            .assert()
            .success()
            .stdout(contains("status"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "ready"])
            .assert()
            .success()
            .stdout(contains("ready"));
    })
    .await
    .unwrap();

    server.abort();
}
