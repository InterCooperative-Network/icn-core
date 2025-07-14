#[path = "federation.rs"]
mod federation;

use federation::{ensure_devnet, wait_for_federation_ready, NODE_A_URL, NODE_B_URL};
use reqwest::Client;
use serde_json::Value;
use std::process::Command;
use tokio::time::{sleep, Duration};

const RETRY_DELAY: Duration = Duration::from_secs(3);
const MAX_RETRIES: u32 = 20;

#[tokio::test]
async fn test_chaos_recovery() {
    let _devnet = ensure_devnet().await;

    Command::new("bash")
        .args([
            "./scripts/chaos_test.sh",
            "--scenario",
            "network_partition",
            "--duration",
            "5",
        ])
        .status()
        .expect("failed to run chaos test");

    wait_for_federation_ready()
        .await
        .expect("federation not ready after chaos");

    let client = Client::new();
    let job_request = serde_json::json!({
        "manifest_cid": "cidv1-chaos-test-manifest",
        "spec_json": { "Echo": { "payload": "Chaos test" } },
        "cost_mana": 25
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

    let mut completed = false;
    for _ in 0..MAX_RETRIES {
        if let Ok(resp) = client
            .get(&format!("{}/mesh/jobs/{}", NODE_B_URL, job_id))
            .send()
            .await
        {
            if resp.status().is_success() {
                let status: Value = resp.json().await.expect("status json");
                if status["status"]["status"] == "completed" {
                    completed = true;
                    break;
                }
            }
        }
        sleep(RETRY_DELAY).await;
    }

    assert!(completed, "job did not complete after chaos test");
}
