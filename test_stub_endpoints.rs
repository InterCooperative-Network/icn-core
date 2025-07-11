use icn_node::app_router_with_options;
use reqwest::Client;
use serde_json::json;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    
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
        .await?;

    if job_response.status().is_success() {
        let job_data: serde_json::Value = job_response.json().await?;
        println!("✅ Job submitted: {}", job_data);
        
        if let Some(job_id) = job_data.get("job_id").and_then(|v| v.as_str()) {
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
                .await?;

            println!("Bid response status: {}", bid_response.status());
            if bid_response.status().is_success() {
                let bid_data: serde_json::Value = bid_response.json().await?;
                println!("✅ Bid injected: {}", bid_data);
            } else {
                println!("❌ Bid injection failed: {}", bid_response.text().await?);
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
                .await?;

            println!("Receipt response status: {}", receipt_response.status());
            if receipt_response.status().is_success() {
                let receipt_data: serde_json::Value = receipt_response.json().await?;
                println!("✅ Receipt injected: {}", receipt_data);
            } else {
                println!("❌ Receipt injection failed: {}", receipt_response.text().await?);
            }

            // Test 4: Check job status
            println!("\n4. Checking job status...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            let status_response = client
                .get(&format!("{}/mesh/jobs/{}", base_url, job_id))
                .send()
                .await?;

            if status_response.status().is_success() {
                let status_data: serde_json::Value = status_response.json().await?;
                println!("✅ Final job status: {}", status_data);
            } else {
                println!("❌ Status check failed: {}", status_response.text().await?);
            }
        }
    } else {
        println!("❌ Job submission failed: {}", job_response.text().await?);
    }

    server.abort();
    println!("\n=== Test Complete ===");
    Ok(())
} 