//! Runtime context factory for ICN
//!
//! This module provides a factory for creating RuntimeContext instances with
//! appropriate service configurations based on environment and requirements.

use super::dag_store_wrapper::DagStoreWrapper;
use super::{
    DagStorageService, DagStoreMutexType, RuntimeContext, ServiceConfig, SimpleManaLedger,
};
use icn_common::{CommonError, Did, TimeProvider};
use icn_identity::{DidResolver, EnhancedDidResolver};
use icn_network::{
    NetworkEnvironment, NetworkService, NetworkServiceFactory, NetworkServiceOptionsBuilder,
};
use icn_reputation::ReputationStore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Runtime environment types for context creation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeEnvironment {
    /// Production environment - all real services required
    Production,
    /// Development environment - mix of real and fallback services
    Development,
    /// Testing environment - optimized for fast, deterministic testing
    Testing,
    /// Integration testing - real services with test configurations
    Integration,
}

/// Configuration for creating runtime contexts
#[derive(Clone)]
pub struct RuntimeCreationConfig {
    /// Target environment
    pub environment: RuntimeEnvironment,
    /// Node identity
    pub identity: Did,
    /// Initial mana balance
    pub initial_mana: Option<u64>,
    /// Network configuration
    pub network_config: Option<icn_network::service_factory::NetworkServiceConfig>,
    /// Data directory for persistent storage
    pub data_dir: Option<PathBuf>,
    /// Custom time provider
    pub time_provider: Option<Arc<dyn TimeProvider>>,
    /// Custom DID resolver
    pub did_resolver: Option<Arc<dyn DidResolver>>,
    /// Custom reputation store
    pub reputation_store: Option<Arc<dyn ReputationStore>>,
    /// Whether to enable federation management
    pub enable_federation_management: bool,
    /// Whether to allow fallback services
    pub allow_fallback: bool,
    /// Optional path to encrypted key file
    pub key_file_path: Option<PathBuf>,
    /// Optional passphrase for encrypted key file
    pub key_passphrase: Option<String>,
    /// Optional HSM key store for hardware-backed keys
    pub hsm: Option<Arc<dyn super::signers::HsmKeyStore>>,
}

impl RuntimeCreationConfig {
    /// Create a production configuration
    pub fn production(identity: Did) -> Self {
        Self {
            environment: RuntimeEnvironment::Production,
            identity,
            initial_mana: Some(1000),
            network_config: None,
            data_dir: Some(PathBuf::from("./data")),
            time_provider: None,
            did_resolver: None,
            reputation_store: None,
            enable_federation_management: true,
            allow_fallback: false,
            key_file_path: None,
            key_passphrase: None,
            hsm: None,
        }
    }

    /// Create a development configuration
    pub fn development(identity: Did) -> Self {
        Self {
            environment: RuntimeEnvironment::Development,
            identity,
            initial_mana: Some(10000),
            network_config: None,
            data_dir: Some(PathBuf::from("./dev-data")),
            time_provider: None,
            did_resolver: None,
            reputation_store: None,
            enable_federation_management: true,
            allow_fallback: true,
            key_file_path: None,
            key_passphrase: None,
            hsm: None,
        }
    }

    /// Create a testing configuration
    pub fn testing(identity: Did) -> Self {
        Self {
            environment: RuntimeEnvironment::Testing,
            identity,
            initial_mana: Some(100000),
            network_config: None,
            data_dir: None, // Use temporary storage
            time_provider: None,
            did_resolver: None,
            reputation_store: None,
            enable_federation_management: false, // Disable for faster tests
            allow_fallback: true,
            key_file_path: None,
            key_passphrase: None,
            hsm: None,
        }
    }

    /// Create an integration testing configuration
    pub fn integration_testing(identity: Did) -> Self {
        Self {
            environment: RuntimeEnvironment::Integration,
            identity,
            initial_mana: Some(50000),
            network_config: None,
            data_dir: Some(PathBuf::from("./test-data")),
            time_provider: None,
            did_resolver: None,
            reputation_store: None,
            enable_federation_management: true,
            allow_fallback: false,
            key_file_path: None,
            key_passphrase: None,
            hsm: None,
        }
    }

    /// Set encrypted key file path and passphrase
    pub fn with_encrypted_key_file<P: Into<PathBuf>, S: Into<String>>(
        mut self,
        path: P,
        passphrase: S,
    ) -> Self {
        self.key_file_path = Some(path.into());
        self.key_passphrase = Some(passphrase.into());
        self
    }

    /// Set HSM key store
    pub fn with_hsm_key_store(mut self, hsm: Arc<dyn super::signers::HsmKeyStore>) -> Self {
        self.hsm = Some(hsm);
        self
    }

    /// Set data directory
    pub fn with_data_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Set initial mana balance
    pub fn with_initial_mana(mut self, mana: u64) -> Self {
        self.initial_mana = Some(mana);
        self
    }
}

/// Factory for creating runtime contexts
pub struct RuntimeContextFactory;

impl RuntimeContextFactory {
    /// Create a runtime context with the given configuration
    pub async fn create(config: RuntimeCreationConfig) -> Result<Arc<RuntimeContext>, CommonError> {
        match config.environment {
            RuntimeEnvironment::Production => Self::create_production(config).await,
            RuntimeEnvironment::Development => Self::create_development(config).await,
            RuntimeEnvironment::Testing => Self::create_testing(config).await,
            RuntimeEnvironment::Integration => Self::create_integration(config).await,
        }
    }

    /// Create a production runtime context
    async fn create_production(
        config: RuntimeCreationConfig,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        log::info!("🏭 Creating production runtime context");

        // Create network service
        let network_service = Self::create_network_service(
            NetworkEnvironment::Production,
            config.network_config,
            config.allow_fallback,
        )
        .await?;

        // Create enhanced DID resolver
        let time_provider = config
            .time_provider
            .unwrap_or_else(|| Arc::new(icn_common::SystemTimeProvider));
        let did_resolver = config
            .did_resolver
            .unwrap_or_else(|| Arc::new(EnhancedDidResolver::with_defaults(time_provider.clone())));

        // Create signer with proper key management for production
        let signer = if let Some(key_path) = &config.key_file_path {
            // Use key from encrypted file if provided
            if let Some(passphrase) = &config.key_passphrase {
                Arc::new(
                    super::signers::Ed25519Signer::from_encrypted_file(
                        key_path,
                        passphrase.as_bytes(),
                    )
                    .map_err(|e| {
                        CommonError::CryptoError(format!("Failed to load production key: {}", e))
                    })?,
                ) as Arc<dyn super::signers::Signer>
            } else {
                return Err(CommonError::CryptoError(
                    "Production environment requires passphrase for encrypted key file".to_string(),
                ));
            }
        } else if let Some(hsm) = &config.hsm {
            // Use HSM if configured
            Arc::new(
                super::signers::Ed25519Signer::from_hsm(hsm.as_ref()).map_err(|e| {
                    CommonError::CryptoError(format!("Failed to load HSM key: {}", e))
                })?,
            )
        } else {
            // Generate a new key for this production instance and warn the user
            log::warn!("🔑 PRODUCTION WARNING: No key file or HSM configured. Generating ephemeral key. This key will not persist across restarts!");
            log::warn!("🔑 For production use, provide either:");
            log::warn!("🔑 - key_file_path and key_passphrase for encrypted file");
            log::warn!("🔑 - hsm configuration for hardware security module");
            let (signing_key, _) = icn_identity::generate_ed25519_keypair();
            Arc::new(super::signers::Ed25519Signer::new(signing_key))
        };

        // Create persistent DAG store
        let data_dir = config.data_dir.unwrap_or_else(|| PathBuf::from("./data"));
        let dag_store_raw =
            Self::create_dag_store(RuntimeEnvironment::Production, Some(data_dir.clone()))?;
        let dag_store = DagStoreWrapper::generic_production(dag_store_raw);

        // Create mana ledger
        let mana_ledger = SimpleManaLedger::new(data_dir.join("mana.sled"));
        if let Some(initial_mana) = config.initial_mana {
            mana_ledger
                .credit(&config.identity, initial_mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        // Create reputation store
        let reputation_store = config
            .reputation_store
            .unwrap_or_else(|| Arc::new(icn_reputation::InMemoryReputationStore::new()));

        // Build service config
        let service_config = ServiceConfig::production(
            config.identity,
            network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_store,
            None, // No policy enforcer for now
        )?;

        RuntimeContext::from_service_config(service_config)
    }

    /// Create a development runtime context
    async fn create_development(
        config: RuntimeCreationConfig,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        log::info!("🛠️ Creating development runtime context");

        // Create network service (prefers real but allows fallback)
        let network_service = Self::create_network_service(
            NetworkEnvironment::Development,
            config.network_config,
            config.allow_fallback,
        )
        .await;

        // Create enhanced DID resolver
        let time_provider = config
            .time_provider
            .unwrap_or_else(|| Arc::new(icn_common::SystemTimeProvider));
        let did_resolver = config
            .did_resolver
            .unwrap_or_else(|| Arc::new(EnhancedDidResolver::with_defaults(time_provider.clone())));

        // Create signer with proper key management for development
        let signer = if let Some(key_path) = &config.key_file_path {
            // Use key from encrypted file if provided
            if let Some(passphrase) = &config.key_passphrase {
                Arc::new(
                    super::signers::Ed25519Signer::from_encrypted_file(
                        key_path,
                        passphrase.as_bytes(),
                    )
                    .map_err(|e| {
                        CommonError::CryptoError(format!("Failed to load development key: {}", e))
                    })?,
                ) as Arc<dyn super::signers::Signer>
            } else {
                return Err(CommonError::CryptoError(
                    "Development environment with key file requires passphrase".to_string(),
                ));
            }
        } else if let Some(hsm) = &config.hsm {
            // Use HSM if configured
            Arc::new(
                super::signers::Ed25519Signer::from_hsm(hsm.as_ref()).map_err(|e| {
                    CommonError::CryptoError(format!("Failed to load HSM key: {}", e))
                })?,
            )
        } else {
            // For development, generate a new key (this is fine for dev environments)
            log::info!("🔧 DEVELOPMENT: Generating ephemeral Ed25519 key for this session");
            let (signing_key, _) = icn_identity::generate_ed25519_keypair();
            Arc::new(super::signers::Ed25519Signer::new(signing_key))
        };

        // Create DAG store (persistent if data_dir provided)
        let dag_store_raw =
            Self::create_dag_store(RuntimeEnvironment::Development, config.data_dir.clone())?;
        let dag_store = DagStoreWrapper::generic_production(dag_store_raw);

        // Create mana ledger
        let mana_ledger = if let Some(data_dir) = config.data_dir {
            SimpleManaLedger::new(data_dir.join("dev-mana.sled"))
        } else {
            // Use temporary file for development
            let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                CommonError::IoError(format!("Failed to create temp mana ledger: {}", e))
            })?;
            let temp_path = temp_file.path().to_path_buf();
            std::mem::forget(temp_file);
            SimpleManaLedger::new(temp_path)
        };

        if let Some(initial_mana) = config.initial_mana {
            mana_ledger
                .credit(&config.identity, initial_mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        // Create reputation store
        let reputation_store = config
            .reputation_store
            .unwrap_or_else(|| Arc::new(icn_reputation::InMemoryReputationStore::new()));

        // Build service config
        let service_config = ServiceConfig::development(
            config.identity,
            signer,
            mana_ledger,
            network_service.ok(),
            Some(dag_store),
        )?;

        RuntimeContext::from_service_config(service_config)
    }

    /// Create a testing runtime context
    async fn create_testing(
        config: RuntimeCreationConfig,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        log::info!("🧪 Creating testing runtime context");

        // For testing, we typically want fast stub services
        let network_service = Self::create_network_service(
            NetworkEnvironment::Testing,
            None,
            true, // Always allow fallback for testing
        )
        .await;

        // Create basic DID resolver
        let did_resolver = config
            .did_resolver
            .unwrap_or_else(|| Arc::new(icn_identity::KeyDidResolver));

        // Create stub signer for testing
        let signer = Arc::new(super::signers::StubSigner::new());

        // Create in-memory DAG store
        let _dag_store = Self::create_dag_store(RuntimeEnvironment::Testing, None)?;

        // Create temporary mana ledger
        let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
            CommonError::IoError(format!("Failed to create temp mana ledger: {}", e))
        })?;
        let temp_path = temp_file.path().to_path_buf();
        std::mem::forget(temp_file);
        let mana_ledger = SimpleManaLedger::new(temp_path);

        if let Some(initial_mana) = config.initial_mana {
            mana_ledger
                .credit(&config.identity, initial_mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        // Create in-memory reputation store
        let _reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());

        // Use the existing testing method which handles all the setup
        RuntimeContext::new_for_testing(config.identity, config.initial_mana)
    }

    /// Create an integration testing runtime context
    async fn create_integration(
        config: RuntimeCreationConfig,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        log::info!("🔗 Creating integration testing runtime context");

        // For integration testing, we want real services with test configurations
        let network_service = Self::create_network_service(
            NetworkEnvironment::Production,
            config.network_config,
            config.allow_fallback,
        )
        .await?;

        // Create enhanced DID resolver
        let time_provider = config
            .time_provider
            .unwrap_or_else(|| Arc::new(icn_common::SystemTimeProvider));
        let did_resolver = config
            .did_resolver
            .unwrap_or_else(|| Arc::new(EnhancedDidResolver::with_defaults(time_provider.clone())));

        // Create signer with proper key management for integration testing
        let signer = if let Some(key_path) = &config.key_file_path {
            // Use key from encrypted file if provided (for consistent test keys)
            if let Some(passphrase) = &config.key_passphrase {
                Arc::new(
                    super::signers::Ed25519Signer::from_encrypted_file(
                        key_path,
                        passphrase.as_bytes(),
                    )
                    .map_err(|e| {
                        CommonError::CryptoError(format!("Failed to load test key: {}", e))
                    })?,
                ) as Arc<dyn super::signers::Signer>
            } else {
                return Err(CommonError::CryptoError(
                    "Integration test environment with key file requires passphrase".to_string(),
                ));
            }
        } else {
            // For integration tests, use a deterministic key based on the identity to ensure reproducible tests
            log::info!("🧪 INTEGRATION TEST: Using deterministic Ed25519 key based on identity");
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(config.identity.to_string().as_bytes());
            hasher.update(b"integration-test-key-seed");
            let seed = hasher.finalize();
            let seed_array: [u8; 32] = seed[..32].try_into().expect("Seed should be 32 bytes");
            let signing_key = icn_identity::SigningKey::from_bytes(&seed_array);
            Arc::new(super::signers::Ed25519Signer::new(signing_key))
        };

        // Create persistent DAG store in test directory
        let data_dir = config
            .data_dir
            .unwrap_or_else(|| PathBuf::from("./test-data"));
        let dag_store_raw =
            Self::create_dag_store(RuntimeEnvironment::Integration, Some(data_dir.clone()))?;
        let dag_store = DagStoreWrapper::generic_production(dag_store_raw);

        // Create persistent mana ledger
        let mana_ledger = SimpleManaLedger::new(data_dir.join("test-mana.sled"));
        if let Some(initial_mana) = config.initial_mana {
            mana_ledger
                .credit(&config.identity, initial_mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        // Create reputation store
        let reputation_store = config
            .reputation_store
            .unwrap_or_else(|| Arc::new(icn_reputation::InMemoryReputationStore::new()));

        // Build service config
        let service_config = ServiceConfig::production(
            config.identity,
            network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_store,
            None,
        )?;

        RuntimeContext::from_service_config(service_config)
    }

    /// Create network service based on environment
    async fn create_network_service(
        environment: NetworkEnvironment,
        config: Option<icn_network::service_factory::NetworkServiceConfig>,
        allow_fallback: bool,
    ) -> Result<Arc<dyn NetworkService>, CommonError> {
        use icn_network::NetworkServiceCreationResult;

        let options = NetworkServiceOptionsBuilder::new()
            .environment(environment)
            .allow_fallback(allow_fallback);

        let options = if let Some(net_config) = config {
            options.config(net_config)
        } else {
            options
        };

        match NetworkServiceFactory::create(options.build()).await {
            NetworkServiceCreationResult::Libp2p(service) => Ok(service),
            NetworkServiceCreationResult::Stub(service) => {
                if environment == NetworkEnvironment::Production && !allow_fallback {
                    Err(CommonError::NetworkError(
                        "Production environment requires real network service".to_string(),
                    ))
                } else {
                    Ok(service)
                }
            }
            NetworkServiceCreationResult::Failed(e) => Err(CommonError::NetworkError(format!(
                "Network service creation failed: {}",
                e
            ))),
        }
    }

    /// Create DAG store based on environment
    fn create_dag_store(
        environment: RuntimeEnvironment,
        data_dir: Option<PathBuf>,
    ) -> Result<Arc<DagStoreMutexType<DagStorageService>>, CommonError> {
        match environment {
            RuntimeEnvironment::Production | RuntimeEnvironment::Integration => {
                let data_dir = data_dir.ok_or_else(|| {
                    CommonError::InternalError(
                        "Data directory required for production/integration".to_string(),
                    )
                })?;

                // Create directory if it doesn't exist
                std::fs::create_dir_all(&data_dir).map_err(|e| {
                    CommonError::IoError(format!("Failed to create data directory: {}", e))
                })?;

                // For now, use sled store as default persistent store
                #[cfg(feature = "persist-sled")]
                {
                    let dag_path = data_dir.join("dag.sled");
                    let store = icn_dag::TokioFileDagStore::new(dag_path).map_err(|e| {
                        CommonError::InternalError(format!(
                            "Failed to create sled DAG store: {}",
                            e
                        ))
                    })?;
                    Ok(Arc::new(DagStoreMutexType::new(store)))
                }

                #[cfg(not(feature = "persist-sled"))]
                {
                    // Fallback to in-memory store if persistence not available
                    log::warn!("Persistent storage not available, using in-memory DAG store");
                    Ok(Arc::new(DagStoreMutexType::new(
                        super::stubs::StubDagStore::new(),
                    )))
                }
            }
            RuntimeEnvironment::Development => {
                if let Some(data_dir) = data_dir {
                    // Try to use persistent store for development
                    std::fs::create_dir_all(&data_dir).map_err(|e| {
                        CommonError::IoError(format!("Failed to create data directory: {}", e))
                    })?;

                    #[cfg(feature = "persist-sled")]
                    {
                        let dag_path = data_dir.join("dev-dag.sled");
                        let store = icn_dag::TokioFileDagStore::new(dag_path).map_err(|e| {
                            CommonError::InternalError(format!(
                                "Failed to create sled DAG store: {}",
                                e
                            ))
                        })?;
                        Ok(Arc::new(DagStoreMutexType::new(store)))
                    }

                    #[cfg(not(feature = "persist-sled"))]
                    {
                        Ok(Arc::new(DagStoreMutexType::new(
                            super::stubs::StubDagStore::new(),
                        )))
                    }
                } else {
                    // Use in-memory store for development without data directory
                    Ok(Arc::new(DagStoreMutexType::new(
                        super::stubs::StubDagStore::new(),
                    )))
                }
            }
            RuntimeEnvironment::Testing => {
                // Always use in-memory store for testing
                Ok(Arc::new(DagStoreMutexType::new(
                    super::stubs::StubDagStore::new(),
                )))
            }
        }
    }

    /// Create a production-ready context with strict validation
    pub async fn create_production_strict(
        identity: Did,
        network_config: icn_network::service_factory::NetworkServiceConfig,
        data_dir: PathBuf,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        let config = RuntimeCreationConfig {
            environment: RuntimeEnvironment::Production,
            identity,
            initial_mana: Some(1000),
            network_config: Some(network_config),
            data_dir: Some(data_dir),
            time_provider: None,
            did_resolver: None,
            reputation_store: None,
            enable_federation_management: true,
            allow_fallback: false,
            key_file_path: None,
            key_passphrase: None,
            hsm: None,
        };

        let context = Self::create(config).await?;

        // Validate that we're using production services
        context.validate_production_services()?;

        Ok(context)
    }

    /// Create a fast testing context with minimal setup
    pub async fn create_fast_testing(identity: Did) -> Result<Arc<RuntimeContext>, CommonError> {
        let config = RuntimeCreationConfig::testing(identity);
        Self::create(config).await
    }

    /// Create a development context with local networking
    pub async fn create_local_development(
        identity: Did,
        data_dir: Option<PathBuf>,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        let mut config = RuntimeCreationConfig::development(identity);
        config.data_dir = data_dir;

        // Configure for local development
        let network_config = icn_network::service_factory::NetworkServiceConfig {
            listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".to_string()],
            enable_mdns: true,
            max_peers: 20,
            ..Default::default()
        };
        config.network_config = Some(network_config);

        Self::create(config).await
    }
}

/// Builder for runtime creation configurations
pub struct RuntimeCreationConfigBuilder {
    config: RuntimeCreationConfig,
}

impl RuntimeCreationConfigBuilder {
    pub fn new(identity: Did) -> Self {
        Self {
            config: RuntimeCreationConfig::development(identity),
        }
    }

    pub fn environment(mut self, env: RuntimeEnvironment) -> Self {
        self.config.environment = env;
        self
    }

    pub fn initial_mana(mut self, mana: u64) -> Self {
        self.config.initial_mana = Some(mana);
        self
    }

    pub fn network_config(
        mut self,
        config: icn_network::service_factory::NetworkServiceConfig,
    ) -> Self {
        self.config.network_config = Some(config);
        self
    }

    pub fn data_dir(mut self, dir: PathBuf) -> Self {
        self.config.data_dir = Some(dir);
        self
    }

    pub fn time_provider(mut self, provider: Arc<dyn TimeProvider>) -> Self {
        self.config.time_provider = Some(provider);
        self
    }

    pub fn did_resolver(mut self, resolver: Arc<dyn DidResolver>) -> Self {
        self.config.did_resolver = Some(resolver);
        self
    }

    pub fn reputation_store(mut self, store: Arc<dyn ReputationStore>) -> Self {
        self.config.reputation_store = Some(store);
        self
    }

    pub fn federation_management(mut self, enable: bool) -> Self {
        self.config.enable_federation_management = enable;
        self
    }

    pub fn allow_fallback(mut self, allow: bool) -> Self {
        self.config.allow_fallback = allow;
        self
    }

    pub fn build(self) -> RuntimeCreationConfig {
        self.config
    }

    pub async fn create(self) -> Result<Arc<RuntimeContext>, CommonError> {
        RuntimeContextFactory::create(self.build()).await
    }
}

/// Convenience functions for common runtime creation patterns
pub mod presets {
    use super::*;

    /// Create a production runtime context
    pub async fn production(
        identity: Did,
        data_dir: PathBuf,
        listen_addr: &str,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        let network_config = icn_network::service_factory::NetworkServiceConfig {
            listen_addresses: vec![listen_addr.to_string()],
            enable_mdns: false,
            max_peers: 1000,
            ..Default::default()
        };

        RuntimeContextFactory::create_production_strict(identity, network_config, data_dir).await
    }

    /// Create a local development runtime context
    pub async fn local_development(identity: Did) -> Result<Arc<RuntimeContext>, CommonError> {
        RuntimeContextFactory::create_local_development(identity, None).await
    }

    /// Create a fast testing runtime context
    pub async fn testing(identity: Did) -> Result<Arc<RuntimeContext>, CommonError> {
        RuntimeContextFactory::create_fast_testing(identity).await
    }

    /// Create an integration testing runtime context
    pub async fn integration_testing(
        identity: Did,
        network_config: icn_network::service_factory::NetworkServiceConfig,
    ) -> Result<Arc<RuntimeContext>, CommonError> {
        let config = RuntimeCreationConfigBuilder::new(identity)
            .environment(RuntimeEnvironment::Integration)
            .network_config(network_config)
            .data_dir(PathBuf::from("./integration-test-data"))
            .allow_fallback(false)
            .build();

        RuntimeContextFactory::create(config).await
    }
}
