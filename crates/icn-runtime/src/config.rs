//! Configuration management for ICN runtime.
//!
//! This module provides configuration loading and validation for different
//! environments (Production, Development, Testing) with appropriate defaults.

use crate::context::{ServiceEnvironment, ServiceConfig, ServiceConfigBuilder};
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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
                    store_type: "file".to_string(),
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
        config.network.listen_addresses = vec![
            "/ip4/127.0.0.1/tcp/4001".to_string(),
        ];
        
        config.storage.data_dir = PathBuf::from("./dev-data");
        config.storage.dag_store.store_type = "memory".to_string();
        config.storage.dag_store.storage_path = None;
        config.storage.mana_ledger.ledger_path = PathBuf::from("./dev-data/mana.db");
        config.storage.reputation_store.store_type = "memory".to_string();
        config.storage.reputation_store.storage_path = None;
        
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
        let content = std::fs::read_to_string(path)
            .map_err(|e| CommonError::IoError(format!("Failed to read config file {}: {}", path, e)))?;
        
        // Try TOML first, then JSON
        if path.ends_with(".toml") {
            toml::from_str(&content)
                .map_err(|e| CommonError::ConfigError(format!("Failed to parse TOML config: {}", e)))
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)
                .map_err(|e| CommonError::ConfigError(format!("Failed to parse JSON config: {}", e)))
        } else {
            // Try both formats
            toml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .map_err(|e| CommonError::ConfigError(format!("Failed to parse config (tried both TOML and JSON): {}", e)))
        }
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &str) -> Result<(), CommonError> {
        let content = if path.ends_with(".toml") {
            toml::to_string_pretty(self)
                .map_err(|e| CommonError::ConfigError(format!("Failed to serialize to TOML: {}", e)))?
        } else if path.ends_with(".json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| CommonError::ConfigError(format!("Failed to serialize to JSON: {}", e)))?
        } else {
            // Default to TOML
            toml::to_string_pretty(self)
                .map_err(|e| CommonError::ConfigError(format!("Failed to serialize to TOML: {}", e)))?
        };
        
        std::fs::write(path, content)
            .map_err(|e| CommonError::IoError(format!("Failed to write config file {}: {}", path, e)))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CommonError> {
        // Validate DID format
        Did::from_str(&self.identity.node_did)
            .map_err(|e| CommonError::ConfigError(format!("Invalid node DID: {}", e)))?;
        
        // Validate environment type
        match self.environment.environment_type.as_str() {
            "production" | "development" | "testing" => {}
            _ => return Err(CommonError::ConfigError(
                "Invalid environment type. Must be 'production', 'development', or 'testing'".to_string()
            )),
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
                "Initial mana cannot exceed max capacity".to_string()
            ));
        }
        
        // Validate governance parameters
        if self.governance.proposals.min_quorum < 0.0 || self.governance.proposals.min_quorum > 1.0 {
            return Err(CommonError::ConfigError(
                "Minimum quorum must be between 0.0 and 1.0".to_string()
            ));
        }
        
        if self.governance.proposals.passing_threshold < 0.0 || self.governance.proposals.passing_threshold > 1.0 {
            return Err(CommonError::ConfigError(
                "Passing threshold must be between 0.0 and 1.0".to_string()
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
                "Invalid environment type in configuration".to_string()
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
        let mut builder = ServiceConfigBuilder::new(environment)
            .with_identity(current_identity);
        
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
        if environment == ServiceEnvironment::Production || 
           (environment == ServiceEnvironment::Development && !self.network.listen_addresses.is_empty()) {
            let network_service = self.create_network_service()?;
            builder = builder.with_network_service(network_service);
        }
        
        // Create DAG store for production/development
        if environment == ServiceEnvironment::Production || 
           (environment == ServiceEnvironment::Development && self.storage.dag_store.store_type != "memory") {
            let dag_store = self.create_dag_store()?;
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
            "stub" => Ok(Arc::new(crate::context::StubSigner::new())),
            "file" => {
                if let Some(ref key_path) = self.identity.key_store.key_file_path {
                    let _expanded_path = expand_path(key_path)?;
                    // For now, create a stub signer with file path awareness
                    // In a real implementation, this would load the key from the file
                    Ok(Arc::new(crate::context::StubSigner::new()))
                } else {
                    Err(CommonError::ConfigError("Key file path is required for file key store".to_string()))
                }
            }
            "hsm" => {
                if self.identity.key_store.hsm_config.is_some() {
                    // For now, create a stub signer with HSM awareness
                    // In a real implementation, this would connect to the HSM
                    Ok(Arc::new(crate::context::StubSigner::new()))
                } else {
                    Err(CommonError::ConfigError("HSM configuration is required for HSM key store".to_string()))
                }
            }
            _ => Err(CommonError::ConfigError(format!("Unsupported key store type: {}", self.identity.key_store.store_type))),
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
            _ => Err(CommonError::ConfigError(format!("Unsupported DID resolver type: {}", self.identity.did_resolver.resolver_type))),
        }
    }
    
    /// Create a mana ledger from the configuration
    fn create_mana_ledger(&self) -> Result<crate::context::SimpleManaLedger, CommonError> {
        let expanded_path = expand_path(&self.storage.mana_ledger.ledger_path)?;
        Ok(crate::context::SimpleManaLedger::new(expanded_path))
    }
    
    /// Create a network service from the configuration
    fn create_network_service(&self) -> Result<Arc<dyn icn_network::NetworkService>, CommonError> {
        // For now, return a stub network service
        // In a real implementation, this would create a proper libp2p network service
        // based on the configuration parameters
        Ok(Arc::new(icn_network::StubNetworkService::default()))
    }
    
    /// Create a DAG store from the configuration
    fn create_dag_store(&self) -> Result<Arc<crate::context::DagStoreMutexType<crate::context::DagStorageService>>, CommonError> {
        match self.storage.dag_store.store_type.as_str() {
            "memory" => Ok(Arc::new(crate::context::DagStoreMutexType::new(crate::context::StubDagStore::new()))),
            "file" | "sled" | "rocksdb" => {
                if let Some(ref storage_path) = self.storage.dag_store.storage_path {
                    let _expanded_path = expand_path(storage_path)?;
                    // For now, create a stub store with path awareness
                    // In a real implementation, this would create the appropriate persistent store
                    Ok(Arc::new(crate::context::DagStoreMutexType::new(crate::context::StubDagStore::new())))
                } else {
                    Err(CommonError::ConfigError("Storage path is required for persistent DAG store".to_string()))
                }
            }
            _ => Err(CommonError::ConfigError(format!("Unsupported DAG store type: {}", self.storage.dag_store.store_type))),
        }
    }
    
    /// Create a reputation store from the configuration
    fn create_reputation_store(&self) -> Result<Arc<dyn icn_reputation::ReputationStore>, CommonError> {
        match self.storage.reputation_store.store_type.as_str() {
            "memory" => Ok(Arc::new(icn_reputation::InMemoryReputationStore::new())),
            "file" | "database" => {
                if let Some(ref storage_path) = self.storage.reputation_store.storage_path {
                    let _expanded_path = expand_path(storage_path)?;
                    // For now, create a memory store with path awareness
                    // In a real implementation, this would create the appropriate persistent store
                    Ok(Arc::new(icn_reputation::InMemoryReputationStore::new()))
                } else {
                    Err(CommonError::ConfigError("Storage path is required for persistent reputation store".to_string()))
                }
            }
            _ => Err(CommonError::ConfigError(format!("Unsupported reputation store type: {}", self.storage.reputation_store.store_type))),
        }
    }
}

/// Expand path with home directory
fn expand_path(path: &PathBuf) -> Result<PathBuf, CommonError> {
    let path_str = path.to_str()
        .ok_or_else(|| CommonError::ConfigError("Invalid path encoding".to_string()))?;
    
    if path_str.starts_with("~/") {
        let home = std::env::var("HOME")
            .map_err(|_| CommonError::ConfigError("HOME environment variable not set".to_string()))?;
        let expanded = path_str.replace("~/", &format!("{}/", home));
        Ok(PathBuf::from(expanded))
    } else {
        Ok(path.clone())
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
        assert_eq!(config.environment.environment_type, deserialized.environment.environment_type);
        
        // Test JSON serialization
        let json_str = serde_json::to_string(&config).unwrap();
        let deserialized: RuntimeConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(config.environment.environment_type, deserialized.environment.environment_type);
    }

    #[test]
    fn test_config_validation() {
        let mut config = RuntimeConfig::testing();
        
        // Test invalid DID
        config.identity.node_did = "invalid-did".to_string();
        assert!(config.validate().is_err());
        
        // Test invalid environment
        config.identity.node_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
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
        assert_eq!(config.environment.environment_type, loaded_config.environment.environment_type);
        
        // Test JSON file I/O
        let json_file = NamedTempFile::new().unwrap();
        let json_path = json_file.path().to_str().unwrap().to_string() + ".json";
        
        config.to_file(&json_path).unwrap();
        let loaded_config = RuntimeConfig::from_file(&json_path).unwrap();
        assert_eq!(config.environment.environment_type, loaded_config.environment.environment_type);
    }
} 