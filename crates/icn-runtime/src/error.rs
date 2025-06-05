// crates/icn-runtime/src/error.rs

use thiserror::Error;
use icn_common::{Cid, Did};
use icn_network::MeshNetworkError; // Assuming MeshNetworkError is accessible
 // Add this import

/// Errors that can occur during mesh job processing within the `icn-runtime` crate.
#[derive(Debug, Error)]
pub enum MeshJobError {
    #[error("Network error encountered during mesh job operation: {0}")]
    Network(#[from] MeshNetworkError),

    #[error("Failed to select a suitable executor for job {job_id:?}")]
    NoSuitableExecutor { job_id: Option<Cid> },

    #[error("Execution receipt missing, invalid, or does not match for job {job_id:?}")]
    MissingOrInvalidReceipt { job_id: Option<Cid> },

    #[error("Job not found or state unknown: {job_id_str}")]
    UnknownJob { job_id_str: String },

    #[error("Job {job_id:?} execution timed out by executor {executor_did:?}")]
    ExecutionTimeout {
        job_id: Cid,
        executor_did: Did,
    },

    #[error("Job {job_id:?} failed during processing: {reason}")]
    ProcessingFailure {
        job_id: Cid,
        reason: String,
    },

    #[error("Attempted to operate on job {job_id:?} in an invalid state ({current_state}) for the operation: {operation}")]
    InvalidJobState {
        job_id: Cid,
        current_state: String, // Consider using JobState directly if Display is implemented well
        operation: String,
    },

    #[error("Internal job state error: {0}")]
    Internal(String),

    #[error("Host ABI error occurred: {0}")]
    HostAbi(#[from] crate::context::HostAbiError), // Assuming HostAbiError is in context module

    #[error("Economic error related to mana or payments: {0}")]
    Economic(String), // Could also be `#[from] EconError` if that's defined

    // TODO [error_handling]: Add more specific error variants as needed
}

// Optional: If you want to convert from icn_common::CommonError directly
impl From<icn_common::CommonError> for MeshJobError {
    fn from(err: icn_common::CommonError) -> Self {
        MeshJobError::Internal(format!("Underlying common error: {}", err))
    }
}

// Remove this conflicting implementation
/*
impl From<HostAbiError> for MeshJobError {
    fn from(e: HostAbiError) -> Self {
        match e {
            HostAbiError::NetworkError(msg) => MeshJobError::Network(MeshNetworkError::SendFailure(msg)),
            HostAbiError::Common(common_err) => MeshJobError::from(common_err), // Utilize existing From<CommonError>
            // Map other HostAbiError variants to appropriate MeshJobError variants
            // For example, InsufficientMana could map to Economic or a new variant
            HostAbiError::InsufficientMana => MeshJobError::Economic("Insufficient mana for host operation".to_string()),
            HostAbiError::AccountNotFound(did) => MeshJobError::UnknownJob { job_id_str: format!("Account (DID) not found for operation: {}", did) }, // Or a more general UserError?
            HostAbiError::InvalidParameters(msg) => MeshJobError::Internal(format!("Invalid parameters in host ABI call: {}", msg)), // Or a specific variant if it makes sense
            HostAbiError::JobSubmissionFailed(reason) => MeshJobError::ProcessingFailure{ job_id: Cid::default(), reason: format!("Job submission failed via host ABI: {}", reason) }, // job_id might be unavailable here
            // Add more mappings as needed
            _ => MeshJobError::HostAbi(e), // Fallback to wrapping the HostAbiError directly
        }
    }
}
*/ 