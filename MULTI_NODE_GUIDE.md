# Multi-Node ICN Cluster Guide

This guide explains how to set up and run a local multi-node InterCooperative Network (ICN) cluster for testing and development purposes.

## Prerequisites

1.  **Build the `icn-node` binary**: Ensure you have successfully built the `icn-node` executable with libp2p support.
    ```bash
    cargo build --package icn-node --features with-libp2p
    ```
2.  **Key CLI Arguments**
    *   `--enable-p2p`: Start the node with real libp2p networking.
    *   `--p2p-listen-addr <MULTIADDR>`: Address to listen for P2P connections (e.g., `/ip4/127.0.0.1/tcp/6001`).
    *   `--bootstrap-peers <MULTIADDR_LIST>`: Comma-separated list of peers to dial on startup.
    *   `--storage-backend file|sqlite`: Persist node data across restarts.
    *   `--storage-path <PATH>`: Unique path for each node's data directory or database.

## Setting Up a Local Multi-Node Cluster

This section describes how to run multiple `icn-node` instances on your local machine that can (eventually) form a P2P network.

### 1. Configuration for Each Node

For each node you want to run (e.g., Node 1, Node 2, Node 3), you will need a unique configuration, especially for:
*   **Storage Path**: To prevent data corruption.
*   **P2P Listen Address**: Each node must listen on a different TCP port.
*   **Peer ID**: Generated automatically from a new Ed25519 keypair at startup.

### 2. Running the Nodes

You'll need to open multiple terminal windows or use a process manager like `tmux` or `systemd` for a more persistent setup.

**Example: Running Two Nodes**

**Node 1 (Bootstrap Node):**

*   Terminal 1:
    ```bash
    # Adjust target path as needed (e.g., target/debug/icn-node or target/release/icn-node)
    ./target/debug/icn-node \
        --enable-p2p \
        --p2p-listen-addr /ip4/127.0.0.1/tcp/6001 \
        --storage-backend file \
        --storage-path ./node1_data
    ```
    Take note of Node 1's listen address and Peer ID when it starts up (this output will be improved in future versions with libp2p).

**Node 2 (Connecting to Node 1):**

*   Terminal 2:
    ```bash
    ./target/debug/icn-node \
        --enable-p2p \
        --p2p-listen-addr /ip4/127.0.0.1/tcp/6002 \
        --bootstrap-peers /ip4/127.0.0.1/tcp/6001/p2p/<Node1_PeerID> \
        --storage-backend file \
        --storage-path ./node2_data
    ```

**Note**
The libp2p implementation now includes Kademlia peer discovery and Gossipsub message propagation. Nodes started with the commands above will automatically connect to configured bootstrap peers and exchange gossip once the handshake succeeds.

### 3. Verifying Connections and DAG Sync

*   **Check Logs**: Each node will print its local peer ID and indicate when peers connect or disconnect.
*   **Peer Discovery**: Connected peers are listed in the debug logs after bootstrap succeeds.
*   **DAG Sync**:
    1. On Node 1, submit a new DAG block using the CLI.
    2. After a short period, retrieve the block from Node 2. Successful retrieval confirms gossip propagation and DAG synchronization.

## Demonstrating Operations in a Multi-Node Setup

With libp2p enabled you can run multiple nodes with independent storage and governance modules. Proposals and blocks will propagate across the mesh once nodes discover each other.

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