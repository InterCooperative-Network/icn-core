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
