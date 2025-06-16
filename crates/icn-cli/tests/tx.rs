use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn tx_command_submits_transaction() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    let tx_json = serde_json::json!({
        "transaction": {
            "id": "t1",
            "timestamp": 0,
            "sender_did": {"method":"key","id_string":"alice"},
            "recipient_did": null,
            "payload_type": "test",
            "payload": [],
            "signature": null
        }
    })
    .to_string();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "tx", &tx_json])
            .assert()
            .success()
            .stdout(predicates::str::contains("Transaction submitted"));
    })
    .await
    .unwrap();

    server.abort();
}
