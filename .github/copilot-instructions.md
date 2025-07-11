# 🧠 ICN Core Copilot Instructions

Welcome to the InterCooperative Network (ICN) Core repository — the heart of a decentralized, non-capitalist infrastructure for federated cooperation.

These instructions tell GitHub Copilot how to understand, navigate, and contribute meaningfully to this monorepo.

---

## 🛠️ Development Context

- **Primary development occurs on the `develop` branch.**
  - Copilot should assume all operations (e.g., issue resolution, code suggestions, PR creation) happen against the `develop` branch unless explicitly told otherwise.
  - All merges to `main` are release-grade and follow strict testing protocols.
  
---

## 🧱 Project Structure & Tech Stack

- This is a **Rust-based monorepo** using a `Cargo.toml` workspace.
- Key directories:
  - `crates/`: Contains modular crates (`icn-common`, `icn-runtime`, `icn-cli`, etc.)
  - `icn-ccl/`: A custom DSL compiler for cooperative governance
  - `icn-devnet/`: Federation simulation powered by `docker-compose`
- Main binary: `icn-node` (in `crates/icn-node`), built via `src/node.rs`.

### 🧰 Toolchain and Standards

- Rust 2021 edition
- Cargo commands should use workspaces (`--workspace`)
- Required tools:
  - `rustfmt`, `clippy`, `cargo test`
  - Docker and Docker Compose for devnet
  - Prometheus/Grafana for observability

---

## ⚙️ Coding Practices

Copilot must follow these **coding principles**:

### ✅ Security & Correctness
- Do **not** use `unwrap()` or `expect()` in production paths.
- Prefer proper `Result<T, E>` returns and custom error enums.
- Use `thiserror` for defining error types.

### ✅ Auditability
- Code must be traceable and testable.
- Functions should be small, modular, and pure when possible.
- Avoid implicit side effects.

### ✅ Documentation & Naming
- All public functions must include doc comments.
- Names should be descriptive, context-aware, and consistent with Rust norms.
- Prefer `snake_case` for variables, `CamelCase` for types/enums, and `ALL_CAPS` for constants.

---

## 🧪 Testing

- Use `#[test]` modules in each crate with descriptive names.
- Include meaningful assertions.
- For DAG or identity-related features, write integration tests where possible.
- Run full test suite: `cargo test --workspace`

---

## 🧬 Cooperative Logic (What This System *Is*)

- This is **not** a blockchain. It uses a **DAG-based content-addressed ledger**.
- Identity is scoped using DIDs (`did:key:...`) and signed using Ed25519.
- Tokens are **non-speculative**, scoped, and tied to cooperative capabilities (e.g., `icn:resource/compute`).
- Governance is programmable via the CCL (Cooperative Contract Language).
- Economic logic avoids staking and speculation in favor of **resource regeneration**, **reputation-based access**, and **participatory control**.

---

## 🧑‍🤝‍🧑 For Pull Requests & Issues

- Copilot should:
  - Default to creating **PRs against `develop`**
  - Include formatting and lint checks
  - Reference related issues using `Fixes #X` or `Closes #Y`
  - Suggest human-readable commit messages

---

## 🧞‍♂️ Behavior Guidelines for Copilot Chat & Agent

- Prioritize clarity over brevity.
- Be precise with Rust idioms and compiler errors.
- When unsure, ask clarifying questions or recommend safe defaults.
- Avoid suggesting outdated syntax or deprecated libraries.

---

## 🤝 Values Embedded in Code

Copilot should align with ICN’s values:

- Cooperative autonomy over coercive hierarchy
- Regenerative systems over extractive economics
- Composability and modularity over complexity and centralization
- Consent-based governance over authoritarian control

---

## 🧩 Suggested Commands & Shortcuts

- `icn-build`: `cargo build --workspace`
- `icn-test`: `cargo test --workspace`
- `icn-fmt`: `cargo fmt --all`
- `icn-clippy`: `cargo clippy --all-targets --all-features`
- `icn-node-build`: `cargo build -p icn-node --features with-libp2p`
- `icn-devnet`: `cd icn-devnet && ./launch_federation.sh`

---

## 🏁 Final Reminders

- Work on `develop` unless otherwise stated.
- Be explicit, cautious, and idiomatic in code suggestions.
- Lean toward safety, transparency, and long-term maintainability.
