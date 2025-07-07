# AGENTS.md

## InterCooperative Network Core – Agent & Contributor Guide

**Welcome, software engineering agent or contributor! This file is your comprehensive guide to building the infrastructure for a cooperative digital economy within the `icn-core` repository.**

---

## 🌐 **Big-Picture Overview: What We're Building & Why It Matters**

The InterCooperative Network (ICN) is a **federated, privacy-respecting infrastructure stack** that lets communities, cooperatives, and federations coordinate as peers—free from the extractive gravity of corporate cloud or speculative blockchains.

### **Core Vision**
A planet-scale fabric where every cooperative can:
- **Publish bylaws as executable policy** using CCL (Cooperative Contract Language)
- **Exchange resources using a regenerating token economy** (mana) that prevents extraction
- **Prove history via content-addressed DAG storage** with tamper-evident lineage
- **Maintain sovereignty** on their own hardware or with trusted peers

### **End-to-End Flow**
1. **Proposal** → A community encodes governance logic in CCL
2. **Consensus-Less Execution** → Proposals evaluated deterministically in CoVM runtime embedded in each ICN node
3. **Mesh Jobs** → Work is sharded across the network and paid in mana; executors anchor ExecutionReceipts to the DAG
4. **Verifiable History** → Receipts, votes, amendments, and economic events form tamper-evident DAG lineage
5. **Human Interfaces** → Wallets, explorers, and governance portals surface the state to people

---

## 🏗 **Repository Constellation & Architecture**

`icn-core` feeds everything. Every binary or UI ultimately links against these crates or the ABI they define. **If it's non-deterministic, it lives outside this repo.**

| Layer | Repo(s) | Purpose | Depends On |
|-------|---------|---------|------------|
| **Core Protocol** | `icn-core` (this repo) | Deterministic libraries: runtime, DAG, identity, economics, governance, mesh APIs | — |
| **Node Binaries** | `icn-node`, `icn-devnet`, `icn-infra` | Wrap core crates with network I/O, CLI, config, orchestration | `icn-core` |
| **Front-Ends** | `icn-wallet`, `icn-web-ui`, `icn-explorer`, `icn-agoranet` | Web & desktop UIs for users, operators, & voters | `icn-core` APIs via gRPC/HTTP |
| **Tooling & Docs** | `icn-docs`, `icn-website` | Specs, onboarding, marketing | All repos for examples |

### **What is `icn-core`?**
- **The authoritative, deterministic Rust workspace** for the InterCooperative Network
- **NOT a blockchain** or generic DLT—it's a programmable cooperative infrastructure
- **Provides:** Modular libraries for runtime, governance, economics (mana), DAG, identity, mesh compute, CCL compilation, and protocol definitions
- **All logic here is:** Protocol-level, deterministic, and suitable for embedding in ICN node binaries

---

## 🔗 **Foundational Rules & Context**

> **You MUST read and follow these comprehensive context files:**

### **Core Rules & Philosophy**
- `.cursor/rules/cursor-rules.mdc` – *Global ICN philosophy, terminology, repo boundaries, and systemic expectations*
- `.cursor/rules/icn-core-context.mdc` – *Precise rules, architectural layout, and change flow specific to `icn-core`*

### **Architecture & Technical Context**
- `.cursor/rules/crate-architecture.mdc` – *Detailed breakdown of crate dependencies, responsibilities, and interaction patterns*
- `.cursor/rules/api-contracts.mdc` – *External interfaces, DTOs, HTTP APIs, and integration patterns*
- `.cursor/rules/security-validation.mdc` – *Security patterns, validation strategies, and safety considerations*

### **Development & Operations**
- `.cursor/rules/development-workflow.mdc` – *Development processes, testing strategies, CI/CD, and code quality standards*
- `.cursor/rules/troubleshooting.mdc` – *Debugging guides, common issues, and problem resolution patterns*

**CI and human review will enforce these guidelines. When in doubt, ask for review.**

---

## 🎯 **Agent Authority & Responsibility**

### **You are empowered to make any changes necessary to achieve the project goals, including:**

- **Core library code** in `/crates/` – The primary ICN protocol implementation
- **CCL compiler and runtime** – Cooperative Contract Language toolchain (may be in `/ccl/` or integrated into crates)
- **CoVM runtime** – The Cooperative Virtual Machine that executes CCL contracts
- **Workspace configuration** – `Cargo.toml`, build scripts, tooling configs that improve development
- **CI/CD pipeline** – `.github/workflows/` to ensure quality, testing, and automation
- **Development tooling** – Scripts, justfiles, pre-commit hooks that support contributors  
- **Project documentation** – READMEs, architectural guides, onboarding materials within this repo
- **Testing infrastructure** – Integration tests, benchmarks, test utilities, CCL test contracts
- **Build and deployment** – Configurations that help build and distribute the libraries

### **Guiding Principles for Changes:**

1. **Goal Alignment** – Every change must advance ICN's mission of creating programmable, governable, cooperative digital infrastructure
2. **Determinism First** – Core logic must be predictable and verifiable across all nodes
3. **Cooperative Values** – Design for sovereignty, mutual aid, and non-extraction
4. **Quality Assurance** – Changes must maintain or improve code quality, security, and reliability
5. **Maintainability** – Consider long-term maintenance and contributor experience
6. **Scope Awareness** – Understand what belongs in `icn-core` vs. other ICN repositories

### **When to Exercise Broader Authority:**

- **Infrastructure Improvements:** Enhance CI/CD, add better tooling, improve build processes
- **Quality Enforcement:** Add linting, formatting, security checks, automated testing
- **Developer Experience:** Improve onboarding, documentation, debugging tools
- **Architecture Evolution:** Refactor workspace structure if it better serves the project goals
- **CCL Enhancement:** Improve the Cooperative Contract Language compiler, runtime, or tooling
- **Integration:** Ensure all components work together effectively

---

## 💡 **Core Systems & Components**

### **1. Cooperative Contract Language (CCL)**

CCL is the domain-specific language for encoding governance policies and cooperative bylaws as executable code.

**Key Concepts:**
- **Governance as Code:** Bylaws, voting mechanisms, and resource policies written in CCL
- **Deterministic Execution:** CCL compiles to WASM for consistent execution across all nodes
- **Policy Templates:** Common cooperative patterns (consensus mechanisms, resource allocation, etc.)
- **Composability:** Policies can reference and build upon other policies

**CCL Workflow:**
```
Community Bylaws (CCL source) 
    ↓ (compile)
WASM Bytecode 
    ↓ (load into)
CoVM Runtime 
    ↓ (execute in)
ICN Node Context
```

**Where CCL Lives:**
- CCL compiler may be in `/ccl/`, `/crates/icn-ccl/`, or integrated into the runtime crates
- Test contracts and examples alongside the compiler
- Standard library of cooperative governance patterns

### **2. Cooperative Virtual Machine (CoVM)**

The CoVM is the runtime environment that executes compiled CCL contracts within ICN nodes.

**Responsibilities:**
- **Secure Execution:** Sandboxed WASM execution with resource limits
- **Host ABI:** Provides CCL contracts access to ICN primitives (mana, DAG, identity, etc.)
- **State Management:** Manages contract state and inter-contract communication
- **Governance Integration:** Executes governance logic and policy enforcement

**CoVM Integration Points:**
- Embedded in `icn-runtime` for job execution
- Used by `icn-governance` for proposal evaluation
- Accessed via Host ABI functions in contract execution context

### **3. Core Protocol Stack**

#### **`icn-runtime`** - Node Host Runtime
- **Purpose:** Node orchestration, Host-ABI, system integration
- **Key Files:** `src/abi.rs` (Host ABI), `src/context.rs` (RuntimeContext), `src/covm.rs` (CoVM integration)
- **Responsibilities:** Job management, mana enforcement, CoVM hosting, receipt anchoring

#### **`icn-mesh`** - Decentralized Compute Mesh
- **Purpose:** Global compute mesh (jobs, bids, executor selection)
- **Key Concepts:** Job specification, bidding mechanisms, executor reputation
- **Integration:** Works with economics for mana enforcement, runtime for execution

#### **`icn-economics`** - Regenerating Resource Economy
- **Purpose:** Mana management, economic policy, anti-extraction measures
- **Key Concepts:** Regenerating mana, reputation-based regeneration, economic governance
- **Critical Invariants:** Mana conservation, no extraction, fair regeneration

#### **`icn-governance`** - Participatory Governance Engine
- **Purpose:** Proposal creation, voting mechanisms, parameter management
- **CCL Integration:** Executes governance policies compiled to WASM
- **Workflows:** Proposal → Discussion → Voting → Implementation → Anchoring

#### **`icn-identity`** - Decentralized Identity & Trust
- **Purpose:** DID management, credential verification, execution receipts
- **Key Concepts:** Scoped identity, verifiable credentials, reputation tracking
- **Security:** All actions attributed to DIDs, signatures verified

#### **`icn-dag`** - Content-Addressed Storage
- **Purpose:** Immutable storage with DAG semantics for receipts and state
- **Key Concepts:** Content addressing, receipt anchoring, tamper-evident history
- **Integration:** All significant actions anchored here for verifiability

#### **`icn-network`** - P2P Networking & Discovery
- **Purpose:** Peer-to-peer communication, message routing, network maintenance
- **Responsibilities:** Peer discovery, message signing/verification, network security

---

## 🔬 **Key Architectural Patterns & Workflows**

### **Primary Workflow: Mesh Job Pipeline**
```
1. Submission (icn-runtime) 
   ↓ host_submit_mesh_job → Check DID, mana ≥ cost
2. Bidding (icn-mesh + icn-network) 
   ↓ JobManager announces → Executors submit bids
3. Assignment (icn-runtime + icn-mesh) 
   ↓ select_executor with policy → Job state = Assigned
4. Execution (by Executor) 
   ↓ Run job → Produce signed ExecutionReceipt
5. Anchoring (icn-runtime + icn-dag + icn-identity) 
   ↓ host_anchor_receipt → Validate & store in DAG → Update reputation
```

### **Governance Workflow: CCL Policy Execution**
```
1. Proposal Creation 
   ↓ Community writes CCL policy → Compile to WASM
2. Governance Submission 
   ↓ Submit proposal with WASM → Validate & anchor
3. Discussion & Amendment 
   ↓ Community debate → Potential CCL modifications
4. Voting Execution 
   ↓ CoVM executes voting logic → Weighted by reputation/mana
5. Policy Implementation 
   ↓ If passed, policy becomes active → Runtime enforcement
6. Historical Anchoring 
   ↓ All steps anchored in DAG → Tamper-evident governance history
```

### **Economic Model: Regenerating Mana**
- **Regeneration:** Mana regenerates over time based on reputation and policy
- **Conservation:** Total mana in system remains constant (no inflation/deflation)
- **Anti-Extraction:** Prevents wealth concentration and rent-seeking
- **Governance:** Regeneration rates controlled by CCL policies

---

## 💻 **Agent Instructions & Coding Guidelines**

### **Core Principles**

1. **Determinism is Paramount**
   - No direct wall-clock time, random seeds, or unabstracted I/O in core logic
   - Use traits and dependency injection for non-deterministic operations
   - CCL execution must be completely deterministic across all nodes

2. **Strict Modularity**
   - Never create cross-crate logic except through defined interfaces
   - Each crate must build and test independently (`cargo test -p <crate>`)
   - CCL contracts interact only through well-defined Host ABI

3. **Security by Design**
   - All actions must be attributable to a DID
   - All economic transactions must be mana-enforced
   - All network messages must be signed and verified
   - CCL contracts execute in sandboxed environment with resource limits

4. **Cooperative Values Integration**
   - Design for mutual aid, not extraction
   - Prioritize collective benefit over individual optimization
   - Ensure governance mechanisms remain participatory and accountable

### **Development Standards**

5. **Testing and Validation**
   ```bash
   # All PRs must pass:
   cargo test --all-features --workspace
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   
   # CCL-specific testing:
   cargo test -p icn-ccl
   just test-ccl-contracts
   just test-covm-execution
   ```

6. **Documentation Standards**
   - Every public type/function documented with Rustdoc (`///`)
   - CCL language features documented with examples
   - CoVM Host ABI functions documented for contract developers
   - Update `.mdc` files for architectural changes

7. **Terminology (Critical for Consistency)**
   - **Always use:** "ICN" (never "ICN Network"), "mesh job," "execution receipt," "mana," "DAG," "CCL policy/contract," "CoVM"
   - **Never use:** "blockchain," "smart contract," "gas," "cryptocurrency"

### **Change Management**

8. **PR/Commit Standards**
   - Title format: `[component] <short description>`
   - Describe affected modules and note breaking changes
   - Link related issues, RFCs, and governance proposals
   - For CCL changes, include example contracts showing usage

9. **Cross-Component Changes**
   - Follow dependency flow: `icn-common` → domain crates → `icn-runtime` → `icn-api`
   - Update API contracts in `icn-api` first for external interfaces
   - Maintain backward compatibility unless explicitly versioned
   - CCL language changes require community discussion and governance approval

---

## 🛡 **Security & Safety Guidelines**

### **Critical Security Invariants**
- Never introduce dependencies with unpatched vulnerabilities
- Never hardcode cryptographic keys or secrets
- All economic, governance, and identity logic requires adversarial testing
- CCL contracts cannot break mana conservation or security boundaries
- CoVM execution must be resource-bounded and deterministic

### **Common Security Patterns**
- **Input Validation:** Structured validation for all external data (DIDs, job specs, CCL source)
- **Error Handling:** Specific error types with proper context
- **Resource Limiting:** Rate limiting, circuit breakers, execution timeouts
- **Cryptographic Hygiene:** Proper signature verification, fresh nonces, secure key derivation

### **CCL Security Considerations**
- **Sandbox Enforcement:** CCL contracts cannot access host system directly
- **Resource Bounds:** CPU, memory, and execution time limits
- **Capability-Based Security:** Contracts can only access explicitly granted capabilities
- **Deterministic Execution:** No sources of non-determinism in contract execution

---

## 🧩 **Example Agent Tasks**

### **Core Development Tasks**
- **Mesh Computing:** Enhance job assignment algorithms in `icn-runtime` and `icn-mesh`
- **Economics:** Improve mana regeneration policies in `icn-economics`
- **Governance:** Add new proposal types and voting mechanisms in `icn-governance`
- **Identity:** Enhance DID management and credential verification in `icn-identity`
- **Storage:** Optimize DAG storage and retrieval in `icn-dag`

### **CCL & CoVM Tasks**
- **Language Features:** Add new CCL syntax for common governance patterns
- **Compiler Optimization:** Improve CCL-to-WASM compilation performance
- **Standard Library:** Create reusable CCL templates for cooperative governance
- **Host ABI:** Extend CoVM Host ABI with new ICN primitives
- **Testing:** Add property-based testing for CCL contract execution

### **Infrastructure & Tooling Tasks**
- **CI/CD Enhancement:** Add CCL contract testing to the pipeline
- **Performance:** Add benchmarking for mesh job execution and CCL compilation
- **Developer Experience:** Create better debugging tools for CCL development
- **Security:** Add automated security scanning for CCL contracts
- **Documentation:** Generate comprehensive API docs and CCL language reference

### **Quality & Maintenance Tasks**
- **Monitoring:** Add health checks for CoVM execution and mesh job processing
- **Error Recovery:** Implement graceful degradation for network partitions
- **Testing:** Expand property-based testing for economic invariants
- **Performance:** Optimize hot paths in mesh job assignment and mana calculations

---

## 📖 **Learning Path & Resources**

### **Essential Reading Order**
1. **Global Context:** `.cursor/rules/cursor-rules.mdc` (ICN philosophy and boundaries)
2. **Local Context:** `.cursor/rules/icn-core-context.mdc` (this repo's architecture)
3. **System Design:** `.cursor/rules/crate-architecture.mdc` (detailed component breakdown)
4. **Development:** `.cursor/rules/development-workflow.mdc` (processes and standards)
5. **Security:** `.cursor/rules/security-validation.mdc` (security patterns)

### **Code Exploration Path**
1. **Start Here:** `crates/icn-common/src/lib.rs` (shared types and utilities)
2. **Core Runtime:** `crates/icn-runtime/src/abi.rs` (Host ABI surface)
3. **Job Pipeline:** `crates/icn-mesh/src/lib.rs` (mesh computing logic)
4. **Economics:** `crates/icn-economics/src/lib.rs` (mana management)
5. **Governance:** `crates/icn-governance/src/lib.rs` (proposal and voting)
6. **CCL System:** `crates/icn-ccl/` or `/ccl/` (language and compiler)

### **Testing Strategy**
- **Unit Tests:** Test individual functions and components
- **Integration Tests:** Test cross-crate interactions and workflows
- **CCL Contract Tests:** Test governance policies and cooperative bylaws
- **Property Tests:** Test economic invariants and security properties
- **End-to-End Tests:** Test complete mesh job and governance workflows

---

## 🚀 **Quick Start for New Contributors**

### **Setup & Validation**
```bash
# 1. Read foundational context files (especially cursor-rules.mdc and icn-core-context.mdc)
# 2. Set up development environment
just setup                    # Install dependencies and hooks
just validate                 # Run full validation suite

# 3. Explore the codebase
cargo doc --open              # Generate and browse API documentation
just test                     # Run all tests
just test-ccl                 # Test CCL compiler and contracts (if available)
```

### **First Contribution Guidelines**
1. **Start Small:** Begin with focused changes that clearly advance project goals
2. **Follow Patterns:** Study existing code patterns before introducing new approaches
3. **Test Thoroughly:** Add comprehensive tests for any changes
4. **Document Well:** Update docs and examples for any public API changes
5. **Seek Review:** Don't hesitate to ask for guidance on architectural decisions

### **Common Pitfalls to Avoid**
- Don't introduce non-determinism in core logic
- Don't bypass mana enforcement or security checks
- Don't create direct dependencies between domain crates
- Don't use blockchain/crypto terminology instead of ICN terms
- Don't modify workspace-level configs without understanding impact

---

## 🌟 **Vision Reminder**

You're building more than software—you're creating infrastructure for **economic democracy**. Every function you write, every test you add, every security check you implement helps communities around the world coordinate without extraction, govern without centralization, and create value that stays with the people who generate it.

The InterCooperative Network isn't just technically different from existing systems—it's **politically different**. It's designed to serve cooperatives, not corporations. To build wealth for communities, not extract it. To enable self-governance, not impose control.

**Every contribution moves us toward a more equitable digital economy.**

---

**Thank you for helping build the foundation of cooperative digital infrastructure. Together, we're creating the tools that communities need to thrive in the digital age.**
