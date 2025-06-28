#[path = "federation.rs"]
mod federation;

use federation::{ensure_devnet, wait_for_federation_ready, NODE_A_URL, NODE_C_URL};
use reqwest::Client;
use serde_json::Value;
use std::process::Command;
use tokio::time::{sleep, Duration};

const RETRY_DELAY: Duration = Duration::from_secs(3);
const MAX_RETRIES: u32 = 20;

#[tokio::test]
async fn test_network_resilience() {
    let _devnet = ensure_devnet().await;

    // Stop Node C to simulate a disconnect
    Command::new("docker-compose")
        .args(&["-f", "icn-devnet/docker-compose.yml", "stop", "icn-node-c"])
        .status()
        .expect("failed to stop node c");

    sleep(Duration::from_secs(5)).await;

    let client = Client::new();

    // Submit a job while Node C is offline
    let job_request = serde_json::json!({
        "manifest_cid": "cidv1-resilience-test-manifest",
        "spec_json": { "Echo": { "payload": "Network resilience test" } },
        "cost_mana": 50
    });

    let submit_res: Value = client
        .post(&format!("{}/mesh/submit", NODE_A_URL))
        .header("Content-Type", "application/json")
        .json(&job_request)
        .send()
        .await
        .expect("submit job")
        .json()
        .await
        .expect("submit json");

    let job_id = submit_res["job_id"].as_str().expect("job_id").to_string();

    // Wait for job completion on Node A
    let mut completed = false;
    for _ in 0..MAX_RETRIES {
        let resp = client
            .get(&format!("{}/mesh/jobs/{}", NODE_A_URL, job_id))
            .send()
            .await
            .expect("job status");
        if resp.status().is_success() {
            let status: Value = resp.json().await.expect("status json");
            if status["status"]["status"] == "completed" {
                completed = true;
                break;
            }
        }
        sleep(RETRY_DELAY).await;
    }

    assert!(completed, "job did not complete while node C was offline");

    // Restart Node C
    Command::new("docker-compose")
        .args(&["-f", "icn-devnet/docker-compose.yml", "start", "icn-node-c"])
        .status()
        .expect("failed to start node c");

    // Wait for federation to be ready again
    wait_for_federation_ready()
        .await
        .expect("federation not ready after restart");

    // Verify Node C receives the job status and receipt
    let mut node_c_synced = false;
    for _ in 0..MAX_RETRIES {
        if let Ok(resp) = client
            .get(&format!("{}/mesh/jobs/{}", NODE_C_URL, &job_id))
            .send()
            .await
        {
            if resp.status().is_success() {
                let status: Value = resp.json().await.expect("status json");
                if status["status"]["status"] == "completed" {
                    let result_cid = status["status"]["result_cid"].as_str().unwrap_or("");
                    if !result_cid.is_empty() {
                        let dag_res = client
                            .post(&format!("{}/dag/get", NODE_C_URL))
                            .json(&serde_json::json!({ "cid": result_cid }))
                            .send()
                            .await
                            .expect("dag get");
                        if dag_res.status().is_success() {
                            node_c_synced = true;
                            break;
                        }
                    }
                }
            }
        }
        sleep(RETRY_DELAY).await;
    }

    assert!(node_c_synced, "Node C did not sync job receipt after reconnect");
}

