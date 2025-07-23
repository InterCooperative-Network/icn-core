# RuntimeContext Production Defaults Migration Guide

This guide explains the new production-by-default RuntimeContext API and how to migrate from the old stub-based constructors.

## Overview

The RuntimeContext has been updated to use production services by default, with explicit methods for testing configurations. This ensures that production deployments are secure and performant by default, while making testing configurations explicit and obvious.

## New API Structure

### Production Constructors (Default)

```rust
// Simple production constructor (requires libp2p feature or explicit services)
let ctx = RuntimeContext::new()?;

// Async production constructor with full libp2p support
let ctx = RuntimeContext::new_async().await?;

// Production constructor with explicit services
let ctx = RuntimeContext::new_with_services(
    identity, network_service, signer, did_resolver, 
    dag_store, mana_ledger, reputation_store, policy_enforcer
)?;
```

### Testing Constructors (Explicit)

```rust
// Explicit testing constructor - clear and obvious
let ctx = RuntimeContext::new_for_testing(test_did, Some(initial_mana))?;

// Deprecated but still functional (forwards to new_for_testing)
#[allow(deprecated)]
let ctx = RuntimeContext::new_testing(test_did, initial_mana)?;
```

## Migration Examples

### Before (Stub by Default)
```rust
// Old way - stub services by default (unsafe for production)
let ctx = RuntimeContext::new_with_stubs("did:key:zExample")?;

// Old way - required manual configuration for production
let ctx = RuntimeContext::new_for_production(
    identity, network, signer, resolver, dag, mana, reputation, policy
)?;
```

### After (Production by Default)
```rust
// New way - production services by default (safe)
let ctx = RuntimeContext::new()?; // Or new_async().await?

// New way - explicit testing (clear intent)  
let test_did = Did::from_str("did:key:zExample")?;
let ctx = RuntimeContext::new_for_testing(test_did, None)?;
```

## Feature Flags

The new API supports feature flags for different deployment scenarios:

### Production Features
- `enable-libp2p`: Enables automatic libp2p network service creation
- `persist-sled`: Enables Sled persistent storage backend  
- `persist-rocksdb`: Enables RocksDB persistent storage backend
- `production`: Strict production mode (prevents stub usage)
- `allow-stubs`: Allow stub services in production builds (for testing)

### Development Features
```toml
[dependencies]
icn-runtime = { version = "0.3.0", features = ["enable-libp2p", "persist-sled"] }
```

### Testing Configuration
```toml
[dev-dependencies]
icn-runtime = { version = "0.3.0", features = ["allow-stubs"] }
```

## Service Validation

The new API includes comprehensive service validation:

```rust
// Automatic validation for production contexts
let ctx = RuntimeContext::new()?; // Validates services automatically

// Manual validation
ctx.validate_production_services()?; // Explicit validation

// Service configuration validation
let config = ServiceConfig::production(/* params */)?; // Auto-validates
config.validate_production_services()?; // Explicit validation
```

## Error Messages

The new API provides clear, actionable error messages:

```
❌ PRODUCTION ERROR: Stub mesh network service detected in production context. 
   Use RuntimeContext::new() with real network service or enable 'enable-libp2p' feature.

❌ Production environment requires libp2p feature enabled. 
   Enable the 'enable-libp2p' feature.

❌ Cannot create libp2p network service in synchronous context. 
   Use new_async() or new_with_network_service() instead.
```

## Breaking Changes

### Removed/Changed
- `RuntimeContext::new()` now requires production services (was not available before)
- `new_testing()` method is deprecated (use `new_for_testing()` instead)
- Service configuration validates production services by default

### 🔒 Critical Security Fix
- **FIXED**: `RuntimeContext::new_with_identity_and_storage()` and `new_async_with_identity_and_storage()` now generate cryptographically matched identity DIDs and signers
- **Impact**: Previously, these methods created non-functional RuntimeContexts where signatures would never verify
- **Result**: RuntimeContexts created with `identity: None` now have working cryptographic operations

### Deprecated (Still Work)
- `RuntimeContext::new_with_stubs()` → Use `RuntimeContext::new_for_testing()`
- `RuntimeContext::new_with_stubs_and_mana()` → Use `RuntimeContext::new_for_testing()`
- `RuntimeContext::new_testing()` → Use `RuntimeContext::new_for_testing()`

## Migration Checklist

### For Production Code
- [ ] Replace `new_with_stubs()` calls with `new()` or `new_async()`
- [ ] Enable `enable-libp2p` feature for automatic networking
- [ ] Enable persistent storage features (`persist-sled` or `persist-rocksdb`)
- [ ] Remove explicit stub service configurations
- [ ] Add production service validation calls

### For Test Code  
- [ ] Replace `new_with_stubs()` with `new_for_testing()`
- [ ] Convert string DIDs to `Did` objects
- [ ] Add `#[allow(deprecated)]` for old methods (if keeping them)
- [ ] Enable `allow-stubs` feature in dev-dependencies

### For Library Code
- [ ] Use explicit service constructors (`new_with_services()`)
- [ ] Provide both production and testing factory methods
- [ ] Document which constructor to use for different scenarios

## Common Patterns

### Testing Helper Functions
```rust
// Helper for creating test contexts
fn create_test_context(initial_mana: Option<u64>) -> Arc<RuntimeContext> {
    let test_did = Did::from_str("did:key:zTestExample").unwrap();
    RuntimeContext::new_for_testing(test_did, initial_mana)
        .expect("Failed to create test context")
}
```

### Production Configuration
```rust
// Production setup with explicit services
async fn create_production_context() -> Result<Arc<RuntimeContext>, CommonError> {
    // Option 1: Simple async setup
    RuntimeContext::new_async().await
    
    // Option 2: Manual service configuration
    let identity = load_or_generate_identity()?;
    let network = create_libp2p_service().await?;
    let storage = create_persistent_storage()?;
    // ... other services
    
    RuntimeContext::new_with_services(
        identity, network, signer, resolver, 
        storage, mana_ledger, reputation, policy
    )
}
```

### Development Configuration
```rust
// Development with mixed services
let ctx = RuntimeContext::new_for_development(
    identity, signer, mana_ledger,
    Some(network_service), // Real network
    None // Stub storage for faster iteration
)?;
```

## Validation and Debugging

### Check Service Types
```rust
// Validate production services
match ctx.validate_production_services() {
    Ok(_) => println!("✅ Using production services"),
    Err(e) => println!("⚠️ Not suitable for production: {}", e),
}
```

### Debug Service Configuration
```rust
// Check what services are being used
println!("Mesh network service: {:?}", ctx.mesh_network_service);
println!("DAG store: {:?}", ctx.dag_store);
```

### 🔒 Verify Cryptographic Functionality
```rust
// Test that identity and signer are properly matched
async fn test_crypto_functionality(ctx: &RuntimeContext) -> Result<(), Box<dyn std::error::Error>> {
    use icn_identity::Signer;
    
    // Create a test message
    let test_message = b"test signature verification";
    
    // Sign with the context's signer
    let signature = ctx.signer.sign(test_message)?;
    
    // Extract public key from the identity DID and verify
    let verifying_key = icn_identity::did_key_to_verifying_key(&ctx.current_identity)?;
    verifying_key.verify(test_message, &signature)?;
    
    println!("✅ Cryptographic operations working correctly");
    Ok(())
}
```

## Best Practices

1. **Always use explicit methods**: `new_for_testing()` instead of deprecated methods
2. **Enable appropriate features**: `enable-libp2p` for production, `allow-stubs` for testing
3. **Validate in production**: Call `validate_production_services()` after creation
4. **Use async constructors**: `new_async()` for full libp2p support
5. **Document configuration**: Clearly document which constructor is appropriate

## Support

For questions or issues with the migration:

1. Check the error messages - they provide specific guidance
2. Look at the examples in `examples/production_defaults.rs`
3. Review the test cases in the test suite
4. File an issue if you encounter migration problems

## Version Compatibility

- **v0.2.x**: Deprecated methods available, warnings emitted
- **v0.3.x**: New API available, deprecated methods still work
- **v0.4.x**: Deprecated methods may be removed (future)

The new API maintains backward compatibility while providing a clear upgrade path.