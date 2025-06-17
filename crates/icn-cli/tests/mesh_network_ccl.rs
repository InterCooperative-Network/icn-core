use assert_cmd::prelude::*;
use icn_node::app_router;
use reqwest::StatusCode;
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

    // Submit a mesh job via HTTP so the CLI can query it
    let job_req = serde_json::json!({
        "manifest_cid": "bafytestmanifest",
        "spec_json": { "Echo": { "payload": "hello" } },
        "cost_mana": 10
    });
    let client = reqwest::Client::new();
    let submit_url = format!("http://{addr}/mesh/submit");
    let res = client
        .post(&submit_url)
        .json(&job_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap().to_string();

    // mesh jobs command
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

    // mesh status command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let job_id_clone2 = job_id.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "status", &job_id_clone2])
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

    // ccl compile command
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ccl");
    std::fs::write(&file_path, "fn main() -> Bool { return true; }").unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let file_str = file_path.to_str().unwrap().to_string();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["ccl", "compile", &file_str])
            .assert()
            .success()
            .stdout(predicates::str::contains("cid"));
    })
    .await
    .unwrap();

    server.abort();
}
