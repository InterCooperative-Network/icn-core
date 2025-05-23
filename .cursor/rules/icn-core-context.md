---
description: ICN Core System – Architecture and Context
alwaysApply: true
---

# InterCooperative Network (ICN) Core – Context & Coding Rules

## About

This repository is the **core monorepo for the InterCooperative Network (ICN)**. It is composed of several Rust crates, each implementing a major architectural subsystem of a decentralized, trust-based economic and governance platform.

**Before making any changes, always read and understand the relevant crate(s) as described below.**

---

## Repository Structure

- `crates/icn-runtime/` — Node host runtime: executes WASM, enforces policy, provides host ABI functions (mesh jobs, mana, receipts)
- `crates/icn-mesh/` — Global compute mesh: job marketplace, job/bid structs, executor selection, job lifecycle
- `crates/icn-economics/` — Regenerating resource (mana), economic policy, token accounting, mana enforcement
- `crates/icn-governance/` — Proposals, voting, parameter management (rarely required for mesh work)
- `crates/icn-identity/` — Decentralized identity (DID), credential verification, used throughout
- `crates/icn-dag/` — Content-addressed DAG (block store, receipt anchoring)
- `crates/icn-api/`, `icn-cli/` — API endpoints, CLI interface (do not change unless working on external interfaces)

---

## System Flow: Mesh Job Pipeline

> The most critical feature is the **planetary mesh compute system**, powered by jobs, bids, and receipts.

**Canonical mesh job lifecycle:**

1. **Submission**
    - Call: `host_submit_mesh_job` (in `icn-runtime/src/abi.rs`)
    - Checks: submitter DID, mana ≥ job cost (`host_account_spend_mana`)
    - Inserts job into `RuntimeContext::pending_mesh_jobs` (in `icn-runtime/src/context.rs`)
2. **Bidding**
    - Executors discover jobs, submit bids (in `icn-mesh`)
    - Bids must come from DIDs with enough mana; reputation is considered (via `ReputationExecutorSelector`)
3. **Assignment**
    - Job manager selects best executor via policy (score: reputation, price, resources)
    - Job state transitions to assigned; executor is notified
4. **Execution**
    - Assigned executor runs job (simulated in test/mock), produces a signed `ExecutionReceipt`
5. **Anchoring**
    - Receipt is anchored via `host_anchor_receipt` to the DAG (in `icn-runtime`)
    - Reputation system is updated
6. **Edge Cases**
    - Insufficient mana or reputation = job or bid rejected
    - No valid bids = refund mana, mark job as failed

---

## Reading Order for Code Generation

**For any mesh, job, or economics change:**

1. **`crates/icn-runtime/src/abi.rs`**
   - Host ABI surface: read all `host_*` fns (esp. `host_submit_mesh_job`, `host_anchor_receipt`, `host_account_spend_mana`)
2. **`crates/icn-runtime/src/context.rs`**
   - `RuntimeContext`: manages queues, mana enforcement, state
3. **`crates/icn-mesh/src/lib.rs`**
   - Job, bid, executor structs and selection logic
   - Read all `select_executor`, `score_bid`, and job state enums
4. **`crates/icn-economics/src/lib.rs`**
   - `ResourcePolicyEnforcer`, `ManaRepositoryAdapter`, mana metering
5. **`crates/icn-runtime/src/lib.rs`**
   - Ties together runtime tasks, job queue management
6. **Tests**: `icn-runtime/tests/`, `icn-mesh/tests/`
   - For end-to-end pipeline, error handling, edge case verification

---

## Coding Guidelines

- **Never bypass mana or reputation checks** in job, bid, or assignment logic.
- **Always update job state** atomically; log all state changes if possible.
- **Extend or update Rustdoc** for all new or changed public APIs, structs, or error types.
- **Write/extend tests** for every new feature, error case, or change to the pipeline.
- **Maintain backward compatibility** unless doing a breaking change with justification.

---

## Example: End-to-End Mesh Job Flow

- Submitter calls `host_submit_mesh_job` (spends mana, job accepted if balance sufficient)
- Executors discover jobs, submit bids (bidder must have sufficient mana; reputation is scored)
- `select_executor` chooses the best executor; assignment is stored
- Executor runs job (simulated), produces a signed `ExecutionReceipt`
- Receipt is anchored and reputation updated
- Error: if no valid bids, job fails and mana is refunded

---

## Tips

- If uncertain, search for function or struct definitions starting from `icn-runtime/src/abi.rs`, then follow the pipeline
- Do not add business logic to `icn-api/` or `icn-cli/`
- Use types from `icn-common` if you need to share code between crates

---

**This rule is always applied when working with this repo.**

---
