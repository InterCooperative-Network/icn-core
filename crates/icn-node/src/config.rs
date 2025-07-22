use icn_common::{CommonError, DagBlock};
#[cfg(feature = "persist-rocksdb")]
use icn_dag::rocksdb_store::RocksDagStore;
#[cfg(feature = "persist-sled")]
use icn_dag::sled_store::SledDagStore;
#[cfg(feature = "persist-sqlite")]
use icn_dag::sqlite_store::SqliteDagStore;
use icn_dag::{CompatAsyncStore, TokioFileDagStore};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex as TokioMutex;

/// Storage backends supported by the node.
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StorageBackendType {
    /// In-memory store, volatile.
    Memory,
    /// File-based persistence.
    File,
    /// SQLite database backend (requires `persist-sqlite` feature).
    Sqlite,
    /// Sled database backend (requires `persist-sled` feature).
    Sled,
    /// RocksDB database backend (requires `persist-rocksdb` feature).
    Rocksdb,
    /// Postgres database backend (requires `persist-postgres` feature).
    #[cfg(feature = "persist-postgres")]
    Postgres,
}

impl std::str::FromStr for StorageBackendType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "file" => Ok(Self::File),
            "sqlite" => Ok(Self::Sqlite),
            "sled" => Ok(Self::Sled),
            "rocksdb" => Ok(Self::Rocksdb),
            #[cfg(feature = "persist-postgres")]
            "postgres" => Ok(Self::Postgres),
            _ => Err(format!("invalid storage backend: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    pub storage_backend: StorageBackendType,
    pub storage_path: PathBuf,
    pub mana_ledger_backend: icn_runtime::context::LedgerBackend,
    pub mana_ledger_path: PathBuf,
    pub reputation_db_path: PathBuf,
    pub governance_db_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IdentityConfig {
    pub node_did: Option<String>,
    pub node_private_key_bs58: Option<String>,
    pub node_did_path: PathBuf,
    pub node_private_key_path: PathBuf,
    pub key_path: Option<PathBuf>,
    pub key_passphrase_env: Option<String>,
    pub hsm_library: Option<PathBuf>,
    pub hsm_key_id: Option<String>,
    /// Additional trusted credential issuer DIDs
    pub trusted_credential_issuers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpConfig {
    pub http_listen_addr: String,
    pub api_key: Option<String>,
    pub auth_token: Option<String>,
    pub auth_token_path: Option<PathBuf>,
    pub open_rate_limit: u64,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
    pub tls_min_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct P2pConfig {
    /// Primary listen address for P2P networking
    pub listen_address: String,
    /// Additional listen addresses
    pub additional_listen_addresses: Vec<String>,
    /// Bootstrap peers to connect to on startup
    pub bootstrap_peers: Option<Vec<String>>,
    /// Enable P2P networking
    pub enable_p2p: bool,
    /// Enable mDNS peer discovery
    pub enable_mdns: bool,
    /// Maximum number of peers to maintain connections with
    pub max_peers: usize,
    /// Maximum peers allowed per IP address
    pub max_peers_per_ip: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Bootstrap retry interval in seconds
    pub bootstrap_interval_secs: u64,
    /// Peer discovery interval in seconds
    pub peer_discovery_interval_secs: u64,
    /// Kademlia DHT replication factor
    pub kademlia_replication_factor: usize,
    /// Custom protocol ID for network isolation
    pub protocol_id: Option<String>,
}

/// Configuration values for running an ICN node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub node_name: String,
    #[serde(flatten)]
    pub storage: StorageConfig,
    #[serde(flatten)]
    pub identity: IdentityConfig,
    #[serde(flatten)]
    pub http: HttpConfig,
    #[serde(flatten)]
    pub p2p: P2pConfig,
    /// Force stub services for development and testing
    pub test_mode: bool,
    /// Enable demo mode with preloaded test data and memory-only storage
    pub demo: bool,
    /// How many days between automatic key rotations.
    pub key_rotation_days: u64,
    /// Peers this node has joined in a federation.
    pub federation_peers: Vec<String>,
    /// Capabilities this node advertises for mesh execution.
    pub executor_capabilities: Vec<String>,
    /// Federations this node is a member of.
    pub federations: Vec<String>,
}

pub(crate) fn default_ledger_backend() -> icn_runtime::context::LedgerBackend {
    #[cfg(feature = "persist-sled")]
    {
        icn_runtime::context::LedgerBackend::Sled
    }
    #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
    {
        icn_runtime::context::LedgerBackend::Sqlite
    }
    #[cfg(all(
        not(feature = "persist-sled"),
        not(feature = "persist-sqlite"),
        feature = "persist-rocksdb"
    ))]
    {
        icn_runtime::context::LedgerBackend::Rocksdb
    }
    #[cfg(all(
        not(feature = "persist-sled"),
        not(feature = "persist-sqlite"),
        not(feature = "persist-rocksdb")
    ))]
    {
        icn_runtime::context::LedgerBackend::File
    }
}

/// Choose the best available persistent storage backend for production use.
///
/// **🏭 PRODUCTION PRIORITY ORDER:**
/// 1. **RocksDB** - Best performance for production workloads
/// 2. **Sled** - Good balance of performance and simplicity  
/// 3. **SQLite** - Lightweight but reliable option
/// 4. **File** - Fallback file-based storage
/// 5. **Memory** - Only for testing/development (volatile)
pub(crate) fn default_storage_backend() -> StorageBackendType {
    #[cfg(feature = "persist-rocksdb")]
    {
        StorageBackendType::Rocksdb
    }
    #[cfg(all(not(feature = "persist-rocksdb"), feature = "persist-sled"))]
    {
        StorageBackendType::Sled
    }
    #[cfg(all(
        not(feature = "persist-rocksdb"),
        not(feature = "persist-sled"),
        feature = "persist-sqlite"
    ))]
    {
        StorageBackendType::Sqlite
    }
    #[cfg(all(
        not(feature = "persist-rocksdb"),
        not(feature = "persist-sled"),
        not(feature = "persist-sqlite")
    ))]
    {
        StorageBackendType::File
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_backend: default_storage_backend(),
            storage_path: "./icn_data/dag_store".into(),
            mana_ledger_backend: default_ledger_backend(),
            mana_ledger_path: "./icn_data/mana_ledger.json".into(),
            reputation_db_path: "./icn_data/reputation.sled".into(),
            governance_db_path: "./icn_data/governance_db".into(),
        }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            node_did: None,
            node_private_key_bs58: None,
            node_did_path: "./icn_data/node_did.txt".into(),
            node_private_key_path: "./icn_data/node_sk.bs58".into(),
            key_path: None,
            key_passphrase_env: None,
            hsm_library: None,
            hsm_key_id: None,
            trusted_credential_issuers: Vec::new(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            http_listen_addr: "127.0.0.1:7845".to_string(),
            api_key: None,
            auth_token: None,
            auth_token_path: None,
            open_rate_limit: 60,
            tls_cert_path: None,
            tls_key_path: None,
            tls_min_version: Some("1.3".into()),
        }
    }
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            additional_listen_addresses: Vec::new(),
            bootstrap_peers: None,
            enable_p2p: cfg!(feature = "enable-libp2p"),
            enable_mdns: true,
            max_peers: 100,
            max_peers_per_ip: 5,
            connection_timeout_ms: 30000,
            request_timeout_ms: 10000,
            heartbeat_interval_ms: 15000,
            bootstrap_interval_secs: 300,
            peer_discovery_interval_secs: 60,
            kademlia_replication_factor: 20,
            protocol_id: None,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_name: "ICN Node".to_string(),
            storage: StorageConfig::default(),
            identity: IdentityConfig::default(),
            http: HttpConfig::default(),
            p2p: P2pConfig::default(),
            test_mode: false,
            demo: false,
            key_rotation_days: 90,
            federation_peers: Vec::new(),
            executor_capabilities: Vec::new(),
            federations: Vec::new(),
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML or YAML file. The format is inferred from the file extension.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let cfg: NodeConfig = match ext {
            "toml" => toml::from_str(&data)?,
            "yaml" | "yml" => serde_yaml::from_str(&data)?,
            _ => return Err(format!("unsupported config extension: {ext}").into()),
        };

        Ok(cfg)
    }

    /// Override configuration values with `ICN_*` environment variables.
    pub fn apply_env_overrides(&mut self) {
        macro_rules! set_from_env {
            ($target:expr, $var:expr, $parse:expr) => {
                if let Ok(val) = std::env::var($var) {
                    if let Ok(parsed) = $parse(&val) {
                        $target = parsed;
                    }
                }
            };
        }
        macro_rules! set_opt_from_env {
            ($target:expr, $var:expr) => {
                if let Ok(val) = std::env::var($var) {
                    $target = Some(val.into());
                }
            };
        }

        set_from_env!(
            self.storage.storage_backend,
            "ICN_STORAGE_BACKEND",
            |v: &str| { v.parse::<StorageBackendType>() }
        );
        set_from_env!(self.p2p.enable_p2p, "ICN_ENABLE_P2P", |v: &str| v
            .parse::<bool>());
        set_from_env!(self.p2p.enable_mdns, "ICN_ENABLE_MDNS", |v: &str| v
            .parse::<bool>());
        set_from_env!(self.test_mode, "ICN_TEST_MODE", |v: &str| v.parse::<bool>());
        set_from_env!(
            self.http.open_rate_limit,
            "ICN_OPEN_RATE_LIMIT",
            |v: &str| v.parse::<u64>()
        );
        set_from_env!(
            self.storage.mana_ledger_backend,
            "ICN_MANA_LEDGER_BACKEND",
            |v: &str| v.parse::<icn_runtime::context::LedgerBackend>()
        );

        if let Ok(val) = std::env::var("ICN_STORAGE_PATH") {
            self.storage.storage_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_MANA_LEDGER_PATH") {
            self.storage.mana_ledger_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_REPUTATION_DB_PATH") {
            self.storage.reputation_db_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_GOVERNANCE_DB_PATH") {
            self.storage.governance_db_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_HTTP_LISTEN_ADDR") {
            self.http.http_listen_addr = val;
        }
        if let Ok(val) = std::env::var("ICN_NODE_DID") {
            self.identity.node_did = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_NODE_PRIVATE_KEY_BS58") {
            self.identity.node_private_key_bs58 = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_NODE_DID_PATH") {
            self.identity.node_did_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_NODE_PRIVATE_KEY_PATH") {
            self.identity.node_private_key_path = val.into();
        }
        if let Ok(val) = std::env::var("ICN_KEY_PATH") {
            self.identity.key_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_KEY_PASSPHRASE_ENV") {
            self.identity.key_passphrase_env = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_HSM_LIBRARY") {
            self.identity.hsm_library = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_HSM_KEY_ID") {
            self.identity.hsm_key_id = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_TRUSTED_ISSUERS") {
            self.identity.trusted_credential_issuers =
                val.split(',').map(|s| s.to_string()).collect();
        }
        if let Ok(val) = std::env::var("ICN_NODE_NAME") {
            self.node_name = val;
        }
        // These are handled by p2p.update_from_env() now
        set_opt_from_env!(self.http.api_key, "ICN_HTTP_API_KEY");
        set_opt_from_env!(self.http.auth_token, "ICN_AUTH_TOKEN");
        if let Ok(val) = std::env::var("ICN_AUTH_TOKEN_PATH") {
            self.http.auth_token_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_CERT_PATH") {
            self.http.tls_cert_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_KEY_PATH") {
            self.http.tls_key_path = Some(val.into());
        }
        if let Ok(val) = std::env::var("ICN_TLS_MIN_VERSION") {
            self.http.tls_min_version = Some(val);
        }
        if let Ok(val) = std::env::var("ICN_KEY_ROTATION_DAYS") {
            if let Ok(days) = val.parse::<u64>() {
                self.key_rotation_days = days;
            }
        }
    }

    /// Apply CLI overrides onto this configuration.
    pub fn apply_cli_overrides(&mut self, cli: &super::node::Cli, matches: &clap::ArgMatches) {
        if let Some(v) = &cli.storage_backend {
            self.storage.storage_backend = v.clone();
        }
        if let Some(v) = &cli.storage_path {
            self.storage.storage_path = v.clone();
        }
        if let Some(v) = &cli.mana_ledger_backend {
            self.storage.mana_ledger_backend = *v;
        }
        if let Some(v) = &cli.mana_ledger_path {
            self.storage.mana_ledger_path = v.clone();
        }
        if let Some(v) = &cli.reputation_db_path {
            self.storage.reputation_db_path = v.clone();
        }
        if let Some(v) = &cli.governance_db_path {
            self.storage.governance_db_path = v.clone();
        }
        if let Some(v) = &cli.http_listen_addr {
            self.http.http_listen_addr = v.clone();
        }
        if let Some(v) = &cli.node_did {
            self.identity.node_did = Some(v.clone());
        }
        if let Some(v) = &cli.node_private_key_bs58 {
            self.identity.node_private_key_bs58 = Some(v.clone());
        }
        if let Some(v) = &cli.node_did_path {
            self.identity.node_did_path = v.clone();
        }
        if let Some(v) = &cli.node_private_key_path {
            self.identity.node_private_key_path = v.clone();
        }
        if let Some(v) = &cli.key_path {
            self.identity.key_path = Some(v.clone());
        }
        if let Some(v) = &cli.key_passphrase_env {
            self.identity.key_passphrase_env = Some(v.clone());
        }
        if let Some(v) = &cli.hsm_library {
            self.identity.hsm_library = Some(v.clone());
        }
        if let Some(v) = &cli.hsm_key_id {
            self.identity.hsm_key_id = Some(v.clone());
        }
        if !cli.trusted_issuers.is_empty() {
            self.identity.trusted_credential_issuers = cli.trusted_issuers.clone();
        }
        if let Some(v) = &cli.node_name {
            self.node_name = v.clone();
        }
        if let Some(v) = &cli.listen_address {
            self.p2p.listen_address = v.clone();
        }
        if let Some(v) = &cli.bootstrap_peers {
            self.p2p.bootstrap_peers = Some(v.clone());
        }
        if cli.enable_p2p {
            self.p2p.enable_p2p = true;
        }
        if cli.enable_mdns {
            self.p2p.enable_mdns = true;
        }
        if cli.test_mode {
            self.test_mode = true;
        }
        if cli.demo {
            self.demo = true;
            // Demo mode implies memory-only storage
            self.storage.storage_backend = StorageBackendType::Memory;
            self.storage.mana_ledger_backend = icn_runtime::context::LedgerBackend::File;
        }
        if let Some(v) = &cli.api_key {
            self.http.api_key = Some(v.clone());
        }
        if let Some(v) = cli.open_rate_limit {
            self.http.open_rate_limit = v;
        }
        if let Some(v) = &cli.auth_token {
            self.http.auth_token = Some(v.clone());
        }
        if let Some(v) = &cli.auth_token_path {
            self.http.auth_token_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_cert_path {
            self.http.tls_cert_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_key_path {
            self.http.tls_key_path = Some(v.clone());
        }
        if let Some(v) = &cli.tls_min_version {
            self.http.tls_min_version = Some(v.clone());
        }
        if let Some(v) = cli.key_rotation_days {
            self.key_rotation_days = v;
        }
    }

    /// Ensure directories for all configured paths exist.
    pub fn prepare_paths(&self) -> std::io::Result<()> {
        if let Some(parent) = self.storage.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.storage.mana_ledger_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.storage.reputation_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.storage.governance_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.identity.node_did_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.identity.node_private_key_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(path) = &self.http.auth_token_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    /// Initialize a DAG store based on this configuration.
    pub async fn init_dag_store(
        &self,
    ) -> Result<
        std::sync::Arc<TokioMutex<dyn icn_dag::AsyncStorageService<DagBlock> + Send>>,
        CommonError,
    > {
        let store: std::sync::Arc<TokioMutex<dyn icn_dag::AsyncStorageService<DagBlock> + Send>> =
            match self.storage.storage_backend {
                StorageBackendType::Memory => std::sync::Arc::new(TokioMutex::new(
                    CompatAsyncStore::new(icn_dag::InMemoryDagStore::new()),
                )) as std::sync::Arc<TokioMutex<_>>,
                StorageBackendType::File => std::sync::Arc::new(TokioMutex::new(
                    TokioFileDagStore::new(self.storage.storage_path.clone())?,
                )) as std::sync::Arc<TokioMutex<_>>,
                StorageBackendType::Sqlite => {
                    #[cfg(feature = "persist-sqlite")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            SqliteDagStore::new(self.storage.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-sqlite"))]
                    {
                        return Err(CommonError::ConfigError(
                            "sqlite backend requires 'persist-sqlite' feature".into(),
                        ));
                    }
                }
                StorageBackendType::Sled => {
                    #[cfg(feature = "persist-sled")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            SledDagStore::new(self.storage.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-sled"))]
                    {
                        return Err(CommonError::ConfigError(
                            "sled backend requires 'persist-sled' feature".into(),
                        ));
                    }
                }
                StorageBackendType::Rocksdb => {
                    #[cfg(feature = "persist-rocksdb")]
                    {
                        std::sync::Arc::new(TokioMutex::new(CompatAsyncStore::new(
                            RocksDagStore::new(self.storage.storage_path.clone())?,
                        ))) as std::sync::Arc<TokioMutex<_>>
                    }
                    #[cfg(not(feature = "persist-rocksdb"))]
                    {
                        return Err(CommonError::ConfigError(
                            "rocksdb backend requires 'persist-rocksdb' feature".into(),
                        ));
                    }
                }
                #[cfg(feature = "persist-postgres")]
                StorageBackendType::Postgres => {
                    let store = icn_dag::postgres_store::PostgresDagStore::new(
                        &self.storage.storage_path.to_string_lossy(),
                    )
                    .await?;
                    std::sync::Arc::new(TokioMutex::new(store)) as std::sync::Arc<TokioMutex<_>>
                }
            };
        Ok(store)
    }

    /// Persist this configuration to the given path in TOML format.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, toml_str)
    }
}

#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::NetworkConfig;
#[cfg(feature = "enable-libp2p")]
// Libp2p imports commented out - will be needed for future network configuration
// use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

impl NodeConfig {
    #[cfg(feature = "enable-libp2p")]
    pub fn libp2p_config(&self) -> Result<NetworkConfig, CommonError> {
        // Use production or development configuration based on test_mode
        let mut config = if self.test_mode {
            NetworkConfig::development()
        } else {
            NetworkConfig::production()
        };

        // Set listen address
        config
            .set_listen_addresses(vec![&self.p2p.listen_address])
            .map_err(|e| CommonError::ConfigError(format!("invalid p2p listen address: {e}")))?;

        // Add bootstrap peers
        if let Some(peers) = &self.p2p.bootstrap_peers {
            for peer_str in peers {
                config.add_bootstrap_peer(peer_str).map_err(|e| {
                    CommonError::ConfigError(format!("invalid bootstrap peer '{peer_str}': {e}"))
                })?;
            }
        }

        // Override mDNS setting from config
        config.enable_mdns = self.p2p.enable_mdns;

        // Validate configuration for production
        if !self.test_mode {
            config.validate_production().map_err(|e| {
                CommonError::ConfigError(format!("production validation failed: {e}"))
            })?;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_ready_defaults() {
        let config = StorageConfig::default();

        // Verify that default storage backend is persistent (not Memory)
        assert_ne!(
            config.storage_backend,
            StorageBackendType::Memory,
            "❌ PRODUCTION ERROR: Default storage backend should not be Memory"
        );

        // Verify that all paths are in icn_data directory (not test fixtures)
        assert!(
            config.storage_path.to_string_lossy().contains("icn_data"),
            "Storage path should be in icn_data directory"
        );
        assert!(
            config
                .mana_ledger_path
                .to_string_lossy()
                .contains("icn_data"),
            "Mana ledger path should be in icn_data directory"
        );
        assert!(
            config
                .reputation_db_path
                .to_string_lossy()
                .contains("icn_data"),
            "Reputation DB path should be in icn_data directory"
        );
        assert!(
            config
                .governance_db_path
                .to_string_lossy()
                .contains("icn_data"),
            "Governance DB path should be in icn_data directory"
        );

        // Verify mana ledger backend is configured (all available backends are persistent)
        // Valid backends: File, Sqlite, Sled, Rocksdb - all are persistent
        match config.mana_ledger_backend {
            icn_runtime::context::LedgerBackend::File => {
                println!("  - Using File backend (persistent)");
            }
            #[cfg(feature = "persist-sqlite")]
            icn_runtime::context::LedgerBackend::Sqlite => {
                println!("  - Using Sqlite backend (persistent)");
            }
            #[cfg(feature = "persist-sled")]
            icn_runtime::context::LedgerBackend::Sled => {
                println!("  - Using Sled backend (persistent)");
            }
            #[cfg(feature = "persist-rocksdb")]
            icn_runtime::context::LedgerBackend::Rocksdb => {
                println!("  - Using RocksDB backend (persistent)");
            }
        }

        println!("✅ Production-ready configuration:");
        println!("  - Storage backend: {:?}", config.storage_backend);
        println!("  - Storage path: {:?}", config.storage_path);
        println!("  - Mana ledger backend: {:?}", config.mana_ledger_backend);
        println!("  - Mana ledger path: {:?}", config.mana_ledger_path);
    }

    #[test]
    fn test_storage_backend_priority() {
        let backend = default_storage_backend();

        // The actual backend depends on which features are enabled
        // But it should never be Memory in production
        assert_ne!(
            backend,
            StorageBackendType::Memory,
            "❌ PRODUCTION ERROR: Default storage backend should not be Memory"
        );

        println!("✅ Default storage backend: {:?}", backend);
    }
}
