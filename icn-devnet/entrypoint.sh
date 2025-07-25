#!/bin/bash
set -e

# ICN Node Entrypoint Script
# Configures and starts ICN node based on environment variables

echo "🚀 Starting ICN Node: ${ICN_NODE_NAME:-Unknown}"
echo "📡 HTTP Listen: ${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
echo "🌐 P2P Listen: ${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
echo "🔗 Bootstrap Peers: ${ICN_BOOTSTRAP_PEERS:-none}"

# Initialize storage directory
mkdir -p /app/data

# Build command line arguments
ARGS=(
    --http-listen-addr "${ICN_HTTP_LISTEN_ADDR:-0.0.0.0:7845}"
    --p2p-listen-addr "${ICN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}"
    --storage-backend "${ICN_STORAGE_BACKEND:-memory}"
)

# Force development mode to enable real networking (not test mode)
# This ensures we get DefaultMeshNetworkService with libp2p instead of StubMeshNetworkService
ARGS+=(--dev)  # Development mode for real networking

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

# Enable P2P if requested
if [ "${ICN_ENABLE_P2P}" = "true" ]; then
    ARGS+=(--enable-p2p)
    
    # Add bootstrap peers if provided
    if [ -n "${ICN_BOOTSTRAP_PEERS}" ] && [ "${ICN_BOOTSTRAP_PEERS}" != "" ] && [ "${ICN_BOOTSTRAP_PEERS}" != "\"\"" ]; then
        # Split comma-separated bootstrap peers
        IFS=',' read -ra PEERS <<< "${ICN_BOOTSTRAP_PEERS}"
        for peer in "${PEERS[@]}"; do
            if [ -n "$peer" ] && [ "$peer" != "\"\"" ]; then
                ARGS+=(--bootstrap-peers "$peer")
            fi
        done
    fi
fi

# Enable mDNS if requested
if [ "${ICN_ENABLE_MDNS}" = "true" ]; then
    ARGS+=(--enable-mdns)
fi

# Add log level configuration for debugging
ARGS+=(--log-level "${ICN_LOG_LEVEL:-debug}")

echo "🔧 Command: icn-node ${ARGS[*]}"

# Ensure directories exist (created in Dockerfile but may be overridden by volumes)
echo "🔧 Ensuring directories are accessible for libp2p..."
# Directories are created in Dockerfile, but we ensure they're accessible

# Wait a moment for dependencies to be ready
if [ -n "${ICN_BOOTSTRAP_PEERS}" ] && [ "${ICN_BOOTSTRAP_PEERS}" != "" ]; then
    echo "⏳ Waiting for bootstrap node to be ready..."
    sleep 10
fi

# Start the ICN node
exec icn-node "${ARGS[@]}" 