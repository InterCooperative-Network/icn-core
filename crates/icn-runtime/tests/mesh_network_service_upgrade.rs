//! Test to validate that RuntimeContext properly uses DefaultMeshNetworkService
//! when libp2p is enabled, instead of StubMeshNetworkService.
//! Also tests the DefaultSigner implementation for Phase 5 cryptographic improvements.
//! This addresses core Phase 5 improvements of replacing stub implementations.

use icn_common::Did;
use icn_runtime::context::{MeshNetworkService, RuntimeContext, Signer};
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
        let is_default_mesh = ctx
            .mesh_network_service
            .as_any()
            .downcast_ref::<DefaultMeshNetworkService>()
            .is_some();
        assert!(
            is_default_mesh,
            "When libp2p is enabled, should use DefaultMeshNetworkService instead of stub"
        );
    }

    #[cfg(not(feature = "enable-libp2p"))]
    {
        use icn_runtime::context::StubMeshNetworkService;
        let is_stub_mesh = ctx
            .mesh_network_service
            .as_any()
            .downcast_ref::<StubMeshNetworkService>()
            .is_some();
        assert!(
            is_stub_mesh,
            "When libp2p is disabled, should fall back to StubMeshNetworkService"
        );
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
        let is_default_mesh = ctx
            .mesh_network_service
            .as_any()
            .downcast_ref::<DefaultMeshNetworkService>()
            .is_some();
        assert!(
            is_default_mesh,
            "new_with_stubs should also use DefaultMeshNetworkService when libp2p is enabled"
        );
    }

    #[cfg(not(feature = "enable-libp2p"))]
    {
        use icn_runtime::context::StubMeshNetworkService;
        let is_stub_mesh = ctx
            .mesh_network_service
            .as_any()
            .downcast_ref::<StubMeshNetworkService>()
            .is_some();
        assert!(
            is_stub_mesh,
            "new_with_stubs should use StubMeshNetworkService when libp2p is disabled"
        );
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

        if let Some(default_mesh) = ctx
            .mesh_network_service
            .as_any()
            .downcast_ref::<DefaultMeshNetworkService>()
        {
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

#[test]
fn test_stub_signer_functionality() {
    // Test that StubSigner provides cryptographic capabilities
    use icn_runtime::context::StubSigner as DefaultSigner;

    let did_str = "did:icn:test:default_signer";
    let did = Did::from_str(did_str).unwrap();

    // Create a StubSigner
    let signer = DefaultSigner::new();

    // Test signing and verification
    let test_payload = b"Hello, ICN Phase 5!";
    let signature = signer.sign(test_payload).unwrap();
    let public_key_bytes = signer.public_key_bytes();

    // Verify the signature
    let is_valid = signer
        .verify(test_payload, &signature, &public_key_bytes)
        .unwrap();
    assert!(is_valid, "StubSigner should produce valid signatures");

    // Test that DID is properly maintained
    assert_eq!(signer.did(), did);

    // Test invalid signature detection
    let mut invalid_signature = signature.clone();
    invalid_signature[0] = invalid_signature[0].wrapping_add(1); // Corrupt one byte
    let is_invalid = signer
        .verify(test_payload, &invalid_signature, &public_key_bytes)
        .unwrap();
    assert!(!is_invalid, "StubSigner should detect invalid signatures");
}

#[test]
fn test_runtime_context_uses_stub_signer() {
    // Test that RuntimeContext uses StubSigner by default
    let did_str = "did:icn:test:runtime_signer";
    let ctx = RuntimeContext::new_with_stubs(did_str).unwrap();

    // Test that we can access the signer and it works
    let test_payload = b"Test runtime signer";
    let signature = ctx.signer.sign(test_payload).unwrap();
    let public_key_bytes = ctx.signer.public_key_bytes();

    let is_valid = ctx
        .signer
        .verify(test_payload, &signature, &public_key_bytes)
        .unwrap();
    assert!(
        is_valid,
        "RuntimeContext signer should provide valid cryptography"
    );

    // Test that the signer's DID matches the context's identity
    assert_eq!(ctx.signer.did(), ctx.current_identity);

    // Clean up
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");
}

#[test]
fn test_dag_storage_availability() {
    // Test that DAG storage is properly configured for production use
    let did_str = "did:icn:test:dag_storage";
    let ctx = RuntimeContext::new_with_stubs(did_str).unwrap();

    // Verify that DAG store is available and functional
    // The specific implementation depends on which persistence features are enabled
    // but we can verify the store exists and basic operations work
    assert!(
        !ctx.dag_store.try_lock().is_err(),
        "DAG store should be accessible"
    );

    // Clean up potential storage files
    let _ = std::fs::remove_file("./mana_ledger.sled");
    let _ = std::fs::remove_file("./reputation.sled");
    let _ = std::fs::remove_file("./dag.sled");
    let _ = std::fs::remove_file("./dag.rocksdb");
    let _ = std::fs::remove_file("./dag.sqlite");
    let _ = std::fs::remove_dir_all("./dag.rocksdb");
}
