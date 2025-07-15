use assert_cmd::prelude::*;
use base64;
use bincode;
use icn_common::{compute_merkle_cid, Cid, DagBlock, Did};
use icn_mesh::{JobKind, JobSpec};
use icn_node::app_router;
use std::process::Command;
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn dag_storage_via_cli() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(100)).await;

    let ts = 0u64;
    let author = Did::new("example", "alice");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, b"data", &[], ts, &author, &sig_opt, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data: b"data".to_vec(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig_opt,
        scope: None,
    };
    let block_json = serde_json::to_string(&block).unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let output = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "dag", "put", &block_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&cid.to_string()));

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base2 = format!("http://{}", addr);
    let cid_json = serde_json::to_string(&cid).unwrap();
    let output = task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base2, "dag", "get", &cid_json])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&cid.to_string()));

    server.abort();
}

#[tokio::test]
async fn governance_proposal_via_cli() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let submit_json = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": { "type": "GenericText", "data": { "text": "hi" } },
        "description": "test",
        "duration_secs": 60
    })
    .to_string();
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
    assert!(stdout.contains("Successfully submitted proposal"));
    let start = stdout.find('"').unwrap();
    let end = stdout[start + 1..].find('"').unwrap() + start + 1;
    let pid = stdout[start + 1..end].to_string();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let pid_clone = pid.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "proposals"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&pid_clone));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let pid_owned = pid.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "proposal", &pid_owned])
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
    let base = format!("http://{}", addr);
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "vote", &vote_json])
            .assert()
            .success()
            .stdout(predicates::str::contains("Vote response"));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let pid_for_tally = pid.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "governance", "tally", &pid_for_tally])
            .assert()
            .success()
            .stdout(predicates::str::contains("Accepted"));
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test]
async fn mesh_job_via_cli() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await.into_make_service()).await.unwrap();
    });
    sleep(Duration::from_millis(100)).await;

    let spec = icn_mesh::JobSpec {
        kind: icn_mesh::JobKind::Echo {
            payload: "hello".into(),
        },
        ..Default::default()
    };
    let job_req = serde_json::json!({
        "manifest_cid": "bafytestmanifest",
        "spec_bytes": base64::encode(bincode::serialize(&spec).unwrap()),
        "spec_json": null,
        "cost_mana": 10
    })
    .to_string();
    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
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
    let base = format!("http://{}", addr);
    let job_id_clone = job_id.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "mesh", "jobs"])
            .assert()
            .success()
            .stdout(predicates::str::contains(&job_id_clone));
    })
    .await
    .unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{}", addr);
    let job_id_clone2 = job_id.clone();
    task::spawn_blocking(move || {
        Command::new(bin)
            .args(["--api-url", &base, "job-status", &job_id_clone2])
            .assert()
            .success()
            .stdout(predicates::str::contains(&job_id_clone2));
    })
    .await
    .unwrap();

    server.abort();
}
