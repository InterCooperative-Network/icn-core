# Multi-Node ICN Cluster Guide

This guide explains how to set up and run a local multi-node InterCooperative Network (ICN) cluster for testing and development purposes.

## Prerequisites

1.  **Build the `icn-node` binary**: Ensure you have successfully built the `icn-node` executable.
    ```bash
    cargo build --package icn-node 
    # For libp2p features (recommended for multi-node testing):
    cargo build --package icn-node --features with-libp2p
    ```
2.  **Understanding CLI Arguments**: Familiarize yourself with the `icn-node` CLI arguments. Key arguments for multi-node setups include:
    *   `--storage-backend file`: To ensure each node has persistent, independent storage.
    *   `--storage-path <PATH>`: Unique path for each node's data (e.g., `./node1_data`, `./node2_data`).
    *   `--network-backend libp2p`: Essential for actual P2P communication.
    *   `--listen-address <MULTIADDR>` (Future): Will specify the network address each node listens on (e.g., `/ip4/127.0.0.1/tcp/6001`).
    *   `--bootstrap-peers <MULTIADDR_LIST>` (Future): Comma-separated list of multiaddresses of other peers to connect to for discovery.

## Setting Up a Local Multi-Node Cluster

This section describes how to run multiple `icn-node` instances on your local machine that can (eventually) form a P2P network.

### 1. Configuration for Each Node

For each node you want to run (e.g., Node 1, Node 2, Node 3), you will need a unique configuration, especially for:
*   **Storage Path**: To prevent data corruption.
*   **Network Listen Address** (Future): So nodes don't try to bind to the same port.
*   **Peer ID** (Future): Libp2p nodes will have unique Peer IDs generated automatically or from a keypair.

### 2. Running the Nodes

You'll need to open multiple terminal windows or use a process manager like `tmux` or `systemd` for a more persistent setup.

**Example: Running Two Nodes**

**Node 1 (Bootstrap Node - conceptually):**

*   Terminal 1:
    ```bash
    # Adjust target path as needed (e.g., target/debug/icn-node or target/release/icn-node)
    ./target/debug/icn-node --storage-backend file --storage-path ./node1_data \
                         --network-backend libp2p 
                         # Future: --listen-address /ip4/127.0.0.1/tcp/6001 
    ```
    Take note of Node 1's listen address and Peer ID when it starts up (this output will be improved in future versions with libp2p).

**Node 2 (Connecting to Node 1):**

*   Terminal 2:
    ```bash
    ./target/debug/icn-node --storage-backend file --storage-path ./node2_data \
                         --network-backend libp2p 
                         # Future: --listen-address /ip4/127.0.0.1/tcp/6002 \
                         # Future: --bootstrap-peers /ip4/127.0.0.1/tcp/6001/p2p/<Node1_PeerID>
    ```

**Note on Current Libp2p Implementation:**
The `Libp2pNetworkService` is currently a skeleton. True P2P connections, peer discovery via bootstrap nodes, and listen address configuration are not yet fully implemented. The commands above reflect the target usage. For now, `--network-backend libp2p` will initialize the libp2p stack, but nodes may not automatically discover or connect to each other without further implementation and explicit bootstrap configuration.

### 3. Verifying Connections and DAG Sync (Future)

Once the `Libp2pNetworkService` is more complete:

*   **Check Logs**: Node logs should indicate successful connections to peers.
*   **Peer Discovery**: Use an API endpoint or CLI command (to be developed) to list connected peers.
*   **DAG Sync**:
    1.  On Node 1, submit a new DAG block using the CLI:
        ```bash
        ./target/debug/icn-node --storage-path ./node1_data [...] demo 
        # Or specific block submission commands (to be developed)
        ```
    2.  After a short period, attempt to retrieve the same block by its CID from Node 2. If DAG sync is working, Node 2 should be able to find it.

## Demonstrating Operations in a Multi-Node Setup

Even with the current stubbed/skeleton networking, you can run multiple nodes with independent storage and governance modules. True federation and propagation of proposals/votes across nodes will require a functional network layer.

### a. Independent DAG Operations

*   **Node 1**:
    ```bash
    ./target/debug/icn-node --storage-backend file --storage-path ./node1_data demo
    ```
    Observe blocks created in `node1_data`.

*   **Node 2**:
    ```bash
    ./target/debug/icn-node --storage-backend file --storage-path ./node2_data demo
    ```
    Observe blocks created in `node2_data`, independently of Node 1.

### b. Independent Governance Operations

Each node maintains its own `GovernanceModule` state.

*   **Node 1**:
    ```bash
    # Submit a proposal
    ./target/debug/icn-node --storage-path ./node1_data proposal submit \
        --proposer-did did:example:node1_proposer \
        --proposal-type-json '{"GenericText":"Proposal from Node 1"}' \
        --description "Test N1"
    
    # List proposals on Node 1
    ./target/debug/icn-node --storage-path ./node1_data proposal list
    ```

*   **Node 2**:
    ```bash
    # Submit a different proposal
    ./target/debug/icn-node --storage-path ./node2_data proposal submit \
        --proposer-did did:example:node2_proposer \
        --proposal-type-json '{"GenericText":"Proposal from Node 2"}' \
        --description "Test N2"

    # List proposals on Node 2 (will not see Node 1's proposal without network sync)
    ./target/debug/icn-node --storage-path ./node2_data proposal list
    ```

### c. Simulating Proposal/Vote Propagation (Conceptual)

Once the network layer supports broadcasting/gossiping `NetworkMessage::AnnounceProposal` or similar:
1.  Node 1 submits a proposal. This would be broadcasted.
2.  Node 2 (and others) would receive this proposal and update their local `GovernanceModule`.
3.  A user on Node 2 could then see and vote on the proposal originating from Node 1.
4.  Votes would similarly be propagated.

The `demonstrate_governance_operations` function in `icn-node` includes a conceptual call to `request_federation_sync`, hinting at this future capability.

## Development Scripts (Future)

To simplify starting a local testnet, scripts can be developed:

*   **Bash/Python Script**:
    *   Takes the number of nodes as an argument.
    *   Creates separate data directories.
    *   Assigns sequential ports (for listen addresses).
    *   Starts each `icn-node` instance in the background or in separate terminal tabs/windows.
    *   Configures nodes after the first to use the first node as a bootstrap peer.
    *   Provides a way to easily stop all nodes.
*   **Makefile Targets**:
    *   `make localnet-start NODES=3`
    *   `make localnet-stop`
    *   `make localnet-logs NODE_ID=1`

These scripts will become more valuable as the libp2p networking implementation matures.

## Next Steps for Multi-Node Functionality

*   Complete `Libp2pNetworkService` implementation:
    *   Reliable peer connection and management.
    *   Bootstrap node processing.
    *   Configurable listen addresses.
    *   Robust message serialization and deserialization for `NetworkMessage`.
    *   Implementation of Kademlia for DHT operations and peer routing.
    *   Implementation of Gossipsub for broadcasting proposals, votes, and block announcements.
*   Develop API/CLI endpoints for network inspection (list peers, status).
*   Integrate proposal/vote propagation into the network layer. 