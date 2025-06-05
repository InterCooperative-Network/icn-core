// crates/icn-network/src/error.rs

use thiserror::Error;

/// Errors that can occur during mesh network operations within the `icn-network` crate.
#[derive(Debug, Error)]
pub enum MeshNetworkError {
    #[error("Message serialization/deserialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Failed to send message: {0}")]
    SendFailure(String),

    #[error("Libp2p specific error: {0}")]
    Libp2p(String),

    #[error("Network setup or configuration error: {0}")]
    SetupError(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Invalid input or parameters: {0}")]
    InvalidInput(String),

    #[error("Underlying common error: {0}")]
    Common(#[from] icn_common::CommonError),

    #[error("Kademlia operation failed: {query_id:?}, reason: {reason}")]
    KadOperationFailed {
        query_id: Option<String>, // Changed from libp2p::kad::QueryId to String for broader use
        reason: String,
    },

    // TODO [error_handling]: Add more specific error variants as needed
} 