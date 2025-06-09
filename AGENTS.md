# AGENTS.md

## InterCooperative Network Core – Agent & Contributor Guide

**Welcome, software engineering agent or contributor! This file is your starting point for effective, safe, and context-aware automation within the `icn-core` repository.**

---

### 🔗 **Foundational Rules**

> **You MUST read and follow:**  
> - `.cursor/rules/cursor-rules.mdc` – *Global ICN philosophy, terminology, repo boundaries, and systemic expectations*  
> - `.cursor/rules/icn-core-context.mdc` – *Precise rules, architectural layout, and change flow specific to `icn-core`*

CI and human review will enforce these. When in doubt, ask for review.

---

## 🏗 **Project Purpose & Scope (What is `icn-core`?)**

- `icn-core` is **the authoritative, deterministic Rust workspace** for the InterCooperative Network (ICN).
- It is *not a blockchain* or a generic DLT. It provides:
  - Modular libraries for runtime, governance, economics (mana), DAG, identity, mesh compute, CCL, and protocol definitions.
  - All logic here is protocol-level, deterministic, and suitable for embedding in ICN node binaries.
- **Out-of-scope:**  
  - UI/frontends, HTTP APIs, deployment/IaC, documentation, standalone CLI apps.  
  - These belong in other repos (`icn-web-ui`, `icn-node`, `icn-docs`, etc).

---

## 🗂 **Where to Work (and Where Not To)**

- **Allowed for agent modification:**  
  - Any crate in `/crates/` (e.g., `icn-runtime`, `icn-mesh`, `icn-governance`, `icn-dag`, `icn-identity`, `icn-economics`, `icn-ccl`, `icn-common`, `icn-protocol`, etc).
  - Test files inside each crate (`/crates/<name>/tests/`).
- **Do NOT modify unless told explicitly:**  
  - Workspace root files: `Cargo.toml`, `.github/`, scripts, onboarding docs.
  - External docs in `docs/` or `icn-docs/`.
  - Any code in non-`icn-core` workspaces, such as deployment, CLI, or web UIs.

---

## 💡 **Agent Instructions & Coding Guidelines**

1. **Determinism is paramount:**  
   - No direct wall-clock time, random seeds, or unabstracted I/O in core logic. Use traits and dependency injection.
2. **Strict modularity:**  
   - *Never* create cross-crate logic except through defined interfaces (traits/types in `icn-common`).
   - Each crate must build and test independently (`cargo test -p <crate>`).
3. **Testing and validation:**  
   - All PRs must pass:  
     - `cargo test --all-features --workspace`  
     - `cargo clippy --all-targets --all-features -- -D warnings`  
     - `cargo fmt --all -- --check`
   - Add/expand tests for all changes. Integration tests for pipeline changes.
4. **Documentation:**  
   - Every public type/function must be documented with Rustdoc (`///`).
   - Update or add examples where helpful.
5. **Terminology:**  
   - Always use: “ICN” (never “ICN Network”), “mesh job,” “execution receipt,” “mana,” “DAG,” “CCL policy/contract.” Never “blockchain,” “smart contract,” or “gas.”
6. **PR/Commit standards:**  
   - Use the PR template.  
   - Title: `[crate] <short description>`, describe affected modules, and note breaking changes.  
   - Link related issues and RFCs.

---

## 🔬 **Key Architectural Patterns**

- **Mesh Job Pipeline:**  
  - See `crates/icn-runtime/src/abi.rs` and `icn-mesh/`, `icn-economics/`, `icn-governance/` for the lifecycle.
  - Jobs must be assigned, executed, and anchored per the design in `.cursor/rules/icn-core-context.mdc`.
- **Mana and Reputation:**  
  - All compute/resource logic is mediated by regenerating mana, enforced by `icn-economics` and runtime APIs.
- **DID and VC:**  
  - All agents, proposals, and receipts are scoped to DIDs, validated through `icn-identity`.
- **DAG Ledger:**  
  - All receipts and blocks must be stored/content-addressed in `icn-dag` via canonical APIs.
- **CCL (Contract Language):**  
  - All executable policy/governance logic is compiled to WASM via `icn-ccl` (see `icn-ccl/`).
- **No direct code imports from legacy or external repos.**  
  - All logic must be fresh, modern Rust.

---

## 🛡 **Security & Safety**

- Never introduce dependencies with unpatched vulnerabilities.
- Never hardcode cryptographic keys or secrets.
- All economic, governance, and identity logic requires adversarial tests where possible.

---

## 🧩 **Example Agent Tasks**

- Refactor job assignment logic in `icn-runtime` and `icn-mesh` for clearer error handling.
- Add tests to `icn-governance` for new proposal types.
- Improve mana enforcement in `icn-economics` to handle edge exhaustion.
- Document all new/changed public types in `icn-identity` and `icn-dag`.
- Ensure all job receipts are properly anchored and content-addressed via `icn-dag`.

---

## 📖 **Further Reading**

- `.cursor/rules/cursor-rules.mdc` (global rules)
- `.cursor/rules/icn-core-context.mdc` (repo-local architectural boundaries and review flow)
- Each crate’s `README.md` and key source files

---

**Thank you for helping build a resilient, programmable cooperative infrastructure. Every contribution is a piece of this new world.**
