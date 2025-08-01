use assert_cmd::prelude::*;
use base64::Engine;
use icn_node::app_router;
use predicates::prelude::PredicateBooleanExt;
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

    // Submit a mesh job via the new root command
    let spec = icn_mesh::JobSpec {
        kind: icn_mesh::JobKind::Echo {
            payload: "hello".into(),
        },
        ..Default::default()
    };
    let job_req = serde_json::json!({
        "manifest_cid": "bafytestmanifest",
        "spec_bytes": base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&spec).unwrap()),
        "spec_json": null,
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

    // compile-ccl upload command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let file_str2 = file_path.to_str().unwrap().to_string();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "compile-ccl", &file_str2])
            .assert()
            .success()
            .stdout(predicates::str::contains("cid"));
    })
    .await
    .unwrap();

    // ccl lint command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let lint_file = file_path.to_str().unwrap().to_string();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["ccl", "lint", &lint_file])
            .assert()
            .success()
            .stdout(predicates::str::contains("passed linting"));
    })
    .await
    .unwrap();

    // ccl explain command
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let explain_file = file_path.to_str().unwrap().to_string();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["ccl", "explain", &explain_file])
            .assert()
            .success()
            .stdout(predicates::str::contains("Function"));
    })
    .await
    .unwrap();

    // federation join, status, leave commands
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "join", "peer1"])
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
            .stdout(predicates::str::contains("peer1"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "federation", "leave", "peer1"])
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
            .stdout(predicates::str::contains("peer1").not());
    })
    .await
    .unwrap();

    server.abort();
}
