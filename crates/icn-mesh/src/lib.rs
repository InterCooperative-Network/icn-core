#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN Mesh Crate
//! This crate focuses on job orchestration, scheduling, and execution within the
//! InterCooperative Network (ICN) mesh network. It handles job definition, resource discovery,
//! scheduling, execution management, and fault tolerance.

use icn_common::{Cid, CommonError, Did, NodeInfo};
use icn_identity::{
    sign_message as identity_sign_message, verify_signature as identity_verify_signature,
    ExecutionReceipt, SignatureBytes, SigningKey as IdentitySigningKey,
    VerifyingKey as IdentityVerifyingKey,
};
use serde::{Deserialize, Serialize};
pub mod aid;
pub mod metrics;

/// Unique identifier for a mesh job.
///
/// Wraps a [`Cid`] to enforce type safety when referencing jobs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Cid);

impl From<Cid> for JobId {
    fn from(c: Cid) -> Self {
        JobId(c)
    }
}

impl From<JobId> for Cid {
    fn from(j: JobId) -> Self {
        j.0
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Execution resource capabilities offered in a bid.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resources {
    /// Number of CPU cores available for the job.
    pub cpu_cores: u32,
    /// Amount of memory in megabytes available for the job.
    pub memory_mb: u32,
    /// Amount of storage in megabytes required or offered for the job.
    #[serde(default)]
    pub storage_mb: u32,
}

/// Represents a job submitted to the ICN mesh computing network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualMeshJob {
    /// Unique identifier for this job instance (typically a CID of core job details).
    pub id: JobId,
    /// Content Identifier (CID) of the job's core executable or primary data package.
    pub manifest_cid: Cid,
    /// Detailed specification of the job, including inputs, outputs, and execution requirements.
    pub spec: JobSpec,
    /// Decentralized Identifier (DID) of the entity that submitted the job.
    pub creator_did: Did,
    /// The amount of mana allocated by the submitter for this job's execution.
    pub cost_mana: u64,
    /// Maximum time in milliseconds the submitter is willing to wait for a receipt.
    /// If `None`, the runtime will use its configured default timeout.
    #[serde(default)]
    pub max_execution_wait_ms: Option<u64>,
    /// Signature from the creator_did over the (id, manifest_cid, spec_hash (if spec is large), creator_did, cost_mana)
    pub signature: SignatureBytes,
}

impl ActualMeshJob {
    /// Creates the canonical message bytes for signing the job.
    /// The fields must be serialized in a deterministic way.
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.id.to_string().as_bytes());
        bytes.extend_from_slice(self.manifest_cid.to_string().as_bytes());
        bytes.extend_from_slice(self.creator_did.to_string().as_bytes());
        bytes.extend_from_slice(&self.cost_mana.to_le_bytes());
        Ok(bytes)
    }

    /// Signs this job with the provided Ed25519 SigningKey.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        // Ensure the job_id is set before signing, as it's part of the signable bytes.
        // Typically, id would be a CID of some core content, generated before this step.
        if self.id.to_string().is_empty() || self.id.to_string().len() < 4 {
            // Basic check, using to_string().as_bytes()
            return Err(CommonError::InternalError(
                "Job ID must be set before signing".to_string(),
            )); // Was InvalidParameters
        }
        let message = self.to_signable_bytes()?;
        let ed_signature = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_signature.to_bytes().to_vec());
        Ok(self)
    }

    /// Verifies the signature of this job against the provided Ed25519 VerifyingKey.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_signature = self.signature.to_ed_signature()?;

        if identity_verify_signature(verifying_key, &message, &ed_signature) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "ActualMeshJob signature verification failed".to_string(),
            )) // Was CryptographyError
        }
    }
}

/// Kinds of mesh jobs that can be executed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum JobKind {
    /// Simple echo job used for basic integration tests.
    Echo { payload: String },
    /// Execute a compiled CCL WASM module referenced by the job's `manifest_cid`.
    ///
    /// When this variant is used the runtime will immediately load the WASM
    /// bytes from the DAG store and invoke its `run` export using the built-in
    /// WASM executor.
    CclWasm,
    /// Placeholder until more kinds are defined.
    #[default]
    GenericPlaceholder,
}

impl JobKind {
    /// Returns `true` if this job represents a compiled CCL WASM module.
    pub fn is_ccl_wasm(&self) -> bool {
        matches!(self, JobKind::CclWasm)
    }
}

/// Detailed specification for a mesh job.
///
/// A `JobSpec` describes the inputs required to execute a job, the expected
/// outputs, and the minimum resources needed for successful execution.  The
/// `kind` field determines how the job should be interpreted (e.g. echo test,
/// WASM module, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobSpec {
    /// The high level kind of job.
    pub kind: JobKind,
    /// CIDs of input data necessary to run the job.
    #[serde(default)]
    pub inputs: Vec<Cid>,
    /// Logical names for outputs that the executor is expected to produce.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Minimum resources required for the job.
    #[serde(default)]
    pub required_resources: Resources,
    /// Required capabilities that executors must have to bid on this job.
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    /// Trust scope requirements for federation-aware job execution.
    #[serde(default)]
    pub required_trust_scope: Option<String>,
    /// Minimum reputation score required for executors.
    #[serde(default)]
    pub min_executor_reputation: Option<u64>,
    /// Federation constraints - only executors from these federations can bid.
    #[serde(default)]
    pub allowed_federations: Vec<String>,
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            kind: JobKind::GenericPlaceholder,
            inputs: Vec::new(),
            outputs: Vec::new(),
            required_resources: Resources::default(),
            required_capabilities: Vec::new(),
            required_trust_scope: None,
            min_executor_reputation: None,
            allowed_federations: Vec::new(),
        }
    }
}

/// Represents a bid submitted by an executor node for a specific mesh job.
#[derive(Debug, Clone, Serialize, Deserialize)] // Added Serialize, Deserialize
pub struct MeshJobBid {
    /// Identifier of the job this bid is for.
    pub job_id: JobId,
    /// Decentralized Identifier (DID) of the executor node submitting the bid.
    pub executor_did: Did,
    /// The price (in mana or a defined token) the executor is charging for the job.
    pub price_mana: u64,
    /// The resources the executor is committing for this job.
    pub resources: Resources,
    /// Capabilities that this executor offers.
    #[serde(default)]
    pub executor_capabilities: Vec<String>,
    /// Federation memberships of the executor.
    #[serde(default)]
    pub executor_federations: Vec<String>,
    /// Trust scope of the executor for this job.
    #[serde(default)]
    pub executor_trust_scope: Option<String>,
    /// Signature from the executor over the bid fields.
    pub signature: SignatureBytes,
}

impl MeshJobBid {
    /// Creates the canonical message bytes for signing the bid.
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.job_id.to_string().as_bytes());
        bytes.extend_from_slice(self.executor_did.to_string().as_bytes());
        bytes.extend_from_slice(&self.price_mana.to_le_bytes());
        bytes.extend_from_slice(&self.resources.cpu_cores.to_le_bytes());
        bytes.extend_from_slice(&self.resources.memory_mb.to_le_bytes());
        bytes.extend_from_slice(&self.resources.storage_mb.to_le_bytes());

        // Include federation metadata in signature
        for capability in &self.executor_capabilities {
            bytes.extend_from_slice(capability.as_bytes());
        }
        for federation in &self.executor_federations {
            bytes.extend_from_slice(federation.as_bytes());
        }
        if let Some(trust_scope) = &self.executor_trust_scope {
            bytes.extend_from_slice(trust_scope.as_bytes());
        }

        Ok(bytes)
    }

    /// Sign the bid using the executor's signing key.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    /// Verify the bid signature against the executor's verifying key.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "MeshJobBid signature verification failed".into(),
            ))
        }
    }
}

/// Represents the current state of a mesh job in its lifecycle.
#[derive(Debug, Clone)] // Added Clone if JobState needs to be stored/copied
pub enum JobState {
    /// The job has been submitted and is awaiting executor assignment.
    Pending,
    /// The job has been assigned to an executor.
    Assigned { executor: Did },
    /// The job has been completed successfully by an executor.
    Completed { receipt: ExecutionReceipt },
    /// The job failed to complete due to an error.
    Failed { reason: String },
}

/// Policy configuration for executor selection.
///
/// Each weight defines how much influence a factor has when calculating a bid's
/// score via [`score_bid`]. A higher value means the factor contributes more to
/// the final score.
#[derive(Debug, Clone)]
pub struct SelectionPolicy {
    /// Weight applied to the bid price (inverse).
    pub weight_price: f64,
    /// Weight applied to the executor's reputation score.
    pub weight_reputation: f64,
    /// Weight applied to the offered resources.
    pub weight_resources: f64,
    /// Weight applied to the network latency score.
    pub weight_latency: f64,
}

impl Default for SelectionPolicy {
    fn default() -> Self {
        Self {
            weight_price: 1.0,
            weight_reputation: 50.0,
            weight_resources: 1.0,
            weight_latency: 1.0,
        }
    }
}

/// Provides latency information for executors.
pub trait LatencyStore: Send + Sync {
    /// Return the round-trip latency in milliseconds for the given executor.
    fn get_latency(&self, did: &Did) -> Option<u64>;
}

/// Latency store that always returns `None`.
pub struct NoOpLatencyStore;

impl LatencyStore for NoOpLatencyStore {
    fn get_latency(&self, _did: &Did) -> Option<u64> {
        None
    }
}

/// Provides dynamic capability checking for executors beyond static bid capabilities.
///
/// This allows for runtime verification of executor capabilities, such as:
/// - Real-time resource availability
/// - Dynamic feature flags
/// - Network connectivity requirements
/// - Hardware-specific capabilities that may change
pub trait DynamicCapabilityChecker: Send + Sync {
    /// Check if an executor currently has the specified capability.
    ///
    /// # Arguments
    /// * `executor_did` - The DID of the executor to check
    /// * `capability` - The capability name to verify
    /// * `context` - Optional context data for capability checking
    ///
    /// # Returns
    /// * `Some(true)` if the executor has the capability
    /// * `Some(false)` if the executor definitively lacks the capability  
    /// * `None` if the capability status cannot be determined
    fn check_capability(
        &self,
        executor_did: &Did,
        capability: &str,
        context: Option<&str>,
    ) -> Option<bool>;

    /// Get current resource availability for an executor.
    ///
    /// Returns `None` if real-time resource information is not available.
    fn get_current_resources(&self, executor_did: &Did) -> Option<Resources>;

    /// Check if an executor is currently available for new jobs.
    fn is_executor_available(&self, executor_did: &Did) -> bool {
        true // Default implementation assumes availability
    }
}

/// No-op implementation of DynamicCapabilityChecker that always returns None/true.
pub struct NoOpCapabilityChecker;

impl DynamicCapabilityChecker for NoOpCapabilityChecker {
    fn check_capability(
        &self,
        _executor_did: &Did,
        _capability: &str,
        _context: Option<&str>,
    ) -> Option<bool> {
        None
    }

    fn get_current_resources(&self, _executor_did: &Did) -> Option<Resources> {
        None
    }

    fn is_executor_available(&self, _executor_did: &Did) -> bool {
        true
    }
}

/// Helper type that wraps bid selection based on reputation.
pub struct ReputationExecutorSelector;

impl ReputationExecutorSelector {
    // This struct might not be strictly needed if select_executor handles scoring directly,
    // or it could encapsulate more complex stateful selection logic in the future.
    // For now, its select method can be a simple helper or be removed if select_executor is self-contained.
    /// Returns the executor DID with the highest bid score according to the
    /// provided policy and reputation store.
    pub fn select(
        &self,
        bids: &[MeshJobBid],
        job_spec: &JobSpec,
        policy: &SelectionPolicy,
        reputation_store: &dyn icn_reputation::ReputationStore,
        mana_ledger: &dyn icn_economics::ManaLedger,
        capability_checker: &dyn DynamicCapabilityChecker,
    ) -> Option<Did> {
        // This helper is retained for future stateful selection logic.
        bids.iter()
            .map(|bid| {
                let balance = mana_ledger.get_balance(&bid.executor_did);
                (bid, balance)
            })
            .max_by_key(|(bid, balance)| {
                score_bid(
                    bid,
                    job_spec,
                    policy,
                    reputation_store,
                    *balance,
                    None,
                    capability_checker,
                )
            })
            .map(|(bid, _)| bid.executor_did.clone())
    }
}

/// Selects the best executor from a list of bids.
///
/// Each bid is scored via [`score_bid`] using the provided [`SelectionPolicy`].
/// Bids from executors that lack sufficient mana are ignored. The executor with
/// the highest resulting score is returned.
///
/// # Arguments
/// * `job_id` - The ID of the job for which an executor is being selected.
/// * `bids` - A vector of `Bid` structs received for a specific job.
/// * `policy` - The `SelectionPolicy` to apply for choosing the best executor.
/// * `reputation_store` - Source of reputation scores for executors.
/// * `latency_store` - Source of network latency information for executors.
/// * `capability_checker` - Dynamic capability validation for executors.
///
/// # Returns
/// * `Some(Did)` of the selected executor if a suitable one is found.
/// * `None` if no suitable executor could be selected based on the bids and policy.
pub fn select_executor(
    job_id: &JobId,
    job_spec: &JobSpec,
    bids: Vec<MeshJobBid>,
    policy: &SelectionPolicy,
    reputation_store: &dyn icn_reputation::ReputationStore,
    mana_ledger: &dyn icn_economics::ManaLedger,
    latency_store: &dyn LatencyStore,
    capability_checker: &dyn DynamicCapabilityChecker,
) -> Option<Did> {
    metrics::SELECT_EXECUTOR_CALLS.inc();
    // Iterate over bids and pick the executor with the highest score as
    // determined by `score_bid`. Bids from executors without enough mana are
    // ignored.
    log::debug!(
        "[Mesh] Selecting executor for job {:?}. Received {} bids.",
        job_id,
        bids.len()
    );

    bids.iter()
        .filter(|bid| {
            // Basic mana check
            if mana_ledger.get_balance(&bid.executor_did) < bid.price_mana {
                return false;
            }

            // Dynamic availability check
            if !capability_checker.is_executor_available(&bid.executor_did) {
                log::debug!(
                    "[Mesh] Executor {} not currently available for job {:?}",
                    bid.executor_did,
                    job_id
                );
                return false;
            }

            true
        })
        .map(|bid| {
            let balance = mana_ledger.get_balance(&bid.executor_did);
            let latency = latency_store.get_latency(&bid.executor_did);
            let score = score_bid(
                bid,
                job_spec,
                policy,
                reputation_store,
                balance,
                latency,
                capability_checker,
            );
            (bid, score)
        })
        .max_by_key(|(_, score)| *score)
        .map(|(bid, _)| bid.executor_did.clone())
}

/// Scores a single bid according to a [`SelectionPolicy`].
///
/// The resulting value combines three components:
/// price (lower is better), executor reputation, and resources offered by the
/// bid. Each component is multiplied by its weight from the policy and then
/// summed. Bids from executors that cannot afford the `price_mana` return a
/// score of zero.
///
/// # Arguments
/// * `bid` - The `Bid` to score.
/// * `policy` - The `SelectionPolicy` to use for calculating the score.
/// * `reputation_store` - Source of reputation scores for executors.
/// * `available_mana` - Mana balance of the executor submitting the bid.
/// * `latency_ms` - Observed network latency to the executor in milliseconds.
///
/// # Returns
/// * A `u64` representing the calculated score for the bid. Higher is generally better.
pub fn score_bid(
    bid: &MeshJobBid,
    job_spec: &JobSpec,
    policy: &SelectionPolicy,
    reputation_store: &dyn icn_reputation::ReputationStore,
    available_mana: u64,
    latency_ms: Option<u64>,
    capability_checker: &dyn DynamicCapabilityChecker,
) -> u64 {
    if available_mana < bid.price_mana {
        return 0;
    }

    // Federation and capability filtering
    // If job specifies allowed federations, executor must be in one of them
    if !job_spec.allowed_federations.is_empty() {
        let has_allowed_federation = bid
            .executor_federations
            .iter()
            .any(|fed| job_spec.allowed_federations.contains(fed));
        if !has_allowed_federation {
            return 0; // Executor not in allowed federation
        }
    }

    // Check required capabilities
    if !job_spec.required_capabilities.is_empty() {
        let has_all_capabilities = job_spec
            .required_capabilities
            .iter()
            .all(|req_cap| bid.executor_capabilities.contains(req_cap));
        if !has_all_capabilities {
            return 0; // Executor missing required capabilities
        }
    }

    // Dynamic capability validation
    for capability in &job_spec.required_capabilities {
        match capability_checker.check_capability(&bid.executor_did, capability, None) {
            Some(false) => {
                log::debug!(
                    "[Mesh] Executor {} failed dynamic capability check for '{}'",
                    bid.executor_did,
                    capability
                );
                return 0; // Executor dynamically lacks required capability
            }
            Some(true) => {
                log::debug!(
                    "[Mesh] Executor {} passed dynamic capability check for '{}'",
                    bid.executor_did,
                    capability
                );
            }
            None => {
                // Cannot determine capability status dynamically, rely on static bid
                log::debug!(
                    "[Mesh] Could not determine dynamic capability '{}' for executor {}, using static bid data",
                    capability,
                    bid.executor_did
                );
            }
        }
    }

    // Check minimum reputation requirement
    if let Some(min_rep) = job_spec.min_executor_reputation {
        let executor_reputation = reputation_store.get_reputation(&bid.executor_did);
        if executor_reputation < min_rep {
            return 0; // Executor reputation below minimum
        }
    }

    // Check trust scope match
    if let Some(required_scope) = &job_spec.required_trust_scope {
        if let Some(executor_scope) = &bid.executor_trust_scope {
            if executor_scope != required_scope {
                return 0; // Trust scope mismatch
            }
        } else {
            return 0; // Job requires trust scope but executor doesn't provide one
        }
    }

    // Lower prices rank higher, so we invert the price.
    let price_score = if bid.price_mana > 0 {
        policy.weight_price / bid.price_mana as f64
    } else {
        policy.weight_price
    };

    let reputation_score =
        policy.weight_reputation * reputation_store.get_reputation(&bid.executor_did) as f64;

    // Determine how well the offered resources satisfy the job requirements.
    // Check if dynamic resources are available and use them if more current
    let effective_resources = capability_checker
        .get_current_resources(&bid.executor_did)
        .unwrap_or_else(|| bid.resources.clone());

    let req = &job_spec.required_resources;
    let resource_match = if effective_resources.cpu_cores >= req.cpu_cores
        && effective_resources.memory_mb >= req.memory_mb
    {
        let cpu_ratio = effective_resources.cpu_cores as f64 / req.cpu_cores.max(1) as f64;
        let mem_ratio = effective_resources.memory_mb as f64 / req.memory_mb.max(1) as f64;
        // Bonus for having significantly more resources than required
        let bonus = if cpu_ratio > 2.0 || mem_ratio > 2.0 {
            0.1
        } else {
            0.0
        };
        (cpu_ratio + mem_ratio) / 2.0 + bonus
    } else {
        // Insufficient resources yields zero score
        0.0
    };

    let resource_score = policy.weight_resources * resource_match;

    let latency_score = latency_ms
        .and_then(|ms| {
            if ms > 0 {
                Some(policy.weight_latency / ms as f64)
            } else {
                None
            }
        })
        .unwrap_or(0.0);

    let weighted = price_score + reputation_score + resource_score + latency_score;

    weighted.max(0.0) as u64
}

/// Placeholder function demonstrating use of common types for mesh operations.
pub fn schedule_mesh_job(info: &NodeInfo, job_id: &str) -> Result<String, CommonError> {
    metrics::SCHEDULE_MESH_JOB_CALLS.inc();
    Ok(format!(
        "Scheduled mesh job {} on node: {} (v{})",
        job_id, info.name, info.version
    ))
}

// --- Mesh Protocol Messages ---

/// Message broadcast by a node to announce a new mesh job that requires executors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobAnnounce {
    /// The ID of the job being announced.
    pub job_id: JobId, // This is a Cid
    /// The CID of the job manifest, allowing potential executors to get more details.
    pub manifest_cid: Cid,
    /// The DID of the job creator/submitter.
    pub creator_did: Did,
    /// The maximum mana the creator is willing to pay.
    pub cost_mana: u64,
    /// Key requirements from the [`JobSpec`] so executors can filter
    /// announcements without downloading the full manifest.
    #[serde(default)]
    pub job_kind: JobKind,
    /// Minimal resources required for the job.
    #[serde(default)]
    pub required_resources: Resources,
}

/// Message sent by an executor to the job's originating node to submit a bid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBidSubmit {
    /// The bid being submitted.
    pub bid: MeshJobBid,
    /// Signature from the executor over the bid fields to authenticate the
    /// message. This should be created using the same key as
    /// `bid.executor_did`.
    pub signature: SignatureBytes,
}

impl MeshBidSubmit {
    /// Bytes used when signing the bid submission.
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        self.bid.to_signable_bytes()
    }

    /// Sign the submission using the executor's signing key.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    /// Verify both the embedded bid and this submission signature.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        self.bid.verify_signature(verifying_key)?;
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "MeshBidSubmit signature verification failed".into(),
            ))
        }
    }
}

/// Message broadcast by the Job Manager to announce the selected executor for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignmentNotice {
    /// The ID of the job that has been assigned.
    pub job_id: JobId,
    /// The DID of the executor that has been assigned the job.
    pub executor_did: Did,
    /// Signature from the job manager confirming this assignment. The
    /// signing key is typically tied to the manager's DID.
    pub signature: SignatureBytes,
    /// Optional CID of the job manifest for convenience.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_cid: Option<Cid>,
}

impl JobAssignmentNotice {
    /// Bytes that must be signed by the job manager when announcing the
    /// assignment.
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.job_id.to_string().as_bytes());
        bytes.extend_from_slice(self.executor_did.to_string().as_bytes());
        Ok(bytes)
    }

    /// Sign this notice with the provided key.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    /// Verify the signature with the manager's verifying key.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "JobAssignmentNotice signature verification failed".into(),
            ))
        }
    }
}

/// Message sent by an Executor to the Job Manager to submit an ExecutionReceipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReceiptMessage {
    /// The execution receipt being submitted.
    pub receipt: icn_identity::ExecutionReceipt,
    /// Optional signature from the executor over the receipt bytes.
    pub signature: SignatureBytes,
}

impl SubmitReceiptMessage {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        self.receipt.to_signable_bytes()
    }

    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "SubmitReceiptMessage signature verification failed".into(),
            ))
        }
    }
}

// --- DAG Lifecycle Structs ---

/// Represents a mesh job stored in the DAG for lifecycle tracking.
/// This is the authoritative record of a job submission event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job (matches the DAG CID).
    pub id: JobId,
    /// Content Identifier (CID) of the job's executable or data package.
    pub manifest_cid: Cid,
    /// Binary-encoded specification of the job (bincode serialized [`JobSpec`]).
    pub spec_bytes: Vec<u8>,
    /// **Deprecated** JSON-serialized job spec for backward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_json: Option<String>,
    /// Decentralized Identifier (DID) of the entity that submitted the job.
    pub submitter_did: Did,
    /// The amount of mana allocated for this job's execution.
    pub cost_mana: u64,
    /// Timestamp when the job was submitted.
    pub submitted_at: u64,
    /// Current status of the job.
    pub status: JobLifecycleStatus,
    /// Optional resource requirements for the job.
    pub resource_requirements: Resources,
}

impl Job {
    /// Decode the job specification from `spec_bytes` or the deprecated
    /// `spec_json` field.
    pub fn decode_spec(&self) -> Result<JobSpec, CommonError> {
        if !self.spec_bytes.is_empty() {
            bincode::deserialize(&self.spec_bytes).map_err(|e| {
                CommonError::InternalError(format!("Failed to decode spec_bytes: {}", e))
            })
        } else if let Some(json) = &self.spec_json {
            serde_json::from_str(json).map_err(|e| {
                CommonError::InternalError(format!("Failed to decode spec_json: {}", e))
            })
        } else {
            Err(CommonError::InternalError("Job spec missing".into()))
        }
    }
}

/// Represents a bid stored in the DAG, linked to a specific job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobBid {
    /// The ID of the job this bid is for.
    pub job_id: JobId,
    /// Unique identifier for this bid.
    pub bid_id: String,
    /// Decentralized Identifier (DID) of the executor submitting the bid.
    pub executor_did: Did,
    /// The price (in mana) the executor is charging for the job.
    pub price_mana: u64,
    /// The resources the executor is committing for this job.
    pub resources: Resources,
    /// Timestamp when the bid was submitted.
    pub submitted_at: u64,
    /// Signature from the executor over the bid fields.
    pub signature: SignatureBytes,
}

/// Represents a job assignment stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignment {
    /// The ID of the job that has been assigned.
    pub job_id: JobId,
    /// The ID of the winning bid.
    pub winning_bid_id: String,
    /// The DID of the executor that has been assigned the job.
    pub assigned_executor_did: Did,
    /// Timestamp when the assignment was made.
    pub assigned_at: u64,
    /// Final negotiated price for the job.
    pub final_price_mana: u64,
    /// Resources committed by the assigned executor.
    pub committed_resources: Resources,
}

/// Represents an execution receipt stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobReceipt {
    /// The ID of the job that was executed.
    pub job_id: JobId,
    /// The executor that completed the job.
    pub executor_did: Did,
    /// Whether the job executed successfully.
    pub success: bool,
    /// CPU time used in milliseconds.
    pub cpu_ms: u64,
    /// CID of the result data.
    pub result_cid: Cid,
    /// Timestamp when execution completed.
    pub completed_at: u64,
    /// Any error message if execution failed.
    pub error_message: Option<String>,
    /// Signature from the executor.
    pub signature: SignatureBytes,
}

/// Status of a job in its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobLifecycleStatus {
    /// Job has been submitted and is awaiting bids.
    Submitted,
    /// Job is collecting bids from executors.
    BiddingOpen,
    /// Bidding period has closed, selection in progress.
    BiddingClosed,
    /// Job has been assigned to an executor.
    Assigned,
    /// Job is currently being executed.
    Executing,
    /// Job has completed successfully.
    Completed,
    /// Job execution failed.
    Failed,
    /// Job was cancelled before completion.
    Cancelled,
}

impl JobLifecycleStatus {
    /// Returns true if the job is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobLifecycleStatus::Completed
                | JobLifecycleStatus::Failed
                | JobLifecycleStatus::Cancelled
        )
    }

    /// Returns true if the job is active (can still change state).
    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }
}

/// Complete lifecycle information for a job, reconstructed from DAG traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLifecycle {
    /// The core job information.
    pub job: Job,
    /// All bids received for this job.
    pub bids: Vec<JobBid>,
    /// Assignment information if the job was assigned.
    pub assignment: Option<JobAssignment>,
    /// Execution receipt if the job was completed.
    pub receipt: Option<JobReceipt>,
    /// Checkpoints saved during job execution.
    #[serde(default)]
    pub checkpoints: Vec<JobCheckpoint>,
    /// Partial output receipts for long-running jobs.
    #[serde(default)]
    pub partial_outputs: Vec<PartialOutputReceipt>,
}

// --- Long-Running Job Support ---

/// Represents a snapshot of a job's execution state for long-running jobs.
/// This allows jobs to be resumed from checkpoints if they are interrupted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCheckpoint {
    /// The ID of the job this checkpoint belongs to.
    pub job_id: JobId,
    /// Unique identifier for this checkpoint.
    pub checkpoint_id: String,
    /// Timestamp when the checkpoint was created.
    pub timestamp: u64,
    /// Current execution stage/step.
    pub stage: String,
    /// Percentage completion (0-100).
    pub progress_percent: f32,
    /// Serialized execution state that can be used to resume.
    pub execution_state: Vec<u8>,
    /// CID of any intermediate data produced up to this point.
    pub intermediate_data_cid: Option<Cid>,
    /// Executor that created this checkpoint.
    pub executor_did: Did,
    /// Signature from the executor over the checkpoint data.
    pub signature: SignatureBytes,
}

impl JobCheckpoint {
    /// Creates the canonical message bytes for signing the checkpoint.
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.job_id.to_string().as_bytes());
        bytes.extend_from_slice(self.checkpoint_id.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(self.stage.as_bytes());
        bytes.extend_from_slice(&self.progress_percent.to_le_bytes());
        bytes.extend_from_slice(self.executor_did.to_string().as_bytes());
        if let Some(cid) = &self.intermediate_data_cid {
            bytes.extend_from_slice(cid.to_string().as_bytes());
        }
        Ok(bytes)
    }

    /// Sign the checkpoint using the executor's signing key.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    /// Verify the checkpoint signature against the executor's verifying key.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "JobCheckpoint signature verification failed".into(),
            ))
        }
    }
}

/// Represents intermediate output produced during a long-running job execution.
/// Multiple partial outputs can be produced for a single job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialOutputReceipt {
    /// The ID of the job this partial output belongs to.
    pub job_id: JobId,
    /// Unique identifier for this partial output.
    pub output_id: String,
    /// The execution stage that produced this output.
    pub stage: String,
    /// Timestamp when this output was produced.
    pub timestamp: u64,
    /// CID of the partial output data.
    pub output_cid: Cid,
    /// Size of the output data in bytes.
    pub output_size: u64,
    /// MIME type or format of the output.
    pub output_format: Option<String>,
    /// Executor that produced this output.
    pub executor_did: Did,
    /// Signature from the executor over the output receipt.
    pub signature: SignatureBytes,
}

impl PartialOutputReceipt {
    /// Creates the canonical message bytes for signing the partial output receipt.
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.job_id.to_string().as_bytes());
        bytes.extend_from_slice(self.output_id.as_bytes());
        bytes.extend_from_slice(self.stage.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(self.output_cid.to_string().as_bytes());
        bytes.extend_from_slice(&self.output_size.to_le_bytes());
        bytes.extend_from_slice(self.executor_did.to_string().as_bytes());
        if let Some(format) = &self.output_format {
            bytes.extend_from_slice(format.as_bytes());
        }
        Ok(bytes)
    }

    /// Sign the partial output receipt using the executor's signing key.
    pub fn sign(mut self, signing_key: &IdentitySigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_sig.to_bytes().to_vec());
        Ok(self)
    }

    /// Verify the partial output receipt signature against the executor's verifying key.
    pub fn verify_signature(
        &self,
        verifying_key: &IdentityVerifyingKey,
    ) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_sig = self.signature.to_ed_signature()?;
        if identity_verify_signature(verifying_key, &message, &ed_sig) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "PartialOutputReceipt signature verification failed".into(),
            ))
        }
    }
}

/// Represents the current progress status of a job execution.
/// This is used for real-time progress reporting and streaming APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    /// The ID of the job this progress report belongs to.
    pub job_id: JobId,
    /// Current execution stage/step.
    pub current_stage: String,
    /// Percentage completion (0.0-100.0).
    pub progress_percent: f32,
    /// Estimated time to completion in seconds (if available).
    pub eta_seconds: Option<u64>,
    /// Human-readable progress message.
    pub message: String,
    /// Timestamp when this progress was reported.
    pub timestamp: u64,
    /// Executor reporting this progress.
    pub executor_did: Did,
    /// List of completed stages.
    #[serde(default)]
    pub completed_stages: Vec<String>,
    /// List of remaining stages.
    #[serde(default)]
    pub remaining_stages: Vec<String>,
}

impl JobLifecycle {
    /// Create a new lifecycle from just the job.
    pub fn new(job: Job) -> Self {
        Self {
            job,
            bids: Vec::new(),
            assignment: None,
            receipt: None,
            checkpoints: Vec::new(),
            partial_outputs: Vec::new(),
        }
    }

    /// Add a bid to this lifecycle.
    pub fn add_bid(&mut self, bid: JobBid) {
        self.bids.push(bid);
    }

    /// Set the assignment for this lifecycle.
    pub fn set_assignment(&mut self, assignment: JobAssignment) {
        self.assignment = Some(assignment);
    }

    /// Set the receipt for this lifecycle.
    pub fn set_receipt(&mut self, receipt: JobReceipt) {
        self.receipt = Some(receipt);
    }

    /// Add a checkpoint to this lifecycle.
    pub fn add_checkpoint(&mut self, checkpoint: JobCheckpoint) {
        self.checkpoints.push(checkpoint);
    }

    /// Add a partial output receipt to this lifecycle.
    pub fn add_partial_output(&mut self, partial_output: PartialOutputReceipt) {
        self.partial_outputs.push(partial_output);
    }

    /// Get the latest checkpoint for this job.
    pub fn latest_checkpoint(&self) -> Option<&JobCheckpoint> {
        self.checkpoints.iter().max_by_key(|c| c.timestamp)
    }

    /// Get the current progress based on the latest checkpoint.
    pub fn current_progress(&self) -> Option<f32> {
        self.latest_checkpoint().map(|c| c.progress_percent)
    }

    /// Get all partial outputs ordered by timestamp.
    pub fn ordered_partial_outputs(&self) -> Vec<&PartialOutputReceipt> {
        let mut outputs = self.partial_outputs.iter().collect::<Vec<_>>();
        outputs.sort_by_key(|o| o.timestamp);
        outputs
    }

    /// Get the current status based on what lifecycle events exist.
    pub fn current_status(&self) -> JobLifecycleStatus {
        if let Some(receipt) = &self.receipt {
            if receipt.success {
                JobLifecycleStatus::Completed
            } else {
                JobLifecycleStatus::Failed
            }
        } else if self.assignment.is_some() {
            // Check if we have recent checkpoint activity to determine if still executing
            if !self.checkpoints.is_empty() {
                JobLifecycleStatus::Executing
            } else {
                JobLifecycleStatus::Assigned
            }
        } else if !self.bids.is_empty() {
            JobLifecycleStatus::BiddingClosed // Could be more nuanced
        } else {
            self.job.status.clone()
        }
    }

    /// Check if this is a long-running job (has checkpoints or partial outputs).
    pub fn is_long_running(&self) -> bool {
        !self.checkpoints.is_empty() || !self.partial_outputs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did, NodeInfo, ICN_CORE_VERSION}; // Kept ICN_CORE_VERSION as it's often for tests
    use icn_economics::ManaLedger;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemoryLedger {
        balances: Mutex<HashMap<Did, u64>>,
    }

    impl InMemoryLedger {
        fn new() -> Self {
            Self {
                balances: Mutex::new(HashMap::new()),
            }
        }
    }

    impl icn_economics::ManaLedger for InMemoryLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.lock().unwrap().get(did).unwrap_or(&0)
        }

        fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
            self.balances.lock().unwrap().insert(did.clone(), amount);
            Ok(())
        }

        fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
            let mut map = self.balances.lock().unwrap();
            let bal = map
                .get_mut(did)
                .ok_or_else(|| icn_common::CommonError::DatabaseError("account".into()))?;
            if *bal < amount {
                return Err(icn_common::CommonError::PolicyDenied("insufficient".into()));
            }
            *bal -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
            let mut map = self.balances.lock().unwrap();
            let entry = map.entry(did.clone()).or_insert(0);
            *entry += amount;
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemoryLatencyStore {
        latencies: Mutex<HashMap<Did, u64>>,
    }

    impl InMemoryLatencyStore {
        fn new() -> Self {
            Self {
                latencies: Mutex::new(HashMap::new()),
            }
        }

        fn set_latency(&self, did: Did, latency: u64) {
            self.latencies.lock().unwrap().insert(did, latency);
        }
    }

    impl LatencyStore for InMemoryLatencyStore {
        fn get_latency(&self, did: &Did) -> Option<u64> {
            self.latencies.lock().unwrap().get(did).cloned()
        }
    }

    /// Test implementation of DynamicCapabilityChecker for unit tests.
    #[derive(Default)]
    struct TestCapabilityChecker {
        capabilities: Mutex<HashMap<Did, HashMap<String, bool>>>,
        resources: Mutex<HashMap<Did, Resources>>,
        availability: Mutex<HashMap<Did, bool>>,
    }

    impl TestCapabilityChecker {
        fn new() -> Self {
            Self {
                capabilities: Mutex::new(HashMap::new()),
                resources: Mutex::new(HashMap::new()),
                availability: Mutex::new(HashMap::new()),
            }
        }

        fn set_capability(&self, executor_did: &Did, capability: &str, has_capability: bool) {
            self.capabilities
                .lock()
                .unwrap()
                .entry(executor_did.clone())
                .or_insert_with(HashMap::new)
                .insert(capability.to_string(), has_capability);
        }

        fn set_resources(&self, executor_did: &Did, resources: Resources) {
            self.resources
                .lock()
                .unwrap()
                .insert(executor_did.clone(), resources);
        }

        fn set_availability(&self, executor_did: &Did, available: bool) {
            self.availability
                .lock()
                .unwrap()
                .insert(executor_did.clone(), available);
        }
    }

    impl DynamicCapabilityChecker for TestCapabilityChecker {
        fn check_capability(
            &self,
            executor_did: &Did,
            capability: &str,
            _context: Option<&str>,
        ) -> Option<bool> {
            self.capabilities
                .lock()
                .unwrap()
                .get(executor_did)
                .and_then(|caps| caps.get(capability))
                .copied()
        }

        fn get_current_resources(&self, executor_did: &Did) -> Option<Resources> {
            self.resources.lock().unwrap().get(executor_did).cloned()
        }

        fn is_executor_available(&self, executor_did: &Did) -> bool {
            self.availability
                .lock()
                .unwrap()
                .get(executor_did)
                .copied()
                .unwrap_or(true)
        }
    }

    #[test]
    fn test_schedule_mesh_job() {
        let node_info = NodeInfo {
            name: "MeshNode".to_string(),
            version: ICN_CORE_VERSION.to_string(), // Restored ICN_CORE_VERSION
            status_message: "Mesh active".to_string(),
        };
        let result = schedule_mesh_job(&node_info, "job-123");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("job-123"));
    }

    #[test]
    fn test_actual_mesh_job_signing_and_verification() {
        let (signing_key, verifying_key) = icn_identity::generate_ed25519_keypair();
        let creator_did_string = icn_identity::did_key_from_verifying_key(&verifying_key);
        let creator_did = Did::from_str(&creator_did_string).unwrap();

        let job_id = dummy_job_id("test_job_data_for_cid_signing"); // Use dummy_job_id helper
        let manifest_cid = dummy_cid("test_manifest_data_for_cid_signing"); // Use dummy_cid helper

        let job_unsigned = ActualMeshJob {
            id: job_id.clone(),
            manifest_cid: manifest_cid.clone(),
            spec: JobSpec::default(),
            creator_did: creator_did.clone(),
            cost_mana: 100,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]), // Placeholder
        };

        let signed_job = job_unsigned.clone().sign(&signing_key).unwrap();
        assert_ne!(signed_job.signature.0, Vec::<u8>::new());

        // Verification should pass with the correct public key
        assert!(signed_job.verify_signature(&verifying_key).is_ok());

        // Verification should fail with a different public key
        let (_other_sk, other_pk) = icn_identity::generate_ed25519_keypair();
        assert!(signed_job.verify_signature(&other_pk).is_err());

        // Verification should fail if the job data is tampered with
        let mut tampered_job = signed_job.clone();
        tampered_job.cost_mana = 200;
        assert!(tampered_job.verify_signature(&verifying_key).is_err());
    }

    #[test]
    fn test_mesh_bid_submit_signing_and_verification() {
        let (sk, vk) = icn_identity::generate_ed25519_keypair();
        let did = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk)).unwrap();

        let bid = MeshJobBid {
            job_id: dummy_job_id("bid_submit"),
            executor_did: did.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk)
        .unwrap();

        let submit = MeshBidSubmit {
            bid: bid.clone(),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk)
        .unwrap();

        assert!(submit.verify_signature(&vk).is_ok());

        let (_sk2, vk2) = icn_identity::generate_ed25519_keypair();
        assert!(submit.verify_signature(&vk2).is_err());

        let mut tampered = submit.clone();
        tampered.bid.price_mana = 50;
        assert!(tampered.verify_signature(&vk).is_err());
    }

    #[test]
    fn test_job_assignment_notice_signing_and_verification() {
        let (sk, vk) = icn_identity::generate_ed25519_keypair();

        let notice = JobAssignmentNotice {
            job_id: dummy_job_id("assign_notice"),
            executor_did: Did::from_str("did:icn:test:exec").unwrap(),
            signature: SignatureBytes(vec![]),
            manifest_cid: None,
        }
        .sign(&sk)
        .unwrap();

        assert!(notice.verify_signature(&vk).is_ok());

        let (_sk2, vk2) = icn_identity::generate_ed25519_keypair();
        assert!(notice.verify_signature(&vk2).is_err());

        let mut tampered = notice.clone();
        tampered.executor_did = Did::from_str("did:icn:test:other").unwrap();
        assert!(tampered.verify_signature(&vk).is_err());
    }

    // Helper to create a dummy Cid for tests
    fn dummy_cid(s: &str) -> Cid {
        Cid::new_v1_sha256(0x55, s.as_bytes())
    }

    // Helper to create a dummy JobId for tests
    fn dummy_job_id(s: &str) -> JobId {
        JobId::from(dummy_cid(s))
    }

    #[test]
    fn test_select_executor_prefers_reputation() {
        let job_id = dummy_job_id("job_sel");
        let (sk_high, vk_high) = icn_identity::generate_ed25519_keypair();
        let high = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_high)).unwrap();
        let (sk_low, vk_low) = icn_identity::generate_ed25519_keypair();
        let low = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_low)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(high.clone(), 5);
        rep_store.set_score(low.clone(), 1);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&high, 50).unwrap();
        ledger.set_balance(&low, 50).unwrap();

        let bid_high = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: high.clone(),
            price_mana: 15,
            resources: Resources {
                cpu_cores: 2,
                memory_mb: 1024,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_high)
        .unwrap();
        let bid_low = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: low.clone(),
            price_mana: 5,
            resources: Resources {
                cpu_cores: 1,
                memory_mb: 512,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_low)
        .unwrap();

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(high.clone(), 10);
        latency.set_latency(low.clone(), 20);
        let spec = JobSpec::default();
        let capability_checker = TestCapabilityChecker::new();
        let selected = select_executor(
            &job_id,
            &spec,
            vec![bid_high.clone(), bid_low.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
            &capability_checker,
        );

        assert_eq!(selected.unwrap(), high);
    }

    #[test]
    fn test_select_executor_uses_price_when_reputation_equal() {
        let job_id = dummy_job_id("job_price");
        let (sk_a, vk_a) = icn_identity::generate_ed25519_keypair();
        let a = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_a)).unwrap();
        let (sk_b, vk_b) = icn_identity::generate_ed25519_keypair();
        let b = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_b)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(a.clone(), 3);
        rep_store.set_score(b.clone(), 3);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&a, 50).unwrap();
        ledger.set_balance(&b, 50).unwrap();

        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: a.clone(),
            price_mana: 20,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_a)
        .unwrap();
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: b.clone(),
            price_mana: 5,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_b)
        .unwrap();

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(a.clone(), 15);
        latency.set_latency(b.clone(), 5);
        let spec = JobSpec::default();
        let capability_checker = TestCapabilityChecker::new();
        let selected = select_executor(
            &job_id,
            &spec,
            vec![bid_a, bid_b.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
            &capability_checker,
            &capability_checker,
        );

        assert_eq!(selected.unwrap(), b);
    }

    #[test]
    fn test_policy_price_weight_overrides_reputation() {
        let job_id = dummy_job_id("job_weight");
        let high_rep = Did::from_str("did:icn:test:highrep").unwrap();
        let cheap = Did::from_str("did:icn:test:cheap").unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(high_rep.clone(), 10);
        rep_store.set_score(cheap.clone(), 1);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&high_rep, 100).unwrap();
        ledger.set_balance(&cheap, 100).unwrap();

        let bid_high_rep = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: high_rep.clone(),
            price_mana: 50,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };
        let bid_cheap = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: cheap.clone(),
            price_mana: 5,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy {
            weight_price: 100.0, // Much higher weight to truly override reputation
            weight_reputation: 1.0,
            weight_resources: 1.0,
            weight_latency: 1.0,
        };

        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(high_rep.clone(), 20);
        latency.set_latency(cheap.clone(), 5);

        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_high_rep, bid_cheap.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        assert_eq!(selected.unwrap(), cheap);
    }

    #[test]
    fn test_bid_skipped_without_mana() {
        let job_id = dummy_job_id("job_mana");
        let a = Did::from_str("did:icn:test:mana_a").unwrap();
        let b = Did::from_str("did:icn:test:mana_b").unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(a.clone(), 5);
        rep_store.set_score(b.clone(), 3);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&a, 4).unwrap(); // less than price
        ledger.set_balance(&b, 50).unwrap();

        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: a.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(a.clone(), 50);
        latency.set_latency(b.clone(), 10);
        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_a, bid_b.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        assert_eq!(selected.unwrap(), b);
    }

    #[test]
    fn test_resource_weight_influences_selection() {
        let job_id = dummy_job_id("job_resource");
        let (sk_fast, vk_fast) = icn_identity::generate_ed25519_keypair();
        let fast = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_fast)).unwrap();
        let (sk_slow, vk_slow) = icn_identity::generate_ed25519_keypair();
        let slow = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_slow)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(fast.clone(), 1);
        rep_store.set_score(slow.clone(), 1);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&fast, 100).unwrap();
        ledger.set_balance(&slow, 100).unwrap();

        let bid_fast = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: fast.clone(),
            price_mana: 10,
            resources: Resources {
                cpu_cores: 4,
                memory_mb: 4096,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_fast)
        .unwrap();

        let bid_slow = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: slow.clone(),
            price_mana: 10,
            resources: Resources {
                cpu_cores: 1,
                memory_mb: 512,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_slow)
        .unwrap();

        let policy = SelectionPolicy {
            weight_price: 1.0,
            weight_reputation: 0.0,
            weight_resources: 10.0,
            weight_latency: 1.0,
        };

        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(fast.clone(), 5);
        latency.set_latency(slow.clone(), 50);

        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_slow.clone(), bid_fast.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        assert_eq!(selected.unwrap(), fast);
    }

    #[test]
    fn test_score_bid_zero_without_mana() {
        let job_id = dummy_job_id("job_score_mana");
        let bidder = Did::from_str("did:icn:test:score_mana").unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(bidder.clone(), 5);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&bidder, 0).unwrap();

        let bid = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: bidder.clone(),
            price_mana: 10,
            resources: Resources {
                cpu_cores: 2,
                memory_mb: 1024,
                storage_mb: 0,
            },
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(bidder.clone(), 15);
        let score = score_bid(
            &bid,
            &JobSpec::default(),
            &policy,
            &rep_store,
            ledger.get_balance(&bid.executor_did),
            latency.get_latency(&bid.executor_did),
            &capability_checker,
        );
        assert_eq!(score, 0);
    }

    #[test]
    fn test_select_executor_returns_none_with_no_bids() {
        let job_id = dummy_job_id("job_nobids");
        let rep_store = icn_reputation::InMemoryReputationStore::new();
        let ledger = InMemoryLedger::new();
        let policy = SelectionPolicy::default();

        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        assert!(selected.is_none());
    }

    #[test]
    fn test_select_executor_all_bids_insufficient_mana() {
        let job_id = dummy_job_id("job_no_mana");
        let did_a = Did::from_str("did:icn:test:no_mana_a").unwrap();
        let did_b = Did::from_str("did:icn:test:no_mana_b").unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(did_a.clone(), 5);
        rep_store.set_score(did_b.clone(), 5);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&did_a, 0).unwrap();
        ledger.set_balance(&did_b, 0).unwrap();

        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_a.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 15,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_a, bid_b],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        assert!(selected.is_none());
    }

    #[test]
    fn test_score_bid_respects_reputation() {
        let job_id = dummy_job_id("job_rep_score");
        let did_a = Did::from_str("did:icn:test:rep_a").unwrap();
        let did_b = Did::from_str("did:icn:test:rep_b").unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(did_a.clone(), 10);
        rep_store.set_score(did_b.clone(), 1);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&did_a, 100).unwrap();
        ledger.set_balance(&did_b, 100).unwrap();

        let bid_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_a.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();
        let capability_checker = TestCapabilityChecker::new();
        latency.set_latency(did_a.clone(), 5);
        latency.set_latency(did_b.clone(), 30);
        let score_a = score_bid(
            &bid_a,
            &JobSpec::default(),
            &policy,
            &rep_store,
            ledger.get_balance(&bid_a.executor_did),
            latency.get_latency(&bid_a.executor_did),
            &capability_checker,
        );
        let score_b = score_bid(
            &bid_b,
            &JobSpec::default(),
            &policy,
            &rep_store,
            ledger.get_balance(&bid_b.executor_did),
            latency.get_latency(&bid_b.executor_did),
            &capability_checker,
        );

        assert!(score_a > score_b);
    }

    #[test]
    fn test_submit_receipt_message_signing_and_verification() {
        let (sk, vk) = icn_identity::generate_ed25519_keypair();
        let receipt = icn_identity::ExecutionReceipt {
            job_id: dummy_cid("receipt"),
            executor_did: Did::from_str(&icn_identity::did_key_from_verifying_key(&vk)).unwrap(),
            result_cid: dummy_cid("result"),
            cpu_ms: 10,
            success: true,
            sig: SignatureBytes(vec![]),
        }
        .sign_with_key(&sk)
        .unwrap();

        let msg = SubmitReceiptMessage {
            receipt: receipt.clone(),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk)
        .unwrap();

        assert!(msg.verify_signature(&vk).is_ok());

        let (_sk2, vk2) = icn_identity::generate_ed25519_keypair();
        assert!(msg.verify_signature(&vk2).is_err());
    }

    #[test]
    fn test_federation_filtering_in_executor_selection() {
        let job_id = dummy_job_id("federation_test");
        let (sk_fed_a, vk_fed_a) = icn_identity::generate_ed25519_keypair();
        let executor_a =
            Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_fed_a)).unwrap();
        let (sk_fed_b, vk_fed_b) = icn_identity::generate_ed25519_keypair();
        let executor_b =
            Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_fed_b)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(executor_a.clone(), 5);
        rep_store.set_score(executor_b.clone(), 5); // Same reputation

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&executor_a, 100).unwrap();
        ledger.set_balance(&executor_b, 100).unwrap();

        // Bid from executor in federation A
        let bid_fed_a = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_a.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["federation_a".to_string()],
            executor_trust_scope: Some("trusted".to_string()),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_fed_a)
        .unwrap();

        // Bid from executor in federation B
        let bid_fed_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["federation_b".to_string()],
            executor_trust_scope: Some("trusted".to_string()),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_fed_b)
        .unwrap();

        // Job that only allows federation A
        let mut job_spec = JobSpec::default();
        job_spec.allowed_federations = vec!["federation_a".to_string()];

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();

        let selected = select_executor(
            &job_id,
            &job_spec,
            vec![bid_fed_a.clone(), bid_fed_b.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        // Should select executor from federation A only
        assert_eq!(selected.unwrap(), executor_a);
    }

    #[test]
    fn test_capability_filtering_in_executor_selection() {
        let job_id = dummy_job_id("capability_test");
        let (sk_a, vk_a) = icn_identity::generate_ed25519_keypair();
        let executor_a = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_a)).unwrap();
        let (sk_b, vk_b) = icn_identity::generate_ed25519_keypair();
        let executor_b = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_b)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(executor_a.clone(), 5);
        rep_store.set_score(executor_b.clone(), 5);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&executor_a, 100).unwrap();
        ledger.set_balance(&executor_b, 100).unwrap();

        // Executor A has GPU capability
        let bid_gpu = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_a.clone(),
            price_mana: 15,
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string(), "gpu".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_a)
        .unwrap();

        // Executor B only has CPU capability
        let bid_cpu = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_b.clone(),
            price_mana: 10, // Cheaper price
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_b)
        .unwrap();

        // Job that requires GPU capability
        let mut job_spec = JobSpec::default();
        job_spec.required_capabilities = vec!["gpu".to_string()];

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();

        let selected = select_executor(
            &job_id,
            &job_spec,
            vec![bid_gpu.clone(), bid_cpu.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        // Should select executor A despite higher price because it has required GPU capability
        assert_eq!(selected.unwrap(), executor_a);
    }

    #[test]
    fn test_reputation_filtering_in_executor_selection() {
        let job_id = dummy_job_id("reputation_test");
        let (sk_high, vk_high) = icn_identity::generate_ed25519_keypair();
        let high_rep = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_high)).unwrap();
        let (sk_low, vk_low) = icn_identity::generate_ed25519_keypair();
        let low_rep = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_low)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(high_rep.clone(), 50);
        rep_store.set_score(low_rep.clone(), 5); // Below minimum

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&high_rep, 100).unwrap();
        ledger.set_balance(&low_rep, 100).unwrap();

        let bid_high_rep = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: high_rep.clone(),
            price_mana: 20, // Higher price
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_high)
        .unwrap();

        let bid_low_rep = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: low_rep.clone(),
            price_mana: 5, // Much cheaper
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_low)
        .unwrap();

        // Job that requires minimum reputation of 10
        let mut job_spec = JobSpec::default();
        job_spec.min_executor_reputation = Some(10);

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();

        let selected = select_executor(
            &job_id,
            &job_spec,
            vec![bid_high_rep.clone(), bid_low_rep.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        // Should select high reputation executor despite higher price
        assert_eq!(selected.unwrap(), high_rep);
    }

    #[test]
    fn test_trust_scope_filtering() {
        let job_id = dummy_job_id("trust_scope_test");
        let (sk_a, vk_a) = icn_identity::generate_ed25519_keypair();
        let executor_a = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_a)).unwrap();
        let (sk_b, vk_b) = icn_identity::generate_ed25519_keypair();
        let executor_b = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk_b)).unwrap();

        let rep_store = icn_reputation::InMemoryReputationStore::new();
        rep_store.set_score(executor_a.clone(), 5);
        rep_store.set_score(executor_b.clone(), 5);

        let ledger = InMemoryLedger::new();
        ledger.set_balance(&executor_a, 100).unwrap();
        ledger.set_balance(&executor_b, 100).unwrap();

        // Executor A has high trust scope
        let bid_high_trust = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_a.clone(),
            price_mana: 15,
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: Some("high_security".to_string()),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_a)
        .unwrap();

        // Executor B has basic trust scope
        let bid_basic_trust = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: executor_b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            executor_capabilities: vec!["compute".to_string()],
            executor_federations: vec!["main_federation".to_string()],
            executor_trust_scope: Some("basic".to_string()),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_b)
        .unwrap();

        // Job that requires high security trust scope
        let mut job_spec = JobSpec::default();
        job_spec.required_trust_scope = Some("high_security".to_string());

        let policy = SelectionPolicy::default();
        let latency = InMemoryLatencyStore::new();
        let capability_checker = TestCapabilityChecker::new();

        let selected = select_executor(
            &job_id,
            &job_spec,
            vec![bid_high_trust.clone(), bid_basic_trust.clone()],
            &policy,
            &rep_store,
            &ledger,
            &latency,
            &capability_checker,
        );

        // Should select executor A because it matches required trust scope
        assert_eq!(selected.unwrap(), executor_a);
    }

    #[test]
    fn test_job_checkpoint_signing_and_verification() {
        let (sk, vk) = icn_identity::generate_ed25519_keypair();
        let executor_did = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk)).unwrap();

        let checkpoint = JobCheckpoint {
            job_id: dummy_job_id("checkpoint_test"),
            checkpoint_id: "checkpoint_001".to_string(),
            timestamp: 1000,
            stage: "processing".to_string(),
            progress_percent: 50.0,
            execution_state: vec![1, 2, 3, 4],
            intermediate_data_cid: Some(dummy_cid("intermediate")),
            executor_did: executor_did.clone(),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk)
        .unwrap();

        assert!(checkpoint.verify_signature(&vk).is_ok());

        let (_sk2, vk2) = icn_identity::generate_ed25519_keypair();
        assert!(checkpoint.verify_signature(&vk2).is_err());

        let mut tampered = checkpoint.clone();
        tampered.progress_percent = 75.0;
        assert!(tampered.verify_signature(&vk).is_err());
    }

    #[test]
    fn test_partial_output_receipt_signing_and_verification() {
        let (sk, vk) = icn_identity::generate_ed25519_keypair();
        let executor_did = Did::from_str(&icn_identity::did_key_from_verifying_key(&vk)).unwrap();

        let partial_output = PartialOutputReceipt {
            job_id: dummy_job_id("output_test"),
            output_id: "output_001".to_string(),
            stage: "stage1".to_string(),
            timestamp: 2000,
            output_cid: dummy_cid("output_data"),
            output_size: 1024,
            output_format: Some("application/json".to_string()),
            executor_did: executor_did.clone(),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk)
        .unwrap();

        assert!(partial_output.verify_signature(&vk).is_ok());

        let (_sk2, vk2) = icn_identity::generate_ed25519_keypair();
        assert!(partial_output.verify_signature(&vk2).is_err());

        let mut tampered = partial_output.clone();
        tampered.output_size = 2048;
        assert!(tampered.verify_signature(&vk).is_err());
    }

    #[test]
    fn test_job_lifecycle_with_checkpoints() {
        let job = Job {
            id: dummy_job_id("lifecycle_test"),
            manifest_cid: dummy_cid("manifest"),
            spec_bytes: vec![],
            spec_json: None,
            submitter_did: Did::from_str("did:key:submitter").unwrap(),
            cost_mana: 100,
            submitted_at: 1000,
            status: JobLifecycleStatus::Submitted,
            resource_requirements: Resources::default(),
        };

        let mut lifecycle = JobLifecycle::new(job);

        // Add a checkpoint
        let checkpoint = JobCheckpoint {
            job_id: dummy_job_id("lifecycle_test"),
            checkpoint_id: "cp1".to_string(),
            timestamp: 2000,
            stage: "processing".to_string(),
            progress_percent: 30.0,
            execution_state: vec![],
            intermediate_data_cid: None,
            executor_did: Did::from_str("did:key:executor").unwrap(),
            signature: SignatureBytes(vec![]),
        };

        lifecycle.add_checkpoint(checkpoint);

        // Add a partial output
        let partial_output = PartialOutputReceipt {
            job_id: dummy_job_id("lifecycle_test"),
            output_id: "out1".to_string(),
            stage: "stage1".to_string(),
            timestamp: 2500,
            output_cid: dummy_cid("output"),
            output_size: 512,
            output_format: None,
            executor_did: Did::from_str("did:key:executor").unwrap(),
            signature: SignatureBytes(vec![]),
        };

        lifecycle.add_partial_output(partial_output);

        assert!(lifecycle.is_long_running());
        assert_eq!(lifecycle.current_progress(), Some(30.0));
        assert_eq!(lifecycle.checkpoints.len(), 1);
        assert_eq!(lifecycle.partial_outputs.len(), 1);
        assert_eq!(lifecycle.latest_checkpoint().unwrap().checkpoint_id, "cp1");
    }

    #[test]
    fn test_progress_report_creation() {
        let progress = ProgressReport {
            job_id: dummy_job_id("progress_test"),
            current_stage: "data_processing".to_string(),
            progress_percent: 65.5,
            eta_seconds: Some(300),
            message: "Processing batch 3 of 5".to_string(),
            timestamp: 3000,
            executor_did: Did::from_str("did:key:executor").unwrap(),
            completed_stages: vec!["initialization".to_string(), "validation".to_string()],
            remaining_stages: vec!["finalization".to_string(), "cleanup".to_string()],
        };

        assert_eq!(progress.progress_percent, 65.5);
        assert_eq!(progress.eta_seconds, Some(300));
        assert_eq!(progress.completed_stages.len(), 2);
        assert_eq!(progress.remaining_stages.len(), 2);
    }
}
