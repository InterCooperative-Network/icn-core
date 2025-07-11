# Contributor Setup

This short guide covers the basic environment configuration needed to work on `icn-core`.

1. **Install Rust and tools**
   ```bash
   rustup toolchain install stable
   rustup override set stable
   rustup component add clippy rustfmt
   rustup target add wasm32-unknown-unknown
   cargo install wasm-tools --locked
   ```
2. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd icn-core
   ```
3. **Run initial checks**
   ```bash
   cargo build
   cargo test --all-features --workspace
   ```
4. **Install optional helpers**
   ```bash
   cargo install just
   just --list
   ```
5. **Set up pre-commit hooks**
   ```bash
   pre-commit install
   ```
6. **VS Code Extension (optional)**
   ```bash
   cd vscode-extension && npm install
   ```
   Launch the extension with `F5` to get CCL syntax highlighting and a compile command.
