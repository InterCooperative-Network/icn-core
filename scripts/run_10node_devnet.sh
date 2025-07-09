#!/bin/bash
set -e

# Spin up 10 node federation and run basic load tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/icn-devnet/docker-compose.yml"
JOB_FILE="$ROOT_DIR/icn-devnet/job_test.json"
CLI_BIN="$ROOT_DIR/target/debug/icn-cli"

NUM_JOBS=${NUM_JOBS:-20}

cd "$ROOT_DIR/icn-devnet"

docker-compose up -d icn-node-a icn-node-b icn-node-c icn-node-d icn-node-e icn-node-f icn-node-g icn-node-h icn-node-i icn-node-j postgres prometheus grafana

# Simple wait for services
sleep 20

for i in $(seq 1 "$NUM_JOBS"); do
    "$CLI_BIN" --api-url http://localhost:5001 submit-job "$(cat "$JOB_FILE")" >/dev/null || true
done

echo "Submitted $NUM_JOBS jobs to Node A"

docker-compose down
