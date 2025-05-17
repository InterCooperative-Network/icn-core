# ICN Network (`icn-network`)

This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN).
It defines the core networking abstractions, message types, and service interfaces. The initial implementation includes a stubbed network service for testing and development, with plans to integrate a full P2P stack (e.g., using libp2p) under a feature flag.

## Purpose

The `icn-network` crate is responsible for:

*   **P2P Communication:** Establishing and managing connections between ICN nodes.
*   **Transport Protocols:** Abstracting underlying transport mechanisms (e.g., TCP, QUIC).
*   **Peer Discovery:** Finding and connecting to other peers in the network.
*   **Message Definition & Routing:** Defining common network message types and handling their exchange between peers.
*   **Federation Synchronization:** Potentially handling specific protocols for data synchronization between federated ICN clusters.

## Key Components

*   **`PeerId`**: A struct representing a unique identifier for a network peer. Currently a simple string wrapper, but intended to be compatible with underlying P2P library IDs (e.g., libp2p `PeerId`).
*   **`NetworkMessage`**: An enum defining the various types of messages that can be exchanged between ICN nodes. This includes messages for block announcements (`AnnounceBlock`), block requests (`RequestBlock`), generic gossip messages (`GossipSub`), and federation sync requests (`FederationSyncRequest`). This enum derives `Serialize` and `Deserialize` for network transmission.
*   **`NetworkService` Trait**: An abstraction defining the core functionalities a network service provider must implement. This includes methods like `discover_peers`, `send_message`, and `broadcast_message`. Methods return `Result<_, CommonError>` using specific error variants like `PeerNotFound`, `MessageSendError`, etc.
*   **`StubNetworkService`**: A default implementation of `NetworkService` that simulates network interactions by logging actions to the console and returning predefined data. It's used for development and testing of higher-level crates without requiring a live P2P network. It demonstrates returning specific `CommonError` variants for simulated network issues.

## Error Handling

Functions and methods within this crate return `Result<T, CommonError>`, utilizing specific variants from `icn_common::CommonError` relevant to networking, such as:
*   `CommonError::PeerNotFound`
*   `CommonError::MessageSendError`
*   `CommonError::NetworkConnectionError`
*   `CommonError::NetworkUnhealthy`

The `StubNetworkService` also simulates these errors to help test error handling in dependent crates.

## `with-libp2p` Feature

This crate includes an optional `with-libp2p` feature. When enabled, it will pull in `libp2p` as a dependency, allowing for the implementation of a `NetworkService` backed by a real libp2p stack.

## Public API Style

This crate provides: 
*   Data structures (`PeerId`, `NetworkMessage`).
*   A core trait (`NetworkService`) for P2P interactions.
*   A concrete stub implementation (`StubNetworkService`) for testing and initial development.

The API aims for modularity, allowing different P2P backends to be integrated by implementing the `NetworkService` trait.

## Contributing

Please refer to the main `CONTRIBUTING.md` in the root of the `icn-core` repository.

Key areas for future contributions:
*   Implementing a `Libp2pNetworkService` that utilizes the `libp2p` stack (under the `with-libp2p` feature).
*   Defining and implementing robust peer discovery mechanisms.
*   Implementing secure and efficient message serialization and transport.
*   Adding support for various transport protocols.

## License

This crate is licensed under the Apache 2.0 license, as is the entire `icn-core` repository. 