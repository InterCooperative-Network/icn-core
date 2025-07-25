# ICN Core Workspace – Context & Rules of Engagement

> **Attention Contributor (Human or AI):** This file defines the specific rules and architectural context for the `icn-core` repository. It complements the global ICN Shared Contributor Rules. Familiarize yourself with both. CI will enforce these guidelines.

---

## 1 · Mission & Current State of `icn-core`

This repository, `icn-core`, is the **advanced development Rust monorepo for the InterCooperative Network's infrastructure**. It houses **substantial working implementations** of deterministic, runtime-critical libraries executed by every ICN node, along with comprehensive frontend applications and developer tools.

**Current Status: Advanced Development (NOT Production Ready)**
- **CCL WASM compilation and execution** working end-to-end ✅
- **Multi-backend persistence** (PostgreSQL, RocksDB, SQLite, Sled) ✅  
- **Governance workflows** with proposals, voting, budget allocation ✅
- **P2P networking** with libp2p, gossipsub, Kademlia DHT ✅
- **Economic systems** with mana accounting and resource tokens ✅
- **Identity management** with DID-based authentication ✅
- **Job execution pipeline** with mesh computing ✅
- **Frontend applications** connecting to real APIs ✅

**Key Responsibilities of `icn-core`:**

### **Core Infrastructure (~75% Implementation)**
- **`icn-runtime`** – Node orchestration, WASM execution, job management (working implementations with optimization needs)
- **`icn-common`** – Shared types, cryptographic primitives, utilities (solid foundation)
- **`icn-api`** – HTTP API definitions with real endpoint implementations (some still return development data)
- **`icn-protocol`** – Message formats and protocol definitions (working P2P messaging)

### **Identity & Security (~60% Implementation)**
- **`icn-identity`** – DID management, credential verification (working but needs security review)
- **`icn-dag`** – Content-addressed storage with multiple working backends
- **`icn-zk`** – Zero-knowledge circuits (framework exists, needs expansion)

### **Governance & Economics (~65% Implementation)**  
- **`icn-governance`** – Proposal engine, ranked choice voting, federation governance (substantial working features)
- **`icn-economics`** – Multi-backend mana ledgers, resource tokens, mutual credit (working transaction logic)  
- **`icn-reputation`** – Trust scoring framework (basic implementation)
- **`icn-eventstore`** – Event sourcing utilities (working persistence)

### **Networking & Computation (~65% Implementation)**
- **`icn-network`** – P2P networking with working libp2p integration
- **`icn-mesh`** – Job scheduling, bidding, execution pipeline (end-to-end functionality working)

### **Language & Tools (~70% Implementation)**
- **`icn-ccl`** – Cooperative Contract Language compiler with WASM backend (substantial implementation)
- **`icn-cli`** – Command-line interface (most commands functional)
- **`icn-node`** – Main daemon binary with HTTP server (working development server)
- **`icn-sdk`** – High-level Rust SDK (connects to real APIs)
- **`icn-templates`** – Template management (basic functionality)
- **`job-audit`** – Job auditing (early implementation)

### **Frontend Applications (~60% Implementation)**
- **`apps/web-ui`** – Federation dashboard (UI working, backend integration partial)
- **`apps/explorer`** – DAG viewer (visualization working, needs real-time data)
- **`apps/wallet-ui`** – Identity management (UI framework, backend integration limited)
- **`apps/agoranet`** – Governance interface (UI working, governance backend functional) 