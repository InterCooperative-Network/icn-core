#!/bin/bash

# Basic chaos testing script for ICN devnet
# Usage: chaos_test.sh --scenario <network_partition|node_failure> [--duration SECONDS] [--failure-rate PERCENT]
set -euo pipefail

SCENARIO=""
DURATION=60
FAILURE_RATE=20

while [[ $# -gt 0 ]]; do
    case "$1" in
        --scenario)
            SCENARIO="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --failure-rate)
            FAILURE_RATE="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$SCENARIO" ]]; then
    echo "Scenario required" >&2
    exit 1
fi

COMPOSE_FILE="$(dirname "$0")/../icn-devnet/docker-compose.yml"
NODES=(icn-node-a icn-node-b icn-node-c)

case "$SCENARIO" in
    network_partition)
        echo "Simulating network partition on icn-node-b for ${DURATION}s"
        docker-compose -f "$COMPOSE_FILE" pause icn-node-b
        sleep "$DURATION"
        docker-compose -f "$COMPOSE_FILE" unpause icn-node-b
        ;;
    node_failure)
        echo "Simulating random node failures for ${DURATION}s"
        end=$((SECONDS + DURATION))
        while [[ $SECONDS -lt $end ]]; do
            node=${NODES[$RANDOM % ${#NODES[@]}]}
            echo "Restarting $node"
            docker-compose -f "$COMPOSE_FILE" restart "$node"
            sleep $((FAILURE_RATE))
        done
        ;;
    *)
        echo "Unknown scenario: $SCENARIO" >&2
        exit 1
        ;;
esac

echo "Chaos test completed"
