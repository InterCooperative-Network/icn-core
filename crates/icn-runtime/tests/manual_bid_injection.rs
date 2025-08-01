#![cfg(feature = "manual-bid-injection-tests")] // Disabled - needs refactoring for private method access

use icn_common::{Cid, Did};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobKind, JobSpec, MeshJobBid, Resources};
use icn_runtime::context::{
    LocalMeshSubmitReceiptMessage, MeshNetworkServiceType, RuntimeContext, StubMeshNetworkService,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Test demonstrating manual bid injection into StubMeshNetworkService
/// to test the complete mesh job lifecycle
#[tokio::test]
async fn test_manual_bid_injection_full_lifecycle() {
    // Initialize logging
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    println!("=== Manual Bid Injection Test ===");

    // Create a test runtime context with stub services
    let submitter_did =
        Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH").unwrap();
    let mut context = RuntimeContext::new_for_testing(submitter_did.clone(), Some(1000))
        .expect("Failed to create test context");

    // Get the stub network service for manual bid injection
    let stub_service = get_stub_network_service(&context);

    println!("✅ Test context created with StubMeshNetworkService");

    // Create a test job
    let test_job = create_test_echo_job(&submitter_did);
    let job_id = test_job.id.clone();

    println!("📝 Created test job: {:?}", job_id);

    // Submit the job to start the lifecycle
    let job_manager = context.clone();
    let job_handle =
        tokio::spawn(async move { RuntimeContext::handle_mesh_job_lifecycle(test_job).await });

    // Give the job manager a moment to announce the job
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify the job was announced
    let announced_jobs = stub_service.get_announced_jobs().await;
    assert_eq!(announced_jobs.len(), 1, "Job should be announced");
    assert_eq!(
        announced_jobs[0].id, job_id,
        "Announced job should match submitted job"
    );
    println!("📢 Job announced successfully");

    // Manually inject multiple bids for the job
    let test_bids = create_test_bids(&job_id);

    for (i, bid) in test_bids.iter().enumerate() {
        stub_service.stage_bid(job_id.clone(), bid.clone()).await;
        println!(
            "💰 Injected bid {}: {} mana from {}",
            i + 1,
            bid.price_mana,
            bid.executor_did
        );
    }

    println!("✅ Injected {} bids for job {:?}", test_bids.len(), job_id);

    // Wait a bit for bid collection to happen
    tokio::time::sleep(Duration::from_millis(100)).await;

    // The job manager should select the best executor (lowest price)
    let best_bid = test_bids.iter().min_by_key(|bid| bid.price_mana).unwrap();
    let selected_executor = &best_bid.executor_did;

    println!(
        "🎯 Expected selected executor: {} (price: {} mana)",
        selected_executor, best_bid.price_mana
    );

    // Create and inject an execution receipt from the selected executor
    let receipt = create_test_receipt(&job_id, selected_executor);
    let receipt_message = LocalMeshSubmitReceiptMessage {
        receipt: receipt.clone(),
    };

    stub_service
        .stage_receipt(job_id.clone(), receipt_message)
        .await;
    println!("📋 Injected execution receipt from {}", selected_executor);

    // Wait for the job lifecycle to complete
    let result = timeout(Duration::from_secs(10), job_handle).await;

    match result {
        Ok(Ok(())) => {
            println!("✅ Job lifecycle completed successfully!");
        }
        Ok(Err(e)) => {
            panic!("❌ Job lifecycle failed: {:?}", e);
        }
        Err(_) => {
            panic!("❌ Job lifecycle timed out");
        }
    }

    // Verify the assignment notice was sent
    let assignment_notices = stub_service.get_assignment_notices().await;
    assert_eq!(
        assignment_notices.len(),
        1,
        "Should have one assignment notice"
    );
    assert_eq!(
        assignment_notices[0].executor_did, *selected_executor,
        "Assignment should go to selected executor"
    );
    assert_eq!(
        assignment_notices[0].agreed_cost_mana, best_bid.price_mana,
        "Assignment should have agreed cost"
    );

    println!(
        "🎯 Assignment notice verified: executor={}, cost={} mana",
        assignment_notices[0].executor_did, assignment_notices[0].agreed_cost_mana
    );

    // Check final mana balances
    let submitter_balance = context
        .get_mana(&submitter_did)
        .await
        .expect("Failed to get submitter balance");
    let executor_balance = context
        .get_mana(selected_executor)
        .await
        .expect("Failed to get executor balance");

    println!("💰 Final balances:");
    println!(
        "  Submitter ({}): {} mana",
        submitter_did, submitter_balance
    );
    println!(
        "  Executor ({}): {} mana",
        selected_executor, executor_balance
    );

    // Submitter should have been charged
    assert!(submitter_balance < 1000, "Submitter should be charged mana");

    // Executor should have been paid (they start with 0, so should now have the agreed cost)
    assert_eq!(
        executor_balance, best_bid.price_mana,
        "Executor should be paid agreed amount"
    );

    println!("✅ Manual bid injection test completed successfully!");
}

/// Test with no bids to verify timeout behavior
#[tokio::test]
async fn test_no_bids_timeout() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    println!("=== No Bids Timeout Test ===");

    let submitter_did =
        Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH").unwrap();
    let context = RuntimeContext::new_for_testing(submitter_did.clone(), Some(1000))
        .expect("Failed to create test context");

    let test_job = create_test_echo_job(&submitter_did);
    let job_id = test_job.id.clone();

    println!("📝 Created test job: {:?}", job_id);

    // Submit job but don't inject any bids
    let job_manager = context.clone();
    let job_handle =
        tokio::spawn(async move { RuntimeContext::handle_mesh_job_lifecycle(test_job).await });

    // Wait for the job to timeout (should be quick since no bids)
    let result = timeout(Duration::from_secs(45), job_handle).await;

    match result {
        Ok(Ok(())) => {
            // This is expected - the job should complete (fail) when no bids are received
            println!("✅ Job lifecycle completed (no bids case)");
        }
        Ok(Err(e)) => {
            println!("✅ Job lifecycle failed as expected: {:?}", e);
        }
        Err(_) => {
            panic!("❌ Job lifecycle should not timeout");
        }
    }

    // Check that submitter's mana was refunded (since job failed due to no bids)
    let final_balance = context
        .get_mana(&submitter_did)
        .await
        .expect("Failed to get final balance");

    println!("💰 Final submitter balance: {} mana", final_balance);
    // Balance should be close to original (may be slightly less due to announcement cost)
    assert!(
        final_balance >= 990,
        "Submitter should get most mana back when no bids"
    );

    println!("✅ No bids timeout test completed successfully!");
}

/// Test with multiple jobs and selective bid injection
#[tokio::test]
async fn test_multiple_jobs_selective_bidding() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    println!("=== Multiple Jobs Selective Bidding Test ===");

    let submitter_did =
        Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH").unwrap();
    let context = RuntimeContext::new_for_testing(submitter_did.clone(), Some(2000))
        .expect("Failed to create test context");

    let stub_service = get_stub_network_service(&context);

    // Create two test jobs
    let job1 = create_test_echo_job(&submitter_did);
    let job2 = create_test_echo_job(&submitter_did);
    let job1_id = job1.id.clone();
    let job2_id = job2.id.clone();

    println!("📝 Created test jobs: {:?} and {:?}", job1_id, job2_id);

    // Start both jobs
    let context1 = context.clone();
    let context2 = context.clone();

    let job1_handle =
        tokio::spawn(async move { RuntimeContext::handle_mesh_job_lifecycle(job1).await });

    let job2_handle =
        tokio::spawn(async move { RuntimeContext::handle_mesh_job_lifecycle(job2).await });

    // Wait for announcements
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create bids only for job1
    let job1_bids = create_test_bids(&job1_id);
    for bid in job1_bids.iter() {
        stub_service.stage_bid(job1_id.clone(), bid.clone()).await;
    }

    // Create receipt for job1
    let best_bid = job1_bids.iter().min_by_key(|bid| bid.price_mana).unwrap();
    let receipt = create_test_receipt(&job1_id, &best_bid.executor_did);
    stub_service
        .stage_receipt(
            job1_id.clone(),
            LocalMeshSubmitReceiptMessage {
                receipt: receipt.clone(),
            },
        )
        .await;

    println!("💰 Injected bids and receipt for job1 only");

    // Wait for completion
    let results = tokio::join!(
        timeout(Duration::from_secs(15), job1_handle),
        timeout(Duration::from_secs(45), job2_handle)
    );

    match results.0 {
        Ok(Ok(())) => println!("✅ Job1 completed successfully (with bids)"),
        _ => panic!("❌ Job1 should have completed successfully"),
    }

    match results.1 {
        Ok(Ok(())) => println!("✅ Job2 completed (no bids case)"),
        Ok(Err(_)) => println!("✅ Job2 failed as expected (no bids)"),
        Err(_) => panic!("❌ Job2 should not timeout"),
    }

    println!("✅ Multiple jobs selective bidding test completed successfully!");
}

// Helper functions

fn get_stub_network_service(context: &Arc<RuntimeContext>) -> Arc<StubMeshNetworkService> {
    match &*context.mesh_network_service {
        MeshNetworkServiceType::Stub(stub_service) => Arc::new(stub_service.clone()),
        _ => panic!("Expected StubMeshNetworkService, but got production service"),
    }
}

fn create_test_echo_job(submitter_did: &Did) -> ActualMeshJob {
    ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job")),
        manifest_cid: Cid::from_str("bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e")
            .unwrap(),
        creator_did: submitter_did.clone(),
        cost_mana: 50,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
        spec: JobSpec {
            kind: JobKind::Echo {
                payload: "Hello from manual bid test!".to_string(),
            },
            inputs: vec![],
            outputs: vec![],
            required_resources: icn_mesh::Resources {
                cpu_cores: 1,
                memory_mb: 100,
                storage_mb: 0,
            },
        },
    }
}

fn create_test_bids(job_id: &JobId) -> Vec<MeshJobBid> {
    vec![
        MeshJobBid {
            job_id: job_id.clone(),
            executor_did: Did::from_str("did:key:z6MkrJvwAfLVgFntBzYCBLXXNGMNPdpJcRw4Qc9vq8vN8oSz")
                .unwrap(),
            price_mana: 30, // Lowest price - should be selected
            resources: Resources {
                cpu_cores: 1,
                memory_mb: 100,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        },
        MeshJobBid {
            job_id: job_id.clone(),
            executor_did: Did::from_str("did:key:z6MkoTHsgNNrby8JzCNQ1iRLyW5QQ6R8Xuu6AA8igGrMVPUM")
                .unwrap(),
            price_mana: 40,
            resources: Resources {
                cpu_cores: 1,
                memory_mb: 120,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        },
        MeshJobBid {
            job_id: job_id.clone(),
            executor_did: Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktM")
                .unwrap(),
            price_mana: 35,
            resources: Resources {
                cpu_cores: 2,
                memory_mb: 150,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        },
    ]
}

fn create_test_receipt(job_id: &JobId, executor_did: &Did) -> ExecutionReceipt {
    ExecutionReceipt {
        job_id: Cid::from(job_id.clone()),
        executor_did: executor_did.clone(),
        result_cid: Cid::from_str("bafybeigdyrztktx5b5m2y4sogf2hf5uq3k5knv5c5k2pvx7aq5w3sh7g5e")
            .unwrap(),
        cpu_ms: 150,
        success: true,
        sig: SignatureBytes(vec![]),
    }
}
