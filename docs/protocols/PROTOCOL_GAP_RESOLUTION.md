# ICN Protocol Gap Resolution Checklist

> Single-source checklist to complete protocol specifications and ensure code alignment. Track spec, implementation, and tests for each item.

## Legend
- [ ] Spec — written and merged in protocol doc
- [ ] Impl — implemented in crates/APIs
- [ ] Tests — unit/integration tests cover behavior

---

## Identity & Credential Protocol
- [ ] Spec: Credential lifecycle (issue, rotate, renew, expire, revoke)
- [ ] Impl: Revocation registry/status lookup
- [ ] Tests: Revocation and expiration verification
- [ ] Spec: Social recovery (guardian thresholds, change guardians)
- [ ] Impl: Recovery flows in `icn-identity`
- [ ] Tests: Recovery scenarios
- [ ] Spec: Proof-of-personhood interface & attestations
- [ ] Impl: PoP verification hooks
- [ ] Tests: PoP edge cases

## Governance Protocol
- [ ] Spec: Working group lifecycle (create, scope, dissolve)
- [ ] Impl: Working group state and permissions
- [ ] Tests: Group-scoped proposals
- [ ] Spec: Delegation (grant, duration, recall, audit)
- [ ] Impl: Delegation records and vote routing
- [ ] Tests: Delegation conflicts and recalls
- [ ] Spec: Quorum/threshold rules per scope/type
- [ ] Impl: Enforcement in tallying
- [ ] Tests: Quorum failure, liveness

## Economic & Incentive Protocol
- [ ] Spec: Mana regeneration algorithm and parameters
- [ ] Impl: Regen with policy multipliers
- [ ] Tests: Regen invariants and limits
- [ ] Spec: Earned mana crediting rules (executors)
- [ ] Impl: `credit_earned_mana` pathway
- [ ] Tests: Credit accrual and caps
- [ ] Spec: Slashing policy (economic sabotage/misreporting)
- [ ] Impl: Slashing hooks and appeals
- [ ] Tests: Dispute and slash outcomes

## DAG & Storage Protocol
- [ ] Spec: Pruning/snapshot/recovery flows
- [ ] Impl: Snapshot write/read and rebuild
- [ ] Tests: Recovery from snapshots
- [ ] Spec: Archive cooperative selection & challenge
- [ ] Impl: Proof-of-replication/challenge protocol
- [ ] Tests: Archive audits
- [ ] Spec: Encrypted/permissioned data flow
- [ ] Impl: Access control and selective disclosure
- [ ] Tests: Confidential data integrity

## Federation Synchronization Protocol
- [ ] Spec: Dispute resolution and arbitration steps
- [ ] Impl: Conflict adjudication logic
- [ ] Tests: Fork/cheat recovery
- [ ] Spec: Bridge federations (trust, data flow)
- [ ] Impl: Bridge connectors and policies
- [ ] Tests: Cross-federation sync
- [ ] Spec: Partition healing (merge, authority, replay)
- [ ] Impl: Merge procedures and guardrails
- [ ] Tests: Rejoin scenarios

## Mesh Job Execution Protocol
- [ ] Spec: Job sharding/assignment/reassembly
- [ ] Impl: Shard coordinator and receipt aggregation
- [ ] Tests: Sharded job correctness
- [ ] Spec: Dispute/slashing protocol (steps, evidence)
- [ ] Impl: Dispute engine and penalties
- [ ] Tests: Fraud/misreporting cases
- [ ] Spec: Executor capability registry & challenge
- [ ] Impl: Capability proofs/benchmarks
- [ ] Tests: Capability spoofing challenges

## Organizational Role & Association Protocol
- [ ] Spec: Membership policy flows (open, invite, admin, consensus)
- [ ] Impl: Policy engine and transitions
- [ ] Tests: Policy switching and appeals
- [ ] Spec: Role elevation/demotion/expulsion + appeals
- [ ] Impl: Role changes with audit trail
- [ ] Tests: Due process checks
- [ ] Spec: Multi-org membership and trust propagation
- [ ] Impl: Cross-org membership graph
- [ ] Tests: Conflicts and aggregation

## Security & Adversarial Resilience Protocol
- [ ] Spec: Incident response (freeze, rollback, notify, forensics)
- [ ] Impl: Emergency hooks and audit trails
- [ ] Tests: Incident simulations
- [ ] Spec: Key/credential compromise response
- [ ] Impl: Rapid re-issuance flows
- [ ] Tests: Compromise recovery
- [ ] Spec: Privacy breach detection/reporting
- [ ] Impl: Telemetry and reporting
- [ ] Tests: Alerting pipelines

## CCL Protocol
- [ ] Spec: Contract access control & capabilities
- [ ] Impl: Capability checks in runtime
- [ ] Tests: Unauthorized access prevention
- [ ] Spec: Contract upgrade/migration (state-safe)
- [ ] Impl: Migration tooling and safeguards
- [ ] Tests: Upgrade/rollback scenarios
- [ ] Spec: Contract registry (publish, discover, audit)
- [ ] Impl: Registry service
- [ ] Tests: Registry integrity

## Integration & Versioning
- [ ] Spec: Protocol versioning & rollout procedures
- [ ] Impl: Feature flags and compatibility layers
- [ ] Tests: Rolling upgrades and downgrades


