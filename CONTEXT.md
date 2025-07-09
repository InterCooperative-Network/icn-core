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

### Async I/O Model
 - **Tokio Runtime**: All networking and persistence layers run on Tokio and expose `async` functions.
 - **Async Storage Interfaces**: The `icn-dag` crate's `AsyncStorageService` is the canonical trait for persistence backends.
 - **Async Crates**: `icn-api`, `icn-cli`, `icn-dag`, `icn-network`, `icn-runtime`, `icn-governance`, and `icn-node` provide async APIs.
 - **Remaining Sync Code**: Only `icn-dag` retains `StorageService` for legacy synchronous environments.
 - See [docs/ASYNC_OVERVIEW.md](docs/ASYNC_OVERVIEW.md) for more detail.

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

## Cooperative Infrastructure Vision

### ICN as Digital Commons for Cooperatives

ICN is designed from the ground up to serve the needs of cooperatives, mutual aid networks, and solidarity economy organizations. The technical architecture explicitly supports cooperative values and organizational structures:

#### **Comprehensive Cooperative Support**
- **Worker Cooperatives**: Profit sharing algorithms, democratic workplace tools, labor coordination systems
- **Consumer Cooperatives**: Patronage dividends, bulk purchasing coordination, member benefits management
- **Housing Cooperatives**: Maintenance coordination, occupancy planning, housing justice advocacy
- **Multi-Stakeholder Cooperatives**: Complex governance structures with multiple membership classes
- **Cooperative Federations**: Cross-cooperative resource sharing and coordinated action

#### **Solidarity Economy Integration**
- **Mutual Aid Networks**: Emergency response coordination, resource sharing, community support matching
- **Time Banking**: Hour-based service exchange systems integrated with mesh computing
- **Local Currencies**: Community-controlled purpose-bound currencies for local economic development
- **Gift Economies**: Non-market resource distribution mechanisms
- **Commons Management**: Shared resource governance and stewardship tools

#### **Economic Justice Tools**
- **Mutual Credit Systems**: Peer-to-peer lending without traditional banking intermediaries
- **Democratic Finance**: Cooperative loan management with member vote-based approval
- **Anti-Speculation Design**: Economic mechanisms that prevent financialization and extraction
- **Wealth Redistribution**: Progressive economic policies encoded in governance contracts
- **Community Ownership**: Tools for collective ownership of productive assets

#### **Democratic Participation Infrastructure**
- **Liquid Democracy**: Delegated voting with revocable trust for scalable participation
- **Consensus Building**: Tools for achieving agreement beyond simple majority voting
- **Participatory Budgeting**: Multi-round democratic resource allocation processes
- **Citizen Assemblies**: Randomly selected representative decision-making bodies
- **Inclusive Facilitation**: Accessibility tools and equity mechanisms for participation

#### **Transformative Justice Systems**
- **Conflict Resolution**: Structured mediation workflows for interpersonal and organizational conflicts
- **Restorative Justice**: Community-based accountability processes that center healing
- **Community Healing**: Collective trauma processing and recovery tools
- **Alternative Accountability**: Justice processes that don't rely on punishment or exclusion

#### **Climate & Environmental Action**
- **Carbon Credit Trading**: Environmental impact tracking and offset exchange systems
- **Renewable Energy Sharing**: Community energy grid coordination and distribution
- **Sustainability Metrics**: Comprehensive environmental impact measurement and reporting
- **Ecological Regeneration**: Tools for coordinating ecosystem restoration and protection

### Implementation Roadmap for Cooperative Features

The cooperative infrastructure will be implemented through a phased approach that builds on ICN's existing foundation:

1. **Phase 1 (Q1-Q2 2027)**: Cooperative Banking MVP - mutual credit, time banking, local currencies, democratic loans
2. **Phase 2 (Q3-Q4 2027)**: Mutual Aid & Emergency Response - resource sharing, community support, skill matching
3. **Phase 3 (Q1-Q2 2028)**: Worker & Consumer Cooperative Tools - profit sharing, patronage, workplace democracy
4. **Phase 4 (Q3-Q4 2028)**: Specialized Domain Systems - supply chain, housing, education, climate action
5. **Phase 5 (Q1-Q2 2029)**: Advanced Democracy & Justice - transformative justice, participatory governance

Each phase builds functional modules that provide immediate value to cooperatives while laying groundwork for more advanced features. See [docs/COOPERATIVE_ROADMAP.md](docs/COOPERATIVE_ROADMAP.md) for detailed implementation plans.

### Cooperative-First Development

All ICN development considers cooperative needs as primary use cases:
- **Feature Design**: Every feature evaluated for cooperative applicability and value
- **API Development**: External interfaces designed for cooperative management dashboards and tools
- **Governance Templates**: CCL contracts include templates for different cooperative structures
- **Community Engagement**: Regular feedback loops with cooperative representatives and pilot programs

---

## Next Steps for Contributors

1. **Read the Complete Vision**: Review `docs/ICN_FEATURE_OVERVIEW.md` for comprehensive understanding
2. **Understand Current State**: Check implementation status and ongoing development priorities
3. **Choose Your Focus**: Pick areas aligned with your skills and interests
4. **Join the Community**: Participate in discussions, ask questions, share ideas
5. **Start Contributing**: Begin with documentation, tests, or small features
6. **Think Systemically**: Consider how your contributions support the broader vision of cooperative digital civilization

ICN is more than a technical project—it's a movement toward systemic transformation. Every line of code, every governance policy, and every community interaction contributes to building infrastructure for a more just and sustainable world.

