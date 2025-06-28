use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tempfile::tempdir;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn mesh_network_and_ccl_commands() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Submit a mesh job via the new submit-job command
    let job_req = serde_json::json!({
        "manifest_cid": "bafytestmanifest",
        "spec_json": { "Echo": { "payload": "hello" } },
        "cost_mana": 10
    })
    .to_string();
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "submit-job", &job_req])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let body: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let job_id = body["job_id"].as_str().unwrap().to_string();

    // mesh jobs command (existing)
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let job_id_clone = job_id.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "jobs"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&job_id_clone));
    })
    .await
    .unwrap();

    // job-status command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let job_id_clone2 = job_id.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "job-status", &job_id_clone2])
            .assert()
            .success()
            .stdout(predicates::str::contains(&job_id_clone2));
    })
    .await
    .unwrap();

    // network stats command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "network", "stats"])
            .assert()
            .success()
            .stdout(predicates::str::contains("peer_count"));
    })
    .await
    .unwrap();

    // network ping command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "network", "ping", "peer123"])
            .assert()
            .success()
            .stdout(predicates::str::contains("peer123"));
    })
    .await
    .unwrap();

    // compile-ccl command uploads the compiled WASM
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ccl");
    std::fs::write(&file_path, "fn main() -> Bool { return true; }").unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let file_str = file_path.to_str().unwrap().to_string();
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "compile-ccl", &file_str])
            .assert()
            .success()
            .stdout(predicates::str::contains("cid"));
    })
    .await
    .unwrap();

    // add-peer then list-peers
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "add-peer", "peer42"])
            .assert()
            .success();
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "list-peers"])
            .assert()
            .success()
            .stdout(predicates::str::contains("peer42"));
    })
    .await
    .unwrap();

    server.abort();
}
