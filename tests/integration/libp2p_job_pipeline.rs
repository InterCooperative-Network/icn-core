#[path = "federation.rs"]
mod federation;

#[cfg(feature = "enable-libp2p")]
mod libp2p_job_pipeline {
    use super::federation::{ensure_devnet, NODE_A_URL, NODE_B_URL, NODE_C_URL};
    use reqwest::Client;
    use serde_json::Value;
    use tokio::time::{sleep, Duration};

    const RETRY_DELAY: Duration = Duration::from_secs(3);
    const MAX_RETRIES: u32 = 20;

    async fn extract_did(client: &Client, url: &str) -> String {
        let info: Value = client
            .get(&format!("{}/info", url))
            .send()
            .await
            .expect("info request")
            .json()
            .await
            .expect("info json");
        let status = info["status_message"].as_str().unwrap_or("");
        status
            .trim_start_matches("Node DID: ")
            .split(',')
            .next()
            .unwrap_or("")
            .to_string()
    }

    #[tokio::test]
    async fn job_exec_across_nodes() {
        let _devnet = ensure_devnet().await;
        let client = Client::new();

        let node_a_did = extract_did(&client, NODE_A_URL).await;
        let node_b_did = extract_did(&client, NODE_B_URL).await;
        let node_c_did = extract_did(&client, NODE_C_URL).await;

        let job_request = serde_json::json!({
            "manifest_cid": "cidv1-libp2p-test-manifest",
            "spec_json": { "Echo": { "payload": "libp2p pipeline test" } },
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

        let mut final_status: Value = Value::Null;
        for _ in 0..MAX_RETRIES {
            let resp = client
                .get(&format!("{}/mesh/jobs/{}", NODE_A_URL, job_id))
                .send()
                .await
                .expect("status");
            if resp.status().is_success() {
                final_status = resp.json().await.expect("status json");
                if final_status["status"]["status"] == "completed" {
                    break;
                }
            }
            sleep(RETRY_DELAY).await;
        }

        assert_eq!(final_status["status"]["status"], "completed");
        let executor = final_status["status"]["executor"]
            .as_str()
            .expect("executor");
        assert_ne!(executor, node_a_did);

        let executor_url = if executor == node_b_did {
            NODE_B_URL
        } else if executor == node_c_did {
            NODE_C_URL
        } else {
            panic!("executor DID not recognized: {}", executor);
        };

        let exec_status: Value = client
            .get(&format!("{}/mesh/jobs/{}", executor_url, job_id))
            .send()
            .await
            .expect("executor status")
            .json()
            .await
            .expect("executor status json");
        assert_eq!(exec_status["status"]["status"], "completed");

        let result_cid = exec_status["status"]["result_cid"]
            .as_str()
            .expect("result_cid");

        // Ensure all nodes report the job as completed
        for url in [NODE_A_URL, NODE_B_URL, NODE_C_URL] {
            let status: Value = client
                .get(&format!("{}/mesh/jobs/{}", url, job_id))
                .send()
                .await
                .expect("status check")
                .json()
                .await
                .expect("status json");
            assert_eq!(status["status"]["status"], "completed", "status mismatch on {}", url);
        }

        // Verify the execution receipt propagates to all nodes via DAG fetch
        for url in [NODE_A_URL, NODE_B_URL, NODE_C_URL] {
            let dag_res = client
                .post(&format!("{}/dag/get", url))
                .json(&serde_json::json!({ "cid": result_cid }))
                .send()
                .await
                .expect("dag get");
            assert!(dag_res.status().is_success(), "dag get failed for {}", url);
        }
    }
}

#[cfg(not(feature = "enable-libp2p"))]
#[tokio::test]
async fn libp2p_feature_disabled_stub() {
    println!("libp2p feature disabled; skipping libp2p job pipeline test");
}

