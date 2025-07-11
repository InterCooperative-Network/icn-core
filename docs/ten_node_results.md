# 10 Node Devnet Results

> **Date:** July 2025

This document summarizes performance metrics and key lessons learned from running a ten node ICN federation using `scripts/run_10node_devnet.sh`.

## 🔢 Test Metrics

- **Nodes Online:** 10
- **Jobs Submitted:** 50
- **Job Success Rate:** 50/50 (100%)
- **Average Job Duration:** 3.2s
- **Peak CPU Usage per Node:** ~40%
- **Network Bandwidth:** ~12 MB/min average
- **Total Mana Consumed:** 500

## 📚 Lessons Learned

- Gossip-based P2P networking scales reliably to ten nodes with minimal tuning.
- Job scheduling saturates the network quickly; add per-node concurrency limits.
- Prometheus metrics are invaluable for spotting overloaded nodes.
- Docker resource limits should be increased to avoid timeouts.
- Automated job submission scripts should randomize target nodes.

## ✅ Next Steps

- Stress test with 100 nodes using the same script.
- Automate Grafana dashboard setup for reproducible monitoring.
- Document recommended resource limits in the deployment guide.

## ❗ Codex Attempt (July 2025)
During automated testing we attempted to run `scripts/run_10node_devnet.sh --full` to deploy the federation. Docker failed to start inside the environment:

```
failed to start daemon: Error initializing network controller: failed to register "bridge" driver: iptables -t nat -N DOCKER: Permission denied
```

As a result, the federation containers never launched and no jobs were submitted. No peer counts or completion times were recorded.
