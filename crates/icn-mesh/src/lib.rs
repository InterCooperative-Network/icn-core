#![doc = include_str!("../README.md")]

//! # ICN Mesh Crate
//! This crate focuses on job orchestration, scheduling, and execution within the
//! InterCooperative Network (ICN) mesh network. It handles job definition, resource discovery,
//! scheduling, execution management, and fault tolerance.

use icn_common::{NodeInfo, CommonError, Cid, Did};
use icn_identity::{ExecutionReceipt, SignatureBytes, VerifyingKey as IdentityVerifyingKey, SigningKey as IdentitySigningKey, sign_message as identity_sign_message, verify_signature as identity_verify_signature};
use serde::{Serialize, Deserialize};

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
    /// An internal error occurred.
    InternalError(String),
    // TODO: Add other specific error variants as needed.
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
            MeshError::InternalError(msg) => write!(f, "Internal mesh error: {}", msg),
        }
    }
}

impl std::error::Error for MeshError {}

// Define JobId and Resources if they are not already defined elsewhere
// For now, let's use a simple type alias or placeholder
pub type JobId = Cid;
// Placeholder for Resources struct, to be defined based on requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources; 

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
        if self.id.to_string().is_empty() || self.id.to_string().as_bytes().len() < 4 { // Basic check, using to_string().as_bytes()
             return Err(CommonError::InternalError("Job ID must be set before signing".to_string())); // Was InvalidParameters
        }
        let message = self.to_signable_bytes()?;
        let ed_signature = identity_sign_message(signing_key, &message);
        self.signature = SignatureBytes(ed_signature.to_bytes().to_vec());
        Ok(self)
    }

    /// Verifies the signature of this job against the provided Ed25519 VerifyingKey.
    pub fn verify_signature(&self, verifying_key: &IdentityVerifyingKey) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_signature = self.signature.to_ed_signature()?;

        if identity_verify_signature(verifying_key, &message, &ed_signature) {
            Ok(())
        } else {
            Err(CommonError::InternalError("ActualMeshJob signature verification failed".to_string())) // Was CryptographyError
        }
    }
}

/// Detailed specification for a mesh job.
/// TODO: Define fields for inputs, outputs, resource requirements, timeouts, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum JobSpec {
    Echo { payload: String },
    // Add other variants as needed, e.g.:
    // Generic { command: String, args: Vec<String> },
    // Wasm { module_cid: Cid, entry_function: String, params: Vec<Value> },
    #[default]
    GenericPlaceholder, // Placeholder until more types are defined
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

// Placeholder for SelectionPolicy enum/struct
/// Defines the policy used for selecting an executor from a set of bids.
/// TODO: Define variants or fields (e.g., prioritize_reputation, prioritize_cost, weighted_score).
#[derive(Debug, Clone, Default)] // Added Default for placeholder
pub struct SelectionPolicy;

// Placeholder for ReputationExecutorSelector struct
pub struct ReputationExecutorSelector;

impl ReputationExecutorSelector {
    // This struct might not be strictly needed if select_executor handles scoring directly,
    // or it could encapsulate more complex stateful selection logic in the future.
    // For now, its select method can be a simple helper or be removed if select_executor is self-contained.
    pub fn select(&self, bids: &[MeshJobBid], policy: &SelectionPolicy) -> Option<Did> {
        // Updated to use scoring
        bids.iter()
            .max_by_key(|bid| score_bid(bid, policy))
            .map(|bid| bid.executor_did.clone())
    }
}

/// Selects the best executor from a list of bids based on a given policy.
/// 
/// This function typically utilizes a `ReputationExecutorSelector` internally to factor in
/// executor reputation alongside other bid parameters like price and resource availability.
/// 
/// # Arguments
/// * `job_id` - The ID of the job for which an executor is being selected.
/// * `bids` - A vector of `Bid` structs received for a specific job.
/// * `policy` - The `SelectionPolicy` to apply for choosing the best executor.
/// 
/// # Returns
/// * `Some(Did)` of the selected executor if a suitable one is found.
/// * `None` if no suitable executor could be selected based on the bids and policy.
pub fn select_executor(job_id: &JobId, bids: Vec<MeshJobBid>, policy: &SelectionPolicy) -> Option<Did> {
    println!("[Mesh] Selecting executor for job {:?}. Received {} bids.", job_id, bids.len());

    if bids.is_empty() {
        println!("[Mesh] No bids available for job {:?}", job_id);
        return None;
    }

    // Find the bid with the highest score
    let best_bid = bids.iter()
        .max_by_key(|bid| {
            let score = score_bid(bid, policy);
            println!("[Mesh] Bid from {:?} scored {} (price: {} mana)", 
                     bid.executor_did, score, bid.price_mana);
            score
        });

    match best_bid {
        Some(bid) => {
            println!("[Mesh] Selected executor {:?} for job {:?} (price: {} mana)", 
                     bid.executor_did, job_id, bid.price_mana);
            Some(bid.executor_did.clone())
        }
        None => {
            println!("[Mesh] No suitable executor found for job {:?}", job_id);
            None
        }
    }
}

/// Scores a single bid based on a selection policy.
/// 
/// The score typically reflects a combination of factors such as the bid price,
/// the executor's available mana (as a proxy for stability/commitment),
/// and the executor's reputation.
/// 
/// # Arguments
/// * `bid` - The `Bid` to score.
/// * `policy` - The `SelectionPolicy` to use for calculating the score.
/// 
/// # Returns
/// * A `u64` representing the calculated score for the bid. Higher is generally better.
pub fn score_bid(bid: &MeshJobBid, _policy: &SelectionPolicy) -> u64 {
    // Improved scoring logic that prioritizes lower prices with reasonable bounds
    
    // Price score: Lower prices get higher scores
    // Use 1000 / price to give good scores for reasonable prices (10-100 mana)
    let price_score = if bid.price_mana > 0 {
        1000 / bid.price_mana
    } else {
        0 // Zero price bids get zero score (suspicious)
    };

    // Add a small deterministic factor based on executor DID to break ties consistently
    // This ensures deterministic selection in test scenarios
    let tie_breaker = bid.executor_did.to_string().len() as u64;
    
    // For now, just use price score + tie breaker
    // TODO: Add reputation and resource scoring when those systems are implemented
    price_score * 100 + tie_breaker
}

/// Placeholder function demonstrating use of common types for mesh operations.
pub fn schedule_mesh_job(info: &NodeInfo, job_id: &str) -> Result<String, CommonError> {
    Ok(format!("Scheduled mesh job {} on node: {} (v{})", job_id, info.name, info.version))
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
    // TODO: Potentially add a summary of JobSpec or key requirements here
    // to allow executors to filter announcements without fetching the full manifest.
}

/// Message sent by an executor to the job's originating node to submit a bid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBidSubmit {
    /// The bid being submitted.
    pub bid: MeshJobBid,
    // TODO: Potentially add a signature from the executor over the bid fields
    // to ensure authenticity, though the bid itself contains the executor_did.
}

/// Message broadcast by the Job Manager to announce the selected executor for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignmentNotice {
    /// The ID of the job that has been assigned.
    pub job_id: JobId,
    /// The DID of the executor that has been assigned the job.
    pub executor_did: Did,
    // TODO: Potentially include the original job details or manifest_cid for executor convenience?
}

/// Message sent by an Executor to the Job Manager to submit an ExecutionReceipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReceiptMessage {
    /// The execution receipt being submitted.
    pub receipt: icn_identity::ExecutionReceipt,
    // TODO: Consider if this message itself needs a signature from the executor,
    // though the receipt inside is already signed by the executor.
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{ICN_CORE_VERSION, NodeInfo, Cid, Did}; // Kept ICN_CORE_VERSION as it's often for tests
    use std::str::FromStr;

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
            spec: JobSpec::GenericPlaceholder,
            creator_did: creator_did.clone(),
            cost_mana: 100,
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

    // Helper to create a dummy Cid for tests
    fn dummy_cid(s: &str) -> Cid {
        Cid::new_v1_dummy(0x55, 0x12, s.as_bytes())
    }
}
