//! Basic usage example for the ICN SDK
//!
//! This example demonstrates common operations with the ICN SDK,
//! including connecting to a node, checking health, and submitting jobs.

use icn_sdk::IcnClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client pointing to a local ICN node
    let client = IcnClient::new("http://localhost:8080")?;
    
    println!("🚀 ICN SDK Basic Usage Example");
    println!("================================");
    
    // 1. Get node information
    println!("\n📋 Getting node information...");
    match client.info().await {
        Ok(info) => {
            println!("✅ Node: {} v{}", info.name, info.version);
            println!("   Status: {}", info.status_message);
        }
        Err(e) => {
            println!("❌ Failed to get node info: {}", e);
            println!("   Make sure an ICN node is running on localhost:8080");
            return Ok(());
        }
    }
    
    // 2. Check node health
    println!("\n🏥 Checking node health...");
    match client.health().await {
        Ok(health) => {
            println!("✅ Health: {}", health.status);
            println!("   Uptime: {}s", health.uptime_seconds);
            println!("   Runtime: {}", health.checks.runtime);
            println!("   DAG Store: {}", health.checks.dag_store);
            println!("   Network: {}", health.checks.network);
            println!("   Mana Ledger: {}", health.checks.mana_ledger);
        }
        Err(e) => {
            println!("⚠️  Failed to get health status: {}", e);
        }
    }
    
    // 3. Check readiness
    println!("\n🔍 Checking node readiness...");
    match client.ready().await {
        Ok(ready) => {
            if ready.ready {
                println!("✅ Node is ready to serve requests");
            } else {
                println!("⚠️  Node is not ready");
            }
            println!("   Can serve requests: {}", ready.checks.can_serve_requests);
            println!("   Mana ledger available: {}", ready.checks.mana_ledger_available);
            println!("   DAG store available: {}", ready.checks.dag_store_available);
            println!("   Network initialized: {}", ready.checks.network_initialized);
        }
        Err(e) => {
            println!("⚠️  Failed to check readiness: {}", e);
        }
    }
    
    // 4. List existing mesh jobs
    println!("\n📝 Listing existing mesh jobs...");
    match client.list_mesh_jobs().await {
        Ok(jobs) => {
            if let Some(jobs_array) = jobs.get("jobs") {
                if let Some(jobs) = jobs_array.as_array() {
                    println!("✅ Found {} job(s)", jobs.len());
                    for (i, job) in jobs.iter().enumerate() {
                        if let (Some(id), Some(status)) = (job.get("id"), job.get("status")) {
                            println!("   {}. Job {}: {}", i + 1, id, status);
                        }
                    }
                } else {
                    println!("✅ No jobs found");
                }
            } else {
                println!("✅ Jobs response: {}", jobs);
            }
        }
        Err(e) => {
            println!("⚠️  Failed to list jobs: {}", e);
        }
    }
    
    // 5. Submit a simple mesh job
    println!("\n🚀 Submitting a simple mesh job...");
    let job_request = json!({
        "manifest_cid": "bafybeigdyrzt5samplecid",
        "spec_bytes": "example_spec_bytes",
        "spec_json": null,
        "cost_mana": 50
    });
    
    match client.submit_mesh_job(&job_request).await {
        Ok(response) => {
            println!("✅ Job submitted successfully!");
            if let Some(job_id) = response.get("job_id") {
                println!("   Job ID: {}", job_id);
                
                // 6. Check the job status
                println!("\n🔍 Checking job status...");
                if let Some(job_id_str) = job_id.as_str() {
                    match client.mesh_job(job_id_str).await {
                        Ok(job_status) => {
                            println!("✅ Job status: {}", job_status.get("status").unwrap_or(&json!("unknown")));
                            println!("   Full job details: {}", job_status);
                        }
                        Err(e) => {
                            println!("⚠️  Failed to get job status: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️  Failed to submit job: {}", e);
        }
    }
    
    // 7. List governance proposals
    println!("\n🏛️  Listing governance proposals...");
    match client.list_proposals().await {
        Ok(proposals) => {
            if let Some(proposals_array) = proposals.get("proposals") {
                if let Some(proposals) = proposals_array.as_array() {
                    println!("✅ Found {} proposal(s)", proposals.len());
                    for (i, proposal) in proposals.iter().enumerate() {
                        if let (Some(id), Some(title)) = (proposal.get("id"), proposal.get("title")) {
                            println!("   {}. Proposal {}: {}", i + 1, id, title);
                        }
                    }
                } else {
                    println!("✅ No proposals found");
                }
            } else {
                println!("✅ Proposals response: {}", proposals);
            }
        }
        Err(e) => {
            println!("⚠️  Failed to list proposals: {}", e);
        }
    }
    
    // 8. Check mana balance for a sample DID
    println!("\n💰 Checking mana balance...");
    match client.account_mana("did:key:sample").await {
        Ok(mana) => {
            println!("✅ Mana info: {}", mana);
        }
        Err(e) => {
            println!("⚠️  Failed to get mana balance: {}", e);
        }
    }
    
    // 9. Get network peers
    println!("\n🌐 Getting network peers...");
    match client.peers().await {
        Ok(peers) => {
            println!("✅ Network peers: {}", peers);
        }
        Err(e) => {
            println!("⚠️  Failed to get peers: {}", e);
        }
    }
    
    // 10. Get federation status
    println!("\n🤝 Getting federation status...");
    match client.federation_status().await {
        Ok(status) => {
            println!("✅ Federation status: {}", status);
        }
        Err(e) => {
            println!("⚠️  Failed to get federation status: {}", e);
        }
    }
    
    // 11. Get Prometheus metrics
    println!("\n📊 Getting Prometheus metrics...");
    match client.metrics().await {
        Ok(metrics) => {
            let lines: Vec<&str> = metrics.lines().take(10).collect();
            println!("✅ Metrics (first 10 lines):");
            for line in lines {
                println!("   {}", line);
            }
            if metrics.lines().count() > 10 {
                println!("   ... and {} more lines", metrics.lines().count() - 10);
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get metrics: {}", e);
        }
    }
    
    println!("\n🎉 Example completed!");
    println!("   This example demonstrated basic ICN SDK operations.");
    println!("   For more advanced usage, check the documentation and other examples.");
    
    Ok(())
} 