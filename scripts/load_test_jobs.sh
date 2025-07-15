#!/bin/bash
set -e

# Load test ICN devnet by submitting many jobs concurrently
# Usage: load_test_jobs.sh [num_jobs] [node_url]

NUM_JOBS=${1:-100}
NODE_URL=${2:-http://localhost:5001}
API_KEY=${API_KEY:-devnet-a-key}
JOB_FILE="${JOB_FILE:-$(dirname "$0")/../icn-devnet/job_test.json}"

submit_job() {
  curl -s -X POST "$NODE_URL/mesh/submit" \
    -H 'Content-Type: application/json' \
    -H "x-api-key: $API_KEY" \
    -d "$(cat "$JOB_FILE")" >/dev/null || true
}

start_time=$(date +%s)
for i in $(seq 1 "$NUM_JOBS"); do
  submit_job &
done
wait
end_time=$(date +%s)

echo "Submitted $NUM_JOBS jobs in $((end_time - start_time)) seconds"

METRICS_URL="http://localhost:9090/api/v1/query?query=icn_jobs_completed_total"
if curl -sf "$METRICS_URL" >/dev/null; then
  curl -s "$METRICS_URL" > load_test_metrics.json
  echo "Prometheus metrics written to load_test_metrics.json"
fi
