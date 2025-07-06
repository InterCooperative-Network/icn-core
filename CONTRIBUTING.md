# Contributing to the InterCooperative Network (ICN) Core

First off, thank you for considering contributing to ICN Core! We welcome contributions from everyone and are excited to see how you can help make this project better.

This document provides a roadmap of good first issues, guidance on setting up your development environment, and how to get started with multi-node testing.

## Getting Started

*   **Project Overview**: Start with the main `README.md` (if available at the root, otherwise check individual crate READMEs) for a general understanding of the project structure and goals.
*   **Onboarding**: Refer to `docs/ONBOARDING.md` (if it exists, otherwise the primary README) for detailed instructions on building, testing, and the general development workflow.
*   **Multi-Node Testing**: Check out the `MULTI_NODE_GUIDE.md` for a comprehensive guide on running local ICN clusters.
*   **Testnet Script**: The `scripts/run_local_testnet.sh` script provides a starting point for launching multiple nodes for testing.

## 🚀 ICN Multi-Node & Contributor Roadmap: "Good First Issues"

Below are actionable, bite-sized issues to drive forward the next major features for the InterCooperative Network. These are ideal entry points for new contributors, and represent real, high-impact steps toward a robust federated protocol.

### 🗄️ Storage (`icn-dag`)

*   **\[Storage] Implement `SqliteDagStore`**
    Create a SQLite-based DAG backend with CRUD, using `rusqlite`.
    *Difficulty: Medium*
*   **\[Storage] Robust CID Validation in `FileDagStore::get`**
    Ensure CIDs match content on retrieval; return validation errors.
    *Difficulty: Easy*
*   **\[Storage] Configurable Directory Sharding**
    Implement sharding of stored files for scalability.
    *Difficulty: Medium*
*   **\[Storage] Block CID Validation on Put**
    Check block CIDs match computed hashes at store time.
    *Difficulty: Medium*

### 🌐 Networking (`icn-network`, `icn-node`)

*   **\[Network] Configurable Listen Address**
    Allow CLI/config selection of libp2p listen address.
    *Difficulty: Medium*
*   **\[Network] Bootstrap Peers via CLI/Config**
    Allow user-defined bootstraps for P2P mesh.
    *Difficulty: Medium*
*   **\[Network] Graceful Swarm Shutdown**
    Implement signal-based or programmatic shutdown for network tasks.
    *Difficulty: Medium*
*   **\[Network] JSON Gossipsub Serialization**
    Encode/decode `ProtocolMessage` via JSON for pub/sub.
    *Difficulty: Easy-Medium*
*   **\[API] List Connected/Known Peers**
    CLI/API to query/display current peers.
    *Difficulty: Medium*

### 👥 Governance & Federation (`icn-governance`, `icn-api`, `icn-node`)

*   **\[Governance] Quorum & Threshold Checks**
    Proposal vote tallying with quorum logic.
    *Difficulty: Medium*
*   **\[Governance] Member List for Voter Validation**
    Only eligible DIDs can vote.
    *Difficulty: Medium*
*   **\[Governance] Proposal Propagation via Network**
    Define & implement proposal propagation over mesh.
    *Difficulty: Medium (initial step)*

### 🛠️ CLI/Node Experience

*   **\[CLI] Improved JSON Parsing for Proposal Submission**
    Better error messages and help for proposal types.
    *Difficulty: Easy*
*   **\[CLI] Display Local Peer ID**
    Command to print node's libp2p Peer ID.
    *Difficulty: Medium*

---

**If you want to take on an issue, please check the project's issue tracker. If these specific issues are not yet created, feel free to open one based on this list, or discuss with a maintainer. Every PR and every discussion helps shape the future of the ICN!**

When issues are created in the tracker, they should ideally be labeled with `good first issue`, `help wanted`, or a component-specific label (e.g. `network`, `storage`, `governance`) to help contributors find tasks that match their interests and skill levels.

## Development Process

1.  **Fork the repository** (if you're an external contributor).
2.  **Target `develop`**: All feature branches and pull requests should target the `develop` branch. The `main` branch is reserved for stable, tagged releases.
3.  **Create a new branch** from `develop` for your feature or bug fix: `git checkout -b feature/my-new-feature develop` or `fix/issue-description develop`.
4.  **Make your changes**. Ensure you add relevant tests and update documentation as needed.
5.  **Run tests**: `cargo test --all` (or per-crate tests).
6.  **Format your code**: `cargo fmt --all`.
7.  **Lint your code**: `cargo clippy --all -- -D warnings` (or per-crate).
8.  **Commit your changes**: Write clear, concise commit messages.
9.  **Push to your branch**.
10. **Open a Pull Request (PR)** against the `develop` branch of the upstream repository.
    *   Provide a clear description of your changes in the PR.
    *   Link to any relevant issues.

## Code of Conduct

Please note that this project is released with a Contributor Code of Conduct. By participating in this project you agree to abide by its terms.

Thank you for your contributions! 