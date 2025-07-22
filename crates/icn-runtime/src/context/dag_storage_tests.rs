//! Tests for production vs testing DAG storage validation

use super::*;
use crate::context::{DagStoreFactory, ServiceConfig, ServiceEnvironment};
use icn_common::Did;
use std::str::FromStr;
use tempfile::tempdir;

#[tokio::test]
async fn test_production_dag_store_validation() {
    let test_did = Did::from_str("did:key:zTestProduction").unwrap();
    
    // Test 1: Production should reject stub DAG stores
    let stub_dag_store = DagStoreFactory::create_testing();
    
    // Create a mock network service for testing
    struct MockNetworkService;
    impl icn_network::NetworkService for MockNetworkService {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        
        async fn start(&mut self) -> Result<(), icn_common::CommonError> {
            Ok(())
        }
        
        async fn stop(&mut self) -> Result<(), icn_common::CommonError> {
            Ok(())
        }
        
        fn local_peer_id(&self) -> String {
            "test_peer".to_string()
        }
    }
    
    let network_service = Arc::new(MockNetworkService);
    let signer = Arc::new(crate::context::signers::StubSigner::new());
    let did_resolver = Arc::new(icn_identity::KeyDidResolver);
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let mana_ledger = crate::context::mana::SimpleManaLedger::new(temp_file.path().to_path_buf());
    let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
    
    // Attempt to create production context with stub DAG store should fail
    let result = RuntimeContext::new(
        test_did.clone(),
        network_service.clone(),
        signer.clone(),
        did_resolver.clone(),
        stub_dag_store,
        mana_ledger.clone(),
        reputation_store.clone(),
        None,
    );
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("PRODUCTION ERROR"));
    assert!(error.to_string().contains("Stub DAG store"));
}

#[test]
fn test_dag_store_factory_backends() {
    // Test that we have at least stub backend available
    let available = DagStoreFactory::available_backends();
    assert!(!available.is_empty());
    assert!(available.contains(&crate::context::dag_store_factory::DagStoreBackend::Stub));
}

#[test]
fn test_production_config_validation() {
    let temp_dir = tempdir().unwrap();
    
    // Test production config creation
    if let Ok(config) = crate::context::dag_store_factory::DagStoreConfig::production(temp_dir.path().to_path_buf()) {
        // Should pass validation
        assert!(config.validate_for_production().is_ok());
    }
    
    // Test that stub config fails validation
    let stub_config = crate::context::dag_store_factory::DagStoreConfig::testing();
    assert!(stub_config.validate_for_production().is_err());
}

#[tokio::test]
async fn test_testing_context_uses_stubs() {
    let test_did = Did::from_str("did:key:zTestStubs").unwrap();
    
    // Testing context should always use stubs
    let ctx = RuntimeContext::new_testing(test_did, Some(100)).unwrap();
    
    // Verify it's using stub services
    let dag_store = ctx.dag_store.clone();
    let store = dag_store.lock().await;
    
    // This should be true for stub stores
    assert!(store.as_any().is::<crate::context::stubs::StubDagStore>());
}

#[test] 
fn test_service_config_environment_validation() {
    let test_did = Did::from_str("did:key:zTestServiceConfig").unwrap();
    
    // Testing config should be valid
    let test_config = ServiceConfig::testing(test_did.clone(), Some(100));
    assert!(test_config.is_ok());
    
    let config = test_config.unwrap();
    assert_eq!(config.environment, ServiceEnvironment::Testing);
    
    // Should pass validation for testing
    assert!(config.validate().is_ok());
}

#[cfg(feature = "persist-sled")]
#[test]
fn test_sled_store_creation() {
    let temp_dir = tempdir().unwrap();
    
    let config = crate::context::dag_store_factory::DagStoreConfig {
        backend: crate::context::dag_store_factory::DagStoreBackend::Sled,
        storage_path: temp_dir.path().to_path_buf(),
        options: crate::context::dag_store_factory::DagStoreOptions::default(),
    };
    
    // Should be able to create sled store
    let store = DagStoreFactory::create(&config);
    assert!(store.is_ok());
    
    // Should pass production validation
    assert!(config.validate_for_production().is_ok());
}

#[test]
fn test_deprecated_methods_warning() {
    // Test that deprecated methods still work but emit warnings
    #[allow(deprecated)]
    let ctx = RuntimeContext::new_with_stubs("did:key:zTestDeprecated");
    assert!(ctx.is_ok());
    
    // Should create a testing context using the new implementation
    let ctx = ctx.unwrap();
    
    // Should have the expected identity
    assert!(ctx.current_identity.to_string().contains("zTestDeprecated"));
}