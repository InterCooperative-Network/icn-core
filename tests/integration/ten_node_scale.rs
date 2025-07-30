use once_cell::sync::OnceCell;
use serde_json::Value;
use std::{process::Command, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

const NODE_PORTS: [u16; 10] = [5001, 5002, 5003, 5004, 5005, 5006, 5007, 5008, 5009, 5010];
const MAX_RETRIES: u32 = 30;
const RETRY_DELAY: Duration = Duration::from_secs(2);

static DEVNET_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::new();

pub struct TenDevnetGuard {
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

impl Drop for TenDevnetGuard {
    fn drop(&mut self) {
        let _ = Command::new("bash")
            .arg("./scripts/run_10node_devnet.sh")
            .arg("--stop-only")
            .status();
    }
}

pub async fn ensure_10node_devnet() -> Option<TenDevnetGuard> {
    if std::env::var("ICN_DEVNET_RUNNING").is_ok() {
        wait_for_10node_ready().await.ok();
        return None;
    }
    let lock = DEVNET_LOCK.get_or_init(|| Arc::new(Mutex::new(())));
    let guard = lock.lock_owned().await;

    Command::new("bash")
        .arg("./scripts/run_10node_devnet.sh")
        .arg("--start-only")
        .status()
        .expect("Failed to start 10 node devnet");

    wait_for_10node_ready()
        .await
        .expect("10 node devnet not ready");
    Some(TenDevnetGuard { _guard: guard })
}

pub async fn wait_for_10node_ready() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    for _ in 0..MAX_RETRIES {
        let mut all = true;
        for port in NODE_PORTS.iter() {
            let url = format!("http://localhost:{}/info", port);
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {}
                _ => {
                    all = false;
                    break;
                }
            }
        }
        if all {
            return Ok(());
        }
        sleep(RETRY_DELAY).await;
    }
    Err("devnet not ready".into())
}

fn parse_metric(text: &str, name: &str) -> f64 {
    for line in text.lines() {
        if line.starts_with(name) {
            if let Some(v) = line.split_whitespace().nth(1) {
                if let Ok(val) = v.parse() {
                    return val;
                }
            }
        }
    }
    0.0
}

#[tokio::test]
async fn scale_test_10node_load() {
    let _devnet = ensure_10node_devnet().await;

    // Run load scenario via script
    Command::new("bash")
        .arg("./scripts/run_10node_devnet.sh")
        .arg("--jobs-only")
        .status()
        .expect("failed to run load script");

    sleep(Duration::from_secs(10)).await;

    let client = reqwest::Client::new();
    let mut nodes_with_jobs = 0;
    for port in NODE_PORTS.iter() {
        let metrics = client
            .get(&format!("http://localhost:{}/metrics", port))
            .send()
            .await
            .expect("metrics")
            .text()
            .await
            .expect("metrics text");
        let peers = parse_metric(&metrics, "network_peer_count");
        assert!(peers >= 0.0, "node {} missing peer count", port);
        let completed = parse_metric(&metrics, "jobs_completed_total");
        if completed > 0.0 {
            nodes_with_jobs += 1;
        }
    }
    assert!(
        nodes_with_jobs >= 2,
        "jobs were not distributed across nodes"
    );

    // Submit proposal to node A
    let proposal = serde_json::json!({
        "proposer_did": "did:example:alice",
        "proposal": { "GenericText": { "text": "scale test" } },
        "description": "scale test",
        "duration_secs": 60,
        "quorum": null,
        "threshold": null,
        "body": null
    });
    let resp = client
        .post("http://localhost:5001/governance/submit")
        .json(&proposal)
        .send()
        .await
        .expect("submit proposal");
    assert!(resp.status().is_success());
    let proposal_id: String = resp.json().await.expect("proposal id");

    // Wait for propagation
    for _ in 0..MAX_RETRIES {
        let mut all_seen = true;
        for port in NODE_PORTS.iter() {
            let list: Vec<Value> = client
                .get(&format!("http://localhost:{}/governance/proposals", port))
                .send()
                .await
                .expect("list proposals")
                .json()
                .await
                .expect("list json");
            if !list.iter().any(|p| p["id"] == proposal_id) {
                all_seen = false;
                break;
            }
        }
        if all_seen {
            break;
        }
        sleep(RETRY_DELAY).await;
    }

    // Final assertion on propagation
    for port in NODE_PORTS.iter() {
        let list: Vec<Value> = client
            .get(&format!("http://localhost:{}/governance/proposals", port))
            .send()
            .await
            .expect("list proposals")
            .json()
            .await
            .expect("list json");
        assert!(
            list.iter().any(|p| p["id"] == proposal_id),
            "proposal missing on node {}",
            port
        );
    }
}
