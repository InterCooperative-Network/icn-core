#[path = "federation.rs"]
mod federation;

use federation::{ensure_devnet, NODE_A_URL, NODE_B_URL, NODE_C_URL};
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

const RETRY_DELAY: Duration = Duration::from_secs(3);
const MAX_RETRIES: u32 = 10;

#[tokio::test]
async fn dag_sync_status_consistency() {
    let _devnet = ensure_devnet().await;
    let client = Client::new();

    for _ in 0..MAX_RETRIES {
        let mut roots = Vec::new();
        for url in [NODE_A_URL, NODE_B_URL, NODE_C_URL] {
            let resp = client
                .get(&format!("{}/dag/status", url))
                .send()
                .await
                .expect("dag sync");
            assert!(resp.status().is_success());
            let status: Value = resp.json().await.expect("json");
            roots.push(status["current_root"].as_str().map(|s| s.to_string()));
        }
        if roots.iter().all(|r| *r == roots[0]) {
            return;
        }
        sleep(RETRY_DELAY).await;
    }
    panic!("DAG roots not consistent across nodes");
}

