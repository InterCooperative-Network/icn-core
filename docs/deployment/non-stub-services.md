# Deploying Non-Stub Services in Production

ICN Core currently runs with a mix of real and stub services. Running a federation in production requires replacing all stub implementations with their production-ready counterparts.

## Requirements

1. **Persistence backends**: Use one of the supported persistent DAG stores such as RocksDB or PostgreSQL. Avoid the in-memory backend used for development.
2. **Mesh networking**: Enable the full libp2p stack with encrypted transport and gossip-based message propagation. Disable any mock networking layers.
3. **Economics services**: Configure a real mana ledger or resource token backend. Stub ledgers are only suitable for testing.
4. **Monitoring**: Deploy Prometheus and Grafana for metrics collection and alerting. Production deployments should have dashboards for node health and network performance.
5. **Backups**: Schedule regular backups of storage volumes and ledger data. Verify recovery procedures before going live.

These steps move the system away from the stub defaults that ship with the development environment. See `docs/production-config-guide.md` for sample configuration files.
