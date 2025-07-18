# ICN Federation Devnet Dockerfile
# Multi-stage build for efficient container size

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    clang \
    libclang-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Install rustfmt component
RUN rustup component add rustfmt

# Copy source code
COPY crates/ ./crates/
COPY icn-ccl/ ./icn-ccl/
COPY tests/ ./tests/

# Build the ICN node binary
# Increase rustc stack size to avoid segfaults during optimized builds
# 16 MiB should be sufficient (as suggested by rustc error)
ENV RUST_MIN_STACK=16777216

# Additional memory optimizations for the build
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=300

# Reduce debug info complexity to avoid LLVM SIGSEGV
ENV RUSTFLAGS="-C debuginfo=1 -C opt-level=1"
ENV CARGO_PROFILE_DEV_DEBUG=1
ENV CARGO_PROFILE_RELEASE_DEBUG=1

# Reduce optimization for C dependencies to avoid GCC crashes
ENV CFLAGS="-O1"
ENV CXXFLAGS="-O1"

# Build with increased stack size and memory limits
# Use release mode with reduced debug info to avoid LLVM issues
RUN bash -c "cargo build --release -p icn-node --features with-libp2p"

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -d /app icn
RUN mkdir -p /app/data && chown icn:icn /app/data

# Copy binary from builder stage (built in release mode)
COPY --from=builder /app/target/release/icn-node /usr/local/bin/icn-node

# Copy entrypoint script
COPY icn-devnet/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Set working directory and user
WORKDIR /app
USER icn

# Expose ports
EXPOSE 7845 4001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:7845/info || exit 1

# Use entrypoint script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
