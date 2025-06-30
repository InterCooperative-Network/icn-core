pub enum PolicyCheckResult {
    Allowed,
    Denied { reason: String },
}

use icn_common::{Did, NodeScope};

/// Operations that may be subject to scoped policy checks when writing to the DAG.
#[derive(Debug, Clone, Copy)]
pub enum DagPayloadOp {
    SubmitBlock,
    AnchorReceipt,
}

/// Trait for enforcing scoped policies on DAG operations.
pub trait ScopedPolicyEnforcer: Send + Sync {
    fn check_permission(
        &self,
        op: DagPayloadOp,
        actor: &Did,
        scope: Option<&NodeScope>,
    ) -> PolicyCheckResult;
}

use std::collections::{HashMap, HashSet};

/// In-memory implementation of [`ScopedPolicyEnforcer`] based on membership lists.
#[derive(Default)]
pub struct InMemoryPolicyEnforcer {
    submitters: HashSet<Did>,
    anchorers: HashSet<Did>,
    memberships: HashMap<NodeScope, HashSet<Did>>,
}

impl InMemoryPolicyEnforcer {
    /// Create a new enforcer with the given allowed members for submitting blocks
    /// and anchoring receipts.
    pub fn new(
        submitters: HashSet<Did>,
        anchorers: HashSet<Did>,
        memberships: HashMap<NodeScope, HashSet<Did>>,
    ) -> Self {
        Self {
            submitters,
            anchorers,
            memberships,
        }
    }
}

impl ScopedPolicyEnforcer for InMemoryPolicyEnforcer {
    fn check_permission(
        &self,
        op: DagPayloadOp,
        actor: &Did,
        scope: Option<&NodeScope>,
    ) -> PolicyCheckResult {
        match op {
            DagPayloadOp::SubmitBlock => {
                if self.submitters.contains(actor) {
                    if let Some(scope) = scope {
                        if self
                            .memberships
                            .get(scope)
                            .map(|m| m.contains(actor))
                            .unwrap_or(false)
                        {
                            PolicyCheckResult::Allowed
                        } else {
                            PolicyCheckResult::Denied {
                                reason: "actor not in scope".to_string(),
                            }
                        }
                    } else {
                        PolicyCheckResult::Allowed
                    }
                } else {
                    PolicyCheckResult::Denied {
                        reason: "actor not authorized to submit DAG blocks".to_string(),
                    }
                }
            }
            DagPayloadOp::AnchorReceipt => {
                if self.anchorers.contains(actor) {
                    if let Some(scope) = scope {
                        if self
                            .memberships
                            .get(scope)
                            .map(|m| m.contains(actor))
                            .unwrap_or(false)
                        {
                            PolicyCheckResult::Allowed
                        } else {
                            PolicyCheckResult::Denied {
                                reason: "actor not in scope".to_string(),
                            }
                        }
                    } else {
                        PolicyCheckResult::Allowed
                    }
                } else {
                    PolicyCheckResult::Denied {
                        reason: "actor not authorized to anchor receipts".to_string(),
                    }
                }
            }
        }
    }
}
