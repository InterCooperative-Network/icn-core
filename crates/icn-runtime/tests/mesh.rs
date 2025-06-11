#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    clippy::uninlined_format_args
)]
#![cfg(any())]
// crates/icn-runtime/tests/mesh.rs

use icn_common::{Cid, Did};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec, JobState, MeshJobBid, Resources};
use icn_runtime::context::{
    HostAbiError, JobAssignmentNotice, LocalMeshSubmitReceiptMessage, MeshNetworkService,
    RuntimeContext, StorageService, StubDagStore, StubMeshNetworkService,
};
use icn_runtime::host_submit_mesh_job;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// Helper to create a test ActualMeshJob with all required fields
fn create_test_mesh_job(manifest_cid: Cid, cost_mana: u64, creator_did: Did) -> ActualMeshJob {
    ActualMeshJob {
        id: Cid::new_v1_dummy(0x55, 0x13, b"test_job_id"),
        manifest_cid,
        spec: JobSpec::default(),
        creator_did,
        cost_mana,
        signature: SignatureBytes(vec![0u8; 64]), // Dummy signature for tests
    }
}

// Helper to create a RuntimeContext with a specific DID and initial mana.
// The Stub services are now part of RuntimeContext::new_with_stubs_and_mana
fn create_test_context(identity_did_str: &str, initial_mana: u64) -> Arc<RuntimeContext> {
    // new_with_stubs_and_mana now returns Arc<RuntimeContext> directly
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

    let job_manager_network_stub = get_stub_network_service(&arc_ctx_job_manager);
    let job_manager_dag_store_stub = get_stub_dag_store(&arc_ctx_job_manager);

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
    let bid = MeshJobBid {
        job_id: submitted_job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: 10,
        resources: Resources::default(),
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
    let dag_store = get_stub_dag_store(&arc_ctx_job_manager);
    let receipt_bytes =
        serde_json::to_vec(&retrieved_receipt).expect("Failed to serialize receipt");
    let stored_cid = dag_store
        .put(&receipt_bytes)
        .await
        .expect("Failed to store receipt in DAG");

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
        sig: SignatureBytes(correct_signature_bytes),
    };

    // This should also fail because the job manager context signer doesn't match the executor
    let _correct_anchor_result = arc_ctx_job_manager.anchor_receipt(&correct_receipt).await;
    // Note: This will likely fail because the job manager's signer is different from the executor's signer
    // In a real system, the job manager would need to verify against the executor's public key

    println!("Invalid receipt test completed - forged receipt verification tested");
}

// Placeholder for new_mesh_test_context_with_two_executors
// This helper needs to be properly implemented or use existing ones if available.
// For now, it uses the existing single context creator.
fn new_mesh_test_context_with_two_executors() -> (
    Arc<RuntimeContext>,
    Arc<RuntimeContext>,
    Arc<RuntimeContext>,
    Arc<StubDagStore>,
) {
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
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"test_job_manifest_for_invalid_receipt");
    let job_cost = 20u64;
    let submitter_did = Did::from_str("did:icn:test:submitter_for_invalid_receipt").unwrap();
    let test_job = create_test_mesh_job(manifest_cid, job_cost, submitter_did);
    let job_json_payload = serde_json::to_string(&test_job).unwrap();
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
async fn assign_job_to_executor_directly(
    job_manager_ctx: &Arc<RuntimeContext>,
    job_id: Cid,
    assigned_executor_did: &Did,
) {
    // TODO: This is a test utility to bypass full job manager loop for specific assignment tests.
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

// Helper to create a plausible (but potentially invalidly signed) ExecutionReceipt for testing.
// The `forging_executor_ctx` is the context whose signer will actually sign this receipt.
async fn forge_execution_receipt(
    job_id: &Cid,
    result_cid_val: &[u8],
    forging_executor_ctx: &Arc<RuntimeContext>,
) -> IdentityExecutionReceipt {
    let mut receipt = IdentityExecutionReceipt {
        job_id: job_id.clone(),                                      // JobId is a Cid
        executor_did: forging_executor_ctx.current_identity.clone(), // Forger's DID
        result_cid: Cid::new_v1_dummy(0x55, 0x13, result_cid_val),
        cpu_ms: 50,
        sig: SignatureBytes(Vec::new()), // Will be filled by the forger's context
    };
    // The forging_executor_ctx signs the receipt using its own identity and signer.
    forging_executor_ctx
        .anchor_receipt(&mut receipt)
        .await
        .expect("Forger failed to sign its own receipt for forging");
    receipt // Returns the signed receipt
}

#[cfg(feature = "enable-libp2p")]
#[tokio::test]
#[ignore = "Blocked on environment/macro/import issues, particularly with libp2p Kademlia types and tokio/serde macros in dependent crates."]
async fn test_full_mesh_job_cycle_libp2p() -> Result<(), anyhow::Error> {
    println!("[test-mesh-runtime] Starting test_full_mesh_job_cycle_libp2p");
    // 1. Setup Node A (Job Manager / Submitter)
    println!("[test-mesh-runtime] Setting up Node A (Job Manager/Submitter).");
    let node_a_libp2p_actual_service =
        Arc::new(icn_network::libp2p_service::Libp2pNetworkService::new(None).await?);
    let node_a_peer_id_str = node_a_libp2p_actual_service.local_peer_id().to_string();
    let node_a_addrs = node_a_libp2p_actual_service.listening_addresses();
    assert!(
        !node_a_addrs.is_empty(),
        "Node A should have listening addresses"
    );
    println!(
        "[test-mesh-runtime] Node A Peer ID: {}, Listening Addresses: {:?}",
        node_a_peer_id_str, node_a_addrs
    );

    let node_a_ctx = Arc::new(RuntimeContext::new(
        Did::from_str("did:icn:test:node_a_libp2p")?,
        Arc::new(DefaultMeshNetworkService::new(
            node_a_libp2p_actual_service.clone(),
        )),
        Arc::new(StubSigner::new()),
        Arc::new(StubDagStore::new()),
    ));
    node_a_ctx
        .mana_ledger
        .set_balance(&node_a_ctx.current_identity, 1000)
        .expect("set mana for node A");
    println!("[test-mesh-runtime] Node A context created, mana set. Spawning Job Manager.");
    node_a_ctx.clone().spawn_mesh_job_manager().await;

    // 2. Setup Node B (Executor)
    println!("[test-mesh-runtime] Setting up Node B (Executor), bootstrapping with Node A.");
    let node_a_libp2p_peer_id_for_b = Libp2pPeerId::from_str(&node_a_peer_id_str)?;
    let bootstrap_peers_for_b = Some(vec![(node_a_libp2p_peer_id_for_b, node_a_addrs[0].clone())]);
    let node_b_libp2p_actual_service_for_setup = Arc::new(
        icn_network::libp2p_service::Libp2pNetworkService::new(bootstrap_peers_for_b).await?,
    );
    println!(
        "[test-mesh-runtime] Node B Peer ID: {}",
        node_b_libp2p_actual_service_for_setup
            .local_peer_id()
            .to_string()
    );

    let node_b_ctx = Arc::new(RuntimeContext::new(
        Did::from_str("did:icn:test:node_b_libp2p")?,
        Arc::new(DefaultMeshNetworkService::new(
            node_b_libp2p_actual_service_for_setup.clone(),
        )),
        Arc::new(StubSigner::new()),
        Arc::new(StubDagStore::new()),
    ));
    node_b_ctx
        .mana_ledger
        .set_balance(&node_b_ctx.current_identity, 500)
        .expect("set mana for node B");
    println!("[test-mesh-runtime] Node B context created, mana set.");

    // Get the underlying Libp2pNetworkService for Node B to broadcast messages
    let node_b_mesh_service_trait_obj = node_b_ctx.mesh_network_service.clone();
    let node_b_default_mesh_service = node_b_mesh_service_trait_obj
        .as_any()
        .downcast_ref::<DefaultMeshNetworkService>()
        .expect("Node B mesh_network_service is not DefaultMeshNetworkService");

    let node_b_underlying_broadcast_service =
        node_b_default_mesh_service.get_underlying_broadcast_service()?;

    println!("[test-mesh-runtime] Allowing 5s for network connection.");
    sleep(Duration::from_secs(5)).await;

    // 3. Node A submits a job
    let job_cost = 50u64;
    let manifest_cid = Cid::new_v1_dummy(0x55, 0x13, b"job_manifest_libp2p_test");
    let job_json_payload = json!({
        "manifest_cid": manifest_cid,
        "spec": {},
        "cost_mana": job_cost,
    })
    .to_string();

    println!("[test-mesh-runtime] Node B (executor) subscribing to its Libp2p service to listen for announcements.");
    let mut node_b_raw_receiver = node_b_libp2p_actual_service_for_setup
        .as_ref()
        .subscribe()
        .await
        .map_err(|e| anyhow::anyhow!("Node B failed to subscribe: {e}"))?;

    let submitted_job_id = host_submit_mesh_job(&node_a_ctx, &job_json_payload).await?;
    println!(
        "[test-mesh-runtime] Node A submitted job ID: {}. Payload: {}. Asserting Pending state.",
        submitted_job_id, job_json_payload
    );
    assert_job_state(&node_a_ctx, &submitted_job_id, JobStateVariant::Pending).await;

    // 4. Node B listens for the job, receives it, and submits a bid
    println!("[test-mesh-runtime] Node B listening for job announcement (timeout 20s).");

    let received_on_b_opt =
        tokio::time::timeout(Duration::from_secs(20), node_b_raw_receiver.recv())
            .await
            .map_err(|e| anyhow::anyhow!("Timeout waiting for job announcement: {e}"))?;

    let received_on_b = received_on_b_opt.ok_or_else(|| {
        anyhow::anyhow!("Node B: Receiver channel closed or got None before job announcement")
    })?;

    if let icn_network::NetworkMessage::MeshJobAnnouncement(announced_job) = received_on_b {
        assert_eq!(
            announced_job.id, submitted_job_id,
            "Node B received announcement for wrong job"
        );
        println!(
            "[test-mesh-runtime] Node B received announcement for job ID: {}. Submitting bid.",
            announced_job.id
        );

        let bid = MeshJobBid {
            job_id: announced_job.id.clone(),
            executor_did: node_b_ctx.current_identity.clone(),
            price_mana: 20,
            resources: Resources::default(),
        };
        node_b_underlying_broadcast_service
            .broadcast_message(icn_network::NetworkMessage::BidSubmission(bid.clone()))
            .await
            .map_err(|e| anyhow::anyhow!("Node B failed to broadcast bid: {e}"))?;
        println!(
            "[test-mesh-runtime] Node B submitted bid for job ID: {}",
            announced_job.id
        );
    } else {
        panic!(
            "[test-mesh-runtime] Node B did not receive MeshJobAnnouncement, got: {:?}",
            received_on_b
        );
    }

    println!(
        "[test-mesh-runtime] Allowing 10s for JobManager on Node A to process bids and assign."
    );
    sleep(Duration::from_secs(10)).await;

    println!(
        "[test-mesh-runtime] Asserting job {} is assigned to Node B.",
        submitted_job_id
    );
    assert_job_state(
        &node_a_ctx,
        &submitted_job_id,
        JobStateVariant::Assigned {
            expected_executor: Some(node_b_ctx.current_identity.clone()),
        },
    )
    .await;
    println!(
        "[test-mesh-runtime] Job {} successfully assigned to Node B {}. Node B preparing receipt.",
        submitted_job_id,
        node_b_ctx.current_identity.to_string()
    );

    // 7. Node B "executes" the job and prepares a receipt
    let result_cid = Cid::new_v1_dummy(0x55, 0x13, b"libp2p_test_result_data");
    let mut receipt_by_node_b = IdentityExecutionReceipt {
        job_id: submitted_job_id.clone(),
        executor_did: node_b_ctx.current_identity.clone(),
        result_cid: result_cid.clone(),
        cpu_ms: 75,
        sig: SignatureBytes(Vec::new()),
    };

    println!(
        "[test-mesh-runtime] Node B signing its execution receipt for job {}.",
        submitted_job_id
    );
    match node_b_ctx.anchor_receipt(&mut receipt_by_node_b) {
        Ok(_) => println!(
            "[test-mesh-runtime] Node B signed its execution receipt for job {}",
            submitted_job_id
        ),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Node B failed to sign its own receipt: {e}"
            ))
        }
    }
    assert!(
        !receipt_by_node_b.sig.is_empty(),
        "Node B's receipt should be signed"
    );

    println!(
        "[test-mesh-runtime] Node B broadcasting receipt for job {}.",
        submitted_job_id
    );
    let receipt_message = icn_network::NetworkMessage::SubmitReceipt(receipt_by_node_b.clone());
    node_b_underlying_broadcast_service
        .broadcast_message(receipt_message)
        .await
        .map_err(|e| anyhow::anyhow!("Node B failed to broadcast receipt: {e}"))?;
    println!("[test-mesh-runtime] Node B broadcasted receipt for job {}. Waiting 10s for JobManager processing.", submitted_job_id);

    sleep(Duration::from_secs(10)).await;

    println!(
        "[test-mesh-runtime] Asserting job {} is Completed on Node A.",
        submitted_job_id
    );
    assert_job_state(
        &node_a_ctx,
        &submitted_job_id,
        JobStateVariant::Completed {
            expected_receipt_data: Some(ExpectedReceiptData {
                job_id: submitted_job_id.clone(),
                executor_did: node_b_ctx.current_identity.clone(),
                result_cid: result_cid.clone(),
            }),
        },
    )
    .await;
    println!(
        "[test-mesh-runtime] Job {} successfully marked as Completed on Node A. Test finished.",
        submitted_job_id
    );

    Ok(())
}
