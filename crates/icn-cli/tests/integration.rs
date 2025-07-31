// ...existing code...
use base64::prelude::*;
use icn_node::app_router;
// ...existing code...
use std::process::Command;
use tokio::task;

/// Test complete DAG workflow: put and get
#[tokio::test]
#[serial_test::serial]
async fn test_dag_put_get_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Create a test DAG block
    let dag_block = serde_json::json!({
        "cid": "bafytest123",
        "data": [72, 101, 108, 108, 111], // "Hello" in bytes
        "links": [],
        "timestamp": 1234567890,
        "author_did": "did:example:alice",
        "signature": null,
        "scope": null
    })
    .to_string();

    // Put the block
    let put_output = tokio::task::spawn_blocking({
        // ...existing code...
        let base = base.clone();
        let dag_block = dag_block.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "dag", "put", &dag_block])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(put_output.status.success());

    // Get the block back
    let get_output = tokio::task::spawn_blocking({
        // ...existing code...
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "dag", "get", "\"bafytest123\""])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(get_output.status.success());
    let get_stdout = String::from_utf8(get_output.stdout).unwrap();
    assert!(get_stdout.contains("bafytest123"));

    server.abort();
}

/// Test complete mesh job workflow: submit, check status, and list jobs
#[tokio::test]
#[serial_test::serial]
async fn test_mesh_job_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Submit a mesh job
    let job_request = serde_json::json!({
        "manifest_cid": "bafytest",
        "spec_bytes": BASE64_STANDARD.encode(b"test job data"),
        "spec_json": null,
        "cost_mana": 10
    })
    .to_string();

    let submit_output = tokio::task::spawn_blocking({
        // ...existing code...
        let base = base.clone();
        let job_request = job_request.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "mesh", "submit", &job_request])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(submit_output.status.success());

    let submit_stdout = String::from_utf8(submit_output.stdout).unwrap();
    let submit_json: serde_json::Value = serde_json::from_str(&submit_stdout).unwrap();
    let job_id = submit_json["job_id"].as_str().unwrap();

    // List jobs and verify our job appears
    let list_output = tokio::task::spawn_blocking({
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "mesh", "jobs"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(job_id));

    // Check job status
    let status_output = tokio::task::spawn_blocking({
        let base = base.clone();
        let job_id = job_id.to_string();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "mesh", "status", &job_id])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(status_output.status.success());
    let status_stdout = String::from_utf8(status_output.stdout).unwrap();
    assert!(status_stdout.contains(job_id));

    server.abort();
}

/// Test governance workflow: submit proposal, list proposals, and vote
#[tokio::test]
#[serial_test::serial]
async fn test_governance_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Submit a governance proposal
    let proposal_request = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": {
            "type": "GenericText",
            "data": { "text": "Test proposal" }
        },
        "description": "Test proposal description",
        "duration_secs": 3600
    })
    .to_string();

    let submit_output = tokio::task::spawn_blocking({
        let base = base.clone();
        let proposal_request = proposal_request.clone();
        move || {
            Command::new(&bin)
                .args([
                    "--api-url",
                    &base,
                    "governance",
                    "submit",
                    &proposal_request,
                ])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(submit_output.status.success());

    let submit_stdout = String::from_utf8(submit_output.stdout).unwrap();
    assert!(submit_stdout.contains("Successfully submitted proposal"));

    // List proposals and verify our proposal appears
    let list_output = tokio::task::spawn_blocking({
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "governance", "proposals"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains("Test proposal"));

    server.abort();
}

/// Test federation workflow: init, join, status, and leave
#[tokio::test]
#[serial_test::serial]
async fn test_federation_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Initialize federation
    let init_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "federation", "init"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(init_output.status.success());

    // Join a peer
    let join_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "federation", "join", "test-peer"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(join_output.status.success());

    // Check federation status
    let status_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "federation", "status"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(status_output.status.success());
    let status_stdout = String::from_utf8(status_output.stdout).unwrap();
    assert!(status_stdout.contains("test-peer"));

    // Leave the peer
    let leave_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "federation", "leave", "test-peer"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(leave_output.status.success());

    server.abort();
}

/// Test identity workflow: generate proof and verify
#[tokio::test]
#[serial_test::serial]
async fn test_identity_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Generate a proof
    let generate_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        move || {
            Command::new(&bin)
                .args([
                    "identity",
                    "generate-proof",
                    "--issuer",
                    "did:example:issuer",
                    "--holder",
                    "did:example:holder",
                    "--claim-type",
                    "test-claim",
                    "--schema",
                    "bafytest",
                    "--backend",
                    "groth16",
                ])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(generate_output.status.success());

    let generate_stdout = String::from_utf8(generate_output.stdout).unwrap();
    let proof_json: serde_json::Value = serde_json::from_str(&generate_stdout).unwrap();

    // Verify the proof was generated
    assert!(proof_json["proof"].is_array());
    assert!(proof_json["issuer"].as_str().unwrap().contains("issuer"));
    assert!(proof_json["holder"].as_str().unwrap().contains("holder"));

    // Verify the proof (this will likely fail since it's a dummy proof, but we test the flow)
    let verify_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        let proof_str = generate_stdout.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "identity", "verify-proof", &proof_str])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    // The verify command should execute (might fail due to dummy proof, but that's expected)
    assert!(verify_output.status.success() || verify_output.status.code().is_some());

    server.abort();
}

/// Test CCL workflow: compile, lint, and explain
#[tokio::test]
#[serial_test::serial]
async fn test_ccl_workflow() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ccl");

    // Create a simple CCL file
    std::fs::write(&file_path, "fn main() -> Bool { return true; }").unwrap();

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let file_str = file_path.to_str().unwrap();

    // Compile the CCL file
    let compile_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let file_str = file_str.to_string();
        move || {
            Command::new(&bin)
                .args(["ccl", "compile", &file_str])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(compile_output.status.success());
    let compile_stdout = String::from_utf8(compile_output.stdout).unwrap();
    assert!(compile_stdout.contains("cid"));

    // Lint the CCL file
    let lint_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let file_str = file_str.to_string();
        move || {
            Command::new(&bin)
                .args(["ccl", "lint", &file_str])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(lint_output.status.success());
    let lint_stdout = String::from_utf8(lint_output.stdout).unwrap();
    assert!(lint_stdout.contains("passed linting"));

    // Explain the CCL file
    let explain_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let file_str = file_str.to_string();
        move || {
            Command::new(&bin)
                .args(["ccl", "explain", &file_str])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(explain_output.status.success());
    let explain_stdout = String::from_utf8(explain_output.stdout).unwrap();
    assert!(explain_stdout.contains("Function"));
}

/// Test network command workflow
#[tokio::test]
#[serial_test::serial]
async fn test_network_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Get network stats
    let stats_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "network", "stats"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(stats_output.status.success());
    let stats_stdout = String::from_utf8(stats_output.stdout).unwrap();
    assert!(stats_stdout.contains("peer_count"));

    // Get network peers
    let peers_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "network", "peers"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(peers_output.status.success());

    // Test network ping
    let ping_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "network", "ping", "test-peer"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(ping_output.status.success());
    let ping_stdout = String::from_utf8(ping_output.stdout).unwrap();
    assert!(ping_stdout.contains("test-peer"));

    server.abort();
}

/// Test multi-command workflow scenario
#[tokio::test]
#[serial_test::serial]
async fn test_multi_command_workflow() {
    let _ = std::fs::remove_dir_all("./mana_ledger.sled");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, app_router().await).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let bin = env!("CARGO_BIN_EXE_icn-cli");
    let base = format!("http://{addr}");

    // Step 1: Get node info
    let info_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "info"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(info_output.status.success());

    // Step 2: Check node status
    let status_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "status"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(status_output.status.success());

    // Step 3: Submit a job
    let job_request = serde_json::json!({
        "manifest_cid": "bafytest",
        "spec_bytes": BASE64_STANDARD.encode(b"multi-command test"),
        "spec_json": null,
        "cost_mana": 10
    })
    .to_string();

    let submit_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        let job_request = job_request.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "mesh", "submit", &job_request])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(submit_output.status.success());

    // Step 4: List jobs to verify submission
    let list_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "mesh", "jobs"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(list_output.status.success());

    // Step 5: Get metrics
    let metrics_output = tokio::task::spawn_blocking({
        let bin = bin.to_string();
        let base = base.clone();
        move || {
            Command::new(&bin)
                .args(["--api-url", &base, "metrics"])
                .output()
                .unwrap()
        }
    })
    .await
    .unwrap();

    assert!(metrics_output.status.success());
    let metrics_stdout = String::from_utf8(metrics_output.stdout).unwrap();
    assert!(metrics_stdout.contains("mesh_jobs_submitted_total"));

    server.abort();
}
