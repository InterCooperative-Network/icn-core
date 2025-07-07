# ICN Common Crate

This crate provides common data structures, types, utilities, and error definitions shared across multiple InterCooperative Network (ICN) core crates.

See [CONTEXT.md](../CONTEXT.md) for ICN Core design philosophy and crate roles.

## Purpose

The `icn-common` crate aims to:

*   **Reduce Code Duplication:** By providing a central place for widely used definitions (e.g., custom error types, identifiers like DIDs or CIDs, serialization helpers).
*   **Ensure Consistency:** By standardizing how common concepts are represented across the ICN ecosystem.
*   **Simplify Dependencies:** By allowing other crates to depend on a single `icn-common` crate instead of numerous small, specialized utility crates.

Examples of items found in this crate might include:
*   Custom `Error` enums and `Result` types.
*   Structs for Decentralized Identifiers (DIDs) or Content Identifiers (CIDs).
*   Common cryptographic utilities or constants.
*   Serialization and deserialization helpers.

## Public API Style

The API style focuses on:

*   **Stability:** Core types are intended to be stable, as they are widely used.
*   **Clarity:** Well-documented and easy-to-understand types and functions.
*   **Generality:** Providing utilities that are broadly applicable within the ICN context.

## Planned Public API

This crate will provide foundational types and utilities including:

*   **Identifiers:** Structs for DIDs (Decentralized Identifiers), CIDs (Content Identifiers), and other ICN-specific IDs.
*   **Error Handling:** A comprehensive error enum (`CommonError`) and `Result` type for standardized error reporting across crates.
*   **Data Structures:** Common data structures used in multiple protocols (e.g., for proposals, votes, economic transactions).
*   **Serialization:** Helpers or traits for consistent serialization/deserialization (e.g., using Serde with CBOR).
*   **Constants:** Widely used network constants (like `ICN_CORE_VERSION`).
*   **Traits:** Common traits for extensibility in other crates (e.g., `Identifiable`, `Signable`).

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 