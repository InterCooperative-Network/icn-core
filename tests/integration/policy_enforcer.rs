use std::collections::HashSet;
use std::str::FromStr;

use icn_common::{compute_merkle_cid, DagBlock, DagLink, Did, NodeScope, SignatureBytes};
use icn_runtime::context::RuntimeContext;
use icn_governance::scoped_policy::{DagPayloadOp, PolicyCheckResult, ScopedPolicyEnforcer};

#[derive(Debug, PartialEq, Eq)]
enum PolicyError {
    Unauthorized,
    InvalidParent,
}

trait MembershipResolver {
    fn is_member(&self, did: &Did, scope: &NodeScope) -> bool;
}

struct StaticMembershipResolver {
    members: HashSet<Did>,
}

impl StaticMembershipResolver {
    fn new(members: HashSet<Did>) -> Self {
        Self { members }
    }
}

impl MembershipResolver for StaticMembershipResolver {
    fn is_member(&self, did: &Did, _scope: &NodeScope) -> bool {
        self.members.contains(did)
    }
}

struct MockScopedPolicyEnforcer<R: MembershipResolver> {
    resolver: R,
}

impl<R: MembershipResolver> MockScopedPolicyEnforcer<R> {
    fn new(resolver: R) -> Self {
        Self { resolver }
    }
}

impl<R: MembershipResolver> ScopedPolicyEnforcer for MockScopedPolicyEnforcer<R> {
    fn check_permission(
        &self,
        _op: DagPayloadOp,
        actor: &Did,
        scope: Option<&NodeScope>,
    ) -> PolicyCheckResult {
        if let Some(scope) = scope {
            if self.resolver.is_member(actor, scope) {
                PolicyCheckResult::Allowed
            } else {
                PolicyCheckResult::Denied {
                    reason: "scope membership missing".to_string(),
                }
            }
        } else if self.resolver.is_member(actor, &NodeScope("default".into())) {
            PolicyCheckResult::Allowed
        } else {
            PolicyCheckResult::Denied {
                reason: "unauthorized".to_string(),
            }
        }
    }
}

async fn anchor_block_with_policy<E: ScopedPolicyEnforcer>(
    ctx: &RuntimeContext,
    block: &DagBlock,
    enforcer: &E,
) -> Result<(), PolicyError> {
    if let PolicyCheckResult::Denied { .. } = enforcer.check_permission(
        DagPayloadOp::SubmitBlock,
        &block.author_did,
        block.scope.as_ref(),
    )
    {
        return Err(PolicyError::Unauthorized);
    }

    {
        let store = ctx.dag_store.lock().await;
        for link in &block.links {
            if !store.contains(&link.cid).unwrap() {
                return Err(PolicyError::InvalidParent);
            }
        }
    }

    {
        let mut store = ctx.dag_store.lock().await;
        store.put(block).unwrap();
    }
    Ok(())
}

#[tokio::test]
async fn authorized_dag_write_succeeds() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut members = HashSet::new();
    members.insert(alice.clone());
    let resolver = StaticMembershipResolver::new(members);
    let enforcer = MockScopedPolicyEnforcer::new(resolver);

    let data = b"block".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    anchor_block_with_policy(&ctx, &block, &enforcer)
        .await
        .expect("write succeeds");

    let stored = ctx.dag_store.lock().await.get(&cid).unwrap();
    assert!(stored.is_some());
}

#[tokio::test]
async fn unauthorized_write_denied() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut members = HashSet::new();
    members.insert(alice.clone());
    let resolver = StaticMembershipResolver::new(members);
    let enforcer = MockScopedPolicyEnforcer::new(resolver);

    let eve = Did::from_str("did:example:eve").unwrap();
    let data = b"bad".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &eve, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: eve.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer).await;
    assert_eq!(res, Err(PolicyError::Unauthorized));
}

#[tokio::test]
async fn invalid_parent_is_rejected() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut members = HashSet::new();
    members.insert(alice.clone());
    let resolver = StaticMembershipResolver::new(members);
    let enforcer = MockScopedPolicyEnforcer::new(resolver);

    let missing_cid = compute_merkle_cid(0x71, b"parent", &[], 0, &alice, &None, &None);
    let link = DagLink {
        cid: missing_cid,
        name: "parent".into(),
        size: 0,
    };
    let data = b"child".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[link.clone()], ts, &alice, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![link],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer).await;
    assert_eq!(res, Err(PolicyError::InvalidParent));
}

#[tokio::test]
async fn scope_membership_enforced() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let scope = NodeScope("testscope".into());
    let mut members = HashSet::new();
    members.insert(alice.clone());
    let resolver = StaticMembershipResolver::new(members);
    let enforcer = MockScopedPolicyEnforcer::new(resolver);

    let data = b"scoped".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &Some(scope.clone()));
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice,
        signature: None,
        scope: Some(scope),
    };

    anchor_block_with_policy(&ctx, &block, &enforcer)
        .await
        .expect("scoped write succeeds");
}

