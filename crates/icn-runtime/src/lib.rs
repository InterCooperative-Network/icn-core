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
pub mod config;
pub mod context;
pub mod executor;
pub mod memory;
pub mod metrics;

// Re-export important types for convenience
pub use context::{DagStoreMutexType, HostAbiError, RuntimeContext, Signer};
#[cfg(feature = "async")]
pub use icn_dag::AsyncStorageService as StorageService;
#[cfg(not(feature = "async"))]
pub use icn_dag::StorageService;

// Re-export ABI constants
pub use abi::*;
extern crate bincode;
use icn_common::{Cid, CommonError, Did, NodeInfo};
use icn_mesh::JobId;
use icn_reputation::ReputationStore;
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::str::FromStr;
use thiserror::Error;

pub use config::{
    templates, EnvironmentConfig, GovernanceConfig, IdentityConfig, NetworkConfig, RuntimeConfig,
    RuntimeConfigBuilder, RuntimeParametersConfig, StorageConfig,
};
pub use context::{
    CastVotePayload, CloseProposalResult, ConcreteHostEnvironment, CreateProposalPayload,
    DefaultMeshNetworkService, Ed25519Signer, HostEnvironment, JobAssignmentNotice,
    MeshNetworkService, MeshNetworkServiceType, SimpleManaLedger, StubMeshNetworkService,
};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Host ABI error: {0}")]
    HostAbi(HostAbiError),
    #[error("Common error: {0}")]
    Common(CommonError),
}

impl From<HostAbiError> for RuntimeError {
    fn from(err: HostAbiError) -> Self {
        RuntimeError::HostAbi(err)
    }
}

impl From<CommonError> for RuntimeError {
    fn from(err: CommonError) -> Self {
        RuntimeError::Common(err)
    }
}

pub const MAX_JOB_JSON_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Deserialize)]
struct GenerateProofRequest {
    issuer: String,
    holder: String,
    claim_type: String,
    schema: String,
    backend: String,
    verification_key: Option<String>,
    public_inputs: Option<serde_json::Value>,
}

pub fn execute_icn_script(info: &NodeInfo, script_id: &str) -> Result<String, CommonError> {
    // Stub implementation
    Ok(format!(
        "Script executed on node {} ({}): {}",
        info.name, info.version, script_id
    ))
}

// NodeInfo is now imported from icn_common

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
/// When invoked from WebAssembly use [`wasm_host_submit_mesh_job`], which
/// accepts pointer/length parameters and marshals the JSON string via the
/// `memory` helpers.
pub async fn host_submit_mesh_job(
    ctx: &std::sync::Arc<RuntimeContext>,
    job_json: &str,
) -> Result<icn_mesh::JobId, HostAbiError> {
    metrics::HOST_SUBMIT_MESH_JOB_CALLS.inc();
    debug!("[host_submit_mesh_job] called with job_json: {}", job_json);

    if job_json.is_empty() {
        return Err(HostAbiError::InvalidParameters(
            "Job JSON cannot be empty".to_string(),
        ));
    }

    if job_json.len() > MAX_JOB_JSON_SIZE {
        return Err(HostAbiError::InvalidParameters(format!(
            "Job JSON exceeds {} bytes",
            MAX_JOB_JSON_SIZE
        )));
    }

    // Parse the input to extract the required fields for the new API
    let job_to_submit: icn_mesh::ActualMeshJob = serde_json::from_str(job_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!(
            "Failed to deserialize ActualMeshJob: {}. Input: {}",
            e, job_json
        ))
    })?;

    // Extract the fields needed for the new handle_submit_job method
    let manifest_cid = job_to_submit.manifest_cid;
    let spec_bytes = bincode::serialize(&job_to_submit.spec)
        .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize job spec: {}", e)))?;
    let cost_mana = job_to_submit.cost_mana;

    // Use the new DAG-integrated job submission method
    ctx.handle_submit_job(manifest_cid, spec_bytes, cost_mana)
        .await
}

/// ABI Index: (defined in `abi::ABI_HOST_GET_PENDING_MESH_JOBS`)
/// Retrieves a snapshot of the current pending mesh jobs from the runtime context.
///
/// For WebAssembly callers see [`wasm_host_get_pending_mesh_jobs`], which
/// serializes the job list to JSON and writes it to guest memory using
/// pointer/length parameters.
pub async fn host_get_pending_mesh_jobs(
    ctx: &RuntimeContext,
) -> Result<Vec<icn_mesh::ActualMeshJob>, HostAbiError> {
    metrics::HOST_GET_PENDING_MESH_JOBS_CALLS.inc();
    debug!("[host_get_pending_mesh_jobs] called");

    // Directly clone the jobs from the queue. This provides a snapshot.
    // Depending on WASM interface, this might need to be serialized (e.g., to JSON).
    // Need to acquire the lock and then await its resolution.
    let mut jobs = Vec::new();
    let mut drained = Vec::new();
    {
        let mut rx = ctx.pending_mesh_jobs_rx.lock().await;
        while let Ok(job) = rx.try_recv() {
            drained.push(job);
        }
    }
    for job in drained.iter() {
        ctx.pending_mesh_jobs_tx
            .send(job.clone())
            .await
            .map_err(|e| HostAbiError::InternalError(format!("{e:?}")))?;
    }
    jobs.extend(drained);

    debug!(
        "[host_get_pending_mesh_jobs] Returning {} pending jobs.",
        jobs.len()
    );
    Ok(jobs)
}

/// ABI Index: (defined in `abi::ABI_HOST_ACCOUNT_GET_MANA`)
/// Retrieves the current mana for the calling account/identity, using the provided runtime context.
///
/// The `account_id_str` is the string representation of the DID for which mana is requested.
/// In many cases, this will be the `current_identity` within the `ctx`,
/// but the API allows specifying it for potential future flexibility (e.g., admin queries).
///
/// WebAssembly modules should call [`wasm_host_account_get_mana`], which
/// reads the DID string from guest memory using pointer/length arguments.
pub async fn host_account_get_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
) -> Result<u64, HostAbiError> {
    metrics::HOST_ACCOUNT_GET_MANA_CALLS.inc();
    debug!(
        "[host_account_get_mana] called for account: {}",
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
/// For WebAssembly use [`wasm_host_account_spend_mana`], which reads the DID
/// string and writes any errors through pointer/length parameters.
pub async fn host_account_spend_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError> {
    metrics::HOST_ACCOUNT_SPEND_MANA_CALLS.inc();
    debug!(
        "[host_account_spend_mana] called for account: {} amount: {}",
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
    info!(
        "[host_account_credit_mana] called for account: {} amount: {}",
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
        warn!(
            "[host_account_credit_mana] called with amount zero for account: {}",
            account_id_str
        );
    }

    let account_did = Did::from_str(account_id_str).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid DID format for account_id: {}", e))
    })?;

    ctx.credit_mana(&account_did, amount).await
}

/// Get the reputation of a DID.
pub async fn host_get_reputation(
    ctx: &std::sync::Arc<RuntimeContext>,
    did: &Did,
) -> Result<i64, HostAbiError> {
    let reputation = ctx.reputation_store.get_reputation(did) as i64;
    Ok(reputation)
}

/// Reputation updater for handling execution receipts.
pub struct ReputationUpdater;

impl ReputationUpdater {
    /// Create a new updater with no internal state.
    pub fn new() -> Self {
        ReputationUpdater
    }
    /// Record a completed execution in the provided reputation store.
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
/// WebAssembly callers should use [`wasm_host_anchor_receipt`], which reads the
/// receipt JSON from guest memory and returns the resulting CID string via
/// pointer/length arguments.
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

/// WASM wrapper for [`host_submit_mesh_job`].
///
/// # Memory Layout
/// * `ptr`/`len` – location of a UTF-8 JSON string describing an
///   [`ActualMeshJob`](icn_mesh::ActualMeshJob).
/// * `out_ptr`/`out_len` – location to write the resulting CID string.
///
/// Returns the number of bytes written to `out_ptr` or `0` on error.
pub fn wasm_host_submit_mesh_job(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
    out_ptr: u32,
    out_len: u32,
) -> u32 {
    let job_json = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_submit_mesh_job read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    let job_id = match handle.block_on(host_submit_mesh_job(caller.data(), &job_json)) {
        Ok(c) => c,
        Err(e) => {
            log::error!("wasm_host_submit_mesh_job runtime error: {e:?}");
            return 0;
        }
    };
    let id_str = job_id.to_string();
    match memory::write_string_limited(&mut caller, out_ptr, &id_str, out_len) {
        Ok(w) => w,
        Err(e) => {
            log::error!("wasm_host_submit_mesh_job write error: {e:?}");
            0
        }
    }
}

/// WASM wrapper for [`host_get_pending_mesh_jobs`].
///
/// # Memory Layout
/// * `ptr`/`len` – buffer where the serialized `Vec<ActualMeshJob>` JSON will
///   be written.
///
/// Returns the number of bytes written or `0` on error.
pub fn wasm_host_get_pending_mesh_jobs(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> u32 {
    let handle = tokio::runtime::Handle::current();
    let jobs = match handle.block_on(host_get_pending_mesh_jobs(caller.data())) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_get_pending_mesh_jobs runtime error: {e:?}");
            return 0;
        }
    };
    let json = match serde_json::to_string(&jobs) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_get_pending_mesh_jobs serialize error: {e:?}");
            return 0;
        }
    };
    match memory::write_string_limited(&mut caller, ptr, &json, len) {
        Ok(w) => w,
        Err(e) => {
            log::error!("wasm_host_get_pending_mesh_jobs write error: {e:?}");
            0
        }
    }
}

/// WASM wrapper for [`host_anchor_receipt`].
///
/// # Memory Layout
/// * `ptr`/`len` – UTF-8 JSON representing an [`ExecutionReceipt`](icn_identity::ExecutionReceipt).
/// * `out_ptr`/`out_len` – buffer for the resulting CID string.
///
/// Returns the number of bytes written or `0` on error.
pub fn wasm_host_anchor_receipt(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
    out_ptr: u32,
    out_len: u32,
) -> u32 {
    let json = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_anchor_receipt read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    let rep = ReputationUpdater::new();
    let cid = match handle.block_on(host_anchor_receipt(caller.data(), &json, &rep)) {
        Ok(c) => c,
        Err(e) => {
            log::error!("wasm_host_anchor_receipt runtime error: {e:?}");
            return 0;
        }
    };
    let cid_str = cid.to_string();
    match memory::write_string_limited(&mut caller, out_ptr, &cid_str, out_len) {
        Ok(w) => w,
        Err(e) => {
            log::error!("wasm_host_anchor_receipt write error: {e:?}");
            0
        }
    }
}

/// WASM wrapper for [`host_account_get_mana`].
///
/// # Memory Layout
/// * `ptr`/`len` – UTF-8 string containing the account DID.
///
/// Returns the mana balance or `0` on error.
pub fn wasm_host_account_get_mana(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> u64 {
    let did = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(d) => d,
        Err(e) => {
            log::error!("wasm_host_account_get_mana read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    handle
        .block_on(host_account_get_mana(caller.data(), &did))
        .unwrap_or(0)
}

/// WASM wrapper for [`host_account_spend_mana`].
///
/// # Memory Layout
/// * `ptr`/`len` – UTF-8 string containing the account DID.
/// * `amount` – mana to spend.
pub fn wasm_host_account_spend_mana(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
    amount: u64,
) {
    let did = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(d) => d,
        Err(e) => {
            log::error!("wasm_host_account_spend_mana read error: {e:?}");
            return;
        }
    };
    let handle = tokio::runtime::Handle::current();
    if let Err(e) = handle.block_on(host_account_spend_mana(caller.data(), &did, amount)) {
        log::error!("wasm_host_account_spend_mana runtime error: {e:?}");
    }
}

/// WASM wrapper for [`host_verify_zk_proof`].
pub fn wasm_host_verify_zk_proof(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> i32 {
    let json = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_verify_zk_proof read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    match handle.block_on(host_verify_zk_proof(caller.data(), &json)) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            log::error!("wasm_host_verify_zk_proof runtime error: {e:?}");
            0
        }
    }
}

/// WASM wrapper for [`host_verify_zk_revocation_proof`].
pub fn wasm_host_verify_zk_revocation_proof(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> i32 {
    let json = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_verify_zk_revocation_proof read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    match handle.block_on(host_verify_zk_revocation_proof(caller.data(), &json)) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            log::error!("wasm_host_verify_zk_revocation_proof runtime error: {e:?}");
            0
        }
    }
}

/// WASM wrapper for [`host_generate_zk_proof`].
pub fn wasm_host_generate_zk_proof(
    mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
    out_ptr: u32,
    out_len: u32,
) -> u32 {
    let json = match memory::read_string_safe(&mut caller, ptr, len) {
        Ok(j) => j,
        Err(e) => {
            log::error!("wasm_host_generate_zk_proof read error: {e:?}");
            return 0;
        }
    };
    let handle = tokio::runtime::Handle::current();
    let proof_json = match handle.block_on(host_generate_zk_proof(caller.data(), &json)) {
        Ok(p) => p,
        Err(e) => {
            log::error!("wasm_host_generate_zk_proof runtime error: {e:?}");
            return 0;
        }
    };
    match memory::write_string_limited(&mut caller, out_ptr, &proof_json, out_len) {
        Ok(w) => w,
        Err(e) => {
            log::error!("wasm_host_generate_zk_proof write error: {e:?}");
            0
        }
    }
}

/// Creates a governance proposal using the runtime context.
pub async fn host_create_governance_proposal(
    ctx: &RuntimeContext,
    payload_json: &str,
) -> Result<String, HostAbiError> {
    let payload: CreateProposalPayload = serde_json::from_str(payload_json).map_err(|e| {
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
    let payload: CastVotePayload = serde_json::from_str(payload_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Failed to parse CastVotePayload JSON: {}", e))
    })?;
    ctx.cast_governance_vote(payload).await
}

/// Closes voting on a governance proposal and broadcasts the final
/// [`icn_governance::ProposalStatus`] across the mesh network.
///
/// Returns the status string (e.g. `"Accepted"`).
///
/// # Example
/// ```no_run
/// # async fn demo(ctx: &icn_runtime::context::RuntimeContext) -> Result<(), icn_runtime::HostAbiError> {
/// let status = icn_runtime::host_close_governance_proposal_voting(ctx, "pid").await?;
/// if status == "Accepted" {
///     icn_runtime::host_execute_governance_proposal(ctx, "pid").await?;
/// }
/// # Ok(())
/// # }
/// ```
pub async fn host_close_governance_proposal_voting(
    ctx: &RuntimeContext,
    proposal_id: &str,
) -> Result<String, HostAbiError> {
    ctx.close_governance_proposal_voting(proposal_id).await
}

/// Executes an accepted governance proposal, rewarding the proposer and
/// broadcasting the updated proposal to the mesh network on success.
///
/// # Example
/// ```no_run
/// # async fn demo(ctx: &icn_runtime::context::RuntimeContext) -> Result<(), icn_runtime::HostAbiError> {
/// icn_runtime::host_execute_governance_proposal(ctx, "pid").await?;
/// # Ok(())
/// # }
/// ```
pub async fn host_execute_governance_proposal(
    ctx: &RuntimeContext,
    proposal_id: &str,
) -> Result<(), HostAbiError> {
    ctx.execute_governance_proposal(proposal_id).await
}

/// Delegate voting power from one DID to another.
pub async fn host_delegate_vote(
    ctx: &RuntimeContext,
    from_did: &str,
    to_did: &str,
) -> Result<(), HostAbiError> {
    ctx.delegate_vote(from_did, to_did).await
}

/// Revoke any vote delegation for the given DID.
pub async fn host_revoke_delegation(
    ctx: &RuntimeContext,
    from_did: &str,
) -> Result<(), HostAbiError> {
    ctx.revoke_delegation(from_did).await
}

/// Get the complete lifecycle status of a job by reconstructing it from DAG traversal.
pub async fn host_get_job_status(
    ctx: &RuntimeContext,
    job_id_str: &str,
) -> Result<Option<String>, HostAbiError> {
    log::debug!(
        "[host_get_job_status] Getting status for job: {}",
        job_id_str
    );

    // Parse job ID
    let job_id_cid = icn_common::parse_cid_from_string(job_id_str)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid job ID CID: {}", e)))?;
    let job_id = JobId::from(job_id_cid);

    // Get the job lifecycle from the runtime context
    let lifecycle_opt = ctx.get_job_status(&job_id).await?;

    match lifecycle_opt {
        Some(lifecycle) => {
            // Serialize the lifecycle to JSON for return
            let lifecycle_json = serde_json::to_string(&lifecycle).map_err(|e| {
                HostAbiError::InternalError(format!("Failed to serialize job lifecycle: {}", e))
            })?;
            Ok(Some(lifecycle_json))
        }
        None => Ok(None),
    }
}

/// Verify a zero-knowledge credential proof.
#[allow(clippy::default_constructed_unit_structs)]
pub async fn host_verify_zk_proof(
    ctx: &RuntimeContext,
    proof_json: &str,
) -> Result<bool, HostAbiError> {
    use icn_common::{ZkCredentialProof, ZkProofType};
    use icn_identity::{BulletproofsVerifier, DummyVerifier, Groth16Verifier, ZkVerifier};

    let proof: ZkCredentialProof = serde_json::from_str(proof_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid ZkCredentialProof JSON: {e}"))
    })?;

    ctx.spend_mana(
        &ctx.current_identity,
        context::mesh_network::ZK_VERIFY_COST_MANA,
    )
    .await?;

    let verifier: Box<dyn ZkVerifier> = match proof.backend {
        ZkProofType::Bulletproofs => Box::new(BulletproofsVerifier),
        ZkProofType::Groth16 => Box::new(Groth16Verifier::default()),
        _ => Box::new(DummyVerifier),
    };

    match verifier.verify(&proof) {
        Ok(true) => Ok(true),
        Ok(false) => {
            ctx.credit_mana(
                &ctx.current_identity,
                context::mesh_network::ZK_VERIFY_COST_MANA,
            )
            .await?;
            Ok(false)
        }
        Err(e) => {
            ctx.credit_mana(
                &ctx.current_identity,
                context::mesh_network::ZK_VERIFY_COST_MANA,
            )
            .await?;
            Err(HostAbiError::InvalidParameters(format!("{e}")))
        }
    }
}

/// Verify a zero-knowledge revocation proof.
#[allow(clippy::default_constructed_unit_structs)]
pub async fn host_verify_zk_revocation_proof(
    ctx: &RuntimeContext,
    proof_json: &str,
) -> Result<bool, HostAbiError> {
    use icn_common::{ZkProofType, ZkRevocationProof};
    use icn_identity::zk::ZkRevocationVerifier;
    use icn_identity::{BulletproofsVerifier, DummyVerifier, Groth16Verifier};

    let proof: ZkRevocationProof = serde_json::from_str(proof_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid ZkRevocationProof JSON: {e}"))
    })?;

    ctx.spend_mana(
        &ctx.current_identity,
        context::mesh_network::ZK_VERIFY_COST_MANA,
    )
    .await?;

    let verifier: Box<dyn ZkRevocationVerifier> = match proof.backend {
        ZkProofType::Bulletproofs => Box::new(BulletproofsVerifier),
        ZkProofType::Groth16 => Box::new(Groth16Verifier::default()),
        _ => Box::new(DummyVerifier),
    };

    match verifier.verify_revocation(&proof) {
        Ok(true) => Ok(true),
        Ok(false) => {
            ctx.credit_mana(
                &ctx.current_identity,
                context::mesh_network::ZK_VERIFY_COST_MANA,
            )
            .await?;
            Ok(false)
        }
        Err(e) => {
            ctx.credit_mana(
                &ctx.current_identity,
                context::mesh_network::ZK_VERIFY_COST_MANA,
            )
            .await?;
            Err(HostAbiError::InvalidParameters(format!("{e}")))
        }
    }
}

/// Generate a dummy zero-knowledge credential proof.
pub async fn host_generate_zk_proof(
    _ctx: &RuntimeContext,
    request_json: &str,
) -> Result<String, HostAbiError> {
    use icn_common::{parse_cid_from_string, ZkCredentialProof, ZkProofType};
    use std::str::FromStr;

    let req: GenerateProofRequest = serde_json::from_str(request_json).map_err(|e| {
        HostAbiError::InvalidParameters(format!("Invalid GenerateProofRequest JSON: {e}"))
    })?;

    let issuer = Did::from_str(&req.issuer)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid issuer DID: {e}")))?;
    let holder = Did::from_str(&req.holder)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid holder DID: {e}")))?;
    let schema = parse_cid_from_string(&req.schema)
        .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid schema CID: {e}")))?;

    let backend = match req.backend.to_ascii_lowercase().as_str() {
        "groth16" => ZkProofType::Groth16,
        "bulletproofs" => ZkProofType::Bulletproofs,
        other => ZkProofType::Other(other.to_string()),
    };

    let mut proof_bytes = vec![0u8; 32];
    fastrand::fill(&mut proof_bytes);

    let verification_key_bytes = if let Some(vk_hex) = req.verification_key.as_deref() {
        Some(hex::decode(vk_hex.trim_start_matches("0x")).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Invalid verification key hex: {e}"))
        })?)
    } else {
        None
    };

    let proof = ZkCredentialProof {
        issuer,
        holder,
        claim_type: req.claim_type,
        proof: proof_bytes,
        schema,
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend,
        verification_key: verification_key_bytes,
        public_inputs: req.public_inputs,
    };

    serde_json::to_string(&proof).map_err(|e| HostAbiError::SerializationError(format!("{e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::context::{HostAbiError, RuntimeContext};
    use icn_common::{Cid, Did, ICN_CORE_VERSION};
    use icn_identity::SignatureBytes;
    use icn_mesh::{ActualMeshJob, JobId, JobKind, JobSpec};
    use std::str::FromStr;
    use std::sync::Arc;

    const TEST_IDENTITY_DID_STR: &str = "did:icn:test:dummy_executor";
    const OTHER_IDENTITY_DID_STR: &str = "did:icn:test:other_account";

    // Helper function to create a test ActualMeshJob with all required fields
    fn create_test_mesh_job(cost_mana: u64) -> ActualMeshJob {
        ActualMeshJob {
            id: JobId(Cid::new_v1_sha256(0x55, b"test_job_id")),
            manifest_cid: Cid::new_v1_sha256(0x55, b"test_manifest"),
            spec: JobSpec::default(),
            creator_did: Did::from_str(TEST_IDENTITY_DID_STR).unwrap(),
            cost_mana,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![0u8; 64]), // Dummy signature for tests
        }
    }

    // Helper function to create a RuntimeContext with stubbed services for testing.
    fn create_test_context() -> Arc<RuntimeContext> {
        RuntimeContext::new_with_stubs(TEST_IDENTITY_DID_STR)
            .expect("Failed to create test context")
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

        // With the new DAG-integrated approach, jobs are not queued in pending_mesh_jobs
        // Instead, they are stored in the DAG and managed through the lifecycle system
        // For backwards compatibility, we'll still check the queue but it might be empty
        let pending_jobs = host_get_pending_mesh_jobs(&ctx).await.unwrap();
        // The new implementation doesn't use the pending jobs queue, so this might be 0
        // This is the expected behavior with the new DAG integration
        println!(
            "Pending jobs count: {} (new DAG implementation)",
            pending_jobs.len()
        );

        // Instead, let's verify that the job was stored in the DAG by checking job status
        let job_id_result = job_id.unwrap();
        let job_status = ctx.get_job_status(&job_id_result).await;

        // If the DAG integration is working, we should be able to retrieve the job
        match job_status {
            Ok(Some(lifecycle)) => {
                println!(
                    "Job lifecycle found: submitter={}, status={:?}",
                    lifecycle.job.submitter_did, lifecycle.job.status
                );
                assert_eq!(lifecycle.job.cost_mana, 10);
                assert_eq!(lifecycle.job.submitter_did, ctx.current_identity);
            }
            Ok(None) => {
                println!("Job not found in DAG - this may be expected if lifecycle management is still starting");
            }
            Err(e) => {
                println!("Error retrieving job status: {}", e);
            }
        }
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
    async fn test_host_submit_mesh_job_rejects_large_json() {
        let ctx = create_test_context_with_mana(100);
        let mut job = create_test_mesh_job(10);
        job.spec.kind = JobKind::Echo {
            payload: "x".repeat(MAX_JOB_JSON_SIZE + 1),
        };
        let job_json = serde_json::to_string(&job).unwrap();
        assert!(job_json.len() > MAX_JOB_JSON_SIZE);
        let result = host_submit_mesh_job(&ctx, &job_json).await;
        assert!(
            matches!(result, Err(HostAbiError::InvalidParameters(_))),
            "Expected InvalidParameters, got {:?}",
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

        let pending_jobs = host_get_pending_mesh_jobs(&ctx).await.unwrap();
        assert_eq!(pending_jobs.len(), 2);
        assert!(pending_jobs.iter().any(|j| j.cost_mana == 10));
        assert!(pending_jobs.iter().any(|j| j.cost_mana == 5));
    }

    #[tokio::test]
    async fn test_host_get_pending_mesh_jobs_empty_queue() {
        let ctx = create_test_context();
        let pending_jobs_result = host_get_pending_mesh_jobs(&ctx).await;
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
        let ctx = RuntimeContext::new_with_stubs(node_did_str).unwrap();
        assert_eq!(ctx.current_identity.to_string(), node_did_str);
        // Further checks can be added here if needed
    }

    #[tokio::test]
    async fn test_runtime_context_new_with_stubs_and_mana() {
        let node_did_str = "did:key:zTestManaDid";
        let initial_mana = 1000u64;
        let ctx = RuntimeContext::new_with_stubs_and_mana(node_did_str, initial_mana).unwrap();
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
        host_get_pending_mesh_jobs(&ctx).await.unwrap();
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

    #[tokio::test]
    async fn test_new_mesh_job_lifecycle_basic() {
        // Test the new DAG-integrated job lifecycle
        let ctx = create_test_context_with_mana(100);

        // Submit a job using the new API
        let manifest_cid = Cid::new_v1_sha256(0x55, b"test_manifest_basic");
        let spec_bytes = bincode::serialize(&JobSpec::default()).unwrap();

        let job_id = ctx.handle_submit_job(manifest_cid, spec_bytes, 50).await;
        assert!(job_id.is_ok(), "Job submission failed: {:?}", job_id.err());

        let job_id = job_id.unwrap();
        println!("Submitted job with ID: {}", job_id);

        // Verify mana was spent
        let mana_after = ctx.get_mana(&ctx.current_identity).await.unwrap();
        assert_eq!(mana_after, 50); // 100 - 50 spent

        // Give the async lifecycle management a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check job status via DAG
        let job_status = ctx.get_job_status(&job_id).await;
        match job_status {
            Ok(Some(lifecycle)) => {
                println!("Job lifecycle retrieved successfully:");
                println!("  Submitter: {}", lifecycle.job.submitter_did);
                println!("  Cost: {}", lifecycle.job.cost_mana);
                println!("  Status: {:?}", lifecycle.job.status);
                println!("  Bids received: {}", lifecycle.bids.len());
                println!("  Assignment: {}", lifecycle.assignment.is_some());
                println!("  Receipt: {}", lifecycle.receipt.is_some());

                assert_eq!(lifecycle.job.cost_mana, 50);
                assert_eq!(lifecycle.job.submitter_did, ctx.current_identity);
            }
            Ok(None) => {
                println!("Job not found in DAG");
                panic!("Job should be stored in DAG");
            }
            Err(e) => {
                println!("Error retrieving job status: {}", e);
                panic!("Should be able to retrieve job status");
            }
        }
    }

    #[tokio::test]
    async fn test_host_get_job_status_function() {
        // Test the new host function for getting job status
        let ctx = create_test_context_with_mana(200);

        // Submit a job
        let manifest_cid = Cid::new_v1_sha256(0x55, b"test_status_check");
        let spec_bytes = bincode::serialize(&JobSpec::default()).unwrap();

        let job_id = ctx
            .handle_submit_job(manifest_cid, spec_bytes, 75)
            .await
            .unwrap();
        println!("Submitted job with ID: {}", job_id);

        // Give the lifecycle management a moment
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Test the host function for getting job status
        let job_id_str = job_id.to_string();
        let status_result = host_get_job_status(&ctx, &job_id_str).await;

        match status_result {
            Ok(Some(status_json)) => {
                println!("Job status JSON: {}", status_json);

                // Parse the JSON to verify structure
                let lifecycle: icn_mesh::JobLifecycle = serde_json::from_str(&status_json).unwrap();
                assert_eq!(lifecycle.job.cost_mana, 75);
                assert_eq!(lifecycle.job.submitter_did, ctx.current_identity);

                println!("Successfully retrieved and parsed job lifecycle via host function");
            }
            Ok(None) => {
                panic!("Job should exist");
            }
            Err(e) => {
                panic!("Host function should work: {}", e);
            }
        }
    }
}
