# AGENTS.md

## InterCooperative Network Core – Agent & Contributor Guide

**Welcome, software engineering agent or contributor! This file is your starting point for effective, safe, and context-aware automation within the `icn-core` repository.**

---

### 🔗 **Foundational Rules & Context**

> **You MUST read and follow these comprehensive context files:**

#### **Core Rules & Philosophy**
- `.cursor/rules/cursor-rules.mdc` – *Global ICN philosophy, terminology, repo boundaries, and systemic expectations*
- `.cursor/rules/icn-core-context.mdc` – *Precise rules, architectural layout, and change flow specific to `icn-core`*

#### **Architecture & Technical Context**
- `.cursor/rules/crate-architecture.mdc` – *Detailed breakdown of crate dependencies, responsibilities, and interaction patterns*
- `.cursor/rules/api-contracts.mdc` – *External interfaces, DTOs, HTTP APIs, and integration patterns*
- `.cursor/rules/security-validation.mdc` – *Security patterns, validation strategies, and safety considerations*

#### **Development & Operations**
- `.cursor/rules/development-workflow.mdc` – *Development processes, testing strategies, CI/CD, and code quality standards*
- `.cursor/rules/troubleshooting.mdc` – *Debugging guides, common issues, and problem resolution patterns*

**CI and human review will enforce these guidelines. When in doubt, ask for review.**

---

## 🏗 **Project Purpose & Scope (What is `icn-core`?)**

- `icn-core` is **the authoritative, deterministic Rust workspace** for the InterCooperative Network (ICN).
- It is *not a blockchain* or a generic DLT. It provides:
  - Modular libraries for runtime, governance, economics (mana), DAG, identity, mesh compute, and protocol definitions.
  - All logic here is protocol-level, deterministic, and suitable for embedding in ICN node binaries.
- **Out-of-scope:**  
  - UI/frontends, HTTP APIs, deployment/IaC, documentation, standalone CLI apps.  
  - These belong in other repos (`icn-web-ui`, `icn-node`, `icn-docs`, etc).

---

## 🗂 **Where to Work (and Where Not To)**

- **Allowed for agent modification:**  
  - Any crate in `/crates/` (e.g., `icn-runtime`, `icn-mesh`, `icn-governance`, `icn-dag`, `icn-identity`, `icn-economics`, `icn-common`, `icn-protocol`, `icn-reputation`, `icn-network`, `icn-api`, `icn-cli`, `icn-node`).
  - Test files inside each crate (`/crates/<name>/tests/`).
- **Do NOT modify unless told explicitly:**  
  - Workspace root files: `Cargo.toml`, `.github/`, scripts, onboarding docs.
  - External docs in `docs/` or `icn-docs/`.
  - Any code in non-`icn-core` workspaces, such as deployment, CLI, or web UIs.

---

## 💡 **Agent Instructions & Coding Guidelines**

### **Core Principles**
1. **Determinism is paramount:**  
   - No direct wall-clock time, random seeds, or unabstracted I/O in core logic. Use traits and dependency injection.
2. **Strict modularity:**  
   - *Never* create cross-crate logic except through defined interfaces (traits/types in `icn-common`).
   - Each crate must build and test independently (`cargo test -p <crate>`).
3. **Security by design:**
   - All actions must be attributable to a DID
   - All economic transactions must be mana-enforced
   - All network messages must be signed and verified
   - Follow validation patterns from `security-validation.mdc`

### **Development Standards**
4. **Testing and validation:**  
   - All PRs must pass:  
     - `cargo test --all-features --workspace`  
     - `cargo clippy --all-targets --all-features -- -D warnings`  
     - `cargo fmt --all -- --check`
   - Add/expand tests for all changes. Integration tests for pipeline changes.
   - Follow testing patterns from `development-workflow.mdc`
5. **Documentation:**  
   - Every public type/function must be documented with Rustdoc (`///`).
   - Update or add examples where helpful.
   - Update relevant `.mdc` files for architectural changes.
6. **Terminology:**  
   - Always use: "ICN" (never "ICN Network"), "mesh job," "execution receipt," "mana," "DAG," "CCL policy/contract." 
   - Never "blockchain," "smart contract," or "gas."

### **Change Management**
7. **PR/Commit standards:**  
   - Use the PR template.  
   - Title: `[crate] <short description>`, describe affected modules, and note breaking changes.  
   - Link related issues and RFCs.
8. **Cross-crate changes:**
   - Follow modification guidelines in `crate-architecture.mdc`
   - Update API contracts in `icn-api` first for external interfaces
   - Maintain backward compatibility unless explicitly versioned

---

## 🔬 **Key Architectural Patterns**

### **Mesh Job Pipeline** (Primary Workflow)
- See `crates/icn-runtime/src/abi.rs` and `icn-mesh/`, `icn-economics/`, `icn-governance/` for the lifecycle.
- Jobs must be assigned, executed, and anchored per the design in `.cursor/rules/icn-core-context.mdc`.
- Follow the canonical flow: Submission → Bidding → Assignment → Execution → Anchoring

### **Economic Model**
- All compute/resource logic is mediated by regenerating mana, enforced by `icn-economics` and runtime APIs.
- Mana conservation must be maintained across all transactions
- Rate limiting and anti-spam protection required

### **Identity & Trust**
- All agents, proposals, and receipts are scoped to DIDs, validated through `icn-identity`.
- Reputation influences resource access and executor selection
- All signatures must be verified and fresh (no replay attacks)

### **Content-Addressed Storage**
- All receipts and blocks must be stored/content-addressed in `icn-dag` via canonical APIs.
- DAG integrity must be maintained
- Receipt anchoring provides verifiable execution history

### **Governance & Policy**
- All executable policy/governance logic is compiled to WASM via CCL
- Proposal and voting workflows must be followed for parameter changes
- No direct code imports from legacy or external repos—all logic must be fresh, modern Rust.

---

## 🛡 **Security & Safety**

### **Critical Security Invariants**
- Never introduce dependencies with unpatched vulnerabilities.
- Never hardcode cryptographic keys or secrets.
- All economic, governance, and identity logic requires adversarial tests where possible.
- Follow input validation patterns for all external data.

### **Common Security Patterns**
- Use structured validation for DIDs, job specifications, and transactions
- Implement proper error handling with specific error types
- Apply rate limiting and resource constraints
- Use circuit breakers for external dependencies
- See `security-validation.mdc` for detailed patterns

---

## 🧩 **Example Agent Tasks**

### **Development Tasks**
- Refactor job assignment logic in `icn-runtime` and `icn-mesh` for clearer error handling.
- Add tests to `icn-governance` for new proposal types.
- Improve mana enforcement in `icn-economics` to handle edge exhaustion.
- Document all new/changed public types in `icn-identity` and `icn-dag`.
- Ensure all job receipts are properly anchored and content-addressed via `icn-dag`.

### **Debugging & Maintenance**
- Investigate performance issues using patterns from `troubleshooting.mdc`
- Add monitoring and health checks for system components
- Implement graceful degradation for network connectivity issues
- Add comprehensive error recovery patterns

### **Security & Validation**
- Implement additional input validation for new message types
- Add property-based tests for economic invariants
- Enhance peer authentication mechanisms
- Audit and improve CCL compilation security

---

## 📖 **Further Reading & Resources**

### **Primary Context Files**
- `.cursor/rules/cursor-rules.mdc` (global rules and philosophy)
- `.cursor/rules/icn-core-context.mdc` (repo-local architectural boundaries)
- `.cursor/rules/crate-architecture.mdc` (detailed crate breakdown)

### **Specialized Guides**
- `.cursor/rules/development-workflow.mdc` (development processes and testing)
- `.cursor/rules/api-contracts.mdc` (external interfaces and integration)
- `.cursor/rules/security-validation.mdc` (security patterns and validation)
- `.cursor/rules/troubleshooting.mdc` (debugging and problem resolution)

### **Implementation References**
- Each crate's `README.md` and key source files
- Integration tests in `/tests/` directories
- API documentation generated by `cargo doc`

---

## 🚀 **Quick Start for New Contributors**

1. **Read the foundational context files** (especially `cursor-rules.mdc` and `icn-core-context.mdc`)
2. **Understand the architecture** using `crate-architecture.mdc`
3. **Set up development environment** following `development-workflow.mdc`
4. **Run the validation suite:** `just validate` (or individual `cargo` commands)
5. **Start with small, focused changes** in a single crate
6. **Follow security patterns** from `security-validation.mdc`
7. **Use troubleshooting guide** when issues arise

---

**Thank you for helping build a resilient, programmable cooperative infrastructure. Every contribution is a piece of this new world.**
