#!/bin/bash
set -e

# Spin up a 10 node federation and optionally run basic load tests.
#
# Usage:
#   run_10node_devnet.sh [--start-only|--jobs-only|--stop-only]
#
#   --start-only : Start the federation containers and exit.
#   --jobs-only  : Run the job submission loop against an already
#                  running federation.
#   --stop-only  : Stop the federation containers and exit.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/icn-devnet/docker-compose.yml"
JOB_FILE="$ROOT_DIR/icn-devnet/job_test.json"
CLI_BIN="$ROOT_DIR/target/debug/icn-cli"

NUM_JOBS=${NUM_JOBS:-20}

MODE="full"
case "$1" in
    --start-only)
        MODE="start"
        ;;
    --jobs-only)
        MODE="jobs"
        ;;
    --stop-only)
        MODE="stop"
        ;;
esac

cd "$ROOT_DIR/icn-devnet"

if [[ "$MODE" == "start" || "$MODE" == "full" ]]; then
    docker-compose up -d icn-node-a icn-node-b icn-node-c icn-node-d icn-node-e icn-node-f icn-node-g icn-node-h icn-node-i icn-node-j postgres prometheus grafana
    # Simple wait for services
    sleep 20
fi

if [[ "$MODE" == "jobs" || "$MODE" == "full" ]]; then
    for i in $(seq 1 "$NUM_JOBS"); do
        "$CLI_BIN" --api-url http://localhost:5001 submit-job "$(cat "$JOB_FILE")" >/dev/null || true
    done
    echo "Submitted $NUM_JOBS jobs to Node A"
fi

if [[ "$MODE" == "stop" || "$MODE" == "full" ]]; then
    docker-compose down --volumes --remove-orphans
fi
