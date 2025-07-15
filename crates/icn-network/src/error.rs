// crates/icn-network/src/error.rs

use thiserror::Error;
use bincode::Error as BincodeError;

/// Errors that can occur during mesh network operations within the `icn-network` crate.
#[derive(Debug, Error)]
pub enum MeshNetworkError {
    #[error("Message serialization/deserialization failed: {0}")]
    Serialization(#[from] BincodeError),

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

    #[error("Connection to peer {peer_id:?} failed: {cause}")]
    ConnectionFailed {
        peer_id: Option<crate::PeerId>,
        cause: String,
    },

    #[error("Network channel unexpectedly closed")]
    ChannelClosed,

    #[error("Unexpected message type: {msg_type}")]
    UnexpectedMessage { msg_type: String },

    #[error("Invalid input or parameters: {0}")]
    InvalidInput(String),

    #[error("Underlying common error: {0}")]
    Common(#[from] icn_common::CommonError),

    #[error("Kademlia operation failed: {query_id:?}, reason: {reason}")]
    KadOperationFailed {
        query_id: Option<String>, // Changed from libp2p::kad::QueryId to String for broader use
        reason: String,
    },

    /// Error occurred during the libp2p noise handshake phase
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    /// Failure decoding a received network message
    #[error("Message decode failed: {0}")]
    MessageDecodeFailed(String),
}
