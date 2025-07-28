#!/bin/bash
set -e

# ICN Node Entrypoint Script with P2P Coordination
# Configures and starts ICN node based on environment variables

echo "🚀 Starting ICN Node: ${ICN_NODE_NAME:-Unknown}"
echo "📡 HTTP Listen: ${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
echo "🌐 P2P Listen: ${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
echo "🔗 Bootstrap Peers: ${ICN_BOOTSTRAP_PEERS:-none}"

# Initialize storage and coordination directories
# Use 'mkdir -p' with error handling in case of permission issues
mkdir -p /app/data 2>/dev/null || {
    echo "⚠️  Warning: Cannot create /app/data (likely volume permission issue), using /tmp fallback"
    mkdir -p /tmp/icn_data
    export ICN_DATA_DIR="/tmp/icn_data"
}
mkdir -p /app/peer_coordination 2>/dev/null || {
    echo "⚠️  Warning: Cannot create /app/peer_coordination, using /tmp fallback"
    mkdir -p /tmp/peer_coordination
}

# Check if this is a bootstrap node (node-a)
IS_BOOTSTRAP_NODE=false
if [ "${ICN_NODE_NAME}" = "Federation-Node-A" ]; then
    IS_BOOTSTRAP_NODE=true
    echo "🌟 This is the bootstrap node (Node A)"
fi

# Build command line arguments
ARGS=(
    --http-listen-addr "${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
    --listen-address "${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
    --storage-backend "${ICN_STORAGE_BACKEND:-memory}"
)

# Enable real networking with P2P but not test mode
# This ensures we get DefaultMeshNetworkService with libp2p instead of StubMeshNetworkService
# Note: Removed --dev flag as it doesn't exist; P2P is enabled via --enable-p2p

# Add API key if provided
if [ -n "${ICN_HTTP_API_KEY}" ]; then
    ARGS+=(--api-key "${ICN_HTTP_API_KEY}")
fi

# Add node name if provided
if [ -n "${ICN_NODE_NAME}" ]; then
    ARGS+=(--node-name "${ICN_NODE_NAME}")
fi

# Determine the data directory (fallback if permission issues)
DATA_DIR="${ICN_DATA_DIR:-/app/data}"

# Add storage path for file backend
if [ "${ICN_STORAGE_BACKEND}" = "file" ]; then
    ARGS+=(--storage-path "${DATA_DIR}/node_store")
fi

# Always set storage path to avoid permission issues
ARGS+=(--storage-path "${DATA_DIR}/node_store")
ARGS+=(--mana-ledger-path "${DATA_DIR}/mana_ledger")
ARGS+=(--reputation-db-path "${DATA_DIR}/reputation_db")
ARGS+=(--governance-db-path "${DATA_DIR}/governance_db")
ARGS+=(--node-did-path "${DATA_DIR}/node_did.txt")
ARGS+=(--node-private-key-path "${DATA_DIR}/node_sk.bs58")

# Handle P2P configuration with proper coordination
if [ "${ICN_ENABLE_P2P}" = "true" ]; then
    ARGS+=(--enable-p2p)
    
    if [ "$IS_BOOTSTRAP_NODE" = "true" ]; then
        echo "🌟 Bootstrap node - starting without bootstrap peers"
        # Bootstrap node starts without bootstrap peers
    else
        echo "🔗 Worker node - configuring with bootstrap peers and mDNS discovery"
        
        # Add bootstrap peers if provided
        if [ -n "${ICN_BOOTSTRAP_PEERS}" ]; then
            echo "🔗 Adding bootstrap peers: ${ICN_BOOTSTRAP_PEERS}"
            ARGS+=(--bootstrap-peers "${ICN_BOOTSTRAP_PEERS}")
        fi
    fi
else
    echo "🚫 P2P networking disabled"
fi

# Enable mDNS if explicitly requested OR if this is a worker node with P2P enabled
if [ "${ICN_ENABLE_MDNS}" = "true" ] || ( [ "${ICN_ENABLE_P2P}" = "true" ] && [ "$IS_BOOTSTRAP_NODE" = "false" ] ); then
    echo "🔍 Enabling mDNS for peer discovery"
    ARGS+=(--enable-mdns)
fi

# Note: Log level is controlled via RUST_LOG environment variable, not CLI argument

echo "🔧 Command: icn-node ${ARGS[*]}"

# Start the ICN node
echo "🚀 Starting ICN node..."
exec icn-node "${ARGS[@]}" 