# Common development commands

# Set up the development environment
setup:
    rustup component add clippy rustfmt
    rustup target add wasm32-unknown-unknown
    cargo install just --locked 2>/dev/null || echo "just already installed"
    cargo install pre-commit --locked 2>/dev/null || echo "pre-commit already installed" 
    pre-commit install 2>/dev/null || echo "pre-commit hooks already installed"
    @echo "✅ Development environment setup complete!"

# Format the code using cargo fmt
format:
    cargo fmt --all -- --check

# Run clippy linting
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the full test suite with default features (stable)
# This uses the default production-ready features (Sled storage)
test:
    cargo test --workspace

# Run tests with all features enabled (including RocksDB)
# This mirrors CI but requires stable C++ compiler
test-all-features:
    cargo test --all-features --workspace

# Run tests with specific storage backend
test-sled:
    cargo test --workspace --features persist-sled

# Run tests with RocksDB (requires Clang)
test-rocksdb:
    cargo test --workspace --features persist-rocksdb

# Run quick tests without persistence features
test-quick:
    cargo test --workspace --no-default-features

# Build with default features (stable, production-ready)
build:
    cargo build --workspace

# Build with all features enabled (including RocksDB)
build-all-features:
    cargo build --all-features --workspace

# Build with specific storage backend
build-sled:
    cargo build --workspace --features persist-sled

# Build with RocksDB (requires Clang)
build-rocksdb:
    cargo build --workspace --features persist-rocksdb

# Launch the containerized federation devnet
# Requires Docker and docker-compose

devnet:
    ./scripts/build-devnet.sh
    cd icn-devnet && ./launch_federation.sh

# Alias for devnet (for consistency with quickstart docs)
run-devnet:
    just devnet

# Run the complete validation suite (format, lint, test)
validate:
    just format && just lint && just test

# Run full validation with all features (for CI)
validate-all:
    just format && just lint && just test-all-features

# Run benchmarks for all crates
bench:
    cargo bench --workspace --all-features

# Run zero-knowledge circuit benchmarks
bench-zk:
    cargo bench -p icn-zk

# Run federation health checks
health-check:
    cargo test --test federation -- --exact test_federation_node_health --nocapture

# Show node status via CLI
status:
    cargo run -p icn-cli -- status

# View recent devnet logs (if running via Docker)
logs:
    cd icn-devnet && docker-compose logs --tail=50

# Fetch Prometheus metrics from the default node
metrics:
    cargo run -p icn-cli -- metrics

# Build documentation for all crates
docs:
    cargo doc --workspace --no-deps

# Frontend Development Commands
# Requires Node.js 18+ and pnpm

# Set up frontend development environment
setup-frontend:
    @echo "Setting up frontend development environment..."
    @if ! command -v node >/dev/null 2>&1; then echo "❌ Node.js not found. Please install Node.js 18+"; exit 1; fi
    @if ! command -v pnpm >/dev/null 2>&1; then echo "❌ pnpm not found. Installing pnpm..."; npm install -g pnpm; fi
    pnpm install
    @echo "✅ Frontend development environment setup complete!"

# Install all frontend dependencies
install-frontend:
    pnpm install

# Start all frontend apps in development mode
dev-frontend:
    pnpm dev

# Start specific frontend app
dev-wallet:
    pnpm dev:wallet

dev-agoranet:
    pnpm dev:agoranet

dev-web-ui:
    pnpm dev:web-ui

dev-explorer:
    pnpm dev:explorer

# Build all frontend apps for production
build-frontend:
    pnpm build

# Build specific frontend app
build-wallet:
    pnpm build:wallet

build-agoranet:
    pnpm build:agoranet

build-web-ui:
    pnpm build:web-ui

build-explorer:
    pnpm build:explorer

# Test frontend applications
test-frontend:
    pnpm test

# Lint frontend code
lint-frontend:
    pnpm lint

# Format frontend code
format-frontend:
    pnpm format

# Type check frontend code
type-check-frontend:
    pnpm type-check

# Clean frontend build artifacts
clean-frontend:
    pnpm clean

# Mobile development commands
dev-mobile:
    pnpm mobile:dev

build-mobile:
    pnpm mobile:build

# Desktop development commands (Tauri)
dev-desktop:
    pnpm tauri:dev

build-desktop:
    pnpm tauri:build

# Complete development setup (Rust + Frontend)
setup-all:
    just setup
    just setup-frontend

# Complete validation (Rust + Frontend)
validate-all-stack:
    just validate-all
    just lint-frontend
    just type-check-frontend
    just test-frontend

# Build everything (Rust + Frontend)
build-all-stack:
    just build-all-features
    just build-frontend

# CCL Developer Tooling Commands

# Start CCL Language Server for IDE integration
ccl-lsp:
    cargo run -p icn-ccl --bin ccl-lsp

# Create a new CCL package
ccl-init name author:
    @echo "Creating new CCL package: {{name}}"
    cargo run -p icn-ccl --bin ccl-cli -- package init {{name}} --author "{{author}}"

# Install CCL package dependencies  
ccl-install:
    @echo "Installing CCL dependencies..."
    cargo run -p icn-ccl --bin ccl-cli -- package install

# Add a CCL dependency
ccl-add-dep name version:
    @echo "Adding CCL dependency: {{name}} = {{version}}"
    cargo run -p icn-ccl --bin ccl-cli -- package add {{name}} {{version}}

# Compile CCL contract with debug info
ccl-compile-debug file:
    @echo "Compiling CCL contract with debug info: {{file}}"
    cargo run -p icn-ccl --bin ccl-cli -- compile {{file}} --debug

# Start CCL debugger
ccl-debug file:
    @echo "Starting CCL debugger for: {{file}}"
    cargo run -p icn-ccl --bin ccl-cli -- debug {{file}}

# Format CCL files
ccl-format:
    @echo "Formatting CCL files..."
    find . -name "*.ccl" -exec cargo run -p icn-ccl --bin ccl-cli -- format {} \;

# Lint CCL files
ccl-lint:
    @echo "Linting CCL files..."
    find . -name "*.ccl" -exec cargo run -p icn-ccl --bin ccl-cli -- lint {} \;

# Run CCL contract tests
ccl-test:
    @echo "Running CCL contract tests..."
    cargo test -p icn-ccl

# Install CCL tooling globally
install-ccl-tools:
    cargo install --path icn-ccl --bin ccl-lsp
    cargo install --path icn-ccl --bin ccl-cli
    @echo "✅ CCL tools installed globally"

# Frontend-Backend Integration Testing

# Validate frontend-backend integration with offline tests
validate-integration-offline:
    @echo "🧪 Running frontend-backend integration validation (offline mode)..."
    node scripts/validate-frontend-integration.js --offline

# Validate frontend-backend integration with live backend  
validate-integration:
    @echo "🧪 Running frontend-backend integration validation..."
    node scripts/validate-frontend-integration.js

# Run TypeScript SDK integration tests
test-sdk-integration:
    @echo "🧪 Running TypeScript SDK integration tests..."
    cd packages/ts-sdk && npm run test:integration:offline
