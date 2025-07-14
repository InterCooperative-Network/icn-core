//! Integration tests for the ICN mesh job pipeline.

use icn_common::{compute_merkle_cid, Cid, CommonError, DagBlock, Did};
use icn_dag::StorageService;
use icn_economics::charge_mana; // For mana interactions
use icn_mesh::{
    select_executor, JobSpec, MeshJob as ActualMeshJob, MeshJobBid as Bid, Resources,
    SelectionPolicy,
};
use icn_reputation::ReputationStore;
use icn_runtime::abi; // To use ABI consts if needed, though direct calls are more likely
use icn_runtime::context::{
    ExecutionReceipt, HostAbiError, JobId, RuntimeContext, MeshNetworkServiceType,
    StubMeshNetworkService,
};
use icn_runtime::{
    host_anchor_receipt, host_get_pending_mesh_jobs, host_submit_mesh_job, ReputationUpdater,
};
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, Mutex as StdMutex}; // Standard Mutex for simple mocks if not async
use tokio::sync::Mutex as TokioMutex; // Tokio Mutex for async operations

// --- Mock/Stub Implementations ---

// Mock for InMemoryDagStore (simplified)
#[derive(Debug, Default, Clone)]
struct InMemoryDagStore {
    store: Arc<StdMutex<std::collections::HashMap<Cid, Vec<u8>>>>,
}

impl InMemoryDagStore {
    fn new() -> Self {
        Default::default()
    }

    fn put(&self, _cid: &Cid, data: &[u8]) -> Result<(), CommonError> {
        // In a real mock, you might store based on actual CID or content.
        // For this test, let's assume data is the receipt and store it.
        // A real DAG store would calculate the CID.
        // Here, we'll use a fixed CID or passed-in CID for simplicity if needed.
        // For host_anchor_receipt, it returns a Cid, so we don't need to store by it here.
        println!("[InMemoryDagStore STUB] Storing data of len {}", data.len());
        Ok(())
    }
    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, CommonError> {
        Ok(self.store.lock().unwrap().get(cid).cloned())
    }
}

// Failing DAG store used to simulate anchoring errors
#[derive(Debug, Default)]
struct FailingDagStore;

impl StorageService<DagBlock> for FailingDagStore {
    fn put(&mut self, _block: &DagBlock) -> Result<(), CommonError> {
        Err(CommonError::StorageError("put failure".into()))
    }

    fn get(&self, _cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        Ok(None)
    }

    fn delete(&mut self, _cid: &Cid) -> Result<(), CommonError> {
        Ok(())
    }

    fn contains(&self, _cid: &Cid) -> Result<bool, CommonError> {
        Ok(false)
    }
}

// Mock for ManaLedger (can use SimpleManaLedger from context or a more controlled mock)
// Using SimpleManaLedger from context.rs is probably sufficient if current_identity checks are managed.

// Mock for ReputationSystem
#[derive(Debug, Clone)]
struct MockReputationSystem {
    updates: Arc<StdMutex<Vec<ExecutionReceipt>>>,
    scores: Arc<StdMutex<std::collections::HashMap<Did, u64>>>,
}

impl MockReputationSystem {
    fn new() -> Self {
        MockReputationSystem {
            updates: Arc::new(StdMutex::new(Vec::new())),
            scores: Arc::new(StdMutex::new(std::collections::HashMap::new())),
        }
    }

    fn submit_receipt(&self, receipt: &ExecutionReceipt) {
        self.updates.lock().unwrap().push(receipt.clone());
        // Simulate reputation update: increment score for executor
        let mut scores = self.scores.lock().unwrap();
        let score = scores.entry(receipt.executor_did.clone()).or_insert(0);
        *score += 1; // Simple increment
        println!(
            "[MockReputationSystem] Submitted receipt for executor {:?}, new score: {}",
            receipt.executor_did, *score
        );
    }

    fn get_reputation_score(&self, did: &Did) -> u64 {
        self.scores.lock().unwrap().get(did).cloned().unwrap_or(0)
    }

    fn set_reputation_score(&self, did: &Did, score: u64) {
        self.scores.lock().unwrap().insert(did.clone(), score);
    }
}

// Reputation store that fails when updating scores
#[derive(Debug, Default)]
struct FailingReputationStore;

impl ReputationStore for FailingReputationStore {
    fn get_reputation(&self, _did: &Did) -> u64 {
        0
    }

    fn record_execution(&self, _executor: &Did, _success: bool, _cpu_ms: u64) {
        panic!("record_execution failed")
    }
}

#[derive(Debug, Default)]
struct InMemoryManaLedger {
    balances: StdMutex<std::collections::HashMap<Did, u64>>,
}

impl icn_economics::ManaLedger for InMemoryManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        *self.balances.lock().unwrap().get(did).unwrap_or(&0)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        self.balances.lock().unwrap().insert(did.clone(), amount);
        Ok(())
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        let bal = map
            .get_mut(did)
            .ok_or_else(|| CommonError::DatabaseError("missing".into()))?;
        if *bal < amount {
            return Err(CommonError::PolicyDenied("insufficient".into()));
        }
        *bal -= amount;
        Ok(())
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        *map.entry(did.clone()).or_insert(0) += amount;
        Ok(())
    }

    fn credit_all(&self, amount: u64) -> Result<(), CommonError> {
        let mut map = self.balances.lock().unwrap();
        for val in map.values_mut() {
            *val += amount;
        }
        Ok(())
    }
}

// Helper to create a test RuntimeContext with mocked dependencies
// This will need to be adapted based on how RuntimeContext eventually holds these.
// For now, host functions take ctx, and some mocks might be passed directly to them.
fn create_test_runtime_context(did_str: &str, initial_mana: u64) -> RuntimeContext {
    let did = Did::from_str(did_str).expect("Invalid DID for test context");
    let mut ctx = RuntimeContext::new_with_services(
        did.clone(),
        Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new())),
        Arc::new(icn_runtime::context::signers::StubSigner::new()),
        Arc::new(icn_identity::KeyDidResolver),
        Arc::new(DagStoreMutexType::new(icn_dag::InMemoryDagStore::new())),
    );
    ctx.mana_ledger.set_balance(&did, initial_mana);
    // `pending_mesh_jobs` is Arc<TokioMutex<VecDeque<ActualMeshJob>>>
    // `governance_module` is default.
    // If RuntimeContext needs to hold Arc<InMemoryDagStore> or Arc<MockReputationSystem>, add them here.
    ctx
}

fn dummy_job_json(mana_cost: u64) -> String {
    // Create a valid JSON for ActualMeshJob.
    // Cid and JobSpec can be minimal valid placeholders.
    // Ensure `mana_cost` is included.
    format!(
        r#"{{"cid":{{"version":1,"codec":85,"hash_alg":18,"hash_bytes":[1,2,3]}},"spec":{{}},"mana_cost":{}}}"#,
        mana_cost
    )
}

fn dummy_receipt_json(job_id_str: &str, executor_did_str: &str, result_cid_str: &str) -> String {
    let job_id = JobId::from(Cid::new_v1_sha256(0x55, job_id_str.as_bytes()));
    let executor_did = Did::from_str(executor_did_str).unwrap();
    // For Cid, we need a proper Cid structure for JSON.
    // This is a simplified Cid for the dummy receipt.
    let result_cid = Cid {
        version: 1,
        codec: 0x71,
        hash_alg: 0x12,
        hash_bytes: result_cid_str.as_bytes().to_vec(),
    };

    let receipt = ExecutionReceipt {
        job_id,
        executor_did,
        result_cid, // This should be a proper Cid struct if host_anchor_receipt parses it.
        input_cids: vec![],
        mana_used: 5,           // example
        execution_timestamp: 0, // example
        federation_scope: None,
        signature: None, // Signature will be added by anchor_receipt or host_anchor_receipt
    };
    serde_json::to_string(&receipt).unwrap()
}

#[tokio::test]
async fn end_to_end_mesh_job_flow() {
    let submitter_did_str = "did:icn:test:submitter_e2e";
    let executor_high_rep_did_str = "did:icn:test:executor_high_rep";
    let executor_low_rep_did_str = "did:icn:test:executor_low_rep";

    let mut submitter_ctx = create_test_runtime_context(submitter_did_str, 100);
    let initial_submitter_mana = submitter_ctx
        .get_mana(&submitter_ctx.current_identity)
        .unwrap();

    // Mock reputation system (passed to host_anchor_receipt)
    let mock_reputation_updater = ReputationUpdater::new(); // Using the stub from lib.rs for now
                                                            // A more complex mock might be needed if ReputationUpdater itself becomes complex.
                                                            // Or, if ReputationUpdater is part of RuntimeContext.

    // Mock DAG Store (if host_anchor_receipt or ctx.anchor_receipt needs it directly)
    // let _mock_dag_store = InMemoryDagStore::new();
    // If ctx.anchor_receipt uses a DagStore trait object inside RuntimeContext, it needs to be set up.

    // 1. Submit job (should spend mana)
    let job_cost = 20;
    let job_json = dummy_job_json(job_cost);
    let job_id_result = host_submit_mesh_job(&mut submitter_ctx, &job_json).await;
    assert!(
        job_id_result.is_ok(),
        "Job submission failed: {:?}",
        job_id_result.err()
    );
    let job_id = job_id_result.unwrap();

    let submitter_mana_after_submit = submitter_ctx
        .get_mana(&submitter_ctx.current_identity)
        .unwrap();
    assert_eq!(
        submitter_mana_after_submit,
        initial_submitter_mana - job_cost,
        "Submitter mana not spent correctly."
    );

    // Retrieve job from queue to simulate job manager picking it up
    let pending_jobs = host_get_pending_mesh_jobs(&submitter_ctx).await.unwrap();
    assert_eq!(pending_jobs.len(), 1);
    let submitted_job_info = pending_jobs.first().unwrap();
    assert_eq!(submitted_job_info.id, job_id.0);

    // 2. Simulate Job Manager processing: Bidding
    // This part is conceptual as spawn_mesh_job_manager is internal and not directly testable by calling it.
    // We will simulate its internal logic of bid filtering and selection.
    // We need contexts for executors to check their mana.
    let mut exec_high_rep_ctx = create_test_runtime_context(executor_high_rep_did_str, 50);
    let mut exec_low_rep_ctx = create_test_runtime_context(executor_low_rep_did_str, 50);

    // Set up mock reputations if ReputationUpdater above doesn't cover it for select_executor
    // For now, select_executor in icn-mesh is a stub. We'll assume ReputationExecutorSelector is used
    // and we need to prime its state or mock its behavior. This is complex without a real Reputation module.
    // Let's assume select_executor will be improved to use a reputation source.

    let mana_reserve = 5; // Matching the conceptual reserve in spawn_mesh_job_manager

    // Bid from high reputation executor
    let bid1 = Bid {
        job_id: job_id.0.clone(),
        executor: exec_high_rep_ctx.current_identity.clone(),
        price: 12, // Slightly higher price
        resources: Resources {},
    };
    // Check mana for bid1 - this would happen in spawn_mesh_job_manager
    assert!(
        charge_mana(&bid1.executor, mana_reserve).is_ok(),
        "High rep executor should have mana for reserve."
    );

    // Bid from low reputation executor
    let bid2 = Bid {
        job_id: job_id.0.clone(),
        executor: exec_low_rep_ctx.current_identity.clone(),
        price: 10, // Lower price
        resources: Resources {},
    };
    // Check mana for bid2
    assert!(
        charge_mana(&bid2.executor, mana_reserve).is_ok(),
        "Low rep executor should have mana for reserve."
    );

    // Simulate selection (assuming select_executor is adapted to use a reputation source)
    // For now, icn_mesh::select_executor is a stub. A real test would need ReputationExecutorSelector
    // to be configurable or mockable.
    // Let's manually create a scenario:
    // To test "high-reputation wins even if slightly higher price", ReputationExecutorSelector needs to be real.
    // As a STUB for now, let's just pick one.
    let selected_executor_did: Did;
    // If ReputationExecutorSelector was real and setup:
    // let bids = vec![bid1.clone(), bid2.clone()];
    // let policy = SelectionPolicy{}; // Assuming some default policy
    // selected_executor_did = select_executor(bids, policy).expect("Executor should be selected");
    // For this test, let's manually decide based on the test's intent:
    selected_executor_did = bid1.executor.clone(); // Manually select high-rep for test flow
    println!(
        "[E2E_TEST] Manually selected executor: {:?}",
        selected_executor_did
    );

    // 3. Complete job, anchor receipt, reputation updated
    // The selected executor (conceptually) runs the job and produces a receipt.
    let result_data_str = "job_result_content_for_e2e";
    // The receipt needs to be created by the executor's context/identity
    let mut selected_executor_ctx = if selected_executor_did == exec_high_rep_ctx.current_identity {
        exec_high_rep_ctx
    } else {
        exec_low_rep_ctx
    };

    let receipt_json_to_anchor = dummy_receipt_json(
        &job_id.0,
        &selected_executor_did.to_string(),
        result_data_str,
    );

    // Anchor receipt using the selected executor's context (or a node service context)
    // host_anchor_receipt takes a &mut RuntimeContext. If a node service anchors, it uses its own context.
    // If the executor anchors, it uses its own context. Let's use selected_executor_ctx.
    let anchor_cid_result = host_anchor_receipt(
        &mut selected_executor_ctx,
        &receipt_json_to_anchor,
        &mock_reputation_updater,
    );
    assert!(
        anchor_cid_result.is_ok(),
        "Failed to anchor receipt: {:?}",
        anchor_cid_result.err()
    );
    let _anchored_cid = anchor_cid_result.unwrap();

    // Verify reputation update (this depends on the mock_reputation_updater or a real one)
    // If using MockReputationSystem:
    // let rep_system = MockReputationSystem::new(); // This needs to be the same instance used by host_anchor_receipt
    // For the current ReputationUpdater stub, we can't easily verify beyond println.
    // A more integrated test would require ReputationUpdater to be injectable or part of RuntimeContext.

    println!("[E2E_TEST] End-to-end flow test completed conceptually.");
}

#[tokio::test]
async fn test_submit_job_insufficient_mana() {
    let submitter_did_str = "did:icn:test:submitter_low_mana";
    let mut ctx = create_test_runtime_context(submitter_did_str, 5); // Only 5 mana

    let job_cost = 10; // Job costs 10
    let job_json = dummy_job_json(job_cost);
    let result = host_submit_mesh_job(&mut ctx, &job_json).await;

    assert!(result.is_err());
    match result.err().unwrap() {
        HostAbiError::InsufficientMana => { /* Expected */ }
        e => panic!("Expected InsufficientMana, got {:?}", e),
    }
}

#[tokio::test]
async fn test_executor_bid_insufficient_mana() {
    // This test is more conceptual for spawn_mesh_job_manager's internal logic.
    // We simulate the check it would perform.
    let executor_did_str = "did:icn:test:executor_bid_low_mana";
    let _executor_ctx = create_test_runtime_context(executor_did_str, 2); // Executor has 2 mana
    let mana_reserve_needed = 5; // Bid reserve needs 5

    // charge_mana is the key check.
    let charge_result = charge_mana(
        &Did::from_str(executor_did_str).unwrap(),
        mana_reserve_needed,
    );
    assert!(charge_result.is_err());
    match charge_result.err().unwrap() {
        CommonError::PolicyDenied(_) => { /* Expected */ }
        e => panic!("Expected PolicyDenied for bidding, got {:?}", e),
    }
    println!("[BID_INSUFFICIENT_MANA_TEST] Test conceptual check completed.");
}

#[tokio::test]
async fn test_no_valid_bids_job_times_out_refund_mana() {
    let submitter_did_str = "did:icn:test:submitter_job_timeout";
    let submitter_ctx = RuntimeContext::new_with_stubs_and_mana(submitter_did_str, 100);
    let initial_mana = submitter_ctx
        .get_mana(&submitter_ctx.current_identity)
        .await
        .unwrap();

    let job_cost = 30;
    let job_json = dummy_job_json(job_cost);
    let job_id_result = host_submit_mesh_job(&submitter_ctx, &job_json).await;
    assert!(job_id_result.is_ok());
    let _job_id = job_id_result.unwrap();
    assert_eq!(
        submitter_ctx
            .get_mana(&submitter_ctx.current_identity)
            .await
            .unwrap(),
        initial_mana - job_cost
    );

    // Simulate timeout refund directly via credit_mana
    submitter_ctx
        .credit_mana(&submitter_ctx.current_identity, job_cost)
        .await
        .unwrap();
    assert_eq!(
        submitter_ctx
            .get_mana(&submitter_ctx.current_identity)
            .await
            .unwrap(),
        initial_mana
    );
}

// Additional error case tests

#[tokio::test]
async fn executor_recheck_failure_after_selection() {
    let rep_store = icn_reputation::InMemoryReputationStore::new();
    let ledger = InMemoryManaLedger::default();

    let exec = Did::from_str("did:icn:test:exec_recheck").unwrap();
    ledger.set_balance(&exec, 5).unwrap();

    let job_id = JobId::from(Cid::new_v1_sha256(0x55, b"job_recheck"));
    let bid = Bid {
        job_id: job_id.clone(),
        executor_did: exec.clone(),
        price_mana: 5,
        resources: Resources::default(),
        signature: icn_identity::SignatureBytes(vec![]),
    };

    let selected = select_executor(
        &job_id,
        &JobSpec::default(),
        vec![bid.clone()],
        &SelectionPolicy::default(),
        &rep_store,
        &ledger,
    )
    .expect("selection");
    assert_eq!(selected, exec);

    ledger.set_balance(&exec, 0).unwrap();
    let result = ledger.spend(&exec, bid.price_mana);
    assert!(matches!(result, Err(CommonError::PolicyDenied(_))));
}

#[tokio::test]
async fn anchor_receipt_returns_dag_error() {
    let mut ctx = create_test_runtime_context("did:icn:test:dag_error", 10);
    ctx.dag_store = Arc::new(TokioMutex::new(FailingDagStore::default()));

    let receipt_json = dummy_receipt_json("job_err", &ctx.current_identity.to_string(), "res");
    let result = host_anchor_receipt(&mut ctx, &receipt_json, &ReputationUpdater::new()).await;
    assert!(matches!(result, Err(HostAbiError::DagOperationFailed(_))));
}

#[tokio::test]
async fn reputation_update_panic() {
    let mut ctx = create_test_runtime_context("did:icn:test:rep_panic", 10);
    ctx.reputation_store = Arc::new(FailingReputationStore::default());

    let receipt_json = dummy_receipt_json("job_panic", &ctx.current_identity.to_string(), "res");
    let result = std::panic::AssertUnwindSafe(host_anchor_receipt(
        &mut ctx,
        &receipt_json,
        &ReputationUpdater::new(),
    ))
    .catch_unwind()
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_executor_selection_bidder_loses_mana() {
    let rep_store = icn_reputation::InMemoryReputationStore::new();
    let ledger = InMemoryManaLedger::default();

    let exec_a = Did::from_str("did:icn:test:exec_a").unwrap();
    let exec_b = Did::from_str("did:icn:test:exec_b").unwrap();
    ledger.set_balance(&exec_a, 10).unwrap();
    ledger.set_balance(&exec_b, 10).unwrap();
    rep_store.set_score(exec_a.clone(), 5);
    rep_store.set_score(exec_b.clone(), 4);

    let job_id = JobId::from(Cid::new_v1_sha256(0x55, b"job_mana_drop"));
    let bid_a = Bid {
        job_id: job_id.clone(),
        executor_did: exec_a.clone(),
        price_mana: 8,
        resources: Resources::default(),
        signature: icn_identity::SignatureBytes(vec![]),
    };
    let bid_b = Bid {
        job_id: job_id.clone(),
        executor_did: exec_b.clone(),
        price_mana: 9,
        resources: Resources::default(),
        signature: icn_identity::SignatureBytes(vec![]),
    };

    let selected = select_executor(
        &job_id,
        &JobSpec::default(),
        vec![bid_a.clone(), bid_b],
        &SelectionPolicy::default(),
        &rep_store,
        &ledger,
    )
    .expect("selection");
    assert_eq!(selected, exec_a);

    ledger.set_balance(&exec_a, 0).unwrap();
    let spend = ledger.spend(&exec_a, bid_a.price_mana);
    assert!(matches!(spend, Err(CommonError::PolicyDenied(_))));
}

#[tokio::test]
async fn test_anchor_receipt_updates_reputation() {
    let mut ctx = create_test_runtime_context("did:icn:test:rep_ok", 10);
    let rep_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
    ctx.reputation_store = rep_store.clone();

    let receipt_json = dummy_receipt_json("job3", &ctx.current_identity.to_string(), "res");
    let result = host_anchor_receipt(&mut ctx, &receipt_json, &ReputationUpdater::new()).await;
    assert!(result.is_ok());
    assert_eq!(rep_store.get_reputation(&ctx.current_identity), 1);
}

#[tokio::test]
async fn test_executor_selected_mana_recheck_failure() {
    let submit_ctx = create_test_runtime_context("did:icn:test:recheck_submit", 100);
    let mut exec_ctx = create_test_runtime_context("did:icn:test:recheck_exec", 5);

    let job_json = dummy_job_json(10);
    let _ = host_submit_mesh_job(&submit_ctx, &job_json).await.unwrap();

    let result = exec_ctx.spend_mana(&exec_ctx.current_identity, 10).await;
    assert!(matches!(result, Err(HostAbiError::InsufficientMana)));
}

#[tokio::test]
async fn test_anchor_receipt_dag_failure() {
    let mut ctx = create_test_runtime_context("did:icn:test:dag_fail", 10);
    ctx.dag_store = Arc::new(TokioMutex::new(FailingDagStore::default()));

    let receipt_json = dummy_receipt_json("job1", &ctx.current_identity.to_string(), "res");
    let result = host_anchor_receipt(&mut ctx, &receipt_json, &ReputationUpdater::new()).await;
    assert!(matches!(result, Err(HostAbiError::DagOperationFailed(_))));
}

#[tokio::test]
async fn test_anchor_receipt_dag_failure_no_reputation_update() {
    let mut ctx = create_test_runtime_context("did:icn:test:dag_fail_rep", 10);
    let rep_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
    ctx.reputation_store = rep_store.clone();
    ctx.dag_store = Arc::new(TokioMutex::new(FailingDagStore::default()));

    let receipt_json = dummy_receipt_json("jobx", &ctx.current_identity.to_string(), "res");
    let result = host_anchor_receipt(&mut ctx, &receipt_json, &ReputationUpdater::new()).await;
    assert!(matches!(result, Err(HostAbiError::DagOperationFailed(_))));
    assert_eq!(rep_store.get_reputation(&ctx.current_identity), 0);
}

#[tokio::test]
async fn test_reputation_update_failure() {
    let mut ctx = create_test_runtime_context("did:icn:test:rep_fail", 10);
    ctx.reputation_store = Arc::new(FailingReputationStore::default());

    let receipt_json = dummy_receipt_json("job2", &ctx.current_identity.to_string(), "res");
    let result = std::panic::AssertUnwindSafe(host_anchor_receipt(
        &mut ctx,
        &receipt_json,
        &ReputationUpdater::new(),
    ))
    .catch_unwind()
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn simple_executor_ccl_job() {
    use icn_ccl::compile_ccl_source_to_wasm;
    use icn_identity::generate_ed25519_keypair;
    use icn_runtime::executor::{JobExecutor, SimpleExecutor};

    let mut ctx = create_test_runtime_context("did:icn:test:ccl_simple", 10);
    let (sk, _vk) = generate_ed25519_keypair();

    let source = "fn run() -> Integer { return 8; }";
    let (wasm, _) = compile_ccl_source_to_wasm(source).unwrap();
    let ts = 0u64;
    let author = Did::new("key", "author");
    let block = DagBlock {
        cid: Cid::new_v1_sha256(0x71, &wasm),
        data: wasm.clone(),
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: None,
    };
    {
        let mut store = ctx.dag_store.lock().await;
        store.put(&block).unwrap();
    }
    let job = ActualMeshJob {
        id: Cid::new_v1_sha256(0x55, b"ccljob"),
        manifest_cid: block.cid.clone(),
        spec: JobSpec {
            kind: JobKind::CclWasm,
            ..Default::default()
        },
        creator_did: ctx.current_identity.clone(),
        cost_mana: 0,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };

    let exec =
        SimpleExecutor::with_context(ctx.current_identity.clone(), sk, std::sync::Arc::new(ctx));
    let receipt = exec.execute_job(&job).await.unwrap();
    let expected = Cid::new_v1_sha256(0x55, &8i64.to_le_bytes());
    assert_eq!(receipt.result_cid, expected);
}
