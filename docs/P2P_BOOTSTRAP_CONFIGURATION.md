# P2P Network Bootstrap Configuration

## Overview

The ICN P2P networking has been enhanced to properly handle node discovery in different network environments, particularly fixing the issue where nodes could not find each other in Docker networks due to mDNS limitations.

## Bootstrap Peer Configuration

### Environment Variables

- `ICN_BOOTSTRAP_PEERS`: Comma-separated list of bootstrap peer multiaddresses
- `ICN_P2P_LISTEN_ADDR`: Local P2P listen address  
- `ICN_ENABLE_P2P`: Enable P2P networking (true/false)
- `ICN_ENABLE_MDNS`: Enable mDNS discovery (true/false)

### Example Configuration

```bash
# Docker environment
export ICN_BOOTSTRAP_PEERS="/ip4/icn-node-a/tcp/4001"
export ICN_P2P_LISTEN_ADDR="/ip4/0.0.0.0/tcp/4001"
export ICN_ENABLE_P2P="true"
export ICN_ENABLE_MDNS="true"

# Production environment with multiple bootstrap peers
export ICN_BOOTSTRAP_PEERS="/ip4/bootstrap1.example.com/tcp/4001,/ip4/bootstrap2.example.com/tcp/4001"
export ICN_ENABLE_MDNS="false"
```

## Network Environments

### Docker Networks
- Use container hostnames for bootstrap peers
- mDNS can be enabled as a supplementary discovery method
- Bootstrap peers should point to stable nodes in the network

### Production Networks
- Use FQDN or IP addresses for bootstrap peers
- mDNS should be disabled for security
- Configure multiple bootstrap peers for redundancy
- Use DNS SRV records for dynamic peer discovery

### Local Development
- mDNS enabled for automatic local peer discovery
- Bootstrap peers optional but recommended
- Smaller peer limits for resource efficiency

## Bootstrap Discovery Strategies

The `BootstrapDiscovery` module provides automatic peer discovery for production environments:

1. **Environment Variables**: Direct configuration via `ICN_BOOTSTRAP_PEERS`
2. **Platform Detection**: Automatic discovery for Docker Swarm, Kubernetes, AWS
3. **DNS SRV Records**: Query `_icn._tcp.domain.com` for bootstrap peers
4. **Service Discovery**: Integration with Consul, etcd, etc. (framework ready)

## Devnet Configuration

The devnet has been configured with Node A as the primary bootstrap peer:

- **Node A**: Acts as bootstrap node (no bootstrap peers needed)
- **Nodes B-J**: All configured to bootstrap from Node A via `/ip4/icn-node-a/tcp/4001`

This ensures all nodes can discover each other through the network topology established via Node A.

## Testing

Use the enhanced connectivity test script to verify P2P networking:

```bash
./test-devnet-connectivity.sh
```

The script now provides detailed information about:
- Individual node peer counts
- Kademlia DHT peer counts  
- Network statistics and diagnostics
- Job submission testing

## Troubleshooting

### Nodes showing 0 peer connections

1. Check bootstrap peer configuration
2. Verify network connectivity to bootstrap peers
3. Check firewall rules for P2P port (default 4001)
4. Review node logs for connection errors

### Bootstrap peer resolution failures

1. Verify hostname/IP resolution
2. Check if bootstrap peers are actually running
3. Ensure P2P ports are accessible
4. Try IP addresses instead of hostnames

### Production deployment issues

1. Use the `BootstrapDiscovery::create_production_config()` function
2. Configure multiple bootstrap peers for redundancy
3. Disable mDNS in production environments
4. Monitor peer counts and connection health