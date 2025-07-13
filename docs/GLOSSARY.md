# ICN Glossary

This glossary defines key terms and concepts used throughout the InterCooperative Network (ICN) project. It serves as a reference for developers, contributors, and users.

## A

### ABI (Application Binary Interface)
The interface between WebAssembly modules and the ICN runtime. The Host ABI defines functions that WASM modules can call to interact with ICN services.

### API Key
An authentication token used to secure access to ICN node HTTP endpoints. Configured via the `--api-key` command line option.

### Async/Await
Rust's asynchronous programming model used throughout ICN for non-blocking I/O operations, particularly in networking and storage.

### Arkworks
A Rust ecosystem for zero-knowledge proofs and cryptographic primitives. Used in `icn-zk` for implementing privacy-preserving circuits.

### Authentication
The process of verifying the identity of a user or system. In ICN, this includes DID-based authentication and API key verification.

### Authorization
The process of determining what actions an authenticated user or system is allowed to perform. Managed through reputation, mana, and governance policies.

## B

### Bidding
The process by which executors compete to perform mesh jobs by submitting bids that include price and capability information.

### Bootstrap Peers
Initial peers that a node connects to when joining the network. These peers help new nodes discover the broader network topology.

### BN254
An elliptic curve used in the `icn-zk` crate for efficient zero-knowledge proofs. Provides 128-bit security with fast verification.

## C

### CCL (Cooperative Contract Language)
A domain-specific language for encoding governance policies and cooperative bylaws. Compiled to WebAssembly for deterministic execution.

### CID (Content Identifier)
A unique identifier for data in the DAG storage system. CIDs are cryptographic hashes that ensure data integrity and enable content addressing.

### Circuit Breaker
A fault tolerance pattern that prevents cascading failures by temporarily stopping requests to a failing service.

### Credentials
Digital certificates that attest to specific attributes or capabilities of an entity. Used in the identity system for authorization and trust.

### Cryptographic Receipt
A signed proof that a specific action was completed. Used to verify job execution and other network activities.

## D

### DAG (Directed Acyclic Graph)
A data structure used for content-addressed storage. Each node in the graph represents a data block, and edges represent dependencies.

### DID (Decentralized Identifier)
A unique identifier that enables verifiable, decentralized identity. ICN supports multiple DID methods including `did:key`, `did:web`, and `did:peer`.

### DID Document
A JSON-LD document that describes a DID, including its public keys, authentication methods, and service endpoints.

### Deterministic Execution
The property that the same input always produces the same output. Critical for consensus and verifiability in distributed systems.

### Devnet
A development network consisting of multiple ICN nodes running in containers. Used for testing and development purposes.

## E

### Economics
The system of resource allocation and incentives in ICN, primarily managed through the mana system and reputation scoring.

### Execution Receipt
A cryptographically signed proof that a mesh job was completed by a specific executor. Includes job details and execution results.

### Executor
A node that performs mesh jobs by executing WebAssembly modules. Executors bid on jobs and are selected based on various criteria.

## F

### Federation
A collection of cooperating ICN nodes that share governance and resources while maintaining local autonomy.

### Federation Sync
The process of synchronizing governance state and decisions across nodes in a federation.

### Fungible Token
A token where individual units are interchangeable. In ICN, mana is fungible within certain constraints.

## G

### Governance
The system for making collective decisions about network parameters, policies, and upgrades through proposals and voting.

### Gossipsub
A libp2p protocol for efficient message broadcasting across the network. Used for peer-to-peer communication in ICN.

### Groth16
A zero-knowledge proof system that produces succinct proofs. Used in the `icn-zk` crate for privacy-preserving credential verification.

## H

### Host ABI
The Application Binary Interface that allows WebAssembly modules to interact with ICN services like storage, networking, and economics.

### HTTP API
The REST interface exposed by ICN nodes for external interaction. Provides endpoints for all major ICN functionality.

### HTTPS/TLS
Secure HTTP communication enabled through TLS certificates. ICN nodes support HTTPS for secure API access.

## I

### ICN (InterCooperative Network)
The overall network protocol and platform for cooperative digital infrastructure. Not "ICN Network" (redundant).

### Identity
The system for managing decentralized identifiers, credentials, and cryptographic keys. Fundamental to trust and security in ICN.

### Integration Tests
Tests that verify the interaction between multiple components or crates. Located in the `tests/` directory.

## J

### Job
A computational task submitted to the mesh network for execution. Jobs are specified in WebAssembly and executed by selected executors.

### Job ID
A unique identifier for a mesh job, typically represented as a CID.

### Job Specification
The detailed description of a mesh job including resource requirements, execution parameters, and cost limits.

### Just
A command runner used for common development tasks. ICN uses a `justfile` for build, test, and deployment commands.

## K

### Kademlia
A distributed hash table (DHT) protocol used for peer discovery and content routing in the libp2p network stack.

### Key Management
The process of generating, storing, and using cryptographic keys securely. Critical for DID management and signing operations.

## L

### Libp2p
A modular peer-to-peer networking library used by ICN for network communication, peer discovery, and message routing.

### Liquid Democracy
A voting system that allows delegation of votes to trusted representatives. Planned for future ICN governance features.

### LRU Cache
A Least Recently Used cache implementation for efficient memory management of frequently accessed data.

## M

### Mana
The regenerating resource credit system used for rate limiting, economic enforcement, and Sybil resistance. Not speculative or extractable.

### Mesh Computing
Distributed computational execution across multiple nodes in the network. Jobs are submitted, bid on, and executed by network participants.

### Mesh Job
A computational task submitted to the mesh network for distributed execution. Distinguished from generic "jobs" or "tasks."

### Metrics
Performance and operational data collected from ICN nodes. Exposed in Prometheus format for monitoring and observability.

### Mutual Aid
Cooperative assistance and resource sharing between individuals and communities. A core principle of ICN's design.

## N

### Network Service
The trait that abstracts peer-to-peer networking functionality. Implemented by both stub and libp2p services.

### Node
An ICN daemon process that participates in the network. Nodes can serve different roles including executors, validators, and storage providers.

### Non-Fungible Token (NFT)
A unique token that represents ownership of a specific item or capability. Used in ICN for specialized permissions and assets.

## O

### Observability
The ability to monitor and understand system behavior through metrics, logs, and tracing. Critical for operating distributed systems.

### OpenAPI
A specification format for REST APIs. ICN provides an OpenAPI specification for its HTTP endpoints.

## P

### P2P (Peer-to-Peer)
A network architecture where nodes communicate directly with each other without central coordination. Used in ICN's network layer.

### Peer Discovery
The process of finding and connecting to other nodes in the network. Implemented using libp2p's Kademlia DHT.

### Peer ID
A unique identifier for a node in the peer-to-peer network. Derived from the node's public key.

### Persistent Storage
Long-term data storage that survives node restarts. ICN supports multiple backend types including SQLite, RocksDB, and Sled.

### Pinning
The process of marking DAG blocks as important to prevent them from being garbage collected during storage cleanup.

### Prometheus
A monitoring system and time series database. ICN exposes metrics in Prometheus format for operational monitoring.

### Proposal
A formal request for a governance decision. Proposals go through a lifecycle of submission, deliberation, voting, and execution.

## Q

### Quorum
The minimum number of participants required for a governance vote to be valid. Configured through CCL policies.

### QUIC
A transport protocol that provides secure, multiplexed connections. Supported by ICN's libp2p networking layer.

## R

### Rate Limiting
The practice of limiting the frequency of requests to prevent abuse and ensure fair resource usage.

### Reputation
A measure of a node's trustworthiness based on past behavior. Used in executor selection and resource allocation.

### Request/Response
A communication pattern where clients send requests and servers provide responses. Used in ICN's HTTP API.

### REST (Representational State Transfer)
An architectural style for web services. ICN's HTTP API follows REST principles with clear resource URLs and HTTP methods.

### RocksDB
A high-performance embedded database used as one of ICN's storage backends. Optimized for write-heavy workloads.

### Runtime Context
The central state management system in ICN that coordinates between different services and maintains system state.

## S

### Sled
A pure Rust embedded database used as one of ICN's storage backends. Provides ACID transactions and crash recovery.

### SQLite
A lightweight, embedded SQL database supported as one of ICN's storage backends. Good for development and smaller deployments.

### Signature
A cryptographic proof that a message was created by the holder of a specific private key. Used throughout ICN for authentication.

### Sybil Resistance
Protection against attacks where a single entity creates multiple fake identities. Addressed through mana, reputation, and governance.

## T

### TLS (Transport Layer Security)
A cryptographic protocol for secure communication. ICN nodes support TLS for HTTPS connections.

### Tokio
An asynchronous runtime for Rust that provides the foundation for ICN's concurrent operations.

### TTL (Time To Live)
An expiration time for data blocks in the DAG storage system. Blocks past their TTL can be garbage collected.

### Trusted Setup
A cryptographic ceremony required for some zero-knowledge proof systems. Needed for Groth16 circuits in `icn-zk`.

## U

### Unit Tests
Tests that verify individual functions or components in isolation. Located alongside the code they test.

### URI/URL
Uniform Resource Identifier/Locator. Used for DID identifiers, API endpoints, and resource addressing.

## V

### Verifiable Credentials
Digital certificates that can be cryptographically verified. Used in ICN's identity system for attestations.

### Voting
The process of making collective decisions in governance. ICN supports various voting methods and delegation patterns.

### VM (Virtual Machine)
The execution environment for WebAssembly modules. ICN uses a secure VM for running untrusted code.

## W

### WASM (WebAssembly)
A binary instruction format that provides secure, sandboxed execution of code. Used in ICN for mesh jobs and governance policies.

### WebAssembly System Interface (WASI)
A standard interface for WebAssembly modules to interact with system resources. Extended by ICN's Host ABI.

### Workspace
A Cargo workspace containing multiple related crates. ICN Core is organized as a workspace with domain-specific crates.

## Z

### Zero-Knowledge Proof
A cryptographic method that allows one party to prove knowledge of information without revealing the information itself.

### ZK Circuit
A mathematical circuit that can be used to generate zero-knowledge proofs. ICN provides circuits for age verification, membership, and reputation.

## Common Abbreviations

| Abbreviation | Full Term |
|--------------|-----------|
| API | Application Programming Interface |
| ABI | Application Binary Interface |
| CCL | Cooperative Contract Language |
| CI/CD | Continuous Integration/Continuous Deployment |
| CID | Content Identifier |
| CLI | Command Line Interface |
| DAG | Directed Acyclic Graph |
| DID | Decentralized Identifier |
| DHT | Distributed Hash Table |
| HTTP | HyperText Transfer Protocol |
| HTTPS | HTTP Secure |
| ICN | InterCooperative Network |
| JSON | JavaScript Object Notation |
| JWT | JSON Web Token |
| P2P | Peer-to-Peer |
| REST | Representational State Transfer |
| RPC | Remote Procedure Call |
| SDK | Software Development Kit |
| TLS | Transport Layer Security |
| TTL | Time To Live |
| URI | Uniform Resource Identifier |
| URL | Uniform Resource Locator |
| VM | Virtual Machine |
| WASM | WebAssembly |
| WASI | WebAssembly System Interface |
| ZK | Zero-Knowledge |

## Conceptual Categories

### Economic Concepts
- **Mana**: Regenerating resource credits
- **Reputation**: Trust scoring system
- **Bidding**: Executor competition mechanism
- **Economic Enforcement**: Resource usage controls

### Governance Concepts
- **Proposal**: Formal change requests
- **Voting**: Decision-making process
- **Quorum**: Minimum participation requirements
- **Federation**: Cooperative governance structure

### Technical Concepts
- **DAG**: Content-addressed storage
- **WASM**: Sandboxed execution environment
- **P2P**: Decentralized networking
- **Consensus**: Agreement mechanisms

### Identity Concepts
- **DID**: Decentralized identifiers
- **Credentials**: Verifiable attestations
- **Signatures**: Cryptographic proofs
- **Trust**: Reputation-based systems

### Security Concepts
- **Encryption**: Data protection
- **Authentication**: Identity verification
- **Authorization**: Permission controls
- **Sandboxing**: Isolation mechanisms

---

This glossary is maintained collaboratively. Please update it when introducing new concepts or clarifying existing ones. For questions about specific terms, refer to the relevant crate documentation or the [ICN Architecture Guide](ARCHITECTURE.md). 