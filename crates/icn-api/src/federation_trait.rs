use async_trait::async_trait;
use icn_common::CommonError;
use serde::{Deserialize, Serialize};

/// Request payload for federation join/leave operations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FederationPeerRequest {
    /// Peer identifier string.
    pub peer: String,
}

/// API surface for federation management.
#[async_trait]
pub trait FederationApi {
    /// Join a federation by connecting to the given peer.
    async fn join_federation(&self, request: FederationPeerRequest) -> Result<(), CommonError>;
    /// Leave a federation, removing the given peer from the known set.
    async fn leave_federation(&self, request: FederationPeerRequest) -> Result<(), CommonError>;
}
