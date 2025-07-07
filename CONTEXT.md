# ICN Core – Context Overview

## Purpose
`icn-core` is the authoritative Rust workspace for the InterCooperative Network (ICN). It provides deterministic libraries for the federated infrastructure stack that supports cooperative, post‑capitalist coordination and enables autonomous federated systems without relying on traditional state or corporate structures.

## ICN Mission & Philosophy

**Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.**

### Core Philosophical Principles
- **Anti-Capitalist Design**: Every choice prioritizes collective benefit over extraction and individual optimization
- **Nonviolent Infrastructure**: Replace systemic violence with cooperative coordination mechanisms
- **Revolutionary Pluralism**: Enable local autonomy within networked solidarity
- **Memetic Security**: Resistance to capture and cooptation through humorous and viral tactics
- **Regenerative Systems**: Ecological and social regeneration patterns embedded in design
- **Dignity & Autonomy**: Technology that enhances human agency rather than controlling it

### Strategic Vision
- **Systemic Sovereignty**: Fully autonomous federated systems independent of nation-state control
- **Consciousness Architecture**: Programmable layers of collective awareness and decision-making
- **Post-Capitalist Coordination**: Tools for economic organization beyond market mechanisms
- **Collective Liberation**: Technology infrastructure for universal human flourishing

## Architectural Principles

### Technical Foundation
- **Strict Modularity**: The workspace is organized into discrete crates, each with a clear responsibility. Modules minimize direct dependencies between domains.
- **Error-First Programming**: All crates return `Result<T, CommonError>` and avoid panics in library code. Error variants provide contextual messages for reliable handling.
- **Deterministic Execution**: All core logic must be predictable and verifiable across nodes to ensure consensus and trust.
- **WASM-First Contracts**: Cooperative Contract Language (CCL) compiles to WASM for deterministic, sandboxed policy execution.

### Identity & Federation
- **Scoped Federation**: Nodes interact via identity-scoped federation protocols, using DIDs to define trust boundaries and access control.
- **Identity-Driven Design**: `icn-identity` manages DIDs, verifiable credentials, and signing utilities so every action is attributable.
- **Three-Tier Topology**: Cooperatives (economic units) → Communities (civic/social) → Federations (coordination layer).
- **Local Autonomy**: Each organization defines governance and economics via CCL while federating through shared protocols.

### Governance & Economics
- **Governance as Code**: All bylaws, voting mechanisms, and policies encoded in CCL and executed deterministically.
- **Purpose-Bound Tokens**: All tokens are scoped to specific capabilities (e.g., `icn:resource/compute`) and cannot be abstracted or speculated on.
- **Anti-Speculation Design**: Economic mechanisms focused on actual resource coordination, not financial extraction.
- **Mana System**: Regenerating, non-speculative resource tokens for compute rights and network participation.

### Storage & Execution
- **Runtime-Based Execution**: The `icn-runtime` crate hosts WASM contracts and orchestrates mesh jobs through a host ABI. Deterministic execution ensures verifiable receipts.
- **DAG Ground Truth**: `icn-dag` anchors execution receipts and stores state in a content-addressed DAG, providing tamper-evident history.
- **Cryptographic Auditability**: Every significant action emits signed execution receipts stored in the DAG for complete transparency.

## Crate Responsibilities

### Core Infrastructure
- **`icn-common`** – shared types, error handling, cryptographic primitives, and constants.
- **`icn-runtime`** – host runtime, WASM execution environment, and job orchestration.
- **`icn-api`** – shared API traits and DTOs for node communication and external interfaces.

### Identity & Security
- **`icn-identity`** – decentralized identity management, verifiable credentials, and cryptographic operations.
- **`icn-dag`** – DAG primitives, content addressing, storage interfaces, and receipt anchoring.

### Governance & Economics
- **`icn-governance`** – proposal engine, voting mechanisms, and policy execution.
- **`icn-economics`** – mana accounting, scoped token management, and economic policy enforcement.
- **`icn-reputation`** – reputation scoring, contribution tracking, and trust metrics.

### Networking & Computation
- **`icn-network`** – P2P networking abstractions with libp2p support and peer discovery.
- **`icn-mesh`** – distributed job definition, bidding, execution, and load balancing.

### Language & Compilation
- **`icn-ccl`** – Cooperative Contract Language compiler, optimizer, and WASM code generation.
- **`icn-protocol`** – message formats, protocol definitions, and serialization standards.

### User Interfaces
- **`icn-cli`** – command-line interface for developers and administrators.
- **`icn-node`** – main node binary with HTTP server for API access.

## Core System Patterns

### Governance & Decision-Making
- **Proposal Lifecycle**: Draft → Deliberation → Vote → Execution → DAG anchoring
- **Liquid Delegation**: Delegated voting with revocable trust relationships
- **Multi-Stage Governance**: Complex proposal flows with amendment and refinement stages
- **Cryptographic Audit Trail**: Tamper-evident history of all decisions and changes

### Economic Coordination
- **Scoped Token Operations**: All economic actions tied to specific capabilities and identities
- **Reputation-Influenced Economics**: Economic flows and access shaped by contribution history
- **Federated Trust Markets**: Cross-cooperative resource sharing and exchange
- **Anti-Extraction Mechanisms**: Economic design prevents speculation and value extraction

### Technical Operations
- **DAG Anchoring**: All significant actions emit signed execution receipts stored in the DAG for auditability
- **Scoped Operations**: Every operation tied to a DID and governed by explicit policy. No unscoped actions or hardcoded IDs
- **Decentralized Networking**: libp2p-based communication enables peer discovery and federation synchronization
- **Mesh Computing**: Distributed WASM job execution with cryptographic proof of completion

### Security & Privacy
- **End-to-End Cryptography**: All data encrypted in transit and at rest
- **Zero-Knowledge Proofs**: Anonymous voting and selective credential disclosure
- **Quantum-Resistant Cryptography**: Future-proof security against quantum computing threats
- **Privacy-Preserving Architecture**: Minimal data collection with user-controlled disclosure

## Development & Governance Rules

### Technical Standards
- Use canonical data types from `icn-common` and API contracts from `icn-api`.
- Maintain deterministic logic; avoid wall-clock time or unseeded randomness in core paths.
- No hardcoded identifiers or manual cross-crate coupling.
- Contributions must include comprehensive tests and Rustdoc for public APIs.
- Follow the repository guidelines in `.cursor/rules` and run `just validate` before committing.

### Cooperative Values Integration
- **Design for Mutual Aid**: Prioritize collective benefit over individual optimization
- **Ensure Participatory Governance**: Governance mechanisms remain accessible and democratic
- **Prevent Centralization**: Avoid single points of failure or control
- **Support Local Autonomy**: Enable communities to govern themselves within federation protocols

### Security & Safety
- Never introduce dependencies with unpatched vulnerabilities
- All economic, governance, and identity logic requires adversarial testing
- CCL contracts cannot break mana conservation or security boundaries
- CoVM execution must be resource-bounded and deterministic

## Developer Experience & Community

### Development Infrastructure
- See `docs/ONBOARDING.md` for setup instructions and comprehensive walkthroughs
- The `justfile` provides common tasks (`just build`, `just test`, `just devnet`)
- Containerized devnet (`icn-devnet`) demonstrates multi-node federation with HTTP APIs
- Rich error messages, observability hooks, and CLI/HTTP tools enable rapid debugging

### Documentation & Learning
- **Comprehensive Feature Overview**: `docs/ICN_FEATURE_OVERVIEW.md` covers all current and planned features
- **API Documentation**: Auto-generated and manual documentation for all interfaces
- **Governance Examples**: CCL templates and policy patterns for common cooperative needs
- **Academic Research**: Papers, case studies, and theoretical foundations

### Community & Contribution
- **Global Community**: Developers, cooperatives, researchers committed to human flourishing
- **Multiple Contribution Types**: Code, governance policies, research, community organizing
- **Inclusive Participation**: Multiple skill levels and backgrounds welcomed
- **Mentorship Programs**: Support for new contributors and cooperative adopters

---

## Next Steps for Contributors

1. **Read the Complete Vision**: Review `docs/ICN_FEATURE_OVERVIEW.md` for comprehensive understanding
2. **Understand Current State**: Check implementation status and ongoing development priorities
3. **Choose Your Focus**: Pick areas aligned with your skills and interests
4. **Join the Community**: Participate in discussions, ask questions, share ideas
5. **Start Contributing**: Begin with documentation, tests, or small features
6. **Think Systemically**: Consider how your contributions support the broader vision of cooperative digital civilization

ICN is more than a technical project—it's a movement toward systemic transformation. Every line of code, every governance policy, and every community interaction contributes to building infrastructure for a more just and sustainable world.

