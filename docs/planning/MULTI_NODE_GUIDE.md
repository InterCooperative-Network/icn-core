# Multi-Node ICN Cluster Guide

This guide explains how to set up and run a local multi-node InterCooperative Network (ICN) cluster for testing and development purposes.

## Prerequisites

1. **Build the `icn-node` binary**: Ensure you have successfully built the `icn-node` executable with libp2p support.

   ```bash
   cargo build --package icn-node --features with-libp2p
   ```

2. **Understanding CLI Arguments**: Key arguments for multi-node setups include:

   * `--enable-p2p`: Start the node with real libp2p networking.
   * `--p2p-listen-addr <MULTIADDR>`: Address to listen for P2P connections (e.g., `/ip4/127.0.0.1/tcp/6001`).
   * `--bootstrap-peers <MULTIADDR_LIST>`: Comma-separated list of peers to dial on startup.
   * `--storage-backend file|sqlite`: Persist node data across restarts.
   * `--storage-path <PATH>`: Unique path for each node's data directory or database.

## Quick Devnet with Docker

If you simply want to see a working federation without manually starting each node, use the containerized devnet.

1. **Launch the devnet** from the repository root:
   ```bash
   cd icn-devnet
   ./launch_federation.sh
   ```
   The script starts three nodes, waits for P2P convergence, and submits a sample job. You should see output similar to:
   ```bash
   🚀 ICN Federation Devnet Launch Starting...
   ✅ Prerequisites checked
   ✅ Node A is healthy
   ✅ Node B is healthy
   ✅ Node C is healthy
   ✅ P2P network has converged
   ✅ Job submitted with ID: cidv1-85-20-abc123...
   🎉 ICN Federation is now running!
   ```
2. **With monitoring enabled** (Prometheus & Grafana):
   ```bash
   cd icn-devnet
   docker-compose --profile monitoring up -d
   ```
   Prometheus is at <http://localhost:9090> and Grafana at
   <http://localhost:3000> (`admin` / `icnfederation`).
2. **Try a job yourself** once the nodes are up:
   ```bash
   curl -X POST http://localhost:5001/mesh/submit \
     -H 'Content-Type: application/json' \
     -d '{"manifest_cid":"example_manifest","spec_json":{"Echo":{"payload":"hello federation"}},"cost_mana":50}'
   ```
   Check the status from another node:
   ```bash
   curl http://localhost:5002/mesh/jobs/JOB_ID
   ```
   When completed, retrieve the receipt data:
   ```bash
   curl -X POST http://localhost:5003/dag/get \
     -H 'Content-Type: application/json' \
     -d '{"cid":"RESULT_CID"}'
   ```

## Setting Up a Local Multi-Node Cluster

This section describes how to run multiple `icn-node` instances on your local machine that can form a P2P network.

### 1. Configuration for Each Node

For each node you want to run (e.g., Node 1, Node 2, Node 3), ensure:

* **Unique Storage Path**
* **Unique P2P Listen Address**
* **Auto-generated Peer ID** at startup from a fresh Ed25519 keypair.

#### Example Configuration File

Create a TOML file per node. This example enables persistent storage and an API
key. TLS is handled via an external reverse proxy.

```toml
# node1.toml
node_name = "Local Node 1"
http_listen_addr = "0.0.0.0:7845"      # behind TLS proxy
storage_backend = "sqlite"
storage_path = "./node1_data/node.sqlite"
api_key = "node1secret"
open_rate_limit = 0
```

Run the node with `./target/debug/icn-node --config node1.toml` and place a TLS
proxy (e.g., Nginx) in front that forwards HTTPS traffic to `http://localhost:7845` while adding `x-api-key: node1secret`.

### 2. Running the Nodes

You'll need multiple terminal windows or use `tmux`, `systemd`, or a script for persistent setup.

#### Example: Running Two Nodes

**Node 1 (Bootstrap Node):**

```bash
./target/debug/icn-node \
    --enable-p2p \
    --p2p-listen-addr /ip4/127.0.0.1/tcp/6001 \
    --storage-backend file \
    --storage-path ./node1_data
```

Note the printed Peer ID on startup.

**Node 2 (Connecting to Node 1):**

```bash
./target/debug/icn-node \
    --enable-p2p \
    --p2p-listen-addr /ip4/127.0.0.1/tcp/6002 \
    --bootstrap-peers /ip4/127.0.0.1/tcp/6001/p2p/<Node1_PeerID> \
    --storage-backend file \
    --storage-path ./node2_data
```

> Nodes will automatically perform peer discovery via Kademlia and begin Gossipsub-based block propagation once connected.

### 3. Verifying Connections and DAG Sync

* **Logs**: Watch for peer connection messages.
* **Submit a Block** on Node 1, then **fetch it** from Node 2.
* Success confirms gossip and DAG sync are functioning.

## Demonstrating Operations in a Multi-Node Setup

### a. Independent DAG Operations

```bash
./target/debug/icn-node --storage-backend file --storage-path ./node1_data demo
./target/debug/icn-node --storage-backend file --storage-path ./node2_data demo
```

### b. Independent Governance Operations

**Node 1:**

```bash
./target/debug/icn-node --storage-path ./node1_data proposal submit \
    --proposer-did did:example:node1_proposer \
    --proposal-type-json '{"GenericText":"Proposal from Node 1"}' \
    --description "Test N1"
./target/debug/icn-node --storage-path ./node1_data proposal list
```

**Node 2:**

```bash
./target/debug/icn-node --storage-path ./node2_data proposal submit \
    --proposer-did did:example:node2_proposer \
    --proposal-type-json '{"GenericText":"Proposal from Node 2"}' \
    --description "Test N2"
./target/debug/icn-node --storage-path ./node2_data proposal list
```

### c. Simulating Proposal/Vote Propagation (Planned)

Planned features include:

* Broadcasting proposals and votes via `ProtocolMessage`.
* Remote nodes receiving and syncing governance state.
* Full inter-node proposal lifecycle with vote aggregation.

## Development Scripts (Planned)

* `start_localnet.sh`: Spins up N nodes with sequential ports.
* `stop_localnet.sh`: Gracefully shuts down all nodes.
* `Makefile` targets:

  * `make localnet-start NODES=3`
  * `make localnet-stop`
  * `make localnet-logs NODE_ID=1`
## Libp2p Integration Tests

To verify cross-node job announcements, bidding, and the runtime pipeline you can run the workspace tests with libp2p enabled.

1. **Build the full workspace** with libp2p support:

```bash
cargo build --workspace --features with-libp2p
```

2. **Run integration tests** that exercise multi-node messaging and execution:

```bash
cargo test --features enable-libp2p --workspace
```

Set `RUST_LOG=info` to see verbose logs. Tests that interact with the containerized devnet check the `ICN_DEVNET_RUNNING` environment variable. If it is unset, the test harness will automatically start `icn-devnet/launch_federation.sh`.


## Roadmap for Multi-Node Networking

* [ ] Full `Libp2pNetworkService` with peer management
* [ ] Gossipsub propagation of `MessagePayload::GovernanceProposalAnnouncement`, `GovernanceVoteAnnouncement`, etc.
* [ ] CLI/API for listing peers, showing connection status
* [ ] Governance state sync hooks across nodes
