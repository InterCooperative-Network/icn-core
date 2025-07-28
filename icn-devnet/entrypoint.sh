#!/bin/bash
set -e

# ICN Node Entrypoint Script with P2P Coordination
# Configures and starts ICN node based on environment variables

echo "🚀 Starting ICN Node: ${ICN_NODE_NAME:-Unknown}"
echo "📡 HTTP Listen: ${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
echo "🌐 P2P Listen: ${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
echo "🔗 Bootstrap Peers: ${ICN_BOOTSTRAP_PEERS:-none}"

# Initialize storage and coordination directories
mkdir -p /app/data
mkdir -p /app/peer_coordination

# Check if this is a bootstrap node (node-a)
IS_BOOTSTRAP_NODE=false
if [ "${ICN_NODE_NAME}" = "Federation-Node-A" ]; then
    IS_BOOTSTRAP_NODE=true
    echo "🌟 This is the bootstrap node (Node A)"
fi

# Build command line arguments
ARGS=(
    --http-listen-addr "${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
    --p2p-listen-addr "${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
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

# Add storage path for file backend
if [ "${ICN_STORAGE_BACKEND}" = "file" ]; then
    ARGS+=(--storage-path "/app/data/node_store")
fi

# Always set storage path to avoid permission issues
ARGS+=(--storage-path "/app/data/node_store")
ARGS+=(--mana-ledger-path "/app/data/mana_ledger")
ARGS+=(--reputation-db-path "/app/data/reputation_db")
ARGS+=(--governance-db-path "/app/data/governance_db")
ARGS+=(--node-did-path "/app/data/node_did.txt")
ARGS+=(--node-private-key-path "/app/data/node_sk.bs58")

# Handle P2P configuration with proper coordination
if [ "${ICN_ENABLE_P2P}" = "true" ]; then
    ARGS+=(--enable-p2p)
    
    if [ "$IS_BOOTSTRAP_NODE" = "true" ]; then
        echo "🌟 Bootstrap node - starting without bootstrap peers"
        # Bootstrap node starts without bootstrap peers
    else
        echo "🔗 Worker node - configuring with mDNS discovery"
        
        # Enable mDNS for worker nodes since bootstrap peer coordination is complex
        echo "🔍 Enabling mDNS for local peer discovery"
        ARGS+=(--enable-mdns)
    fi
else
    echo "🚫 P2P networking disabled"
fi

# Enable mDNS if explicitly requested (in addition to automatic enabling for workers)
if [ "${ICN_ENABLE_MDNS}" = "true" ]; then
    ARGS+=(--enable-mdns)
fi

# Note: Log level is controlled via RUST_LOG environment variable, not CLI argument

echo "🔧 Command: icn-node ${ARGS[*]}"

# Start the ICN node
echo "🚀 Starting ICN node..."
exec icn-node "${ARGS[@]}" 