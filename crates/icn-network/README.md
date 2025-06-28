# ICN Network (`icn-network`)

This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN).
It defines the core networking abstractions, message types, and service interfaces. A lightweight stub service is available for tests, while production builds enable a libp2p-based implementation via the `libp2p` feature.

## Purpose

The `icn-network` crate is responsible for:

*   **P2P Communication:** Establishing and managing connections between ICN nodes.
*   **Transport Protocols:** Abstracting underlying transport mechanisms (e.g., TCP, QUIC).
*   **Peer Discovery:** Finding and connecting to other peers in the network.
    The `Libp2pNetworkService` performs periodic discovery using Kademlia with
    a configurable `peer_discovery_interval` and will retry dialing configured
    bootstrap peers.
*   **Message Definition & Routing:** Defining common network message types and handling their exchange between peers.
*   **Federation Synchronization:** Potentially handling specific protocols for data synchronization between federated ICN clusters.

## Key Components

*   **`PeerId`**: A struct representing a unique identifier for a network peer. Currently a simple string wrapper, but intended to be compatible with underlying P2P library IDs (e.g., libp2p `PeerId`).
*   **`NetworkMessage`**: An enum defining the various types of messages that can be exchanged between ICN nodes. This includes messages for block announcements (`AnnounceBlock`), block requests (`RequestBlock`), generic gossip messages (`GossipSub`), federation sync requests (`FederationSyncRequest`), and federation join handshakes (`FederationJoinRequest`/`FederationJoinResponse`). This enum derives `Serialize` and `Deserialize` for network transmission.
*   **`NetworkService` Trait**: An abstraction defining the core functionalities a network service provider must implement. This includes methods like `discover_peers`, `send_message`, and `broadcast_message`. Methods return `Result<_, CommonError>` using specific error variants like `PeerNotFound`, `MessageSendError`, etc.
*   **`StubNetworkService`**: A default implementation of `NetworkService` that simulates network interactions by logging actions to the console and returning predefined data. It's used for development and testing of higher-level crates without requiring a live P2P network. It demonstrates returning specific `CommonError` variants for simulated network issues.

## Error Handling

Functions and methods within this crate return `Result<T, CommonError>`, utilizing specific variants from `icn_common::CommonError` relevant to networking, such as:
*   `CommonError::PeerNotFound`
*   `CommonError::MessageSendError`
*   `CommonError::NetworkConnectionError`
*   `CommonError::NetworkUnhealthy`

The `StubNetworkService` also simulates these errors to help test error handling in dependent crates.

## `libp2p` Feature

This crate exposes an optional `libp2p` feature. When enabled it pulls in the `libp2p` and `libp2p-request-response` dependencies, providing a production ready `NetworkService` backed by the libp2p stack.

## Kademlia DHT

When compiled with the `libp2p` feature, the network layer exposes
basic Kademlia distributed hash table (DHT) functionality. This allows nodes to
store small records in the DHT and to discover peers through the standard
libp2p Kademlia protocol.

Kademlia commands are disabled in the stub service and only become available in
the `Libp2pNetworkService` when the feature flag is enabled. Be sure to compile
with:

```bash
cargo build --features libp2p
```

to use the `get_kademlia_record` and `put_kademlia_record` APIs as well as peer
discovery via the DHT.

## Public API Style

This crate provides:
*   Data structures (`PeerId`, `NetworkMessage`).
*   A core trait (`NetworkService`) for P2P interactions.
*   A concrete stub implementation (`StubNetworkService`) for testing.
*   With the `libp2p` feature enabled, a full `Libp2pNetworkService` and DHT record APIs (`get_kademlia_record` and `put_kademlia_record`).

The API aims for modularity, allowing different P2P backends to be integrated by implementing the `NetworkService` trait.

## Contributing

Please refer to the main `CONTRIBUTING.md` in the root of the `icn-core` repository.

Key areas for future contributions:
*   Extending the existing `Libp2pNetworkService` and refining peer discovery.
*   Defining and implementing robust peer discovery mechanisms.
*   Implementing secure and efficient message serialization and transport.
*   Adding support for various transport protocols.

## License

This crate is licensed under the Apache 2.0 license, as is the entire `icn-core` repository. 