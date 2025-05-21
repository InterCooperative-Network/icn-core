#![doc = include_str!("../README.md")]

//! # ICN Runtime Crate
//! This crate provides the execution environment for InterCooperative Network (ICN) logic,
//! possibly including WebAssembly (WASM) runtimes and host interaction capabilities.
//! It focuses on a secure, performant, and modular execution environment with well-defined host functions.

pub mod abi;
pub mod context;

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION, Did, Cid};
use context::{RuntimeContext, HostAbiError, JobId, ExecutionReceipt};
use icn_mesh::MeshJob as ActualMeshJob;
use icn_economics::{charge_mana, EconError};
use serde_json;
use std::str::FromStr;

/// Placeholder function demonstrating use of common types for runtime operations.
/// This function is not directly part of the Host ABI layer discussed below but serves as an example.
pub fn execute_icn_script(info: &NodeInfo, script_id: &str) -> Result<String, CommonError> {
    Ok(format!("Executed script {} for node: {} (v{})", script_id, info.name, info.version))
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
pub async fn host_submit_mesh_job(ctx: &mut RuntimeContext, job_json: &str) -> Result<JobId, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_submit_mesh_job"}`
    println!("[RUNTIME_ABI] host_submit_mesh_job called with job_json: {}", job_json);

    if job_json.is_empty() {
        return Err(HostAbiError::InvalidParameters("Job JSON cannot be empty".to_string()));
    }

    // 1. Deserialize MeshJob
    let mut job_to_submit: ActualMeshJob = serde_json::from_str(job_json)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Failed to deserialize MeshJob: {}. Input: {}", e, job_json)))?;

    // 2. Call ResourcePolicyEnforcer::spend_mana(did, cost).
    match charge_mana(&ctx.current_identity, job_to_submit.mana_cost) {
        Ok(_) => { /* Mana spent successfully */ }
        Err(EconError::InsufficientBalance(_)) => return Err(HostAbiError::InsufficientMana),
        Err(e) => return Err(HostAbiError::InternalError(format!("Economic error during mana spend: {:?}", e))),
    }

    // 3. Prepare and queue the job. 
    //    ID and submitter are overridden here.
    let job_id_val = context::NEXT_JOB_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let job_id_str = format!("job_{}", job_id_val);
    
    job_to_submit.id = job_id_str.clone();
    job_to_submit.submitter = ctx.current_identity.clone();

    // Call the internal queuing function on RuntimeContext
    ctx.internal_queue_mesh_job(job_to_submit.clone()).await?; // Await the async call
    
    println!("[RUNTIME_ABI] Job {} submitted by {:?} with cost {} was queued successfully.", 
             job_id_str, ctx.current_identity, job_to_submit.mana_cost);

    // 4. Return JobId.
    Ok(JobId(job_id_str))
}

/// ABI Index: (defined in `abi::ABI_HOST_GET_PENDING_MESH_JOBS`)
/// Retrieves a snapshot of the current pending mesh jobs from the runtime context.
///
/// TODO: WASM bindings will need to handle memory marshalling for the returned Vec<ActualMeshJob> (e.g., serialize to JSON string).
pub fn host_get_pending_mesh_jobs(ctx: &RuntimeContext) -> Result<Vec<ActualMeshJob>, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_get_pending_mesh_jobs"}`
    println!("[RUNTIME_ABI] host_get_pending_mesh_jobs called.");

    // Directly clone the jobs from the queue. This provides a snapshot.
    // Depending on WASM interface, this might need to be serialized (e.g., to JSON).
    let jobs: Vec<ActualMeshJob> = ctx.pending_mesh_jobs.iter().cloned().collect();
    
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
pub fn host_account_get_mana(ctx: &RuntimeContext, account_id_str: &str) -> Result<u64, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_account_get_mana"}`
    println!("[RUNTIME_ABI] host_account_get_mana called for account: {}", account_id_str);

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters("Account ID string cannot be empty".to_string()));
    }

    let account_did = Did::from_str(account_id_str)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e)))?;
    
    ctx.get_mana(&account_did)
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
pub fn host_account_spend_mana(ctx: &mut RuntimeContext, account_id_str: &str, amount: u64) -> Result<(), HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_account_spend_mana"}`
    println!("[RUNTIME_ABI] host_account_spend_mana called for account: {} amount: {}", account_id_str, amount);

    if account_id_str.is_empty() {
        return Err(HostAbiError::InvalidParameters("Account ID string cannot be empty".to_string()));
    }
    if amount == 0 {
        return Err(HostAbiError::InvalidParameters("Spend amount must be greater than zero".to_string()));
    }

    let account_did = Did::from_str(account_id_str)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e)))?;
    
    ctx.spend_mana(&account_did, amount)
}

// Placeholder for a reputation updater service/struct
#[derive(Debug)]
pub struct ReputationUpdater;

impl ReputationUpdater {
    pub fn new() -> Self { ReputationUpdater }
    pub fn submit(&self, receipt: &ExecutionReceipt) {
        // TODO: Implement actual reputation update logic
        println!("[ReputationUpdater STUB] Submitted receipt for job_id: {:?}, executor: {:?}", 
                 receipt.job_id, receipt.executor_did);
    }
}

/// ABI Index: (Not yet defined, suggest reserving one, e.g., 23)
/// Anchors an execution receipt to the DAG and updates reputation.
///
/// The `receipt_json` is expected to be a JSON string serializing `context::ExecutionReceipt`.
///
/// TODO: WASM bindings will need to handle memory marshalling for `receipt_json` and returned `Cid`.
pub fn host_anchor_receipt(ctx: &mut RuntimeContext, receipt_json: &str, reputation_updater: &ReputationUpdater) -> Result<Cid, HostAbiError> {
    // TODO: record metric `icn_runtime_abi_call_total{method="host_anchor_receipt"}`
    println!("[RUNTIME_ABI] host_anchor_receipt called with receipt_json: {}", receipt_json);

    if receipt_json.is_empty() {
        return Err(HostAbiError::InvalidParameters("Receipt JSON cannot be empty".to_string()));
    }

    // 1. Deserialize ExecutionReceipt
    let mut receipt: ExecutionReceipt = serde_json::from_str(receipt_json)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Failed to deserialize ExecutionReceipt: {}. Input: {}", e, receipt_json)))?;

    // Ensure the receipt is for the current identity if it's being submitted by the executor itself
    // This check might be more nuanced depending on who is allowed to call this ABI function.
    // If only the executor who ran the job calls it, then current_identity should match receipt.executor_did.
    if ctx.current_identity != receipt.executor_did {
        // This could be an error, or it could be a case where a node service is anchoring on behalf of an executor.
        // For now, let's log a warning. The `ctx.anchor_receipt` itself has a stricter check.
        println!("[RUNTIME_ABI_WARN] host_anchor_receipt called by {:?} for a receipt from executor {:?}. Ensure this is intended.", 
                 ctx.current_identity, receipt.executor_did);
    }

    // 2. Call ctx.anchor_receipt (which handles internal signing and DAG storage)
    let anchored_cid = ctx.anchor_receipt(&mut receipt)?; // anchor_receipt might modify receipt (e.g. add signature)
    println!("[RUNTIME_ABI] Receipt for job_id {:?} anchored with CID: {:?}", receipt.job_id, anchored_cid);

    // 3. Submit to reputation updater
    reputation_updater.submit(&receipt);

    Ok(anchored_cid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::abi;
    use super::context::{RuntimeContext, HostAbiError, JobId, SimpleManaLedger};
    use icn_common::Did;
    use std::str::FromStr;

    const TEST_IDENTITY_DID_STR: &str = "did:icn:test:dummy_executor";
    const OTHER_IDENTITY_DID_STR: &str = "did:icn:test:other_account";

    fn create_test_context() -> RuntimeContext {
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).expect("Failed to create test DID");
        RuntimeContext::new(test_did)
    }

    fn create_test_context_with_mana(initial_mana: u64) -> RuntimeContext {
        let test_did = Did::from_str(TEST_IDENTITY_DID_STR).expect("Failed to create test DID");
        let mut ctx = RuntimeContext::new(test_did.clone());
        ctx.mana_ledger.set_balance(&test_did, initial_mana);
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
        let mut ctx = create_test_context();
        let job_spec = r#"{"cid":{"version":1,"codec":85,"hash_alg":18,"hash_bytes":[]},"spec":{},"mana_cost":10}"#;
        let result = host_submit_mesh_job(&mut ctx, job_spec).await;
        assert!(result.is_ok(), "host_submit_mesh_job failed: {:?}", result.err());
        let queue = ctx.pending_mesh_jobs.lock().await;
        assert_eq!(queue.len(), 1, "Job not added to queue");
        assert_eq!(queue.front().unwrap().mana_cost, 10);
    }

    #[tokio::test]
    async fn test_host_submit_mesh_job_empty_spec() {
        let mut ctx = create_test_context();
        let job_spec = "";
        let result = host_submit_mesh_job(&mut ctx, job_spec).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Job JSON cannot be empty");
            }
            e => panic!("Expected InvalidParameters error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_host_submit_mesh_job_insufficient_mana() {
        let mut ctx = create_test_context_with_mana(5);
        let job_spec = r#"{"cid":{"version":1,"codec":85,"hash_alg":18,"hash_bytes":[]},"spec":{},"mana_cost":10}"#;
        let result = host_submit_mesh_job(&mut ctx, job_spec).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InsufficientMana => { /* Expected */ }
            e => panic!("Expected InsufficientMana, got {:?}", e),
        }
        let queue = ctx.pending_mesh_jobs.lock().await;
        assert_eq!(queue.len(), 0, "Job should not be added to queue on insufficient mana");
    }

    #[tokio::test]
    async fn test_host_get_pending_mesh_jobs_retrieves_correctly() {
        let mut ctx = create_test_context_with_mana(100);
        let job_spec1 = r#"{"cid":{"version":1,"codec":1,"hash_alg":1,"hash_bytes":[1]},"spec":{},"mana_cost":10}"#;
        let job_spec2 = r#"{"cid":{"version":1,"codec":2,"hash_alg":2,"hash_bytes":[2]},"spec":{},"mana_cost":20}"#;
        
        host_submit_mesh_job(&mut ctx, job_spec1).await.unwrap();
        host_submit_mesh_job(&mut ctx, job_spec2).await.unwrap();

        let pending_jobs_result = host_get_pending_mesh_jobs(&ctx);
        assert!(pending_jobs_result.is_ok());
        let pending_jobs = pending_jobs_result.unwrap();
        assert_eq!(pending_jobs.len(), 2);
        assert_eq!(pending_jobs[0].mana_cost, 10);
        assert_eq!(pending_jobs[1].mana_cost, 20);
    }

    #[tokio::test]
    async fn test_host_get_pending_mesh_jobs_empty_queue() {
        let ctx = create_test_context();
        let pending_jobs_result = host_get_pending_mesh_jobs(&ctx);
        assert!(pending_jobs_result.is_ok());
        let pending_jobs = pending_jobs_result.unwrap();
        assert_eq!(pending_jobs.len(), 0);
    }

    #[test]
    fn test_host_account_get_mana_calls_context() {
        let ctx = create_test_context_with_mana(100);
        let account_id = ctx.current_identity.to_string();
        let _ = host_account_get_mana(&ctx, &account_id);
    }

    #[test]
    fn test_host_account_get_mana_empty_id() {
        let ctx = create_test_context();
        let account_id = "";
        let result = host_account_get_mana(&ctx, account_id);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!("Expected InvalidParameters error, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_get_mana_invalid_did_format() {
        let ctx = create_test_context();
        let account_id = "not-a-valid-did";
        let result = host_account_get_mana(&ctx, account_id);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!("Expected InvalidParameters error for DID format, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_successful() {
        let mut ctx = create_test_context_with_mana(100);
        let spend_amount = 30;
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, spend_amount);
        assert!(result.is_ok(), "Expected successful spend, got {:?}", result.err());
        let remaining_mana = ctx.get_mana(&ctx.current_identity).unwrap();
        assert_eq!(remaining_mana, 70, "Mana not deducted correctly");
    }

    #[test]
    fn test_host_account_spend_mana_insufficient_funds() {
        let mut ctx = create_test_context_with_mana(20);
        let spend_amount = 30;
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, spend_amount);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InsufficientMana => { /* Expected */ }
            e => panic!("Expected InsufficientMana, got {:?}", e),
        }
        let remaining_mana = ctx.get_mana(&ctx.current_identity).unwrap();
        assert_eq!(remaining_mana, 20, "Mana should not change on failed spend");
    }

    #[test]
    fn test_host_account_spend_mana_account_not_found() {
        let mut ctx = create_test_context_with_mana(100);
        let non_existent_did_str = "did:icn:test:non_existent_account";
        let result = host_account_spend_mana(&mut ctx, non_existent_did_str, 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::AccountNotFound(did) => {
                assert_eq!(did.to_string(), non_existent_did_str);
            }
            e => panic!("Expected AccountNotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_zero_amount() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, TEST_IDENTITY_DID_STR, 0);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Spend amount must be greater than zero");
            }
            e => panic!("Expected InvalidParameters for zero amount, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_empty_account_id() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, "", 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert_eq!(msg, "Account ID string cannot be empty");
            }
            e => panic!("Expected InvalidParameters for empty account ID, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_invalid_did_format() {
        let mut ctx = create_test_context_with_mana(100);
        let result = host_account_spend_mana(&mut ctx, "not-a-valid-did", 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid DID format for account_id"));
            }
            e => panic!("Expected InvalidParameters for invalid DID, got {:?}", e),
        }
    }

    #[test]
    fn test_host_account_spend_mana_for_other_account_fails_policy() {
        let mut ctx = create_test_context_with_mana(100);
        let other_did = Did::from_str(OTHER_IDENTITY_DID_STR).unwrap();
        ctx.mana_ledger.set_balance(&other_did, 50);

        let result = host_account_spend_mana(&mut ctx, OTHER_IDENTITY_DID_STR, 10);
        assert!(result.is_err());
        match result.err().unwrap() {
            HostAbiError::InvalidParameters(msg) => {
                assert!(msg.contains("Attempting to spend mana for an account other than the current context identity"));
            }
            e => panic!("Expected InvalidParameters due to policy violation, got {:?}", e),
        }
        let other_mana = ctx.get_mana(&other_did).unwrap();
        assert_eq!(other_mana, 50);
    }
}
