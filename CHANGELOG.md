# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Summary
The develop branch has been merged into `main`, launching the **v0.2.0 beta**
cycle. This milestone focuses on stabilizing production deployments and lands
several major components: reputation-based executor selection, cross-node job
execution with signed receipts, an HTTP gateway, a containerized federation
devnet, and expanded security guidance.

### Added
- Initial project setup.
- Basic scaffolding for all core crates.
- CI setup with format, clippy, test, and doc checks.
- Repository hygiene files (LICENSE, CODE_OF_CONDUCT.md, CONTRIBUTING.md, SECURITY.md, .editorconfig).
- Workspace consistency for Cargo.toml files.
- Optional improvements: rust-toolchain.toml, dependabot.yml, issue templates, CHANGELOG.md.
- Kademlia DHT record storage and peer discovery behind `experimental-libp2p`.
- New scoring algorithm in `icn-mesh` with reputation-based `select_executor`.
- Introduced `icn-reputation` crate providing `ReputationStore` trait and in-memory implementation.
- Multi-node CLI with libp2p networking and bootstrap peer discovery.
- Cross-node mesh job execution pipeline with signed receipts anchored to the DAG.
- HTTP gateway enabling REST job submission and status queries.
- Containerized 3-node federation devnet with Docker and integration tests.
- `Ed25519Signer` replaces `StubSigner` for production runtime; tests may continue using `StubSigner`.
- Comprehensive production security guide covering Ed25519 migration, TLS, and authentication.
- Enhanced API documentation with all current endpoints, authentication, and TLS features.
- Updated feature overview reflecting production-ready status and completed Phase 5 milestones.
- Federation management CLI commands documentation and examples.
- Production deployment architecture documentation with security best practices.

### Changed
- Updated main README.md to reflect production-ready status with Phase 5 completion.
- Revised onboarding guide with comprehensive federation management instructions.
- Enhanced API documentation to include TLS, authentication, and all current endpoints.
- Updated feature overview moving many items from "In Development" to "Implemented" status.
- Improved quick start examples with production configuration options.

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0] - YYYY-MM-DD

### Added
- First version of crate skeletons.

[Unreleased]: https://github.com/InterCooperative/icn-core/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/InterCooperative/icn-core/releases/tag/v0.2.0
[0.1.0]: https://github.com/InterCooperative/icn-core/releases/tag/v0.1.0
