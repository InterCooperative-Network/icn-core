# ICN DAG Crate

This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG) storage and manipulation, crucial for the InterCooperative Network (ICN) data model.

## Purpose

The `icn-dag` crate is responsible for:

*   **DAG Primitives:** Defining core DAG structures (nodes, links, blocks) and operations (put, get, traverse).
*   **Content Addressing:** Ensuring data integrity and retrievability using cryptographic hashes (e.g., CIDs - Content Identifiers).
*   **Storage Abstraction:** Potentially providing interfaces for various backing stores (e.g., in-memory, disk-based, distributed).
*   **Serialization Formats:** Defining how DAGs are serialized and deserialized (e.g., CBOR, IPLD codecs).

This forms a foundational layer for data representation and exchange within the ICN.

## Public API Style

The API style prioritizes:

*   **Composability:** Allowing different DAG-based data structures to be built on top.
*   **Performance:** Efficient handling of DAG operations, especially for large graphs.
*   **Flexibility:** Supporting different codecs and storage backends where appropriate.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 