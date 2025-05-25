// crates/icn-runtime/tests/mesh.rs

use icn_common::{Did, Cid};
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_runtime::context::{RuntimeContext, HostAbiError, StubMeshNetworkService, StubDagStore, DagStore};
use icn_runtime::host_submit_mesh_job;
use icn_mesh::{JobId, ActualMeshJob, MeshJobBid, JobState, SubmitReceiptMessage, JobSpec, Resources};
use icn_network::NetworkService;
use libp2p::PeerId as Libp2pPeerId;
use anyhow::Result;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use log::{info, debug}; // Added for potential future use, using println! for now


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

// Placeholder for new_mesh_test_context_with_two_executors
// This helper needs to be properly implemented or use existing ones if available.
// For now, it uses the existing single context creator.
fn new_mesh_test_context_with_two_executors() -> (Arc<RuntimeContext>, Arc<RuntimeContext>, Arc<RuntimeContext>, Arc<StubDagStore>) {
    // TODO: This is a simplified stub. Properly implement context creation for multiple distinct DIDs.
    // The main issue is that create_test_context initializes SimpleManaLedger anew each time.
    // For a multi-actor test, they might need to share a ManaLedger or have distinct pre-funded DIDs.
    // For now, we create separate contexts. The DAG store can be from any of them if it's a shared stub.
    let submitter_ctx = create_test_context("did:icn:test:submitter_multi_exec", 200);
    let executor1_ctx = create_test_context("did:icn:test:executor1_multi_exec", 100);
    let executor2_ctx = create_test_context("did:icn:test:executor2_multi_exec", 100);
    let dag_store = get_stub_dag_store(&submitter_ctx); // Assumes StubDagStore is shareable or cloned appropriately
    (submitter_ctx, executor1_ctx, executor2_ctx, dag_store)
}

// Placeholder for create_test_job
// Returns a JSON string payload for host_submit_mesh_job, and the expected cost
fn create_test_job_payload_and_cost() -> (String, u64) {
    // TODO: Implement this helper function more robustly
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest_for_invalid_receipt");
    let job_cost = 20u64;
    let job_json_payload = json!({
        "manifest_cid": manifest_cid,
        "spec": { "details": "job for invalid receipt test"}, 
        "cost_mana": job_cost,
    }).to_string();
    (job_json_payload, job_cost)
}

// Placeholder for create_test_bid
// Returns a MeshJobBid
fn create_test_bid(job_id: &Cid, executor_ctx: &Arc<RuntimeContext>, price: u64) -> MeshJobBid {
    // TODO: Implement this helper function
    MeshJobBid {
        job_id: job_id.clone(), // JobId is a Cid here
        executor_did: executor_ctx.current_identity.clone(),
        price_mana: price,
        resources: Resources::default(),
    }
}

// Placeholder for assign_job_to_executor (simulated)
// In a real test, this would involve the job manager's logic.
// Here, we directly update the job_manager_ctx's state for simplicity.
async fn assign_job_to_executor_directly(job_manager_ctx: &Arc<RuntimeContext>, job_id: Cid, assigned_executor_did: &Did) {
    // TODO: This is a test utility to bypass full job manager loop for specific assignment tests.
    println!("Test util: Directly assigning job {:?} to executor {:?}", job_id, assigned_executor_did);
    let mut states = job_manager_ctx.job_states.lock().await;
    states.insert(job_id, JobState::Assigned { executor: assigned_executor_did.clone() });
}


// Placeholder for forge_execution_receipt
// Creates an IdentityExecutionReceipt, signed by the forging_executor_ctx.
// IMPORTANT: The actual `anchor_receipt` on the `job_manager_ctx` is what validates.
// This helper just creates a receipt structure and has the forger sign it.
fn forge_execution_receipt(job_id: &Cid, result_cid_val: &[u8], forging_executor_ctx: &Arc<RuntimeContext>) -> IdentityExecutionReceipt {
    // TODO: Implement this helper function
    let mut receipt = IdentityExecutionReceipt {
        job_id: job_id.clone(), // JobId is a Cid
        executor_did: forging_executor_ctx.current_identity.clone(), // Forged: signed by this DID
        result_cid: Cid::new_v1_dummy(0x55, 0x13, result_cid_val),
        cpu_ms: 50,
        sig: Vec::new(), // Will be filled by the forger's context
    };
    // The forging_executor_ctx signs the receipt using its own identity and signer.
    forging_executor_ctx.anchor_receipt(&mut receipt).expect("Forger failed to sign its own receipt for forging");
    receipt // Returns the signed receipt
}


#[tokio::test]
async fn test_invalid_receipt_wrong_executor() {
    // Setup:
    // Job Manager context - this will manage the job states and process bids/receipts.
    let job_manager_ctx = create_test_context("did:icn:test:job_manager_for_invalid_receipt_test", 0);
    job_manager_ctx.spawn_mesh_job_manager().await; // Start the job manager task

    // Actors
    let submitter_ctx = create_test_context("did:icn:test:submitter_for_invalid_receipt", 100);
    let executor1_ctx = create_test_context("did:icn:test:legit_executor_for_invalid_receipt", 100); // Legitimate
    let executor2_ctx = create_test_context("did:icn:test:forging_executor_for_invalid_receipt", 100); // Forger

    // 1. Submit a mesh job as the submitter.
    let (job_payload, job_cost) = create_test_job_payload_and_cost();
    let submitted_job_id = host_submit_mesh_job(&submitter_ctx, &job_payload) // Corrected: host_submit_mesh_job is a free function
        .await
        .expect("Job submission by submitter failed");

    // Manually transfer/inform the job manager about the job for this test setup.
    // This simulates the job appearing in the manager's queue.
    let job_details_for_manager = ActualMeshJob {
        id: submitted_job_id.clone(),
        manifest_cid: Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest_for_invalid_receipt"), // Match manifest from payload
        spec: serde_json::from_str::<serde_json::Value>(&job_payload).unwrap()["spec"].as_object().cloned().map_or_else(JobSpec::default, |_| JobSpec::default()), // Simplified spec
        creator_did: submitter_ctx.current_identity.clone(),
        cost_mana: job_cost,
    };
    job_manager_ctx.internal_queue_mesh_job(job_details_for_manager).await.expect("Failed to queue job in job manager");
    assert_job_state(&job_manager_ctx, &submitted_job_id, JobStateVariant::Pending).await;


    // 2. Executor 1 (legitimate) bids for the job.
    let bid1 = create_test_bid(&submitted_job_id, &executor1_ctx, 10);
    let network_stub_for_job_manager = get_stub_network_service(&job_manager_ctx);
    network_stub_for_job_manager.stage_bid(submitted_job_id.clone(), bid1).await;

    // Allow job manager to process the bid and assign.
    sleep(Duration::from_millis(1200)).await; // Adjust as needed for job manager cycle time
    assert_job_state(&job_manager_ctx, &submitted_job_id, JobStateVariant::Assigned { expected_executor: Some(executor1_ctx.current_identity.clone()) }).await;

    // 3. Executor 2 (the *wrong* one) forges an execution receipt.
    // The receipt is for the submitted_job_id, but signed by executor2_ctx.
    let forged_receipt = forge_execution_receipt(&submitted_job_id, b"forged_result_data", &executor2_ctx);
    
    // Sanity check: the forged_receipt should have executor2's DID
    assert_eq!(forged_receipt.executor_did, executor2_ctx.current_identity);
    // And it should have a signature (filled by forge_execution_receipt helper)
    assert!(!forged_receipt.sig.is_empty(), "Forged receipt should have a signature");


    // 4. Submit the forged receipt to the job manager. This should fail.
    // The job_manager_ctx.anchor_receipt method is synchronous.
    let result = job_manager_ctx.anchor_receipt(&mut forged_receipt.clone()); // Clone as anchor_receipt might mutate for its own signing if it were the executor

    // 5. Assertions
    assert!(result.is_err(), "Anchoring a forged receipt by the wrong executor should fail. Result: {:?}", result);
    
    let err = result.err().unwrap();
    println!("Anchor receipt failed with error: {:?}", err); // For debugging

    match err {
        HostAbiError::InvalidParameters(msg) => {
            println!("Correctly failed with InvalidParameters: {}", msg);
            // Optionally, assert that msg contains something about executor mismatch if the implementation provides that.
            // e.g., assert!(msg.to_lowercase().contains("executor"));
        }
        HostAbiError::SignatureError(msg) => {
            println!("Failed with SignatureError: {}. This might be secondary if executor check is first.", msg);
            // This could happen if the signature is invalid for *any* reason, not just wrong DID.
        }
        HostAbiError::InternalError(msg) => {
            println!("Failed with InternalError: {}. Check internal logic.", msg);
        }
        HostAbiError::NotImplemented(msg) => {
             panic!("Test failed due to 'NotImplemented': {}. Anchor receipt logic for executor validation needs to be implemented.", msg);
        }
        other_error => {
            panic!("Expected InvalidParameters, SignatureError, or InternalError related to executor mismatch, but got: {:?}", other_error);
        }
    }

    // Ensure job state remains 'Assigned' to executor1 and not 'Completed'.
    assert_job_state(&job_manager_ctx, &submitted_job_id, JobStateVariant::Assigned { expected_executor: Some(executor1_ctx.current_identity.clone()) }).await;
    println!("Test 'test_invalid_receipt_wrong_executor' completed successfully.");
}

// TODO: test_invalid_receipt_bad_signature (requires more control over StubSigner or a mockable signer)
// TODO: test_duplicate_bids_same_executor
// TODO: test_insufficient_mana_on_submission (already covered by lib.rs tests, but could have e2e)
// TODO: test_multiple_concurrent_jobs
// TODO: test_no_bids_job_re_queued

#[tokio::test]
#[ignore = "Blocked on environment/macro/import issues, particularly with libp2p Kademlia types and tokio/serde macros in dependent crates."]
async fn test_full_mesh_job_cycle_libp2p() -> Result<(), anyhow::Error> {
    println!("[test-mesh-runtime] Starting test_full_mesh_job_cycle_libp2p");
    // 1. Setup Node A (Job Manager / Submitter)
    println!("[test-mesh-runtime] Setting up Node A (Job Manager/Submitter).");
    let node_a_libp2p_actual_service = Arc::new(icn_network::libp2p_service::Libp2pNetworkService::new(None).await?);
    let node_a_peer_id_str = node_a_libp2p_actual_service.local_peer_id().to_string();
    let node_a_addrs = node_a_libp2p_actual_service.listening_addresses();
    assert!(!node_a_addrs.is_empty(), "Node A should have listening addresses");
    println!("[test-mesh-runtime] Node A Peer ID: {}, Listening Addresses: {:?}", node_a_peer_id_str, node_a_addrs);

    let node_a_ctx = Arc::new(RuntimeContext::new(
        Did::from_str("did:icn:test:node_a_libp2p")?,
        Arc::new(icn_runtime::context::DefaultMeshNetworkService::new(node_a_libp2p_actual_service.clone())),
        Arc::new(icn_runtime::context::StubSigner),
        Arc::new(icn_runtime::context::StubDagStore::new()),
    ));
    node_a_ctx.mana_ledger.set_balance(&node_a_ctx.current_identity, 1000).await; 
    println!("[test-mesh-runtime] Node A context created, mana set. Spawning Job Manager.");
    node_a_ctx.spawn_mesh_job_manager().await;

    // 2. Setup Node B (Executor)
    println!("[test-mesh-runtime] Setting up Node B (Executor), bootstrapping with Node A.");
    let node_a_libp2p_peer_id_for_b = icn_network::libp2p_service::Libp2pPeerId::from_str(&node_a_peer_id_str)?;
    let bootstrap_peers_for_b = Some(vec![(node_a_libp2p_peer_id_for_b, node_a_addrs[0].clone())]);
    let node_b_libp2p_actual_service = Arc::new(icn_network::libp2p_service::Libp2pNetworkService::new(bootstrap_peers_for_b).await?);
    println!("[test-mesh-runtime] Node B Peer ID: {}", node_b_libp2p_actual_service.local_peer_id().to_string());
    
    let node_b_ctx = Arc::new(RuntimeContext::new(
        Did::from_str("did:icn:test:node_b_libp2p")?,
        Arc::new(icn_runtime::context::DefaultMeshNetworkService::new(node_b_libp2p_actual_service.clone())),
        Arc::new(icn_runtime::context::StubSigner),
        Arc::new(icn_runtime::context::StubDagStore::new()),
    ));
    node_b_ctx.mana_ledger.set_balance(&node_b_ctx.current_identity, 500).await;
    println!("[test-mesh-runtime] Node B context created, mana set.");

    println!("[test-mesh-runtime] Allowing 5s for network connection.");
    sleep(Duration::from_secs(5)).await;

    // 3. Node A submits a job
    let job_cost = 50u64;
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"job_manifest_libp2p_test");
    let job_json_payload = json!({
        "manifest_cid": manifest_cid,
        "spec": {},
        "cost_mana": job_cost,
    }).to_string();

    println!("[test-mesh-runtime] Node B (executor) subscribing to its Libp2p service to listen for announcements.");
    let mut node_b_raw_receiver = node_b_libp2p_actual_service.as_ref().subscribe()
        .map_err(|e| anyhow::anyhow!("Node B failed to subscribe: {e}"))?;

    let submitted_job_id = host_submit_mesh_job(&node_a_ctx, &job_json_payload).await?;
    println!("[test-mesh-runtime] Node A submitted job ID: {}. Payload: {}. Asserting Pending state.", submitted_job_id, job_json_payload);
    assert_job_state(&node_a_ctx, &submitted_job_id, JobStateVariant::Pending).await;

    // 4. Node B listens for the job, receives it, and submits a bid
    println!("[test-mesh-runtime] Node B listening for job announcement (timeout 20s).");
    let received_on_b = tokio::time::timeout(Duration::from_secs(20), node_b_raw_receiver.recv()).await??;
    assert!(received_on_b.is_some(), "Node B: Raw receiver channel closed or got None");

    if let Some(icn_network::NetworkMessage::MeshJobAnnouncement(announced_job)) = received_on_b {
        assert_eq!(announced_job.id, submitted_job_id, "Node B received announcement for wrong job");
        println!("[test-mesh-runtime] Node B received announcement for job ID: {}. Submitting bid.", announced_job.id);

        let bid = MeshJobBid {
            job_id: announced_job.id.clone(),
            executor_did: node_b_ctx.current_identity.clone(),
            price_mana: 20,
            resources: Resources::default(),
        };
        node_b_ctx.mesh_network_service.as_ref().broadcast_message(
            icn_network::NetworkMessage::BidSubmission(bid.clone())
        ).await.map_err(|e| anyhow::anyhow!("Node B failed to broadcast bid: {e}"))?;
        println!("[test-mesh-runtime] Node B submitted bid for job ID: {}", announced_job.id);
    } else {
        panic!("[test-mesh-runtime] Node B did not receive MeshJobAnnouncement, got: {:?}", received_on_b);
    }

    println!("[test-mesh-runtime] Allowing 10s for JobManager on Node A to process bids and assign.");
    sleep(Duration::from_secs(10)).await;
    
    println!("[test-mesh-runtime] Asserting job {} is assigned to Node B.", submitted_job_id);
    assert_job_state(&node_a_ctx, &submitted_job_id, JobStateVariant::Assigned { expected_executor: Some(node_b_ctx.current_identity.clone()) }).await;
    println!("[test-mesh-runtime] Job {} successfully assigned to Node B {}. Node B preparing receipt.", submitted_job_id, node_b_ctx.current_identity.to_string());
    
    // 7. Node B "executes" the job and prepares a receipt
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"libp2p_test_result_data");
    let mut receipt_by_node_b = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: node_b_ctx.current_identity.clone(),
        result_cid: result_cid.clone(),
        cpu_ms: 75,
        sig: Vec::new(),
    };

    println!("[test-mesh-runtime] Node B signing its execution receipt for job {}.", submitted_job_id);
    match node_b_ctx.anchor_receipt(&mut receipt_by_node_b) {
        Ok(_) => println!("[test-mesh-runtime] Node B signed its execution receipt for job {}", submitted_job_id),
        Err(e) => return Err(anyhow::anyhow!("Node B failed to sign its own receipt: {}", e)),
    }
    assert!(!receipt_by_node_b.sig.is_empty(), "Node B's receipt should be signed");

    println!("[test-mesh-runtime] Node B broadcasting receipt for job {}.", submitted_job_id);
    let receipt_message = icn_network::NetworkMessage::SubmitReceipt(receipt_by_node_b.clone());
    node_b_ctx.mesh_network_service.as_ref().broadcast_message(receipt_message).await
        .map_err(|e| anyhow::anyhow!("Node B failed to broadcast receipt: {e}"))?;
    println!("[test-mesh-runtime] Node B broadcasted receipt for job {}. Waiting 10s for JobManager processing.", submitted_job_id);

    sleep(Duration::from_secs(10)).await;

    println!("[test-mesh-runtime] Asserting job {} is Completed on Node A.", submitted_job_id);
    assert_job_state(&node_a_ctx, &submitted_job_id, JobStateVariant::Completed {
        expected_receipt_data: Some(ExpectedReceiptData {
            job_id: submitted_job_id.clone(),
            executor_did: node_b_ctx.current_identity.clone(),
            result_cid: result_cid.clone(),
        })
    }).await;
    println!("[test-mesh-runtime] Job {} successfully marked as Completed on Node A. Test finished.", submitted_job_id);

    Ok(())
}
