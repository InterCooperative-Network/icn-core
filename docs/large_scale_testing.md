# Large Federation Scale Testing

This guide explains how to deploy a multi‑node federation and run load testing
against it. These scripts build on the devnet tooling found in this repository
and are intended for local experimentation.

## Deploying a Large Federation

Use the `deploy_large_federation.sh` script to spin up a federation with ten or
more nodes. Monitoring (Prometheus/Grafana) and a shared Postgres instance can be
enabled via flags.

```bash
# Start a 12 node federation with monitoring and persistence
scripts/deploy_large_federation.sh \
  --nodes 12 \
  --network-mode distributed \
  --monitoring-enabled \
  --persistence-enabled
```

The script relies on Docker Compose definitions under `icn-devnet/`. Once the
containers are running, each node exposes its HTTP API on ports `5001` through
`5010` (and higher for additional nodes).

## Running a Load Test

After the federation is online, run the load test utility to generate concurrent
mesh jobs across the cluster:

```bash
scripts/load_test.sh \
  --concurrent-jobs 100 \
  --duration 30m \
  --job-types mixed \
  --metrics-output ./load_test_results.json
```

The script submits random job types (`echo`, `compute`, `transform`) to random
nodes until the duration expires. A small JSON summary is written to the file
specified by `--metrics-output`.

## Interpreting Results

1. **Prometheus/Grafana** – If monitoring was enabled, open Grafana at
   `http://localhost:3000` to view dashboards for job throughput, peer counts and
   resource usage. Prometheus metrics are available on `http://localhost:9090`.
2. **JSON Metrics** – The output file reports the number of jobs started. Combine
   this with Grafana dashboards to evaluate success rates and latency.
3. **Logs** – Container logs for each node provide detailed error information and
   can help diagnose networking or persistence issues.

Successful scale tests should show all nodes remaining healthy, jobs completing
across the federation and peer counts recovering after any induced partitions.
