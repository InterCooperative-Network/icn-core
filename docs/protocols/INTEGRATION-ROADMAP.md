# ICN Protocol Integration Roadmap

## Purpose

This document lays out a comprehensive, step-by-step plan to integrate formalized protocol specifications (Governance, Mesh Job, CCL, DAG Storage, Economics) into the existing ICN codebase. It bridges current implementation and protocol documentation, ensuring all critical theoretical features are realized in working code, with clear accountability and progress tracking.

---

## Integration Principles

- **Security First:** Prioritize time-locks, veto mechanisms, slashing, and resource metering.
- **Democratic Governance:** Membership-based, not wealth-based voting; implement all advanced voting methods.
- **Economic Alignment:** Mana regeneration, anti-speculation, and economic feedback loops per spec.
- **Federation Coordination:** X-org governance, job routing, contract calls, checkpointing.
- **Storage Resilience:** Archive cooperatives, erasure coding, adversarial-resistant DAG.
- **Progressive Implementation:** Phase by phase, with milestone reviews, tests, and documentation.

---

## Phase Breakdown

| Phase           | Target Timeline | Milestone |
|-----------------|----------------|-----------|
| Phase 1: Critical Security & Governance | Weeks 1-2         | Core protocols ported |
| Phase 2: Economic Mechanisms & Voting   | Weeks 2-4         | Working mana formulas, advanced voting |
| Phase 3: DAG Checkpointing & Storage    | Month 2           | Distributed storage active |
| Phase 4: Mesh Job & CCL Expansion       | Month 2-3         | Advanced mesh features, full contract library |
| Phase 5: Federation Coordination        | Month 3           | Cross-federation working |
| Phase 6: Testing, Auditing, Review      | Month 3-4         | All features tested, documented |

---

## Detailed Task List

### **Governance Protocol**

- [ ] **Time-locks:** Implement time-locks per proposal category (constitutional, emergency, economic, default).
  - *Action:* Extend `Proposal` struct and methods with delay logic from spec.
  - *Test:* Simulate proposal lifecycle with time-lock enforcement.
- [ ] **Veto Mechanisms:** Safety committee and guardian roles, veto logic.
  - *Action:* Add veto roles, veto action and event recording.
  - *Test:* Trigger veto on constitutional and emergency proposals.
- [ ] **Advanced Voting:** Port ranked choice, quadratic, conviction, consensus voting.
  - *Action:* Extend `VotingMethod` enum; refactor voting/tallying algorithms.
  - *Test:* End-to-end voting scenarios for each method.
- [ ] **Proposal Lifecycle Extensions:** Add sponsorship, grace period, and DAG audit trail.
  - *Action:* Update proposal state transitions and DAG integration.
  - *Test:* Validate full lifecycle, including edge cases.

### **Economic Protocol**

- [ ] **Mana Regeneration:** Implement regeneration formulas (compute score, trust multiplier, participation).
  - *Action:* Refactor `ManaLedger` and background regeneration tasks.
  - *Test:* Regression tests for edge cases (low/high trust, emergency mode).
- [ ] **Token System Upgrades:** Demurrage, soul-binding, velocity limits, purpose locks.
  - *Action:* Extend token ledger logic; enforce transfer/mint/burn restrictions.
  - *Test:* Attempt invalid operations, verify correct enforcement.
- [ ] **Anti-Spam Mechanics:** Waivable voting/proposal fees, stake logic.
  - *Action:* Add fee waiver checks based on member mana balance.
  - *Test:* Low-balance member can vote/propose without fee barrier.

### **DAG & Storage Protocol**

- [ ] **Checkpointing:** Implement federation checkpoint creation, signing, and validation.
  - *Action:* New `CheckpointManager`; periodic checkpoint block creation; validator multisig.
  - *Test:* Simulate checkpoint creation and fork reconciliation.
- [ ] **Archive Cooperatives:** Elected archives, erasure coding, proof-of-storage challenges.
  - *Action:* Add archive node roles, erasure code integration, challenge protocol.
  - *Test:* Archive failure and recovery simulation.
- [ ] **Storage Economics:** Mana rebates/gateway rewards, storage token payouts.
  - *Action:* Integrate with economic layer, periodic incentive calculation.
  - *Test:* Storage operation cost/reward tracking.

### **Mesh Job Protocol**

- [ ] **Sandboxing:** Add Docker sandboxing to existing WASM executor.
  - *Action:* Extend `JobExecution` to support Docker; resource isolation.
  - *Test:* Run jobs in both WASM and Docker, validate security limits.
- [ ] **Sharded Execution & Federated Learning:** Implement job sharding, federated learning coordination.
  - *Action:* New sharding logic, federated learning primitives.
  - *Test:* Multi-participant jobs and model aggregation.
- [ ] **Dispute & Validation:** Result validation, dispute protocol, validator re-execution.
  - *Action:* Add validation and dispute flows.
  - *Test:* Trigger dispute, observe re-execution and resolution.

### **CCL Protocol**

- [ ] **Standard Library Expansion:** Port governance, economic, resource contracts.
  - *Action:* Implement standard contracts per spec; extend compiler/runtime.
  - *Test:* Deploy/test all standard contracts in test environment.
- [ ] **Federation Scoping & Cross-Contract Calls:** Federation-aware contracts, cross-federation call support.
  - *Action:* Extend contract scoping logic; implement federated call protocol.
  - *Test:* Simulate cross-federation contract invocation.
- [ ] **Formal Verification:** Add property/invariant verification test framework.
  - *Action:* Extend test harness to support formal specs.
  - *Test:* Run property-based tests on contracts.

---

## Ownership & Dependencies

| Area         | Lead Developer(s) | Dependencies                |
|--------------|-------------------|-----------------------------|
| Governance   | [Assign here]     | icn-dag, icn-economics      |
| Economics    | [Assign here]     | icn-identity, icn-dag       |
| Mesh Job     | [Assign here]     | icn-dag, icn-economics      |
| CCL          | [Assign here]     | ccl-core, icn-dag           |
| Storage/DAG  | [Assign here]     | icn-dag, icn-economics      |
| Federation   | [Assign here]     | icn-dag, icn-governance     |

---

## Milestones & Review Points

- **End of Week 2:** Security-critical features merged, reviewed.
- **End of Month 1:** Economic formulas, voting, checkpointing live in testnet.
- **End of Month 2:** Mesh job sharding, federated learning, CCL contracts functional.
- **End of Month 3:** Cross-federation working, archive proofs active.
- **End of Month 4:** All features audited and documented.

---

## Risk Areas & Testing

- **Backward compatibility:** Ensure new proposal/voting logic does not break legacy workflows.
- **Security audit:** All mana, token, archive, and voting features must be fuzzed and pen-tested.
- **Performance:** DAG throughput, job sandboxing, federated calls must meet latency specs.
- **Economic correctness:** Mana regeneration and token operations must align with real resource usage.

---

## Appendix: Progress Checklist

- [ ] Time-locks & vetoes in governance
- [ ] Mana regeneration formulas
- [ ] Advanced voting methods
- [ ] Checkpointing & archive protocols
- [ ] Mesh job sharding & sandboxing
- [ ] CCL standard library
- [ ] Federation bridges & calls
- [ ] Testing, documentation, audit

---

**Document Control:**  
- Last Updated: 2025-08-07  
- Owner: fahertym  
- Status: Draft (to be updated as phases are completed)