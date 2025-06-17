#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]
#![allow(clippy::to_string_in_format_args)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unnecessary_mut_passed)]

//! This is the core ICN Runtime crate.
//!
//! It provides:
//! - The Host ABI that WASM modules call into for accessing ICN services.
//! - The RuntimeContext which manages state (identity, mana, jobs, governance).
//! - The node runtime integration for libp2p networking.

pub mod abi;
pub mod context;
pub mod executor;
pub mod metrics;
pub mod error;

// Re-export important types for convenience
pub use context::{HostAbiError, RuntimeContext, Signer};
pub use icn_dag::StorageService;

// Re-export ABI constants
pub use abi::*;

use icn_common::{Cid, CommonError, Did, NodeInfo};
use log::{debug, info};
use std::str::FromStr;
#[cfg(test)]
use std::sync::Arc;

/// Placeholder function demonstrating use of common types for runtime operations.
/// This function is not directly part of the Host ABI layer discussed below but serves as an example.
pub fn execute_icn_script(info: &NodeInfo, script_id: &str) -> Result<String, CommonError> {
    Ok(format!(
        "Executed script {} for node: {} (v{})",
        script_id, info.name, info.version
    ))
}

// --- Host ABI Functions ---
// These functions are intended to be callable from a WASM environment,
// mediated by the `HostEnvironment` and using a `RuntimeContext`.

/// ABI Index: (defined in `abi::ABI_HOST_SUBMIT_MESH_JOB`)
/// Submits a job to the mesh network using the provided runtime context.
///
/// The `job_json` is expected to be a JSON string serializing `icn_mesh::MeshJob`.
/// The `id` and `submitter` fields within the deserialized job will be overridden
/// by the runtime (new JobId generation, context's current_identity).
///
/// TODO: WASM bindings will need to handle memory marshalling for `job_json`.
pub async fn host_submit_mesh_job(
    ctx: &RuntimeContext,
    job_json: &str,
) -> Result<Cid, HostAbiError> {
    metrics::HOST_SUBMIT_MESH_JOB_CALLS.inc();
    println!(
        "[RUNTIME_ABI] host_submit_mesh_job called with job_json: {}",
        job_json
    );

    if job_json.is_empty() {
        return Err(HostAbiError::InvalidParameters(
            "Job JSON cannot be empty".to_string(),
        ));
    }

    // 1. Deserialize MeshJob
    let mut job_to_submit: icn_mesh::ActualMeshJob =
        serde_json::from_str(job_json).map_err(|e| {
            HostAbiError::InvalidParameters(format!(
                "Failed to deserialize ActualMeshJob: {}. Input: {}",
                e, job_json
            ))
        })?;

    // 2. Call ResourcePolicyEnforcer::spend_mana(did, cost).
    // This should use the RuntimeContext's spend_mana which is now async.
    ctx.spend_mana(&ctx.current_identity, job_to_submit.cost_mana)
        .await
        .map_err(|e| match e {
            HostAbiError::InsufficientMana => HostAbiError::InsufficientMana,
            _ => HostAbiError::InternalError(format!("Error spending mana: {:?}", e)),
        })?;

    // The charge_mana function was a placeholder for ctx.spend_mana.
    // We'll remove the direct call to charge_mana and rely on ctx.spend_mana.
    // match charge_mana(&ctx.current_identity, job_to_submit.cost_mana) {
    //     Ok(_) => { /* Mana spent successfully */ }
    //     Err(EconError::InsufficientBalance(_)) => return Err(HostAbiError::InsufficientMana),
    //     Err(e) => return Err(HostAbiError::InternalError(format!("Economic error during mana spend: {:?}", e))),
    // }

    // 3. Prepare and queue the job.
    //    ID and submitter are overridden here.
    let job_id_val = context::NEXT_JOB_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // Create a dummy CID for the JobId for now.
    // In a real scenario, this might be derived from the job content or other unique inputs.
    let job_id_cid = Cid::new_v1_dummy(0x55, 0x13, format!("job_cid_{}", job_id_val).as_bytes());

    job_to_submit.id = job_id_cid.clone();
    job_to_submit.creator_did = ctx.current_identity.clone();

    // Call the internal queuing function on RuntimeContext
    ctx.internal_queue_mesh_job(job_to_submit.clone()).await?; // Await the async call

    println!(
        "[RUNTIME_ABI] Job {:?} submitted by {:?} with cost {} was queued successfully.",
        job_id_cid, ctx.current_identity, job_to_submit.cost_mana
    );

    // 4. Return JobId (which is now a Cid).
    Ok(job_id_cid)
}

/// ABI Index: (defined in `abi::ABI_HOST_GET_PENDING_MESH_JOBS`)
/// Retrieves a snapshot of the current pending mesh jobs from the runtime context.
///
/// TODO: WASM bindings will need to handle memory marshalling for the returned Vec<ActualMeshJob> (e.g., serialize to JSON string).
pub fn host_get_pending_mesh_jobs(
    ctx: &RuntimeContext,
) -> Result<Vec<icn_mesh::ActualMeshJob>, HostAbiError> {
    metrics::HOST_GET_PENDING_MESH_JOBS_CALLS.inc();
    println!("[RUNTIME_ABI] host_get_pending_mesh_jobs called.");

    // Directly clone the jobs from the queue. This provides a snapshot.
    // Depending on WASM interface, this might need to be serialized (e.g., to JSON).
    // Need to acquire the lock and then await its resolution.
    let jobs = futures::executor::block_on(async {
        ctx.pending_mesh_jobs
            .lock()
            .await
            .iter()
            .cloned()
            .collect::<Vec<icn_mesh::ActualMeshJob>>()
    });

    println!("[RUNTIME_ABI] Returning {} pending jobs.", jobs.len());
    Ok(jobs)
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_GET_MANA`)
/// Retrieves the current mana for the calling account/identity, using the provided runtime context.
///
/// The `account_id_str` is the string representation of the DID for which mana is requested.
/// In many cases, this will be the `current_identity` within the `ctx`,
/// but the API allows specifying it for potential future flexibility (e.g., admin queries).
///
/// TODO: WASM bindings will need to handle memory marshalling for `account_id_str`.
pub async fn host_account_get_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
) -> Result<u64, HostAbiError> {
    metrics::HOST_ACCOUNT_GET_MANA_CALLS.inc();
    println!(
        "[RUNTIME_ABI] host_account_get_mana called for account: {}",
        account_id_str
    );

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters(
            "Account ID string cannot be empty".to_string(),
        ));
    }

    let account_did = Did::from_str(account_id_str).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e))
    })?;

    ctx.get_mana(&account_did).await
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_SPEND_MANA`)
/// Attempts to spend mana from the specified account, using the provided runtime context.
///
/// The `account_id_str` is the string representation of the DID from which mana should be spent.
/// The `amount` is the quantity of mana to spend.
///
/// Policy Note: `RuntimeContext::spend_mana` currently only allows spending from `ctx.current_identity`.
///
/// TODO: WASM bindings will need to handle memory marshalling for `account_id_str` and `amount`.
pub async fn host_account_spend_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError> {
    metrics::HOST_ACCOUNT_SPEND_MANA_CALLS.inc();
    println!(
        "[RUNTIME_ABI] host_account_spend_mana called for account: {} amount: {}",
        account_id_str, amount
    );

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters(
            "Account ID string cannot be empty".to_string(),
        ));
    }
    if amount == 0 {
        return Err(HostAbiError::InvalidParameters(
            "Spend amount must be greater than zero".to_string(),
        ));
    }

    let account_did = Did::from_str(account_id_str).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e))
    })?;

    // Ensure current_identity matches account_did for spending, as per RuntimeContext::spend_mana policy
    if account_did != ctx.current_identity {
        return Err(HostAbiError::InvalidParameters(
            "Attempting to spend mana for an account other than the current context identity."
                .to_string(),
        ));
    }

    ctx.spend_mana(&account_did, amount).await
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_CREDIT_MANA`)
/// Credits mana to the specified account.
pub async fn host_account_credit_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError> {
    println!(
        "[RUNTIME_ABI] host_account_credit_mana called for account: {} amount: {}",
        account_id_str, amount
    );

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters(
            "Account ID string cannot be empty".to_string(),
        ));
    }
    if amount == 0 {
        // Crediting zero might be permissible, but often indicates an issue.
        // For now, let's allow it but log a warning. Or return InvalidParameters.
        // Sticking with allowing it for now.
        println!(
            "[RUNTIME_ABI_WARN] host_account_credit_mana called with amount zero for account: {}",
            account_id_str
        );
    }

    let account_did = Did::from_str(account_id_str).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e))
    })?;

    ctx.credit_mana(&account_did, amount).await
}

// Placeholder for a reputation updater service/struct
use icn_reputation::ReputationStore;

pub struct ReputationUpdater;

impl ReputationUpdater {
    pub fn new() -> Self {
        ReputationUpdater
    }
    pub fn submit(&self, store: &dyn ReputationStore, receipt: &icn_identity::ExecutionReceipt) {
        let before = store.get_reputation(&receipt.executor_did);
        store.record_execution(&receipt.executor_did, receipt.success, receipt.cpu_ms);
        let after = store.get_reputation(&receipt.executor_did);
        log::debug!(
            "[ReputationUpdater] Executor {:?} reputation {} -> {} via receipt {:?}",
            receipt.executor_did,
            before,
            after,
            receipt.job_id
        );
    }
}

/// ABI Index: (defined in `abi::ABI_HOST_ANCHOR_RECEIPT`)
/// Anchors an execution receipt to the DAG and updates reputation.
///
/// The `receipt_json` is expected to be a JSON string serializing `icn_identity::ExecutionReceipt`.
///
/// TODO: WASM bindings will need to handle memory marshalling for `receipt_json` and returned `Cid`.
pub async fn host_anchor_receipt(
    ctx: &RuntimeContext,
    receipt_json: &str,
    reputation_updater: &ReputationUpdater,
) -> Result<Cid, HostAbiError> {
    debug!(
        "[host_anchor_receipt] Received receipt JSON: {}",
        receipt_json
    );
    let mut receipt: icn_identity::ExecutionReceipt =
        serde_json::from_str(receipt_json).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Failed to deserialize receipt JSON: {}", e))
        })?;

    // The original code in the job manager calls signer.verify and dag_store.put which are async.
    // RuntimeContext::anchor_receipt was made async to accommodate this.
    // Therefore, this host function, if it directly calls it, must also be async and awaited.
    let anchored_cid = ctx.anchor_receipt(&mut receipt).await?; // Now awaiting the async call

    info!("[host_anchor_receipt] Receipt for job {:?} (executor {:?}) anchored with CID: {:?}. CPU cost: {}ms", 
          receipt.job_id, receipt.executor_did, anchored_cid, receipt.cpu_ms);

    reputation_updater.submit(ctx.reputation_store.as_ref(), &receipt);
    Ok(anchored_cid)
}

/// Creates a governance proposal using the runtime context.
pub async fn host_create_governance_proposal(
    ctx: &RuntimeContext,
    payload_json: &str,
) -> Result<String, HostAbiError> {
    let payload: context::CreateProposalPayload =
        serde_json::from_str(payload_json).map_err(|e| {
            HostAbiError::InvalidParameters(format!(
                "Failed to parse CreateProposalPayload JSON: {}",
                e
            ))
        })?;
    ctx.create_governance_proposal(payload).await
}

/// Casts a governance vote using the runtime context.
pub async fn host_cast_governance_vote(
    ctx: &RuntimeContext,
    payload_json: &str,
) -> Result<(), HostAbiError> {
    let payload: context::CastVotePayload = serde_json::from_str(payload_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Failed to parse CastVotePayload JSON: {}", e))
    })?;
    ctx.cast_governance_vote(payload).await
}

/// Closes voting on a governance proposal. Currently not implemented.
pub async fn host_close_governance_proposal_voting(
    ctx: &RuntimeContext,
    proposal_id: &str,
) -> Result<String, HostAbiError> {
    ctx.close_governance_proposal_voting(proposal_id).await
}

/// Executes an accepted governance proposal. Currently not implemented.
pub async fn host_execute_governance_proposal(
    ctx: &RuntimeContext,
    proposal_id: &str,
) -> Result<(), HostAbiError> {
    ctx.execute_governance_proposal(proposal_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::context::{
        HostAbiError, RuntimeContext, StubDagStore, StubMeshNetworkService, StubSigner,
    };
    use icn_common::{Cid, Did, ICN_CORE_VERSION};
    use icn_identity::SignatureBytes;
    use icn_mesh::{ActualMeshJob, JobSpec};
    use std::str::FromStr;

    const TEST_IDENTITY_DID_STR: &str = "did:icn:test:dummy_executor";
    const OTHER_IDENTITY_DID_STR: &str = "did:icn:test:other_account";

    // Helper function to create a test ActualMeshJob with all required fields
    fn create_test_mesh_job(cost_mana: u64) -> ActualMeshJob {
        ActualMeshJob {
            id: Cid::new_v1_dummy(0x55, 0x13, b"test_job_id"),
            manifest_cid: Cid::new_v1_dummy(0x55, 0x12, b"test_manifest"),
            spec: JobSpec::default(),
            creator_did: Did::from_str(TEST_IDENTITY_DID_STR).unwrap(),
            cost_mana,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]), // Dummy signature for tests
        }
    }

    // Helper function to create a RuntimeContext with stubbed services for testing.
    // This function is NOT async because new_with_stubs is not async.
    fn create_test_context() -> Arc<RuntimeContext> {
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).expect("Failed to create test DID");
        RuntimeContext::new(
            test_did,
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            Arc::new(tokio::sync::Mutex::new(StubDagStore::new())),
        )
    }

    fn create_test_context_with_mana(initial_mana: u64) -> Arc<RuntimeContext> {
        let ctx = create_test_context();
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).unwrap();
        ctx.mana_ledger
            .set_balance(&test_did, initial_mana)
            .expect("set initial mana");
        ctx
    }

    #[test]
    fn test_execute_icn_script() {
        let node_info = NodeInfo {
            name: "RuntimeNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Runtime active".to_string(),
        };
        let result = execute_icn_script(&node_info, "script-xyz");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("script-xyz"));
    }

    #[tokio::test]
    async fn test_host_submit_mesh_job_calls_context() {
        let ctx = create_test_context_with_mana(100); // Job cost is 10
        let test_job = create_test_mesh_job(10);
        let job_json = serde_json::to_string(&test_job).unwrap();
        let job_id = host_submit_mesh_job(&ctx, &job_json).await;
        assert!(
            job_id.is_ok(),
            "host_submit_mesh_job failed: {:?}",
            job_id.err()
        );

        // Verify mana was spent
        let mana_after = ctx.get_mana(&ctx.current_identity).await.unwrap();
        assert_eq!(mana_after, 90);

        // Verify job was queued
        let pending_jobs = host_get_pending_mesh_jobs(&ctx).unwrap();
        assert_eq!(pending_jobs.len(), 1);
        assert_eq!(pending_jobs[0].cost_mana, 10);
    }

    #[tokio::test]
    async fn test_host_submit_mesh_job_empty_spec() {
        let ctx = create_test_context_with_mana(100);
        let test_job = create_test_mesh_job(10);
        let job_json = serde_json::to_string(&test_job).unwrap();
        let job_id = host_submit_mesh_job(&ctx, &job_json).await;
        assert!(
            job_id.is_ok(),
            "host_submit_mesh_job with empty spec failed: {:?}",
            job_id.err()
        );
        let mana_after = ctx.get_mana(&ctx.current_identity).await.unwrap();
        assert_eq!(mana_after, 90);
    }

    #[tokio::test]
    async fn test_host_submit_mesh_job_insufficient_mana() {
        let ctx = create_test_context_with_mana(5); // Not enough for cost 10
        let test_job = create_test_mesh_job(10);
        let job_json = serde_json::to_string(&test_job).unwrap();
        let result = host_submit_mesh_job(&ctx, &job_json).await;
        assert!(
            matches!(result, Err(HostAbiError::InsufficientMana)),
            "Expected InsufficientMana, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_host_get_pending_mesh_jobs_retrieves_correctly() {
        let ctx = create_test_context_with_mana(100);
        let test_job1 = create_test_mesh_job(10);
        let test_job2 = create_test_mesh_job(5);
        let job_json1 = serde_json::to_string(&test_job1).unwrap();
        let job_json2 = serde_json::to_string(&test_job2).unwrap();

        let _job_id1 = host_submit_mesh_job(&ctx, &job_json1).await.unwrap();
        let _job_id2 = host_submit_mesh_job(&ctx, &job_json2).await.unwrap();

        let pending_jobs = host_get_pending_mesh_jobs(&ctx).unwrap();
        assert_eq!(pending_jobs.len(), 2);
        assert!(pending_jobs.iter().any(|j| j.cost_mana == 10));
        assert!(pending_jobs.iter().any(|j| j.cost_mana == 5));
    }

    #[tokio::test]
    async fn test_host_get_pending_mesh_jobs_empty_queue() {
        let ctx = create_test_context();
        let pending_jobs_result = host_get_pending_mesh_jobs(&ctx);
        assert!(pending_jobs_result.is_ok());
        let pending_jobs = pending_jobs_result.unwrap();
        assert_eq!(pending_jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_host_account_get_mana_calls_context() {
        let ctx = create_test_context_with_mana(50);
        let mana = host_account_get_mana(&ctx, TEST_IDENTITY_DID_STR).await;
        assert!(mana.is_ok());
        assert_eq!(mana.unwrap(), 50);
    }

    #[tokio::test]
    async fn test_host_account_get_mana_empty_id() {
        let ctx = create_test_context();
        let account_id = "";
        let result = host_account_get_mana(&ctx, account_id).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!("Expected InvalidParameters error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_host_account_get_mana_invalid_did_format() {
        let ctx = create_test_context();
        let account_id = "invalid-did";
        let result = host_account_get_mana(&ctx, account_id).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!(
                "Expected InvalidParameters error for DID format, got {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_successful() {
        let ctx = create_test_context_with_mana(20);
        let result = host_account_spend_mana(&ctx, TEST_IDENTITY_DID_STR, 10).await;
        assert!(result.is_ok());
        let mana_after = ctx.get_mana(&ctx.current_identity).await.unwrap();
        assert_eq!(mana_after, 10);
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_insufficient_funds() {
        let ctx = create_test_context_with_mana(5);
        let result = host_account_spend_mana(&ctx, TEST_IDENTITY_DID_STR, 10).await;
        assert!(
            matches!(result, Err(HostAbiError::InsufficientMana)),
            "Expected InsufficientMana, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_account_not_found() {
        let mut ctx = create_test_context_with_mana(100);
        let account_id = "did:icn:test:nonexistent";
        let spend_amount = 10u64;
        let result = host_account_spend_mana(&mut ctx, account_id, spend_amount).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Attempting to spend mana for an account other than the current context identity."));
            }
            e => panic!(
                "Expected InvalidParameters (policy) or AccountNotFound, got {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_zero_amount() {
        let mut ctx = create_test_context_with_mana(100);
        let account_id = ctx.current_identity.to_string();
        let spend_amount = 0u64;
        let result = host_account_spend_mana(&mut ctx, &account_id, spend_amount).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Spend amount must be greater than zero");
            }
            e => panic!("Expected InvalidParameters for zero amount, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_empty_account_id() {
        let mut ctx = create_test_context_with_mana(100);
        let account_id = "";
        let spend_amount = 10u64;
        let result = host_account_spend_mana(&mut ctx, account_id, spend_amount).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!(
                "Expected InvalidParameters for empty account ID, got {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_invalid_did_format() {
        let mut ctx = create_test_context_with_mana(100);
        let account_id = "invalid-did";
        let spend_amount = 10u64;
        let result = host_account_spend_mana(&mut ctx, account_id, spend_amount).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!("Expected InvalidParameters for invalid DID, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_host_account_spend_mana_for_other_account_fails_policy() {
        let mut ctx = create_test_context_with_mana(100);
        let other_account_id = OTHER_IDENTITY_DID_STR;

        let other_did = Did::from_str(other_account_id).unwrap();
        ctx.mana_ledger
            .set_balance(&other_did, 50)
            .expect("set mana for other did");

        let spend_amount = 10u64;
        let result = host_account_spend_mana(&mut ctx, other_account_id, spend_amount).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Attempting to spend mana for an account other than the current context identity."));
            }
            e => panic!(
                "Expected InvalidParameters error due to policy, got {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_runtime_context_new_with_stubs() {
        let node_did_str = "did:key:z6MkjL4FwS3np2p2NLiqH57sX99pZtG9x3Fy9bYh3xHqs14z";
        let ctx = RuntimeContext::new_with_stubs(node_did_str);
        assert_eq!(ctx.current_identity.to_string(), node_did_str);
        // Further checks can be added here if needed
    }

    #[tokio::test]
    async fn test_runtime_context_new_with_stubs_and_mana() {
        let node_did_str = "did:key:zTestManaDid";
        let initial_mana = 1000u64;
        let ctx = RuntimeContext::new_with_stubs_and_mana(node_did_str, initial_mana);
        assert_eq!(ctx.current_identity.to_string(), node_did_str);
        let balance = ctx.get_mana(&ctx.current_identity).await.unwrap();
        assert_eq!(balance, initial_mana);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_metrics_increment_on_host_submit_mesh_job() {
        use crate::metrics::HOST_SUBMIT_MESH_JOB_CALLS;
        let ctx = create_test_context_with_mana(50);
        let job = create_test_mesh_job(10);
        let job_json = serde_json::to_string(&job).unwrap();
        let before = HOST_SUBMIT_MESH_JOB_CALLS.get();
        host_submit_mesh_job(&ctx, &job_json).await.unwrap();
        assert!(HOST_SUBMIT_MESH_JOB_CALLS.get() > before);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_metrics_increment_on_host_get_pending_mesh_jobs() {
        use crate::metrics::HOST_GET_PENDING_MESH_JOBS_CALLS;
        let ctx = create_test_context();
        let before = HOST_GET_PENDING_MESH_JOBS_CALLS.get();
        host_get_pending_mesh_jobs(&ctx).unwrap();
        assert!(HOST_GET_PENDING_MESH_JOBS_CALLS.get() > before);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_metrics_increment_on_host_account_get_mana() {
        use crate::metrics::HOST_ACCOUNT_GET_MANA_CALLS;
        let ctx = create_test_context_with_mana(20);
        let before = HOST_ACCOUNT_GET_MANA_CALLS.get();
        host_account_get_mana(&ctx, TEST_IDENTITY_DID_STR)
            .await
            .unwrap();
        assert!(HOST_ACCOUNT_GET_MANA_CALLS.get() > before);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_metrics_increment_on_host_account_spend_mana() {
        use crate::metrics::HOST_ACCOUNT_SPEND_MANA_CALLS;
        let ctx = create_test_context_with_mana(20);
        let before = HOST_ACCOUNT_SPEND_MANA_CALLS.get();
        host_account_spend_mana(&ctx, TEST_IDENTITY_DID_STR, 5)
            .await
            .unwrap();
        assert!(HOST_ACCOUNT_SPEND_MANA_CALLS.get() > before);
    }
}
