//! Error types for the ICN runtime context and host ABI.

use icn_common::{CommonError, Did};

/// Error type for Host ABI operations.
#[derive(Debug, thiserror::Error)]
pub enum HostAbiError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Insufficient mana")]
    InsufficientMana,
    #[error("Account not found: {0}")]
    AccountNotFound(Did),
    #[error("Job submission failed: {0}")]
    JobSubmissionFailed(String),
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("DAG operation failed: {0}")]
    DagOperationFailed(String),
    #[error("Signature error: {0}")]
    SignatureError(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("WASM execution error: {0}")]
    WasmExecutionError(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Invalid system API call: {0}")]
    InvalidSystemApiCall(String),
    #[error("Internal runtime error: {0}")]
    InternalError(String),
    #[error("Common error: {0}")]
    Common(CommonError),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Governance error: {0}")]
    GovernanceError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("DAG error: {0}")]
    DagError(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Operation timeout: {0}")]
    OperationTimeout(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Job not found: {0}")]
    JobNotFound(String),
    #[error("Invalid job state: {0}")]
    InvalidJobState(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<CommonError> for HostAbiError {
    fn from(err: CommonError) -> Self {
        HostAbiError::Common(err)
    }
}

impl From<HostAbiError> for CommonError {
    fn from(err: HostAbiError) -> Self {
        match err {
            HostAbiError::Common(common_err) => common_err,
            other => CommonError::InternalError(other.to_string()),
        }
    }
}
