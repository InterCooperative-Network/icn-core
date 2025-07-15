#!/bin/bash
set -e

# deploy_large_federation.sh
# Launch a federation with a configurable number of nodes using Docker
# Compose. Nodes beyond the ten provided in the default compose file are
# generated dynamically.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BASE_COMPOSE="$ROOT_DIR/icn-devnet/docker-compose.yml"
LARGE_COMPOSE="$ROOT_DIR/icn-devnet/docker-compose.large.yml"

NODE_COUNT=20

while [[ $# -gt 0 ]]; do
  case "$1" in
    --nodes)
      NODE_COUNT="$2"
      shift 2
      ;;
    *)
      echo "Usage: $0 [--nodes N]"
      exit 1
      ;;
  esac
done

# Copy base compose without the trailing volumes section so we can
# append our dynamically generated volumes list.
sed '/^volumes:/,$d' "$BASE_COMPOSE" > "$LARGE_COMPOSE"

append_node() {
  local letter="$1"
  local index="$2"
  local port_http=$((5000 + index))
  local port_p2p=$((4000 + index))
  local upper
  upper=$(echo "$letter" | tr '[:lower:]' '[:upper:]')

  cat >> "$LARGE_COMPOSE" <<EOF2
  icn-node-$letter:
    build:
      context: ..
      dockerfile: icn-devnet/Dockerfile
    container_name: icn-node-$letter
    hostname: icn-node-$letter
    environment:
      - ICN_NODE_NAME=Federation-Node-$upper
      - ICN_HTTP_LISTEN_ADDR=0.0.0.0:7845
      - ICN_P2P_LISTEN_ADDR=/ip4/0.0.0.0/tcp/4001
      - ICN_ENABLE_P2P=true
      - ICN_ENABLE_MDNS=true
      - ICN_BOOTSTRAP_PEERS=/ip4/172.20.0.2/tcp/4001
      - ICN_STORAGE_BACKEND=memory
      - ICN_HTTP_API_KEY=devnet-${letter}-key
      - RUST_LOG=info,icn_node=debug,icn_runtime=debug,icn_network=debug
    ports:
      - "$port_http:7845"
      - "$port_p2p:4001"
    networks:
      - icn-federation
    volumes:
      - node-$letter-data:/app/data
      - ./certs:/app/certs:ro
    depends_on:
      - icn-node-a
    healthcheck:
      test: ["CMD", "curl", "-f", "-H", "X-API-Key: devnet-${letter}-key", "http://localhost:7845/info"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
EOF2
}

if [ "$NODE_COUNT" -gt 10 ]; then
for idx in $(seq 11 "$NODE_COUNT"); do
  letter=$(printf "%c" $((96 + idx)))
  append_node "$letter" "$idx"
  EXTRA_VOLUMES+="  node-$letter-data:\n"
done
fi

# Append new volumes at end of compose
cat >> "$LARGE_COMPOSE" <<EOF2
volumes:
  node-a-data:
  node-b-data:
  node-c-data:
  node-d-data:
  node-e-data:
  node-f-data:
  node-g-data:
  node-h-data:
  node-i-data:
  node-j-data:
$EXTRA_VOLUMES  postgres-data:
  grafana-data:
EOF2

cd "$ROOT_DIR/icn-devnet"

services=""
for idx in $(seq 1 "$NODE_COUNT"); do
  letter=$(printf "%c" $((96 + idx)))
  services+=" icn-node-$letter"
done
services+=" postgres prometheus grafana"

docker-compose -f docker-compose.large.yml up -d $services

echo "Large federation running using compose file: $LARGE_COMPOSE"
