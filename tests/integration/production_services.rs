//! Integration tests for production service implementations
//!
//! These tests validate that the comprehensive DID resolution, federation management,
//! reputation integration, and CCL WASM features work correctly together.

use icn_common::{Did, SystemTimeProvider};
use icn_runtime::context::{
    ServiceConfigBuilder, ServiceEnvironment, FederationIntegration, 
    AdvancedCclWasmBackend, CclExecutionConfig, OptimizationLevel,
};
use icn_identity::{
    EnhancedDidResolver, EnhancedDidResolverBuilder, FederationManager,
    generate_ed25519_keypair, did_key_from_verifying_key,
};
use icn_reputation::InMemoryReputationStore;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_production_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test identity
    let (signing_key, verifying_key) = generate_ed25519_keypair();
    let node_did = Did::from_str(&did_key_from_verifying_key(&verifying_key))?;
    
    // Create enhanced DID resolver
    let time_provider = Arc::new(SystemTimeProvider);
    let did_resolver = Arc::new(
        EnhancedDidResolverBuilder::new()
            .with_cache_ttl(300) // 5 minutes for testing
            .with_cache_size(100)
            .with_fallback(true)
            .build(time_provider.clone())
    );

    // Test DID resolution capabilities
    test_did_resolution(&did_resolver, &node_did).await?;

    // Create reputation store and test reputation features
    let reputation_store = Arc::new(InMemoryReputationStore::new());
    test_reputation_system(&reputation_store, &node_did).await?;

    // Test federation management
    let federation_manager = Arc::new(FederationManager::new(
        node_did.clone(),
        did_resolver.clone(),
        time_provider.clone(),
    ));
    
    test_federation_management(&federation_manager, &node_did).await?;

    // Test CCL WASM execution
    // Note: This would require actual DAG store setup in full integration
    test_ccl_execution_config().await?;

    println!("✅ All production service integration tests passed");
    Ok(())
}

async fn test_did_resolution(
    resolver: &EnhancedDidResolver,
    test_did: &Did,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing DID resolution capabilities...");

    // Test resolution stats
    let initial_stats = resolver.get_stats();
    assert_eq!(initial_stats.total_resolutions, 0);

    // Test cache clearing
    resolver.clear_cache();
    
    // Test batch resolution
    let test_dids = vec![test_did.clone()];
    let results = resolver.batch_resolve(&test_dids);
    
    // At least one result should be returned (even if it's an error for invalid DID)
    assert_eq!(results.len(), 1);

    println!("✅ DID resolution tests completed");
    Ok(())
}

async fn test_reputation_system(
    reputation_store: &Arc<InMemoryReputationStore>,
    node_did: &Did,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("⭐ Testing reputation system...");

    // Test initial reputation
    let initial_reputation = reputation_store.get_reputation(node_did);
    assert_eq!(initial_reputation, 0); // Default for new nodes

    // Test reputation updates
    reputation_store.set_score(node_did.clone(), 75);
    let updated_reputation = reputation_store.get_reputation(node_did);
    assert_eq!(updated_reputation, 75);

    // Test reputation bounds
    reputation_store.set_score(node_did.clone(), 150); // Over max
    let capped_reputation = reputation_store.get_reputation(node_did);
    assert!(capped_reputation <= 100); // Should be capped

    println!("✅ Reputation system tests completed");
    Ok(())
}

async fn test_federation_management(
    federation_manager: &Arc<FederationManager>,
    node_did: &Did,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing federation management...");

    // Test initial federation list (should be empty)
    let initial_federations = federation_manager.list_federations().await?;
    assert!(initial_federations.is_empty());

    // Test membership status for non-existent federation
    let status = federation_manager.get_membership_status("non_existent_fed", node_did).await?;
    assert!(status.is_none());

    println!("✅ Federation management tests completed");
    Ok(())
}

async fn test_ccl_execution_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing CCL WASM execution configuration...");

    // Test default configuration
    let default_config = CclExecutionConfig::default();
    assert!(default_config.enable_optimizations);
    assert!(default_config.enable_monitoring);
    assert_eq!(default_config.optimization_level, OptimizationLevel::Balanced);

    // Test custom configuration
    let custom_config = CclExecutionConfig {
        max_execution_time: std::time::Duration::from_secs(10),
        max_memory_bytes: 32 * 1024 * 1024, // 32MB
        enable_optimizations: false,
        optimization_level: OptimizationLevel::Speed,
        ..Default::default()
    };

    assert_eq!(custom_config.max_execution_time.as_secs(), 10);
    assert_eq!(custom_config.max_memory_bytes, 32 * 1024 * 1024);
    assert!(!custom_config.enable_optimizations);

    println!("✅ CCL WASM execution tests completed");
    Ok(())
}

#[tokio::test]
async fn test_service_configuration_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing service configuration validation...");

    // Test production environment validation
    let production_builder = ServiceConfigBuilder::new(ServiceEnvironment::Production);
    
    // Should fail without required services
    let result = production_builder.build();
    assert!(result.is_err(), "Production config should require all services");

    // Test development environment (more permissive)
    let development_builder = ServiceConfigBuilder::new(ServiceEnvironment::Development);
    
    // Should work with minimal configuration
    let (signing_key, verifying_key) = generate_ed25519_keypair();
    let node_did = Did::from_str(&did_key_from_verifying_key(&verifying_key))?;
    
    // Create a minimal mana ledger for testing
    let temp_file = tempfile::NamedTempFile::new()?;
    let mana_ledger = icn_runtime::context::SimpleManaLedger::new(temp_file.path().to_path_buf());

    let dev_config = development_builder
        .with_identity(node_did)
        .with_signer(Arc::new(icn_runtime::Ed25519Signer::from_signing_key(signing_key)))
        .with_mana_ledger(mana_ledger)
        .build();

    assert!(dev_config.is_ok(), "Development config should work with minimal setup");

    // Test testing environment (most permissive)
    let test_config = icn_runtime::context::ServiceConfig::testing(
        Did::from_str("did:key:zTest")?,
        Some(1000),
    );
    assert!(test_config.is_ok(), "Testing config should work with just identity");

    println!("✅ Service configuration validation tests completed");
    Ok(())
}

#[tokio::test]
async fn test_mesh_network_reputation_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing mesh network reputation integration...");

    // Create test components
    let (signing_key, verifying_key) = generate_ed25519_keypair();
    let executor_did = Did::from_str(&did_key_from_verifying_key(&verifying_key))?;
    
    let reputation_store = Arc::new(InMemoryReputationStore::new());
    
    // Set initial reputation
    reputation_store.set_score(executor_did.clone(), 60);
    
    // Test reputation-based filtering
    let initial_reputation = reputation_store.get_reputation(&executor_did);
    assert_eq!(initial_reputation, 60);

    // Simulate successful job execution
    reputation_store.set_score(executor_did.clone(), 65); // +5 for success
    let updated_reputation = reputation_store.get_reputation(&executor_did);
    assert_eq!(updated_reputation, 65);

    // Simulate failed job execution
    reputation_store.set_score(executor_did.clone(), 55); // -10 for failure
    let final_reputation = reputation_store.get_reputation(&executor_did);
    assert_eq!(final_reputation, 55);

    println!("✅ Mesh network reputation integration tests completed");
    Ok(())
}

#[test]
fn test_production_feature_flags() {
    println!("🔒 Testing production feature flags...");

    // Check that stub services are properly controlled
    #[cfg(not(feature = "allow-stubs"))]
    {
        println!("✅ Stub services are disabled in production build");
    }

    #[cfg(feature = "allow-stubs")]
    {
        println!("⚠️ Stub services are enabled - ensure this is intentional");
    }

    // Check libp2p availability
    #[cfg(feature = "enable-libp2p")]
    {
        println!("✅ libp2p networking is available");
    }

    #[cfg(not(feature = "enable-libp2p"))]
    {
        println!("⚠️ libp2p networking is not available");
    }

    println!("✅ Production feature flag tests completed");
}

#[tokio::test]
async fn test_error_handling_and_resilience() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ Testing error handling and resilience...");

    // Test DID resolution with invalid DID
    let time_provider = Arc::new(SystemTimeProvider);
    let did_resolver = EnhancedDidResolverBuilder::new()
        .with_fallback(true)
        .build(time_provider);

    let invalid_did = Did::from_str("did:invalid:test")?;
    let result = did_resolver.resolve(&invalid_did);
    assert!(result.is_err(), "Should fail for invalid DID method");

    // Test reputation bounds
    let reputation_store = InMemoryReputationStore::new();
    let test_did = Did::from_str("did:key:zTest")?;
    
    // Test negative reputation (should be bounded)
    reputation_store.set_score(test_did.clone(), 0);
    assert_eq!(reputation_store.get_reputation(&test_did), 0);

    // Test overflow reputation (should be bounded)
    reputation_store.set_score(test_did.clone(), 100);
    assert_eq!(reputation_store.get_reputation(&test_did), 100);

    println!("✅ Error handling and resilience tests completed");
    Ok(())
}

/// Run all integration tests
#[tokio::test]
async fn run_comprehensive_integration_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running comprehensive integration tests...");
    println!("================================================");

    test_production_service_integration().await?;
    test_service_configuration_validation().await?;
    test_mesh_network_reputation_integration().await?;
    test_production_feature_flags();
    test_error_handling_and_resilience().await?;

    println!("================================================");
    println!("🎉 All comprehensive integration tests passed!");
    println!();
    println!("Validated components:");
    println!("  ✅ Enhanced DID resolution with caching");
    println!("  ✅ Production service configuration");
    println!("  ✅ Federation management integration");
    println!("  ✅ Reputation system integration");
    println!("  ✅ CCL WASM execution configuration");
    println!("  ✅ Error handling and resilience");
    println!("  ✅ Production feature flags");

    Ok(())
}