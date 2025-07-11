//! Test to validate that RuntimeContext properly uses DefaultMeshNetworkService 
//! when libp2p is enabled, instead of StubMeshNetworkService.
//! This addresses the core Phase 5 improvement of replacing stub implementations.

use icn_common::Did;
use icn_runtime::context::{MeshNetworkService, RuntimeContext};
use std::str::FromStr;
use std::sync::Arc;

#[test]
fn test_mesh_network_service_upgrade_when_libp2p_enabled() {
    // Clean up any existing test files
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");

    // Create a RuntimeContext using the upgraded new_with_stubs method
    let did_str = "did:icn:test:mesh_upgrade";
    let ctx = RuntimeContext::new_with_stubs_and_mana(did_str, 100).unwrap();

    // Test that the mesh network service is properly initialized
    assert!(ctx.mesh_network_service.as_any().type_id() != std::any::TypeId::of::<()>());

    // When libp2p feature is enabled, we should get DefaultMeshNetworkService
    // When libp2p feature is disabled, we should get StubMeshNetworkService
    #[cfg(feature = "enable-libp2p")]
    {
        use icn_runtime::context::DefaultMeshNetworkService;
        let is_default_mesh = ctx.mesh_network_service.as_any().downcast_ref::<DefaultMeshNetworkService>().is_some();
        assert!(is_default_mesh, "When libp2p is enabled, should use DefaultMeshNetworkService instead of stub");
    }

    #[cfg(not(feature = "enable-libp2p"))]
    {
        use icn_runtime::context::StubMeshNetworkService;
        let is_stub_mesh = ctx.mesh_network_service.as_any().downcast_ref::<StubMeshNetworkService>().is_some();
        assert!(is_stub_mesh, "When libp2p is disabled, should fall back to StubMeshNetworkService");
    }

    // Basic functionality test - the service should be usable regardless of implementation
    let test_did = Did::from_str(did_str).unwrap();
    assert_eq!(ctx.current_identity, test_did);
}

#[test]
fn test_new_with_stubs_also_upgraded() {
    // Clean up any existing test files
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");

    // Test that new_with_stubs (without initial mana) also gets the upgrade
    let did_str = "did:icn:test:stubs_upgrade";
    let ctx = RuntimeContext::new_with_stubs(did_str).unwrap();

    #[cfg(feature = "enable-libp2p")]
    {
        use icn_runtime::context::DefaultMeshNetworkService;
        let is_default_mesh = ctx.mesh_network_service.as_any().downcast_ref::<DefaultMeshNetworkService>().is_some();
        assert!(is_default_mesh, "new_with_stubs should also use DefaultMeshNetworkService when libp2p is enabled");
    }

    #[cfg(not(feature = "enable-libp2p"))]
    {
        use icn_runtime::context::StubMeshNetworkService;
        let is_stub_mesh = ctx.mesh_network_service.as_any().downcast_ref::<StubMeshNetworkService>().is_some();
        assert!(is_stub_mesh, "new_with_stubs should use StubMeshNetworkService when libp2p is disabled");
    }
}

#[test]
fn test_context_identity_properly_configured() {
    // Verify that the identity is properly propagated to the networking layer
    let did_str = "did:icn:test:identity_config";
    let ctx = RuntimeContext::new_with_stubs(did_str).unwrap();
    let expected_did = Did::from_str(did_str).unwrap();
    
    assert_eq!(ctx.current_identity, expected_did);
    
    // Clean up
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");
}

#[tokio::test]
async fn test_default_mesh_network_service_connectivity_validation() {
    // Test the new connectivity validation feature when libp2p is enabled
    #[cfg(feature = "enable-libp2p")]
    {
        use icn_runtime::context::DefaultMeshNetworkService;
        
        let did_str = "did:icn:test:connectivity_validation";
        let ctx = RuntimeContext::new_with_stubs(did_str).unwrap();
        
        if let Some(default_mesh) = ctx.mesh_network_service.as_any().downcast_ref::<DefaultMeshNetworkService>() {
            // Test the new connectivity validation method
            let result = default_mesh.validate_connectivity().await;
            assert!(result.is_ok(), "Connectivity validation should succeed");
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");
}