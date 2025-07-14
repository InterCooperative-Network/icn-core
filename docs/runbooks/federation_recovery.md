# Federation Recovery Runbook

This runbook describes procedures for recovering an ICN federation from common failures.

## Network Partition

1. Detect unreachable nodes using `icn-cli status`.
2. Restore connectivity or restart affected containers.
3. Verify recovery with `just health-check`.

## Node Crash

1. Restart the crashed node container:
   ```bash
   docker-compose -f icn-devnet/docker-compose.yml up -d <node-name>
   ```
2. Wait for the node to rejoin the federation using `just health-check`.

## Chaos Testing

Use `scripts/chaos_test.sh` to simulate failures during testing:

```bash
./scripts/chaos_test.sh --scenario network_partition --duration 5
./scripts/chaos_test.sh --scenario node_crash --duration 10
```

After running the script, ensure all nodes are healthy and that job submission works across the federation.
