#!/bin/bash
# Simple ICN federation load test utility
# Options:
#   --concurrent-jobs N   Number of jobs to submit in parallel
#   --duration D          Test duration (e.g. 30m, 60s)
#   --job-types LIST      Comma separated job types (echo,compute,transform or mixed)
#   --metrics-output FILE Path to write JSON summary
set -euo pipefail

CONCURRENT=20
DURATION="5m"
JOB_TYPES="mixed"
METRICS_OUT="load_test_results.json"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --concurrent-jobs)
            CONCURRENT="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --job-types)
            JOB_TYPES="$2"
            shift 2
            ;;
        --metrics-output)
            METRICS_OUT="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

parse_duration() {
    case "$1" in
        *s) echo "${1%s}" ;;
        *m) echo $(( ${1%m} * 60 )) ;;
        *h) echo $(( ${1%h} * 3600 )) ;;
        *) echo "$1" ;;
    esac
}

DURATION_SEC=$(parse_duration "$DURATION")

NODES=(5001 5002 5003 5004 5005 5006 5007 5008 5009 5010)
API_KEYS=(devnet-a-key devnet-b-key devnet-c-key devnet-d-key devnet-e-key devnet-f-key devnet-g-key devnet-h-key devnet-i-key devnet-j-key)
JOB_TYPES_ARR=(echo compute transform)
if [[ "$JOB_TYPES" != "mixed" ]]; then
    IFS=',' read -ra JOB_TYPES_ARR <<< "$JOB_TYPES"
fi

JOB_ECHO='{ "manifest_cid": "cidv1-echo", "spec_json": { "kind": { "Echo": { "payload": "load" } } }, "cost_mana": 1 }'
JOB_COMPUTE='{ "manifest_cid": "cidv1-compute", "spec_json": { "kind": { "Compute": { "program": "fibonacci", "args": ["5"] } } }, "cost_mana": 2 }'
JOB_TRANSFORM='{ "manifest_cid": "cidv1-transform", "spec_json": { "kind": { "Transform": { "input_format": "json", "output_format": "json" } }, "inputs": [], "outputs": [] }, "cost_mana": 2 }'

submit_job() {
    local node=$1
    local key=$2
    local spec=$3
    curl -s -X POST "http://localhost:${node}/mesh/submit" \
        -H 'Content-Type: application/json' \
        -H "x-api-key: ${key}" \
        -d "$spec" >/dev/null
}

start_time=$(date +%s)
end_time=$((start_time + DURATION_SEC))
started=0

while (( $(date +%s) < end_time )); do
    for ((i=0;i<CONCURRENT;i++)); do
        idx=$((RANDOM % ${#NODES[@]}))
        key=${API_KEYS[$idx]}
        node=${NODES[$idx]}
        case "${JOB_TYPES_ARR[$((RANDOM % ${#JOB_TYPES_ARR[@]}))]}" in
            echo) spec="$JOB_ECHO";;
            compute) spec="$JOB_COMPUTE";;
            transform) spec="$JOB_TRANSFORM";;
        esac
        submit_job "$node" "$key" "$spec" &
        ((started++))
    done
    wait
done

cat > "$METRICS_OUT" <<JSON
{ "jobs_started": $started }
JSON

echo "Load test complete: $started jobs started"
