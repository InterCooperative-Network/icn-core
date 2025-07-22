# DAG Storage Backend Selection Guide

This document explains how to properly configure DAG storage backends for ICN nodes to ensure that stub implementations are never accidentally used in production.

## Problem Addressed

Previously, the `StubDagStore` (in-memory HashMap) could accidentally be used in production contexts where persistent storage was required. This led to:

- Data loss on node restarts
- Inability to maintain DAG integrity across deployments
- Performance issues with large datasets

## Solution: DAG Store Factory

The new `DagStoreFactory` provides type-safe, environment-aware DAG storage backend creation with automatic validation to prevent production misconfigurations.

## Available Storage Backends

### Production Backends (Persistent)

1. **Sled** (Default, requires `persist-sled` feature)
   - Embedded key-value database
   - High performance, ACID compliance
   - Single-node deployments
   - Default choice for production

2. **RocksDB** (requires `persist-rocksdb` feature)
   - High-performance storage engine
   - Optimized for write-heavy workloads
   - Requires C++ toolchain

3. **SQLite** (requires `persist-sqlite` feature) 
   - Single-file database
   - Good for development and small deployments
   - ACID compliance

4. **PostgreSQL** (requires `persist-postgres` feature)
   - Full SQL database
   - Multi-node deployments
   - Advanced querying capabilities

### Testing Backend

5. **Stub** (In-memory only)
   - Fast, deterministic testing
   - **NEVER use in production**
   - Automatically used for `RuntimeContext::new_testing()`

## Usage Examples

### Production Deployment

```rust
use icn_runtime::context::{RuntimeContext, DagStoreFactory};
use std::path::PathBuf;

// Method 1: Automatic backend selection with storage directory
let storage_path = PathBuf::from("/var/lib/icn/dag");
let ctx = RuntimeContext::new_for_production_with_storage(
    current_identity,
    network_service,
    signer,
    storage_path,
    mana_ledger,
)?;

// Method 2: Explicit DAG store creation
let dag_store = DagStoreFactory::create_production(storage_path)?;
let ctx = RuntimeContext::new(
    current_identity,
    network_service, 
    signer,
    did_resolver,
    dag_store,
    mana_ledger,
    reputation_store,
    None,
)?;
```

### Development Setup

```rust
// With persistent storage (recommended)
let storage_path = Some(PathBuf::from("./dev_data/dag"));
let ctx = RuntimeContext::new_development_with_storage(
    current_identity,
    signer,
    mana_ledger,
    network_service,
    storage_path,
)?;

// With stub storage (for quick iteration)
let ctx = RuntimeContext::new_development_with_storage(
    current_identity,
    signer, 
    mana_ledger,
    network_service,
    None, // No storage path = stub store
)?;
```

### Testing

```rust
// Always uses stub services - perfect for unit tests
let ctx = RuntimeContext::new_testing(
    test_identity,
    Some(100), // Initial mana
)?;
```

## Production Validation

The system provides multiple layers of validation to prevent accidental stub usage:

### 1. Factory Validation
```rust
// This will fail if no persistent backend is available
let config = DagStoreConfig::production(storage_path)?;
config.validate_for_production()?; // Ensures not using stub
```

### 2. Runtime Validation
```rust
// Production constructors validate DAG store type
let ctx = RuntimeContext::new(/* ... */)?; // Fails if stub provided

// Manual validation
ctx.validate_production_services()?; // Checks all services
```

### 3. Service Config Validation
```rust
let config = ServiceConfig::production(/* ... */)?;
config.validate()?; // Comprehensive service validation
```

## Feature Configuration

Enable the appropriate persistence features in your `Cargo.toml`:

```toml
[dependencies]
icn-runtime = { 
    version = "0.2.0",
    features = ["persist-sled"] # or persist-rocksdb, persist-sqlite, etc.
}
```

## Backend Selection Priority

The factory automatically selects backends in this order:

1. Sled (if `persist-sled` enabled)
2. RocksDB (if `persist-rocksdb` enabled)  
3. SQLite (if `persist-sqlite` enabled)
4. PostgreSQL (if `persist-postgres` enabled)

## Migration from Legacy Code

### Old (Deprecated)
```rust
// ⚠️  DEPRECATED: Uses stubs, not safe for production
let ctx = RuntimeContext::new_with_stubs("did:key:example")?;
```

### New (Recommended)
```rust
// ✅ PRODUCTION-SAFE: Uses persistent storage
let ctx = RuntimeContext::new_for_production_with_storage(
    Did::from_str("did:key:example")?,
    network_service,
    signer,
    storage_path,
    mana_ledger,
)?;

// ✅ TESTING: Clear intent, stub usage
let ctx = RuntimeContext::new_testing(
    Did::from_str("did:key:example")?,
    Some(100),
)?;
```

## Error Handling

Common error scenarios and solutions:

### No Persistent Backend Available
```
Error: No persistent DAG storage backend available. Enable one of: persist-sled, persist-rocksdb, persist-sqlite
```
**Solution:** Add appropriate feature to `Cargo.toml`

### Stub in Production
```
Error: ❌ PRODUCTION ERROR: Stub DAG store cannot be used in production contexts
```
**Solution:** Use `DagStoreFactory::create_production()` or `RuntimeContext::new_for_production_with_storage()`

### Storage Path Issues
```
Error: Failed to create storage directory /var/lib/icn/dag: Permission denied
```
**Solution:** Ensure directory exists and has proper permissions

## Testing Your Configuration

```rust
#[tokio::test]
async fn test_production_dag_store() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Should successfully create production store
    let store = DagStoreFactory::create_production(temp_dir.path().to_path_buf());
    assert!(store.is_ok());
    
    // Should pass production validation
    let ctx = RuntimeContext::new_for_production_with_storage(/* ... */);
    assert!(ctx.is_ok());
    
    // Should reject stub stores
    ctx.unwrap().validate_production_services().unwrap();
}
```

## Best Practices

1. **Always use factory methods** instead of direct constructor calls
2. **Validate configuration early** in your application startup
3. **Use appropriate environment methods** based on your deployment context
4. **Test production configurations** in staging environments
5. **Enable appropriate features** for your target deployment environment

## Troubleshooting

See the [ICN Runtime documentation](../README.md) for additional troubleshooting information and deployment guides.