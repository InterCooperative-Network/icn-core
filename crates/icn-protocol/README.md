# ICN Protocol Crate

This crate defines core message formats, communication protocols, and potentially helpers for a domain-specific language like CCL (Cooperative Contract Language) within the InterCooperative Network (ICN).

## Purpose

The `icn-protocol` crate is responsible for:

*   **Message Serialization:** Defining the structure and serialization/deserialization (e.g., using Protobuf, Serde, CBOR) of messages exchanged between ICN nodes.
*   **Protocol Definitions:** Specifying the state machines, interaction patterns, and rules for various ICN sub-protocols (e.g., for data synchronization, consensus, governance actions).
*   **Federation Membership:** Structures like `FederationJoinRequest` and `FederationJoinResponse` enable handshake-style membership negotiation between nodes.
*   **CCL Helpers (if applicable):** If ICN uses a Cooperative Contract Language, this crate might contain compiler utilities, AST definitions, or interpreter components for it.
*   **Version Negotiation:** Handling different versions of protocols or message formats to ensure backward and forward compatibility.

This crate ensures that nodes can understand each other and correctly participate in network operations.

## Public API Style

The API style emphasizes:

*   **Clarity and Precision:** Unambiguous definitions of message structures and protocol rules.
*   **Efficiency:** Compact and performant serialization formats.
*   **Extensibility:** Allowing new message types and protocol versions to be added.
*   **Interoperability:** Ensuring that different implementations can correctly interoperate.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 