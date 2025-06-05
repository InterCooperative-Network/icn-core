use std::time::Duration;
use tokio::time::sleep;
use serde_json::Value;

// Integration test for ICN federation devnet
// This test assumes the federation is running via docker-compose

const NODE_A_URL: &str = "http://localhost:5001";
const NODE_B_URL: &str = "http://localhost:5002"; 
const NODE_C_URL: &str = "http://localhost:5003";
const MAX_RETRIES: u32 = 20;
const RETRY_DELAY: Duration = Duration::from_secs(3);

#[tokio::test]
#[ignore] // Run with --ignored since it requires running federation
async fn test_federation_node_health() {
    println!("🏥 Testing federation node health...");
    
    let client = reqwest::Client::new();
    
    // Test each node's info endpoint
    for (name, url) in [
        ("Node A", NODE_A_URL),
        ("Node B", NODE_B_URL), 
        ("Node C", NODE_C_URL)
    ] {
        println!("  Testing {} at {}", name, url);
        
        let response = client
            .get(&format!("{}/info", url))
            .send()
            .await
            .expect("Failed to connect to node");
            
        assert_eq!(response.status(), 200);
        
        let info: Value = response.json().await.expect("Failed to parse JSON");
        assert!(info["name"].is_string());
        assert!(info["version"].is_string());
        
        println!("    ✅ {} is healthy", name);
    }
    
    println!("✅ All nodes are healthy");
}

#[tokio::test]
#[ignore] // Run with --ignored since it requires running federation
async fn test_federation_p2p_convergence() {
    println!("🌐 Testing P2P network convergence...");
    
    let client = reqwest::Client::new();
    
    // Wait for network convergence
    for attempt in 1..=MAX_RETRIES {
        println!("  Convergence check attempt {}/{}", attempt, MAX_RETRIES);
        
        let mut all_connected = true;
        let mut peer_counts = Vec::new();
        
        for (name, url) in [
            ("Node A", NODE_A_URL),
            ("Node B", NODE_B_URL),
            ("Node C", NODE_C_URL)
        ] {
            let response = client
                .get(&format!("{}/status", url))
                .send()
                .await
                .expect("Failed to get node status");
                
            let status: Value = response.json().await.expect("Failed to parse status JSON");
            let peer_count = status["peer_count"].as_u64().unwrap_or(0);
            
            peer_counts.push((name, peer_count));
            
            // Each node should have at least 1 peer (ideally 2 for full mesh)
            if peer_count == 0 {
                all_connected = false;
            }
        }
        
        println!("    Peer counts: {:?}", peer_counts);
        
        if all_connected {
            println!("✅ P2P network has converged");
            return;
        }
        
        if attempt < MAX_RETRIES {
            sleep(RETRY_DELAY).await;
        }
    }
    
    panic!("❌ P2P network failed to converge within {} attempts", MAX_RETRIES);
}

#[tokio::test]
#[ignore] // Run with --ignored since it requires running federation
async fn test_federation_mesh_job_lifecycle() {
    println!("🚀 Testing mesh job lifecycle across federation...");
    
    let client = reqwest::Client::new();
    
    // Submit job to Node A
    println!("  📤 Submitting mesh job to Node A...");
    
    let job_request = serde_json::json!({
        "manifest_cid": "cidv1-85-20-integration_test_manifest",
        "spec_json": { "Echo": { "payload": "Federation integration test!" } },
        "cost_mana": 150
    });
    
    let submit_response = client
        .post(&format!("{}/mesh/submit", NODE_A_URL))
        .header("Content-Type", "application/json")
        .json(&job_request)
        .send()
        .await
        .expect("Failed to submit job");
        
    assert_eq!(submit_response.status(), 202);
    
    let submit_result: Value = submit_response.json().await.expect("Failed to parse submit response");
    let job_id = submit_result["job_id"]
        .as_str()
        .expect("No job_id in response")
        .to_string();
        
    println!("    ✅ Job submitted with ID: {}", job_id);
    
    // Check job status on Node A
    println!("  📊 Checking job status on Node A...");
    
    let mut job_found = false;
    for attempt in 1..=MAX_RETRIES {
        let status_response = client
            .get(&format!("{}/mesh/jobs/{}", NODE_A_URL, job_id))
            .send()
            .await
            .expect("Failed to get job status");
            
        if status_response.status() == 200 {
            let job_status: Value = status_response.json().await.expect("Failed to parse job status");
            println!("    📋 Job status: {:?}", job_status["status"]);
            job_found = true;
            break;
        }
        
        if attempt < MAX_RETRIES {
            sleep(RETRY_DELAY).await;
        }
    }
    
    assert!(job_found, "Job not found on Node A");
    println!("    ✅ Job found and tracked on Node A");
    
    // Check job appears in job listings across all nodes
    println!("  📋 Checking job visibility across all nodes...");
    
    for (name, url) in [
        ("Node A", NODE_A_URL),
        ("Node B", NODE_B_URL),
        ("Node C", NODE_C_URL)
    ] {
        let jobs_response = client
            .get(&format!("{}/mesh/jobs", url))
            .send()
            .await
            .expect("Failed to get jobs list");
            
        assert_eq!(jobs_response.status(), 200);
        
        let jobs_list: Value = jobs_response.json().await.expect("Failed to parse jobs list");
        let jobs_array = jobs_list["jobs"].as_array().expect("No jobs array");
        
        println!("    {} sees {} jobs", name, jobs_array.len());
        
        // For now, we just verify the endpoint works
        // In a full P2P implementation, jobs would propagate to all nodes
    }
    
    println!("✅ Mesh job lifecycle test completed");
}

#[tokio::test]
#[ignore] // Run with --ignored since it requires running federation
async fn test_federation_cross_node_api_consistency() {
    println!("🔄 Testing API consistency across federation nodes...");
    
    let client = reqwest::Client::new();
    
    // Test that all nodes expose the same API endpoints
    let endpoints = [
        "/info",
        "/status", 
        "/mesh/jobs",
        // Note: POST endpoints require data, so we only test GET endpoints here
    ];
    
    for endpoint in endpoints {
        println!("  Testing endpoint: {}", endpoint);
        
        for (name, url) in [
            ("Node A", NODE_A_URL),
            ("Node B", NODE_B_URL),
            ("Node C", NODE_C_URL)
        ] {
            let response = client
                .get(&format!("{}{}", url, endpoint))
                .send()
                .await
                .expect(&format!("Failed to reach {} on {}", endpoint, name));
                
            assert!(
                response.status().is_success(),
                "{} failed on {} with status: {}",
                endpoint, name, response.status()
            );
        }
        
        println!("    ✅ {} works on all nodes", endpoint);
    }
    
    println!("✅ API consistency verified across federation");
}

#[tokio::test]
#[ignore] // Run with --ignored since it requires running federation
async fn test_federation_complete_workflow() {
    println!("🎯 Testing complete federation workflow...");
    
    // Run all tests in sequence to validate the complete workflow
    test_federation_node_health().await;
    test_federation_p2p_convergence().await;
    test_federation_mesh_job_lifecycle().await;
    test_federation_cross_node_api_consistency().await;
    
    println!("🎉 Complete federation workflow test PASSED!");
}

// Helper function for manual testing
#[allow(dead_code)]
pub async fn wait_for_federation_ready() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏳ Waiting for federation to be ready...");
    
    let client = reqwest::Client::new();
    
    // Wait for all nodes to be healthy
    for attempt in 1..=MAX_RETRIES {
        let mut all_healthy = true;
        
        for (name, url) in [
            ("Node A", NODE_A_URL),
            ("Node B", NODE_B_URL),
            ("Node C", NODE_C_URL)
        ] {
            match client.get(&format!("{}/info", url)).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("  ✅ {} is ready", name);
                }
                _ => {
                    println!("  ⏳ {} not ready yet", name);
                    all_healthy = false;
                }
            }
        }
        
        if all_healthy {
            println!("🎉 Federation is ready!");
            return Ok(());
        }
        
        if attempt < MAX_RETRIES {
            sleep(RETRY_DELAY).await;
        }
    }
    
    Err("Federation failed to become ready".into())
} 