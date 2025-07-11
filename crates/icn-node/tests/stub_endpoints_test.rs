use icn_node::app_router_with_options;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::test]
async fn test_stub_endpoints_full_lifecycle() {
    println!("Testing ICN Stub Endpoints");

    // Create router without API key (None) to disable authentication
    let (router, _ctx) = app_router_with_options(
        None, // No API key required
        None, // No auth token required
        None, // No rate limit
        None, None, None, None, None, None, None,
    )
    .await;

    // Start server on localhost
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let server = task::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = Client::new();
    let base_url = format!("http://{}", addr);

    println!("Server started on {}", base_url);

    // Test 1: Submit a job
    println!("\n1. Testing job submission...");
    let job_response = client
        .post(&format!("{}/mesh/submit", base_url))
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

    assert!(job_response.status().is_success(), "Job submission should succeed");
    
    let job_data: serde_json::Value = job_response.json().await.unwrap();
    println!("✅ Job submitted: {}", job_data);
    
    let job_id = job_data.get("job_id").and_then(|v| v.as_str()).unwrap();

    // Test 2: Inject a bid
    println!("\n2. Testing stub bid injection...");
    let bid_response = client
        .post(&format!("{}/mesh/stub/bid", base_url))
        .json(&json!({
            "job_id": job_id,
            "executor_id": "test-executor",
            "estimated_cost": 5
        }))
        .send()
        .await
        .unwrap();

    println!("Bid response status: {}", bid_response.status());
    if !bid_response.status().is_success() {
        let error_text = bid_response.text().await.unwrap();
        println!("Bid injection failed: {}", error_text);
    } else {
        let bid_data: serde_json::Value = bid_response.json().await.unwrap();
        println!("✅ Bid injected: {}", bid_data);
    }

    // Test 3: Inject a receipt
    println!("\n3. Testing stub receipt injection...");
    let receipt_response = client
        .post(&format!("{}/mesh/stub/receipt", base_url))
        .json(&json!({
            "job_id": job_id,
            "executor_id": "test-executor",
            "result": {
                "status": "Success",
                "outputs": {
                    "result": "Echo complete: testing"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    println!("Receipt response status: {}", receipt_response.status());
    if !receipt_response.status().is_success() {
        let error_text = receipt_response.text().await.unwrap();
        println!("Receipt injection failed: {}", error_text);
    } else {
        let receipt_data: serde_json::Value = receipt_response.json().await.unwrap();
        println!("✅ Receipt injected: {}", receipt_data);
    }

    // Test 4: Check job status
    println!("\n4. Checking job status...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let status_response = client
        .get(&format!("{}/mesh/jobs/{}", base_url, job_id))
        .send()
        .await
        .unwrap();

    if status_response.status().is_success() {
        let status_data: serde_json::Value = status_response.json().await.unwrap();
        println!("✅ Final job status: {}", status_data);
    } else {
        let error_text = status_response.text().await.unwrap();
        println!("❌ Status check failed: {}", error_text);
    }

    server.abort();
    println!("\n=== Test Complete ===");
} 