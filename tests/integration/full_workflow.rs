use icn_node::app_router_with_options;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::test]
async fn workflow_via_router() {
    let (router, _ctx) = app_router_with_options(
        None, None, None, None, None, None, None, None, None, None,
    )
    .await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });

    let client = Client::new();
    let base = format!("http://{}", addr);

    let job_response = client
        .post(format!("{}/mesh/submit", base))
        .json(&json!({
            "manifest_cid": "bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e",
            "spec_json": {
                "kind": { "Echo": { "payload": "testing" } },
                "inputs": [],
                "outputs": ["result"],
                "required_resources": { "cpu_cores": 1, "memory_mb": 128 }
            },
            "cost_mana": 10
        }))
        .send()
        .await
        .unwrap();
    assert!(job_response.status().is_success());
    let job_data: serde_json::Value = job_response.json().await.unwrap();
    let job_id = job_data.get("job_id").and_then(|v| v.as_str()).unwrap();

    let resp = client
        .post(format!("{}/mesh/stub/bid", base))
        .json(&json!({"job_id": job_id, "executor_id": "did:example:exec", "estimated_cost": 5}))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .post(format!("{}/mesh/stub/receipt", base))
        .json(&json!({
            "job_id": job_id,
            "executor_id": "did:example:exec",
            "result": {"status": "Success", "outputs": {"result": "ok"}}
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let status_resp = client
        .get(format!("{}/mesh/jobs/{}", base, job_id))
        .send()
        .await
        .unwrap();
    assert!(status_resp.status().is_success());

    server.abort();
}

