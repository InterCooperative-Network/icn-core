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
    submitters: HashMap<Option<NodeScope>, HashSet<Did>>,
    anchorers: HashMap<Option<NodeScope>, HashSet<Did>>,
}

impl InMemoryPolicyEnforcer {
    /// Create a new enforcer with the given allowed members for submitting blocks
    /// and anchoring receipts.
    pub fn new(submitters: HashSet<Did>, anchorers: HashSet<Did>) -> Self {
        let mut s_map = HashMap::new();
        s_map.insert(None, submitters);
        let mut a_map = HashMap::new();
        a_map.insert(None, anchorers);
        Self {
            submitters: s_map,
            anchorers: a_map,
        }
    }

    /// Grant submit permission for a DID within an optional scope.
    pub fn allow_submitter(&mut self, scope: Option<NodeScope>, did: Did) {
        self.submitters.entry(scope).or_default().insert(did);
    }

    /// Grant anchor permission for a DID within an optional scope.
    pub fn allow_anchorer(&mut self, scope: Option<NodeScope>, did: Did) {
        self.anchorers.entry(scope).or_default().insert(did);
    }
}

impl ScopedPolicyEnforcer for InMemoryPolicyEnforcer {
    fn check_permission(
        &self,
        op: DagPayloadOp,
        actor: &Did,
        scope: Option<&NodeScope>,
    ) -> PolicyCheckResult {
        let scope_key = scope.cloned();
        match op {
            DagPayloadOp::SubmitBlock => {
                let allowed = self
                    .submitters
                    .get(&scope_key)
                    .or_else(|| self.submitters.get(&None))
                    .map(|set| set.contains(actor))
                    .unwrap_or(false);
                if allowed {
                    PolicyCheckResult::Allowed
                } else {
                    PolicyCheckResult::Denied {
                        reason: "actor not authorized to submit DAG blocks".to_string(),
                    }
                }
            }
            DagPayloadOp::AnchorReceipt => {
                let allowed = self
                    .anchorers
                    .get(&scope_key)
                    .or_else(|| self.anchorers.get(&None))
                    .map(|set| set.contains(actor))
                    .unwrap_or(false);
                if allowed {
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
