use assert_cmd::prelude::*;
use icn_node::app_router;
use predicates::prelude::PredicateBooleanExt;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn federation_commands_work() {
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
            .args(["--api-url", &base, "federation", "init"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "join", "peerA"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("peerA"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "leave", "peerA"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "list-peers"])
            .assert()
            .success()
            .stdout(predicates::str::contains("peerA").not());
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "sync"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    server.abort();
}
