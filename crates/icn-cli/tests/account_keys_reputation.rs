use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn account_keys_reputation_commands() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let base = format!("http://{addr}");
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    // keys show
    let output = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "keys", "show"])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let body: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let did = body["did"].as_str().unwrap().to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_bal = format!("http://{addr}");
    let did_clone = did.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_bal, "accounts", "balance", &did_clone])
            .assert()
            .success()
            .stdout(predicates::str::contains("balance"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_rep = format!("http://{addr}");
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_rep, "reputation", "get", &did])
            .assert()
            .success()
            .stdout(predicates::str::contains("score"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_net = format!("http://{addr}");
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_net, "network", "peers"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Local Peer ID"));
    })
    .await
    .unwrap();

    server.abort();
}
