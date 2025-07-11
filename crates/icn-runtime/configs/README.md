# ICN Runtime Configuration

This directory contains sample configuration files for the ICN runtime system. The configuration system supports both TOML and JSON formats and provides environment-specific defaults.

## Configuration Files

### Production Configuration (`production.toml`)
- **Purpose**: Production-ready settings optimized for stability and performance
- **Key Features**:
  - Persistent storage (RocksDB for DAG, file-based for other stores)
  - Conservative resource limits
  - Longer timeouts for network operations
  - Enhanced security settings
  - Comprehensive logging and metrics

### Development Configuration (`development.toml`, `development.json`)
- **Purpose**: Development-friendly settings for local testing
- **Key Features**:
  - In-memory storage for fast iteration
  - mDNS discovery enabled for easy peer discovery
  - Shorter timeouts for faster feedback
  - Higher mana regeneration rates for testing
  - Debug logging enabled

### Testing Configuration (`testing.toml`)
- **Purpose**: Optimized for automated testing environments
- **Key Features**:
  - All in-memory storage
  - No network operations
  - Very short timeouts
  - High mana balances and regeneration for testing
  - Minimal resource usage

## Usage

### Loading Configuration

```rust
use icn_runtime::RuntimeConfig;

// Load from TOML file
let config = RuntimeConfig::from_file("configs/production.toml")?;

// Load from JSON file
let config = RuntimeConfig::from_file("configs/development.json")?;

// Create programmatically
let config = RuntimeConfig::production();
let config = RuntimeConfig::development();
let config = RuntimeConfig::testing();
```

### Creating a RuntimeContext from Configuration

```rust
use icn_runtime::{RuntimeConfig, RuntimeContext};

// Load configuration
let mut config = RuntimeConfig::from_file("configs/production.toml")?;

// Expand paths (resolves ~ and environment variables)
config.expand_paths()?;

// Validate configuration
config.validate()?;

// Convert to service configuration
let service_config = config.to_service_config()?;

// Create runtime context
let runtime_context = RuntimeContext::from_service_config(service_config)?;
```

### Environment-Specific Usage

```rust
use icn_runtime::{RuntimeConfig, RuntimeContext};

// Production environment
let config = RuntimeConfig::production();
let service_config = config.to_service_config()?;
let runtime_context = RuntimeContext::from_service_config(service_config)?;

// Development environment with custom settings
let mut config = RuntimeConfig::development();
config.storage.mana_ledger.initial_mana = 50000;
config.network.listen_addresses = vec!["/ip4/0.0.0.0/tcp/4001".to_string()];
let service_config = config.to_service_config()?;
let runtime_context = RuntimeContext::from_service_config(service_config)?;

// Testing environment
let config = RuntimeConfig::testing();
let service_config = config.to_service_config()?;
let runtime_context = RuntimeContext::from_service_config(service_config)?;
```

## Configuration Structure

### Environment Settings
- `environment_type`: "production", "development", or "testing"
- `debug`: Enable debug features
- `log_level`: Logging verbosity
- `metrics`: Enable metrics collection

### Identity Configuration
- `node_did`: Decentralized identifier for this node
- `key_store`: Key storage configuration (file, HSM, or stub)
- `did_resolver`: DID resolution configuration

### Network Configuration
- `listen_addresses`: P2P network listen addresses
- `bootstrap_peers`: Known peers to connect to on startup
- `enable_mdns`: Enable mDNS peer discovery
- `timeouts`: Network operation timeouts
- `connection_limits`: Maximum connection limits

### Storage Configuration
- `data_dir`: Base directory for all data storage
- `dag_store`: DAG storage configuration
- `mana_ledger`: Mana ledger database configuration
- `reputation_store`: Reputation storage configuration

### Governance Configuration
- `enabled`: Enable governance features
- `voting`: Voting parameters and costs
- `proposals`: Proposal creation and validation parameters

### Runtime Configuration
- `default_receipt_wait_ms`: Default timeout for receipt waiting
- `max_job_queue_size`: Maximum pending jobs
- `max_concurrent_jobs`: Maximum concurrent job execution
- `job_execution_timeout_ms`: Job execution timeout
- `cleanup_interval_ms`: Cleanup task interval

## Customization

### Environment Variables
You can override configuration values using environment variables:

```bash
# Override log level
export ICN_LOG_LEVEL=trace

# Override data directory
export ICN_DATA_DIR=/custom/data/path

# Override mana settings
export ICN_INITIAL_MANA=50000
export ICN_MANA_REGENERATION_RATE=2.0
```

### Configuration Validation
The configuration system includes comprehensive validation:
- DID format validation
- Network address validation
- Path validation
- Parameter range validation
- Environment-specific requirement validation

### Path Expansion
The configuration system automatically expands paths:
- `~` is expanded to the user's home directory
- Environment variables are supported in paths
- Relative paths are resolved relative to the configuration file

## Best Practices

1. **Use environment-specific configurations**: Don't use production settings for development
2. **Validate configurations**: Always call `validate()` before using a configuration
3. **Expand paths**: Call `expand_paths()` to resolve relative paths and environment variables
4. **Store sensitive data securely**: Use environment variables for sensitive configuration values
5. **Version control**: Keep configuration files in version control but exclude sensitive data
6. **Documentation**: Document custom configuration changes and their purpose

## Troubleshooting

### Common Issues

1. **Invalid DID format**: Ensure the `node_did` follows the correct DID format
2. **Path not found**: Check that storage paths exist and are writable
3. **Network binding failed**: Verify that listen addresses are valid and available
4. **Validation errors**: Check that all required fields are present and valid

### Debug Configuration
To debug configuration issues:

```rust
use icn_runtime::RuntimeConfig;

let config = RuntimeConfig::from_file("path/to/config.toml")?;
println!("Loaded config: {:#?}", config);

// Validate and see detailed error messages
match config.validate() {
    Ok(_) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration validation failed: {}", e),
}
```

For more detailed documentation, see the [ICN Runtime Documentation](https://docs.intercooperative.network/runtime/). 