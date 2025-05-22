// crates/icn-runtime/tests/mesh.rs

use icn_common::{Did, Cid};
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt; // Use the alias consistently
use icn_mesh::{ActualMeshJob, JobId, JobSpec, JobState, MeshJobBid, Resources, SubmitReceiptMessage};
use icn_runtime::context::{RuntimeContext, HostAbiError, StubMeshNetworkService, StubSigner, StubDagStore, DagStore, JobState as ContextJobState};
use icn_runtime::host_submit_mesh_job;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


// Helper to create a RuntimeContext with a specific DID and initial mana.
// The Stub services are now part of RuntimeContext::new_with_stubs_and_mana
fn create_test_context(identity_did_str: &str, initial_mana: u64) -> Arc<RuntimeContext> {
    Arc::new(RuntimeContext::new_with_stubs_and_mana(identity_did_str, initial_mana))
}

// Helper to assert the state of a job
async fn assert_job_state(ctx: &Arc<RuntimeContext>, job_id: &JobId, expected_state_variant: JobStateVariant) {
    tokio::task::yield_now().await;
    sleep(Duration::from_millis(100)).await; // Increased delay slightly for job manager processing

    let states = ctx.job_states.lock().await;
    let job_state = states.get(job_id).unwrap_or_else(|| panic!("Job ID {:?} not found in states map. States: {:?}", job_id, states));

    match (job_state, &expected_state_variant) {
        (JobState::Pending, JobStateVariant::Pending) => {}
        (JobState::Assigned { executor }, JobStateVariant::Assigned { expected_executor }) => {
            if let Some(expected_exec_did) = expected_executor {
                assert_eq!(executor, expected_exec_did, "Job {:?} assigned to unexpected executor. Expected {:?}, got {:?}", job_id, expected_exec_did, executor);
            }
        }
        (JobState::Completed { receipt }, JobStateVariant::Completed { expected_receipt_data }) => {
            if let Some(data) = expected_receipt_data {
                assert_eq!(&receipt.job_id, &data.job_id, "Completed receipt job_id mismatch");
                assert_eq!(&receipt.executor_did, &data.executor_did, "Completed receipt executor_did mismatch");
                assert_eq!(&receipt.result_cid, &data.result_cid, "Completed receipt result_cid mismatch");
            }
        }
        (JobState::Failed { reason: _ }, JobStateVariant::Failed) => {}
        (actual, expected) => panic!("Job {:?} is in state {:?}, expected variant {:?}", job_id, actual, expected),
    }
}

// Simplified enum for asserting job state variants
#[derive(Debug, PartialEq, Clone)]
enum JobStateVariant {
    Pending,
    Assigned { expected_executor: Option<Did> },
    Completed { expected_receipt_data: Option<ExpectedReceiptData> },
    Failed,
}

#[derive(Debug, PartialEq, Clone)]
struct ExpectedReceiptData {
    job_id: JobId,
    executor_did: Did,
    result_cid: Cid,
}


// Helper to get the underlying StubMeshNetworkService from the RuntimeContext
fn get_stub_network_service(ctx: &Arc<RuntimeContext>) -> Arc<StubMeshNetworkService> {
    ctx.mesh_network_service
        .clone()
        .downcast_arc::<StubMeshNetworkService>()
        .expect("RuntimeContext in test was not initialized with StubMeshNetworkService")
}

fn get_stub_dag_store(ctx: &Arc<RuntimeContext>) -> Arc<StubDagStore> {
    ctx.dag_store
        .clone()
        .downcast_arc::<StubDagStore>()
        .expect("RuntimeContext in test was not initialized with StubDagStore")
}


#[tokio::test]
async fn test_mesh_job_full_lifecycle_happy_path() {
    let submitter_did_str = "did:icn:test:submitter_happy";
    let executor_did_str = "did:icn:test:executor_happy";
    
    let submitter_did = Did::from_str(submitter_did_str).unwrap();
    let executor_did = Did::from_str(executor_did_str).unwrap();

    // Context for the submitter
    let ctx_submitter = create_test_context(submitter_did_str, 100);
    // Context for the Job Manager node
    let arc_ctx_job_manager = create_test_context("did:icn:test:job_manager_node_happy", 0); 

    arc_ctx_job_manager.spawn_mesh_job_manager().await;
    
    let job_manager_network_stub = get_stub_network_service(&arc_ctx_job_manager);
    let job_manager_dag_store_stub = get_stub_dag_store(&arc_ctx_job_manager);


    // 1. SUBMISSION
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"manifest_happy");
    let job_cost = 20u64;
    let job_json_payload = json!({
        "manifest_cid": manifest_cid,
        "spec": {}, 
        "cost_mana": job_cost,
    }).to_string();

    let submitted_job_id = host_submit_mesh_job(&ctx_submitter, &job_json_payload)
        .await
        .expect("Job submission failed");

    assert_eq!(ctx_submitter.get_mana(&submitter_did).await.unwrap(), 100 - job_cost, "Submitter mana not deducted correctly");
    
    // Manually queue the job into the Job Manager's context as host_submit_mesh_job operates on its own context.
    let submitted_job_details = ActualMeshJob {
        id: submitted_job_id.clone(),
        manifest_cid: manifest_cid.clone(),
        spec: JobSpec::default(), // Ensure this matches what host_submit_mesh_job would create
        creator_did: submitter_did.clone(),
        cost_mana: job_cost,
    };
    arc_ctx_job_manager.internal_queue_mesh_job(submitted_job_details.clone()).await.unwrap();
    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Pending).await;


    // 2. BIDDING & ASSIGNMENT
    let bid = MeshJobBid {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: 10, 
        resources: Resources::default(),
    };
    job_manager_network_stub.stage_bid(submitted_job_id.clone(), bid).await;
    
    sleep(Duration::from_millis(1200)).await; // Allow time for announcement, bid collection, assignment

    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Assigned { expected_executor: Some(executor_did.clone()) }).await;

    // 3. EXECUTION & ANCHORING (RECEIPT SUBMISSION)
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"result_happy");
    let mut receipt_to_submit = IdentityExecutionReceipt { // Mut so anchor_receipt can fill sig
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(), 
        result_cid: result_cid.clone(),
        cpu_ms: 100,
        sig: Vec::new(), // Signature will be filled by anchor_receipt
    };
    
    // Simulate the executor's context anchoring its own receipt before sending
    // This is a bit simplified; in reality, the executor calls host_anchor_receipt.
    // For the job manager to receive a valid receipt, it must already be signed.
    // The job_manager_node itself (current_identity of arc_ctx_job_manager) cannot sign for executor_did.
    // Let's create a temporary context for the executor to sign.
    let ctx_executor_for_signing = create_test_context(executor_did_str, 0); // Mana not important here
    ctx_executor_for_signing.anchor_receipt(&mut receipt_to_submit).expect("Executor failed to co-sign its own receipt for testing");

    let receipt_msg = SubmitReceiptMessage { receipt: receipt_to_submit.clone() }; // now signed
    job_manager_network_stub.stage_receipt(receipt_msg).await;

    sleep(Duration::from_millis(1200)).await; 

    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Completed { 
        expected_receipt_data: Some(ExpectedReceiptData { 
            job_id: submitted_job_id.clone(), 
            executor_did: executor_did.clone(), 
            result_cid: result_cid.clone() 
        }) 
    }).await;
    
    // Assert DAG anchoring
    let final_receipt_bytes = serde_json::to_vec(&receipt_to_submit).expect("Failed to serialize final receipt for CID calculation");
    let mut hasher = DefaultHasher::new();
    final_receipt_bytes.hash(&mut hasher);
    let hash_val = hasher.finish();
    // StubDagStore uses a fixed codec and hash_alg for dummy CIDs
    let expected_cid = Cid::new_v1_dummy(0x70, 0x12, &hash_val.to_ne_bytes());
    
    let stored_data = job_manager_dag_store_stub.get(&expected_cid).expect("DAG get failed").expect("Receipt not found in DAG store by expected CID");
    assert_eq!(stored_data, final_receipt_bytes, "Stored DAG data does not match original receipt");

    // Reputation updater is called internally by job manager, stub just prints.
    println!("Happy path test completed and DAG anchoring verified.");
}


#[tokio::test(start_paused = true)]
async fn test_mesh_job_timeout_and_refund() {
    let submitter_did_str = "did:icn:test:submitter_timeout";
    let submitter_did = Did::from_str(submitter_did_str).unwrap();
    let initial_mana = 100u64;
    let job_cost = 30u64;

    let arc_ctx_job_manager = create_test_context("did:icn:test:job_manager_node_timeout", 0);
    arc_ctx_job_manager.spawn_mesh_job_manager().await;

    let ctx_submitter = create_test_context(submitter_did_str, initial_mana); // Submitter has their own context and ledger Arc
    
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"manifest_timeout");
    let job_json_payload = json!({
        "manifest_cid": manifest_cid,
        "spec": {},
        "cost_mana": job_cost,
    }).to_string();

    let submitted_job_id = host_submit_mesh_job(&ctx_submitter, &job_json_payload)
        .await
        .expect("Job submission failed");
    
    assert_eq!(ctx_submitter.get_mana(&submitter_did).await.unwrap(), initial_mana - job_cost, "Submitter mana not deducted correctly post-submission");

    let submitted_job_details = ActualMeshJob {
        id: submitted_job_id.clone(),
        manifest_cid: manifest_cid.clone(),
        spec: JobSpec::default(),
        creator_did: submitter_did.clone(),
        cost_mana: job_cost,
    };
    arc_ctx_job_manager.internal_queue_mesh_job(submitted_job_details.clone()).await.unwrap();
    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Pending).await;

    let executor_did_str = "did:icn:test:executor_timeout_assign";
    let executor_did = Did::from_str(executor_did_str).unwrap();
    let bid = MeshJobBid {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: 5, 
        resources: Resources::default(),
    };
    let job_manager_network_stub = get_stub_network_service(&arc_ctx_job_manager);
    job_manager_network_stub.stage_bid(submitted_job_id.clone(), bid).await;
    
    sleep(Duration::from_millis(1200)).await; 
    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Assigned { expected_executor: Some(executor_did.clone()) }).await;

    println!("Advancing time for job timeout...");
    tokio::time::advance(Duration::from_secs(5 * 60 + 5)).await; // Advance past the 5-min timeout + buffer
    
    sleep(Duration::from_millis(1200)).await; // Allow job manager loop to process timeout and refund

    assert_job_state(&arc_ctx_job_manager, &submitted_job_id, JobStateVariant::Failed).await;
    
    // Check submitter's mana on their original context.
    // This now relies on SimpleManaLedger's balances being Arc<Mutex<HashMap<...>>>
    // and RuntimeContext.mana_ledger cloning the SimpleManaLedger struct (which clones the Arc).
    let submitter_mana_after_refund = ctx_submitter.get_mana(&submitter_did).await.unwrap();
    assert_eq!(submitter_mana_after_refund, initial_mana, "Submitter mana not refunded correctly. Expected {}, got {}", initial_mana, submitter_mana_after_refund);
    
    println!("Timeout and refund test completed. Submitter mana checked.");
}

// TODO: test_invalid_receipt_wrong_executor
// TODO: test_invalid_receipt_bad_signature (requires more control over StubSigner or a mockable signer)
// TODO: test_duplicate_bids_same_executor
// TODO: test_insufficient_mana_on_submission (already covered by lib.rs tests, but could have e2e)
// TODO: test_multiple_concurrent_jobs
// TODO: test_no_bids_job_re_queued
