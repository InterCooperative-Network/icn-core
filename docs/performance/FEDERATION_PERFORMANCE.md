# Federation Performance Guide

This document outlines tuning parameters and throughput expectations when operating multi‑node federations. Figures are approximate and reflect recent devnet testing.

## Tuning Parameters

- **Max Peers**: 200 per node for small deployments
- **Gossip Fanout**: 12 peers per message
- **Mesh Job Concurrency**: 4 parallel jobs per node
- **Metrics Interval**: scrape Prometheus metrics every 15 seconds

Adjust these values based on available CPU and network bandwidth. Higher peer counts increase gossip traffic and may require raising connection limits.

## Expected Throughput

A 10 node federation with default settings processes roughly:

- 200–300 simple jobs per second total
- 50–80 complex jobs (WASM compute) per second total
- Typical end‑to‑end job latency: 1–3 seconds under light load

Performance scales nearly linearly up to about 20 nodes. Beyond that, monitor network saturation and disk I/O on persistence backends.

## Monitoring Tips

- Use `network_peer_count` and `mesh_jobs_pending` metrics to watch for bottlenecks.
- Enable Grafana dashboards in `monitoring/` for federation‑wide visibility.
- Alert on sustained job failures or network error rates above 1%.

Regularly review node logs for connection churn or abnormal RPC failures. Adjust gossip parameters if message queues grow unexpectedly.

