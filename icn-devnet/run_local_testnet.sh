#!/bin/bash

# Basic script to run multiple icn-node instances for local testing.
# This is a conceptual script and will need enhancements as icn-node capabilities evolve
# (e.g., configurable listen ports, bootstrap peer arguments).

NUM_NODES=${1:-2} # Default to 2 nodes if no argument is provided
ICN_NODE_BINARY="./target/debug/icn-node" # Adjust path if necessary
BASE_STORAGE_PATH="./tmp_testnet_data"

echo "Starting a local testnet with $NUM_NODES nodes..."

# Ensure the binary exists
if [ ! -f "$ICN_NODE_BINARY" ]; then
    echo "Error: icn-node binary not found at $ICN_NODE_BINARY" 
    echo "Please build the project first (e.g., cargo build --package icn-node)"
    exit 1
fi

# Clean up previous testnet data (optional)
# read -p "Clean up previous testnet data from $BASE_STORAGE_PATH? (y/N) " -n 1 -r
# echo
# if [[ $REPLY =~ ^[Yy]$ ]]
# then
#     echo "Cleaning up $BASE_STORAGE_PATH..."
#     rm -rf "$BASE_STORAGE_PATH"
# fi

mkdir -p "$BASE_STORAGE_PATH"

for i in $(seq 1 "$NUM_NODES")
do
    NODE_ID="node$i"
    STORAGE_PATH="$BASE_STORAGE_PATH/$NODE_ID"
    # Future: Add network port configurations, bootstrap peer logic
    # LISTEN_PORT=$((6000 + i))
    # BOOTSTRAP_ARG=""
    # if [ "$i" -gt 1 ] && [ -n "$NODE1_PEER_ID" ] && [ -n "$NODE1_LISTEN_ADDR" ]; then
    #     BOOTSTRAP_ARG="--bootstrap-peers $NODE1_LISTEN_ADDR/p2p/$NODE1_PEER_ID"
    # fi

    mkdir -p "$STORAGE_PATH"
    echo "Starting $NODE_ID..."
    echo "  Storage: $STORAGE_PATH"
    # echo "  Network: (libp2p, future port: $LISTEN_PORT)"
    
    # Run in background. For actual testing, you might want to pipe logs to files.
    # Use --features with-libp2p if you built with it and want to test libp2p.
    # The command below assumes you want to run the demo subcommand.
    "$ICN_NODE_BINARY" \
        --storage-backend file \
        --storage-path "$STORAGE_PATH" \
        --network-backend stub \ # Change to libp2p when ready, and add --features with-libp2p to cargo run/build
        demo > "$STORAGE_PATH/output.log" 2>&1 &
    
    echo "  $NODE_ID PID: $! (logs at $STORAGE_PATH/output.log)"

    # Crude way to capture first node's info (will need proper mechanism)
    # if [ "$i" -eq 1 ]; then
    #     sleep 2 # Give it a moment to start and print Peer ID (conceptual)
    #     # NODE1_PEER_ID=$(grep 'Local Peer ID' "$STORAGE_PATH/output.log" | awk '{print $NF}') # This needs Libp2pNetworkService to log it
    #     # NODE1_LISTEN_ADDR=$(grep 'Listening on' "$STORAGE_PATH/output.log" | awk '{print $3}') # This needs Libp2pNetworkService to log it
    #     # echo "  Node 1 Info: PeerID=$NODE1_PEER_ID, Addr=$NODE1_LISTEN_ADDR (conceptual)"
    # fi
done

echo ""
echo "Testnet nodes started. Use 'killall icn-node' or pkill -f icn-node to stop them."
echo "Check individual logs in $BASE_STORAGE_PATH/node*/output.log" 