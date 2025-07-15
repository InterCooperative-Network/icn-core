# ICN Core Developer Guide

This guide provides comprehensive information for developers working on the InterCooperative Network (ICN) Core project. It covers setup, development workflows, testing strategies, and contribution guidelines.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Testing Guide](#testing-guide)
5. [Code Quality Standards](#code-quality-standards)
6. [Debugging and Troubleshooting](#debugging-and-troubleshooting)
7. [Performance Optimization](#performance-optimization)
8. [Contribution Guidelines](#contribution-guidelines)
9. [Release Process](#release-process)
10. [Common Development Tasks](#common-development-tasks)

## Development Environment Setup

### Prerequisites

Before starting development, ensure you have the following installed:

```bash
# Rust toolchain (use the version specified in rust-toolchain.toml)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add rustfmt clippy

# Just command runner
cargo install just

# Git hooks and formatting
pip install pre-commit
pre-commit install
```

### System Dependencies

**macOS**:
```bash
# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install required packages
brew install openssl pkg-config
```

**Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  libssl-dev \
  pkg-config \
  libsqlite3-dev \
  librocksdb-dev
```

**Fedora/CentOS**:
```bash
sudo dnf install -y \
  gcc \
  openssl-devel \
  pkgconfig \
  sqlite-devel \
  rocksdb-devel
```

### Initial Setup

1. **Clone the repository**:
```bash
git clone https://github.com/InterCooperative-Network/icn-core.git
cd icn-core
```

2. **Run initial setup**:
```bash
# Install dependencies and set up development environment
just setup

# Verify installation
just health-check
```

3. **Configure your IDE**:
   - **VS Code**: Install the Rust extension and configure rust-analyzer
   - **IntelliJ**: Install the Rust plugin
   - **Vim/Neovim**: Configure rust-analyzer with your preferred plugin

### Environment Configuration

Create a `.env` file in the project root for local development:

```bash
# Development environment variables
ICN_LOG_LEVEL=debug
ICN_TEST_MODE=true
ICN_DAG_STORE_PATH=./dev_data/dag_store
ICN_MANA_LEDGER_PATH=./dev_data/mana_ledger
ICN_GOVERNANCE_DB_PATH=./dev_data/governance.db
RUST_BACKTRACE=1
```

## Project Structure

### Workspace Organization

```
icn-core/
├── crates/                    # Core library crates
│   ├── icn-common/           # Shared types and utilities
│   ├── icn-protocol/         # Message formats and protocols
│   ├── icn-identity/         # DID management and credentials
│   ├── icn-dag/              # Content-addressed storage
│   ├── icn-economics/        # Mana and economic policies
│   ├── icn-governance/       # Proposals and voting
│   ├── icn-mesh/             # Distributed job execution
│   ├── icn-reputation/       # Trust and reputation scoring
│   ├── icn-network/          # P2P networking (libp2p)
│   ├── icn-runtime/          # WASM execution and orchestration
│   ├── icn-api/              # HTTP API interfaces
│   ├── icn-zk/               # Zero-knowledge circuits
│   ├── icn-cli/              # Command-line interface
│   └── icn-node/             # Main node daemon
├── docs/                     # Documentation
├── tests/                    # Integration tests
├── scripts/                  # Development and deployment scripts
├── icn-devnet/              # Containerized development network
└── examples/                # Example configurations and contracts
```

### Crate Dependencies

Each crate has clearly defined dependencies. Follow these principles:

- **Foundation crates** (`icn-common`, `icn-protocol`) have minimal dependencies
- **Domain crates** build upon foundation crates
- **Infrastructure crates** orchestrate domain functionality
- **Application crates** provide user interfaces

## Development Workflow

### Branch Strategy

We use a simplified GitFlow workflow:

- **`main`**: Production-ready code
- **`develop`**: Integration branch for features
- **`feature/*`**: Feature development branches
- **`hotfix/*`**: Critical bug fixes

### Feature Development Process

1. **Create a feature branch**:
```bash
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name
```

2. **Implement your feature**:
   - Write tests first (TDD approach)
   - Implement the feature
   - Update documentation
   - Run quality checks

3. **Pre-commit validation**:
```bash
just format     # Format code
just lint       # Run linting
just test       # Run all tests
just docs       # Generate documentation
```

4. **Commit and push**:
```bash
git add .
git commit -m "[crate] Brief description of changes"
git push origin feature/your-feature-name
```

5. **Create a pull request**:
   - Use the PR template
   - Include a clear description
   - Reference any related issues
   - Ensure CI passes

### Commit Message Format

Use the following format for commit messages:

```
[affected-crate] Brief description (max 50 chars)

Detailed explanation of changes:
- What was changed
- Why it was changed
- Any breaking changes

Closes #issue-number
```

Examples:
```
[icn-runtime] Add mana validation to job submission

- Added mana balance check before job submission
- Improved error messaging for insufficient mana
- Added comprehensive tests for edge cases

Closes #123
```

### Code Review Process

1. **Self-review**: Check your own code before submitting
2. **Peer review**: At least one maintainer must approve
3. **CI validation**: All tests and checks must pass
4. **Documentation**: Update relevant documentation

## Testing Guide

### Test Categories

#### Unit Tests

Located in `src/` directories alongside implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mana_calculation() {
        // Test individual functions
        let account = ManaAccount::new(1000);
        assert_eq!(account.balance(), 1000);
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        // Test async functions
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

#### Integration Tests

Located in `tests/` directory:

```rust
// tests/mesh_integration.rs
use icn_runtime::*;
use icn_mesh::*;
use icn_economics::*;

#[tokio::test]
async fn test_job_submission_flow() {
    // Test complete workflows
    let context = create_test_context().await;
    let job = create_test_job();
    
    let result = context.submit_job(job).await;
    assert!(result.is_ok());
}
```

#### End-to-End Tests

Full system tests using the devnet:

```bash
# Run E2E tests
just test-e2e

# Run specific E2E test
cargo test --test e2e_mesh_jobs
```

### Testing Best Practices

1. **Test Structure**: Follow the Arrange-Act-Assert pattern
2. **Test Data**: Use builders and factories for test data
3. **Mocking**: Use trait objects for external dependencies
4. **Deterministic Tests**: Use fixed seeds for randomness
5. **Async Testing**: Use `tokio::test` for async functions

### Test Utilities

Common test utilities are provided in each crate:

```rust
// Test fixtures
pub fn create_test_job() -> MeshJob {
    MeshJob {
        id: JobId::new(),
        command: "echo hello".to_string(),
        // ... other fields
    }
}

// Test context
pub async fn create_test_context() -> RuntimeContext {
    use icn_runtime::RuntimeContext;
    let did = icn_common::Did::new("key", "test_node");
    RuntimeContext::new_for_testing(did, Some(1000)).unwrap()
}

// Mock implementations
pub struct MockNetworkService {
    // Mock implementation
}
```

## Code Quality Standards

### Linting and Formatting

```bash
# Format all code
just format

# Check formatting
cargo fmt --all -- --check

# Run clippy
just lint

# Run clippy with all features
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation Standards

#### Rustdoc Comments

All public APIs must have comprehensive rustdoc:

```rust
/// Submits a mesh job to the network for execution.
/// 
/// This function validates the job specification, checks mana requirements,
/// and adds the job to the pending queue for bidding.
/// 
/// # Arguments
/// 
/// * `job` - The job specification to submit
/// * `submitter` - DID of the job submitter
/// 
/// # Returns
/// 
/// * `Ok(job_id)` - Unique identifier for the submitted job
/// * `Err(error)` - If validation fails or insufficient mana
/// 
/// # Examples
/// 
/// ```rust
/// let job = MeshJob::new("echo hello");
/// let job_id = submit_mesh_job(job, submitter_did).await?;
/// ```
/// 
/// # Errors
/// 
/// Returns `RuntimeError` if:
/// - Job specification is invalid
/// - Submitter has insufficient mana
/// - Network is unavailable
pub async fn submit_mesh_job(job: MeshJob, submitter: Did) -> Result<JobId, RuntimeError> {
    // Implementation
}
```

#### Architecture Documentation

Update architecture documentation when making structural changes:

- Add new components to `docs/ARCHITECTURE.md`
- Update data flow diagrams
- Document new interfaces and protocols

### Error Handling

Follow these error handling patterns:

```rust
// Use specific error types
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("Insufficient mana: required {required}, available {available}")]
    InsufficientMana { required: u64, available: u64 },
    
    #[error("Invalid job specification: {reason}")]
    InvalidJob { reason: String },
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

// Chain errors appropriately
fn process_job(job: MeshJob) -> Result<JobId, MeshError> {
    validate_job(&job)
        .map_err(|e| MeshError::InvalidJob { reason: e.to_string() })?;
    
    // Continue processing
}
```

## Debugging and Troubleshooting

### Logging Configuration

Configure logging for development:

```rust
// In your development code
use tracing::{info, warn, error, debug, trace};

// Use structured logging
info!(
    job_id = %job.id,
    submitter = %job.submitter,
    "Job submitted successfully"
);

// Use appropriate levels
trace!("Detailed debugging information");
debug!("Development debugging info");
info!("Important state changes");
warn!("Recoverable issues");
error!("Critical failures");
```

### Debug Configuration

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging for specific crates
RUST_LOG=icn_runtime=trace,icn_mesh=debug cargo run

# Enable backtraces
RUST_BACKTRACE=1 cargo run
```

### Common Debug Commands

```bash
# Run with debug information
cargo run --bin icn-node -- --log-level debug

# Run tests with output
cargo test -- --nocapture

# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Profile memory usage
cargo run --bin icn-node --profile dev
```

### Debugging Tools

#### GDB Integration

```bash
# Build with debug symbols
cargo build --profile dev

# Run with GDB
gdb target/debug/icn-node
```

#### Valgrind for Memory Issues

```bash
# Install valgrind
sudo apt install valgrind

# Run with valgrind
valgrind --leak-check=full ./target/debug/icn-node
```

## Performance Optimization

### Profiling

```bash
# CPU profiling
cargo build --release
perf record ./target/release/icn-node
perf report

# Memory profiling
cargo build --release
valgrind --tool=massif ./target/release/icn-node
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench mesh_performance
```

### Performance Best Practices

1. **Async Operations**: Use async/await for I/O operations
2. **Memory Management**: Avoid unnecessary allocations
3. **Caching**: Use appropriate caching strategies
4. **Batch Processing**: Group operations when possible
5. **Connection Pooling**: Reuse network connections

## Contribution Guidelines

### Before Contributing

1. **Check existing issues**: Look for related work
2. **Discuss major changes**: Open an issue for discussion
3. **Read the code**: Understand the existing patterns
4. **Run tests**: Ensure everything works locally

### Pull Request Process

1. **Create a feature branch** from `develop`
2. **Implement your changes** following code standards
3. **Add comprehensive tests** for new functionality
4. **Update documentation** as needed
5. **Run quality checks** (`just validate`)
6. **Submit a pull request** with clear description

### Code Review Guidelines

#### For Authors

- Keep PRs focused and reasonably sized
- Write clear commit messages
- Respond to feedback promptly
- Update based on review comments

#### For Reviewers

- Focus on code quality and correctness
- Suggest improvements, not just point out issues
- Be constructive and respectful
- Test locally if needed

## Release Process

### Version Management

We use semantic versioning (SemVer):

- **Major** (X.0.0): Breaking changes
- **Minor** (0.X.0): New features, backward compatible
- **Patch** (0.0.X): Bug fixes, backward compatible

### Release Steps

1. **Prepare release branch**:
```bash
git checkout develop
git pull origin develop
git checkout -b release/v1.2.0
```

2. **Update version numbers**:
```bash
# Update Cargo.toml files
# Update CHANGELOG.md
# Update documentation versions
```

3. **Final testing**:
```bash
just test-all
just test-e2e
```

4. **Create release**:
```bash
git tag v1.2.0
git push origin v1.2.0
```

5. **Deploy to crates.io** (if applicable):
```bash
cargo publish --dry-run
cargo publish
```

## Common Development Tasks

### Adding a New Crate

1. **Create crate structure**:
```bash
mkdir crates/icn-newcrate
cd crates/icn-newcrate
cargo init --lib
```

2. **Update workspace Cargo.toml**:
```toml
[workspace]
members = [
    # ... existing crates
    "crates/icn-newcrate",
]
```

3. **Add README and documentation**:
```bash
# Create README.md
# Add rustdoc comments
# Update architecture documentation
```

### Implementing New Features

1. **Design phase**:
   - Write RFC if significant
   - Design interfaces and data structures
   - Plan testing strategy

2. **Implementation phase**:
   - Start with tests (TDD)
   - Implement incrementally
   - Update documentation

3. **Integration phase**:
   - Test with other components
   - Update integration tests
   - Validate performance

### Updating Dependencies

1. **Check for updates**:
```bash
cargo outdated
```

2. **Update Cargo.toml**:
```bash
# Update version constraints
# Test with new versions
```

3. **Validate compatibility**:
```bash
just test-all
just test-e2e
```

### Working with Features

Enable/disable features during development:

```bash
# Build with specific features
cargo build --features "libp2p,persist-sqlite"

# Build without default features
cargo build --no-default-features --features "minimal"

# Test with all features
cargo test --all-features
```

## IDE Configuration

### VS Code

Create `.vscode/settings.json`:

```json
{
    "rust-analyzer.server.path": "rust-analyzer",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.inlayHints.typeHints": false,
    "rust-analyzer.lens.methodReferences": true,
    "editor.formatOnSave": true,
    "files.watcherExclude": {
        "**/target/**": true
    }
}
```

### IntelliJ IDEA

1. Install Rust plugin
2. Configure Rust toolchain in settings
3. Enable Clippy integration
4. Set up code formatting

## Troubleshooting Common Issues

### Build Issues

**Problem**: Compilation fails with linking errors
**Solution**: Install system dependencies, check Rust version

**Problem**: Out of memory during compilation
**Solution**: Use `export CARGO_BUILD_JOBS=1` or increase RAM

### Runtime Issues

**Problem**: Node fails to start
**Solution**: Check configuration, verify file permissions

**Problem**: Network connectivity issues
**Solution**: Check firewall settings, verify bootstrap peers

### Test Issues

**Problem**: Tests fail intermittently
**Solution**: Check for race conditions, use deterministic test data

**Problem**: Integration tests timeout
**Solution**: Increase timeouts, check resource constraints

## Additional Resources

- [Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://docs.rs/tokio/)
- [libp2p Documentation](https://docs.libp2p.io/)
- [ICN Architecture Guide](ARCHITECTURE.md)
- [ICN API Reference](API.md)

---

This guide is a living document. Please update it as the project evolves and new development patterns emerge. 