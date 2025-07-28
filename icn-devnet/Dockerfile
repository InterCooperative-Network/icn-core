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
# 64 MiB as suggested by rustc error message (67108864 bytes)
ENV RUST_MIN_STACK=16777216

# Additional memory optimizations for the build
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=300

# Conservative compilation flags to avoid compiler crashes
ENV RUSTFLAGS="-C debuginfo=1 -C lto=off"
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=8

# Limit concurrent compilation jobs to reduce memory pressure
ENV CARGO_BUILD_JOBS=2
ENV CARGO_INCREMENTAL=0

# Reduce optimization for C dependencies
ENV CFLAGS="-O1"
ENV CXXFLAGS="-O1"

# Build with increased stack size and memory limits
# Use release mode with reduced debug info to avoid LLVM issues
RUN bash -c "cargo build --release -p icn-node --features with-libp2p"
RUN bash -c "cargo build --release -p icn-cli"

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user and directories  
RUN useradd -r -s /bin/false -d /app icn
RUN mkdir -p /app/data /app/icn_data /app/.icn /app/config /app/.local /app/.cache && \
    chown -R icn:icn /app && \
    chmod -R 755 /app

# Copy binaries from builder stage (built in release mode)
COPY --from=builder /app/target/release/icn-node /usr/local/bin/icn-node
COPY --from=builder /app/target/release/icn-cli /usr/local/bin/icn-cli

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
