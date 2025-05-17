# ICN Node Crate

This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon process.

## Purpose

The `icn-node` is a binary application that brings together various ICN core components (like identity, network, DAG, economics, governance, etc.) to run a fully functional ICN participant node. Its responsibilities include:

*   **Initialization:** Setting up all necessary services and modules when the node starts.
*   **Lifecycle Management:** Managing the node's operational state (starting, stopping, health checks).
*   **Configuration:** Loading and managing node-specific configurations.
*   **Service Hosting:** Exposing APIs (via `icn-api`) and participating in network protocols (via `icn-network`, `icn-protocol`, etc.).
*   **Persistence:** Managing any on-disk storage required by the node's components.

It acts as the primary executable for individuals or organizations wishing to run an ICN node.

## Public API Style

As a daemon application, its primary interactions are through:

*   **Configuration Files:** For setup and customization.
*   **Logs:** For monitoring and diagnostics.
*   **Exposed APIs:** Provided by the `icn-api` crate for remote interaction (e.g., via the `icn-cli` or other clients).
*   **Network Protocols:** For interacting with other ICN nodes.

The emphasis is on stability, reliability, and manageability for long-running deployments.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 