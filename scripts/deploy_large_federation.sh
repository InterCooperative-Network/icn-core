#!/bin/bash
# Deploy a large ICN federation using docker-compose
# Options:
#   --nodes N              Number of nodes to start (default 10)
#   --network-mode MODE    Unused placeholder for future distributed/local modes
#   --monitoring-enabled   Start Prometheus and Grafana services
#   --persistence-enabled  Start the shared Postgres service
set -euo pipefail

NODES=10
NETWORK_MODE="distributed"
MONITORING=0
PERSISTENCE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --nodes)
            NODES="$2"
            shift 2
            ;;
        --network-mode)
            NETWORK_MODE="$2"
            shift 2
            ;;
        --monitoring-enabled)
            MONITORING=1
            shift
            ;;
        --persistence-enabled)
            PERSISTENCE=1
            shift
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

SERVICES=()
LETTERS=(a b c d e f g h i j k l m n o p)
for i in $(seq 1 "$NODES"); do
    idx=$((i-1))
    letter=${LETTERS[$idx]}
    SERVICES+=("icn-node-${letter}")
done

if [[ "$MONITORING" -eq 1 ]]; then
    SERVICES+=(prometheus grafana)
fi
if [[ "$PERSISTENCE" -eq 1 ]]; then
    SERVICES+=(postgres)
fi

COMPOSE_FILE="$(dirname "$0")/../icn-devnet/docker-compose.yml"
DOCKER_COMPOSE="docker-compose"
if docker compose version &>/dev/null; then
    DOCKER_COMPOSE="docker compose"
fi

$DOCKER_COMPOSE -f "$COMPOSE_FILE" up -d "${SERVICES[@]}"

echo "Federation started with $NODES nodes (mode: $NETWORK_MODE)"
