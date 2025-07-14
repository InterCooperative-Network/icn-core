#!/bin/bash
set -e

# deploy_large_federation.sh
# Spin up a 20 node federation with Prometheus/Grafana monitoring.
# This script generates a temporary docker-compose file extending the
# default devnet compose with additional nodes.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BASE_COMPOSE="$ROOT_DIR/icn-devnet/docker-compose.yml"
LARGE_COMPOSE="$ROOT_DIR/icn-devnet/docker-compose.large.yml"

# Generate compose with extra nodes K-T derived from node-j definition
cp "$BASE_COMPOSE" "$LARGE_COMPOSE"

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

for idx in {11..20}; do
  letter=$(printf "%c" $((96 + idx)))
  append_node "$letter" "$idx"
  EXTRA_VOLUMES+="  node-$letter-data:\n"
done

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

docker-compose -f docker-compose.large.yml up -d icn-node-a icn-node-b icn-node-c icn-node-d icn-node-e icn-node-f icn-node-g icn-node-h icn-node-i icn-node-j icn-node-k icn-node-l icn-node-m icn-node-n icn-node-o icn-node-p icn-node-q icn-node-r icn-node-s icn-node-t postgres prometheus grafana

echo "Large federation running using compose file: $LARGE_COMPOSE"
