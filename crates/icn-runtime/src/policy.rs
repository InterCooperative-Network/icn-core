use icn_common::{Did, DagBlock};
use async_trait::async_trait;

/// Type alias for data anchored in the DAG.
pub type DagPayload = DagBlock;

/// Represents the scope within which a policy check applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeScope {
    Node,
    Cooperative,
    Federation,
}

/// Errors that can occur during policy enforcement.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("actor not member of required scope {0:?}")]
    NotMemberOfScope(NodeScope),
    #[error("missing required attestation")]
    MissingAttestation,
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("additional approvals required")] 
    QuorumRequired,
}

/// Result type used by policy checks.
pub type PolicyResult = Result<(), PolicyError>;

/// Trait for enforcing scoped runtime policies.
///
/// Implementations validate whether `actor` is allowed to submit the
/// provided [`DagPayload`] to the DAG. The runtime calls this prior to
/// anchoring blocks such as execution receipts.
#[async_trait]
pub trait ScopedPolicyEnforcer: Send + Sync {
    /// Validate that `actor` may submit `payload`.
    async fn check_dag_submit(&self, payload: &DagPayload, actor: &Did) -> PolicyResult;
}

/// Trivial [`ScopedPolicyEnforcer`] that allows all operations.
pub struct AllowAllPolicyEnforcer;

#[async_trait]
impl ScopedPolicyEnforcer for AllowAllPolicyEnforcer {
    async fn check_dag_submit(&self, _payload: &DagPayload, _actor: &Did) -> PolicyResult {
        Ok(())
    }
}
