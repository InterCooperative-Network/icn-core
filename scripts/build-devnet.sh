#!/bin/bash
set -e

# ICN Devnet Build Script - Works around Rust compiler SIGSEGV issues

echo "🔧 Building ICN for devnet deployment..."

# Set environment variables to work around LLVM/Rust compiler issues
export RUST_MIN_STACK=16777216  # 16MB stack as suggested by error
export RUSTFLAGS="-C debuginfo=1 -C opt-level=1"  # Reduce debug info complexity
export CARGO_PROFILE_DEV_DEBUG=1  # Limit debug info level
export CARGO_PROFILE_RELEASE_DEBUG=1

# Increase system limits if possible
ulimit -s 32768 2>/dev/null || echo "Warning: Could not increase stack size"

echo "Environment variables set:"
echo "  RUST_MIN_STACK=$RUST_MIN_STACK"
echo "  RUSTFLAGS=$RUSTFLAGS"
echo "  Stack limit: $(ulimit -s)"

# Clean previous builds to avoid incremental compilation issues
echo "🧹 Cleaning previous builds..."
cargo clean

# Build in release mode with reduced debug info to avoid LLVM issues
echo "🏗️ Building ICN node in release mode with reduced debug info..."
cargo build --release --package icn-node --features with-libp2p

# Also build CLI to avoid compilation during federation setup
echo "🏗️ Building ICN CLI in release mode..."
cargo build --release --package icn-cli

# If release build succeeds but we need debug for development, try with limited debug
if [ $? -eq 0 ]; then
    echo "✅ Release builds successful"
    
    echo "🔧 Attempting debug builds with limited debug info..."
    # Try debug build with very limited debug info
    RUSTFLAGS="-C debuginfo=0" cargo build --package icn-node --features with-libp2p || {
        echo "⚠️ Debug build failed, using release build for devnet"
        # Copy release binary to debug location for devnet
        mkdir -p target/debug
        cp target/release/icn-node target/debug/icn-node
    }
    
    # Try CLI debug build
    RUSTFLAGS="-C debuginfo=0" cargo build --package icn-cli || {
        echo "⚠️ CLI debug build failed, using release build"
        # Copy release CLI binary to debug location
        mkdir -p target/debug
        cp target/release/icn-cli target/debug/icn-cli
    }
else
    echo "❌ Release build failed"
    exit 1
fi

echo "✅ ICN build completed successfully"
echo "Ready to run: just devnet" 