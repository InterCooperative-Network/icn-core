# ICN Core - Quick Start Guide

Get up and running with ICN Core development in under 10 minutes.

## Prerequisites

- **Rust**: Install via [rustup.rs](https://rustup.rs/)
- **Docker & Docker Compose**: For devnet testing
- **Git**: For version control

## Setup

1. **Clone and setup the project:**
   ```bash
   git clone https://github.com/InterCooperative-Network/icn-core.git
   cd icn-core
   just setup
   ```

2. **Build and test:**
   ```bash
   just test
   ```

3. **Launch a local development federation:**
   ```bash
   just run-devnet
   ```

## Quick Commands

| Command | Purpose |
|---------|---------|
| `just setup` | Install dependencies and development tools |
| `just test` | Run the full test suite |
| `just run-devnet` | Launch 3-node federation with demo data |
| `just format` | Format code |
| `just lint` | Run linting checks |
| `just validate` | Run format + lint + test |

## Try It Out

After running `just run-devnet`, you can:

```bash
# Check node status
cargo run -p icn-cli -- --api-url http://localhost:5001 status

# Submit a test job
cargo run -p icn-cli -- --api-url http://localhost:5001 mesh submit '{
  "manifest_cid": "example_manifest",
  "spec_bytes": "dGVzdCBqb2I=",
  "cost_mana": 50
}'

# View governance proposals
cargo run -p icn-cli -- --api-url http://localhost:5001 governance proposals
```

## Interactive Setup

For guided setup with DID creation and test data:

```bash
# Developer onboarding wizard
cargo run -p icn-cli -- wizard init-dev

# Federation setup wizard  
cargo run -p icn-cli -- wizard onboard-federation
```

## Next Steps

- Read the [Full Onboarding Guide](ONBOARDING.md) for detailed documentation
- Explore [Developer Guide](DEVELOPER_GUIDE.md) for contributing
- Check [Deployment Guide](deployment-guide.md) for production setup

## Demo Mode

For quick testing without persistence:

```bash
cargo run -p icn-node -- --demo
```

This starts a node with:
- Memory-only storage
- Preloaded test jobs and proposals  
- Sample governance data
- Development-friendly defaults