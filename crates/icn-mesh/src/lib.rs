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

/// Errors that can occur within the ICN Mesh subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshError {
    /// Operation failed due to insufficient mana.
    InsufficientMana(String),
    /// Executor's reputation is too low for the operation.
    ReputationTooLow(String),
    /// No suitable executor could be found for the job.
    NoSuitableExecutor(String),
    /// The provided bid was invalid.
    InvalidBid(String),
    /// The job specification was invalid.
    InvalidJobSpec(String),
    /// A bid was submitted more than once for the same executor and job.
    DuplicateBid(String),
    /// A network operation failed.
    NetworkFailure(String),
    /// An internal error occurred.
    InternalError(String),
}

// Optional: Implement std::error::Error and std::fmt::Display for MeshError
impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::InsufficientMana(msg) => write!(f, "Insufficient mana: {}", msg),
            MeshError::ReputationTooLow(msg) => write!(f, "Reputation too low: {}", msg),
            MeshError::NoSuitableExecutor(msg) => write!(f, "No suitable executor: {}", msg),
            MeshError::InvalidBid(msg) => write!(f, "Invalid bid: {}", msg),
            MeshError::InvalidJobSpec(msg) => write!(f, "Invalid job spec: {}", msg),
            MeshError::DuplicateBid(msg) => write!(f, "Duplicate bid: {}", msg),
            MeshError::NetworkFailure(msg) => write!(f, "Network failure: {}", msg),
            MeshError::InternalError(msg) => write!(f, "Internal mesh error: {}", msg),
        }
    }
}

impl std::error::Error for MeshError {}

// Define JobId and Resources if they are not already defined elsewhere
// For now, let's use a simple type alias or placeholder
pub type JobId = Cid;
/// Execution resource capabilities offered in a bid.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resources {
    /// Number of CPU cores available for the job.
    pub cpu_cores: u32,
    /// Amount of memory in megabytes available for the job.
    pub memory_mb: u32,
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
    /// The runtime will load the module from the DAG store and run its `run` function.
    CclWasm,
    /// Placeholder until more kinds are defined.
    #[default]
    GenericPlaceholder,
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
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            kind: JobKind::GenericPlaceholder,
            inputs: Vec::new(),
            outputs: Vec::new(),
            required_resources: Resources::default(),
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
}

impl Default for SelectionPolicy {
    fn default() -> Self {
        Self {
            weight_price: 1.0,
            weight_reputation: 50.0,
            weight_resources: 1.0,
        }
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
    ) -> Option<Did> {
        // This helper is retained for future stateful selection logic.
        bids.iter()
            .max_by_key(|bid| score_bid(bid, job_spec, policy, reputation_store, mana_ledger))
            .map(|bid| bid.executor_did.clone())
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
) -> Option<Did> {
    // Iterate over bids and pick the executor with the highest score as
    // determined by `score_bid`. Bids from executors without enough mana are
    // ignored.
    println!(
        "[Mesh] Selecting executor for job {:?}. Received {} bids.",
        job_id,
        bids.len()
    );

    bids.iter()
        .filter(|bid| mana_ledger.get_balance(&bid.executor_did) >= bid.price_mana)
        .max_by_key(|bid| score_bid(bid, job_spec, policy, reputation_store, mana_ledger))
        .map(|bid| bid.executor_did.clone())
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
///
/// # Returns
/// * A `u64` representing the calculated score for the bid. Higher is generally better.
pub fn score_bid(
    bid: &MeshJobBid,
    job_spec: &JobSpec,
    policy: &SelectionPolicy,
    reputation_store: &dyn icn_reputation::ReputationStore,
    mana_ledger: &dyn icn_economics::ManaLedger,
) -> u64 {
    if mana_ledger.get_balance(&bid.executor_did) < bid.price_mana {
        return 0;
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
    let req = &job_spec.required_resources;
    let resource_match =
        if bid.resources.cpu_cores >= req.cpu_cores && bid.resources.memory_mb >= req.memory_mb {
            let cpu_ratio = bid.resources.cpu_cores as f64 / req.cpu_cores.max(1) as f64;
            let mem_ratio = bid.resources.memory_mb as f64 / req.memory_mb.max(1) as f64;
            (cpu_ratio + mem_ratio) / 2.0
        } else {
            // Insufficient resources yields zero score
            0.0
        };

    let resource_score = policy.weight_resources * resource_match;

    let weighted = price_score + reputation_score + resource_score;

    weighted.max(0.0) as u64
}

/// Placeholder function demonstrating use of common types for mesh operations.
pub fn schedule_mesh_job(info: &NodeInfo, job_id: &str) -> Result<String, CommonError> {
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

    pub fn verify_signature(&self, verifying_key: &IdentityVerifyingKey) -> Result<(), CommonError> {
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

        fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_economics::EconError> {
            let mut map = self.balances.lock().unwrap();
            let bal = map
                .get_mut(did)
                .ok_or_else(|| icn_economics::EconError::AdapterError("account".into()))?;
            if *bal < amount {
                return Err(icn_economics::EconError::InsufficientBalance(
                    "insufficient".into(),
                ));
            }
            *bal -= amount;
            Ok(())
        }

        fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_economics::EconError> {
            let mut map = self.balances.lock().unwrap();
            let entry = map.entry(did.clone()).or_insert(0);
            *entry += amount;
            Ok(())
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

        let job_id = dummy_cid("test_job_data_for_cid_signing"); // Use dummy_cid helper
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
            job_id: dummy_cid("bid_submit"),
            executor_did: did.clone(),
            price_mana: 10,
            resources: Resources::default(),
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
            job_id: dummy_cid("assign_notice"),
            executor_did: Did::from_str("did:icn:test:exec").unwrap(),
            signature: SignatureBytes(vec![]),
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

    #[test]
    fn test_select_executor_prefers_reputation() {
        let job_id = dummy_cid("job_sel");
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
            },
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
            },
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_low)
        .unwrap();

        let policy = SelectionPolicy::default();
        let spec = JobSpec::default();
        let selected = select_executor(
            &job_id,
            &spec,
            vec![bid_high.clone(), bid_low.clone()],
            &policy,
            &rep_store,
            &ledger,
        );

        assert_eq!(selected.unwrap(), high);
    }

    #[test]
    fn test_select_executor_uses_price_when_reputation_equal() {
        let job_id = dummy_cid("job_price");
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
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_a)
        .unwrap();
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: b.clone(),
            price_mana: 5,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_b)
        .unwrap();

        let policy = SelectionPolicy::default();
        let spec = JobSpec::default();
        let selected = select_executor(
            &job_id,
            &spec,
            vec![bid_a, bid_b.clone()],
            &policy,
            &rep_store,
            &ledger,
        );

        assert_eq!(selected.unwrap(), b);
    }

    #[test]
    fn test_policy_price_weight_overrides_reputation() {
        let job_id = dummy_cid("job_weight");
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
            signature: SignatureBytes(vec![]),
        };
        let bid_cheap = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: cheap.clone(),
            price_mana: 5,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy {
            weight_price: 10.0,
            weight_reputation: 1.0,
            weight_resources: 1.0,
        };

        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_high_rep, bid_cheap.clone()],
            &policy,
            &rep_store,
            &ledger,
        );

        assert_eq!(selected.unwrap(), cheap);
    }

    #[test]
    fn test_bid_skipped_without_mana() {
        let job_id = dummy_cid("job_mana");
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
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_a, bid_b.clone()],
            &policy,
            &rep_store,
            &ledger,
        );

        assert_eq!(selected.unwrap(), b);
    }

    #[test]
    fn test_resource_weight_influences_selection() {
        let job_id = dummy_cid("job_resource");
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
            },
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
            },
            signature: SignatureBytes(vec![]),
        }
        .sign(&sk_slow)
        .unwrap();

        let policy = SelectionPolicy {
            weight_price: 1.0,
            weight_reputation: 0.0,
            weight_resources: 10.0,
        };

        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_slow.clone(), bid_fast.clone()],
            &policy,
            &rep_store,
            &ledger,
        );

        assert_eq!(selected.unwrap(), fast);
    }

    #[test]
    fn test_score_bid_zero_without_mana() {
        let job_id = dummy_cid("job_score_mana");
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
            },
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let score = score_bid(&bid, &JobSpec::default(), &policy, &rep_store, &ledger);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_select_executor_returns_none_with_no_bids() {
        let job_id = dummy_cid("job_nobids");
        let rep_store = icn_reputation::InMemoryReputationStore::new();
        let ledger = InMemoryLedger::new();
        let policy = SelectionPolicy::default();

        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![],
            &policy,
            &rep_store,
            &ledger,
        );

        assert!(selected.is_none());
    }

    #[test]
    fn test_select_executor_all_bids_insufficient_mana() {
        let job_id = dummy_cid("job_no_mana");
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
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 15,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let selected = select_executor(
            &job_id,
            &JobSpec::default(),
            vec![bid_a, bid_b],
            &policy,
            &rep_store,
            &ledger,
        );

        assert!(selected.is_none());
    }

    #[test]
    fn test_score_bid_respects_reputation() {
        let job_id = dummy_cid("job_rep_score");
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
            signature: SignatureBytes(vec![]),
        };
        let bid_b = MeshJobBid {
            job_id: job_id.clone(),
            executor_did: did_b.clone(),
            price_mana: 10,
            resources: Resources::default(),
            signature: SignatureBytes(vec![]),
        };

        let policy = SelectionPolicy::default();
        let score_a = score_bid(&bid_a, &JobSpec::default(), &policy, &rep_store, &ledger);
        let score_b = score_bid(&bid_b, &JobSpec::default(), &policy, &rep_store, &ledger);

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
    fn test_mesh_error_display() {
        let dup = MeshError::DuplicateBid("dup".into()).to_string();
        assert!(dup.contains("Duplicate bid"));

        let net = MeshError::NetworkFailure("net".into()).to_string();
        assert!(net.contains("Network failure"));
    }
}
