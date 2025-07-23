//! Validation tests for the production-by-default RuntimeContext implementation
//! 
//! These tests validate that the new API correctly enforces production-by-default
//! behavior while maintaining explicit testing configurations.

#[cfg(test)]
mod validation_tests {
    use super::*;
    use icn_runtime::context::{RuntimeContext, ServiceConfig};
    use icn_common::Did;
    use std::str::FromStr;

    #[test]
    fn test_production_by_default_behavior() {
        // The new RuntimeContext::new() should attempt production services
        let result = RuntimeContext::new();
        
        // Should fail in current test environment since we don't have libp2p configured
        // but should give helpful error message
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        
        // Should mention libp2p requirement or synchronous context
        assert!(
            error_msg.contains("libp2p") || error_msg.contains("synchronous"),
            "Error message should guide user: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_explicit_testing_context() {
        let test_did = Did::from_str("did:key:zValidationTest").unwrap();
        
        // new_for_testing should work and be explicit
        let ctx = RuntimeContext::new_for_testing(test_did.clone(), Some(500)).unwrap();
        
        // Should have correct identity and mana
        assert_eq!(ctx.current_identity, test_did);
        assert_eq!(ctx.get_mana(&test_did).await.unwrap(), 500);
        
        // Should fail production validation (uses stub services)
        let validation_result = ctx.validate_production_services();
        assert!(validation_result.is_err());
        assert!(validation_result.unwrap_err().to_string().contains("PRODUCTION ERROR"));
    }

    #[tokio::test] 
    async fn test_deprecated_methods_compatibility() {
        // Deprecated methods should still work but forward to new implementation
        #[allow(deprecated)]
        let ctx1 = RuntimeContext::new_with_stubs("did:key:zDeprecated1").unwrap();
        
        #[allow(deprecated)]
        let ctx2 = RuntimeContext::new_with_stubs_and_mana("did:key:zDeprecated2", 100).unwrap();
        
        // Should create valid contexts
        assert!(ctx1.current_identity.to_string().contains("zDeprecated1"));
        assert!(ctx2.current_identity.to_string().contains("zDeprecated2"));
        
        // Should have correct mana
        assert_eq!(ctx2.get_mana(&ctx2.current_identity).await.unwrap(), 100);
        
        // Both should fail production validation (use stub services)
        assert!(ctx1.validate_production_services().is_err());
        assert!(ctx2.validate_production_services().is_err());
    }

    #[test]
    fn test_service_config_validation() {
        // Production defaults should require explicit configuration  
        let result = ServiceConfig::production_defaults();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("explicit services"));
        
        // Testing defaults should work
        let testing_config = ServiceConfig::testing_defaults();
        assert!(testing_config.is_ok());
        
        let config = testing_config.unwrap();
        assert_eq!(config.environment, icn_runtime::context::service_config::ServiceEnvironment::Testing);
    }

    #[test] 
    fn test_feature_flag_behavior() {
        // Test that the code correctly handles different feature flag scenarios
        
        #[cfg(feature = "enable-libp2p")]
        {
            // With libp2p feature, sync constructor should fail with context message
            let result = RuntimeContext::new();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("synchronous"));
        }
        
        #[cfg(not(feature = "enable-libp2p"))]
        {
            // Without libp2p feature, should fail with feature requirement message
            let result = RuntimeContext::new();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("libp2p feature"));
        }
    }

    #[test]
    fn test_error_message_quality() {
        // Validate that error messages are helpful and actionable
        
        let production_result = RuntimeContext::new();
        assert!(production_result.is_err());
        
        let error_msg = production_result.unwrap_err().to_string();
        
        // Should provide actionable guidance
        assert!(
            error_msg.contains("new_async") || 
            error_msg.contains("enable-libp2p") ||
            error_msg.contains("new_with_network_service"),
            "Error message should provide actionable guidance: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_builder_pattern_compatibility() {
        use icn_runtime::context::{RuntimeContextBuilder, EnvironmentType};
        
        // Builder pattern should still work for advanced configurations
        let test_did = Did::from_str("did:key:zBuilderTest").unwrap();
        
        let ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
            .with_identity(test_did.clone())
            .with_initial_mana(750)
            .build()
            .unwrap();
        
        assert_eq!(ctx.current_identity, test_did);
        assert_eq!(ctx.get_mana(&test_did).await.unwrap(), 750);
    }

    #[test]
    fn test_documentation_examples_compile() {
        // Test that the examples in our documentation actually compile and work
        
        // Example from migration guide - explicit testing
        let test_did = Did::from_str("did:key:zExample").unwrap(); 
        let _ctx = RuntimeContext::new_for_testing(test_did, None).unwrap();
        
        // Should compile without issues - validates our documentation examples
    }
}

fn main() {
    println!("🧪 Running validation tests for production-by-default RuntimeContext");
    println!("All tests should pass to validate the implementation");
}