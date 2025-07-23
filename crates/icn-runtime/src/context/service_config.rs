//! Service configuration for RuntimeContext.
//!
//! This module provides type-safe service creation and mapping to ensure
//! that stub services are never accidentally used in production.

use super::dag_store_factory::{DagStoreConfig, DagStoreFactory};
use super::dag_store_wrapper::DagStoreWrapper;
use super::mesh_network::DefaultMeshNetworkService;
use super::runtime_context::MeshNetworkServiceType;
use super::signers::Signer;
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::{DagStorageService, DagStoreMutexType};
use icn_common::{CommonError, Did};
use icn_reputation::ReputationStore;
use std::path::PathBuf;
use std::sync::Arc;

/// Service configuration environments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceEnvironment {
    /// Production environment - all services must be production-ready
    Production,
    /// Development environment - mix of production and stub services allowed
    Development,
    /// Testing environment - stub services allowed and preferred
    Testing,
}

/// Service configuration builder that ensures type safety
pub struct ServiceConfigBuilder {
    environment: ServiceEnvironment,
    current_identity: Option<Did>,
    network_service: Option<Arc<dyn icn_network::NetworkService>>,
    signer: Option<Arc<dyn Signer>>,
    did_resolver: Option<Arc<dyn icn_identity::DidResolver>>,
    dag_store: Option<DagStoreWrapper>,
    mana_ledger: Option<super::mana::SimpleManaLedger>,
    reputation_store: Option<Arc<dyn ReputationStore>>,
    time_provider: Option<Arc<dyn icn_common::TimeProvider>>,
    policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
}

impl ServiceConfigBuilder {
    /// Create a new service configuration builder
    pub fn new(environment: ServiceEnvironment) -> Self {
        Self {
            environment,
            current_identity: None,
            network_service: None,
            signer: None,
            did_resolver: None,
            dag_store: None,
            mana_ledger: None,
            reputation_store: None,
            time_provider: None,
            policy_enforcer: None,
        }
    }

    /// Set the current identity (required)
    pub fn with_identity(mut self, identity: Did) -> Self {
        self.current_identity = Some(identity);
        self
    }

    /// Set the network service (required for production)
    pub fn with_network_service(mut self, service: Arc<dyn icn_network::NetworkService>) -> Self {
        self.network_service = Some(service);
        self
    }

    /// Set the signer (required)
    pub fn with_signer(mut self, signer: Arc<dyn Signer>) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Set the DID resolver (required)
    pub fn with_did_resolver(mut self, resolver: Arc<dyn icn_identity::DidResolver>) -> Self {
        self.did_resolver = Some(resolver);
        self
    }

    /// Set the DAG store (required for production)
    pub fn with_dag_store(mut self, store: DagStoreWrapper) -> Self {
        self.dag_store = Some(store);
        self
    }

    /// Set the mana ledger (required)
    pub fn with_mana_ledger(mut self, ledger: super::mana::SimpleManaLedger) -> Self {
        self.mana_ledger = Some(ledger);
        self
    }

    /// Set the reputation store (optional)
    pub fn with_reputation_store(mut self, store: Arc<dyn ReputationStore>) -> Self {
        self.reputation_store = Some(store);
        self
    }

    /// Set the time provider (optional)
    pub fn with_time_provider(mut self, provider: Arc<dyn icn_common::TimeProvider>) -> Self {
        self.time_provider = Some(provider);
        self
    }

    /// Set the policy enforcer (optional)
    pub fn with_policy_enforcer(
        mut self,
        enforcer: Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>,
    ) -> Self {
        self.policy_enforcer = Some(enforcer);
        self
    }

    /// Build the service configuration
    pub fn build(self) -> Result<ServiceConfig, CommonError> {
        let current_identity = self.current_identity.clone().ok_or_else(|| {
            CommonError::InternalError("Current identity is required".to_string())
        })?;

        let signer = self
            .signer
            .clone()
            .ok_or_else(|| CommonError::InternalError("Signer is required".to_string()))?;

        let did_resolver = self.create_did_resolver();

        let mana_ledger = self
            .mana_ledger
            .clone()
            .ok_or_else(|| CommonError::InternalError("Mana ledger is required".to_string()))?;

        // Validate production requirements
        if self.environment == ServiceEnvironment::Production {
            if self.network_service.is_none() {
                return Err(CommonError::InternalError(
                    "Network service is required for production environment".to_string(),
                ));
            }
            if self.dag_store.is_none() {
                return Err(CommonError::InternalError(
                    "DAG store is required for production environment".to_string(),
                ));
            }
        }

        // Create appropriate services based on environment
        let mesh_network_service = self.create_mesh_network_service(&signer)?;
        let dag_store = self.create_dag_store()?;
        let reputation_store = self.create_reputation_store();
        let time_provider = self.create_time_provider();

        Ok(ServiceConfig {
            environment: self.environment,
            current_identity,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_store,
            time_provider,
            policy_enforcer: self.policy_enforcer,
        })
    }

    /// Create mesh network service based on environment
    fn create_mesh_network_service(
        &self,
        signer: &Arc<dyn Signer>,
    ) -> Result<Arc<MeshNetworkServiceType>, CommonError> {
        match self.environment {
            ServiceEnvironment::Production => {
                // First try to use provided network service
                if let Some(network_service) = self.network_service.as_ref() {
                    let service = Arc::new(MeshNetworkServiceType::Default(
                        DefaultMeshNetworkService::new(network_service.clone(), signer.clone()),
                    ));

                    // Compile-time check for production builds
                    #[cfg(all(feature = "production", not(feature = "allow-stubs")))]
                    {
                        // This will be checked at runtime to ensure we're not using stubs
                        if matches!(&*service, MeshNetworkServiceType::Stub(_)) {
                            return Err(CommonError::InternalError(
                                "Stub mesh network service cannot be used in production"
                                    .to_string(),
                            ));
                        }
                    }

                    return Ok(service);
                }

                // If no network service provided, try to create libp2p service with defaults
                #[cfg(feature = "enable-libp2p")]
                {
                    // For production, require explicit network service since we can't async create here
                    return Err(CommonError::InternalError(
                        "Production environment requires explicit network service in non-async context".to_string(),
                    ));
                }

                // If libp2p feature is not enabled, require explicit network service
                #[cfg(not(feature = "enable-libp2p"))]
                {
                    return Err(CommonError::InternalError(
                        "Production environment requires either explicit network service or libp2p feature enabled".to_string(),
                    ));
                }
            }
            ServiceEnvironment::Development => {
                if let Some(network_service) = &self.network_service {
                    Ok(Arc::new(MeshNetworkServiceType::Default(
                        DefaultMeshNetworkService::new(network_service.clone(), signer.clone()),
                    )))
                } else {
                    let service =
                        Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));

                    // Ensure we're not accidentally using stubs in production
                    #[cfg(all(feature = "production", not(feature = "allow-stubs")))]
                    {
                        return Err(CommonError::InternalError(
                            "Stub services cannot be used in production builds".to_string(),
                        ));
                    }

                    Ok(service)
                }
            }
            ServiceEnvironment::Testing => {
                let service = Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));

                // Ensure we're not accidentally using stubs in production
                #[cfg(all(feature = "production", not(feature = "allow-stubs")))]
                {
                    return Err(CommonError::InternalError(
                        "Stub services cannot be used in production builds".to_string(),
                    ));
                }

                Ok(service)
            }
        }
    }

    /// Create DAG store based on environment
    fn create_dag_store(&self) -> Result<DagStoreWrapper, CommonError> {
        match self.environment {
            ServiceEnvironment::Production => {
                // For production, require explicit DAG store to ensure persistence
                if let Some(store_wrapper) = &self.dag_store {
                    // Validate that it's not a stub store (synchronous check)
                    store_wrapper.validate_for_production()?;
                    Ok(store_wrapper.clone())
                } else {
                    // If no explicit store provided, try to create a default production store
                    // This requires a storage path, which we don't have in this context
                    Err(CommonError::InternalError(
                        "DAG store is required for production environment. Use DagStoreFactory::create_production() to create one.".to_string()
                    ))
                }
            }
            ServiceEnvironment::Development => {
                if let Some(store_wrapper) = &self.dag_store {
                    Ok(store_wrapper.clone())
                } else {
                    // For development, use factory to create a stub store
                    Ok(DagStoreFactory::create_testing())
                }
            }
            ServiceEnvironment::Testing => {
                // Always use stub for testing, even if a real store was provided
                Ok(DagStoreFactory::create_testing())
            }
        }
    }

    /// Create reputation store
    fn create_reputation_store(&self) -> Arc<dyn ReputationStore> {
        self.reputation_store
            .as_ref()
            .map(|store| store.clone())
            .unwrap_or_else(|| Arc::new(icn_reputation::InMemoryReputationStore::new()))
    }

    /// Create time provider
    fn create_time_provider(&self) -> Arc<dyn icn_common::TimeProvider> {
        self.time_provider
            .as_ref()
            .map(|provider| provider.clone())
            .unwrap_or_else(|| Arc::new(icn_common::SystemTimeProvider))
    }

    /// Create DID resolver with enhanced capabilities
    fn create_did_resolver(&self) -> Arc<dyn icn_identity::DidResolver> {
        if let Some(resolver) = &self.did_resolver {
            return resolver.clone();
        }

        // Create enhanced DID resolver with production capabilities
        let time_provider = self.create_time_provider();

        match self.environment {
            ServiceEnvironment::Production | ServiceEnvironment::Development => {
                // Use enhanced resolver with caching and multiple method support
                let config = icn_identity::DidResolutionConfig {
                    cache_ttl_seconds: 3600, // 1 hour cache
                    max_cache_size: 10000,
                    web_timeout_seconds: 30,
                    enable_fallback: true,
                    method_preference: vec![
                        "key".to_string(),
                        "peer".to_string(),
                        "web".to_string(),
                    ],
                };

                Arc::new(icn_identity::EnhancedDidResolver::new(
                    config,
                    time_provider,
                ))
            }
            ServiceEnvironment::Testing => {
                // For testing, use simple key resolver for faster tests
                Arc::new(icn_identity::KeyDidResolver)
            }
        }
    }
}

/// Complete service configuration
pub struct ServiceConfig {
    pub environment: ServiceEnvironment,
    pub current_identity: Did,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: DagStoreWrapper,
    pub mana_ledger: super::mana::SimpleManaLedger,
    pub reputation_store: Arc<dyn ReputationStore>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
}

impl ServiceConfig {
    /// Validate that the configuration is appropriate for the environment
    pub fn validate(&self) -> Result<(), CommonError> {
        match self.environment {
            ServiceEnvironment::Production => {
                self.validate_production_services()
            }
            ServiceEnvironment::Development | ServiceEnvironment::Testing => {
                // Allow any configuration for dev/test
                Ok(())
            }
        }
    }

    /// Validate that production services are being used correctly.
    ///
    /// This function performs comprehensive checks to ensure that stub services
    /// are not accidentally used in production contexts.
    pub fn validate_production_services(&self) -> Result<(), CommonError> {
        // Check if we're using stub mesh network service
        if let MeshNetworkServiceType::Stub(_) = &*self.mesh_network_service {
            return Err(CommonError::InternalError(
                "❌ PRODUCTION ERROR: Stub mesh network service detected in production context. Use production network service or enable 'enable-libp2p' feature.".to_string()
            ));
        }

        // Check signer type - validate it's not a stub signer
        if self.signer.as_any().is::<super::signers::StubSigner>() {
            return Err(CommonError::InternalError(
                "❌ PRODUCTION ERROR: Stub signer detected in production context. Use Ed25519Signer or other production signer.".to_string(),
            ));
        }

        // Check DAG store type (synchronous check)
        self.dag_store.validate_for_production()?;

        // Check reputation store type - in-memory is acceptable for now but warn
        if self.reputation_store.as_any().is::<icn_reputation::InMemoryReputationStore>() {
            log::warn!("⚠️ PRODUCTION WARNING: Using in-memory reputation store. Consider using persistent reputation storage for production.");
        }

        // Additional production checks can be added here
        log::info!("✅ PRODUCTION VALIDATION: All services validated for production use");
        Ok(())
    }
}

/// Factory methods for common service configurations
impl ServiceConfig {
    /// Create a production configuration with defaults.
    ///
    /// **🏭 PRODUCTION**: This creates a production-ready configuration with sensible defaults.
    /// All services will be production services, no stubs allowed.
    pub fn production_defaults() -> Result<Self, CommonError> {
        Err(CommonError::InternalError(
            "Cannot create production defaults without explicit services. Use production() with explicit services or production_with_storage() for automatic storage creation.".to_string()
        ))
    }

    /// Create a testing configuration with defaults.
    ///
    /// **🧪 TESTING**: This creates a testing configuration with stub services.
    pub fn testing_defaults() -> Result<Self, CommonError> {
        use icn_identity::generate_ed25519_keypair;
        
        // Generate a test identity
        let (_, verifying_key) = generate_ed25519_keypair();
        let test_identity = icn_identity::did_key_from_verifying_key(&verifying_key);
        
        Self::testing(test_identity, Some(1000)) // 1000 initial mana for testing
    }

    /// Create a production configuration
    #[allow(clippy::too_many_arguments)]
    pub fn production(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: DagStoreWrapper,
        mana_ledger: super::mana::SimpleManaLedger,
        reputation_store: Arc<dyn ReputationStore>,
        policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    ) -> Result<Self, CommonError> {
        let mut builder = ServiceConfigBuilder::new(ServiceEnvironment::Production)
            .with_identity(current_identity)
            .with_network_service(network_service)
            .with_signer(signer)
            .with_did_resolver(did_resolver)
            .with_dag_store(dag_store)
            .with_mana_ledger(mana_ledger)
            .with_reputation_store(reputation_store);

        if let Some(enforcer) = policy_enforcer {
            builder = builder.with_policy_enforcer(enforcer);
        }

        let config = builder.build()?;

        // Validate production configuration
        config.validate_production_services()?;
        Ok(config)
    }

    /// Create a development configuration
    pub fn development(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: super::mana::SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        dag_store: Option<DagStoreWrapper>,
    ) -> Result<Self, CommonError> {
        let mut builder = ServiceConfigBuilder::new(ServiceEnvironment::Development)
            .with_identity(current_identity)
            .with_signer(signer)
            .with_mana_ledger(mana_ledger);

        if let Some(network_service) = network_service {
            builder = builder.with_network_service(network_service);
        }

        if let Some(dag_store) = dag_store {
            builder = builder.with_dag_store(dag_store);
        }

        builder.build()
    }

    /// Create a testing configuration
    pub fn testing(current_identity: Did, initial_mana: Option<u64>) -> Result<Self, CommonError> {
        // Create temporary mana ledger for testing
        let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
            CommonError::IoError(format!("Failed to create temp file for testing: {}", e))
        })?;
        let temp_path = temp_file.path().to_path_buf();
        std::mem::forget(temp_file);
        let mana_ledger = super::mana::SimpleManaLedger::new(temp_path);

        // Set initial mana if provided
        if let Some(mana) = initial_mana {
            mana_ledger
                .set_balance(&current_identity, mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        ServiceConfigBuilder::new(ServiceEnvironment::Testing)
            .with_identity(current_identity)
            .with_signer(Arc::new(super::signers::StubSigner::new()))
            .with_mana_ledger(mana_ledger)
            .build()
    }

    /// Create a production configuration with automatic DAG store creation
    pub fn production_with_storage(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        storage_path: PathBuf,
        mana_ledger: super::mana::SimpleManaLedger,
    ) -> Result<Self, CommonError> {
        // Create production DAG store
        let dag_store = DagStoreFactory::create_production(storage_path)?;

        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());

        Self::production(
            current_identity,
            network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_store,
            None,
        )
    }

    /// Create a development configuration with optional persistent storage
    pub fn development_with_storage(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: super::mana::SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        storage_path: Option<PathBuf>,
    ) -> Result<Self, CommonError> {
        // Create DAG store based on whether storage path is provided
        let dag_store = if let Some(path) = storage_path {
            Some(DagStoreFactory::create_production(path)?)
        } else {
            None
        };

        Self::development(current_identity, signer, mana_ledger, network_service, dag_store)
    }
}
