# P2P+DAG End-to-End Integration Tests

This directory contains comprehensive end-to-end tests for the integration between ICN's P2P networking layer (`icn-network`) and the DAG storage system (`icn-dag`). These tests validate the complete interaction between distributed networking and content-addressed storage in realistic multi-node scenarios.

## Overview

The P2P+DAG integration tests (`p2p_dag_e2e.rs`) cover critical aspects of ICN's distributed architecture:

- **Multi-node DAG synchronization**: How DAG blocks propagate across the P2P network
- **Cross-node receipt anchoring**: Job execution receipts stored and verified across nodes
- **DAG fork detection and resolution**: Handling conflicting DAG states deterministically
- **Network partition recovery**: DAG consistency after network splits and healing
- **Performance under load**: Scalability of P2P+DAG operations
- **Integrity validation**: Ensuring DAG link integrity across the network

## Test Architecture

### Test Harness (`P2PDagTestHarness`)

The test harness manages multiple libp2p nodes with in-memory DAG stores:

- **Bootstrap topology**: First node acts as bootstrap, others connect to it
- **Network convergence**: Waits for P2P network to stabilize before tests
- **Node coordination**: Provides utilities for inter-node operations

### Individual Test Nodes (`TestNode`)

Each test node represents a complete ICN node with:

- **DID-based identity**: Unique decentralized identifier
- **libp2p networking**: Full P2P communication stack
- **DAG storage**: In-memory content-addressed store
- **Cryptographic signing**: Default signer for receipts and blocks

## Test Scenarios

### 1. Multi-node DAG Synchronization (`test_multi_node_dag_synchronization`)

**Purpose**: Validates basic DAG block propagation across P2P network

**Flow**:
1. Node 0 creates a DAG block with test data
2. Node 0 announces the block to the network
3. Other nodes request and receive the block
4. All nodes verify they have the same block

**Validates**: 
- Block announcement protocols
- Network message routing
- DAG store consistency

### 2. Cross-node Receipt Anchoring (`test_cross_node_receipt_anchoring`)

**Purpose**: Tests job execution receipt flow between submitter and executor

**Flow**:
1. Submitter node creates a mock mesh job
2. Executor node generates execution receipt with cryptographic signature
3. Executor stores receipt in local DAG and sends to submitter
4. Submitter validates and anchors the receipt

**Validates**:
- Receipt creation and signature verification
- Cross-node messaging for receipts
- DAG anchoring of execution proofs

### 3. DAG Fork Resolution (`test_dag_fork_resolution`)

**Purpose**: Tests deterministic resolution of conflicting DAG states

**Flow**:
1. Creates a common parent block on all nodes
2. Generates two conflicting child blocks on different nodes
3. Applies deterministic resolution rule (lexicographic CID ordering)
4. Verifies all nodes converge to the canonical fork

**Validates**:
- Fork detection mechanisms
- Deterministic resolution algorithms
- Network-wide state convergence

### 4. Network Partition Recovery (`test_network_partition_recovery`)

**Purpose**: Tests DAG consistency after network partitions heal

**Flow**:
1. Establishes shared state across all nodes
2. Simulates network partition with divergent state creation
3. Simulates partition healing with state synchronization
4. Verifies all nodes have consistent final state

**Validates**:
- Partition tolerance
- State reconciliation protocols
- Data consistency guarantees

### 5. Performance Under Load (`test_performance_under_load`)

**Purpose**: Tests P2P+DAG performance with concurrent operations

**Flow**:
1. Creates 50 DAG blocks concurrently across 5 nodes
2. Measures creation throughput and latency
3. Validates memory usage remains reasonable
4. Ensures no data loss or corruption

**Validates**:
- Concurrent operation handling
- Performance characteristics
- Resource usage patterns

### 6. DAG Integrity Validation (`test_dag_integrity_validation`)

**Purpose**: Tests DAG link integrity across the network

**Flow**:
1. Creates a chain of linked DAG blocks
2. Replicates the chain across all nodes
3. Validates all links are resolvable on each node
4. Ensures referential integrity

**Validates**:
- DAG link resolution
- Cross-reference validation
- Data integrity preservation

### 7. Comprehensive Integration (`test_comprehensive_p2p_dag_integration`)

**Purpose**: Combines all scenarios in a single comprehensive test

**Flow**:
1. **Phase 1**: Basic synchronization
2. **Phase 2**: Receipt anchoring
3. **Phase 3**: Load testing with concurrent operations
4. **Phase 4**: Final consistency verification

**Validates**:
- End-to-end system functionality
- Multi-phase operation coordination
- Overall system robustness

## Running the Tests

### Quick Start

```bash
# Run all P2P+DAG integration tests
./scripts/run_p2p_dag_tests.sh

# Run with verbose output
./scripts/run_p2p_dag_tests.sh --verbose

# Run quick subset (basic tests only)
./scripts/run_p2p_dag_tests.sh --quick
```

### Manual Execution

```bash
# Run all tests
cargo test --package icn-integration-tests --features enable-libp2p --test p2p_dag_e2e

# Run specific test
cargo test --package icn-integration-tests --features enable-libp2p --test p2p_dag_e2e test_multi_node_dag_synchronization -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test --package icn-integration-tests --features enable-libp2p --test p2p_dag_e2e -- --nocapture
```

### Script Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose test output with detailed logging |
| `-q, --quick` | Run only basic synchronization and receipt tests |
| `-t, --test NAME` | Run a specific test by name |
| `--features FEAT` | Override feature flags (default: enable-libp2p) |
| `-h, --help` | Show help message with all options |

## Prerequisites

### Required Features

- `enable-libp2p`: Enables libp2p networking stack (required)

### Dependencies

- **Rust nightly**: For workspace features
- **tokio runtime**: For async test execution
- **tracing**: For structured logging during tests

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level (trace, debug, info, warn, error) | `info` |
| `RUST_BACKTRACE` | Enable backtraces on panic | `1` |

## Test Limitations

### Current Scope

These tests focus on the **integration patterns** between P2P and DAG rather than:

- Full libp2p feature testing (covered in `icn-network` unit tests)
- DAG storage backend testing (covered in `icn-dag` unit tests)
- Complete job execution pipeline (covered in other integration tests)

### Simulation vs. Reality

Some aspects are **simulated** for testing purposes:

- **Block transfer**: Manual copying rather than automatic P2P sync
- **Fork resolution**: Simplified deterministic rules
- **Partition healing**: Manual state reconciliation

In production, these would be handled by:
- Automatic DAG sync protocols
- Governance-defined resolution policies
- Built-in partition recovery mechanisms

## Debugging Failed Tests

### Common Issues

1. **Network convergence timeout**
   - Increase convergence wait time
   - Check libp2p configuration
   - Verify bootstrap peer connectivity

2. **DAG integrity failures**
   - Check CID calculation consistency
   - Validate block serialization/deserialization
   - Ensure proper link resolution

3. **Receipt verification failures**
   - Verify cryptographic key consistency
   - Check signature algorithm compatibility
   - Validate receipt format

### Debug Techniques

```bash
# Run with maximum logging
RUST_LOG=trace cargo test --package icn-integration-tests --features enable-libp2p --test p2p_dag_e2e test_name -- --nocapture

# Run single test with backtraces
RUST_BACKTRACE=full cargo test --package icn-integration-tests --features enable-libp2p --test p2p_dag_e2e test_name -- --nocapture

# Check test artifacts (if preserved)
ls -la ./dag_* ./mana_* ./rep_*
```

## Integration with CI/CD

### Continuous Integration

These tests are designed to run in CI environments:

- **Isolated execution**: Each test cleans up artifacts
- **Deterministic results**: No random behavior or timing dependencies
- **Reasonable timeouts**: Complete within CI time limits

### Performance Benchmarking

The performance test provides metrics for tracking:

- **Blocks per second**: DAG creation throughput
- **Memory usage**: Per-node storage overhead
- **Network convergence time**: P2P network stabilization

## Future Enhancements

### Planned Improvements

1. **Automatic block synchronization**: Real P2P DAG sync protocols
2. **Advanced fork resolution**: Governance-based conflict resolution
3. **Network partition detection**: Automatic partition/healing detection
4. **Persistent storage testing**: Integration with database backends
5. **Byzantine fault tolerance**: Malicious node behavior testing

### Contributing

When adding new P2P+DAG integration tests:

1. Follow the existing test harness patterns
2. Include comprehensive documentation
3. Add appropriate cleanup and error handling
4. Update this README with new test descriptions

## Related Documentation

- [ICN Core Architecture](../../CONTEXT.md)
- [Networking Guide](../../crates/icn-network/README.md)
- [DAG Storage Guide](../../crates/icn-dag/README.md)
- [Integration Test Overview](../README.md) 