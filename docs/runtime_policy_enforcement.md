# Runtime Policy Enforcement with Scoped Trust

This guide covers how to configure `ScopedPolicyEnforcer` and `TrustPolicyEngine` so runtime operations respect federation trust policies. Use this when deploying an ICN node that must validate DAG writes and governance actions at runtime.

## 1. Configure `TrustPolicyEngine`

```rust
use icn_identity::{TrustPolicyEngine, TrustPolicyRule, TrustContext, TrustLevel};

let mut trust_engine = TrustPolicyEngine::new();
trust_engine.add_rule(TrustPolicyRule {
    name: "resource-sharing".to_string(),
    applicable_contexts: [TrustContext::ResourceSharing].into_iter().collect(),
    min_trust_level: TrustLevel::Partial,
    require_federation_membership: true,
    allow_cross_federation: false,
    ..Default::default()
});
```

Memberships and federation bridges can then be added using `add_federation_membership` and `add_bridge`.

## 2. Set up `ScopedPolicyEnforcer`

```rust
use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
use icn_common::{Did, NodeScope};
use std::collections::{HashMap, HashSet};

let submitters: HashSet<Did> = [alice.clone()].into_iter().collect();
let anchorers: HashSet<Did> = [alice.clone()].into_iter().collect();
let memberships: HashMap<NodeScope, HashSet<Did>> = [
    (NodeScope("housing".into()), [alice.clone()].into_iter().collect()),
]
.into_iter()
.collect();

let enforcer = InMemoryPolicyEnforcer::new(submitters, anchorers, memberships, false);
```

The enforcer validates that DAG writes and receipt anchoring only occur from members of the appropriate `NodeScope`.

## 3. Build `ServiceConfig` and `RuntimeContext`

```rust
use icn_runtime::context::{ServiceConfigBuilder, ServiceEnvironment};
use icn_runtime::RuntimeContext;
use std::sync::Arc;

let config = ServiceConfigBuilder::new(ServiceEnvironment::Testing)
    .with_identity(alice.clone())
    .with_signer(Arc::new(icn_runtime::context::signers::StubSigner::new()))
    .with_did_resolver(Arc::new(icn_identity::KeyDidResolver))
    .with_mana_ledger(icn_runtime::context::mana::SimpleManaLedger::new(temp_path))
    .with_policy_enforcer(Arc::new(enforcer))
    .build()?;

let mut ctx = RuntimeContext::new(config, trust_engine)?;
```

## 4. Trust-Scoped Job Execution Example

Jobs can specify a `NodeScope` so that only authorized cooperatives may anchor receipts. The runtime checks the `ScopedPolicyEnforcer` before accepting the job.

```rust
use icn_runtime::context::resource_ledger::ResourceAction;

let scope = Some(NodeScope("housing".into()));
ctx.record_resource_event("server-42".into(), ResourceAction::Acquire, scope.clone(), 5).await?;
```

If `alice` is not a member of the `housing` scope, the call fails with `PermissionDenied`.

## 5. CCL Contract Enforcement

`FederationGovernanceEngine` uses the same `TrustPolicyEngine` to validate proposals or votes embedded in CCL policies. A contract can require a specific trust level:

```rust
use icn_governance::{FederationGovernanceEngine, TrustAwareGovernancePolicy, GovernanceAction};

let mut gov = FederationGovernanceEngine::new(trust_engine.clone(), Some(fed_id));

gov.add_policy("add-resource".to_string(), TrustAwareGovernancePolicy {
    action: GovernanceAction::ExecuteProposal { proposal_id: 1.into() },
    required_context: TrustContext::ResourceSharing,
    min_trust_level: TrustLevel::Partial,
    require_federation_membership: true,
    voting_threshold: 0.6,
    quorum_requirement: 0.3,
    allow_cross_federation: false,
});
```

When a CCL policy triggers `ExecuteProposal`, `FederationGovernanceEngine` verifies that the caller satisfies the rule via the trust engine.

---

This combination of `ScopedPolicyEnforcer` and `TrustPolicyEngine` ensures that runtime operations, mesh jobs, and CCL contracts all respect the cooperative trust boundaries encoded by each federation.
