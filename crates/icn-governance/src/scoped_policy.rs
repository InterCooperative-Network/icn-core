pub enum PolicyCheckResult {
    Allowed,
    Denied { reason: String },
}

use icn_common::{Did, NodeScope, ZkCredentialProof, ZkRevocationProof};
use icn_identity::{Groth16Verifier, ZkVerifier};

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
        proof: Option<&ZkCredentialProof>,
        revocation: Option<&ZkRevocationProof>,
    ) -> PolicyCheckResult;
}

use std::collections::{HashMap, HashSet};

/// In-memory implementation of [`ScopedPolicyEnforcer`] based on membership lists.
#[derive(Default)]
pub struct InMemoryPolicyEnforcer {
    submitters: HashSet<Did>,
    anchorers: HashSet<Did>,
    memberships: HashMap<NodeScope, HashSet<Did>>,
    require_proof: bool,
    verifier: Groth16Verifier,
}

impl InMemoryPolicyEnforcer {
    /// Create a new enforcer with the given allowed members for submitting blocks
    /// and anchoring receipts.
    pub fn new(
        submitters: HashSet<Did>,
        anchorers: HashSet<Did>,
        memberships: HashMap<NodeScope, HashSet<Did>>,
        require_proof: bool,
    ) -> Self {
        Self {
            submitters,
            anchorers,
            memberships,
            require_proof,
            verifier: Groth16Verifier::default(),
        }
    }

    fn validate_proof(
        &self,
        proof: Option<&ZkCredentialProof>,
        revocation: Option<&ZkRevocationProof>,
    ) -> PolicyCheckResult {
        if !self.require_proof {
            return PolicyCheckResult::Allowed;
        }

        let cred_check = match proof {
            Some(p) => match self.verifier.verify(p) {
                Ok(true) => PolicyCheckResult::Allowed,
                _ => PolicyCheckResult::Denied {
                    reason: "credential proof invalid".to_string(),
                },
            },
            None => PolicyCheckResult::Denied {
                reason: "credential proof required".to_string(),
            },
        };

        if let PolicyCheckResult::Denied { .. } = cred_check {
            return cred_check;
        }

        match revocation {
            Some(rp) => match self.verifier.verify_revocation(rp) {
                Ok(true) => PolicyCheckResult::Allowed,
                _ => PolicyCheckResult::Denied {
                    reason: "revocation proof invalid".to_string(),
                },
            },
            None => PolicyCheckResult::Allowed,
        }
    }
}

impl ScopedPolicyEnforcer for InMemoryPolicyEnforcer {
    fn check_permission(
        &self,
        op: DagPayloadOp,
        actor: &Did,
        scope: Option<&NodeScope>,
        proof: Option<&ZkCredentialProof>,
        revocation: Option<&ZkRevocationProof>,
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
                            self.validate_proof(proof, revocation)
                        } else {
                            PolicyCheckResult::Denied {
                                reason: "actor not in scope".to_string(),
                            }
                        }
                    } else {
                        self.validate_proof(proof, revocation)
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
                            self.validate_proof(proof, revocation)
                        } else {
                            PolicyCheckResult::Denied {
                                reason: "actor not in scope".to_string(),
                            }
                        }
                    } else {
                        self.validate_proof(proof, revocation)
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
