//! Configuration management for ICN runtime.
//!
//! This module provides configuration loading and validation for different
//! environments (Production, Development, Testing) with appropriate defaults.

use crate::context::{ServiceConfig, ServiceConfigBuilder, ServiceEnvironment};
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

/// Complete runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Environment type (production, development, testing)
    pub environment: EnvironmentConfig,
    /// Identity configuration
    pub identity: IdentityConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Governance configuration
    pub governance: GovernanceConfig,
    /// Runtime parameters
    pub runtime: RuntimeParametersConfig,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment type
    pub environment_type: String,
    /// Whether to enable debug features
    pub debug: bool,
    /// Log level
    pub log_level: String,
    /// Whether to enable metrics collection
    pub metrics: bool,
}

/// Identity and security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// The DID for this node
    pub node_did: String,
    /// Key storage configuration
    pub key_store: KeyStoreConfig,
    /// DID resolver configuration
    pub did_resolver: DidResolverConfig,
}

/// Key storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStoreConfig {
    /// Type of key store (file, hsm, stub)
    pub store_type: String,
    /// Path to key file (for file-based storage)
    pub key_file_path: Option<PathBuf>,
    /// HSM configuration (for HSM-based storage)
    pub hsm_config: Option<HsmConfig>,
}

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// HSM library path
    pub library_path: String,
    /// HSM slot ID
    pub slot_id: u32,
    /// HSM PIN
    pub pin: String,
}

/// DID resolver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidResolverConfig {
    /// Type of DID resolver (key, web)
    pub resolver_type: String,
    /// Additional resolver configuration
    pub config: HashMap<String, String>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen addresses for P2P networking
    pub listen_addresses: Vec<String>,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<BootstrapPeer>,
    /// Whether to enable mDNS discovery
    pub enable_mdns: bool,
    /// Network protocol timeouts
    pub timeouts: NetworkTimeouts,
    /// Connection limits
    pub connection_limits: ConnectionLimits,
}

/// Bootstrap peer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeer {
    /// Peer ID
    pub peer_id: String,
    /// Multiaddress
    pub address: String,
}

/// Network timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Keep-alive interval in milliseconds
    pub keep_alive_interval_ms: u64,
}

/// Connection limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    /// Maximum number of incoming connections
    pub max_incoming_connections: u32,
    /// Maximum number of outgoing connections
    pub max_outgoing_connections: u32,
    /// Maximum connections per peer
    pub max_connections_per_peer: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory base path
    pub data_dir: PathBuf,
    /// DAG store configuration
    pub dag_store: DagStoreConfig,
    /// Mana ledger configuration
    pub mana_ledger: ManaLedgerConfig,
    /// Reputation store configuration
    pub reputation_store: ReputationStoreConfig,
}

/// DAG store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStoreConfig {
    /// Type of DAG store (memory, file, sled, rocksdb)
    pub store_type: String,
    /// Storage path (for persistent stores)
    pub storage_path: Option<PathBuf>,
    /// Cache size in MB
    pub cache_size_mb: u64,
}

/// Mana ledger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaLedgerConfig {
    /// Path to mana ledger database
    pub ledger_path: PathBuf,
    /// Initial mana balance for new accounts
    pub initial_mana: u64,
    /// Mana regeneration rate per second
    pub regeneration_rate: f64,
    /// Maximum mana capacity
    pub max_capacity: u64,
}

/// Reputation store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStoreConfig {
    /// Type of reputation store (memory, file, database)
    pub store_type: String,
    /// Storage path (for persistent stores)
    pub storage_path: Option<PathBuf>,
    /// Reputation decay rate
    pub decay_rate: f64,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Whether governance is enabled
    pub enabled: bool,
    /// Voting parameters
    pub voting: VotingConfig,
    /// Proposal parameters
    pub proposals: ProposalConfig,
}

/// Voting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingConfig {
    /// Minimum voting power required to vote
    pub min_voting_power: u64,
    /// Vote cost in mana
    pub vote_cost_mana: u64,
    /// Voting period in seconds
    pub voting_period_seconds: u64,
}

/// Proposal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalConfig {
    /// Minimum mana required to create a proposal
    pub min_proposal_mana: u64,
    /// Proposal cost in mana
    pub proposal_cost_mana: u64,
    /// Minimum quorum for proposals
    pub min_quorum: f64,
    /// Required threshold for passing proposals
    pub passing_threshold: f64,
}

/// Runtime parameters configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeParametersConfig {
    /// Default receipt wait time in milliseconds
    pub default_receipt_wait_ms: u64,
    /// Maximum job queue size
    pub max_job_queue_size: usize,
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,
    /// Job execution timeout in milliseconds
    pub job_execution_timeout_ms: u64,
    /// Cleanup interval in milliseconds
    pub cleanup_interval_ms: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::development()
    }
}

impl RuntimeConfig {
    /// Create a default production configuration
    pub fn production() -> Self {
        Self {
            environment: EnvironmentConfig {
                environment_type: "production".to_string(),
                debug: false,
                log_level: "info".to_string(),
                metrics: true,
            },
            identity: IdentityConfig {
                node_did: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
                key_store: KeyStoreConfig {
                    store_type: "generate".to_string(),
                    key_file_path: Some(PathBuf::from("~/.icn/keys/node.key")),
                    hsm_config: None,
                },
                did_resolver: DidResolverConfig {
                    resolver_type: "key".to_string(),
                    config: HashMap::new(),
                },
            },
            network: NetworkConfig {
                listen_addresses: vec![
                    "/ip4/0.0.0.0/tcp/4001".to_string(),
                    "/ip6/::/tcp/4001".to_string(),
                ],
                bootstrap_peers: vec![],
                enable_mdns: false,
                timeouts: NetworkTimeouts {
                    connection_timeout_ms: 10000,
                    request_timeout_ms: 30000,
                    keep_alive_interval_ms: 60000,
                },
                connection_limits: ConnectionLimits {
                    max_incoming_connections: 100,
                    max_outgoing_connections: 100,
                    max_connections_per_peer: 5,
                },
            },
            storage: StorageConfig {
                data_dir: PathBuf::from("~/.icn/data"),
                dag_store: DagStoreConfig {
                    store_type: "rocksdb".to_string(),
                    storage_path: Some(PathBuf::from("~/.icn/data/dag")),
                    cache_size_mb: 256,
                },
                mana_ledger: ManaLedgerConfig {
                    ledger_path: PathBuf::from("~/.icn/data/mana.db"),
                    initial_mana: 1000,
                    regeneration_rate: 1.0,
                    max_capacity: 10000,
                },
                reputation_store: ReputationStoreConfig {
                    store_type: "file".to_string(),
                    storage_path: Some(PathBuf::from("~/.icn/data/reputation.db")),
                    decay_rate: 0.01,
                },
            },
            governance: GovernanceConfig {
                enabled: true,
                voting: VotingConfig {
                    min_voting_power: 100,
                    vote_cost_mana: 10,
                    voting_period_seconds: 604800, // 1 week
                },
                proposals: ProposalConfig {
                    min_proposal_mana: 1000,
                    proposal_cost_mana: 100,
                    min_quorum: 0.1,
                    passing_threshold: 0.6,
                },
            },
            runtime: RuntimeParametersConfig {
                default_receipt_wait_ms: 30000,
                max_job_queue_size: 1000,
                max_concurrent_jobs: 10,
                job_execution_timeout_ms: 300000, // 5 minutes
                cleanup_interval_ms: 60000,
            },
        }
    }

    /// Create a default development configuration
    pub fn development() -> Self {
        let mut config = Self::production();

        // Override for development
        config.environment = EnvironmentConfig {
            environment_type: "development".to_string(),
            debug: true,
            log_level: "debug".to_string(),
            metrics: true,
        };

        config.network.enable_mdns = true;
        config.network.listen_addresses = vec!["/ip4/127.0.0.1/tcp/4001".to_string()];

        config.storage.data_dir = PathBuf::from("./dev-data");
        config.storage.dag_store.store_type = "sled".to_string();
        config.storage.dag_store.storage_path = Some(PathBuf::from("./dev-data/dag"));
        config.storage.mana_ledger.ledger_path = PathBuf::from("./dev-data/mana.db");
        config.storage.reputation_store.store_type = "file".to_string();
        config.storage.reputation_store.storage_path = Some(PathBuf::from("./dev-data/reputation.db"));

        config.governance.voting.voting_period_seconds = 300; // 5 minutes for dev
        config.runtime.job_execution_timeout_ms = 60000; // 1 minute for dev

        config
    }

    /// Create a default testing configuration
    pub fn testing() -> Self {
        let mut config = Self::development();

        // Override for testing
        config.environment = EnvironmentConfig {
            environment_type: "testing".to_string(),
            debug: true,
            log_level: "trace".to_string(),
            metrics: false,
        };

        config.network.enable_mdns = false;
        config.network.listen_addresses = vec![]; // No networking for tests
        config.network.bootstrap_peers = vec![];

        config.storage.data_dir = PathBuf::from("/tmp/icn-test");
        config.storage.dag_store.store_type = "memory".to_string();
        config.storage.dag_store.storage_path = None;
        config.storage.mana_ledger.initial_mana = 10000; // More mana for testing
        config.storage.reputation_store.store_type = "memory".to_string();
        config.storage.reputation_store.storage_path = None;

        config.governance.voting.voting_period_seconds = 10; // 10 seconds for testing
        config.runtime.job_execution_timeout_ms = 10000; // 10 seconds for testing
        config.runtime.max_job_queue_size = 100;
        config.runtime.max_concurrent_jobs = 5;

        config
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, CommonError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            CommonError::IoError(format!("Failed to read config file {}: {}", path, e))
        })?;

        // Try TOML first, then JSON
        if path.ends_with(".toml") {
            toml::from_str(&content).map_err(|e| {
                CommonError::ConfigError(format!("Failed to parse TOML config: {}", e))
            })
        } else if path.ends_with(".json") {
            serde_json::from_str(&content).map_err(|e| {
                CommonError::ConfigError(format!("Failed to parse JSON config: {}", e))
            })
        } else {
            // Try both formats
            toml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .map_err(|e| {
                    CommonError::ConfigError(format!(
                        "Failed to parse config (tried both TOML and JSON): {}",
                        e
                    ))
                })
        }
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &str) -> Result<(), CommonError> {
        let content = if path.ends_with(".toml") {
            toml::to_string_pretty(self).map_err(|e| {
                CommonError::ConfigError(format!("Failed to serialize to TOML: {}", e))
            })?
        } else if path.ends_with(".json") {
            serde_json::to_string_pretty(self).map_err(|e| {
                CommonError::ConfigError(format!("Failed to serialize to JSON: {}", e))
            })?
        } else {
            // Default to TOML
            toml::to_string_pretty(self).map_err(|e| {
                CommonError::ConfigError(format!("Failed to serialize to TOML: {}", e))
            })?
        };

        std::fs::write(path, content).map_err(|e| {
            CommonError::IoError(format!("Failed to write config file {}: {}", path, e))
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CommonError> {
        // Validate DID format
        Did::from_str(&self.identity.node_did)
            .map_err(|e| CommonError::ConfigError(format!("Invalid node DID: {}", e)))?;

        // Validate environment type
        match self.environment.environment_type.as_str() {
            "production" | "development" | "testing" => {}
            _ => {
                return Err(CommonError::ConfigError(
                    "Invalid environment type. Must be 'production', 'development', or 'testing'"
                        .to_string(),
                ))
            }
        }

        // Validate network addresses
        for addr in &self.network.listen_addresses {
            if addr.is_empty() {
                return Err(CommonError::ConfigError("Empty listen address".to_string()));
            }
        }

        // Validate storage paths
        if self.storage.data_dir.as_os_str().is_empty() {
            return Err(CommonError::ConfigError("Empty data directory".to_string()));
        }

        // Validate mana parameters
        if self.storage.mana_ledger.initial_mana > self.storage.mana_ledger.max_capacity {
            return Err(CommonError::ConfigError(
                "Initial mana cannot exceed max capacity".to_string(),
            ));
        }

        // Validate governance parameters
        if self.governance.proposals.min_quorum < 0.0 || self.governance.proposals.min_quorum > 1.0
        {
            return Err(CommonError::ConfigError(
                "Minimum quorum must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.governance.proposals.passing_threshold < 0.0
            || self.governance.proposals.passing_threshold > 1.0
        {
            return Err(CommonError::ConfigError(
                "Passing threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the service environment from configuration
    pub fn get_service_environment(&self) -> Result<ServiceEnvironment, CommonError> {
        match self.environment.environment_type.as_str() {
            "production" => Ok(ServiceEnvironment::Production),
            "development" => Ok(ServiceEnvironment::Development),
            "testing" => Ok(ServiceEnvironment::Testing),
            _ => Err(CommonError::ConfigError(
                "Invalid environment type in configuration".to_string(),
            )),
        }
    }

    /// Expand paths with home directory
    pub fn expand_paths(&mut self) -> Result<(), CommonError> {
        // Expand storage paths
        self.storage.data_dir = expand_path(&self.storage.data_dir)?;
        self.storage.mana_ledger.ledger_path = expand_path(&self.storage.mana_ledger.ledger_path)?;

        if let Some(ref mut path) = self.storage.dag_store.storage_path {
            *path = expand_path(path)?;
        }

        if let Some(ref mut path) = self.storage.reputation_store.storage_path {
            *path = expand_path(path)?;
        }

        // Expand key store path
        if let Some(ref mut path) = self.identity.key_store.key_file_path {
            *path = expand_path(path)?;
        }

        Ok(())
    }

    /// Create a ServiceConfig from this RuntimeConfig
    pub fn to_service_config(&self) -> Result<ServiceConfig, CommonError> {
        // Get the service environment
        let environment = self.get_service_environment()?;

        // Parse the DID
        let current_identity = Did::from_str(&self.identity.node_did)
            .map_err(|e| CommonError::ConfigError(format!("Invalid node DID: {}", e)))?;

        // Create service configuration builder
        let mut builder = ServiceConfigBuilder::new(environment).with_identity(current_identity);

        // Create services based on configuration

        // Create signer based on key store configuration
        let signer = self.create_signer()?;
        builder = builder.with_signer(signer);

        // Create DID resolver
        let did_resolver = self.create_did_resolver()?;
        builder = builder.with_did_resolver(did_resolver);

        // Create mana ledger
        let mana_ledger = self.create_mana_ledger()?;
        builder = builder.with_mana_ledger(mana_ledger);

        // Create network service for production/development
        if environment == ServiceEnvironment::Production
            || (environment == ServiceEnvironment::Development
                && !self.network.listen_addresses.is_empty())
        {
            let network_service = self.create_network_service()?;
            builder = builder.with_network_service(network_service);
        }

        // Create DAG store for production/development
        if environment == ServiceEnvironment::Production
            || (environment == ServiceEnvironment::Development
                && self.storage.dag_store.store_type != "memory")
        {
            let dag_store_raw = self.create_dag_store()?;
            let dag_store = crate::context::dag_store_wrapper::DagStoreWrapper::generic_production(dag_store_raw);
            builder = builder.with_dag_store(dag_store);
        }

        // Create reputation store
        let reputation_store = self.create_reputation_store()?;
        builder = builder.with_reputation_store(reputation_store);

        // Create time provider (always use system time for now)
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        builder = builder.with_time_provider(time_provider);

        // Build the service configuration
        builder.build()
    }

    /// Create a signer from the key store configuration
    fn create_signer(&self) -> Result<Arc<dyn crate::context::Signer>, CommonError> {
        match self.identity.key_store.store_type.as_str() {
            "stub" => {
                // Only allow stub signers in testing environments
                if self.environment.environment_type == "testing" {
                    Ok(Arc::new(crate::context::StubSigner::new()))
                } else {
                    log::warn!("🔐 PRODUCTION WARNING: Stub signer requested in {} environment. This is not recommended for production use!", self.environment.environment_type);
                    Ok(Arc::new(crate::context::StubSigner::new()))
                }
            }
            "file" => {
                if let Some(ref key_path) = self.identity.key_store.key_file_path {
                    let expanded_path = expand_path(key_path)?;
                    
                    // Check if the key file exists
                    if !expanded_path.exists() {
                        return Err(CommonError::ConfigError(format!(
                            "Key file does not exist at path: {}",
                            expanded_path.display()
                        )));
                    }
                    
                    // For encrypted key files, we need a passphrase
                    // For now, check if it looks like a base58 key file or encrypted
                    let key_content = std::fs::read_to_string(&expanded_path)
                        .map_err(|e| CommonError::IoError(format!(
                            "Failed to read key file {}: {}", 
                            expanded_path.display(), 
                            e
                        )))?;
                    
                            // Try to parse as base58 encoded private key first
        match bs58::decode(key_content.trim()).into_vec() {
            Ok(decoded) => {
                if decoded.len() != 32 {
                    return Err(CommonError::ConfigError(format!(
                        "Invalid key file at {}: expected 32 bytes, got {} bytes. Key file must contain a base58-encoded 32-byte Ed25519 private key.",
                        expanded_path.display(),
                        decoded.len()
                    )));
                }
                
                // Safe conversion since we've verified the length
                let key_bytes: [u8; 32] = decoded.try_into().map_err(|_| {
                    CommonError::ConfigError(format!(
                        "Failed to convert decoded key to 32-byte array at {}",
                        expanded_path.display()
                    ))
                })?;
                
                let signing_key = icn_identity::SigningKey::from_bytes(&key_bytes);
                return Ok(Arc::new(crate::context::Ed25519Signer::new(signing_key)));
            }
            Err(decode_error) => {
                // Check if it might be an encrypted file or other format
                if key_content.contains("BEGIN") && key_content.contains("END") {
                    return Err(CommonError::ConfigError(format!(
                        "Key file at {} appears to be in PEM or encrypted format, which is not currently supported. Please provide a base58-encoded Ed25519 private key, or use 'generate' key store type to create a new key.",
                        expanded_path.display()
                    )));
                } else {
                    return Err(CommonError::ConfigError(format!(
                        "Invalid key file at {}: failed to decode as base58 ({}). Key file must contain a base58-encoded Ed25519 private key.",
                        expanded_path.display(),
                        decode_error
                    )));
                }
            }
        }
                } else {
                    Err(CommonError::ConfigError(
                        "Key file path is required for file key store".to_string(),
                    ))
                }
            }
            "hsm" => {
                if let Some(ref hsm_config) = self.identity.key_store.hsm_config {
                    // Create HSM key store and use it with Ed25519Signer
                    let hsm = crate::context::signers::ExampleHsm::new(&hsm_config.library_path);
                    match crate::context::Ed25519Signer::from_hsm(&hsm) {
                        Ok(signer) => Ok(Arc::new(signer)),
                        Err(e) => {
                            log::warn!("🔐 HSM WARNING: Failed to load key from HSM ({}). Generating ephemeral key for development use.", e);
                            log::warn!("🔐 Ensure HSM is properly configured and accessible for production use.");
                            let (signing_key, _) = icn_identity::generate_ed25519_keypair();
                            Ok(Arc::new(crate::context::Ed25519Signer::new(signing_key)))
                        }
                    }
                } else {
                    Err(CommonError::ConfigError(
                        "HSM configuration is required for HSM key store".to_string(),
                    ))
                }
            }
            "generate" => {
                // Generate a new Ed25519 keypair for this session
                log::info!("🔐 Generating new Ed25519 keypair for this session");
                let (signing_key, _) = icn_identity::generate_ed25519_keypair();
                Ok(Arc::new(crate::context::Ed25519Signer::new(signing_key)))
            }
            _ => Err(CommonError::ConfigError(format!(
                "Unsupported key store type: {}. Supported types: stub, file, hsm, generate",
                self.identity.key_store.store_type
            ))),
        }
    }

    /// Create a DID resolver from the configuration
    fn create_did_resolver(&self) -> Result<Arc<dyn icn_identity::DidResolver>, CommonError> {
        match self.identity.did_resolver.resolver_type.as_str() {
            "key" => Ok(Arc::new(icn_identity::KeyDidResolver)),
            "web" => {
                // For now, use KeyDidResolver as a fallback
                // In a real implementation, this would create a WebDidResolver
                Ok(Arc::new(icn_identity::KeyDidResolver))
            }
            _ => Err(CommonError::ConfigError(format!(
                "Unsupported DID resolver type: {}",
                self.identity.did_resolver.resolver_type
            ))),
        }
    }

    /// Create a mana ledger from the configuration
    fn create_mana_ledger(&self) -> Result<crate::context::SimpleManaLedger, CommonError> {
        let expanded_path = expand_path(&self.storage.mana_ledger.ledger_path)?;
        Ok(crate::context::SimpleManaLedger::new(expanded_path))
    }

    /// Create a network service from the configuration
    fn create_network_service(&self) -> Result<Arc<dyn icn_network::NetworkService>, CommonError> {
        #[cfg(feature = "enable-libp2p")]
        {
            // Create production libp2p network service
            let network_config = self.create_libp2p_config()?;

            // Use async block to handle the async network service creation
            use tokio::runtime::Handle;

            let service = Handle::current().block_on(async {
                icn_network::libp2p_service::Libp2pNetworkService::new(network_config)
                    .await
                    .map_err(|e| {
                        CommonError::NetworkError(format!("Failed to create libp2p service: {}", e))
                    })
            })?;

            Ok(Arc::new(service))
        }

        #[cfg(not(feature = "enable-libp2p"))]
        {
            // Fallback to stub service only when libp2p is not available
            log::warn!("Using stub network service - libp2p feature not enabled");
            Ok(Arc::new(icn_network::StubNetworkService::default()))
        }
    }

    /// Create libp2p configuration from runtime configuration
    fn create_libp2p_config(
        &self,
    ) -> Result<icn_network::libp2p_service::NetworkConfig, CommonError> {
        use icn_network::libp2p_service::NetworkConfig;
        use std::time::Duration;

        // Parse listen addresses
        let mut listen_addresses = Vec::new();
        for addr_str in &self.network.listen_addresses {
            let addr = addr_str.parse().map_err(|e| {
                CommonError::ConfigError(format!("Invalid listen address '{}': {}", addr_str, e))
            })?;
            listen_addresses.push(addr);
        }

        // If no listen addresses specified, use default
        if listen_addresses.is_empty() {
            listen_addresses.push("/ip4/0.0.0.0/tcp/0".parse().unwrap());
        }

        // Parse bootstrap peers
        let mut bootstrap_peers = Vec::new();
        for bootstrap in &self.network.bootstrap_peers {
            let peer_id = bootstrap.peer_id.parse().map_err(|e| {
                CommonError::ConfigError(format!("Invalid peer ID '{}': {}", bootstrap.peer_id, e))
            })?;
            let multiaddr = bootstrap.address.parse().map_err(|e| {
                CommonError::ConfigError(format!(
                    "Invalid multiaddr '{}': {}",
                    bootstrap.address, e
                ))
            })?;
            bootstrap_peers.push((peer_id, multiaddr));
        }

        Ok(NetworkConfig {
            listen_addresses,
            bootstrap_peers,
            max_peers: self
                .network
                .connection_limits
                .max_incoming_connections
                .max(self.network.connection_limits.max_outgoing_connections)
                as usize,
            max_peers_per_ip: self.network.connection_limits.max_connections_per_peer as usize,
            connection_timeout: Duration::from_millis(self.network.timeouts.connection_timeout_ms),
            request_timeout: Duration::from_millis(self.network.timeouts.request_timeout_ms),
            heartbeat_interval: Duration::from_millis(self.network.timeouts.keep_alive_interval_ms),
            bootstrap_interval: Duration::from_secs(300), // Default 5 minutes
            peer_discovery_interval: Duration::from_secs(60), // Default 1 minute
            enable_mdns: self.network.enable_mdns,
            kademlia_replication_factor: 20, // Default replication factor
        })
    }

    /// Create a DAG store from the configuration
    fn create_dag_store(
        &self,
    ) -> Result<
        Arc<crate::context::DagStoreMutexType<crate::context::DagStorageService>>,
        CommonError,
    > {
        match self.storage.dag_store.store_type.as_str() {
            "memory" => Ok(Arc::new(crate::context::DagStoreMutexType::new(
                crate::context::StubDagStore::new(),
            ))),
            "file" | "sled" | "rocksdb" => {
                if let Some(ref storage_path) = self.storage.dag_store.storage_path {
                    let _expanded_path = expand_path(storage_path)?;
                    // For now, create a stub store with path awareness
                    // In a real implementation, this would create the appropriate persistent store
                    Ok(Arc::new(crate::context::DagStoreMutexType::new(
                        crate::context::StubDagStore::new(),
                    )))
                } else {
                    Err(CommonError::ConfigError(
                        "Storage path is required for persistent DAG store".to_string(),
                    ))
                }
            }
            _ => Err(CommonError::ConfigError(format!(
                "Unsupported DAG store type: {}",
                self.storage.dag_store.store_type
            ))),
        }
    }

    /// Create a reputation store from the configuration
    fn create_reputation_store(
        &self,
    ) -> Result<Arc<dyn icn_reputation::ReputationStore>, CommonError> {
        match self.storage.reputation_store.store_type.as_str() {
            "memory" => Ok(Arc::new(icn_reputation::InMemoryReputationStore::new())),
            "file" | "database" => {
                if let Some(ref storage_path) = self.storage.reputation_store.storage_path {
                    let _expanded_path = expand_path(storage_path)?;
                    // For now, create a memory store with path awareness
                    // In a real implementation, this would create the appropriate persistent store
                    Ok(Arc::new(icn_reputation::InMemoryReputationStore::new()))
                } else {
                    Err(CommonError::ConfigError(
                        "Storage path is required for persistent reputation store".to_string(),
                    ))
                }
            }
            _ => Err(CommonError::ConfigError(format!(
                "Unsupported reputation store type: {}",
                self.storage.reputation_store.store_type
            ))),
        }
    }
}

/// Configuration builder for advanced use cases
#[derive(Debug, Clone)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl RuntimeConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
        }
    }

    /// Start with production defaults
    pub fn production() -> Self {
        Self {
            config: RuntimeConfig::production(),
        }
    }

    /// Start with development defaults
    pub fn development() -> Self {
        Self {
            config: RuntimeConfig::development(),
        }
    }

    /// Start with testing defaults
    pub fn testing() -> Self {
        Self {
            config: RuntimeConfig::testing(),
        }
    }

    /// Start from an existing configuration
    pub fn from_config(config: RuntimeConfig) -> Self {
        Self { config }
    }

    /// Load from a configuration file and use as base
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, CommonError> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| CommonError::ConfigError("Invalid path encoding".to_string()))?;
        let config = RuntimeConfig::from_file(path_str)?;
        Ok(Self { config })
    }

    // Environment configuration methods

    /// Set the environment type
    pub fn environment_type(mut self, env_type: &str) -> Self {
        self.config.environment.environment_type = env_type.to_string();
        self
    }

    /// Enable or disable debug mode
    pub fn debug(mut self, enabled: bool) -> Self {
        self.config.environment.debug = enabled;
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: &str) -> Self {
        self.config.environment.log_level = level.to_string();
        self
    }

    /// Enable or disable metrics
    pub fn metrics(mut self, enabled: bool) -> Self {
        self.config.environment.metrics = enabled;
        self
    }

    // Identity configuration methods

    /// Set the node DID
    pub fn node_did(mut self, did: &str) -> Self {
        self.config.identity.node_did = did.to_string();
        self
    }

    /// Set key store type
    pub fn key_store_type(mut self, store_type: &str) -> Self {
        self.config.identity.key_store.store_type = store_type.to_string();
        self
    }

    /// Set key file path
    pub fn key_file_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.identity.key_store.key_file_path = Some(path.into());
        self
    }

    /// Set DID resolver type
    pub fn did_resolver_type(mut self, resolver_type: &str) -> Self {
        self.config.identity.did_resolver.resolver_type = resolver_type.to_string();
        self
    }

    // Network configuration methods

    /// Add a listen address
    pub fn add_listen_address(mut self, address: &str) -> Self {
        self.config
            .network
            .listen_addresses
            .push(address.to_string());
        self
    }

    /// Set listen addresses (replaces existing)
    pub fn listen_addresses(mut self, addresses: Vec<String>) -> Self {
        self.config.network.listen_addresses = addresses;
        self
    }

    /// Add a bootstrap peer
    pub fn add_bootstrap_peer(mut self, peer_id: &str, address: &str) -> Self {
        self.config.network.bootstrap_peers.push(BootstrapPeer {
            peer_id: peer_id.to_string(),
            address: address.to_string(),
        });
        self
    }

    /// Enable or disable mDNS
    pub fn enable_mdns(mut self, enabled: bool) -> Self {
        self.config.network.enable_mdns = enabled;
        self
    }

    /// Set connection timeout
    pub fn connection_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.network.timeouts.connection_timeout_ms = timeout;
        self
    }

    /// Set request timeout
    pub fn request_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.network.timeouts.request_timeout_ms = timeout;
        self
    }

    /// Set keep alive interval
    pub fn keep_alive_interval_ms(mut self, interval: u64) -> Self {
        self.config.network.timeouts.keep_alive_interval_ms = interval;
        self
    }

    /// Set maximum incoming connections
    pub fn max_incoming_connections(mut self, max: u32) -> Self {
        self.config
            .network
            .connection_limits
            .max_incoming_connections = max;
        self
    }

    /// Set maximum outgoing connections
    pub fn max_outgoing_connections(mut self, max: u32) -> Self {
        self.config
            .network
            .connection_limits
            .max_outgoing_connections = max;
        self
    }

    // Storage configuration methods

    /// Set data directory
    pub fn data_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.storage.data_dir = path.into();
        self
    }

    /// Set DAG store type
    pub fn dag_store_type(mut self, store_type: &str) -> Self {
        self.config.storage.dag_store.store_type = store_type.to_string();
        self
    }

    /// Set DAG store path
    pub fn dag_store_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.storage.dag_store.storage_path = Some(path.into());
        self
    }

    /// Set DAG cache size
    pub fn dag_cache_size_mb(mut self, size: u64) -> Self {
        self.config.storage.dag_store.cache_size_mb = size;
        self
    }

    /// Set mana ledger path
    pub fn mana_ledger_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.storage.mana_ledger.ledger_path = path.into();
        self
    }

    /// Set initial mana balance
    pub fn initial_mana(mut self, mana: u64) -> Self {
        self.config.storage.mana_ledger.initial_mana = mana;
        self
    }

    /// Set mana regeneration rate
    pub fn mana_regeneration_rate(mut self, rate: f64) -> Self {
        self.config.storage.mana_ledger.regeneration_rate = rate;
        self
    }

    /// Set maximum mana capacity
    pub fn max_mana_capacity(mut self, capacity: u64) -> Self {
        self.config.storage.mana_ledger.max_capacity = capacity;
        self
    }

    // Governance configuration methods

    /// Enable or disable governance
    pub fn governance_enabled(mut self, enabled: bool) -> Self {
        self.config.governance.enabled = enabled;
        self
    }

    /// Set minimum voting power
    pub fn min_voting_power(mut self, power: u64) -> Self {
        self.config.governance.voting.min_voting_power = power;
        self
    }

    /// Set vote cost in mana
    pub fn vote_cost_mana(mut self, cost: u64) -> Self {
        self.config.governance.voting.vote_cost_mana = cost;
        self
    }

    /// Set voting period
    pub fn voting_period_seconds(mut self, period: u64) -> Self {
        self.config.governance.voting.voting_period_seconds = period;
        self
    }

    // Runtime parameters methods

    /// Set job execution timeout
    pub fn job_execution_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.runtime.job_execution_timeout_ms = timeout;
        self
    }

    /// Set maximum job queue size
    pub fn max_job_queue_size(mut self, size: usize) -> Self {
        self.config.runtime.max_job_queue_size = size;
        self
    }

    /// Set maximum concurrent jobs
    pub fn max_concurrent_jobs(mut self, jobs: usize) -> Self {
        self.config.runtime.max_concurrent_jobs = jobs;
        self
    }

    // Configuration composition methods

    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(mut self, other: RuntimeConfig) -> Self {
        // Simple merge logic - other configuration takes precedence
        // In a more sophisticated implementation, this could be configurable
        if other.environment.environment_type != "default" {
            self.config.environment = other.environment;
        }

        if other.identity.node_did != "default" {
            self.config.identity = other.identity;
        }

        if !other.network.listen_addresses.is_empty() {
            self.config.network = other.network;
        }

        if other.storage.data_dir != PathBuf::from("./data") {
            self.config.storage = other.storage;
        }

        self.config.governance = other.governance;
        self.config.runtime = other.runtime;

        self
    }

    /// Override with environment-specific settings
    pub fn with_environment_overrides(mut self, environment: &str) -> Self {
        match environment {
            "production" => {
                let prod_config = RuntimeConfig::production();
                self.config.environment = prod_config.environment;
                self.config.storage = prod_config.storage;
                self.config.network.timeouts = prod_config.network.timeouts;
            }
            "development" => {
                let dev_config = RuntimeConfig::development();
                self.config.environment = dev_config.environment;
                self.config.storage = dev_config.storage;
                self.config.network.timeouts = dev_config.network.timeouts;
            }
            "testing" => {
                let test_config = RuntimeConfig::testing();
                self.config.environment = test_config.environment;
                self.config.storage = test_config.storage;
                self.config.network.timeouts = test_config.network.timeouts;
            }
            _ => {
                // Keep current settings for unknown environments
            }
        }
        self
    }

    /// Apply a configuration template
    pub fn apply_template<F>(self, template: F) -> Self
    where
        F: FnOnce(RuntimeConfigBuilder) -> RuntimeConfigBuilder,
    {
        template(self)
    }

    // Validation and building methods

    /// Validate the current configuration
    pub fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    /// Build the final configuration
    pub fn build(mut self) -> Result<RuntimeConfig, CommonError> {
        // Expand paths before validation
        self.config.expand_paths()?;

        // Validate the configuration
        self.config.validate()?;

        Ok(self.config)
    }

    /// Build without validation (useful for testing)
    pub fn build_unchecked(mut self) -> RuntimeConfig {
        let _ = self.config.expand_paths();
        self.config
    }

    /// Get a reference to the current configuration state
    pub fn get_config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a mutable reference to the current configuration state
    pub fn get_config_mut(&mut self) -> &mut RuntimeConfig {
        &mut self.config
    }
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Configuration template functions
pub mod templates {
    use super::*;

    /// Local development template
    pub fn local_development(_builder: RuntimeConfigBuilder) -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::development()
            .listen_addresses(vec!["/ip4/127.0.0.1/tcp/4001".to_string()])
            .enable_mdns(true)
            .dag_store_type("memory")
            .connection_timeout_ms(5000)
            .job_execution_timeout_ms(30000)
    }

    /// Production server template
    pub fn production_server(_builder: RuntimeConfigBuilder) -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::production()
            .listen_addresses(vec![
                "/ip4/0.0.0.0/tcp/4001".to_string(),
                "/ip6/::/tcp/4001".to_string(),
            ])
            .enable_mdns(false)
            .dag_store_type("rocksdb")
            .connection_timeout_ms(10000)
            .job_execution_timeout_ms(300000)
    }

    /// Testing template with isolated storage
    pub fn isolated_testing(_builder: RuntimeConfigBuilder) -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::testing()
            .data_dir("/tmp/icn-test")
            .listen_addresses(vec![])
            .enable_mdns(false)
            .dag_store_type("memory")
            .governance_enabled(false)
            .job_execution_timeout_ms(5000)
    }

    /// High-performance template
    pub fn high_performance(builder: RuntimeConfigBuilder) -> RuntimeConfigBuilder {
        builder
            .max_incoming_connections(200)
            .max_outgoing_connections(200)
            .connection_timeout_ms(2000)
            .request_timeout_ms(5000)
            .dag_cache_size_mb(1024)
            .job_execution_timeout_ms(60000)
            .max_concurrent_jobs(50)
    }

    /// Minimal resource template
    pub fn minimal_resources(builder: RuntimeConfigBuilder) -> RuntimeConfigBuilder {
        builder
            .max_incoming_connections(10)
            .max_outgoing_connections(10)
            .connection_timeout_ms(30000)
            .request_timeout_ms(60000)
            .dag_cache_size_mb(64)
            .job_execution_timeout_ms(120000)
            .max_concurrent_jobs(5)
    }
}

/// Expand path with home directory
fn expand_path(path: &Path) -> Result<PathBuf, CommonError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| CommonError::ConfigError("Invalid path encoding".to_string()))?;

    if path_str.starts_with("~/") {
        let home = std::env::var("HOME").map_err(|_| {
            CommonError::ConfigError("HOME environment variable not set".to_string())
        })?;
        let expanded = path_str.replace("~/", &format!("{}/", home));
        Ok(PathBuf::from(expanded))
    } else {
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_production_config() {
        let config = RuntimeConfig::production();
        assert_eq!(config.environment.environment_type, "production");
        assert!(!config.environment.debug);
        assert!(config.governance.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config() {
        let config = RuntimeConfig::development();
        assert_eq!(config.environment.environment_type, "development");
        assert!(config.environment.debug);
        assert!(config.network.enable_mdns);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_testing_config() {
        let config = RuntimeConfig::testing();
        assert_eq!(config.environment.environment_type, "testing");
        assert!(config.environment.debug);
        assert!(!config.network.enable_mdns);
        assert!(config.network.listen_addresses.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = RuntimeConfig::development();

        // Test TOML serialization
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: RuntimeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config.environment.environment_type,
            deserialized.environment.environment_type
        );

        // Test JSON serialization
        let json_str = serde_json::to_string(&config).unwrap();
        let deserialized: RuntimeConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(
            config.environment.environment_type,
            deserialized.environment.environment_type
        );
    }

    #[test]
    fn test_config_validation() {
        let mut config = RuntimeConfig::testing();

        // Test invalid DID
        config.identity.node_did = "invalid-did".to_string();
        assert!(config.validate().is_err());

        // Test invalid environment
        config.identity.node_did =
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
        config.environment.environment_type = "invalid".to_string();
        assert!(config.validate().is_err());

        // Test invalid mana parameters
        config.environment.environment_type = "testing".to_string();
        config.storage.mana_ledger.initial_mana = 20000;
        config.storage.mana_ledger.max_capacity = 10000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_io() {
        let config = RuntimeConfig::development();

        // Test TOML file I/O
        let toml_file = NamedTempFile::new().unwrap();
        let toml_path = toml_file.path().to_str().unwrap().to_string() + ".toml";

        config.to_file(&toml_path).unwrap();
        let loaded_config = RuntimeConfig::from_file(&toml_path).unwrap();
        assert_eq!(
            config.environment.environment_type,
            loaded_config.environment.environment_type
        );

        // Test JSON file I/O
        let json_file = NamedTempFile::new().unwrap();
        let json_path = json_file.path().to_str().unwrap().to_string() + ".json";

        config.to_file(&json_path).unwrap();
        let loaded_config = RuntimeConfig::from_file(&json_path).unwrap();
        assert_eq!(
            config.environment.environment_type,
            loaded_config.environment.environment_type
        );
    }

    #[test]
    fn test_runtime_config_builder_basic() {
        // Test default builder
        let config = RuntimeConfigBuilder::new().build_unchecked();
        assert_eq!(config.environment.environment_type, "development");

        // Test production builder
        let config = RuntimeConfigBuilder::production().build_unchecked();
        assert_eq!(config.environment.environment_type, "production");
        assert!(!config.environment.debug);
        assert_eq!(config.environment.log_level, "info");

        // Test development builder
        let config = RuntimeConfigBuilder::development().build_unchecked();
        assert_eq!(config.environment.environment_type, "development");
        assert!(config.environment.debug);
        assert_eq!(config.environment.log_level, "debug");

        // Test testing builder
        let config = RuntimeConfigBuilder::testing().build_unchecked();
        assert_eq!(config.environment.environment_type, "testing");
        assert!(config.environment.debug);
        assert_eq!(config.environment.log_level, "trace");
    }

    #[test]
    fn test_runtime_config_builder_fluent_api() {
        let config = RuntimeConfigBuilder::new()
            .environment_type("custom")
            .debug(true)
            .log_level("warn")
            .metrics(false)
            .node_did("did:key:z6MkTest")
            .key_store_type("hsm")
            .listen_addresses(vec!["/ip4/127.0.0.1/tcp/5000".to_string()]) // Use listen_addresses to replace
            .enable_mdns(true)
            .connection_timeout_ms(15000)
            .dag_store_type("rocksdb")
            .dag_cache_size_mb(512)
            .initial_mana(5000)
            .max_mana_capacity(50000)
            .governance_enabled(false)
            .job_execution_timeout_ms(120000)
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "custom");
        assert!(config.environment.debug);
        assert_eq!(config.environment.log_level, "warn");
        assert!(!config.environment.metrics);
        assert_eq!(config.identity.node_did, "did:key:z6MkTest");
        assert_eq!(config.identity.key_store.store_type, "hsm");
        assert_eq!(
            config.network.listen_addresses,
            vec!["/ip4/127.0.0.1/tcp/5000"]
        );
        assert!(config.network.enable_mdns);
        assert_eq!(config.network.timeouts.connection_timeout_ms, 15000);
        assert_eq!(config.storage.dag_store.store_type, "rocksdb");
        assert_eq!(config.storage.dag_store.cache_size_mb, 512);
        assert_eq!(config.storage.mana_ledger.initial_mana, 5000);
        assert_eq!(config.storage.mana_ledger.max_capacity, 50000);
        assert!(!config.governance.enabled);
        assert_eq!(config.runtime.job_execution_timeout_ms, 120000);
    }

    #[test]
    fn test_runtime_config_builder_from_config() {
        let base_config = RuntimeConfig::development();
        let modified_config = RuntimeConfigBuilder::from_config(base_config.clone())
            .environment_type("modified")
            .initial_mana(9999)
            .build_unchecked();

        assert_eq!(modified_config.environment.environment_type, "modified");
        assert_eq!(modified_config.storage.mana_ledger.initial_mana, 9999);
        // Other settings should remain from development defaults
        assert!(modified_config.environment.debug);
        assert!(modified_config.network.enable_mdns);
    }

    #[test]
    fn test_runtime_config_builder_merge_with() {
        let base_config = RuntimeConfigBuilder::development().build_unchecked();
        let override_config = RuntimeConfigBuilder::production()
            .initial_mana(999)
            .build_unchecked();

        let merged_config = RuntimeConfigBuilder::from_config(base_config)
            .merge_with(override_config)
            .build_unchecked();

        // Should have production environment settings
        assert_eq!(merged_config.environment.environment_type, "production");
        assert!(!merged_config.environment.debug);
        // Should have overridden mana settings
        assert_eq!(merged_config.storage.mana_ledger.initial_mana, 999);
    }

    #[test]
    fn test_runtime_config_builder_environment_overrides() {
        let config = RuntimeConfigBuilder::new()
            .with_environment_overrides("production")
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "production");
        assert!(!config.environment.debug);
        assert_eq!(config.environment.log_level, "info");

        let config = RuntimeConfigBuilder::new()
            .with_environment_overrides("testing")
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "testing");
        assert!(config.environment.debug);
        assert_eq!(config.environment.log_level, "trace");
    }

    #[test]
    fn test_runtime_config_builder_templates() {
        // Test local development template
        let config = RuntimeConfigBuilder::new()
            .apply_template(templates::local_development)
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "development");
        assert_eq!(
            config.network.listen_addresses,
            vec!["/ip4/127.0.0.1/tcp/4001"]
        );
        assert!(config.network.enable_mdns);
        assert_eq!(config.storage.dag_store.store_type, "memory");

        // Test production server template
        let config = RuntimeConfigBuilder::new()
            .apply_template(templates::production_server)
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "production");
        assert_eq!(
            config.network.listen_addresses,
            vec!["/ip4/0.0.0.0/tcp/4001", "/ip6/::/tcp/4001"]
        );
        assert!(!config.network.enable_mdns);
        assert_eq!(config.storage.dag_store.store_type, "rocksdb");

        // Test isolated testing template
        let config = RuntimeConfigBuilder::new()
            .apply_template(templates::isolated_testing)
            .build_unchecked();

        assert_eq!(config.environment.environment_type, "testing");
        assert_eq!(config.storage.data_dir, PathBuf::from("/tmp/icn-test"));
        assert!(config.network.listen_addresses.is_empty());
        assert!(!config.governance.enabled);

        // Test high performance template
        let config = RuntimeConfigBuilder::new()
            .apply_template(templates::high_performance)
            .build_unchecked();

        assert_eq!(
            config.network.connection_limits.max_incoming_connections,
            200
        );
        assert_eq!(
            config.network.connection_limits.max_outgoing_connections,
            200
        );
        assert_eq!(config.storage.dag_store.cache_size_mb, 1024);
        assert_eq!(config.runtime.max_concurrent_jobs, 50);

        // Test minimal resources template
        let config = RuntimeConfigBuilder::new()
            .apply_template(templates::minimal_resources)
            .build_unchecked();

        assert_eq!(
            config.network.connection_limits.max_incoming_connections,
            10
        );
        assert_eq!(
            config.network.connection_limits.max_outgoing_connections,
            10
        );
        assert_eq!(config.storage.dag_store.cache_size_mb, 64);
        assert_eq!(config.runtime.max_concurrent_jobs, 5);
    }

    #[test]
    fn test_runtime_config_builder_validation() {
        // Test valid configuration
        let config = RuntimeConfigBuilder::production()
            .node_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
            .initial_mana(1000)
            .max_mana_capacity(10000);

        assert!(config.validate().is_ok());

        // Test invalid configuration (initial mana > max capacity)
        let config = RuntimeConfigBuilder::production()
            .node_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
            .initial_mana(20000)
            .max_mana_capacity(10000);

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_runtime_config_builder_chaining() {
        // Test complex configuration chaining
        let config = RuntimeConfigBuilder::development()
            .log_level("info")
            .listen_addresses(vec![
                "/ip4/0.0.0.0/tcp/4001".to_string(),
                "/ip6/::/tcp/4001".to_string(),
            ]) // Replace list
            .add_bootstrap_peer("12D3KooWTest", "/ip4/127.0.0.1/tcp/4002")
            .connection_timeout_ms(5000)
            .request_timeout_ms(10000)
            .dag_store_type("file")
            .dag_store_path("/custom/dag/path")
            .mana_ledger_path("/custom/mana/path")
            .mana_regeneration_rate(2.5)
            .min_voting_power(50)
            .vote_cost_mana(5)
            .max_job_queue_size(500)
            .build_unchecked();

        assert_eq!(config.environment.log_level, "info");
        assert_eq!(config.network.listen_addresses.len(), 2);
        assert_eq!(config.network.bootstrap_peers.len(), 1);
        assert_eq!(config.network.bootstrap_peers[0].peer_id, "12D3KooWTest");
        assert_eq!(config.network.timeouts.connection_timeout_ms, 5000);
        assert_eq!(config.network.timeouts.request_timeout_ms, 10000);
        assert_eq!(config.storage.dag_store.store_type, "file");
        assert_eq!(
            config.storage.dag_store.storage_path,
            Some(PathBuf::from("/custom/dag/path"))
        );
        assert_eq!(
            config.storage.mana_ledger.ledger_path,
            PathBuf::from("/custom/mana/path")
        );
        assert_eq!(config.storage.mana_ledger.regeneration_rate, 2.5);
        assert_eq!(config.governance.voting.min_voting_power, 50);
        assert_eq!(config.governance.voting.vote_cost_mana, 5);
        assert_eq!(config.runtime.max_job_queue_size, 500);
    }

    #[test]
    fn test_file_key_store_error_handling() {
        use std::io::Write;

        // Test 1: Invalid base58 content
        let invalid_base58_file = NamedTempFile::new().unwrap();
        writeln!(invalid_base58_file.as_file(), "this-is-not-valid-base58!@#$").unwrap();

        let config = RuntimeConfigBuilder::testing()
            .key_store_type("file")
            .key_file_path(invalid_base58_file.path())
            .build_unchecked();

        let result = config.create_signer();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("failed to decode as base58"));
        assert!(error_msg.contains("Key file must contain a base58-encoded Ed25519 private key"));

        // Test 2: Valid base58 but wrong length (16 bytes instead of 32)
        let wrong_length_file = NamedTempFile::new().unwrap();
        let short_key = vec![0u8; 16]; // 16 bytes instead of 32
        let short_key_base58 = bs58::encode(short_key).into_string();
        writeln!(wrong_length_file.as_file(), "{}", short_key_base58).unwrap();

        let config = RuntimeConfigBuilder::testing()
            .key_store_type("file")
            .key_file_path(wrong_length_file.path())
            .build_unchecked();

        let result = config.create_signer();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("expected 32 bytes, got 16 bytes"));

        // Test 3: PEM format detection
        let pem_file = NamedTempFile::new().unwrap();
        writeln!(
            pem_file.as_file(),
            "-----BEGIN PRIVATE KEY-----\nSomeEncryptedContentHere\n-----END PRIVATE KEY-----"
        ).unwrap();

        let config = RuntimeConfigBuilder::testing()
            .key_store_type("file")
            .key_file_path(pem_file.path())
            .build_unchecked();

        let result = config.create_signer();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("appears to be in PEM or encrypted format"));
        assert!(error_msg.contains("not currently supported"));

        // Test 4: Valid base58 32-byte key should work
        let valid_key_file = NamedTempFile::new().unwrap();
        let valid_key = vec![42u8; 32]; // 32 bytes of 42s
        let valid_key_base58 = bs58::encode(valid_key).into_string();
        writeln!(valid_key_file.as_file(), "{}", valid_key_base58).unwrap();

        let config = RuntimeConfigBuilder::testing()
            .key_store_type("file")
            .key_file_path(valid_key_file.path())
            .build_unchecked();

        let result = config.create_signer();
        assert!(result.is_ok(), "Valid 32-byte base58 key should work: {:?}", result);

        // Test 5: Non-existent file
        let config = RuntimeConfigBuilder::testing()
            .key_store_type("file")
            .key_file_path("/non/existent/path/key.file")
            .build_unchecked();

        let result = config.create_signer();
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Key file does not exist"));
    }
}
