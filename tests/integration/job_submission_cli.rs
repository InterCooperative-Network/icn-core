use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn job_submission_via_cli() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move { axum::serve(listener, app_router().await).await.unwrap(); });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let job_json = serde_json::json!({
        "manifest_cid": "cidv1-85-20-cli_job_manifest",
        "spec_json": { "Echo": { "payload": "cli job" } },
        "cost_mana": 50
    })
    .to_string();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "submit", &job_json])
            .assert()
            .success()
            .stdout(predicates::str::contains("Successfully submitted job"));
    })
    .await
    .unwrap();

    server.abort();
}
