use assert_cmd::prelude::*;
use icn_node::app_router;
use std::process::Command;
use tokio::task;

#[tokio::test]
async fn info_status_basic() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    Command::new(bin)
        .args(["--api-url", &base, "info"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Node Information"));

    Command::new(bin)
        .args(["--api-url", &base, "status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Node Status"));

    server.abort();
}

#[tokio::test]
#[ignore]
async fn governance_endpoints() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    let submit_json = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": { "GenericText": { "text": "hi" } },
        "description": "test",
        "duration_secs": 60
    })
    .to_string();

    let output = Command::new(bin)
        .args(["--api-url", &base, "governance", "submit", &submit_json])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully submitted proposal"));
    let start = stdout.find('"').unwrap();
    let end = stdout[start + 1..].find('"').unwrap() + start + 1;
    let pid = &stdout[start + 1..end];

    Command::new(bin)
        .args(["--api-url", &base, "governance", "proposals"])
        .assert()
        .success()
        .stdout(predicates::str::contains(pid));

    Command::new(bin)
        .args(["--api-url", &base, "governance", "proposal", pid])
        .assert()
        .success()
        .stdout(predicates::str::contains(pid));

    let vote_json = serde_json::json!({
        "voter_did": "did:example:bob",
        "proposal_id": pid,
        "vote_option": "yes"
    })
    .to_string();

    Command::new(bin)
        .args(["--api-url", &base, "governance", "vote", &vote_json])
        .assert()
        .success()
        .stdout(predicates::str::contains("Vote response"));

    server.abort();
}
