use assert_cmd::prelude::*;
use icn_node::app_router;
// ...existing code...
use std::process::Command;
use tokio::task;

#[tokio::test]
#[serial_test::serial]
async fn info_status_basic() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");
    let base_info = base.clone();

    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_info, "info"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Node Information"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_status = base;
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_status, "status"])
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
async fn governance_endpoints() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

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

    let base_submit = base.clone();
    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "--api-url",
                &base_submit,
                "governance",
                "submit",
                &submit_json,
            ])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Successfully submitted proposal"));
    let start = stdout.find('"').unwrap();
    let end = stdout[start + 1..].find('"').unwrap() + start + 1;
    let pid = stdout[start + 1..end].to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_proposals = base.clone();
    let pid_clone = pid.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_proposals, "governance", "proposals"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&pid_clone));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_proposal = base.clone();
    let pid_owned = pid.clone();
    tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "--api-url",
                &base_proposal,
                "governance",
                "proposal",
                &pid_owned,
            ])
            .assert()
            .success()
            .stdout(predicates::str::contains(&pid_owned));
    })
    .await
    .unwrap();

    let vote_json = serde_json::json!({
        "voter_did": "did:example:bob",
        "proposal_id": pid,
        "vote_option": "yes"
    })
    .to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_vote = base;
    let _ = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base_vote, "governance", "vote", &vote_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base_tally = format!("http://{addr}");
    let pid_for_tally = pid.clone();
    let _ = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "--api-url",
                &base_tally,
                "governance",
                "tally",
                &pid_for_tally,
            ])
            .output()
            .unwrap()
    })
    .await
    .unwrap();

    server.abort();
}
