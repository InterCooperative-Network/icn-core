#[path = "ten_node_scale.rs"]
mod ten;
use ten::{ensure_10node_devnet, wait_for_10node_ready};

use reqwest::Client;
use serde_json::Value;
use std::process::Command;
use tokio::time::{sleep, Duration};

const RETRY_DELAY: Duration = Duration::from_secs(3);
const MAX_RETRIES: u32 = 20;

#[tokio::test]
async fn test_network_partition_recovery() {
    let _devnet = ensure_10node_devnet().await;

    Command::new("docker-compose")
        .args(["-f", "icn-devnet/docker-compose.yml", "pause", "icn-node-c"])
        .status()
        .expect("pause c");
    Command::new("docker-compose")
        .args(["-f", "icn-devnet/docker-compose.yml", "pause", "icn-node-d"])
        .status()
        .expect("pause d");

    let client = Client::new();
    let job: Value = client
        .post("http://localhost:5001/mesh/submit")
        .json(&serde_json::json!({
            "manifest_cid": "cidv1-partition-test", 
            "spec_json": {"Echo": {"payload": "partition"}},
            "cost_mana": 10
        }))
        .send()
        .await
        .expect("submit")
        .json()
        .await
        .expect("json");
    let job_id = job["job_id"].as_str().unwrap().to_string();

    Command::new("docker-compose")
        .args(["-f", "icn-devnet/docker-compose.yml", "unpause", "icn-node-c"])
        .status()
        .expect("unpause c");
    Command::new("docker-compose")
        .args(["-f", "icn-devnet/docker-compose.yml", "unpause", "icn-node-d"])
        .status()
        .expect("unpause d");

    wait_for_10node_ready().await.expect("devnet ready");

    for _ in 0..MAX_RETRIES {
        let resp = client
            .get(&format!("http://localhost:5001/mesh/jobs/{}", job_id))
            .send()
            .await
            .expect("status");
        if resp.status().is_success() {
            let v: Value = resp.json().await.expect("json");
            if v["status"]["status"] == "completed" {
                break;
            }
        }
        sleep(RETRY_DELAY).await;
    }

    for port in [5003u16, 5004u16] {
        let mut synced = false;
        for _ in 0..MAX_RETRIES {
            if let Ok(resp) = client
                .get(&format!("http://localhost:{}/mesh/jobs/{}", port, job_id))
                .send()
                .await
            {
                if resp.status().is_success() {
                    let v: Value = resp.json().await.expect("json");
                    if v["status"]["status"] == "completed" {
                        synced = true;
                        break;
                    }
                }
            }
            sleep(RETRY_DELAY).await;
        }
        assert!(synced, "node {} did not sync receipt", port);
    }
}
