// crates/icn-runtime/src/error.rs

use thiserror::Error;
use icn_common::{Cid, Did};
use icn_network::MeshNetworkError; // Assuming MeshNetworkError is accessible
use crate::context::HostAbiError;

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

    #[error("Serialization failure: {0}")]
    Serialization(String),

    #[error("Job specification invalid for {job_id:?}: {reason}")]
    InvalidSpec { job_id: Option<Cid>, reason: String },

    #[error("Permission denied for job {job_id:?}: {reason}")]
    PermissionDenied { job_id: Cid, reason: String },

    #[error("Attempted to operate on job {job_id:?} in an invalid state ({current_state}) for the operation: {operation}")]
    InvalidJobState {
        job_id: Cid,
        current_state: String, // Consider using JobState directly if Display is implemented well
        operation: String,
    },

    #[error("Internal job state error: {0}")]
    Internal(String),

    #[error("Host ABI error occurred: {0}")]
    HostAbi(crate::context::HostAbiError),

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

impl From<HostAbiError> for MeshJobError {
    fn from(e: HostAbiError) -> Self {
        match e {
            HostAbiError::NetworkError(msg) => MeshJobError::Network(
                MeshNetworkError::ConnectionFailed {
                    peer_id: None,
                    cause: msg,
                },
            ),
            HostAbiError::Common(common_err) => MeshJobError::from(common_err),
            HostAbiError::InsufficientMana => {
                MeshJobError::Economic("Insufficient mana for host operation".to_string())
            }
            HostAbiError::AccountNotFound(did) => MeshJobError::UnknownJob {
                job_id_str: format!("Account (DID) not found for operation: {}", did),
            },
            HostAbiError::InvalidParameters(msg) => MeshJobError::InvalidSpec {
                job_id: None,
                reason: msg,
            },
            HostAbiError::JobSubmissionFailed(reason) => MeshJobError::ProcessingFailure {
                job_id: Cid::new_v1_dummy(0, 0, b"host_abi_failure"),
                reason: format!("Job submission failed via host ABI: {}", reason),
            },
            other => MeshJobError::HostAbi(other),
        }
    }
}
