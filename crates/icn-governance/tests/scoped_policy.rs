use icn_common::{Did, NodeScope};
use icn_governance::scoped_policy::{
    DagPayloadOp, InMemoryPolicyEnforcer, PolicyCheckResult, ScopedPolicyEnforcer,
};
use std::str::FromStr;

#[test]
fn scoped_permission_allows_member() {
    let mut enforcer = InMemoryPolicyEnforcer::default();
    let scope = NodeScope("alpha".into());
    let did = Did::from_str("did:icn:test:alice").unwrap();
    enforcer.allow_submitter(Some(scope.clone()), did.clone());
    assert!(matches!(
        enforcer.check_permission(DagPayloadOp::SubmitBlock, &did, Some(&scope)),
        PolicyCheckResult::Allowed
    ));
}

#[test]
fn scoped_permission_denies_non_member() {
    let enforcer = InMemoryPolicyEnforcer::default();
    let scope = NodeScope("alpha".into());
    let did = Did::from_str("did:icn:test:bob").unwrap();
    assert!(matches!(
        enforcer.check_permission(DagPayloadOp::SubmitBlock, &did, Some(&scope)),
        PolicyCheckResult::Denied { .. }
    ));
}
