use icn_common::{Did, Cid};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubDagStore, StubSigner};
use icn_mesh::{JobId, ActualMeshJob, MeshJobBid, JobState, JobSpec, Resources};
use anyhow;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

fn create_test_mesh_job(manifest_cid: Cid, cost_mana: u64, creator_did: Did) -> ActualMeshJob {
    ActualMeshJob {
        id: JobId::new_v1_dummy(0x55, 0x13, b"test_job_id"),
        manifest_cid,
        spec: JobSpec::GenericPlaceholder,
        creator_did,
        cost_mana,
        signature: SignatureBytes(vec![0u8; 64]),
    }
}

#[tokio::test]
async fn test_mesh_job_bidding_basic_logic() -> Result<(), anyhow::Error> {
    println!("🔧 [TEST] Testing basic mesh job bidding logic");
    
    // Test job creation
    let submitter_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")?;
    let executor_did = Did::from_str("did:key:z6MkrJVnaZjsXaHdNBKAZBmMfhVKYY6BQp3RfAuRgBCVq1234")?;
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest");
    let job_cost = 50u64;
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    
    println!("✅ [TEST] Created test job with ID: {:?}", test_job.id);
    
    // Test bid creation
    let bid = MeshJobBid {
        job_id: test_job.id.clone(),
        executor_did: executor_did.clone(),
        price_mana: job_cost / 2,
        resources: Resources::default(),
    };
    
    assert_eq!(bid.job_id, test_job.id);
    assert_eq!(bid.executor_did, executor_did);
    assert_eq!(bid.price_mana, job_cost / 2);
    
    println!("✅ [TEST] Created valid bid: executor={:?}, price={} mana", 
             bid.executor_did, bid.price_mana);
    
    // Test receipt creation
    let receipt = IdentityExecutionReceipt {
        job_id: test_job.id.clone(),
        executor_did: executor_did.clone(),
        result_cid: Cid::new_v1_dummy(0x55, 0x13, b"test_result"),
        cpu_ms: 2000,
        sig: SignatureBytes(vec![1u8; 64]), // Non-empty signature
    };
    
    assert_eq!(receipt.job_id, test_job.id);
    assert_eq!(receipt.executor_did, executor_did);
    assert!(receipt.cpu_ms > 0);
    assert!(!receipt.sig.0.is_empty());
    
    println!("✅ [TEST] Created valid receipt: executor={:?}, result_cid={:?}, cpu_ms={}", 
             receipt.executor_did, receipt.result_cid, receipt.cpu_ms);
    
    // Test bid selection logic (basic functionality from icn-mesh)
    let bids = vec![bid.clone()];
    let selected = icn_mesh::select_executor(&bids);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap(), executor_did);
    
    println!("✅ [TEST] Bid selection works correctly");
    
    println!("✅ [TEST] Basic mesh job bidding logic test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_multi_node_mesh_job_bidding_and_execution() -> Result<(), anyhow::Error> {
    println!("🔧 [TEST] Starting multi-node mesh job bidding and execution test");
    
    // Create submitter node (Node A)
    let submitter_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")?;
    let mesh_service_a = Arc::new(StubMeshNetworkService::new());
    let signer_a = Arc::new(StubSigner::new());
    let dag_store_a = Arc::new(StubDagStore::new());
    
    let ctx_a = RuntimeContext::new(
        submitter_did.clone(),
        mesh_service_a.clone(),
        signer_a,
        dag_store_a,
    );
    ctx_a.credit_mana(&submitter_did, 200).await?;
    
    // Create executor node (Node B)
    let executor_did = Did::from_str("did:key:z6MkrJVnaZjsXaHdNBKAZBmMfhVKYY6BQp3RfAuRgBCVq1234")?;
    let mesh_service_b = Arc::new(StubMeshNetworkService::new());
    let signer_b = Arc::new(StubSigner::new());
    let dag_store_b = Arc::new(StubDagStore::new());
    
    let ctx_b = RuntimeContext::new(
        executor_did.clone(),
        mesh_service_b.clone(),
        signer_b,
        dag_store_b,
    );
    ctx_b.credit_mana(&executor_did, 100).await?;
    
    // Create a test job
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest");
    let job_cost = 50u64;
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    
    println!("🔧 [TEST] Created test job with ID: {:?}", test_job.id);
    
    // Test the bidding logic
    println!("🔧 [TEST] Testing bidding logic");
    let should_bid = ctx_b.should_bid_on_job(&test_job).await;
    assert!(should_bid, "Executor should bid on the job (has sufficient mana)");
    
    // Test bid submission
    println!("🔧 [TEST] Testing bid submission");
    ctx_b.submit_bid_for_job(&test_job).await?;
    
    // Verify the bid was staged in the network service
    let staged_bids = mesh_service_b.get_staged_bids_for_job(&test_job.id);
    assert_eq!(staged_bids.len(), 1, "Should have one staged bid");
    
    let bid = &staged_bids[0];
    assert_eq!(bid.executor_did, executor_did);
    assert_eq!(bid.job_id, test_job.id);
    assert_eq!(bid.price_mana, job_cost / 2); // Our pricing strategy is half the job cost
    
    println!("✅ [TEST] Bid submitted successfully: executor={:?}, price={} mana", 
             bid.executor_did, bid.price_mana);
    
    // Test job execution
    println!("🔧 [TEST] Testing job execution");
    ctx_b.execute_assigned_job(&test_job.id).await?;
    
    // Verify the receipt was submitted
    let staged_receipts = mesh_service_b.get_staged_receipts_for_job(&test_job.id);
    assert_eq!(staged_receipts.len(), 1, "Should have one staged receipt");
    
    let receipt = &staged_receipts[0];
    assert_eq!(receipt.executor_did, executor_did);
    assert_eq!(receipt.job_id, test_job.id);
    assert!(receipt.cpu_ms > 0, "Receipt should have execution time");
    assert!(!receipt.sig.0.is_empty(), "Receipt should be signed");
    
    println!("✅ [TEST] Job executed successfully: executor={:?}, result_cid={:?}, cpu_ms={}", 
             receipt.executor_did, receipt.result_cid, receipt.cpu_ms);
    
    // Test that executor doesn't bid on their own jobs
    println!("🔧 [TEST] Testing self-job bidding prevention");
    let self_job = create_test_mesh_job(manifest_cid.clone(), job_cost, executor_did.clone());
    let should_bid_on_self = ctx_b.should_bid_on_job(&self_job).await;
    assert!(!should_bid_on_self, "Executor should not bid on their own job");
    
    // Test insufficient mana scenario
    println!("🔧 [TEST] Testing insufficient mana scenario");
    let expensive_job = create_test_mesh_job(manifest_cid.clone(), 200, submitter_did.clone());
    let should_bid_expensive = ctx_b.should_bid_on_job(&expensive_job).await;
    assert!(!should_bid_expensive, "Executor should not bid on job that costs more than their mana");
    
    println!("✅ [TEST] Multi-node mesh job bidding and execution test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_job_manager_lifecycle() -> Result<(), anyhow::Error> {
    println!("🔧 [TEST] Starting job manager lifecycle test");
    
    // Create submitter node
    let submitter_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")?;
    let mesh_service = Arc::new(StubMeshNetworkService::new());
    let signer = Arc::new(StubSigner::new());
    let dag_store = Arc::new(StubDagStore::new());
    
    let ctx = RuntimeContext::new(
        submitter_did.clone(),
        mesh_service.clone(),
        signer,
        dag_store,
    );
    ctx.credit_mana(&submitter_did, 200).await?;
    
    // Create executor and stage a bid
    let executor_did = Did::from_str("did:key:z6MkrJVnaZjsXaHdNBKAZBmMfhVKYY6BQp3RfAuRgBCVq1234")?;
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest");
    let job_cost = 50u64;
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    
    // Stage a bid for the job
    let bid = MeshJobBid {
        job_id: test_job.id.clone(),
        executor_did: executor_did.clone(),
        price_mana: job_cost / 2,
        resources: Resources::default(),
    };
    mesh_service.stage_bid_for_job(test_job.id.clone(), bid);
    
    // Stage a receipt for the job
    let receipt = IdentityExecutionReceipt {
        job_id: test_job.id.clone(),
        executor_did: executor_did.clone(),
        result_cid: Cid::new_v1_dummy(0x55, 0x13, b"test_result"),
        cpu_ms: 2000,
        sig: SignatureBytes(vec![1u8; 64]), // Non-empty signature
    };
    mesh_service.stage_receipt_for_job(test_job.id.clone(), receipt);
    
    // Submit the job
    println!("🔧 [TEST] Submitting job to mesh network");
    ctx.internal_queue_mesh_job(test_job.clone()).await?;
    
    // Process the job manually (simulating job manager)
    println!("🔧 [TEST] Processing pending jobs");
    ctx.process_pending_jobs().await?;
    
    // Check that the job was processed and completed
    let job_states = ctx.job_states.lock().await;
    let job_state = job_states.get(&test_job.id).expect("Job should exist");
    
    match job_state {
        JobState::Completed { receipt } => {
            println!("✅ [TEST] Job completed successfully: executor={:?}, result_cid={:?}", 
                     receipt.executor_did, receipt.result_cid);
            assert_eq!(receipt.executor_did, executor_did);
            assert_eq!(receipt.job_id, test_job.id);
        }
        other_state => {
            return Err(anyhow::anyhow!("Expected job to be completed, but got: {:?}", other_state));
        }
    }
    
    // Verify mana was charged
    let final_mana = ctx.get_mana(&submitter_did).await?;
    assert_eq!(final_mana, 200 - job_cost, "Submitter mana should be reduced by job cost");
    
    println!("✅ [TEST] Job manager lifecycle test completed successfully!");
    Ok(())
} 