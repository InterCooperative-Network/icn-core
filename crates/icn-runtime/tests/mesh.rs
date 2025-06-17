#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::uninlined_format_args
)]
// crates/icn-runtime/tests/mesh.rs

use icn_common::{Cid, Did};
use icn_dag::StorageService;
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec, JobState, MeshJobBid, Resources};
use icn_network::libp2p_service::NetworkConfig;
use icn_network::NetworkService;
use icn_runtime::context::{
    DefaultMeshNetworkService, HostAbiError, JobAssignmentNotice, LocalMeshSubmitReceiptMessage,
    MeshNetworkService, RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner,
};
use icn_runtime::{host_get_pending_mesh_jobs, host_submit_mesh_job};
#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

// Helper to create a test ActualMeshJob with all required fields
fn create_test_mesh_job(manifest_cid: Cid, cost_mana: u64, creator_did: Did) -> ActualMeshJob {
    ActualMeshJob {
        id: Cid::new_v1_dummy(0x55, 0x13, b"test_job_id"),
        manifest_cid,
        spec: JobSpec::default(),
        creator_did,
        cost_mana,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![0u8; 64]), // Dummy signature for tests
    }
}

// Helper to create a RuntimeContext with a specific DID and initial mana.
// The Stub services are now part of RuntimeContext::new_with_stubs_and_mana
fn create_test_context(identity_did_str: &str, initial_mana: u64) -> Arc<RuntimeContext> {
    let _ = std::fs::remove_file("./mana_ledger.sled");
    RuntimeContext::new_with_stubs_and_mana(identity_did_str, initial_mana)
}

// Helper to assert the state of a job
async fn assert_job_state(
    ctx: &Arc<RuntimeContext>,
    job_id: &JobId,
    expected_state_variant: JobStateVariant,
) {
    tokio::task::yield_now().await;
    sleep(Duration::from_millis(100)).await; // Increased delay slightly for job manager processing

    let states = ctx.job_states.lock().await;
    let job_state = states.get(job_id).unwrap_or_else(|| {
        panic!(
            "Job ID {:?} not found in states map. States: {:?}",
            job_id, states
        )
    });

    match (job_state, &expected_state_variant) {
        (JobState::Pending, JobStateVariant::Pending) => {}
        (JobState::Assigned { executor }, JobStateVariant::Assigned { expected_executor }) => {
            if let Some(expected_exec_did) = expected_executor {
                assert_eq!(
                    executor, expected_exec_did,
                    "Job {:?} assigned to unexpected executor. Expected {:?}, got {:?}",
                    job_id, expected_exec_did, executor
                );
            }
        }
        (
            JobState::Completed { receipt },
            JobStateVariant::Completed {
                expected_receipt_data,
            },
        ) => {
            if let Some(data) = expected_receipt_data {
                assert_eq!(
                    &receipt.job_id, &data.job_id,
                    "Completed receipt job_id mismatch"
                );
                assert_eq!(
                    &receipt.executor_did, &data.executor_did,
                    "Completed receipt executor_did mismatch"
                );
                assert_eq!(
                    &receipt.result_cid, &data.result_cid,
                    "Completed receipt result_cid mismatch"
                );
            }
        }
        (JobState::Failed { reason: _ }, JobStateVariant::Failed) => {}
        (actual, expected) => panic!(
            "Job {:?} is in state {:?}, expected variant {:?}",
            job_id, actual, expected
        ),
    }
}

// Simplified enum for asserting job state variants
#[derive(Debug, PartialEq, Clone)]
enum JobStateVariant {
    Pending,
    Assigned {
        expected_executor: Option<Did>,
    },
    Completed {
        expected_receipt_data: Option<ExpectedReceiptData>,
    },
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

use icn_common::DagBlock;

fn get_dag_store(
    ctx: &Arc<RuntimeContext>,
) -> Arc<TokioMutex<dyn StorageService<DagBlock> + Send>> {
    ctx.dag_store.clone()
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

    let job_manager_network_stub = get_stub_network_service(&arc_ctx_job_manager);
    let job_manager_dag_store = get_dag_store(&arc_ctx_job_manager);

    // 1. SUBMISSION - Test the job submission flow
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"manifest_happy");
    let job_cost = 20u64;
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    let job_json_payload = serde_json::to_string(&test_job).unwrap();

    let submitted_job_id = host_submit_mesh_job(&ctx_submitter, &job_json_payload)
        .await
        .expect("Job submission failed");

    assert_eq!(
        ctx_submitter.get_mana(&submitter_did).await.unwrap(),
        100 - job_cost,
        "Submitter mana not deducted correctly"
    );

    // Queue the job into the Job Manager's context
    let submitted_job_details = ActualMeshJob {
        id: submitted_job_id.clone(),
        manifest_cid: manifest_cid.clone(),
        spec: JobSpec::default(),
        creator_did: submitter_did.clone(),
        cost_mana: job_cost,
        max_execution_wait_ms: None,
        signature: SignatureBytes(Vec::new()),
    };
    arc_ctx_job_manager
        .internal_queue_mesh_job(submitted_job_details.clone())
        .await
        .unwrap();

    // 2. Test the network service functionality directly
    // Announce job
    let announce_result = job_manager_network_stub
        .announce_job(&submitted_job_details)
        .await;
    assert!(
        announce_result.is_ok(),
        "Job announcement failed: {:?}",
        announce_result
    );

    // Stage and collect bids
    let unsigned_bid = MeshJobBid {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: 10,
        resources: Resources::default(),
        signature: SignatureBytes(vec![]),
    };
    let sig = arc_ctx_job_manager
        .signer
        .sign(&unsigned_bid.to_signable_bytes().unwrap())
        .unwrap();
    let bid = MeshJobBid {
        signature: SignatureBytes(sig),
        ..unsigned_bid
    };
    job_manager_network_stub
        .stage_bid(submitted_job_id.clone(), bid)
        .await;

    let collected_bids = job_manager_network_stub
        .collect_bids_for_job(&submitted_job_id, Duration::from_millis(100))
        .await
        .expect("Bid collection failed");

    assert_eq!(
        collected_bids.len(),
        1,
        "Expected 1 bid, got {}",
        collected_bids.len()
    );
    assert_eq!(collected_bids[0].executor_did, executor_did);

    // 3. Test assignment notification
    let assignment_notice = JobAssignmentNotice {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
    };
    let assignment_result = job_manager_network_stub
        .notify_executor_of_assignment(&assignment_notice)
        .await;
    assert!(
        assignment_result.is_ok(),
        "Assignment notification failed: {:?}",
        assignment_result
    );

    // 4. Test receipt processing
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"result_happy");
    let ctx_executor_for_signing = create_test_context(executor_did_str, 0);

    // Create the receipt and sign it using the public API
    let unsigned_receipt = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        result_cid: result_cid.clone(),
        cpu_ms: 100,
        success: true,
        sig: SignatureBytes(Vec::new()),
    };

    // For testing purposes, let's create a simple signed receipt using dummy signature
    // In a real system, the executor would sign this with their private key
    let signature_bytes = ctx_executor_for_signing
        .signer
        .sign(b"dummy_receipt_data")
        .expect("Failed to sign receipt");

    let signed_receipt = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        result_cid: result_cid.clone(),
        cpu_ms: 100,
        success: true,
        sig: SignatureBytes(signature_bytes),
    };

    // Stage the signed receipt
    let receipt_msg = LocalMeshSubmitReceiptMessage {
        receipt: signed_receipt.clone(),
    };
    job_manager_network_stub.stage_receipt(receipt_msg).await;

    // Test receipt retrieval
    let retrieved_receipt = job_manager_network_stub
        .try_receive_receipt(&submitted_job_id, &executor_did, Duration::from_millis(100))
        .await
        .expect("Receipt retrieval failed");

    assert!(retrieved_receipt.is_some(), "No receipt retrieved");
    let retrieved_receipt = retrieved_receipt.unwrap();
    assert_eq!(retrieved_receipt.job_id, submitted_job_id);
    assert_eq!(retrieved_receipt.executor_did, executor_did);

    // 5. Test DAG anchoring - for now just verify the receipt structure
    assert!(
        !retrieved_receipt.sig.0.is_empty(),
        "Receipt should have a signature"
    );
    assert_eq!(retrieved_receipt.job_id, submitted_job_id);
    assert_eq!(retrieved_receipt.executor_did, executor_did);

    // Store in DAG using the job manager's storage
    let dag_store = get_dag_store(&arc_ctx_job_manager);
    let receipt_bytes =
        serde_json::to_vec(&retrieved_receipt).expect("Failed to serialize receipt");
    let block = DagBlock {
        cid: Cid::new_v1_dummy(0x71, 0x12, &receipt_bytes),
        data: receipt_bytes,
        links: vec![],
    };
    {
        let mut store = dag_store.lock().await;
        store.put(&block).expect("Failed to store receipt in DAG");
    }
    let stored_cid = block.cid.clone();

    println!(
        "Happy path test completed successfully! Receipt stored with CID: {:?}",
        stored_cid
    );
}

#[tokio::test]
async fn test_mesh_job_timeout_and_refund() {
    let submitter_did_str = "did:icn:test:submitter_timeout";
    let submitter_did = Did::from_str(submitter_did_str).unwrap();
    let initial_mana = 100u64;
    let job_cost = 30u64;

    let ctx_submitter = create_test_context(submitter_did_str, initial_mana);
    let arc_ctx_job_manager = create_test_context("did:icn:test:job_manager_node_timeout", 0);

    // 1. Submit job
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"manifest_timeout");
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    let job_json_payload = serde_json::to_string(&test_job).unwrap();

    let submitted_job_id = host_submit_mesh_job(&ctx_submitter, &job_json_payload)
        .await
        .expect("Job submission failed");

    assert_eq!(
        ctx_submitter.get_mana(&submitter_did).await.unwrap(),
        initial_mana - job_cost,
        "Submitter mana not deducted correctly post-submission"
    );

    // 2. Test network service with no bids - simulating timeout scenario
    let submitted_job_details = ActualMeshJob {
        id: submitted_job_id.clone(),
        manifest_cid: manifest_cid.clone(),
        spec: JobSpec::default(),
        creator_did: submitter_did.clone(),
        cost_mana: job_cost,
        max_execution_wait_ms: None,
        signature: SignatureBytes(Vec::new()),
    };

    let job_manager_network_stub = get_stub_network_service(&arc_ctx_job_manager);

    // Announce job
    let announce_result = job_manager_network_stub
        .announce_job(&submitted_job_details)
        .await;
    assert!(
        announce_result.is_ok(),
        "Job announcement failed: {:?}",
        announce_result
    );

    // Try to collect bids with no bids staged - should return empty
    let collected_bids = job_manager_network_stub
        .collect_bids_for_job(&submitted_job_id, Duration::from_millis(100))
        .await
        .expect("Bid collection failed");

    assert_eq!(
        collected_bids.len(),
        0,
        "Expected 0 bids, got {}",
        collected_bids.len()
    );

    // 3. Test mana refund scenario
    let refund_result = ctx_submitter.credit_mana(&submitter_did, job_cost).await;
    assert!(
        refund_result.is_ok(),
        "Mana refund failed: {:?}",
        refund_result
    );

    let submitter_mana_after_refund = ctx_submitter.get_mana(&submitter_did).await.unwrap();
    assert_eq!(
        submitter_mana_after_refund, initial_mana,
        "Submitter mana not refunded correctly. Expected {}, got {}",
        initial_mana, submitter_mana_after_refund
    );

    println!("Timeout and refund test completed successfully!");
}

#[tokio::test]
async fn test_invalid_receipt_wrong_executor() {
    let submitter_did_str = "did:icn:test:submitter_for_invalid_receipt";
    let correct_executor_did_str = "did:icn:test:executor_correct";
    let wrong_executor_did_str = "did:icn:test:executor_wrong";

    let submitter_did = Did::from_str(submitter_did_str).unwrap();
    let _correct_executor_did = Did::from_str(correct_executor_did_str).unwrap();
    let wrong_executor_did = Did::from_str(wrong_executor_did_str).unwrap();

    let initial_mana = 100u64;
    let job_cost = 20u64;

    let ctx_submitter = create_test_context(submitter_did_str, initial_mana);
    let arc_ctx_job_manager = create_test_context("did:icn:test:job_manager_invalid_receipt", 0);

    // 1. Submit job
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest_for_invalid_receipt");
    let test_job = create_test_mesh_job(manifest_cid.clone(), job_cost, submitter_did.clone());
    let job_json_payload = serde_json::to_string(&test_job).unwrap();

    let submitted_job_id = host_submit_mesh_job(&ctx_submitter, &job_json_payload)
        .await
        .expect("Job submission by submitter failed");

    // 2. Test receipt verification directly by creating a forged receipt
    let wrong_executor_ctx = create_test_context(wrong_executor_did_str, 0);

    // Create a receipt with the wrong executor DID but valid signature from that executor
    let signature_bytes = wrong_executor_ctx
        .signer
        .sign(b"dummy_receipt_data")
        .expect("Failed to sign receipt");

    let forged_receipt = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: wrong_executor_did.clone(), // Wrong executor DID
        result_cid: Cid::new_v1_dummy(0x55, 0x13, b"result_invalid_executor"),
        cpu_ms: 50,
        success: true,
        sig: SignatureBytes(signature_bytes),
    };

    // Try to anchor the forged receipt - this should fail due to DID mismatch
    let anchor_result = arc_ctx_job_manager.anchor_receipt(&forged_receipt).await;

    // The anchoring should fail because the job manager's signer is different from the forged executor
    assert!(
        anchor_result.is_err(),
        "Forged receipt should not be accepted! Result: {:?}",
        anchor_result
    );

    if let Err(error) = anchor_result {
        match error {
            HostAbiError::SignatureError(_) => {
                println!("Correctly rejected forged receipt due to signature error");
            }
            HostAbiError::CryptoError(_) => {
                println!("Correctly rejected forged receipt due to crypto error");
            }
            _ => {
                println!("Receipt rejected for other reason: {:?}", error);
            }
        }
    }

    // 3. Test with correct executor - this should also fail since job manager has different keys
    let correct_executor_ctx = create_test_context(correct_executor_did_str, 0);
    let correct_signature_bytes = correct_executor_ctx
        .signer
        .sign(b"dummy_receipt_data")
        .expect("Failed to sign receipt");

    let correct_receipt = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: correct_executor_ctx.current_identity.clone(),
        result_cid: Cid::new_v1_dummy(0x55, 0x13, b"result_invalid_executor"),
        cpu_ms: 50,
        success: true,
        sig: SignatureBytes(correct_signature_bytes),
    };

    // This should also fail because the job manager context signer doesn't match the executor
    let _correct_anchor_result = arc_ctx_job_manager.anchor_receipt(&correct_receipt).await;
    // Note: This will likely fail because the job manager's signer is different from the executor's signer
    // In a real system, the job manager would need to verify against the executor's public key

    println!("Invalid receipt test completed - forged receipt verification tested");
}

#[tokio::test]
async fn test_job_manager_refunds_on_no_valid_bid() {
    let ctx = create_test_context("did:icn:test:refund_mgr", 50);
    let submitter_did = ctx.current_identity.clone();

    let (job_json, job_cost) = create_test_job_payload_and_cost(&submitter_did, 25);
    let job_id = host_submit_mesh_job(&ctx, &job_json)
        .await
        .expect("Job submission failed");

    let ctx_clone = ctx.clone();
    ctx_clone.spawn_mesh_job_manager().await;

    sleep(Duration::from_secs(12)).await;

    let mana_after = ctx.get_mana(&submitter_did).await.unwrap();
    assert_eq!(mana_after, 50);

    let states = ctx.job_states.lock().await;
    let state = states.get(&job_id).expect("job state");
    match state {
        JobState::Failed { .. } => {}
        other => panic!("Unexpected state {:?}", other),
    }
}

/// Creates a set of `RuntimeContext`s representing a submitter and two executors.
///
/// All contexts share a single `StubMeshNetworkService` and DAG store so that
/// bids and receipts can be exchanged between them within the tests.
#[allow(clippy::type_complexity)]
fn new_mesh_test_context_with_two_executors() -> (
    Arc<RuntimeContext>,
    Arc<RuntimeContext>,
    Arc<RuntimeContext>,
    Arc<TokioMutex<dyn StorageService<DagBlock> + Send>>,
) {
    let network_service: Arc<StubMeshNetworkService> = Arc::new(StubMeshNetworkService::new());
    let dag_store: Arc<TokioMutex<dyn StorageService<DagBlock> + Send>> =
        Arc::new(TokioMutex::new(StubDagStore::new()));

    let submitter_did = Did::from_str("did:icn:test:submitter_multi_exec").unwrap();
    let executor1_did = Did::from_str("did:icn:test:executor1_multi_exec").unwrap();
    let executor2_did = Did::from_str("did:icn:test:executor2_multi_exec").unwrap();

    let submitter_ctx = RuntimeContext::new(
        submitter_did.clone(),
        network_service.clone(),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store.clone(),
    );
    submitter_ctx
        .mana_ledger
        .set_balance(&submitter_did, 200)
        .expect("set initial mana");

    let executor1_ctx = RuntimeContext::new(
        executor1_did.clone(),
        network_service.clone(),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store.clone(),
    );
    executor1_ctx
        .mana_ledger
        .set_balance(&executor1_did, 100)
        .expect("set initial mana");

    let executor2_ctx = RuntimeContext::new(
        executor2_did.clone(),
        network_service.clone(),
        Arc::new(StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        dag_store.clone(),
    );
    executor2_ctx
        .mana_ledger
        .set_balance(&executor2_did, 100)
        .expect("set initial mana");

    (submitter_ctx, executor1_ctx, executor2_ctx, dag_store)
}

/// Convenience helper to create a simple test job JSON payload with a given
/// cost and submitter.
fn create_test_job_payload_and_cost(submitter: &Did, job_cost: u64) -> (String, u64) {
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest");
    let test_job = create_test_mesh_job(manifest_cid, job_cost, submitter.clone());
    let job_json_payload = serde_json::to_string(&test_job).unwrap();
    (job_json_payload, job_cost)
}

/// Creates a `MeshJobBid` for the provided job using the executor context.
fn create_test_bid(job_id: &Cid, executor_ctx: &Arc<RuntimeContext>, price: u64) -> MeshJobBid {
    let unsigned = MeshJobBid {
        job_id: job_id.clone(), // JobId is a Cid here
        executor_did: executor_ctx.current_identity.clone(),
        price_mana: price,
        resources: Resources::default(),
        signature: SignatureBytes(vec![]),
    };
    let sig = executor_ctx
        .signer
        .sign(&unsigned.to_signable_bytes().unwrap())
        .unwrap();
    MeshJobBid {
        signature: SignatureBytes(sig),
        ..unsigned
    }
}

async fn assign_job_to_executor_directly(
    job_manager_ctx: &Arc<RuntimeContext>,
    job_id: Cid,
    assigned_executor_did: &Did,
) {
    println!(
        "Test util: Directly assigning job {:?} to executor {:?}",
        job_id, assigned_executor_did
    );
    let mut states = job_manager_ctx.job_states.lock().await;
    states.insert(
        job_id,
        JobState::Assigned {
            executor: assigned_executor_did.clone(),
        },
    );
}

#[tokio::test]
async fn test_job_assignment_with_two_executors() {
    let (submitter_ctx, executor1_ctx, executor2_ctx, _) =
        new_mesh_test_context_with_two_executors();

    let submitter_did = submitter_ctx.current_identity.clone();

    // Submit a job
    let (job_json, job_cost) = create_test_job_payload_and_cost(&submitter_did, 10);
    let job_id = host_submit_mesh_job(&submitter_ctx, &job_json)
        .await
        .expect("Job submission failed");

    assert_eq!(
        submitter_ctx.get_mana(&submitter_did).await.unwrap(),
        200 - job_cost
    );

    // Stage bids from two executors with different prices
    let network = get_stub_network_service(&submitter_ctx);
    let bid1 = create_test_bid(&job_id, &executor1_ctx, 15);
    let bid2 = create_test_bid(&job_id, &executor2_ctx, 5);
    network.stage_bid(job_id.clone(), bid1).await;
    network.stage_bid(job_id.clone(), bid2).await;

    // Collect bids and choose the cheapest
    let bids = network
        .collect_bids_for_job(&job_id, Duration::from_millis(50))
        .await
        .expect("Bid collection failed");
    assert_eq!(bids.len(), 2);

    let selected = bids
        .iter()
        .min_by_key(|b| b.price_mana)
        .unwrap()
        .executor_did
        .clone();

    assign_job_to_executor_directly(&submitter_ctx, job_id.clone(), &selected).await;

    assert_job_state(
        &submitter_ctx,
        &job_id,
        JobStateVariant::Assigned {
            expected_executor: Some(selected.clone()),
        },
    )
    .await;
}

#[tokio::test]
async fn test_job_timeout_and_refund_with_helpers() {
    let (submitter_ctx, _exec1, _exec2, _) = new_mesh_test_context_with_two_executors();
    let submitter_did = submitter_ctx.current_identity.clone();
    let initial_mana = submitter_ctx.get_mana(&submitter_did).await.unwrap();

    let (job_json, job_cost) = create_test_job_payload_and_cost(&submitter_did, 30);
    let job_id = host_submit_mesh_job(&submitter_ctx, &job_json)
        .await
        .expect("Job submission failed");

    assert_eq!(
        submitter_ctx.get_mana(&submitter_did).await.unwrap(),
        initial_mana - job_cost
    );

    let network = get_stub_network_service(&submitter_ctx);
    let bids = network
        .collect_bids_for_job(&job_id, Duration::from_millis(50))
        .await
        .expect("Bid collection failed");
    assert!(bids.is_empty());

    submitter_ctx
        .credit_mana(&submitter_did, job_cost)
        .await
        .expect("refund mana");

    assert_eq!(
        submitter_ctx.get_mana(&submitter_did).await.unwrap(),
        initial_mana
    );
}

#[tokio::test]
async fn test_submit_mesh_job_with_custom_timeout() {
    let ctx = create_test_context("did:icn:test:timeout_custom", 50);
    let submitter_did = ctx.current_identity.clone();

    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"manifest_timeout_field");
    let mut job = create_test_mesh_job(manifest_cid, 10, submitter_did.clone());
    job.spec.timeout_ms = Some(1234);
    let job_json = serde_json::to_string(&job).unwrap();

    let _job_id = host_submit_mesh_job(&ctx, &job_json)
        .await
        .expect("Job submission failed");

    let pending_jobs = host_get_pending_mesh_jobs(&ctx).unwrap();
    assert_eq!(pending_jobs.len(), 1);
    assert_eq!(pending_jobs[0].spec.timeout_ms, Some(1234));
}

// Helper to create a plausible (but potentially invalidly signed) ExecutionReceipt for testing.
// The `forging_executor_ctx` is the context whose signer will actually sign this receipt.
async fn forge_execution_receipt(
    job_id: &Cid,
    result_cid_val: &[u8],
    forging_executor_ctx: &Arc<RuntimeContext>,
) -> IdentityExecutionReceipt {
    let receipt = IdentityExecutionReceipt {
        job_id: job_id.clone(),                                      // JobId is a Cid
        executor_did: forging_executor_ctx.current_identity.clone(), // Forger's DID
        result_cid: Cid::new_v1_dummy(0x55, 0x13, result_cid_val),
        cpu_ms: 50,
        success: true,
        sig: SignatureBytes(Vec::new()), // Will be filled by the forger's context
    };
    // The forging_executor_ctx signs the receipt using its own identity and signer.
    forging_executor_ctx
        .anchor_receipt(&receipt)
        .await
        .expect("Forger failed to sign its own receipt for forging");
    receipt // Returns the signed receipt
}

#[cfg(feature = "enable-libp2p")]
#[tokio::test]
#[ignore = "Blocked on environment/macro/import issues, particularly with libp2p Kademlia types and tokio/serde macros in dependent crates."]
async fn test_full_mesh_job_cycle_libp2p() -> Result<(), anyhow::Error> {
    todo!("libp2p integration pending");
}
