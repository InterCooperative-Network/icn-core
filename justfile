# Common development commands

# Format the code using cargo fmt
format:
    cargo fmt --all -- --check

# Run clippy linting
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the full test suite
# This mirrors CI by testing all crates with all features enabled
# and executing integration tests for the CLI crate

test:
    cargo test --all-features --workspace

# Build all crates with all features
build:
    cargo build --all-features --workspace

# Launch the containerized federation devnet
# Requires Docker and docker-compose

devnet:
    cd icn-devnet && ./launch_federation.sh

# Run the complete validation suite (format, lint, test)
validate:
    just format && just lint && just test

# Run benchmarks for all crates
bench:
    cargo bench --all-features --workspace

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
