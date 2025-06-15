use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn submit_governance_proposal() {
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

    let submit_json = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": { "type": "GenericText", "data": { "text": "hi" } },
        "description": "test",
        "duration_secs": 60
    })
    .to_string();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "submit", &submit_json])
            .assert()
            .success()
            .stdout(predicates::str::contains("Successfully submitted proposal"));
    })
    .await
    .unwrap();

    server.abort();
}
