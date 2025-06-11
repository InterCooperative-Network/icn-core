# ICN Deployment Guide

This guide provides minimal examples for launching `icn-node` in common scenarios.

## Single Node Local

This mode runs a standalone node for development or testing.

```bash
icn-node --storage-backend memory --http-listen-addr 127.0.0.1:7845 \
         --api-key mylocalkey
```

A sample TOML configuration is in `configs/single_node.toml`.

## Small Federation

For a small group of cooperating nodes, each node may use a persistent store and
bootstrap to known peers.

```bash
icn-node --storage-backend sqlite --storage-path ./icn_data/node1.sqlite \
         --bootstrap-peers /ip4/1.2.3.4/tcp/7000/p2p/QmPeer \
         --api-key node1secret --open-rate-limit 0
```

See `configs/small_federation.toml` for an example configuration file.
