#!/bin/bash
set -e

# load_test.sh
# Submit a large number of jobs concurrently against a running node.

CONCURRENT_JOBS=100
NODE_URL="http://localhost:5001"
API_KEY="${API_KEY:-devnet-a-key}"
JOB_FILE="${JOB_FILE:-$(dirname "$0")/../icn-devnet/job_test.json}"
METRICS_OUTPUT="load_test_results.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --jobs)
      CONCURRENT_JOBS="$2"; shift 2;;
    --node-url)
      NODE_URL="$2"; shift 2;;
    --metrics-output)
      METRICS_OUTPUT="$2"; shift 2;;
    *) echo "Usage: $0 [--jobs N] [--node-url URL] [--metrics-output FILE]"; exit 1;;
  esac
done

submit_job() {
  curl -s -X POST "$NODE_URL/mesh/submit" \
    -H 'Content-Type: application/json' \
    -H "X-API-Key: $API_KEY" \
    -d "$(cat "$JOB_FILE")" >/dev/null || true
}

start_time=$(date +%s)
for i in $(seq 1 "$CONCURRENT_JOBS"); do
  submit_job &
done
wait
end_time=$(date +%s)

echo "Submitted $CONCURRENT_JOBS jobs in $((end_time - start_time)) seconds"

METRICS_URL="http://localhost:9090/api/v1/query?query=icn_jobs_completed_total"
if curl -sf "$METRICS_URL" >/dev/null; then
  curl -s "$METRICS_URL" > "$METRICS_OUTPUT"
  echo "Metrics written to $METRICS_OUTPUT"
fi

