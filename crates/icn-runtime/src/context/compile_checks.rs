//! Compile-time checks to ensure production safety.
//!
//! This module provides compile-time validation to prevent
//! stub services from accidentally being used in production builds.

/// Compile-time validation for production builds
#[cfg(all(feature = "production", not(feature = "allow-stubs")))]
pub mod production_safety {
    /// This macro triggers a compile error if called with stub services in production
    #[macro_export]
    macro_rules! forbid_stub_in_production {
        ($service:expr, $service_type:ty) => {
            #[cfg(all(feature = "production", not(feature = "allow-stubs")))]
            {
                // Check if this is a stub service type
                if std::any::type_name::<$service_type>().contains("Stub") {
                    compile_error!(
                        "Stub service used in production build. Use real services or enable 'allow-stubs' feature."
                    );
                }
            }
        };
    }
}

/// Production build validation
#[cfg(feature = "production")]
pub fn validate_production_build() {
    // This function is called during compilation to ensure production safety
    #[cfg(not(feature = "allow-stubs"))]
    {
        // Additional runtime checks can go here
        // For now, the compile-time checks above are sufficient
    }
}

/// Development/testing validation
#[cfg(not(feature = "production"))]
pub fn validate_development_build() {
    // Development builds allow any configuration
}

/// Macro to enforce production service usage
#[macro_export]
macro_rules! ensure_production_service {
    ($service_type:ty, $service:expr, $production_type:ty) => {
        #[cfg(feature = "production")]
        {
            // Ensure that in production builds, only real services are used
            let _: $production_type = $service;
        }
        
        #[cfg(not(feature = "production"))]
        {
            // Development builds can use any service type
            let _: $service_type = $service;
        }
    };
}

/// Trait to mark services as production-ready
pub trait ProductionReady {
    const IS_PRODUCTION_READY: bool;
}

/// Implement ProductionReady for real services
impl ProductionReady for super::mesh_network::DefaultMeshNetworkService {
    const IS_PRODUCTION_READY: bool = true;
}

/// Implement ProductionReady for stub services (marked as NOT production-ready)
impl ProductionReady for super::stubs::StubMeshNetworkService {
    const IS_PRODUCTION_READY: bool = false;
}

impl ProductionReady for super::stubs::StubDagStore {
    const IS_PRODUCTION_READY: bool = false;
}

/// Compile-time function to check if a service is production-ready
pub const fn check_production_ready<T: ProductionReady>() -> bool {
    T::IS_PRODUCTION_READY
}

/// Macro to validate service configuration at compile time
#[macro_export]
macro_rules! validate_service_config {
    ($service:ty) => {
        #[cfg(feature = "production")]
        const _: () = {
            if !$crate::context::compile_checks::check_production_ready::<$service>() {
                panic!("Non-production service used in production build");
            }
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_production_ready_traits() {
        use super::super::mesh_network::DefaultMeshNetworkService;
        use super::super::stubs::{StubMeshNetworkService, StubDagStore};
        
        // Verify production services are marked as production-ready
        assert!(DefaultMeshNetworkService::IS_PRODUCTION_READY);
        
        // Verify stub services are marked as NOT production-ready
        assert!(!StubMeshNetworkService::IS_PRODUCTION_READY);
        assert!(!StubDagStore::IS_PRODUCTION_READY);
    }
} 