pub enum PolicyCheckResult {
    Allowed,
    Denied { reason: String },
}

use icn_common::Did;

/// Operations that may be subject to scoped policy checks when writing to the DAG.
#[derive(Debug, Clone, Copy)]
pub enum DagPayloadOp {
    SubmitBlock,
    AnchorReceipt,
}

/// Trait for enforcing scoped policies on DAG operations.
pub trait ScopedPolicyEnforcer: Send + Sync {
    fn check_permission(&self, op: DagPayloadOp, actor: &Did) -> PolicyCheckResult;
}

use std::collections::HashSet;

/// In-memory implementation of [`ScopedPolicyEnforcer`] based on membership lists.
#[derive(Default)]
pub struct InMemoryPolicyEnforcer {
    submitters: HashSet<Did>,
    anchorers: HashSet<Did>,
}

impl InMemoryPolicyEnforcer {
    /// Create a new enforcer with the given allowed members for submitting blocks
    /// and anchoring receipts.
    pub fn new(submitters: HashSet<Did>, anchorers: HashSet<Did>) -> Self {
        Self {
            submitters,
            anchorers,
        }
    }
}

impl ScopedPolicyEnforcer for InMemoryPolicyEnforcer {
    fn check_permission(&self, op: DagPayloadOp, actor: &Did) -> PolicyCheckResult {
        match op {
            DagPayloadOp::SubmitBlock => {
                if self.submitters.contains(actor) {
                    PolicyCheckResult::Allowed
                } else {
                    PolicyCheckResult::Denied {
                        reason: "actor not authorized to submit DAG blocks".to_string(),
                    }
                }
            }
            DagPayloadOp::AnchorReceipt => {
                if self.anchorers.contains(actor) {
                    PolicyCheckResult::Allowed
                } else {
                    PolicyCheckResult::Denied {
                        reason: "actor not authorized to anchor receipts".to_string(),
                    }
                }
            }
        }
    }
}
