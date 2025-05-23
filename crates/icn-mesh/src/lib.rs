#![doc = include_str!("../README.md")]

//! # ICN Mesh Crate
//! This crate focuses on job orchestration, scheduling, and execution within the
//! InterCooperative Network (ICN) mesh network. It handles job definition, resource discovery,
//! scheduling, execution management, and fault tolerance.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION, Cid, Did};
use icn_identity::ExecutionReceipt;
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
    /// Unique identifier for this job instance.
    pub id: JobId,
    /// Content Identifier (CID) of the job's core executable or primary data package.
    pub manifest_cid: Cid,
    /// Detailed specification of the job, including inputs, outputs, and execution requirements.
    pub spec: JobSpec, 
    /// Decentralized Identifier (DID) of the entity that submitted the job.
    pub creator_did: Did,
    /// The amount of mana allocated by the submitter for this job's execution.
    pub cost_mana: u64,
}

/// Detailed specification for a mesh job.
/// TODO: Define fields for inputs, outputs, resource requirements, timeouts, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobSpec {}

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
/// * `bids` - A vector of `Bid` structs received for a specific job.
/// * `policy` - The `SelectionPolicy` to apply for choosing the best executor.
/// 
/// # Returns
/// * `Some(Did)` of the selected executor if a suitable one is found.
/// * `None` if no suitable executor could be selected based on the bids and policy.
pub fn select_executor(bids: Vec<MeshJobBid>, policy: SelectionPolicy) -> Option<Did> {
    // Use the scoring logic directly or via ReputationExecutorSelector
    // let selector = ReputationExecutorSelector; 
    // selector.select(&bids, &policy)

    // Direct implementation for clarity for now:
    if bids.is_empty() {
        return None;
    }

    let mut best_bid: Option<&MeshJobBid> = None;
    let mut highest_score = 0u64;

    for bid in &bids {
        let current_score = score_bid(bid, &policy);
        if best_bid.is_none() || current_score > highest_score {
            highest_score = current_score;
            best_bid = Some(bid);
        }
    }

    best_bid.map(|b| b.executor_did.clone())
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
pub fn score_bid(bid: &MeshJobBid, policy: &SelectionPolicy) -> u64 {
    // TODO: Implement actual scoring logic based on price, mana, reputation, advertised_perf
    // TODO: Weights (w_price, w_rep, w_perf) should come from policy or config.
    // TODO: Reputation needs to be fetched for bid.executor_did.
    // TODO: Advertised performance should be part of the bid (e.g., in bid.resources).

    let w_price = 1.0; // Placeholder weight
    let w_rep = 0.0;   // Placeholder weight, disabling reputation for now
    let w_perf = 0.0;  // Placeholder weight, disabling performance for now

    // Price score: higher for lower price. Avoid division by zero.
    let price_score = if bid.price_mana > 0 {
        (1.0 / bid.price_mana as f64) * 1000.0 // Scale factor to make it a reasonable integer part
    } else {
        0.0 // Or a very low score if 0 price is disallowed/undesirable
    };

    let reputation_score = 0.0; // Placeholder
    let performance_score = 0.0; // Placeholder

    let total_score = w_price * price_score 
                    + w_rep * reputation_score 
                    + w_perf * performance_score;

    // Return as u64. Ensure it doesn't overflow or underflow if scores can be negative.
    // For now, assuming positive scores and simple truncation.
    total_score.max(0.0) as u64
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

    #[test]
    fn test_schedule_mesh_job() {
        let node_info = NodeInfo {
            name: "MeshNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Mesh active".to_string(),
        };
        let result = schedule_mesh_job(&node_info, "job-123");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("job-123"));
    }
}
