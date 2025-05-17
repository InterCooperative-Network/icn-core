# ICN Core

A monorepo of core Rust crates for the InterCooperative Network (ICN).

## Crate Structure

- **icn-common**: Shared types & utilities (DIDs, CIDs, errors)  
- **icn-identity**: DID management, VC issuance & crypto  
- **icn-dag**: Content-addressed DAG store interfaces  
- **icn-mesh**: Mesh job orchestration & scheduling  
- **icn-runtime**: Host environment & WASM execution  
- **icn-protocol**: Message formats & CCL compiler helpers  
- **icn-governance**: Proposal, voting & quorum logic  
- **icn-economics**: Mana/token models & ledgers  
- **icn-network**: libp2p transport & federation sync  
- **icn-api**: JSON-RPC / gRPC server definitions  
- **icn-cli**: Command-line client (binary)  
- **icn-node**: Long-running daemon (binary)  

## Getting Started

```bash
# Clone
git clone git@github.com:InterCooperative-Network/icn-core.git
cd icn-core

# Build & test
cargo build
cargo test
