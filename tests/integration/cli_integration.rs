use assert_cmd::prelude::*;
use icn_node::app_router;
use icn_common::{compute_merkle_cid, DagBlock, Did};
use predicates::str::contains;
use std::process::Command;
use tokio::task::{self, JoinHandle};
use tokio::time::{sleep, Duration};

async fn spawn_node() -> (String, JoinHandle<()>) {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });
    sleep(Duration::from_millis(100)).await;
    (format!("http://{}", addr), handle)
}

#[tokio::test]
#[serial_test::serial]
async fn dag_cli_round_trip() {
    let (base, server) = spawn_node().await;

    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, b"data", &[], ts, &author, &sig_opt);
    let block = DagBlock {
        cid,
        data: b"data".to_vec(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
    };
    let block_json = serde_json::to_string(&block).unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let put_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "dag", "put", &block_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(put_out.status.success());
    let stdout = String::from_utf8(put_out.stdout).unwrap();
    let start = stdout.find('"').unwrap();
    let end = stdout[start + 1..].find('"').unwrap() + start + 1;
    let cid_str = stdout[start + 1..end].to_string();
    let cid_json = format!("\"{}\"", cid_str);

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let get_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "dag", "get", &cid_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(get_out.status.success());
    let stdout = String::from_utf8(get_out.stdout).unwrap();
    let json_start = stdout.find('{').unwrap();
    let json_end = stdout.rfind('}').unwrap() + 1;
    let retrieved: DagBlock = serde_json::from_str(&stdout[json_start..json_end]).unwrap();
    assert_eq!(retrieved.data, block.data);

    server.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn governance_cli_flow() {
    let (base, server) = spawn_node().await;

    let submit_json = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": { "type": "GenericText", "data": { "text": "hi" } },
        "description": "test",
        "duration_secs": 60
    })
    .to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let output = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "submit", &submit_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let start = stdout.find('"').unwrap();
    let end = stdout[start + 1..].find('"').unwrap() + start + 1;
    let pid = stdout[start + 1..end].to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let proposals_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "proposals"])
            .assert()
            .success()
            .stdout(contains(&pid));
    })
    .await;
    proposals_out.unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let proposal_pid = pid.clone();
    let proposal_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "proposal", &proposal_pid])
            .assert()
            .success()
            .stdout(contains(&proposal_pid));
    })
    .await;
    proposal_out.unwrap();

    let vote_json = serde_json::json!({
        "voter_did": "did:example:bob",
        "proposal_id": pid,
        "vote_option": "yes"
    })
    .to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let vote_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "vote", &vote_json])
            .assert()
            .success()
            .stdout(contains("Vote response"));
    })
    .await;
    vote_out.unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let tally_pid = pid.clone();
    let tally_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "--api-url",
                &base,
                "governance",
                "tally",
                &tally_pid,
            ])
            .assert()
            .success()
            .stdout(contains("Accepted"));
    })
    .await;
    tally_out.unwrap();

    server.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn mesh_job_submission_via_cli() {
    let (base, server) = spawn_node().await;

    let job_req = serde_json::json!({
        "manifest_cid": "bafytestmanifest",
        "spec_json": { "Echo": { "payload": "hello" } },
        "cost_mana": 10
    })
    .to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let output = task::spawn_blocking(move || {
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

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let jobs_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "jobs"])
            .assert()
            .success()
            .stdout(contains(&job_id));
    })
    .await;
    jobs_out.unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let status_id = job_id.clone();
    let status_out = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "job-status", &status_id])
            .assert()
            .success()
            .stdout(contains(&status_id));
    })
    .await;
    status_out.unwrap();

    server.abort();
}
