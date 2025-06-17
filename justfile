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
