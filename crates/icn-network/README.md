# ICN Network Crate

This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN), likely utilizing libp2p. It includes transport protocols and federation synchronization mechanisms.

## Purpose

The `icn-network` crate is responsible for:

*   **P2P Communication:** Establishing and managing connections between ICN nodes using a P2P library (e.g., libp2p).
*   **Transport Protocols:** Implementing or configuring the underlying transport protocols (e.g., TCP, QUIC, WebSockets) for message exchange.
*   **Peer Discovery:** Enabling nodes to find and connect to other peers in the network.
*   **Message Routing:** Handling the routing of messages between nodes, potentially across different sub-networks or federations.
*   **Federation Sync:** Managing data synchronization and communication between different federated ICN instances or communities.
*   **Network Address Translation (NAT) Traversal:** Helping nodes behind NATs connect to the public network.

This crate forms the communication backbone of the ICN.

## Public API Style

The API style emphasizes:

*   **Reliability:** Ensuring dependable message delivery and connection management.
*   **Scalability:** Supporting a large and dynamic set of network participants.
*   **Security:** Protecting communication channels (e.g., through encryption) and authenticating peers.
*   **Modularity:** Allowing different transport or discovery mechanisms to be plugged in.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 