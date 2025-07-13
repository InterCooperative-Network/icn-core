//! Service configuration for RuntimeContext.
//!
//! This module provides type-safe service creation and mapping to ensure
//! that stub services are never accidentally used in production.

use super::mesh_network::DefaultMeshNetworkService;
use super::runtime_context::MeshNetworkServiceType;
use super::signers::Signer;
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::{DagStorageService, DagStoreMutexType};
use icn_common::{CommonError, Did};
use icn_reputation::ReputationStore;
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
    dag_store: Option<Arc<DagStoreMutexType<DagStorageService>>>,
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
    pub fn with_dag_store(mut self, store: Arc<DagStoreMutexType<DagStorageService>>) -> Self {
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

        let did_resolver = self
            .did_resolver
            .clone()
            .ok_or_else(|| CommonError::InternalError("DID resolver is required".to_string()))?;

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
                let network_service = self.network_service.as_ref().ok_or_else(|| {
                    CommonError::InternalError(
                        "Network service required for production".to_string(),
                    )
                })?;
                let service = Arc::new(MeshNetworkServiceType::Default(
                    DefaultMeshNetworkService::new(network_service.clone(), signer.clone()),
                ));

                // Compile-time check for production builds
                #[cfg(all(feature = "production", not(feature = "allow-stubs")))]
                {
                    // This will be checked at runtime to ensure we're not using stubs
                    if matches!(&*service, MeshNetworkServiceType::Stub(_)) {
                        return Err(CommonError::InternalError(
                            "Stub mesh network service cannot be used in production".to_string(),
                        ));
                    }
                }

                Ok(service)
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
    fn create_dag_store(&self) -> Result<Arc<DagStoreMutexType<DagStorageService>>, CommonError> {
        match self.environment {
            ServiceEnvironment::Production => self
                .dag_store
                .as_ref()
                .ok_or_else(|| {
                    CommonError::InternalError("DAG store required for production".to_string())
                })
                .cloned(),
            ServiceEnvironment::Development => {
                if let Some(store) = &self.dag_store {
                    Ok(store.clone())
                } else {
                    Ok(Arc::new(DagStoreMutexType::new(StubDagStore::new())))
                }
            }
            ServiceEnvironment::Testing => {
                Ok(Arc::new(DagStoreMutexType::new(StubDagStore::new())))
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
}

/// Complete service configuration
pub struct ServiceConfig {
    pub environment: ServiceEnvironment,
    pub current_identity: Did,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: Arc<DagStoreMutexType<DagStorageService>>,
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
                // Ensure no stub services are used in production
                if let MeshNetworkServiceType::Stub(_) = &*self.mesh_network_service {
                    return Err(CommonError::InternalError(
                        "Stub mesh network service cannot be used in production".to_string(),
                    ));
                }

                // Check if DAG store is a stub (this is a bit tricky due to trait objects)
                // We'll use a runtime check
                let dag_store = self.dag_store.clone();
                tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        let store = dag_store.lock().await;
                        if store.as_any().is::<StubDagStore>() {
                            return Err(CommonError::InternalError(
                                "Stub DAG store cannot be used in production".to_string(),
                            ));
                        }
                        Ok(())
                    })
                })
            }
            ServiceEnvironment::Development | ServiceEnvironment::Testing => {
                // Allow any configuration for dev/test
                Ok(())
            }
        }
    }
}

/// Factory methods for common service configurations
impl ServiceConfig {
    /// Create a production configuration
    #[allow(clippy::too_many_arguments)]
    pub fn production(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
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

        config.validate()?;
        Ok(config)
    }

    /// Create a development configuration
    pub fn development(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: super::mana::SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        dag_store: Option<Arc<DagStoreMutexType<DagStorageService>>>,
    ) -> Result<Self, CommonError> {
        let mut builder = ServiceConfigBuilder::new(ServiceEnvironment::Development)
            .with_identity(current_identity)
            .with_signer(signer)
            .with_did_resolver(Arc::new(icn_identity::KeyDidResolver))
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
            .with_did_resolver(Arc::new(icn_identity::KeyDidResolver))
            .with_mana_ledger(mana_ledger)
            .build()
    }
}
