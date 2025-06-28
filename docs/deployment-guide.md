# ICN Deployment Guide

This guide provides minimal examples for launching `icn-node` in common scenarios.

## Single Node Local

This mode runs a standalone node for development or testing.

```bash
icn-node --storage-backend memory --http-listen-addr 127.0.0.1:7845 \
         --api-key mylocalkey \
         --tls-cert-path ./cert.pem --tls-key-path ./key.pem
```

Providing certificate and key paths makes the server listen on HTTPS instead of HTTP.

A sample TOML configuration is in `configs/single_node.toml`.
You can also supply these values via configuration or environment variables:

```toml
http_listen_addr = "127.0.0.1:7845"
api_key = "mylocalkey"
tls_cert_path = "./cert.pem"
tls_key_path = "./key.pem"
```

## Small Federation

For a small group of cooperating nodes, each node may use a persistent store and
bootstrap to known peers.

```bash
icn-node --storage-backend rocksdb --storage-path ./icn_data/node1.rocks \
         --bootstrap-peers /ip4/1.2.3.4/tcp/7000/p2p/QmPeer \
         --api-key node1secret --open-rate-limit 0 \
         --tls-cert-path ./cert.pem --tls-key-path ./key.pem
```

See `configs/small_federation.toml` for an example configuration file.
A configuration file might contain:

```toml
http_listen_addr = "0.0.0.0:7845"
storage_backend = "rocksdb"
storage_path = "./icn_data/node1.rocks"
bootstrap_peers = ["/ip4/1.2.3.4/tcp/7000/p2p/QmPeer"]
api_key = "node1secret"
tls_cert_path = "./cert.pem"
tls_key_path = "./key.pem"
```
