#[path = "federation.rs"]
mod federation;

use base64;
use bincode;
use federation::{ensure_devnet, wait_for_federation_ready, NODE_A_URL, NODE_C_URL};
use icn_common::retry_with_backoff;
use icn_mesh::{JobKind, JobSpec};
use reqwest::Client;
use serde_json::Value;
use std::process::Command;
use std::time::Instant;
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
    let spec = icn_mesh::JobSpec {
        kind: icn_mesh::JobKind::Echo {
            payload: "Network resilience test".into(),
        },
        ..Default::default()
    };
    let job_request = serde_json::json!({
        "manifest_cid": "cidv1-resilience-test-manifest",
        "spec_bytes": base64::encode(bincode::serialize(&spec).unwrap()),
        "spec_json": null,
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

    assert!(
        node_c_synced,
        "Node C did not sync job receipt after reconnect"
    );
}

struct TestCircuitBreaker {
    failures: u32,
    threshold: u32,
    open_until: Option<Instant>,
    timeout: Duration,
}

impl TestCircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failures: 0,
            threshold,
            open_until: None,
            timeout,
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.open_until, Some(t) if t > Instant::now())
    }

    fn reset_if_needed(&mut self) {
        if let Some(t) = self.open_until {
            if Instant::now() >= t {
                self.open_until = None;
                self.failures = 0;
            }
        }
    }

    async fn call<F, Fut>(&mut self, op: F) -> Result<(), &'static str>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), reqwest::Error>>,
    {
        self.reset_if_needed();
        if self.is_open() {
            return Err("circuit open");
        }
        match op().await {
            Ok(_) => {
                self.failures = 0;
                Ok(())
            }
            Err(_) => {
                self.failures += 1;
                if self.failures >= self.threshold {
                    self.open_until = Some(Instant::now() + self.timeout);
                }
                Err("operation failed")
            }
        }
    }
}

#[tokio::test]
async fn test_long_partition_circuit_breaker() {
    let _devnet = ensure_devnet().await;

    Command::new("docker-compose")
        .args(&["-f", "icn-devnet/docker-compose.yml", "stop", "icn-node-c"])
        .status()
        .expect("failed to stop node c");

    sleep(Duration::from_secs(5)).await;

    let client = Client::new();
    let mut breaker = TestCircuitBreaker::new(3, Duration::from_secs(5));
    let url = format!("{}/info", NODE_C_URL);
    let mut attempt_times: Vec<Instant> = Vec::new();

    for _ in 0..3 {
        let _ = breaker
            .call(|| {
                let c = &client;
                let u = &url;
                attempt_times.push(Instant::now());
                async move {
                    retry_with_backoff(
                        || async { c.get(u).send().await.map(|_| ()) },
                        3,
                        Duration::from_millis(100),
                        Duration::from_secs(1),
                    )
                    .await
                }
            })
            .await;
        sleep(Duration::from_secs(1)).await;
    }

    assert!(
        breaker.is_open(),
        "circuit breaker should open after failures"
    );

    let before = attempt_times.len();
    let err = breaker
        .call(|| async { Ok(()) })
        .await
        .expect_err("expected circuit open error");
    assert_eq!(err, "circuit open");
    assert_eq!(
        before,
        attempt_times.len(),
        "no request made when circuit open"
    );

    Command::new("docker-compose")
        .args(&["-f", "icn-devnet/docker-compose.yml", "start", "icn-node-c"])
        .status()
        .expect("failed to start node c");

    wait_for_federation_ready()
        .await
        .expect("federation not ready after restart");

    sleep(Duration::from_secs(6)).await;

    breaker
        .call(|| {
            let c = &client;
            let u = &url;
            attempt_times.push(Instant::now());
            async move {
                retry_with_backoff(
                    || async { c.get(u).send().await.map(|_| ()) },
                    3,
                    Duration::from_millis(100),
                    Duration::from_secs(1),
                )
                .await
            }
        })
        .await
        .expect("operation should succeed after reconnect");

    assert!(
        !breaker.is_open(),
        "circuit breaker should reset after success"
    );

    assert!(attempt_times.len() >= 3);
    if attempt_times.len() >= 3 {
        let interval1 = attempt_times[1] - attempt_times[0];
        let interval2 = attempt_times[2] - attempt_times[1];
        assert!(interval2 >= interval1);
    }
}

#[tokio::test]
async fn test_stub_network_breaker_open_close() {
    use icn_common::Did;
    use icn_network::{PeerId, StubNetworkService};
    use icn_protocol::{GossipMessage, MessagePayload, ProtocolMessage};
    use std::time::Duration;
    use tokio::time::sleep;

    let service = StubNetworkService::default();
    let msg = ProtocolMessage::new(
        MessagePayload::GossipMessage(GossipMessage {
            topic: "cb_test".to_string(),
            payload: vec![],
            ttl: 1,
        }),
        Did::new("key", "stub"),
        None,
    );

    for _ in 0..3 {
        let _ = service
            .send_message(&PeerId("error_peer".into()), msg.clone())
            .await;
    }

    let err = service
        .send_message(&PeerId("error_peer".into()), msg.clone())
        .await
        .expect_err("breaker should be open");
    assert!(matches!(err, icn_network::MeshNetworkError::Timeout(_)));

    sleep(Duration::from_secs(5)).await;

    service
        .send_message(&PeerId("ok_peer".into()), msg.clone())
        .await
        .expect("breaker should allow after timeout");

    let err2 = service
        .send_message(&PeerId("error_peer".into()), msg)
        .await
        .expect_err("expected send failure");
    assert!(!matches!(err2, icn_network::MeshNetworkError::Timeout(_)));
}
